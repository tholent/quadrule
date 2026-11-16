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

//! Pattern regions drawn (napkin §5.2, §6.3; rung 9 bbq, docs/bbq/2026-11-13-rung-9-pattern-library.md).
//!
//! A pattern region draws every line and dot whose centre lies in it, whole (ADR-014). Horizontal
//! and vertical lines and dots sit on whole pixels, as hard-edged rectangles ([`Item::Rect`]); the
//! slanted lines of an isometric grid or a honeycomb are anti-aliased strokes, one path for all of
//! a region's slanted lines ([`Item::StrokePath`]). An isometric grid has vertical lines one
//! column apart (axes at 30, 90 and 150 degrees). Its slanted lines, and every edge of a
//! honeycomb, run only as far as the region's frame, ending where they meet it; a hexagon's edge
//! is drawn once, whichever hexagons share it.
//!
//! A turned region (rung 12b) is drawn upright, in its own axes, by the same code, and the result is
//! then turned whole about the region's middle ([`pattern_items_turned`]): the ruling stays parallel to
//! the frame, parallel lines and dots keep ADR-014's reach, slanted lines stay cut at the frame, all
//! in the region's own axes. A quarter turn stays on whole pixels.
//!
//! Lines, dots and the square grid are the rung 1 patterns and draw exactly as they did (the
//! goldens depend on it); the rest arrived with rung 9.

use crate::{Item, RhythmPx, snap_center, stroke};
use kurbo::{Affine, BezPath};
use quadrule_model::{
    HexOrientation, IsoStyle, LineCap, LineRole, Pattern, PatternRegion, Rhythm, RhythmResolution,
    Style,
};
use quadrule_profiles::Profile;

/// The smallest spacing an unbound region may have, in pixels: a floor, so a mistyped spacing cannot
/// ask for millions of dots.
pub const MIN_SPACING_PX: f64 = 4.0;

/// The most cells (dots, hexagons) one region draws; a region asking for more draws nothing.
const MAX_CELLS: f64 = 600_000.0;

/// `sqrt(3)`.
const SQRT_3: f64 = 1.732_050_807_568_877_2;

/// A stroke's width in whole pixels and its gray.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pen {
    /// Width in pixels, at least 1.
    pub w: u32,
    /// 0 = black, 255 = white.
    pub gray: u8,
}

/// The rhythm a region follows, in pixels: where it starts and its two steps. A bound region
/// takes the document's; an unbound one starts at its own top-left corner with its own spacing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    /// Origin x.
    pub ox: f64,
    /// Origin y.
    pub oy: f64,
    /// Horizontal step.
    pub ux: f64,
    /// Vertical step.
    pub uy: f64,
}

/// What a region needs besides itself: the document's rhythm and the device.
#[derive(Clone, Copy)]
pub struct Env<'a> {
    /// The device the page is drawn for.
    pub target: &'a Profile,
    /// The document's rhythm settings (type, resolution, major lines).
    pub rhythm: &'a Rhythm,
    /// The rhythm resolved on `target`.
    pub rhythm_px: RhythmPx,
}

/// A spacing in mm as `target` resolves it, in pixels: whole when the rhythm is pixel-locked, never
/// less than [`MIN_SPACING_PX`].
pub fn spacing_px(mm: f64, rhythm: &Rhythm, target: &Profile) -> f64 {
    let px = target.mm_to_px(mm);
    let px = match rhythm.resolution {
        RhythmResolution::PixelLocked => px.round(),
        RhythmResolution::PhysicalLocked => px,
    };
    if px.is_finite() {
        px.max(MIN_SPACING_PX)
    } else {
        MIN_SPACING_PX
    }
}

/// The steps of `pr` on `env.target`: the rhythm's when it is bound, else its own spacing (the
/// rhythm's step where it has none), resolved like the rhythm.
pub fn region_spacing_px(pr: &PatternRegion, env: &Env) -> (f64, f64) {
    if pr.bound {
        return (env.rhythm_px.unit_x, env.rhythm_px.unit_y);
    }
    let (sx, sy) = env.rhythm.steps_mm();
    (
        spacing_px(pr.h_spacing_mm.unwrap_or(sx), env.rhythm, env.target),
        spacing_px(pr.v_spacing_mm.unwrap_or(sy), env.rhythm, env.target),
    )
}

/// The grid `pr` follows inside `region` (left, top, right, bottom): see [`Grid`].
pub fn region_grid(pr: &PatternRegion, env: &Env, region: (f64, f64, f64, f64)) -> Grid {
    let (ux, uy) = region_spacing_px(pr, env);
    if pr.bound {
        Grid {
            ox: env.rhythm_px.origin_x,
            oy: env.rhythm_px.origin_y,
            ux,
            uy,
        }
    } else {
        Grid {
            ox: region.0,
            oy: region.1,
            ux,
            uy,
        }
    }
}

