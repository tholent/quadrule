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

//! The rules chris's first starters showed to misfire (record `docs/bbq/2026-11-23-adr-starter-lint.md`): the Full bleed
//! mark and L10's edge (ADR-019 items 1 and 2), page edges on the rhythm (item 3), L14 and L09 on drawing grids (ADR-021
//! items 2 and 3), and L18's negative zero. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn run(doc: &Document, target: &Profile) -> Vec<Finding> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let laid = lay_out_text(doc, &primary, target, &mut FontRegistry::bundled_only()).unwrap();
    lint(doc, &primary, target, &laid.outlines)
}

/// Whether `rule` is found on the object `id`.
fn has(f: &[Finding], rule: &str, id: u128) -> bool {
    f.iter()
        .any(|x| x.rule == rule && x.object == Some(Uuid::from_u128(id)))
}

fn object(id: u128, frame: Frame, style: Style, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(id),
        name: format!("object {id}"),
        frame,
        constraints: Constraints::default(),
        visible: true,
        locked: false,
        style,
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        free_placement: false,
        full_bleed: false,
        kind,
        extra: Extra::new(),
    }
}

fn line(id: u128, frame: Frame, role: LineRole) -> Object {
    object(
        id,
        frame,
        Style::Role(role),
        ObjectKind::Line(Line::default()),
    )
}

fn region(id: u128, frame: Frame, pattern: Pattern) -> Object {
    object(
        id,
        frame,
        Style::Role(LineRole::Guide),
        ObjectKind::Pattern(PatternRegion::new(pattern)),
    )
}

/// The slice page (Manta first, a Template, 6 mm rhythm from 8 mm margins) holding only `objects`.
fn page(objects: Vec<Object>) -> Document {
    let mut doc = slice_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc
}

fn whole_page() -> Frame {
    Frame::new(0.0, 0.0, MANTA_W_MM, MANTA_H_MM)
}

