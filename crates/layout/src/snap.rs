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

//! The snap engine (napkin §6.2; rung 4 bbq, Defaulted "Snap distance", "What snaps"). It works
//! in the device pixels of the profile on screen, with no UI in it, so every rule is tested
//! headless.
//!
//! Rung 4a has three helpers:
//! - **Rhythm**: positions on the page's rhythm, and on its subdivisions when they are on (§6.3).
//!   A resize also snaps its *size* to whole units or subdivisions.
//! - **Margins and safe zones**: the margin lines, and the edges of the safe-zone rectangles.
//! - **Pixel grid**: when nothing else caught, the result goes onto whole device pixels.
//!
//! A snap catches within `reach` pixels (the caller converts 6 screen pixels to device pixels, so
//! it does not grow with zoom). When two targets are in reach the nearest wins; on a tie, rhythm
//! first, then margins and safe zones ([`SnapKind`] order).

use crate::RhythmPx;
use crate::units::Axis;
use quadrule_model::Document;
use quadrule_profiles::Profile;

/// Which helpers are on (rung 4 bbq, Defaulted "Toggles"). Holding Ctrl while dragging turns
/// them all off for that drag ([`SnapToggles::OFF`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapToggles {
    /// Put the result on whole device pixels when nothing else caught.
    pub pixel_grid: bool,
    /// Snap to the rhythm and its subdivisions, and sizes to whole steps.
    pub rhythm: bool,
    /// Snap to the margin lines and the safe zones' edges.
    pub margins_safe: bool,
}

impl SnapToggles {
    /// Every helper off.
    pub const OFF: SnapToggles = SnapToggles {
        pixel_grid: false,
        rhythm: false,
        margins_safe: false,
    };
}

impl Default for SnapToggles {
    /// All on, as napkin §6.2's table says.
    fn default() -> Self {
        SnapToggles {
            pixel_grid: true,
            rhythm: true,
            margins_safe: true,
        }
    }
}

/// What a snap caught. The order is the tie-break order (rung 4 bbq, Defaulted "Snap distance").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SnapKind {
    /// A rhythm unit line.
    Rhythm,
    /// A line between rhythm units (a subdivision).
    Subdivision,
    /// A size that is a whole number of units or subdivisions.
    RhythmSize,
    /// A margin line.
    Margin,
    /// The edge of a safe zone.
    SafeZone,
}

/// A line to draw while a snap holds: at `at` along `axis` (an `X` guide is a vertical line).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    /// Which coordinate `at` is.
    pub axis: Axis,
    /// Where, in device pixels.
    pub at: f64,
    /// What caught.
    pub kind: SnapKind,
}

/// A rectangle in device pixels, `x0 <= x1` and `y0 <= y1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PxRect {
    /// Left edge.
    pub x0: f64,
    /// Top edge.
    pub y0: f64,
    /// Right edge.
    pub x1: f64,
    /// Bottom edge.
    pub y1: f64,
}

impl PxRect {
    /// A rectangle from two corners in any order.
    pub fn from_corners(ax: f64, ay: f64, bx: f64, by: f64) -> Self {
        PxRect {
            x0: ax.min(bx),
            y0: ay.min(by),
            x1: ax.max(bx),
            y1: ay.max(by),
        }
    }

    /// Width.
    pub fn w(&self) -> f64 {
        self.x1 - self.x0
    }

    /// Height.
    pub fn h(&self) -> f64 {
        self.y1 - self.y0
    }

    /// The smallest rectangle holding both.
    pub fn union(&self, o: &PxRect) -> PxRect {
        PxRect {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }

    /// Moved by (dx, dy).
    pub fn moved(&self, dx: f64, dy: f64) -> PxRect {
        PxRect {
            x0: self.x0 + dx,
            y0: self.y0 + dy,
            x1: self.x1 + dx,
            y1: self.y1 + dy,
        }
    }

    /// The low and high edge along `axis`.
    pub fn span(&self, axis: Axis) -> (f64, f64) {
        match axis {
            Axis::X => (self.x0, self.x1),
            Axis::Y => (self.y0, self.y1),
        }
    }

    /// Whether the two rectangles overlap or touch.
    pub fn touches(&self, o: &PxRect) -> bool {
        self.x0 <= o.x1 && o.x0 <= self.x1 && self.y0 <= o.y1 && o.y0 <= self.y1
    }

    /// Whether the segment from (ax, ay) to (bx, by) touches this rectangle (Liang–Barsky).
    pub fn touches_segment(&self, ax: f64, ay: f64, bx: f64, by: f64) -> bool {
        let (dx, dy) = (bx - ax, by - ay);
        let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
        for (p, q) in [
            (-dx, ax - self.x0),
            (dx, self.x1 - ax),
            (-dy, ay - self.y0),
            (dy, self.y1 - ay),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return false;
                }
            } else {
                let r = q / p;
                if p < 0.0 {
                    t0 = t0.max(r);
                } else {
                    t1 = t1.min(r);
                }
                if t0 > t1 {
                    return false;
                }
            }
        }
        true
    }
}

