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

//! Dragging on the canvas, with no UI in it (rung 4a, docs/bbq/2026-11-10-rung-4-drag-and-snap.md):
//! drag to move, resize handles, a line's two end handles, marquee select and Alt-drag
//! duplicate. The window feeds in the pointer (in device pixels of the profile on screen) and
//! reads back what to draw; everything here is tested headless.
//!
//! A drag is one undo step (Defaulted "One drag, one undo step"). While the pointer is down the
//! open document is not touched: the page shows a *preview* copy with the new frames
//! ([`Open::preview`]), and the real change is one command, made by [`Viewer::drag_end`]. Escape
//! ([`Viewer::drag_cancel`]) drops the preview.
//!
//! Locked objects, and objects in locked layers, do not drag. Frames are stored in the primary
//! profile's mm. On a secondary device a drag writes the device's override for the object (rung 8
//! bbq, Q3 and Q4): its place, and its size if that changed; a group gets one override, its box,
//! and what is in it follows. The stored design is not touched. Snapping is done on that device's
//! pixels (Q10).

use crate::viewer::Viewer;
use quadrule_layout::rotate;
use quadrule_layout::snap::{
    Guide, Handle, Mark, Other, PxRect, ResizeMods, SnapToggles, SnapWorld,
};
use quadrule_layout::tree::Ctx;
use quadrule_layout::units::Axis;
use quadrule_layout::{Placed, Shape, unresolve_frame};
use quadrule_model::edit::{
    Command, EditError, GroupMap, Placement, duplicate, is_empty, set_placements,
};
use quadrule_model::geom::map_frame;
use quadrule_model::{Constraints, Document, Frame, LineDir, ObjectKind, Uuid, normalize_deg};
use quadrule_profiles::Profile;

/// How strongly a line of the rhythm overlay is drawn (rung 9 bbq, Q6: major lines stronger).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayLine {
    /// A subdivision of a unit.
    Sub,
    /// A full unit.
    Unit,
    /// Every Nth unit, when the rhythm has "Major every N".
    Major,
}

/// The rhythm overlay's lines on the device shown, in device pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct RhythmOverlay {
    /// Vertical lines: x and strength.
    pub xs: Vec<(f64, OverlayLine)>,
    /// Horizontal lines: y and strength (none under an isometric rhythm).
    pub ys: Vec<(f64, OverlayLine)>,
    /// Slanted lines of an isometric rhythm: start, end and strength.
    #[allow(clippy::type_complexity)]
    pub slanted: Vec<((f64, f64), (f64, f64), OverlayLine)>,
}

/// A snap catches within this many screen points (rung 4 bbq, Defaulted "Snap distance"); the
/// window turns it into device pixels with the zoom, so it does not grow when zoomed in.
pub const SNAP_REACH_POINTS: f64 = 6.0;

/// A handle is picked within this many screen points of its centre.
pub const HANDLE_SLACK_POINTS: f64 = 6.0;

/// The fewest device pixels a box can be resized down to.
pub const MIN_SIZE_PX: f64 = 1.0;

/// A frame change smaller than this many mm is float noise from converting there and back.
const NOISE_MM: f64 = 1e-7;

/// How far the rotate handle stands from the top edge, in screen points (rung 6b, Q5).
pub const ROTATE_STALK_POINTS: f64 = 24.0;

/// The rotate handle catches at 0, 90, 180 and 270 degrees within this many degrees (rung 6b, Q5).
pub const ROTATE_CATCH_DEG: f64 = 4.0;

/// With Shift the angle is held to steps of this many degrees (rung 6b, Q5).
pub const ROTATE_STEP_DEG: f64 = 15.0;

/// A handle on the selection: one of the eight on its box, or one end of a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleHit {
    /// A handle of the selection's box.
    Box(Handle),
    /// A line's start: its left end (the top-left of its frame for a line running down).
    LineStart,
    /// A line's end: its right end (the bottom-right of its frame for a line running down).
    LineEnd,
    /// The round handle on a stalk above the middle of the top edge: turns the object (rung 6b).
    Rotate,
}

/// The rotate handle of the selected object: where its stalk starts (the middle of the top
/// edge) and where the handle sits, in device pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotateHandle {
    /// The middle of the top edge.
    pub base: (f64, f64),
    /// The round handle's centre.
    pub tip: (f64, f64),
}

/// What a drag does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragKind {
    /// Moves the selection.
    Move,
    /// Pulls a handle.
    Handle(HandleHit),
    /// Draws a box that selects what it touches.
    Marquee,
}

/// The keys held during a drag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DragMods {
    /// Shift: add to the selection (marquee, press), keep proportions (resize).
    pub shift: bool,
    /// Alt/Option: duplicate (at the press, for a move), resize from the centre.
    pub alt: bool,
    /// Ctrl/Cmd: snapping is off for this drag (napkin §6.2).
    pub snapping_off: bool,
}

/// One object being dragged.
#[derive(Clone, Debug)]
struct Item {
    /// The object whose frame changes (the copy, when duplicating).
    id: Uuid,
    constraints: Constraints,
    /// What it sits in: the page it resolves against and the turns of the groups round it (a group
    /// is see-through to constraints, rung 7a look change).
    ctx: Ctx,
    /// Where it falls in the unturned space it is in on the device shown, in mm, as the drag began
    /// (for a group: the box round its children).
    local: Frame,
    /// Its own angle, apart from the groups' (what is stored).
    own_rotation: f64,
    /// The box its constraints give on the device shown, in mm (for a group, the box round its
    /// children): what an override of it is measured against.
    base: Frame,
    /// Where it is before the overrides of the groups round it, as the drag began, in mm.
    unfit: Frame,
    /// Its frame on the device shown, in pixels, as the drag began, where it falls on the page.
    resolved: PxRect,
    /// A text box: its height is its text's, so only its position and width change.
    text: bool,
    /// A group: resizing it stops at zero size instead of flipping it (rung 7a).
    group: bool,
    /// Its angle on the page in degrees as the drag began, the groups' turns included (always 0
    /// for text and patterns).
    rotation: f64,
    /// A line's diagonal as the drag began.
    dir: Option<LineDir>,
}

