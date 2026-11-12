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

//! Lint inside groups (rung 7a, docs/bbq/2026-11-11-rung-7-groups-and-align.md, Defaulted "Lint"):
//! every rule runs on each leaf object at its resolved, turned place; a finding names the object
//! and outlines it; a group itself has no rules. Each test prints one structured `CHECK` line
//! (run with `-- --nocapture`).

use quadrule_layout::placed_objects;
use quadrule_lint::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn empty(id: &str) -> ProfileOutlines {
    ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn findings(doc: &Document, id: &str) -> Vec<Finding> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    lint(doc, &primary, &target, &empty(id))
}

#[test]
fn a_child_in_a_group_is_linted_where_it_falls() {
    // A child half off the page, inside a group inside a group: L10 names the child (not a group),
    // and its area is the box where it falls, which is what placed_objects says.
    let mut doc = groups_document();
    let off = Uuid::from_u128(0x7777);
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut child = doc.object(group_ids::nth(5)).unwrap().clone();
    child.id = off;
    child.name = "Off the page".into();
    child.frame = Frame::new(primary.page_mm().0 - 6.0, 40.0, 20.0, 20.0);
    child.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    child.rotation_deg = 0.0;
    child.kind = ObjectKind::Rect(RectShape::default());
    doc.apply(quadrule_model::edit::Command::InsertIn {
        parent: group_ids::nth(4),
        index: 0,
        object: child,
    })
    .unwrap();
    let f = findings(&doc, MANTA_ID);
    let l10: Vec<&Finding> = f.iter().filter(|x| x.rule == "L10").collect();
    let names_child = l10.iter().any(|x| x.object == Some(off));
    let no_group_findings = f.iter().all(|x| {
        x.object
            .is_none_or(|o| doc.object(o).is_some_and(|o| !o.is_group()))
    });
    let placed = placed_objects(&doc, &primary, &primary, &empty(MANTA_ID));
    let b = placed.iter().find(|p| p.id == off).unwrap().shape.bounds();
    let area = l10
        .iter()
        .find(|x| x.object == Some(off))
        .and_then(|x| x.area);
    // The area is the box with half the stroke round it.
    let area_ok =
        area.is_some_and(|a| a.x1 > a.x0 && (a.x0 - b.0).abs() < 4.0 && (a.x1 - b.2).abs() < 4.0);
    let ok = names_child && no_group_findings && area_ok;
    println!(
        "\nCHECK crate=lint name=lint_inside_groups result={} l10_names_child={names_child} no_finding_on_a_group={no_group_findings} area_matches_placed={area_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_hidden_group_is_not_linted_and_a_turned_one_is_judged_turned() {
    let mut doc = groups_document();
    let base = findings(&doc, MANTA_ID).len();
    // Hiding the nested group takes its children's findings away with it.
    doc.object_mut(group_ids::NESTED).unwrap().visible = false;
    let hidden = findings(&doc, MANTA_ID);
    let none_inside = hidden.iter().all(|f| {
        f.object.is_none_or(|o| {
            ![
                group_ids::nth(2),
                group_ids::nth(5),
                group_ids::nth(6),
                group_ids::nth(7),
            ]
            .contains(&o)
        })
    });
    // A turned child's L06 uses its turned corners: the group turned by a quarter turn puts a
    // child that sat clear of the top toolbar over it.
    let mut turned = groups_document();
    let primary = builtin(&turned.primary_profile_id).unwrap();
    let g = turned.object_mut(group_ids::TURNED).unwrap();
    g.rotation_deg = 90.0;
    let placed = placed_objects(&turned, &primary, &primary, &empty(MANTA_ID));
    let corners = placed
        .iter()
        .find(|p| p.id == group_ids::nth(21))
        .map(|p| p.node.angle);
    let angle_ok = corners.is_some_and(|a| (a - 105.0).abs() < 1e-9);
    let ok = none_inside && angle_ok && base >= hidden.len();
    println!(
        "\nCHECK crate=lint name=lint_hidden_and_turned_groups result={} hidden_group_not_linted={none_inside} child_judged_at_105_degrees={angle_ok}",
        pass(ok)
    );
    assert!(ok);
}
