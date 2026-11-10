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

//! Autosave and recovery (napkin §5.6; rung 3 bbq, Defaulted "Autosave"). While there are unsaved
//! changes, the document is written every [`AUTOSAVE_EVERY_SECS`] seconds, and when the window
//! loses focus, to one recovery file per document in a folder the app chooses (the user's data
//! folder). A clean Save or close deletes it. On start, any recovery file left behind (by a
//! crash) is offered back: Restore or Discard.
//!
//! Each recovery file is a normal `.quad` (`<key>.quad`) with a small note beside it
//! (`<key>.json`) saying which file it belongs to.

use crate::viewer::{Source, Viewer};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// How often unsaved changes are written to the recovery file. The napkin leaves N open; the
/// rung 3 bbq chose 30 seconds.
pub const AUTOSAVE_EVERY_SECS: f64 = 30.0;

/// The folder recovery files live in.
#[derive(Clone, Debug)]
pub struct Recovery {
    dir: PathBuf,
}

/// The note beside a recovery file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Note {
    /// The file the document belongs to; `None` for the sample page (never saved).
    source: Option<PathBuf>,
}

/// A recovery file found on start.
#[derive(Clone, Debug, PartialEq)]
pub struct Recovered {
    /// Where the document came from.
    pub source: Source,
    /// The recovery file.
    pub quad: PathBuf,
}

impl Recovery {
    /// Recovery files in `dir` (made when first needed).
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Recovery { dir: dir.into() }
    }

    /// The folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// A document's recovery file name, without extension: `sample`, or a short hash of the
    /// file's path, so one document always has the same recovery file.
    pub fn key(source: &Source) -> String {
        match source {
            Source::Sample => "sample".into(),
            Source::File(p) => {
                let hash = quadrule_format::sha256_hex(p.to_string_lossy().as_bytes());
                format!("file-{}", &hash[..16])
            }
        }
    }

    fn paths(&self, source: &Source) -> (PathBuf, PathBuf) {
        let key = Self::key(source);
        (
            self.dir.join(format!("{key}.quad")),
            self.dir.join(format!("{key}.json")),
        )
    }

    /// Writes the open document's recovery file.
    pub fn write(&self, viewer: &mut Viewer) -> Result<PathBuf, String> {
        let source = viewer
            .open
            .as_ref()
            .ok_or("nothing is open")?
            .source
            .clone();
        let bytes = viewer.file_bytes()?;
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let (quad, json) = self.paths(&source);
        let note = Note {
            source: match &source {
                Source::File(p) => Some(p.clone()),
                Source::Sample => None,
            },
        };
        let note = serde_json::to_vec_pretty(&note).map_err(|e| e.to_string())?;
        crate::editor::write_replacing(&json, &note).map_err(|e| e.to_string())?;
        crate::editor::write_replacing(&quad, &bytes).map_err(|e| e.to_string())?;
        Ok(quad)
    }

    /// Deletes a document's recovery file, if it has one (after a clean Save or close).
    pub fn discard(&self, source: &Source) {
        let (quad, json) = self.paths(source);
        let _ = std::fs::remove_file(quad);
        let _ = std::fs::remove_file(json);
    }

    /// The recovery files left in the folder, oldest name first.
    pub fn list(&self) -> Vec<Recovered> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut found: Vec<Recovered> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .filter_map(|json| {
                let note: Note = serde_json::from_slice(&std::fs::read(&json).ok()?).ok()?;
                let quad = json.with_extension("quad");
                quad.exists().then(|| Recovered {
                    source: note.source.map_or(Source::Sample, Source::File),
                    quad,
                })
            })
            .collect();
        found.sort_by(|a, b| a.quad.cmp(&b.quad));
        found
    }

    /// Opens a recovery file as the document it belongs to, with unsaved changes, so Save writes
    /// it back to its own file. The recovery file stays until a clean Save or close.
    pub fn restore(&self, r: &Recovered, viewer: &mut Viewer) -> Result<(), String> {
        let bytes = std::fs::read(&r.quad).map_err(|e| e.to_string())?;
        viewer.open_bytes(r.source.clone(), &bytes)?;
        if let Some(o) = &mut viewer.open {
            o.history.mark_unsaved();
        }
        Ok(())
    }

    /// Deletes a recovery file the user chose not to restore.
    pub fn forget(&self, r: &Recovered) {
        let _ = std::fs::remove_file(&r.quad);
        let _ = std::fs::remove_file(r.quad.with_extension("json"));
    }
}

/// When to autosave.
#[derive(Clone, Debug, Default)]
pub struct Autosave {
    /// When the last autosave ran (or the clock started), in seconds.
    last: Option<f64>,
    /// The document revision last written.
    written: Option<(u64, u64)>,
}

impl Autosave {
    /// Whether to write the recovery file now: there are unsaved changes not yet written, and
    /// either the window just lost focus or [`AUTOSAVE_EVERY_SECS`] have passed. `now` is a clock
    /// in seconds. When it says yes, it counts the write as done.
    pub fn due(&mut self, viewer: &Viewer, now: f64, lost_focus: bool) -> bool {
        let last = *self.last.get_or_insert(now);
        let state = (viewer.generation, viewer.revision);
        let unwritten = viewer.is_dirty() && self.written != Some(state);
        if unwritten && (lost_focus || now - last >= AUTOSAVE_EVERY_SECS) {
            self.last = Some(now);
            self.written = Some(state);
            true
        } else {
            if !unwritten {
                // The 30 seconds count from the first unsaved change, not from long ago.
                self.last = Some(now);
            }
            false
        }
    }
}
