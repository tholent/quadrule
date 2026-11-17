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

//! The Lint tab's line per device about the checks that did not run (rung 14a, docs/bbq/2026-11-17-rung-14-
//! lint-and-preflight.md, Q2) and the lint page (`--lint-page`). Prints one structured `CHECK` line per test
//! (run with `-- --nocapture`).

use quadrule_ui::{Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

#[test]
fn the_lint_tab_says_per_device_which_checks_did_not_run_and_why() {
    // The lint page on two Supernotes: every check ran.
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_lint();
    let lines = ui.lint_skipped_lines();
    let full = lines.len() == 2
        && lines
            .iter()
            .all(|l| l.starts_with("No checks were skipped on "));
    // The paper page: A4 and A5 skip the screen checks, and say so in plain words.
    let mut paper = ViewerUi::new(Viewer::default());
    paper.viewer.open_paper();
    let lines = paper.lint_skipped_lines();
    let says_paper = lines.len() == 2
        && lines.iter().all(|l| {
            l.contains("L03 (paper has no screen to judge against)")
                && l.contains("L14 (paper has no screen to judge against)")
                && !l.contains("L07")
        });
    // Nothing open: nothing said.
    let none = ViewerUi::new(Viewer::default())
        .lint_skipped_lines()
        .is_empty();
    let ok = full && says_paper && none;
    println!(
        "CHECK crate=ui name=lint_tab_skipped_lines result={} supernotes_none_skipped={full} paper_says_why={says_paper} nothing_open_nothing_said={none}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_lint_page_opens_and_the_tab_lists_its_findings() {
    let mut v = Viewer::default();
    v.open_lint();
    let page = v.page().map(|p| {
        let rules: std::collections::BTreeSet<&str> = p.findings.iter().map(|f| f.rule).collect();
        (p.findings.len(), rules)
    });
    let ok = page.as_ref().is_some_and(|(n, rules)| {
        *n >= 8
            && ["L03", "L05", "L07", "L09", "L13", "L14"]
                .iter()
                .all(|r| rules.contains(r))
    });
    println!(
        "CHECK crate=ui name=lint_page_opens result={} findings={:?}",
        pass(ok),
        page
    );
    assert!(ok);
}
