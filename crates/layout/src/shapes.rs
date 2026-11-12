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

//! Shapes, dashes and caps in device pixels (rung 6a, docs/bbq/2026-11-10-rung-6-shapes.md).
//!
//! Q2: curves (ellipses, rounded corners) and slanted edges (polygons, stars) are anti-aliased
//! ([`Item::Fill`], [`Item::StrokePath`]); straight, unrotated edges stay pixel-exact
//! ([`Item::Rect`]), so an unrotated rectangle with a square corner draws as hard-edged whole-pixel
//! rectangles, as lines do. Strokes are centred on the frame's edge, an odd width on a pixel
//! centre (napkin §10.2). The fill goes under the stroke.
//!
//! Dashes are worked out here, in whole device pixels, so every renderer draws the same ones. A
//! pattern starts at a line's start (its top-left end) and, on a closed shape, at its top-left
//! corner (for a polygon, the corner nearest it), and runs clockwise.

use crate::{Item, snap_center, stroke_opt};
use kurbo::{Affine, Arc, BezPath, PathEl, Point, Shape};
use quadrule_model::{DashStyle, Fill, LineCap, Object, ObjectKind, PolygonShape};
use quadrule_profiles::Profile;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// A dash and gap in whole device pixels, or `None` for a solid stroke. Dotted is as long as the
/// stroke is wide; the presets and a custom pair are millimetres rounded to whole pixels, each at
/// least 1.
pub fn dash_px(dash: DashStyle, width_px: u32, p: &Profile) -> Option<(f64, f64)> {
    let whole = |mm: f64| {
        let v = p.mm_to_px(mm).round();
        if v.is_finite() { v.max(1.0) } else { 1.0 }
    };
    match dash {
        DashStyle::Solid => None,
        DashStyle::Dotted => Some((width_px as f64, width_px as f64)),
        d => d.lengths_mm().map(|(on, off)| (whole(on), whole(off))),
    }
}

/// One dash: a run of the path.
#[derive(Clone, Debug, PartialEq)]
pub struct DashPiece {
    /// The points of the run, at least two.
    pub pts: Vec<(f64, f64)>,
    /// It begins at the start of the path.
    pub at_start: bool,
    /// It ends at the end of the path.
    pub at_end: bool,
}

/// Cuts a polyline into dashes of `on` pixels with `off` between, starting a dash at its first
/// point. A closed shape passes its first point again as the last.
pub fn dash_polyline(pts: &[(f64, f64)], on: f64, off: f64) -> Vec<DashPiece> {
    const EPS: f64 = 1e-9;
    let mut out: Vec<DashPiece> = Vec::new();
    if pts.len() < 2 || on <= 0.0 || off <= 0.0 {
        return out;
    }
    let mut cur: Vec<(f64, f64)> = vec![pts[0]];
    let mut cur_start = true;
    let mut dashing = true;
    let mut left = on;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if len <= EPS {
            continue;
        }
        let mut pos = 0.0;
        while len - pos > EPS {
            let step = (len - pos).min(left);
            pos += step;
            left -= step;
            let t = pos / len;
            let pt = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            if dashing {
                cur.push(pt);
            }
            if left <= EPS {
                if dashing {
                    out.push(DashPiece {
                        pts: std::mem::take(&mut cur),
                        at_start: cur_start,
                        at_end: false,
                    });
                    cur_start = false;
                    dashing = false;
                    left = off;
                } else {
                    dashing = true;
                    cur = vec![pt];
                    left = on;
                }
            }
        }
    }
    if dashing && cur.len() >= 2 {
        out.push(DashPiece {
            pts: cur,
            at_start: cur_start,
            at_end: true,
        });
    }
    out
}

