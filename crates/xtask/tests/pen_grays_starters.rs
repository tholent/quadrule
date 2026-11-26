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

//! ADR-027 with `--pen-grays` on: every starter still lints clean on both tablets (ADR-021, item 1), so nothing has
//! to change in a starter when the flag comes out. This file is its own test program, so the switch is on for it
//! alone. Prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::QuadFile;
use quadrule_xtask::{lintrun, starters};

#[test]
fn every_starter_lints_clean_on_the_pen_scale() {
    quadrule_model::pen_grays::set(true);
    let opts = lintrun::options(lintrun::DEFAULT_TOOLBARS).unwrap();
    let mut findings = Vec::new();
    let mut checked = 0;
    for (rel, bytes) in starters::files().unwrap() {
        let f = QuadFile::from_bytes(&bytes).unwrap();
        let lines = lintrun::lint_lines(&f.doc, &f.outlines, &opts).unwrap();
        checked += 1;
        findings.extend(lines.into_iter().map(|l| format!("{}: {l}", rel.display())));
    }
    let ok = findings.is_empty() && checked == 20;
    println!(
        "CHECK crate=xtask name=pen_grays_starters_lint_clean result={} starters={checked} tablets=2 findings={} first=[{}]",
        if ok { "pass" } else { "fail" },
        findings.len(),
        findings.first().cloned().unwrap_or_default()
    );
    assert!(ok, "{findings:#?}");
}
