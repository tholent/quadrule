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

//! Rotation, line direction and opacity in layout (rung 6b, docs/bbq/2026-11-10-rung-6-shapes.md).
//! Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::rotate::*;
use quadrule_layout::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn angles_are_exact_at_quarter_turns_and_clockwise() {
    let c = (100.0, 100.0);
    // To the right of the centre, turned 90 clockwise on a page whose y points down, it is below.
    let p = rotate_point((150.0, 100.0), c, 90.0);
    let exact = p == (100.0, 150.0)
        && rotate_point((150.0, 100.0), c, 180.0) == (50.0, 100.0)
        && rotate_point((150.0, 100.0), c, 270.0) == (100.0, 50.0)
        && rotate_point((150.0, 100.0), c, 0.0) == (150.0, 100.0)
        && rotate_point((150.0, 100.0), c, 360.0) == (150.0, 100.0);
    let back = {
        let q = rotate_point(rotate_point((130.0, 70.0), c, 37.0), c, -37.0);
        near(q.0, 130.0) && near(q.1, 70.0)
    };
    let quarter = is_quarter_turn(90.0) && is_quarter_turn(-90.0) && !is_quarter_turn(45.0);
    // A 40 by 10 box turned 90: 10 wide, 40 tall, about the same centre, exactly.
    let corners = rotated_corners((0.0, 0.0, 40.0, 10.0), 90.0);
    let bb = bounds_of(&corners);
    let swapped = bb == (15.0, -15.0, 25.0, 25.0);
    let ok = exact && back && quarter && swapped;
    println!(
        "\nCHECK crate=layout name=rotate_maths result={} clockwise_and_exact={exact} inverse={back} quarter={quarter} box_swaps={swapped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_line_runs_down_or_up_and_turns_with_its_box() {
    let r = (10.0, 20.0, 50.0, 40.0);
    let down = line_ends(r, LineDir::Down) == ((10.0, 20.0), (50.0, 40.0));
    let up = line_ends(r, LineDir::Up) == ((10.0, 40.0), (50.0, 20.0));
    // No width or no height: one diagonal, whichever is asked.
    let flat = line_ends((10.0, 20.0, 50.0, 20.0), LineDir::Up) == ((10.0, 20.0), (50.0, 20.0));
    let tall = line_ends((10.0, 20.0, 10.0, 60.0), LineDir::Up) == ((10.0, 20.0), (10.0, 60.0));
    // Turned 180 about the centre the ends swap places.
    let (a, b) = rotated_line_ends(r, LineDir::Down, 180.0);
    let turned = near(a.0, 50.0) && near(a.1, 40.0) && near(b.0, 10.0) && near(b.1, 20.0);
    let ok = down && up && flat && tall && turned;
    println!(
        "\nCHECK crate=layout name=line_direction result={} down={down} up={up} flat_has_one_diagonal={flat} tall_has_one_diagonal={tall} turned_180_swaps_ends={turned}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_turned_box_meets_a_rectangle_by_its_corners() {
    // A diamond (a square turned 45, half-diagonal 100) at (200, 200): its corner box reaches
    // 100..300, but the point (110, 110), inside that box, is outside the diamond.
    let sq = (
        200.0 - 70.710678,
        200.0 - 70.710678,
        200.0 + 70.710678,
        200.0 + 70.710678,
    );
    let corner_box = touches_clear(sq, 45.0, 0.0, (100.0, 100.0, 120.0, 120.0));
    let on_edge = rotated_touches(sq, 45.0, 0.0, (130.0, 195.0, 150.0, 205.0));
    let inside = rotated_touches(sq, 45.0, 0.0, (195.0, 195.0, 205.0, 205.0));
    let margin = rotated_touches(sq, 45.0, 45.0, (100.0, 100.0, 120.0, 120.0));
    let far = !rotated_touches(sq, 45.0, 0.0, (400.0, 400.0, 500.0, 500.0));
    // Unturned it is the plain box test.
    let plain = rotated_touches((0.0, 0.0, 10.0, 10.0), 0.0, 0.0, (5.0, 5.0, 20.0, 20.0))
        && !rotated_touches((0.0, 0.0, 10.0, 10.0), 0.0, 0.0, (11.0, 0.0, 20.0, 5.0));
    let ok = corner_box && on_edge && inside && margin && far && plain;
    println!(
        "\nCHECK crate=layout name=turned_box_overlap result={} corner_of_box_misses={corner_box} touches_edge={on_edge} inside={inside} margin_reaches={margin} far_misses={far} plain_box={plain}",
        pass(ok)
    );
    assert!(ok);
}

/// `true` when the box turned by `deg` does not touch `other`, though the box round its corners would.
fn touches_clear(r: Box4, deg: f64, margin: f64, other: Box4) -> bool {
    let bb = bounds_of(&rotated_corners(r, deg));
    let box_hit = bb.0 <= other.2 && other.0 <= bb.2 && bb.1 <= other.3 && other.1 <= bb.3;
    box_hit && !rotated_touches(r, deg, margin, other)
}

fn lay(o: Object, target: &Profile) -> (Vec<Item>, Vec<Placed>) {
    let mut doc = shapes_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects: vec![o],
        extra: Extra::new(),
    }];
    let outlines = ProfileOutlines {
        profile_id: target.id.clone(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    };
    let primary = builtin(MANTA_ID).unwrap();
    let items = display_list(&doc, &primary, target, &outlines)
        .unwrap()
        .items;
    (items, placed_objects(&doc, &primary, target, &outlines))
}

fn object(kind: ObjectKind, frame: Frame) -> Object {
    let mut o = transforms_document().layers[1].objects[0].clone();
    o.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    o.frame = frame;
    o.rotation_deg = 0.0;
    o.opacity_pct = 100;
    o.fill = None;
    o.style = Style::Role(LineRole::Structure);
    o.dash = DashStyle::Solid;
    o.cap = LineCap::Butt;
    o.kind = kind;
    o
}

#[test]
fn a_line_pointing_up_draws_and_picks_along_its_other_diagonal() {
    let manta = builtin(MANTA_ID).unwrap();
    let k = manta.mm_to_px(1.0);
    let frame = Frame::new(20.0, 30.0, 40.0, 20.0);
    let mk = |dir| {
        object(
            ObjectKind::Line(Line {
                direction: dir,
                ..Line::default()
            }),
            frame,
        )
    };
    let (items, placed) = lay(mk(LineDir::Up), &manta);
    let want = (20.0 * k, 50.0 * k, 60.0 * k, 30.0 * k);
    let stroke = matches!(
        items.as_slice(),
        [Item::Stroke { x0, y0, x1, y1, .. }]
            if near(*x0, want.0) && near(*y0, want.1) && near(*x1, want.2) && near(*y1, want.3)
    );
    // Picking follows the same diagonal: the middle is on it, the top-left corner is not.
    let shape = placed[0].shape;
    let mid = shape.hit(40.0 * k, 40.0 * k, 0.0);
    let corner_off = !shape.hit(20.0 * k + 5.0, 30.0 * k + 5.0, 3.0);
    // Down, the old way, is the other diagonal.
    let (down, _) = lay(mk(LineDir::Down), &manta);
    let down_ok = matches!(
        down.as_slice(),
        [Item::Stroke { x0, y0, x1, y1, .. }]
            if near(*x0, 20.0 * k) && near(*y0, 30.0 * k) && near(*x1, 60.0 * k) && near(*y1, 50.0 * k)
    );
    let ok = stroke && mid && corner_off && down_ok;
    println!(
        "\nCHECK crate=layout name=line_up_layout result={} up_stroke_bl_to_tr={stroke} pick_on_diagonal={mid} corner_off={corner_off} down_unchanged={down_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_turned_shape_is_placed_as_a_rotated_box() {
    let manta = builtin(MANTA_ID).unwrap();
    let k = manta.mm_to_px(1.0);
    let mut o = object(
        ObjectKind::Rect(RectShape::default()),
        Frame::new(20.0, 30.0, 40.0, 10.0),
    );
    o.rotation_deg = 90.0;
    let (_, placed) = lay(o.clone(), &manta);
    let (cx, cy) = (40.0 * k, 35.0 * k);
    let rot = matches!(placed[0].shape, Shape::Rotated { cx: x, cy: y, hw, hh, deg }
        if near(x, cx) && near(y, cy) && near(hw, 20.0 * k) && near(hh, 5.0 * k) && near(deg, 90.0));
    // Turned 90 it is 10 wide and 40 tall about the centre: a point above the centre is on it,
    // a point right of the centre, where the unturned box was, is not.
    let s = placed[0].shape;
    let hit = s.hit(cx, cy - 15.0 * k, 0.0) && !s.hit(cx + 15.0 * k, cy, 0.0);
    let b = s.bounds();
    let bounds = near(b.0, cx - 5.0 * k) && near(b.2, cx + 5.0 * k) && near(b.1, cy - 20.0 * k);
    // Unturned it stays a plain box, and a text-like kind that cannot turn ignores the field.
    o.rotation_deg = 0.0;
    let (_, flat) = lay(o, &manta);
    let plain = matches!(flat[0].shape, Shape::Box { .. });
    let mut pat = object(
        ObjectKind::Pattern(PatternRegion {
            pattern: Pattern::Lines,
            extra: Extra::new(),
        }),
        Frame::new(20.0, 30.0, 40.0, 10.0),
    );
    pat.rotation_deg = 90.0;
    let (_, pp) = lay(pat, &manta);
    let ignored = matches!(pp[0].shape, Shape::Box { .. });
    let ok = rot && hit && bounds && plain && ignored;
    println!(
        "\nCHECK crate=layout name=turned_shape_placed result={} rotated_box={rot} hit_by_turned_corners={hit} bounds_round_corners={bounds} unturned_is_box={plain} pattern_ignores_rotation={ignored}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn opacity_wraps_an_objects_items_in_a_group() {
    let manta = builtin(MANTA_ID).unwrap();
    let mut o = object(
        ObjectKind::Rect(RectShape::default()),
        Frame::new(20.0, 30.0, 40.0, 10.0),
    );
    o.fill = Some(Fill::Role(FillRole::Shade));
    let (plain, _) = lay(o.clone(), &manta);
    o.opacity_pct = 50;
    let (half, _) = lay(o.clone(), &manta);
    let wrapped = half.len() == plain.len() + 2
        && matches!(half.first(), Some(Item::BeginAlpha { alpha: 128, .. }))
        && matches!(half.last(), Some(Item::EndAlpha))
        && half[1..half.len() - 1] == plain[..];
    // The group's box holds every item.
    let bounds_hold = match (half.first(), items_bounds(&plain)) {
        (Some(Item::BeginAlpha { bounds, .. }), Some(b)) => *bounds == b,
        _ => false,
    };
    o.opacity_pct = 0;
    let (none, _) = lay(o.clone(), &manta);
    o.opacity_pct = 100;
    let (full, _) = lay(o, &manta);
    let alphas = alpha_of_percent(0) == 0
        && alpha_of_percent(100) == 255
        && alpha_of_percent(50) == 128
        && alpha_of_percent(1) == 3;
    let ok = wrapped && bounds_hold && none.is_empty() && full == plain && alphas;
    println!(
        "\nCHECK crate=layout name=opacity_group result={} wrapped_in_begin_and_end={wrapped} bounds_hold_items={bounds_hold} zero_draws_nothing={} hundred_is_plain={} alpha_values={alphas}",
        pass(ok),
        none.is_empty(),
        full == plain
    );
    assert!(ok);
}

#[test]
fn a_pixel_exact_page_is_unchanged_by_the_new_fields_at_their_defaults() {
    // Every object of the shapes page has rotation 0 and opacity 100: its display list holds no
    // group and no path that was not there in 6a (a rectangle with square corners is whole-pixel
    // rectangles still).
    let manta = builtin(MANTA_ID).unwrap();
    let doc = shapes_document();
    let outlines = ProfileOutlines {
        profile_id: manta.id.clone(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    };
    let dl = display_list(&doc, &manta, &manta, &outlines).unwrap();
    let no_groups = !dl
        .items
        .iter()
        .any(|i| matches!(i, Item::BeginAlpha { .. } | Item::EndAlpha));
    let all_default = doc
        .objects()
        .all(|o| o.rotation_deg == 0.0 && o.opacity_pct == 100);
    let ok = no_groups && all_default;
    println!(
        "\nCHECK crate=layout name=defaults_draw_as_before result={} no_groups={no_groups} all_default={all_default}",
        pass(ok)
    );
    assert!(ok);
}
