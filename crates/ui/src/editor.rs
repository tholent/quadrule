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

//! Editing the open document (rung 3, docs/bbq/2026-11-10-rung-3-first-editor.md), with no UI
//! in it: every change goes through [`Viewer::edit`] as a command in the document's history,
//! after which the shown device is laid out, rendered and linted again (Defaulted "Live
//! update"). Picking, nudging, Delete, Duplicate, reordering and Save live here too, so all of
//! it is tested headless.

use crate::viewer::{Source, Viewer};
use quadrule_format::{FontRef, QuadFile};
use quadrule_i18n::Message;
use quadrule_layout::lay_out_text;
use quadrule_layout::units::{Axis, UnitContext};
use quadrule_model::clipboard;
use quadrule_model::edit::{
    Command, EditError, LayerProps, OverrideField, Place, duplicate, group, is_empty, paste,
    remove, reorder_into, reset_override, set_font_size_override, set_frame_override,
    set_position_override, set_visible_override, ungroup_with,
};
use quadrule_model::{Document, Extra, Frame, Object, ObjectKind, Uuid};
use quadrule_profiles::{Profile, builtin_for};
use quadrule_text::BundledFont;
use std::path::Path;

/// Recorded in each saved file as the app that last saved it (ADR-001 item 5).
pub const SAVED_BY: &str = concat!("quadrule ", env!("CARGO_PKG_VERSION"));

/// Pointer slack for picking thin lines, in screen points (rung 3 bbq, Defaulted "Selecting").
pub const PICK_SLACK_POINTS: f64 = 4.0;

/// Which way a reorder goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toward {
    /// Up the stack: drawn later, over more.
    Front,
    /// Down the stack.
    Back,
}

impl Viewer {
    /// The open document.
    pub fn doc(&self) -> Option<&Document> {
        self.open.as_ref().map(|o| &o.doc)
    }

    /// Does `cmd` as one undo step. With a `merge` key, steps with the same key join until the
    /// history is sealed (typing in one field; Defaulted "Commands and undo"). A refused command
    /// changes nothing and the notice says why.
    pub fn edit(&mut self, cmd: Command, merge: Option<&str>) -> Result<(), EditError> {
        let Some(open) = &mut self.open else {
            return Ok(());
        };
        match open.history.apply(&mut open.doc, cmd, merge) {
            Ok(()) => {
                self.changed();
                Ok(())
            }
            Err(e) => {
                self.notice = Some(crate::viewer::Notice {
                    message: edit_error_message(&e, &open.doc),
                    error: true,
                });
                Err(e)
            }
        }
    }

    /// Ends merging into the last step (a field lost focus).
    pub fn seal(&mut self) {
        if let Some(o) = &mut self.open {
            o.history.seal();
        }
    }

    /// Undoes the last step.
    pub fn undo(&mut self) -> bool {
        let done = self
            .open
            .as_mut()
            .is_some_and(|o| o.history.undo(&mut o.doc));
        if done {
            self.changed();
        }
        done
    }

    /// Redoes the last undone step.
    pub fn redo(&mut self) -> bool {
        let done = self
            .open
            .as_mut()
            .is_some_and(|o| o.history.redo(&mut o.doc));
        if done {
            self.changed();
        }
        done
    }

    /// Whether the open document has changes its file does not have.
    pub fn is_dirty(&self) -> bool {
        self.open.as_ref().is_some_and(|o| o.history.is_dirty())
    }

    /// After any change: drop the rendered pages (each is made again on first use), drop
    /// selected objects that are gone, and clear an outlined finding (its index may be stale).
    fn changed(&mut self) {
        self.revision += 1;
        self.selected = None;
        if let Some(o) = &mut self.open {
            o.pages.clear();
            let doc = &o.doc;
            self.selection.retain(|id| doc.object(*id).is_some());
            // Only the groups still there, still one inside the other.
            let mut keep = 0;
            for (i, g) in self.entered.iter().enumerate() {
                let outer_ok = i == 0 || doc.parent_of(*g) == Some(self.entered[i - 1]);
                if doc.object(*g).is_some_and(Object::is_group) && outer_ok {
                    keep = i + 1;
                } else {
                    break;
                }
            }
            self.entered.truncate(keep);
        }
    }

    /// The primary profile and the profile on screen.
    pub fn profiles(&self) -> Option<(Profile, Profile)> {
        let doc = self.doc()?;
        Some((
            builtin_for(&doc.primary_profile_id, doc.page.orientation).ok()?,
            builtin_for(&self.profile_id, doc.page.orientation).ok()?,
        ))
    }

    /// How typed lengths read on `axis` (rung 3 bbq Q10).
    pub fn unit_context(&self, axis: Axis) -> Option<UnitContext> {
        let (primary, shown) = self.profiles()?;
        let doc = self.doc()?;
        Some(UnitContext::new(doc, &primary, &shown, axis))
    }

