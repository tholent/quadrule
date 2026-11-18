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

//! The app's settings file (rung 15 bbq, Q3, Q4 and Defaulted "Settings file" and "The very first folder"):
//! `settings.ron` in the app's config folder (`XDG_CONFIG_HOME/quadrule` on Linux), next to `profiles/`. It holds
//! what Export remembers: the path template, the Set, the text-note tick, this device or all devices, the folder
//! each document last exported to, the last folder used for any document, and whether the completion sheet's
//! "Where do these go?" note is open. Nothing about folders is ever written into a `.quad` file.
//!
//! No screen in here. A missing file is the defaults with no fuss; a damaged one is the defaults and a reason the
//! window turns into a notice. A save writes a new file beside the old one and renames it over it.

use crate::export_path::DEFAULT_TEMPLATE;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The file's name in the config folder.
pub const FILE_NAME: &str = "settings.ron";

/// How many documents' folders are kept; the oldest go first.
pub const MAX_FOLDERS: usize = 200;

/// How many recent files are kept (rung 16 bbq, Defaulted "Recent files"); the oldest go first.
pub const MAX_RECENT: usize = 10;

/// Which devices Export writes (Q2).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DevicesChoice {
    /// The device on screen.
    This,
    /// Every device the document has.
    #[default]
    All,
}

/// One document's remembered folder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FolderEntry {
    /// The document's file path.
    pub document: String,
    /// The folder it last exported to.
    pub folder: String,
}

/// Everything the file holds. A field missing from the file takes its default, and a field this build does not
/// know is ignored, so an older or newer file still loads.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// The path template, app-wide (Q3).
    pub template: String,
    /// The Set box.
    pub set: String,
    /// Whether exports carry the text note (Q6).
    pub note: bool,
    /// This device or all devices.
    pub devices: DevicesChoice,
    /// The folder each document last exported to, oldest first (Q4). At most [`MAX_FOLDERS`].
    pub folders: Vec<FolderEntry>,
    /// The last folder used for any document.
    pub last_folder: Option<String>,
    /// Whether the completion sheet's "Where do these go?" note is open (Q7); folded by default.
    pub completion_note_open: bool,
    /// The files opened or saved last, newest first, at most [`MAX_RECENT`] (rung 16 bbq, Defaulted "Recent
    /// files"). A file written by an older build has none, and loads all the same.
    pub recent: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            template: DEFAULT_TEMPLATE.into(),
            set: String::new(),
            note: true,
            devices: DevicesChoice::All,
            folders: Vec::new(),
            last_folder: None,
            completion_note_open: false,
            recent: Vec::new(),
        }
    }
}

/// What reading the file gave.
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    /// The settings: the file's, or the defaults.
    pub settings: Settings,
    /// Why the file was not used, when it was damaged (it is left on disk as it was until the next change).
    pub damaged: Option<String>,
}

impl Settings {
    /// Reads `path`. No file gives the defaults; a file that cannot be read or parsed gives the defaults and says
    /// why.
    pub fn load(path: &Path) -> Loaded {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Loaded {
                    settings: Settings::default(),
                    damaged: None,
                };
            }
            Err(e) => {
                return Loaded {
                    settings: Settings::default(),
                    damaged: Some(format!("{}: {e}", path.display())),
                };
            }
        };
        match Settings::parse(&text) {
            Ok(settings) => Loaded {
                settings,
                damaged: None,
            },
            Err(e) => Loaded {
                settings: Settings::default(),
                damaged: Some(format!("{}: {e}", path.display())),
            },
        }
    }

    /// Reads settings from RON text.
    pub fn parse(text: &str) -> Result<Settings, String> {
        let mut s: Settings = ron::from_str(text).map_err(|e| e.to_string())?;
        if s.folders.len() > MAX_FOLDERS {
            let extra = s.folders.len() - MAX_FOLDERS;
            s.folders.drain(..extra);
        }
        s.recent.truncate(MAX_RECENT);
        Ok(s)
    }

    /// The file's text.
    pub fn to_text(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .expect("settings are plain data")
    }

    /// Writes the file: the folder is made if need be, and a new file is renamed over the old one so a failed save
    /// never leaves half a file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        crate::editor::write_replacing(path, self.to_text().as_bytes())
    }

    /// Notes that `document` (a saved file, if it has a path) exported to `folder`: the folder becomes the last
    /// one used, and this document's entry moves to the newest place.
    pub fn remember_folder(&mut self, document: Option<&Path>, folder: &Path) {
        let folder = folder.to_string_lossy().into_owned();
        if let Some(doc) = document {
            let doc = doc.to_string_lossy().into_owned();
            self.folders.retain(|e| e.document != doc);
            self.folders.push(FolderEntry {
                document: doc,
                folder: folder.clone(),
            });
            if self.folders.len() > MAX_FOLDERS {
                let extra = self.folders.len() - MAX_FOLDERS;
                self.folders.drain(..extra);
            }
        }
        self.last_folder = Some(folder);
    }

    /// Notes that `path` was opened or saved: it goes first in the recent list, a copy lower down is dropped, and
    /// only the newest [`MAX_RECENT`] are kept.
    pub fn note_recent(&mut self, path: &Path) {
        let path = path.to_string_lossy().into_owned();
        self.recent.retain(|p| *p != path);
        self.recent.insert(0, path);
        self.recent.truncate(MAX_RECENT);
    }

    /// Takes `path` off the recent list.
    pub fn forget_recent(&mut self, path: &str) {
        self.recent.retain(|p| p != path);
    }

    /// The folder this document last exported to, if it has a path and one is kept.
    pub fn folder_of(&self, document: &Path) -> Option<PathBuf> {
        let doc = document.to_string_lossy();
        self.folders
            .iter()
            .find(|e| e.document == doc)
            .map(|e| PathBuf::from(&e.folder))
    }

    /// The folder Export starts in (Defaulted "The very first folder"): the one this document's path last
    /// exported to, else the last folder used for any document, else the document's own folder if it has been
    /// saved, else `home`. A remembered folder that is gone from the disk is passed over.
    pub fn start_folder(&self, document: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
        let remembered = document.and_then(|d| self.folder_of(d));
        let last = self.last_folder.as_ref().map(PathBuf::from);
        let own = document.and_then(|d| d.parent()).map(Path::to_path_buf);
        remembered
            .into_iter()
            .chain(last)
            .filter(|p| p.is_dir())
            .chain(own.filter(|p| !p.as_os_str().is_empty()))
            .chain(home.map(Path::to_path_buf))
            .next()
    }
}

/// The user's home folder, from the environment (`HOME`, or `USERPROFILE` on Windows).
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}