/// Half a side of an isometric grid in pixels, given its grid. Under an isometric rhythm the
/// bound region's half side is the rhythm's vertical step; otherwise the column is the horizontal
/// step and the side is the column times 2 over the square root of three, so half a side is the
/// column over the square root of three.
pub fn iso_half(pr: &PatternRegion, env: &Env, g: &Grid) -> f64 {
    if pr.bound && env.rhythm.is_isometric() {
        g.uy
    } else {
        g.ux / SQRT_3
    }
}

/// The cue column's width and the summary band's height of a Cornell region, in rhythm units:
/// the region's own, else 30 percent of its width and 20 percent of its height, each rounded to
/// whole units and at least one.
pub fn cornell_units(
    cue: Option<u32>,
    summary: Option<u32>,
    size_px: (f64, f64),
    steps_px: (f64, f64),
) -> (u32, u32) {
    let whole = |share: f64, len: f64, unit: f64| ((share * len / unit).round().max(1.0)) as u32;
    (
        cue.unwrap_or_else(|| whole(0.3, size_px.0, steps_px.0))
            .max(1),
        summary
            .unwrap_or_else(|| whole(0.2, size_px.1, steps_px.1))
            .max(1),
    )
}

/// The rhythm positions between `start` and `end` (both included) with their index from the
/// origin, on whole pixels: [`RhythmPx::positions`] with the count of each.
fn indexed(origin: f64, unit: f64, start: f64, end: f64) -> Vec<(i64, f64)> {
    const EPS: f64 = 1e-9;
    if !(unit.is_finite() && unit > 0.0) {
        return Vec::new();
    }
    let first = ((start - origin) / unit - EPS).ceil() as i64;
    let last = ((end - origin) / unit + EPS).floor() as i64;
    (first..=last)
        .map(|k| (k, (origin + k as f64 * unit).round()))
        .collect()
}

fn is_major(k: i64, n: Option<u8>) -> bool {
    n.is_some_and(|n| k.rem_euclid(i64::from(n)) == 0)
}

/// What a pattern is drawn with.
struct Ctx<'a> {
    items: &'a mut Vec<Item>,
    /// The region: left, top, right, bottom, on whole pixels.
    reg: (f64, f64, f64, f64),
    /// The region's own line.
    pen: Pen,
    /// The major lines' role.
    major_pen: Pen,
    /// A major dot: one pixel bigger than a Dot grid dot, in the major role's gray.
    major_dot: Pen,
    /// The Structure role: Cornell's dividers and a grand staff's brace.
    divider: Pen,
    /// Every Nth line is major.
    major: Option<u8>,
    /// The index in `items` of each dot drawn (a turned ruling keeps a dot whole or drops it).
    dots: &'a mut Vec<usize>,
}

impl Ctx<'_> {
    fn pen_for(&self, k: i64) -> Pen {
        if is_major(k, self.major) {
            self.major_pen
        } else {
            self.pen
        }
    }

    /// A horizontal line across `xa..xb`, centred on the rhythm position `y`.
    fn hline(&mut self, xa: f64, xb: f64, y: f64, pen: Pen) {
        let c = snap_center(y, pen.w);
        let half = f64::from(pen.w) / 2.0;
        self.items.push(Item::Rect {
            x0: xa,
            y0: c - half,
            x1: xb,
            y1: c + half,
            gray: pen.gray,
        });
    }

    /// A vertical line down `ya..yb`, centred on the rhythm position `x`.
    fn vline(&mut self, x: f64, ya: f64, yb: f64, pen: Pen) {
        let c = snap_center(x, pen.w);
        let half = f64::from(pen.w) / 2.0;
        self.items.push(Item::Rect {
            x0: c - half,
            y0: ya,
            x1: c + half,
            y1: yb,
            gray: pen.gray,
        });
    }

    /// A dot centred on the rhythm crossing (`x`, `y`).
    fn dot(&mut self, x: f64, y: f64, pen: Pen) {
        let (cx, cy) = (snap_center(x, pen.w), snap_center(y, pen.w));
        let half = f64::from(pen.w) / 2.0;
        self.dots.push(self.items.len());
        self.items.push(Item::Rect {
            x0: cx - half,
            y0: cy - half,
            x1: cx + half,
            y1: cy + half,
            gray: pen.gray,
        });
    }

    /// Strokes `normal` and `major` (paths of slanted segments) with their pens.
    fn slanted(&mut self, normal: BezPath, major: BezPath, cap: LineCap) {
        for (path, pen) in [(normal, self.pen), (major, self.major_pen)] {
            if !path.elements().is_empty() {
                self.items.push(Item::StrokePath {
                    path,
                    width: f64::from(pen.w),
                    gray: pen.gray,
                    cap,
                });
            }
        }
    }
}