    /// The innermost object under (x, y) on the device on screen, in device pixels, within `slack`
    /// pixels for thin lines: visible, unlocked, and not a group (a group is hit where one of its
    /// children is, rung 7a bbq, "Picking, dragging, snapping").
    pub fn pick_leaf(&mut self, x: f64, y: f64, slack: f64) -> Option<Uuid> {
        let placed = self.page()?.placed.clone();
        placed
            .iter()
            .rev()
            .find(|p| !p.node.group && !p.node.hidden && !p.node.locked && p.shape.hit(x, y, slack))
            .map(|p| p.id)
    }

    /// What a click at (x, y) picks given the groups entered: the outermost group over the object
    /// under the pointer, or, inside an entered group, that group's child on the way to it (Q4).
    pub fn pick(&mut self, x: f64, y: f64, slack: f64) -> Option<Uuid> {
        let leaf = self.pick_leaf(x, y, slack)?;
        self.target_of(leaf, &self.entered.clone())
    }

    /// The object that stands for `leaf` when the groups `entered` are open: the first object on
    /// its way down from the outside that is not one of those groups. A leaf outside the entered
    /// groups is stood for by its outermost group (or itself).
    pub(crate) fn target_of(&self, leaf: Uuid, entered: &[Uuid]) -> Option<Uuid> {
        let doc = self.doc()?;
        let mut chain = doc.ancestors(leaf);
        chain.push(leaf);
        let k = entered
            .iter()
            .zip(&chain)
            .take_while(|(e, c)| e == c)
            .count();
        chain.get(k).copied()
    }

    /// A click on the page at (x, y) device pixels: selects what is under it, or with `add`
    /// (Shift) adds or removes it. A click on nothing clears the selection, unless adding.
    pub fn click(&mut self, x: f64, y: f64, slack: f64, add: bool) {
        self.click_with(x, y, slack, add, false);
    }

    /// [`Viewer::click`], and with `deep` (Ctrl) it picks the innermost object at once, entering
    /// the groups on the way (Q4). A click outside the group entered steps back out one level.
    pub fn click_with(&mut self, x: f64, y: f64, slack: f64, add: bool, deep: bool) {
        let leaf = self.pick_leaf(x, y, slack);
        let Some(leaf) = leaf else {
            if !add {
                self.selection.clear();
                if self.outside_entered(x, y) {
                    self.entered.pop();
                }
            }
            return;
        };
        if deep {
            self.entered = self.doc().map(|d| d.ancestors(leaf)).unwrap_or_default();
            self.select_one(leaf, add);
            return;
        }
        // A leaf that is not inside the group entered: step out one level first.
        let inside = |v: &Viewer, entered: &[Uuid]| {
            v.doc().is_some_and(|d| {
                let chain = d.ancestors(leaf);
                entered.len() <= chain.len() && entered.iter().zip(&chain).all(|(a, b)| a == b)
            })
        };
        if !inside(self, &self.entered.clone()) {
            self.entered.pop();
        }
        if let Some(t) = self.target_of(leaf, &self.entered.clone()) {
            self.select_one(t, add);
        }
    }

    fn select_one(&mut self, id: Uuid, add: bool) {
        if add {
            self.toggle(id);
        } else {
            self.selection = vec![id];
        }
    }

    /// Whether (x, y) is outside the box of the innermost group entered (true when none is).
    fn outside_entered(&mut self, x: f64, y: f64) -> bool {
        let Some(g) = self.entered.last().copied() else {
            return false;
        };
        let Some(page) = self.page() else {
            return false;
        };
        page.placed
            .iter()
            .find(|p| p.id == g)
            .is_none_or(|p| !p.shape.hit(x, y, 0.0))
    }

    /// A double-click at (x, y): goes one level into the group under the pointer and picks the
    /// child there (Q4). Tested through this method, since the test harness cannot make a
    /// double-click. False when there is no group to go into there.
    pub fn enter_at(&mut self, x: f64, y: f64, slack: f64) -> bool {
        let Some(leaf) = self.pick_leaf(x, y, slack) else {
            return false;
        };
        let Some(doc) = self.doc() else {
            return false;
        };
        let mut chain = doc.ancestors(leaf);
        chain.push(leaf);
        let k = self
            .entered
            .iter()
            .zip(&chain)
            .take_while(|(e, c)| e == c)
            .count();
        // Not inside the group entered, or nothing to enter (the object itself is a leaf).
        if k < self.entered.len() || k + 1 >= chain.len() {
            return false;
        }
        let (group, child) = (chain[k], chain[k + 1]);
        self.entered.push(group);
        self.selection = vec![child];
        true
    }

    /// Escape: steps back out of the innermost group entered, selecting that group. False when
    /// none is entered.
    pub fn leave_group(&mut self) -> bool {
        match self.entered.pop() {
            Some(g) => {
                self.selection = vec![g];
                true
            }
            None => false,
        }
    }

    /// Picks `id` at any depth, entering the groups round it (a row of the layers panel; Q4).
    pub fn select_row(&mut self, id: Uuid, add: bool) {
        self.entered = self.doc().map(|d| d.ancestors(id)).unwrap_or_default();
        self.select_one(id, add);
    }

