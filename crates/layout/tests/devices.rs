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

//! Per-device overrides in layout (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md, Q3 and
//! Q4): a place, a size, a font size and the eye on one secondary device replace what the
//! constraints give there, and only there; a group's override is its box, and what is in it is
//! scaled into that box; the primary device never reads an override. Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::device::{make_primary, set_targets, ungroup_overrides};
use quadrule_layout::tree::{Node, resolve_nodes_on};
use quadrule_layout::*;
use quadrule_model::edit::{
    OverrideField, reset_override, set_frame_override, set_position_override, ungroup_with,
};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn empty_outlines(id: &str) -> ProfileOutlines {
    ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn nodes(doc: &Document, id: &str) -> Vec<Node> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    resolve_nodes_on(
        doc,
        primary.page_mm(),
        target.page_mm(),
        override_key(&primary, &target),
    )
}

fn node(doc: &Document, dev: &str, id: Uuid) -> Node {
    *nodes(doc, dev).iter().find(|n| n.id == id).unwrap()
}

fn list(doc: &Document, id: &str) -> DisplayList {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    display_list(doc, &primary, &target, &empty_outlines(id)).unwrap()
}

/// The overrides page with no text (the display list then needs no fonts).
fn page() -> Document {
    let mut doc = devices_document();
    for l in &mut doc.layers {
        l.objects.retain(|o| !matches!(o.kind, ObjectKind::Text(_)));
    }
    doc
}

fn bare(doc: &Document) -> Document {
    let mut d = doc.clone();
    quadrule_model::edit::strip_overrides(&mut d.layers, NOMAD_ID);
    d
}

