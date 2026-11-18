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

//! The Export sheet and the completion sheet driven with the pointer and the keyboard (rung 15 bbq, Q2, Q7 and Q9),
//! headless with egui_kittest. Every export goes into a scratch folder and every settings file is a scratch file.
//! Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_ui::export_ui::reveal_command;
use quadrule_ui::{DevicesChoice, Request, Settings, Viewer, ViewerUi};
use std::path::PathBuf;

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

/// The sample page in a window whose settings live in `dir/settings.ron` and whose last folder is `dir/out`.
fn window(dir: &std::path::Path) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
    v.open_sample();
    v.export_date = Some("2026-11-17".into());
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();
    v.settings.last_folder = Some(out.to_string_lossy().into_owned());
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

/// Whether some node has exactly this label (a name may appear in the toolbar too).
fn has(h: &Harness<'static, Window>, label: &str) -> bool {
    h.query_all_by_label(label).count() > 0
}

fn has_part(h: &Harness<'static, Window>, part: &str) -> bool {
    h.query_all_by_label_contains(part).count() > 0
}

fn settle(h: &mut Harness<'static, Window>) {
    // A sheet fades in over a few frames; the pointer goes to where the button ends up.
    for _ in 0..6 {
        h.step();
    }
}

fn press(h: &mut Harness<'static, Window>, label: &str) {
    settle(h);
    h.get_by_label(label).hover();
    h.run();
    h.get_by_label(label).click();
    h.run();
}

/// The Export sheet's Export button (File > Export… is a menu item, not a button, so this is the only one).
fn go(h: &mut Harness<'static, Window>) {
    settle(h);
    let n = h.get_all_by_role_and_label(Role::Button, "Export").count();
    h.get_all_by_role_and_label(Role::Button, "Export")
        .nth(n - 1)
        .unwrap()
        .hover();
    h.run();
    h.get_all_by_role_and_label(Role::Button, "Export")
        .nth(n - 1)
        .unwrap()
        .click();
    h.run();
}

fn open_sheet(h: &mut Harness<'static, Window>) {
    h.get_by_label("File").click();
    h.run();
    h.run();
    h.get_by_label_contains("Export…").click();
    h.run();
    settle(h);
}

