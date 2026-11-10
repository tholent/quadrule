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

//! Snap engine checks (rung 4 bbq, Defaulted "Tests"). Each test prints one structured `CHECK`
//! line, starting with a newline so `grep '^CHECK'` finds it (run with `-- --nocapture`).
//!
//! The sample page on the Manta (300 ppi): margins 8 mm = 94 px, rhythm 6 mm = 71 px from the
//! margin corner, so the rhythm lines run 94, 165, 236, 307, ... across and down.

use quadrule_layout::snap::*;
use quadrule_layout::units::Axis;
use quadrule_layout::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const ALL: SnapToggles = SnapToggles {
    pixel_grid: true,
    rhythm: true,
    margins_safe: true,
};

fn world_with(subdivisions: Option<u8>, zones: &[PxRect]) -> SnapWorld {
    let mut doc = slice_document();
    doc.rhythm.subdivisions = subdivisions;
    SnapWorld::new(&doc, &builtin(MANTA_ID).unwrap(), zones)
}

fn world() -> SnapWorld {
    world_with(None, &[])
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn rhythm_catches_within_reach() {
    let w = world();
    // A box whose left edge the pointer left 4 px past the line at x = 165, and whose top is far
    // from every line.
    let near = w.snap_span(Axis::X, 169.0, 269.0, REACH, ALL);
    // 7 px past: out of reach. The pixel grid still rounds a fractional edge.
    let far = w.snap_span(Axis::X, 172.4, 272.4, REACH, ALL);
    // Exactly at reach (6 px) still catches.
    let edge = w.snap_span(Axis::X, 171.0, 271.0, REACH, ALL);
    let ok = close(near.delta, -4.0)
        && near.guide.map(|g| (g.at, g.kind)) == Some((165.0, SnapKind::Rhythm))
        && far.guide.is_none()
        && close(far.delta, -0.4)
        && close(edge.delta, -6.0);
    println!(
        "\nCHECK crate=layout name=snap_rhythm_reach result={} near_delta={} far_guide={:?} far_delta={:.1} at_reach_delta={}",
        pass(ok),
        near.delta,
        far.guide.is_some(),
        far.delta,
        edge.delta
    );
    assert!(ok);
}

#[test]
fn centre_and_far_edge_snap_too() {
    let w = world();
    // The box's centre (304) is 3 px from the line at 307; its edges are far from any line.
    let centre = w.snap_span(Axis::X, 284.0, 324.0, REACH, ALL);
    // Right edge 2 px short of the line at 378 (= 94 + 4 * 71 = 378).
    let right = w.snap_span(Axis::X, 300.0, 376.0, REACH, ALL);
    let ok = close(centre.delta, 3.0) && close(right.delta, 2.0);
    println!(
        "\nCHECK crate=layout name=snap_centre_and_far_edge result={} centre_delta={} right_delta={}",
        pass(ok),
        centre.delta,
        right.delta
    );
    assert!(ok);
}

#[test]
fn nearest_wins_and_ties_go_to_rhythm() {
    let w = world();
    // Left edge at 97: the margin and the first rhythm line are both at x = 94, a tie, and
    // rhythm wins (napkin §6.2: rhythm first).
    let tie = w.snap_span(Axis::X, 97.0, 150.0, REACH, ALL);
    // With the rhythm off the margin line catches the same edge.
    let margin_only = w.snap_span(
        Axis::X,
        97.0,
        150.0,
        REACH,
        SnapToggles {
            rhythm: false,
            ..ALL
        },
    );
    // A safe-zone edge 2 px away beats a rhythm line 5 px away.
    let zone = PxRect {
        x0: 0.0,
        y0: 0.0,
        x1: 303.0,
        y1: 100.0,
    };
    let wz = world_with(None, &[zone]);
    let nearest = wz.snap_span(Axis::X, 250.0, 301.0, REACH, ALL);
    let ok = tie.guide.map(|g| g.kind) == Some(SnapKind::Rhythm)
        && margin_only.guide.map(|g| g.kind) == Some(SnapKind::Margin)
        && nearest.guide.map(|g| (g.kind, g.at)) == Some((SnapKind::SafeZone, 303.0))
        && close(nearest.delta, 2.0);
    println!(
        "\nCHECK crate=layout name=snap_priority result={} tie={:?} margin_only={:?} nearest={:?}",
        pass(ok),
        tie.guide.map(|g| g.kind),
        margin_only.guide.map(|g| g.kind),
        nearest.guide.map(|g| g.kind)
    );
    assert!(ok);
}

#[test]
fn margins_and_safe_zones_snap_edges_not_centres() {
    let zone = PxRect {
        x0: 0.0,
        y0: 0.0,
        x1: 200.0,
        y1: 2560.0,
    };
    let w = world_with(None, &[zone]);
    let off = SnapToggles {
        rhythm: false,
        ..ALL
    };
    // Right margin line = 1920 - 94 = 1826: a right edge 3 px short catches it.
    let right = w.snap_span(Axis::X, 1600.0, 1823.0, REACH, off);
    // A centre on the margin does not catch (the box is wide, so no edge is near).
    let centre = w.snap_span(Axis::X, 1726.0, 1926.0, REACH, off);
    // The zone's right edge, x = 200, catches a left edge 4 px past it.
    let zone_edge = w.snap_span(Axis::X, 204.0, 400.0, REACH, off);
    // Down the page: top margin at y = 94.
    let top = w.snap_span(Axis::Y, 91.0, 300.0, REACH, off);
    let ok = right.guide.map(|g| (g.kind, g.at)) == Some((SnapKind::Margin, 1826.0))
        && centre
            .guide
            .is_none_or(|g| g.kind != SnapKind::Margin || g.at != 1826.0)
        && zone_edge.guide.map(|g| (g.kind, g.at)) == Some((SnapKind::SafeZone, 200.0))
        && top.guide.map(|g| (g.kind, g.at)) == Some((SnapKind::Margin, 94.0));
    println!(
        "\nCHECK crate=layout name=snap_margins_and_zones result={} right={:?} centre_caught={} zone_edge={:?} top={:?}",
        pass(ok),
        right.guide.map(|g| g.at),
        centre.guide.is_some(),
        zone_edge.guide.map(|g| g.at),
        top.guide.map(|g| g.at)
    );
    assert!(ok);
}

#[test]
fn subdivisions_add_lines_between_units() {
    // Four per unit: lines every 17.75 px, rounded: 94, 112 (111.75), 130 (129.5), 147, 165.
    let lines = rhythm_lines(94.0, 71.0, Some(4), 94.0, 170.0);
    let positions: Vec<f64> = lines.iter().map(|l| l.0).collect();
    let majors: Vec<f64> = lines.iter().filter(|l| l.1).map(|l| l.0).collect();
    let sub = world_with(Some(4), &[]);
    let plain = world();
    // 112 is a subdivision: a left edge 3 px past it catches with subdivisions on, and nothing
    // catches without (the nearest unit lines, 94 and 165, are 21 and 10 px from the edges).
    let with = sub.snap_span(Axis::X, 115.0, 155.0, REACH, ALL);
    let without = plain.snap_span(Axis::X, 115.0, 155.0, REACH, ALL);
    let ok = positions == vec![94.0, 112.0, 130.0, 147.0, 165.0]
        && majors == vec![94.0, 165.0]
        && with.guide.map(|g| g.kind) == Some(SnapKind::Subdivision)
        && close(with.delta, -3.0)
        && without
            .guide
            .is_none_or(|g| g.kind != SnapKind::Subdivision);
    println!(
        "\nCHECK crate=layout name=snap_subdivisions result={} lines={positions:?} majors={majors:?} with={:?} without_subdivision={}",
        pass(ok),
        with.guide.map(|g| g.kind),
        without
            .guide
            .is_some_and(|g| g.kind == SnapKind::Subdivision)
    );
    assert!(ok);
}

#[test]
fn pixel_grid_rounds_when_nothing_caught_and_ctrl_turns_all_off() {
    let w = world();
    // 330.3 to 340.3 has no rhythm line within 6 px of its edges or its centre (307 is 23 px away).
    let rounded = w.snap_span(Axis::X, 330.3, 340.3, REACH, ALL);
    let only_grid = w.snap_span(
        Axis::X,
        330.3,
        340.3,
        REACH,
        SnapToggles {
            pixel_grid: true,
            rhythm: false,
            margins_safe: false,
        },
    );
    // Off: nothing moves, even a box on a line.
    let off = w.snap_span(Axis::X, 166.0, 266.4, REACH, SnapToggles::OFF);
    let no_grid = w.snap_span(
        Axis::X,
        330.3,
        340.3,
        REACH,
        SnapToggles {
            pixel_grid: false,
            ..ALL
        },
    );
    let ok = close(rounded.delta, -0.3)
        && rounded.guide.is_none()
        && close(only_grid.delta, -0.3)
        && off.delta == 0.0
        && off.guide.is_none()
        && no_grid.delta == 0.0;
    println!(
        "\nCHECK crate=layout name=snap_pixel_grid_and_ctrl result={} rounded_delta={:.1} off_delta={} no_grid_delta={}",
        pass(ok),
        rounded.delta,
        off.delta,
        no_grid.delta
    );
    assert!(ok);
}

#[test]
fn a_snapped_move_works_on_both_axes_with_guides() {
    let w = world();
    let raw = PxRect {
        x0: 168.0,
        y0: 305.0,
        x1: 400.0,
        y1: 500.0,
    };
    let m = w.snap_move(&raw, REACH, ALL);
    // x: left edge to 165 (-3). y: top edge to 307 (+2).
    let ok = close(m.dx, -3.0)
        && close(m.dy, 2.0)
        && m.guides.len() == 2
        && m.guides.iter().any(|g| g.axis == Axis::X && g.at == 165.0)
        && m.guides.iter().any(|g| g.axis == Axis::Y && g.at == 307.0);
    println!(
        "\nCHECK crate=layout name=snap_move_both_axes result={} dx={} dy={} guides={}",
        pass(ok),
        m.dx,
        m.dy,
        m.guides.len()
    );
    assert!(ok);
}

#[test]
fn resize_snaps_the_edge_and_the_size() {
    let w = world();
    // A box from x = 120 to 220 (not on a line); drag the right edge by +95: raw edge 315,
    // 8 px past the line at 307: out of reach of the position; but the size 195 is 1 px short of
    // 197.. whole units are 71 px: 195 / 71 = 2.75 -> 3 units = 213, 18 px away: no catch.
    let start = PxRect {
        x0: 120.0,
        y0: 120.0,
        x1: 220.0,
        y1: 220.0,
    };
    let none = w.snap_resize(
        &start,
        Handle::E,
        (95.0, 0.0),
        ResizeMods::default(),
        REACH,
        ALL,
        1.0,
    );
    // Drag by +85: raw edge 305, 2 px short of the line at 307: the position catches.
    let pos = w.snap_resize(
        &start,
        Handle::E,
        (85.0, 0.0),
        ResizeMods::default(),
        REACH,
        ALL,
        1.0,
    );
    // The same box, rhythm positions off but sizes on is not a thing: the rhythm toggle covers
    // both. Size snap: drag by +21 with the position targets far: new size 121 -> 71 * 2 = 142
    // is 21 px away, no; use +20.5 -> size 120.5; 2 units = 142: no. Drag so the size is 3 px off
    // two units: size 145 -> edge 265, size candidate 262 (3 px short). Position targets: 236 and
    // 307 are 29 and 42 px from 265, so only the size catches.
    let size = w.snap_resize(
        &start,
        Handle::E,
        (45.0, 0.0),
        ResizeMods::default(),
        REACH,
        ALL,
        1.0,
    );
    let ok = none.1.is_empty()
        && close(none.0.x1, 315.0)
        && close(pos.0.x1, 307.0)
        && pos.1.first().map(|g| g.kind) == Some(SnapKind::Rhythm)
        && close(size.0.x1, 262.0)
        && size.1.first().map(|g| g.kind) == Some(SnapKind::RhythmSize)
        && close(size.0.w(), 142.0);
    println!(
        "\nCHECK crate=layout name=snap_resize_edge_and_size result={} far_x1={} edge_x1={} size_x1={} size_w={} size_kind={:?}",
        pass(ok),
        none.0.x1,
        pos.0.x1,
        size.0.x1,
        size.0.w(),
        size.1.first().map(|g| g.kind)
    );
    assert!(ok);
}

#[test]
fn resize_with_shift_keeps_proportions_and_alt_works_from_the_centre() {
    let w = world();
    let start = PxRect {
        x0: 120.0,
        y0: 100.0,
        x1: 320.0,
        y1: 200.0,
    }; // 200 x 100, ratio 2
    let off = SnapToggles::OFF;
    // Shift on the SE corner, pointer +60, +10: the larger ask is x (1.3), so the scale is 1.3.
    let (r, _) = w.snap_resize(
        &start,
        Handle::SE,
        (60.0, 10.0),
        ResizeMods {
            keep_ratio: true,
            from_center: false,
        },
        REACH,
        off,
        1.0,
    );
    let ratio_kept = close(r.w() / r.h(), 2.0) && close(r.x0, 120.0) && close(r.y0, 100.0);
    let scaled = close(r.w(), 260.0) && close(r.h(), 130.0);
    // Alt on the E edge, pointer +30: both sides move: 120 - 30 .. 320 + 30.
    let (c, _) = w.snap_resize(
        &start,
        Handle::E,
        (30.0, 0.0),
        ResizeMods {
            keep_ratio: false,
            from_center: true,
        },
        REACH,
        off,
        1.0,
    );
    let centred =
        close(c.x0, 90.0) && close(c.x1, 350.0) && close(c.y0, 100.0) && close(c.y1, 200.0);
    // Shift on an edge handle grows the other axis about its centre.
    let (e, _) = w.snap_resize(
        &start,
        Handle::E,
        (100.0, 0.0),
        ResizeMods {
            keep_ratio: true,
            from_center: false,
        },
        REACH,
        off,
        1.0,
    );
    let edge_ratio = close(e.w(), 300.0) && close(e.h(), 150.0) && close(e.y0, 75.0);
    // Pulling a side past the other flips the box; it never collapses below the minimum.
    let (f, _) = w.snap_resize(
        &start,
        Handle::E,
        (-250.0, 0.0),
        ResizeMods::default(),
        REACH,
        off,
        1.0,
    );
    let flipped = close(f.x0, 70.0) && close(f.x1, 120.0);
    let (m, _) = w.snap_resize(
        &start,
        Handle::E,
        (-200.0, 0.0),
        ResizeMods::default(),
        REACH,
        off,
        1.0,
    );
    let min = m.w() >= 1.0 - 1e-9;
    let ok = ratio_kept && scaled && centred && edge_ratio && flipped && min;
    println!(
        "\nCHECK crate=layout name=resize_shift_alt result={} ratio_kept={ratio_kept} scaled={scaled} from_centre={centred} edge_ratio={edge_ratio} flipped={flipped} min_size={min}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn line_ends_snap_as_points() {
    let w = world();
    let (p, g) = w.snap_point((168.0, 305.0), REACH, ALL);
    let (q, h) = w.snap_point((168.0, 305.0), REACH, SnapToggles::OFF);
    let ok = close(p.0, 165.0)
        && close(p.1, 307.0)
        && g.len() == 2
        && q == (168.0, 305.0)
        && h.is_empty();
    println!(
        "\nCHECK crate=layout name=snap_line_end result={} snapped=({},{}) off=({},{})",
        pass(ok),
        p.0,
        p.1,
        q.0,
        q.1
    );
    assert!(ok);
}

#[test]
fn unresolve_is_the_inverse_of_resolve() {
    let f = Frame::new(20.0, 30.0, 40.0, 50.0);
    let (from, to) = ((148.0, 210.0), (100.0, 160.0));
    let mut all_ok = true;
    let mut n = 0;
    for h in [
        HConstraint::Left,
        HConstraint::Right,
        HConstraint::LeftRight,
        HConstraint::Center,
        HConstraint::Scale,
    ] {
        for v in [
            VConstraint::Top,
            VConstraint::Bottom,
            VConstraint::TopBottom,
            VConstraint::Center,
            VConstraint::Scale,
        ] {
            let c = Constraints { h, v };
            let back = unresolve_frame(resolve_frame(f, c, from, to), c, from, to);
            let same = close(back.x_mm, f.x_mm)
                && close(back.y_mm, f.y_mm)
                && close(back.w_mm, f.w_mm)
                && close(back.h_mm, f.h_mm);
            all_ok &= same;
            n += 1;
        }
    }
    println!(
        "\nCHECK crate=layout name=unresolve_inverse result={} combinations={n}",
        pass(all_ok)
    );
    assert!(all_ok);
}

#[test]
fn rect_helpers_for_the_marquee() {
    let r = PxRect::from_corners(100.0, 100.0, 200.0, 200.0);
    let overlap = r.touches(&PxRect::from_corners(190.0, 190.0, 300.0, 300.0));
    let apart = !r.touches(&PxRect::from_corners(201.0, 100.0, 300.0, 200.0));
    // A segment crossing the box with both ends outside it touches; one passing beside it does not.
    let crossing = r.touches_segment(50.0, 150.0, 250.0, 150.0);
    let beside = !r.touches_segment(50.0, 50.0, 250.0, 60.0);
    let diagonal = r.touches_segment(0.0, 0.0, 300.0, 300.0);
    let ok = overlap && apart && crossing && beside && diagonal;
    println!(
        "\nCHECK crate=layout name=rect_touches result={} overlap={overlap} apart={apart} crossing={crossing} beside={beside} diagonal={diagonal}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn nothing_snaps_to_rhythm_lines_off_the_page() {
    // The rhythm runs on past the page edge (a line at 23, the next at -48), but only lines on the
    // page catch: an edge 3 px past x = -48 finds nothing, one near 23 still does.
    let w = world();
    let off_page = w.snap_span(Axis::X, -45.0, -20.0, REACH, ALL);
    let on_page = w.snap_span(Axis::X, 26.0, 60.0, REACH, ALL);
    let lines: Vec<f64> = w
        .lines(Axis::X, -200.0, 200.0)
        .iter()
        .map(|l| l.0)
        .collect();
    let ok = off_page.guide.is_none()
        && on_page.guide.map(|g| g.at) == Some(23.0)
        && lines.iter().all(|&x| x >= 0.0)
        && lines.first() == Some(&23.0);
    println!(
        "\nCHECK crate=layout name=snap_rhythm_stays_on_the_page result={} off_page_guide={:?} on_page_guide={:?} first_line={:?}",
        pass(ok),
        off_page.guide.map(|g| g.at),
        on_page.guide.map(|g| g.at),
        lines.first()
    );
    assert!(ok);
}

#[test]
fn union_outline_leaves_out_borders_inside_the_combined_area() {
    let len = |segs: &[(usize, Segment)]| -> f64 {
        segs.iter()
            .map(|(_, ((x0, y0), (x1, y1)))| (x1 - x0).abs() + (y1 - y0).abs())
            .sum()
    };
    let r = |x0, y0, x1, y1| PxRect { x0, y0, x1, y1 };
    // One rectangle: its whole border (40).
    let one = union_outline(&[r(0.0, 0.0, 10.0, 10.0)]);
    // Two that overlap by 5 x 5: the union's border is 40 + 40 - 20 = 60, and no piece lies strictly
    // inside the other rectangle.
    let (a, b) = (r(0.0, 0.0, 10.0, 10.0), r(5.0, 5.0, 15.0, 15.0));
    let two = union_outline(&[a, b]);
    let strictly_inside = |rc: &PxRect, ((x0, y0), (x1, y1)): ((f64, f64), (f64, f64))| {
        let (mx, my) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        mx > rc.x0 && mx < rc.x1 && my > rc.y0 && my < rc.y1
    };
    let none_inside = two
        .iter()
        .all(|&(i, s)| !strictly_inside(if i == 0 { &b } else { &a }, s));
    // Two that meet along a whole edge: the shared edge goes (60 = the 20 x 10 box's border).
    let met = union_outline(&[r(0.0, 0.0, 10.0, 10.0), r(10.0, 0.0, 20.0, 10.0)]);
    // One inside another: only the outer border is left (40).
    let nested = union_outline(&[r(0.0, 0.0, 10.0, 10.0), r(2.0, 2.0, 4.0, 4.0)]);
    // Apart: both borders (80). A rectangle with no area draws nothing.
    let apart = union_outline(&[r(0.0, 0.0, 10.0, 10.0), r(20.0, 0.0, 30.0, 10.0)]);
    let flat = union_outline(&[r(0.0, 0.0, 0.0, 10.0)]);
    let ok = len(&one) == 40.0
        && len(&two) == 60.0
        && none_inside
        && len(&met) == 60.0
        && len(&nested) == 40.0
        && len(&apart) == 80.0
        && flat.is_empty();
    println!(
        "\nCHECK crate=layout name=union_outline result={} one={} overlap={} meet={} nested={} apart={}",
        pass(ok),
        len(&one),
        len(&two),
        len(&met),
        len(&nested),
        len(&apart)
    );
    assert!(ok);
}
