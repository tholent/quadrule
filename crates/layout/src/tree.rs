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
//! Q3 as amended at the look): groups are see-through to constraints, as in Figma. Every object
//! that is not a group resolves against the page by its own constraints, exactly as if it were not
//! in a group; a group has no constraints, and its box is its children's outer box (each child's
//! turned box) on every device; then the group's angle turns its resolved children about the
//! centre of that box. Everything that needs to know where an object falls (drawing, picking,
//! handles, snapping, lint) takes it from here.
//!
//! Positions are in millimetres on the profile's page; the angles add up down the tree.
//!
//! Overrides (rung 8 bbq, Q3 and Q4): on a secondary device an object may carry its own place and
//! size, which replace what its constraints give. A group's override is its box; what is in the
//! group is laid out as usual, and the box round it is then moved and stretched onto the override
//! box ([`Fit`]). The functions here that take `ov` read the overrides of that device; `None` lays
//! out the stored design (the primary profile, where no override is ever read).

use crate::rotate::{cos_sin_deg, rotate_point};
use quadrule_model::geom::{Fit, Holder};
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

/// What an object is laid out in: the page it resolves against, and where the unturned space it
/// is in sits on the page (the turns of the groups round it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ctx {
    /// Stored page → page shown. Always the page: a group does not hold its children against its
    /// own box.
    pub holder: Holder,
    /// From the unturned space the object is resolved in to the page.
    pub xf: Rigid,
    /// What the overrides of the groups round the object do to it, before the turns: the box round
    /// each group's children onto the group's override box (identity when no group has one).
    pub fit: Fit,
}

impl Ctx {
    /// The page itself.
    pub fn page(from: (f64, f64), to: (f64, f64)) -> Self {
        Ctx {
            holder: Holder::page(from, to),
            xf: Rigid::IDENTITY,
            fit: Fit::IDENTITY,
        }
    }

    /// The frame `unfit` (laid out by constraints and its own override) after the overrides of the
    /// groups round it, `deg` being the angle it is turned by itself (`text`: a text box, which
    /// keeps its height).
    pub fn fitted(&self, unfit: Frame, deg: f64, text: bool) -> Frame {
        self.fit.apply(unfit, deg, text)
    }

