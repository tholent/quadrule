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

//! The border of a pattern region (docs/bbq/2026-11-18-pattern-borders.md; format 0.17).
//!
//! A border is a line centred on the region's frame, like the stroke of a rectangle: on a side that is on,
//! an unrounded border is hard-edged rectangles on whole pixels; a rounded one is an anti-aliased stroke.
//! A [`Rim`] is the border as the pattern sees it. It leaves out the pattern lines and dots that sit on a
//! bordered edge (the centre within half a device pixel of it), so nothing doubles up, and it cuts what
//! would reach into a rounded corner's missing part: a ruling line stops at the curve, a dot whose centre
//! is outside the curve is dropped. A corner is rounded only where both its sides are on, with a radius
//! of at most half the shorter side. Sides that are off keep ADR-014's rule.

use crate::pattern::Pen;
use crate::{Item, snap_center, stroke_opt};
use kurbo::{BezPath, PathEl, Point};
use quadrule_model::{Border, BorderSides, LineCap};
use quadrule_profiles::Profile;
use std::f64::consts::{FRAC_PI_2, PI};

/// The least distance from an edge a pattern line or dot's centre can have and count as on it, in device
/// pixels: half a pixel (docs/bbq/2026-11-18-pattern-borders.md, Defaulted "What on the edge means"; ADR-011).
/// A thicker border or line widens it ([`Rim::band`]): one whose ink touches the border's is on it too, since a
/// rhythm that is whole pixels can sit a pixel off an edge set in millimetres.
const ON_EDGE_PX: f64 = 0.5;

/// The border of a region as the pattern is laid out against it. All coordinates are device pixels in the
/// axes the pattern is laid out in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rim {
    /// The region (left, top, right, bottom) on whole pixels: where pattern lines and dots sit.
    pub frame: (f64, f64, f64, f64),
    /// The border's centre line: the frame's edges moved onto the stroke's pixel grid, as a rectangle's are.
    geo: (f64, f64, f64, f64),
    /// The sides that are drawn.
    pub sides: BorderSides,
    /// The corner radius in pixels (0 is square), at most half the shorter side.
    pub radius: f64,
    /// The border's line.
    pub pen: Pen,
}

/// A corner, clockwise from the top-left one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

const CORNERS: [Corner; 4] = [
    Corner::TopLeft,
    Corner::TopRight,
    Corner::BottomRight,
    Corner::BottomLeft,
];

impl Rim {
    /// The border of `border` on a region `frame` (left, top, right, bottom, whole pixels) as `target`
    /// draws it; `None` when it draws nothing (no stroke, no side on).
    pub fn new(frame: (f64, f64, f64, f64), border: &Border, target: &Profile) -> Option<Rim> {
        let (w, gray) = stroke_opt(border.style, target)?;
        if !border.sides.any() {
            return None;
        }
        let geo = (
            snap_center(frame.0, w),
            snap_center(frame.1, w),
            snap_center(frame.2, w),
            snap_center(frame.3, w),
        );
        let radius = target.mm_to_px(border.radius_mm());
        let cap = ((geo.2 - geo.0).min(geo.3 - geo.1) / 2.0).max(0.0);
        let radius = if radius.is_finite() {
            radius.clamp(0.0, cap)
        } else {
            0.0
        };
        Some(Rim {
            frame,
            geo,
            sides: border.sides,
            radius,
            pen: Pen { w, gray },
        })
    }

