// Copyright 2026 Chris Wells
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Live thumbnails for the gallery (rung 16 bbq, Q3, and Defaulted "Thumbnails, in detail"). Each tile is drawn for
//! the Device filter's device by the export renderer, at the tile's size times the screen's pixel ratio, on **one
//! background thread**, then kept for the session by (source, device, file time, size). The interface thread never
//! renders a page: it only hands the thread a job, and turns a finished picture into a texture when one arrives.
//!
//! Jobs are fed in the order the tiles are on screen. Every frame the gallery says which tiles it wants; a job still
//! waiting that is no longer wanted (another tab, another device) is dropped, and the rest are put in screen order.
//! Nothing is asked of the thread while the gallery sits idle, and the thread asks for a repaint only when a picture
//! is finished. A page that fails to draw (or a thread that panics on it) is marked failed and shows a plain tile.
//!
//! The thread has its own copy of the profile registry (the registry is per thread), sent with each job, and the
//! bundled fonts only: a text box in a font this computer has but the file does not carry is drawn from the outlines
//! the file saved for it, as on any computer that lacks the font.

use crate::viewer::shrink;
use egui::{ColorImage, TextureHandle, TextureOptions};
use quadrule_format::QuadFile;
use quadrule_layout::{display_list, lay_out_text_saved};
use quadrule_profiles::Registry;
use quadrule_profiles::active::{self, lookup_in};
use quadrule_render::{Gray, Math, render, turn_clockwise};
use quadrule_text::FontRegistry;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

/// What a thumbnail is of.
#[derive(Clone, Debug)]
pub enum Source {
    /// A `.quad` file, read by the thread.
    File(PathBuf),
    /// A `.quad` file's bytes (a built-in starter).
    Bytes(Arc<Vec<u8>>),
}

/// What makes one thumbnail the same as another: the same page, on the same device, in the same state of the
/// file, at the same size.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    /// Which page: a template's path, or `starter/<folder>/<slug>.quad`.
    pub id: String,
    /// The device it is drawn on.
    pub device: String,
    /// The file's time of last change, in nanoseconds (0 for a starter).
    pub stamp: u128,
    /// The box the picture must fit, in screen pixels.
    pub box_px: (u32, u32),
}

/// One picture to draw.
#[derive(Clone, Debug)]
pub struct Job {
    /// What it is.
    pub key: Key,
    /// Where the page comes from.
    pub source: Source,
}

/// Where a thumbnail is.
pub enum Thumb {
    /// Asked for, not finished.
    Pending,
    /// Drawn, and a texture made of it.
    Ready(TextureHandle),
    /// The page could not be drawn.
    Failed,
}

/// A finished job, on its way back.
struct Done {
    key: Key,
    result: Result<Gray, String>,
    /// Which thread drew it (for the test that says it was not the interface thread).
    thread: std::thread::ThreadId,
}

#[derive(Default)]
struct Queue {
    jobs: VecDeque<(Job, Registry)>,
    running: Option<Key>,
    shutdown: bool,
}

struct Shared {
    queue: Mutex<Queue>,
    wake: Condvar,
}

/// The gallery's thumbnails: the cache, and the one thread that fills it.
pub struct Thumbs {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    done_tx: Sender<Done>,
    done_rx: Receiver<Done>,
    cache: HashMap<Key, Thumb>,
    /// How many jobs were ever handed to the thread.
    sent: u64,
    /// The threads that drew a finished picture, in the order they finished.
    drawn_by: Vec<std::thread::ThreadId>,
}

impl Default for Thumbs {
    fn default() -> Self {
        let (done_tx, done_rx) = channel();
        Thumbs {
            shared: Arc::new(Shared {
                queue: Mutex::new(Queue::default()),
                wake: Condvar::new(),
            }),
            thread: None,
            done_tx,
            done_rx,
            cache: HashMap::new(),
            sent: 0,
            drawn_by: Vec::new(),
        }
    }
}

impl Drop for Thumbs {
    fn drop(&mut self) {
        if let Ok(mut q) = self.shared.queue.lock() {
            q.shutdown = true;
            q.jobs.clear();
        }
        self.shared.wake.notify_all();
        // The thread ends after the picture it is drawing; it is not waited for.
        self.thread.take();
    }
}

impl Thumbs {
    /// Says which thumbnails the gallery wants now, in screen order. Tiles not seen before are queued (the picture is
    /// asked for once); a queued job that is not wanted any more is dropped; the queue follows the new order.
    pub fn want(&mut self, ctx: &egui::Context, wanted: Vec<Job>) {
        let registry = active::with(Clone::clone);
        let mut next: VecDeque<(Job, Registry)> = VecDeque::new();
        {
            let Ok(mut q) = self.shared.queue.lock() else {
                return;
            };
            let mut old: HashMap<Key, (Job, Registry)> = q
                .jobs
                .drain(..)
                .map(|(j, r)| (j.key.clone(), (j, r)))
                .collect();
            for job in wanted {
                if let Some(queued) = old.remove(&job.key) {
                    next.push_back(queued);
                } else if !self.cache.contains_key(&job.key) {
                    self.cache.insert(job.key.clone(), Thumb::Pending);
                    self.sent += 1;
                    next.push_back((job, registry.clone()));
                }
            }
            // What was waiting and is not wanted now goes, so it can be asked for again later; the one being drawn
            // is left to finish.
            for (key, _) in old {
                if q.running.as_ref() != Some(&key) {
                    self.cache.remove(&key);
                }
            }
            q.jobs = next;
            if q.jobs.is_empty() {
                return;
            }
        }
        if self.thread.is_none() {
            self.start(ctx);
        }
        self.shared.wake.notify_one();
    }

