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

//! Rung 7a: groups in the editor (docs/bbq/2026-11-11-rung-7-groups-and-align.md, Q2, Q3, Q4, Q7).
//! Headless. Each test prints one structured `CHECK` line starting with a newline (run with
//! `-- --nocapture`).
//!
//! `egui_kittest` cannot make a double-click (two clicks give `double_clicked() == false`), so
//! going into a group is tested through [`Viewer::enter_at`], which the window calls on a
//! double-click, as rung 5's keep-tool is through `Viewer::keep_tool`; chris's look is its first
//! real check. Coordinates are Manta device pixels.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_layout::snap::Handle;
use quadrule_model::edit::{Command, Place};
use quadrule_model::sample::*;
use quadrule_model::{Frame, ObjectKind, Uuid};
use quadrule_ui::{DragKind, DragMods, HandleHit, Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: false,
};
const FREE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn open() -> Viewer {
    let mut v = Viewer::default();
    v.open_groups();
    v.rotate_stalk_px = 60.0;
    v
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

/// The centre of an object's box on the Manta page, in pixels.
fn centre(v: &mut Viewer, id: Uuid) -> (f64, f64) {
    let b = v
        .page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .shape
        .bounds();
    ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0)
}

fn px_per_mm() -> f64 {
    quadrule_profiles::builtin(MANTA_ID).unwrap().mm_to_px(1.0)
}

fn frame(v: &Viewer, id: Uuid) -> Frame {
    v.doc().unwrap().object(id).unwrap().frame
}

#[test]
fn click_picks_the_group_double_click_goes_in_escape_and_ctrl_click() {
    let mut v = open();
    let star = group_ids::nth(5);
    let (inner, middle, outer) = (group_ids::nth(4), group_ids::nth(3), group_ids::NESTED);
    let at = centre(&mut v, star);
    // A click picks the outermost group.
    v.click(at.0, at.1, 4.0, false);
    let outermost = v.selection == vec![outer] && v.entered.is_empty();
    // A double-click goes one level in and picks the child under the pointer; again, and again.
    let a = v.enter_at(at.0, at.1, 4.0);
    let one = v.entered == vec![outer] && v.selection == vec![middle];
    let b = v.enter_at(at.0, at.1, 4.0);
    let b_ok = v.entered == vec![outer, middle] && v.selection == vec![inner];
    let c = v.enter_at(at.0, at.1, 4.0);
    let c_ok = v.entered == vec![outer, middle, inner] && v.selection == vec![star];
    // A leaf has nothing to go into.
    let d = v.enter_at(at.0, at.1, 4.0);
    // A plain click now picks among the entered group's children.
    v.click(at.0, at.1, 4.0, false);
    let among = v.selection == vec![star];
    // Escape steps out one level and selects the group left.
    let left = v.leave_group() && v.entered == vec![outer, middle] && v.selection == vec![inner];
    // A click outside the entered group steps out one level: the heavy group's cell is outside.
    let cell = centre(&mut v, group_ids::nth(120));
    v.click(cell.0, cell.1, 4.0, false);
    let outside = v.selection == vec![group_ids::HEAVY] && v.entered == vec![outer];
    // Ctrl+click picks the innermost object at once, entering the groups on the way.
    v.entered.clear();
    v.click_with(at.0, at.1, 4.0, false, true);
    let deep = v.selection == vec![star] && v.entered == vec![outer, middle, inner];
    // Escape on the way out of the last one, then none left.
    while v.leave_group() {}
    let none = !v.leave_group();
    let ok = outermost
        && a
        && one
        && b
        && b_ok
        && c
        && c_ok
        && !d
        && among
        && left
        && outside
        && deep
        && none;
    println!(
        "\nCHECK crate=ui name=group_pick_enter_leave result={} click_outermost={outermost} enter_levels={} leaf_stops={} among_children={among} escape_steps_out={left} click_outside_steps_out={outside} ctrl_click_innermost={deep}",
        pass(ok),
        a && one && b && b_ok && c && c_ok,
        !d
    );
    assert!(ok);
}