    /// The same border seen in other axes: the frame, the centre line and the sides carried through the quarter
    /// turn `turn` (which must map a box to a box). Used to lay a ruling out in its own turned axes.
    pub fn through_quarter_turn(&self, turn: kurbo::Affine) -> Rim {
        let map_box = |b: (f64, f64, f64, f64)| {
            let p = turn * Point::new(b.0, b.1);
            let q = turn * Point::new(b.2, b.3);
            (p.x.min(q.x), p.y.min(q.y), p.x.max(q.x), p.y.max(q.y))
        };
        // Where each side's outward normal points after the turn tells which side it became.
        let [a, b, c, d, _, _] = turn.as_coeffs();
        let side_after = |nx: f64, ny: f64| {
            let (x, y) = (a * nx + c * ny, b * nx + d * ny);
            if y.abs() >= x.abs() {
                if y < 0.0 { 0 } else { 2 }
            } else if x > 0.0 {
                1
            } else {
                3
            }
        };
        let was = [
            self.sides.top,
            self.sides.right,
            self.sides.bottom,
            self.sides.left,
        ];
        let normals = [(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)];
        let mut now = [false; 4];
        for (i, (nx, ny)) in normals.into_iter().enumerate() {
            now[side_after(nx, ny)] = was[i];
        }
        Rim {
            frame: map_box(self.frame),
            geo: map_box(self.geo),
            sides: BorderSides {
                top: now[0],
                right: now[1],
                bottom: now[2],
                left: now[3],
            },
            ..*self
        }
    }

    /// How near an edge the centre of a line or dot `lw` pixels thick must be to count as on it: half a pixel, or
    /// as near as makes its ink touch the border's when that is more.
    fn band(&self, lw: f64) -> f64 {
        ON_EDGE_PX.max((f64::from(self.pen.w) + lw) / 2.0) + 1e-9
    }

    /// Whether the corner is rounded: its radius is above 0 and both its sides are on.
    fn rounded(&self, c: Corner) -> bool {
        let s = &self.sides;
        self.radius > 0.0
            && match c {
                Corner::TopLeft => s.top && s.left,
                Corner::TopRight => s.top && s.right,
                Corner::BottomRight => s.bottom && s.right,
                Corner::BottomLeft => s.bottom && s.left,
            }
    }

    /// Whether any corner is rounded.
    pub fn has_round_corner(&self) -> bool {
        CORNERS.iter().any(|&c| self.rounded(c))
    }

    /// The centre of a corner's arc.
    fn centre(&self, c: Corner) -> (f64, f64) {
        let (x0, y0, x1, y1) = self.geo;
        let r = self.radius;
        match c {
            Corner::TopLeft => (x0 + r, y0 + r),
            Corner::TopRight => (x1 - r, y0 + r),
            Corner::BottomRight => (x1 - r, y1 - r),
            Corner::BottomLeft => (x0 + r, y1 - r),
        }
    }

    /// Whether the point is in the part of the plane beyond a rounded corner's arc: on the corner's
    /// side of both its arc centre's lines, and farther from the centre than `reach`.
    fn beyond_arc(&self, c: Corner, p: (f64, f64), reach: f64) -> bool {
        let (cx, cy) = self.centre(c);
        let in_quadrant = match c {
            Corner::TopLeft => p.0 < cx && p.1 < cy,
            Corner::TopRight => p.0 > cx && p.1 < cy,
            Corner::BottomRight => p.0 > cx && p.1 > cy,
            Corner::BottomLeft => p.0 < cx && p.1 > cy,
        };
        in_quadrant && (p.0 - cx).hypot(p.1 - cy) > reach
    }

    /// Whether a horizontal line on the rhythm position `y` sits on a bordered edge, so it is left out.
    pub fn drops_hline(&self, y: f64, lw: f64) -> bool {
        (self.sides.top && (y - self.frame.1).abs() <= self.band(lw))
            || (self.sides.bottom && (y - self.frame.3).abs() <= self.band(lw))
    }

    /// Whether a vertical line on the rhythm position `x` sits on a bordered edge, so it is left out.
    pub fn drops_vline(&self, x: f64, lw: f64) -> bool {
        (self.sides.left && (x - self.frame.0).abs() <= self.band(lw))
            || (self.sides.right && (x - self.frame.2).abs() <= self.band(lw))
    }

    /// Whether a dot centred at (`x`, `y`) is left out: on a bordered edge, or beyond a rounded corner's
    /// curve (a dot on the curve itself is on the border, as one on a straight edge is).
    pub fn drops_dot(&self, x: f64, y: f64, size: f64) -> bool {
        self.drops_hline(y, size)
            || self.drops_vline(x, size)
            || CORNERS.iter().any(|&c| {
                self.rounded(c) && self.beyond_arc(c, (x, y), self.radius - self.band(size))
            })
    }

