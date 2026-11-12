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

pub mod align;
pub mod device;
pub mod newpage;
pub mod rotate;
pub mod shapes;
pub mod snap;
pub mod tree;
pub mod units;

use kurbo::{BezPath, Point};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{
    Document, LineCap, ObjectKind, Pattern, RhythmOrigin, RhythmResolution, Style, Uuid,
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
    /// An anti-aliased filled path, non-zero rule: text, and the fill of a curved or slanted shape.
    Fill {
        /// The outline, in device pixels.
        path: BezPath,
        /// 0 = black, 255 = white.
        gray: u8,
    },
    /// An anti-aliased stroked path with mitre joins: the outline of a curved or slanted shape, and
    /// its dashes (rung 6, format 0.5). Every subpath is stroked with `cap` at its open ends.
    StrokePath {
        /// The centre line, in device pixels.
        path: BezPath,
        /// Width in pixels.
        width: f64,
        /// 0 = black, 255 = white.
        gray: u8,
        /// How open ends are finished.
        cap: LineCap,
    },
    /// Starts an opacity group (format 0.6, napkin §5.2): everything up to the matching
    /// [`Item::EndAlpha`] is drawn together, as one picture, and laid over what lies beneath with
    /// this opacity, so the export holds the flattened gray (no alpha in the PNG). Groups nest (rung
    /// 7a: a group of objects that are themselves faded): each is composited on its own, inside the
    /// one round it.
    BeginAlpha {
        /// 0 (clear) to 255 (opaque).
        alpha: u8,
        /// A box holding everything in the group, (left, top, right, bottom) in device pixels.
        bounds: (f64, f64, f64, f64),
    },
    /// Ends the opacity group begun by [`Item::BeginAlpha`].
    EndAlpha,
}

/// The box (left, top, right, bottom) that holds every item, strokes' width included, or `None`
/// if there are none. Used for the bounds of an opacity group.
pub fn items_bounds(items: &[Item]) -> Option<(f64, f64, f64, f64)> {
    let mut b: Option<(f64, f64, f64, f64)> = None;
    let mut add = |x0: f64, y0: f64, x1: f64, y1: f64| {
        b = Some(match b {
            None => (x0, y0, x1, y1),
            Some(o) => (o.0.min(x0), o.1.min(y0), o.2.max(x1), o.3.max(y1)),
        });
    };
    for it in items {
        match it {
            Item::Rect { x0, y0, x1, y1, .. } => {
                add(x0.min(*x1), y0.min(*y1), x0.max(*x1), y0.max(*y1))
            }
            Item::Stroke {
                x0,
                y0,
                x1,
                y1,
                width,
                ..
            } => {
                let h = width / 2.0;
                add(
                    x0.min(*x1) - h,
                    y0.min(*y1) - h,
                    x0.max(*x1) + h,
                    y0.max(*y1) + h,
                );
            }
            Item::Fill { path, .. } => {
                let r = kurbo::Shape::bounding_box(path);
                add(r.x0, r.y0, r.x1, r.y1);
            }
            Item::StrokePath { path, width, .. } => {
                // A miter join reaches up to the limit (4) times half the width past the centre line.
                let r = kurbo::Shape::bounding_box(path);
                let h = width * 2.0;
                add(r.x0 - h, r.y0 - h, r.x1 + h, r.y1 + h);
            }
            Item::BeginAlpha { .. } | Item::EndAlpha => {}
        }
    }
    b
}

/// The 0 to 255 alpha of an opacity in percent.
pub fn alpha_of_percent(pct: u8) -> u8 {
    ((u32::from(pct.min(100)) * 255 + 50) / 100) as u8
}