/// A straight piece of an outline, from one point to another (always horizontal or vertical).
pub type Segment = ((f64, f64), (f64, f64));

/// The outline of the combined area of `rects`: for each rectangle, the pieces of its four edges
/// that are not inside another rectangle, each with the index of the rectangle it belongs to.
/// Where two rectangles overlap or meet, the borders inside the combined area are left out
/// (chris, 2026-11-10: "when two safe areas overlap, don't draw the borders that occur inside the
/// combined area"). Rectangles with no area are skipped.
pub fn union_outline(rects: &[PxRect]) -> Vec<(usize, Segment)> {
    // A probe this far outside an edge tells whether another rectangle lies beyond it.
    const EPS: f64 = 1e-4;
    let inside = |skip: usize, x: f64, y: f64| {
        rects.iter().enumerate().any(|(j, b)| {
            j != skip && b.w() > 0.0 && b.h() > 0.0 && x > b.x0 && x < b.x1 && y > b.y0 && y < b.y1
        })
    };
    let mut out = Vec::new();
    for (i, a) in rects.iter().enumerate() {
        if a.w() <= 0.0 || a.h() <= 0.0 {
            continue;
        }
        // (fixed coordinate, outward sign, runs along x?)
        let edges = [
            (a.y0, -1.0, true),
            (a.y1, 1.0, true),
            (a.x0, -1.0, false),
            (a.x1, 1.0, false),
        ];
        for (fixed, sign, along_x) in edges {
            let (lo, hi) = if along_x { (a.x0, a.x1) } else { (a.y0, a.y1) };
            let mut cuts = vec![lo, hi];
            for b in rects {
                let (b0, b1) = if along_x { (b.x0, b.x1) } else { (b.y0, b.y1) };
                cuts.extend([b0, b1].into_iter().filter(|&c| c > lo && c < hi));
            }
            cuts.sort_by(f64::total_cmp);
            cuts.dedup();
            for w in cuts.windows(2) {
                let mid = (w[0] + w[1]) / 2.0;
                let beyond = fixed + sign * EPS;
                let covered = if along_x {
                    inside(i, mid, beyond)
                } else {
                    inside(i, beyond, mid)
                };
                if !covered {
                    let seg = if along_x {
                        ((w[0], fixed), (w[1], fixed))
                    } else {
                        ((fixed, w[0]), (fixed, w[1]))
                    };
                    out.push((i, seg));
                }
            }
        }
    }
    out
}

/// One of the eight handles of a selection's box.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Handle {
    /// Top-left corner.
    NW,
    /// Top edge.
    N,
    /// Top-right corner.
    NE,
    /// Right edge.
    E,
    /// Bottom-right corner.
    SE,
    /// Bottom edge.
    S,
    /// Bottom-left corner.
    SW,
    /// Left edge.
    W,
}

impl Handle {
    /// Every handle.
    pub const ALL: [Handle; 8] = [
        Handle::NW,
        Handle::N,
        Handle::NE,
        Handle::E,
        Handle::SE,
        Handle::S,
        Handle::SW,
        Handle::W,
    ];

    /// The handles on the left and right edges only (text boxes grow downward by themselves).
    pub const SIDES: [Handle; 2] = [Handle::W, Handle::E];

    /// Which way the handle pulls on x: -1 the left edge, 1 the right edge, 0 neither.
    pub fn x_side(self) -> i8 {
        match self {
            Handle::NW | Handle::W | Handle::SW => -1,
            Handle::NE | Handle::E | Handle::SE => 1,
            Handle::N | Handle::S => 0,
        }
    }

    /// Which way the handle pulls on y: -1 the top edge, 1 the bottom edge, 0 neither.
    pub fn y_side(self) -> i8 {
        match self {
            Handle::NW | Handle::N | Handle::NE => -1,
            Handle::SW | Handle::S | Handle::SE => 1,
            Handle::W | Handle::E => 0,
        }
    }