/// A drag in progress.
#[derive(Clone, Debug)]
pub struct Drag {
    /// What it does.
    pub kind: DragKind,
    origin: (f64, f64),
    items: Vec<Item>,
    /// The box round the dragged objects as the drag began (their geometry, no stroke).
    start_box: PxRect,
    /// Alt-drag: the command that makes the copies, and the document with them in.
    duplicate: Option<(Command, Document)>,
    /// The selection before the drag (put back by Escape).
    selection_before: Vec<Uuid>,
    /// Marquee: what stays selected under the box.
    base: Vec<Uuid>,
    world: SnapWorld,
    /// What caught on the last move.
    pub guides: Vec<Guide>,
    /// Points and equal gaps to draw for the last move (rung 4b).
    pub marks: Vec<Mark>,
    /// Marquee: the box being drawn, in device pixels.
    pub marquee: Option<PxRect>,
    /// Where the drag would leave the objects (primary mm), as of the last move.
    places: Vec<(Uuid, Placement)>,
    /// One selected line's two ends (start, end) on the device shown, as the drag began.
    ends: Option<((f64, f64), (f64, f64))>,
    /// A rotation: the angle (degrees) from the object's centre to the pointer at the press.
    grab_angle: f64,
}

fn snap_rect(s: &Shape) -> PxRect {
    match *s {
        Shape::Segment { x0, y0, x1, y1, .. } => PxRect::from_corners(x0, y0, x1, y1),
        Shape::Box { x0, y0, x1, y1 } => PxRect { x0, y0, x1, y1 },
        // A turned shape snaps by the box round its turned corners (rung 6b).
        Shape::Rotated { .. } => {
            let (x0, y0, x1, y1) = s.bounds();
            PxRect { x0, y0, x1, y1 }
        }
    }
}

/// The objects on the page other than `skip`, for point snap, smart guides and equal spacing.
pub(crate) fn others(
    doc: &Document,
    placed: &[quadrule_layout::Placed],
    skip: &[Uuid],
) -> Vec<Other> {
    others_at(doc, placed, skip, &[])
}

/// [`others`] with the groups `entered`: a group is one object to snap to, and inside an entered
/// group its children count too, as do the objects outside it (rung 7a bbq, "Picking, dragging,
/// snapping"). What is being dragged, and what is inside it, is left out, and so is an entered
/// group (the box its own children are in).
pub(crate) fn others_at(
    doc: &Document,
    placed: &[quadrule_layout::Placed],
    skip: &[Uuid],
    entered: &[Uuid],
) -> Vec<Other> {
    placed
        .iter()
        .filter(|p| !skip.contains(&p.id) && !entered.contains(&p.id))
        .filter(|p| p.node.parent.is_none_or(|g| entered.contains(&g)))
        .filter(|p| !doc.ancestors(p.id).iter().any(|a| skip.contains(a)))
        .map(|p| {
            let line = matches!(doc.object(p.id).map(|o| &o.kind), Some(ObjectKind::Line(_)));
            Other {
                rect: snap_rect(&p.shape),
                line,
                // A line's anchors are its real ends, whichever way it points.
                ends: match p.shape {
                    Shape::Segment { x0, y0, x1, y1, .. } if line => Some(((x0, y0), (x1, y1))),
                    _ => None,
                },
            }
        })
        .collect()
}

/// Where an object falls on `shown`, in pixels: its unturned box on the page, the turns of the
/// groups round it already in the place of its centre.
fn placed_px(p: &Placed, shown: &Profile) -> PxRect {
    let f = p.node.frame;
    let px = |mm: f64| shown.mm_to_px(mm);
    PxRect {
        x0: px(f.x_mm),
        y0: px(f.y_mm),
        x1: px(f.x_mm + f.w_mm),
        y1: px(f.y_mm + f.h_mm),
    }
}

impl Item {
    /// The stored frame and own angle that put the object on the page at `r` (pixels, unturned
    /// box) turned `total_deg` in all: back through the groups' turns and against the page. On the
    /// page itself, with no groups, this is [`frame_from_px`]. Also the box it then falls in, in
    /// the unturned space it is in (mm on the device shown). A group has no constraints, so its
    /// stored frame is only a stand-in: the box round its children, which follow
    /// ([`Placement::map`]), is what it is refitted to.
    fn stored(
        &self,
        r: PxRect,
        total_deg: f64,
        shown: &Profile,
        on_device: bool,
    ) -> (Frame, f64, Frame) {
        let per_mm = shown.mm_to_px(1.0);
        let placed = Frame::new(
            r.x0 / per_mm,
            r.y0 / per_mm,
            (r.x1 - r.x0) / per_mm,
            (r.y1 - r.y0) / per_mm,
        );
        let (local, rot) = self.ctx.unplace(placed, total_deg);
        let rot = if total_deg == self.rotation {
            self.own_rotation
        } else {
            normalize_deg(rot)
        };
        if on_device {
            // A secondary device: where the object is there, before the groups' overrides.
            return (self.ctx.unfit(local, rot, self.text), rot, local);
        }
        let c = if self.group {
            Constraints::default()
        } else {
            self.constraints
        };
        (self.ctx.holder.unresolve(local, c), rot, local)
    }
}

