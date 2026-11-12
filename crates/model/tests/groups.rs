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

//! Groups in the model (rung 7a, docs/bbq/2026-11-11-rung-7-groups-and-align.md): making and
//! breaking, nesting, the box that refits, lock and turn rules, resizing a group (its contents stretch),
//! copy and paste. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::edit::*;
use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn leaf(n: u128, frame: Frame) -> Object {
    let mut o = groups_document().layers[1].objects[0].children()[0].clone();
    o.id = Uuid::from_u128(0x7000 + n);
    o.name = format!("o{n}");
    o.frame = frame;
    o.constraints = Constraints::default();
    o.rotation_deg = 0.0;
    o.opacity_pct = 100;
    o.visible = true;
    o.locked = false;
    o.kind = ObjectKind::Rect(RectShape::default());
    o
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x7000 + n)
}

/// Three objects on two layers, a fourth elsewhere.
fn page() -> Document {
    let mut doc = groups_document();
    doc.layers = vec![
        Layer {
            name: "A".into(),
            visible: true,
            locked: false,
            objects: vec![
                leaf(1, Frame::new(10.0, 10.0, 20.0, 10.0)),
                leaf(2, Frame::new(40.0, 20.0, 10.0, 30.0)),
            ],
            extra: Extra::new(),
        },
        Layer {
            name: "B".into(),
            visible: true,
            locked: false,
            objects: vec![
                leaf(3, Frame::new(60.0, 60.0, 10.0, 10.0)),
                leaf(4, Frame::new(100.0, 100.0, 5.0, 5.0)),
            ],
            extra: Extra::new(),
        },
    ];
    doc
}

#[test]
fn group_and_ungroup_round_trip() {
    let doc = page();
    let mut work = doc.clone();
    let mut hist = History::new();
    let gid = Uuid::from_u128(0x9000);
    let cmd = group(&doc, &[id(1), id(3), id(2)], gid, "Group").unwrap();
    hist.apply(&mut work, cmd, None).unwrap();
    let g = work.object(gid).unwrap();
    // The group sits on the layer of the topmost picked object (3, on layer B), in its place; the
    // children keep their stacking order and their frames; the box is round them.
    let (place, index) = work.place_of(gid).unwrap();
    let order: Vec<Uuid> = g.children().iter().map(|c| c.id).collect();
    let same_frames = [1, 2, 3]
        .iter()
        .all(|n| work.object(id(*n)).unwrap().frame == doc.object(id(*n)).unwrap().frame);
    let boxed = g.frame == Frame::new(10.0, 10.0, 60.0, 60.0);
    let grouped_ok = place == Place::Layer(1)
        && index == 0
        && order == vec![id(1), id(2), id(3)]
        && same_frames
        && boxed
        && work.layers[0].objects.is_empty()
        && work.layers[1].objects.len() == 2;
    // Undo restores the document exactly; redo makes the group again.
    hist.undo(&mut work);
    let undone = work == doc;
    hist.redo(&mut work);
    let redone = work.object(gid).is_some();
    // Ungroup puts the children back where the group was, frames unchanged.
    let cmd = ungroup(&work, gid).unwrap();
    hist.apply(&mut work, cmd, None).unwrap();
    let flat = work.object(gid).is_none()
        && work.layers[1]
            .objects
            .iter()
            .map(|o| o.id)
            .collect::<Vec<_>>()
            == vec![id(1), id(2), id(3), id(4)]
        && work.layers[0].objects.is_empty()
        && [1, 2, 3, 4]
            .iter()
            .all(|n| work.object(id(*n)).unwrap().frame == doc.object(id(*n)).unwrap().frame);
    hist.undo(&mut work);
    let regrouped = work.object(gid).is_some();
    println!(
        "\nCHECK crate=model name=group_ungroup_round_trip result={} grouped={grouped_ok} undone={undone} redone={redone} ungrouped_flat={flat} undo_ungroup={regrouped}",
        pass(grouped_ok && undone && redone && flat && regrouped)
    );
    assert!(grouped_ok && undone && redone && flat && regrouped);
}