    /// Where the handle sits on `r`.
    pub fn at(self, r: &PxRect) -> (f64, f64) {
        let x = match self.x_side() {
            -1 => r.x0,
            1 => r.x1,
            _ => (r.x0 + r.x1) / 2.0,
        };
        let y = match self.y_side() {
            -1 => r.y0,
            1 => r.y1,
            _ => (r.y0 + r.y1) / 2.0,
        };
        (x, y)
    }

    /// Whether the handle is a corner.
    pub fn is_corner(self) -> bool {
        self.x_side() != 0 && self.y_side() != 0
    }
}

/// How the pointer's drag changes a resize (napkin §6.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ResizeMods {
    /// Shift: keep the proportions.
    pub keep_ratio: bool,
    /// Alt/Option: resize from the centre, both sides together.
    pub from_center: bool,
}

/// The result of snapping a move.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveSnap {
    /// How much further to move along x, on top of the pointer's own movement.
    pub dx: f64,
    /// How much further to move along y.
    pub dy: f64,
    /// What caught.
    pub guides: Vec<Guide>,
}

/// The rhythm lines of one axis between `lo` and `hi` (whole pixels, both included): the
/// position and whether it is a full unit rather than a subdivision. With `subdivisions` on, a
/// unit is split in that many steps (2 to 10).
pub fn rhythm_lines(
    origin: f64,
    unit: f64,
    subdivisions: Option<u8>,
    lo: f64,
    hi: f64,
) -> Vec<(f64, bool)> {
    const EPS: f64 = 1e-9;
    if unit.is_nan() || unit <= 0.0 || !lo.is_finite() || !hi.is_finite() || lo > hi {
        return Vec::new();
    }
    let n = i64::from(subdivisions.unwrap_or(1).max(1));
    let step = unit / n as f64;
    let first = ((lo - origin) / step - EPS).ceil() as i64;
    let last = ((hi - origin) / step + EPS).floor() as i64;
    (first..=last)
        .map(|k| ((origin + k as f64 * step).round(), k.rem_euclid(n) == 0))
        .collect()
}

#[derive(Clone, Copy, Debug)]
struct Target {
    at: f64,
    kind: SnapKind,
}

/// A candidate snap: how far the feature must move, and what caught.
#[derive(Clone, Copy, Debug)]
struct Cand {
    delta: f64,
    guide_at: f64,
    kind: SnapKind,
}

fn better(a: &Cand, b: &Cand) -> bool {
    // Distances within a millionth of a pixel tie; then the kind decides.
    let (da, db) = (a.delta.abs(), b.delta.abs());
    if (da - db).abs() > 1e-6 {
        da < db
    } else {
        a.kind < b.kind
    }
}

fn best(cands: impl IntoIterator<Item = Cand>) -> Option<Cand> {
    cands.into_iter().fold(None, |acc, c| match acc {
        Some(a) if !better(&c, &a) => Some(a),
        _ => Some(c),
    })
}

/// What one axis decided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisSnap {
    /// How far to move on top of the raw position.
    pub delta: f64,
    /// The guide to show, if a target caught (not for the pixel grid).
    pub guide: Option<Guide>,
}

/// What stays put while an edge is dragged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    /// The opposite edge, at this position.
    Fixed(f64),
    /// The centre, at this position: the opposite edge mirrors the dragged one.
    Center(f64),
    /// Nothing: a single point (the end of a line).
    Free,
}

/// The page as the snap engine sees it: the rhythm, the margin lines and the safe zones, on one
/// device.
#[derive(Clone, Debug)]
pub struct SnapWorld {
    origin: (f64, f64),
    unit: (f64, f64),
    subdivisions: Option<u8>,
    page: (f64, f64),
    margin_x: Vec<Target>,
    margin_y: Vec<Target>,
}

