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

//! Layout (napkin §10.1): turns the document, designed in millimetres on its primary profile,
//! into a display list in one target profile's device pixels. Constraints (§5.4) move and size
//! each object; the rhythm (§6.3) resolves per profile; lines and dots snap to whole pixels
//! (§10.2); text is drawn from [`ProfileOutlines`], the same structure that is saved.
//!
//! The display list lives here, not in `render` as napkin §15.1's table words it: layout makes
//! it and render only reads it, so render depends on layout and not the other way round.

#![forbid(unsafe_code)]

pub mod newpage;
pub mod snap;
pub mod units;

use kurbo::{BezPath, Point};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{
    Constraints, Document, Frame, HConstraint, ObjectKind, Pattern, RhythmOrigin, RhythmResolution,
    Style, Uuid, VConstraint,
};
use quadrule_profiles::Profile;
use quadrule_text::{FontRegistry, Laid, TextError, TextRequest};

/// One drawing command in device pixels, shared by every renderer.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A hard-edged axis-aligned rectangle on whole pixels: snapped lines and dots.
    Rect {
        /// Left edge.
        x0: f64,
        /// Top edge.
        y0: f64,
        /// Right edge.
        x1: f64,
        /// Bottom edge.
        y1: f64,
        /// 0 = black, 255 = white.
        gray: u8,
    },
    /// An anti-aliased straight stroke with butt caps (lines that are not axis-aligned).
    Stroke {
        /// Start x.
        x0: f64,
        /// Start y.
        y0: f64,
        /// End x.
        x1: f64,
        /// End y.
        y1: f64,
        /// Width in pixels.
        width: f64,
        /// 0 = black, 255 = white.
        gray: u8,
    },
    /// An anti-aliased filled path, non-zero rule: text.
    Fill {
        /// The outline, in device pixels.
        path: BezPath,
        /// 0 = black, 255 = white.
        gray: u8,
    },
}

/// Everything to draw on one profile's page.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayList {
    /// Page width in device pixels.
    pub width: u32,
    /// Page height in device pixels.
    pub height: u32,
    /// Page background gray.
    pub background: u8,
    /// Drawn in order.
    pub items: Vec<Item>,
}

/// Why a page could not be laid out.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    /// The outlines are for another profile.
    WrongProfile {
        /// The profile being laid out.
        want: String,
        /// The profile the outlines are for.
        got: String,
    },
    /// A text object has no laid-out text in the outlines.
    MissingText(Uuid),
    /// The outlines refer to a table or glyph they do not hold, or a shape is malformed.
    BadOutlines(String),
    /// Text could not be laid out.
    Text(TextError),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::WrongProfile { want, got } => {
                write!(f, "outlines are for {got}, not {want}")
            }
            LayoutError::MissingText(id) => write!(f, "no laid-out text for object {id}"),
            LayoutError::BadOutlines(e) => write!(f, "bad outlines: {e}"),
            LayoutError::Text(e) => write!(f, "text: {e}"),
        }
    }
}

impl std::error::Error for LayoutError {}

impl From<TextError> for LayoutError {
    fn from(e: TextError) -> Self {
        LayoutError::Text(e)
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

/// Where a stroke's centre goes so its edges land on whole pixels: odd widths on pixel
/// centres, even widths on pixel edges (napkin §10.2).
pub fn snap_center(pos: f64, width_px: u32) -> f64 {
    if width_px % 2 == 1 {
        pos.floor() + 0.5
    } else {
        pos.round()
    }
}

/// A style's stroke on a profile: (width in whole pixels, at least 1; gray).
pub fn stroke(style: Style, p: &Profile) -> (u32, u8) {
    match style {
        Style::Role(r) => {
            let s = p.role(r);
            (s.width_px.max(1), s.gray)
        }
        Style::Custom { width_mm, gray } => ((p.mm_to_px(width_mm).round() as u32).max(1), gray),
    }
}

/// The rhythm resolved to one profile's pixels (napkin §6.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmPx {
    /// Origin x, whole pixels.
    pub origin_x: f64,
    /// Origin y, whole pixels.
    pub origin_y: f64,
    /// Horizontal unit in pixels (whole when pixel-locked).
    pub unit_x: f64,
    /// Vertical unit in pixels (whole when pixel-locked).
    pub unit_y: f64,
    /// Pixel-locked or physical-locked.
    pub resolution: RhythmResolution,
}

impl RhythmPx {
    /// Resolves the document's rhythm on a profile.
    pub fn new(doc: &Document, p: &Profile) -> Self {
        let r = &doc.rhythm;
        let m = &doc.page.margins_mm;
        let px = |mm: f64| p.mm_to_px(mm).round();
        let unit = |mm: f64| match r.resolution {
            RhythmResolution::PixelLocked => p.mm_to_px(mm).round().max(1.0),
            RhythmResolution::PhysicalLocked => p.mm_to_px(mm),
        };
        let (unit_x, unit_y) = (unit(r.h_unit_mm), unit(r.v_unit_mm));
        // Centred: n = floor(A / u) positions span (n - 1) units, leaving at least half a unit at
        // each side of the margin box; the leftover is split evenly, an odd pixel to the end.
        let centred = |start: f64, end: f64, u: f64| {
            let a = end - start;
            let n = (a / u).floor().max(1.0);
            start + ((a - (n - 1.0) * u) / 2.0).floor()
        };
        let (origin_x, origin_y) = match r.origin {
            RhythmOrigin::MarginCorner => (px(m.left), px(m.top)),
            RhythmOrigin::PageCorner => (0.0, 0.0),
            RhythmOrigin::Centered => (
                centred(px(m.left), p.width_px as f64 - px(m.right), unit_x),
                centred(px(m.top), p.height_px as f64 - px(m.bottom), unit_y),
            ),
        };
        RhythmPx {
            origin_x,
            origin_y,
            unit_x,
            unit_y,
            resolution: r.resolution,
        }
    }

