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

//! Drawing a new object with a tool (rung 5 step 3, docs/bbq/2026-11-10-rung-5-new-and-insert.md,
//! Defaulted "Drawing (Q9)" and "Per kind"), with no UI in it. The window feeds in the pointer
//! in device pixels of the profile on screen; everything here is tested headless.
//!
//! A drag draws the object live and lands it as one undo step on release, like a move
//! ([`crate::drag`]): while the pointer is down the open document is not touched; the page
//! shows a *preview* copy with the new object in it. A plain click, or a drag shorter than the
//! snap reach, drops the object at its default size. The drawn start corner and end snap with
//! rung 4's snap engine, so a drawn size lands on whole rhythm units.

use crate::drag::{DragMods, MIN_SIZE_PX, frame_from_px, others};
use crate::tools::Tool;
use crate::viewer::{Notice, Viewer};
use quadrule_i18n::Message;
use quadrule_layout::RhythmPx;
use quadrule_layout::snap::{Guide, Handle, Mark, PxRect, ResizeMods, SnapToggles, SnapWorld};
use quadrule_model::edit::Command;
use quadrule_model::sample::ATKINSON;
use quadrule_model::{
    Constraints, DashStyle, EllipseShape, Extra, Frame, HConstraint, Line, LineCap, LineDir,
    LineRole, Object, ObjectKind, Pattern, PatternRegion, PolygonShape, RectShape, Style,
    TextAlign, TextBox, Uuid, VConstraint,
};
use quadrule_profiles::Profile;

/// How long a clicked line and a clicked text box are, in rhythm units (Defaulted "Per kind").
pub const DEFAULT_UNITS: f64 = 10.0;

/// What a drawing tool makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawKind {
    /// A straight line, role Guide.
    Line,
    /// A text box, role Text.
    Text,
    /// A pattern region of ruled lines, role Guide, spacing bound to the rhythm.
    Pattern,
    /// A rectangle, stroke role Structure, no fill, square corners (rung 6a).
    Rect,
    /// An ellipse, stroke role Structure, no fill (rung 6a).
    Ellipse,
    /// A six-sided polygon, point up, stroke role Structure, no fill (rung 6a).
    Polygon,
}

impl DrawKind {
    /// A rectangle, ellipse or polygon: the kinds that draw from a corner to a corner and that
    /// Shift keeps regular.
    pub fn is_shape(self) -> bool {
        matches!(self, DrawKind::Rect | DrawKind::Ellipse | DrawKind::Polygon)
    }

    /// The width over the height Shift keeps: 1 for a square or circle, the regular polygon's own
    /// shape for a polygon (a regular hexagon is narrower than it is tall).
    pub fn regular_ratio(self) -> f64 {
        match self {
            DrawKind::Polygon => quadrule_layout::shapes::natural_ratio(&new_polygon()),
            _ => 1.0,
        }
    }
}

/// What a new polygon is: six sides, a regular polygon with a point up (rung 6 bbq, Defaulted
/// "Polygon geometry").
pub fn new_polygon() -> PolygonShape {
    PolygonShape::polygon(6)
}

/// The names a new object gets, and a new text box's text, in the words on screen (the window
/// fills them from Fluent). They are stored in the document as its own data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewNames {
    /// A line's name.
    pub line: String,
    /// A text box's name.
    pub text: String,
    /// A pattern region's name.
    pub pattern: String,
    /// A rectangle's name.
    pub rect: String,
    /// An ellipse's name.
    pub ellipse: String,
    /// A polygon's name.
    pub polygon: String,
    /// What a new text box says.
    pub text_content: String,
    /// A new group's name (rung 7a).
    pub group: String,
}

impl Default for NewNames {
    fn default() -> Self {
        NewNames {
            line: "Line".into(),
            text: "Text".into(),
            pattern: "Pattern".into(),
            rect: "Rectangle".into(),
            ellipse: "Ellipse".into(),
            polygon: "Polygon".into(),
            text_content: "Text".into(),
            group: "Group".into(),
        }
    }
}

