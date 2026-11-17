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

//! The Profiles window driven with the pointer (rung 14a, paying the rung 13a debt, docs/bbq/2026-11-17-rung-14-
//! lint-and-preflight.md, Q9): the collapsing sections open on a click, a DragValue takes a typed number, the
//! lint fields clear and set, the gray-list field takes text, the safe-zone table adds and removes a zone, and a
//! damaged file is deleted from the list after a question. Each test uses folders of its own under the
//! system's temporary folder, never the config folder, and prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::NOMAD_ID;
use quadrule_profiles::{Registry, active, builtin, to_ron};
use quadrule_ui::{Viewer, ViewerUi};
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn computer(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-ui-ptr-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    active::set(Registry::load(&d));
    d
}

fn harness() -> Harness<'static, ViewerUi> {
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    ui.open_profiles_window();
    ui.profiles_select(NOMAD_ID);
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

fn lint_of(h: &Harness<'static, ViewerUi>) -> quadrule_profiles::LintThresholds {
    h.state().profiles_current().unwrap().lint
}

#[test]
fn the_lint_section_opens_and_its_fields_clear_set_and_take_typed_numbers() {
    computer("lint");
    let mut h = harness();
    let shipped = builtin(NOMAD_ID).unwrap().lint;
    // Folded at first: the field is not there. A click on the header opens it.
    let folded = h.query_by_label("Lightest gray, 1 px line").is_none();
    h.get_by_label("Lint checks").click();
    h.run();
    let opened = h.query_by_label("Lightest gray, 1 px line").is_some();
    // Clear: the value goes, the field reads "skipped" and a Set button takes its place.
    h.get_by_label("Clear Lightest gray, 1 px line")
        .scroll_to_me();
    h.run();
    h.get_by_label("Clear Lightest gray, 1 px line").click();
    h.run();
    let cleared = lint_of(&h).lightest_gray_line_1px.is_none()
        && h.query_by_label("Set Lightest gray, 1 px line").is_some();
    let dirty = h.state().profiles_dirty();
    // Set: it comes back with a starting value.
    h.get_by_label("Set Lightest gray, 1 px line").click();
    h.run();
    let set = lint_of(&h).lightest_gray_line_1px.is_some();
    // A DragValue: a click opens it for typing, a number and Enter set it.
    h.get_by_label("Lightest gray, fill").scroll_to_me();
    h.run();
    h.get_by_label("Lightest gray, fill").click();
    h.run();
    h.get_by_label("Lightest gray, fill").type_text("190");
    h.key_press(egui::Key::Enter);
    h.run();
    let typed = lint_of(&h).lightest_gray_fill == Some(190);
    // The page share is shown as a percentage.
    h.get_by_label("Dark fills: page share, %").scroll_to_me();
    h.run();
    h.get_by_label("Dark fills: page share, %").click();
    h.run();
    h.get_by_label("Dark fills: page share, %").type_text("25");
    h.key_press(egui::Key::Enter);
    h.run();
    let share = lint_of(&h).dark_fill_page_share == Some(0.25);
    let changed = lint_of(&h) != shipped;
    let ok = folded && opened && cleared && dirty && set && typed && share && changed;
    println!(
        "CHECK crate=ui name=profiles_pointer_lint result={} folded_at_first={folded} header_opens={opened} clear_button={cleared} draft_made={dirty} set_button={set} dragvalue_typed={typed} share_percent_typed={share}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_gray_list_and_the_safe_zone_table_take_the_pointer() {
    computer("lists");
    let mut h = harness();
    h.get_by_label("Gray levels and pen grays").click();
    h.run();
    let field = "Gray levels the screen renders cleanly";
    h.get_by_label(field).scroll_to_me();
    h.run();
    h.get_by_label(field).click();
    h.run();
    h.key_combination_modifiers(egui::Modifiers::COMMAND, &[egui::Key::A]);
    h.get_by_label(field).type_text("0, 100, 255");
    h.run();
    let typed = h.state().profiles_current().unwrap().gray_levels == [0, 100, 255];
    // Half typed, not a list: the draft keeps the last good one and the form says so.
    h.get_by_label(field).type_text(", 300");
    h.run();
    let kept = h.state().profiles_current().unwrap().gray_levels == [0, 100, 255]
        && h.query_by_label_contains("whole numbers from 0 to 255")
            .is_some();
    // The safe zones: the header opens the table, × removes a row, Add a zone adds one.
    let before = h.state().profiles_current().unwrap().safe_zones.len();
    h.get_by_label_contains("Safe zones").click();
    h.run();
    h.get_by_label("Remove the zone footer_left").scroll_to_me();
    h.run();
    h.get_by_label("Remove the zone footer_left").click();
    h.run();
    let removed = h.state().profiles_current().unwrap().safe_zones.len() == before - 1;
    h.get_by_label("Add a zone").scroll_to_me();
    h.run();
    h.get_by_label("Add a zone").click();
    h.run();
    let added = h.state().profiles_current().unwrap().safe_zones.len() == before;
    let ok = typed && kept && removed && added;
    println!(
        "CHECK crate=ui name=profiles_pointer_lists result={} gray_list_typed={typed} bad_text_keeps_last_good={kept} zone_removed={removed} zone_added={added}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_damaged_file_is_deleted_from_the_list_after_a_question() {
    let d = computer("damaged");
    let good = to_ron(&builtin(NOMAD_ID).unwrap());
    std::fs::write(d.join("typo.ron"), good.replace("ppi: 300.0", "ppi: three")).unwrap();
    active::set(Registry::load(&d));
    let mut h = harness();
    let listed = h.query_all_by_label_contains("typo.ron").count() >= 1;
    h.get_by_label_contains("Delete the file typo.ron").click();
    h.run();
    // Asked first: Keep leaves the file where it is.
    let asked = h.query_by_label("Delete damaged file").is_some();
    h.get_by_label("Keep").click();
    h.run();
    let kept = d.join("typo.ron").exists();
    h.get_by_label_contains("Delete the file typo.ron").click();
    h.run();
    h.get_by_label("Delete").click();
    h.run();
    h.run();
    let gone = !d.join("typo.ron").exists()
        && h.state().profiles_damaged().is_empty()
        && h.query_all_by_label_contains("Delete the file typo.ron")
            .count()
            == 0
        && h.query_by_label_contains("Deleted typo.ron").is_some();
    let ok = listed && asked && kept && gone;
    println!(
        "CHECK crate=ui name=profiles_pointer_delete_damaged result={} listed={listed} asked_first={asked} keep_leaves_it={kept} deleted_and_unlisted={gone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_edited_lint_block_is_saved_exported_imported_and_reset_with_the_profile() {
    let d = computer("lintfile");
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    ui.open_profiles_window();
    ui.profiles_select(NOMAD_ID);
    let shipped = builtin(NOMAD_ID).unwrap();
    ui.profiles_edit(|p| {
        p.lint.lightest_gray_line_1px = Some(150);
        p.lint.smallest_text_pt = None;
    });
    ui.profiles_save();
    let saved = std::fs::read_to_string(d.join(format!("{NOMAD_ID}.ron"))).unwrap();
    let in_file =
        saved.contains("lightest_gray_line_1px: Some(150)") && !saved.contains("smallest_text_pt");
    let live = active::lookup(NOMAD_ID)
        .unwrap()
        .lint
        .lightest_gray_line_1px
        == Some(150);
    // Export writes it, and an import of that text on a fresh computer brings it back.
    let out = d.join("export.ron");
    ui.profiles_export_to(&out);
    let text = std::fs::read_to_string(&out).unwrap();
    let exported = text.contains("lightest_gray_line_1px: Some(150)");
    let other = computer("lintfile-other");
    let mut ui2 = ViewerUi::new(Viewer::default());
    ui2.viewer.open_sample();
    ui2.open_profiles_window();
    let _ = ui2.profiles_import_text(&text);
    // The import was a clash with the shipped Nomad (same id, other values): Replace mine.
    ui2.profiles_import_answer(Some(quadrule_ui::ImportPick::Replace));
    let imported = active::lookup(NOMAD_ID)
        .unwrap()
        .lint
        .lightest_gray_line_1px
        == Some(150);
    // Reset to built-in on the first computer: the shipped lint block is back.
    active::set(Registry::load(&d));
    let mut ui3 = ViewerUi::new(Viewer::default());
    ui3.viewer.open_sample();
    ui3.open_profiles_window();
    ui3.profiles_select(NOMAD_ID);
    ui3.profiles_ask(quadrule_ui::Confirm::Reset(NOMAD_ID.into()));
    ui3.profiles_confirm(true);
    let reset = active::lookup(NOMAD_ID).unwrap().lint == shipped.lint;
    drop(other);
    let ok = in_file && live && exported && imported && reset;
    println!(
        "CHECK crate=ui name=profiles_lint_block_file result={} saved_in_override={in_file} used_at_once={live} exported={exported} imported={imported} reset_restores_shipped={reset}",
        pass(ok)
    );
    assert!(ok);
}
