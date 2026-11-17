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

//! Lint on a paper size (rung 13b; napkin §3.3.1, rung 13 record Defaulted "Paper lint"): the rules that read
//! safe zones, gray levels or pen grays (L03 to L06) are skipped on paper, whatever the profile holds, and the
//! rest run as on a device. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, SafeZone, builtin_at};
use quadrule_text::FontRegistry;
use std::collections::BTreeSet;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn run(doc: &Document, target: &Profile) -> Vec<Finding> {
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, target, target, &mut reg).unwrap();
    lint(doc, target, target, &laid.outlines)
}

fn rules(f: &[Finding]) -> BTreeSet<&'static str> {
    f.iter().map(|x| x.rule).collect()
}

/// A page on A4 with a thin line, a line half off the page and a header box where a toolbar would be.
fn page() -> Document {
    let mut doc = paper_document();
    doc.profiles.truncate(1);
    let mk = |n: u128, frame: Frame, style: Style| {
        let mut o = doc.layers[1].objects[0].clone();
        o.id = Uuid::from_u128(0x7000 + n);
        o.name = format!("probe {n}");
        o.frame = frame;
        o.style = style;
        o.kind = ObjectKind::Line(Line::default());
        o.constraints = Constraints::default();
        o.fill = None;
        o
    };
    let thin = mk(
        1,
        Frame::new(20.0, 120.0, 50.0, 0.0),
        Style::Custom {
            width_mm: 0.02,
            gray: 0,
        },
    );
    let off = mk(
        2,
        Frame::new(190.0, 130.0, 60.0, 0.0),
        Style::Role(LineRole::Guide),
    );
    let in_corner = mk(
        3,
        Frame::new(0.5, 0.5, 30.0, 0.0),
        Style::Role(LineRole::Guide),
    );
    doc.layers[1].objects.extend([thin, off, in_corner]);
    doc
}

#[test]
fn device_only_rules_are_skipped_on_paper_and_the_others_still_run() {
    let doc = page();
    let a4 = builtin_at("paper.a4", Orientation::Portrait, 300).unwrap();
    let on_paper = run(&doc, &a4);
    let paper_rules = rules(&on_paper);
    let skipped = SKIPPED_ON_PAPER.iter().all(|r| !paper_rules.contains(r));
    // The same profile held as a screen with toolbar zones and gray data (what a hand-edited file
    // could claim) is judged on its zones: the skip is by kind, not by luck.
    let mut screen = a4.clone();
    screen.paper = None;
    screen.safe_zones = vec![SafeZone {
        name: "toolbar_top".into(),
        orientation: Orientation::Portrait,
        rect: (0, 0, 2480, 120),
        advisory: false,
    }];
    let as_screen = run(&doc, &screen);
    let screen_hits_l06 = rules(&as_screen).contains("L06");
    let mut paper_with_zone = a4.clone();
    paper_with_zone.safe_zones = screen.safe_zones.clone();
    let zone_ignored = !rules(&run(&doc, &paper_with_zone)).contains("L06");
    // L01 (thinner than a pixel) and L10 (off the page) run on paper.
    let l01 = on_paper.iter().any(|f| f.rule == "L01");
    let l10 = on_paper.iter().any(|f| f.rule == "L10");
    let ok = skipped && screen_hits_l06 && zone_ignored && l01 && l10;
    println!(
        "CHECK crate=lint name=paper_skips_device_rules result={} skipped={skipped} same_page_as_screen_has_l06={screen_hits_l06} zone_on_paper_ignored={zone_ignored} l01_runs={l01} l10_runs={l10} rules_on_paper={paper_rules:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_skipped_list_is_l03_to_l06() {
    let ok = SKIPPED_ON_PAPER == ["L03", "L04", "L05", "L06"];
    println!("CHECK crate=lint name=skipped_on_paper result={}", pass(ok));
    assert!(ok);
}
