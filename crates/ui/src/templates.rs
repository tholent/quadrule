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

//! My templates (rung 16 bbq, Q4, and Defaulted "Save as template…" and "A starter or template becomes a document as
//! an untitled copy"): ordinary `.quad` files kept in `templates/` in the app's data folder, beside `recovery/`
//! (`~/.local/share/quadrule/templates` on Linux). Save as template… writes the open document there through the
//! same code as Save, picture included, and never changes the open document: its file, its unsaved changes and
//! its recent list are as they were. A template opens as an untitled copy, so the original is only ever changed
//! by Save as template… again.
//!
//! No screen in here. A starter (built into the app, rung 16b) opens through the same [`Viewer::open_copy`].

use crate::viewer::{Notice, Source, Viewer};
use quadrule_format::{QuadFile, preview_path};
use quadrule_i18n::Message;
use std::path::{Path, PathBuf};

/// The folder's name in the app's data folder.
pub const DIR_NAME: &str = "templates";

/// Why a template could not be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateError {
    /// Nothing is open.
    NothingOpen,
    /// The app gave no folder for templates.
    NoFolder,
    /// A template of that name is there already; [`Viewer::save_as_template`] with `replace` takes its place.
    Exists(PathBuf),
    /// The file could not be written, and why.
    Write(String),
}

/// One template in the folder, as the welcome screen lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateEntry {
    /// The file.
    pub path: PathBuf,
    /// What the tile says: the document's title, or the file's name when the title is empty or the file
    /// cannot be read.
    pub name: String,
    /// The document's description, for search.
    pub description: String,
    /// The document's tags, for search.
    pub tags: Vec<String>,
    /// The id of the device the template was designed on.
    pub primary: String,
    /// The ids of every device it targets.
    pub devices: Vec<String>,
    /// The picture stored in the file for its main device (PNG bytes), if it has one.
    pub preview: Option<Vec<u8>>,
    /// The file's own time of last change, to tell a template that was saved again.
    pub modified: Option<std::time::SystemTime>,
    /// The file could not be read as a `.quad`; its tile is plain, and picking it says why.
    pub unreadable: bool,
}

impl TemplateEntry {
    /// The entry for the file at `path`.
    pub fn read(path: &Path) -> TemplateEntry {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        let plain = |unreadable| TemplateEntry {
            path: path.to_path_buf(),
            name: stem.clone(),
            description: String::new(),
            tags: Vec::new(),
            primary: String::new(),
            devices: Vec::new(),
            preview: None,
            modified,
            unreadable,
        };
        let Some(file) = std::fs::read(path)
            .ok()
            .and_then(|b| QuadFile::from_bytes(&b).ok())
        else {
            return plain(true);
        };
        let doc = &file.doc;
        let title = doc.metadata.title.trim();
        let name = if title.is_empty() {
            stem.clone()
        } else {
            title.to_string()
        };
        TemplateEntry {
            name,
            description: doc.metadata.description.clone(),
            tags: doc.metadata.tags.clone(),
            primary: doc.primary_profile_id.clone(),
            devices: doc.profile_ids().into_iter().map(String::from).collect(),
            preview: file
                .previews
                .get(&preview_path(&doc.primary_profile_id))
                .cloned(),
            ..plain(false)
        }
    }
}

/// The `.quad` files in `dir`, by name (ignoring case). A folder that is not there is an empty list.
pub fn list_templates(dir: &Path) -> Vec<TemplateEntry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<TemplateEntry> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "quad"))
        // The half-written file Save leaves for an instant is `.name.saving`, which is not a `.quad`.
        .map(|p| TemplateEntry::read(&p))
        .collect();
    out.sort_by_key(|t| (t.name.to_lowercase(), t.path.clone()));
    out
}

impl Viewer {
    /// The file Save as template… would write for the open document: `templates/<slug of its title>.quad`.
    pub fn template_path(&self) -> Option<PathBuf> {
        let dir = self.templates_dir.as_ref()?;
        self.open.as_ref()?;
        let title = self.title().unwrap_or("untitled");
        Some(dir.join(format!("{}.quad", crate::viewer::slug(title))))
    }

