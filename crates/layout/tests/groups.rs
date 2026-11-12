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

//! Groups in layout (rung 7a, docs/bbq/2026-11-11-rung-7-groups-and-align.md, Defaulted "Layout
//! (7a)"): a group resolves against the page by its own constraints, its children against the
//! group's box, then the group's angle turns them about its centre. Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::tree::resolve_nodes;
use quadrule_layout::units::{Axis, UnitContext, parse_length};
use quadrule_layout::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn empty_outlines(id: &str) -> ProfileOutlines {
    ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn list(id: &str, doc: &Document) -> DisplayList {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    display_list(doc, &primary, &target, &empty_outlines(&target.id)).unwrap()
}

#[test]
fn angles_add_down_the_tree_and_nesting_resolves() {
    let doc = groups_document();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let nodes = resolve_nodes(&doc, primary.page_mm(), primary.page_mm());
    let by = |id: Uuid| nodes.iter().find(|n| n.id == id).unwrap();
    // The turned group turns by 25, its child by 15 of its own: 40 on the page; the line has none
    // of its own: 25.
    let child = by(group_ids::nth(21));
    let rule = by(group_ids::nth(23));
    let angles = near(child.angle, 40.0)
        && near(rule.angle, 25.0)
        && near(by(group_ids::TURNED).angle, 25.0);
    // Three levels: Outer (depth 0), Middle (1), Inner (2), Star (3), and on the primary page a
    // child sits where it is stored.
    let star = by(group_ids::nth(5));
    let depth = by(group_ids::NESTED).depth == 0
        && by(group_ids::nth(3)).depth == 1
        && by(group_ids::nth(4)).depth == 2
        && star.depth == 3
        && star.parent == Some(group_ids::nth(4));
    let stored = doc.object(group_ids::nth(5)).unwrap().frame;
    let at_home = star.frame.near(&stored) && star.angle == 0.0;
    // A turned group's child is turned about the group's centre: the centre of the 'Disc' goes
    // where rotate_point puts it.
    let g = doc.object(group_ids::TURNED).unwrap();
    let disc = doc.object(group_ids::nth(22)).unwrap();
    let want = quadrule_model::geom::rotate_point(disc.frame.centre(), g.frame.centre(), 25.0);
    let got = by(group_ids::nth(22)).frame.centre();
    let centre = near(want.0, got.0) && near(want.1, got.1);
    let ok = angles && depth && at_home && centre;
    println!(
        "\nCHECK crate=layout name=group_tree result={} angles_add={angles} depth_and_parent={depth} primary_unchanged={at_home} turned_about_group_centre={centre} nodes={}",
        pass(ok),
        nodes.len()
    );
    assert!(ok);
}

#[test]
fn a_pinned_child_keeps_its_size_and_its_edges_on_the_other_device() {
    let doc = groups_document();
    let (manta, nomad) = (builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap());
    let on = |p: &quadrule_profiles::Profile| resolve_nodes(&doc, manta.page_mm(), p.page_mm());
    let (a, b) = (on(&manta), on(&nomad));
    let get = |nodes: &[tree::Node], id: Uuid| *nodes.iter().find(|n| n.id == id).unwrap();
    let (pg_a, pg_b) = (get(&a, group_ids::PINNED), get(&b, group_ids::PINNED));
    let (pc_a, pc_b) = (
        get(&a, group_ids::PINNED_CHILD),
        get(&b, group_ids::PINNED_CHILD),
    );
    // The group scales with the page (it has the default rules); the pinned child does not.
    let k = nomad.page_mm().0 / manta.page_mm().0;
    let group_scales = near(pg_b.frame.w_mm, pg_a.frame.w_mm * k);
    let size_kept =
        near(pc_b.frame.w_mm, pc_a.frame.w_mm) && near(pc_b.frame.h_mm, pc_a.frame.h_mm);
    let right_gap = |g: &tree::Node, c: &tree::Node| g.frame.right() - c.frame.right();
    let top_gap = |g: &tree::Node, c: &tree::Node| c.frame.y_mm - g.frame.y_mm;
    let edges = near(right_gap(&pg_a, &pc_a), right_gap(&pg_b, &pc_b))
        && near(top_gap(&pg_a, &pc_a), top_gap(&pg_b, &pc_b));
    // A scaled child grows with the group.
    let (sc_a, sc_b) = (get(&a, group_ids::nth(43)), get(&b, group_ids::nth(43)));
    let scaled = near(
        sc_b.frame.w_mm / sc_a.frame.w_mm,
        pg_b.frame.w_mm / pg_a.frame.w_mm,
    );
    let ok = group_scales && size_kept && edges && scaled;
    println!(
        "\nCHECK crate=layout name=group_children_follow_their_constraints result={} group_scales={group_scales} pinned_size_kept={size_kept} pinned_edges_kept={edges} scaled_child_grows={scaled}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_group_fades_as_one_and_hides_what_is_in_it() {
    let doc = groups_document();
    let dl = list(MANTA_ID, &doc);
    let begins: Vec<&Item> = dl
        .items
        .iter()
        .filter(|i| matches!(i, Item::BeginAlpha { .. }))
        .collect();
    let ends = dl
        .items
        .iter()
        .filter(|i| matches!(i, Item::EndAlpha))
        .count();
    // The half group (50 percent), the right half of the heavy group (80); balanced.
    let alphas: Vec<u8> = begins
        .iter()
        .filter_map(|i| match i {
            Item::BeginAlpha { alpha, .. } => Some(*alpha),
            _ => None,
        })
        .collect();
    let balanced = begins.len() == ends && alphas.contains(&128) && alphas.contains(&204);
    // Nested fades nest: a child at 50 inside a group at 50 makes two groups, one inside the other.
    let mut nest = groups_document();
    let g = nest.object_mut(group_ids::HALF).unwrap();
    if let ObjectKind::Group(s) = &mut g.kind {
        s.children[0].opacity_pct = 50;
    }
    let n = list(MANTA_ID, &nest);
    let mut depth = 0i32;
    let mut max = 0i32;
    for i in &n.items {
        match i {
            Item::BeginAlpha { .. } => {
                depth += 1;
                max = max.max(depth)
            }
            Item::EndAlpha => depth -= 1,
            _ => {}
        }
    }
    let nested = max >= 2 && depth == 0;
    // A hidden group draws nothing of its own children; a group at 0 percent likewise.
    let mut hidden = groups_document();
    hidden.object_mut(group_ids::NESTED).unwrap().visible = false;
    let h = list(MANTA_ID, &hidden);
    let mut gone = groups_document();
    gone.object_mut(group_ids::NESTED).unwrap().opacity_pct = 0;
    let z = list(MANTA_ID, &gone);
    let fewer = h.items.len() < dl.items.len() && h.items == z.items;
    let ok = balanced && nested && fewer;
    println!(
        "\nCHECK crate=layout name=group_opacity_and_visibility result={} groups={} balanced={balanced} nested_fades_nest={nested} hidden_equals_zero_percent={fewer}",
        pass(ok),
        begins.len()
    );
    assert!(ok);
}

#[test]
fn placed_objects_lists_groups_and_their_children() {
    let doc = groups_document();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let placed = placed_objects(&doc, &primary, &primary, &empty_outlines(&primary.id));
    let all = placed.len() == doc.objects().count();
    let groups = placed.iter().filter(|p| p.node.group).count();
    // A group comes before its children and its shape is its box, turned with it.
    let order = {
        let i = placed
            .iter()
            .position(|p| p.id == group_ids::TURNED)
            .unwrap();
        let j = placed
            .iter()
            .position(|p| p.id == group_ids::nth(21))
            .unwrap();
        i < j
    };
    let turned = matches!(
        placed.iter().find(|p| p.id == group_ids::TURNED).unwrap().shape,
        Shape::Rotated { deg, .. } if near(deg, 25.0)
    );
    let ok = all && groups == doc.objects().filter(|o| o.is_group()).count() && order && turned;
    println!(
        "\nCHECK crate=layout name=placed_groups result={} listed={} groups={groups} group_before_children={order} turned_shape={turned}",
        pass(ok),
        placed.len()
    );
    assert!(ok);
}

#[test]
fn percent_is_of_the_group_inside_one() {
    let doc = groups_document();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let cx = UnitContext::new(&doc, &primary, &primary, Axis::X);
    let g = doc.object(group_ids::NESTED).unwrap();
    let in_group = cx.with_percent_of(g.frame.w_mm);
    let page = parse_length("50%", &cx).unwrap();
    let inner = parse_length("50%", &in_group).unwrap();
    let ok = near(page, primary.page_mm().0 / 2.0) && near(inner, g.frame.w_mm / 2.0);
    println!(
        "\nCHECK crate=layout name=percent_of_group result={} page_half_mm={page:.3} group_half_mm={inner:.3}",
        pass(ok)
    );
    assert!(ok);
}
