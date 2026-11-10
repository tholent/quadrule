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

//! Editing (napkin §5.6, §15.1; rung 3 bbq, Defaulted "Commands and undo"). Every change to a
//! document is a [`Command`]. Doing a command gives back the command that undoes it, so a
//! [`History`] of them has unlimited undo and redo with no copies of the whole document.
//!
//! Locks (napkin §5.5): a locked object, or any object in a locked layer, refuses edits. The
//! layers panel's own controls (name, eye, lock) still work on it, so it can be unlocked. Undo
//! and redo skip the lock check: they only return the document to a state it was already in.

use crate::{Document, Frame, Layer, Metadata, Object, PageSettings, Rhythm, Unit, Uuid};

/// One change to a document.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Replaces the object with the same id, wherever it is.
    SetObject(Object),
    /// Sets a layer's name, visibility and lock.
    SetLayer {
        /// The layer's index, bottom first.
        index: usize,
        /// The new values.
        props: LayerProps,
    },
    /// Replaces the page settings.
    SetPage(PageSettings),
    /// Replaces the rhythm.
    SetRhythm(Rhythm),
    /// Replaces the metadata.
    SetMetadata(Metadata),
    /// Sets the display unit.
    SetUnits(Unit),
    /// Puts an object into a layer at `index` (0 is the bottom).
    Insert {
        /// The layer's index.
        layer: usize,
        /// Where in the layer.
        index: usize,
        /// The object; its id must be new to the document.
        object: Object,
    },
    /// Takes an object out of the document.
    Remove(Uuid),
    /// Moves an object to `index` in `layer` (counted after it is taken out of its place).
    MoveObject {
        /// The object.
        id: Uuid,
        /// The layer it goes to.
        layer: usize,
        /// Where in that layer.
        index: usize,
    },
    /// Moves a layer from one place in the stack to another.
    MoveLayer {
        /// Its index now.
        from: usize,
        /// Its index afterwards.
        to: usize,
    },
    /// Several commands as one step, done in order and undone in reverse. If one fails, the
    /// ones before it are undone and the document is left as it was.
    Batch(Vec<Command>),
}

/// A layer's own settings: what the layers panel changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerProps {
    /// Shown in the layers panel.
    pub name: String,
    /// Hidden layers are not drawn or exported.
    pub visible: bool,
    /// Locked layers refuse edits to their objects.
    pub locked: bool,
}

impl LayerProps {
    /// A layer's current settings.
    pub fn of(layer: &Layer) -> Self {
        LayerProps {
            name: layer.name.clone(),
            visible: layer.visible,
            locked: layer.locked,
        }
    }
}

/// Why a command was refused. The document is unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// No object has this id.
    NoSuchObject(Uuid),
    /// No layer has this index.
    NoSuchLayer(usize),
    /// The position is past the end of the layer.
    BadIndex(usize),
    /// An object with this id is already in the document.
    DuplicateId(Uuid),
    /// The object, or its layer, is locked.
    Locked(Uuid),
    /// The layer is locked.
    LayerLocked(usize),
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::NoSuchObject(id) => write!(f, "no object {id}"),
            EditError::NoSuchLayer(i) => write!(f, "no layer {i}"),
            EditError::BadIndex(i) => write!(f, "no place {i} in the layer"),
            EditError::DuplicateId(id) => write!(f, "object {id} is already in the document"),
            EditError::Locked(id) => write!(f, "object {id} is locked"),
            EditError::LayerLocked(i) => write!(f, "layer {i} is locked"),
        }
    }
}

impl std::error::Error for EditError {}

impl Document {
    /// Where an object is: (layer index, index in the layer).
    pub fn find(&self, id: Uuid) -> Option<(usize, usize)> {
        self.layers
            .iter()
            .enumerate()
            .find_map(|(li, l)| l.objects.iter().position(|o| o.id == id).map(|oi| (li, oi)))
    }

    /// The object with this id.
    pub fn object(&self, id: Uuid) -> Option<&Object> {
        self.find(id).map(|(l, o)| &self.layers[l].objects[o])
    }