    /// Adds `id` to the selection, or takes it out if it is there. A group and something inside it
    /// are never both selected: picking one drops the other.
    pub fn toggle(&mut self, id: Uuid) {
        if let Some(i) = self.selection.iter().position(|s| *s == id) {
            self.selection.remove(i);
            return;
        }
        if let Some(doc) = self.doc() {
            let up = doc.ancestors(id);
            let inside: Vec<Uuid> = self
                .selection
                .iter()
                .copied()
                .filter(|s| doc.ancestors(*s).contains(&id))
                .collect();
            if self.selection.iter().any(|s| up.contains(s)) {
                return;
            }
            self.selection.retain(|s| !inside.contains(s));
        }
        self.selection.push(id);
    }

    /// The selected objects that can be edited (not locked, nor in a locked layer).
    pub fn editable_selection(&self) -> Vec<Uuid> {
        let Some(doc) = self.doc() else {
            return Vec::new();
        };
        self.selection
            .iter()
            .copied()
            .filter(|id| doc.editable(*id))
            .filter(|id| {
                !doc.ancestors(*id)
                    .iter()
                    .any(|a| self.selection.contains(a))
            })
            .collect()
    }

    /// Changes every editable selected object with `f`, as one step.
    pub fn edit_selected(
        &mut self,
        merge: Option<&str>,
        f: impl Fn(&mut Object),
    ) -> Result<(), EditError> {
        let ids = self.editable_selection();
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmds: Vec<Command> = ids
            .iter()
            .filter_map(|id| doc.object(*id))
            .map(|o| {
                let mut o = o.clone();
                f(&mut o);
                Command::SetObject(o)
            })
            .collect();
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), merge)?;
        self.say_shared();
        Ok(())
    }

    /// Says in the status bar that an edit just made changed the design on every device, when it
    /// was made with a secondary device on screen (rung 8 bbq, Q3).
    pub(crate) fn say_shared(&mut self) {
        if let Some(dev) = self.override_device() {
            let name = crate::viewer::device_label(&dev);
            self.notice = Some(crate::viewer::Notice {
                message: Message::new("notice-shared-edit").text("device", name),
                error: false,
            });
        }
    }

    /// The device whose overrides an edit writes: the one on screen, when it is not the primary
    /// one (rung 8 bbq, Q3). `None` when the primary device is on screen: edits are the design's.
    pub fn override_device(&self) -> Option<String> {
        let doc = self.doc()?;
        (self.profile_id != doc.primary_profile_id
            && doc.profile_ids().contains(&self.profile_id.as_str()))
        .then(|| self.profile_id.clone())
    }

    /// Moves the selection by `dx`, `dy` device pixels of the device on screen (arrow keys), or,
    /// with `rhythm`, by that many rhythm units (or subdivisions, when on) as that device resolves them
    /// (Shift+arrow; napkin §6.2). Designs live in the primary profile's mm, so the step is converted at the shown
    /// device's pixel size (rung 3 bbq, Defaulted "Edits change the shared design").
    pub fn nudge(&mut self, dx: f64, dy: f64, rhythm: bool) -> Result<(), EditError> {
        let (Some(cx), Some(cy)) = (self.unit_context(Axis::X), self.unit_context(Axis::Y)) else {
            return Ok(());
        };
        // One rhythm unit, or one subdivision of it when subdivisions are on (napkin §6.2).
        let parts = self
            .doc()
            .and_then(|d| d.rhythm.subdivisions_on())
            .map_or(1.0, f64::from);
        let (sx, sy) = if rhythm {
            (cx.rhythm_mm / parts, cy.rhythm_mm / parts)
        } else {
            (1.0 / cx.px_per_mm, 1.0 / cy.px_per_mm)
        };
        self.locked_notice_if_none_editable();
        let ids = self.editable_selection();
        if let Some(dev) = self.override_device() {
            // A secondary device: the step is a position override there (rung 8 bbq, Q3).
            return self.nudge_on_device(&dev, &ids, (dx * sx, dy * sy));
        }
        // Inside a turned group the page's down and right are not the group's: carry the step back
        // through the turns round the object.
        let degs: Vec<f64> = match self.page() {
            Some(page) => ids
                .iter()
                .map(|id| {
                    page.placed
                        .iter()
                        .find(|p| p.id == *id)
                        .map_or(0.0, |p| p.node.ctx.xf.deg)
                })
                .collect(),
            None => vec![0.0; ids.len()],
        };
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmds: Vec<Command> = ids
            .iter()
            .zip(&degs)
            .filter_map(|(id, deg)| {
                let mut o = doc.object(*id)?.clone();
                let (mx, my) =
                    quadrule_layout::rotate::rotate_point((dx * sx, dy * sy), (0.0, 0.0), -deg);
                o.translate(mx, my);
                Some(Command::SetObject(o))
            })
            .collect();
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), None)
    }

    /// The layout of `ids` on the device shown, for working out overrides: each object with its
    /// place in the space it is in.
    fn nodes_shown(&mut self, ids: &[Uuid]) -> Vec<quadrule_layout::tree::Node> {
        let Some(page) = self.page() else {
            return Vec::new();
        };
        ids.iter()
            .filter_map(|id| page.placed.iter().find(|p| p.id == *id).map(|p| p.node))
            .collect()
    }

    /// Moves each of `ids` by `step` mm of the device `dev` on screen (as the page sees it), as a
    /// position override there.
    fn nudge_on_device(
        &mut self,
        dev: &str,
        ids: &[Uuid],
        step: (f64, f64),
    ) -> Result<(), EditError> {
        let nodes = self.nodes_shown(ids);
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmds: Vec<Command> = nodes
            .iter()
            .filter_map(|n| {
                let mut o = doc.object(n.id)?.clone();
                // Inside a turned group the page's down and right are not the group's.
                let (mx, my) =
                    quadrule_layout::rotate::rotate_point(step, (0.0, 0.0), -n.ctx.xf.deg);
                let moved = Frame {
                    x_mm: n.local.x_mm + mx,
                    y_mm: n.local.y_mm + my,
                    ..n.local
                };
                let unfit = n
                    .ctx
                    .unfit(moved, o.rotation(), matches!(o.kind, ObjectKind::Text(_)));
                set_position_override(&mut o, dev, n.base, unfit.x_mm, unfit.y_mm);
                Some(Command::SetObject(o))
            })
            .collect();
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), None)
    }

    /// Sets the place and size of every editable selected object on the secondary device on screen
    /// (the Arrange tab's X, Y, width and height there): `f` gets each object's frame as that
    /// device shows it and gives the one to show. An override for each part that differs from what
    /// the constraints give; the design is untouched (rung 8 bbq, Q3). Does nothing on the primary
    /// device.
    pub fn edit_frames_on_device(
        &mut self,
        merge: Option<&str>,
        f: impl Fn(Frame) -> Frame,
    ) -> Result<(), EditError> {
        let Some(dev) = self.override_device() else {
            return Ok(());
        };
        let ids = self.editable_selection();
        let nodes = self.nodes_shown(&ids);
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmds: Vec<Command> = nodes
            .iter()
            .filter_map(|n| {
                let mut o = doc.object(n.id)?.clone();
                let text = matches!(o.kind, ObjectKind::Text(_));
                let unfit = n.ctx.unfit(f(n.local), o.rotation(), text);
                set_frame_override(&mut o, &dev, n.base, unfit);
                (o != *doc.object(n.id)?).then_some(Command::SetObject(o))
            })
            .collect();
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), merge)
    }

    /// Sets the font size, in points, of the editable selected text boxes on the secondary device
    /// on screen (an override there; the design's size is untouched).
    pub fn set_font_size_on_device(
        &mut self,
        merge: Option<&str>,
        pt: f64,
    ) -> Result<(), EditError> {
        let Some(dev) = self.override_device() else {
            return Ok(());
        };
        self.edit_selected_raw(merge, |o| set_font_size_override(o, &dev, pt))
    }

    /// Shows or hides `id` on the secondary device on screen: an override of the eye there (the
    /// layers panel's eye). Allowed on locked objects, as the eye is.
    pub fn set_visible_on_device(&mut self, id: Uuid, shown: bool) -> Result<(), EditError> {
        let Some(dev) = self.override_device() else {
            return Ok(());
        };
        let Some(o) = self.doc().and_then(|d| d.object(id)) else {
            return Ok(());
        };
        let mut o = o.clone();
        set_visible_override(&mut o, &dev, shown);
        self.edit(Command::SetObject(o), None)
    }

    /// Like [`Viewer::edit_selected`] without the note that the design changed everywhere.
    fn edit_selected_raw(
        &mut self,
        merge: Option<&str>,
        f: impl Fn(&mut Object),
    ) -> Result<(), EditError> {
        let ids = self.editable_selection();
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmds: Vec<Command> = ids
            .iter()
            .filter_map(|id| doc.object(*id))
            .filter_map(|o| {
                let mut n = o.clone();
                f(&mut n);
                (n != *o).then_some(Command::SetObject(n))
            })
            .collect();
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), merge)
    }

    /// Takes the parts `fields` (or every part) of the override on the device on screen off each
    /// of `ids`: Reset in the Overrides tab. One undo step.
    pub fn reset_overrides(
        &mut self,
        ids: &[Uuid],
        fields: Option<&[OverrideField]>,
    ) -> Result<(), EditError> {
        let Some(dev) = self.override_device() else {
            return Ok(());
        };
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let fields = fields.unwrap_or(&OverrideField::ALL);
        let cmds: Vec<Command> = ids
            .iter()
            .filter_map(|id| doc.object(*id))
            .filter_map(|o| {
                let mut n = o.clone();
                reset_override(&mut n, &dev, fields);
                (n != *o).then_some(Command::SetObject(n))
            })
            .collect();
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), None)
    }

    /// Makes `id` the document's primary device (the Page tab): its layout, overrides included,
    /// becomes the design, and the other devices get the overrides that keep what they show (rung 8
    /// bbq, Defaulted "Switching the primary device"). One undo step.
    pub fn make_primary(&mut self, id: &str) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        match quadrule_layout::device::make_primary(doc, id) {
            Ok(cmd) => self.edit(cmd, None),
            Err(e) => {
                self.notice = Some(crate::viewer::Notice {
                    message: Message::new("notice-edit-failed").text("error", e.to_string()),
                    error: true,
                });
                Ok(())
            }
        }
    }

    /// Holds the page `to` (the Page tab's Orientation, rung 8b): the design follows its objects'
    /// own constraints onto the turned page and every override box maps onto it in proportion
    /// ([`quadrule_layout::device::set_orientation`]). One undo step.
    pub fn set_orientation(&mut self, to: quadrule_model::Orientation) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        if doc.page.orientation == to {
            return Ok(());
        }
        match quadrule_layout::device::set_orientation(doc, to) {
            Ok(cmd) => self.edit(cmd, None),
            Err(e) => {
                self.notice = Some(crate::viewer::Notice {
                    message: Message::new("notice-edit-failed").text("error", e.to_string()),
                    error: true,
                });
                Ok(())
            }
        }
    }

    /// Makes the document target exactly the devices `ids` (the Page tab's ticks); a device taken
    /// out loses its overrides, and undo brings them back. If the device on screen goes, the
    /// primary one is shown.
    pub fn set_targets(&mut self, ids: &[&str]) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmd = match quadrule_layout::device::set_targets(doc, ids) {
            Ok(c) => c,
            Err(e) => {
                self.notice = Some(crate::viewer::Notice {
                    message: Message::new("notice-edit-failed").text("error", e.to_string()),
                    error: true,
                });
                return Ok(());
            }
        };
        self.edit(cmd, None)?;
        if let Some(doc) = self.doc()
            && !doc.profile_ids().contains(&self.profile_id.as_str())
        {
            let primary = doc.primary_profile_id.clone();
            self.set_profile(&primary);
        }
        Ok(())
    }

    /// Deletes the editable selected objects (the Delete key).
    pub fn delete_selection(&mut self) -> Result<(), EditError> {
        let ids = self.editable_selection();
        self.locked_notice_if_none_editable();
        if ids.is_empty() {
            return Ok(());
        }
        self.edit(remove(&ids), None)?;
        self.selection.clear();
        Ok(())
    }

    /// Duplicates the selected objects in place, each just above its original, and selects the
    /// copies (Ctrl+D; rung 3 bbq, Defaulted "Delete and Duplicate"). Copies are unlocked.
    pub fn duplicate_selection(&mut self) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let ids: Vec<Uuid> = self
            .selection
            .iter()
            .copied()
            .filter(|id| doc.layer_of(*id).is_some_and(|l| !doc.layers[l].locked))
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        let (cmd, copies) = duplicate(doc, &ids, Uuid::new_v4);
        self.edit(cmd, None)?;
        self.selection = copies;
        Ok(())
    }

    /// The clipboard text for the selected objects, locked ones too (Ctrl+C; rung 4b bbq,
    /// Defaulted "Copy and paste"). Nothing is changed.
    pub fn copy_text(&mut self) -> Option<String> {
        let text = clipboard::to_text(self.doc()?, &self.selection)?;
        // A new copy starts the paste steps again.
        self.paste_run = None;
        Some(text)
    }

    /// Copies the editable selected objects and deletes them (Ctrl+X); the text is for the
    /// clipboard. Locked objects stay, and are not copied.
    pub fn cut_text(&mut self) -> Result<Option<String>, EditError> {
        let ids = self.editable_selection();
        self.locked_notice_if_none_editable();
        let Some(text) = self.doc().and_then(|d| clipboard::to_text(d, &ids)) else {
            return Ok(None);
        };
        self.paste_run = None;
        self.edit(remove(&ids), None)?;
        self.selection.retain(|id| !ids.contains(id));
        Ok(Some(text))
    }

    /// Pastes clipboard `text` if it is quadrule's (Ctrl+V); returns whether anything was
    /// pasted. The copies are new objects on top of the layer of the first selected object (or
    /// the top unlocked layer), and become the selection. Into the document they came from they
    /// land one rhythm unit down and right, and each paste in a row one unit further; into
    /// another document the first lands in the same place.
    pub fn paste_text(&mut self, text: &str) -> Result<bool, EditError> {
        let Some(objects) = clipboard::from_text(text) else {
            return Ok(false);
        };
        let Some(doc) = self.doc() else {
            return Ok(false);
        };
        let layer = self
            .selection
            .iter()
            .find_map(|id| doc.layer_of(*id))
            .filter(|&l| !doc.layers[l].locked)
            .or_else(|| doc.layers.iter().rposition(|l| !l.locked));
        let Some(layer) = layer else {
            self.notice = Some(crate::viewer::Notice {
                message: Message::new("notice-locked"),
                error: true,
            });
            return Ok(false);
        };
        let times = match &self.paste_run {
            Some((t, n)) if t == text => n + 1,
            _ => 1,
        };
        let from_here = objects.iter().any(|o| doc.object(o.id).is_some());
        let steps = f64::from(if from_here { times } else { times - 1 });
        let (step_x, step_y) = doc.rhythm.steps_mm();
        let offset = (step_x * steps, step_y * steps);
        let (cmd, new) = paste(doc, &objects, layer, offset, Uuid::new_v4);
        if is_empty(&cmd) {
            return Ok(false);
        }
        self.edit(cmd, None)?;
        self.selection = new;
        self.paste_run = Some((text.to_owned(), times));
        Ok(true)
    }

    /// Moves each selected object one place up or down within what holds it (its layer, or its
    /// group). An object already at the end stays.
    pub fn reorder_selection(&mut self, toward: Toward) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let mut places: Vec<(quadrule_model::edit::Loc, Place, Uuid)> = self
            .editable_selection()
            .iter()
            .filter_map(|id| {
                let loc = doc.locate(*id)?;
                let (place, _) = doc.place_of(*id)?;
                Some((loc, place, *id))
            })
            .collect();
        // Move the ones nearest the end first, so they make room for the others.
        places.sort_by(|a, b| (a.0.layer, &a.0.path).cmp(&(b.0.layer, &b.0.path)));
        if toward == Toward::Front {
            places.reverse();
        }
        let mut cmds = Vec::new();
        let mut shadow = doc.clone();
        for (_, place, id) in places {
            let (_, o) = shadow.place_of(id).expect("still there");
            let len = shadow.children_of(&place).map_or(0, <[Object]>::len);
            let to = match toward {
                Toward::Front if o + 1 < len => o + 1,
                Toward::Back if o > 0 => o - 1,
                _ => continue,
            };
            // Never jump over another selected object: they keep their order.
            let neighbour = shadow.children_of(&place).expect("a place")[to].id;
            if self.selection.contains(&neighbour) {
                continue;
            }
            let cmd = Command::move_to(id, &place, to);
            shadow.apply(cmd.clone())?;
            cmds.push(cmd);
        }
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), None)
    }

    /// Moves layer `index` one place up or down the stack.
    pub fn reorder_layer(&mut self, index: usize, toward: Toward) -> Result<(), EditError> {
        let Some(n) = self.doc().map(|d| d.layers.len()) else {
            return Ok(());
        };
        let to = match toward {
            Toward::Front if index + 1 < n => index + 1,
            Toward::Back if index > 0 => index - 1,
            _ => return Ok(()),
        };
        self.edit(Command::MoveLayer { from: index, to }, None)
    }

    /// Moves an object to `index` in `layer` (counted after it is taken out of its place): the
    /// layers panel's drag-to-reorder (rung 4 bbq Q6). One undo step; a move to the place it is
    /// already in does nothing. Locked objects and locked layers refuse it.
    pub fn move_object_to(
        &mut self,
        id: Uuid,
        layer: usize,
        index: usize,
    ) -> Result<(), EditError> {
        self.move_object_into(id, &Place::Layer(layer), index)
    }

    /// The same into any container: a layer, or a group (rung 7a bbq, "Layers tree"). Moving across
    /// the edge of a turned group is refused with a note, because the object would move on the page.
    pub fn move_object_into(
        &mut self,
        id: Uuid,
        place: &Place,
        index: usize,
    ) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let turned = |chain: &[Uuid]| {
            chain
                .iter()
                .any(|g| doc.object(*g).is_some_and(|o| o.rotation() != 0.0))
        };
        let from = doc.ancestors(id);
        let mut to = match place {
            Place::Layer(_) => Vec::new(),
            Place::Group(g) => doc.ancestors(*g),
        };
        if let Place::Group(g) = place {
            to.push(*g);
        }
        if from != to && (turned(&from) || turned(&to)) {
            self.notice = Some(crate::viewer::Notice {
                message: Message::new("notice-turned-move"),
                error: true,
            });
            return Ok(());
        }
        let cmd = reorder_into(doc, id, place, index);
        if is_empty(&cmd) {
            return Ok(());
        }
        self.edit(cmd, None)
    }

    /// Groups the selected objects into one new group (Ctrl+G; rung 7a bbq, "Making and
    /// breaking") and selects it. One undo step.
    pub fn group_selection(&mut self) -> Result<(), EditError> {
        let ids = self.editable_selection();
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let new_id = Uuid::new_v4();
        let cmd = match group(doc, &ids, new_id, &self.names.group) {
            Ok(c) => c,
            Err(e) => {
                self.notice = Some(crate::viewer::Notice {
                    message: edit_error_message(&e, doc),
                    error: true,
                });
                return Err(e);
            }
        };
        self.edit(cmd, None)?;
        self.selection = vec![new_id];
        Ok(())
    }

    /// Ungroups the selected groups (Ctrl+Shift+G) and selects what was inside them. One undo
    /// step.
    pub fn ungroup_selection(&mut self) -> Result<(), EditError> {
        let ids: Vec<Uuid> = self
            .editable_selection()
            .into_iter()
            .filter(|id| {
                self.doc()
                    .and_then(|d| d.object(*id))
                    .is_some_and(Object::is_group)
            })
            .collect();
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        if ids.is_empty() {
            self.notice = Some(crate::viewer::Notice {
                message: Message::new("notice-ungroup-none"),
                error: true,
            });
            return Ok(());
        }
        let mut cmds = Vec::new();
        let mut freed = Vec::new();
        let Some((primary, _)) = self.profiles() else {
            return Ok(());
        };
        for id in &ids {
            // What the group's overrides did to the children on each device is baked in.
            let baked = quadrule_layout::device::ungroup_overrides(doc, &primary, *id);
            cmds.push(ungroup_with(doc, *id, &baked)?);
            freed.extend(
                doc.object(*id)
                    .map(|g| g.children().iter().map(|c| c.id).collect::<Vec<_>>())
                    .unwrap_or_default(),
            );
        }
        self.edit(Command::Batch(cmds), None)?;
        self.selection = freed;
        Ok(())
    }

    /// Moves layer `from` to index `to` in the stack (drag-to-reorder of a layer row).
    pub fn move_layer_to(&mut self, from: usize, to: usize) -> Result<(), EditError> {
        if from == to {
            return Ok(());
        }
        self.edit(Command::MoveLayer { from, to }, None)
    }

    /// Sets a layer's name, eye or lock (the layers panel).
    pub fn set_layer(
        &mut self,
        index: usize,
        f: impl Fn(&mut LayerProps),
    ) -> Result<(), EditError> {
        let Some(layer) = self.doc().and_then(|d| d.layers.get(index)) else {
            return Ok(());
        };
        let mut props = LayerProps::of(layer);
        f(&mut props);
        self.edit(Command::SetLayer { index, props }, None)
    }

    /// Sets one object's name, eye or lock (the layers panel; allowed on locked objects).
    pub fn set_object_panel(&mut self, id: Uuid, f: impl Fn(&mut Object)) -> Result<(), EditError> {
        let Some(o) = self.doc().and_then(|d| d.object(id)) else {
            return Ok(());
        };
        let mut o = o.clone();
        f(&mut o);
        self.edit(Command::SetObject(o), None)
    }

    pub(crate) fn locked_notice_if_none_editable(&mut self) {
        if !self.selection.is_empty() && self.editable_selection().is_empty() {
            self.notice = Some(crate::viewer::Notice {
                message: Message::new("notice-locked"),
                error: true,
            });
        }
    }

    /// The file's bytes as Save would write them: the document and, for every profile it has,
    /// its text outlines (ADR-001 item 2), through the `format` crate. A profile whose text
    /// cannot be laid out here (a missing font) keeps the outlines the file had.
    pub fn file_bytes(&mut self) -> Result<Vec<u8>, String> {
        let Viewer { open, fonts, .. } = self;
        let open = open.as_mut().ok_or("nothing is open")?;
        if let Some(r) = &open.read_only {
            return Err(format!("{r:?}"));
        }
        let doc = &open.doc;
        let primary = builtin_for(&doc.primary_profile_id, doc.page.orientation)
            .map_err(|e| e.to_string())?;
        let mut outlines = Vec::new();
        for id in doc.profile_ids() {
            let target = builtin_for(id, doc.page.orientation).map_err(|e| e.to_string())?;
            let laid = lay_out_text(doc, &primary, &target, fonts);
            let missing = laid.as_ref().map_or(true, |l| l.missing_glyphs > 0);
            match open.saved.get(id) {
                Some(saved) if missing => outlines.push(saved.clone()),
                _ => outlines.push(laid.map_err(|e| e.to_string())?.outlines),
            }
        }
        let mut file = QuadFile::new(doc.clone(), outlines, font_refs(doc));
        if let Some(m) = &open.manifest {
            // Keep what the file's manifest carried (extras, a newer version label); the font
            // list is made fresh from the document.
            let fonts = file.manifest.fonts.clone();
            file.manifest = m.clone();
            file.manifest.fonts = fonts;
        }
        file.to_bytes(SAVED_BY).map_err(|e| e.to_string())
    }

    /// Saves to `path` (Save As, or Save to the file it came from) and makes `path` the
    /// document's file. Writes a new file beside it, then renames it over the old one, so a
    /// failed save never leaves half a file. On failure the notice says why.
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let result = self.file_bytes().and_then(|bytes| {
            write_replacing(path, &bytes).map_err(|e| format!("{}: {e}", path.display()))
        });
        match &result {
            Ok(()) => {
                if let Some(o) = &mut self.open {
                    o.source = Source::File(path.to_path_buf());
                    o.history.mark_saved();
                }
                self.notice = Some(crate::viewer::Notice {
                    message: Message::new("notice-saved").text("path", path.display().to_string()),
                    error: false,
                });
            }
            Err(e) => {
                self.notice = Some(crate::viewer::Notice {
                    message: Message::new("notice-save-failed").text("error", e.clone()),
                    error: true,
                });
            }
        }
        result
    }

    /// The file the open document saves to with Save, if it has one (the sample page has none,
    /// so Save asks where, as Save As does).
    pub fn file_path(&self) -> Option<&Path> {
        match &self.open.as_ref()?.source {
            Source::File(p) => Some(p),
            Source::Sample | Source::Untitled => None,
        }
    }
}

