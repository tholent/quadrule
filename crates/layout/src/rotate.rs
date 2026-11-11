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

//! Rotation and line direction in device pixels (rung 6b, docs/bbq/2026-11-10-rung-6-shapes.md,
//! Defaulted "6b, settled now for its builder"). An object's frame stays the unrotated box; its
//! rotation is in degrees, clockwise on the page (y points down), about the frame's centre. Every
//! consumer (drawing, picking, handles, snapping, lint) takes its corners from here, so they all
//! agree.
//!
//! The angle is exact at multiples of 90 degrees: a quarter turn swaps the box's sides with no
//! rounding, so a rectangle turned by 90 stays on whole pixels.

use quadrule_model::LineDir;

/// A box in device pixels: (left, top, right, bottom).
pub type Box4 = (f64, f64, f64, f64);

/// The cosine and sine of `deg` degrees, exactly 0, 1 or -1 at multiples of 90.
pub fn cos_sin_deg(deg: f64) -> (f64, f64) {
    let d = quadrule_model::normalize_deg(deg);
    if d == 0.0 {
        (1.0, 0.0)
    } else if d == 90.0 {
        (0.0, 1.0)
    } else if d == 180.0 {
        (-1.0, 0.0)
    } else if d == 270.0 {
        (0.0, -1.0)
    } else {
        let r = d.to_radians();
        (r.cos(), r.sin())
    }
}

/// Whether `deg` is a multiple of 90 (0, 90, 180 or 270), where edges stay horizontal and vertical.
pub fn is_quarter_turn(deg: f64) -> bool {
    quadrule_model::normalize_deg(deg).rem_euclid(90.0) == 0.0
}

/// `p` turned clockwise by `deg` degrees about `c`.
pub fn rotate_point(p: (f64, f64), c: (f64, f64), deg: f64) -> (f64, f64) {
    let (cs, sn) = cos_sin_deg(deg);
    let (dx, dy) = (p.0 - c.0, p.1 - c.1);
    (c.0 + dx * cs - dy * sn, c.1 + dx * sn + dy * cs)
}

/// The centre of a box.
pub fn centre(r: Box4) -> (f64, f64) {
    ((r.0 + r.2) / 2.0, (r.1 + r.3) / 2.0)
}

/// The box's corners after turning it by `deg` about its centre: top-left, top-right,
/// bottom-right, bottom-left (of the unrotated box), in that order.
pub fn rotated_corners(r: Box4, deg: f64) -> [(f64, f64); 4] {
    let c = centre(r);
    [(r.0, r.1), (r.2, r.1), (r.2, r.3), (r.0, r.3)].map(|p| rotate_point(p, c, deg))
}

/// The smallest box that holds all of `pts`.
pub fn bounds_of(pts: &[(f64, f64)]) -> Box4 {
    let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for &(x, y) in pts {
        b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
    }
    b
}

/// A line's two ends in a box: from the left end to the right end. `Down` runs from the top-left
/// corner to the bottom-right one, `Up` from the bottom-left to the top-right; a box with no width
/// or no height has only one diagonal, so the direction does not matter there.
pub fn line_ends(r: Box4, dir: LineDir) -> ((f64, f64), (f64, f64)) {
    let (x0, x1) = (r.0.min(r.2), r.0.max(r.2));
    let (y0, y1) = (r.1.min(r.3), r.1.max(r.3));
    if dir == LineDir::Up && x0 != x1 && y0 != y1 {
        ((x0, y1), (x1, y0))
    } else {
        ((x0, y0), (x1, y1))
    }
}

/// A line's two ends after turning its box by `deg` about the box's centre.
pub fn rotated_line_ends(r: Box4, dir: LineDir, deg: f64) -> ((f64, f64), (f64, f64)) {
    let (a, b) = line_ends(r, dir);
    let c = centre(r);
    (rotate_point(a, c, deg), rotate_point(b, c, deg))
}

/// Whether a box turned by `deg` about its centre, grown by `margin` all round, overlaps or
/// touches the box `other` (the separating-axis test: exact for a turned rectangle).
pub fn rotated_touches(r: Box4, deg: f64, margin: f64, other: Box4) -> bool {
    let c = centre(r);
    let (hw, hh) = (
        (r.2 - r.0).abs() / 2.0 + margin,
        (r.3 - r.1).abs() / 2.0 + margin,
    );
    let corners = [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
        .map(|p| rotate_point((c.0 + p.0, c.1 + p.1), c, deg));
    let qb = bounds_of(&corners);
    if qb.2 < other.0 || other.2 < qb.0 || qb.3 < other.1 || other.3 < qb.1 {
        return false;
    }
    let (cs, sn) = cos_sin_deg(deg);
    // The turned box's own axes: project the other box's corners onto them.
    let others = [
        (other.0, other.1),
        (other.2, other.1),
        (other.2, other.3),
        (other.0, other.3),
    ];
    for (ux, uy, half) in [(cs, sn, hw), (-sn, cs, hh)] {
        let centre_proj = c.0 * ux + c.1 * uy;
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for p in others {
            let t = p.0 * ux + p.1 * uy;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        if hi < centre_proj - half || lo > centre_proj + half {
            return false;
        }
    }
    true
}