    /// Rhythm positions from `start` to `end` (pixels, both included), on whole pixels.
    pub fn positions(origin: f64, unit: f64, start: f64, end: f64) -> Vec<f64> {
        const EPS: f64 = 1e-9;
        let first = ((start - origin) / unit - EPS).ceil() as i64;
        let last = ((end - origin) / unit + EPS).floor() as i64;
        (first..=last)
            .map(|k| (origin + k as f64 * unit).round())
            .collect()
    }
}

/// The text boxes of the document as they fall on `target`, ready for [`quadrule_text::lay_out`].
/// Hidden text is included: saved outlines must cover it too.
pub fn text_requests<'a>(
    doc: &'a Document,
    primary: &Profile,
    target: &Profile,
) -> Vec<TextRequest<'a>> {
    let (from, to) = (primary.page_mm(), target.page_mm());
    doc.objects()
        .filter_map(|o| match &o.kind {
            ObjectKind::Text(tb) => Some(TextRequest {
                object_id: o.id,
                text: tb,
                width_mm: resolve_frame(o.frame, o.constraints, from, to).w_mm,
            }),
            _ => None,
        })
        .collect()
}

/// Lays out the document's text on `target` from the fonts (live text).
pub fn lay_out_text(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    reg: &mut FontRegistry,
) -> Result<Laid, LayoutError> {
    Ok(quadrule_text::lay_out(
        reg,
        &target.id,
        &text_requests(doc, primary, target),
    )?)
}