    fn start(&mut self, ctx: &egui::Context) {
        let shared = self.shared.clone();
        let done = self.done_tx.clone();
        let ctx = ctx.clone();
        self.thread = std::thread::Builder::new()
            .name("quadrule-thumbnails".into())
            .spawn(move || worker(shared, done, ctx))
            .ok();
    }

    /// Takes the pictures the thread has finished and makes textures of them. True when any arrived (the frame that
    /// shows them is already being drawn, so no repaint is asked for here).
    pub fn poll(&mut self, ctx: &egui::Context) -> bool {
        let mut any = false;
        while let Ok(d) = self.done_rx.try_recv() {
            any = true;
            self.drawn_by.push(d.thread);
            let thumb = match d.result {
                Ok(img) => {
                    let image =
                        ColorImage::from_gray([img.width as usize, img.height as usize], &img.data);
                    Thumb::Ready(ctx.load_texture(
                        format!("thumb-{}-{}", d.key.id, d.key.device),
                        image,
                        TextureOptions::LINEAR,
                    ))
                }
                Err(_) => Thumb::Failed,
            };
            // A picture that arrives for a tile not wanted any more is still kept: it may be wanted again.
            self.cache.insert(d.key, thumb);
        }
        any
    }

    /// Where the thumbnail `key` is, if it was ever asked for.
    pub fn get(&self, key: &Key) -> Option<&Thumb> {
        self.cache.get(key)
    }

    /// How many thumbnails are asked for and not finished.
    pub fn pending(&self) -> usize {
        self.cache
            .values()
            .filter(|t| matches!(t, Thumb::Pending))
            .count()
    }

    /// How many thumbnails are drawn and ready.
    pub fn ready(&self) -> usize {
        self.cache
            .values()
            .filter(|t| matches!(t, Thumb::Ready(_)))
            .count()
    }

    /// How many thumbnails failed to draw.
    pub fn failed(&self) -> usize {
        self.cache
            .values()
            .filter(|t| matches!(t, Thumb::Failed))
            .count()
    }

    /// How many jobs were ever handed to the thread.
    pub fn jobs_sent(&self) -> u64 {
        self.sent
    }

    /// The threads that drew the finished pictures.
    pub fn drawn_by(&self) -> &[std::thread::ThreadId] {
        &self.drawn_by
    }
}

/// The thread: waits for a job, draws it, sends the picture back, asks for a repaint.
fn worker(shared: Arc<Shared>, done: Sender<Done>, ctx: egui::Context) {
    let mut fonts = FontRegistry::bundled_only();
    loop {
        let (job, registry) = {
            let Ok(mut q) = shared.queue.lock() else {
                return;
            };
            loop {
                if q.shutdown {
                    return;
                }
                if let Some(next) = q.jobs.pop_front() {
                    q.running = Some(next.0.key.clone());
                    break next;
                }
                q = match shared.wake.wait(q) {
                    Ok(q) => q,
                    Err(_) => return,
                };
            }
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            draw(&job, registry, &mut fonts)
        }))
        .unwrap_or_else(|_| Err("the page could not be drawn".into()));
        if let Ok(mut q) = shared.queue.lock() {
            q.running = None;
        }
        let sent = done.send(Done {
            key: job.key,
            result,
            thread: std::thread::current().id(),
        });
        if sent.is_err() {
            return;
        }
        ctx.request_repaint();
    }
}

/// Draws one page on its device with the export renderer and shrinks it to fit the box.
fn draw(job: &Job, registry: Registry, fonts: &mut FontRegistry) -> Result<Gray, String> {
    let bytes = match &job.source {
        Source::File(p) => std::fs::read(p).map_err(|e| e.to_string())?,
        Source::Bytes(b) => b.as_ref().clone(),
    };
    let file = QuadFile::from_bytes(&bytes).map_err(|e| e.to_string())?;
    // This thread's own registry: the app's, as it was when the job was handed over, and the profiles the file
    // carries.
    active::set(registry);
    active::with_mut(|r| {
        r.set_carried(
            file.profiles.values().cloned().collect(),
            Default::default(),
        )
    });
    let carried: Vec<(String, Arc<Vec<u8>>)> = file
        .manifest
        .fonts
        .iter()
        .flat_map(|f| {
            f.files.iter().filter_map(|ff| {
                file.font_files
                    .get(&ff.path)
                    .map(|b| (f.family.clone(), Arc::new(b.clone())))
            })
        })
        .collect();
    fonts.set_embedded(&carried);
    let doc = &file.doc;
    let id = if doc.profile_ids().contains(&job.key.device.as_str()) {
        job.key.device.as_str()
    } else {
        doc.primary_profile_id.as_str()
    };
    let primary = lookup_in(doc, &doc.primary_profile_id).map_err(|e| e.to_string())?;
    let target = lookup_in(doc, id).map_err(|e| e.to_string())?;
    let (laid, _) = lay_out_text_saved(doc, &primary, &target, fonts, &file.outlines)
        .map_err(|e| e.to_string())?;
    let dl = display_list(doc, &primary, &target, &laid.outlines).map_err(|e| e.to_string())?;
    let image = render(&dl, Math::Fastest);
    let image = if target.turned {
        turn_clockwise(&image)
    } else {
        image
    };
    let (bw, bh) = job.key.box_px;
    let scale = (f64::from(bw.max(1)) / f64::from(image.width))
        .min(f64::from(bh.max(1)) / f64::from(image.height))
        .min(1.0);
    let w = ((f64::from(image.width) * scale).round() as u32).max(1);
    let h = ((f64::from(image.height) * scale).round() as u32).max(1);
    Ok(shrink(&image, w, h))
}
