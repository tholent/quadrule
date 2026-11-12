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

//! Geometry the model itself needs, in millimetres (rung 7a, docs/bbq/2026-11-11-rung-7-groups-and-align.md):
//! how a constraint moves and sizes a box between two pages or two group boxes (napkin §5.4), and
//! the turn of a point about another. Layout re-exports these (`quadrule_layout::resolve_frame`),
//! so there is one copy of the maths.

use crate::{Constraints, Frame, HConstraint, LineDir, VConstraint};

/// The cosine and sine of `deg` degrees, exactly 0, 1 or -1 at multiples of 90.
pub fn cos_sin_deg(deg: f64) -> (f64, f64) {
    let d = crate::normalize_deg(deg);
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

/// `p` turned clockwise (y points down) by `deg` degrees about `c`.
pub fn rotate_point(p: (f64, f64), c: (f64, f64), deg: f64) -> (f64, f64) {
    let (cs, sn) = cos_sin_deg(deg);
    let (dx, dy) = (p.0 - c.0, p.1 - c.1);
    (c.0 + dx * cs - dy * sn, c.1 + dx * sn + dy * cs)
}

impl Frame {
    /// The centre of the box.
    pub fn centre(&self) -> (f64, f64) {
        (self.x_mm + self.w_mm / 2.0, self.y_mm + self.h_mm / 2.0)
    }

    /// The right edge.
    pub fn right(&self) -> f64 {
        self.x_mm + self.w_mm
    }

    /// The bottom edge.
    pub fn bottom(&self) -> f64 {
        self.y_mm + self.h_mm
    }

    /// The smallest box holding both.
    pub fn union(&self, o: &Frame) -> Frame {
        let (x0, y0) = (self.x_mm.min(o.x_mm), self.y_mm.min(o.y_mm));
        let (x1, y1) = (self.right().max(o.right()), self.bottom().max(o.bottom()));
        Frame::new(x0, y0, x1 - x0, y1 - y0)
    }

    /// The box with the same size, centred on `c`.
    pub fn centred_on(&self, c: (f64, f64)) -> Frame {
        Frame::new(
            c.0 - self.w_mm / 2.0,
            c.1 - self.h_mm / 2.0,
            self.w_mm,
            self.h_mm,
        )
    }

    /// The box round this one's four corners after turning it by `deg` about its centre. For a
    /// line, pass its direction: the box round its two ends.
    pub fn turned_bounds(&self, deg: f64, line: Option<LineDir>) -> Frame {
        let c = self.centre();
        let pts: Vec<(f64, f64)> = match line {
            Some(dir) => {
                let (a, b) = if dir == LineDir::Up && self.w_mm != 0.0 && self.h_mm != 0.0 {
                    ((self.x_mm, self.bottom()), (self.right(), self.y_mm))
                } else {
                    ((self.x_mm, self.y_mm), (self.right(), self.bottom()))
                };
                vec![rotate_point(a, c, deg), rotate_point(b, c, deg)]
            }
            None => [
                (self.x_mm, self.y_mm),
                (self.right(), self.y_mm),
                (self.right(), self.bottom()),
                (self.x_mm, self.bottom()),
            ]
            .iter()
            .map(|&p| rotate_point(p, c, deg))
            .collect(),
        };
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (x, y) in pts {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        Frame::new(x0, y0, x1 - x0, y1 - y0)
    }

    /// Whether two boxes are the same within float noise.
    pub fn near(&self, o: &Frame) -> bool {
        const E: f64 = 1e-9;
        (self.x_mm - o.x_mm).abs() < E
            && (self.y_mm - o.y_mm).abs() < E
            && (self.w_mm - o.w_mm).abs() < E
            && (self.h_mm - o.h_mm).abs() < E
    }
}

/// Moves and sizes a frame from the primary page to the target page (napkin §5.4).
/// Page sizes are (width, height) in mm.
pub fn resolve_frame(f: Frame, c: Constraints, from: (f64, f64), to: (f64, f64)) -> Frame {
    let (x_mm, w_mm) = match c.h {
        HConstraint::Left => axis_keep_start(f.x_mm, f.w_mm),
        HConstraint::Right => axis_keep_end(f.x_mm, f.w_mm, from.0, to.0),
        HConstraint::LeftRight => axis_stretch(f.x_mm, f.w_mm, from.0, to.0),
        HConstraint::Center => axis_center(f.x_mm, f.w_mm, from.0, to.0),
        HConstraint::Scale => axis_scale(f.x_mm, f.w_mm, from.0, to.0),
    };
    let (y_mm, h_mm) = match c.v {
        VConstraint::Top => axis_keep_start(f.y_mm, f.h_mm),
        VConstraint::Bottom => axis_keep_end(f.y_mm, f.h_mm, from.1, to.1),
        VConstraint::TopBottom => axis_stretch(f.y_mm, f.h_mm, from.1, to.1),
        VConstraint::Center => axis_center(f.y_mm, f.h_mm, from.1, to.1),
        VConstraint::Scale => axis_scale(f.y_mm, f.h_mm, from.1, to.1),
    };
    Frame {
        x_mm,
        y_mm,
        w_mm,
        h_mm,
    }
}

/// The inverse of [`resolve_frame`]: the frame on the primary page that lands on `resolved` on
/// the target page. Dragging on a secondary device works in that device's frame; this turns the
/// result back into the stored one. Exact for every constraint: `resolve_frame(unresolve_frame(f))`
/// gives `f` back, except an axis whose page is the same size on both (any rule) and the width of
/// `Scale` on a zero-size page.
pub fn unresolve_frame(resolved: Frame, c: Constraints, from: (f64, f64), to: (f64, f64)) -> Frame {
    let (x_mm, w_mm) = match c.h {
        HConstraint::Left => (resolved.x_mm, resolved.w_mm),
        HConstraint::Right => (resolved.x_mm - (to.0 - from.0), resolved.w_mm),
        HConstraint::LeftRight => (resolved.x_mm, resolved.w_mm - (to.0 - from.0)),
        HConstraint::Center => (resolved.x_mm - (to.0 - from.0) / 2.0, resolved.w_mm),
        HConstraint::Scale => {
            let k = from.0 / to.0;
            (resolved.x_mm * k, resolved.w_mm * k)
        }
    };
    let (y_mm, h_mm) = match c.v {
        VConstraint::Top => (resolved.y_mm, resolved.h_mm),
        VConstraint::Bottom => (resolved.y_mm - (to.1 - from.1), resolved.h_mm),
        VConstraint::TopBottom => (resolved.y_mm, resolved.h_mm - (to.1 - from.1)),
        VConstraint::Center => (resolved.y_mm - (to.1 - from.1) / 2.0, resolved.h_mm),
        VConstraint::Scale => {
            let k = from.1 / to.1;
            (resolved.y_mm * k, resolved.h_mm * k)
        }
    };
    Frame {
        x_mm,
        y_mm,
        w_mm,
        h_mm,
    }
}

fn axis_keep_start(pos: f64, size: f64) -> (f64, f64) {
    (pos, size)
}

fn axis_keep_end(pos: f64, size: f64, from: f64, to: f64) -> (f64, f64) {
    let gap = from - (pos + size);
    (to - gap - size, size)
}

fn axis_stretch(pos: f64, size: f64, from: f64, to: f64) -> (f64, f64) {
    let gap = from - (pos + size);
    (pos, to - gap - pos)
}

fn axis_center(pos: f64, size: f64, from: f64, to: f64) -> (f64, f64) {
    (pos + (to - from) / 2.0, size)
}

fn axis_scale(pos: f64, size: f64, from: f64, to: f64) -> (f64, f64) {
    let k = to / from;
    (pos * k, size * k)
}

/// A box that holds children: the box they were designed against (`from`) and the box they are
/// laid out in (`to`), both in mm on the page. The page is one (origin 0, 0); a group is another
/// (rung 7a bbq, Defaulted "Layout (7a)").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Holder {
    /// The box the children's frames are stored against.
    pub from: Frame,
    /// The box they fall in now.
    pub to: Frame,
}

/// A box with no width or no height has nothing to scale by: along such an axis the target is
/// taken to be the same, so nothing divides by zero.
const DEGENERATE_MM: f64 = 1e-9;

impl Holder {
    /// The page itself, from the primary page size to the target page size.
    pub fn page(from: (f64, f64), to: (f64, f64)) -> Self {
        Holder {
            from: Frame::new(0.0, 0.0, from.0, from.1),
            to: Frame::new(0.0, 0.0, to.0, to.1),
        }
    }

    /// The same holder with its target box set.
    fn guarded(&self) -> (Frame, Frame) {
        let mut to = self.to;
        if self.from.w_mm.abs() < DEGENERATE_MM {
            to.w_mm = self.from.w_mm;
            to.x_mm = self.from.x_mm;
        }
        if self.from.h_mm.abs() < DEGENERATE_MM {
            to.h_mm = self.from.h_mm;
            to.y_mm = self.from.y_mm;
        }
        (self.from, to)
    }

    /// Where a child stored as `f` with constraints `c` falls: moved and sized against the box,
    /// not the page. On the page this is exactly [`resolve_frame`].
    pub fn resolve(&self, f: Frame, c: Constraints) -> Frame {
        let (from, to) = self.guarded();
        let rel = Frame::new(f.x_mm - from.x_mm, f.y_mm - from.y_mm, f.w_mm, f.h_mm);
        let r = resolve_frame(rel, c, (from.w_mm, from.h_mm), (to.w_mm, to.h_mm));
        Frame::new(r.x_mm + to.x_mm, r.y_mm + to.y_mm, r.w_mm, r.h_mm)
    }

    /// The inverse of [`Holder::resolve`]: the stored frame that falls on `resolved`.
    pub fn unresolve(&self, resolved: Frame, c: Constraints) -> Frame {
        let (from, to) = self.guarded();
        let rel = Frame::new(
            resolved.x_mm - to.x_mm,
            resolved.y_mm - to.y_mm,
            resolved.w_mm,
            resolved.h_mm,
        );
        let r = unresolve_frame(rel, c, (from.w_mm, from.h_mm), (to.w_mm, to.h_mm));
        Frame::new(r.x_mm + from.x_mm, r.y_mm + from.y_mm, r.w_mm, r.h_mm)
    }
}
