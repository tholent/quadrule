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

//! Align and distribute (rung 7b, docs/bbq/2026-11-11-rung-7-groups-and-align.md, Q5 and Q6):
//! the maths, with no editor in it. Everything works on the boxes round the objects as they sit
//! on the primary page, in millimetres: a turned object's box is the box round its turned corners
//! (a group's, round the group as turned with everything above it); there is no key object.
//!
//! With the pixel-grid snap on, results land on whole device pixels of the primary profile
//! (Q6): the edge that is aligned to is on a pixel, and distribute makes the gaps a whole number
//! of pixels, spreading what is left over a pixel at a time from the first gap on. An object's
//! extent in pixels is its width rounded to a pixel, which is how the export draws it. With the
//! snap off the results are exact millimetres.

use crate::override_key;
use crate::rotate::rotate_point;
use crate::tree::{Node, resolve_nodes_on};
use crate::units::Axis;
use quadrule_model::{Document, Frame, ObjectKind, Uuid};
use quadrule_profiles::Profile;

/// What the align buttons align to (spec §6.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AlignTo {
    /// The box round the objects picked (the default for two or more).
    #[default]
    Selection,
    /// The page.
    Page,
    /// The page inside its margins.
    Margins,
}

impl AlignTo {
    /// A short stable word for keeping the choice between runs.
    pub fn key(self) -> &'static str {
        match self {
            AlignTo::Selection => "selection",
            AlignTo::Page => "page",
            AlignTo::Margins => "margins",
        }
    }

    /// The choice a [`AlignTo::key`] names.
    pub fn from_key(key: &str) -> Option<AlignTo> {
        [AlignTo::Selection, AlignTo::Page, AlignTo::Margins]
            .into_iter()
            .find(|a| a.key() == key)
    }
}

/// Which part of the box lines up, along one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The left edge (x) or the top edge (y).
    Start,
    /// The middle.
    Centre,
    /// The right edge (x) or the bottom edge (y).
    End,
}

/// What distribute keeps equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spread {
    /// The gaps between neighbouring boxes.
    Spacing,
    /// The distances between neighbouring centres.
    Centres,
}

/// The page, or the page inside its margins, as a box on the page of `profile` (mm): the primary
/// profile, or the device on screen.
pub fn target_box(doc: &Document, primary: &Profile, to: AlignTo) -> Option<Frame> {
    let (pw, ph) = primary.page_mm();
    match to {
        AlignTo::Selection => None,
        AlignTo::Page => Some(Frame::new(0.0, 0.0, pw, ph)),
        AlignTo::Margins => {
            let m = doc.page.margins_mm;
            Some(Frame::new(
                m.left,
                m.top,
                (pw - m.left - m.right).max(0.0),
                (ph - m.top - m.bottom).max(0.0),
            ))
        }
    }
}

/// The smallest box round `boxes`; `None` for none.
pub fn union_of(boxes: &[Frame]) -> Option<Frame> {
    boxes.iter().copied().reduce(|a, b| a.union(&b))
}

/// An object as align and distribute see it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boxed {
    /// The object.
    pub id: Uuid,
    /// The box round it on the primary page, turned corners included (mm).
    pub outer: Frame,
    /// The turn of everything above it (the groups it is in), in degrees: a step on the page
    /// becomes a step in the object's own space by turning back by this.
    pub holder_turn: f64,
    /// The object laid out (its frame in the space it is in, the box its constraints give, and the
    /// context it sits in): what a device's overrides are worked out against.
    pub node: Node,
}

impl Boxed {
    /// A page-space step `(dx, dy)` as a step in the space the object is stored in.
    pub fn stored_step(&self, dx: f64, dy: f64) -> (f64, f64) {
        if self.holder_turn == 0.0 {
            (dx, dy)
        } else {
            rotate_point((dx, dy), (0.0, 0.0), -self.holder_turn)
        }
    }
}

/// The boxes of `ids` on the primary page, in the order given (an id that is not there is left
/// out). The page is laid out against itself, so an object's box is where its stored frame puts it.
pub fn boxes_of(doc: &Document, primary: &Profile, ids: &[Uuid]) -> Vec<Boxed> {
    boxes_on(doc, primary, primary, ids)
}

/// The boxes of `ids` on the page of device `shown` (rung 8 bbq, Defaulted "Align and distribute on
/// a secondary"): where its constraints and its overrides put them, in the mm of that page.
pub fn boxes_on(doc: &Document, primary: &Profile, shown: &Profile, ids: &[Uuid]) -> Vec<Boxed> {
    let nodes = resolve_nodes_on(
        doc,
        primary.page_mm(),
        shown.page_mm(),
        override_key(primary, shown),
    );
    ids.iter()
        .filter_map(|id| {
            let n = nodes.iter().find(|n| n.id == *id)?;
            let o = doc.object(*id)?;
            let line = match &o.kind {
                ObjectKind::Line(l) => Some(l.direction),
                _ => None,
            };
            let outer = if n.angle == 0.0 && line.is_none() {
                n.frame
            } else {
                n.frame.turned_bounds(n.angle, line)
            };
            Some(Boxed {
                id: *id,
                outer,
                holder_turn: n.ctx.xf.deg,
                node: *n,
            })
        })
        .collect()
}

/// The grid results land on: millimetres, or the pixels of the primary profile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Grid {
    /// Exact millimetres.
    Mm,
    /// Whole device pixels, `px_per_mm` of them to the millimetre.
    Px(f64),
}