#[test]
fn a_group_is_hit_where_a_child_is_and_locks_and_eyes_cascade() {
    let mut v = open();
    // The empty middle of the nested group's box is not the group: its Box child has no fill, and
    // a hollow shape is picked by its frame, so use a gap between heavy cells instead (x between
    // two cells) which no child covers.
    let cell = centre(&mut v, group_ids::nth(120));
    let gap = (cell.0 + 8.0 * px_per_mm(), cell.1);
    let miss = v.pick(gap.0, gap.1, 1.0).is_none();
    let hit = v.pick(cell.0, cell.1, 1.0) == Some(group_ids::HEAVY);
    // Lock the heavy group: nothing in it is picked; hide it: likewise, and it is not drawn.
    v.set_object_panel(group_ids::HEAVY, |o| o.locked = true)
        .unwrap();
    let locked = v.pick(cell.0, cell.1, 1.0).is_none();
    v.set_object_panel(group_ids::HEAVY, |o| o.locked = false)
        .unwrap();
    v.set_object_panel(group_ids::HEAVY, |o| o.visible = false)
        .unwrap();
    let hidden = v.pick(cell.0, cell.1, 1.0).is_none();
    let ok = miss && hit && locked && hidden;
    println!(
        "\nCHECK crate=ui name=group_picking_and_cascade result={} gap_misses={miss} cell_picks_group={hit} locked_group_not_picked={locked} hidden_group_not_picked={hidden}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn dragging_a_group_moves_everything_in_it_as_one_undo_step() {
    let mut v = open();
    let before = v.doc().unwrap().clone();
    let cell = centre(&mut v, group_ids::nth(120));
    let steps0 = steps(&v);
    // A plain press picks the group (snapping would round the step, so Ctrl is held for the
    // drag itself): with the group selected, Ctrl only turns snapping off.
    v.click(cell.0, cell.1, 4.0, false);
    assert!(v.drag_begin(cell.0, cell.1, FREE, 4.0, 6.0));
    let picked = v.selection == vec![group_ids::HEAVY];
    v.drag_update(cell.0 + 59.0, cell.1 + 118.0, FREE, REACH);
    v.drag_end().unwrap();
    let k = px_per_mm();
    let (dx, dy) = (59.0 / k, 118.0 / k);
    let g0 = before.object(group_ids::HEAVY).unwrap();
    let g1 = v.doc().unwrap().object(group_ids::HEAVY).unwrap();
    let group_moved = (g1.frame.x_mm - g0.frame.x_mm - dx).abs() < 1e-6
        && (g1.frame.y_mm - g0.frame.y_mm - dy).abs() < 1e-6;
    // Every descendant moved by the same step, and none changed size.
    let all = before
        .objects()
        .filter(|o| before.ancestors(o.id).contains(&group_ids::HEAVY))
        .all(|o| {
            let n = v.doc().unwrap().object(o.id).unwrap();
            (n.frame.x_mm - o.frame.x_mm - dx).abs() < 1e-6
                && (n.frame.y_mm - o.frame.y_mm - dy).abs() < 1e-6
                && n.frame.w_mm == o.frame.w_mm
        });
    let one_step = steps(&v) == steps0 + 1;
    v.undo();
    let undone = *v.doc().unwrap() == before;
    let ok = picked && group_moved && all && one_step && undone;
    println!(
        "\nCHECK crate=ui name=group_drag result={} picked_group={picked} group_moved={group_moved} descendants_moved={all} one_undo_step={one_step} undo_exact={undone}",
        pass(ok)
    );
    assert!(ok);
}

fn handle_at(v: &mut Viewer, h: Handle) -> (f64, f64) {
    v.handles()
        .into_iter()
        .find(|(k, ..)| *k == HandleHit::Box(h))
        .map(|(_, x, y)| (x, y))
        .expect("the handle")
}

#[test]
fn resizing_a_group_resolves_each_child_by_its_constraints() {
    let mut v = open();
    let before = v.doc().unwrap().clone();
    v.selection = vec![group_ids::PINNED];
    let se = handle_at(&mut v, Handle::SE);
    assert!(v.drag_begin(se.0, se.1, FREE, 4.0, 6.0));
    let kind = v.drag.as_ref().map(|d| d.kind);
    let k = px_per_mm();
    // Wider by 20 mm and taller by 10.
    v.drag_update(se.0 + 20.0 * k, se.1 + 10.0 * k, FREE, REACH);
    v.drag_end().unwrap();
    let g0 = before.object(group_ids::PINNED).unwrap().frame;
    let g1 = frame(&v, group_ids::PINNED);
    let grew = (g1.w_mm - g0.w_mm - 20.0).abs() < 1e-6 && (g1.h_mm - g0.h_mm - 10.0).abs() < 1e-6;
    // The child pinned right and top keeps its size and its gap to the top and right edges.
    let (p0, p1) = (
        before.object(group_ids::PINNED_CHILD).unwrap().frame,
        frame(&v, group_ids::PINNED_CHILD),
    );
    let pinned = (p1.w_mm - p0.w_mm).abs() < 1e-6
        && (p1.h_mm - p0.h_mm).abs() < 1e-6
        && ((g1.right() - p1.right()) - (g0.right() - p0.right())).abs() < 1e-6
        && ((p1.y_mm - g1.y_mm) - (p0.y_mm - g0.y_mm)).abs() < 1e-6;
    // The child that scales grows by the same ratio as the group.
    let (s0, s1) = (
        before.object(group_ids::nth(43)).unwrap().frame,
        frame(&v, group_ids::nth(43)),
    );
    let scaled = (s1.w_mm / s0.w_mm - g1.w_mm / g0.w_mm).abs() < 1e-6;
    let ok = kind == Some(DragKind::Handle(HandleHit::Box(Handle::SE))) && grew && pinned && scaled;
    println!(
        "\nCHECK crate=ui name=group_resize result={} group_grew={grew} pinned_child_keeps_size_and_edges={pinned} scaled_child_scales={scaled}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn turning_a_group_turns_its_children_and_a_pattern_stops_it() {
    let mut v = open();
    v.selection = vec![group_ids::HALF];
    let h = v.rotate_handle().expect("a group can turn");
    let c = centre(&mut v, group_ids::HALF);
    assert!(v.drag_begin(h.tip.0, h.tip.1, NONE, 4.0, 6.0));
    // A quarter turn clockwise: the pointer goes from straight above to the right of the centre.
    v.drag_update(c.0 + 500.0, c.1, FREE, REACH);
    v.drag_end().unwrap();
    let g = v.doc().unwrap().object(group_ids::HALF).unwrap();
    let turned = (g.rotation() - 90.0).abs() < 1e-6;
    // Its children keep their own angle (0): the two add up on the page, not in the file.
    let kids_plain = g.children().iter().all(|c| c.rotation() == 0.0);
    let page_angle = v
        .page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == group_ids::nth(32))
        .map(|p| p.node.angle);
    let adds = page_angle.is_some_and(|a| (a - 90.0).abs() < 1e-6);
    // A pattern region in the group: no rotate handle, and the group cannot turn.
    let pattern = {
        let mut o = v.doc().unwrap().object(group_ids::nth(32)).unwrap().clone();
        o.id = Uuid::from_u128(0x7A77);
        o.kind = ObjectKind::Pattern(quadrule_model::PatternRegion {
            pattern: quadrule_model::Pattern::Lines,
            extra: quadrule_model::Extra::new(),
        });
        o
    };
    let mut flat = v.doc().unwrap().object(group_ids::HALF).unwrap().clone();
    flat.rotation_deg = 0.0;
    v.edit(Command::SetObject(flat), None).unwrap();
    // Turned groups refuse the pattern; an unturned one takes it, and then cannot turn.
    let ok_in = v
        .edit(
            Command::InsertIn {
                parent: group_ids::HALF,
                index: 0,
                object: pattern,
            },
            None,
        )
        .is_ok();
    v.selection = vec![group_ids::HALF];
    let no_handle = v.rotate_handle().is_none();
    let angle_ignored = v
        .doc()
        .unwrap()
        .object(group_ids::HALF)
        .is_some_and(|g| g.rotation() == 0.0 && !g.kind.can_rotate());
    let ok = turned && kids_plain && adds && ok_in && no_handle && angle_ignored;
    println!(
        "\nCHECK crate=ui name=group_turn result={} group_turned={turned} children_keep_own_angle={kids_plain} angles_add_on_page={adds} pattern_hides_handle={no_handle}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_child_in_a_turned_group_moves_by_what_the_pointer_moves() {
    // The turned group (25 degrees): go in, drag the disc 30 px right on the page. It moves 30 px
    // right on the page, whichever way the group is turned, and the group's box refits round it.
    let mut v = open();
    let disc = group_ids::nth(22);
    let at = centre(&mut v, disc);
    assert!(v.enter_at(at.0, at.1, 2.0));
    v.select_row(disc, false);
    let was = centre(&mut v, disc);
    let steps0 = steps(&v);
    assert!(v.drag_begin(was.0, was.1, FREE, 4.0, 6.0));
    v.drag_update(was.0 + 30.0, was.1, FREE, REACH);
    v.drag_end().unwrap();
    let now = centre(&mut v, disc);
    let moved = (now.0 - was.0 - 30.0).abs() < 0.5 && (now.1 - was.1).abs() < 0.5;
    let one = steps(&v) == steps0 + 1;
    // The group's box is the union of its children's boxes again.
    let g = v.doc().unwrap().object(group_ids::TURNED).unwrap();
    let union = g
        .children()
        .iter()
        .map(|c| c.outer_box())
        .reduce(|a, b| a.union(&b))
        .unwrap();
    let refit = g.frame.near(&union);
    // Undo puts everything back exactly.
    let before = quadrule_model::sample::groups_document();
    v.undo();
    let undone = *v.doc().unwrap() == before;
    // The arrow keys carry the step back through the turn: one pixel right is one pixel right.
    v.select_row(disc, false);
    let c0 = centre(&mut v, disc);
    v.nudge(1.0, 0.0, false).unwrap();
    let c1 = centre(&mut v, disc);
    let nudged = (c1.0 - c0.0 - 1.0).abs() < 0.05 && (c1.1 - c0.1).abs() < 0.05;
    let ok = moved && one && refit && undone && nudged;
    println!(
        "\nCHECK crate=ui name=child_in_turned_group result={} moved_with_pointer={moved} one_undo_step={one} box_refit={refit} undo_exact={undone} nudge_in_page_pixels={nudged}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn make_and_break_groups_by_key_and_clipboard() {
    let mut h = harness_with(open());
    {
        let v = &mut h.state_mut().ui.viewer;
        v.selection = vec![group_ids::HALF, group_ids::PINNED];
    }
    let before = h.state().ui.viewer.doc().unwrap().clone();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::G]);
    h.run();
    let v = &mut h.state_mut().ui.viewer;
    let gid = v.selection[0];
    let grouped = v.selection.len() == 1
        && v.doc()
            .unwrap()
            .object(gid)
            .is_some_and(|g| g.is_group() && g.children().len() == 2)
        && v.doc().unwrap().parent_of(group_ids::HALF) == Some(gid);
    let named = v.doc().unwrap().object(gid).unwrap().name == "Group";
    // Copy the group, paste it: the whole tree, all new ids.
    let text = v.copy_text().unwrap();
    let before_count = v.doc().unwrap().objects().count();
    assert!(v.paste_text(&text).unwrap());
    let added = v.doc().unwrap().objects().count() - before_count;
    let ids: std::collections::BTreeSet<_> = v.doc().unwrap().objects().map(|o| o.id).collect();
    let unique = ids.len() == v.doc().unwrap().objects().count();
    let subtree = 1 + v
        .doc()
        .unwrap()
        .objects()
        .filter(|o| v.doc().unwrap().ancestors(o.id).contains(&gid))
        .count();
    let tree_copied = added == subtree && subtree > 5;
    v.undo();
    // Ctrl+Shift+G breaks it again.
    v.selection = vec![gid];
    h.key_combination_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, &[Key::G]);
    h.run();
    let v = &mut h.state_mut().ui.viewer;
    let broken = v.doc().unwrap().object(gid).is_none()
        && v.selection.len() == 2
        && v.doc().unwrap().layers[1]
            .objects
            .iter()
            .any(|o| o.id == group_ids::HALF);
    // Two undo steps back to the start.
    v.undo();
    v.undo();
    let back = *v.doc().unwrap() == before;
    let ok = grouped && named && unique && tree_copied && broken && back;
    println!(
        "\nCHECK crate=ui name=group_ungroup_keys result={} ctrl_g_groups={grouped} named_from_fluent={named} paste_all_new_ids={unique} ctrl_shift_g_breaks={broken} undo_back_to_start={back}",
        pass(ok)
    );
    assert!(ok);
}

