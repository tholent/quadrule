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

//! The built-in starters (rung 16 bbq, Defaulted "Where starters live"): `.quad` files kept in
//! `crates/ui/starters/<category>/<slug>.quad`, built into the program by `build.rs`. The folder names the category
//! (`paper`, `notes`, `planning`, `bullet-journal`, `creative`), so the file format does not change. The files are
//! written by `cargo xtask starters` (`crates/xtask/src/starters.rs`), not by hand. A starter opens as an untitled copy exactly as a template does
//! ([`crate::Viewer::open_copy`]).
//!
//! No screen in here. [`load`] turns a list of (folder, slug, bytes) into gallery entries, so a test can hand it
//! starters of its own.

use crate::templates::{Origin, TemplateEntry};
use crate::welcome::Category;
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/starters.rs"));

/// Gallery entries for `list`: (category folder, slug, the `.quad` file's bytes). A folder that is not one of the
/// five categories is left out. Sorted by category in tab order, then by name ignoring case.
pub fn load<B: AsRef<[u8]>>(list: &[(&str, &str, B)]) -> Vec<TemplateEntry> {
    let mut out: Vec<(usize, TemplateEntry)> = list
        .iter()
        .filter_map(|(folder, slug, bytes)| {
            let rank = Category::ALL
                .iter()
                .position(|c| c.folder() == Some(folder))?;
            let path = PathBuf::from(format!("starter/{folder}/{slug}.quad"));
            let entry = TemplateEntry::from_bytes(
                path,
                Origin::Starter {
                    folder: (*folder).to_string(),
                },
                Some(bytes.as_ref()),
            );
            Some((rank, entry))
        })
        .collect();
    out.sort_by_key(|(rank, e)| (*rank, e.name.to_lowercase()));
    out.into_iter().map(|(_, e)| e).collect()
}

/// The starters built into this program.
pub fn built_in() -> Vec<TemplateEntry> {
    load(BUILT_IN)
}
