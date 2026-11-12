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

//! Several devices in one document (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md): the
//! edits that need to know how a page falls on a device, with no editor in them. Making a device
//! the primary one, choosing which devices a document targets, and what Ungroup bakes into the
//! children of a group that has an override. Each gives a command (or the pieces of one) for the
//! editor to do as one undo step.

use crate::override_key;
use crate::tree::{Node, resolve_nodes_on};
use quadrule_model::edit::{
    Command, children_box, prune_override, set_font_size_override, set_frame_override,
    set_visible_override, strip_overrides,
};
use quadrule_model::{Document, Layer, Object, ObjectKind, Override, ProfileContext, Uuid};
use quadrule_profiles::{Profile, builtin};
use std::collections::{BTreeMap, HashMap};

/// Why a device change could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceError {
    /// The device is not one of the document's devices.
    NotATarget(String),
    /// The device is not one this build has a profile for.
    Unknown(String),
    /// The primary device cannot be taken out of the document.
    IsPrimary(String),
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceError::NotATarget(id) => write!(f, "{id} is not one of the document's devices"),
            DeviceError::Unknown(id) => write!(f, "no profile {id}"),
            DeviceError::IsPrimary(id) => write!(f, "{id} is the primary device"),
        }
    }
}

impl std::error::Error for DeviceError {}

fn profile(id: &str) -> Result<Profile, DeviceError> {
    builtin(id).map_err(|_| DeviceError::Unknown(id.to_string()))
}

fn index_nodes(nodes: Vec<Node>) -> HashMap<Uuid, Node> {
    nodes.into_iter().map(|n| (n.id, n)).collect()
}

/// What Ungroup bakes into the children of group `group` (rung 8 bbq, Defaulted "Clipboard and
/// duplicate"): on each secondary device, the group's override and the turn of the groups round it
/// moved the children, and they must stay where the device shows them. For every child whose
/// overrides would change, its overrides after Ungroup, by device. Give them to
/// `quadrule_model::edit::ungroup_with`.
pub fn ungroup_overrides(
    doc: &Document,
    primary: &Profile,
    group: Uuid,
) -> BTreeMap<Uuid, BTreeMap<String, Override>> {
    let mut out: BTreeMap<Uuid, BTreeMap<String, Override>> = BTreeMap::new();
    let Some(g) = doc.object(group).filter(|o| o.is_group()) else {
        return out;
    };
    for dev in doc.profile_ids() {
        if dev == primary.id {
            continue;
        }
        let Ok(target) = profile(dev) else {
            continue;
        };
        let nodes = index_nodes(resolve_nodes_on(
            doc,
            primary.page_mm(),
            target.page_mm(),
            override_key(primary, &target),
        ));
        let Some(gn) = nodes.get(&group) else {
            continue;
        };
        for c in g.children() {
            let Some(cn) = nodes.get(&c.id) else {
                continue;
            };
            // The child as the device shows it, put back in the space the group is in.
            let text = matches!(c.kind, ObjectKind::Text(_));
            let (local, rot) = gn.ctx.unplace(cn.frame, cn.angle);
            let unfit = gn.ctx.unfit(local, rot, text);
            let mut baked = c.clone();
            baked.visible = c.visible && g.visible;
            set_frame_override(&mut baked, dev, cn.base, unfit);
            set_visible_override(&mut baked, dev, c.visible_on(dev) && g.visible_on(dev));
            let all = out.entry(c.id).or_insert_with(|| c.overrides.clone());
            match baked.overrides.get(dev) {
                Some(ov) => {
                    all.insert(dev.to_string(), ov.clone());
                }
                None => {
                    all.remove(dev);
                }
            }
        }
    }
    // Only the children whose overrides change.
    out.retain(|id, all| {
        g.children()
            .iter()
            .find(|c| c.id == *id)
            .is_some_and(|c| c.overrides != *all)
    });
    out
}

/// Takes the profile `new_id`'s layout for the stored design, bottom up.
fn rebase(o: &mut Object, new_id: &str, old_id: &str, nodes: &HashMap<Uuid, Node>) {
    if let ObjectKind::Group(g) = &mut o.kind {
        for c in &mut g.children {
            rebase(c, new_id, old_id, nodes);
        }
    }
    o.visible = o.visible_on(new_id);
    if let (ObjectKind::Text(t), Some(pt)) = (
        &mut o.kind,
        o.overrides.get(new_id).and_then(|x| x.font_size_pt),
    ) && pt.is_finite()
        && pt > 0.0
    {
        t.size_pt = pt;
    }
    o.overrides.remove(new_id);
    // A stale override for the old primary was never read; it must not start to be.
    o.overrides.remove(old_id);
    let local = nodes.get(&o.id).map(|n| n.local);
    match &mut o.kind {
        ObjectKind::Group(g) => {
            if let Some(b) = children_box(&g.children).or(local) {
                o.frame = b;
            }
        }
        _ => {
            if let Some(l) = local {
                o.frame = l;
            }
        }
    }
}

/// What a device showed before: per object, the eye, the font size, the frame in its holder's
/// space and its own angle.
struct Before {
    visible: bool,
    font_pt: Option<f64>,
    local: quadrule_model::Frame,
    angle_own: f64,
}

/// Takes the group overrides' place and size out (a group's box is its children's again), so the
/// children's own overrides can carry what the device showed.
fn drop_group_boxes(o: &mut Object, dev: &str) {
    if o.is_group() {
        if let Some(ov) = o.overrides.get_mut(dev) {
            ov.position = None;
            ov.size = None;
        }
        prune_override(o, dev);
    }
    if let ObjectKind::Group(g) = &mut o.kind {
        for c in &mut g.children {
            drop_group_boxes(c, dev);
        }
    }
}

