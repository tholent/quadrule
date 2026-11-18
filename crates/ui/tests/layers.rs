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

//! New layer and Delete layer in the layers panel (docs/bbq/2026-11-18-new-and-delete-layers.md, Rules Q1 to Q4 and the
//! Defaulted list): the two buttons and the row menu, the name "Layer N", where the layer goes, the name field open, the
//! ask before a layer with objects goes, the greyed cases. Headless. Each test prints one structured `CHECK` line
//! starting with a newline (run with `-- --nocapture`).
//!
//! `egui_kittest` cannot make a double-click, so the name field is read from the window's state
//! (`ViewerUi::renaming_layer`) and typed into through the real text field, which New layer opens with focus.

use egui::{Key, Role};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::edit::{Command, LayerProps};
use quadrule_model::sample::*;
use quadrule_model::{Document, Orientation};
use quadrule_ui::{Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn harness(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
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

/// The sample page: layers Grid (the dot grid and the ruled lines) and Header (the title and its rule).
fn sample() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    harness(v)
}

/// A new blank page: one layer, "Layer 1".
fn blank() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Test page".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: Orientation::Portrait,
    })
    .unwrap();
    harness(v)
}

fn doc<'a>(h: &'a Harness<'static, Window>) -> &'a Document {
    h.state().ui.viewer.doc().unwrap()
}

fn names(h: &Harness<'static, Window>) -> Vec<String> {
    doc(h).layers.iter().map(|l| l.name.clone()).collect()
}

fn steps(h: &Harness<'static, Window>) -> usize {
    h.state()
        .ui
        .viewer
        .open
        .as_ref()
        .unwrap()
        .history
        .undo_len()
}

fn press(h: &mut Harness<'static, Window>, label: &str) {
    h.get_by_label(label).hover();
    h.run();
    h.get_by_label(label).click();
    h.run();
    h.run();
}

fn disabled(h: &Harness<'static, Window>, label: &str) -> bool {
    h.get_by_role_and_label(Role::Button, label)
        .accesskit_node()
        .is_disabled()
}

fn undo(h: &mut Harness<'static, Window>) {
    h.state_mut().ui.viewer.undo();
    h.run();
}

