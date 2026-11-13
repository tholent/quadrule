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

use crate::geom::{Holder, cos_sin_deg, map_frame, rotate_point};
use crate::{
    Constraints, Document, Frame, GroupShape, Layer, LineDir, Metadata, Object, ObjectKind,
    Override, PageSettings, Position, ProfileContext, Rhythm, Size, Style, Unit, Uuid,
    normalize_deg,
};

/// Where objects are kept: a layer's list, or a group's children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// The layer at this index (0 is the bottom one).
    Layer(usize),
    /// The group with this id.
    Group(Uuid),
}

/// Where an object is in the tree: its layer and the index at each level down to it (the first
/// into the layer's list, the rest into groups' children). Never empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loc {
    /// The layer's index.
    pub layer: usize,
    /// The index at each level; the last is the object's own place in its container.
    pub path: Vec<usize>,
}

/// One change to a document.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Replaces the object with the same id, wherever it is (inside a group too).
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
    /// Replaces the devices and the design together (rung 8 bbq, Defaulted "Switching the primary
    /// device", "Which devices a document targets"): the primary profile, the profiles (primary
    /// first) and every layer, as one step with no refit and no lock check, because it changes
    /// what a device shows, not what a person drew.
    SetDesign {
        /// The primary profile's id.
        primary_profile_id: String,
        /// The target profiles, primary first.
        profiles: Vec<ProfileContext>,
        /// Every layer, with its objects.
        layers: Vec<Layer>,
    },
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
    /// Puts an object into a group's children at `index` (rung 7a).
    InsertIn {
        /// The group.
        parent: Uuid,
        /// Where among its children.
        index: usize,
        /// The object; its id must be new to the document.
        object: Object,
    },
    /// Takes an object out of the document, wherever it is.
    Remove(Uuid),
    /// Moves an object to `index` in `layer` (counted after it is taken out of its place). It may
    /// come out of a group.
    MoveObject {
        /// The object.
        id: Uuid,
        /// The layer it goes to.
        layer: usize,
        /// Where in that layer.
        index: usize,
    },
    /// Moves an object to `index` among a group's children (counted after it is taken out of its
    /// place) (rung 7a).
    MoveObjectIn {
        /// The object.
        id: Uuid,
        /// The group it goes into.
        parent: Uuid,
        /// Where among the children.
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

impl Command {
    /// The command that puts `object` at `index` in `place`.
    pub fn insert_at(place: &Place, index: usize, object: Object) -> Command {
        match place {
            Place::Layer(layer) => Command::Insert {
                layer: *layer,
                index,
                object,
            },
            Place::Group(parent) => Command::InsertIn {
                parent: *parent,
                index,
                object,
            },
        }
    }

    /// The command that moves object `id` to `index` in `place`.
    pub fn move_to(id: Uuid, place: &Place, index: usize) -> Command {
        match place {
            Place::Layer(layer) => Command::MoveObject {
                id,
                layer: *layer,
                index,
            },
            Place::Group(parent) => Command::MoveObjectIn {
                id,
                parent: *parent,
                index,
            },
        }
    }
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
    /// The object, or its layer, or a group around it, is locked.
    Locked(Uuid),
    /// The layer is locked.
    LayerLocked(usize),
    /// The object with this id is not a group.
    NotAGroup(Uuid),
    /// A group cannot go into itself or into something inside it.
    IntoItself(Uuid),
    /// A text box or a pattern region cannot go into a turned group (this one): they do not turn
    /// (rung 7a bbq, Q2).
    TurnedGroup(Uuid),
    /// Objects can be grouped only from one place: the layers, or one group (rung 7a).
    MixedParents,
    /// Fewer than two objects to group.
    TooFew,
    /// Nothing is picked to repeat.
    NothingToRepeat,
    /// The object with this id is not a repeat.
    NotARepeat(Uuid),
    /// The source of a repeat cannot be taken out of it or deleted (this repeat's): Expand the
    /// repeat, or delete the repeat itself.
    SourceStays(Uuid),
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
            EditError::NotAGroup(id) => write!(f, "object {id} is not a group"),
            EditError::IntoItself(id) => write!(f, "group {id} cannot go into itself"),
            EditError::TurnedGroup(id) => {
                write!(f, "group {id} is turned; text and patterns do not turn")
            }
            EditError::MixedParents => write!(f, "the objects are in different groups"),
            EditError::TooFew => write!(f, "a group needs two or more objects"),
            EditError::NothingToRepeat => write!(f, "nothing is picked to repeat"),
            EditError::NotARepeat(id) => write!(f, "object {id} is not a repeat"),
            EditError::SourceStays(id) => {
                write!(f, "the source of repeat {id} cannot be taken out of it")
            }
        }
    }
}

impl std::error::Error for EditError {}

fn locate_in(objs: &[Object], id: Uuid, path: &mut Vec<usize>) -> bool {
    for (i, o) in objs.iter().enumerate() {
        path.push(i);
        if o.id == id {
            return true;
        }
        if locate_in(o.children(), id, path) {
            return true;
        }
        path.pop();
    }
    false
}

fn object_in<'a>(objs: &'a [Object], path: &[usize]) -> Option<&'a Object> {
    let o = objs.get(*path.first()?)?;
    if path.len() == 1 {
        Some(o)
    } else {
        object_in(o.children(), &path[1..])
    }
}

fn object_in_mut<'a>(objs: &'a mut [Object], path: &[usize]) -> Option<&'a mut Object> {
    let o = objs.get_mut(*path.first()?)?;
    if path.len() == 1 {
        Some(o)
    } else {
        object_in_mut(o.children_mut(), &path[1..])
    }
}

/// Adds to `out` the id of `o` if it is a group or a repeat, and of every one inside it: a
/// container set whole is refitted, and so are the ones in it.
fn collect_groups(o: &Object, out: &mut Vec<Uuid>) {
    if o.is_container() {
        out.push(o.id);
    }
    for c in o.children() {
        collect_groups(c, out);
    }
}

/// Whether the object is, or holds at any depth, something that cannot turn.
fn holds_unturnable(o: &Object) -> bool {
    !o.kind.can_rotate()
}

