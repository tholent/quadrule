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

//! Pattern regions turned whole (rung 12b, docs/bbq/2026-11-16-rung-12-missing-fonts.md, Q8): the
//! region is drawn upright in its own axes and turned about its middle, so the ruling stays parallel
//! to the frame. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use kurbo::{BezPath, PathEl};
use quadrule_layout::pattern::{self, Env, Pen, pattern_items, pattern_items_turned};
use quadrule_layout::rotate::rotate_point;
use quadrule_layout::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// Every kind of pattern, with a name.
fn kinds() -> Vec<(&'static str, Pattern)> {
    let hex = |orientation| Pattern::Hex {
        orientation,
        height_units: 2,
    };
    vec![
        ("lines", Pattern::Lines),
        ("dots", Pattern::Dots),
        ("grid", Pattern::Grid),
        (
            "iso_lines",
            Pattern::Isometric {
                style: IsoStyle::Lines,
            },
        ),
        (
            "iso_dots",
            Pattern::Isometric {
                style: IsoStyle::Dots,
            },
        ),
        ("hex_flat", hex(HexOrientation::Flat)),
        ("hex_pointy", hex(HexOrientation::Pointy)),
        (
            "cornell",
            Pattern::Cornell {
                cue_units: None,
                summary_units: None,
            },
        ),
        (
            "staff",
            Pattern::MusicStaff {
                gap_mm: 1.5,
                pitch_units: 5,
                grand: false,
            },
        ),
        (
            "grand_staff",
            Pattern::MusicStaff {
                gap_mm: 1.5,
                pitch_units: 5,
                grand: true,
            },
        ),
    ]
}

const REGION: (f64, f64, f64, f64) = (300.0, 500.0, 700.0, 801.0);

/// The page's rhythm is 5 mm with a major line every 5, as on the turned-patterns page.
fn doc() -> Document {
    turned_patterns_document()
}

/// Draws `p` in `REGION` turned by `deg`.
fn draw(p: Pattern, deg: f64, bound: bool) -> Vec<Item> {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let env = Env {
        target: &manta,
        rhythm: &doc.rhythm,
        rhythm_px: RhythmPx::new(&doc, &manta),
    };
    let mut pr = PatternRegion::new(p);
    pr.bound = bound;
    let mut items = Vec::new();
    pattern_items_turned(&pr, Pen { w: 2, gray: 0 }, REGION, deg, &env, &mut items);
    items
}

/// Every point an item list is made of, in order: a rectangle's four corners, a path's points.
fn points(items: &[Item]) -> Vec<(f64, f64)> {
    let path = |p: &BezPath, out: &mut Vec<(f64, f64)>| {
        for el in p.elements() {
            match el {
                PathEl::MoveTo(a) | PathEl::LineTo(a) => out.push((a.x, a.y)),
                _ => {}
            }
        }
    };
    let mut out = Vec::new();
    for it in items {
        match it {
            Item::Rect { x0, y0, x1, y1, .. } => {
                out.extend([(*x0, *y0), (*x1, *y0), (*x1, *y1), (*x0, *y1)])
            }
            Item::Fill { path: p, .. } | Item::StrokePath { path: p, .. } => path(p, &mut out),
            _ => {}
        }
    }
    out
}

#[test]
fn a_region_turned_by_nothing_draws_exactly_as_before() {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let env = Env {
        target: &manta,
        rhythm: &doc.rhythm,
        rhythm_px: RhythmPx::new(&doc, &manta),
    };
    let mut same = true;
    for (_, p) in kinds() {
        let pr = PatternRegion::new(p);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        pattern_items(&pr, Pen { w: 2, gray: 0 }, REGION, &env, &mut a);
        pattern_items_turned(&pr, Pen { w: 2, gray: 0 }, REGION, 0.0, &env, &mut b);
        let mut full = Vec::new();
        pattern_items_turned(&pr, Pen { w: 2, gray: 0 }, REGION, 360.0, &env, &mut full);
        same &= a == b && a == full && !a.is_empty();
    }
    println!(
        "\nCHECK crate=layout name=turned_pattern_zero result={} kinds={} identical_to_upright={same}",
        pass(same),
        kinds().len()
    );
    assert!(same);
}

#[test]
fn every_kind_turns_rigidly_about_the_regions_middle() {
    let mut bad = Vec::new();
    for (name, p) in kinds() {
        for deg in [15.0, 30.0, 45.0, 123.0, 333.3] {
            let upright = points(&draw(p, 0.0, true));
            let turned = points(&draw(p, deg, true));
            let pivot = pattern::turn_pivot(REGION, deg);
            let back: Vec<_> = turned
                .iter()
                .map(|&q| rotate_point(q, pivot, -deg))
                .collect();
            // Rectangles of one gray are merged into a path, which closes with no extra point.
            let rigid = back.len() == upright.len()
                && back
                    .iter()
                    .zip(&upright)
                    .all(|(a, b)| near(a.0, b.0) && near(a.1, b.1));
            if !rigid || upright.is_empty() {
                bad.push(format!("{name}@{deg}"));
            }
        }
    }
    println!(
        "\nCHECK crate=layout name=turned_pattern_rigid result={} kinds={} angles=5 not_rigid={bad:?}",
        pass(bad.is_empty()),
        kinds().len()
    );
    assert!(bad.is_empty());
}