/// The frame on the primary page that lands on `r` (pixels) on `shown`.
pub(crate) fn frame_from_px(
    r: PxRect,
    c: Constraints,
    primary: &Profile,
    shown: &Profile,
) -> Frame {
    let per_mm = shown.mm_to_px(1.0);
    let resolved = Frame::new(
        r.x0 / per_mm,
        r.y0 / per_mm,
        (r.x1 - r.x0) / per_mm,
        (r.y1 - r.y0) / per_mm,
    );
    unresolve_frame(resolved, c, primary.page_mm(), shown.page_mm())
}

/// Where `item` goes when the selection's box `old` becomes `new`: moved, and scaled about the box
/// ([`map_frame`], also what the contents of a group follow).
/// A text box keeps its height; a box with no width (or height) is moved along that axis only.
fn map_rect(item: &Item, old: &PxRect, new: &PxRect) -> PxRect {
    let frame = |r: &PxRect| Frame::new(r.x0, r.y0, r.w(), r.h());
    let f = map_frame(
        frame(&item.resolved),
        item.rotation,
        item.text,
        &frame(old),
        &frame(new),
    );
    PxRect {
        x0: f.x_mm,
        y0: f.y_mm,
        x1: f.x_mm + f.w_mm,
        y1: f.y_mm + f.h_mm,
    }
}

/// `r`, a resize of `start` by handle `h`, made not to cross the side opposite the handle: it
/// stops at [`MIN_SIZE_PX`] across instead of mirroring.
fn not_past(start: &PxRect, h: Handle, r: PxRect) -> PxRect {
    let mut out = r;
    match h.x_side() {
        1 => {
            let e = if r.x0 < start.x0 - 1e-9 { r.x0 } else { r.x1 };
            out.x0 = start.x0;
            out.x1 = e.max(start.x0 + MIN_SIZE_PX);
        }
        -1 => {
            let e = if r.x1 > start.x1 + 1e-9 { r.x1 } else { r.x0 };
            out.x1 = start.x1;
            out.x0 = e.min(start.x1 - MIN_SIZE_PX);
        }
        _ => {}
    }
    match h.y_side() {
        1 => {
            let e = if r.y0 < start.y0 - 1e-9 { r.y0 } else { r.y1 };
            out.y0 = start.y0;
            out.y1 = e.max(start.y0 + MIN_SIZE_PX);
        }
        -1 => {
            let e = if r.y1 > start.y1 + 1e-9 { r.y1 } else { r.y0 };
            out.y1 = start.y1;
            out.y0 = e.min(start.y1 - MIN_SIZE_PX);
        }
        _ => {}
    }
    out
}

fn near(a: &Frame, b: &Frame) -> bool {
    (a.x_mm - b.x_mm).abs() < NOISE_MM
        && (a.y_mm - b.y_mm).abs() < NOISE_MM
        && (a.w_mm - b.w_mm).abs() < NOISE_MM
        && (a.h_mm - b.h_mm).abs() < NOISE_MM
}

/// An angle after a turn: normalised to 0 up to 360; with `step` (Shift) held to multiples of
/// [`ROTATE_STEP_DEG`]; otherwise, with `catch` on, pulled onto 0, 90, 180 or 270 degrees when
/// within [`ROTATE_CATCH_DEG`] of one.
pub fn turned_angle(deg: f64, step: bool, catch: bool) -> f64 {
    let d = quadrule_model::normalize_deg(deg);
    if step {
        return quadrule_model::normalize_deg((d / ROTATE_STEP_DEG).round() * ROTATE_STEP_DEG);
    }
    if catch {
        let q = (d / 90.0).round() * 90.0;
        if (d - q).abs() <= ROTATE_CATCH_DEG {
            return quadrule_model::normalize_deg(q);
        }
    }
    d
}

/// Resizes one turned object by a handle (rung 6b): the pointer is worked out along the object's
/// own axes, the side opposite the handle stays where it is on the page (the centre, with Alt),
/// and the object keeps its angle. A side never goes under [`MIN_SIZE_PX`] (no flipping); Shift
/// keeps the proportions of a corner. The dragged point snaps as a point does. Returns the new
/// unturned box in pixels.
fn resize_turned(
    world: &SnapWorld,
    item: &Item,
    h: Handle,
    pointer: (f64, f64),
    mods: DragMods,
    reach: f64,
    toggles: SnapToggles,
) -> (PxRect, Vec<Guide>, Vec<Mark>) {
    let snapped = world.snap_point(pointer, reach, toggles);
    let r = item.resolved;
    let c = rotate::centre((r.x0, r.y0, r.x1, r.y1));
    let (w, hgt) = (r.w(), r.h());
    // The pointer on the object's own axes, from its old centre.
    let p = rotate::rotate_point(snapped.at, c, -item.rotation);
    let (px, py) = (p.0 - c.0, p.1 - c.1);
    let (xs, ys) = (f64::from(h.x_side()), f64::from(h.y_side()));
    // Each moving axis: the new length, and where its centre lands (on the old axes, from the old
    // centre). A fixed side is `-side * half`; from the centre both sides move.
    let axis = |side: f64, half: f64, pos: f64| -> (f64, f64) {
        if side == 0.0 {
            return (half * 2.0, 0.0);
        }
        if mods.alt {
            let len = (2.0 * side * pos).max(MIN_SIZE_PX);
            (len, 0.0)
        } else {
            let anchor = -side * half;
            let len = (side * (pos - anchor)).max(MIN_SIZE_PX);
            (len, anchor + side * len / 2.0)
        }
    };
    let (mut nw, mut cxo) = axis(xs, w / 2.0, px);
    let (mut nh, mut cyo) = axis(ys, hgt / 2.0, py);
    if mods.shift && h.is_corner() && w > 0.0 && hgt > 0.0 {
        let k = (nw / w).max(nh / hgt);
        (nw, nh) = (w * k, hgt * k);
        if !mods.alt {
            cxo = -xs * w / 2.0 + xs * nw / 2.0;
            cyo = -ys * hgt / 2.0 + ys * nh / 2.0;
        }
    }
    let nc = rotate::rotate_point((c.0 + cxo, c.1 + cyo), c, item.rotation);
    (
        PxRect {
            x0: nc.0 - nw / 2.0,
            y0: nc.1 - nh / 2.0,
            x1: nc.0 + nw / 2.0,
            y1: nc.1 + nh / 2.0,
        },
        snapped.guides,
        snapped.marks,
    )
}