impl Document {
    /// Where an object is in the tree.
    pub fn locate(&self, id: Uuid) -> Option<Loc> {
        for (layer, l) in self.layers.iter().enumerate() {
            let mut path = Vec::new();
            if locate_in(&l.objects, id, &mut path) {
                return Some(Loc { layer, path });
            }
        }
        None
    }

    /// The object at `loc`.
    pub fn at(&self, loc: &Loc) -> Option<&Object> {
        object_in(&self.layers.get(loc.layer)?.objects, &loc.path)
    }

    fn at_mut(&mut self, loc: &Loc) -> Option<&mut Object> {
        object_in_mut(&mut self.layers.get_mut(loc.layer)?.objects, &loc.path)
    }

    /// Where a top-level object is: (layer index, index in the layer). An object inside a group
    /// gives `None`: use [`Document::place_of`] or [`Document::locate`].
    pub fn find(&self, id: Uuid) -> Option<(usize, usize)> {
        self.layers
            .iter()
            .enumerate()
            .find_map(|(li, l)| l.objects.iter().position(|o| o.id == id).map(|oi| (li, oi)))
    }

    /// The object with this id, at any depth.
    pub fn object(&self, id: Uuid) -> Option<&Object> {
        self.at(&self.locate(id)?)
    }

    /// The object with this id, to change in place; the caller keeps the document consistent.
    pub fn object_mut(&mut self, id: Uuid) -> Option<&mut Object> {
        let loc = self.locate(id)?;
        self.at_mut(&loc)
    }

    /// The layer an object is in, at any depth.
    pub fn layer_of(&self, id: Uuid) -> Option<usize> {
        self.locate(id).map(|l| l.layer)
    }

    /// The container an object is in and its index there.
    pub fn place_of(&self, id: Uuid) -> Option<(Place, usize)> {
        let loc = self.locate(id)?;
        let index = *loc.path.last()?;
        if loc.path.len() == 1 {
            return Some((Place::Layer(loc.layer), index));
        }
        let parent = self.at(&Loc {
            layer: loc.layer,
            path: loc.path[..loc.path.len() - 1].to_vec(),
        })?;
        Some((Place::Group(parent.id), index))
    }

    /// The group an object is directly in, if any.
    pub fn parent_of(&self, id: Uuid) -> Option<Uuid> {
        match self.place_of(id)? {
            (Place::Group(g), _) => Some(g),
            _ => None,
        }
    }

    /// The groups round an object, outermost first (none for a top-level object).
    pub fn ancestors(&self, id: Uuid) -> Vec<Uuid> {
        let Some(loc) = self.locate(id) else {
            return Vec::new();
        };
        (1..loc.path.len())
            .filter_map(|n| {
                self.at(&Loc {
                    layer: loc.layer,
                    path: loc.path[..n].to_vec(),
                })
                .map(|o| o.id)
            })
            .collect()
    }

    /// The objects in a container, bottom first.
    pub fn children_of(&self, place: &Place) -> Option<&[Object]> {
        match place {
            Place::Layer(l) => self.layers.get(*l).map(|l| &l.objects[..]),
            Place::Group(g) => self.object(*g).map(Object::children),
        }
    }

    fn children_mut(&mut self, place: &Place) -> Option<&mut Vec<Object>> {
        match place {
            Place::Layer(l) => self.layers.get_mut(*l).map(|l| &mut l.objects),
            Place::Group(g) => match &mut self.object_mut(*g)?.kind {
                ObjectKind::Group(g) => Some(&mut g.children),
                _ => None,
            },
        }
    }

    /// Whether an object is hidden by itself, by a group round it or by its layer (a hidden
    /// group hides what is in it).
    pub fn effectively_hidden(&self, id: Uuid) -> bool {
        let Some(loc) = self.locate(id) else {
            return false;
        };
        !self.layers[loc.layer].visible
            || (1..=loc.path.len()).any(|n| {
                self.at(&Loc {
                    layer: loc.layer,
                    path: loc.path[..n].to_vec(),
                })
                .is_some_and(|o| !o.visible)
            })
    }

    /// Whether an object is locked by itself, by a group round it or by its layer (a locked
    /// group locks what is in it).
    pub fn effectively_locked(&self, id: Uuid) -> bool {
        let Some(loc) = self.locate(id) else {
            return false;
        };
        self.layers[loc.layer].locked
            || (1..=loc.path.len()).any(|n| {
                self.at(&Loc {
                    layer: loc.layer,
                    path: loc.path[..n].to_vec(),
                })
                .is_some_and(|o| o.locked)
            })
    }

    /// Whether an object can be edited: it exists, and neither it, nor a group round it, nor its
    /// layer is locked.
    pub fn editable(&self, id: Uuid) -> bool {
        self.locate(id).is_some() && !self.effectively_locked(id)
    }

    /// Whether `group` or a group round it has a turn: a text box or pattern region cannot go in.
    fn turned_chain(&self, group: Uuid) -> bool {
        self.ancestors(group)
            .into_iter()
            .chain(std::iter::once(group))
            .filter_map(|id| self.object(id))
            .any(|g| normalize_deg(g.rotation_deg) != 0.0)
    }

    /// Does `cmd` and returns the command that undoes it. Refused commands change nothing.
    ///
    /// A group's box is refitted to its children's outer box after an edit inside it (rung 7a bbq,
    /// Defaulted "The group's box"); the undo command restores the boxes too. A group left with no
    /// children goes away.
    pub fn apply(&mut self, cmd: Command) -> Result<Command, EditError> {
        let mut dirty = Vec::new();
        let undo = self.run(cmd, true, &mut dirty)?;
        let mut fixes = self.refit_dirty(dirty);
        if fixes.is_empty() {
            return Ok(undo);
        }
        fixes.reverse();
        fixes.push(undo);
        Ok(Command::Batch(fixes))
    }

    /// Notes that the group `place` is (or its groups round it are) changed inside.
    fn mark_dirty(&self, place: &Place, dirty: &mut Vec<Uuid>) {
        if let Place::Group(g) = place {
            dirty.extend(self.ancestors(*g));
            dirty.push(*g);
        }
    }

