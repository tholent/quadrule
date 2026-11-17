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

//! The lint block in a profile (rung 14a bbq, Q2, Q3 and Defaulted "Paper profiles' lint blocks"): the
//! numbers the linter judges a page against. Both Supernote profiles carry the measured values, a paper
//! size only the text size and the handwriting step; a file written before the block existed still
//! loads, with every value empty (an override of a built-in then keeps the shipped ones); and the block
//! travels through RON text unchanged. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_profiles::*;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-lintblock-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn both_supernotes_carry_the_measured_values_and_paper_only_two() {
    let want = LintThresholds {
        lightest_gray_line_1px: Some(187),
        lightest_gray_line_2px_up: Some(204),
        lightest_gray_dot_2px: Some(102),
        lightest_gray_dot_3px: Some(170),
        lightest_gray_dot_4px_up: Some(187),
        lightest_gray_fill: Some(204),
        lightest_gray_text: Some(153),
        smallest_text_pt: Some(6.0),
        smallest_handwriting_step_mm: Some(5.0),
        dark_fill_gray: Some(102),
        dark_fill_page_share: Some(0.1),
        thin_line_px: Some(2),
    };
    let nomad = builtin("supernote.a6x2").unwrap().lint == want;
    let manta = builtin("supernote.a5x2").unwrap().lint == want;
    let paper_want = LintThresholds {
        smallest_text_pt: Some(6.0),
        smallest_handwriting_step_mm: Some(5.0),
        ..LintThresholds::default()
    };
    let papers: Vec<&str> = builtin_paper_ids().collect();
    let paper = !papers.is_empty()
        && papers
            .iter()
            .all(|id| builtin(id).unwrap().lint == paper_want);
    let dots = want.lightest_gray_dot(1) == Some(102)
        && want.lightest_gray_dot(2) == Some(102)
        && want.lightest_gray_dot(3) == Some(170)
        && want.lightest_gray_dot(4) == Some(187)
        && want.lightest_gray_dot(9) == Some(187);
    let lines = want.lightest_gray_line(1) == Some(187) && want.lightest_gray_line(5) == Some(204);
    let ok = nomad && manta && paper && dots && lines;
    println!(
        "CHECK crate=profiles name=lint_block_shipped result={} nomad={nomad} manta={manta} paper_sizes={} paper_only_two={paper} dot_sizes={dots} line_widths={lines}",
        pass(ok),
        papers.len()
    );
    assert!(ok);
}

#[test]
fn a_file_without_the_block_loads_empty_and_the_block_round_trips() {
    let nomad = builtin("supernote.a6x2").unwrap();
    // Cut the block out of the RON text a build before 14a wrote.
    let text = to_ron(&nomad);
    let start = text.find("lint: (").expect("the block is written");
    let end = start + text[start..].find("\n    ),").expect("the block ends") + "\n    ),".len();
    let old = format!("{}{}", &text[..start], &text[end..]);
    let loaded = from_ron_checked(&old);
    let empty = loaded.as_ref().is_ok_and(|p| p.lint.is_empty());
    let back = from_ron_checked(&to_ron(&nomad)).is_ok_and(|p| p == nomad);
    // An empty block is still written, so "no block" and "emptied on purpose" stay apart.
    let mut bare = nomad.clone();
    bare.lint = LintThresholds::default();
    let unwritten = to_ron(&bare).contains("lint:")
        && from_ron_checked(&to_ron(&bare)).is_ok_and(|p| p == bare);
    // A value outside its limit is refused.
    let mut bad = nomad.clone();
    bad.lint.dark_fill_page_share = Some(1.5);
    let refused = from_ron_checked(&to_ron(&bad)).is_err();
    let mut bad = nomad.clone();
    bad.lint.smallest_text_pt = Some(f64::NAN);
    let refused_nan = parse_checked(&to_ron(&bad)).is_err();
    let ok = empty && back && unwritten && refused && refused_nan;
    println!(
        "CHECK crate=profiles name=lint_block_file result={} old_file_loads_empty={empty} round_trip={back} empty_block_written={unwritten} share_over_1_refused={refused} nan_refused={refused_nan}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_override_saved_before_the_block_keeps_the_shipped_values_until_edited() {
    let d = folder("override");
    // What a 0.13 build wrote for an edited Nomad: the whole profile, no lint block.
    let mut edited = builtin("supernote.a6x2").unwrap();
    edited.line_roles.guide.gray = 140;
    let text = to_ron(&edited);
    let start = text.find("lint: (").unwrap();
    let end = start + text[start..].find("\n    ),").unwrap() + "\n    ),".len();
    let old = format!("{}{}", &text[..start], &text[end..]);
    std::fs::write(d.join("supernote.a6x2.ron"), &old).unwrap();
    let r = Registry::load(&d);
    let p = r.get("supernote.a6x2").unwrap();
    let kept = p.lint == builtin("supernote.a6x2").unwrap().lint && p.line_roles.guide.gray == 140;
    // A block the person emptied on purpose stays empty.
    let mut cleared = edited.clone();
    cleared.lint = LintThresholds::default();
    let mut r2 = Registry::at(&d);
    r2.save(cleared).unwrap();
    let r3 = Registry::load(&d);
    let first = r3.get("supernote.a6x2").unwrap().lint.is_empty();
    let ok = kept && first;
    println!(
        "CHECK crate=profiles name=lint_block_override result={} old_override_keeps_shipped={kept} emptied_on_purpose_stays_empty={first}",
        pass(ok)
    );
    assert!(ok);
}