    /// Save as template…: writes the open document to [`Viewer::template_path`] as Save would write it (the main
    /// device's picture included). A template of that name already there is left alone and
    /// [`TemplateError::Exists`] says so, unless `replace` is set. The open document is not changed. On
    /// success the notice says where it went.
    pub fn save_as_template(&mut self, replace: bool) -> Result<PathBuf, TemplateError> {
        if self.open.is_none() {
            return Err(TemplateError::NothingOpen);
        }
        let Some(path) = self.template_path() else {
            return Err(TemplateError::NoFolder);
        };
        if path.exists() && !replace {
            return Err(TemplateError::Exists(path));
        }
        let written = self.save_bytes().and_then(|bytes| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            crate::editor::write_replacing(&path, &bytes)
                .map_err(|e| format!("{}: {e}", path.display()))
        });
        match written {
            Ok(()) => {
                self.notice = Some(Notice {
                    message: Message::new("notice-template-saved")
                        .text("name", self.title().unwrap_or_default()),
                    error: false,
                });
                Ok(path)
            }
            Err(e) => {
                self.notice = Some(Notice {
                    message: Message::new("notice-template-save-failed").text("error", e.clone()),
                    error: true,
                });
                Err(TemplateError::Write(e))
            }
        }
    }

    /// The templates in the app's folder (none when it gave no folder).
    pub fn templates(&self) -> Vec<TemplateEntry> {
        self.templates_dir
            .as_deref()
            .map(list_templates)
            .unwrap_or_default()
    }

    /// Deletes a template's file. One-way: the welcome screen asks first. Only a `.quad` in the templates folder
    /// can go.
    pub fn remove_template(&mut self, path: &Path) -> Result<(), String> {
        let inside = self
            .templates_dir
            .as_deref()
            .is_some_and(|d| path.parent() == Some(d))
            && path.extension().is_some_and(|x| x == "quad");
        if !inside {
            return Err(path.display().to_string());
        }
        std::fs::remove_file(path).map_err(|e| {
            self.notice = Some(Notice {
                message: Message::new("notice-template-remove-failed")
                    .text("path", path.display().to_string())
                    .text("error", e.to_string()),
                error: true,
            });
            e.to_string()
        })
    }

    /// Opens a `.quad` file's bytes as an **untitled copy** (Defaulted "A starter or template becomes a
    /// document"): no file until Save, the title it had, created and modified now, and no unsaved changes until
    /// it is edited. `device_id` becomes the main device: added to the devices the page targets first if the
    /// file lacks it, then made primary, as the Page tab does. If that cannot be done the copy opens on its own
    /// main device and the notice says why.
    pub fn open_copy(&mut self, bytes: &[u8], device_id: &str) -> Result<(), String> {
        self.open_bytes(Source::Untitled, bytes)?;
        let now = quadrule_model::blank::now_rfc3339();
        let Some(o) = self.open.as_mut() else {
            return Ok(());
        };
        o.doc.metadata.created = now.clone();
        o.doc.metadata.modified = now;
        let moved = if o.doc.primary_profile_id == device_id {
            Ok(None)
        } else {
            main_device_of(&o.doc, device_id).map(Some)
        };
        match moved {
            Ok(doc) => {
                if let Some(doc) = doc
                    && let Some(o) = self.open.as_mut()
                {
                    o.doc = doc;
                }
                self.profile_id = device_id.to_string();
            }
            Err(e) => {
                self.notice = Some(Notice {
                    message: Message::new("notice-copy-device")
                        .text("device", crate::viewer::device_label(device_id))
                        .text("error", e),
                    error: false,
                });
            }
        }
        self.invalidate_pages();
        Ok(())
    }

    /// Opens the template or other `.quad` file at `path` as an untitled copy, on `device_id` (see
    /// [`Viewer::open_copy`]). On failure the open document stays as it was and the notice says why.
    pub fn open_template(&mut self, path: &Path, device_id: &str) -> Result<(), String> {
        let result = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| self.open_copy(&b, device_id));
        if let Err(e) = &result {
            self.notice = Some(Notice {
                message: Message::new("notice-open-failed")
                    .text("path", path.display().to_string())
                    .text("error", e.clone()),
                error: true,
            });
        }
        result
    }
}

/// `doc` with `device_id` among its devices and made the main one: the Page tab's two steps, applied to a copy so a
/// failure leaves nothing half done.
fn main_device_of(
    doc: &quadrule_model::Document,
    device_id: &str,
) -> Result<quadrule_model::Document, String> {
    let mut doc = doc.clone();
    let mut ids: Vec<String> = doc.profile_ids().into_iter().map(String::from).collect();
    if !ids.iter().any(|i| i == device_id) {
        ids.push(device_id.to_string());
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let cmd = quadrule_layout::device::set_targets(&doc, &refs).map_err(|e| e.to_string())?;
        doc.apply(cmd).map_err(|e| format!("{e:?}"))?;
    }
    let cmd = quadrule_layout::device::make_primary(&doc, device_id).map_err(|e| e.to_string())?;
    doc.apply(cmd).map_err(|e| format!("{e:?}"))?;
    Ok(doc)
}
