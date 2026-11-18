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

//! The welcome screen, the File menu and My templates driven with the pointer and the keyboard (rung 16 bbq, Q2, Q4,
//! Q5, Q6 and the Defaulted lists), headless with egui_kittest. Every file, settings file and templates folder is a
//! scratch one. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_ui::{Request, Source, Viewer, ViewerUi};
use std::path::{Path, PathBuf};

/// A file the system dropped on the window.
#[derive(Debug)]
struct Dropped(PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &Path {
        &self.0
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
    requests: Vec<Request>,
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A window on nothing open, its settings in `dir/settings.ron` and its templates in `dir/templates`.
fn window(dir: &Path) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
    v.templates_dir = Some(dir.join("templates"));
    harness(v)
}

fn harness(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let r = w.ui.show(ui);
                w.requests.extend(r);
            },
            Window {
                ui: ViewerUi::new(v),
                requests: Vec::new(),
            },
        );
    h.run();
    h
}

fn has(h: &Harness<'static, Window>, label: &str) -> bool {
    h.query_all_by_label(label).count() > 0
}

fn has_part(h: &Harness<'static, Window>, part: &str) -> bool {
    h.query_all_by_label_contains(part).count() > 0
}

fn settle(h: &mut Harness<'static, Window>) {
    for _ in 0..6 {
        h.step();
    }
}

/// A click that lands on a button in a sheet: the pointer first arrives on it, as a hand's does.
fn press(h: &mut Harness<'static, Window>, label: &str) {
    settle(h);
    h.get_by_label(label).hover();
    h.run();
    h.get_by_label(label).click();
    h.run();
}

fn file_item(h: &mut Harness<'static, Window>, item: &str) {
    h.get_by_label("File").click();
    h.run();
    h.get_all_by_label_contains(item).next().unwrap().click();
    h.run();
}

/// A small page saved at `path`.
fn save_page(path: &Path, title: &str) {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: title.into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
    v.save_to(path).unwrap();
}