/// The pieces of one path as strokes: hard-edged rectangles when every segment is axis-aligned
/// and the cap is not round, otherwise one anti-aliased path.
fn piece_items(
    pieces: &[DashPiece],
    closed: bool,
    width: f64,
    gray: u8,
    cap: LineCap,
    out: &mut Vec<Item>,
) {
    if pieces.is_empty() {
        return;
    }
    let axis_aligned = pieces.iter().all(|pc| {
        pc.pts
            .windows(2)
            .all(|s| s[0].0 == s[1].0 || s[0].1 == s[1].1)
    });
    if axis_aligned && cap != LineCap::Round {
        let h = width / 2.0;
        let cap_ext = if cap == LineCap::Square { h } else { 0.0 };
        for pc in pieces {
            let n = pc.pts.len() - 1;
            for i in 0..n {
                let (a, b) = (pc.pts[i], pc.pts[i + 1]);
                let horizontal = a.1 == b.1;
                let (s0, s1, c) = if horizontal {
                    (a.0, b.0, a.1)
                } else {
                    (a.1, b.1, a.0)
                };
                // A joint inside the dash is a corner: both sides reach the stroke's outer edge.
                // The start of a closed shape's pattern is a corner too.
                let ext0 = if i > 0 || (closed && pc.at_start) {
                    h
                } else {
                    cap_ext
                };
                let ext1 = if i + 1 < n || (closed && pc.at_end) {
                    h
                } else {
                    cap_ext
                };
                let sign = if s1 >= s0 { 1.0 } else { -1.0 };
                // Ends land on whole pixels (an odd width's centre is on a half pixel).
                let e0 = (s0 - sign * ext0 + 0.5).floor();
                let e1 = (s1 + sign * ext1 + 0.5).floor();
                let (lo, hi) = (e0.min(e1), e0.max(e1));
                out.push(if horizontal {
                    Item::Rect {
                        x0: lo,
                        y0: c - h,
                        x1: hi,
                        y1: c + h,
                        gray,
                    }
                } else {
                    Item::Rect {
                        x0: c - h,
                        y0: lo,
                        x1: c + h,
                        y1: hi,
                        gray,
                    }
                });
            }
        }
        return;
    }
    // Anti-aliased. On a closed shape a dash that runs through the start joins its two halves.
    let mut runs: Vec<Vec<(f64, f64)>> = pieces.iter().map(|pc| pc.pts.clone()).collect();
    if closed
        && runs.len() > 1
        && pieces[0].at_start
        && pieces[pieces.len() - 1].at_end
        && let Some(first) = runs.first().cloned()
    {
        let last = runs.pop().expect("more than one run");
        let mut joined = last;
        joined.extend(first.into_iter().skip(1));
        runs[0] = joined;
    }
    let mut path = BezPath::new();
    for run in &runs {
        path.move_to(run[0]);
        for &q in &run[1..] {
            path.line_to(q);
        }
    }
    out.push(Item::StrokePath {
        path,
        width,
        gray,
        cap,
    });
}

/// A line (the frame's diagonal, top-left to bottom-right) with a dash or a cap other than the
/// default. Horizontal and vertical lines stay hard-edged unless the cap is round.
pub fn line_items(
    (x0, y0, x1, y1): (f64, f64, f64, f64),
    w: u32,
    gray: u8,
    dash: DashStyle,
    cap: LineCap,
    p: &Profile,
) -> Vec<Item> {
    let wf = w as f64;
    let pts = if y0 == y1 {
        let c = snap_center(y0, w);
        vec![(x0.min(x1).round(), c), (x0.max(x1).round(), c)]
    } else if x0 == x1 {
        let c = snap_center(x0, w);
        vec![(c, y0.min(y1).round()), (c, y0.max(y1).round())]
    } else {
        vec![(x0, y0), (x1, y1)]
    };
    let mut out = Vec::new();
    match dash_px(dash, w, p) {
        Some((on, off)) => piece_items(
            &dash_polyline(&pts, on, off),
            false,
            wf,
            gray,
            cap,
            &mut out,
        ),
        None => piece_items(
            &[DashPiece {
                pts,
                at_start: true,
                at_end: true,
            }],
            false,
            wf,
            gray,
            cap,
            &mut out,
        ),
    }
    out
}

/// A shape's items: its fill, then its stroke on top. `frame` is (left, top, right, bottom) in
/// device pixels, already resolved for the profile.
pub fn shape_items(o: &Object, frame: (f64, f64, f64, f64), p: &Profile) -> Vec<Item> {
    shape_items_turned(o, frame, o.rotation(), p)
}

