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
use quadrule_model::edit::{
    Command, EditError, LayerProps, duplicate, is_empty, remove, reorder_to,
};
use quadrule_model::{Document, Extra, Object, ObjectKind, Uuid};
use quadrule_profiles::{Profile, builtin};
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
            self.selection.retain(|id| doc.find(*id).is_some());
        }
    }

    /// The primary profile and the profile on screen.
    pub fn profiles(&self) -> Option<(Profile, Profile)> {
        let doc = self.doc()?;
        Some((
            builtin(&doc.primary_profile_id).ok()?,
            builtin(&self.profile_id).ok()?,
        ))
    }

    /// How typed lengths read on `axis` (rung 3 bbq Q10).
    pub fn unit_context(&self, axis: Axis) -> Option<UnitContext> {
        let (primary, shown) = self.profiles()?;
        Some(UnitContext::new(self.doc()?, &primary, &shown, axis))
    }

    /// The topmost visible, unlocked object under (x, y) on the device on screen, in device
    /// pixels, within `slack` pixels for thin lines (rung 3 bbq, Defaulted "Selecting").
    pub fn pick(&mut self, x: f64, y: f64, slack: f64) -> Option<Uuid> {
        let placed = self.page()?.placed.clone();
        let doc = self.doc()?;
        placed.iter().rev().find_map(|p| {
            let layer = &doc.layers[p.layer];
            let o = doc.object(p.id)?;
            (layer.visible && !layer.locked && o.visible && !o.locked && p.shape.hit(x, y, slack))
                .then_some(p.id)
        })
    }

    /// A click on the page at (x, y) device pixels: selects what is under it, or with `add`
    /// (Shift) adds or removes it. A click on nothing clears the selection, unless adding.
    pub fn click(&mut self, x: f64, y: f64, slack: f64, add: bool) {
        match (self.pick(x, y, slack), add) {
            (Some(id), true) => self.toggle(id),
            (Some(id), false) => self.selection = vec![id],
            (None, true) => {}
            (None, false) => self.selection.clear(),
        }
    }

    /// Adds `id` to the selection, or takes it out if it is there.
    pub fn toggle(&mut self, id: Uuid) {
        if let Some(i) = self.selection.iter().position(|s| *s == id) {
            self.selection.remove(i);
        } else {
            self.selection.push(id);
        }
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
        self.edit(Command::Batch(cmds), merge)
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
        self.edit_selected(None, |o| {
            o.frame.x_mm += dx * sx;
            o.frame.y_mm += dy * sy;
        })
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
            .filter(|id| doc.find(*id).is_some_and(|(l, _)| !doc.layers[l].locked))
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        let (cmd, copies) = duplicate(doc, &ids, Uuid::new_v4);
        self.edit(cmd, None)?;
        self.selection = copies;
        Ok(())
    }

    /// Moves each selected object one place up or down within its layer (Move up / Move down).
    /// An object already at the end stays.
    pub fn reorder_selection(&mut self, toward: Toward) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let mut places: Vec<(usize, usize, Uuid)> = self
            .editable_selection()
            .iter()
            .filter_map(|id| doc.find(*id).map(|(l, o)| (l, o, *id)))
            .collect();
        // Move the ones nearest the end first, so they make room for the others.
        places.sort_by_key(|&(l, o, _)| (l, o));
        if toward == Toward::Front {
            places.reverse();
        }
        let mut cmds = Vec::new();
        let mut shadow = doc.clone();
        for (l, _, id) in places {
            let (_, o) = shadow.find(id).expect("still there");
            let len = shadow.layers[l].objects.len();
            let to = match toward {
                Toward::Front if o + 1 < len => o + 1,
                Toward::Back if o > 0 => o - 1,
                _ => continue,
            };
            // Never jump over another selected object: they keep their order.
            let neighbour = shadow.layers[l].objects[to].id;
            if self.selection.contains(&neighbour) {
                continue;
            }
            let cmd = Command::MoveObject {
                id,
                layer: l,
                index: to,
            };
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
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let cmd = reorder_to(doc, id, layer, index);
        if is_empty(&cmd) {
            return Ok(());
        }
        self.edit(cmd, None)
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

    fn locked_notice_if_none_editable(&mut self) {
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
        let primary = builtin(&doc.primary_profile_id).map_err(|e| e.to_string())?;
        let mut outlines = Vec::new();
        for id in doc.profile_ids() {
            let target = builtin(id).map_err(|e| e.to_string())?;
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
            Source::Sample => None,
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