/// A drawing in progress.
#[derive(Clone, Debug)]
pub struct Draw {
    kind: DrawKind,
    /// Where the pointer went down, in device pixels.
    press: (f64, f64),
    /// The start corner, snapped.
    start: (f64, f64),
    /// The layer the object goes into.
    layer: usize,
    world: SnapWorld,
    /// The pointer has gone further than the snap reach: this is a drag, not a click.
    moved: bool,
    /// The box drawn so far, in device pixels; none until the pointer has moved.
    rect: Option<PxRect>,
    /// The object being drawn.
    object: Option<Object>,
    /// The diagonal a line being drawn runs along: up when the pointer went up and to the right
    /// (or down and to the left) of the start.
    line_dir: LineDir,
    /// What caught on the last move.
    pub guides: Vec<Guide>,
    /// Points to draw for the last move.
    pub marks: Vec<Mark>,
    /// The selection before, put back by Escape.
    selection_before: Vec<Uuid>,
}

impl Draw {
    /// What is being drawn.
    pub fn kind(&self) -> DrawKind {
        self.kind
    }
}

fn mm_object(
    id: Uuid,
    name: &str,
    frame: quadrule_model::Frame,
    c: Constraints,
    style: Style,
    kind: ObjectKind,
) -> Object {
    Object {
        id,
        name: name.into(),
        frame,
        constraints: c,
        visible: true,
        locked: false,
        style,
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        kind,
        extra: Extra::new(),
    }
}

/// A box snapped on its dominant side, made regular again: `ratio` wide for every 1 tall, from the
/// corner `from` (or about it, with `centred`), going the way the pointer went (`delta`).
fn regularise(r: PxRect, from: (f64, f64), delta: (f64, f64), ratio: f64, centred: bool) -> PxRect {
    let w = r.w().max(r.h() * ratio);
    let h = w / ratio;
    let (sx, sy) = (delta.0.signum(), delta.1.signum());
    if centred {
        let (cx, cy) = ((r.x0 + r.x1) / 2.0, (r.y0 + r.y1) / 2.0);
        PxRect::from_corners(cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0)
    } else {
        PxRect::from_corners(from.0, from.1, from.0 + sx * w, from.1 + sy * h)
    }
}

/// The frame, on the primary page, of a new object of `kind` filling `rect` (device pixels on
/// `shown`). A line's box is its diagonal (top-left to bottom-right); a text box takes the
/// box's left, top and width, and is one rhythm unit tall until its text decides.
fn frame_for(
    kind: DrawKind,
    rect: PxRect,
    constraints: Constraints,
    primary: &Profile,
    shown: &Profile,
    unit_y: f64,
) -> Frame {
    let rect = match kind {
        DrawKind::Text => PxRect {
            y1: rect.y0 + unit_y,
            ..rect
        },
        _ => rect,
    };
    frame_from_px(rect, constraints, primary, shown)
}

/// A new object of `kind` in `frame`; `dir` is the diagonal of a line.
fn make_object(
    kind: DrawKind,
    frame: Frame,
    constraints: Constraints,
    names: &NewNames,
    id: Uuid,
    dir: LineDir,
) -> Object {
    match kind {
        DrawKind::Line => mm_object(
            id,
            &names.line,
            frame,
            constraints,
            Style::Role(LineRole::Guide),
            ObjectKind::Line(Line {
                direction: dir,
                ..Line::default()
            }),
        ),
        DrawKind::Text => mm_object(
            id,
            &names.text,
            frame,
            constraints,
            Style::Role(LineRole::Text),
            ObjectKind::Text(TextBox {
                content: names.text_content.clone(),
                family: ATKINSON.into(),
                weight: 400,
                size_pt: 12.0,
                line_height: 1.25,
                letter_spacing_em: 0.0,
                align: TextAlign::Left,
                extra: Extra::new(),
            }),
        ),
        DrawKind::Pattern => mm_object(
            id,
            &names.pattern,
            frame,
            constraints,
            Style::Role(LineRole::Guide),
            ObjectKind::Pattern(PatternRegion {
                pattern: Pattern::Lines,
                extra: Extra::new(),
            }),
        ),
        DrawKind::Rect => mm_object(
            id,
            &names.rect,
            frame,
            constraints,
            Style::Role(LineRole::Structure),
            ObjectKind::Rect(RectShape::default()),
        ),
        DrawKind::Ellipse => mm_object(
            id,
            &names.ellipse,
            frame,
            constraints,
            Style::Role(LineRole::Structure),
            ObjectKind::Ellipse(EllipseShape::default()),
        ),
        DrawKind::Polygon => mm_object(
            id,
            &names.polygon,
            frame,
            constraints,
            Style::Role(LineRole::Structure),
            ObjectKind::Polygon(new_polygon()),
        ),
    }
}