/// Appends the items of `pr` drawn in `region` (left, top, right, bottom, whole pixels) to
/// `items`. `pen` is the object's own line: the ruling, the dots. The region's own ruling angle
/// (`PatternRegion::angle_deg`) is not read here: see [`pattern_items_turned`].
pub fn pattern_items(
    pr: &PatternRegion,
    pen: Pen,
    region: (f64, f64, f64, f64),
    env: &Env,
    items: &mut Vec<Item>,
) {
    layout_in(
        pr,
        pen,
        region,
        (region.0, region.1),
        env,
        items,
        &mut Vec::new(),
    );
}

/// [`pattern_items`] with the grid of an unbound region counted from `anchor`, not from the region's
/// top-left corner (a ruling turned inside its frame lays out in a bigger box but is anchored at the
/// frame's corner, so turning it is rigid), and the index of each dot appended to `dots`.
fn layout_in(
    pr: &PatternRegion,
    pen: Pen,
    region: (f64, f64, f64, f64),
    anchor: (f64, f64),
    env: &Env,
    items: &mut Vec<Item>,
    dot_ids: &mut Vec<usize>,
) {
    let g = region_grid(pr, env, (anchor.0, anchor.1, region.2, region.3));
    let (major_w, major_gray) = stroke(Style::Role(pr.major_role), env.target);
    let (div_w, div_gray) = stroke(Style::Role(LineRole::Structure), env.target);
    // A major dot is a Dot grid dot one device pixel bigger (rung 9, chris's look).
    let (dot_w, _) = stroke(Style::Role(LineRole::DotGrid), env.target);
    let mut c = Ctx {
        items,
        reg: region,
        pen,
        major_pen: Pen {
            w: major_w,
            gray: major_gray,
        },
        major_dot: Pen {
            w: dot_w + 1,
            gray: major_gray,
        },
        divider: Pen {
            w: div_w,
            gray: div_gray,
        },
        major: pr.major_every(env.rhythm),
        dots: dot_ids,
    };
    match pr.pattern {
        Pattern::Lines => lines(&mut c, &g),
        Pattern::Dots => dots(&mut c, &g),
        Pattern::Grid => grid(&mut c, &g),
        Pattern::Isometric { style } => {
            let half = iso_half(pr, env, &g);
            match style {
                IsoStyle::Lines => isometric_lines(&mut c, &g, half),
                IsoStyle::Dots => isometric_dots(&mut c, &g, half),
            }
        }
        Pattern::Hex {
            orientation,
            height_units,
        } => hex(&mut c, &g, orientation, height_units),
        Pattern::Cornell {
            cue_units,
            summary_units,
        } => cornell(&mut c, &g, cue_units, summary_units),
        Pattern::MusicStaff {
            gap_mm,
            pitch_units,
            grand,
        } => {
            let gap = spacing_px(gap_mm, env.rhythm, env.target).max(1.0);
            music_staff(&mut c, &g, gap, pitch_units, grand);
        }
    }
}

/// The point a region of `region` (left, top, right, bottom, whole pixels) turns about by `deg`: its
/// middle. A quarter or three-quarter turn swaps the box's sides, so the middle moves to a whole
/// pixel (by at most half a pixel in each direction) to keep every line on whole pixels; a half turn
/// keeps the middle exactly, which sends whole pixels to whole pixels.
pub fn turn_pivot(region: (f64, f64, f64, f64), deg: f64) -> (f64, f64) {
    let c = ((region.0 + region.2) / 2.0, (region.1 + region.3) / 2.0);
    let quarter = {
        let d = quadrule_model::normalize_deg(deg);
        d == 90.0 || d == 270.0
    };
    if quarter {
        (c.0.round(), c.1.round())
    } else {
        c
    }
}

/// [`pattern_items`] for a region turned by `deg` degrees clockwise about its middle: the pattern is
/// laid out upright in `region`, then every item is turned with it, so the frame and its ruling
/// turn together (rung 12b bbq, Q8). At 0 this is exactly [`pattern_items`]. At a quarter turn the
/// hard-edged rectangles stay hard-edged rectangles; at any other angle they are anti-aliased
/// polygons, drawn one per run of the same gray.
///
/// A region with a ruling angle of its own (`PatternRegion::angle_deg`, Q8 as amended) first turns
/// the ruling inside the frame, about the frame's middle, and cuts it at the frame ([`ruling_items`]);
/// the frame's turn `deg` then adds to it.
pub fn pattern_items_turned(
    pr: &PatternRegion,
    pen: Pen,
    region: (f64, f64, f64, f64),
    deg: f64,
    env: &Env,
    items: &mut Vec<Item>,
) {
    let deg = quadrule_model::normalize_deg(deg);
    let own = pr.pattern_angle();
    if deg == 0.0 && own == 0.0 {
        pattern_items(pr, pen, region, env, items);
        return;
    }
    let mut upright = Vec::new();
    if own == 0.0 {
        pattern_items(pr, pen, region, env, &mut upright);
    } else {
        ruling_items(pr, pen, region, own, env, &mut upright);
    }
    if deg == 0.0 {
        items.extend(upright);
        return;
    }
    turn_items(
        upright,
        crate::shapes::rotation_about(turn_pivot(region, deg), deg),
        crate::rotate::is_quarter_turn(deg),
        items,
    );
}