struct Window {
    ui: ViewerUi,
}

fn harness_with(v: Viewer) -> Harness<'static, Window> {
    harness_sized(v, 860.0)
}

fn harness_sized(v: Viewer, height: f32) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, height))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window {
                ui: ViewerUi::new(v),
            },
        );
    h.run();
    h
}

#[test]
fn the_layers_tree_shows_folds_and_picks_at_depth() {
    // Only the nested group on the page, so every row fits the small layers panel.
    let mut v = open();
    {
        let o = v.open.as_mut().unwrap();
        o.doc.layers[1]
            .objects
            .retain(|o| o.id == group_ids::NESTED);
        o.doc.layers[0].objects.clear();
    }
    let mut h = harness_with(v);
    // The tree shows a group's children under it; its fold arrow is named for screen readers, and
    // folding hides them.
    let shows = h.query_by_label("Inner group").is_some() && h.query_by_label("Star").is_some();
    let arrow_named = h
        .query_by_label("Show what is inside Outer group")
        .is_some();
    h.get_by_label("Show what is inside Outer group").click();
    h.run();
    h.run();
    let folded = h.state().ui.is_folded(group_ids::NESTED)
        && h.query_by_label("Inner group").is_none()
        && h.query_by_label("Outer group").is_some();
    // Picking a row selects at any depth, entering its groups.
    h.get_by_label("Show what is inside Outer group").click();
    h.run();
    h.run();
    h.get_by_label("Star").click();
    h.run();
    let row = h.state().ui.viewer.selection == vec![group_ids::nth(5)]
        && h.state().ui.viewer.entered
            == vec![group_ids::NESTED, group_ids::nth(3), group_ids::nth(4)];
    let ok = shows && arrow_named && folded && row;
    println!(
        "\nCHECK crate=ui name=layers_tree_rows result={} children_shown={shows} fold_arrow_named={arrow_named} folds={folded} row_picks_at_depth={row}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn objects_move_into_and_out_of_groups_in_the_layers_tree() {
    let mut v = open();
    // Moves: the line of the half group goes into the pinned group, then out to the top of the
    // layer; one undo step each. Dropped on the middle of a group row it goes in on top.
    let slash = group_ids::nth(33);
    let before = v.doc().unwrap().clone();
    let (place, index) = quadrule_ui::row_drop_place(
        &before,
        slash,
        group_ids::PINNED,
        quadrule_ui::RowZone::Inside,
    )
    .unwrap();
    let top = before.object(group_ids::PINNED).unwrap().children().len();
    let target_ok = place == Place::Group(group_ids::PINNED) && index == top;
    let s0 = steps(&v);
    v.move_object_into(slash, &place, index).unwrap();
    let inside =
        v.doc().unwrap().parent_of(slash) == Some(group_ids::PINNED) && steps(&v) == s0 + 1;
    // The half group's box refit round the two left.
    let shrank = v
        .doc()
        .unwrap()
        .object(group_ids::HALF)
        .unwrap()
        .children()
        .len()
        == 2;
    v.move_object_to(slash, 1, 0).unwrap();
    let out = v.doc().unwrap().parent_of(slash).is_none() && steps(&v) == s0 + 2;
    v.undo();
    v.undo();
    let undone = *v.doc().unwrap() == before;
    // Into itself or its own child: no place. Across the edge of a turned group: refused with a
    // note, nothing changes.
    let itself = quadrule_ui::row_drop_place(
        &before,
        group_ids::NESTED,
        group_ids::nth(5),
        quadrule_ui::RowZone::Inside,
    )
    .is_none();
    v.notice = None;
    v.move_object_into(slash, &Place::Group(group_ids::TURNED), 0)
        .unwrap();
    let refused = *v.doc().unwrap() == before && v.notice.as_ref().is_some_and(|n| n.error);
    let ok = target_ok && inside && shrank && out && undone && itself && refused;
    println!(
        "\nCHECK crate=ui name=layers_tree_moves result={} drop_inside_group_on_top={inside} out_to_layer={out} undo={undone} not_into_itself={itself} turned_edge_refused={refused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_inspector_for_a_group_and_the_new_controls_are_named() {
    let mut h = harness_with(open());
    {
        let v = &mut h.state_mut().ui.viewer;
        v.selection = vec![group_ids::HALF];
    }
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    // Style: Opacity only, no stroke, fill, dash or cap.
    let opacity_only = h.query_by_label("Opacity").is_some()
        && h.query_by_label("Dash").is_none()
        && h.query_by_label("Cap").is_none()
        && h.query_by_label("Fill").is_none();
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let rotation = h.query_by_label("Rotation").is_some();
    let buttons =
        h.query_all_by_label("Group").count() >= 2 && h.query_all_by_label("Ungroup").count() >= 2;
    // Constraints inside a group read "against the group".
    {
        let v = &mut h.state_mut().ui.viewer;
        v.selection = vec![group_ids::PINNED_CHILD];
    }
    h.state_mut().ui.tab = Tab::Constraints;
    h.run();
    h.run();
    let against = h
        .query_by_label_contains("work against the group \"Pinned group\"")
        .is_some();
    let ok = opacity_only && rotation && buttons && against;
    println!(
        "\nCHECK crate=ui name=group_inspector result={} style_is_opacity_only={opacity_only} rotation_field={rotation} group_and_ungroup_buttons_named={buttons} constraints_say_against_the_group={against}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_group_with_a_pattern_greys_the_rotation_field_with_a_note() {
    let mut v = open();
    let pattern = {
        let mut o = v.doc().unwrap().object(group_ids::nth(32)).unwrap().clone();
        o.id = Uuid::from_u128(0x7A77);
        o.kind = ObjectKind::Pattern(quadrule_model::PatternRegion {
            pattern: quadrule_model::Pattern::Lines,
            extra: quadrule_model::Extra::new(),
        });
        o
    };
    v.edit(
        Command::InsertIn {
            parent: group_ids::HALF,
            index: 0,
            object: pattern,
        },
        None,
    )
    .unwrap();
    v.selection = vec![group_ids::HALF];
    let mut h = harness_with(v);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let note = h.query_by_label_contains("cannot turn").is_some();
    let field = h.query_by_label("Rotation").is_some();
    println!(
        "\nCHECK crate=ui name=group_rotation_refusal result={} note_shown={note} field_named={field}",
        pass(note && field)
    );
    assert!(note && field);
}

#[test]
fn the_benchmarks_run_over_the_groups_page() {
    // `quadrule --groups-page --bench-drag` and `--bench-draw`: headless they finish, time their
    // frames and leave the page as it was.
    let want = quadrule_model::sample::groups_document();
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 2] = [
        ("drag", quadrule_ui::DragBench::new(12).with_groups()),
        (
            "draw_rect",
            quadrule_ui::DragBench::draw(12)
                .with_groups()
                .with_tool(quadrule_ui::Tool::Rect),
        ),
    ];
    for (name, bench) in runs {
        let mut h = harness_with(Viewer::default());
        h.state_mut().ui.bench = Some(bench);
        let mut steps = 0;
        while !h.state().ui.bench_finished() && steps < 80 {
            h.step();
            steps += 1;
        }
        let ui = &h.state().ui;
        let done = ui.bench_finished();
        let timed = ui.drag_clock.frames > 0 && ui.drag_clock.mean_ms() > 0.0;
        let clean = ui.viewer.doc().is_some_and(|d| *d == want)
            && !ui.viewer.is_dirty()
            && !ui.viewer.dragging();
        ok &= done && timed && clean;
        parts.push(format!(
            "{name}:finished={done}:frames={}:clean={clean}",
            ui.drag_clock.frames
        ));
    }
    println!(
        "\nCHECK crate=ui name=window_groups_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_group_resize_stops_at_zero_size_and_does_not_flip() {
    // Pulling the bottom-right handle of the heavy group far past its top-left corner leaves it
    // one device pixel across, not mirrored; its children keep their order and stay the right way
    // round (rung 4a debt, paid for real groups).
    let mut v = open();
    let before = v.doc().unwrap().clone();
    v.selection = vec![group_ids::HEAVY];
    let se = handle_at(&mut v, Handle::SE);
    assert!(v.drag_begin(se.0, se.1, FREE, 4.0, 6.0));
    v.drag_update(se.0 - 4000.0, se.1 - 4000.0, FREE, REACH);
    v.drag_end().unwrap();
    let k = px_per_mm();
    let g = frame(&v, group_ids::HEAVY);
    let tiny = g.w_mm >= 0.0 && g.w_mm <= 2.0 / k && g.h_mm >= 0.0 && g.h_mm <= 2.0 / k;
    let g0 = before.object(group_ids::HEAVY).unwrap();
    let upright = g.x_mm == g0.frame.x_mm && g.y_mm == g0.frame.y_mm;
    let order = before
        .leaves()
        .filter(|o| before.ancestors(o.id).contains(&group_ids::HEAVY))
        .all(|o| {
            let n = v.doc().unwrap().object(o.id).unwrap();
            n.frame.w_mm >= 0.0 && n.frame.h_mm >= 0.0
        });
    let ok = tiny && upright && order;
    println!(
        "\nCHECK crate=ui name=group_resize_stops_at_zero result={} width_mm={:.4} height_mm={:.4} not_flipped={upright} children_not_mirrored={order}",
        pass(ok),
        g.w_mm,
        g.h_mm
    );
    assert!(ok);
}