#[test]
fn new_layer_goes_on_top_when_nothing_is_picked_named_with_the_lowest_free_number_and_opens_its_name()
 {
    let mut h = blank();
    let both =
        h.query_by_label("New layer").is_some() && h.query_by_label("Delete layer").is_some();
    let before = steps(&h);
    press(&mut h, "New layer");
    let on_top = names(&h) == ["Layer 1", "Layer 2"];
    let one_step = steps(&h) == before + 1;
    let picked = h.state().ui.picked_layer() == Some(1);
    let open = h.state().ui.renaming_layer() == Some(1);
    let field = h.query_by_role_and_label(Role::TextInput, "Name").is_some();
    // Another, with the field still open: above the picked layer (the top), so "Layer 3" on top again.
    press(&mut h, "New layer");
    let third = names(&h) == ["Layer 1", "Layer 2", "Layer 3"];
    // A name taken is skipped; a gap is filled with the lowest free number.
    h.state_mut()
        .ui
        .viewer
        .edit(
            Command::SetLayer {
                index: 0,
                props: LayerProps {
                    name: "Grid".into(),
                    visible: true,
                    locked: false,
                },
            },
            None,
        )
        .unwrap();
    h.state_mut().ui.new_layer(None);
    let gap = names(&h).last().map(String::as_str) == Some("Layer 1");
    undo(&mut h);
    undo(&mut h);
    undo(&mut h);
    undo(&mut h);
    let back = names(&h) == ["Layer 1"];
    let ok = both && on_top && one_step && picked && open && field && third && gap && back;
    println!(
        "\nCHECK crate=ui name=new_layer_name_place_and_open_field result={} both_buttons={both} on_top_named_layer_2={on_top} one_undo_step={one_step} picked={picked} name_field_open={open} field_in_window={field} third_is_layer_3={third} lowest_free_number={gap} undo_back_to_one={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn new_layer_goes_just_above_the_picked_layer_or_the_picked_objects_layer() {
    let mut h = sample();
    // Pick Grid (the bottom layer): the new one is between Grid and Header.
    press(&mut h, "Grid");
    press(&mut h, "New layer");
    let above_row = names(&h) == ["Grid", "Layer 1", "Header"];
    // Pick an object of Header (layer 2 now): the new one goes just above Header, on top.
    h.state_mut().ui.viewer.select_row(ids::TITLE, false);
    h.run();
    let object_layer = h.state().ui.picked_layer() == Some(2);
    let picked = h.state().ui.picked_layer();
    h.state_mut().ui.new_layer(picked);
    h.run();
    let above_object = names(&h) == ["Grid", "Layer 1", "Header", "Layer 2"];
    // The right-click menu says the same of the row it is on.
    h.state_mut().ui.viewer.selection.clear();
    h.run();
    h.get_by_label("Grid").click_secondary();
    h.run();
    h.run();
    let menu_new = h.get_all_by_label("New layer").count() == 2;
    h.get_all_by_label("New layer").last().unwrap().click();
    h.run();
    h.run();
    let from_menu = names(&h) == ["Grid", "Layer 3", "Layer 1", "Header", "Layer 2"]
        && h.state().ui.picked_layer() == Some(1)
        && h.state().ui.renaming_layer() == Some(1);
    let ok = above_row && object_layer && above_object && menu_new && from_menu;
    println!(
        "\nCHECK crate=ui name=new_layer_placement result={} above_picked_row={above_row} picked_object_layer={object_layer} above_picked_object={above_object} menu_has_new_layer={menu_new} menu_new_layer_above_its_row={from_menu}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_name_field_keeps_a_typed_name_on_enter_and_layer_n_on_escape() {
    let mut h = blank();
    press(&mut h, "New layer");
    let before = steps(&h);
    h.get_by_role_and_label(Role::TextInput, "Name")
        .type_text("Labels");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    h.run();
    let typed = names(&h) == ["Layer 1", "Labels"] && h.state().ui.renaming_layer().is_none();
    // The name is its own undo step after the layer's.
    let two_steps = steps(&h) == before + 1;
    // Escape keeps "Layer N".
    press(&mut h, "New layer");
    h.get_by_role_and_label(Role::TextInput, "Name")
        .type_text("Scratch");
    h.run();
    h.key_press(Key::Escape);
    h.run();
    h.run();
    let kept =
        names(&h) == ["Layer 1", "Labels", "Layer 2"] && h.state().ui.renaming_layer().is_none();
    // Clicking away keeps what was typed.
    press(&mut h, "New layer");
    h.get_by_role_and_label(Role::TextInput, "Name")
        .type_text("Notes");
    h.run();
    h.get_by_label("Layer 1").click();
    h.run();
    h.run();
    let away = names(&h).contains(&"Notes".to_string());
    let ok = typed && two_steps && kept && away;
    println!(
        "\nCHECK crate=ui name=new_layer_name_field result={} enter_keeps_typed_name={typed} name_is_one_more_undo_step={two_steps} escape_keeps_layer_n={kept} clicking_away_keeps_it={away}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_empty_layer_goes_at_once_and_a_layer_with_objects_asks_first() {
    // Empty: at once, one undo step brings it back.
    let mut h = sample();
    press(&mut h, "New layer");
    h.key_press(Key::Escape);
    h.run();
    let empty_picked =
        names(&h) == ["Grid", "Header", "Layer 1"] && h.state().ui.picked_layer() == Some(2);
    let before = steps(&h);
    let with_empty = doc(&h).clone();
    press(&mut h, "Delete layer");
    let at_once = names(&h) == ["Grid", "Header"] && h.state().ui.layer_delete_ask().is_none();
    let one_step = steps(&h) == before + 1;
    undo(&mut h);
    let empty_back = *doc(&h) == with_empty;

    // With objects: a sheet asks, in words; Cancel keeps everything.
    let mut h = sample();
    let page = doc(&h).clone();
    let rows = page.layer_object_count(1);
    press(&mut h, "Header");
    press(&mut h, "Delete layer");
    let asks = h.state().ui.layer_delete_ask() == Some((1, "Header"));
    h.run();
    let words = h
        .query_by_label(&format!(
            "Delete the layer \"Header\" and its {rows} objects?"
        ))
        .is_some();
    let nothing_yet = *doc(&h) == page;
    press(&mut h, "Cancel");
    let cancelled = *doc(&h) == page && h.state().ui.layer_delete_ask().is_none();
    // Escape cancels too.
    press(&mut h, "Delete layer");
    h.key_press(Key::Escape);
    h.run();
    h.run();
    let escaped = *doc(&h) == page && h.state().ui.layer_delete_ask().is_none();
    // Delete: the layer and its objects go, as one step; Undo brings them back whole.
    press(&mut h, "Delete layer");
    let before = steps(&h);
    press(&mut h, "Delete");
    let gone = names(&h) == ["Grid"] && doc(&h).object(ids::TITLE).is_none();
    let one_step_ask = steps(&h) == before + 1;
    let cleared = h.state().ui.picked_layer().is_none();
    undo(&mut h);
    let whole = *doc(&h) == page;
    let ok = empty_picked
        && at_once
        && one_step
        && empty_back
        && asks
        && words
        && nothing_yet
        && cancelled
        && escaped
        && gone
        && one_step_ask
        && cleared
        && whole;
    println!(
        "\nCHECK crate=ui name=delete_layer_asks_first result={} empty_layer_picked={empty_picked} empty_goes_at_once={at_once} one_undo_step={one_step} empty_undo_returns={empty_back} sheet_asks={asks} words_with_name_and_count={words} nothing_changed_before_answer={nothing_yet} cancel_keeps={cancelled} escape_keeps={escaped} delete_removes={gone} delete_is_one_step={one_step_ask} nothing_picked_after={cleared} undo_returns_whole={whole}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_sheet_counts_one_object_in_the_singular() {
    let mut h = sample();
    // Take the Header layer down to one object, then delete it.
    h.state_mut()
        .ui
        .viewer
        .edit(Command::Remove(ids::RULE), None)
        .unwrap();
    h.run();
    press(&mut h, "Header");
    press(&mut h, "Delete layer");
    h.run();
    let one = h
        .query_by_label("Delete the layer \"Header\" and its 1 object?")
        .is_some();
    println!(
        "\nCHECK crate=ui name=delete_layer_sheet_singular result={} one_object_worded_as_one={one}",
        pass(one)
    );
    assert!(one);
}

#[test]
fn delete_layer_is_greyed_with_a_note_for_no_pick_the_last_layer_and_a_locked_layer() {
    // Nothing picked.
    let mut h = sample();
    let none_greyed = disabled(&h, "Delete layer");
    h.get_by_role_and_label(Role::Button, "Delete layer")
        .hover();
    h.run();
    h.run();
    let none_note = h.query_by_label("Pick a layer first.").is_some();
    // A layer picked: enabled.
    press(&mut h, "Header");
    let enabled = !disabled(&h, "Delete layer");
    // The last layer.
    let mut h1 = blank();
    press(&mut h1, "Layer 1");
    let last_greyed = disabled(&h1, "Delete layer");
    h1.get_by_role_and_label(Role::Button, "Delete layer")
        .hover();
    h1.run();
    h1.run();
    let last_note = h1
        .query_by_label("A page needs at least one layer.")
        .is_some();
    // A locked layer: Unlock it to delete it.
    let mut hl = sample();
    press(&mut hl, "Header");
    hl.state_mut()
        .ui
        .viewer
        .edit(
            Command::SetLayer {
                index: 1,
                props: LayerProps {
                    name: "Header".into(),
                    visible: true,
                    locked: true,
                },
            },
            None,
        )
        .unwrap();
    hl.run();
    let locked_greyed = disabled(&hl, "Delete layer");
    hl.get_by_role_and_label(Role::Button, "Delete layer")
        .hover();
    hl.run();
    hl.run();
    let locked_note = hl.query_by_label("Unlock it to delete it.").is_some();
    // Its menu item is greyed the same way.
    hl.get_by_label("Header").click_secondary();
    hl.run();
    hl.run();
    let menu_greyed = hl
        .get_all_by_role_and_label(Role::Button, "Delete layer")
        .last()
        .unwrap()
        .accesskit_node()
        .is_disabled();
    let ok = none_greyed
        && none_note
        && enabled
        && last_greyed
        && last_note
        && locked_greyed
        && locked_note
        && menu_greyed;
    println!(
        "\nCHECK crate=ui name=delete_layer_greyed result={} nothing_picked_greyed={none_greyed} nothing_picked_note={none_note} picked_layer_enabled={enabled} last_layer_greyed={last_greyed} last_layer_note={last_note} locked_greyed={locked_greyed} locked_note={locked_note} locked_menu_item_greyed={menu_greyed}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_layer_row_menu_deletes_its_own_row_and_a_picked_object_deletes_its_layer() {
    let mut h = sample();
    // Pick Grid's row, right-click Header's: the menu works on Header, and the right-click picks it.
    press(&mut h, "Grid");
    h.get_by_label("Header").click_secondary();
    h.run();
    h.run();
    let picked_header = h.state().ui.picked_layer() == Some(1);
    let menu_delete = h
        .get_all_by_role_and_label(Role::Button, "Delete layer")
        .count()
        == 2;
    h.get_all_by_role_and_label(Role::Button, "Delete layer")
        .last()
        .unwrap()
        .click();
    h.run();
    h.run();
    let asked = h.state().ui.layer_delete_ask() == Some((1, "Header"));
    press(&mut h, "Cancel");
    // A picked object: Delete layer deletes its layer.
    h.state_mut().ui.viewer.select_row(ids::DOTS, false);
    h.run();
    press(&mut h, "Delete layer");
    let object_layer = h.state().ui.layer_delete_ask() == Some((0, "Grid"));
    let ok = picked_header && menu_delete && asked && object_layer;
    println!(
        "\nCHECK crate=ui name=layer_row_menu_and_picked_object result={} right_click_picks_row={picked_header} menu_has_delete_layer={menu_delete} menu_delete_asks_about_its_row={asked} picked_object_means_its_layer={object_layer}",
        pass(ok)
    );
    assert!(ok);
}