/// The ruling of `pr` turned by its own angle `own` (not 0) about the middle of `region` (the frame,
/// upright), and cut at the frame. A ruling (lines, dots, grid, isometric, hexagons) is laid out over a box
/// that covers the frame at any angle, anchored at the frame's corner so turning it is rigid, so it
/// fills the whole frame; Cornell and the music staves are laid out in the frame's own box (they have
/// their own edges) and turned with it. A quarter turn keeps every rule whole-pixel and on the same
/// rule as an unturned region (ADR-014: a rule parallel to the frame reaches half a stroke past it).
/// Any other angle cuts lines at the frame exactly; a dot is kept whole when its centre is in the
/// frame (ADR-014) and dropped otherwise.
pub fn ruling_items(
    pr: &PatternRegion,
    pen: Pen,
    region: (f64, f64, f64, f64),
    own: f64,
    env: &Env,
    items: &mut Vec<Item>,
) {
    let quarter = crate::rotate::is_quarter_turn(own);
    let pivot = turn_pivot(region, own);
    let turn = crate::shapes::rotation_about(pivot, own);
    let structured = matches!(
        pr.pattern,
        Pattern::Cornell { .. } | Pattern::MusicStaff { .. }
    );
    let laid = if structured {
        region
    } else {
        // The frame's corners in the ruling's own axes: the box that holds them covers the frame.
        let corners = [
            (region.0, region.1),
            (region.2, region.1),
            (region.2, region.3),
            (region.0, region.3),
        ]
        .map(|p| crate::rotate::rotate_point(p, pivot, -own));
        let b = crate::rotate::bounds_of(&corners);
        if quarter {
            b
        } else {
            (b.0.floor(), b.1.floor(), b.2.ceil(), b.3.ceil())
        }
    };
    let mut up = Vec::new();
    let mut dots = Vec::new();
    layout_in(pr, pen, laid, (region.0, region.1), env, &mut up, &mut dots);
    if quarter {
        turn_items(up, turn, true, items);
    } else {
        turn_cut_items(up, &dots, turn, region, items);
    }
}

/// Adds the polygon `pts` to the run of `gray`, starting a new run when the gray changes.
fn add_polygon(
    run: &mut Option<(u8, BezPath)>,
    out: &mut Vec<Item>,
    gray: u8,
    pts: &[kurbo::Point],
) {
    if pts.len() < 3 {
        return;
    }
    if run.as_ref().is_some_and(|(g, _)| *g != gray) {
        flush_run(run, out);
    }
    let path = &mut run.get_or_insert_with(|| (gray, BezPath::new())).1;
    path.move_to(pts[0]);
    for p in &pts[1..] {
        path.line_to(*p);
    }
    path.close_path();
}

fn flush_run(run: &mut Option<(u8, BezPath)>, out: &mut Vec<Item>) {
    if let Some((gray, path)) = run.take() {
        out.push(Item::Fill { path, gray });
    }
}

/// The part of the convex polygon `pts` inside the box (left, top, right, bottom).
fn clip_polygon(pts: &[kurbo::Point], (x0, y0, x1, y1): (f64, f64, f64, f64)) -> Vec<kurbo::Point> {
    // Each edge of the box: keeps the points where `inside` holds; `cut` finds where a segment crosses.
    type Edge = (fn(kurbo::Point, f64) -> f64, f64);
    let edges: [Edge; 4] = [
        (|p, v| p.x - v, x0),
        (|p, v| v - p.x, x1),
        (|p, v| p.y - v, y0),
        (|p, v| v - p.y, y1),
    ];
    let mut poly = pts.to_vec();
    for (dist, v) in edges {
        if poly.is_empty() {
            break;
        }
        let mut next = Vec::with_capacity(poly.len() + 2);
        for k in 0..poly.len() {
            let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
            let (da, db) = (dist(a, v), dist(b, v));
            if da >= 0.0 {
                next.push(a);
            }
            if (da >= 0.0) != (db >= 0.0) {
                let t = da / (da - db);
                next.push(a + (b - a) * t);
            }
        }
        poly = next;
    }
    poly
}

/// The least area, in square pixels, of a ruling's line cut at the frame that is still drawn: a corner
/// the line only grazes would be a speck.
const MIN_CUT_AREA_PX: f64 = 4.0;