#[test]
fn full_bleed_is_left_alone_by_l06_and_l10_and_the_rest_is_still_checked() {
    let manta = builtin(MANTA_ID).unwrap();
    // A ruled region over the whole page and 5 mm past its right edge; a line under the top toolbar beside it.
    let past = Frame::new(0.0, 0.0, MANTA_W_MM + 5.0, MANTA_H_MM);
    let under_toolbar = Frame::new(30.0, 2.0, 40.0, 0.0);
    let doc = |marked: bool| {
        let mut r = region(1, past, Pattern::Lines);
        r.full_bleed = marked;
        page(vec![r, line(2, under_toolbar, LineRole::Guide)])
    };
    let (plain, marked) = (run(&doc(false), &manta), run(&doc(true), &manta));
    let plain_caught = has(&plain, "L06", 1) && has(&plain, "L10", 1);
    let marked_left = !has(&marked, "L06", 1) && !has(&marked, "L10", 1);
    let neighbour_still = has(&marked, "L06", 2);
    // A group's mark covers the line inside it.
    let grouped = |marked: bool| {
        let mut g = object(
            3,
            under_toolbar,
            Style::Role(LineRole::Guide),
            ObjectKind::Group(GroupShape {
                children: vec![line(4, under_toolbar, LineRole::Guide)],
                extra: Extra::new(),
            }),
        );
        g.full_bleed = marked;
        run(&page(vec![g]), &manta)
    };
    let group_plain = has(&grouped(false), "L06", 4);
    let group_marked = !has(&grouped(true), "L06", 4);
    // A repeat's mark covers every copy: the repeats page marked whole finds no L06 or L10.
    let mut reps = repeats_document();
    let before = run(&reps, &manta)
        .iter()
        .filter(|f| f.rule == "L06" || f.rule == "L10")
        .count();
    for l in &mut reps.layers {
        for o in &mut l.objects {
            o.full_bleed = true;
        }
    }
    let after = run(&reps, &manta)
        .iter()
        .filter(|f| f.rule == "L06" || f.rule == "L10")
        .count();
    let ok = plain_caught
        && marked_left
        && neighbour_still
        && group_plain
        && group_marked
        && before > 0
        && after == 0;
    println!(
        "\nCHECK crate=lint name=full_bleed result={} unmarked_caught={plain_caught} marked_left={marked_left} unmarked_neighbour_caught={neighbour_still} group_unmarked_caught={group_plain} group_mark_covers_child={group_marked} repeats_l06_l10_before={before} after={after}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l10_judges_the_frame_not_half_a_stroke_on_an_edge() {
    let manta = builtin(MANTA_ID).unwrap();
    // A 3 px Emphasis line edge to edge across the page, and the same line starting 5 mm off the left edge.
    let across = Frame::new(0.0, 100.0, MANTA_W_MM, 0.0);
    let past = Frame::new(-5.0, 100.0, MANTA_W_MM, 0.0);
    // A rectangle drawn round the whole page, its stroke half outside.
    let rect = object(
        3,
        whole_page(),
        Style::Role(LineRole::Emphasis),
        ObjectKind::Rect(RectShape::default()),
    );
    let f = run(
        &page(vec![
            line(1, across, LineRole::Emphasis),
            line(2, past, LineRole::Emphasis),
            rect,
        ]),
        &manta,
    );
    let (edge_line, edge_rect, off) = (has(&f, "L10", 1), has(&f, "L10", 3), has(&f, "L10", 2));
    let ok = !edge_line && !edge_rect && off;
    println!(
        "\nCHECK crate=lint name=l10_frame_not_stroke result={} edge_to_edge_line_clear={} page_frame_rect_clear={} line_past_edge_caught={off}",
        pass(ok),
        !edge_line,
        !edge_rect
    );
    assert!(ok);
}

#[test]
fn l16_counts_a_page_edge_as_on_the_rhythm_on_that_axis() {
    let manta = builtin(MANTA_ID).unwrap();
    let h = MANTA_H_MM;
    // The rhythm is 6 mm from the 8 mm margin: 26 mm is on it across, 27.3 mm is not, and 0 mm down is not.
    let on_x = line(1, Frame::new(26.0, 0.0, 0.0, h), LineRole::Guide);
    let off_x = line(2, Frame::new(27.3, 0.0, 0.0, h), LineRole::Guide);
    // A box flush with the right edge, off the rhythm across, on it down (8 + 6 × 5 = 38 mm).
    let flush_right = object(
        3,
        Frame::new(MANTA_W_MM - 33.3, 38.0, 33.3, 12.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Rect(RectShape::default()),
    );
    // Off on both axes and touching no edge.
    let loose = line(4, Frame::new(27.3, 40.3, 20.0, 0.0), LineRole::Guide);
    let f = run(&page(vec![on_x, off_x, flush_right, loose]), &manta);
    let msg = |id: u128| {
        f.iter()
            .find(|x| x.rule == "L16" && x.object == Some(Uuid::from_u128(id)))
            .map(|x| quadrule_i18n::Strings::english().format(&x.message))
    };
    let top_edge_ok = msg(1).is_none();
    let across_only = msg(2).is_some_and(|m| m.contains("(across)"));
    let right_edge_ok = msg(3).is_none();
    let loose_both = msg(4).is_some_and(|m| m.contains("across and down"));
    let ok = top_edge_ok && across_only && right_edge_ok && loose_both;
    println!(
        "\nCHECK crate=lint name=l16_page_edges result={} full_height_on_rhythm_clear={top_edge_ok} full_height_off_rhythm_across_only={across_only} flush_right_clear={right_edge_ok} loose_caught_both={loose_both}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l14_and_l09_leave_drawing_grids_alone() {
    let (manta, nomad) = (builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap());
    let frame = Frame::new(20.0, 30.0, 100.0, 100.0);
    let hex = region(
        1,
        frame,
        Pattern::Hex {
            orientation: HexOrientation::Flat,
            height_units: 1,
        },
    );
    let iso = region(
        2,
        frame,
        Pattern::Isometric {
            style: IsoStyle::Lines,
        },
    );
    // A slanted 1 px line is still soft.
    let slanted = line(3, Frame::new(20.0, 140.0, 60.0, 30.0), LineRole::Guide);
    let mut l14_hex = false;
    let mut l14_iso = false;
    let mut l14_line = true;
    for p in [&manta, &nomad] {
        l14_hex |= has(&run(&page(vec![hex.clone()]), p), "L14", 1);
        l14_iso |= has(&run(&page(vec![iso.clone()]), p), "L14", 2);
        l14_line &= has(&run(&page(vec![slanted.clone()]), p), "L14", 3);
    }
    // L09: an isometric rhythm of 6 mm sides steps 3 mm down the page; a rectangular 3 mm rhythm does too.
    let l09 = |isometric: bool| {
        let mut doc = page(vec![]);
        if isometric {
            doc.rhythm.rhythm_type = RhythmType::Isometric;
            doc.rhythm.side_mm = Some(6.0);
        } else {
            doc.rhythm.v_unit_mm = 3.0;
        }
        run(&doc, &nomad).iter().any(|f| f.rule == "L09")
    };
    let (l09_iso, l09_rect) = (l09(true), l09(false));
    let ok = !l14_hex && !l14_iso && l14_line && !l09_iso && l09_rect;
    println!(
        "\nCHECK crate=lint name=drawing_grids result={} l14_hex_clear={} l14_isometric_clear={} l14_slanted_line_caught={l14_line} l09_isometric_rhythm_clear={} l09_rectangular_3mm_caught={l09_rect}",
        pass(ok),
        !l14_hex,
        !l14_iso,
        !l09_iso
    );
    assert!(ok);
}

#[test]
fn l18_never_says_minus_zero() {
    // The bullet journal's first upload: a 6.35 mm rhythm from the page corner, 12.7 mm margins, so the left margin
    // is exactly two units from the origin and the Manta's right side leaves 45 px.
    let manta = builtin(MANTA_ID).unwrap();
    let mut doc = page(vec![]);
    doc.rhythm.h_unit_mm = 6.35;
    doc.rhythm.v_unit_mm = 6.35;
    doc.rhythm.origin = RhythmOrigin::PageCorner;
    doc.page.margins_mm = Margins {
        top: 12.7,
        right: 12.7,
        bottom: 12.7,
        left: 12.7,
    };
    let words: Vec<String> = run(&doc, &manta)
        .iter()
        .filter(|f| f.rule == "L18")
        .map(|f| quadrule_i18n::Strings::english().format(&f.message))
        .collect();
    let minus = words.iter().any(|w| w.contains("-0"));
    let ok = !words.is_empty() && !minus;
    println!(
        "\nCHECK crate=lint name=l18_no_minus_zero result={} findings={} says_minus_zero={minus}",
        pass(ok),
        words.len()
    );
    assert!(ok);
}
