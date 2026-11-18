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

//! Builds the list of starter pages into the program (rung 16 bbq, Defaulted "Where starters live"): every
//! `starters/<category>/<slug>.quad` in this crate becomes one entry of `BUILT_IN` in `starters.rs`, with its bytes
//! included. The folder names the category, so the file format does not change. With no starters (rung 16a ships
//! none) the list is empty.

use std::path::{Path, PathBuf};

fn main() {
    let root =
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets it")).join("starters");
    println!("cargo:rerun-if-changed=starters");
    let mut found: Vec<(String, String, PathBuf)> = Vec::new();
    for category in std::fs::read_dir(&root).into_iter().flatten().flatten() {
        let dir = category.path();
        if !dir.is_dir() {
            continue;
        }
        let folder = category.file_name().to_string_lossy().into_owned();
        for file in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = file.path();
            if path.extension().is_some_and(|x| x == "quad")
                && let Some(slug) = path.file_stem()
            {
                println!("cargo:rerun-if-changed={}", path.display());
                found.push((folder.clone(), slug.to_string_lossy().into_owned(), path));
            }
        }
    }
    found.sort();
    let mut out = String::from(
        "/// Every starter page built in: (category folder, slug, the `.quad` file's bytes).\n",
    );
    out.push_str("pub const BUILT_IN: &[(&str, &str, &[u8])] = &[\n");
    for (folder, slug, path) in &found {
        out.push_str(&format!(
            "    ({folder:?}, {slug:?}, include_bytes!({:?})),\n",
            path.display().to_string()
        ));
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR").expect("cargo sets it")).join("starters.rs");
    std::fs::write(target, out).expect("write the starters list");
}