impl SnapWorld {
    /// The world for `doc` on `shown`. `zones` are the safe-zone rectangles in device pixels
    /// that count (the caller leaves out those of toolbars that are not in use).
    pub fn new(doc: &Document, shown: &Profile, zones: &[PxRect]) -> Self {
        let r = RhythmPx::new(doc, shown);
        let m = &doc.page.margins_mm;
        let px = |mm: f64| shown.mm_to_px(mm).round();
        let (w, h) = (shown.width_px as f64, shown.height_px as f64);
        let mut margin_x = vec![
            Target {
                at: px(m.left),
                kind: SnapKind::Margin,
            },
            Target {
                at: w - px(m.right),
                kind: SnapKind::Margin,
            },
        ];
        let mut margin_y = vec![
            Target {
                at: px(m.top),
                kind: SnapKind::Margin,
            },
            Target {
                at: h - px(m.bottom),
                kind: SnapKind::Margin,
            },
        ];
        for z in zones {
            margin_x.push(Target {
                at: z.x0,
                kind: SnapKind::SafeZone,
            });
            margin_x.push(Target {
                at: z.x1,
                kind: SnapKind::SafeZone,
            });
            margin_y.push(Target {
                at: z.y0,
                kind: SnapKind::SafeZone,
            });
            margin_y.push(Target {
                at: z.y1,
                kind: SnapKind::SafeZone,
            });
        }
        SnapWorld {
            origin: (r.origin_x, r.origin_y),
            unit: (r.unit_x, r.unit_y),
            subdivisions: doc.rhythm.subdivisions_on(),
            page: (w, h),
            margin_x,
            margin_y,
        }
    }

    /// The page size in device pixels.
    pub fn page(&self) -> (f64, f64) {
        self.page
    }

    /// The rhythm lines along `axis` between `lo` and `hi`, subdivisions included when on. Only
    /// lines on the page count: the rhythm runs on past its edge, but nothing snaps there.
    pub fn lines(&self, axis: Axis, lo: f64, hi: f64) -> Vec<(f64, bool)> {
        let (o, u, page) = match axis {
            Axis::X => (self.origin.0, self.unit.0, self.page.0),
            Axis::Y => (self.origin.1, self.unit.1, self.page.1),
        };
        rhythm_lines(o, u, self.subdivisions, lo.max(0.0), hi.min(page))
    }

    /// One step of the rhythm along `axis`: a unit, or a subdivision of it.
    fn step(&self, axis: Axis) -> f64 {
        let u = match axis {
            Axis::X => self.unit.0,
            Axis::Y => self.unit.1,
        };
        u / f64::from(self.subdivisions.unwrap_or(1).max(1))
    }

    /// Position targets within `reach` of `at` along `axis`, for the helpers that are on.
    fn targets(&self, axis: Axis, at: f64, reach: f64, t: SnapToggles) -> Vec<Target> {
        let mut out = Vec::new();
        if t.rhythm {
            for (p, major) in self.lines(axis, at - reach, at + reach) {
                out.push(Target {
                    at: p,
                    kind: if major {
                        SnapKind::Rhythm
                    } else {
                        SnapKind::Subdivision
                    },
                });
            }
        }
        if t.margins_safe {
            let list = match axis {
                Axis::X => &self.margin_x,
                Axis::Y => &self.margin_y,
            };
            out.extend(list.iter().copied().filter(|m| (m.at - at).abs() <= reach));
        }
        out
    }

    /// Snaps a whole box along one axis: its low edge, centre and high edge are tried against the
    /// rhythm; its edges (not the centre) against margins and safe zones. `lo` and `hi` are where
    /// the pointer has put the box.
    pub fn snap_span(&self, axis: Axis, lo: f64, hi: f64, reach: f64, t: SnapToggles) -> AxisSnap {
        let features = [(lo, true), ((lo + hi) / 2.0, false), (hi, true)];
        let mut cands = Vec::new();
        for (f, edge) in features {
            for tg in self.targets(axis, f, reach, t) {
                let rhythm = matches!(tg.kind, SnapKind::Rhythm | SnapKind::Subdivision);
                if rhythm || edge {
                    cands.push(Cand {
                        delta: tg.at - f,
                        guide_at: tg.at,
                        kind: tg.kind,
                    });
                }
            }
        }
        self.finish(axis, best(cands), lo, t)
    }

