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

//! Copy and paste between documents (rung 4 bbq Q8, Defaulted "Copy and paste"). The clipboard
//! holds text: the copied objects as JSON under a top-level marker, so a paste reads only what
//! quadrule put there and ignores any other text on the clipboard. No window code is in here.
//!
//! ```json
//! {"quadrule-clipboard": 1, "objects": [ ...objects... ]}
//! ```
//!
//! Objects are written in stacking order (bottom first). Text outlines are not copied: they are
//! made again for each profile when a document is saved, as Save does (napkin §7.3).

use crate::{Document, Object, Uuid};
use serde::{Deserialize, Serialize};

/// The marker key at the top of the clipboard JSON.
pub const MARKER: &str = "quadrule-clipboard";

/// The clipboard format's version, the marker's value.
pub const VERSION: u64 = 1;

#[derive(Serialize)]
struct Out<'a> {
    #[serde(rename = "quadrule-clipboard")]
    marker: u64,
    objects: &'a [Object],
}

#[derive(Deserialize)]
struct In {
    #[serde(rename = "quadrule-clipboard")]
    marker: u64,
    objects: Vec<Object>,
}

/// The clipboard text for `ids`: their objects in stacking order, or `None` when none of the ids
/// is in the document.
pub fn to_text(doc: &Document, ids: &[Uuid]) -> Option<String> {
    let mut found: Vec<(usize, usize)> = ids.iter().filter_map(|&id| doc.find(id)).collect();
    found.sort_unstable();
    found.dedup();
    let objects: Vec<Object> = found
        .into_iter()
        .map(|(l, o)| doc.layers[l].objects[o].clone())
        .collect();
    if objects.is_empty() {
        return None;
    }
    serde_json::to_string(&Out {
        marker: VERSION,
        objects: &objects,
    })
    .ok()
}

/// The objects in clipboard `text`, or `None` when it is not quadrule's (no marker, a marker for
/// a newer format, broken JSON, or no objects).
pub fn from_text(text: &str) -> Option<Vec<Object>> {
    // Cheap test first: most clipboard text is not ours, and parsing a big paste is not free.
    if !text.trim_start().starts_with('{') || !text.contains(MARKER) {
        return None;
    }
    let read: In = serde_json::from_str(text).ok()?;
    (read.marker == VERSION && !read.objects.is_empty()).then_some(read.objects)
}