/// [`shape_items`] with the angle given: an object in a turned group is drawn at its own angle
/// plus the group's (rung 7a).
pub fn shape_items_turned(
    o: &Object,
    frame: (f64, f64, f64, f64),
    rotation_deg: f64,
    p: &Profile,
) -> Vec<Item> {
    let (mut fx0, mut fy0) = (frame.0.min(frame.2), frame.1.min(frame.3));
    let (mut fx1, mut fy1) = (frame.0.max(frame.2), frame.1.max(frame.3));
    // Format 0.6: a rotation turns the shape about the frame's centre. A rectangle or an ellipse
    // turned by a whole number of quarter turns is the same shape, so it is drawn as an unrotated
    // one in the turned box (sides swapped for 90 and 270): straight edges stay pixel-exact.
    let mut rot = rotation_deg;
    if !matches!(o.kind, ObjectKind::Polygon(_)) && crate::rotate::is_quarter_turn(rot) {
        if rot == 90.0 || rot == 270.0 {
            let (cx, cy) = ((fx0 + fx1) / 2.0, (fy0 + fy1) / 2.0);
            let (hw, hh) = ((fx1 - fx0) / 2.0, (fy1 - fy0) / 2.0);
            (fx0, fy0, fx1, fy1) = (cx - hh, cy - hw, cx + hh, cy + hw);
        }
        rot = 0.0;
    }
    let stroke = stroke_opt(o.style, p);
    let fill = o.fill.map(|f: Fill| p.fill_gray(f));
    // The geometry: the frame, its edges moved onto the stroke's pixel grid.
    let (gx0, gy0, gx1, gy1) = match stroke {
        Some((w, _)) => (
            snap_center(fx0, w),
            snap_center(fy0, w),
            snap_center(fx1, w),
            snap_center(fy1, w),
        ),
        None => (fx0.round(), fy0.round(), fx1.round(), fy1.round()),
    };
    let mut items = Vec::new();
    let radius_px = match &o.kind {
        ObjectKind::Rect(r) if r.corner_radius_mm.is_finite() => {
            let max = ((gx1 - gx0).min(gy1 - gy0) / 2.0).max(0.0);
            p.mm_to_px(r.corner_radius_mm).clamp(0.0, max)
        }
        _ => 0.0,
    };
    let hard = matches!(o.kind, ObjectKind::Rect(_)) && radius_px <= 0.0 && rot == 0.0;

    if hard {
        if let Some(g) = fill {
            let (x0, y0, x1, y1) = (fx0.round(), fy0.round(), fx1.round(), fy1.round());
            if x1 > x0 && y1 > y0 {
                items.push(Item::Rect {
                    x0,
                    y0,
                    x1,
                    y1,
                    gray: g,
                });
            }
        }
        if let Some((w, gray)) = stroke {
            hard_rect_stroke((gx0, gy0, gx1, gy1), w, gray, o, p, &mut items);
        }
        return items;
    }

    if gx1 <= gx0 || gy1 <= gy0 {
        return items;
    }
    let mut path = match &o.kind {
        ObjectKind::Rect(_) if radius_px <= 0.0 => sharp_rect_path((gx0, gy0, gx1, gy1)),
        ObjectKind::Rect(_) => rounded_rect_path((gx0, gy0, gx1, gy1), radius_px),
        ObjectKind::Ellipse(_) => ellipse_path((gx0, gy0, gx1, gy1)),
        ObjectKind::Polygon(pg) => polygon_path((gx0, gy0, gx1, gy1), pg),
        _ => return items,
    };
    if rot != 0.0 {
        path.apply_affine(rotation_about(((gx0 + gx1) / 2.0, (gy0 + gy1) / 2.0), rot));
    }
    if let Some(g) = fill {
        items.push(Item::Fill {
            path: path.clone(),
            gray: g,
        });
    }
    if let Some((w, gray)) = stroke {
        match dash_px(o.dash, w, p) {
            None => items.push(Item::StrokePath {
                path,
                width: w as f64,
                gray,
                cap: o.cap,
            }),
            Some((on, off)) => {
                let mut pts: Vec<(f64, f64)> = Vec::new();
                kurbo::flatten(&path, 0.1, |el| match el {
                    PathEl::MoveTo(q) | PathEl::LineTo(q) => pts.push((q.x, q.y)),
                    _ => {}
                });
                // `flatten` leaves a closed path's end point off; close the loop by hand.
                if let Some(&first) = pts.first() {
                    pts.push(first);
                }
                piece_items(
                    &dash_polyline(&pts, on, off),
                    true,
                    w as f64,
                    gray,
                    o.cap,
                    &mut items,
                );
            }
        }
    }
    items
}

