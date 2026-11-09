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

//! Layout checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn profiles() -> (Profile, Profile) {
    (builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap())
}

fn page(doc: &Document, target: &Profile) -> DisplayList {
    let (manta, _) = profiles();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &manta, target, &mut reg).unwrap();
    display_list(doc, &manta, target, &laid.outlines).unwrap()
}

fn rects(dl: &DisplayList) -> Vec<(f64, f64, f64, f64, u8)> {
    dl.items
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

#[test]
fn constraints_resolve() {
    // A 10 mm box at x = 20 on a 100 mm page, moved to a 50 mm page.
    let f = Frame::new(20.0, 20.0, 10.0, 10.0);
    let (from, to) = ((100.0, 100.0), (50.0, 50.0));
    let h = |h| {
        resolve_frame(
            f,
            Constraints {
                h,
                v: VConstraint::Top,
            },
            from,
            to,
        )
    };
    let v = |v| {
        resolve_frame(
            f,
            Constraints {
                h: HConstraint::Left,
                v,
            },
            from,
            to,
        )
    };
    let cases = [
        (
            "left",
            (h(HConstraint::Left).x_mm, h(HConstraint::Left).w_mm),
            (20.0, 10.0),
        ),
        (
            "right",
            (h(HConstraint::Right).x_mm, h(HConstraint::Right).w_mm),
            (-30.0, 10.0),
        ),
        (
            "left_right",
            (
                h(HConstraint::LeftRight).x_mm,
                h(HConstraint::LeftRight).w_mm,
            ),
            (20.0, -40.0),
        ),
        (
            "center",
            (h(HConstraint::Center).x_mm, h(HConstraint::Center).w_mm),
            (-5.0, 10.0),
        ),
        (
            "scale",
            (h(HConstraint::Scale).x_mm, h(HConstraint::Scale).w_mm),
            (10.0, 5.0),
        ),
        (
            "top",
            (v(VConstraint::Top).y_mm, v(VConstraint::Top).h_mm),
            (20.0, 10.0),
        ),
        (
            "bottom",
            (v(VConstraint::Bottom).y_mm, v(VConstraint::Bottom).h_mm),
            (-30.0, 10.0),
        ),
        (
            "top_bottom",
            (
                v(VConstraint::TopBottom).y_mm,
                v(VConstraint::TopBottom).h_mm,
            ),
            (20.0, -40.0),
        ),
        (
            "v_center",
            (v(VConstraint::Center).y_mm, v(VConstraint::Center).h_mm),
            (-5.0, 10.0),
        ),
        (
            "v_scale",
            (v(VConstraint::Scale).y_mm, v(VConstraint::Scale).h_mm),
            (10.0, 5.0),
        ),
    ];
    let fails: Vec<&str> = cases
        .iter()
        .filter(|(_, got, want)| got != want)
        .map(|(n, _, _)| *n)
        .collect();
    println!(
        "CHECK crate=layout name=constraints_resolve result={} cases={} fails={fails:?}",
        pass(fails.is_empty()),
        cases.len()
    );
    assert!(fails.is_empty());
}

#[test]
fn snap_odd_on_centres_even_on_edges() {
    let ok = snap_center(10.3, 1) == 10.5
        && snap_center(10.3, 3) == 10.5
        && snap_center(10.3, 2) == 10.0
        && snap_center(10.6, 4) == 11.0;
    println!(
        "CHECK crate=layout name=snap_center result={} cases=4",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn rhythm_resolves_per_profile() {
    // 6 mm at 300 PPI = 70.87 px, pixel-locked to 71; the 8 mm margin origin = 94.49 → 94.
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let (m, n) = (RhythmPx::new(&doc, &manta), RhythmPx::new(&doc, &nomad));
    let ok =
        m.unit_y == 71.0 && m.unit_x == 71.0 && m.origin_x == 94.0 && m.origin_y == 94.0 && n == m;
    println!(
        "CHECK crate=layout name=rhythm_px result={} unit_px={} origin_px=({},{})",
        pass(ok),
        m.unit_y,
        m.origin_x,
        m.origin_y
    );
    assert!(ok);
}

#[test]
fn physical_locked_rhythm_varies_by_one_pixel() {
    let mut doc = slice_document();
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    let (manta, _) = profiles();
    let r = RhythmPx::new(&doc, &manta);
    let ys = RhythmPx::positions(r.origin_y, r.unit_y, 0.0, 2560.0);
    let mut gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    gaps.sort_by(f64::total_cmp);
    gaps.dedup();
    let ok = gaps == [70.0, 71.0];
    println!(
        "CHECK crate=layout name=physical_locked_gaps result={} gaps_px={gaps:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn slice_page_snaps_to_whole_pixels() {
    // Every hard-edged rectangle sits on whole pixels and is exactly its role's width.
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let mut parts = Vec::new();
    let mut ok = true;
    for p in [&manta, &nomad] {
        let dl = page(&doc, p);
        let rs = rects(&dl);
        let whole = rs
            .iter()
            .all(|r| [r.0, r.1, r.2, r.3].iter().all(|v| v.fract() == 0.0));
        // Roles may share a gray (on the Nomad, Dot grid, Guide and Structure are all 51), so
        // shapes are told apart by form: dots are squares of the dot size, lines are wide, and
        // the title rule is the one wide rect of the Structure width.
        let (dot, guide, structure) = (
            p.role(LineRole::DotGrid),
            p.role(LineRole::Guide),
            p.role(LineRole::Structure),
        );
        let d = dot.width_px as f64;
        let dots = rs
            .iter()
            .filter(|r| r.4 == dot.gray && r.2 - r.0 == d && r.3 - r.1 == d)
            .count();
        let is_rule = |r: &&(f64, f64, f64, f64, u8)| {
            r.4 == structure.gray
                && structure.width_px != guide.width_px
                && r.3 - r.1 == structure.width_px as f64
        };
        let lines: Vec<_> = rs
            .iter()
            .filter(|r| r.4 == guide.gray && r.2 - r.0 > r.3 - r.1)
            .filter(|r| !is_rule(r))
            .collect();
        let line_w_ok = lines
            .iter()
            .all(|r| r.3 - r.1 == p.role(LineRole::Guide).width_px as f64);
        let fills = dl
            .items
            .iter()
            .filter(|i| matches!(i, Item::Fill { .. }))
            .count();
        ok &= whole && line_w_ok && dots > 0 && !lines.is_empty() && fills == 1;
        parts.push(format!(
            "{}:rects={}:dots={dots}:lines={}:text_fills={fills}:whole_px={whole}",
            p.id,
            rs.len(),
            lines.len()
        ));
    }
    println!(
        "CHECK crate=layout name=slice_whole_pixels result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn pinned_header_keeps_physical_distance() {
    // The title rule is pinned to the top (VConstraint::Top), so it sits at the same physical
    // height on both profiles; both are 300 PPI, so the same place. The rule's width differs by
    // profile (odd widths centre on a pixel, even on an edge), so centres agree to half a pixel.
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let rule_y = |p: &Profile| {
        let s = p.role(LineRole::Structure);
        rects(&page(&doc, p))
            .into_iter()
            .find(|r| r.4 == s.gray && r.3 - r.1 == s.width_px as f64 && r.2 - r.0 > r.3 - r.1)
            .map(|r| (r.1 + r.3) / 2.0)
    };
    let (m, n) = (rule_y(&manta), rule_y(&nomad));
    let ok = matches!((m, n), (Some(a), Some(b)) if (a - b).abs() <= 0.5);
    println!(
        "CHECK crate=layout name=pinned_header result={} rule_centre_manta={m:?} rule_centre_nomad={n:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn bad_outlines_are_errors_not_panics() {
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let mut reg = FontRegistry::bundled_only();
    let good = lay_out_text(&doc, &manta, &manta, &mut reg)
        .unwrap()
        .outlines;

    let wrong_profile = display_list(&doc, &manta, &nomad, &good);
    let empty = ProfileOutlines {
        texts: vec![],
        ..good.clone()
    };
    let missing_text = display_list(&doc, &manta, &manta, &empty);
    let mut no_tables = good.clone();
    no_tables.tables.clear();
    let missing_table = display_list(&doc, &manta, &manta, &no_tables);
    let mut broken = good.clone();
    for s in broken.tables[0].glyphs.values_mut() {
        s.coords.pop();
    }
    let malformed = display_list(&doc, &manta, &manta, &broken);

    let ok = matches!(wrong_profile, Err(LayoutError::WrongProfile { .. }))
        && matches!(missing_text, Err(LayoutError::MissingText(_)))
        && matches!(missing_table, Err(LayoutError::BadOutlines(_)))
        && matches!(malformed, Err(LayoutError::BadOutlines(_)));
    println!(
        "CHECK crate=layout name=bad_outlines_errors result={} cases=4",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn hidden_objects_and_layers_are_not_drawn() {
    let mut doc = slice_document();
    let (manta, _) = profiles();
    let all = page(&doc, &manta).items.len();
    doc.layers[0].visible = false; // the grid
    let no_grid = page(&doc, &manta).items.len();
    doc.layers[1].objects[0].visible = false; // the title
    let no_title = page(&doc, &manta).items.len();
    let ok = no_grid == 2 && no_title == 1 && all > no_grid;
    println!(
        "CHECK crate=layout name=hidden_not_drawn result={} items_all={all} no_grid={no_grid} no_title={no_title}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn pattern_shapes_are_centred_in_their_region_and_drawn_whole() {
    // Every line and dot has its centre inside its region and reaches at most half a stroke past
    // the edge; the slice's first dot row and column sit exactly on the region's top-left edge.
    let doc = slice_document();
    let (manta, _) = profiles();
    let dl = page(&doc, &manta);
    let px = |mm: f64| (mm * 300.0 / 25.4).round();
    let region = |i: usize| {
        let f = doc.layers[0].objects[i].frame;
        (
            px(f.x_mm),
            px(f.y_mm),
            px(f.x_mm + f.w_mm),
            px(f.y_mm + f.h_mm),
        )
    };
    let (dots, lines) = (region(0), region(1));
    let mut checked = 0;
    let mut on_edge = 0;
    let mut bad = 0;
    for (r, role) in [(dots, LineRole::DotGrid), (lines, LineRole::Guide)] {
        let style = manta.role(role);
        let half = style.width_px as f64 / 2.0;
        for (x0, y0, x1, y1, _) in rects(&dl).into_iter().filter(|q| q.4 == style.gray) {
            let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
            checked += 1;
            let centred =
                cy >= r.1 && cy <= r.3 && (role == LineRole::Guide || (cx >= r.0 && cx <= r.2));
            let whole = y1 - y0 == style.width_px as f64;
            let within = y0 >= r.1 - half && y1 <= r.3 + half;
            if y0 < r.1 || x0 < r.0 {
                on_edge += 1;
            }
            if !(centred && whole && within) {
                bad += 1;
            }
        }
    }
    let ok = bad == 0 && on_edge > 0 && checked > 0;
    println!(
        "CHECK crate=layout name=pattern_shapes_centred_whole result={} shapes={checked} straddling_edge={on_edge} bad={bad}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn centred_rhythm_balances_the_margin_box() {
    // 5 mm at 300 PPI = 59 px. With no margins, floor(L / 59) positions span (n - 1) units and the
    // leftover splits evenly, an odd pixel to the end: Manta 32 across from x 45, 43 down from y 41.
    let mut doc = slice_document();
    doc.page.margins_mm = Margins::all(0.0);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.origin = RhythmOrigin::Centered;
    let (manta, nomad) = profiles();
    let mut parts = Vec::new();
    let mut ok = true;
    for (p, want) in [
        (&manta, (45.0, 41.0, 32, 43)),
        (&nomad, (53.0, 51.0, 23, 31)),
    ] {
        let r = RhythmPx::new(&doc, p);
        let xs = RhythmPx::positions(r.origin_x, r.unit_x, 0.0, p.width_px as f64 - 1.0);
        let ys = RhythmPx::positions(r.origin_y, r.unit_y, 0.0, p.height_px as f64 - 1.0);
        let got = (xs[0], ys[0], xs.len(), ys.len());
        let left = xs[0];
        let right = p.width_px as f64 - xs[xs.len() - 1];
        ok &= got == want && (left - right).abs() <= 1.0;
        parts.push(format!(
            "{}:first=({},{}):count=({},{})",
            p.id, got.0, got.1, got.2, got.3
        ));
    }
    println!(
        "CHECK crate=layout name=centred_rhythm result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn grid_pattern_draws_both_directions() {
    let mut doc = slice_document();
    let ObjectKind::Pattern(pr) = &mut doc.layers[0].objects[0].kind else {
        panic!("dots")
    };
    pr.pattern = Pattern::Grid;
    let (manta, _) = profiles();
    let style = manta.role(LineRole::DotGrid);
    let rs: Vec<_> = rects(&page(&doc, &manta))
        .into_iter()
        .filter(|r| r.4 == style.gray)
        .collect();
    let w = style.width_px as f64;
    let horizontal = rs
        .iter()
        .filter(|r| r.3 - r.1 == w && r.2 - r.0 > w)
        .count();
    let vertical = rs
        .iter()
        .filter(|r| r.2 - r.0 == w && r.3 - r.1 > w)
        .count();
    // The dot grid had 25 columns and 14 rows of dots on the Manta: the same rhythm crossings.
    let ok = horizontal == 14 && vertical == 25;
    println!(
        "CHECK crate=layout name=grid_pattern result={} horizontal={horizontal} vertical={vertical}",
        pass(ok)
    );
    assert!(ok);
}