fn files_under(dir: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(d: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, base, out);
            } else {
                out.push(
                    p.strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    walk(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn the_export_menu_opens_one_sheet_with_the_remembered_choices() {
    let dir = tmp("sheets-open");
    let mut h = window(&dir);
    open_sheet(&mut h);
    let open = h.state().ui.export_sheet_open();
    let labels = has(&h, "Devices")
        && has(&h, "This device: Manta")
        && has(&h, "All devices: Manta, Nomad")
        && has(&h, "Folder")
        && has(&h, "Change…")
        && has(&h, "Path")
        && has(&h, "Set")
        && h.query_by_label("Add a text note (app, version, title)")
            .is_some()
        && has(&h, "Cancel");
    // The default path and the files it would write, one line each.
    let default = has(&h, "rung-1-slice-manta.png") && has(&h, "rung-1-slice-nomad.png");
    // Cancel closes it and writes nothing.
    press(&mut h, "Cancel");
    let closed = !h.state().ui.export_sheet_open();
    let nothing = files_under(&dir.join("out")).is_empty();
    // The File menu holds the one Export…, with Ctrl+E written beside it, and no Export again (Q9).
    h.get_by_label("File").click();
    h.run();
    h.run();
    let item = has_part(&h, "Export…");
    let no_again = !has_part(&h, "Export again");
    let ok = open && labels && default && closed && nothing && item && no_again;
    println!(
        "CHECK crate=ui name=export_sheet_opens result={} open={open} labels={labels} default_preview={default} cancel_closes={closed} writes_nothing={nothing} one_export_item={item} no_export_again={no_again}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_preset_changes_the_preview_and_every_choice_is_remembered() {
    let dir = tmp("sheets-preset");
    let mut h = window(&dir);
    open_sheet(&mut h);
    // The presets menu beside the path field.
    h.get_by_label("Presets").click();
    h.run();
    let three = has(&h, "Flat") && has(&h, "Folder per device") && has(&h, "Device, then set");
    h.get_by_label("Device, then set").click();
    h.run();
    let template = h.state().ui.viewer.settings.template.clone();
    let preset = template == "{device}/{set}/{title}-{device}.png"
        && has(&h, "manta/rung-1-slice-manta.png");
    // A Set fills its folder in.
    h.get_by_label("Set").click();
    h.run();
    h.get_by_label("Set").type_text("Planner 2027");
    h.run();
    let with_set = h
        .query_by_label("manta/planner-2027/rung-1-slice-manta.png")
        .is_some()
        && h.query_by_label("nomad/planner-2027/rung-1-slice-nomad.png")
            .is_some();
    // This device alone, and no note.
    h.get_by_label("This device: Manta").click();
    h.run();
    h.get_by_label("Add a text note (app, version, title)")
        .click();
    h.run();
    let one_file = h
        .query_by_label("manta/planner-2027/rung-1-slice-manta.png")
        .is_some()
        && h.query_by_label("nomad/planner-2027/rung-1-slice-nomad.png")
            .is_none();
    // All of it is in settings.ron already, and a new window reads it back.
    press(&mut h, "Cancel");
    let saved = Settings::load(&dir.join("settings.ron")).settings;
    let remembered = saved.template == "{device}/{set}/{title}-{device}.png"
        && saved.set == "Planner 2027"
        && saved.devices == DevicesChoice::This
        && !saved.note;
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
    let reread = v.settings == saved;
    let ok = three && preset && with_set && one_file && remembered && reread;
    println!(
        "CHECK crate=ui name=export_sheet_preset_and_memory result={} three_presets={three} preset={preset} set_fills_folder={with_set} this_device_one_file={one_file} saved_in_settings={remembered} read_back={reread}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_mistake_shows_in_red_and_holds_export_back() {
    let dir = tmp("sheets-mistake");
    let mut h = window(&dir);
    // All devices and a path with no device in it: both would write the same file.
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::All;
    h.state_mut().ui.viewer.settings.template = "{title}.png".into();
    open_sheet(&mut h);
    let said = h
        .query_by_label_contains("would both write rung-1-slice.png")
        .is_some();
    let n = h.get_all_by_role_and_label(Role::Button, "Export").count();
    let disabled = h
        .get_all_by_role_and_label(Role::Button, "Export")
        .nth(n - 1)
        .unwrap()
        .accesskit_node()
        .is_disabled();
    // An unknown word in braces is a mistake too.
    h.state_mut().ui.viewer.settings.template = "{title}-{colour}".into();
    h.run();
    let unknown = has_part(&h, "colour");
    // Fix it with a preset: Export is back.
    h.get_by_label("Presets").click();
    h.run();
    h.get_by_label("Flat").click();
    h.run();
    let n = h.get_all_by_role_and_label(Role::Button, "Export").count();
    let enabled = !h
        .get_all_by_role_and_label(Role::Button, "Export")
        .nth(n - 1)
        .unwrap()
        .accesskit_node()
        .is_disabled();
    let nothing = files_under(&dir.join("out")).is_empty();
    let ok = said && disabled && unknown && enabled && nothing;
    println!(
        "CHECK crate=ui name=export_sheet_mistake result={} said_in_words={said} export_greyed={disabled} unknown_token_said={unknown} preset_clears_it={enabled} nothing_written={nothing}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn export_writes_the_files_then_the_preflight_sheet_then_the_completion_sheet() {
    let dir = tmp("sheets-export");
    let out = dir.join("out");
    let mut h = window(&dir);
    h.state_mut().ui.viewer.settings.template = "{device}/{set}/{title}-{device}.png".into();
    h.state_mut().ui.viewer.settings.set = "planner-2027".into();
    open_sheet(&mut h);
    go(&mut h);
    // The sample page has warnings: the one preflight sheet, the Export sheet closed, nothing written yet.
    let pre = h.state().ui.preflight_open()
        && !h.state().ui.export_sheet_open()
        && !h.state().ui.completion_open()
        && files_under(&out).is_empty();
    press(&mut h, "Export anyway");
    let files = files_under(&out);
    let written = files
        == [
            "manta/planner-2027/rung-1-slice-manta.png",
            "nomad/planner-2027/rung-1-slice-nomad.png",
        ];
    // The completion sheet: count, the pictures' names, the folder, the note folded, Reveal and Done.
    let done_sheet = h.state().ui.completion_open()
        && !h.state().ui.preflight_open()
        && has(&h, "Exported 2 files")
        && h.query_by_label_contains("rung-1-slice-manta.png")
            .is_some()
        && h.query_by_label_contains("rung-1-slice-nomad.png")
            .is_some()
        && has(&h, "Reveal in folder")
        && has(&h, "Done")
        && !has_part(&h, "replaced");
    let folded = has(&h, "Where do these go?") && !has_part(&h, "Customization tab");
    // Open the note: it is plain words about MyStyle, and it is remembered.
    h.get_by_label("Where do these go?").click();
    h.run();
    let opened = has_part(&h, "Customization tab")
        && Settings::load(&dir.join("settings.ron"))
            .settings
            .completion_note_open;
    // Reveal in folder asks the app, with the folder all the files share.
    h.get_by_label("Reveal in folder").click();
    h.run();
    let revealed = h
        .state()
        .requests
        .contains(&Request::RevealFolder(out.clone()));
    press(&mut h, "Done");
    let gone = !h.state().ui.completion_open();
    // Again, into the same place: every file took the place of one, and the note stays open as it was left.
    open_sheet(&mut h);
    go(&mut h);
    press(&mut h, "Export anyway");
    let replaced = h.query_by_label_contains("2 files replaced").is_some()
        && has_part(&h, "Customization tab");
    let ok = pre && written && done_sheet && folded && opened && revealed && gone && replaced;
    println!(
        "CHECK crate=ui name=export_sheet_to_completion result={} preflight_first={pre} files={written} completion_sheet={done_sheet} note_folded={folded} fold_opens_and_is_saved={opened} reveal_asks_app={revealed} done_closes={gone} replaced_said_and_fold_remembered={replaced}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn ctrl_e_opens_the_sheet_with_the_last_choices_and_enter_writes() {
    let dir = tmp("sheets-again");
    let out = dir.join("out");
    let mut h = window(&dir);
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    // Ctrl+E opens the sheet and writes nothing; Export is the button in focus.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::E]);
    h.run();
    settle(&mut h);
    let sheet = h.state().ui.export_sheet_open() && files_under(&out).is_empty();
    let focused = h.get_by_role_and_label(Role::Button, "Export").is_focused();
    // With the sheet open the key does not stack another sheet, and Ctrl+Shift+E does nothing now.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::E]);
    h.run();
    h.key_combination_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, &[Key::E]);
    h.run();
    let not_stacked = h.state().ui.export_sheet_open() && !h.state().ui.preflight_open();
    // Enter writes: through the one preflight sheet (the sample page has warnings), then the completion sheet.
    h.key_press(Key::Enter);
    h.run();
    settle(&mut h);
    let held = h.state().ui.preflight_open()
        && !h.state().ui.export_sheet_open()
        && files_under(&out).is_empty();
    press(&mut h, "Export anyway");
    let first = files_under(&out) == ["rung-1-slice-manta.png"] && h.state().ui.completion_open();
    press(&mut h, "Done");
    std::fs::remove_file(out.join("rung-1-slice-manta.png")).unwrap();
    // The next Ctrl+E has the last choices filled in (this device only, the same folder): Enter repeats the export.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::E]);
    h.run();
    settle(&mut h);
    let filled = h.state().ui.export_sheet_open()
        && has(&h, "rung-1-slice-manta.png")
        && h.query_by_label("rung-1-slice-nomad.png").is_none()
        && h.get_by_role_and_label(Role::Button, "Export").is_focused();
    h.key_press(Key::Enter);
    h.run();
    settle(&mut h);
    press(&mut h, "Export anyway");
    let repeated = files_under(&out) == ["rung-1-slice-manta.png"];
    press(&mut h, "Done");
    // The File menu item does what the key does.
    h.get_by_label("File").click();
    h.run();
    h.get_all_by_label_contains("Export…")
        .next()
        .unwrap()
        .click();
    h.run();
    settle(&mut h);
    let menu = h.state().ui.export_sheet_open()
        && h.get_by_role_and_label(Role::Button, "Export").is_focused();
    press(&mut h, "Cancel");
    let ok = sheet && focused && not_stacked && held && first && filled && repeated && menu;
    println!(
        "CHECK crate=ui name=export_ctrl_e_enter result={} ctrl_e_opens_sheet={sheet} export_button_focused={focused} not_stacked_and_shift_e_gone={not_stacked} enter_goes_to_preflight={held} first_written={first} last_choices_filled={filled} enter_repeats={repeated} menu_item_same={menu}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn reveal_in_folder_names_the_systems_file_manager() {
    let (program, args) = reveal_command(std::path::Path::new("/some/folder"));
    let want = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let ok = program == want && args == [std::ffi::OsString::from("/some/folder")];
    println!(
        "CHECK crate=ui name=reveal_command result={} program={program}",
        pass(ok)
    );
    assert!(ok);
}