#[test]
fn ungroup_bakes_angle_and_opacity() {
    let doc = page();
    let mut work = doc.clone();
    let gid = Uuid::from_u128(0x9000);
    let cmd = group(&doc, &[id(1), id(2)], gid, "G").unwrap();
    work.apply(cmd).unwrap();
    // Turn the group by 90 degrees and fade it to 50 percent, one child to 50 as well.
    let mut g = work.object(gid).unwrap().clone();
    g.rotation_deg = 90.0;
    g.opacity_pct = 50;
    work.apply(Command::SetObject(g.clone())).unwrap();
    let mut c = work.object(id(2)).unwrap().clone();
    c.opacity_pct = 50;
    work.apply(Command::SetObject(c)).unwrap();
    let before_corners: Vec<(f64, f64)> = [1, 2]
        .iter()
        .flat_map(|n| {
            let o = work.object(id(*n)).unwrap();
            let c = o.frame.centre();
            let r = quadrule_model::geom::rotate_point(
                c,
                work.object(gid).unwrap().frame.centre(),
                90.0,
            );
            [r]
        })
        .collect();
    let cmd = ungroup(&work, gid).unwrap();
    work.apply(cmd).unwrap();
    let after: Vec<&Object> = [1, 2]
        .iter()
        .map(|n| work.object(id(*n)).unwrap())
        .collect();
    // Each child's centre is where the turn put it, its angle is 90, opacity multiplied.
    let centres = after.iter().zip(&before_corners).all(|(o, b)| {
        (o.frame.centre().0 - b.0).abs() < 1e-9 && (o.frame.centre().1 - b.1).abs() < 1e-9
    });
    let angles = after.iter().all(|o| o.rotation() == 90.0);
    let opacity = after[0].opacity() == 50 && after[1].opacity() == 25;
    println!(
        "\nCHECK crate=model name=ungroup_bakes_angle_and_opacity result={} centres_moved={centres} angles_added={angles} opacity_multiplied={opacity}",
        pass(centres && angles && opacity)
    );
    assert!(centres && angles && opacity);
}

#[test]
fn nesting_three_deep_and_beyond() {
    let doc = groups_document();
    let ids = [
        group_ids::NESTED,
        group_ids::nth(3),
        group_ids::nth(4),
        group_ids::nth(5),
    ];
    let chain = doc.ancestors(group_ids::nth(5));
    let depth_ok = chain == ids[..3].to_vec();
    let found = doc.objects().filter(|o| o.is_group()).count();
    let leaves = doc.leaves().count();
    // Group again, over and over: no limit.
    let mut work = page();
    let mut top = Uuid::from_u128(0x9000);
    let cmd = group(&work, &[id(1), id(2)], top, "G0").unwrap();
    work.apply(cmd).unwrap();
    let mut nested_ok = true;
    for k in 1..12u128 {
        let next = Uuid::from_u128(0x9000 + k);
        // Something new beside the group each time, to group it with.
        work.apply(Command::Insert {
            layer: 1,
            index: 0,
            object: leaf(100 + k, Frame::new(5.0, 5.0, 3.0, 3.0)),
        })
        .unwrap();
        let cmd = group(&work, &[top, id(100 + k)], next, "G").unwrap();
        nested_ok &= work.apply(cmd).is_ok();
        top = next;
    }
    let deep = work.ancestors(id(1)).len();
    println!(
        "\nCHECK crate=model name=nesting result={} page_chain_len={} groups_on_page={found} leaves_on_page={leaves} grouped_twelve_levels={nested_ok} deepest={deep}",
        pass(depth_ok && nested_ok && deep == 12),
        chain.len()
    );
    assert!(depth_ok && nested_ok && deep == 12);
}