#[test]
fn the_welcome_screen_has_its_frame_and_every_tab_says_what_is_in_it() {
    let dir = tmp("welcome-frame");
    let mut h = window(&dir);
    let left = ["+ Blank page", "Open…", "Recent", "Open sample page"]
        .iter()
        .all(|l| has(&h, l));
    let right = ["Device", "Search"].iter().all(|l| has(&h, l))
        && [
            "All",
            "Paper",
            "Notes",
            "Planning",
            "Bullet journal",
            "Creative",
            "My templates",
        ]
        .iter()
        .all(|l| has(&h, l));
    let recent_none = has_part(&h, "Files you open or save are listed here");
    // Each tab with no starters in it says they are coming (Q5); My templates says how to fill it.
    let soon_all = has_part(&h, "Starter pages are coming soon");
    let mut soon = true;
    for tab in ["Paper", "Notes", "Planning", "Bullet journal", "Creative"] {
        h.get_by_label(tab).click();
        h.run();
        soon &= has_part(&h, "Starter pages for this group are coming soon");
    }
    h.get_by_label("My templates").click();
    h.run();
    let mine = has_part(&h, "No templates yet") && !has_part(&h, "coming soon");
    // The top bar has the File menu and the tools, not New, Open, Save or Export buttons of their own.
    let bar = has(&h, "File")
        && !has(&h, "Save")
        && !has(&h, "Save As…")
        && !has(&h, "New…")
        && !has(&h, "Export")
        && h.query_all_by_label("Open…").count() == 1; // the welcome screen's own Open… is the only one
    // Open… asks the app for the file dialog; Open sample page opens the sample.
    h.get_by_label("Open…").click();
    h.run();
    let asked = h.state().requests.contains(&Request::OpenDialog);
    h.get_by_label("Open sample page").click();
    h.run();
    let sample = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .is_some_and(|o| o.source == Source::Sample)
        && !h.state().ui.welcome_showing();
    let ok = left && right && recent_none && soon_all && soon && mine && bar && asked && sample;
    println!(
        "CHECK crate=ui name=welcome_frame result={} left_column={left} right_side={right} recent_empty_line={recent_none} all_tab_soon={soon_all} empty_tabs_soon={soon} my_templates_line={mine} bar_has_file_only={bar} open_asks_dialog={asked} sample_link={sample}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn blank_page_opens_the_new_dialog_on_the_filters_device() {
    let dir = tmp("welcome-blank");
    let mut h = window(&dir);
    // The filter starts on the device New last used.
    let starts = h.state().ui.welcome_device() == MANTA_ID;
    h.state_mut().ui.set_welcome_device(NOMAD_ID);
    h.run();
    h.get_by_label("+ Blank page").click();
    h.run();
    let dialog = h.state().ui.new_dialog_open() && h.state().ui.new_device() == NOMAD_ID;
    h.get_by_label("Create").click();
    h.run();
    let o = h.state().ui.viewer.open.as_ref();
    let made = o
        .is_some_and(|o| o.source == Source::Untitled && o.doc.primary_profile_id == NOMAD_ID)
        && h.state().ui.viewer.profile_id == NOMAD_ID
        && !h.state().ui.welcome_showing();
    let ok = starts && dialog && made;
    println!(
        "CHECK crate=ui name=welcome_blank_page result={} filter_starts_on_last_device={starts} dialog_on_filter_device={dialog} page_made_on_it={made}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_recent_list_opens_greys_missing_files_and_offers_their_removal() {
    let dir = tmp("welcome-recent");
    for n in ["weekly", "dots", "meeting"] {
        save_page(&dir.join(format!("{n}.quad")), n);
    }
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
    v.templates_dir = Some(dir.join("templates"));
    for n in ["weekly", "dots", "meeting"] {
        v.open_path(&dir.join(format!("{n}.quad"))).unwrap();
    }
    v.close_document();
    let mut h = harness(v);
    // Newest first, each with its folder under it.
    let lines = has(&h, "meeting.quad") && has(&h, "dots.quad") && has(&h, "weekly.quad");
    let folders = h.query_all_by_label(&dir.display().to_string()).count() == 3;
    // A click opens it, asking nothing when nothing is unsaved; it goes to the top of the list.
    h.get_by_label("weekly.quad").click();
    h.run();
    let opened = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .is_some_and(|o| o.source == Source::File(dir.join("weekly.quad")))
        && !h.state().ui.welcome_showing();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::W]);
    h.run();
    let back = h.state().ui.welcome_showing() && h.state().ui.viewer.open.is_none();
    // A file gone from the disk shows greyed with Not found; a click offers to take it off the list.
    std::fs::remove_file(dir.join("dots.quad")).unwrap();
    h.state_mut().ui.show_welcome();
    h.run();
    let not_found = h.query_all_by_label_contains("Not found").count() == 1;
    h.get_by_label("dots.quad").click();
    h.run();
    let offer = h.state().ui.welcome_question_open() && has_part(&h, "no longer where it was");
    // Keep it: Cancel. Then ask again and remove it.
    press(&mut h, "Cancel");
    let kept =
        !h.state().ui.welcome_question_open() && h.state().ui.viewer.recent_files().len() == 3;
    h.get_by_label("dots.quad").click();
    h.run();
    press(&mut h, "Remove from list");
    let removed = h.state().ui.viewer.recent_files().len() == 2 && !has(&h, "dots.quad");
    // A right-click on a line has Remove from list too, with no question.
    h.get_by_label("meeting.quad").click_secondary();
    h.run();
    h.get_by_label("Remove from list").click();
    h.run();
    let menu =
        h.state().ui.viewer.recent_files().len() == 1 && !h.state().ui.welcome_question_open();
    let ok = lines && folders && opened && back && not_found && offer && kept && removed && menu;
    println!(
        "CHECK crate=ui name=welcome_recent result={} lines={lines} folders_shown={folders} click_opens={opened} close_returns={back} missing_greyed={not_found} click_offers_removal={offer} cancel_keeps={kept} remove_takes_off={removed} right_click_removes={menu}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn new_over_a_document_returns_with_esc_or_cancel_and_close_brings_the_welcome_back() {
    let dir = tmp("welcome-over");
    let mut h = window(&dir);
    h.get_by_label("Open sample page").click();
    h.run();
    // File > New… puts the welcome screen over the document; the document's panels step back.
    file_item(&mut h, "New…");
    let over = h.state().ui.welcome_over_document()
        && has(&h, "+ Blank page")
        && has(&h, "Cancel")
        && !has_part(&h, "Lint (");
    // Esc goes back to the document.
    h.key_press(Key::Escape);
    h.run();
    let esc = !h.state().ui.welcome_showing() && has_part(&h, "Lint (");
    // So does Cancel; the same with Ctrl+N.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::N]);
    h.run();
    let again = h.state().ui.welcome_over_document();
    h.get_by_label("Cancel").click();
    h.run();
    let cancel = !h.state().ui.welcome_showing();
    // Opening the sample over a document with unsaved changes asks first; Cancel leaves everything as it was.
    h.state_mut().ui.viewer.selection = vec![quadrule_model::sample::ids::RULE];
    h.state_mut().ui.viewer.nudge(1.0, 0.0, false).unwrap();
    h.run();
    file_item(&mut h, "New…");
    h.get_by_label("Open sample page").click();
    h.run();
    let asked = h.state().ui.prompt_open() && has(&h, "Save changes?");
    h.get_all_by_label("Cancel").last().unwrap().click();
    h.run();
    let still_over = !h.state().ui.prompt_open()
        && h.state().ui.welcome_over_document()
        && h.state().ui.viewer.is_dirty();
    h.key_press(Key::Escape);
    h.run();
    // Close asks about the unsaved changes, then shows the welcome screen with nothing open.
    file_item(&mut h, "Close");
    let close_asks = h.state().ui.prompt_open() && h.state().ui.viewer.open.is_some();
    h.get_by_label("Don’t save").click();
    h.run();
    let closed = h.state().ui.viewer.open.is_none()
        && h.state().ui.welcome_showing()
        && has(&h, "Open sample page");
    // A clean page closes at once, with Ctrl+W.
    h.get_by_label("Open sample page").click();
    h.run();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::W]);
    h.run();
    let ctrl_w = h.state().ui.viewer.open.is_none() && !h.state().ui.prompt_open();
    let ok =
        over && esc && again && cancel && asked && still_over && close_asks && closed && ctrl_w;
    println!(
        "CHECK crate=ui name=welcome_over_document result={} new_shows_over_page={over} esc_returns={esc} ctrl_n={again} cancel_returns={cancel} unsaved_asked_first={asked} cancel_keeps_all={still_over} close_asks={close_asks} close_shows_welcome={closed} ctrl_w_clean={ctrl_w}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_quad_dropped_on_the_welcome_screen_opens() {
    let dir = tmp("welcome-drop");
    let file = dir.join("dropped.quad");
    save_page(&file, "Dropped");
    let mut h = window(&dir);
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(file.clone())));
    h.run();
    let opened = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .is_some_and(|o| o.source == Source::File(file.clone()))
        && !h.state().ui.welcome_showing();
    let recent = h
        .state()
        .ui
        .viewer
        .recent_files()
        .first()
        .map(|r| r.path.clone())
        == Some(file);
    let ok = opened && recent;
    println!(
        "CHECK crate=ui name=welcome_drop result={} opened={opened} in_recent={recent}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_file_menu_holds_every_item_and_each_key_does_what_its_item_does() {
    let dir = tmp("welcome-file-menu");
    let mut h = window(&dir);
    h.get_by_label("Open sample page").click();
    h.run();
    h.get_by_label("File").click();
    h.run();
    h.run();
    let items = [
        "New…",
        "Open…",
        "Open Recent",
        "Save",
        "Save As…",
        "Save as template…",
        "Export…",
        "Close",
    ];
    let all = items.iter().all(|i| has_part(&h, i));
    let keys = [
        "Ctrl+N",
        "Ctrl+O",
        "Ctrl+S",
        "Ctrl+Shift+S",
        "Ctrl+E",
        "Ctrl+W",
    ]
    .iter()
    .all(|k| has_part(&h, k));
    let no_again = !has_part(&h, "Export again");
    // Open Recent lists the recent files; with none it says so.
    h.get_all_by_label_contains("Open Recent")
        .next()
        .unwrap()
        .hover();
    h.run();
    h.run();
    let none = has(&h, "No recent files");
    h.key_press(Key::Escape);
    h.run();
    h.get_by_label("File").click();
    h.run();
    // The keys: Ctrl+O the dialog, Ctrl+S on the sample page asks where, Ctrl+Shift+S the same.
    h.state_mut().requests.clear();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::O]);
    h.run();
    let o = h.state().requests == vec![Request::OpenDialog];
    h.state_mut().requests.clear();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::S]);
    h.run();
    let s = h.state().requests == vec![Request::SaveAsDialog(None)];
    h.state_mut().requests.clear();
    h.key_combination_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, &[Key::S]);
    h.run();
    let ss = h.state().requests == vec![Request::SaveAsDialog(None)];
    // Ctrl+E the Export sheet, with Export in focus; the old Ctrl+Shift+E opens nothing.
    h.key_combination_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, &[Key::E]);
    h.run();
    let shift_e = !h.state().ui.export_sheet_open();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::E]);
    h.run();
    settle(&mut h);
    let e = h.state().ui.export_sheet_open()
        && h.get_by_role_and_label(Role::Button, "Export").is_focused();
    press(&mut h, "Cancel");
    let ok = all && keys && no_again && none && o && s && ss && shift_e && e;
    println!(
        "CHECK crate=ui name=file_menu result={} items={all} keys_shown={keys} no_export_again={no_again} recent_empty_says_so={none} ctrl_o={o} ctrl_s={s} ctrl_shift_s={ss} ctrl_shift_e_gone={shift_e} ctrl_e={e}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn open_recent_in_the_file_menu_lists_the_files_and_greys_the_missing() {
    let dir = tmp("welcome-open-recent");
    save_page(&dir.join("a.quad"), "a");
    save_page(&dir.join("b.quad"), "b");
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
    v.open_path(&dir.join("a.quad")).unwrap();
    v.open_path(&dir.join("b.quad")).unwrap();
    std::fs::remove_file(dir.join("a.quad")).unwrap();
    let mut h = harness(v);
    h.get_by_label("File").click();
    h.run();
    h.get_all_by_label_contains("Open Recent")
        .next()
        .unwrap()
        .hover();
    h.run();
    h.run();
    let listed = has(&h, "b.quad") && has_part(&h, "a.quad (Not found)");
    h.get_by_label("b.quad").click();
    h.run();
    let opened = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .is_some_and(|o| o.source == Source::File(dir.join("b.quad")));
    let ok = listed && opened;
    println!(
        "CHECK crate=ui name=file_menu_open_recent result={} listed_and_missing_marked={listed} click_opens={opened}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn save_as_template_fills_my_templates_and_a_pick_opens_an_untitled_copy() {
    let dir = tmp("welcome-templates");
    let t = dir.join("templates");
    let mut h = window(&dir);
    h.get_by_label("Open sample page").click();
    h.run();
    file_item(&mut h, "Save as template…");
    let slug = t.join("rung-1-slice.quad");
    let wrote = slug.is_file() && h.state().ui.viewer.file_path().is_none();
    // Again: the same name asks first; Cancel leaves it, Replace takes its place.
    let first = std::fs::read(&slug).unwrap();
    h.state_mut().ui.viewer.selection = vec![quadrule_model::sample::ids::RULE];
    h.state_mut().ui.viewer.nudge(1.0, 0.0, false).unwrap();
    h.run();
    file_item(&mut h, "Save as template…");
    let asks = h.state().ui.replace_template_open() && has(&h, "Replace this template?");
    press(&mut h, "Cancel");
    let cancel_keeps =
        std::fs::read(&slug).unwrap() == first && !h.state().ui.replace_template_open();
    file_item(&mut h, "Save as template…");
    press(&mut h, "Replace");
    let replaced = std::fs::read(&slug).unwrap() != first && !h.state().ui.replace_template_open();
    // Close (don't save), and the My templates tab has a tile for it.
    file_item(&mut h, "Close");
    h.get_by_label("Don’t save").click();
    h.run();
    h.get_by_label("My templates").click();
    h.run();
    let tile = has(&h, "Rung 1 slice") && !has_part(&h, "No templates yet");
    // The device filter decides the copy's main device; the tile opens an untitled copy.
    h.state_mut().ui.set_welcome_device(NOMAD_ID);
    h.run();
    h.get_by_label("Rung 1 slice").click();
    h.run();
    let o = h.state().ui.viewer.open.as_ref();
    let copy = o
        .is_some_and(|o| o.source == Source::Untitled && o.doc.primary_profile_id == NOMAD_ID)
        && !h.state().ui.viewer.is_dirty()
        && !h.state().ui.welcome_showing()
        && h.state().ui.viewer.settings.recent.is_empty();
    let ok = wrote && asks && cancel_keeps && replaced && tile && copy;
    println!(
        "CHECK crate=ui name=my_templates_save_and_pick result={} slug_written_no_file_change={wrote} replace_asks={asks} cancel_keeps={cancel_keeps} replace_takes_place={replaced} tile_listed={tile} pick_opens_untitled_copy_on_filter_device={copy}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_template_is_removed_after_a_confirm_that_says_so_and_search_looks_through_every_tab() {
    let dir = tmp("welcome-remove");
    let t = dir.join("templates");
    let mut v = Viewer::default();
    v.templates_dir = Some(t.clone());
    for title in ["Dot grid", "Weekly planner"] {
        v.open_new(&NewPage {
            title: title.into(),
            device_id: MANTA_ID.into(),
            rhythm: RhythmPreset::Handwriting,
            orientation: quadrule_model::Orientation::Portrait,
        })
        .unwrap();
        v.save_as_template(false).unwrap();
    }
    v.close_document();
    let mut h = harness(v);
    // All lists the templates too; search narrows by name, ignoring case, and looks through every tab.
    let both = has(&h, "Dot grid") && has(&h, "Weekly planner");
    h.get_by_label("Paper").click();
    h.run();
    let paper_empty = !has(&h, "Dot grid");
    h.get_by_label("Search").click();
    h.get_by_label("Search").type_text("PLANNER");
    h.run();
    let found = has(&h, "Weekly planner") && !has(&h, "Dot grid");
    h.get_by_label("Search").type_text("zzz");
    h.run();
    let none = has_part(&h, "Nothing matches");
    // Clear the box, pick My templates.
    h.state_mut().ui.welcome_search_clear();
    h.get_by_label("My templates").click();
    h.run();
    // Right-click: Remove asks first, in words that say it cannot be undone; Cancel keeps the file.
    h.get_by_label("Dot grid").click_secondary();
    h.run();
    let menu = has(&h, "Remove") && has(&h, "Show in folder");
    h.get_by_label("Show in folder").click();
    h.run();
    let reveal = h
        .state()
        .requests
        .contains(&Request::RevealFolder(t.clone()));
    h.get_by_label("Dot grid").click_secondary();
    h.run();
    h.get_by_label("Remove").click();
    h.run();
    let confirm = h.state().ui.welcome_question_open() && has_part(&h, "cannot be undone");
    press(&mut h, "Cancel");
    let kept = t.join("dot-grid.quad").is_file() && has(&h, "Dot grid");
    h.get_by_label("Dot grid").click_secondary();
    h.run();
    h.get_by_label("Remove").click();
    h.run();
    h.get_all_by_label("Remove").last().unwrap().hover();
    h.run();
    h.get_all_by_label("Remove").last().unwrap().click();
    h.run();
    let gone =
        !t.join("dot-grid.quad").exists() && !has(&h, "Dot grid") && has(&h, "Weekly planner");
    let ok = both && paper_empty && found && none && menu && reveal && confirm && kept && gone;
    println!(
        "CHECK crate=ui name=my_templates_remove_and_search result={} all_lists_templates={both} paper_tab_empty={paper_empty} search_by_name={found} search_none_says_so={none} right_click_menu={menu} show_in_folder_asks_app={reveal} confirm_says_cannot_undo={confirm} cancel_keeps={kept} remove_deletes={gone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_looks_start_on_the_welcome_screen_and_on_the_open_file_menu() {
    let dir = tmp("welcome-looks");
    let mut h = window(&dir);
    h.state_mut().ui.show_look("welcome");
    h.run();
    let welcome = h.state().ui.welcome_showing() && has(&h, "+ Blank page");
    let mut h = window(&dir);
    h.state_mut().ui.viewer.open_sample();
    h.state_mut().ui.show_look("file-menu");
    h.run();
    h.run();
    let menu = has_part(&h, "Save as template…") && has_part(&h, "Open Recent");
    let ok = welcome && menu;
    println!(
        "CHECK crate=ui name=welcome_looks result={} welcome={welcome} file_menu_open={menu}",
        pass(ok)
    );
    assert!(ok);
}
