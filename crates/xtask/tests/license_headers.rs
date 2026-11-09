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

//! Every Rust file under `crates/` starts with the Apache-2.0 header (LICENSE, appendix), so
//! a new file without it fails the build. `spikes/` is left as it was (rung 1 record, Q8).

use std::path::{Path, PathBuf};

const HEADER: &str = "// Copyright 2026 Chris Wells
//
// Licensed under the Apache License, Version 2.0 (the \"License\");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an \"AS IS\" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
";

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("read dir").flatten() {
        let p = e.path();
        if p.is_dir() {
            // Build folders hold generated files, not ours.
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn every_rust_file_has_the_license_header() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .to_path_buf();
    let mut files = Vec::new();
    rust_files(&crates, &mut files);
    // Windows checkouts may use CRLF if someone overrides .gitattributes; compare without it.
    let missing: Vec<String> = files
        .iter()
        .filter(|p| {
            !std::fs::read_to_string(p)
                .unwrap_or_default()
                .replace("\r\n", "\n")
                .starts_with(HEADER)
        })
        .map(|p| p.strip_prefix(&crates).unwrap_or(p).display().to_string())
        .collect();
    let ok = missing.is_empty() && files.len() > 10;
    println!(
        "CHECK crate=xtask name=license_headers result={} files={} missing={missing:?}",
        if ok { "pass" } else { "fail" },
        files.len()
    );
    assert!(ok);
}
