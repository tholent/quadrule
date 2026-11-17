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

//! The export engine (rung 15 bbq, Q2 to Q9). Headless: an [`ExportRequest`] (which devices, which folder, the
//! path template, the Set, the text note) goes in; files are written, folders made, and an [`ExportResult`] comes
//! back with every file written, how many replaced an old one, and a shrunk picture of each for the completion
//! sheet. [`Viewer::export_again`] repeats the last choices with no sheet. The sheets themselves are the next
//! leg's; the toolbar's two items ride on this through [`Viewer::export`] until then.
//!
//! The bytes of each PNG are the export renderer's (ADR-003), as ever; with the note on, the file only gains
//! text chunks (Software `quadrule <version>`, Title the document's title).

use crate::export_path::{self, DeviceTokens, DocTokens, PathError};
use crate::settings::{DevicesChoice, Settings, home_dir};
use crate::viewer::{ExportWhich, Notice, Source, Viewer, export_pixels, shrink};
use quadrule_i18n::Message;
use quadrule_model::{DocumentType, Orientation};
use quadrule_profiles::active::lookup_in;
use quadrule_render::{Gray, PngMeta, encode_png};
use std::path::{Path, PathBuf};

/// The longest side of the picture of each exported file, in pixels, for the completion sheet.
pub const THUMB_SIDE: u32 = 240;

impl From<DevicesChoice> for ExportWhich {
    fn from(c: DevicesChoice) -> Self {
        match c {
            DevicesChoice::This => ExportWhich::ThisDevice,
            DevicesChoice::All => ExportWhich::AllDevices,
        }
    }
}

impl From<ExportWhich> for DevicesChoice {
    fn from(w: ExportWhich) -> Self {
        match w {
            ExportWhich::ThisDevice => DevicesChoice::This,
            ExportWhich::AllDevices => DevicesChoice::All,
        }
    }
}

/// Everything an export needs to know.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportRequest {
    /// The profile ids to write, in the document's order.
    pub devices: Vec<String>,
    /// The chosen folder; the template's folders are made inside it.
    pub folder: PathBuf,
    /// The path template ([`crate::export_path`]).
    pub template: String,
    /// The Set box, for `{set}`.
    pub set: String,
    /// Whether each PNG carries the text note.
    pub note: bool,
    /// The export day, `YYYY-MM-DD`, for `{date}`.
    pub date: String,
}

/// One file written.
#[derive(Clone, Debug)]
pub struct ExportedFile {
    /// The profile id.
    pub device: String,
    /// Where it was written (the folder joined with `relative`).
    pub path: PathBuf,
    /// The path inside the chosen folder, with `/` between parts.
    pub relative: String,
    /// Whether it took the place of a file that was there.
    pub replaced: bool,
    /// The PNG's size in pixels.
    pub size: (u32, u32),
    /// The pixels just written, shrunk to [`THUMB_SIDE`] (never enlarged), for the completion sheet.
    pub thumbnail: Gray,
}

/// What an export did.
#[derive(Clone, Debug)]
pub struct ExportResult {
    /// The folder chosen.
    pub folder: PathBuf,
    /// Every file written, in the document's device order.
    pub files: Vec<ExportedFile>,
    /// How many of them replaced a file that was there.
    pub replaced: usize,
    /// The deepest folder every file is in (what Reveal in folder opens).
    pub shared_folder: PathBuf,
}

impl ExportResult {
    /// The paths written.
    pub fn paths(&self) -> Vec<PathBuf> {
        self.files.iter().map(|f| f.path.clone()).collect()
    }
}

/// Why an export did not happen.
#[derive(Clone, Debug, PartialEq)]
pub enum ExportError {
    /// No document is open.
    NothingOpen,
    /// The template (or what it makes) is not usable.
    Path(PathError),
    /// A page could not be made.
    Page(String),
    /// A file or folder could not be written.
    Write(String),
}

impl From<PathError> for ExportError {
    fn from(e: PathError) -> Self {
        ExportError::Path(e)
    }
}

impl ExportError {
    /// The words for it, through Fluent.
    pub fn message(&self) -> Message {
        match self {
            ExportError::NothingOpen => Message::new("export-error-nothing-open"),
            ExportError::Path(PathError::UnknownToken(t)) => {
                Message::new("export-error-unknown-token").text("token", t.clone())
            }
            ExportError::Path(PathError::Unclosed) => Message::new("export-error-unclosed"),
            ExportError::Path(PathError::NoFileName) => Message::new("export-error-no-file-name"),
            ExportError::Path(PathError::SamePath {
                first,
                second,
                path,
            }) => Message::new("export-error-same-path")
                .text("first", first.clone())
                .text("second", second.clone())
                .text("path", path.clone()),
            ExportError::Page(e) | ExportError::Write(e) => {
                Message::new("notice-export-failed").text("error", e.clone())
            }
        }
    }
}