#[test]
fn the_box_refits_after_an_edit_inside() {
    let doc = page();
    let mut work = doc.clone();
    let mut hist = History::new();
    let gid = Uuid::from_u128(0x9000);
    let cmd = group(&doc, &[id(1), id(2)], gid, "G").unwrap();
    hist.apply(&mut work, cmd, None).unwrap();
    let start = work.clone();
    // Pull a child out to the right: the box grows to hold it.
    let mut c = work.object(id(2)).unwrap().clone();
    c.frame.x_mm = 80.0;
    hist.apply(&mut work, Command::SetObject(c), None).unwrap();
    let grown = work.object(gid).unwrap().frame == Frame::new(10.0, 10.0, 80.0 + 10.0 - 10.0, 40.0);
    hist.undo(&mut work);
    let undone = work == start;
    // Take a child out of the group: the box shrinks round the one left.
    let cmd = Command::MoveObject {
        id: id(2),
        layer: 0,
        index: 0,
    };
    hist.apply(&mut work, cmd, None).unwrap();
    let shrunk = work.object(gid).unwrap().frame == Frame::new(10.0, 10.0, 20.0, 10.0);
    // And the last child out: the empty group goes away, and comes back on undo.
    let cmd = Command::MoveObject {
        id: id(1),
        layer: 0,
        index: 0,
    };
    hist.apply(&mut work, cmd, None).unwrap();
    let gone = work.object(gid).is_none();
    hist.undo(&mut work);
    let back = work.object(gid).is_some();
    println!(
        "\nCHECK crate=model name=group_box_refits result={} grew={grown} undo_exact={undone} shrank={shrunk} empty_group_removed={gone} undo_restores_it={back}",
        pass(grown && undone && shrunk && gone && back)
    );
    assert!(grown && undone && shrunk && gone && back);
}

#[test]
fn a_turned_groups_children_stay_put_when_the_box_refits() {
    // A group turned 30 degrees: pulling one child out changes the box and so the centre the group
    // turns about; the rest must not move on the page.
    let mut work = page();
    let gid = Uuid::from_u128(0x9000);
    let cmd = group(&work, &[id(1), id(2), id(3)], gid, "G").unwrap();
    work.apply(cmd).unwrap();
    let mut g = work.object(gid).unwrap().clone();
    g.rotation_deg = 30.0;
    work.apply(Command::SetObject(g)).unwrap();
    let on_page = |doc: &Document, n: u128| {
        let g = doc.object(gid).unwrap();
        quadrule_model::geom::rotate_point(
            doc.object(id(n)).unwrap().frame.centre(),
            g.frame.centre(),
            g.rotation(),
        )
    };
    let before = (on_page(&work, 1), on_page(&work, 2));
    // Take the third out (its layer's top).
    work.apply(Command::MoveObject {
        id: id(3),
        layer: 1,
        index: 0,
    })
    .unwrap();
    let after = (on_page(&work, 1), on_page(&work, 2));
    let same = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9;
    let ok = same(before.0, after.0) && same(before.1, after.1);
    println!(
        "\nCHECK crate=model name=turned_group_refit_keeps_the_page result={} moved_px={:.9}",
        pass(ok),
        (before.0.0 - after.0.0).abs() + (before.0.1 - after.0.1).abs()
    );
    assert!(ok);
}