    /// Whether an object can be edited: it exists, and neither it nor its layer is locked.
    pub fn editable(&self, id: Uuid) -> bool {
        self.find(id)
            .is_some_and(|(l, o)| !self.layers[l].locked && !self.layers[l].objects[o].locked)
    }

    /// Does `cmd` and returns the command that undoes it. Refused commands change nothing.
    pub fn apply(&mut self, cmd: Command) -> Result<Command, EditError> {
        self.run(cmd, true)
    }

    fn run(&mut self, cmd: Command, check_locks: bool) -> Result<Command, EditError> {
        match cmd {
            Command::SetObject(new) => {
                let (li, oi) = self.find(new.id).ok_or(EditError::NoSuchObject(new.id))?;
                let old = &self.layers[li].objects[oi];
                let locked = self.layers[li].locked || old.locked;
                if check_locks && locked && !only_panel_fields_differ(old, &new) {
                    return Err(EditError::Locked(new.id));
                }
                let old = std::mem::replace(&mut self.layers[li].objects[oi], new);
                Ok(Command::SetObject(old))
            }
            Command::SetLayer { index, props } => {
                let l = self
                    .layers
                    .get_mut(index)
                    .ok_or(EditError::NoSuchLayer(index))?;
                let old = LayerProps::of(l);
                l.name = props.name;
                l.visible = props.visible;
                l.locked = props.locked;
                Ok(Command::SetLayer { index, props: old })
            }
            Command::SetPage(p) => Ok(Command::SetPage(std::mem::replace(&mut self.page, p))),
            Command::SetRhythm(r) => Ok(Command::SetRhythm(std::mem::replace(&mut self.rhythm, r))),
            Command::SetMetadata(m) => Ok(Command::SetMetadata(std::mem::replace(
                &mut self.metadata,
                m,
            ))),
            Command::SetUnits(u) => Ok(Command::SetUnits(std::mem::replace(&mut self.units, u))),
            Command::Insert {
                layer,
                index,
                object,
            } => {
                let id = object.id;
                if self.find(id).is_some() {
                    return Err(EditError::DuplicateId(id));
                }
                let l = self
                    .layers
                    .get_mut(layer)
                    .ok_or(EditError::NoSuchLayer(layer))?;
                if check_locks && l.locked {
                    return Err(EditError::LayerLocked(layer));
                }
                if index > l.objects.len() {
                    return Err(EditError::BadIndex(index));
                }
                l.objects.insert(index, object);
                Ok(Command::Remove(id))
            }
            Command::Remove(id) => {
                let (li, oi) = self.find(id).ok_or(EditError::NoSuchObject(id))?;
                if check_locks && !self.editable(id) {
                    return Err(EditError::Locked(id));
                }
                let object = self.layers[li].objects.remove(oi);
                Ok(Command::Insert {
                    layer: li,
                    index: oi,
                    object,
                })
            }
            Command::MoveObject { id, layer, index } => {
                let (li, oi) = self.find(id).ok_or(EditError::NoSuchObject(id))?;
                let dest = self
                    .layers
                    .get(layer)
                    .ok_or(EditError::NoSuchLayer(layer))?;
                if check_locks && !self.editable(id) {
                    return Err(EditError::Locked(id));
                }
                if check_locks && dest.locked {
                    return Err(EditError::LayerLocked(layer));
                }
                let room = dest.objects.len() - usize::from(li == layer);
                if index > room {
                    return Err(EditError::BadIndex(index));
                }
                let o = self.layers[li].objects.remove(oi);
                self.layers[layer].objects.insert(index, o);
                Ok(Command::MoveObject {
                    id,
                    layer: li,
                    index: oi,
                })
            }
            Command::MoveLayer { from, to } => {
                let n = self.layers.len();
                if from >= n {
                    return Err(EditError::NoSuchLayer(from));
                }
                if to >= n {
                    return Err(EditError::NoSuchLayer(to));
                }
                let l = self.layers.remove(from);
                self.layers.insert(to, l);
                Ok(Command::MoveLayer { from: to, to: from })
            }
            Command::Batch(cmds) => {
                let mut undo = Vec::with_capacity(cmds.len());
                for c in cmds {
                    match self.run(c, check_locks) {
                        Ok(u) => undo.push(u),
                        Err(e) => {
                            for u in undo.into_iter().rev() {
                                self.run(u, false).expect("undoing a step just done");
                            }
                            return Err(e);
                        }
                    }
                }
                undo.reverse();
                Ok(Command::Batch(undo))
            }
        }
    }
}