/// The fonts a document's text uses, each once: bundled ones by id and version (napkin §7.3).
pub fn font_refs(doc: &Document) -> Vec<FontRef> {
    let mut families: Vec<&str> = doc
        .objects()
        .filter_map(|o| match &o.kind {
            ObjectKind::Text(t) => Some(t.family.as_str()),
            _ => None,
        })
        .collect();
    families.sort();
    families.dedup();
    families
        .into_iter()
        .map(|f| match BundledFont::for_family(f) {
            Some(b) => FontRef {
                family: b.family.into(),
                bundled_id: Some(b.id.into()),
                version: b.version(),
                extra: Extra::new(),
            },
            None => FontRef {
                family: f.into(),
                bundled_id: None,
                version: None,
                extra: Extra::new(),
            },
        })
        .collect()
}

/// Writes `bytes` to a new file beside `path`, then renames it over `path`.
pub fn write_replacing(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.saving"));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

fn edit_error_message(e: &EditError, doc: &Document) -> Message {
    match e {
        EditError::Locked(id) => Message::new("notice-object-locked").text(
            "name",
            doc.object(*id).map(|o| o.name.clone()).unwrap_or_default(),
        ),
        EditError::LayerLocked(i) => Message::new("notice-layer-locked").text(
            "name",
            doc.layers
                .get(*i)
                .map(|l| l.name.clone())
                .unwrap_or_default(),
        ),
        EditError::TurnedGroup(id) => Message::new("notice-turned-group").text(
            "name",
            doc.object(*id).map(|o| o.name.clone()).unwrap_or_default(),
        ),
        EditError::MixedParents => Message::new("notice-group-mixed"),
        EditError::TooFew => Message::new("notice-group-few"),
        other => Message::new("notice-edit-failed").text("error", other.to_string()),
    }
}

/// Where a dragged object was dropped in the layers panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropOn {
    /// On the row of the object at this place: above it (drawn earlier in the list, later in the
    /// stack) or below it.
    Object {
        /// The target's layer.
        layer: usize,
        /// The target's index in its layer, bottom first.
        index: usize,
    },
    /// On a layer's own row: the object goes to the top of that layer.
    LayerTop(usize),
}

