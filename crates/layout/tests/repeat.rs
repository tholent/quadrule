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

//! The repeat tool in layout (rung 10, docs/bbq/2026-11-13-rung-10-repeat-tool.md): where the
//! copies fall on each device, in whole pixels, with the alternation rules, Fill, Centre and the cap.
//! Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn empty_outlines(id: &str) -> ProfileOutlines {
    ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn list(id: &str, doc: &Document) -> DisplayList {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    display_list(doc, &primary, &target, &empty_outlines(&target.id)).unwrap()
}

fn obj(n: u128, frame: Frame, style: Style, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(0x5155_4144_000a_4000_8000_0000_0000_0000 + n),
        name: format!("Object {n}"),
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
        kind,
        extra: Extra::new(),
    }
}

fn rect(n: u128, frame: Frame) -> Object {
    let mut o = obj(
        n,
        frame,
        Style::Role(LineRole::Structure),
        ObjectKind::Rect(RectShape::default()),
    );
    o.fill = Some(Fill::Role(FillRole::Shade));
    o
}

/// A page with only `objects` on one layer; 5 mm rhythm.
fn page(objects: Vec<Object>) -> Document {
    let mut doc = slice_document();
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc
}

fn repeat(n: u128, source: Object, tweak: impl FnOnce(&mut RepeatShape)) -> Object {
    let mut shape = RepeatShape::new(source);
    tweak(&mut shape);
    let mut o = obj(
        n,
        Frame::new(0.0, 0.0, 1.0, 1.0),
        Style::None,
        ObjectKind::Repeat(shape),
    );
    // The box a repeat must have, from its source.
    o.frame = o.repeat_fit((5.0, 5.0)).unwrap();
    o
}

fn rects(dl: &DisplayList) -> Vec<(f64, f64, f64, f64, u8)> {
    dl.items
        .iter()
        .filter_map(|i| match i {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } => Some((*x0, *y0, *x1, *y1, *gray)),
            _ => None,
        })
        .collect()
}

#[test]
fn one_copy_draws_exactly_the_source() {
    let src = rect(1, Frame::new(20.0, 30.0, 10.0, 10.0));
    let alone = page(vec![src.clone()]);
    let one = page(vec![repeat(2, src, |r| r.count = 1)]);
    // On the primary the source stays exactly where it is stored; on another device it keeps its
    // shape and the repeat's box (Scale, here) places it.
    let same = list(MANTA_ID, &alone).items == list(MANTA_ID, &one).items;
    println!(
        "\nCHECK crate=layout name=repeat_one_copy_is_the_source result={}",
        pass(same)
    );
    assert!(same);
}