/// True when `a` and `b` differ only in what the layers panel sets: name, visibility, lock.
fn only_panel_fields_differ(a: &Object, b: &Object) -> bool {
    let mut a = a.clone();
    a.name.clone_from(&b.name);
    a.visible = b.visible;
    a.locked = b.locked;
    a == *b
}

impl Command {
    /// True for commands that replace a value outright (`Set…`, or a batch of them). Only these
    /// merge, because the first one's undo restores the value from before them all.
    pub fn is_setter(&self) -> bool {
        match self {
            Command::SetObject(_)
            | Command::SetLayer { .. }
            | Command::SetPage(_)
            | Command::SetRhythm(_)
            | Command::SetMetadata(_)
            | Command::SetUnits(_) => true,
            Command::Batch(c) => c.iter().all(Command::is_setter),
            _ => false,
        }
    }
}

/// One undo step.
#[derive(Clone, Debug)]
struct Step {
    /// Unique per change, so "saved" can name the step it was saved at.
    id: u64,
    /// The command that undoes (on the undo stack) or redoes (on the redo stack) this step.
    cmd: Command,
    /// While set, a following command with the same key joins this step.
    merge: Option<String>,
}

/// Unlimited undo and redo for one document (napkin §5.6), and whether it has unsaved changes.
#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
    next_id: u64,
    /// The id of the step on top of the undo stack when the document was last saved (0: none).
    saved_at: u64,
}

impl History {
    /// An empty history for a document that matches its file.
    pub fn new() -> Self {
        History {
            next_id: 1,
            ..Default::default()
        }
    }

    /// Does `cmd` on `doc` as a new undo step, and clears what could be redone.
    ///
    /// With a `merge` key (a text field's id, say), a command that follows one with the same key
    /// joins its step instead of making a new one, so typing in one field undoes in one go. The
    /// step stays open to merging until [`History::seal`] is called (the field loses focus) or
    /// another step is made. Only setter commands merge ([`Command::is_setter`]).
    pub fn apply(
        &mut self,
        doc: &mut Document,
        cmd: Command,
        merge: Option<&str>,
    ) -> Result<(), EditError> {
        let setter = cmd.is_setter();
        let undo = doc.apply(cmd)?;
        let id = self.fresh_id();
        self.redo.clear();
        let joins = setter
            && undo.is_setter()
            && merge.is_some()
            && self
                .undo
                .last()
                .is_some_and(|s| s.merge.as_deref() == merge);
        if joins {
            // Keep the first undo (the value from before the typing); the step is new content,
            // so it gets a new id and a save in between counts as stale.
            self.undo.last_mut().expect("checked above").id = id;
        } else {
            self.undo.push(Step {
                id,
                cmd: undo,
                merge: merge.map(String::from),
            });
        }
        Ok(())
    }

    /// Ends merging into the last step (a field lost focus).
    pub fn seal(&mut self) {
        if let Some(s) = self.undo.last_mut() {
            s.merge = None;
        }
    }

    /// Undoes the last step. Returns false when there is nothing to undo.
    pub fn undo(&mut self, doc: &mut Document) -> bool {
        let Some(step) = self.undo.pop() else {
            return false;
        };
        let redo = doc
            .run(step.cmd, false)
            .expect("an undo step always fits the document it was made on");
        self.redo.push(Step {
            id: step.id,
            cmd: redo,
            merge: None,
        });
        true
    }

