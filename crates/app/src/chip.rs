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

//! The graphics-chip canvas renderer (rung 3 bbq Q4): vello 0.11 on egui's own wgpu device (the
//! same wgpu 30.0.1, spike R14), drawing into a texture egui shows. It never makes an export
//! (ADR-003 item 3). Ideas from `spikes/render/examples/canvas.rs`.

use eframe::egui;
use eframe::egui_wgpu;
use eframe::wgpu;
use quadrule_layout::{DisplayList, Item};
use std::time::Instant;
use vello::kurbo::{Affine, Cap, Join, Line, Rect, Stroke};
use vello::peniko::{Color, Fill, Mix};

/// A texture the page is drawn into, kept while its size stays the same.
struct Target {
    size: (u32, u32),
    texture: wgpu::Texture,
    id: egui::TextureId,
}

/// What the last frame held, to find what has not changed since.
struct Last {
    size: (u32, u32),
    background: u8,
    items: Vec<Item>,
}

/// The unchanged lower part of the page, drawn once into a texture of its own that vello then
/// draws as an image under what has changed.
struct Base {
    /// How many items from the bottom of the stack it holds.
    len: usize,
    size: (u32, u32),
    texture: wgpu::Texture,
    image: vello::peniko::ImageData,
}

/// Fewest items worth keeping apart.
const SPLIT_MIN_ITEMS: usize = 6;
/// Frames timed of each way before choosing, and frames the choice then holds.
const PROBE_FRAMES: usize = 3;
const HOLD_FRAMES: u32 = 100;

/// Which way of drawing a page that can be kept apart is faster on this chip, found by timing
/// both: drawing the unchanged lower part as an image costs something of its own, so it pays only
/// when the lower part is heavy enough, and that is not worked out from the page but measured.
#[derive(Default)]
struct Pace {
    /// Times (ms) of frames that kept a base, and of whole frames when a base could have been kept.
    split: Vec<f64>,
    whole: Vec<f64>,
    /// The way chosen, and how many frames it still holds before both are timed again.
    use_split: bool,
    hold: u32,
}

impl Pace {
    /// Whether the next frame that could keep a base should.
    fn split_next(&mut self) -> bool {
        if self.hold > 0 {
            self.hold -= 1;
            return self.use_split;
        }
        // Timing both: the kept way first, then the whole.
        if self.split.len() < PROBE_FRAMES {
            return true;
        }
        if self.whole.len() < PROBE_FRAMES {
            return false;
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        self.use_split = mean(&self.split) < mean(&self.whole) * 0.9;
        self.hold = HOLD_FRAMES;
        self.split.clear();
        self.whole.clear();
        self.hold -= 1;
        self.use_split
    }
}

/// Vello on the window's graphics chip.
///
/// Its time grows with what is drawn, so a frame that changes only what is on top (a shape being
/// drawn, an object being dragged) draws the unchanged lower part of the page once, into a texture
/// of its own that later frames reuse, and only the rest again, over it as an image. The picture is
/// the one drawing everything in order makes.
pub struct Chip {
    device: wgpu::Device,
    queue: wgpu::Queue,
    egui: std::sync::Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>,
    /// Started on the first frame, so the window opens without waiting for vello's shaders.
    vello: Option<vello::Renderer>,
    /// The texture shown.
    target: Option<Target>,
    /// The kept lower part, when there is one.
    base: Option<Base>,
    last: Option<Last>,
    pace: Pace,
}

impl Chip {
    /// The renderer for egui's device. Vello itself starts on the first frame (building its
    /// shaders takes seconds on the Pi), which the fallback rule does not judge; if it cannot
    /// start, that frame fails and the canvas falls back.
    pub fn new(rs: &egui_wgpu::RenderState) -> Self {
        Chip {
            device: rs.device.clone(),
            queue: rs.queue.clone(),
            egui: rs.renderer.clone(),
            vello: None,
            target: None,
            base: None,
            last: None,
            pace: Pace::default(),
        }
    }