/// The area of the polygon `pts` (shoelace).
fn polygon_area(pts: &[kurbo::Point]) -> f64 {
    let n = pts.len();
    (0..n)
        .map(|k| {
            let (a, b) = (pts[k], pts[(k + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        .abs()
        / 2.0
}

/// Appends `up` turned by `turn` and cut at `frame` (left, top, right, bottom) to `out`: rectangles
/// (lines) become polygons cut at the frame, a dot (one of `dots`, by index) stays whole if its centre
/// is in the frame and is dropped otherwise, and slanted strokes are cut at the frame along their
/// centre line. Runs of one gray are one filled path.
fn turn_cut_items(
    up: Vec<Item>,
    dots: &[usize],
    turn: Affine,
    frame: (f64, f64, f64, f64),
    out: &mut Vec<Item>,
) {
    let mut run: Option<(u8, BezPath)> = None;
    // `dots` is in item order, so one pass along it finds each dot.
    let mut dots = dots.iter().peekable();
    for (k, it) in up.into_iter().enumerate() {
        let is_dot = dots.next_if(|d| **d == k).is_some();
        match it {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } => {
                let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
                    .map(|(x, y)| turn * kurbo::Point::new(x, y));
                if is_dot {
                    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                    let c = turn * kurbo::Point::new(cx, cy);
                    if c.x >= frame.0 && c.x <= frame.2 && c.y >= frame.1 && c.y <= frame.3 {
                        add_polygon(&mut run, out, gray, &corners);
                    }
                } else {
                    // A cut that leaves a speck (under two pixels of a two pixel line) is left out.
                    let cut = clip_polygon(&corners, frame);
                    if polygon_area(&cut) >= MIN_CUT_AREA_PX {
                        add_polygon(&mut run, out, gray, &cut);
                    }
                }
            }
            Item::StrokePath {
                path,
                width,
                gray,
                cap,
            } => {
                flush_run(&mut run, out);
                let mut cut = BezPath::new();
                let mut from: Option<kurbo::Point> = None;
                for el in path.elements() {
                    match el {
                        kurbo::PathEl::MoveTo(p) => from = Some(turn * *p),
                        kurbo::PathEl::LineTo(p) => {
                            let b = turn * *p;
                            if let Some(a) = from
                                && let Some((p0, p1)) = clip_segment((a.x, a.y), (b.x, b.y), frame)
                            {
                                cut.move_to(p0);
                                cut.line_to(p1);
                            }
                            from = Some(b);
                        }
                        _ => {}
                    }
                }
                if !cut.elements().is_empty() {
                    out.push(Item::StrokePath {
                        path: cut,
                        width,
                        gray,
                        cap,
                    });
                }
            }
            other => {
                flush_run(&mut run, out);
                turn_items(vec![other], turn, false, out);
            }
        }
    }
    flush_run(&mut run, out);
}

/// Appends `upright` moved by `turn` to `out`. At a quarter turn (`exact`) a rectangle stays a
/// rectangle; otherwise rectangles of one gray, side by side in the list, become one filled path.
fn turn_items(upright: Vec<Item>, turn: Affine, exact: bool, out: &mut Vec<Item>) {
    let mut run: Option<(u8, BezPath)> = None;
    let flush = |run: &mut Option<(u8, BezPath)>, out: &mut Vec<Item>| {
        if let Some((gray, path)) = run.take() {
            out.push(Item::Fill { path, gray });
        }
    };
    for it in upright {
        match it {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } if exact => {
                let a = turn * kurbo::Point::new(x0, y0);
                let b = turn * kurbo::Point::new(x1, y1);
                out.push(Item::Rect {
                    x0: a.x.min(b.x),
                    y0: a.y.min(b.y),
                    x1: a.x.max(b.x),
                    y1: a.y.max(b.y),
                    gray,
                });
            }
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } => {
                if run.as_ref().is_some_and(|(g, _)| *g != gray) {
                    flush(&mut run, out);
                }
                let path = &mut run.get_or_insert_with(|| (gray, BezPath::new())).1;
                let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
                    .map(|(x, y)| turn * kurbo::Point::new(x, y));
                path.move_to(corners[0]);
                for c in &corners[1..] {
                    path.line_to(*c);
                }
                path.close_path();
            }
            Item::StrokePath {
                mut path,
                width,
                gray,
                cap,
            } => {
                flush(&mut run, out);
                path.apply_affine(turn);
                out.push(Item::StrokePath {
                    path,
                    width,
                    gray,
                    cap,
                });
            }
            Item::Stroke {
                x0,
                y0,
                x1,
                y1,
                width,
                gray,
            } => {
                flush(&mut run, out);
                let a = turn * kurbo::Point::new(x0, y0);
                let b = turn * kurbo::Point::new(x1, y1);
                out.push(Item::Stroke {
                    x0: a.x,
                    y0: a.y,
                    x1: b.x,
                    y1: b.y,
                    width,
                    gray,
                });
            }
            Item::Fill { mut path, gray } => {
                flush(&mut run, out);
                path.apply_affine(turn);
                out.push(Item::Fill { path, gray });
            }
            other @ (Item::BeginAlpha { .. } | Item::EndAlpha) => {
                flush(&mut run, out);
                out.push(other);
            }
        }
    }
    flush(&mut run, out);
}

fn lines(c: &mut Ctx, g: &Grid) {
    let (x0, y0, x1, y1) = c.reg;
    for (k, y) in indexed(g.oy, g.uy, y0, y1) {
        let pen = c.pen_for(k);
        c.hline(x0, x1, y, pen);
    }
}

fn dots(c: &mut Ctx, g: &Grid) {
    let (x0, y0, x1, y1) = c.reg;
    let cols = indexed(g.ox, g.ux, x0, x1);
    for (ky, y) in indexed(g.oy, g.uy, y0, y1) {
        for &(kx, x) in &cols {
            let pen = if is_major(kx, c.major) && is_major(ky, c.major) {
                c.major_dot
            } else {
                c.pen
            };
            c.dot(x, y, pen);
        }
    }
}

fn grid(c: &mut Ctx, g: &Grid) {
    let (x0, y0, x1, y1) = c.reg;
    // Horizontal lines across the region, then vertical lines down it.
    for (k, y) in indexed(g.oy, g.uy, y0, y1) {
        let pen = c.pen_for(k);
        c.hline(x0, x1, y, pen);
    }
    for (k, x) in indexed(g.ox, g.ux, x0, x1) {
        let pen = c.pen_for(k);
        c.vline(x, y0, y1, pen);
    }
}

/// One slanted line of an isometric grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IsoSegment {
    /// -1 for "/" (y falls as x grows), 1 for "\".
    pub dir: i8,
    /// Which line of its family: the one through the crossing `index` full sides below the origin
    /// on the origin column.
    pub index: i64,
    /// Where it starts (the left end).
    pub from: (f64, f64),
    /// Where it ends (the right end).
    pub to: (f64, f64),
}

/// The slanted lines of the isometric grid with origin (`ox`, `oy`), vertical lines `col` apart and
/// half a side `half` between crossings down a column, each running edge to edge of `region`
/// (left, top, right, bottom): both families, "/" first. Empty when the grid is degenerate or
/// would hold too many lines.
pub fn iso_segments(
    (ox, oy): (f64, f64),
    col: f64,
    half: f64,
    region: (f64, f64, f64, f64),
) -> Vec<IsoSegment> {
    // The same lines as for a grid turned a quarter turn, with the two axes swapped: work in
    // (v, u) = (y, x), then swap each point back.
    let (x0, y0, x1, y1) = region;
    iso_segments_turned((oy, ox), col, half, (y0, x0, y1, x1))
        .into_iter()
        .map(|s| IsoSegment {
            dir: s.dir,
            index: s.index,
            from: (s.from.1, s.from.0),
            to: (s.to.1, s.to.0),
        })
        .collect()
}

/// The slanted lines of a grid whose horizontal rows are `row` apart (the turned grid): see
/// [`iso_segments`], which swaps its axes.
fn iso_segments_turned(
    (ox, oy): (f64, f64),
    row: f64,
    half: f64,
    region: (f64, f64, f64, f64),
) -> Vec<IsoSegment> {
    let (x0, y0, x1, y1) = region;
    let mut out = Vec::new();
    if !(half.is_finite() && half > 0.0 && row.is_finite() && row > 0.0) {
        return out;
    }
    let slope = half / row;
    let span = 2.0 * half;
    // Down-left ("/", x falls as y grows) and down-right ("\", x grows with y).
    for dir in [-1_i8, 1] {
        let d = f64::from(dir);
        // The line's x at y: `c0 + d * slope * (y - oy)`, with `c0 = ox + span * i`.
        let (lo, hi) = (slope * (y0 - oy), slope * (y1 - oy));
        let (c_min, c_max) = if dir > 0 {
            (x0 - hi, x1 - lo)
        } else {
            (x0 + lo, x1 + hi)
        };
        let i_lo = ((c_min - ox) / span).ceil() as i64;
        let i_hi = ((c_max - ox) / span).floor() as i64;
        if (i_hi - i_lo) as f64 > MAX_CELLS {
            return Vec::new();
        }
        for i in i_lo..=i_hi {
            let c0 = ox + span * i as f64;
            // The y range in which x lies inside the region.
            let (ya, yb) = if dir > 0 {
                (oy + (x0 - c0) / slope, oy + (x1 - c0) / slope)
            } else {
                (oy + (c0 - x1) / slope, oy + (c0 - x0) / slope)
            };
            let (ya, yb) = (ya.max(y0), yb.min(y1));
            if yb - ya < 1e-6 {
                continue;
            }
            let at = |y: f64| c0 + d * slope * (y - oy);
            out.push(IsoSegment {
                dir,
                index: i,
                from: (at(ya), ya),
                to: (at(yb), yb),
            });
        }
    }
    out
}

/// The triangle grid: vertical lines one column (`g.ux`) apart, and the two slanted families
/// through the crossings, half a side apart down a column, each running edge to edge of the region.
fn isometric_lines(c: &mut Ctx, g: &Grid, half: f64) {
    let (x0, y0, x1, y1) = c.reg;
    if !(half.is_finite() && half > 0.0 && g.ux > 0.0) {
        return;
    }
    for (k, x) in indexed(g.ox, g.ux, x0, x1) {
        let pen = c.pen_for(k);
        c.vline(x, y0, y1, pen);
    }
    let (mut normal, mut major) = (BezPath::new(), BezPath::new());
    for seg in iso_segments((g.ox, g.oy), g.ux, half, c.reg) {
        let path = if is_major(seg.index, c.major) {
            &mut major
        } else {
            &mut normal
        };
        path.move_to(seg.from);
        path.line_to(seg.to);
    }
    c.slanted(normal, major, LineCap::Butt);
}

/// A dot at each crossing of the isometric grid: on column `k`, at `oy + half * m` for `m` of the
/// column's parity.
fn isometric_dots(c: &mut Ctx, g: &Grid, half: f64) {
    let (x0, y0, x1, y1) = c.reg;
    if !(half.is_finite() && half > 0.0) {
        return;
    }
    let m_lo = ((y0 - g.oy) / half - 1e-9).ceil() as i64;
    let m_hi = ((y1 - g.oy) / half + 1e-9).floor() as i64;
    let cols = indexed(g.ox, g.ux, x0, x1);
    if (m_hi - m_lo) as f64 * cols.len() as f64 / 2.0 > MAX_CELLS {
        return;
    }
    for (k, x) in cols {
        for m in m_lo..=m_hi {
            if m.rem_euclid(2) != k.rem_euclid(2) {
                continue;
            }
            let y = (g.oy + half * m as f64).round();
            // A major dot: on a major column, at a major place down it (every Nth pair of steps).
            let pen = if is_major(k, c.major) && is_major(m.div_euclid(2), c.major) {
                c.major_dot
            } else {
                c.pen
            };
            c.dot(x, y, pen);
        }
    }
}

/// The part of the segment `a`-`b` inside the box (left, top, right, bottom), or `None` when
/// nothing of it is (or only a point).
fn clip_segment(
    a: (f64, f64),
    b: (f64, f64),
    (x0, y0, x1, y1): (f64, f64, f64, f64),
) -> Option<((f64, f64), (f64, f64))> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, a.0 - x0),
        (dx, x1 - a.0),
        (-dy, a.1 - y0),
        (dy, y1 - a.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    if t1 - t0 < 1e-9 {
        return None;
    }
    let at = |t: f64| (a.0 + t * dx, a.1 + t * dy);
    let (p0, p1) = (at(t0), at(t1));
    ((p1.0 - p0.0).hypot(p1.1 - p0.1) > 1e-6).then_some((p0, p1))
}

