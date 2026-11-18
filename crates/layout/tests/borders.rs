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

//! Pattern region borders (docs/bbq/2026-11-18-pattern-borders.md, Q1 to Q3 and Defaulted): a line centred on the
//! frame, on the sides that are on; the pattern lines and dots that sit on a bordered edge are left out; a corner
//! rounds only where both its sides are on, and the ruling stops at the curve. Each test prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use kurbo::{BezPath, PathEl};
use quadrule_layout::border::Rim;
use quadrule_layout::pattern::{self, Env, Pen, pattern_items, pattern_items_turned, region_rim};
use quadrule_layout::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// The borders page with its margins taken off and its rhythm whole pixels, so the rhythm starts at the page's corner
/// and every step is 59 px on the Manta: a region on whole steps has a line or dot exactly on each of its edges.
fn doc() -> Document {
    let mut d = pattern_borders_document();
    d.page.margins_mm = Margins::all(0.0);
    d.rhythm.resolution = RhythmResolution::PixelLocked;
    d
}

/// A region on the rhythm: columns 2 to 8, rows 2 to 6 of the page's 5 mm rhythm on the Manta, so a line or dot
/// sits on every edge.
fn region(doc: &Document) -> (f64, f64, f64, f64) {
    let manta = builtin(MANTA_ID).unwrap();
    let r = RhythmPx::new(doc, &manta);
    let at = |o: f64, u: f64, k: f64| (o + u * k).round();
    (
        at(r.origin_x, r.unit_x, 2.0),
        at(r.origin_y, r.unit_y, 2.0),
        at(r.origin_x, r.unit_x, 8.0),
        at(r.origin_y, r.unit_y, 6.0),
    )
}

fn border(sides: BorderSides, radius: f64) -> Option<Border> {
    let mut b = Border::new();
    b.sides = sides;
    b.corner_radius_mm = radius;
    Some(b)
}

const PEN: Pen = Pen { w: 1, gray: 128 };

/// The items of a region of `p` with `border`, turned by `deg`, and the border's own items (what the region's border
/// draws), so the pattern's share is the difference.
fn draw(p: Pattern, border: Option<Border>, deg: f64, own: f64) -> (Vec<Item>, usize) {
    let mut pr = PatternRegion::new(p);
    pr.border = border;
    pr.angle_deg = own;
    draw_region(pr, deg)
}

/// [`draw`] for a region built by the caller.
fn draw_region(pr: PatternRegion, deg: f64) -> (Vec<Item>, usize) {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let env = Env {
        target: &manta,
        rhythm: &doc.rhythm,
        rhythm_px: RhythmPx::new(&doc, &manta),
    };
    let reg = region(&doc);
    let borders = region_rim(&pr, reg, &env).map_or(0, |r| r.items().len());
    let mut items = Vec::new();
    pattern_items_turned(&pr, PEN, reg, deg, &env, &mut items);
    (items, borders)
}

fn rects(items: &[Item]) -> Vec<(f64, f64, f64, f64)> {
    items
        .iter()
        .filter_map(|i| match i {
            Item::Rect { x0, y0, x1, y1, .. } => Some((*x0, *y0, *x1, *y1)),
            _ => None,
        })
        .collect()
}