    /// Snaps one dragged edge (or a single point, with [`Anchor::Free`]): its position against
    /// the targets and, with an anchor and the rhythm on, the box's size against whole steps.
    pub fn snap_edge(
        &self,
        axis: Axis,
        edge: f64,
        anchor: Anchor,
        reach: f64,
        t: SnapToggles,
    ) -> AxisSnap {
        let mut cands: Vec<Cand> = self
            .targets(axis, edge, reach, t)
            .into_iter()
            .map(|tg| Cand {
                delta: tg.at - edge,
                guide_at: tg.at,
                kind: tg.kind,
            })
            .collect();
        if t.rhythm {
            let step = self.step(axis);
            let (from, whole) = match anchor {
                Anchor::Fixed(a) => (Some(a), 1.0),
                Anchor::Center(c) => (Some(c), 2.0),
                Anchor::Free => (None, 1.0),
            };
            if let Some(a) = from
                && step > 0.0
            {
                let side = if edge < a { -1.0 } else { 1.0 };
                let size = (edge - a).abs() * whole;
                let k = (size / step).round().max(1.0);
                let snapped = a + side * k * step / whole;
                if (snapped - edge).abs() <= reach {
                    cands.push(Cand {
                        delta: snapped - edge,
                        guide_at: snapped,
                        kind: SnapKind::RhythmSize,
                    });
                }
            }
        }
        self.finish(axis, best(cands), edge, t)
    }

    fn finish(&self, axis: Axis, hit: Option<Cand>, pos: f64, t: SnapToggles) -> AxisSnap {
        match hit {
            Some(c) => AxisSnap {
                delta: c.delta,
                guide: Some(Guide {
                    axis,
                    at: c.guide_at,
                    kind: c.kind,
                }),
            },
            None if t.pixel_grid => AxisSnap {
                delta: pos.round() - pos,
                guide: None,
            },
            None => AxisSnap {
                delta: 0.0,
                guide: None,
            },
        }
    }

    /// Snaps a box that the pointer has moved to `raw`: how much further to move it, and the
    /// guides to show.
    pub fn snap_move(&self, raw: &PxRect, reach: f64, t: SnapToggles) -> MoveSnap {
        let x = self.snap_span(Axis::X, raw.x0, raw.x1, reach, t);
        let y = self.snap_span(Axis::Y, raw.y0, raw.y1, reach, t);
        MoveSnap {
            dx: x.delta,
            dy: y.delta,
            guides: x.guide.into_iter().chain(y.guide).collect(),
        }
    }

    /// Snaps a point (the end of a line).
    pub fn snap_point(
        &self,
        p: (f64, f64),
        reach: f64,
        t: SnapToggles,
    ) -> ((f64, f64), Vec<Guide>) {
        let x = self.snap_edge(Axis::X, p.0, Anchor::Free, reach, t);
        let y = self.snap_edge(Axis::Y, p.1, Anchor::Free, reach, t);
        (
            (p.0 + x.delta, p.1 + y.delta),
            x.guide.into_iter().chain(y.guide).collect(),
        )
    }

    /// The box after dragging `handle` of `start` by the pointer's (dx, dy), snapped (napkin §6.5).
    /// With `mods.keep_ratio` the proportions are kept; with `mods.from_center` the opposite
    /// side moves the other way. Without `keep_ratio` a side may cross its opposite and flip the
    /// box; no box gets smaller than `min_size` pixels along a side it is resized along.
    #[allow(clippy::too_many_arguments)]
    pub fn snap_resize(
        &self,
        start: &PxRect,
        handle: Handle,
        pointer: (f64, f64),
        mods: ResizeMods,
        reach: f64,
        t: SnapToggles,
        min_size: f64,
    ) -> (PxRect, Vec<Guide>) {
        let drag = [
            Dragged::new(start, Axis::X, handle.x_side(), mods.from_center),
            Dragged::new(start, Axis::Y, handle.y_side(), mods.from_center),
        ];
        let d = [pointer.0, pointer.1];
        let mut guides = Vec::new();
        let keep = mods.keep_ratio && start.w() > 0.0 && start.h() > 0.0;
        let mut spans = [start.span(Axis::X), start.span(Axis::Y)];

        if keep {
            // One scale for both axes: the larger of what the pointer asks along the dragged
            // axes. The edges that scale implies are snapped, the nearest catch decides, and the
            // other axis (an edge handle pulls only one) grows about its centre.
            let raw = drag
                .iter()
                .zip(d)
                .filter_map(|(g, d)| g.as_ref().map(|g| g.scale_for(d)))
                .fold(f64::NEG_INFINITY, f64::max);
            let raw = if raw.is_finite() { raw.max(0.0) } else { 1.0 };
            let mut caught: Option<(f64, f64, Guide)> = None;
            for g in drag.iter().flatten() {
                let implied = g.edge_at(raw);
                let snap = self.snap_edge(g.axis, implied, g.anchor, reach, t);
                if let Some(guide) = snap.guide
                    && caught.is_none_or(|(dist, ..)| snap.delta.abs() < dist)
                {
                    caught = Some((snap.delta.abs(), g.scale_to(implied + snap.delta), guide));
                }
            }
            let mut s = match caught {
                Some((_, s, guide)) => {
                    guides.push(guide);
                    s
                }
                None if t.pixel_grid => drag.iter().flatten().next().map_or(raw, |g| {
                    let implied = g.edge_at(raw);
                    g.scale_to(implied.round())
                }),
                None => raw,
            };
            let longest = start.w().max(start.h());
            s = s.max(min_size / longest).max(1e-9);
            for (i, g) in drag.iter().enumerate() {
                let (lo, hi) = start.span(if i == 0 { Axis::X } else { Axis::Y });
                spans[i] = match g {
                    Some(g) => g.span_at(s),
                    None => {
                        let (c, half) = ((lo + hi) / 2.0, (hi - lo) / 2.0 * s);
                        (c - half, c + half)
                    }
                };
            }
        } else {
            for (i, g) in drag.iter().enumerate() {
                let Some(g) = g else { continue };
                let raw = g.edge + d[i];
                let snap = self.snap_edge(g.axis, raw, g.anchor, reach, t);
                guides.extend(snap.guide);
                spans[i] = g.span_for_edge(raw + snap.delta, min_size);
            }
        }
        (
            PxRect {
                x0: spans[0].0,
                y0: spans[1].0,
                x1: spans[0].1,
                y1: spans[1].1,
            },
            guides,
        )
    }
}

