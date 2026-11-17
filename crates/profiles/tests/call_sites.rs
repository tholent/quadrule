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

//! The one lookup (rung 13a bbq, Defaulted "One registry"): production code reads a profile through
//! `quadrule_profiles::active`, never `builtin` or `builtin_for`, which give the shipped values only.
//! The calibration and safe-zone tools in `xtask` work on the shipped values on purpose.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn production_code_looks_profiles_up_in_the_registry() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for c in std::fs::read_dir(&crates).unwrap().flatten() {
        rust_files(&c.path().join("src"), &mut files);
    }
    let mut stray = Vec::new();
    let mut scanned = 0;
    for f in files {
        let rel = f
            .strip_prefix(&crates)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        // The profiles crate defines them; xtask's calibration sheets and safe-zone tool use the
        // shipped values by design.
        if rel.starts_with("profiles/") || rel.starts_with("xtask/") {
            continue;
        }
        scanned += 1;
        let text = std::fs::read_to_string(&f).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            // A call to one of the three, not a longer name that ends the same way.
            let calls = |name: &str| {
                code.match_indices(name)
                    .any(|(i, _)| !code[..i].ends_with(|c: char| c.is_alphanumeric() || c == '_'))
            };
            // `builtin_ids` lists the shipped devices, which a new page targets.
            if calls("builtin_for(") || calls("builtin(") {
                stray.push(format!("{rel}:{}", n + 1));
            }
        }
    }
    let ok = stray.is_empty() && scanned > 20;
    println!(
        "CHECK crate=profiles name=one_lookup result={} files_scanned={scanned} stray={stray:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