    /// Refits the groups named in `dirty`, deepest first, and removes the empty ones; returns the
    /// commands that undo this, in the order they were done.
    fn refit_dirty(&mut self, mut dirty: Vec<Uuid>) -> Vec<Command> {
        dirty.sort();
        dirty.dedup();
        let mut groups: Vec<(usize, Uuid)> = dirty
            .into_iter()
            .filter(|id| self.object(*id).is_some_and(Object::is_container))
            .map(|id| (self.ancestors(id).len(), id))
            .collect();
        groups.sort_by_key(|&(depth, _)| std::cmp::Reverse(depth));
        let mut undo = Vec::new();
        for (_, id) in groups {
            let Some(o) = self.object(id) else {
                continue;
            };
            if o.is_repeat() {
                let steps = self.rhythm.steps_mm();
                let before = o.clone();
                if let Some(o) = self.object_mut(id)
                    && refit_repeat(o, steps)
                {
                    undo.push(Command::SetObject(before));
                }
                continue;
            }
            if o.children().is_empty() {
                if let Some((place, index)) = self.place_of(id)
                    && let Ok((object, ..)) = self.take(id)
                {
                    undo.push(Command::insert_at(&place, index, object));
                }
                continue;
            }
            let before = o.clone();
            if let Some(o) = self.object_mut(id)
                && refit_group(o)
            {
                undo.push(Command::SetObject(before));
            }
        }
        undo
    }

    /// Takes an object out of its container, giving it back with where it was.
    fn take(&mut self, id: Uuid) -> Result<(Object, Place, usize), EditError> {
        let (place, index) = self.place_of(id).ok_or(EditError::NoSuchObject(id))?;
        if let Place::Group(p) = &place
            && self.object(*p).is_some_and(Object::is_repeat)
        {
            return Err(EditError::SourceStays(*p));
        }
        let list = self
            .children_mut(&place)
            .ok_or(EditError::NoSuchObject(id))?;
        Ok((list.remove(index), place, index))
    }

    /// Puts `object` into `place` at `index`, or gives it back with the error.
    fn put(
        &mut self,
        place: &Place,
        index: usize,
        object: Object,
    ) -> Result<(), (EditError, Box<Object>)> {
        let err = match place {
            Place::Layer(l) => EditError::NoSuchLayer(*l),
            Place::Group(g) => EditError::NotAGroup(*g),
        };
        let Some(list) = self.children_mut(place) else {
            return Err((err, Box::new(object)));
        };
        if index > list.len() {
            return Err((EditError::BadIndex(index), Box::new(object)));
        }
        list.insert(index, object);
        Ok(())
    }

    /// The checks every way of putting `object` into `place` shares.
    fn may_put(&self, place: &Place, object: &Object) -> Result<(), EditError> {
        match place {
            Place::Layer(l) => {
                let layer = self.layers.get(*l).ok_or(EditError::NoSuchLayer(*l))?;
                if layer.locked {
                    return Err(EditError::LayerLocked(*l));
                }
            }
            Place::Group(g) => {
                let group = self.object(*g).ok_or(EditError::NoSuchObject(*g))?;
                if !group.is_group() {
                    return Err(EditError::NotAGroup(*g));
                }
                if self.effectively_locked(*g) {
                    return Err(EditError::Locked(*g));
                }
                if *g == object.id || self.ancestors(*g).contains(&object.id) {
                    return Err(EditError::IntoItself(object.id));
                }
                if holds_unturnable(object) && self.turned_chain(*g) {
                    return Err(EditError::TurnedGroup(*g));
                }
            }
        }
        Ok(())
    }

