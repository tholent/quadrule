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

//! Align and distribute, the maths (rung 7b, docs/bbq/2026-11-11-rung-7-groups-and-align.md, Q5
//! and Q6). Each test prints one structured `CHECK` line starting with a newline (run with
//! `-- --nocapture`).

use quadrule_layout::align::*;
use quadrule_layout::units::Axis;
use quadrule_model::Frame;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn moved(f: Frame, dx: f64, dy: f64) -> Frame {
    Frame::new(f.x_mm + dx, f.y_mm + dy, f.w_mm, f.h_mm)
}

/// Three boxes that differ in place and size.
fn three() -> [Frame; 3] {
    [
        Frame::new(10.0, 20.0, 30.0, 10.0),
        Frame::new(50.0, 5.0, 20.0, 40.0),
        Frame::new(100.0, 60.0, 10.0, 10.0),
    ]
}

#[test]
fn every_align_edge_lands_on_the_target_in_exact_millimetres() {
    let b = three();
    let sel = union_of(&b).unwrap();
    let page = Frame::new(0.0, 0.0, 148.0, 210.0);
    let mut ok = true;
    let mut checked = 0;
    for target in [sel, page] {
        for axis in [Axis::X, Axis::Y] {
            for edge in [Edge::Start, Edge::Centre, Edge::End] {
                let d = align_deltas(&b, axis, edge, target, Grid::Mm);
                for (f, d) in b.iter().zip(&d) {
                    let g = match axis {
                        Axis::X => moved(*f, *d, 0.0),
                        Axis::Y => moved(*f, 0.0, *d),
                    };
                    let (lo, len, t_lo, t_len) = match axis {
                        Axis::X => (g.x_mm, g.w_mm, target.x_mm, target.w_mm),
                        Axis::Y => (g.y_mm, g.h_mm, target.y_mm, target.h_mm),
                    };
                    let got = match edge {
                        Edge::Start => lo,
                        Edge::Centre => lo + len / 2.0,
                        Edge::End => lo + len,
                    };
                    let want = match edge {
                        Edge::Start => t_lo,
                        Edge::Centre => t_lo + t_len / 2.0,
                        Edge::End => t_lo + t_len,
                    };
                    ok &= near(got, want);
                    checked += 1;
                }
            }
        }
    }
    println!(
        "\nCHECK crate=layout name=align_edges_mm result={} edges_checked={checked}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn distribute_spacing_and_centres_in_exact_millimetres() {
    // Out of order on purpose: the order given does not matter, the place along the axis does.
    let b = [
        Frame::new(100.0, 0.0, 10.0, 10.0),
        Frame::new(0.0, 0.0, 20.0, 10.0),
        Frame::new(30.0, 0.0, 40.0, 10.0),
        Frame::new(80.0, 0.0, 5.0, 10.0),
    ];
    let sp = distribute_deltas(&b, Axis::X, Spread::Spacing, Grid::Mm);
    let mut spans: Vec<(f64, f64)> = b
        .iter()
        .zip(&sp)
        .map(|(f, d)| (f.x_mm + d, f.x_mm + d + f.w_mm))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let gaps: Vec<f64> = spans.windows(2).map(|w| w[1].0 - w[0].1).collect();
    let equal = gaps.iter().all(|g| near(*g, gaps[0]));
    let ends_stay = near(sp[0], 0.0) && near(sp[1], 0.0);
    // 110 - 0 - (10 + 20 + 40 + 5) = 35 over 3 gaps.
    let value = near(gaps[0], 35.0 / 3.0);

    let ce = distribute_deltas(&b, Axis::X, Spread::Centres, Grid::Mm);
    let mut centres: Vec<f64> = b
        .iter()
        .zip(&ce)
        .map(|(f, d)| f.x_mm + d + f.w_mm / 2.0)
        .collect();
    centres.sort_by(f64::total_cmp);
    let steps: Vec<f64> = centres.windows(2).map(|w| w[1] - w[0]).collect();
    let centres_equal = steps.iter().all(|s| near(*s, steps[0])) && near(centres[0], 10.0);

    // Vertical: boxes stacked with different heights.
    let v = [
        Frame::new(0.0, 0.0, 10.0, 10.0),
        Frame::new(0.0, 15.0, 10.0, 30.0),
        Frame::new(0.0, 100.0, 10.0, 20.0),
    ];
    let vs = distribute_deltas(&v, Axis::Y, Spread::Spacing, Grid::Mm);
    let g1 = (v[1].y_mm + vs[1]) - (v[0].y_mm + vs[0] + v[0].h_mm);
    let g2 = (v[2].y_mm + vs[2]) - (v[1].y_mm + vs[1] + v[1].h_mm);
    let vertical = near(g1, g2) && near(g1, 30.0);

    let two = distribute_deltas(&b[..2], Axis::X, Spread::Spacing, Grid::Mm);
    let few = two.iter().all(|d| *d == 0.0);
    let ok = equal && ends_stay && value && centres_equal && vertical && few;
    println!(
        "\nCHECK crate=layout name=distribute_mm result={} gaps_equal={equal} ends_stay={ends_stay} gap_is_35_over_3={value} centres_equal={centres_equal} vertical_gap_30={vertical} two_move_nothing={few}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn on_a_pixel_grid_edges_land_on_whole_pixels_and_gaps_spread_the_leftover() {
    let mut all = true;
    let mut report = String::new();
    for id in ["supernote.a5x2", "supernote.a6x2"] {
        let p = builtin(id).unwrap();
        let s = p.mm_to_px(1.0);
        let grid = Grid::Px(s);
        // Boxes in whole device pixels, 10 px wide, so their extents are exact.
        let px = |x: f64, w: f64| Frame::new(x / s, 3.3, w / s, 4.1);
        let b = [
            px(0.4, 10.0),
            px(20.7, 10.0),
            px(31.2, 10.0),
            px(52.6, 10.0),
        ];
        // Aligned: the left edge of every box is on a pixel, whatever target.
        let sel = union_of(&b).unwrap();
        let d = align_deltas(&b, Axis::X, Edge::Start, sel, grid);
        let left_ok = b
            .iter()
            .zip(&d)
            .all(|(f, d)| ((f.x_mm + d) * s - ((f.x_mm + d) * s).round()).abs() < 1e-6);
        let d = align_deltas(&b, Axis::X, Edge::End, sel, grid);
        let right_ok = b.iter().zip(&d).all(|(f, d)| {
            let r = (f.x_mm + d) * s + 10.0;
            (r - r.round()).abs() < 1e-6
        });
        let d = align_deltas(&b, Axis::Y, Edge::Centre, sel, grid);
        let mid_ok = b.iter().zip(&d).all(|(f, d)| {
            let top = (f.y_mm + d) * s;
            (top - top.round()).abs() < 1e-6
        });
        // Distribute: first at round(0.4) = 0, last's right edge at round(62.6) = 63; 63 - 40 = 23
        // over 3 gaps is 7 each with 2 left over, so the first two gaps get a pixel more: 8, 8, 7.
        let d = distribute_deltas(&b, Axis::X, Spread::Spacing, grid);
        let mut at: Vec<f64> = b.iter().zip(&d).map(|(f, d)| (f.x_mm + d) * s).collect();
        at.sort_by(f64::total_cmp);
        let whole = at.iter().all(|a| (a - a.round()).abs() < 1e-6);
        let gaps: Vec<f64> = at
            .windows(2)
            .map(|w| (w[1] - w[0] - 10.0).round())
            .collect();
        let spread = gaps == vec![8.0, 8.0, 7.0];
        all &= left_ok && right_ok && mid_ok && whole && spread;
        report += &format!(
            " {id}_left={left_ok} {id}_right={right_ok} {id}_middle={mid_ok} {id}_whole={whole} {id}_gaps_8_8_7={spread}"
        );
    }
    println!(
        "\nCHECK crate=layout name=align_distribute_pixels result={}{report}",
        pass(all)
    );
    assert!(all);
}

#[test]
fn the_page_and_margins_boxes_and_the_align_to_keys() {
    let doc = quadrule_model::sample::slice_document();
    let p = builtin(&doc.primary_profile_id).unwrap();
    let (pw, ph) = p.page_mm();
    let page = target_box(&doc, &p, AlignTo::Page).unwrap();
    let m = doc.page.margins_mm;
    let margins = target_box(&doc, &p, AlignTo::Margins).unwrap();
    let ok_page = near(page.w_mm, pw) && near(page.h_mm, ph) && page.x_mm == 0.0;
    let ok_margins = near(margins.x_mm, m.left)
        && near(margins.y_mm, m.top)
        && near(margins.right(), pw - m.right)
        && near(margins.bottom(), ph - m.bottom);
    let none_for_selection = target_box(&doc, &p, AlignTo::Selection).is_none();
    let keys = [AlignTo::Selection, AlignTo::Page, AlignTo::Margins]
        .into_iter()
        .all(|a| AlignTo::from_key(a.key()) == Some(a))
        && AlignTo::from_key("nope").is_none();
    let ok = ok_page && ok_margins && none_for_selection && keys;
    println!(
        "\nCHECK crate=layout name=align_targets result={} page={ok_page} margins={ok_margins} selection_has_no_box={none_for_selection} keys_round_trip={keys}",
        pass(ok)
    );
    assert!(ok);
}