    /// A horizontal line's span `xa..xb`, for a line covering `ya..yb` down the page, cut where a rounded
    /// corner's curve crosses it; `None` when nothing is left.
    pub fn hline_span(&self, xa: f64, xb: f64, ya: f64, yb: f64) -> Option<(f64, f64)> {
        let (mut xa, mut xb) = (xa, xb);
        let r = self.radius;
        for c in CORNERS {
            if !self.rounded(c) {
                continue;
            }
            let (cx, cy) = self.centre(c);
            let top = matches!(c, Corner::TopLeft | Corner::TopRight);
            // The row of the line nearest the page edge is the one the curve cuts shortest.
            let y = if top { ya } else { yb };
            let d = if top { cy - y } else { y - cy };
            if d <= 0.0 {
                continue;
            }
            let reach = (r * r - d.min(r) * d.min(r)).max(0.0).sqrt();
            if matches!(c, Corner::TopLeft | Corner::BottomLeft) {
                xa = xa.max(cx - reach);
            } else {
                xb = xb.min(cx + reach);
            }
        }
        (xb > xa).then_some((xa, xb))
    }

    /// A vertical line's span `ya..yb`, for a line covering `xa..xb` across the page: see [`Rim::hline_span`].
    pub fn vline_span(&self, ya: f64, yb: f64, xa: f64, xb: f64) -> Option<(f64, f64)> {
        let (mut ya, mut yb) = (ya, yb);
        let r = self.radius;
        for c in CORNERS {
            if !self.rounded(c) {
                continue;
            }
            let (cx, cy) = self.centre(c);
            let left = matches!(c, Corner::TopLeft | Corner::BottomLeft);
            let x = if left { xa } else { xb };
            let d = if left { cx - x } else { x - cx };
            if d <= 0.0 {
                continue;
            }
            let reach = (r * r - d.min(r) * d.min(r)).max(0.0).sqrt();
            if matches!(c, Corner::TopLeft | Corner::TopRight) {
                ya = ya.max(cy - reach);
            } else {
                yb = yb.min(cy + reach);
            }
        }
        (yb > ya).then_some((ya, yb))
    }

    /// The pieces of the segment `a`-`b` that stay once the parts beyond rounded corners are cut away.
    pub fn cut_segment(&self, a: Point, b: Point) -> Vec<(Point, Point)> {
        // Intervals of t in 0..=1 that are cut away.
        let mut gone: Vec<(f64, f64)> = Vec::new();
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        for c in CORNERS {
            if !self.rounded(c) {
                continue;
            }
            let (cx, cy) = self.centre(c);
            // The corner's quadrant: the points on its side of both centre lines.
            let (sx, sy) = match c {
                Corner::TopLeft => (-1.0, -1.0),
                Corner::TopRight => (1.0, -1.0),
                Corner::BottomRight => (1.0, 1.0),
                Corner::BottomLeft => (-1.0, 1.0),
            };
            let mut quad = (0.0_f64, 1.0_f64);
            let mut empty = false;
            // sign * (p - centre) > 0 along each axis.
            for (sign, p0, d, centre) in [(sx, a.x, dx, cx), (sy, a.y, dy, cy)] {
                let k = sign * d;
                let q = sign * (p0 - centre);
                if k == 0.0 {
                    if q <= 0.0 {
                        empty = true;
                    }
                } else {
                    let t = -q / k;
                    if k > 0.0 {
                        quad.0 = quad.0.max(t);
                    } else {
                        quad.1 = quad.1.min(t);
                    }
                }
            }
            if empty || quad.1 <= quad.0 {
                continue;
            }
            // The disc: where the segment is within the radius of the centre.
            let (fx, fy) = (a.x - cx, a.y - cy);
            let qa = dx * dx + dy * dy;
            let qb = 2.0 * (fx * dx + fy * dy);
            let qc = fx * fx + fy * fy - self.radius * self.radius;
            let disc = if qa > 0.0 {
                let det = qb * qb - 4.0 * qa * qc;
                (det > 0.0).then(|| {
                    let s = det.sqrt();
                    ((-qb - s) / (2.0 * qa), (-qb + s) / (2.0 * qa))
                })
            } else {
                None
            };
            match disc {
                None => gone.push(quad),
                Some((t0, t1)) => {
                    if t0 > quad.0 {
                        gone.push((quad.0, t0.min(quad.1)));
                    }
                    if t1 < quad.1 {
                        gone.push((t1.max(quad.0), quad.1));
                    }
                }
            }
        }
        if gone.is_empty() {
            return vec![(a, b)];
        }
        gone.sort_by(|p, q| p.0.total_cmp(&q.0));
        let at = |t: f64| Point::new(a.x + t * dx, a.y + t * dy);
        let len = dx.hypot(dy);
        let mut out = Vec::new();
        let mut from = 0.0_f64;
        for (g0, g1) in gone {
            if g0 > from && (g0 - from) * len > 1e-6 {
                out.push((at(from), at(g0)));
            }
            from = from.max(g1);
        }
        if 1.0 > from && (1.0 - from) * len > 1e-6 {
            out.push((at(from), at(1.0)));
        }
        out
    }

