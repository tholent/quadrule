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

//! Rung 8a: several devices in the editor (docs/bbq/2026-11-12-rung-8-several-devices.md, Q3, Q4,
//! Q5 and the Defaulted items). Headless. Each test prints one structured `CHECK` line starting
//! with a newline (run with `-- --nocapture`). Coordinates are device pixels of the device on
//! screen.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_layout::align::{Edge, Spread};
use quadrule_layout::units::Axis;
use quadrule_model::edit::OverrideField;
use quadrule_model::sample::*;
use quadrule_model::{Frame, Object, Uuid};
use quadrule_ui::{ArrangeOp, DragMods, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const FREE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn sample() -> Viewer {
    let mut v = Viewer::default();
    v.open_sample();
    v
}

fn devices() -> Viewer {
    let mut v = Viewer::default();
    v.open_devices();
    v
}

fn obj(v: &Viewer, id: Uuid) -> Object {
    v.doc().unwrap().object(id).unwrap().clone()
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

/// The export pixels of device `id` now.
fn pixels(v: &mut Viewer, id: &str) -> Vec<u8> {
    v.page_for(id).unwrap().image.data.clone()
}

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

#[test]
fn a_move_resize_nudge_font_size_and_eye_on_the_nomad_touch_only_the_nomad() {
    let mut v = sample();
    let manta_before = pixels(&mut v, MANTA_ID);
    let design_before = v.doc().unwrap().clone();
    v.set_profile(NOMAD_ID);
    let nomad_before = pixels(&mut v, NOMAD_ID);

    // Nudge: a position override and no size.
    v.selection = vec![ids::RULE];
    v.nudge(3.0, 2.0, false).unwrap();
    let ov = obj(&v, ids::RULE).overrides.get(NOMAD_ID).cloned();
    let nudge = ov
        .as_ref()
        .is_some_and(|o| o.position.is_some() && o.size.is_none());

    // A resize by the fields: a move leaves the size alone, a resize writes both.
    v.selection = vec![ids::DOTS];
    v.edit_frames_on_device(None, |f| Frame {
        x_mm: f.x_mm + 4.0,
        ..f
    })
    .unwrap();
    let moved = {
        let o = obj(&v, ids::DOTS);
        let ov = &o.overrides[NOMAD_ID];
        ov.position.is_some() && ov.size.is_none()
    };
    v.edit_frames_on_device(None, |f| Frame {
        w_mm: f.w_mm - 10.0,
        ..f
    })
    .unwrap();
    let resized = {
        let o = obj(&v, ids::DOTS);
        let ov = &o.overrides[NOMAD_ID];
        ov.position.is_some() && ov.size.is_some()
    };

    // Font size and the eye.
    v.selection = vec![ids::TITLE];
    v.set_font_size_on_device(None, 20.0).unwrap();
    let font = obj(&v, ids::TITLE).overrides[NOMAD_ID].font_size_pt == Some(20.0);
    v.set_visible_on_device(ids::RULE, false).unwrap();
    let eye = obj(&v, ids::RULE).overrides[NOMAD_ID].visible == Some(false);

    // The design is untouched and the Manta's pixels are the same bytes; the Nomad changed.
    let design_same = {
        let now = v.doc().unwrap();
        design_before.layers.iter().zip(&now.layers).all(|(a, b)| {
            a.objects
                .iter()
                .zip(&b.objects)
                .all(|(x, y)| x.frame == y.frame && x.visible == y.visible && x.kind == y.kind)
        })
    };
    let manta_same = pixels(&mut v, MANTA_ID) == manta_before;
    let nomad_changed = pixels(&mut v, NOMAD_ID) != nomad_before;
    // Each was one undo step (nudge, move, resize, font, eye).
    let five = steps(&v) == 5;
    let ok = nudge
        && moved
        && resized
        && font
        && eye
        && design_same
        && manta_same
        && nomad_changed
        && five;
    println!(
        "\nCHECK crate=ui name=edits_on_secondary_set_only_its_override result={} nudge_is_position={nudge} move_is_position_only={moved} resize_is_both={resized} font_size={font} eye={eye} design_untouched={design_same} manta_bytes_unchanged={manta_same} nomad_changed={nomad_changed} undo_steps={}",
        pass(ok),
        steps(&v)
    );
    assert!(ok);
}

#[test]
fn a_style_edit_on_the_nomad_changes_the_design_everywhere_and_says_so() {
    let mut v = sample();
    let manta_before = pixels(&mut v, MANTA_ID);
    v.set_profile(NOMAD_ID);
    let nomad_before = pixels(&mut v, NOMAD_ID);
    v.selection = vec![ids::DOTS];
    v.edit_selected(None, |o| o.opacity_pct = 40).unwrap();
    let shared = obj(&v, ids::DOTS).opacity_pct == 40 && obj(&v, ids::DOTS).overrides.is_empty();
    let both = pixels(&mut v, MANTA_ID) != manta_before && pixels(&mut v, NOMAD_ID) != nomad_before;
    let said = v
        .notice
        .as_ref()
        .is_some_and(|n| n.message.id == "notice-shared-edit" && !n.error);
    // On the primary device nothing is said.
    v.undo();
    v.set_profile(MANTA_ID);
    v.notice = None;
    v.edit_selected(None, |o| o.opacity_pct = 40).unwrap();
    let quiet = v.notice.is_none();
    let ok = shared && both && said && quiet;
    println!(
        "\nCHECK crate=ui name=style_edit_on_secondary_is_shared result={} design_changed_no_override={shared} both_devices_changed={both} status_says_so={said} quiet_on_primary={quiet}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_drag_on_the_nomad_writes_overrides_and_never_unresolves_into_the_design() {
    let mut v = sample();
    let manta_before = pixels(&mut v, MANTA_ID);
    let design = obj(&v, ids::DOTS).frame;
    v.set_profile(NOMAD_ID);
    let (cx, cy) = centre(&mut v, ids::DOTS);
    assert!(v.drag_begin(cx, cy, FREE, 4.0, 6.0));
    v.drag_update(cx + 60.0, cy + 40.0, FREE, REACH);
    v.drag_end().unwrap();
    let o = obj(&v, ids::DOTS);
    let ov = o.overrides.get(NOMAD_ID);
    let wrote = ov.is_some_and(|x| x.position.is_some() && x.size.is_none());
    let design_same = o.frame == design;
    let manta_same = pixels(&mut v, MANTA_ID) == manta_before;
    // A handle drag resizes: both parts.
    v.selection = vec![ids::DOTS];
    let handles = v.handles();
    let (hit, hx, hy) = handles
        .iter()
        .copied()
        .find(|(h, ..)| matches!(h, quadrule_ui::HandleHit::Box(b) if b.x_side() == 1 && b.y_side() == 1))
        .unwrap();
    assert!(v.drag_begin(hx, hy, FREE, 4.0, 6.0));
    let _ = hit;
    v.drag_update(hx + 30.0, hy + 30.0, FREE, REACH);
    v.drag_end().unwrap();
    let both = obj(&v, ids::DOTS).overrides[NOMAD_ID].size.is_some();
    // Undo twice: back to nothing.
    v.undo();
    v.undo();
    let back = obj(&v, ids::DOTS).overrides.is_empty() && steps(&v) == 0;
    let ok = wrote && design_same && manta_same && both && back;
    println!(
        "\nCHECK crate=ui name=drag_on_secondary_writes_overrides result={} move_writes_position={wrote} design_frame_same={design_same} manta_bytes_same={manta_same} resize_writes_size={both} undo_clears={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn dragging_a_group_on_the_nomad_writes_one_override_on_the_group() {
    // Pays the rung 7a debt: the group's override is its box; the children are not touched.
    let mut v = Viewer::default();
    v.open_groups();
    let g = group_ids::NESTED;
    let kids_before: Vec<Frame> = [2, 3, 4, 5, 6, 7]
        .iter()
        .map(|n| obj(&v, group_ids::nth(*n)).frame)
        .collect();
    let manta_before = pixels(&mut v, MANTA_ID);
    v.set_profile(NOMAD_ID);
    let (cx, cy) = centre(&mut v, g);
    v.click(cx, cy, 4.0, false);
    assert_eq!(v.selection, vec![g]);
    assert!(v.drag_begin(cx, cy, FREE, 4.0, 6.0));
    v.drag_update(cx + 50.0, cy + 30.0, FREE, REACH);
    v.drag_end().unwrap();
    let overridden: Vec<Uuid> = v
        .doc()
        .unwrap()
        .objects()
        .filter(|o| o.overrides.contains_key(NOMAD_ID))
        .map(|o| o.id)
        .collect();
    let one = overridden == vec![g];
    let kids_same: Vec<Frame> = [2, 3, 4, 5, 6, 7]
        .iter()
        .map(|n| obj(&v, group_ids::nth(*n)).frame)
        .collect();
    let children_untouched = kids_same == kids_before;
    // The group's box on the Nomad moved by the drag.
    let moved = {
        let (nx, ny) = centre(&mut v, g);
        (nx - cx - 50.0).abs() < 1.5 && (ny - cy - 30.0).abs() < 1.5
    };
    let manta_same = pixels(&mut v, MANTA_ID) == manta_before;
    let ok = one && children_untouched && moved && manta_same;
    println!(
        "\nCHECK crate=ui name=group_drag_on_secondary result={} one_override_on_the_group={one} children_untouched={children_untouched} box_followed_pointer={moved} manta_bytes_same={manta_same}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn reset_and_reset_all_are_one_undo_step_each() {
    let mut v = devices();
    v.set_profile(NOMAD_ID);
    let before = steps(&v);
    // One part of one object.
    v.reset_overrides(&[device_ids::SIZED], Some(&[OverrideField::Size]))
        .unwrap();
    let o = obj(&v, device_ids::SIZED);
    let one_part = o.overrides[NOMAD_ID].size.is_none() && o.overrides[NOMAD_ID].position.is_some();
    // Everything on two objects.
    v.reset_overrides(&[device_ids::MOVED, device_ids::HIDDEN], None)
        .unwrap();
    let all = obj(&v, device_ids::MOVED).overrides.is_empty()
        && obj(&v, device_ids::HIDDEN).overrides.is_empty();
    let two_steps = steps(&v) == before + 2;
    v.undo();
    v.undo();
    let restored = v.doc().unwrap() == &quadrule_model::sample::devices_document();
    // Nothing on the primary device to reset.
    v.set_profile(MANTA_ID);
    let n = steps(&v);
    v.reset_overrides(&[device_ids::SIZED], None).unwrap();
    let primary_noop = steps(&v) == n;
    let ok = one_part && all && two_steps && restored && primary_noop;
    println!(
        "\nCHECK crate=ui name=reset_and_reset_all result={} one_part={one_part} all_parts={all} one_step_each={two_steps} undo_restores={restored} primary_noop={primary_noop}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn align_on_the_nomad_writes_position_overrides_there() {
    let mut v = devices();
    v.set_profile(NOMAD_ID);
    let design: Vec<Frame> = [device_ids::MOVED, device_ids::SIZED, device_ids::HIDDEN]
        .iter()
        .map(|id| obj(&v, *id).frame)
        .collect();
    let manta_before = pixels(&mut v, MANTA_ID);
    v.selection = vec![device_ids::MOVED, device_ids::SIZED];
    v.snap.pixel_grid = false;
    v.arrange(ArrangeOp::Align(Axis::X, Edge::Start)).unwrap();
    let left = |v: &mut Viewer, id: Uuid| {
        v.page()
            .unwrap()
            .placed
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .node
            .frame
            .x_mm
    };
    let (a, b) = (
        left(&mut v, device_ids::MOVED),
        left(&mut v, device_ids::SIZED),
    );
    let aligned = (a - b).abs() < 1e-6;
    let overrides = obj(&v, device_ids::MOVED).overrides[NOMAD_ID]
        .position
        .is_some()
        && obj(&v, device_ids::SIZED).overrides[NOMAD_ID]
            .position
            .is_some();
    let design_same = [device_ids::MOVED, device_ids::SIZED, device_ids::HIDDEN]
        .iter()
        .zip(&design)
        .all(|(id, f)| obj(&v, *id).frame == *f);
    let manta_same = pixels(&mut v, MANTA_ID) == manta_before;
    // Distribute works on the same boxes (three objects).
    v.selection = vec![device_ids::MOVED, device_ids::SIZED, device_ids::HIDDEN];
    v.arrange(ArrangeOp::Distribute(Axis::X, Spread::Centres))
        .unwrap();
    let one_step_each = steps(&v) <= 2;
    let ok = aligned && overrides && design_same && manta_same && one_step_each;
    println!(
        "\nCHECK crate=ui name=align_on_secondary result={} left_edges_meet={aligned} position_overrides={overrides} design_untouched={design_same} manta_bytes_same={manta_same} one_step_each={one_step_each}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn copy_paste_duplicate_and_ungroup_carry_overrides() {
    let mut v = devices();
    // Duplicate: the copy carries the overrides.
    v.selection = vec![device_ids::SIZED];
    v.duplicate_selection().unwrap();
    let copy = v.selection[0];
    let dup = obj(&v, copy).overrides == obj(&v, device_ids::SIZED).overrides;
    // Copy and paste: carries them too, moved by the paste offset on the Nomad as on the Manta.
    v.selection = vec![device_ids::MOVED];
    let text = v.copy_text().unwrap();
    assert!(v.paste_text(&text).unwrap());
    let pasted = obj(&v, v.selection[0]);
    let orig = obj(&v, device_ids::MOVED);
    let (p, o) = (
        pasted.overrides[NOMAD_ID].position.unwrap(),
        orig.overrides[NOMAD_ID].position.unwrap(),
    );
    let doc = v.doc().unwrap();
    let carried = (p.x_mm - o.x_mm - doc.rhythm.h_unit_mm).abs() < 1e-9
        && (p.y_mm - o.y_mm - doc.rhythm.v_unit_mm).abs() < 1e-9;
    // Ungroup bakes the group's override into the children; neither device's pixels change.
    let (m0, n0) = (pixels(&mut v, MANTA_ID), pixels(&mut v, NOMAD_ID));
    v.selection = vec![device_ids::GROUP];
    v.set_profile(NOMAD_ID);
    let n = steps(&v);
    v.ungroup_selection().unwrap();
    let one_step = steps(&v) == n + 1;
    let kept = obj(&v, device_ids::GROUP_CHILD)
        .overrides
        .contains_key(NOMAD_ID)
        && obj(&v, device_ids::nth(6)).overrides.contains_key(NOMAD_ID);
    let same = pixels(&mut v, MANTA_ID) == m0 && pixels(&mut v, NOMAD_ID) == n0;
    // Group again: the children keep theirs, and nothing moves.
    v.selection = vec![device_ids::nth(6), device_ids::GROUP_CHILD];
    v.group_selection().unwrap();
    let regrouped = pixels(&mut v, NOMAD_ID) == n0
        && obj(&v, device_ids::nth(6)).overrides.contains_key(NOMAD_ID);
    let ok = dup && carried && one_step && kept && same && regrouped;
    println!(
        "\nCHECK crate=ui name=clipboard_duplicate_ungroup_carry_overrides result={} duplicate={dup} paste_shifted_on_nomad={carried} ungroup_one_step={one_step} children_have_overrides={kept} pixels_same={same} group_again_same={regrouped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn make_primary_and_the_ticks_through_the_viewer() {
    let mut v = devices();
    let (m0, n0) = (pixels(&mut v, MANTA_ID), pixels(&mut v, NOMAD_ID));
    let n = steps(&v);
    v.make_primary(NOMAD_ID).unwrap();
    let doc = v.doc().unwrap();
    let switched = doc.primary_profile_id == NOMAD_ID && steps(&v) == n + 1;
    let (m1, n1) = (pixels(&mut v, MANTA_ID), pixels(&mut v, NOMAD_ID));
    let same = m0 == m1 && n0 == n1;
    // The Nomad is the primary device now: an edit with it on screen is the design's.
    v.set_profile(NOMAD_ID);
    let design_edit = v.override_device().is_none();
    v.undo();
    let back = v.doc().unwrap().primary_profile_id == MANTA_ID;
    // One device only; undo brings the overrides back.
    v.set_targets(&[MANTA_ID]).unwrap();
    let one = v.doc().unwrap().profile_ids() == vec![MANTA_ID]
        && !v.doc().unwrap().has_overrides_on(NOMAD_ID)
        && v.profile_id == MANTA_ID;
    v.undo();
    let returned = v.doc().unwrap().overridden_count(NOMAD_ID) == 6;
    let ok = switched && same && design_edit && back && one && returned;
    println!(
        "\nCHECK crate=ui name=make_primary_and_device_ticks result={} switched_one_step={switched} no_pixel_changed={same} edits_are_design_on_new_primary={design_edit} undo_back={back} one_device_only={one} undo_brings_overrides_back={returned}",
        pass(ok)
    );
    assert!(ok);
}

fn window() -> Harness<'static, ViewerUi> {
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_devices();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    h
}

#[test]
fn the_strip_shows_each_device_and_a_click_puts_one_on_screen() {
    let mut h = window();
    let on_manta = h.state().viewer.profile_id == MANTA_ID;
    // Both pictures, named for screen readers, and the count of what is changed on the Nomad.
    let manta_pic = h.query_by_label("Show Manta").is_some();
    let nomad_pic = h.query_by_label("Show Nomad").is_some();
    let count = h.query_by_label_contains("6 changes").is_some();
    h.get_by_label("Show Nomad").click();
    h.run();
    let on_nomad = h.state().viewer.profile_id == NOMAD_ID;
    // The strip and the device picker are one state (the picker reads `profile_id` too).
    let picker = h.state().viewer.profile_ids() == vec![MANTA_ID, NOMAD_ID];
    // Fold: the pictures go, the header stays; the fold is a setting that stays.
    h.get_by_label("Hide the device previews").click();
    h.run();
    let folded = !h.state().viewer.show_strip && h.query_by_label("Show Manta").is_none();
    h.get_by_label("Show the device previews").click();
    h.run();
    let back = h.state().viewer.show_strip && h.query_by_label("Show Manta").is_some();
    let ok = on_manta && manta_pic && nomad_pic && count && on_nomad && picker && folded && back;
    println!(
        "\nCHECK crate=ui name=strip result={} on_manta_first={on_manta} both_pictures={manta_pic}/{nomad_pic} nomad_count_shown={count} click_shows_nomad={on_nomad} picker_agrees={picker} fold_hides={folded} unfold_shows={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_overrides_tab_and_the_devices_list_are_named_controls() {
    let mut h = window();
    // Nothing selected: the Page tab lists the devices, the primary marked, Make primary on the other.
    let list = h.query_by_label("Target Manta").is_some()
        && h.query_by_label("Target Nomad").is_some()
        && h.query_by_label("Make Nomad the primary device").is_some();
    // With the Manta on screen an object has no Overrides tab.
    h.state_mut().viewer.selection = vec![device_ids::SIZED];
    h.run();
    let none_on_primary = h.query_by_label("Overrides").is_none();
    h.get_by_label("Show Nomad").click();
    h.run();
    let tab = h.query_by_label("Overrides").is_some();
    h.get_by_label("Overrides").click();
    h.run();
    let rows = h.query_by_label("Reset Position on Nomad").is_some()
        && h.query_by_label("Reset Size on Nomad").is_some();
    h.get_by_label("Reset Size on Nomad").click();
    h.run();
    let reset = h
        .state()
        .viewer
        .doc()
        .unwrap()
        .object(device_ids::SIZED)
        .unwrap()
        .overrides[NOMAD_ID]
        .size
        .is_none();
    h.get_by_label("Reset all changes on Nomad").click();
    h.run();
    let all = h
        .state()
        .viewer
        .doc()
        .unwrap()
        .object(device_ids::SIZED)
        .unwrap()
        .overrides
        .is_empty();
    // The Arrange tab's fields carry the dot, named "Changed on Nomad", for an overridden value.
    h.state_mut().viewer.selection = vec![device_ids::MOVED];
    h.run();
    h.get_by_label("Arrange").click();
    h.run();
    // The layers row and both position fields carry one.
    let dot = h.query_all_by_label("Changed on Nomad").count() >= 3;
    let ok = list && none_on_primary && tab && rows && reset && all && dot;
    println!(
        "\nCHECK crate=ui name=overrides_tab_and_device_list result={} devices_list={list} no_tab_on_primary={none_on_primary} tab_on_secondary={tab} reset_buttons_named={rows} reset_one={reset} reset_all={all} badge_named={dot}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn unticking_a_device_with_overrides_asks_first() {
    let mut h = window();
    h.get_by_label("Target Nomad").click();
    h.run();
    // Asked, nothing taken out yet.
    let asked = h.query_by_label("Take Nomad out of this page?").is_some()
        && h.state().viewer.doc().unwrap().profile_ids().len() == 2;
    h.get_by_label("Keep it").click();
    h.run();
    let kept = h.state().viewer.doc().unwrap().profile_ids().len() == 2;
    h.get_by_label("Target Nomad").click();
    h.run();
    h.get_by_label("Take out").click();
    h.run();
    let gone = h.state().viewer.doc().unwrap().profile_ids().len() == 1
        && !h.state().viewer.doc().unwrap().has_overrides_on(NOMAD_ID);
    h.state_mut().viewer.undo();
    h.run();
    let back = h.state().viewer.doc().unwrap().overridden_count(NOMAD_ID) == 6;
    let ok = asked && kept && gone && back;
    println!(
        "\nCHECK crate=ui name=untick_asks_first result={} asked={asked} keep_leaves_it={kept} take_out_drops_overrides={gone} undo_restores={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_style_tab_reads_mixed_where_the_picked_objects_differ() {
    // Pays the rung 6a/7b debt (rung 8a, Defaulted "Debt paid in code the rung touches").
    let mut h = window();
    {
        let v = &mut h.state_mut().viewer;
        v.selection = vec![device_ids::SIZED];
        v.edit_selected(None, |o| o.opacity_pct = 40).unwrap();
        v.selection = vec![device_ids::SIZED, device_ids::MOVED];
    }
    h.state_mut().tab = quadrule_ui::Tab::Style;
    h.run();
    h.run();
    let mixed: Vec<String> = h
        .query_all_by_value("mixed")
        .filter_map(|n| n.accesskit_node().label().map(|l| l.to_string()))
        .collect();
    let opacity = mixed.iter().any(|l| l == "Opacity");
    // The two have the same role, so the role does not read mixed.
    let same_role = !mixed.iter().any(|l| l == "Role");
    // One picked: nothing is mixed.
    h.state_mut().viewer.selection = vec![device_ids::SIZED];
    h.run();
    h.run();
    let none = h.query_all_by_value("mixed").count() == 0;
    let ok = opacity && same_role && none;
    println!(
        "\nCHECK crate=ui name=style_tab_reads_mixed result={} opacity_mixed={opacity} equal_fields_not_mixed={same_role} one_picked_not_mixed={none} labels={mixed:?}",
        pass(ok)
    );
    assert!(ok);
}
