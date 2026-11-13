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

//! Rung 10: the repeat tool in the editor (docs/bbq/2026-11-13-rung-10-repeat-tool.md, Q2, Q3, Q7,
//! Defaulted "Making a repeat", "Snapping", "The Repeat tab"). Headless. Each test prints one
//! structured `CHECK` line starting with a newline (run with `-- --nocapture`).
//!
//! `egui_kittest` cannot make a double-click, so going into a repeat is tested through
//! [`Viewer::enter_at`], which the window calls on a double-click. Coordinates are Manta device
//! pixels.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_layout::snap::Handle;
use quadrule_model::edit::Command;
use quadrule_model::sample::*;
use quadrule_model::{Frame, ObjectKind, Spacing, Uuid};
use quadrule_ui::{DragMods, HandleHit, Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const FREE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn open() -> Viewer {
    let mut v = Viewer::default();
    v.open_repeats();
    v
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

fn source_of(rep: Uuid) -> Uuid {
    rep_source(&open(), rep)
}

fn rep_source(v: &Viewer, rep: Uuid) -> Uuid {
    v.doc().unwrap().object(rep).unwrap().children()[0].id
}

/// The centre of the `n`th (0-based) drawn copy of the source `id`, in Manta pixels.
fn copy_centre(v: &mut Viewer, id: Uuid, n: usize) -> (f64, f64) {
    let b = v
        .page()
        .unwrap()
        .placed
        .iter()
        .filter(|p| p.id == id)
        .nth(n)
        .unwrap()
        .shape
        .bounds();
    ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0)
}

fn page_bytes(v: &mut Viewer) -> Vec<u8> {
    v.page().unwrap().image.data.clone()
}

#[test]
fn the_repeat_button_wraps_the_selection_in_one_undo_step() {
    let mut v = Viewer::default();
    v.open_shapes();
    let ids = [shape_ids::nth(1), shape_ids::nth(2)];
    let before = v.doc().unwrap().clone();
    let s0 = steps(&v);
    // Several objects are grouped first, and the group is the source.
    v.selection = ids.to_vec();
    v.repeat_selection().unwrap();
    let rep = v.selection[0];
    let o = v.doc().unwrap().object(rep).unwrap().clone();
    let r = o.repeat().unwrap();
    let defaults = r.count == 4
        && !r.fill
        && r.h_spacing == Spacing::Auto
        && r.source.is_group()
        && r.source.children().len() == 2;
    let one_step = steps(&v) == s0 + 1;
    v.undo();
    let undone = *v.doc().unwrap() == before;
    // One object: it is the source itself.
    v.selection = vec![ids[0]];
    v.repeat_selection().unwrap();
    let rep = v.selection[0];
    let single = v.doc().unwrap().object(rep).unwrap().children()[0].id == ids[0];
    // Nothing picked: a note and no step.
    v.selection.clear();
    let n = steps(&v);
    let none = v.repeat_selection().is_err()
        && steps(&v) == n
        && v.notice.as_ref().is_some_and(|n| n.error);
    let ok = defaults && one_step && undone && single && none;
    println!(
        "\nCHECK crate=ui name=repeat_button result={} defaults={defaults} one_step={one_step} undone={undone} single_is_source={single} nothing_picked_notes={none}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn any_copy_picks_the_repeat_double_click_enters_and_escape_leaves() {
    let mut v = open();
    let (rep, src) = (repeat_ids::ACROSS, source_of(repeat_ids::ACROSS));
    // The fourth copy.
    let at = copy_centre(&mut v, src, 3);
    v.click(at.0, at.1, 4.0, false);
    let picks = v.selection == vec![rep] && v.entered.is_empty();
    // A double-click on a copy enters the repeat and selects the source.
    let entered = v.enter_at(at.0, at.1, 4.0) && v.entered == vec![rep] && v.selection == vec![src];
    // Escape steps out and selects the repeat.
    let left = v.leave_group() && v.entered.is_empty() && v.selection == vec![rep];
    // A click in the gap between two copies picks nothing (the 12 mm pitch leaves 4 mm).
    let mm_src = rep_source(&v, repeat_ids::MM_PITCH);
    let a = copy_centre(&mut v, mm_src, 0);
    let b = copy_centre(&mut v, mm_src, 1);
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    v.selection.clear();
    v.click(mid.0, mid.1, 1.0, false);
    let gap = v.selection.is_empty();
    let ok = picks && entered && left && gap;
    println!(
        "\nCHECK crate=ui name=repeat_pick_enter_leave result={} click_copy_picks_repeat={picks} double_click_enters_source={entered} escape_leaves={left} gap_misses={gap}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn editing_the_source_changes_every_copy_in_one_step() {
    let mut v = open();
    let (rep, src) = (repeat_ids::ACROSS, source_of(repeat_ids::ACROSS));
    let before: Vec<(f64, f64)> = (0..4).map(|n| copy_centre(&mut v, src, n)).collect();
    // Widen the source from 10 to 20 mm: the pitch grows from 2 to 4 units, and every copy follows.
    let mut s = v.doc().unwrap().object(src).unwrap().clone();
    s.frame.w_mm = 20.0;
    let s0 = steps(&v);
    v.edit(Command::SetObject(s), None).unwrap();
    let after: Vec<(f64, f64)> = (0..4).map(|n| copy_centre(&mut v, src, n)).collect();
    let k = quadrule_profiles::builtin(MANTA_ID).unwrap().mm_to_px(1.0);
    let step_before = before[1].0 - before[0].0;
    let step_after = after[1].0 - after[0].0;
    let follows = (step_before - 10.0 * k).abs() < 2.0 && (step_after - 20.0 * k).abs() < 2.0;
    let one_step = steps(&v) == s0 + 1;
    // The box is the new copies' outer box.
    let f = v.doc().unwrap().object(rep).unwrap().frame;
    let boxed = (f.w_mm - (3.0 * 20.0 + 20.0)).abs() < 1e-6;
    // Moving the source (entered) moves the whole repeat.
    let mut s = v.doc().unwrap().object(src).unwrap().clone();
    s.frame.x_mm += 7.0;
    v.edit(Command::SetObject(s), None).unwrap();
    let moved = (v.doc().unwrap().object(rep).unwrap().frame.x_mm - (f.x_mm + 7.0)).abs() < 1e-6;
    let ok = follows && one_step && boxed && moved;
    println!(
        "\nCHECK crate=ui name=repeat_live_edit result={} copies_follow={follows} one_step={one_step} box_refits={boxed} source_move_moves_repeat={moved}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn expand_makes_one_group_and_changes_no_pixel_on_the_primary() {
    let mut v = open();
    let before = page_bytes(&mut v);
    let doc = v.doc().unwrap().clone();
    v.selection = vec![repeat_ids::HABIT];
    let s0 = steps(&v);
    v.expand_selection().unwrap();
    let after = page_bytes(&mut v);
    let group = v.selection[0];
    let g = v.doc().unwrap().object(group).unwrap().clone();
    // 14 columns by 4 rows, with row 2 skipped: 42 copies, each a plain box (a copy of the source).
    let shape = g.is_group()
        && g.name == "Habit tracker"
        && g.children().len() == 42
        && g.children()
            .iter()
            .all(|c| matches!(c.kind, ObjectKind::Rect(_)));
    let ids_new = {
        let mut ids: Vec<Uuid> = g.children().iter().map(|c| c.id).collect();
        ids.sort();
        ids.dedup();
        ids.len() == 42 && ids.iter().all(|i| doc.object(*i).is_none())
    };
    let same_pixels = before == after;
    let one_step = steps(&v) == s0 + 1;
    let gone = v.doc().unwrap().object(repeat_ids::HABIT).is_none();
    let note = v.notice.as_ref().is_some_and(|n| !n.error);
    v.undo();
    let undone = *v.doc().unwrap() == doc;
    // The Fill repeat draws another number on the Nomad: the note says so.
    v.selection = vec![repeat_ids::FILL];
    v.expand_selection().unwrap();
    let differs = v
        .notice
        .as_ref()
        .is_some_and(|n| n.message.id == "notice-expanded-differs");
    let ok = shape && ids_new && same_pixels && one_step && gone && note && undone && differs;
    println!(
        "\nCHECK crate=ui name=repeat_expand result={} one_group={shape} new_ids={ids_new} primary_pixels_same={same_pixels} one_step={one_step} undone={undone} fill_note_on_other_device={differs}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn on_a_secondary_device_the_repeat_box_carries_the_override_not_the_source() {
    let mut v = open();
    let nomad = NOMAD_ID;
    v.set_profile(nomad);
    let (rep, src) = (repeat_ids::ACROSS, source_of(repeat_ids::ACROSS));
    // Picking the source works on the repeat there.
    v.selection = vec![src];
    let target = v.editable_selection() == vec![rep];
    let doc0 = v.doc().unwrap().clone();
    v.nudge(10.0, 0.0, false).unwrap();
    let doc = v.doc().unwrap();
    let on_box = doc
        .object(rep)
        .unwrap()
        .override_on(nomad)
        .is_some_and(|o| o.position.is_some());
    let none_on_source = doc.object(src).unwrap().overrides.is_empty();
    // The primary's design is untouched.
    let design = doc.object(rep).unwrap().frame == doc0.object(rep).unwrap().frame;
    let ok = target && on_box && none_on_source && design;
    println!(
        "\nCHECK crate=ui name=repeat_overrides result={} source_pick_targets_repeat={target} override_on_box={on_box} none_on_source={none_on_source} design_untouched={design}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn fill_handles_add_and_take_away_copies_and_a_counted_repeat_has_none() {
    let mut v = open();
    let rep = repeat_ids::FILL;
    let src = rep_source(&v, rep);
    // A counted repeat has no box to pull.
    v.selection = vec![repeat_ids::ACROSS];
    let counted_none = v.handles().is_empty();
    v.selection = vec![rep];
    let handles = v.handles();
    let e = handles
        .iter()
        .find(|(h, ..)| *h == HandleHit::Box(Handle::E))
        .copied()
        .unwrap();
    let count = |v: &mut Viewer| {
        v.page()
            .unwrap()
            .placed
            .iter()
            .filter(|p| p.id == src)
            .count()
    };
    let n0 = count(&mut v);
    let k = quadrule_profiles::builtin(MANTA_ID).unwrap().mm_to_px(1.0);
    // Pull the right edge in by 30 mm (six 5 mm steps): six copies fewer.
    let s0 = steps(&v);
    assert!(v.drag_begin(e.1, e.2, FREE, 4.0, 6.0));
    v.drag_update(e.1 - 30.0 * k, e.2, FREE, REACH);
    v.drag_end().unwrap();
    let n1 = count(&mut v);
    let fewer = n1 + 6 == n0 || n1 + 7 == n0 || n1 + 5 == n0;
    let one_step = steps(&v) == s0 + 1;
    // And out again.
    v.undo();
    let n2 = count(&mut v);
    let back = n2 == n0;
    let ok = counted_none && fewer && one_step && back && n1 < n0;
    println!(
        "\nCHECK crate=ui name=repeat_fill_handles result={} counted_has_no_handles={counted_none} copies_before={n0} after_pull={n1} one_step={one_step} undo_restores={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn copies_count_for_snapping_and_the_cap_is_said_in_the_status_bar() {
    let mut v = open();
    let src = rep_source(&v, repeat_ids::ACROSS);
    let placed = v.page().unwrap().placed.clone();
    let doc = v.doc().unwrap().clone();
    // Dragging something else lines up with all four copies: each is an "other".
    let others = quadrule_ui::drag::others_at(&doc, &placed, &[repeat_ids::TITLE], &[]);
    let copy_boxes = placed.iter().filter(|p| p.id == src).count();
    let counted = copy_boxes == 4
        && placed
            .iter()
            .filter(|p| p.id == src)
            .all(|p| others.iter().any(|o| o.rect.x0 == p.shape.bounds().0));
    // Dragging the repeat itself leaves its copies out.
    let own = quadrule_ui::drag::others_at(&doc, &placed, &[repeat_ids::ACROSS], &[]);
    let left_out = own.iter().all(|o| {
        !placed
            .iter()
            .filter(|p| p.id == src)
            .any(|p| o.rect.x0 == p.shape.bounds().0 && o.rect.y0 == p.shape.bounds().1)
    });
    // 100 x 100 copies is over the cap of 5,000: the page says so.
    let mut big = Viewer::default();
    big.open_repeats();
    big.set_repeat(repeat_ids::HABIT, "t", |r| {
        r.columns = 100;
        r.rows = 100;
    })
    .unwrap();
    let capped =
        big.page().unwrap().repeats_capped == 1 && open().page().unwrap().repeats_capped == 0;
    let ok = counted && left_out && capped;
    println!(
        "\nCHECK crate=ui name=repeat_snap_and_cap result={} copies_are_snap_targets={counted} own_copies_left_out={left_out} cap_reported={capped}",
        pass(ok)
    );
    assert!(ok);
}

struct Window {
    ui: ViewerUi,
}

fn harness_with(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 1000.0))
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
fn the_layers_tree_shows_the_source_under_the_repeat_and_the_repeat_tab_edits() {
    let mut v = open();
    // Only the habit tracker on the page, so its rows fit the small layers panel.
    {
        let o = v.open.as_mut().unwrap();
        o.doc.layers[0]
            .objects
            .retain(|o| o.id == repeat_ids::HABIT);
    }
    let mut h = harness_with(v);
    let tree = h.query_by_label("Habit tracker").is_some() && h.query_by_label("Day").is_some();
    let arrow = h
        .query_by_label("Show what is inside Habit tracker")
        .is_some();
    // Picking the repeat row selects it and offers the Repeat tab.
    h.get_by_label("Habit tracker").click();
    h.run();
    h.run();
    let picked = h.state().ui.viewer.selection == vec![repeat_ids::HABIT];
    h.state_mut().ui.tab = Tab::Repeat;
    h.run();
    h.run();
    let tab = h.query_by_label("Expand").is_some() && h.query_by_label("Add a rule").is_some();
    // The Repeat button is beside Group and Ungroup.
    let button = h.query_all_by_label("Repeat").count() >= 2;
    let ok = tree && arrow && picked && tab && button;
    println!(
        "\nCHECK crate=ui name=repeat_layers_and_tab result={} source_under_repeat={tree} fold_arrow_named={arrow} row_picks={picked} tab_has_expand_and_rules={tab} button={button}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_repeat_tab_edit_is_one_undo_step_and_the_box_refits() {
    let mut v = open();
    let rep = repeat_ids::ACROSS;
    let s0 = steps(&v);
    v.set_repeat(rep, "repeat-count", |r| r.count = 6).unwrap();
    let f = v.doc().unwrap().object(rep).unwrap().frame;
    // Six boxes at a pitch of 10 mm and 10 mm wide: 5 * 10 + 10.
    let boxed = (f.w_mm - 60.0).abs() < 1e-6;
    // Typing into a field joins one step.
    v.set_repeat(rep, "repeat-count", |r| r.count = 7).unwrap();
    let one = steps(&v) == s0 + 1;
    v.seal();
    v.set_repeat(rep, "repeat-step-x", |r| r.h_spacing = Spacing::Units(3.0))
        .unwrap();
    let two = steps(&v) == s0 + 2;
    let f = v.doc().unwrap().object(rep).unwrap().frame;
    let moved = (f.w_mm - (6.0 * 15.0 + 10.0)).abs() < 1e-6;
    let _ = Frame::new(0.0, 0.0, 0.0, 0.0);
    let ok = boxed && one && two && moved;
    println!(
        "\nCHECK crate=ui name=repeat_settings_edits result={} box_refits={boxed} merge_within_a_field={one} next_field_new_step={two} pitch_applies={moved}",
        pass(ok)
    );
    assert!(ok);
}