    /// The reverse of [`Ctx::fitted`]: the frame, before the groups' overrides, that ends up as
    /// `fitted`. A group squashed to nothing on an axis cannot be undone: the frame is then taken
    /// as it is.
    pub fn unfit(&self, fitted: Frame, deg: f64, text: bool) -> Frame {
        match self.fit.inverse() {
            Some(inv) => inv.apply(fitted, deg, text),
            None => fitted,
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

    /// The context for the children of a group whose box (their outer box, see [`group_box`]) is
    /// `local` (in this context's space), turned by `rot`: the same page, one more turn.
    pub fn child(&self, local: Frame, rot: f64) -> Ctx {
        Ctx {
            holder: self.holder,
            xf: self.xf.then_turn(local.centre(), rot),
            fit: self.fit,
        }
    }

    /// The same context with one more group's override map applied to what is in it first.
    pub fn with_fit(&self, group_fit: Fit) -> Ctx {
        Ctx {
            fit: group_fit.then(self.fit),
            ..*self
        }
    }
}

/// Where `o` falls in the unturned space it is in, resolved on the page `holder` describes: a
/// shape, line, text box or pattern region by its own constraints against the page; a group as the
/// box round its children's turned boxes (so it is the same on every device, whatever the
/// constraints of the group itself, which are ignored). A group with nothing in it keeps its
/// stored box. This is the stored design: no override is read.
pub fn local_frame(o: &Object, holder: &Holder) -> Frame {
    unfit_frame(o, holder, None).1
}

/// Where `o` falls before the overrides of the groups round it, on the page `holder` describes,
/// laid out for device `ov` (`None`: the stored design): the box its constraints (or, for a group,
/// its children) give, and that box with the object's own place and size override on it. The
/// second is where the object is in the space of the group it is in, before that group's own
/// override moves it.
pub fn unfit_frame(o: &Object, holder: &Holder, ov: Option<&str>) -> (Frame, Frame) {
    let base = match &o.kind {
        ObjectKind::Group(g) => group_box(&g.children, holder, ov).unwrap_or(o.frame),
        _ => holder.resolve(o.frame, o.constraints),
    };
    let own = ov
        .and_then(|p| o.override_on(p))
        .map_or(base, |x| x.apply_to(base));
    (base, own)
}

/// The box round `children` as resolved on `holder`'s page for device `ov`, each as turned by its
/// own angle; `None` for no children.
pub fn group_box(children: &[Object], holder: &Holder, ov: Option<&str>) -> Option<Frame> {
    children
        .iter()
        .map(|c| c.outer_box_of(unfit_frame(c, holder, ov).1))
        .reduce(|a, b| a.union(&b))
}

/// An object placed in its context: the stages between its constraints and the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placing {
    /// What its constraints give (for a group, the box round its children): no override read.
    pub base: Frame,
    /// `base` with the object's own place and size override.
    pub unfit: Frame,
    /// `unfit` after the overrides of the groups round it: its frame in its holder's space.
    pub local: Frame,
    /// Its unturned box on the page.
    pub frame: Frame,
    /// Its angle on the page.
    pub angle: f64,
    /// For a group: what its children are laid out in.
    pub inner: Option<Ctx>,
}

/// Lays `o` out in `ctx` for device `ov`.
pub fn place_object(o: &Object, ctx: &Ctx, ov: Option<&str>) -> Placing {
    let (base, unfit) = unfit_frame(o, &ctx.holder, ov);
    let rot = o.rotation();
    let text = matches!(o.kind, ObjectKind::Text(_));
    let local = ctx.fitted(unfit, rot, text);
    let (frame, angle) = ctx.place(local, rot);
    // A group's contents are mapped from the box round them onto its override box first, then
    // through the groups round it.
    let inner = o
        .is_group()
        .then(|| ctx.child(local, rot).with_fit(Fit::between(&base, &unfit)));
    Placing {
        base,
        unfit,
        local,
        frame,
        angle,
        inner,
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
    /// Its frame resolved in its holder's space (before the holder's turns), overrides included.
    pub local: Frame,
    /// What its constraints give (for a group, the box round its children), before any override.
    pub base: Frame,
    /// `base` with the object's own place and size override, before the overrides of the groups
    /// round it move it.
    pub unfit: Frame,
    /// Whether it carries a place or size override on this device.
    pub framed: bool,
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
    resolve_nodes_on(doc, from, to, None)
}

/// [`resolve_nodes`] for device `ov`: its objects' overrides are read (`None` is the stored design,
/// which is what the primary profile shows).
pub fn resolve_nodes_on(
    doc: &Document,
    from: (f64, f64),
    to: (f64, f64),
    ov: Option<&str>,
) -> Vec<Node> {
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
            ov,
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
    ov: Option<&str>,
    out: &mut Vec<Node>,
) {
    for o in objs {
        let Placing {
            base,
            unfit,
            local,
            frame,
            angle,
            inner,
        } = place_object(o, ctx, ov);
        // A kind that cannot turn is not turned by a group round it either (a hand-made file).
        let angle = if o.kind.can_rotate() { angle } else { 0.0 };
        let visible = ov.map_or(o.visible, |p| o.visible_on(p));
        let (hidden, locked) = (hidden || !visible, locked || o.locked);
        let framed = ov
            .and_then(|p| o.override_on(p))
            .is_some_and(quadrule_model::Override::has_frame);
        out.push(Node {
            id: o.id,
            layer,
            parent,
            depth,
            group: o.is_group(),
            constraints: o.constraints,
            local,
            base,
            unfit,
            framed,
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
                ov,
                out,
            );
        }
    }
}

/// The turn of `p` about `c` by `deg`, for callers that hold only this module.
pub fn turn(p: (f64, f64), c: (f64, f64), deg: f64) -> (f64, f64) {
    rotate_point(p, c, deg)
}