/// An unrotated rectangle with square corners: hard-edged rectangles. Solid, it is the ring
/// between the outer and inner outlines; dashed, runs of the ring from the top-left corner.
fn hard_rect_stroke(
    (gx0, gy0, gx1, gy1): (f64, f64, f64, f64),
    w: u32,
    gray: u8,
    o: &Object,
    p: &Profile,
    items: &mut Vec<Item>,
) {
    let h = w as f64 / 2.0;
    match dash_px(o.dash, w, p) {
        None => {
            let (ox0, oy0, ox1, oy1) = (gx0 - h, gy0 - h, gx1 + h, gy1 + h);
            let (ix0, iy0, ix1, iy1) = (gx0 + h, gy0 + h, gx1 - h, gy1 - h);
            let rect = |x0, y0, x1, y1| Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            };
            if ix1 <= ix0 || iy1 <= iy0 {
                items.push(rect(ox0, oy0, ox1, oy1));
            } else {
                items.push(rect(ox0, oy0, ox1, iy0));
                items.push(rect(ox0, iy1, ox1, oy1));
                items.push(rect(ox0, iy0, ix0, iy1));
                items.push(rect(ix1, iy0, ox1, iy1));
            }
        }
        Some((on, off)) => {
            let pts = [(gx0, gy0), (gx1, gy0), (gx1, gy1), (gx0, gy1), (gx0, gy0)];
            piece_items(
                &dash_polyline(&pts, on, off),
                true,
                w as f64,
                gray,
                o.cap,
                items,
            );
        }
    }
}

/// The affine map that turns the page clockwise by `deg` degrees about `c`; exact at quarter turns.
pub fn rotation_about(c: (f64, f64), deg: f64) -> Affine {
    let (cs, sn) = crate::rotate::cos_sin_deg(deg);
    Affine::new([
        cs,
        sn,
        -sn,
        cs,
        c.0 - cs * c.0 + sn * c.1,
        c.1 - sn * c.0 - cs * c.1,
    ])
}

/// A rectangle with square corners as a path (for a turned rectangle), clockwise from the
/// top-left corner.
pub fn sharp_rect_path((x0, y0, x1, y1): (f64, f64, f64, f64)) -> BezPath {
    let mut path = BezPath::new();
    path.move_to((x0, y0));
    path.line_to((x1, y0));
    path.line_to((x1, y1));
    path.line_to((x0, y1));
    path.close_path();
    path
}

/// Kurbo's arc as path elements without its first MoveTo.
fn push_arc(path: &mut BezPath, centre: Point, radii: (f64, f64), start: f64, sweep: f64) {
    let arc = Arc::new(centre, (radii.0, radii.1), start, sweep, 0.0);
    for el in arc.path_elements(0.05).skip(1) {
        path.push(el);
    }
}

/// A rectangle with rounded corners, clockwise from the end of the top-left corner (the start of
/// the top edge). `r` is at most half the shorter side.
pub fn rounded_rect_path((x0, y0, x1, y1): (f64, f64, f64, f64), r: f64) -> BezPath {
    let mut path = BezPath::new();
    path.move_to((x0 + r, y0));
    path.line_to((x1 - r, y0));
    push_arc(
        &mut path,
        Point::new(x1 - r, y0 + r),
        (r, r),
        -FRAC_PI_2,
        FRAC_PI_2,
    );
    path.line_to((x1, y1 - r));
    push_arc(
        &mut path,
        Point::new(x1 - r, y1 - r),
        (r, r),
        0.0,
        FRAC_PI_2,
    );
    path.line_to((x0 + r, y1));
    push_arc(
        &mut path,
        Point::new(x0 + r, y1 - r),
        (r, r),
        FRAC_PI_2,
        FRAC_PI_2,
    );
    path.line_to((x0, y0 + r));
    push_arc(&mut path, Point::new(x0 + r, y0 + r), (r, r), PI, FRAC_PI_2);
    path.close_path();
    path
}