#[test]
fn an_override_replaces_the_constraints_on_its_device_only() {
    let doc = page();
    let plain = bare(&doc);
    let moved = device_ids::MOVED;
    let sized = device_ids::SIZED;
    // The Nomad: the moved box is at its override position and keeps the size it scales to; the
    // sized ellipse takes position and size.
    let (a, b) = (node(&doc, NOMAD_ID, moved), node(&plain, NOMAD_ID, moved));
    let position_only = near(a.frame.x_mm, 20.0)
        && near(a.frame.y_mm, 62.0)
        && near(a.frame.w_mm, b.frame.w_mm)
        && near(a.frame.h_mm, b.frame.h_mm)
        && a.framed;
    let s = node(&doc, NOMAD_ID, sized);
    let both = near(s.frame.x_mm, 56.0)
        && near(s.frame.y_mm, 24.0)
        && near(s.frame.w_mm, 36.0)
        && near(s.frame.h_mm, 26.0);
    // The Manta, the primary device, never reads an override: the nodes are those of the bare page.
    let manta_same = nodes(&doc, MANTA_ID) == nodes(&plain, MANTA_ID);
    let manta_png_list = list(&doc, MANTA_ID) == list(&plain, MANTA_ID);
    // An override keyed to the primary profile is never read, even if a file holds one.
    let mut odd = plain.clone();
    odd.object_mut(moved).unwrap().overrides.insert(
        MANTA_ID.into(),
        Override {
            position: Some(Position {
                x_mm: 1.0,
                y_mm: 1.0,
            }),
            visible: Some(false),
            ..Override::default()
        },
    );
    let primary_ignored = list(&odd, MANTA_ID) == list(&plain, MANTA_ID);
    // An override for a device the document does not target is kept and unused.
    let mut stray = plain.clone();
    stray.object_mut(moved).unwrap().overrides.insert(
        "supernote.elsewhere".into(),
        Override {
            visible: Some(false),
            ..Override::default()
        },
    );
    let stray_unused =
        list(&stray, NOMAD_ID) == list(&plain, NOMAD_ID) && stray.objects().count() > 0;
    let ok =
        position_only && both && manta_same && manta_png_list && primary_ignored && stray_unused;
    println!(
        "\nCHECK crate=layout name=override_replaces_constraints_on_its_device result={} position_only={position_only} position_and_size={both} manta_nodes_unchanged={manta_same} manta_display_list_unchanged={manta_png_list} primary_override_ignored={primary_ignored} untargeted_device_unused={stray_unused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_eye_and_a_hidden_group_override_hide_on_one_device() {
    let doc = page();
    let hidden = device_ids::HIDDEN;
    let plain = bare(&doc);
    // The star shows on the Nomad without its override, and not with it; the Manta draws it either
    // way.
    let with = list(&doc, NOMAD_ID);
    let without = list(&plain, NOMAD_ID);
    let fewer = with.items.len() < without.items.len();
    let manta = list(&doc, MANTA_ID) == list(&plain, MANTA_ID);
    let n = node(&doc, NOMAD_ID, hidden);
    let flagged = n.hidden && !node(&doc, MANTA_ID, hidden).hidden;
    // A group hidden on a device hides what is in it there.
    let mut g = plain.clone();
    g.object_mut(device_ids::GROUP).unwrap().overrides.insert(
        NOMAD_ID.into(),
        Override {
            visible: Some(false),
            ..Override::default()
        },
    );
    let child_hidden = node(&g, NOMAD_ID, device_ids::nth(6)).hidden
        && list(&g, NOMAD_ID).items.len() < without.items.len();
    let ok = fewer && manta && flagged && child_hidden;
    println!(
        "\nCHECK crate=layout name=eye_override_hides_on_one_device result={} fewer_items_on_nomad={fewer} manta_unchanged={manta} node_flagged_hidden={flagged} hidden_group_hides_children={child_hidden}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_font_size_override_changes_the_text_on_its_device_only() {
    let doc = devices_document();
    let (manta, nomad) = (builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap());
    let mut reg = quadrule_text::FontRegistry::bundled_only();
    let small = lay_out_text(&doc, &manta, &nomad, &mut reg).unwrap();
    let bare_doc = bare(&doc);
    let big = lay_out_text(&bare_doc, &manta, &nomad, &mut reg).unwrap();
    let id = device_ids::SMALL_TEXT;
    let (hs, hb) = (
        small.outlines.text(id).unwrap().height_mm,
        big.outlines.text(id).unwrap().height_mm,
    );
    // 11 pt against 16 pt: shorter (the 16 pt line wraps on the narrower page, the 11 pt one fits).
    let smaller = hs < hb;
    // The Manta lays the text out at the design's size either way.
    let on_manta = lay_out_text(&doc, &manta, &manta, &mut reg)
        .unwrap()
        .outlines
        == lay_out_text(&bare_doc, &manta, &manta, &mut reg)
            .unwrap()
            .outlines;
    let ok = smaller && on_manta;
    println!(
        "\nCHECK crate=layout name=font_size_override result={} nomad_height_mm={hs:.3} design_height_mm={hb:.3} manta_unchanged={on_manta}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_group_override_scales_its_children_into_its_box() {
    // Q4: the children resolve by their own constraints (and overrides) to an outer box, which is
    // scaled into the group's override box; one override on the group.
    let doc = page();
    let plain = bare(&doc);
    let g = device_ids::GROUP;
    let after = node(&doc, NOMAD_ID, g);
    let want = Frame::new(10.0, 100.0, 66.0, 36.0);
    // Its base is the box round its children as laid out before the group's own override (the
    // child's override included).
    let box_is_override = after.frame.near(&want) && !after.base.near(&want);
    // The children's union on the Nomad is the override box (the child with an override of its
    // own is inside it too).
    let kids: Vec<Node> = [device_ids::nth(6), device_ids::GROUP_CHILD]
        .iter()
        .map(|id| node(&doc, NOMAD_ID, *id))
        .collect();
    let union = kids
        .iter()
        .map(|k| k.frame)
        .reduce(|a, b| a.union(&b))
        .unwrap();
    let fits = union.near(&want);
    // The child with its own override: position override applies first, then the scale. Without
    // the group's override (and so with an identity fit) it sits at (40, 118).
    let mut just_child = plain.clone();
    just_child
        .object_mut(device_ids::GROUP_CHILD)
        .unwrap()
        .overrides
        .insert(
            NOMAD_ID.into(),
            doc.object(device_ids::GROUP_CHILD).unwrap().overrides[NOMAD_ID].clone(),
        );
    let alone = node(&just_child, NOMAD_ID, device_ids::GROUP_CHILD);
    let own_override_first = near(alone.frame.x_mm, 40.0) && near(alone.frame.y_mm, 118.0);
    // The other device: the group has no override there, so its box is its children's.
    let manta = node(&doc, MANTA_ID, g)
        .frame
        .near(&node(&plain, MANTA_ID, g).frame);
    // A group that is not overridden keeps its children's box exactly (identity fit).
    let identity = node(&plain, NOMAD_ID, device_ids::nth(6)).frame
        == plain
            .object(device_ids::nth(6))
            .map(|o| node(&plain, NOMAD_ID, o.id).frame)
            .unwrap();
    let ok = box_is_override && fits && own_override_first && manta && identity;
    println!(
        "\nCHECK crate=layout name=group_override_scales_children result={} group_box_is_override={box_is_override} children_fit_the_box={fits} child_override_applies_before_the_fit={own_override_first} primary_unchanged={manta}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_set_and_reset_helpers_keep_only_real_differences() {
    let doc = page();
    let base = node(&bare(&doc), NOMAD_ID, device_ids::MOVED).base;
    let mut o = bare(&doc).object(device_ids::MOVED).unwrap().clone();
    // A move writes a position and no size; back to the base writes nothing at all.
    set_frame_override(
        &mut o,
        NOMAD_ID,
        base,
        Frame {
            x_mm: base.x_mm + 5.0,
            ..base
        },
    );
    let moved = o.overrides[NOMAD_ID].position.is_some() && o.overrides[NOMAD_ID].size.is_none();
    set_frame_override(&mut o, NOMAD_ID, base, base);
    let gone = o.overrides.is_empty();
    // A resize writes both; Reset takes one part and prunes an empty override.
    set_frame_override(
        &mut o,
        NOMAD_ID,
        base,
        Frame {
            x_mm: base.x_mm + 1.0,
            w_mm: base.w_mm + 2.0,
            ..base
        },
    );
    let both = o.overrides[NOMAD_ID].position.is_some() && o.overrides[NOMAD_ID].size.is_some();
    reset_override(&mut o, NOMAD_ID, &[OverrideField::Position]);
    let one_left = o.overrides[NOMAD_ID].position.is_none() && o.overrides[NOMAD_ID].size.is_some();
    reset_override(&mut o, NOMAD_ID, &[OverrideField::Size]);
    let pruned = o.overrides.is_empty();
    // A nudge sets position only and leaves a size alone.
    set_frame_override(
        &mut o,
        NOMAD_ID,
        base,
        Frame {
            w_mm: base.w_mm + 3.0,
            ..base
        },
    );
    set_position_override(&mut o, NOMAD_ID, base, base.x_mm + 2.0, base.y_mm);
    let nudge_keeps_size =
        o.overrides[NOMAD_ID].size.is_some() && o.overrides[NOMAD_ID].position.is_some();
    let ok = moved && gone && both && one_left && pruned && nudge_keeps_size;
    println!(
        "\nCHECK crate=layout name=override_set_and_reset result={} move_is_position_only={moved} back_to_base_is_no_override={gone} resize_is_both={both} reset_one_part={one_left} empty_override_pruned={pruned} nudge_keeps_size={nudge_keeps_size}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn make_primary_changes_no_pixel_on_either_device() {
    // The Nomad becomes the primary device: its layout (overrides included) is the design, its
    // overrides are gone, and the Manta gets overrides where its place would move. Neither
    // device's display list changes.
    let doc = page();
    let before: Vec<DisplayList> = [MANTA_ID, NOMAD_ID].iter().map(|d| list(&doc, d)).collect();
    let Ok(cmd) = make_primary(&doc, NOMAD_ID) else {
        panic!("make_primary refused");
    };
    let mut next = doc.clone();
    let undo = next.apply(cmd).unwrap();
    let after: Vec<DisplayList> = [MANTA_ID, NOMAD_ID]
        .iter()
        .map(|d| list(&next, d))
        .collect();
    let same = before == after;
    let switched =
        next.primary_profile_id == NOMAD_ID && next.profile_ids() == vec![NOMAD_ID, MANTA_ID];
    let no_nomad_overrides = !next.has_overrides_on(NOMAD_ID);
    let manta_overridden = next.has_overrides_on(MANTA_ID);
    // One undo step puts everything back.
    next.apply(undo).unwrap();
    let back = next == doc;
    // Making the primary the primary again is refused.
    let refused = make_primary(&doc, MANTA_ID).is_err();
    let ok = same && switched && no_nomad_overrides && manta_overridden && back && refused;
    println!(
        "\nCHECK crate=layout name=make_primary_changes_no_pixel result={} display_lists_equal={same} primary_switched={switched} new_primary_overrides_cleared={no_nomad_overrides} old_primary_gets_overrides={manta_overridden} undo_restores={back} already_primary_refused={refused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn make_primary_on_a_plain_page_adds_no_override() {
    // A page with no overrides at all switches without a single override on the old primary: the
    // constraints are undone exactly.
    let doc = bare(&page());
    let cmd = make_primary(&doc, NOMAD_ID).unwrap();
    let mut next = doc.clone();
    next.apply(cmd).unwrap();
    let none = !next.has_overrides_on(MANTA_ID) && !next.has_overrides_on(NOMAD_ID);
    let same = list(&doc, MANTA_ID) == list(&next, MANTA_ID)
        && list(&doc, NOMAD_ID) == list(&next, NOMAD_ID);
    println!(
        "\nCHECK crate=layout name=make_primary_plain_page result={} no_overrides_needed={none} display_lists_equal={same}",
        pass(none && same)
    );
    assert!(none && same);
}

#[test]
fn the_devices_a_document_targets_can_be_ticked_in_and_out() {
    let doc = page();
    // One device only; the overrides of the one taken out go with it, and undo brings them back.
    let cmd = set_targets(&doc, &[MANTA_ID]).unwrap();
    let mut one = doc.clone();
    let undo = one.apply(cmd).unwrap();
    let only_primary = one.profile_ids() == vec![MANTA_ID] && !one.has_overrides_on(NOMAD_ID);
    one.apply(undo).unwrap();
    let restored = one == doc;
    // The primary cannot be taken out.
    let primary_kept = set_targets(&doc, &[NOMAD_ID]).is_err();
    // And a device can be added again, last.
    let again = {
        let cmd = set_targets(&only_manta(&doc), &[MANTA_ID, NOMAD_ID]).unwrap();
        let mut d = only_manta(&doc);
        d.apply(cmd).unwrap();
        d.profile_ids() == vec![MANTA_ID, NOMAD_ID]
    };
    let ok = only_primary && restored && primary_kept && again;
    println!(
        "\nCHECK crate=layout name=device_ticks result={} one_device_drops_overrides={only_primary} undo_restores_them={restored} primary_cannot_go={primary_kept} device_added_again={again}",
        pass(ok)
    );
    assert!(ok);
}

fn only_manta(doc: &Document) -> Document {
    let mut d = doc.clone();
    let cmd = set_targets(&d, &[MANTA_ID]).unwrap();
    d.apply(cmd).unwrap();
    d
}

#[test]
fn ungroup_bakes_the_group_override_into_the_children() {
    let doc = page();
    let primary = builtin(MANTA_ID).unwrap();
    let before = list(&doc, NOMAD_ID);
    let manta_before = list(&doc, MANTA_ID);
    let baked = ungroup_overrides(&doc, &primary, device_ids::GROUP);
    let cmd = ungroup_with(&doc, device_ids::GROUP, &baked).unwrap();
    let mut next = doc.clone();
    next.apply(cmd).unwrap();
    // The group is gone, its children are on the layer, and neither device shows a different page.
    let flat = next.object(device_ids::GROUP).is_none()
        && next
            .object(device_ids::GROUP_CHILD)
            .unwrap()
            .overrides
            .len()
            == 1;
    let nomad = list(&next, NOMAD_ID) == before;
    let manta = list(&next, MANTA_ID) == manta_before;
    // Plain ungroup (the old way, no baked overrides) would have moved the children on the Nomad.
    let mut naive = doc.clone();
    naive
        .apply(quadrule_model::edit::ungroup(&doc, device_ids::GROUP).unwrap())
        .unwrap();
    let naive_differs = list(&naive, NOMAD_ID) != before;
    let ok = flat && nomad && manta && naive_differs;
    println!(
        "\nCHECK crate=layout name=ungroup_bakes_group_override result={} children_kept_overrides={flat} nomad_unchanged={nomad} manta_unchanged={manta} without_baking_would_move={naive_differs}",
        pass(ok)
    );
    assert!(ok);
}
