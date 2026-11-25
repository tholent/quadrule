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

//! The speed reference's logic (rung 17, Q2) without a window: reading a bench's `CHECK` line, judging a number
//! against the 50 ms line and the recorded one, the recorded file, and the fixed set of benches. Each test prints
//! one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_xtask::bench::{
    HEAVIEST_STARTER, LIMIT_MS, Status, benches, change_pct, judge, parse_check, parse_recorded,
    write_recorded,
};
use std::collections::BTreeMap;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const LINE: &str = "\nCHECK crate=ui name=drag_bench result=pass build=release canvas=export timed=canvas page=isometric device=primary profile=supernote.a6x2 strip=on tool=move frames=143 mean_ms=1.5 p90_ms=1.7 worst_ms=2.3 first_ms=2.7 over_budget=0 budget_ms=50";

#[test]
fn a_bench_line_is_read_into_its_fields() {
    let f = parse_check(LINE).unwrap();
    let ok = f["p90_ms"] == "1.7"
        && f["profile"] == "supernote.a6x2"
        && f["canvas"] == "export"
        && f["build"] == "release"
        && parse_check("CHECK crate=ui name=other result=pass").is_none()
        && parse_check("not a check line p90_ms=1").is_none()
        && parse_check("CHECK crate=ui name=drag_bench result=fail reason=no_target page=x")
            .is_none();
    println!(
        "CHECK crate=xtask name=bench_line_parsed result={}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_number_is_judged_against_the_line_and_its_record() {
    let cases = [
        // (p90, recorded, has a record, expected)
        (60.0, Some(40.0), true, Status::Over),
        (49.9, None, false, Status::Ok),
        (49.9, Some(49.0), true, Status::Ok),
        (12.1, Some(10.0), true, Status::Slower),
        (11.9, Some(10.0), true, Status::Ok),
        // Over 20 % of 1.7 ms is 0.4 ms: noise, not slower.
        (2.2, Some(1.7), true, Status::Ok),
        (20.0, None, true, Status::Unrecorded),
        (5.0, Some(8.0), true, Status::Ok),
    ];
    let wrong: Vec<_> = cases
        .iter()
        .filter(|(p, r, h, want)| judge(*p, *r, *h) != *want)
        .collect();
    let ok = wrong.is_empty() && LIMIT_MS == 50.0 && change_pct(12.0, 10.0) == "+20";
    println!(
        "CHECK crate=xtask name=bench_judged result={} wrong={}",
        pass(ok),
        wrong.len()
    );
    assert!(ok);
}

#[test]
fn the_recorded_file_reads_back_what_was_written() {
    let mut numbers = BTreeMap::new();
    numbers.insert("drag-a".to_string(), 1.7);
    numbers.insert("zoom-b".to_string(), 12.34);
    let text = write_recorded("gibson", "2026-11-24", &numbers);
    let back = parse_recorded(&text);
    let ok = back.len() == 2
        && (back["drag-a"] - 1.7).abs() < 1e-9
        && (back["zoom-b"] - 12.3).abs() < 1e-9
        && text.starts_with("# Recorded bench numbers for `gibson`");
    println!(
        "CHECK crate=xtask name=bench_record_roundtrip result={}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_fixed_set_covers_what_the_rung_asks() {
    let set = benches(HEAVIEST_STARTER, HEAVIEST_STARTER);
    let names: Vec<&str> = set.iter().map(|b| b.name.as_str()).collect();
    let unique = {
        let mut n = names.clone();
        n.sort_unstable();
        n.dedup();
        n.len() == names.len()
    };
    let has = |part: &str| names.iter().any(|n| n.contains(part));
    // Drag and draw on the starter on both tablets and both canvases; the paper page; A3 and Tabloid at 600 PPI on
    // both canvases; typing; zoom; the Basic tab.
    let starter = ["nomad", "manta"].iter().all(|d| {
        ["export", "chip"]
            .iter()
            .all(|c| has(&format!("drag-starter-{d}-{c}")) && has(&format!("draw-starter-{d}-{c}")))
    });
    let paper = has("paper-a4-300-export") && has("paper-a4-300-chip");
    let large = ["a3", "tabloid"]
        .iter()
        .all(|p| has(&format!("{p}-600-export")) && has(&format!("{p}-600-chip")));
    let rest = has("type-") && has("zoom-sample") && has("basic-tab");
    let all_release_canvas = set
        .iter()
        .all(|b| b.args.windows(2).any(|w| w == ["--canvas", b.canvas]));
    let ok = unique && starter && paper && large && rest && all_release_canvas;
    println!(
        "CHECK crate=xtask name=bench_set result={} benches={} starter_both_tablets_both_canvases={starter} paper={paper} large_paper={large} type_zoom_basic={rest}",
        pass(ok),
        set.len()
    );
    assert!(ok);
}