    fn run(
        &mut self,
        cmd: Command,
        check_locks: bool,
        dirty: &mut Vec<Uuid>,
    ) -> Result<Command, EditError> {
        match cmd {
            Command::SetObject(new) => {
                let loc = self.locate(new.id).ok_or(EditError::NoSuchObject(new.id))?;
                let locked = self.effectively_locked(new.id);
                let old = self.at(&loc).expect("just located");
                if check_locks && locked && !only_panel_fields_differ(old, &new) {
                    return Err(EditError::Locked(new.id));
                }
                dirty.extend(self.ancestors(new.id));
                // A group set whole (a resize of its contents) is refitted too, and the groups in it.
                collect_groups(&new, dirty);
                let slot = self.at_mut(&loc).expect("just located");
                let old = std::mem::replace(slot, new);
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
            Command::SetDesign {
                primary_profile_id,
                profiles,
                layers,
            } => Ok(Command::SetDesign {
                primary_profile_id: std::mem::replace(
                    &mut self.primary_profile_id,
                    primary_profile_id,
                ),
                profiles: std::mem::replace(&mut self.profiles, profiles),
                layers: std::mem::replace(&mut self.layers, layers),
            }),
            Command::SetRhythm(r) => {
                // A repeat's Auto step follows the rhythm, so its box is refitted (rung 10 bbq, Q4).
                let repeats: Vec<Uuid> = self
                    .objects()
                    .filter(|o| o.is_repeat())
                    .map(|o| o.id)
                    .collect();
                dirty.extend(repeats);
                Ok(Command::SetRhythm(std::mem::replace(&mut self.rhythm, r)))
            }
            Command::SetMetadata(m) => Ok(Command::SetMetadata(std::mem::replace(
                &mut self.metadata,
                m,
            ))),
            Command::SetUnits(u) => Ok(Command::SetUnits(std::mem::replace(&mut self.units, u))),
            Command::Insert {
                layer,
                index,
                object,
            } => self.run_insert(Place::Layer(layer), index, object, check_locks, dirty),
            Command::InsertIn {
                parent,
                index,
                object,
            } => self.run_insert(Place::Group(parent), index, object, check_locks, dirty),
            Command::Remove(id) => {
                if self.locate(id).is_none() {
                    return Err(EditError::NoSuchObject(id));
                }
                if check_locks && !self.editable(id) {
                    return Err(EditError::Locked(id));
                }
                let (object, place, index) = self.take(id)?;
                self.mark_dirty(&place, dirty);
                Ok(Command::insert_at(&place, index, object))
            }
            Command::MoveObject { id, layer, index } => {
                self.run_move(id, Place::Layer(layer), index, check_locks, dirty)
            }
            Command::MoveObjectIn { id, parent, index } => {
                self.run_move(id, Place::Group(parent), index, check_locks, dirty)
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
                    match self.run(c, check_locks, dirty) {
                        Ok(u) => undo.push(u),
                        Err(e) => {
                            let mut scratch = Vec::new();
                            for u in undo.into_iter().rev() {
                                self.run(u, false, &mut scratch)
                                    .expect("undoing a step just done");
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

    fn run_insert(
        &mut self,
        place: Place,
        index: usize,
        object: Object,
        check_locks: bool,
        dirty: &mut Vec<Uuid>,
    ) -> Result<Command, EditError> {
        let id = object.id;
        // The object and every object inside it must be new.
        for o in std::iter::once(&object).chain(object.children().iter()) {
            if self.locate(o.id).is_some() {
                return Err(EditError::DuplicateId(o.id));
            }
        }
        if check_locks {
            self.may_put(&place, &object)?;
        }
        self.put(&place, index, object).map_err(|(e, _)| e)?;
        self.mark_dirty(&place, dirty);
        Ok(Command::Remove(id))
    }

    fn run_move(
        &mut self,
        id: Uuid,
        dest: Place,
        index: usize,
        check_locks: bool,
        dirty: &mut Vec<Uuid>,
    ) -> Result<Command, EditError> {
        if self.locate(id).is_none() {
            return Err(EditError::NoSuchObject(id));
        }
        if check_locks && !self.editable(id) {
            return Err(EditError::Locked(id));
        }
        if let Place::Group(g) = &dest
            && (*g == id || self.ancestors(*g).contains(&id))
        {
            return Err(EditError::IntoItself(id));
        }
        if check_locks {
            let object = self.object(id).expect("just located");
            self.may_put(&dest, object)?;
        }
        let (object, from, from_index) = self.take(id)?;
        match self.put(&dest, index, object) {
            Ok(()) => {
                self.mark_dirty(&from, dirty);
                self.mark_dirty(&dest, dirty);
                Ok(Command::move_to(id, &from, from_index))
            }
            Err((e, object)) => {
                // Put it back where it was.
                let _ = self.put(&from, from_index, *object);
                Err(e)
            }
        }
    }
}

/// The box round `children` (each child's turned box), as stored; `None` for none.
pub fn children_box(children: &[Object]) -> Option<Frame> {
    let mut boxes = children.iter().map(Object::outer_box);
    let first = boxes.next()?;
    Some(boxes.fold(first, |a, b| a.union(&b)))
}

/// Refits a group's box to its children's outer box (each child's turned box); false if it
/// already is, or there are no children. A turned group's children are moved so the drawing does
/// not shift when the box, and so the centre it turns about, changes.
fn refit_group(g: &mut Object) -> bool {
    let theta = g.rotation();
    let old_frame = g.frame;
    let ObjectKind::Group(shape) = &mut g.kind else {
        return false;
    };
    let Some(mut union) = children_box(&shape.children) else {
        return false;
    };
    let mut shift = (0.0, 0.0);
    if theta != 0.0 {
        // Rendering is `c + R(q - c)` about the box centre c; with the centre moving to the union's
        // centre plus the shift, nothing moves on the page if the children shift by (R - I)(cu - c).
        let (cs, sn) = cos_sin_deg(theta);
        let cu = union.centre();
        let c = old_frame.centre();
        let d = (cu.0 - c.0, cu.1 - c.1);
        shift = ((cs - 1.0) * d.0 - sn * d.1, sn * d.0 + (cs - 1.0) * d.1);
        if shift != (0.0, 0.0) {
            for ch in &mut shape.children {
                ch.translate(shift.0, shift.1);
            }
            union.x_mm += shift.0;
            union.y_mm += shift.1;
        }
    }
    if shift == (0.0, 0.0) && union.near(&old_frame) {
        return false;
    }
    g.frame = union;
    true
}

/// Refits a repeat's box to its source as it stands (rung 10 bbq, Q3 and Defaulted "The source's
/// place"): the box's top-left corner is the source's, and in Count mode its size is the copies'
/// outer box; false if it already is.
fn refit_repeat(o: &mut Object, steps: (f64, f64)) -> bool {
    let Some(f) = o.repeat_fit(steps) else {
        return false;
    };
    if f.near(&o.frame) {
        return false;
    }
    o.frame = f;
    true
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
            .run(step.cmd, false, &mut Vec::new())
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
            .run(step.cmd, false, &mut Vec::new())
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

/// Gives `o` and everything inside it new ids from `new_id`.
pub fn renew_ids(o: &mut Object, new_id: &mut impl FnMut() -> Uuid) {
    o.id = new_id();
    for c in o.children_mut() {
        renew_ids(c, new_id);
    }
}

/// The ids of those of `ids` that are in the document and have no group round them that is also in
/// `ids` (a selected group stands for what is inside it), in document order.
pub(crate) fn outermost(doc: &Document, ids: &[Uuid]) -> Vec<(Loc, Uuid)> {
    let mut found: Vec<(Loc, Uuid)> = ids
        .iter()
        .filter_map(|&id| doc.locate(id).map(|l| (l, id)))
        .collect();
    found.sort_by(|a, b| (a.0.layer, &a.0.path).cmp(&(b.0.layer, &b.0.path)));
    found.dedup_by_key(|f| f.1);
    let all: Vec<Uuid> = found.iter().map(|f| f.1).collect();
    found
        .into_iter()
        .filter(|(_, id)| !doc.ancestors(*id).iter().any(|a| all.contains(a)))
        .collect()
}

/// The command that duplicates objects in place, each copy just above its original in the same
/// place (a layer or a group; rung 3 bbq, Defaulted "Delete and Duplicate"), and the copies' ids
/// in the order of `ids`. A group is copied with everything in it, all with new ids. `new_id`
/// makes each id. Ids not in the document are skipped, and so is an object whose group is also in
/// `ids`.
pub fn duplicate(
    doc: &Document,
    ids: &[Uuid],
    mut new_id: impl FnMut() -> Uuid,
) -> (Command, Vec<Uuid>) {
    let mut found = outermost(doc, ids);
    let mut copies = vec![Uuid::nil(); ids.len()];
    let mut cmds = Vec::new();
    // Top of the stack first, so each insert leaves the places below it unchanged.
    found.reverse();
    for (loc, id) in found {
        let Some((place, index)) = doc.place_of(id) else {
            continue;
        };
        let Some(original) = doc.at(&loc) else {
            continue;
        };
        let mut copy = original.clone();
        renew_ids(&mut copy, &mut new_id);
        copy.locked = false;
        if let Some(n) = ids.iter().position(|&i| i == id) {
            copies[n] = copy.id;
        }
        cmds.push(Command::insert_at(&place, index + 1, copy));
    }
    copies.retain(|c| !c.is_nil());
    (Command::Batch(cmds), copies)
}

/// The command that pastes `objects` (read from the clipboard, [`crate::clipboard::from_text`])
/// on top of `layer`, each with a new id from `new_id` (and every object inside a group too) and
/// unlocked, moved by `offset_mm` (x, y), and the new ids in the order of `objects` (rung 4 bbq,
/// Defaulted "Copy and paste"). They keep their order, so the first is lowest. A layer that is
/// not there gives an empty command.
pub fn paste(
    doc: &Document,
    objects: &[Object],
    layer: usize,
    offset_mm: (f64, f64),
    mut new_id: impl FnMut() -> Uuid,
) -> (Command, Vec<Uuid>) {
    let Some(target) = doc.layers.get(layer) else {
        return (Command::Batch(Vec::new()), Vec::new());
    };
    let mut ids = Vec::with_capacity(objects.len());
    let cmds = objects
        .iter()
        .enumerate()
        .map(|(n, o)| {
            let mut copy = o.clone();
            renew_ids(&mut copy, &mut new_id);
            copy.locked = false;
            copy.translate(offset_mm.0, offset_mm.1);
            translate_overrides(&mut copy, offset_mm.0, offset_mm.1);
            ids.push(copy.id);
            Command::Insert {
                layer,
                index: target.objects.len() + n,
                object: copy,
            }
        })
        .collect();
    (Command::Batch(cmds), ids)
}

/// The command that gives objects new frames: a move or a resize, made when the mouse is let go
/// (rung 4 bbq, Defaulted "One drag, one undo step"). Objects that are gone, or that already
/// have the frame, are left out; the rest are one step. It is refused, with nothing changed, if
/// any object it touches is locked. A group's contents follow it ([`Placement::apply`]).
pub fn set_frames(doc: &Document, frames: &[(Uuid, Frame)]) -> Command {
    Command::Batch(
        frames
            .iter()
            .filter_map(|&(id, frame)| {
                let o = doc.object(id)?;
                (o.frame != frame).then(|| {
                    let mut o = o.clone();
                    reframe(&mut o, frame);
                    Command::SetObject(o)
                })
            })
            .collect(),
    )
}

/// How a group's contents follow when the group's box is moved or stretched: every leaf in it is
/// mapped from the group's old box to its new one, as a loose multiple selection is when it is
/// resized (rung 7a look change, Q3 as amended; [`crate::geom::map_frame`]). The boxes are in the
/// unturned space the group's contents are resolved in on the page `holder` describes, which is
/// the page shown while dragging (the leaves' stored frames go there and back by their own
/// constraints); `None` is the primary page itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupMap {
    /// The page the boxes are measured on, or `None` for the primary page.
    pub holder: Option<Holder>,
    /// The group's box before.
    pub old: Frame,
    /// The group's box after.
    pub new: Frame,
}

/// Moves and stretches everything in the group `o` by `map`, then refits the box of every group
/// among them (and of `o`) round its children, so the boxes stay what they must be. A turned
/// child stretches approximately: it keeps its angle.
pub fn map_contents(o: &mut Object, map: &GroupMap) {
    // The primary page shown on itself resolves nothing: skip it, so nothing picks up float noise.
    let holder = map.holder.filter(|h| h.from != h.to);
    fn walk(o: &mut Object, map: &GroupMap, holder: &Option<Holder>, deg: f64) {
        let ObjectKind::Group(g) = &mut o.kind else {
            return;
        };
        for c in &mut g.children {
            let ang = deg + c.rotation();
            if c.is_group() {
                walk(c, map, holder, ang);
                continue;
            }
            if c.is_repeat() {
                // A repeat in the group moves with it; its box and source are refitted after.
                let at = match holder {
                    Some(h) => h.resolve(c.frame, c.constraints),
                    None => c.frame,
                };
                let to = map_frame(at, 0.0, false, &map.old, &map.new);
                let to = match holder {
                    Some(h) => h.unresolve(to, c.constraints),
                    None => to,
                };
                if !to.near(&c.frame) {
                    reframe(c, to);
                }
                continue;
            }
            let text = matches!(c.kind, ObjectKind::Text(_));
            let at = match holder {
                Some(h) => h.resolve(c.frame, c.constraints),
                None => c.frame,
            };
            let to = map_frame(at, ang, text, &map.old, &map.new);
            let to = match holder {
                Some(h) => h.unresolve(to, c.constraints),
                None => to,
            };
            // Float noise is not a move.
            if !to.near(&c.frame) {
                c.frame = to;
            }
        }
        if let Some(b) = children_box(&g.children)
            && !b.near(&o.frame)
        {
            o.frame = b;
        }
    }
    walk(o, map, &holder, 0.0);
}

/// A move or a resize under this many mm is float noise, not an override.
pub const OVERRIDE_NOISE_MM: f64 = 1e-6;

/// One part of an override, for Reset (rung 8 bbq, Defaulted "The Overrides tab").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverrideField {
    /// The place.
    Position,
    /// The width and height.
    Size,
    /// A text box's font size.
    FontSize,
    /// The eye.
    Visible,
}

impl OverrideField {
    /// Every part, in the order the Overrides tab lists them.
    pub const ALL: [OverrideField; 4] = [
        OverrideField::Position,
        OverrideField::Size,
        OverrideField::FontSize,
        OverrideField::Visible,
    ];

    /// Whether `ov` holds this part.
    pub fn is_set(self, ov: &Override) -> bool {
        match self {
            OverrideField::Position => ov.position.is_some(),
            OverrideField::Size => ov.size.is_some(),
            OverrideField::FontSize => ov.font_size_pt.is_some(),
            OverrideField::Visible => ov.visible.is_some(),
        }
    }
}

/// The override `o` carries on `profile`, made if there is none yet; [`prune_override`] drops it
/// again when it ends up empty.
fn override_mut<'a>(o: &'a mut Object, profile: &str) -> &'a mut Override {
    o.overrides.entry(profile.to_string()).or_default()
}

/// Drops the override on `profile` if nothing is left in it.
pub fn prune_override(o: &mut Object, profile: &str) {
    if o.overrides.get(profile).is_some_and(Override::is_empty) {
        o.overrides.remove(profile);
    }
}

fn differs(a: f64, b: f64) -> bool {
    (a - b).abs() > OVERRIDE_NOISE_MM
}

/// Puts `o` at `to` on device `profile`, where `base` is the box its constraints give there (for
/// a group, the box round its children): the place is an override if it is not `base`'s, and the
/// size too; a part that is back to `base` is no override at all (rung 8 bbq, Defaulted "Position
/// and size are separate overrides"). A move leaves the size alone, because the size it had is
/// `to`'s.
pub fn set_frame_override(o: &mut Object, profile: &str, base: Frame, to: Frame) {
    let ov = override_mut(o, profile);
    ov.position =
        (differs(to.x_mm, base.x_mm) || differs(to.y_mm, base.y_mm)).then_some(Position {
            x_mm: to.x_mm,
            y_mm: to.y_mm,
        });
    ov.size = (differs(to.w_mm, base.w_mm) || differs(to.h_mm, base.h_mm)).then_some(Size {
        w_mm: to.w_mm,
        h_mm: to.h_mm,
    });
    prune_override(o, profile);
}

/// Sets only the place on `profile` (a nudge, an align): the size override, if any, stays.
pub fn set_position_override(o: &mut Object, profile: &str, base: Frame, x_mm: f64, y_mm: f64) {
    let ov = override_mut(o, profile);
    ov.position =
        (differs(x_mm, base.x_mm) || differs(y_mm, base.y_mm)).then_some(Position { x_mm, y_mm });
    prune_override(o, profile);
}

/// Sets a text box's font size on `profile`; the size it already has is no override.
pub fn set_font_size_override(o: &mut Object, profile: &str, pt: f64) {
    let own = match &o.kind {
        ObjectKind::Text(t) => t.size_pt,
        _ => return,
    };
    let ov = override_mut(o, profile);
    ov.font_size_pt = (pt.is_finite() && pt > 0.0 && (pt - own).abs() > 1e-9).then_some(pt);
    prune_override(o, profile);
}

/// Sets whether `o` shows on `profile`; the eye it already has is no override.
pub fn set_visible_override(o: &mut Object, profile: &str, shown: bool) {
    let own = o.visible;
    let ov = override_mut(o, profile);
    ov.visible = (shown != own).then_some(shown);
    prune_override(o, profile);
}

/// Takes the named parts out of `o`'s override on `profile` (Reset).
pub fn reset_override(o: &mut Object, profile: &str, fields: &[OverrideField]) {
    if let Some(ov) = o.overrides.get_mut(profile) {
        for f in fields {
            match f {
                OverrideField::Position => ov.position = None,
                OverrideField::Size => ov.size = None,
                OverrideField::FontSize => ov.font_size_pt = None,
                OverrideField::Visible => ov.visible = None,
            }
        }
    }
    prune_override(o, profile);
}

/// Moves every override place inside `o` (a group's children too) by (`dx`, `dy`) mm: a pasted
/// copy is offset on each device as on the primary.
pub fn translate_overrides(o: &mut Object, dx: f64, dy: f64) {
    for ov in o.overrides.values_mut() {
        if let Some(p) = &mut ov.position {
            p.x_mm += dx;
            p.y_mm += dy;
        }
    }
    for c in o.children_mut() {
        translate_overrides(c, dx, dy);
    }
}

impl Document {
    /// How many objects carry an override on device `profile` (the number on its picture in the
    /// previews strip; rung 8 bbq, Defaulted "Badges").
    pub fn overridden_count(&self, profile: &str) -> usize {
        self.objects()
            .filter(|o| o.override_on(profile).is_some_and(|x| !x.is_empty()))
            .count()
    }

    /// Whether any object carries an override on device `profile`.
    pub fn has_overrides_on(&self, profile: &str) -> bool {
        self.overridden_count(profile) > 0
    }
}

/// Takes every override on device `profile` out of `layers`, groups' children too.
pub fn strip_overrides(layers: &mut [Layer], profile: &str) {
    fn strip(o: &mut Object, profile: &str) {
        o.overrides.remove(profile);
        for c in o.children_mut() {
            strip(c, profile);
        }
    }
    for l in layers {
        for o in &mut l.objects {
            strip(o, profile);
        }
    }
}

/// Gives `o` the frame `new`. A group's contents go with it: they move by the same amount if only
/// its place changes, and are stretched with the box if its size does, as a resize of a loose
/// selection does ([`map_contents`]; rung 7a look change, Q3 as amended). A group's box then is
/// round its children again.
pub fn reframe(o: &mut Object, new: Frame) {
    let old = o.frame;
    if o.is_repeat() {
        // Its source goes with the box's corner; in Fill mode the box takes the new size, in Count
        // mode the size is the copies' (refitted after).
        let (dx, dy) = (new.x_mm - old.x_mm, new.y_mm - old.y_mm);
        if dx != 0.0 || dy != 0.0 {
            for c in o.children_mut() {
                c.translate(dx, dy);
            }
        }
        let fill = o.repeat().is_some_and(|r| r.fill);
        o.frame = if fill {
            new
        } else {
            Frame::new(new.x_mm, new.y_mm, old.w_mm, old.h_mm)
        };
        return;
    }
    if o.is_group() {
        if (old.w_mm - new.w_mm).abs() < 1e-9 && (old.h_mm - new.h_mm).abs() < 1e-9 {
            let (dx, dy) = (new.x_mm - old.x_mm, new.y_mm - old.y_mm);
            if (dx != 0.0 || dy != 0.0)
                && let ObjectKind::Group(g) = &mut o.kind
            {
                for c in &mut g.children {
                    c.translate(dx, dy);
                }
            }
        } else {
            map_contents(
                o,
                &GroupMap {
                    holder: None,
                    old,
                    new,
                },
            );
        }
        if let Some(b) = children_box(o.children()) {
            o.frame = b;
            return;
        }
    }
    o.frame = new;
}

/// A placement made on a secondary device (rung 8 bbq, Q3): the frame is an override there, not
/// the stored design.
#[derive(Clone, Debug, PartialEq)]
pub struct DevicePlace {
    /// The device.
    pub profile: String,
    /// The box the object's constraints give on that device (for a group, the box round its
    /// children): the frame is an override unless it is this box.
    pub base: Frame,
}

/// Where an object is: its frame, its rotation and, for a line, the diagonal it runs along (format
/// 0.6). A drag changes all of them as one step.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    /// The unrotated box, primary mm.
    pub frame: Frame,
    /// Degrees clockwise, 0 up to (not including) 360.
    pub rotation_deg: f64,
    /// A line's diagonal; `None` for any other kind.
    pub dir: Option<LineDir>,
    /// A group dragged on a device other than the primary: how its contents follow (the `frame`
    /// is then only for the group's own bookkeeping). `None` moves and sizes them on the primary
    /// page by `frame`.
    pub map: Option<GroupMap>,
    /// Made on a secondary device: the `frame` is then where the object is there (before the
    /// overrides of the groups round it), written as the device's override, and the stored frame
    /// is left alone. The rotation and a line's diagonal are still the shared design's.
    pub device: Option<DevicePlace>,
}

impl Placement {
    /// Where `o` is now.
    pub fn of(o: &Object) -> Self {
        Placement {
            frame: o.frame,
            rotation_deg: o.rotation(),
            dir: match &o.kind {
                ObjectKind::Line(l) => Some(l.direction),
                _ => None,
            },
            map: None,
            device: None,
        }
    }

    /// A placement with only the frame given (the rotation and direction `o` has).
    pub fn with_frame(o: &Object, frame: Frame) -> Self {
        Placement {
            frame,
            ..Placement::of(o)
        }
    }

    /// The same placement, made on a secondary device.
    pub fn on_device(mut self, profile: &str, base: Frame) -> Self {
        self.device = Some(DevicePlace {
            profile: profile.to_string(),
            base,
        });
        self.map = None;
        self
    }

    /// Puts `o` there. A kind that cannot rotate keeps rotation 0; a direction is for lines only.
    /// A group's contents follow its frame ([`reframe`]).
    pub fn apply(&self, o: &mut Object) {
        match (&self.device, &self.map, o.is_group()) {
            (Some(d), _, _) => set_frame_override(o, &d.profile, d.base, self.frame),
            (None, Some(map), true) => map_contents(o, map),
            _ => reframe(o, self.frame),
        }
        if o.kind.can_rotate() {
            o.rotation_deg = normalize_deg(self.rotation_deg);
        }
        if let (ObjectKind::Line(l), Some(d)) = (&mut o.kind, self.dir) {
            l.direction = d;
        }
    }
}

/// The command that gives objects new placements: a move, a resize or a rotation, made when the
/// mouse is let go. Objects that are gone, or already there, are left out; the rest are one step.
/// Refused, with nothing changed, if any object it touches is locked.
pub fn set_placements(doc: &Document, places: &[(Uuid, Placement)]) -> Command {
    Command::Batch(
        places
            .iter()
            .filter_map(|(id, place)| {
                let o = doc.object(*id)?;
                let mut n = o.clone();
                place.apply(&mut n);
                (n != *o).then_some(Command::SetObject(n))
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
    reorder_into(doc, id, &Place::Layer(layer), index)
}

/// The same for any container: a layer, or a group's children (rung 7a).
pub fn reorder_into(doc: &Document, id: Uuid, place: &Place, index: usize) -> Command {
    match doc.place_of(id) {
        Some((p, i)) if p == *place && i == index => Command::Batch(Vec::new()),
        _ => Command::move_to(id, place, index),
    }
}

/// The command that removes objects (the Delete key). Ids are objects at any depth.
pub fn remove(ids: &[Uuid]) -> Command {
    Command::Batch(ids.iter().map(|&id| Command::Remove(id)).collect())
}

/// The command that groups `ids` into one new group `new_id` called `name` (Ctrl+G; rung 7a bbq,
/// Defaulted "Making and breaking"). The group goes in the place, on the layer, of the topmost
/// object, and the objects (from any layer) move into it keeping their stacking order; the
/// group's box is refitted round them when the command is done. Objects must share one container
/// kind: all in layers, or all in one group. An object whose group is also in `ids` is left to
/// that group.
pub fn group(doc: &Document, ids: &[Uuid], new_id: Uuid, name: &str) -> Result<Command, EditError> {
    let found = outermost(doc, ids);
    if found.len() < 2 {
        return Err(EditError::TooFew);
    }
    let parents: Vec<Option<Uuid>> = found.iter().map(|(_, id)| doc.parent_of(*id)).collect();
    if parents.iter().any(|p| *p != parents[0]) {
        return Err(EditError::MixedParents);
    }
    let (_, top) = found.last().expect("two or more");
    let (place, index) = doc.place_of(*top).ok_or(EditError::NoSuchObject(*top))?;
    let boxes = found
        .iter()
        .filter_map(|(l, _)| doc.at(l).map(Object::outer_box));
    let frame = boxes
        .reduce(|a, b| a.union(&b))
        .unwrap_or(Frame::new(0.0, 0.0, 0.0, 0.0));
    let group = Object {
        id: new_id,
        name: name.into(),
        frame,
        constraints: Constraints::default(),
        visible: true,
        locked: false,
        style: Style::None,
        fill: None,
        dash: crate::DashStyle::Solid,
        cap: crate::LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: crate::OPACITY_MAX,
        overrides: Default::default(),
        kind: ObjectKind::Group(GroupShape::default()),
        extra: crate::Extra::new(),
    };
    let mut cmds = vec![Command::insert_at(&place, index + 1, group)];
    for (n, (_, id)) in found.iter().enumerate() {
        cmds.push(Command::MoveObjectIn {
            id: *id,
            parent: new_id,
            index: n,
        });
    }
    Ok(Command::Batch(cmds))
}

/// A plain object to build a group or a repeat on.
fn container_object(id: Uuid, name: &str, frame: Frame, kind: ObjectKind) -> Object {
    Object {
        id,
        name: name.into(),
        frame,
        constraints: Constraints::default(),
        visible: true,
        locked: false,
        style: Style::None,
        fill: None,
        dash: crate::DashStyle::Solid,
        cap: crate::LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: crate::OPACITY_MAX,
        overrides: Default::default(),
        kind,
        extra: crate::Extra::new(),
    }
}

/// The command that puts `with` where the object `id` is: in the same place of its layer or group,
/// or, when `id` is the source of a repeat, as that repeat's new source. The old object goes (its
/// ids may be used again inside `with`).
pub fn replace_object(doc: &Document, id: Uuid, with: Object) -> Result<Command, EditError> {
    let (place, index) = doc.place_of(id).ok_or(EditError::NoSuchObject(id))?;
    if let Place::Group(p) = &place
        && let Some(parent) = doc.object(*p)
        && parent.is_repeat()
    {
        let mut np = parent.clone();
        if let Some(r) = np.repeat_mut() {
            *r.source = with;
        }
        return Ok(Command::SetObject(np));
    }
    Ok(Command::Batch(vec![
        Command::Remove(id),
        Command::insert_at(&place, index, with),
    ]))
}

/// The command that turns the picked objects into a repeat `repeat_id` called `name` (rung 10 bbq,
/// Defaulted "Making a repeat"): one object becomes the source; several are grouped first, into
/// group `group_id` called `group_name`, and the group is the source. The repeat is across, a
/// count of 4, Auto pitch, from the start, and goes where the source was. One step.
pub fn make_repeat(
    doc: &Document,
    ids: &[Uuid],
    repeat_id: Uuid,
    group_id: Uuid,
    name: &str,
    group_name: &str,
) -> Result<Command, EditError> {
    let found = outermost(doc, ids);
    let mut cmds = Vec::new();
    let (source_id, source) = match found.len() {
        0 => return Err(EditError::NothingToRepeat),
        1 => {
            let (loc, id) = &found[0];
            (*id, doc.at(loc).expect("just located").clone())
        }
        _ => {
            let g = group(doc, ids, group_id, group_name)?;
            let mut scratch = doc.clone();
            scratch.apply(g.clone())?;
            let made = scratch
                .object(group_id)
                .ok_or(EditError::NoSuchObject(group_id))?
                .clone();
            cmds.push(g);
            (group_id, made)
        }
    };
    let mut repeat = container_object(
        repeat_id,
        name,
        source.outer_box(),
        ObjectKind::Repeat(crate::RepeatShape::new(source)),
    );
    if let Some(f) = repeat.repeat_fit(doc.rhythm.steps_mm()) {
        repeat.frame = f;
    }
    // The grouping is part of this command: the group is replaced by the repeat that holds it.
    if cmds.is_empty() {
        cmds.push(replace_object(doc, source_id, repeat)?);
    } else {
        let mut scratch = doc.clone();
        scratch.apply(cmds[0].clone())?;
        cmds.push(replace_object(&scratch, source_id, repeat)?);
    }
    Ok(Command::Batch(cmds))
}

/// The command that dissolves group `id` (Ctrl+Shift+G): its children go where the group was,
/// in order, with the group's turn baked in (each child's angle plus the group's, its centre
/// moved about the group's), the group's opacity multiplied into theirs, and a hidden group
/// leaving hidden children. They keep their own constraints, now against whatever holds the
/// group (rung 7a bbq, Defaulted "Making and breaking").
pub fn ungroup(doc: &Document, id: Uuid) -> Result<Command, EditError> {
    ungroup_with(doc, id, &std::collections::BTreeMap::new())
}

/// [`ungroup`] with the overrides each child gets, by child id and device: what the group's own
/// override on each device did to the child, baked in (rung 8 bbq, Defaulted "Clipboard and
/// duplicate"; worked out by `quadrule_layout::device::ungroup_overrides`). A child not named
/// keeps the overrides it has.
pub fn ungroup_with(
    doc: &Document,
    id: Uuid,
    baked_overrides: &std::collections::BTreeMap<
        Uuid,
        std::collections::BTreeMap<String, Override>,
    >,
) -> Result<Command, EditError> {
    let g = doc.object(id).ok_or(EditError::NoSuchObject(id))?;
    if !g.is_group() {
        return Err(EditError::NotAGroup(id));
    }
    let (place, index) = doc.place_of(id).ok_or(EditError::NoSuchObject(id))?;
    let theta = g.rotation();
    let centre = g.frame.centre();
    let mut cmds = Vec::new();
    for c in g.children() {
        let mut baked = c.clone();
        if theta != 0.0 {
            let moved = rotate_point(c.frame.centre(), centre, theta);
            let old = c.frame.centre();
            baked.translate(moved.0 - old.0, moved.1 - old.1);
            if baked.kind.can_rotate() {
                baked.rotation_deg = normalize_deg(c.rotation_deg + theta);
            }
        }
        baked.opacity_pct = ((u32::from(c.opacity()) * u32::from(g.opacity()) + 50) / 100) as u8;
        baked.visible = c.visible && g.visible;
        baked.locked = c.locked || g.locked;
        if let Some(ov) = baked_overrides.get(&c.id) {
            baked.overrides = ov.clone();
        }
        cmds.push(Command::SetObject(baked));
    }
    for (k, c) in g.children().iter().enumerate() {
        cmds.push(Command::move_to(c.id, &place, index + k));
    }
    cmds.push(Command::Remove(id));
    Ok(Command::Batch(cmds))
}