    /// `path` (move-to and line-to pairs: slanted ruling) with the parts beyond rounded corners cut away.
    pub fn cut_path(&self, path: &BezPath) -> BezPath {
        if !self.has_round_corner() {
            return path.clone();
        }
        let mut out = BezPath::new();
        let mut from: Option<Point> = None;
        for el in path.elements() {
            match *el {
                PathEl::MoveTo(p) => from = Some(p),
                PathEl::LineTo(p) => {
                    if let Some(a) = from {
                        for (s, e) in self.cut_segment(a, p) {
                            out.move_to(s);
                            out.line_to(e);
                        }
                    }
                    from = Some(p);
                }
                _ => {}
            }
        }
        out
    }

    /// The region's outline with its rounded corners, clockwise from the top-left, as a convex polygon: the
    /// frame, with each rounded corner's arc in place of the corner.
    pub fn outline(&self) -> Vec<Point> {
        let (x0, y0, x1, y1) = self.frame;
        let r = self
            .radius
            .min(((x1 - x0).min(y1 - y0) / 2.0).max(0.0))
            .max(0.0);
        let steps = ((r * FRAC_PI_2 / 2.0).ceil() as usize).clamp(4, 32);
        let mut pts = Vec::new();
        for c in CORNERS {
            let (corner, centre, from) = match c {
                Corner::TopLeft => ((x0, y0), (x0 + r, y0 + r), PI),
                Corner::TopRight => ((x1, y0), (x1 - r, y0 + r), -FRAC_PI_2),
                Corner::BottomRight => ((x1, y1), (x1 - r, y1 - r), 0.0),
                Corner::BottomLeft => ((x0, y1), (x0 + r, y1 - r), FRAC_PI_2),
            };
            if self.rounded(c) && r > 0.0 {
                for k in 0..=steps {
                    let a = from + FRAC_PI_2 * k as f64 / steps as f64;
                    pts.push(Point::new(centre.0 + r * a.cos(), centre.1 + r * a.sin()));
                }
            } else {
                pts.push(Point::new(corner.0, corner.1));
            }
        }
        pts
    }

    /// Whether a dot `size` pixels across with its centre at `p` stays: it is not on a bordered edge and not
    /// beyond a curve.
    pub fn keeps_dot(&self, p: Point, size: f64) -> bool {
        !self.drops_dot(p.x, p.y, size)
    }