#[test]
fn locks_hide_and_turn_rules() {
    let mut work = page();
    let gid = Uuid::from_u128(0x9000);
    let cmd = group(&work, &[id(1), id(2)], gid, "G").unwrap();
    work.apply(cmd).unwrap();
    // A locked group locks what is in it; a hidden group hides it.
    let mut g = work.object(gid).unwrap().clone();
    g.locked = true;
    g.visible = false;
    work.apply(Command::SetObject(g)).unwrap();
    let cascades = work.effectively_locked(id(1))
        && work.effectively_hidden(id(2))
        && !work.editable(id(1))
        && work.editable(id(3));
    let mut moved = work.object(id(1)).unwrap().clone();
    moved.frame.x_mm += 1.0;
    let refused = work.apply(Command::SetObject(moved)) == Err(EditError::Locked(id(1)));
    // A turned group refuses a text box and a pattern region.
    let mut work2 = page();
    let cmd = group(&work2, &[id(1), id(2)], gid, "G").unwrap();
    work2.apply(cmd).unwrap();
    let mut g = work2.object(gid).unwrap().clone();
    g.rotation_deg = 20.0;
    work2.apply(Command::SetObject(g)).unwrap();
    let mut text = work2.object(id(3)).unwrap().clone();
    text.kind = ObjectKind::Pattern(PatternRegion {
        pattern: Pattern::Lines,
        extra: Extra::new(),
    });
    work2.apply(Command::SetObject(text)).unwrap();
    let into = work2.apply(Command::MoveObjectIn {
        id: id(3),
        parent: gid,
        index: 0,
    });
    let refused_turned = into == Err(EditError::TurnedGroup(gid));
    // And a group holding a pattern region cannot turn.
    let mut work3 = work2.clone();
    work3
        .apply(Command::MoveObject {
            id: id(1),
            layer: 0,
            index: 0,
        })
        .ok();
    let holds = ObjectKind::Group(GroupShape {
        children: vec![work2.object(id(3)).unwrap().clone()],
        extra: Extra::new(),
    });
    let cannot_turn = !holds.can_rotate();
    println!(
        "\nCHECK crate=model name=group_locks_and_turn_rules result={} cascade={cascades} locked_child_refuses_edit={refused} turned_group_refuses_pattern={refused_turned} group_with_pattern_cannot_turn={cannot_turn}",
        pass(cascades && refused && refused_turned && cannot_turn)
    );
    assert!(cascades && refused && refused_turned && cannot_turn);
}

#[test]
fn resizing_a_group_stretches_everything_inside_like_a_loose_selection() {
    // A group whose box is round its two children, (10, 10) to (100, 100), grows to 135 x 108: each
    // child is mapped as in a loose multiple selection's resize, whatever its constraints (a group
    // is see-through to them), then the box is round the children again.
    let group_with = |c: Constraints| {
        let mut child = leaf(1, Frame::new(10.0, 10.0, 20.0, 20.0));
        child.constraints = c;
        let mut other = leaf(2, Frame::new(80.0, 80.0, 20.0, 20.0));
        other.constraints = Constraints::default();
        let mut g = leaf(9, Frame::new(10.0, 10.0, 90.0, 90.0));
        g.kind = ObjectKind::Group(GroupShape {
            children: vec![child, other],
            extra: Extra::new(),
        });
        g
    };
    let grown = |c: Constraints| {
        let mut g = group_with(c);
        reframe(&mut g, Frame::new(10.0, 10.0, 135.0, 108.0));
        g
    };
    let c = |h, v| Constraints { h, v };
    let near = |a: Frame, b: Frame| a.near(&b);
    let first = Frame::new(10.0, 10.0, 30.0, 24.0);
    let second = Frame::new(115.0, 94.0, 30.0, 24.0);
    let mut each_rule = true;
    for k in [
        c(HConstraint::Left, VConstraint::Top),
        c(HConstraint::Right, VConstraint::Bottom),
        c(HConstraint::LeftRight, VConstraint::TopBottom),
        c(HConstraint::Center, VConstraint::Center),
        Constraints::default(),
    ] {
        let g = grown(k);
        each_rule &= near(g.children()[0].frame, first)
            && near(g.children()[1].frame, second)
            && near(g.frame, Frame::new(10.0, 10.0, 135.0, 108.0));
    }
    // A move alone carries everything by the same step, and the box with it.
    let mut g = group_with(Constraints::default());
    reframe(&mut g, Frame::new(15.0, 17.0, 90.0, 90.0));
    let moved = g.children()[1].frame == Frame::new(85.0, 87.0, 20.0, 20.0)
        && g.frame == Frame::new(15.0, 17.0, 90.0, 90.0);
    // A turned child keeps its angle and stretches approximately; the box ends up round it.
    let mut t = group_with(Constraints::default());
    if let ObjectKind::Group(s) = &mut t.kind {
        s.children[0].rotation_deg = 90.0;
        s.children[0].frame = Frame::new(10.0, 10.0, 10.0, 30.0);
    }
    t.frame = children_outer(&t);
    let wide = Frame::new(t.frame.x_mm, t.frame.y_mm, t.frame.w_mm * 2.0, t.frame.h_mm);
    reframe(&mut t, wide);
    let kid = &t.children()[0];
    let turned = kid.rotation_deg == 90.0 && near(t.frame, children_outer(&t));
    let ok = each_rule && moved && turned;
    println!(
        "\nCHECK crate=model name=group_resize_stretches result={} every_rule_alike={each_rule} move_ok={moved} turned_child_keeps_angle_box_refits={turned}",
        pass(ok)
    );
    assert!(ok);
}

