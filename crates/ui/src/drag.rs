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
//! profile's mm, so on a secondary device the new frame is the inverse of the constraints
//! ([`unresolve_frame`]): a drag lands where the pointer put the object on the device shown.
//! Snapping is done on that device's pixels (Q10).

use crate::viewer::Viewer;
use quadrule_layout::snap::{Guide, Handle, PxRect, ResizeMods, SnapToggles, SnapWorld};
use quadrule_layout::units::Axis;
use quadrule_layout::{Shape, resolve_frame, unresolve_frame};
use quadrule_model::edit::{Command, EditError, duplicate, is_empty, set_frames};
use quadrule_model::{Constraints, Document, Frame, Object, ObjectKind, Uuid};
use quadrule_profiles::Profile;

/// A snap catches within this many screen points (rung 4 bbq, Defaulted "Snap distance"); the
/// window turns it into device pixels with the zoom, so it does not grow when zoomed in.
pub const SNAP_REACH_POINTS: f64 = 6.0;

/// A handle is picked within this many screen points of its centre.
pub const HANDLE_SLACK_POINTS: f64 = 6.0;

/// The fewest device pixels a box can be resized down to.
pub const MIN_SIZE_PX: f64 = 1.0;

/// A frame change smaller than this many mm is float noise from converting there and back.
const NOISE_MM: f64 = 1e-7;

/// A handle on the selection: one of the eight on its box, or one end of a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleHit {
    /// A handle of the selection's box.
    Box(Handle),
    /// A line's start: the top-left of its frame.
    LineStart,
    /// A line's end: the bottom-right of its frame.
    LineEnd,
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
    /// Its frame on the device shown, in pixels, as the drag began.
    resolved: PxRect,
    /// A text box: its height is its text's, so only its position and width change.
    text: bool,
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
    /// Marquee: the box being drawn, in device pixels.
    pub marquee: Option<PxRect>,
    /// The frames the drag would leave the objects with (primary mm), latest first-to-last.
    frames: Vec<(Uuid, Frame)>,
}

fn snap_rect(s: &Shape) -> PxRect {
    match *s {
        Shape::Segment { x0, y0, x1, y1, .. } => PxRect::from_corners(x0, y0, x1, y1),
        Shape::Box { x0, y0, x1, y1 } => PxRect { x0, y0, x1, y1 },
    }
}

/// An object's frame on `shown`, in pixels.
fn resolved_px(o: &Object, primary: &Profile, shown: &Profile) -> PxRect {
    let f = resolve_frame(o.frame, o.constraints, primary.page_mm(), shown.page_mm());
    let px = |mm: f64| shown.mm_to_px(mm);
    PxRect {
        x0: px(f.x_mm),
        y0: px(f.y_mm),
        x1: px(f.x_mm + f.w_mm),
        y1: px(f.y_mm + f.h_mm),
    }
}

/// The frame on the primary page that lands on `r` (pixels) on `shown`.
fn frame_from_px(r: PxRect, c: Constraints, primary: &Profile, shown: &Profile) -> Frame {
    let per_mm = shown.mm_to_px(1.0);
    let resolved = Frame::new(
        r.x0 / per_mm,
        r.y0 / per_mm,
        (r.x1 - r.x0) / per_mm,
        (r.y1 - r.y0) / per_mm,
    );
    unresolve_frame(resolved, c, primary.page_mm(), shown.page_mm())
}

/// Where `item` goes when the group's box `old` becomes `new`: moved, and scaled about the box.
/// A text box keeps its height; a box with no width (or height) is moved along that axis only.
fn map_rect(item: &Item, old: &PxRect, new: &PxRect) -> PxRect {
    let sx = if old.w() > 0.0 {
        new.w() / old.w()
    } else {
        1.0
    };
    let sy = if old.h() > 0.0 {
        new.h() / old.h()
    } else {
        1.0
    };
    let mx = |x: f64| new.x0 + (x - old.x0) * sx;
    let my = |y: f64| new.y0 + (y - old.y0) * sy;
    let r = item.resolved;
    let y0 = my(r.y0);
    PxRect {
        x0: mx(r.x0),
        y0,
        x1: mx(r.x1),
        y1: if item.text { y0 + r.h() } else { my(r.y1) },
    }
}

