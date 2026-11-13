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

//! The rung 9 patterns drawn (docs/bbq/2026-11-13-rung-9-pattern-library.md): isometric lines and
//! dots, hexagons both ways, Cornell, music staff and grand staff, major lines, a region unbound
//! from the rhythm. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

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

/// A page holding one region of `pr` drawn with `role`, at (20, 20) mm, 100 by 80 mm; the rhythm
/// is 5 mm (59 px on the Manta) with `major` as its "Major every N".
fn page_with(pr: PatternRegion, role: LineRole, major: Option<u8>) -> Document {
    let mut doc = slice_document();
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = major;
    let mut o = doc.layers[0].objects[0].clone();
    o.frame = Frame::new(20.0, 20.0, 100.0, 80.0);
    o.constraints = Constraints::default();
    o.style = Style::Role(role);
    o.kind = ObjectKind::Pattern(pr);
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

/// Hard-edged rectangles: (x0, y0, x1, y1, gray).
fn rects(items: &[Item]) -> Vec<(f64, f64, f64, f64, u8)> {
    items
        .iter()
        .filter_map(|i| match *i {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } => Some((x0, y0, x1, y1, gray)),
            _ => None,
        })
        .collect()
}

/// Every segment of every stroked path: ((x0, y0), (x1, y1)).
fn segments(items: &[Item]) -> Vec<((f64, f64), (f64, f64))> {
    let mut out = Vec::new();
    for i in items {
        if let Item::StrokePath { path, .. } = i {
            let mut at = (0.0, 0.0);
            for el in path.elements() {
                match *el {
                    kurbo::PathEl::MoveTo(p) => at = (p.x, p.y),
                    kurbo::PathEl::LineTo(p) => {
                        out.push((at, (p.x, p.y)));
                        at = (p.x, p.y);
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

fn whole(v: f64) -> bool {
    v.fract() == 0.0
}

fn region_px() -> (f64, f64, f64, f64) {
    let m = manta();
    let p = |mm: f64| m.mm_to_px(mm).round();
    (p(20.0), p(20.0), p(120.0), p(100.0))
}

#[test]
fn isometric_lines_have_sharp_columns_and_slanted_lines_edge_to_edge() {
    let doc = page_with(
        PatternRegion::new(Pattern::Isometric {
            style: IsoStyle::Lines,
        }),
        LineRole::Guide,
        None,
    );
    let items = items_of(&doc);
    let (x0, y0, x1, y1) = region_px();
    let cols = rects(&items);
    let cols_sharp = !cols.is_empty()
        && cols
            .iter()
            .all(|r| [r.0, r.1, r.2, r.3].iter().all(|v| whole(*v)) && r.1 == y0 && r.3 == y1);
    let segs = segments(&items);
    // Every slanted line is 30 degrees from horizontal: |dy / dx| = tan 30 (half a side 59 / sqrt 3
    // down a column 59 px wide), and its ends are on the region's frame.
    let ratio = 1.0 / 3f64.sqrt();
    let angles = segs
        .iter()
        .all(|((ax, ay), (bx, by))| (((by - ay) / (bx - ax)).abs() - ratio).abs() < 1e-6);
    let on_frame = segs.iter().all(|((ax, ay), (bx, by))| {
        let end = |x: f64, y: f64| {
            (x - x0).abs() < 1e-6
                || (x - x1).abs() < 1e-6
                || (y - y0).abs() < 1e-6
                || (y - y1).abs() < 1e-6
        };
        end(*ax, *ay) && end(*bx, *by)
    });
    let one_path = items
        .iter()
        .filter(|i| matches!(i, Item::StrokePath { .. }))
        .count()
        == 1;
    // Columns: one per 59 px.
    let xs: Vec<f64> = cols.iter().map(|r| (r.0 + r.2) / 2.0).collect();
    let even = xs.windows(2).all(|w| (w[1] - w[0] - 59.0).abs() < 1.0);
    let ok = cols_sharp && angles && on_frame && one_path && even && segs.len() > 20;
    println!(
        "\nCHECK crate=layout name=isometric_lines result={} columns={} slanted={} columns_sharp={cols_sharp} thirty_degrees={angles} ends_on_frame={on_frame} one_path={one_path} columns_even={even}",
        pass(ok),
        cols.len(),
        segs.len()
    );
    assert!(ok);
}

#[test]
fn isometric_dots_sit_at_the_crossings_of_the_lines() {
    let lines = page_with(
        PatternRegion::new(Pattern::Isometric {
            style: IsoStyle::Lines,
        }),
        LineRole::Guide,
        None,
    );
    let dots = page_with(
        PatternRegion::new(Pattern::Isometric {
            style: IsoStyle::Dots,
        }),
        LineRole::DotGrid,
        None,
    );
    let d = rects(&items_of(&dots));
    let m = manta();
    let (w, _) = stroke(Style::Role(LineRole::DotGrid), &m);
    let square = d.iter().all(|r| {
        (r.2 - r.0 - f64::from(w)).abs() < 1e-9 && (r.3 - r.1 - f64::from(w)).abs() < 1e-9
    });
    // Each dot's centre lies on a vertical column, and where the column meets a slanted line: take
    // the column's rhythm x, and the slanted lines' y there.
    let segs = segments(&items_of(&lines));
    let cols_x: Vec<f64> = rects(&items_of(&lines))
        .iter()
        .map(|r| ((r.0 + r.2) / 2.0).floor())
        .collect();
    let on_line = |cx: f64, cy: f64| {
        segs.iter().any(|((ax, ay), (bx, by))| {
            let t = (cx - ax) / (bx - ax);
            (0.0..=1.0).contains(&t) && (ay + t * (by - ay) - cy).abs() <= 1.0
        })
    };
    let centres: Vec<(f64, f64)> = d
        .iter()
        .map(|r| ((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0))
        .collect();
    let on_cols = centres
        .iter()
        .all(|c| cols_x.iter().any(|x| (x - c.0.floor()).abs() < 1.0));
    let crossings = centres.iter().filter(|c| on_line(c.0, c.1)).count();
    let ok = square && on_cols && crossings * 10 >= centres.len() * 9 && centres.len() > 30;
    println!(
        "\nCHECK crate=layout name=isometric_dots result={} dots={} square={square} on_columns={on_cols} on_a_line={crossings}",
        pass(ok),
        centres.len()
    );
    assert!(ok);
}

#[test]
fn a_hex_edge_two_hexagons_share_is_drawn_once() {
    let mut all_ok = true;
    let mut parts = Vec::new();
    for (name, orientation) in [
        ("flat", HexOrientation::Flat),
        ("pointy", HexOrientation::Pointy),
    ] {
        let doc = page_with(
            PatternRegion::new(Pattern::Hex {
                orientation,
                height_units: 1,
            }),
            LineRole::Guide,
            None,
        );
        let items = items_of(&doc);
        let sharp = rects(&items);
        let segs = segments(&items);
        // An edge as an unordered pair of its (rounded to a tenth) ends.
        let key = |a: (f64, f64), b: (f64, f64)| {
            let r = |v: f64| (v * 10.0).round() as i64;
            let (p, q) = ((r(a.0), r(a.1)), (r(b.0), r(b.1)));
            if p <= q { (p, q) } else { (q, p) }
        };
        let mut keys: Vec<_> = segs.iter().map(|(a, b)| key(*a, *b)).collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        let slanted_once = keys.len() == n && n > 20;
        // The flat edges: horizontal for flat top, vertical for pointy top, on whole pixels, and
        // no two the same.
        let sharp_whole = !sharp.is_empty()
            && sharp
                .iter()
                .all(|r| [r.0, r.1, r.2, r.3].iter().all(|v| whole(*v)));
        let mut spans: Vec<_> = sharp
            .iter()
            .map(|r| (r.0 as i64, r.1 as i64, r.2 as i64, r.3 as i64))
            .collect();
        let m = spans.len();
        spans.sort_unstable();
        spans.dedup();
        let sharp_once = spans.len() == m;
        // Orientation: flat top has wide sharp edges, pointy top tall ones.
        let shape = sharp.iter().all(|r| {
            if orientation == HexOrientation::Flat {
                r.2 - r.0 >= r.3 - r.1
            } else {
                r.3 - r.1 >= r.2 - r.0
            }
        });
        // Every edge is cut at the region's frame: a flat edge's length and centre line, and a
        // slanted edge's two ends, lie inside it (chris's look: no stubs past the frame).
        let (fx0, fy0, fx1, fy1) = region_px();
        let eps = 1e-6;
        let sharp_in = sharp.iter().all(|r| {
            let (cx, cy) = ((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0);
            if orientation == HexOrientation::Flat {
                r.0 >= fx0 && r.2 <= fx1 && cy >= fy0 && cy <= fy1
            } else {
                r.1 >= fy0 && r.3 <= fy1 && cx >= fx0 && cx <= fx1
            }
        });
        let slanted_in = segs.iter().all(|(a, b)| {
            [a, b].iter().all(|p| {
                p.0 >= fx0 - eps && p.0 <= fx1 + eps && p.1 >= fy0 - eps && p.1 <= fy1 + eps
            })
        });
        // And the frame is reached on every side, by a slanted edge or a flat one.
        let min_x = segs
            .iter()
            .map(|(a, b)| a.0.min(b.0))
            .chain(sharp.iter().map(|r| r.0))
            .fold(f64::MAX, f64::min);
        let max_x = segs
            .iter()
            .map(|(a, b)| a.0.max(b.0))
            .chain(sharp.iter().map(|r| r.2))
            .fold(f64::MIN, f64::max);
        let min_y = segs
            .iter()
            .map(|(a, b)| a.1.min(b.1))
            .chain(sharp.iter().map(|r| r.1))
            .fold(f64::MAX, f64::min);
        let max_y = segs
            .iter()
            .map(|(a, b)| a.1.max(b.1))
            .chain(sharp.iter().map(|r| r.3))
            .fold(f64::MIN, f64::max);
        let touches =
            min_x <= fx0 + 1.0 && max_x >= fx1 - 1.0 && min_y <= fy0 + 1.0 && max_y >= fy1 - 1.0;
        // A slanted edge is 60 degrees from the flat ones: |dy / dx| is tan 60 (flat top) or
        // tan 30 (pointy top).
        let want = if orientation == HexOrientation::Flat {
            3f64.sqrt()
        } else {
            1.0 / 3f64.sqrt()
        };
        let angles = segs
            .iter()
            .all(|((ax, ay), (bx, by))| (((by - ay) / (bx - ax)).abs() - want).abs() < 1e-6);
        let ok = slanted_once
            && sharp_whole
            && sharp_once
            && shape
            && angles
            && sharp_in
            && slanted_in
            && touches;
        all_ok &= ok;
        parts.push(format!(
            "{name}:slanted={n}:sharp={m}:slanted_once={slanted_once}:sharp_whole={sharp_whole}:sharp_once={sharp_once}:shape={shape}:angles={angles}:cut_at_frame={}",
            sharp_in && slanted_in && touches
        ));
    }
    println!(
        "\nCHECK crate=layout name=hex_edges result={} {}",
        pass(all_ok),
        parts.join(" ")
    );
    assert!(all_ok);
}

#[test]
fn a_hexagon_is_as_high_as_its_whole_rhythm_units() {
    // Flat top, 2 units: the rows of sharp edges sit 2 * 59 px apart in a column.
    let doc = page_with(
        PatternRegion::new(Pattern::Hex {
            orientation: HexOrientation::Flat,
            height_units: 2,
        }),
        LineRole::Guide,
        None,
    );
    let r = rects(&items_of(&doc));
    let mut ys: Vec<f64> = r.iter().filter(|r| r.0 < 400.0).map(|r| r.1).collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup();
    // Edges of the even columns are 118 apart; the odd columns' are half a height (59) off.
    let steps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    let ok = !steps.is_empty() && steps.iter().all(|s| (*s - 59.0).abs() <= 1.0);
    println!(
        "\nCHECK crate=layout name=hex_height_units result={} edge_rows={} steps_px={:?}",
        pass(ok),
        ys.len(),
        steps.iter().take(4).collect::<Vec<_>>()
    );
    assert!(ok);
}

#[test]
fn cornell_has_a_cue_column_a_summary_band_and_structure_dividers() {
    let doc = page_with(
        PatternRegion::new(Pattern::Cornell {
            cue_units: Some(4),
            summary_units: Some(3),
        }),
        LineRole::Guide,
        None,
    );
    let m = manta();
    let (_, guide) = stroke(Style::Role(LineRole::Guide), &m);
    let (sw, structure) = stroke(Style::Role(LineRole::Structure), &m);
    let r = rects(&items_of(&doc));
    let (x0, _, x1, y1) = region_px();
    let dividers: Vec<_> = r
        .iter()
        .filter(|r| r.4 == structure && (r.3 - r.1 >= f64::from(sw)))
        .collect();
    let vertical: Vec<_> = dividers.iter().filter(|r| r.3 - r.1 > r.2 - r.0).collect();
    let horizontal: Vec<_> = dividers.iter().filter(|r| r.2 - r.0 > r.3 - r.1).collect();
    // One vertical divider, 4 columns in from the first rhythm column at or after the left edge.
    let col0 = (94.0 + 59.0 * ((x0 - 94.0) / 59.0).ceil()).round();
    let v_ok = vertical.len() == 1
        && ((vertical[0].0 + vertical[0].2) / 2.0 - (col0 + 4.0 * 59.0)).abs() <= 1.0;
    // One horizontal divider across the whole width, with 3 ruled rows below it.
    let h_ok = horizontal.len() == 1 && horizontal[0].0 == x0 && horizontal[0].2 == x1;
    let div_y = (horizontal[0].1 + horizontal[0].3) / 2.0;
    let ruling = |r: &&(f64, f64, f64, f64, u8)| {
        r.4 == guide && (r.3 - r.1 - 1.0).abs() < 1e-9 && (r.2 - r.0) > 2.0
    };
    let below = r.iter().filter(ruling).filter(|r| r.1 > div_y).count();
    // The notes rows start at the cue divider; the cue column has no ruling.
    let notes = r
        .iter()
        .filter(ruling)
        .filter(|r| r.1 < div_y)
        .all(|r| r.0 >= (vertical[0].0 + vertical[0].2) / 2.0 - 1.0);
    let summary_rows = below == 3 || below == 2;
    let ok = v_ok && h_ok && notes && summary_rows && y1 > div_y;
    println!(
        "\nCHECK crate=layout name=cornell result={} vertical_dividers={} horizontal_dividers={} cue_at_4_units={v_ok} full_width_divider={h_ok} cue_unruled={notes} summary_rows={below}",
        pass(ok),
        vertical.len(),
        horizontal.len()
    );
    assert!(ok);
}

#[test]
fn cornell_defaults_are_30_percent_and_20_percent_in_whole_units() {
    let m = manta();
    let (x0, y0, x1, y1) = region_px();
    let (cue, sum) = pattern::cornell_units(None, None, (x1 - x0, y1 - y0), (59.0, 59.0));
    let _ = m;
    // 100 mm = 1181 px wide: 30 % is 354 px = 6 units. 80 mm = 945 px tall: 20 % is 189 px = 3 units.
    let ok = (cue, sum) == (6, 3);
    println!(
        "\nCHECK crate=layout name=cornell_defaults result={} cue_units={cue} summary_units={sum}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_music_staff_has_five_even_lines_starting_on_a_rhythm_row() {
    let doc = page_with(
        PatternRegion::new(Pattern::MusicStaff {
            gap_mm: 2.0,
            pitch_units: 3,
            grand: false,
        }),
        LineRole::Guide,
        None,
    );
    let r = rects(&items_of(&doc));
    let mut ys: Vec<f64> = r.iter().map(|r| (r.1 + r.3) / 2.0).collect();
    ys.sort_by(f64::total_cmp);
    // 2 mm at 300 PPI is 23.6, rounded to whole pixels: 24.
    let gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    let in_staff = gaps.iter().filter(|g| (**g - 24.0).abs() <= 1.0).count();
    let staves = ys.len() / 5;
    // Each staff's top line is on a rhythm row (origin 94 px, unit 59).
    let tops_on_rhythm = (0..staves).all(|s| {
        ((ys[s * 5].floor() - 94.0) % 59.0).abs() <= 1.0
            || ((ys[s * 5].floor() - 94.0) % 59.0 - 59.0).abs() <= 1.0
    });
    // Staves are 3 units (177 px) apart.
    let pitch_ok =
        (0..staves.saturating_sub(1)).all(|s| (ys[(s + 1) * 5] - ys[s * 5] - 177.0).abs() <= 1.0);
    let ok = ys.len().is_multiple_of(5)
        && staves >= 2
        && in_staff == staves * 4
        && tops_on_rhythm
        && pitch_ok;
    println!(
        "\nCHECK crate=layout name=music_staff result={} lines={} staves={staves} lines_24px_apart={in_staff} tops_on_rhythm={tops_on_rhythm} pitch_177px={pitch_ok}",
        pass(ok),
        ys.len()
    );
    assert!(ok);
}

#[test]
fn a_grand_staff_pairs_the_staves_with_a_brace() {
    let doc = page_with(
        PatternRegion::new(Pattern::MusicStaff {
            gap_mm: 2.0,
            pitch_units: 6,
            grand: true,
        }),
        LineRole::Guide,
        None,
    );
    let m = manta();
    let (_, structure) = stroke(Style::Role(LineRole::Structure), &m);
    let r = rects(&items_of(&doc));
    let brace: Vec<_> = r
        .iter()
        .filter(|r| r.4 == structure && r.3 - r.1 > r.2 - r.0)
        .collect();
    let mut ys: Vec<f64> = r
        .iter()
        .filter(|r| r.4 != structure || r.2 - r.0 > r.3 - r.1)
        .map(|r| (r.1 + r.3) / 2.0)
        .collect();
    ys.sort_by(f64::total_cmp);
    // Ten lines a pair; the gap between the two staves is 59 px more than a gap's 24.
    let pairs = ys.len() / 10;
    let first: Vec<f64> = ys[..10].windows(2).map(|w| w[1] - w[0]).collect();
    let between = first[4];
    // The bass staff starts one rhythm unit (59 px) below the treble's last line.
    let between_ok = (between - 59.0).abs() <= 1.0;
    let brace_ok = brace.len() == pairs
        && brace.iter().enumerate().all(|(i, b)| {
            b.1 <= ys[10 * i] + 1.0
                && b.3 >= ys[10 * i + 9] - 1.0
                && ((b.0 + b.2) / 2.0 - region_px().0).abs() <= 1.0
        });
    let ok = pairs >= 1 && ys.len().is_multiple_of(10) && between_ok && brace_ok;
    println!(
        "\nCHECK crate=layout name=grand_staff result={} pairs={pairs} braces={} gap_between_staves_px={between} brace_spans_pair={brace_ok}",
        pass(ok),
        brace.len()
    );
    assert!(ok);
}

#[test]
fn major_lines_use_the_major_role_on_the_nth_line_from_the_origin() {
    let m = manta();
    let (mw, mgray) = stroke(Style::Role(LineRole::Structure), &m);
    let (gw, ggray) = stroke(Style::Role(LineRole::Guide), &m);
    let rows = |doc: &Document| -> Vec<(f64, f64, u8)> {
        rects(&items_of(doc))
            .iter()
            .filter(|r| r.2 - r.0 > r.3 - r.1)
            .map(|r| ((r.1 + r.3) / 2.0, r.3 - r.1, r.4))
            .collect()
    };
    // Off by default: no row is major.
    let none = rows(&page_with(
        PatternRegion::new(Pattern::Lines),
        LineRole::Guide,
        None,
    ));
    let off_ok = none.iter().all(|r| r.2 == ggray && r.1 == f64::from(gw));
    // The rhythm's N = 4: rows 0, 4, 8 (from the origin at 94 px) are major.
    let follow = rows(&page_with(
        PatternRegion::new(Pattern::Lines),
        LineRole::Guide,
        Some(4),
    ));
    let is_major =
        |y: f64, n: f64| (((y.floor() - 94.0) / 59.0).round() as i64).rem_euclid(n as i64) == 0;
    let follow_ok = follow
        .iter()
        .all(|r| (r.2 == mgray && r.1 == f64::from(mw)) == is_major(r.0, 4.0))
        && follow.iter().any(|r| r.2 == mgray && r.1 == f64::from(mw));
    // A region turns them off, or has its own N.
    let mut off = PatternRegion::new(Pattern::Lines);
    off.major = MajorLines::Off;
    let off_region = rows(&page_with(off, LineRole::Guide, Some(4)));
    let region_off_ok = off_region.iter().all(|r| r.2 == ggray);
    let mut own = PatternRegion::new(Pattern::Lines);
    own.major = MajorLines::Every(3);
    own.major_role = LineRole::Emphasis;
    let (ew, egray) = stroke(Style::Role(LineRole::Emphasis), &m);
    let own_rows = rows(&page_with(own, LineRole::Guide, Some(4)));
    let own_ok = own_rows
        .iter()
        .all(|r| (r.2 == egray && r.1 == f64::from(ew)) == is_major(r.0, 3.0))
        && own_rows
            .iter()
            .any(|r| r.2 == egray && r.1 == f64::from(ew));
    // Grid: rows and columns both. Dots: a dot is major when its row and column both are.
    let g = rects(&items_of(&page_with(
        PatternRegion::new(Pattern::Grid),
        LineRole::Guide,
        Some(4),
    )));
    let cols_major = g
        .iter()
        .filter(|r| r.3 - r.1 > r.2 - r.0 && r.4 == mgray)
        .count();
    let grid_ok = cols_major >= 2;
    let d = rects(&items_of(&page_with(
        PatternRegion::new(Pattern::Dots),
        LineRole::DotGrid,
        Some(4),
    )));
    // A major dot is a Dot grid dot one device pixel bigger, in the major role's gray.
    let (dot_w, dot_gray) = stroke(Style::Role(LineRole::DotGrid), &m);
    let big_w = f64::from(dot_w + 1);
    let big = d
        .iter()
        .filter(|r| r.4 == mgray && (r.2 - r.0) == big_w && (r.3 - r.1) == big_w)
        .count();
    let small = d
        .iter()
        .filter(|r| r.4 == dot_gray && (r.2 - r.0) == f64::from(dot_w))
        .count();
    let dots_ok = big >= 4 && big * 8 < d.len() && big + small == d.len() && dot_w + 1 == 4;
    // Isometric: every Nth line on each of the three axes; Cornell: its ruling; Hex and the
    // staff have none.
    let iso = items_of(&page_with(
        PatternRegion::new(Pattern::Isometric {
            style: IsoStyle::Lines,
        }),
        LineRole::Guide,
        Some(3),
    ));
    let iso_major_paths = iso
        .iter()
        .filter(|i| matches!(i, Item::StrokePath { width, .. } if *width == f64::from(mw)))
        .count();
    let iso_rows_major = rects(&iso)
        .iter()
        .filter(|r| (r.2 - r.0) == f64::from(mw))
        .count();
    let iso_ok = iso_major_paths == 1 && iso_rows_major >= 2;
    let hex = items_of(&page_with(
        PatternRegion::new(PatternKind::Hex.default_pattern()),
        LineRole::Guide,
        Some(3),
    ));
    let hex_plain = hex.iter().all(|i| match i {
        Item::Rect { gray, .. } | Item::StrokePath { gray, .. } => *gray == ggray,
        _ => true,
    });
    let ok =
        off_ok && follow_ok && region_off_ok && own_ok && grid_ok && dots_ok && iso_ok && hex_plain;
    println!(
        "\nCHECK crate=layout name=major_lines result={} off_by_default={off_ok} follows_rhythm={follow_ok} region_off={region_off_ok} region_own_n_and_role={own_ok} grid={grid_ok} dots={dots_ok} isometric={iso_ok} hex_none={hex_plain}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_unbound_region_has_its_own_spacing_from_its_own_corner() {
    let mut pr = PatternRegion::new(Pattern::Grid);
    pr.bound = false;
    pr.h_spacing_mm = Some(4.0);
    pr.v_spacing_mm = Some(10.0);
    let doc = page_with(pr, LineRole::Guide, None);
    let r = rects(&items_of(&doc));
    let (x0, y0, _, _) = region_px();
    let mut ys: Vec<f64> = r
        .iter()
        .filter(|r| r.2 - r.0 > r.3 - r.1)
        .map(|r| (r.1 + r.3) / 2.0)
        .collect();
    let mut xs: Vec<f64> = r
        .iter()
        .filter(|r| r.3 - r.1 > r.2 - r.0)
        .map(|r| (r.0 + r.2) / 2.0)
        .collect();
    ys.sort_by(f64::total_cmp);
    xs.sort_by(f64::total_cmp);
    // 4 mm = 47.2 -> 47 px, 10 mm = 118.1 -> 118 px (pixel-locked), the first line on the region's
    // top-left corner.
    let dy: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    let dx: Vec<f64> = xs.windows(2).map(|w| w[1] - w[0]).collect();
    let spaced =
        dy.iter().all(|d| (*d - 118.0).abs() < 1.0) && dx.iter().all(|d| (*d - 47.0).abs() < 1.0);
    let corner = (ys[0] - y0).abs() <= 1.0 && (xs[0] - x0).abs() <= 1.0;
    // Bound again, the same region follows the rhythm (59 px).
    let bound = page_with(PatternRegion::new(Pattern::Grid), LineRole::Guide, None);
    let rb = rects(&items_of(&bound));
    let mut ysb: Vec<f64> = rb
        .iter()
        .filter(|r| r.2 - r.0 > r.3 - r.1)
        .map(|r| (r.1 + r.3) / 2.0)
        .collect();
    ysb.sort_by(f64::total_cmp);
    let bound_ok = ysb.windows(2).all(|w| (w[1] - w[0] - 59.0).abs() < 1.0);
    let ok = spaced && corner && bound_ok;
    println!(
        "\nCHECK crate=layout name=unbound_region result={} rows_118px_apart_cols_47px_apart={spaced} first_line_on_corner={corner} bound_follows_rhythm={bound_ok}",
        pass(ok)
    );
    assert!(ok);
}