impl Grid {
    /// The grid for the primary profile with the pixel snap on or off.
    pub fn for_profile(primary: &Profile, pixel_snap: bool) -> Grid {
        if pixel_snap {
            Grid::Px(primary.mm_to_px(1.0))
        } else {
            Grid::Mm
        }
    }

    fn scale(self) -> f64 {
        match self {
            Grid::Mm => 1.0,
            Grid::Px(s) => s,
        }
    }

    /// A position put on the grid.
    fn q(self, v: f64) -> f64 {
        match self {
            Grid::Mm => v,
            Grid::Px(_) => v.round(),
        }
    }
}

/// One box along one axis, in grid units: where it starts and how long it counts as.
#[derive(Clone, Copy)]
struct Span {
    lo: f64,
    len: f64,
}

impl Span {
    fn of(f: &Frame, axis: Axis, grid: Grid) -> Span {
        let (lo, len) = match axis {
            Axis::X => (f.x_mm, f.w_mm),
            Axis::Y => (f.y_mm, f.h_mm),
        };
        let s = grid.scale();
        Span {
            lo: lo * s,
            // A pixel grid draws an extent of whole pixels.
            len: grid.q(len * s),
        }
    }

    fn hi(&self) -> f64 {
        self.lo + self.len
    }

    fn mid(&self) -> f64 {
        self.lo + self.len / 2.0
    }
}

/// How far (mm, along `axis`) each of `boxes` moves to line up its `edge` with the same part of
/// `target`. A step under a nanometre is no step.
pub fn align_deltas(
    boxes: &[Frame],
    axis: Axis,
    edge: Edge,
    target: Frame,
    grid: Grid,
) -> Vec<f64> {
    let s = grid.scale();
    let t = Span::of(&target, axis, Grid::Mm);
    // The target is exact: its length is not rounded to a pixel.
    let (t_lo, t_hi, t_mid) = (t.lo * s, t.hi() * s, t.mid() * s);
    boxes
        .iter()
        .map(|b| {
            let sp = Span::of(b, axis, grid);
            let lo = match edge {
                Edge::Start => grid.q(t_lo),
                Edge::End => grid.q(t_hi) - sp.len,
                Edge::Centre => grid.q(t_mid - sp.len / 2.0),
            };
            steady((lo - sp.lo) / s)
        })
        .collect()
}

/// How far (mm, along `axis`) each of `boxes` moves to spread them evenly: the first and the last
/// along the axis stay (put on the grid), the rest go between. `Spread::Spacing` makes the gaps
/// equal; on a pixel grid they are a whole number of pixels and what is left over goes a pixel at a
/// time into the first gaps. `Spread::Centres` makes the centres evenly spaced. Fewer than three
/// boxes move nothing.
pub fn distribute_deltas(boxes: &[Frame], axis: Axis, by: Spread, grid: Grid) -> Vec<f64> {
    let n = boxes.len();
    let none = vec![0.0; n];
    if n < 3 {
        return none;
    }
    let s = grid.scale();
    let spans: Vec<Span> = boxes.iter().map(|b| Span::of(b, axis, grid)).collect();
    // The ends, and the ones between them in order along the axis (ties by the order given).
    let key = |i: usize| match by {
        Spread::Spacing => spans[i].lo,
        Spread::Centres => spans[i].mid(),
    };
    let end = |i: usize| match by {
        Spread::Spacing => spans[i].hi(),
        Spread::Centres => spans[i].mid(),
    };
    let mut first = 0;
    let mut last = 0;
    for i in 1..n {
        if key(i) < key(first) {
            first = i;
        }
        if end(i) > end(last) {
            last = i;
        }
    }
    if first == last {
        return none;
    }
    let mut mid: Vec<usize> = (0..n).filter(|i| *i != first && *i != last).collect();
    mid.sort_by(|a, b| key(*a).total_cmp(&key(*b)).then(a.cmp(b)));
    let order: Vec<usize> = std::iter::once(first)
        .chain(mid)
        .chain(std::iter::once(last))
        .collect();

    let mut lo = vec![0.0; n];
    match by {
        Spread::Spacing => {
            let start = grid.q(spans[first].lo);
            let finish = grid.q(spans[last].hi());
            let used: f64 = spans.iter().map(|s| s.len).sum();
            let free = finish - start - used;
            let gaps = (n - 1) as f64;
            let (base, extra) = match grid {
                Grid::Mm => (free / gaps, 0.0),
                Grid::Px(_) => {
                    let base = (free / gaps).floor();
                    (base, free - base * gaps)
                }
            };
            let mut at = start;
            for (k, i) in order.iter().enumerate() {
                lo[*i] = at;
                at += spans[*i].len + base + if (k as f64) < extra { 1.0 } else { 0.0 };
            }
        }
        Spread::Centres => {
            let c0 = grid.q(spans[first].lo) + spans[first].len / 2.0;
            let c1 = grid.q(spans[last].lo) + spans[last].len / 2.0;
            let steps = (n - 1) as f64;
            for (k, i) in order.iter().enumerate() {
                let c = c0 + (c1 - c0) * (k as f64) / steps;
                lo[*i] = grid.q(c - spans[*i].len / 2.0);
            }
        }
    }
    (0..n).map(|i| steady((lo[i] - spans[i].lo) / s)).collect()
}

/// A step that is only float noise is no step.
fn steady(d: f64) -> f64 {
    if d.abs() < 1e-9 { 0.0 } else { d }
}