impl Viewer {
    /// Whether a drawing is in progress.
    pub fn drawing(&self) -> bool {
        self.draw.is_some()
    }

    /// Switches tool. A drawing in progress is dropped; a tool picked this way is not kept on.
    pub fn set_tool(&mut self, tool: Tool) {
        self.draw_cancel();
        self.tool = tool;
        self.tool_kept = false;
    }

    /// Keeps `tool` on after each object (a double-click on its button).
    pub fn keep_tool(&mut self, tool: Tool) {
        self.draw_cancel();
        self.tool = tool;
        self.tool_kept = tool != Tool::Select;
    }

    /// Escape with a tool on and nothing being drawn: back to Select. Returns whether it did.
    pub fn end_tool(&mut self) -> bool {
        if self.tool == Tool::Select {
            return false;
        }
        self.set_tool(Tool::Select);
        true
    }

    /// The layer a new object goes into: the first selected object's layer, or else the top
    /// unlocked one. `None`, with a notice, when that layer is locked or every layer is.
    fn draw_layer(&mut self) -> Option<usize> {
        let doc = self.doc()?;
        let from_selection = self.selection.iter().find_map(|id| doc.layer_of(*id));
        let layer = from_selection.or_else(|| doc.layers.iter().rposition(|l| !l.locked));
        if layer.is_none_or(|l| doc.layers[l].locked) {
            self.notice = Some(Notice {
                message: Message::new("notice-locked"),
                error: true,
            });
            return None;
        }
        layer
    }

    /// The press at (x, y) device pixels with the drawing tool on. Returns false when nothing
    /// started (no drawing tool, nothing open, something already being dragged or drawn, or the
    /// layer is locked).
    pub fn draw_begin(&mut self, x: f64, y: f64, mods: DragMods, reach: f64) -> bool {
        let Some(kind) = self.tool.draws() else {
            return false;
        };
        if self.open.is_none() || self.drag.is_some() || self.draw.is_some() {
            return false;
        }
        let Some(layer) = self.draw_layer() else {
            return false;
        };
        let Some((_, shown, placed)) = self.geometry() else {
            return false;
        };
        let Some(doc) = self.doc().cloned() else {
            return false;
        };
        let zones = crate::viewer::active_zone_rects(&shown);
        let world = SnapWorld::new(&doc, &shown, &zones).with_others(others(&doc, &placed, &[]));
        let toggles = self.toggles_for(mods);
        let start = world.snap_point((x, y), reach, toggles).at;
        self.draw = Some(Draw {
            kind,
            press: (x, y),
            start,
            layer,
            world,
            moved: false,
            rect: None,
            object: None,
            line_dir: LineDir::Down,
            guides: Vec::new(),
            marks: Vec::new(),
            selection_before: self.selection.clone(),
        });
        true
    }

    fn toggles_for(&self, mods: DragMods) -> SnapToggles {
        if mods.snapping_off {
            SnapToggles::OFF
        } else {
            self.snap
        }
    }