/// The edge a resize pulls along one axis, and what stays put.
#[derive(Clone, Copy, Debug)]
struct Dragged {
    axis: Axis,
    /// Where the dragged edge starts.
    edge: f64,
    /// The fixed point: the opposite edge, or the centre when resizing from the centre.
    fixed: f64,
    anchor: Anchor,
}

impl Dragged {
    /// `side` is -1 for the low edge, 1 for the high edge, 0 when the handle does not pull this
    /// axis.
    fn new(r: &PxRect, axis: Axis, side: i8, from_center: bool) -> Option<Dragged> {
        if side == 0 {
            return None;
        }
        let (lo, hi) = r.span(axis);
        let (edge, other) = if side < 0 { (lo, hi) } else { (hi, lo) };
        let fixed = if from_center { (lo + hi) / 2.0 } else { other };
        Some(Dragged {
            axis,
            edge,
            fixed,
            anchor: if from_center {
                Anchor::Center(fixed)
            } else {
                Anchor::Fixed(fixed)
            },
        })
    }

    /// The scale that puts the dragged edge where the pointer, moved by `d`, asks.
    fn scale_for(&self, d: f64) -> f64 {
        ((self.edge + d - self.fixed) / (self.edge - self.fixed)).max(0.0)
    }

    /// The dragged edge's position at scale `s`.
    fn edge_at(&self, s: f64) -> f64 {
        self.fixed + (self.edge - self.fixed) * s
    }

    /// The scale that puts the dragged edge at `e`.
    fn scale_to(&self, e: f64) -> f64 {
        (e - self.fixed) / (self.edge - self.fixed)
    }

    /// The low and high edge at scale `s`, keeping the fixed point.
    fn span_at(&self, s: f64) -> (f64, f64) {
        let e = self.edge_at(s);
        match self.anchor {
            Anchor::Center(c) => (c.min(2.0 * c - e), c.max(2.0 * c - e)),
            _ => (e.min(self.fixed), e.max(self.fixed)),
        }
    }

    /// The low and high edge with the dragged edge at `e`, at least `min` apart.
    fn span_for_edge(&self, e: f64, min: f64) -> (f64, f64) {
        match self.anchor {
            Anchor::Center(c) => {
                let half = (e - c).abs().max(min / 2.0);
                (c - half, c + half)
            }
            _ => grow_to_min(e.min(self.fixed), e.max(self.fixed), e, min),
        }
    }
}

/// `lo..hi` stretched to at least `min` long, on the side away from `dragged`.
fn grow_to_min(lo: f64, hi: f64, dragged: f64, min: f64) -> (f64, f64) {
    if hi - lo >= min {
        (lo, hi)
    } else if (dragged - lo).abs() < (dragged - hi).abs() {
        (hi - min, hi)
    } else {
        (lo, lo + min)
    }
}