fn path_points(p: &BezPath) -> Vec<(f64, f64)> {
    p.elements()
        .iter()
        .filter_map(|e| match e {
            PathEl::MoveTo(a) | PathEl::LineTo(a) => Some((a.x, a.y)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_bordered_side_loses_the_lines_on_its_edge_and_a_side_without_keeps_them() {
    let none = draw(Pattern::Lines, None, 0.0, 0.0).0.len();
    // Rows 2 to 6 of the rhythm: five lines, the first on the top edge and the last on the bottom edge.
    let top_bottom = BorderSides {
        left: false,
        right: false,
        ..BorderSides::ALL
    };
    let (both, b_both) = draw(Pattern::Lines, border(top_bottom, 0.0), 0.0, 0.0);
    let top_only = BorderSides {
        top: true,
        right: false,
        bottom: false,
        left: false,
    };
    let (top, b_top) = draw(Pattern::Lines, border(top_only, 0.0), 0.0, 0.0);
    let (all, b_all) = draw(Pattern::Lines, border(BorderSides::ALL, 0.0), 0.0, 0.0);
    // The pattern's share: the items less the border's.
    let (n_both, n_top, n_all) = (both.len() - b_both, top.len() - b_top, all.len() - b_all);
    let ok = none == 5 && n_both == 3 && n_top == 4 && n_all == 3 && b_both == 2 && b_top == 1;
    println!(
        "\nCHECK crate=layout name=border_drops_edge_lines result={} unbordered={none} top_and_bottom={n_both} top_only={n_top} all_four={n_all} border_items={b_both}/{b_top}/{b_all}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_bordered_dot_grid_leaves_out_the_dots_on_the_edge_and_a_side_off_keeps_its_own() {
    let count = |b: Option<Border>| {
        let (items, borders) = draw(Pattern::Dots, b, 0.0, 0.0);
        items.len() - borders
    };
    let none = count(None);
    let all = count(border(BorderSides::ALL, 0.0));
    let no_right = BorderSides {
        right: false,
        ..BorderSides::ALL
    };
    let right_off = count(border(no_right, 0.0));
    // Seven columns by five rows; all four sides take the outer ring (the 3 by 5 columns are not...): the inner 5 by 3.
    let ok = none == 35 && all == 15 && right_off == 15 + 3;
    println!(
        "\nCHECK crate=layout name=border_drops_edge_dots result={} unbordered={none} all_four={all} right_side_off={right_off}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_rounded_corner_cuts_the_ruling_and_drops_dots_beyond_the_curve_and_only_where_both_sides_are_on()
 {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let reg = region(&doc);
    let r_mm = 8.0;
    let r_px = manta.mm_to_px(r_mm);
    // Every point of the grid's lines (not the border) stays inside the rounded outline, a pixel of slack for the
    // border's own half width.
    let inside = |items: &[Item], skip: usize| {
        let (x0, y0, x1, y1) = reg;
        let corners = [
            (x0 + r_px, y0 + r_px, -1.0, -1.0),
            (x1 - r_px, y0 + r_px, 1.0, -1.0),
            (x1 - r_px, y1 - r_px, 1.0, 1.0),
            (x0 + r_px, y1 - r_px, -1.0, 1.0),
        ];
        rects(&items[..items.len() - skip])
            .iter()
            .all(|&(a, b, c, d)| {
                [(a, b), (c, b), (c, d), (a, d)].iter().all(|&(x, y)| {
                    corners.iter().all(|&(cx, cy, sx, sy)| {
                        !((x - cx) * sx > 0.0 && (y - cy) * sy > 0.0)
                            || (x - cx).hypot(y - cy) <= r_px + 1.5
                    })
                })
            })
    };
    let (rounded, b) = draw(Pattern::Grid, border(BorderSides::ALL, r_mm), 0.0, 0.0);
    let cut_ok = inside(&rounded, b);
    let (square, bs) = draw(Pattern::Grid, border(BorderSides::ALL, 0.0), 0.0, 0.0);
    let square_reaches = !inside(&square, bs);
    let has_arc = rounded.iter().any(|i| matches!(i, Item::StrokePath { .. }));
    // Dots: fewer dots than with square corners, and none beyond the curve.
    let dots = |radius: f64| {
        // Dots 2 mm apart, so some sit in the corner's missing part.
        let mut pr = PatternRegion::new(Pattern::Dots);
        pr.set_bound(false, (2.0, 2.0));
        pr.border = border(BorderSides::ALL, radius);
        let (items, b) = draw_region(pr, 0.0);
        items.len() - b
    };
    let (d_round, d_square) = (dots(8.0), dots(0.0));
    // A corner with one side off stays square: top only gives no arc and cuts no line at the right end.
    let top_only = BorderSides {
        top: true,
        right: false,
        bottom: false,
        left: false,
    };
    let (t, tb) = draw(Pattern::Grid, border(top_only, r_mm), 0.0, 0.0);
    let no_arc = !t.iter().any(|i| matches!(i, Item::StrokePath { .. }));
    let right_end = rects(&t[..t.len() - tb])
        .iter()
        .fold(0.0_f64, |m, r| m.max(r.2));
    let reaches_right = right_end >= reg.2;
    let ok = cut_ok && square_reaches && has_arc && d_round < d_square && no_arc && reaches_right;
    println!(
        "\nCHECK crate=layout name=border_rounded_corners result={} ruling_inside_curve={cut_ok} square_reaches_corner={square_reaches} arc_drawn={has_arc} dots_round={d_round} dots_square={d_square} one_side_stays_square={no_arc} line_reaches_right_edge={reaches_right}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_radius_is_capped_at_half_the_shorter_side_and_a_missing_stroke_draws_and_drops_nothing() {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let reg = region(&doc);
    let mut b = Border::new();
    b.corner_radius_mm = 10_000.0;
    let rim = Rim::new(reg, &b, &manta).unwrap();
    let half = ((reg.2 - reg.0).min(reg.3 - reg.1)) / 2.0;
    let capped = (rim.radius - half).abs() <= 1.0;
    // Not-a-number and negative radii are square.
    let mut nan = Border::new();
    nan.corner_radius_mm = f64::NAN;
    let square = Rim::new(reg, &nan, &manta).unwrap().radius == 0.0;
    // No stroke: no border, and every line is kept.
    let mut none = Border::new();
    none.style = Style::None;
    let (items, borders) = draw(Pattern::Lines, Some(none), 0.0, 0.0);
    let kept = borders == 0 && items.len() == 5;
    let mut off = Border::new();
    off.sides = BorderSides {
        top: false,
        right: false,
        bottom: false,
        left: false,
    };
    let no_sides = Rim::new(reg, &off, &manta).is_none();
    let ok = capped && square && kept && no_sides;
    println!(
        "\nCHECK crate=layout name=border_limits result={} radius_capped={capped} nan_is_square={square} style_none_draws_nothing={kept} no_side_no_border={no_sides}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_border_turns_with_the_frame_and_not_with_the_ruling() {
    let doc = doc();
    let reg = region(&doc);
    // The ruling's own angle does not move the border: it stays on the upright frame.
    let (items, b) = draw(Pattern::Lines, border(BorderSides::ALL, 0.0), 0.0, 30.0);
    let borders = &items[items.len() - b..];
    let bx: Vec<f64> = rects(borders).iter().flat_map(|r| [r.0, r.2]).collect();
    let by: Vec<f64> = rects(borders).iter().flat_map(|r| [r.1, r.3]).collect();
    let upright = bx.iter().cloned().fold(f64::MAX, f64::min) < reg.0
        && by.iter().cloned().fold(f64::MIN, f64::max) > reg.3
        && rects(borders).len() == 4;
    // The frame's turn turns it: a half turn about the middle sends the box onto itself.
    let (half, hb) = draw(Pattern::Lines, border(BorderSides::ALL, 0.0), 180.0, 0.0);
    let half_borders = &half[half.len() - hb..];
    let near = |a: f64, b: f64| (a - b).abs() < 1.5;
    let (mut lo_x, mut hi_x) = (f64::MAX, f64::MIN);
    for r in rects(half_borders) {
        lo_x = lo_x.min(r.0);
        hi_x = hi_x.max(r.2);
    }
    let half_ok = near(lo_x, reg.0) && near(hi_x, reg.2);
    // A turn of 20 degrees makes the border polygons (anti-aliased) in the same order as the others.
    let (t, tb) = draw(Pattern::Lines, border(BorderSides::ALL, 3.0), 20.0, 0.0);
    let turned = t[t.len() - tb..]
        .iter()
        .all(|i| matches!(i, Item::StrokePath { .. }));
    let moved = t[t.len() - tb..].iter().any(|i| match i {
        Item::StrokePath { path, .. } => path_points(path).iter().any(|&(x, y)| {
            x < reg.0 - 2.0 || y < reg.1 - 2.0 || y > reg.3 + 2.0 || x > reg.2 + 2.0
        }),
        _ => false,
    });
    let ok = upright && half_ok && turned && moved;
    println!(
        "\nCHECK crate=layout name=border_turns_with_the_frame result={} ruling_angle_leaves_it_upright={upright} half_turn_same_box={half_ok} turned_frame_turns_it={turned}/{moved}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_ruling_turned_a_quarter_still_loses_the_lines_on_a_bordered_edge() {
    // Lines turned 90 degrees are vertical: the left and right edges carry lines. With the left and right
    // sides on, those are left out; with only the top and bottom on, they stay.
    let sides_lr = BorderSides {
        top: false,
        bottom: false,
        ..BorderSides::ALL
    };
    let sides_tb = BorderSides {
        left: false,
        right: false,
        ..BorderSides::ALL
    };
    let count = |s: BorderSides| {
        let (items, b) = draw(Pattern::Lines, border(s, 0.0), 0.0, 90.0);
        items.len() - b
    };
    let none = {
        let (items, _) = draw(Pattern::Lines, None, 0.0, 90.0);
        items.len()
    };
    let (lr, tb) = (count(sides_lr), count(sides_tb));
    let ok = lr == none - 2 && tb == none;
    println!(
        "\nCHECK crate=layout name=border_quarter_ruling result={} lines={none} left_right_on={lr} top_bottom_on={tb}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_bordered_region_in_a_group_and_in_a_repeat_draws_its_border_in_every_copy() {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let nomad = builtin(NOMAD_ID).unwrap();
    // The page's plain ruled cell, alone, in a group, and as the source of a repeat of three.
    let cell = doc
        .objects()
        .find(|o| matches!(&o.kind, ObjectKind::Pattern(p) if p.border.is_some() && p.pattern == Pattern::Lines && o.rotation() == 0.0))
        .unwrap()
        .clone();
    let page = |objs: Vec<Object>| {
        let mut d = doc.clone();
        d.layers = vec![Layer {
            name: "t".into(),
            visible: true,
            locked: false,
            objects: objs,
            extra: Extra::new(),
        }];
        d
    };
    let draw_on = |d: &Document, target: &quadrule_profiles::Profile| {
        let mut reg = quadrule_text::FontRegistry::bundled_only();
        let laid = lay_out_text(d, &manta, target, &mut reg).unwrap();
        display_list(d, &manta, target, &laid.outlines)
            .unwrap()
            .items
    };
    let mut group = cell.clone();
    group.id = Uuid::from_u128(0xB0_0001);
    group.kind = ObjectKind::Group(GroupShape {
        children: vec![cell.clone()],
        extra: Extra::new(),
    });
    let mut shape = RepeatShape::new(cell.clone());
    shape.count = 3;
    shape.h_spacing = Spacing::Mm(30.0);
    let mut repeat = cell.clone();
    repeat.id = Uuid::from_u128(0xB0_0002);
    repeat.kind = ObjectKind::Repeat(shape);
    if let Some(f) = repeat.repeat_fit((5.0, 5.0)) {
        repeat.frame = f;
    }
    let mut parts = Vec::new();
    let mut ok = true;
    for (name, target) in [("manta", &manta), ("nomad", &nomad)] {
        let one = draw_on(&page(vec![cell.clone()]), target).len();
        let grouped = draw_on(&page(vec![group.clone()]), target).len();
        let repeated = draw_on(&page(vec![repeat.clone()]), target).len();
        let good = one > 4 && grouped == one && repeated == 3 * one;
        ok &= good;
        parts.push(format!(
            "{name}:one={one}:group={grouped}:repeat={repeated}"
        ));
    }
    println!(
        "\nCHECK crate=layout name=border_in_group_and_repeat result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_region_with_no_border_draws_the_same_items_through_every_entry_point() {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let env = Env {
        target: &manta,
        rhythm: &doc.rhythm,
        rhythm_px: RhythmPx::new(&doc, &manta),
    };
    let reg = region(&doc);
    let mut same = true;
    for kind in [
        Pattern::Lines,
        Pattern::Dots,
        Pattern::Grid,
        Pattern::Isometric {
            style: IsoStyle::Lines,
        },
    ] {
        let pr = PatternRegion::new(kind);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        pattern_items(&pr, PEN, reg, &env, &mut a);
        pattern_items_turned(&pr, PEN, reg, 0.0, &env, &mut b);
        same &= a == b && pr.border.is_none() && pattern::region_rim(&pr, reg, &env).is_none();
    }
    println!(
        "\nCHECK crate=layout name=border_none_is_inert result={} same={same}",
        pass(same)
    );
    assert!(same);
}

#[test]
fn slanted_ruling_stops_at_a_rounded_corner() {
    // An isometric grid and a honeycomb in a region with 8 mm corners: no slanted stroke has a point in a corner's
    // missing part (a pixel of slack), and the same region with square corners does reach into it.
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let reg = region(&doc);
    let r_px = manta.mm_to_px(8.0);
    let (x0, y0, x1, y1) = reg;
    let corners = [
        (x0 + r_px, y0 + r_px, -1.0, -1.0),
        (x1 - r_px, y0 + r_px, 1.0, -1.0),
        (x1 - r_px, y1 - r_px, 1.0, 1.0),
        (x0 + r_px, y1 - r_px, -1.0, 1.0),
    ];
    let beyond = |items: &[Item]| {
        items.iter().any(|i| match i {
            Item::StrokePath { path, width, .. } if *width <= 1.0 => {
                path_points(path).iter().any(|&(x, y)| {
                    corners.iter().any(|&(cx, cy, sx, sy)| {
                        (x - cx) * sx > 0.0
                            && (y - cy) * sy > 0.0
                            && (x - cx).hypot(y - cy) > r_px + 1.5
                    })
                })
            }
            _ => false,
        })
    };
    let mut parts = Vec::new();
    let mut ok = true;
    for (name, kind) in [
        (
            "iso",
            Pattern::Isometric {
                style: IsoStyle::Lines,
            },
        ),
        (
            "hex",
            Pattern::Hex {
                orientation: HexOrientation::Flat,
                height_units: 1,
            },
        ),
    ] {
        let (round, _) = draw(kind, border(BorderSides::ALL, 8.0), 0.0, 0.0);
        let (square, _) = draw(kind, border(BorderSides::ALL, 0.0), 0.0, 0.0);
        let good = !beyond(&round) && beyond(&square);
        ok &= good;
        parts.push(format!(
            "{name}:cut={}:square_reaches={}",
            !beyond(&round),
            beyond(&square)
        ));
    }
    println!(
        "\nCHECK crate=layout name=border_slanted_cut result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
