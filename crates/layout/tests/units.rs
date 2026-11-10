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

//! Typed lengths (rung 3 bbq Q10) and object shapes for picking. Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::units::*;
use quadrule_layout::{Shape, lay_out_text, placed_objects};
use quadrule_model::sample::*;
use quadrule_model::{MM_PER_IN, Unit};
use quadrule_profiles::builtin;
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn every_unit_parses() {
    let doc = slice_document();
    let manta = builtin(MANTA_ID).unwrap();
    let x = UnitContext::new(&doc, &manta, &manta, Axis::X);
    let y = UnitContext::new(&doc, &manta, &manta, Axis::Y);
    // The slice's 6 mm rhythm is pixel-locked: 71 px on the Manta (300 PPI).
    let unit_mm = 71.0 * MM_PER_IN / 300.0;
    let cases: Vec<(&str, &UnitContext, f64)> = vec![
        ("5mm", &x, 5.0),
        (" 5 MM ", &x, 5.0),
        ("12pt", &x, 12.0 * 25.4 / 72.0),
        ("40px", &x, 40.0 * 25.4 / 300.0),
        ("0.25in", &x, 6.35),
        ("50%", &x, MANTA_W_MM / 2.0),
        ("50%", &y, MANTA_H_MM / 2.0),
        ("2u", &x, 2.0 * unit_mm),
        ("-3", &x, -3.0),
        ("+1.5mm", &x, 1.5),
        ("1,5", &x, 1.5),
        (".5in", &x, 12.7),
    ];
    let wrong: Vec<String> = cases
        .iter()
        .filter(|(s, c, want)| !parse_length(s, c).is_ok_and(|v| close(v, *want)))
        .map(|(s, c, _)| format!("{s}->{:?}", parse_length(s, c)))
        .collect();
    let errors = parse_length("", &x) == Err(UnitError::Empty)
        && parse_length("abc", &x) == Err(UnitError::BadNumber(String::new()))
        && parse_length("5 furlongs", &x) == Err(UnitError::UnknownUnit("furlongs".into()))
        && parse_length("1.2.3mm", &x).is_err();
    // A bare number is in the display unit.
    let mut pt = x;
    pt.display = Unit::Pt;
    let bare = parse_length("72", &pt).is_ok_and(|v| close(v, 25.4));
    let ok = wrong.is_empty() && errors && bare;
    println!(
        "CHECK crate=layout name=every_unit_parses result={} cases={} wrong={wrong:?} errors_reported={errors} bare_number_in_display_unit={bare}",
        pass(ok),
        cases.len()
    );
    assert!(ok);
}

#[test]
fn lengths_show_in_the_display_unit() {
    let doc = slice_document();
    let manta = builtin(MANTA_ID).unwrap();
    let mut c = UnitContext::new(&doc, &manta, &manta, Axis::X);
    let mut shown = Vec::new();
    for (unit, mm) in [
        (Unit::Mm, 12.5),
        (Unit::Mm, 8.0),
        (Unit::Px, 25.4),
        (Unit::Pt, 25.4),
        (Unit::In, 12.5),
        (Unit::Mm, -0.0001),
    ] {
        c.display = unit;
        shown.push(format_length(mm, &c));
    }
    let want = ["12.5 mm", "8 mm", "300 px", "72 pt", "0.492 in", "0 mm"];
    // What is shown reads back to the same value, to the places shown.
    c.display = Unit::Mm;
    let round_trip = parse_length(&format_length(8.0, &c), &c) == Ok(8.0);
    let ok = shown == want && round_trip;
    println!(
        "CHECK crate=layout name=lengths_shown_in_display_unit result={} shown={shown:?} round_trip={round_trip}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn shapes_pick_what_is_drawn() {
    let doc = slice_document();
    let manta = builtin(MANTA_ID).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(&doc, &manta, &manta, &mut reg).unwrap();
    let placed = placed_objects(&doc, &manta, &manta, &laid.outlines);
    let find = |id| placed.iter().find(|p| p.id == id).unwrap().shape;
    let px = |mm: f64| mm * 300.0 / MM_PER_IN;
    // The rule under the title runs at y = 22 mm, 2 px wide (Structure on the Manta).
    let rule = find(ids::RULE);
    let on_rule = rule.hit(px(50.0), px(22.0), 0.0);
    let near_rule =
        !rule.hit(px(50.0), px(22.0) + 4.0, 0.0) && rule.hit(px(50.0), px(22.0) + 4.0, 4.0);
    // The dot grid picks anywhere in its region, between dots too.
    let dots = find(ids::DOTS);
    let in_dots = dots.hit(px(30.0), px(60.0), 0.0) && !dots.hit(px(30.0), px(120.0), 0.0);
    // The title is as tall as its laid-out text, not its 11 mm frame.
    let title = find(ids::TITLE);
    let text_h = laid.outlines.text(ids::TITLE).unwrap().height_mm as f64;
    let tall = matches!(title, Shape::Box { y0, y1, .. } if (y1 - y0 - px(text_h)).abs() < 1e-6);
    let order = placed.iter().map(|p| p.id).collect::<Vec<_>>()
        == vec![ids::DOTS, ids::LINES, ids::TITLE, ids::RULE];
    let ok = on_rule && near_rule && in_dots && tall && order;
    println!(
        "CHECK crate=layout name=shapes_pick_what_is_drawn result={} on_line={on_rule} slack={near_rule} pattern_region={in_dots} text_height={tall} drawing_order={order}",
        pass(ok)
    );
    assert!(ok);
}