    /// The pointer is at (x, y) device pixels with `mods` held; `reach` is the snap distance in
    /// device pixels. The object is shown being drawn; nothing is changed in the document yet.
    pub fn draw_update(&mut self, x: f64, y: f64, mods: DragMods, reach: f64) {
        let Some(mut draw) = self.draw.take() else {
            return;
        };
        let toggles = self.toggles_for(mods);
        if !draw.moved && (x - draw.press.0).hypot(y - draw.press.1) >= reach {
            draw.moved = true;
        }
        if !draw.moved {
            self.draw = Some(draw);
            return;
        }
        let Some((primary, shown)) = self.profiles() else {
            self.draw = Some(draw);
            return;
        };
        let Some(base) = self.doc().cloned() else {
            self.draw = Some(draw);
            return;
        };
        let s = draw.start;
        let (rect, guides, marks) = match draw.kind {
            DrawKind::Line => {
                // A line goes where the pointer goes, up or down, left or right (its frame is
                // the box between the two ends and it keeps whichever diagonal they make). With
                // Shift held it runs along whichever axis the pointer went further on.
                let (dx, dy) = (x - s.0, y - s.1);
                let axis = mods.shift;
                let horizontal = dx.abs() >= dy.abs();
                let raw = match (axis, horizontal) {
                    (true, true) => (x, s.1),
                    (true, false) => (s.0, y),
                    (false, _) => (x, y),
                };
                let snapped = draw.world.snap_point(raw, reach, toggles);
                let end = match (axis, horizontal) {
                    (true, true) => (snapped.at.0, s.1),
                    (true, false) => (s.0, snapped.at.1),
                    (false, _) => snapped.at,
                };
                draw.line_dir = if (end.0 - s.0) * (end.1 - s.1) < 0.0 {
                    LineDir::Up
                } else {
                    LineDir::Down
                };
                (
                    PxRect::from_corners(s.0, s.1, end.0, end.1),
                    snapped.guides,
                    snapped.marks,
                )
            }
            DrawKind::Text
            | DrawKind::Pattern
            | DrawKind::Rect
            | DrawKind::Ellipse
            | DrawKind::Polygon => {
                let (mut dx, mut dy) = (x - s.0, y - s.1);
                // Shift on a shape: a square, a circle or a regular polygon, whichever way the
                // pointer went (the dominant side wins), then snapped along it.
                let ratio = draw.kind.regular_ratio();
                let regular = mods.shift && draw.kind.is_shape();
                if regular {
                    let w = dx.abs().max(dy.abs() * ratio);
                    (dx, dy) = (w.copysign(dx), (w / ratio).copysign(dy));
                }
                // The handle on the side the pointer went to; a text box only has width.
                let handle = match (draw.kind, dx >= 0.0, dy >= 0.0) {
                    (DrawKind::Text, true, _) => Handle::E,
                    (DrawKind::Text, false, _) => Handle::W,
                    (_, true, true) => Handle::SE,
                    (_, false, true) => Handle::SW,
                    (_, true, false) => Handle::NE,
                    (_, false, false) => Handle::NW,
                };
                let point = PxRect {
                    x0: s.0,
                    y0: s.1,
                    x1: s.0,
                    y1: s.1,
                };
                let (mut r, guides) = draw.world.snap_resize(
                    &point,
                    handle,
                    (dx, dy),
                    ResizeMods {
                        keep_ratio: false,
                        from_center: mods.alt,
                    },
                    reach,
                    toggles,
                    MIN_SIZE_PX,
                );
                if regular {
                    r = regularise(r, s, (dx, dy), ratio, mods.alt);
                }
                (r, guides, Vec::new())
            }
        };
        draw.guides = guides;
        draw.marks = marks;
        if draw.rect != Some(rect) {
            let unit_y = RhythmPx::new(&base, &shown).unit_y;
            let constraints = Constraints::default();
            let frame = frame_for(draw.kind, rect, constraints, &primary, &shown, unit_y);
            let object = make_object(
                draw.kind,
                frame,
                constraints,
                &self.names,
                draw.object.as_ref().map_or_else(Uuid::new_v4, |o| o.id),
                draw.line_dir,
            );
            let mut preview = base;
            let at = preview.layers[draw.layer].objects.len();
            let _ = preview.apply(Command::Insert {
                layer: draw.layer,
                index: at,
                object: object.clone(),
            });
            draw.rect = Some(rect);
            draw.object = Some(object);
            self.show_preview(preview);
        }
        self.draw = Some(draw);
    }

    /// Lets go: the object is added as one undo step and selected, and the tool goes back to
    /// Select unless it is kept. A drag shorter than the snap reach, or one that came to
    /// nothing, drops the object at its default size, as a click does. Returns the new object's
    /// id.
    pub fn draw_end(&mut self) -> Option<Uuid> {
        let draw = self.draw.take()?;
        self.hide_preview();
        let degenerate = draw.rect.is_none_or(|r| match draw.kind {
            DrawKind::Line => r.w() < MIN_SIZE_PX && r.h() < MIN_SIZE_PX,
            DrawKind::Text => r.w() < MIN_SIZE_PX,
            DrawKind::Pattern | DrawKind::Rect | DrawKind::Ellipse | DrawKind::Polygon => {
                r.w() < MIN_SIZE_PX || r.h() < MIN_SIZE_PX
            }
        });
        let object = if draw.moved && !degenerate {
            draw.object.clone()?
        } else {
            self.default_object(&draw)?
        };
        self.insert_drawn(draw.layer, object, draw.kind)
    }