/// Wraps `items[start..]` in an opacity group of `pct` percent: nothing at 100, and at 0 the items
/// are dropped (they would not show).
fn wrap_opacity(items: &mut Vec<Item>, start: usize, pct: u8) {
    if pct >= 100 || start >= items.len() {
        return;
    }
    if pct == 0 {
        items.truncate(start);
        return;
    }
    let Some(bounds) = items_bounds(&items[start..]) else {
        return;
    };
    items.insert(
        start,
        Item::BeginAlpha {
            alpha: alpha_of_percent(pct),
            bounds,
        },
    );
    items.push(Item::EndAlpha);
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

pub use quadrule_model::geom::{Holder, resolve_frame, unresolve_frame};

/// Where a stroke's centre goes so its edges land on whole pixels: odd widths on pixel
/// centres, even widths on pixel edges (napkin §10.2).
pub fn snap_center(pos: f64, width_px: u32) -> f64 {
    if width_px % 2 == 1 {
        pos.floor() + 0.5
    } else {
        pos.round()
    }
}

/// The renderers' cap for the display list's (one kurbo is shared by vello_cpu and vello).
pub fn kurbo_cap(c: LineCap) -> kurbo::Cap {
    match c {
        LineCap::Butt => kurbo::Cap::Butt,
        LineCap::Round => kurbo::Cap::Round,
        LineCap::Square => kurbo::Cap::Square,
    }
}

/// A style's stroke on a profile: (width in whole pixels, at least 1; gray). [`Style::None`] gives
/// width 0 (and white): use [`stroke_opt`] to tell a missing stroke from a drawn one.
pub fn stroke(style: Style, p: &Profile) -> (u32, u8) {
    stroke_opt(style, p).unwrap_or((0, 255))
}

/// A style's stroke on a profile, or `None` for [`Style::None`].
pub fn stroke_opt(style: Style, p: &Profile) -> Option<(u32, u8)> {
    Some(match style {
        Style::None => return None,
        Style::Role(r) => {
            let s = p.role(r);
            (s.width_px.max(1), s.gray)
        }
        Style::Custom { width_mm, gray } => ((p.mm_to_px(width_mm).round() as u32).max(1), gray),
    })
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

/// The key under which `target`'s overrides are read: its profile id, or `None` for the primary
/// profile, which shows the stored design (an override for it is never read; rung 8 bbq, Q3).
pub fn override_key<'a>(primary: &Profile, target: &'a Profile) -> Option<&'a str> {
    (target.id != primary.id).then_some(target.id.as_str())
}

/// One text box to lay out on a profile: the object's text as it falls there (with its font-size
/// override, if any) and the width of its box there.
#[derive(Debug, Clone)]
pub struct TextJob<'a> {
    /// The text object's id.
    pub object_id: Uuid,
    /// Its text and style on this device.
    pub text: std::borrow::Cow<'a, quadrule_model::TextBox>,
    /// The box width on this profile, in mm.
    pub width_mm: f64,
}

/// The text boxes of the document as they fall on `target`, ready for [`quadrule_text::lay_out`]
/// (see [`lay_out_text`]): a text box takes the width its place gives it on this device, and the
/// font size an override gives. Hidden text is included: saved outlines must cover it too.
pub fn text_jobs<'a>(doc: &'a Document, primary: &Profile, target: &Profile) -> Vec<TextJob<'a>> {
    let ov = override_key(primary, target);
    let nodes = tree::resolve_nodes_on(doc, primary.page_mm(), target.page_mm(), ov);
    doc.objects()
        .filter_map(|o| match &o.kind {
            ObjectKind::Text(tb) => {
                let size = ov
                    .and_then(|p| o.override_on(p))
                    .and_then(|x| x.font_size_pt)
                    .filter(|s| s.is_finite() && *s > 0.0);
                let text = match size {
                    Some(pt) if pt != tb.size_pt => {
                        let mut t = tb.clone();
                        t.size_pt = pt;
                        std::borrow::Cow::Owned(t)
                    }
                    _ => std::borrow::Cow::Borrowed(tb),
                };
                Some(TextJob {
                    object_id: o.id,
                    text,
                    // The width it falls at on this page.
                    width_mm: nodes.iter().find(|n| n.id == o.id)?.local.w_mm,
                })
            }
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
    let jobs = text_jobs(doc, primary, target);
    let requests: Vec<TextRequest> = jobs
        .iter()
        .map(|j| TextRequest {
            object_id: j.object_id,
            text: &j.text,
            width_mm: j.width_mm,
        })
        .collect();
    Ok(quadrule_text::lay_out(reg, &target.id, &requests)?)
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
    let rhythm = RhythmPx::new(doc, target);
    let env = Env {
        target,
        rhythm,
        outlines,
        ov: override_key(primary, target),
    };
    let mut items = Vec::new();
    let ctx = tree::Ctx::page(primary.page_mm(), target.page_mm());
    for layer in doc.layers.iter().filter(|l| l.visible) {
        draw_objects(&layer.objects, &ctx, &env, &mut items)?;
    }
    Ok(DisplayList {
        width: target.width_px,
        height: target.height_px,
        background: doc.page.background_gray,
        items,
    })
}

/// What drawing an object needs besides the object.
struct Env<'a> {
    target: &'a Profile,
    rhythm: RhythmPx,
    outlines: &'a ProfileOutlines,
    /// The device whose overrides are read; `None` for the primary profile.
    ov: Option<&'a str>,
}