/// A honeycomb. In the frame (u along the columns, v across the flats) a flat-top hexagon has its
/// flats horizontal; a pointy-top one is the same drawn with the two axes swapped. Each hexagon
/// owns three of its edges (the top, and the two on its left), so an edge two hexagons share is
/// drawn once, and cut at the region's frame.
fn hex(c: &mut Ctx, g: &Grid, orientation: HexOrientation, height_units: u32) {
    let flat = orientation == HexOrientation::Flat;
    let (x0, y0, x1, y1) = c.reg;
    // (u, v) of a point, and back.
    let uv = |x: f64, y: f64| if flat { (x, y) } else { (y, x) };
    let xy = |u: f64, v: f64| if flat { (u, v) } else { (v, u) };
    let (ou, ov) = uv(g.ox, g.oy);
    let (u0, v0) = uv(x0, y0);
    let (u1, v1) = uv(x1, y1);
    let unit = if flat { g.uy } else { g.ux };
    let h = f64::from(height_units.clamp(1, 99)) * unit;
    if !(h.is_finite() && h > 0.0) {
        return;
    }
    let s = h / SQRT_3;
    let pitch = 1.5 * s;
    let i_lo = ((u0 - ou - s) / pitch).floor() as i64 - 2;
    let i_hi = ((u1 - ou) / pitch).ceil() as i64 + 2;
    let rows = ((v1 - v0) / h).ceil().max(1.0) + 5.0;
    if (i_hi - i_lo) as f64 * rows > MAX_CELLS {
        return;
    }
    let pen = c.pen;
    let mut slanted = BezPath::new();
    for i in i_lo..=i_hi {
        let cu = ou + s + pitch * i as f64;
        let off = if i.rem_euclid(2) == 1 { 0.5 * h } else { 0.0 };
        let j_lo = ((v0 - ov - off) / h).floor() as i64 - 2;
        let j_hi = ((v1 - ov - off) / h).ceil() as i64 + 2;
        for j in j_lo..=j_hi {
            // The hexagon's centre; its top is on the rhythm for the even columns.
            let cv = ov + h * (j as f64 + 0.5) + off;
            // The top edge, flat: sharp, on whole pixels, cut at the region's frame.
            let top = (cv - h / 2.0).round();
            if top >= v0 && top <= v1 {
                let ua = (cu - s / 2.0).round().max(u0);
                let ub = (cu + s / 2.0).round().min(u1);
                if ub > ua {
                    if flat {
                        c.hline(ua, ub, top, pen);
                    } else {
                        c.vline(top, ua, ub, pen);
                    }
                }
            }
            // The two edges at the left corner, slanted, each cut at the frame.
            let corner = (cu - s, cv);
            for far in [(cu - s / 2.0, cv - h / 2.0), (cu - s / 2.0, cv + h / 2.0)] {
                if let Some((a, b)) = clip_segment(corner, far, (u0, v0, u1, v1)) {
                    slanted.move_to(xy(a.0, a.1));
                    slanted.line_to(xy(b.0, b.1));
                }
            }
        }
    }
    // Round ends hide the joints where a slanted edge meets a sharp one.
    c.slanted(slanted, BezPath::new(), LineCap::Round);
}

