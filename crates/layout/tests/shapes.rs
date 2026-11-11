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

//! Shapes, dashes and caps in layout (rung 6a, docs/bbq/2026-11-10-rung-6-shapes.md). Each test
//! prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::shapes::*;
use quadrule_layout::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn both() -> [Profile; 2] {
    [builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap()]
}

/// A one-object page: `o` on the Manta's page, laid out on `target`.
fn lay(o: Object, target: &Profile) -> Vec<Item> {
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
    display_list(&doc, &builtin(MANTA_ID).unwrap(), target, &outlines)
        .unwrap()
        .items
}

/// The shapes page's object of this name.
fn named(name: &str) -> Object {
    shapes_document()
        .objects()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("no object {name}"))
        .clone()
}

/// A constraint-free copy, so the frame is the same size in mm on every device.
fn pinned(mut o: Object) -> Object {
    o.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    o
}

fn whole(v: f64) -> bool {
    v.fract() == 0.0
}

#[test]
fn dash_presets_land_on_whole_pixels_on_both_devices() {
    // Q6: Dotted = the stroke's width; Short dash 0.5/0.5 mm = 6/6 px, Dash 1/0.5 mm = 12/6 px,
    // Long dash 2/1 mm = 24/12 px at 300 PPI; each rounded to whole device pixels.
    let mut parts = Vec::new();
    let mut ok = true;
    for p in both() {
        let got: Vec<Option<(f64, f64)>> = [
            DashStyle::Solid,
            DashStyle::Dotted,
            DashStyle::Short,
            DashStyle::Dash,
            DashStyle::Long,
            DashStyle::Custom {
                dash_mm: 1.5,
                gap_mm: 1.0,
            },
            DashStyle::Custom {
                dash_mm: 0.0,
                gap_mm: f64::NAN,
            },
        ]
        .iter()
        .map(|d| dash_px(*d, 2, &p))
        .collect();
        let want = [
            None,
            Some((2.0, 2.0)),
            Some((6.0, 6.0)),
            Some((12.0, 6.0)),
            Some((24.0, 12.0)),
            Some((18.0, 12.0)),
            Some((1.0, 1.0)),
        ];
        ok &= got == want;
        parts.push(format!(
            "{}:short={:?}:dash={:?}:long={:?}",
            p.short_name, got[2], got[3], got[4]
        ));
    }
    println!(
        "\nCHECK crate=layout name=dash_presets_whole_px result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

/// The runs along a horizontal dashed line: (start, length) of each Rect, left to right.
fn runs(items: &[Item]) -> Vec<(f64, f64)> {
    let mut v: Vec<(f64, f64)> = items
        .iter()
        .filter_map(|i| match i {
            Item::Rect { x0, x1, .. } => Some((*x0, x1 - x0)),
            _ => None,
        })
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v
}

#[test]
fn dashed_lines_have_whole_pixel_dashes_starting_at_the_line_start() {
    let mut parts = Vec::new();
    let mut ok = true;
    for p in both() {
        for (name, on, off) in [
            ("Line, Short dash", 6.0, 6.0),
            ("Line, Dash", 12.0, 6.0),
            ("Line, Long dash", 24.0, 12.0),
        ] {
            let o = pinned(named(name));
            let items = lay(o.clone(), &p);
            let r = runs(&items);
            let x_start = p.mm_to_px(o.frame.x_mm).round();
            let all_rects = items.iter().all(|i| matches!(i, Item::Rect { .. }));
            let starts = r.first().is_some_and(|&(x, _)| x == x_start);
            // Every dash but the last is the full dash, and the next begins `off` later.
            let regular = r
                .windows(2)
                .all(|w| w[0].1 == on && w[1].0 - w[0].0 == on + off);
            let whole_px = items.iter().all(|i| match i {
                Item::Rect { x0, y0, x1, y1, .. } => [x0, y0, x1, y1].iter().all(|v| whole(**v)),
                _ => false,
            });
            // The last dash may be cut by the line's end.
            let last_ok = r.last().is_some_and(|&(x, l)| {
                let end = p.mm_to_px(o.frame.x_mm + o.frame.w_mm).round();
                l <= on && x + l <= end
            });
            ok &= all_rects && starts && regular && whole_px && last_ok && r.len() > 10;
            parts.push(format!("{}:{name}:dashes={}", p.short_name, r.len()));
        }
    }
    println!(
        "\nCHECK crate=layout name=dashed_lines_whole_px result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn caps_butt_round_square() {
    // A thick dashed line: butt dashes are exactly the dash long; square ones reach half the
    // width further at each end; round ones are drawn as an anti-aliased path with round caps.
    let mut parts = Vec::new();
    let mut ok = true;
    for p in both() {
        let (w, _) = stroke(
            Style::Custom {
                width_mm: 1.0,
                gray: 51,
            },
            &p,
        );
        let dash = p.mm_to_px(2.0).round();
        let mut base = pinned(named("Line, Butt cap"));
        let butt = lay(base.clone(), &p);
        base.cap = LineCap::Square;
        let square = lay(base.clone(), &p);
        base.cap = LineCap::Round;
        let round = lay(base, &p);
        let (rb, rs) = (runs(&butt), runs(&square));
        let butt_ok = rb.iter().take(rb.len() - 1).all(|r| r.1 == dash);
        let square_ok = rs.iter().take(rs.len() - 1).all(|r| r.1 == dash + w as f64);
        let shifted = rs[1].0 == rb[1].0 - (w as f64 / 2.0);
        let round_ok = matches!(
            round.as_slice(),
            [Item::StrokePath {
                cap: LineCap::Round,
                ..
            }]
        );
        ok &= butt_ok && square_ok && shifted && round_ok;
        parts.push(format!(
            "{}:w={w}:butt_len={}:square_len={}",
            p.short_name, rb[0].1, rs[1].1
        ));
    }
    println!(
        "\nCHECK crate=layout name=caps result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn dash_polyline_cuts_the_path() {
    let line = [(0.0, 0.0), (20.0, 0.0)];
    let d = dash_polyline(&line, 6.0, 4.0);
    let starts: Vec<f64> = d.iter().map(|p| p.pts[0].0).collect();
    let ends: Vec<f64> = d.iter().map(|p| p.pts.last().unwrap().0).collect();
    // Around a corner, one dash bends.
    let corner = dash_polyline(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)], 6.0, 2.0);
    let bent = corner.iter().any(|p| p.pts.len() == 3);
    let ok = starts
        == [0.0, 10.0, 20.0 - 0.0]
            .iter()
            .copied()
            .take(2)
            .collect::<Vec<_>>()
        && ends == [6.0, 16.0]
        && d[0].at_start
        && !d[0].at_end
        && d.last().is_some_and(|p| !p.at_end)
        && bent
        && dash_polyline(&line, 0.0, 4.0).is_empty();
    println!(
        "\nCHECK crate=layout name=dash_polyline result={} dashes={} bent_at_corner={bent}",
        pass(ok),
        d.len()
    );
    assert!(ok);
}

#[test]
fn hard_edged_rectangles_stay_whole_pixels() {
    // Q2: a square-cornered, unrotated rectangle is rectangles on whole pixels, fill under stroke.
    let mut parts = Vec::new();
    let mut ok = true;
    for p in both() {
        for name in [
            "Rectangle",
            "Rounded rectangle, Shade",
            "Rectangle, Block, no stroke",
            "Rectangle, Emphasis, Shade",
            "Thick rectangle, Block fill",
        ] {
            let o = named(name);
            let hard = matches!(&o.kind, ObjectKind::Rect(r) if r.corner_radius_mm == 0.0);
            let items = lay(o.clone(), &p);
            let rects_only = items.iter().all(|i| matches!(i, Item::Rect { .. }));
            let whole_px = items.iter().all(|i| match i {
                Item::Rect { x0, y0, x1, y1, .. } => [x0, y0, x1, y1].iter().all(|v| whole(**v)),
                _ => true,
            });
            let fill_first = match (o.fill, items.first()) {
                (Some(f), Some(Item::Rect { gray, .. })) => *gray == p.fill_gray(f),
                (None, _) => true,
                _ => false,
            };
            let expect_items = match (hard, o.style == Style::None, o.fill.is_some()) {
                (true, true, true) => 1,
                (true, false, true) => 5,
                (true, false, false) => 4,
                _ => 0,
            };
            if hard {
                ok &= rects_only && whole_px && fill_first && items.len() == expect_items;
                parts.push(format!("{}:{name}:items={}", p.short_name, items.len()));
            } else {
                // Rounded corners are curves: anti-aliased.
                ok &= items.iter().any(|i| matches!(i, Item::Fill { .. }))
                    && items.iter().any(|i| matches!(i, Item::StrokePath { .. }));
            }
        }
    }
    println!(
        "\nCHECK crate=layout name=hard_edged_rects result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn strokes_centre_on_the_frame_edge_odd_widths_on_pixel_centres() {
    let mut ok = true;
    let mut parts = Vec::new();
    for p in both() {
        for (style, w) in [
            (Style::Role(LineRole::Guide), 1),
            (Style::Role(LineRole::Structure), 2),
            (Style::Role(LineRole::Emphasis), 3),
        ] {
            let mut o = pinned(named("Rectangle"));
            o.style = style;
            o.frame = Frame::new(20.01, 30.02, 40.0, 30.0);
            let items = lay(o.clone(), &p);
            let f = o.frame;
            let (l, t) = (p.mm_to_px(f.x_mm), p.mm_to_px(f.y_mm));
            let centre_l = snap_center(l, w);
            // The leftmost rect's outer edge is half a stroke outside the (snapped) frame edge.
            let min_x = items
                .iter()
                .filter_map(|i| match i {
                    Item::Rect { x0, .. } => Some(*x0),
                    _ => None,
                })
                .fold(f64::MAX, f64::min);
            let min_y = items
                .iter()
                .filter_map(|i| match i {
                    Item::Rect { y0, .. } => Some(*y0),
                    _ => None,
                })
                .fold(f64::MAX, f64::min);
            let half = w as f64 / 2.0;
            let centred = min_x == centre_l - half
                && min_y == snap_center(t, w) - half
                && (min_x + half - l).abs() <= 0.5;
            // An odd width's centre is on a pixel centre.
            let odd_ok = w % 2 == 0 || (centre_l - 0.5).fract() == 0.0;
            ok &= centred && odd_ok;
            parts.push(format!("{}:w={w}:outer_left={min_x}", p.short_name));
        }
    }
    println!(
        "\nCHECK crate=layout name=stroke_centred_on_edge result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn each_kind_makes_the_right_items() {
    let p = builtin(MANTA_ID).unwrap();
    let kinds = [
        ("Ellipse, custom fill", 2usize),
        ("Hexagon, Shade", 2),
        ("Star, Block", 2),
        ("Circle", 1),
        ("Octagon, Block", 1),
    ];
    let mut ok = true;
    let mut parts = Vec::new();
    for (name, n) in kinds {
        let o = named(name);
        let items = lay(o.clone(), &p);
        // A Fill (when filled) then a StrokePath (when stroked), all anti-aliased.
        let grays_ok = match (o.fill, items.first()) {
            (Some(f), Some(Item::Fill { gray, .. })) => *gray == p.fill_gray(f),
            (None, Some(Item::StrokePath { .. })) => true,
            _ => false,
        };
        let stroked_last =
            o.style == Style::None || matches!(items.last(), Some(Item::StrokePath { .. }));
        ok &= items.len() == n && grays_ok && stroked_last;
        parts.push(format!("{name}:items={}", items.len()));
    }
    // Style::None draws no stroke on a line either; a hidden or no-fill shape draws nothing extra.
    let mut line = named("Line, Dash");
    line.style = Style::None;
    ok &= lay(line, &p).is_empty();
    println!(
        "\nCHECK crate=layout name=kinds_make_items result={} {}",
        pass(ok),
        parts.join(" | ")
    );
    assert!(ok);
}

#[test]
fn polygon_and_star_geometry() {
    // The first corner is straight up; n sides give n corners, a star 2n; the shape touches all
    // four sides of its frame (it is stretched to fill it).
    let mut ok = true;
    let mut parts = Vec::new();
    for (n, star) in [
        (3u8, None),
        (5, None),
        (6, None),
        (7, None),
        (32, None),
        (5, Some(0.5)),
        (8, Some(0.3)),
    ] {
        let pg = PolygonShape {
            sides: n,
            star,
            extra: Extra::new(),
        };
        let pts = polygon_unit_points(&pg);
        let want = if star.is_some() {
            2 * n as usize
        } else {
            n as usize
        };
        let (min_x, max_x) = pts
            .iter()
            .fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.0), a.1.max(p.0)));
        let (min_y, max_y) = pts
            .iter()
            .fold((f64::MAX, f64::MIN), |a, p| (a.0.min(p.1), a.1.max(p.1)));
        let first_up = (pts[0].0 - 0.5).abs() < 1e-9 && pts[0].1.abs() < 1e-9;
        let fills = [min_x, min_y].iter().all(|v| v.abs() < 1e-9)
            && [max_x, max_y].iter().all(|v| (v - 1.0).abs() < 1e-9);
        ok &= pts.len() == want && first_up && fills;
        parts.push(format!(
            "{n}{}={}",
            if star.is_some() { "*" } else { "" },
            pts.len()
        ));
    }
    println!(
        "\nCHECK crate=layout name=polygon_geometry result={} corners={}",
        pass(ok),
        parts.join(",")
    );
    assert!(ok);
}

#[test]
fn old_objects_draw_as_before() {
    // The existing Item kinds keep drawing exactly as now: a slice-page line is still one Rect,
    // a slanted one still an Item::Stroke, with the new fields at their defaults.
    let p = builtin(MANTA_ID).unwrap();
    let doc = slice_document();
    let rule = doc.layers[1].objects[1].clone();
    let one = lay(rule.clone(), &p);
    let mut slanted = rule;
    slanted.frame = Frame::new(10.0, 10.0, 30.0, 20.0);
    let slant = lay(slanted, &p);
    let ok = matches!(one.as_slice(), [Item::Rect { .. }])
        && matches!(slant.as_slice(), [Item::Stroke { .. }]);
    println!(
        "\nCHECK crate=layout name=old_objects_unchanged result={} rule={} slanted={}",
        pass(ok),
        one.len(),
        slant.len()
    );
    assert!(ok);
}
