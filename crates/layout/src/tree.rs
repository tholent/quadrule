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

//! The tree of objects laid out on one profile's page (rung 7a, docs/bbq/2026-11-11-rung-7-groups-and-align.md,
//! Defaulted "Layout (7a)"): a group's frame resolves against the page by its own constraints; its
//! children resolve against the group's stored box → the group's resolved box, recursively; then
//! the group's angle turns them all about the group's centre. Everything that needs to know where
//! an object falls (drawing, picking, handles, snapping, lint) takes it from here.
//!
//! Positions are in millimetres on the profile's page; the angles add up down the tree.

use crate::rotate::{cos_sin_deg, rotate_point};
use quadrule_model::geom::Holder;
use quadrule_model::{Constraints, Document, Frame, Object, ObjectKind, Uuid, normalize_deg};

/// A rigid move of the page: a turn of `deg` degrees clockwise, then a shift by `t` (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rigid {
    /// The turn, 0 up to (not including) 360.
    pub deg: f64,
    /// The shift after the turn.
    pub t: (f64, f64),
}

impl Rigid {
    /// No move at all.
    pub const IDENTITY: Rigid = Rigid {
        deg: 0.0,
        t: (0.0, 0.0),
    };

    /// Whether this moves nothing.
    pub fn is_identity(&self) -> bool {
        self.deg == 0.0 && self.t == (0.0, 0.0)
    }

    /// Where `p` goes.
    pub fn apply(&self, p: (f64, f64)) -> (f64, f64) {
        let (cs, sn) = cos_sin_deg(self.deg);
        (
            p.0 * cs - p.1 * sn + self.t.0,
            p.0 * sn + p.1 * cs + self.t.1,
        )
    }

    /// The point that goes to `p`.
    pub fn unapply(&self, p: (f64, f64)) -> (f64, f64) {
        let (cs, sn) = cos_sin_deg(self.deg);
        let (x, y) = (p.0 - self.t.0, p.1 - self.t.1);
        (x * cs + y * sn, -x * sn + y * cs)
    }

    /// This move after a turn of `deg` about `c`: what a group's children go through when the
    /// group (which sits in this move's space) turns.
    pub fn then_turn(&self, c: (f64, f64), deg: f64) -> Rigid {
        if deg == 0.0 {
            return *self;
        }
        let (cs, sn) = cos_sin_deg(deg);
        // The shift of the turn about c: c - R(deg) c, then carried through this move's turn.
        let q = (c.0 - (c.0 * cs - c.1 * sn), c.1 - (c.0 * sn + c.1 * cs));
        let (ds, dn) = cos_sin_deg(self.deg);
        Rigid {
            deg: normalize_deg(self.deg + deg),
            t: (
                q.0 * ds - q.1 * dn + self.t.0,
                q.0 * dn + q.1 * ds + self.t.1,
            ),
        }
    }
}

/// What a child is laid out in: the box it was designed against and the box it falls in (the
/// page, or a group), and where that box's space sits on the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ctx {
    /// Stored box → resolved box.
    pub holder: Holder,
    /// From the holder's unturned space to the page.
    pub xf: Rigid,
}

impl Ctx {
    /// The page itself.
    pub fn page(from: (f64, f64), to: (f64, f64)) -> Self {
        Ctx {
            holder: Holder::page(from, to),
            xf: Rigid::IDENTITY,
        }
    }

    /// The box and angle a child ends up with on the page, from its resolved frame `local` in the
    /// holder's space and its own angle `rot`: the centre goes where the holder's turns put it, and
    /// the angles add. In an unturned holder nothing changes.
    pub fn place(&self, local: Frame, rot: f64) -> (Frame, f64) {
        if self.xf.is_identity() {
            return (local, rot);
        }
        let c = self.xf.apply(local.centre());
        (local.centred_on(c), normalize_deg(rot + self.xf.deg))
    }

    /// The reverse of [`Ctx::place`]: the frame (resolved, in the holder's space) and own angle of
    /// a child that is on the page as `placed` at the total angle `total`.
    pub fn unplace(&self, placed: Frame, total: f64) -> (Frame, f64) {
        if self.xf.is_identity() {
            return (placed, total);
        }
        let c = self.xf.unapply(placed.centre());
        (placed.centred_on(c), normalize_deg(total - self.xf.deg))
    }

    /// The context for the children of a group whose resolved frame is `local` (in this
    /// context's space), stored as `stored`, turned by `rot`.
    pub fn child(&self, local: Frame, stored: Frame, rot: f64) -> Ctx {
        Ctx {
            holder: Holder {
                from: stored,
                to: local,
            },
            xf: self.xf.then_turn(local.centre(), rot),
        }
    }
}

/// One object laid out: where it falls on the page, and what it sits in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    /// The object.
    pub id: Uuid,
    /// Its layer's index, bottom first.
    pub layer: usize,
    /// The group it is directly in.
    pub parent: Option<Uuid>,
    /// How many groups it is inside.
    pub depth: usize,
    /// Whether it is a group.
    pub group: bool,
    /// Its constraints.
    pub constraints: Constraints,
    /// Its frame resolved in its holder's space (before the holder's turns).
    pub local: Frame,
    /// Its frame on the page: the unturned box, centred where the turns put it.
    pub frame: Frame,
    /// Its angle on the page: its own plus every group's round it.
    pub angle: f64,
    /// What it was resolved in.
    pub ctx: Ctx,
    /// For a group: what its children are resolved in.
    pub inner: Option<Ctx>,
    /// Hidden by itself, a group round it or its layer.
    pub hidden: bool,
    /// Locked by itself, a group round it or its layer.
    pub locked: bool,
}

/// Lays out every object of the document, groups and what is in them, in drawing order (a group
/// comes before its children), on a page of `to` mm for a design made on a page of `from` mm.
/// Hidden and locked objects are included, flagged.
pub fn resolve_nodes(doc: &Document, from: (f64, f64), to: (f64, f64)) -> Vec<Node> {
    let mut out = Vec::new();
    for (li, layer) in doc.layers.iter().enumerate() {
        walk(
            &layer.objects,
            li,
            None,
            0,
            &Ctx::page(from, to),
            !layer.visible,
            layer.locked,
            &mut out,
        );
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn walk(
    objs: &[Object],
    layer: usize,
    parent: Option<Uuid>,
    depth: usize,
    ctx: &Ctx,
    hidden: bool,
    locked: bool,
    out: &mut Vec<Node>,
) {
    for o in objs {
        let local = ctx.holder.resolve(o.frame, o.constraints);
        let rot = o.rotation();
        let (frame, angle) = ctx.place(local, rot);
        // A kind that cannot turn is not turned by a group round it either (a hand-made file).
        let angle = if o.kind.can_rotate() { angle } else { 0.0 };
        let (hidden, locked) = (hidden || !o.visible, locked || o.locked);
        let inner = o.is_group().then(|| ctx.child(local, o.frame, rot));
        out.push(Node {
            id: o.id,
            layer,
            parent,
            depth,
            group: o.is_group(),
            constraints: o.constraints,
            local,
            frame,
            angle,
            ctx: *ctx,
            inner,
            hidden,
            locked,
        });
        if let (ObjectKind::Group(g), Some(inner)) = (&o.kind, &inner) {
            walk(
                &g.children,
                layer,
                Some(o.id),
                depth + 1,
                inner,
                hidden,
                locked,
                out,
            );
        }
    }
}

/// The turn of `p` about `c` by `deg`, for callers that hold only this module.
pub fn turn(p: (f64, f64), c: (f64, f64), deg: f64) -> (f64, f64) {
    rotate_point(p, c, deg)
}