/// Ruled lines on the rhythm with a cue column at the left and a summary band at the bottom. The
/// dividers are on the rhythm: the cue divider `cue` columns in from the first, the summary divider
/// `summary` rows above the last. The cue column is not ruled; the notes area and the summary
/// band are.
fn cornell(c: &mut Ctx, g: &Grid, cue: Option<u32>, summary: Option<u32>) {
    let (x0, y0, x1, y1) = c.reg;
    let rows = indexed(g.oy, g.uy, y0, y1);
    let cols = indexed(g.ox, g.ux, x0, x1);
    let (Some(&(first_row, _)), Some(&(last_row, _)), Some(&(first_col, _))) =
        (rows.first(), rows.last(), cols.first())
    else {
        return;
    };
    let (cue_u, sum_u) = cornell_units(cue, summary, (x1 - x0, y1 - y0), (g.ux, g.uy));
    // The summary divider's row, if the region is tall enough to hold one.
    let div_row = last_row - i64::from(sum_u);
    let div_row = (div_row > first_row).then_some(div_row);
    // The cue divider's column, if the region is wide enough.
    let div_col = first_col + i64::from(cue_u);
    let div_x = cols.iter().find(|(k, _)| *k == div_col).map(|&(_, x)| x);
    let divider = c.divider;
    let notes_x = div_x.unwrap_or(x0);
    let mut div_y = y1;
    for &(k, y) in &rows {
        if Some(k) == div_row {
            div_y = y;
            c.hline(x0, x1, y, divider);
            continue;
        }
        let pen = c.pen_for(k);
        match div_row {
            Some(d) if k > d => c.hline(x0, x1, y, pen),
            _ => c.hline(notes_x, x1, y, pen),
        }
    }
    if let Some(x) = div_x {
        c.vline(x, y0, div_y, divider);
    }
}