fn near(a: &Frame, b: &Frame) -> bool {
    (a.x_mm - b.x_mm).abs() < NOISE_MM
        && (a.y_mm - b.y_mm).abs() < NOISE_MM
        && (a.w_mm - b.w_mm).abs() < NOISE_MM
        && (a.h_mm - b.h_mm).abs() < NOISE_MM
}

impl Viewer {
    /// Whether a drag is in progress.
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// What caught on the last move of the drag, to draw.
    pub fn drag_guides(&self) -> &[Guide] {
        self.drag.as_ref().map_or(&[], |d| &d.guides)
    }

    /// The marquee being drawn, in device pixels.
    pub fn drag_marquee(&self) -> Option<PxRect> {
        self.drag.as_ref().and_then(|d| d.marquee)
    }

    /// The primary profile, the profile on screen, and the shapes on the page.
    fn geometry(&mut self) -> Option<(Profile, Profile, Vec<quadrule_layout::Placed>)> {
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
    /// one line is selected. None when nothing editable is selected.
    pub fn handles(&mut self) -> Vec<(HandleHit, f64, f64)> {
        let ids = self.editable_selection();
        if ids.is_empty() {
            return Vec::new();
        }
        let Some((_, _, placed)) = self.geometry() else {
            return Vec::new();
        };
        let Some(doc) = self.doc() else {
            return Vec::new();
        };
        if let [id] = ids[..]
            && matches!(doc.object(id).map(|o| &o.kind), Some(ObjectKind::Line(_)))
            && let Some(Shape::Segment { x0, y0, x1, y1, .. }) =
                placed.iter().find(|p| p.id == id).map(|p| p.shape)
        {
            return vec![(HandleHit::LineStart, x0, y0), (HandleHit::LineEnd, x1, y1)];
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

    /// The handle under (x, y) device pixels, within `slack` pixels; the nearest if several.
    pub fn handle_at(&mut self, x: f64, y: f64, slack: f64) -> Option<HandleHit> {
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
        let Some((primary, shown, placed)) = self.geometry() else {
            return false;
        };
        let before_press = self.selection.clone();

        let hit = self.handle_at(x, y, handle_slack);
        let kind = if let Some(h) = hit {
            DragKind::Handle(h)
        } else if let Some(id) = self.pick(x, y, slack) {
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
            marquee: None,
            frames: Vec::new(),
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
            let shape = placed
                .iter()
                .find(|p| p.id == *id)
                .map(|p| snap_rect(&p.shape));
            let resolved = resolved_px(o, &primary, &shown);
            drag.start_box = match (drag.items.is_empty(), shape) {
                (true, Some(s)) => s,
                (false, Some(s)) => drag.start_box.union(&s),
                (_, None) => drag.start_box,
            };
            drag.items.push(Item {
                id: *target,
                constraints: o.constraints,
                resolved,
                text: matches!(o.kind, ObjectKind::Text(_)),
            });
        }
        if drag.items.is_empty() {
            return false;
        }
        drag.frames = drag
            .items
            .iter()
            .filter_map(|i| base_doc.object(i.id).map(|o| (i.id, o.frame)))
            .collect();
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
        let start = drag.start_box;
        let (new_box, guides) = match drag.kind {
            DragKind::Move => {
                let raw = start.moved(delta.0, delta.1);
                let s = drag.world.snap_move(&raw, reach, toggles);
                (start.moved(delta.0 + s.dx, delta.1 + s.dy), s.guides)
            }
            DragKind::Handle(HandleHit::Box(h)) => {
                let all_text = drag.items.iter().all(|i| i.text);
                drag.world.snap_resize(
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
                )
            }
            DragKind::Handle(end) => {
                // One end of a line moves; the other stays.
                let (moving, fixed) = if end == HandleHit::LineStart {
                    ((start.x0, start.y0), (start.x1, start.y1))
                } else {
                    ((start.x1, start.y1), (start.x0, start.y0))
                };
                let (p, guides) =
                    drag.world
                        .snap_point((moving.0 + delta.0, moving.1 + delta.1), reach, toggles);
                (PxRect::from_corners(p.0, p.1, fixed.0, fixed.1), guides)
            }
            DragKind::Marquee => unreachable!("handled above"),
        };
        let line_end = matches!(
            drag.kind,
            DragKind::Handle(HandleHit::LineStart | HandleHit::LineEnd)
        );
        let base_doc = drag
            .duplicate
            .as_ref()
            .map(|(_, d)| d.clone())
            .or_else(|| self.doc().cloned());
        let Some(base_doc) = base_doc else {
            self.drag = Some(drag);
            return;
        };
        let mut frames = Vec::with_capacity(drag.items.len());
        for item in &drag.items {
            let px = if line_end {
                new_box
            } else {
                map_rect(item, &start, &new_box)
            };
            let mut frame = frame_from_px(px, item.constraints, &primary, &shown);
            if let Some(o) = base_doc.object(item.id)
                && near(&o.frame, &frame)
            {
                frame = o.frame;
            }
            frames.push((item.id, frame));
        }
        drag.guides = guides;
        if frames != drag.frames {
            let mut preview = base_doc;
            for (id, frame) in &frames {
                if let Some((l, o)) = preview.find(*id) {
                    preview.layers[l].objects[o].frame = *frame;
                }
            }
            drag.frames = frames;
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
                let moved = set_frames(with_copies, &drag.frames);
                if is_empty(&moved) {
                    // Nowhere to put the copies: leave things as they were.
                    self.selection = drag.selection_before;
                    self.bump();
                    return Ok(());
                }
                let copies: Vec<Uuid> = drag.items.iter().map(|i| i.id).collect();
                (Command::Batch(vec![dup.clone(), moved]), Some(copies))
            }
            None => (set_frames(&doc, &drag.frames), None),
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
        let Some(drag) = self.drag.take() else {
            return;
        };
        self.selection = drag.selection_before;
        self.hide_preview();
    }

    /// The rhythm lines to draw over the page for the device shown (rung 4 bbq Q9): the vertical
    /// lines' x positions and the horizontal lines' y positions, each with whether it is a full
    /// unit rather than a subdivision. They are an overlay and never exported.
    #[allow(clippy::type_complexity)]
    pub fn rhythm_overlay(&mut self) -> Option<(Vec<(f64, bool)>, Vec<(f64, bool)>)> {
        let (_, shown) = self.profiles()?;
        let world = SnapWorld::new(self.doc()?, &shown, &[]);
        let (w, h) = world.page();
        Some((world.lines(Axis::X, 0.0, w), world.lines(Axis::Y, 0.0, h)))
    }

    /// Objects a marquee box touches: visible, unlocked, in unlocked layers (what a click could
    /// pick), bottom first.
    fn marquee_hits(&mut self, r: &PxRect) -> Vec<Uuid> {
        let Some((_, _, placed)) = self.geometry() else {
            return Vec::new();
        };
        let Some(doc) = self.doc() else {
            return Vec::new();
        };
        placed
            .iter()
            .filter(|p| {
                let layer = &doc.layers[p.layer];
                let Some(o) = doc.object(p.id) else {
                    return false;
                };
                if !(layer.visible && !layer.locked && o.visible && !o.locked) {
                    return false;
                }
                match p.shape {
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
            .collect()
    }

    /// Shows `doc` instead of the open document until the preview is hidden.
    fn show_preview(&mut self, doc: Document) {
        if let Some(o) = &mut self.open {
            o.preview = Some(doc);
        }
        self.bump();
    }

    fn hide_preview(&mut self) {
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
