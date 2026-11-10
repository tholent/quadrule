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

//! Window checks: the egui widgets driven headless by egui_kittest, by their labels (no GPU, no
//! screenshots; rung 2 bbq, Defaulted "Tests"). Each test prints one structured `CHECK` line.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::ids;
use quadrule_ui::{After, ExportWhich, Recovery, Request, Tab, Viewer, ViewerUi, Zoom};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
    requests: Vec<Request>,
}

fn harness() -> Harness<'static, Window> {
    let state = Window {
        ui: ViewerUi::new(Viewer::default()),
        requests: Vec::new(),
    };
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 860.0))
        // Opening a page adds many widgets at once; each new one asks for one more frame.
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let r = w.ui.show(ui);
                w.requests.extend(r);
            },
            state,
        );
    h.run();
    h
}

#[test]
fn empty_start_then_sample() {
    // Rung 2 bbq, Q7: start empty; "Open sample page" opens the slice; the lint count shows.
    let mut h = harness();
    let empty = h.query_by_label("Nothing open").is_some() && h.state().ui.viewer.open.is_none();
    h.get_by_label("Open sample page").click();
    h.run();
    let opened = h.state().ui.viewer.open.is_some();
    // In the toolbar and the status bar.
    let count = h.query_all_by_label("⚠ 5").count() == 2;
    let panel = h.query_by_label("Lint (5)").is_some();
    let ok = empty && opened && count && panel;
    println!(
        "CHECK crate=ui name=window_empty_then_sample result={} empty_start={empty} opened={opened} lint_count_shown={count} lint_panel={panel}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn dialogs_are_requested_not_opened() {
    // The window asks the app for the file and folder dialogs (Q7, Q8).
    let mut h = harness();
    h.get_all_by_label("Open…").next().unwrap().click();
    h.run();
    let open = h.state().requests.contains(&Request::OpenDialog);
    h.get_by_label("Open sample page").click();
    h.run();
    h.get_by_label("Export").click();
    h.run();
    h.get_by_label("Export all devices…").click();
    h.run();
    let all = h
        .state()
        .requests
        .contains(&Request::ExportDialog(ExportWhich::AllDevices));
    let ok = open && all;
    println!(
        "CHECK crate=ui name=window_requests result={} open_dialog={open} export_all={all} requests={:?}",
        pass(ok),
        h.state().requests
    );
    assert!(ok);
}

#[test]
fn findings_zoom_and_theme() {
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    // Clicking a finding selects it, for the outline on the page (Q6). Findings are on the
    // inspector's Lint tab (rung 3 bbq Q8).
    h.get_by_label("Lint (5)").click();
    h.run();
    h.get_by_label_contains("L16").click();
    h.run();
    let selected = h.state().ui.viewer.selected.is_some();
    // Zoom (Q10).
    h.get_by_label("1:1").click();
    h.run();
    let one = h.state().ui.viewer.zoom == Zoom::Scale(1.0);
    h.get_by_label("+").click();
    h.run();
    let two = h.state().ui.viewer.zoom == Zoom::Scale(2.0);
    // Real size with no ruler check yet opens the check; saving it switches to real size.
    h.get_by_label("Real size").click();
    h.run();
    let ruler = h.state().ui.ruler_open();
    // The dialog's Save, not the toolbar's: dialogs are drawn last.
    h.get_all_by_label("Save").last().unwrap().click();
    h.run();
    let real = h.state().ui.viewer.zoom == Zoom::RealSize
        && h.state().ui.viewer.screen_px_per_mm.is_some();
    // Theme (Q11).
    h.get_by_label("View").click();
    h.run();
    h.get_by_label("Dark").click();
    h.run();
    let dark = h.state().ui.theme == egui::ThemePreference::Dark;
    let ok = selected && one && two && ruler && real && dark;
    println!(
        "CHECK crate=ui name=window_findings_zoom_theme result={} finding_selected={selected} one_to_one={one} zoom_in={two} ruler_opened={ruler} real_size={real} dark={dark}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn every_message_id_exists() {
    // Every `"app-…"` and `"notice-…"` id in the window code (this crate and the app) is in the
    // English strings, so a typo shows here and never as `⟨id⟩` on screen.
    let sources = [
        include_str!("../src/view.rs"),
        include_str!("../src/viewer.rs"),
        include_str!("../src/panels.rs"),
        include_str!("../src/editor.rs"),
        include_str!("../src/recovery.rs"),
        include_str!("../../app/src/main.rs"),
    ];
    let strings = quadrule_i18n::Strings::english();
    let mut ids: Vec<&str> = sources
        .iter()
        .flat_map(|s| s.split('"').skip(1).step_by(2))
        .filter(|s| {
            (s.starts_with("app-") || s.starts_with("notice-"))
                && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
        .collect();
    ids.sort();
    ids.dedup();
    let missing: Vec<_> = ids.iter().filter(|id| !strings.has(id)).collect();
    let ok = missing.is_empty() && ids.len() > 30;
    println!(
        "CHECK crate=ui name=message_ids_exist result={} ids={} missing={missing:?}",
        pass(ok),
        ids.len()
    );
    assert!(ok);
}

fn x_of(h: &Harness<'static, Window>, id: quadrule_model::Uuid) -> f64 {
    h.state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(id)
        .unwrap()
        .frame
        .x_mm
}

#[test]
fn select_in_layers_and_edit_in_the_inspector() {
    // Rung 3 bbq Q8, Q9: a row in the layers panel selects; Arrange's X field reads any unit;
    // typing is one undo step; the title shows a dot while there are unsaved changes.
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    let page_tab = h.state().ui.tab == Tab::Page && h.query_by_label("Margins").is_some();
    h.get_by_label("Title rule").click();
    h.run();
    let selected = h.state().ui.viewer.selection == vec![ids::RULE];
    let arrange = h.state().ui.tab == Tab::Arrange;
    let x0 = x_of(&h, ids::RULE);
    h.get_by_label("X").focus();
    h.run();
    // Select all the text and type a new value with a unit.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::A]);
    h.run();
    h.get_by_label("X").type_text("20mm");
    h.run();
    let typed = (x_of(&h, ids::RULE) - 20.0).abs() < 1e-9;
    let dirty_title = h.state().ui.window_title().starts_with('●');
    // Leave the field, then one undo puts it all back.
    h.key_press(Key::Escape);
    h.run();
    h.get_by_label("Undo").click();
    h.run();
    let undone = x_of(&h, ids::RULE) == x0 && !h.state().ui.viewer.is_dirty();
    let clean_title = h.state().ui.window_title() == "Sample page — quadrule";
    let ok = page_tab && selected && arrange && typed && dirty_title && undone && clean_title;
    println!(
        "CHECK crate=ui name=window_select_and_inspect result={} page_tab_when_none={page_tab} row_selects={selected} arrange_tab={arrange} typed_20mm={typed} title_dot={dirty_title} one_undo={undone} title_clean={clean_title}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn keys_nudge_delete_duplicate() {
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    h.get_by_label("Title rule").click();
    h.run();
    let x0 = x_of(&h, ids::RULE);
    h.key_press(Key::ArrowRight);
    h.run();
    let one_px = ((x_of(&h, ids::RULE) - x0) - 25.4 / 300.0).abs() < 1e-9;
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::D]);
    h.run();
    let dup = h.state().ui.viewer.selection.len() == 1
        && h.state().ui.viewer.selection[0] != ids::RULE
        && h.state().ui.viewer.doc().unwrap().layers[1].objects.len() == 3;
    h.key_press(Key::Delete);
    h.run();
    let deleted = h.state().ui.viewer.doc().unwrap().layers[1].objects.len() == 2;
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::Z]);
    h.run();
    let undo_key = h.state().ui.viewer.doc().unwrap().layers[1].objects.len() == 3;
    let ok = one_px && dup && deleted && undo_key;
    println!(
        "CHECK crate=ui name=window_keys result={} arrow_nudges={one_px} ctrl_d={dup} delete={deleted} ctrl_z={undo_key}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn unsaved_changes_prompt() {
    // Rung 3 bbq, Defaulted "Save": opening another file with unsaved changes asks first.
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    h.state_mut().ui.viewer.selection = vec![ids::RULE];
    h.state_mut().ui.viewer.nudge(1.0, 0.0, false).unwrap();
    h.run();
    h.get_all_by_label("Open…").next().unwrap().click();
    h.run();
    let asked = h.state().ui.prompt_open() && h.query_by_label("Save changes?").is_some();
    let nothing_yet = h.state().requests.is_empty();
    h.get_by_label("Cancel").click();
    h.run();
    let cancelled = !h.state().ui.prompt_open() && h.state().requests.is_empty();
    // Save on the sample page asks where (it has no file), then carries on to Open.
    h.get_all_by_label("Open…").next().unwrap().click();
    h.run();
    // The dialog's Save, not the toolbar's: dialogs are drawn last.
    h.get_all_by_label("Save").last().unwrap().click();
    h.run();
    let save_as = h
        .state()
        .requests
        .contains(&Request::SaveAsDialog(Some(After::OpenDialog)));
    // Don't save goes straight on.
    h.state_mut().requests.clear();
    h.get_all_by_label("Open…").next().unwrap().click();
    h.run();
    h.get_by_label("Don’t save").click();
    h.run();
    let discarded = h.state().requests == vec![Request::OpenDialog];
    // Closing asks too.
    let close_waits = !h.state_mut().ui.may_close() && h.state().ui.prompt_open();
    let ok = asked && nothing_yet && cancelled && save_as && discarded && close_waits;
    println!(
        "CHECK crate=ui name=window_unsaved_prompt result={} asked={asked} cancel={cancelled} save_asks_where={save_as} dont_save_proceeds={discarded} close_asks={close_waits}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn recovery_offered_on_start() {
    // Napkin §5.6: after a crash, the recovery file is offered back.
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("window_recovery");
    let _ = std::fs::remove_dir_all(&dir);
    let rec = Recovery::new(&dir);
    let mut v = Viewer::default();
    v.open_sample();
    v.selection = vec![ids::TITLE];
    v.nudge(0.0, 3.0, false).unwrap();
    rec.write(&mut v).unwrap();
    let edited = v.doc().unwrap().clone();

    let mut h = harness();
    h.state_mut().ui.recovery = Some(rec.clone());
    h.state_mut().ui.offer_recovery();
    h.run();
    let offered = h.query_by_label("Recover unsaved work?").is_some();
    h.get_by_label("Restore").click();
    h.run();
    let restored = h.state().ui.viewer.doc() == Some(&edited)
        && h.state().ui.viewer.is_dirty()
        && !h.state().ui.recovery_open();
    let ok = offered && restored;
    println!(
        "CHECK crate=ui name=window_recovery_offer result={} offered={offered} restored_dirty={restored}",
        pass(ok)
    );
    assert!(ok);
}