impl Viewer {
    /// Whether a drag, or a drawing with a tool (rung 5), is in progress.
    pub fn dragging(&self) -> bool {
        self.drag.is_some() || self.draw.is_some()
    }

    /// What caught on the last move of the drag, to draw.
    pub fn drag_guides(&self) -> &[Guide] {
        match (&self.draw, &self.drag) {
            (Some(d), _) => &d.guides,
            (None, Some(d)) => &d.guides,
            (None, None) => &[],
        }
    }

    /// The points and equal gaps to draw for the last move of the drag.
    pub fn drag_marks(&self) -> &[Mark] {
        match (&self.draw, &self.drag) {
            (Some(d), _) => &d.marks,
            (None, Some(d)) => &d.marks,
            (None, None) => &[],
        }
    }

    /// Lets go of whatever the pointer is doing: ends the drag or the drawing.
    pub fn gesture_end(&mut self) {
        if self.draw.is_some() {
            let _ = self.draw_end();
        } else {
            let _ = self.drag_end();
        }
    }

    /// The marquee being drawn, in device pixels.
    pub fn drag_marquee(&self) -> Option<PxRect> {
        self.drag.as_ref().and_then(|d| d.marquee)
    }

    /// What a press at (x, y) picks: as a click does, or with `deep` (Ctrl) the innermost object,
    /// entering the groups round it (rung 7a bbq, Q4).
    pub(crate) fn pick_for_press(
        &mut self,
        x: f64,
        y: f64,
        slack: f64,
        deep: bool,
    ) -> Option<Uuid> {
        if !deep {
            return self.pick(x, y, slack);
        }
        let leaf = self.pick_leaf(x, y, slack)?;
        // Something already selected that holds or is the object under the pointer is what is
        // dragged: Ctrl then only turns snapping off.
        let doc = self.doc()?;
        let mut chain = doc.ancestors(leaf);
        chain.push(leaf);
        if let Some(sel) = chain.iter().find(|c| self.selection.contains(c)) {
            return Some(*sel);
        }
        self.entered = doc.ancestors(leaf);
        Some(leaf)
    }

    /// The primary profile, the profile on screen, and the shapes on the page.
    pub(crate) fn geometry(&mut self) -> Option<(Profile, Profile, Vec<quadrule_layout::Placed>)> {
        let (primary, shown) = self.profiles()?;
        let placed = self.page()?.placed.clone();
        Some((primary, shown, placed))
    }

    /// The box round the editable selected objects' geometry (no stroke), on the device shown.
    pub fn selection_box(&mut self) -> Option<PxRect> {
        let ids = self.editable_selection();
        let (_, _, placed) = self.geometry()?;
        placed
            .iter()
            .filter(|p| ids.contains(&p.id))
            .map(|p| snap_rect(&p.shape))
            .reduce(|a, b| a.union(&b))
    }

    /// The handles to draw on the selection, with their places in device pixels: the eight of the
    /// box (just left and right for text, which grows down by itself), or a line's two ends when
    /// one line is selected. A single turned shape has its eight on its turned box (rung 6b). None
    /// when nothing editable is selected. The rotate handle is apart: [`Viewer::rotate_handle`].
    pub fn handles(&mut self) -> Vec<(HandleHit, f64, f64)> {
        let ids = self.editable_selection();
        if ids.is_empty() {
            return Vec::new();
        }
        let Some((_, shown, placed)) = self.geometry() else {
            return Vec::new();
        };
        // What is drawn: during a drag, the preview, so the handles turn and move with the object.
        let Some(doc) = self.shown_doc() else {
            return Vec::new();
        };
        if let [id] = ids[..]
            && matches!(doc.object(id).map(|o| &o.kind), Some(ObjectKind::Line(_)))
            && let Some(Shape::Segment { x0, y0, x1, y1, .. }) =
                placed.iter().find(|p| p.id == id).map(|p| p.shape)
        {
            return vec![(HandleHit::LineStart, x0, y0), (HandleHit::LineEnd, x1, y1)];
        }
        // One turned shape, or a group that is turned or inside a turned group: the handles of its
        // own box, turned with it.
        if let [id] = ids[..]
            && let Some(p) = placed.iter().find(|p| p.id == id)
            && p.node.angle != 0.0
        {
            let r = placed_px(p, &shown);
            let c = rotate::centre((r.x0, r.y0, r.x1, r.y1));
            return Handle::ALL
                .iter()
                .map(|&h| {
                    let (x, y) = rotate::rotate_point(h.at(&r), c, p.node.angle);
                    (HandleHit::Box(h), x, y)
                })
                .collect();
        }
        let all_text = ids
            .iter()
            .all(|id| matches!(doc.object(*id).map(|o| &o.kind), Some(ObjectKind::Text(_))));
        let Some(bx) = placed
            .iter()
            .filter(|p| ids.contains(&p.id))
            .map(|p| snap_rect(&p.shape))
            .reduce(|a, b| a.union(&b))
        else {
            return Vec::new();
        };
        let list: &[Handle] = if all_text {
            &Handle::SIDES
        } else {
            &Handle::ALL
        };
        list.iter()
            .map(|&h| {
                let (x, y) = h.at(&bx);
                (HandleHit::Box(h), x, y)
            })
            .collect()
    }

