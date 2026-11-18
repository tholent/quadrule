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

//! Recent files (rung 16 bbq, Defaulted "Recent files"): the files opened or saved last, newest first, at most ten,
//! kept in `settings.ron`. Opening a file, Save and Save As add one; a file that has gone from the disk stays in
//! the list, shown greyed as "Not found", until the person takes it off. No screen in here: the welcome screen and
//! File > Open Recent show what [`Viewer::recent_files`] gives.

use crate::viewer::Viewer;
use std::path::{Path, PathBuf};

/// One line of the recent list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecentFile {
    /// Where the file is (or was).
    pub path: PathBuf,
    /// The file's name, as the list shows it.
    pub name: String,
    /// The folder it is in, shown small under the name.
    pub folder: String,
    /// Whether the file is on the disk now.
    pub exists: bool,
}

impl RecentFile {
    /// The line for `path`, looking at the disk to see whether the file is still there.
    pub fn of(path: &Path) -> RecentFile {
        RecentFile {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            folder: path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            exists: path.is_file(),
        }
    }
}

impl Viewer {
    /// Puts `path` first in the recent list and writes the settings. A relative path is kept as an absolute
    /// one, so the list means the same thing whichever folder the app is started from.
    pub fn note_recent(&mut self, path: &Path) {
        let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        self.change_settings(|s| s.note_recent(&path));
    }

    /// Takes `path` off the recent list and writes the settings.
    pub fn forget_recent(&mut self, path: &Path) {
        let path = path.to_string_lossy().into_owned();
        self.change_settings(|s| s.forget_recent(&path));
    }

    /// The recent list, newest first, each line checked against the disk.
    pub fn recent_files(&self) -> Vec<RecentFile> {
        self.settings
            .recent
            .iter()
            .map(|p| RecentFile::of(Path::new(p)))
            .collect()
    }
}