/// What Export again did.
#[derive(Clone, Debug)]
pub enum Again {
    /// It wrote the files.
    Done(ExportResult),
    /// Nothing was exported from this document yet, so the Export sheet is needed.
    NeedsSheet,
    /// It could not.
    Failed(ExportError),
}

/// The deepest folder that holds every path in `paths` (their parents' common start).
fn shared_folder(folder: &Path, paths: &[PathBuf]) -> PathBuf {
    let mut shared: Option<Vec<std::path::Component>> = None;
    for p in paths {
        let parent: Vec<_> = p.parent().unwrap_or(folder).components().collect();
        shared = Some(match shared {
            None => parent,
            Some(s) => s
                .into_iter()
                .zip(parent)
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect(),
        });
    }
    shared
        .map(|c| c.into_iter().collect())
        .unwrap_or_else(|| folder.to_path_buf())
}

impl Viewer {
    /// Reads the settings from `path` and keeps writing them there (the app gives its config folder's
    /// `settings.ron`). A damaged file gives the defaults and a notice.
    pub fn use_settings_file(&mut self, path: PathBuf) {
        let loaded = Settings::load(&path);
        self.settings = loaded.settings;
        self.settings_path = Some(path);
        if let Some(why) = loaded.damaged {
            self.notice = Some(Notice {
                message: Message::new("notice-settings-damaged").text("error", why),
                error: true,
            });
        }
    }

    /// Changes the settings and saves them, if there is a file for them.
    pub fn change_settings(&mut self, f: impl FnOnce(&mut Settings)) {
        f(&mut self.settings);
        self.save_settings();
    }

    /// Writes the settings file, if there is one to write.
    pub fn save_settings(&mut self) {
        let Some(path) = &self.settings_path else {
            return;
        };
        if let Err(e) = self.settings.save(path) {
            self.notice = Some(Notice {
                message: Message::new("notice-settings-save-failed")
                    .text("error", format!("{}: {e}", path.display())),
                error: true,
            });
        }
    }