#[test]
fn copies_are_the_source_moved_by_whole_pixels_and_the_pitch_is_the_source_rounded_up_to_steps() {
    let src = rect(1, Frame::new(20.0, 30.0, 10.0, 10.0));
    let doc = page(vec![repeat(2, src, |r| r.count = 4)]);
    let mut ok = true;
    let mut parts = Vec::new();
    for id in [MANTA_ID, NOMAD_ID] {
        let target = builtin(id).unwrap();
        let unit = RhythmPx::new(&doc, &target).unit_x;
        let dl = list(id, &doc);
        // A 10 mm source on a 5 mm rhythm is two steps: the pitch is two rhythm units.
        let stride = 2.0 * unit;
        let items: Vec<&Item> = dl.items.iter().collect();
        let per = items.len() / 4;
        let first = &dl.items[..per];
        let moved = (0..4).all(|k| {
            dl.items[k * per..(k + 1) * per]
                .iter()
                .zip(first)
                .all(|(a, b)| *a == translate_item(b, k as f64 * stride, 0.0))
        });
        ok &= dl.items.len().is_multiple_of(4) && moved && stride.fract() == 0.0;
        parts.push(format!("{id}:stride_px={stride}:moved={moved}"));
    }
    println!(
        "\nCHECK crate=layout name=repeat_whole_pixel_pitch result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn auto_pitch_rounds_the_source_up_and_follows_it() {
    let steps = (5.0, 5.0);
    let size = |w: f64| {
        let src = rect(1, Frame::new(0.0, 0.0, w, 5.0));
        let r = RepeatShape::new(src);
        r.stride_mm((w, 5.0), steps).0
    };
    // 1 unit gets 1u, 2.3 units get 3u, exactly 2 units get 2u.
    let ok = size(5.0) == 5.0 && size(11.5) == 15.0 && size(10.0) == 10.0 && size(0.5) == 5.0;
    println!(
        "\nCHECK crate=layout name=repeat_auto_rounds_up result={} one={} two_point_three={} two={}",
        pass(ok),
        size(5.0),
        size(11.5),
        size(10.0)
    );
    assert!(ok);
}

#[test]
fn pitch_gap_and_millimetres() {
    let src = rect(1, Frame::new(20.0, 30.0, 10.0, 10.0));
    let stride = |tweak: &dyn Fn(&mut RepeatShape)| {
        let mut r = RepeatShape::new(src.clone());
        tweak(&mut r);
        r.stride_mm((10.0, 10.0), (5.0, 5.0))
    };
    let pitch_u = stride(&|r| r.h_spacing = Spacing::Units(3.0));
    let gap_u = stride(&|r| {
        r.h_spacing = Spacing::Units(1.0);
        r.basis = SpacingBasis::Gap;
    });
    let pitch_mm = stride(&|r| r.h_spacing = Spacing::Mm(12.0));
    let gap_mm = stride(&|r| {
        r.h_spacing = Spacing::Mm(2.0);
        r.basis = SpacingBasis::Gap;
        r.v_spacing = Spacing::Mm(3.0);
    });
    let ok = pitch_u.0 == 15.0 && gap_u.0 == 15.0 && pitch_mm.0 == 12.0 && gap_mm == (12.0, 13.0);
    // A spacing in u follows a rhythm change; one in mm does not: on a 4 mm rhythm 3u is 12 mm.
    let r = RepeatShape {
        h_spacing: Spacing::Units(3.0),
        ..RepeatShape::new(src.clone())
    };
    let follows = r.stride_mm((10.0, 10.0), (4.0, 4.0)).0 == 12.0;
    let r = RepeatShape {
        h_spacing: Spacing::Mm(12.0),
        ..RepeatShape::new(src)
    };
    let fixed = r.stride_mm((10.0, 10.0), (4.0, 4.0)).0 == 12.0;
    println!(
        "\nCHECK crate=layout name=repeat_pitch_gap_units result={} pitch_u={} gap_u={} pitch_mm={} gap_mm={:?} u_follows_rhythm={follows} mm_fixed={fixed}",
        pass(ok && follows && fixed),
        pitch_u.0,
        gap_u.0,
        pitch_mm.0,
        gap_mm
    );
    assert!(ok && follows && fixed);
}

#[test]
fn alternation_skips_and_changes_roles() {
    let src = rect(1, Frame::new(20.0, 30.0, 5.0, 5.0));
    let mut r = RepeatShape::new(src.clone());
    r.mode = RepeatMode::Grid;
    r.columns = 7;
    r.rows = 3;
    r.rules = vec![
        AlternationRule {
            counts: RuleCounts::Columns,
            when: RuleWhen::Every { n: 7, start: None },
            action: RuleAction::LineRole(LineRole::Emphasis),
        },
        AlternationRule {
            counts: RuleCounts::Rows,
            when: RuleWhen::List(vec![2]),
            action: RuleAction::Skip,
        },
        AlternationRule {
            counts: RuleCounts::Cells,
            when: RuleWhen::List(vec![1, 3]),
            action: RuleAction::FillRole(FillRole::Block),
        },
    ];
    // Every 7th column is column 7; row 2 is skipped; cells 1 and 3 take the Block fill.
    let e = |c, rw| r.effect(c, rw, 7);
    let ok = e(7, 1).line == Some(LineRole::Emphasis)
        && e(6, 1).line.is_none()
        && e(1, 2).skip
        && e(7, 2).skip
        && !e(1, 3).skip
        && e(1, 1).fill == Some(FillRole::Block)
        && e(3, 1).fill == Some(FillRole::Block)
        && e(2, 1).fill.is_none()
        && e(7, 3).line == Some(LineRole::Emphasis);
    // Every Nth with a start: 2, 5, 8 ...
    let every = AlternationRule {
        counts: RuleCounts::Cells,
        when: RuleWhen::Every {
            n: 3,
            start: Some(2),
        },
        action: RuleAction::Skip,
    };
    let hits: Vec<u32> = (1..=9).filter(|k| every.hits(*k)).collect();
    let starts = hits == [2, 5, 8];
    // Drawn: the row 2 cells are not there; the others are, and the Emphasis column is thicker.
    let doc = page(vec![{
        let mut o = obj(
            2,
            Frame::new(0.0, 0.0, 1.0, 1.0),
            Style::None,
            ObjectKind::Repeat(r.clone()),
        );
        o.frame = o.repeat_fit((5.0, 5.0)).unwrap();
        o
    }]);
    let dl = list(MANTA_ID, &doc);
    let plan_total = 7 * 3;
    let drawn_cells = 7 * 2;
    // Each shape is a Shade/Block fill plus an outline.
    let per_cell = {
        let one = page(vec![src]);
        list(MANTA_ID, &one).items.len()
    };
    let counts = dl.items.len() == per_cell * drawn_cells;
    println!(
        "\nCHECK crate=layout name=repeat_alternation result={} rules_ok={ok} every_from_start={starts} items={} per_cell={per_cell} cells={plan_total}->{drawn_cells} counts={counts}",
        pass(ok && starts && counts),
        dl.items.len()
    );
    assert!(ok && starts && counts);
}

#[test]
fn fill_counts_follow_the_box_on_each_device_and_centre_splits_the_leftover() {
    let src = rect(1, Frame::new(8.0, 40.0, 5.0, 5.0));
    let mut rep = repeat(2, src, |r| {
        r.fill = true;
        r.mode = RepeatMode::Across;
        r.leftover = Leftover::Start;
    });
    // A box pinned to the margins: LeftRight, so each device gets its own width.
    rep.frame = Frame::new(8.0, 40.0, MANTA_W_MM - 16.0, 5.0);
    rep.constraints = Constraints {
        h: HConstraint::LeftRight,
        v: VConstraint::Top,
    };
    let doc = page(vec![rep]);
    let count = |id: &str, doc: &Document| {
        let target = builtin(id).unwrap();
        let primary = builtin(&doc.primary_profile_id).unwrap();
        let nodes = tree::resolve_nodes_env(
            doc,
            primary.page_mm(),
            target.page_mm(),
            override_key(&primary, &target),
            Some(device_scale(doc, &target)),
        );
        let p = nodes.iter().find_map(|n| n.plan).unwrap();
        (p.cols, p.rows)
    };
    let (m, n) = (count(MANTA_ID, &doc), count(NOMAD_ID, &doc));
    // Manta: 1920 px minus margins (8 mm at 300 PPI is 94 px each side); a 5 mm step is 59 px.
    let manta_w = (MANTA_W_MM - 16.0) * 300.0 / MM_PER_IN;
    let want_m = ((manta_w.round() - 59.0) / 59.0).floor() as u32 + 1;
    let differ = m.0 != n.0 && m.1 == 1 && n.1 == 1;
    // Centre: the spare pixels are split, the odd one to the end.
    let mut centred = doc.clone();
    if let ObjectKind::Repeat(r) = &mut centred.layers[0].objects[0].kind {
        r.leftover = Leftover::Centre;
    }
    let target = builtin(MANTA_ID).unwrap();
    let nodes = tree::resolve_nodes_env(
        &centred,
        target.page_mm(),
        target.page_mm(),
        None,
        Some(device_scale(&centred, &target)),
    );
    let p = nodes.iter().find_map(|n| n.plan).unwrap();
    let spare = manta_w.round() - (f64::from(m.0 - 1) * 59.0 + 59.0);
    let want_shift = (spare / 2.0).floor() / (300.0 / MM_PER_IN);
    let centre_ok = (p.shift0.0 - want_shift).abs() < 1e-9 && p.shift0.1 == 0.0;
    let ok = m.0 == want_m && differ && centre_ok;
    println!(
        "\nCHECK crate=layout name=repeat_fill_and_centre result={} manta_cols={} nomad_cols={} want_manta={want_m} spare_px={spare} centre_shift_mm={:.4}",
        pass(ok),
        m.0,
        n.0,
        p.shift0.0
    );
    assert!(ok);
}

#[test]
fn the_cap_holds_nested_repeats_to_five_thousand_copies() {
    let src = rect(1, Frame::new(8.0, 40.0, 1.0, 1.0));
    let inner = repeat(2, src, |r| {
        r.mode = RepeatMode::Grid;
        r.columns = 30;
        r.rows = 30;
    });
    let outer = repeat(3, inner, |r| {
        r.mode = RepeatMode::Grid;
        r.columns = 10;
        r.rows = 10;
    });
    let mut doc = page(vec![outer]);
    doc.page.margins_mm = Margins::all(0.0);
    let target = builtin(MANTA_ID).unwrap();
    let nodes = tree::resolve_nodes_env(
        &doc,
        target.page_mm(),
        target.page_mm(),
        None,
        Some(device_scale(&doc, &target)),
    );
    let o = nodes.iter().find(|n| n.depth == 0).unwrap().plan.unwrap();
    // The inner repeat weighs 900 copies, so the outer may draw 5000 / 900 = 5 of its 100.
    let i = nodes.iter().find(|n| n.depth == 1).unwrap().plan.unwrap();
    let ok = o.budget == 5 && o.capped() && i.budget == 5000 && !i.capped();
    let dl = list(MANTA_ID, &doc);
    let rect_items = rects(&dl).len();
    println!(
        "\nCHECK crate=layout name=repeat_cap result={} outer_budget={} inner_budget={} items={}",
        pass(ok),
        o.budget,
        i.budget,
        dl.items.len()
    );
    assert!(ok && rect_items > 0);
}

#[test]
fn every_copy_is_found_by_the_pointer_and_stands_for_the_source() {
    let src = rect(1, Frame::new(20.0, 30.0, 10.0, 10.0));
    let doc = page(vec![repeat(2, src, |r| r.count = 3)]);
    let primary = builtin(MANTA_ID).unwrap();
    let placed = placed_objects(&doc, &primary, &primary, &empty_outlines(MANTA_ID));
    // The repeat, its source, and the two copies after the source.
    let copies = placed.iter().filter(|p| p.copy).count();
    let src_id = Uuid::from_u128(0x5155_4144_000a_4000_8000_0000_0000_0001);
    let ids_ok = placed.iter().filter(|p| p.id == src_id).count() == 3;
    // 20..30 mm is 236..354 px; the next copy is 2 units (118 px) on.
    let (cx, cy) = (primary.mm_to_px(25.0) + 2.0 * 59.0, primary.mm_to_px(35.0));
    let hit = placed
        .iter()
        .rev()
        .find(|p| !p.node.group && p.shape.hit(cx, cy, 0.0));
    let found = hit.is_some_and(|p| p.id == src_id && p.copy);
    let ok = copies == 2 && ids_ok && found && placed[0].node.repeat && placed[0].node.group;
    println!(
        "\nCHECK crate=layout name=repeat_copies_are_pickable result={} copies={copies} found_copy={found}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn turning_the_page_and_making_another_device_primary_keep_the_source_on_the_box() {
    let doc = repeats_document();
    let corner_ok = |d: &Document| {
        d.objects().filter(|o| o.is_repeat()).all(|o| {
            let sb = o.repeat().unwrap().source.outer_box();
            (sb.x_mm - o.frame.x_mm).abs() < 1e-6 && (sb.y_mm - o.frame.y_mm).abs() < 1e-6
        })
    };
    let mut turned = doc.clone();
    let cmd = device::set_orientation(&doc, Orientation::Landscape).unwrap();
    turned.apply(cmd).unwrap();
    let a = corner_ok(&turned) && turned.page.orientation == Orientation::Landscape;
    let mut moved = doc.clone();
    let cmd = device::make_primary(&doc, NOMAD_ID).unwrap();
    moved.apply(cmd).unwrap();
    let b = corner_ok(&moved) && moved.primary_profile_id == NOMAD_ID;
    // No override was written on a source.
    let c = moved
        .objects()
        .filter(|o| o.is_repeat())
        .all(|o| o.children().iter().all(|s| s.overrides.is_empty()));
    println!(
        "\nCHECK crate=layout name=repeat_page_turn_and_primary result={} turn_keeps_corner={a} primary_keeps_corner={b} no_source_overrides={c}",
        pass(a && b && c)
    );
    assert!(a && b && c);
}