    /// A texture of `size` to draw into in `slot`, and its id in egui: the last one when the size
    /// is the same, else a new one under the same id.
    fn target_in(
        device: &wgpu::Device,
        egui: &egui::mutex::RwLock<egui_wgpu::Renderer>,
        slot: &mut Option<Target>,
        size: (u32, u32),
    ) -> Result<(wgpu::TextureView, egui::TextureId), String> {
        if let Some(t) = slot
            && t.size == size
        {
            return Ok((t.texture.create_view(&Default::default()), t.id));
        }
        let max = device.limits().max_texture_dimension_2d;
        if size.0 > max || size.1 > max {
            return Err(format!("{}×{} is over the chip's {max} px", size.0, size.1));
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("quadrule-chip-canvas"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let mut egui = egui.write();
        let id = match slot.take() {
            Some(old) => {
                egui.update_egui_texture_from_wgpu_texture(
                    device,
                    &view,
                    wgpu::FilterMode::Nearest,
                    old.id,
                );
                old.id
            }
            None => egui.register_native_texture(device, &view, wgpu::FilterMode::Nearest),
        };
        *slot = Some(Target { size, texture, id });
        Ok((view, id))
    }

    fn draw(
        &mut self,
        sc: &vello::Scene,
        view: &wgpu::TextureView,
        size: (u32, u32),
        base_color: Color,
    ) -> Result<(), String> {
        let params = vello::RenderParams {
            base_color,
            width: size.0,
            height: size.1,
            antialiasing_method: vello::AaConfig::Area,
        };
        self.vello
            .as_mut()
            .expect("started by render")
            .render_to_texture(&self.device, &self.queue, sc, view, &params)
            .map_err(|e| e.to_string())
    }
}

impl Chip {
    /// Lets go of the kept lower part.
    fn drop_base(&mut self) {
        if let Some(b) = self.base.take()
            && let Some(v) = self.vello.as_mut()
        {
            v.unregister_texture(b.image);
        }
    }

    /// Draws `items` into the kept lower part (a new texture when the size changed).
    fn make_base(
        &mut self,
        items: &[Item],
        size: (u32, u32),
        background: Color,
    ) -> Result<(), String> {
        if self.base.as_ref().is_some_and(|b| b.size != size) {
            self.drop_base();
        }
        if self.base.is_none() {
            let max = self.device.limits().max_texture_dimension_2d;
            if size.0 > max || size.1 > max {
                return Err(format!("{}×{} is over the chip's {max} px", size.0, size.1));
            }
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("quadrule-chip-base"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let image = self
                .vello
                .as_mut()
                .expect("started by render")
                .register_texture(texture.clone());
            self.base = Some(Base {
                len: 0,
                size,
                texture,
                image,
            });
        }
        let view = self
            .base
            .as_ref()
            .expect("made above")
            .texture
            .create_view(&Default::default());
        self.draw(&scene(items), &view, size, background)?;
        if let Some(b) = self.base.as_mut() {
            b.len = items.len();
        }
        Ok(())
    }
}

/// How many items from the bottom are the same in both lists.
fn common_prefix(a: &[Item], b: &[Item]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

/// The largest `n` up to `limit` at which no opacity group is open.
fn closed_cut(items: &[Item], limit: usize) -> usize {
    let (mut depth, mut cut) = (0usize, 0usize);
    for (i, it) in items.iter().take(limit).enumerate() {
        match it {
            Item::BeginAlpha { .. } => depth += 1,
            Item::EndAlpha => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 {
            cut = i + 1;
        }
    }
    cut
}

/// A rough cost of drawing an item: its path's size.
fn weight(it: &Item) -> usize {
    match it {
        Item::Fill { path, .. } | Item::StrokePath { path, .. } => path.elements().len(),
        _ => 4,
    }
}

impl quadrule_ui::ChipRenderer for Chip {
    fn render(&mut self, dl: &DisplayList) -> Result<(egui::TextureId, f64), String> {
        let start = Instant::now();
        if self.vello.is_none() {
            let v = vello::Renderer::new(
                &self.device,
                vello::RendererOptions {
                    antialiasing_support: vello::AaSupport::area_only(),
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?;
            self.vello = Some(v);
        }
        let size = (dl.width, dl.height);
        // What is the same as last frame, from the bottom of the stack.
        let same = match &self.last {
            Some(l) if l.size == size && l.background == dl.background => {
                common_prefix(&l.items, &dl.items)
            }
            _ => 0,
        };
        // The very same page again: the texture shown still holds it.
        if same == dl.items.len()
            && self.last.as_ref().is_some_and(|l| l.items.len() == same)
            && let Some(t) = self.target.as_ref().filter(|t| t.size == size)
        {
            return Ok((t.id, start.elapsed().as_secs_f64() * 1000.0));
        }
        self.last = Some(Last {
            size,
            background: dl.background,
            items: dl.items.clone(),
        });
        // A base drawn from items that have changed since is no good.
        if self
            .base
            .as_ref()
            .is_some_and(|b| b.len > same || b.size != size)
        {
            self.drop_base();
        }
        let cut = closed_cut(&dl.items, same);
        // Worth keeping apart when it holds at least half of what there is to draw.
        let (below, total): (usize, usize) = (
            dl.items[..cut].iter().map(weight).sum(),
            dl.items.iter().map(weight).sum(),
        );
        let could = cut >= SPLIT_MIN_ITEMS && below * 2 >= total;
        let split = could && self.pace.split_next();
        let (view, id) = Self::target_in(&self.device, &self.egui, &mut self.target, size)?;
        if !split {
            self.drop_base();
            self.draw(&scene(&dl.items), &view, size, gray(dl.background))?;
            self.wait()?;
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            if could && self.pace.whole.len() < PROBE_FRAMES && self.pace.hold == 0 {
                self.pace.whole.push(ms);
            }
            return Ok((id, ms));
        }
        // Keep the base it has unless a good deal more has stood unchanged since, else draw a new one.
        let keep = self.base.as_ref().is_some_and(|b| {
            let between: usize = dl.items[b.len.min(cut)..cut].iter().map(weight).sum();
            between * 5 <= total
        });
        if !keep {
            self.make_base(&dl.items[..cut], size, gray(dl.background))?;
        }
        let base = self.base.as_ref().expect("made above");
        // Vello keeps images in an atlas it may reuse between frames, so the picture is handed over
        // again each time (a copy of the texture, which costs next to nothing).
        if let Some(v) = self.vello.as_mut() {
            v.mark_override_image_dirty(&base.image);
        }
        let mut sc = vello::Scene::new();
        sc.draw_image(&base.image, Affine::IDENTITY);
        sc.append(&scene(&dl.items[base.len..]), None);
        self.draw(&sc, &view, size, gray(dl.background))?;
        self.wait()?;
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        // A frame that had to draw the base is not a fair sample.
        if keep && self.pace.split.len() < PROBE_FRAMES && self.pace.hold == 0 {
            self.pace.split.push(ms);
        }
        Ok((id, ms))
    }
}

impl Chip {
    /// Waits for the chip, so the time measured is the whole frame (the fallback rule).
    fn wait(&self) -> Result<(), String> {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

fn gray(v: u8) -> Color {
    Color::from_rgba8(v, v, v, 255)
}

/// `items` as a vello scene.
fn scene(items: &[Item]) -> vello::Scene {
    let mut s = vello::Scene::new();
    let id = Affine::IDENTITY;
    for item in items {
        match item {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray: g,
            } => s.fill(
                Fill::NonZero,
                id,
                gray(*g),
                None,
                &Rect::new(*x0, *y0, *x1, *y1),
            ),
            Item::Stroke {
                x0,
                y0,
                x1,
                y1,
                width,
                gray: g,
            } => s.stroke(
                &Stroke::new(*width).with_caps(Cap::Butt),
                id,
                gray(*g),
                None,
                &Line::new((*x0, *y0), (*x1, *y1)),
            ),
            Item::Fill { path, gray: g } => s.fill(Fill::NonZero, id, gray(*g), None, path),
            Item::StrokePath {
                path,
                width,
                gray: g,
                cap,
            } => s.stroke(
                &Stroke::new(*width)
                    .with_caps(quadrule_layout::kurbo_cap(*cap))
                    .with_join(Join::Miter),
                id,
                gray(*g),
                None,
                path,
            ),
            Item::BeginAlpha { alpha, bounds } => {
                let (x0, y0, x1, y1) = *bounds;
                s.push_layer(
                    Fill::NonZero,
                    Mix::Normal,
                    f32::from(*alpha) / 255.0,
                    id,
                    &Rect::new(
                        x0.floor() - 1.0,
                        y0.floor() - 1.0,
                        x1.ceil() + 1.0,
                        y1.ceil() + 1.0,
                    ),
                )
            }
            Item::EndAlpha => s.pop_layer(),
        }
    }
    s
}