    /// The document as it is drawn: the preview while a drag or a drawing runs, else the open one.
    pub(crate) fn shown_doc(&self) -> Option<&Document> {
        self.open
            .as_ref()
            .map(|o| o.preview.as_ref().unwrap_or(&o.doc))
    }

    /// The angle the selection's box is drawn at: that of the one selected object (as it is drawn
    /// now, so during a turn it follows the pointer), 0 for none or for several (rung 6b).
    pub fn selection_angle(&self) -> f64 {
        let ids = self.editable_selection();
        let page = self
            .open
            .as_ref()
            .and_then(|o| o.pages.get(&self.profile_id));
        match (&ids[..], page) {
            ([id], Some(page)) => page
                .placed
                .iter()
                .find(|p| p.id == *id)
                .map_or(0.0, |p| p.node.angle),
            _ => 0.0,
        }
    }

    /// The square each box or end handle is painted as: `half` pixels from its centre to a side,
    /// turned by [`Viewer::selection_angle`] so the box and its handles read as turned together.
    /// Corners in device pixels, clockwise from the top-left.
    pub fn handle_squares(&mut self, half: f64) -> Vec<(HandleHit, [(f64, f64); 4])> {
        let angle = self.selection_angle();
        self.handles()
            .into_iter()
            .map(|(h, x, y)| {
                let c = (x, y);
                let corners = [(-half, -half), (half, -half), (half, half), (-half, half)]
                    .map(|(dx, dy)| rotate::rotate_point((x + dx, y + dy), c, angle));
                (h, corners)
            })
            .collect()
    }

    /// The outline of a turned line: its frame turned with it, grown by `pad` pixels, so a turned
    /// line is outlined like a turned shape. None for a line that is not turned, or not a line.
    pub fn turned_line_outline(&mut self, id: Uuid, pad: f64) -> Option<[(f64, f64); 4]> {
        let (_, shown, placed) = self.geometry()?;
        let o = self.shown_doc()?.object(id)?;
        let p = placed.iter().find(|p| p.id == id)?;
        if !matches!(o.kind, ObjectKind::Line(_)) || p.node.angle == 0.0 {
            return None;
        }
        let r = placed_px(p, &shown);
        Some(rotate::rotated_corners(
            (r.x0 - pad, r.y0 - pad, r.x1 + pad, r.y1 + pad),
            p.node.angle,
        ))
    }

    /// The rotate handle (Q5): a round handle on a short stalk above the middle of the top edge
    /// of the one selected line or shape, turned with it. None for anything else (a text box, a
    /// pattern region, several objects) or when it is locked.
    pub fn rotate_handle(&mut self) -> Option<RotateHandle> {
        let ids = self.editable_selection();
        let [id] = ids[..] else {
            return None;
        };
        let (_, shown, placed) = self.geometry()?;
        let o = self.shown_doc()?.object(id)?;
        if !o.kind.can_rotate() {
            return None;
        }
        let p = placed.iter().find(|p| p.id == id)?;
        let r = placed_px(p, &shown);
        let c = rotate::centre((r.x0, r.y0, r.x1, r.y1));
        let rot = p.node.angle;
        let base = rotate::rotate_point((c.0, r.y0), c, rot);
        let tip = rotate::rotate_point((c.0, r.y0 - self.rotate_stalk_px), c, rot);
        Some(RotateHandle { base, tip })
    }