fn children_outer(g: &Object) -> Frame {
    g.children()
        .iter()
        .map(Object::outer_box)
        .reduce(|a, b| a.union(&b))
        .unwrap()
}

#[test]
fn duplicate_and_paste_give_every_descendant_a_new_id() {
    let doc = groups_document();
    let mut counter = 0u128;
    let mut fresh = || {
        counter += 1;
        Uuid::from_u128(0xA000 + counter)
    };
    let (cmd, copies) = duplicate(&doc, &[group_ids::NESTED], &mut fresh);
    let mut work = doc.clone();
    work.apply(cmd).unwrap();
    let copy = work.object(copies[0]).unwrap();
    let all_new = work
        .objects()
        .map(|o| o.id)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == work.objects().count();
    let before = doc.objects().count();
    let after = work.objects().count();
    let sub = doc.object(group_ids::NESTED).unwrap().children().len();
    let copied_whole = after - before == 1 + 1 + 1 + 1 + 1 + 1 + 1 && copy.children().len() == sub;
    // Through the clipboard.
    let text = clipboard::to_text(&doc, &[group_ids::NESTED, group_ids::nth(3)]).unwrap();
    let objs = clipboard::from_text(&text).unwrap();
    // A child whose group is also picked is not copied twice.
    let once = objs.len() == 1 && objs[0].id == group_ids::NESTED;
    let (cmd, ids) = paste(&doc, &objs, 1, (6.0, 6.0), &mut fresh);
    let mut work = doc.clone();
    work.apply(cmd).unwrap();
    let pasted = work.object(ids[0]).unwrap();
    let moved = pasted.frame.x_mm == doc.object(group_ids::NESTED).unwrap().frame.x_mm + 6.0
        && pasted.children()[0].frame.x_mm
            == doc.object(group_ids::NESTED).unwrap().children()[0]
                .frame
                .x_mm
                + 6.0;
    println!(
        "\nCHECK crate=model name=group_clipboard result={} duplicate_whole_tree={copied_whole} ids_unique={all_new} child_not_copied_twice={once} paste_moves_tree={moved}",
        pass(copied_whole && all_new && once && moved)
    );
    assert!(copied_whole && all_new && once && moved);
}

#[test]
fn a_group_survives_the_clipboard_and_keeps_unknown_fields() {
    let doc = groups_document();
    let mut g = doc.object(group_ids::HALF).unwrap().clone();
    if let ObjectKind::Group(s) = &mut g.kind {
        s.extra
            .insert("future".into(), ciborium::Value::Text("kept".into()));
    }
    let json = serde_json::to_string(&g).unwrap();
    let back: Object = serde_json::from_str(&json).unwrap();
    let ok =
        back == g && matches!(&back.kind, ObjectKind::Group(s) if s.extra.contains_key("future"));
    println!(
        "\nCHECK crate=model name=group_extra_fields result={} round_trip={ok}",
        pass(ok)
    );
    assert!(ok);
}