/// Lays out the whole page on `target`. Text comes from `outlines`, which are either freshly
/// laid out ([`lay_out_text`]) or read from a saved file; both draw identically.
pub fn display_list(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    outlines: &ProfileOutlines,
) -> Result<DisplayList, LayoutError> {
    if outlines.profile_id != target.id {
        return Err(LayoutError::WrongProfile {
            want: target.id.clone(),
            got: outlines.profile_id.clone(),
        });
    }
    let (from, to) = (primary.page_mm(), target.page_mm());
    let rhythm = RhythmPx::new(doc, target);
    let px = |mm: f64| target.mm_to_px(mm);
    let mut items = Vec::new();
    for layer in doc.layers.iter().filter(|l| l.visible) {
        for o in layer.objects.iter().filter(|o| o.visible) {
            let f = resolve_frame(o.frame, o.constraints, from, to);
            match &o.kind {
                ObjectKind::Line(_) => {
                    let (w, gray) = stroke(o.style, target);
                    items.push(line_item(
                        px(f.x_mm),
                        px(f.y_mm),
                        px(f.x_mm + f.w_mm),
                        px(f.y_mm + f.h_mm),
                        w,
                        gray,
                    ));
                }
                ObjectKind::Pattern(pr) => {
                    let (w, gray) = stroke(o.style, target);
                    let (x0, y0) = (px(f.x_mm).round(), px(f.y_mm).round());
                    let (x1, y1) = (px(f.x_mm + f.w_mm).round(), px(f.y_mm + f.h_mm).round());
                    let ys = RhythmPx::positions(rhythm.origin_y, rhythm.unit_y, y0, y1);
                    let half = w as f64 / 2.0;
                    // A pattern draws every line and dot whose centre lies in its region, whole: a dot
                    // on the region's edge (where the rhythm meets the margin) straddles it by half a
                    // stroke rather than being cut in two.
                    match pr.pattern {
                        Pattern::Lines => {
                            for y in ys {
                                let c = snap_center(y, w);
                                items.push(Item::Rect {
                                    x0,
                                    y0: c - half,
                                    x1,
                                    y1: c + half,
                                    gray,
                                });
                            }
                        }
                        Pattern::Dots => {
                            let xs = RhythmPx::positions(rhythm.origin_x, rhythm.unit_x, x0, x1);
                            for &y in &ys {
                                let cy = snap_center(y, w);
                                for &x in &xs {
                                    let cx = snap_center(x, w);
                                    items.push(Item::Rect {
                                        x0: cx - half,
                                        y0: cy - half,
                                        x1: cx + half,
                                        y1: cy + half,
                                        gray,
                                    });
                                }
                            }
                        }
                        Pattern::Grid => {
                            // Horizontal lines across the region, then vertical lines down it.
                            for &y in &ys {
                                let c = snap_center(y, w);
                                items.push(Item::Rect {
                                    x0,
                                    y0: c - half,
                                    x1,
                                    y1: c + half,
                                    gray,
                                });
                            }
                            for x in RhythmPx::positions(rhythm.origin_x, rhythm.unit_x, x0, x1) {
                                let c = snap_center(x, w);
                                items.push(Item::Rect {
                                    x0: c - half,
                                    y0,
                                    x1: c + half,
                                    y1,
                                    gray,
                                });
                            }
                        }
                    }
                }
                ObjectKind::Text(_) => {
                    let text = outlines.text(o.id).ok_or(LayoutError::MissingText(o.id))?;
                    let (_, gray) = stroke(o.style, target);
                    let origin = Point::new(px(f.x_mm).round(), px(f.y_mm).round());
                    let path = text_path(
                        outlines,
                        text,
                        origin,
                        target.ppi / quadrule_model::MM_PER_IN,
                    )?;
                    items.push(Item::Fill { path, gray });
                }
            }
        }
    }
    Ok(DisplayList {
        width: target.width_px,
        height: target.height_px,
        background: doc.page.background_gray,
        items,
    })
}

/// A line from (x0, y0) to (x1, y1) in pixels. Horizontal and vertical lines snap to whole
/// pixels; others are anti-aliased strokes.
fn line_item(x0: f64, y0: f64, x1: f64, y1: f64, w: u32, gray: u8) -> Item {
    let half = w as f64 / 2.0;
    if y0 == y1 {
        let c = snap_center(y0, w);
        Item::Rect {
            x0: x0.min(x1).round(),
            y0: c - half,
            x1: x0.max(x1).round(),
            y1: c + half,
            gray,
        }
    } else if x0 == x1 {
        let c = snap_center(x0, w);
        Item::Rect {
            x0: c - half,
            y0: y0.min(y1).round(),
            x1: c + half,
            y1: y0.max(y1).round(),
            gray,
        }
    } else {
        Item::Stroke {
            x0,
            y0,
            x1,
            y1,
            width: w as f64,
            gray,
        }
    }
}

/// One text box's letters as a single path in device pixels. `origin` is the box's top-left
/// corner in pixels; `px_per_mm` the profile's scale. Used for document text and by the
/// maintainer tools, so both draw text the same way.
pub fn text_path(
    outlines: &ProfileOutlines,
    text: &quadrule_model::outlines::TextOutlines,
    origin: Point,
    px_per_mm: f64,
) -> Result<BezPath, LayoutError> {
    let mut path = BezPath::new();
    let size_mm = text.size_mm as f64;
    for g in &text.glyphs {
        let table = outlines
            .tables
            .get(g.table())
            .ok_or_else(|| LayoutError::BadOutlines(format!("no table {}", g.table())))?;
        let shape = table.glyphs.get(&g.glyph()).ok_or_else(|| {
            LayoutError::BadOutlines(format!("no glyph {} in table {}", g.glyph(), g.table()))
        })?;
        if !shape.is_well_formed() || table.units_per_em == 0 {
            return Err(LayoutError::BadOutlines(format!(
                "malformed glyph {}",
                g.glyph()
            )));
        }
        // Half font units → mm → px. Font y points up; the page's y points down.
        let mm_per_half_unit = size_mm / table.units_per_em as f64 / 2.0;
        let (gx, gy) = (g.x_mm() as f64, g.y_mm() as f64);
        let pt = |cx: i16, cy: i16| {
            Point::new(
                origin.x + (gx + cx as f64 * mm_per_half_unit) * px_per_mm,
                origin.y + (gy - cy as f64 * mm_per_half_unit) * px_per_mm,
            )
        };
        let mut c = shape.coords.iter().copied();
        let mut next = || {
            let x = c.next().unwrap_or(0);
            let y = c.next().unwrap_or(0);
            pt(x, y)
        };
        for v in shape.verbs.bytes() {
            match v {
                b'M' => path.move_to(next()),
                b'L' => path.line_to(next()),
                b'Q' => {
                    let (a, b) = (next(), next());
                    path.quad_to(a, b)
                }
                b'C' => {
                    let (a, b, d) = (next(), next(), next());
                    path.curve_to(a, b, d)
                }
                _ => path.close_path(),
            }
        }
    }
    Ok(path)
}