/// The (layer, index) to give [`Viewer::move_object_to`] for `dragged` dropped on `target`.
/// `above` is whether the pointer was in the upper half of a target object's row (the panel
/// lists the top of the stack first, so above means later in the layer).
pub fn object_drop_place(
    doc: &Document,
    dragged: Uuid,
    target: DropOn,
    above: bool,
) -> Option<(usize, usize)> {
    let (dl, di) = doc.find(dragged)?;
    let (layer, raw) = match target {
        DropOn::Object { layer, index } => (layer, if above { index + 1 } else { index }),
        DropOn::LayerTop(layer) => (layer, doc.layers.get(layer)?.objects.len()),
    };
    // The index counts after the dragged object leaves its place.
    let index = if dl == layer && di < raw {
        raw - 1
    } else {
        raw
    };
    Some((layer, index))
}

/// Which part of an object's row in the layers tree a drop landed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowZone {
    /// The upper part: above it in the list, so later in the stack.
    Above,
    /// The middle of a group's row: inside the group, on top (rung 7a bbq, "Layers tree").
    Inside,
    /// The lower part.
    Below,
}

/// Where `dragged` goes when dropped on the row of object `row`, at any depth: the container and
/// the index there (counted after it leaves its place), for [`Viewer::move_object_into`]. `None`
/// when the row is no place for it (the row is the dragged object, inside it, or nowhere).
pub fn row_drop_place(
    doc: &Document,
    dragged: Uuid,
    row: Uuid,
    zone: RowZone,
) -> Option<(Place, usize)> {
    doc.locate(dragged)?;
    if row == dragged || doc.ancestors(row).contains(&dragged) {
        return None;
    }
    let target = doc.object(row)?;
    let (place, raw) = match zone {
        RowZone::Inside if target.is_group() => (Place::Group(row), target.children().len()),
        RowZone::Above | RowZone::Inside => {
            let (p, i) = doc.place_of(row)?;
            (p, i + 1)
        }
        RowZone::Below => doc.place_of(row)?,
    };
    // The index counts after the dragged object leaves its place.
    let index = match doc.place_of(dragged) {
        Some((p, di)) if p == place && di < raw => raw - 1,
        _ => raw,
    };
    Some((place, index))
}

/// The index to give [`Viewer::move_layer_to`] for layer `from` dropped on layer `target`'s row,
/// above it (later in the stack) or below it.
pub fn layer_drop_place(from: usize, target: usize, above: bool) -> usize {
    match (above, from < target) {
        (true, true) => target,
        (true, false) => target + 1,
        (false, true) => target - 1,
        (false, false) => target,
    }
}