/// Draws `objs` in order into `items`; a group's children are drawn inside it, and a group below
/// 100 percent is one opacity group, so its children do not show through each other (rung 7a bbq,
/// Q7).
fn draw_objects(
    objs: &[quadrule_model::Object],
    ctx: &tree::Ctx,
    env: &Env,
    items: &mut Vec<Item>,
) -> Result<(), LayoutError> {
    let (target, rhythm, outlines) = (env.target, env.rhythm, env.outlines);
    let px = |mm: f64| target.mm_to_px(mm);
    for o in objs
        .iter()
        .filter(|o| env.ov.map_or(o.visible, |p| o.visible_on(p)))
    {
        let placing = tree::place_object(o, ctx, env.ov);
        let (f, angle) = (placing.frame, placing.angle);
        let first_item = items.len();
        if let (ObjectKind::Group(g), Some(inner)) = (&o.kind, &placing.inner) {
            draw_objects(&g.children, inner, env, items)?;
            wrap_opacity(items, first_item, o.opacity());
            continue;
        }
        match &o.kind {
            ObjectKind::Line(line) => {
                let Some((w, gray)) = stroke_opt(o.style, target) else {
                    continue;
                };
                let frame_px = (
                    px(f.x_mm),
                    px(f.y_mm),
                    px(f.x_mm + f.w_mm),
                    px(f.y_mm + f.h_mm),
                );
                // The line's two ends: along the frame's diagonal it points, turned about the
                // frame's centre.
                let ((x0, y0), (x1, y1)) =
                    rotate::rotated_line_ends(frame_px, line.direction, angle);
                if o.dash.is_solid() && o.cap.is_butt() {
                    items.push(line_item(x0, y0, x1, y1, w, gray));
                } else {
                    items.extend(shapes::line_items(
                        (x0, y0, x1, y1),
                        w,
                        gray,
                        o.dash,
                        o.cap,
                        target,
                    ));
                }
            }
            ObjectKind::Rect(_) | ObjectKind::Ellipse(_) | ObjectKind::Polygon(_) => {
                let (x0, y0) = (px(f.x_mm), px(f.y_mm));
                let (x1, y1) = (px(f.x_mm + f.w_mm), px(f.y_mm + f.h_mm));
                items.extend(shapes::shape_items_turned(
                    o,
                    (x0, y0, x1, y1),
                    angle,
                    target,
                ));
            }
            ObjectKind::Pattern(pr) => {
                let Some((w, gray)) = stroke_opt(o.style, target) else {
                    continue;
                };
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
                let Some((_, gray)) = stroke_opt(o.style, target) else {
                    continue;
                };
                let origin = Point::new(px(f.x_mm).round(), px(f.y_mm).round());
                let path = text_path(
                    outlines,
                    text,
                    origin,
                    target.ppi / quadrule_model::MM_PER_IN,
                )?;
                items.push(Item::Fill { path, gray });
            }
            // Handled above: a group draws its children.
            ObjectKind::Group(_) => {}
        }

        wrap_opacity(items, first_item, o.opacity());
    }
    Ok(())
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
    /// Its shape. A group's is its box, turned with it.
    pub shape: Shape,
    /// Where it sits in the tree and falls on the page (rung 7a).
    pub node: tree::Node,
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
    /// A rotated shape's box (format 0.6): centred on (`cx`, `cy`), `hw` and `hh` the half width and
    /// half height of the unrotated box, turned `deg` degrees clockwise about its centre.
    Rotated {
        /// Centre x.
        cx: f64,
        /// Centre y.
        cy: f64,
        /// Half the unrotated width.
        hw: f64,
        /// Half the unrotated height.
        hh: f64,
        /// Degrees clockwise.
        deg: f64,
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
            Shape::Rotated { .. } => rotate::bounds_of(&self.corners().unwrap_or([(0.0, 0.0); 4])),
        }
    }

    /// A rotated box's four corners (top-left, top-right, bottom-right, bottom-left of the
    /// unrotated box, after turning); `None` for the other shapes.
    pub fn corners(&self) -> Option<[(f64, f64); 4]> {
        match *self {
            Shape::Rotated {
                cx,
                cy,
                hw,
                hh,
                deg,
            } => Some(rotate::rotated_corners(
                (cx - hw, cy - hh, cx + hw, cy + hh),
                deg,
            )),
            _ => None,
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
            Shape::Rotated {
                cx,
                cy,
                hw,
                hh,
                deg,
            } => {
                // Turn the point back by the angle, then it is a plain box.
                let (px, py) = rotate::rotate_point((x, y), (cx, cy), -deg);
                (px - cx).abs() <= hw + slack && (py - cy).abs() <= hh + slack
            }
        }
    }
}