/// Where an object falls on one profile's page, in device pixels, for picking it with the
/// pointer and outlining it (rung 3). Worked out apart from [`display_list`], so the export path
/// is untouched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// The object.
    pub id: Uuid,
    /// Its layer's index, bottom first.
    pub layer: usize,
    /// Its shape.
    pub shape: Shape,
}

/// An object's shape for picking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A line: its centre line and stroke width.
    Segment {
        /// Start x.
        x0: f64,
        /// Start y.
        y0: f64,
        /// End x.
        x1: f64,
        /// End y.
        y1: f64,
        /// Stroke width in pixels.
        width: f64,
    },
    /// A filled area: a pattern region's box, or a text box's width by its laid-out height.
    Box {
        /// Left edge.
        x0: f64,
        /// Top edge.
        y0: f64,
        /// Right edge.
        x1: f64,
        /// Bottom edge.
        y1: f64,
    },
}

impl Shape {
    /// The shape's bounds (x0, y0, x1, y1), half the stroke included for a line.
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        match *self {
            Shape::Segment {
                x0,
                y0,
                x1,
                y1,
                width,
            } => {
                let h = width / 2.0;
                (
                    x0.min(x1) - h,
                    y0.min(y1) - h,
                    x0.max(x1) + h,
                    y0.max(y1) + h,
                )
            }
            Shape::Box { x0, y0, x1, y1 } => (x0, y0, x1, y1),
        }
    }

    /// Whether (x, y) is on the shape, or within `slack` pixels of it.
    pub fn hit(&self, x: f64, y: f64, slack: f64) -> bool {
        match *self {
            Shape::Segment {
                x0,
                y0,
                x1,
                y1,
                width,
            } => {
                let (dx, dy) = (x1 - x0, y1 - y0);
                let len2 = dx * dx + dy * dy;
                let t = if len2 == 0.0 {
                    0.0
                } else {
                    (((x - x0) * dx + (y - y0) * dy) / len2).clamp(0.0, 1.0)
                };
                let (px, py) = (x0 + t * dx, y0 + t * dy);
                ((x - px).powi(2) + (y - py).powi(2)).sqrt() <= width / 2.0 + slack
            }
            Shape::Box { x0, y0, x1, y1 } => {
                x >= x0 - slack && x <= x1 + slack && y >= y0 - slack && y <= y1 + slack
            }
        }
    }
}

/// Every object's shape on `target`, hidden and locked ones included, bottom first. A text box
/// is as tall as its laid-out text in `outlines` (or its frame, when the outlines lack it).
pub fn placed_objects(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    outlines: &ProfileOutlines,
) -> Vec<Placed> {
    let (from, to) = (primary.page_mm(), target.page_mm());
    let px = |mm: f64| target.mm_to_px(mm);
    let mut out = Vec::new();
    for (li, layer) in doc.layers.iter().enumerate() {
        for o in &layer.objects {
            let f = resolve_frame(o.frame, o.constraints, from, to);
            let (x0, y0, x1, y1) = (
                px(f.x_mm),
                px(f.y_mm),
                px(f.x_mm + f.w_mm),
                px(f.y_mm + f.h_mm),
            );
            let shape = match &o.kind {
                ObjectKind::Line(_) => Shape::Segment {
                    x0,
                    y0,
                    x1,
                    y1,
                    width: stroke(o.style, target).0 as f64,
                },
                ObjectKind::Pattern(_) => Shape::Box {
                    x0: x0.min(x1),
                    y0: y0.min(y1),
                    x1: x0.max(x1),
                    y1: y0.max(y1),
                },
                ObjectKind::Text(_) => {
                    let h = outlines
                        .text(o.id)
                        .map_or(y1 - y0, |t| px(t.height_mm as f64));
                    Shape::Box {
                        x0,
                        y0,
                        x1,
                        y1: y0 + h,
                    }
                }
            };
            out.push(Placed {
                id: o.id,
                layer: li,
                shape,
            });
        }
    }
    out
}
