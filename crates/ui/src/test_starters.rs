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

//! Test starters (rung 16 bbq, Defaulted "Where starters live" and "Speed"): starter pages made from the sample
//! documents the benchmarks already use, so the gallery can be tested and timed with a full grid before chris has
//! made any real ones. They are made in memory, never written to the repository.

use crate::viewer::Viewer;

/// The five category folders, in tab order.
const FOLDERS: [&str; 5] = ["paper", "notes", "planning", "bullet-journal", "creative"];

/// How to open each sample document, with a name for it.
type Maker = (&'static str, fn(&mut Viewer));

const MAKERS: [Maker; 16] = [
    ("Sample", Viewer::open_sample),
    ("Shapes", Viewer::open_shapes),
    ("Transforms", Viewer::open_transforms),
    ("Groups", Viewer::open_groups),
    ("Devices", Viewer::open_devices),
    ("Landscape", Viewer::open_landscape),
    ("Patterns", Viewer::open_patterns),
    ("Turned patterns", Viewer::open_turned_patterns),
    ("Pattern angle", Viewer::open_pattern_angle),
    ("Lint", Viewer::open_lint),
    ("Paper", Viewer::open_paper),
    ("Repeats", Viewer::open_repeats),
    ("Fonts", Viewer::open_fonts),
    ("Text box", Viewer::open_textbox),
    ("Paper landscape", Viewer::open_paper_landscape),
    ("Patterns landscape", Viewer::open_patterns_landscape),
];

/// `count` starters as (category folder, slug, `.quad` bytes), cycling through the sample documents and the five
/// categories. Each is titled `Test starter NN: <document>`; the heavy ones (the groups page, A4 paper at 600 PPI)
/// are in the first sixteen.
pub fn test_starters(count: usize) -> Vec<(String, String, Vec<u8>)> {
    let mut out = Vec::new();
    for i in 0..count {
        let (name, make) = MAKERS[i % MAKERS.len()];
        let mut v = Viewer::default();
        make(&mut v);
        let Some(o) = v.open.as_mut() else {
            continue;
        };
        o.doc.metadata.title = format!("Test starter {:02}: {name}", i + 1);
        o.doc.metadata.description = format!("Made from the {name} sample page for tests.");
        o.doc.metadata.tags = vec!["test".into(), name.to_lowercase()];
        let Ok(bytes) = v.save_bytes() else {
            continue;
        };
        out.push((
            FOLDERS[i % FOLDERS.len()].to_string(),
            format!("test-{:02}", i + 1),
            bytes,
        ));
    }
    out
}