    /// Today's date as `YYYY-MM-DD`, in local time (a test can set `export_date` to fix it).
    pub fn export_date(&self) -> String {
        self.export_date
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string())
    }

    /// The file path of the open document, if it has been saved (what the folder memory is keyed by).
    fn document_path(&self) -> Option<PathBuf> {
        match &self.open.as_ref()?.source {
            Source::File(p) => Some(p.clone()),
            Source::Sample | Source::Untitled => None,
        }
    }

    /// The folder the Export sheet or dialog starts in: the one this document last exported to, else the last
    /// folder used for any document, else the document's own folder, else the home folder (Q4).
    pub fn export_start_folder(&self) -> Option<PathBuf> {
        self.settings
            .start_folder(self.document_path().as_deref(), home_dir().as_deref())
    }

    /// The values a template can use that do not depend on the device.
    pub fn export_doc_tokens(&self, set: &str, date: &str) -> DocTokens {
        let fallback = match self.open.as_ref().map(|o| &o.source) {
            Some(Source::Sample) => "sample",
            _ => "untitled",
        };
        let doc = self.doc();
        DocTokens {
            title: self.title().unwrap_or(fallback).to_string(),
            orientation: match doc.map(|d| d.page.orientation) {
                Some(Orientation::Landscape) => "landscape",
                _ => "portrait",
            }
            .into(),
            kind: match doc.map(|d| d.document_type) {
                Some(DocumentType::Wallpaper) => "wallpaper",
                _ => "template",
            }
            .into(),
            date: date.into(),
            version: env!("CARGO_PKG_VERSION").replace('.', "-"),
            set: set.into(),
        }
    }

    /// The values that belong to device `id`.
    pub fn export_device_tokens(&self, id: &str) -> Result<DeviceTokens, ExportError> {
        let doc = self.doc().ok_or(ExportError::NothingOpen)?;
        let p = lookup_in(doc, id).map_err(|e| ExportError::Page(e.to_string()))?;
        let (width, height) = p.device_size();
        Ok(DeviceTokens {
            id: id.into(),
            device: p.short_name.clone(),
            model: p.display_name.clone(),
            width,
            height,
        })
    }

    /// The path every device of `req` would be written to, inside the chosen folder (the Export sheet's live
    /// preview; the first mistake is the error).
    pub fn plan_export(&self, req: &ExportRequest) -> Result<Vec<(String, String)>, ExportError> {
        let doc_tokens = self.export_doc_tokens(&req.set, &req.date);
        let devices: Vec<DeviceTokens> = req
            .devices
            .iter()
            .map(|id| self.export_device_tokens(id))
            .collect::<Result<_, _>>()?;
        let paths = export_path::plan(&req.template, &doc_tokens, &devices)?;
        Ok(req.devices.iter().cloned().zip(paths).collect())
    }

    /// A request built from the remembered settings: the devices the settings choose (or `which`, when given),
    /// into `folder`.
    pub fn export_request(&self, which: Option<ExportWhich>, folder: PathBuf) -> ExportRequest {
        let which = which.unwrap_or(self.settings.devices.into());
        ExportRequest {
            devices: self.export_devices(which),
            folder,
            template: self.settings.template.clone(),
            set: self.settings.set.clone(),
            note: self.settings.note,
            date: self.export_date(),
        }
    }

    /// Writes the PNGs of `req`. Every path is worked out first, so a mistake in the template writes nothing.
    /// Files that are there are replaced (Defaulted "Existing files are replaced"). On success the folder is
    /// remembered for this document and as the last one used, and the notice says how many files were written;
    /// on failure the notice says why.
    pub fn run_export(&mut self, req: &ExportRequest) -> Result<ExportResult, ExportError> {
        self.end_typing();
        let result = self.write_export(req);
        match &result {
            Ok(r) => {
                let doc = self.document_path();
                if let Some(o) = &mut self.open {
                    o.exported_to = Some(r.folder.clone());
                }
                self.export_dir = Some(r.folder.clone());
                self.change_settings(|s| s.remember_folder(doc.as_deref(), &r.folder));
                self.notice = Some(Notice {
                    message: Message::new("notice-exported")
                        .num("count", r.files.len() as f64, 0)
                        .text("folder", r.folder.display().to_string()),
                    error: false,
                });
            }
            Err(e) => {
                self.notice = Some(Notice {
                    message: e.message(),
                    error: true,
                });
            }
        }
        result
    }

    fn write_export(&mut self, req: &ExportRequest) -> Result<ExportResult, ExportError> {
        let plan = self.plan_export(req)?;
        let title = self.title().unwrap_or("").to_string();
        let software = crate::editor::SAVED_BY;
        let mut files = Vec::new();
        for (device, relative) in plan {
            let page = self.page_for(&device).map_err(ExportError::Page)?;
            let px = export_pixels(page);
            let meta = req.note.then_some(PngMeta {
                software,
                title: &title,
            });
            let bytes = encode_png(&px, meta);
            let scale = (THUMB_SIDE as f64 / px.width.max(px.height) as f64).min(1.0);
            let thumbnail = shrink(
                &px,
                ((px.width as f64 * scale).round() as u32).max(1),
                ((px.height as f64 * scale).round() as u32).max(1),
            );
            let size = (px.width, px.height);
            let mut path = req.folder.clone();
            path.extend(relative.split('/'));
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| ExportError::Write(format!("{}: {e}", dir.display())))?;
            }
            let replaced = path.exists();
            crate::editor::write_replacing(&path, &bytes)
                .map_err(|e| ExportError::Write(format!("{}: {e}", path.display())))?;
            files.push(ExportedFile {
                device,
                path,
                relative,
                replaced,
                size,
                thumbnail,
            });
        }
        let paths: Vec<PathBuf> = files.iter().map(|f| f.path.clone()).collect();
        Ok(ExportResult {
            shared_folder: shared_folder(&req.folder, &paths),
            replaced: files.iter().filter(|f| f.replaced).count(),
            folder: req.folder.clone(),
            files,
        })
    }

    /// Writes PNGs into `dir`: the device on screen, or every device, named by the remembered template (the
    /// toolbar's two items, until the Export sheet replaces them). Returns the files written. The choice of
    /// devices is remembered too. On failure the notice says why.
    pub fn export(&mut self, dir: &Path, which: ExportWhich) -> Result<Vec<PathBuf>, String> {
        self.change_settings(|s| s.devices = which.into());
        let req = self.export_request(Some(which), dir.to_path_buf());
        self.run_export(&req)
            .map(|r| r.paths())
            .map_err(|e| match e {
                ExportError::Page(s) | ExportError::Write(s) => s,
                other => quadrule_i18n::Strings::english().format(&other.message()),
            })
    }

    /// The folder Export again would write to: where this document exported this session, else the folder
    /// remembered for its file. `None` when nothing was exported from it yet.
    pub fn export_again_folder(&self) -> Option<PathBuf> {
        let o = self.open.as_ref()?;
        o.exported_to.clone().or_else(|| {
            self.document_path()
                .and_then(|p| self.settings.folder_of(&p))
        })
    }

    /// Whether Export again can write at once (something was exported from this document before).
    pub fn can_export_again(&self) -> bool {
        self.export_again_folder().is_some()
    }

    /// Export again: writes once more with the remembered choices (devices, template, Set, note) into the
    /// remembered folder, with no sheet. When nothing was exported from this document yet, it writes nothing and
    /// says the Export sheet is needed.
    pub fn export_again(&mut self) -> Again {
        let Some(folder) = self.export_again_folder() else {
            return Again::NeedsSheet;
        };
        let req = self.export_request(None, folder);
        match self.run_export(&req) {
            Ok(r) => Again::Done(r),
            Err(e) => Again::Failed(e),
        }
    }
}