#[test]
fn a_quarter_turn_keeps_every_rule_a_rectangle_on_whole_pixels() {
    let mut bad = Vec::new();
    for (name, p) in kinds() {
        for deg in [90.0, 180.0, 270.0] {
            let upright = draw(p, 0.0, true);
            let turned = draw(p, deg, true);
            let rects = |v: &[Item]| v.iter().filter(|i| matches!(i, Item::Rect { .. })).count();
            let whole = turned.iter().all(|i| match i {
                Item::Rect { x0, y0, x1, y1, .. } => [x0, y0, x1, y1]
                    .iter()
                    .all(|v| (**v - v.round()).abs() == 0.0),
                Item::StrokePath { .. } => true,
                _ => false,
            });
            if !(whole && rects(&upright) == rects(&turned) && upright.len() == turned.len()) {
                bad.push(format!("{name}@{deg}"));
            }
        }
    }
    println!(
        "\nCHECK crate=layout name=turned_pattern_quarter result={} kinds={} not_whole={bad:?}",
        pass(bad.is_empty()),
        kinds().len()
    );
    assert!(bad.is_empty());
}

#[test]
fn slanted_lines_are_cut_at_the_turned_frame_and_the_rest_reach_half_a_stroke() {
    let mut bad = Vec::new();
    for (name, p) in kinds() {
        for deg in [20.0, 45.0, 200.0] {
            let pivot = pattern::turn_pivot(REGION, deg);
            let turned = draw(p, deg, true);
            // Back in the region's own axes: a slanted line ends inside the frame; a rectangle (a
            // rule or a dot, 2 px wide) reaches at most a stroke past it.
            for it in &turned {
                let own = |q: (f64, f64)| rotate_point(q, pivot, -deg);
                let reach = match it {
                    Item::StrokePath { .. } => 1e-6,
                    _ => 2.0 + 1e-6,
                };
                for q in points(std::slice::from_ref(it)) {
                    let (x, y) = own(q);
                    let inside = x >= REGION.0 - reach
                        && x <= REGION.2 + reach
                        && y >= REGION.1 - reach
                        && y <= REGION.3 + reach;
                    if !inside {
                        bad.push(format!("{name}@{deg}"));
                        break;
                    }
                }
            }
        }
    }
    bad.dedup();
    println!(
        "\nCHECK crate=layout name=turned_pattern_cut result={} outside={bad:?}",
        pass(bad.is_empty())
    );
    assert!(bad.is_empty());
}

#[test]
fn a_bound_region_keeps_the_rhythms_steps_along_its_own_axes() {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let unit = RhythmPx::new(&doc, &manta).unit_y;
    let deg = 37.0;
    let pivot = pattern::turn_pivot(REGION, deg);
    // Ruled lines: one filled path of rectangles; each rectangle's own-axes centre is a step on.
    let items = draw(Pattern::Lines, deg, true);
    let pts = points(&items);
    let mut ys: Vec<f64> = pts
        .chunks(4)
        .map(|r| {
            let own: Vec<_> = r.iter().map(|&q| rotate_point(q, pivot, -deg)).collect();
            (own[0].1 + own[2].1) / 2.0
        })
        .collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup_by(|a, b| near(*a, *b));
    let steps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    // A major line is thicker, so its centre sits up to a pixel off the whole-pixel step.
    let even = steps.len() >= 3 && steps.iter().all(|s| (*s - unit).abs() <= 1.0);
    println!(
        "\nCHECK crate=layout name=turned_pattern_rhythm result={} lines={} step_px={unit} even={even}",
        pass(even),
        ys.len()
    );
    assert!(even);
}

#[test]
fn a_turned_region_in_a_group_and_in_a_repeat_turns_with_them() {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let mut reg = quadrule_text::FontRegistry::bundled_only();
    let laid = lay_out_text(&doc, &manta, &manta, &mut reg).unwrap();
    let placed = placed_objects(&doc, &manta, &manta, &laid.outlines);
    let angle_of = |id: Uuid| placed.iter().find(|p| p.id == id).map(|p| p.node.angle);
    // The grid in the group is turned 10, the group 15.
    let in_group = angle_of(turned_ids::nth(20)).is_some_and(|a| near(a, 25.0));
    let group = angle_of(turned_ids::GROUP).is_some_and(|a| near(a, 15.0));
    // The repeat's source is turned 25, and so is each copy drawn from it.
    let source = angle_of(turned_ids::nth(22)).is_some_and(|a| near(a, 25.0));
    let shape_turned = placed
        .iter()
        .find(|p| p.id == turned_ids::nth(22))
        .is_some_and(|p| matches!(p.shape, Shape::Rotated { deg, .. } if near(deg, 25.0)));
    let dl = display_list(&doc, &manta, &manta, &laid.outlines).unwrap();
    let fills = dl
        .items
        .iter()
        .filter(|i| matches!(i, Item::Fill { .. }))
        .count();
    // Three copies of a dot region turned off a quarter: at least a filled path each.
    let copies = fills >= 3;
    let ok = in_group && group && source && shape_turned && copies;
    println!(
        "\nCHECK crate=layout name=turned_pattern_group_repeat result={} child_total_25={in_group} group_15={group} repeat_source_25={source} picked_as_turned_box={shape_turned} filled_paths={fills}",
        pass(ok)
    );
    assert!(ok);
}