/// Staves of five lines `gap` pixels apart. Staff `n` starts on the rhythm row `pitch` times `n`
/// below the first row in the region; only a staff whose five lines all lie in the region is
/// drawn. A grand staff pairs two (treble and bass), the bass one rhythm unit below the treble's
/// last line, a pair every `pitch` rows, with a brace line down the region's left edge joining them.
fn music_staff(c: &mut Ctx, g: &Grid, gap: f64, pitch: u32, grand: bool) {
    let (x0, y0, x1, y1) = c.reg;
    let rows = indexed(g.oy, g.uy, y0, y1);
    let Some(&(first, _)) = rows.first() else {
        return;
    };
    let pitch = i64::from(pitch.clamp(1, 99));
    let height = 4.0 * gap;
    let pen = c.pen;
    for &(k, top) in &rows {
        if (k - first) % pitch != 0 {
            continue;
        }
        let staves: &[f64] = if grand { &[0.0, height + g.uy] } else { &[0.0] };
        let bottom = top + staves[staves.len() - 1] + height;
        if bottom > y1 {
            continue;
        }
        for &dy in staves {
            for i in 0..5 {
                let y = (top + dy + gap * f64::from(i)).round();
                c.hline(x0, x1, y, pen);
            }
        }
        if grand {
            let divider = c.divider;
            c.vline(x0.round(), top.round(), bottom.round(), divider);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_positions_count_from_the_origin() {
        let p = indexed(10.0, 5.0, 12.0, 27.0);
        assert_eq!(p, vec![(1, 15.0), (2, 20.0), (3, 25.0)]);
        assert!(is_major(0, Some(5)) && is_major(-5, Some(5)) && !is_major(1, Some(5)));
        assert!(!is_major(0, None));
    }

    #[test]
    fn cornell_defaults_are_a_third_and_a_fifth_in_whole_units() {
        let (cue, sum) = cornell_units(None, None, (1000.0, 2000.0), (100.0, 100.0));
        assert_eq!((cue, sum), (3, 4));
        assert_eq!(
            cornell_units(Some(2), Some(5), (1000.0, 2000.0), (100.0, 100.0)),
            (2, 5)
        );
    }
}