/// The ellipse inscribed in the box, clockwise from the point nearest the top-left corner.
pub fn ellipse_path((x0, y0, x1, y1): (f64, f64, f64, f64)) -> BezPath {
    let c = Point::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let radii = ((x1 - x0) / 2.0, (y1 - y0) / 2.0);
    let start = -3.0 * FRAC_PI_2 / 2.0;
    let mut path = BezPath::new();
    path.move_to((c.x + radii.0 * start.cos(), c.y + radii.1 * start.sin()));
    push_arc(&mut path, c, radii, start, TAU);
    path.close_path();
    path
}

/// The corners on a circle of radius 1 (star dips included), first one straight up, y down.
fn raw_points(pg: &PolygonShape) -> Vec<(f64, f64)> {
    let n = pg.sides_clamped() as usize;
    let star = pg.star_clamped();
    let mut pts = Vec::with_capacity(2 * n);
    for k in 0..n {
        let a = -FRAC_PI_2 + TAU * k as f64 / n as f64;
        pts.push((a.cos(), a.sin()));
        if let Some(r) = star {
            let b = -FRAC_PI_2 + TAU * (k as f64 + 0.5) / n as f64;
            pts.push((r * b.cos(), r * b.sin()));
        }
    }
    pts
}

/// The corners of a regular polygon or star in the unit square, first corner at the top (the
/// shape stretched to touch all four sides), going clockwise.
pub fn polygon_unit_points(pg: &PolygonShape) -> Vec<(f64, f64)> {
    let pts = raw_points(pg);
    let (mut lx, mut ly, mut hx, mut hy) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for &(x, y) in &pts {
        lx = lx.min(x);
        ly = ly.min(y);
        hx = hx.max(x);
        hy = hy.max(y);
    }
    let (sx, sy) = ((hx - lx).max(1e-9), (hy - ly).max(1e-9));
    pts.iter()
        .map(|&(x, y)| ((x - lx) / sx, (y - ly) / sy))
        .collect()
}

/// The width over the height of the shape before it is stretched to its frame: what a regular
/// polygon (or star) is when it keeps its proportions, as Shift keeps them while drawing.
pub fn natural_ratio(pg: &PolygonShape) -> f64 {
    let (mut lx, mut ly, mut hx, mut hy) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (x, y) in raw_points(pg) {
        lx = lx.min(x);
        ly = ly.min(y);
        hx = hx.max(x);
        hy = hy.max(y);
    }
    ((hx - lx) / (hy - ly).max(1e-9)).max(1e-9)
}

/// Whether an object has a straight edge that is not slanted (horizontal or vertical), the only
/// kind Q2 draws pixel-exact and L02 judges: lines and patterns count (L02 treats a diagonal line
/// on its own), a rectangle has them, an ellipse never does, and a polygon or star does when one
/// of its edges runs along an axis. A rotation that is a multiple of 90 degrees keeps every edge
/// where it was (horizontal edges turn vertical and back); any other angle slants them all, so
/// none is judged (rung 6b).
pub fn has_axis_aligned_edge(kind: &ObjectKind, rotation_deg: f64) -> bool {
    if !crate::rotate::is_quarter_turn(rotation_deg) {
        return false;
    }
    match kind {
        ObjectKind::Ellipse(_) => false,
        ObjectKind::Polygon(pg) => {
            let pts = polygon_unit_points(pg);
            (0..pts.len()).any(|i| {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                (a.0 - b.0).abs() < 1e-9 || (a.1 - b.1).abs() < 1e-9
            })
        }
        _ => true,
    }
}

/// A polygon or star filling the box, starting at the corner nearest the top-left.
pub fn polygon_path((x0, y0, x1, y1): (f64, f64, f64, f64), pg: &PolygonShape) -> BezPath {
    let unit = polygon_unit_points(pg);
    let start = unit
        .iter()
        .enumerate()
        .min_by(|a, b| (a.1.0 + a.1.1).total_cmp(&(b.1.0 + b.1.1)))
        .map_or(0, |(i, _)| i);
    let mut path = BezPath::new();
    for k in 0..unit.len() {
        let (ux, uy) = unit[(start + k) % unit.len()];
        let q = Point::new(x0 + ux * (x1 - x0), y0 + uy * (y1 - y0));
        if k == 0 {
            path.move_to(q);
        } else {
            path.line_to(q);
        }
    }
    path.close_path();
    path
}