    /// Redoes the last undone step. Returns false when there is nothing to redo.
    pub fn redo(&mut self, doc: &mut Document) -> bool {
        let Some(step) = self.redo.pop() else {
            return false;
        };
        let undo = doc
            .run(step.cmd, false)
            .expect("a redo step always fits the document it was undone on");
        self.undo.push(Step {
            id: step.id,
            cmd: undo,
            merge: None,
        });
        true
    }

    /// Whether there is a step to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is a step to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// How many steps can be undone.
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Marks the document as matching its file now.
    pub fn mark_saved(&mut self) {
        self.saved_at = self.top();
    }

    /// Marks the document as differing from any file (a recovered document, say).
    pub fn mark_unsaved(&mut self) {
        self.saved_at = u64::MAX;
    }

    /// Whether the document has changes its file does not have.
    pub fn is_dirty(&self) -> bool {
        self.top() != self.saved_at
    }

    fn top(&self) -> u64 {
        self.undo.last().map_or(0, |s| s.id)
    }

    fn fresh_id(&mut self) -> u64 {
        self.next_id = self.next_id.max(1);
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/// The command that duplicates objects in place, each copy just above its original in the same
/// layer (rung 3 bbq, Defaulted "Delete and Duplicate"), and the copies' ids in the order of
/// `ids`. `new_id` makes each copy's id. Ids not in the document are skipped.
pub fn duplicate(
    doc: &Document,
    ids: &[Uuid],
    mut new_id: impl FnMut() -> Uuid,
) -> (Command, Vec<Uuid>) {
    let mut found: Vec<(usize, usize, usize)> = ids
        .iter()
        .enumerate()
        .filter_map(|(n, &id)| doc.find(id).map(|(l, o)| (l, o, n)))
        .collect();
    let mut copies = vec![Uuid::nil(); ids.len()];
    let mut cmds = Vec::new();
    // Top of the stack first, so each insert leaves the places below it unchanged.
    found.sort_by_key(|&(l, o, _)| std::cmp::Reverse((l, o)));
    for (l, o, n) in found {
        let mut copy = doc.layers[l].objects[o].clone();
        copy.id = new_id();
        copy.locked = false;
        copies[n] = copy.id;
        cmds.push(Command::Insert {
            layer: l,
            index: o + 1,
            object: copy,
        });
    }
    copies.retain(|c| !c.is_nil());
    (Command::Batch(cmds), copies)
}

/// The command that gives objects new frames: a move or a resize, made when the mouse is let go
/// (rung 4 bbq, Defaulted "One drag, one undo step"). Objects that are gone, or that already
/// have the frame, are left out; the rest are one step. It is refused, with nothing changed, if
/// any object it touches is locked.
pub fn set_frames(doc: &Document, frames: &[(Uuid, Frame)]) -> Command {
    Command::Batch(
        frames
            .iter()
            .filter_map(|&(id, frame)| {
                let o = doc.object(id)?;
                (o.frame != frame).then(|| {
                    let mut o = o.clone();
                    o.frame = frame;
                    Command::SetObject(o)
                })
            })
            .collect(),
    )
}

/// Whether a command does nothing (an empty batch).
pub fn is_empty(cmd: &Command) -> bool {
    matches!(cmd, Command::Batch(c) if c.iter().all(is_empty))
}

/// Where a dragged object in the layers panel goes: into `layer` at `index` (counted after it is
/// taken out of its place), as one step. A move to the place it is already in is empty.
pub fn reorder_to(doc: &Document, id: Uuid, layer: usize, index: usize) -> Command {
    match doc.find(id) {
        Some((l, o)) if l == layer && o == index => Command::Batch(Vec::new()),
        _ => Command::MoveObject { id, layer, index },
    }
}

/// The command that removes objects (the Delete key).
pub fn remove(ids: &[Uuid]) -> Command {
    Command::Batch(ids.iter().map(|&id| Command::Remove(id)).collect())
}