    /// The press and release at (x, y) device pixels with no drag between: the object at its
    /// default size (Defaulted "Per kind"). Returns the new object's id.
    pub fn place_click(&mut self, x: f64, y: f64, mods: DragMods, reach: f64) -> Option<Uuid> {
        if !self.draw_begin(x, y, mods, reach) {
            return None;
        }
        self.draw_end()
    }

    /// Escape: drops the drawing as if it had not begun. The tool stays.
    pub fn draw_cancel(&mut self) {
        if let Some(draw) = self.draw.take() {
            self.selection = draw.selection_before;
            self.hide_preview();
        }
    }

    /// A line or text box 10 rhythm units long from the start corner, kept on the page; a
    /// pattern region filling the page margins, which keeps its distance from every edge on
    /// every device (so it fills the margins on the other device too).
    fn default_object(&mut self, draw: &Draw) -> Option<Object> {
        let (primary, shown) = self.profiles()?;
        let doc = self.doc()?;
        let r = RhythmPx::new(doc, &shown);
        let w = f64::from(shown.width_px);
        let (s, unit_y) = (draw.start, r.unit_y);
        if draw.kind == DrawKind::Pattern {
            // Exactly the margins, in mm as the document holds them (not rounded to pixels).
            let m = doc.page.margins_mm;
            let (pw, ph) = primary.page_mm();
            let frame = Frame::new(m.left, m.top, pw - m.left - m.right, ph - m.top - m.bottom);
            let constraints = Constraints {
                h: HConstraint::LeftRight,
                v: VConstraint::TopBottom,
            };
            return Some(make_object(
                draw.kind,
                frame,
                constraints,
                &self.names,
                Uuid::new_v4(),
                LineDir::Down,
            ));
        }
        let (rect, constraints) = match draw.kind {
            DrawKind::Rect | DrawKind::Ellipse | DrawKind::Polygon => {
                // 10 rhythm units across, as tall as the shape is regular (a square, a circle;
                // a polygon in its own proportions), from the click point, slid to fit the page.
                let (w_px, h_px) = {
                    let side = DEFAULT_UNITS * r.unit_x;
                    (side, side / draw.kind.regular_ratio())
                };
                let h = f64::from(shown.height_px);
                let x0 = s.0.min((w - w_px).max(0.0));
                let y0 = s.1.min((h - h_px).max(0.0));
                (
                    PxRect {
                        x0,
                        y0,
                        x1: (x0 + w_px).min(w),
                        y1: (y0 + h_px).min(h),
                    },
                    Constraints::default(),
                )
            }
            DrawKind::Line | DrawKind::Text => {
                let len = DEFAULT_UNITS * r.unit_x;
                // Slide left to fit; a page too narrow for it takes what there is.
                let x0 = s.0.min((w - len).max(0.0));
                let x1 = (x0 + len).min(w);
                (
                    PxRect {
                        x0,
                        y0: s.1,
                        x1,
                        y1: s.1,
                    },
                    Constraints::default(),
                )
            }
            DrawKind::Pattern => unreachable!("returned above"),
        };
        let frame = frame_for(draw.kind, rect, constraints, &primary, &shown, unit_y);
        Some(make_object(
            draw.kind,
            frame,
            constraints,
            &self.names,
            Uuid::new_v4(),
            LineDir::Down,
        ))
    }

    fn insert_drawn(&mut self, layer: usize, object: Object, kind: DrawKind) -> Option<Uuid> {
        let id = object.id;
        let index = self.doc()?.layers.get(layer)?.objects.len();
        self.edit(
            Command::Insert {
                layer,
                index,
                object,
            },
            None,
        )
        .ok()?;
        self.selection = vec![id];
        if kind == DrawKind::Text {
            // The window opens the Text tab with its content selected, so typing replaces it.
            self.focus_new_text = true;
        }
        if !self.tool_kept {
            self.tool = Tool::Select;
        }
        Some(id)
    }
}