/// Every object's shape on `target`, hidden and locked ones included, bottom first; a group is
/// listed before what is in it, and its shape is its box (rung 7a). A text box is as tall as its
/// laid-out text in `outlines` (or its frame, when the outlines lack it).
pub fn placed_objects(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    outlines: &ProfileOutlines,
) -> Vec<Placed> {
    let px = |mm: f64| target.mm_to_px(mm);
    let nodes = tree::resolve_nodes_on(
        doc,
        primary.page_mm(),
        target.page_mm(),
        override_key(primary, target),
    );
    let mut out = Vec::with_capacity(nodes.len());
    for node in nodes {
        let Some(o) = doc.object(node.id) else {
            continue;
        };
        let f = node.frame;
        let angle = node.angle;
        let (x0, y0, x1, y1) = (
            px(f.x_mm),
            px(f.y_mm),
            px(f.x_mm + f.w_mm),
            px(f.y_mm + f.h_mm),
        );
        let shape = match &o.kind {
            ObjectKind::Line(line) => {
                let ((x0, y0), (x1, y1)) =
                    rotate::rotated_line_ends((x0, y0, x1, y1), line.direction, angle);
                Shape::Segment {
                    x0,
                    y0,
                    x1,
                    y1,
                    width: stroke(o.style, target).0 as f64,
                }
            }
            ObjectKind::Rect(_)
            | ObjectKind::Ellipse(_)
            | ObjectKind::Polygon(_)
            | ObjectKind::Group(_)
                if angle != 0.0 =>
            {
                let b = (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1));
                let (cx, cy) = rotate::centre(b);
                Shape::Rotated {
                    cx,
                    cy,
                    hw: (b.2 - b.0) / 2.0,
                    hh: (b.3 - b.1) / 2.0,
                    deg: angle,
                }
            }
            ObjectKind::Pattern(_)
            | ObjectKind::Rect(_)
            | ObjectKind::Ellipse(_)
            | ObjectKind::Polygon(_)
            | ObjectKind::Group(_) => Shape::Box {
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
            layer: node.layer,
            shape,
            node,
        });
    }
    out
}
