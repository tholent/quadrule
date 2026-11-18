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

//! Grand staves with a space of their own (docs/bbq/2026-11-18-fixes-before-16b.md, Q2; format 0.18): with Grand staff on
//! and `grand_space_units` set, the space from the bottom of one grand staff (its bass staff) to the top of the next is that
//! many rhythm units, so they never overlap and the space holds when the line gap changes; with the field absent the older
//! meaning, `pitch_units` top to top, is exactly what it was; a single staff ignores the field. Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn manta() -> Profile {
    builtin(MANTA_ID).unwrap()
}

/// A page holding one music staff region at (20, 20) mm, 100 by 80 mm, on a 5 mm rhythm (59 px on the Manta).
fn page_with(gap_mm: f64, pitch_units: u32, grand: bool, space: Option<u32>) -> Document {
    let mut doc = slice_document();
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = None;
    let mut o = doc.layers[0].objects[0].clone();
    o.frame = Frame::new(20.0, 20.0, 100.0, 80.0);
    o.constraints = Constraints::default();
    o.style = Style::Role(LineRole::Guide);
    o.kind = ObjectKind::Pattern(PatternRegion::new(Pattern::MusicStaff {
        gap_mm,
        pitch_units,
        grand,
        grand_space_units: space,
    }));
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects: vec![o],
        extra: Extra::new(),
    }];
    doc
}

fn items_of(doc: &Document) -> Vec<Item> {
    let m = manta();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &m, &m, &mut reg).unwrap();
    display_list(doc, &m, &m, &laid.outlines).unwrap().items
}

/// The centre of every staff line, top to bottom (the brace is a tall rectangle and is left out).
fn lines(doc: &Document) -> Vec<f64> {
    let mut ys: Vec<f64> = items_of(doc)
        .iter()
        .filter_map(|i| match *i {
            Item::Rect { x0, y0, x1, y1, .. } if x1 - x0 > y1 - y0 => Some((y0 + y1) / 2.0),
            _ => None,
        })
        .collect();
    ys.sort_by(f64::total_cmp);
    ys
}

/// For ten lines a grand staff: (top of each, bottom of each).
fn grands(ys: &[f64]) -> Vec<(f64, f64)> {
    ys.chunks(10)
        .filter(|c| c.len() == 10)
        .map(|c| (c[0], c[9]))
        .collect()
}

const UNIT: f64 = 59.0;

#[test]
fn the_space_is_from_the_bass_staff_bottom_to_the_next_top() {
    let mut parts = Vec::new();
    let mut ok = true;
    for space in 1..=3u32 {
        let g = grands(&lines(&page_with(2.0, 3, true, Some(space))));
        let gaps: Vec<f64> = g.windows(2).map(|w| w[1].0 - w[0].1).collect();
        let want = f64::from(space) * UNIT;
        let right = !gaps.is_empty() && gaps.iter().all(|d| (d - want).abs() <= 1.5);
        ok &= right && !g.is_empty();
        parts.push(format!(
            "space{space}:grands={}:gap_px={:?}",
            g.len(),
            gaps.iter().map(|d| d.round()).collect::<Vec<_>>()
        ));
    }
    println!(
        "\nCHECK crate=layout name=grand_space result={} unit_px=59 {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_space_holds_when_the_line_gap_changes_and_never_overlaps() {
    let mut parts = Vec::new();
    let mut ok = true;
    for gap in [1.0, 2.0, 3.0] {
        let g = grands(&lines(&page_with(gap, 3, true, Some(2))));
        let gaps: Vec<f64> = g.windows(2).map(|w| w[1].0 - w[0].1).collect();
        let right = g.len() >= 2 && gaps.iter().all(|d| (d - 2.0 * UNIT).abs() <= 1.5);
        let heights: Vec<f64> = g.iter().map(|b| (b.1 - b.0).round()).collect();
        ok &= right;
        parts.push(format!(
            "gap{gap}mm:grands={}:between_px={:?}:height_px={:?}",
            g.len(),
            gaps.iter().map(|d| d.round()).collect::<Vec<_>>(),
            &heights[..1]
        ));
    }
    // The smallest space, 1 unit, never overlaps even with a big line gap.
    let g = grands(&lines(&page_with(3.0, 3, true, Some(1))));
    let clear = g.windows(2).all(|w| w[1].0 > w[0].1);
    ok &= clear;
    println!(
        "\nCHECK crate=layout name=grand_space_holds result={} {} one_unit_big_gap_clear={clear}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn without_the_field_a_grand_staff_is_top_to_top_as_it_was() {
    // Pitch 6 units top to top: 354 px between tops, whatever the line gap, and the field's absence is what says so.
    let g = grands(&lines(&page_with(2.0, 6, true, None)));
    let tops: Vec<f64> = g.windows(2).map(|w| w[1].0 - w[0].0).collect();
    let top_to_top = !tops.is_empty() && tops.iter().all(|d| (d - 6.0 * UNIT).abs() <= 1.5);
    // The field does not matter with Grand staff off: a single staff draws the same with or without it.
    let single =
        lines(&page_with(2.0, 3, false, None)) == lines(&page_with(2.0, 3, false, Some(7)));
    // And a grand staff with the field differs from the one without it (the field is what changes the spacing).
    let differs = lines(&page_with(2.0, 6, true, Some(2))) != lines(&page_with(2.0, 6, true, None));
    let ok = top_to_top && single && differs;
    println!(
        "\nCHECK crate=layout name=grand_space_absent_is_top_to_top result={} tops_px={:?} single_ignores_field={single} field_changes_it={differs}",
        pass(ok),
        tops.iter().map(|d| d.round()).collect::<Vec<_>>()
    );
    assert!(ok);
}