fn walk_mut(objs: &mut [Object], f: &mut impl FnMut(&mut Object)) {
    for o in objs {
        f(o);
        if let ObjectKind::Group(g) = &mut o.kind {
            walk_mut(&mut g.children, f);
        }
    }
}

/// The command that makes the document's device `new_id` its primary one (rung 8 bbq, Defaulted
/// "Switching the primary device"): its layout, with its overrides, becomes the stored design and
/// its overrides are cleared; every other device (the old primary first) gets the overrides that
/// keep what it shows where it is, so no pixel changes on any device. A device's place and size
/// are overridden wherever the new design would put them anywhere but exactly where they were
/// (float noise aside), stricter than half a device pixel, so the pixels cannot move.
///
/// An object in a turned group can still shift on a device whose group box differs, because the
/// group turns about the middle of its box.
pub fn make_primary(doc: &Document, new_id: &str) -> Result<Command, DeviceError> {
    if !doc.profile_ids().contains(&new_id) {
        return Err(DeviceError::NotATarget(new_id.to_string()));
    }
    let old_id = doc.primary_profile_id.clone();
    if new_id == old_id {
        return Err(DeviceError::IsPrimary(new_id.to_string()));
    }
    let old_primary = profile(&old_id)?;
    let new_primary = profile(new_id)?;

    // What each device shows now.
    let mut before: BTreeMap<String, HashMap<Uuid, Before>> = BTreeMap::new();
    for dev in doc.profile_ids() {
        if dev == new_id {
            continue;
        }
        let target = profile(dev)?;
        let key = override_key(&old_primary, &target);
        let nodes = resolve_nodes_on(doc, old_primary.page_mm(), target.page_mm(), key);
        let mut map = HashMap::new();
        for n in nodes {
            let Some(o) = doc.object(n.id) else {
                continue;
            };
            map.insert(
                n.id,
                Before {
                    visible: key.map_or(o.visible, |k| o.visible_on(k)),
                    font_pt: match &o.kind {
                        ObjectKind::Text(t) => Some(
                            key.and_then(|k| o.override_on(k))
                                .and_then(|x| x.font_size_pt)
                                .unwrap_or(t.size_pt),
                        ),
                        _ => None,
                    },
                    local: n.local,
                    angle_own: o.rotation(),
                },
            );
        }
        before.insert(dev.to_string(), map);
    }

    // The new primary's layout becomes the design.
    let q_nodes = index_nodes(resolve_nodes_on(
        doc,
        old_primary.page_mm(),
        new_primary.page_mm(),
        Some(new_id),
    ));
    let mut next = doc.clone();
    next.primary_profile_id = new_id.to_string();
    let mut profiles: Vec<ProfileContext> = Vec::new();
    for want in
        std::iter::once(new_id).chain(doc.profile_ids().into_iter().filter(|p| *p != new_id))
    {
        if let Some(c) = doc.profiles.iter().find(|c| c.profile_id == want) {
            profiles.push(c.clone());
        }
    }
    next.profiles = profiles;
    for l in &mut next.layers {
        for o in &mut l.objects {
            rebase(o, new_id, &old_id, &q_nodes);
        }
    }

    // The other devices keep what they showed.
    for dev in doc.profile_ids() {
        if dev == new_id {
            continue;
        }
        let target = profile(dev)?;
        for l in &mut next.layers {
            for o in &mut l.objects {
                drop_group_boxes(o, dev);
            }
        }
        let nodes = index_nodes(resolve_nodes_on(
            &next,
            new_primary.page_mm(),
            target.page_mm(),
            Some(dev),
        ));
        let Some(was) = before.get(dev) else {
            continue;
        };
        for l in &mut next.layers {
            walk_mut(&mut l.objects, &mut |o| {
                let (Some(n), Some(b)) = (nodes.get(&o.id), was.get(&o.id)) else {
                    return;
                };
                if !o.is_group() {
                    let text = matches!(o.kind, ObjectKind::Text(_));
                    let unfit = n.ctx.unfit(b.local, b.angle_own, text);
                    set_frame_override(o, dev, n.base, unfit);
                }
                if let Some(pt) = b.font_pt {
                    set_font_size_override(o, dev, pt);
                }
                set_visible_override(o, dev, b.visible);
            });
        }
    }

    Ok(Command::SetDesign {
        primary_profile_id: next.primary_profile_id,
        profiles: next.profiles,
        layers: next.layers,
    })
}

/// The command that makes the document target exactly the devices `ids` (rung 8 bbq, Defaulted
/// "Which devices a document targets"): the primary stays first and cannot be left out, a device
/// taken out loses its overrides (the editor asks first, [`Document::has_overrides_on`]), and a
/// new one comes last. Undoing it brings the overrides back.
pub fn set_targets(doc: &Document, ids: &[&str]) -> Result<Command, DeviceError> {
    if !ids.contains(&doc.primary_profile_id.as_str()) {
        return Err(DeviceError::IsPrimary(doc.primary_profile_id.clone()));
    }
    let mut profiles: Vec<ProfileContext> = doc
        .profiles
        .iter()
        .filter(|c| ids.contains(&c.profile_id.as_str()))
        .cloned()
        .collect();
    for id in ids {
        if !profiles.iter().any(|c| c.profile_id == *id) {
            profile(id)?;
            profiles.push(ProfileContext {
                profile_id: (*id).to_string(),
                extra: Default::default(),
            });
        }
    }
    let mut layers: Vec<Layer> = doc.layers.clone();
    for gone in doc.profile_ids().into_iter().filter(|p| !ids.contains(p)) {
        strip_overrides(&mut layers, gone);
    }
    Ok(Command::SetDesign {
        primary_profile_id: doc.primary_profile_id.clone(),
        profiles,
        layers,
    })
}