    /// The border itself, drawn on top of the pattern: unrounded, hard-edged rectangles on the sides that are
    /// on (a corner joined where both sides are on); rounded, one anti-aliased stroke whose open ends are
    /// square cut.
    pub fn items(&self) -> Vec<Item> {
        let (gx0, gy0, gx1, gy1) = self.geo;
        let Pen { w, gray } = self.pen;
        if gx1 <= gx0 || gy1 <= gy0 {
            return Vec::new();
        }
        let s = self.sides;
        if !CORNERS.iter().any(|&c| self.rounded(c)) {
            let h = f64::from(w) / 2.0;
            let ext = |on: bool| if on { h } else { 0.0 };
            let mut out = Vec::new();
            let mut rect = |x0: f64, y0: f64, x1: f64, y1: f64| {
                if x1 > x0 && y1 > y0 {
                    out.push(Item::Rect {
                        x0,
                        y0,
                        x1,
                        y1,
                        gray,
                    });
                }
            };
            if s.top {
                rect(gx0 - ext(s.left), gy0 - h, gx1 + ext(s.right), gy0 + h);
            }
            if s.bottom {
                rect(gx0 - ext(s.left), gy1 - h, gx1 + ext(s.right), gy1 + h);
            }
            if s.left {
                rect(gx0 - h, gy0 + ext(s.top), gx0 + h, gy1 - ext(s.bottom));
            }
            if s.right {
                rect(gx1 - h, gy0 + ext(s.top), gx1 + h, gy1 - ext(s.bottom));
            }
            return out;
        }
        let path = if s.top && s.right && s.bottom && s.left {
            crate::shapes::rounded_rect_path(self.geo, self.radius)
        } else {
            self.open_path()
        };
        vec![Item::StrokePath {
            path,
            width: f64::from(w),
            gray,
            cap: LineCap::Butt,
        }]
    }

    /// The sides that are on as runs of the outline, each run one subpath, rounded where its two sides meet.
    fn open_path(&self) -> BezPath {
        let (gx0, gy0, gx1, gy1) = self.geo;
        let r = self.radius;
        // Each side, clockwise: whether it is on, and where it starts and ends along the straight part
        // (cut short by the radius at a rounded corner).
        let rr = |c: Corner| if self.rounded(c) { r } else { 0.0 };
        let (tl, tr, br, bl) = (
            rr(Corner::TopLeft),
            rr(Corner::TopRight),
            rr(Corner::BottomRight),
            rr(Corner::BottomLeft),
        );
        let on = [
            self.sides.top,
            self.sides.right,
            self.sides.bottom,
            self.sides.left,
        ];
        let ends = [
            ((gx0 + tl, gy0), (gx1 - tr, gy0)),
            ((gx1, gy0 + tr), (gx1, gy1 - br)),
            ((gx1 - br, gy1), (gx0 + bl, gy1)),
            ((gx0, gy1 - bl), (gx0, gy0 + tl)),
        ];
        // The arc at the end of side `i`, which joins it to side `i + 1`.
        let arcs = [
            (Corner::TopRight, -FRAC_PI_2),
            (Corner::BottomRight, 0.0),
            (Corner::BottomLeft, FRAC_PI_2),
            (Corner::TopLeft, PI),
        ];
        let mut path = BezPath::new();
        for first in 0..4 {
            // A run starts at a side that is on after one that is off.
            if !on[first] || on[(first + 3) % 4] {
                continue;
            }
            let (start, end) = ends[first];
            path.move_to(start);
            path.line_to(end);
            let mut i = first;
            while on[(i + 1) % 4] {
                let (corner, from) = arcs[i];
                let (cx, cy) = self.centre(corner);
                crate::shapes::push_arc(&mut path, Point::new(cx, cy), (r, r), from, FRAC_PI_2);
                i = (i + 1) % 4;
                path.line_to(ends[i].1);
            }
        }
        path
    }
}

/// The part of the convex polygon `pts` inside the convex polygon `clip` (clockwise on screen: y down),
/// by Sutherland-Hodgman.
pub fn clip_convex(pts: &[Point], clip: &[Point]) -> Vec<Point> {
    let mut poly = pts.to_vec();
    for k in 0..clip.len() {
        if poly.is_empty() {
            break;
        }
        let (a, b) = (clip[k], clip[(k + 1) % clip.len()]);
        // Positive on the inside of the edge a-b for a clockwise polygon on a y-down page.
        let side = |p: Point| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        let mut next = Vec::with_capacity(poly.len() + 2);
        for i in 0..poly.len() {
            let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
            let (sp, sq) = (side(p), side(q));
            if sp >= 0.0 {
                next.push(p);
            }
            if (sp >= 0.0) != (sq >= 0.0) {
                let t = sp / (sp - sq);
                next.push(p + (q - p) * t);
            }
        }
        poly = next;
    }
    poly
}