    /// The handle under (x, y) device pixels, within `slack` pixels; the nearest if several. The
    /// rotate handle wins where it overlaps another.
    pub fn handle_at(&mut self, x: f64, y: f64, slack: f64) -> Option<HandleHit> {
        if let Some(r) = self.rotate_handle()
            && (r.tip.0 - x).hypot(r.tip.1 - y) <= slack * 1.3
        {
            return Some(HandleHit::Rotate);
        }
        self.handles()
            .into_iter()
            .map(|(h, hx, hy)| {
                (
                    h,
                    (hx - x).hypot(hy - y),
                    (hx - x).abs().max((hy - y).abs()),
                )
            })
            .filter(|&(_, _, cheb)| cheb <= slack)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(h, ..)| h)
    }

    /// The press at (x, y) device pixels turned into a drag: on a handle it pulls the handle;
    /// on an object it moves the selection (selecting the object first when it is not in it;
    /// with Alt, duplicating); on nothing it draws a marquee. Returns false when nothing
    /// started (nothing open, a drag already running, or only locked objects to move).
    /// `slack` is for picking, `handle_slack` for handles, both in device pixels.
    pub fn drag_begin(
        &mut self,
        x: f64,
        y: f64,
        mods: DragMods,
        slack: f64,
        handle_slack: f64,
    ) -> bool {
        if self.open.is_none() || self.drag.is_some() {
            return false;
        }
        let Some((_, shown, placed)) = self.geometry() else {
            return false;
        };
        let before_press = self.selection.clone();

        let hit = self.handle_at(x, y, handle_slack);
        let kind = if let Some(h) = hit {
            DragKind::Handle(h)
        } else if let Some(id) = self.pick_for_press(x, y, slack, mods.snapping_off) {
            if !self.selection.contains(&id) {
                if mods.shift {
                    self.selection.push(id);
                } else {
                    self.selection = vec![id];
                }
            }
            DragKind::Move
        } else {
            DragKind::Marquee
        };

        // Escape puts the selection back as it was before a marquee; for a move or a handle the
        // press has already selected the object, and that stays.
        let selection_before = if kind == DragKind::Marquee {
            before_press
        } else {
            self.selection.clone()
        };
        let Some(doc) = self.doc().cloned() else {
            return false;
        };
        let zones = crate::viewer::active_zone_rects(&shown);
        let world = SnapWorld::new(&doc, &shown, &zones);
        let base = if kind == DragKind::Marquee {
            if mods.shift {
                self.selection.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        let mut drag = Drag {
            kind,
            origin: (x, y),
            items: Vec::new(),
            start_box: PxRect {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
            duplicate: None,
            selection_before,
            base,
            world,
            guides: Vec::new(),
            marks: Vec::new(),
            marquee: None,
            places: Vec::new(),
            ends: None,
            grab_angle: 0.0,
        };
        if kind == DragKind::Marquee {
            self.drag = Some(drag);
            return true;
        }

        // The objects the drag changes: the editable part of the selection.
        let ids = self.editable_selection();
        if ids.is_empty() {
            self.notice = Some(crate::viewer::Notice {
                message: quadrule_i18n::Message::new("notice-locked"),
                error: true,
            });
            return false;
        }
        let mut base_doc = doc.clone();
        let mut targets = ids.clone();
        // What the drag can line up with: everything on the page it is not moving. An Alt-drag
        // leaves the originals where they are, so they count.
        let leave_out: &[Uuid] = if kind == DragKind::Move && mods.alt {
            &[]
        } else {
            &ids
        };
        drag.world = drag
            .world
            .with_others(others_at(&doc, &placed, leave_out, &self.entered));
        if kind == DragKind::Move && mods.alt {
            let (cmd, copies) = duplicate(&doc, &ids, Uuid::new_v4);
            if copies.len() == ids.len()
                && let Ok(_undo) = base_doc.apply(cmd.clone())
            {
                targets = copies;
                drag.duplicate = Some((cmd, base_doc.clone()));
            }
        }
        for (id, target) in ids.iter().zip(&targets) {
            let Some(o) = base_doc.object(*target) else {
                continue;
            };
            let node = placed.iter().find(|p| p.id == *id);
            let shape = node.map(|p| snap_rect(&p.shape));
            let (Some(node), Some(resolved)) = (node, node.map(|p| placed_px(p, &shown))) else {
                continue;
            };
            drag.start_box = match (drag.items.is_empty(), shape) {
                (true, Some(s)) => s,
                (false, Some(s)) => drag.start_box.union(&s),
                (_, None) => drag.start_box,
            };
            drag.items.push(Item {
                id: *target,
                constraints: o.constraints,
                ctx: node.node.ctx,
                local: node.node.local,
                own_rotation: o.rotation(),
                base: node.node.base,
                unfit: node.node.unfit,
                resolved,
                text: matches!(o.kind, ObjectKind::Text(_)),
                group: o.is_group(),
                rotation: node.node.angle,
                dir: match &o.kind {
                    ObjectKind::Line(l) => Some(l.direction),
                    _ => None,
                },
            });
        }
        if drag.items.is_empty() {
            return false;
        }
        drag.places = drag
            .items
            .iter()
            .filter_map(|i| base_doc.object(i.id).map(|o| (i.id, Placement::of(o))))
            .collect();
        if let [item] = &drag.items[..] {
            // One line's two ends (they may point any way), and the angle a rotation starts from.
            drag.ends = ids.first().and_then(|id| {
                placed
                    .iter()
                    .find(|p| p.id == *id)
                    .and_then(|p| match p.shape {
                        Shape::Segment { x0, y0, x1, y1, .. } => Some(((x0, y0), (x1, y1))),
                        _ => None,
                    })
            });
            let c = rotate::centre((
                item.resolved.x0,
                item.resolved.y0,
                item.resolved.x1,
                item.resolved.y1,
            ));
            drag.grab_angle = (y - c.1).atan2(x - c.0).to_degrees();
        }
        if drag.duplicate.is_some() {
            // The copies are what moves now, and what is selected while they do.
            self.selection = targets;
            self.show_preview(base_doc);
        }
        self.drag = Some(drag);
        true
    }

    /// The pointer is at (x, y) device pixels with `mods` held; `reach` is the snap distance in
    /// device pixels (the window passes [`SNAP_REACH_POINTS`] over the zoom).
    pub fn drag_update(&mut self, x: f64, y: f64, mods: DragMods, reach: f64) {
        let Some(mut drag) = self.drag.take() else {
            return;
        };
        let toggles = if mods.snapping_off {
            SnapToggles::OFF
        } else {
            self.snap
        };
        let delta = (x - drag.origin.0, y - drag.origin.1);
        if drag.kind == DragKind::Marquee {
            let r = PxRect::from_corners(drag.origin.0, drag.origin.1, x, y);
            drag.marquee = Some(r);
            let touched = self.marquee_hits(&r);
            let mut sel = drag.base.clone();
            for id in touched {
                if !sel.contains(&id) {
                    sel.push(id);
                }
            }
            self.selection = sel;
            self.drag = Some(drag);
            return;
        }
        let Some((primary, shown)) = self.profiles() else {
            self.drag = Some(drag);
            return;
        };
        // On a secondary device the drag writes overrides there (rung 8 bbq, Q3).
        let on_device = primary.id != shown.id;
        let start = drag.start_box;
        let mut marks = Vec::new();
        // Each object's angle and line direction after this move: unchanged unless it is turned,
        // or a line's end is pulled.
        let mut rot_dir: Vec<(f64, Option<LineDir>)> =
            drag.items.iter().map(|i| (i.rotation, i.dir)).collect();
        // One object's frame in pixels, when the drag decides it directly (a turn, a turned
        // object's resize, a line's end).
        let mut direct: Option<PxRect> = None;
        let single_turned = matches!(&drag.items[..], [i] if i.rotation != 0.0 && !i.text);
        let (new_box, guides) = match drag.kind {
            DragKind::Move => {
                let raw = start.moved(delta.0, delta.1);
                let s = drag.world.snap_move(&raw, reach, toggles);
                marks = s.marks;
                (start.moved(delta.0 + s.dx, delta.1 + s.dy), s.guides)
            }
            DragKind::Handle(HandleHit::Box(h)) if single_turned => {
                let (r, guides, m) =
                    resize_turned(&drag.world, &drag.items[0], h, (x, y), mods, reach, toggles);
                marks = m;
                direct = Some(r);
                (r, guides)
            }
            DragKind::Handle(HandleHit::Box(h)) => {
                let all_text = drag.items.iter().all(|i| i.text);
                let has_group = drag.items.iter().any(|i| i.group);
                let (r, guides) = drag.world.snap_resize(
                    &start,
                    h,
                    delta,
                    ResizeMods {
                        keep_ratio: mods.shift && !all_text,
                        from_center: mods.alt,
                    },
                    reach,
                    toggles,
                    MIN_SIZE_PX,
                );
                // A resize that holds a group stops at the least size instead of flipping the box
                // (its contents would not be mirrored; rung 4a debt, rung 7a).
                if has_group && !mods.alt {
                    (not_past(&start, h, r), guides)
                } else {
                    (r, guides)
                }
            }
            DragKind::Handle(HandleHit::Rotate) => {
                // The object turns about its centre by how far the pointer has gone round it;
                // Shift holds the angle to steps, and it catches at the quarter turns.
                let item = &drag.items[0];
                let r = item.resolved;
                let c = rotate::centre((r.x0, r.y0, r.x1, r.y1));
                let angle = (y - c.1).atan2(x - c.0).to_degrees();
                let deg = turned_angle(
                    item.rotation + (angle - drag.grab_angle),
                    mods.shift,
                    !mods.snapping_off,
                );
                rot_dir[0].0 = deg;
                direct = Some(r);
                (start, Vec::new())
            }
            DragKind::Handle(end) => {
                // One end of a line moves; the other stays. The ends may cross in any direction:
                // the frame is the box round them and the line keeps whichever diagonal they make.
                let (a, b) = drag
                    .ends
                    .unwrap_or(((start.x0, start.y0), (start.x1, start.y1)));
                let (moving, fixed) = if end == HandleHit::LineStart {
                    (a, b)
                } else {
                    (b, a)
                };
                let snapped =
                    drag.world
                        .snap_point((moving.0 + delta.0, moving.1 + delta.1), reach, toggles);
                marks = snapped.marks;
                let (dx, dy) = (snapped.at.0 - fixed.0, snapped.at.1 - fixed.1);
                let old = rot_dir[0].1;
                let dir = if dx * dy < 0.0 {
                    Some(LineDir::Up)
                } else if dx * dy > 0.0 {
                    Some(LineDir::Down)
                } else {
                    old
                };
                // A pulled end sets the line as it is, so a turn is folded into the frame.
                rot_dir[0] = (0.0, dir);
                let r = PxRect::from_corners(snapped.at.0, snapped.at.1, fixed.0, fixed.1);
                direct = Some(r);
                (r, snapped.guides)
            }
            DragKind::Marquee => unreachable!("handled above"),
        };
        let base_doc = drag
            .duplicate
            .as_ref()
            .map(|(_, d)| d.clone())
            .or_else(|| self.doc().cloned());
        let Some(base_doc) = base_doc else {
            self.drag = Some(drag);
            return;
        };
        let mut places = Vec::with_capacity(drag.items.len());
        for (n, item) in drag.items.iter().enumerate() {
            let px = match direct {
                Some(r) if drag.items.len() == 1 => r,
                _ => map_rect(item, &start, &new_box),
            };
            let (total, dir) = rot_dir[n];
            let (mut frame, rotation_deg, local) = item.stored(px, total, &shown, on_device);
            if on_device {
                if near(&item.unfit, &frame) {
                    frame = item.unfit;
                }
                places.push((
                    item.id,
                    Placement {
                        frame,
                        rotation_deg,
                        dir,
                        map: None,
                        device: None,
                    }
                    .on_device(&shown.id, item.base),
                ));
                continue;
            }
            if let Some(o) = base_doc.object(item.id)
                && near(&o.frame, &frame)
            {
                frame = o.frame;
            }
            // A group's contents follow its box: moved and stretched as a loose selection's are.
            let map = item.group.then_some(GroupMap {
                holder: Some(item.ctx.holder),
                old: item.local,
                new: local,
            });
            places.push((
                item.id,
                Placement {
                    frame,
                    rotation_deg,
                    dir,
                    map,
                    device: None,
                },
            ));
        }
        drag.guides = guides;
        drag.marks = marks;
        if places != drag.places {
            let mut preview = base_doc;
            for (id, place) in &places {
                if let Some(o) = preview.object_mut(*id) {
                    place.apply(o);
                }
            }
            drag.places = places;
            self.show_preview(preview);
        }
        self.drag = Some(drag);
    }

    /// Lets go: the objects get the frames the drag left them with, as one undo step. A drag
    /// that moved nothing makes no step (and, for an Alt-drag, no copies).
    pub fn drag_end(&mut self) -> Result<(), EditError> {
        let Some(drag) = self.drag.take() else {
            return Ok(());
        };
        if drag.kind == DragKind::Marquee {
            return Ok(());
        }
        self.hide_preview();
        let Some(doc) = self.doc().cloned() else {
            return Ok(());
        };
        let (cmd, copies) = match &drag.duplicate {
            Some((dup, with_copies)) => {
                let moved = set_placements(with_copies, &drag.places);
                if is_empty(&moved) {
                    // Nowhere to put the copies: leave things as they were.
                    self.selection = drag.selection_before;
                    self.bump();
                    return Ok(());
                }
                let copies: Vec<Uuid> = drag.items.iter().map(|i| i.id).collect();
                (Command::Batch(vec![dup.clone(), moved]), Some(copies))
            }
            None => (set_placements(&doc, &drag.places), None),
        };
        if is_empty(&cmd) {
            self.bump();
            return Ok(());
        }
        self.edit(cmd, None)?;
        if let Some(c) = copies {
            self.selection = c;
        }
        Ok(())
    }

    /// Escape: drops the drag, as if it had not begun.
    pub fn drag_cancel(&mut self) {
        if self.draw.is_some() {
            self.draw_cancel();
            return;
        }
        let Some(drag) = self.drag.take() else {
            return;
        };
        self.selection = drag.selection_before;
        self.hide_preview();
    }

    /// The rhythm to draw over the page for the device shown (rung 4 bbq Q9; rung 9 bbq, Q6 and
    /// Q7): the vertical lines' x positions and the horizontal lines' y positions, each with how
    /// strong it is (a subdivision, a unit or a major line), and under an isometric rhythm the
    /// slanted lines of its triangles with the vertical ones (and no horizontal ones). They are an overlay and never
    /// exported.
    pub fn rhythm_overlay(&mut self) -> Option<RhythmOverlay> {
        let (_, shown) = self.profiles()?;
        let doc = self.doc()?;
        let world = SnapWorld::new(doc, &shown, &[]);
        let r = quadrule_layout::RhythmPx::new(doc, &shown);
        let major = doc.rhythm.major_on();
        let (w, h) = world.page();
        let level = |pos: f64, origin: f64, unit: f64, full: bool| {
            if !full {
                return OverlayLine::Sub;
            }
            let k = ((pos - origin) / unit).round() as i64;
            if major.is_some_and(|n| k.rem_euclid(i64::from(n)) == 0) {
                OverlayLine::Major
            } else {
                OverlayLine::Unit
            }
        };
        let iso = doc.rhythm.is_isometric();
        // Under an isometric rhythm the triangles are drawn: the vertical lines one column apart
        // and the two slanted families; the half-side rows snapping also uses are not lines of the
        // grid and are not drawn.
        let xs = world
            .lines(Axis::X, 0.0, w)
            .into_iter()
            .map(|(x, full)| (x, level(x, r.origin_x, r.unit_x, full)))
            .collect();
        let ys = if iso {
            Vec::new()
        } else {
            world
                .lines(Axis::Y, 0.0, h)
                .into_iter()
                .map(|(y, full)| (y, level(y, r.origin_y, r.unit_y, full)))
                .collect()
        };
        let slanted = if iso {
            quadrule_layout::pattern::iso_segments(
                (r.origin_x, r.origin_y),
                r.unit_x,
                r.unit_y,
                (0.0, 0.0, w, h),
            )
            .into_iter()
            .map(|s| {
                let strength = if major.is_some_and(|n| s.index.rem_euclid(i64::from(n)) == 0) {
                    OverlayLine::Major
                } else {
                    OverlayLine::Unit
                };
                (s.from, s.to, strength)
            })
            .collect()
        } else {
            Vec::new()
        };
        Some(RhythmOverlay { xs, ys, slanted })
    }

    /// Objects a marquee box touches: visible, unlocked, in unlocked layers (what a click could
    /// pick), bottom first; a group once, where any of its children is touched.
    fn marquee_hits(&mut self, r: &PxRect) -> Vec<Uuid> {
        let Some((_, _, placed)) = self.geometry() else {
            return Vec::new();
        };
        // What it touches, leaf by leaf; a leaf in a group is stood for by that group, at the level
        // entered (rung 7a bbq, Q4), once.
        let leaves: Vec<Uuid> = placed
            .iter()
            .filter(|p| {
                if p.node.group || p.node.hidden || p.node.locked {
                    return false;
                }
                match p.shape {
                    Shape::Rotated {
                        cx,
                        cy,
                        hw,
                        hh,
                        deg,
                    } => rotate::rotated_touches(
                        (cx - hw, cy - hh, cx + hw, cy + hh),
                        deg,
                        0.0,
                        (r.x0, r.y0, r.x1, r.y1),
                    ),
                    Shape::Box { x0, y0, x1, y1 } => r.touches(&PxRect { x0, y0, x1, y1 }),
                    Shape::Segment {
                        x0,
                        y0,
                        x1,
                        y1,
                        width,
                    } => {
                        let h = width / 2.0;
                        let grown = PxRect {
                            x0: r.x0 - h,
                            y0: r.y0 - h,
                            x1: r.x1 + h,
                            y1: r.y1 + h,
                        };
                        grown.touches_segment(x0, y0, x1, y1)
                    }
                }
            })
            .map(|p| p.id)
            .collect();
        let entered = self.entered.clone();
        let mut out: Vec<Uuid> = Vec::new();
        for leaf in leaves {
            if let Some(t) = self.target_of(leaf, &entered)
                && !out.contains(&t)
            {
                out.push(t);
            }
        }
        out
    }

    /// Shows `doc` instead of the open document until the preview is hidden.
    pub(crate) fn show_preview(&mut self, doc: Document) {
        if let Some(o) = &mut self.open {
            o.preview = Some(doc);
        }
        self.bump();
    }

    pub(crate) fn hide_preview(&mut self) {
        if let Some(o) = &mut self.open {
            o.preview = None;
        }
        self.bump();
    }

    /// The page shown must be made again.
    fn bump(&mut self) {
        self.revision += 1;
        if let Some(o) = &mut self.open {
            o.pages.clear();
        }
    }
}
