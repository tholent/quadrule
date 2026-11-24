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

//! The menus (fixes batch 2, docs/bbq/2026-11-18-fixes-before-16b.md, Q3, Q6 and Q7): the menu bar along the top of the
//! window, what each menu holds, Edit greyed when it cannot act, View > Snapping, Help > About, the status bar without
//! the switches, the window at 1440 × 720, and the one table the Mac's menu bar is built from. Headless with
//! egui_kittest. Each test prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::sample::ids::RULE;
use quadrule_ui::menus::{Entry, Kind, Native, app_menu, items_of, menus, window_menu};
use quadrule_ui::{Cmd, Platform, Request, Snap, Source, Viewer, ViewerUi};
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

/// A window of this size on nothing open, its settings in a scratch folder.
fn window(name: &str, size: egui::Vec2) -> Harness<'static, Window> {
    let dir = tmp(name);
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
    v.templates_dir = Some(dir.join("templates"));
    let mut h = Harness::builder()
        .with_size(size)
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

fn sample_window(name: &str) -> Harness<'static, Window> {
    let mut h = window(name, egui::vec2(1400.0, 1000.0));
    h.get_by_label("Open sample page").click();
    h.run();
    h
}

/// The sample page with the rule picked and nudged once: Undo, Copy, Cut and Duplicate can act.
fn picked(name: &str) -> Harness<'static, Window> {
    let mut h = sample_window(name);
    h.state_mut().ui.viewer.selection = vec![RULE];
    h.state_mut().ui.viewer.nudge(1.0, 0.0, false).unwrap();
    h.run();
    h
}

/// Opens a menu of the bar and lets its popup settle. Any menu still open is closed first: a click on the title of
/// an open menu would close it instead.
fn open_menu(h: &mut Harness<'static, Window>, title: &str) {
    egui::Popup::close_all(&h.ctx);
    h.run();
    h.get_by_label(title).hover();
    h.run();
    h.get_by_label(title).click();
    h.run();
    h.run();
}

/// The menu items with these words: an item's name carries its key after them ("Undo Ctrl+Z"). An open menu can lie
/// over the toolbar, so the toolbar's own button of the same words (Fit) is told apart by its row, not by height.
fn in_menu<'a>(h: &'a Harness<'static, Window>, label: &'a str) -> Vec<egui_kittest::Node<'a>> {
    let toolbar_top = h.get_by_label("Emoji").rect().top();
    // The words, then one key with no space in it ("Ctrl+Shift+S", "⇧⌘S"): "Save" is not "Save As… Ctrl+Shift+S".
    let item = move |l: &str| {
        l == label
            || l.strip_prefix(label)
                .and_then(|rest| rest.strip_prefix(' '))
                .is_some_and(|key| !key.is_empty() && !key.contains(char::is_whitespace))
    };
    h.query_all_by(move |n| n.label().is_some_and(|l| item(&l)))
        .filter(|n| n.rect().top() != toolbar_top)
        .collect()
}

fn menu_y(h: &Harness<'static, Window>, label: &str) -> Option<f32> {
    in_menu(h, label).first().map(|n| n.rect().center().y)
}

fn has_part(h: &Harness<'static, Window>, part: &str) -> bool {
    h.query_all_by_label_contains(part).count() > 0
}

#[test]
fn the_bar_holds_four_menus_in_order_and_each_menu_its_items_in_order() {
    let mut h = sample_window("menus-order");
    // File, Edit, View and Help, left to right, along the top, above the toolbar (whose Undo is its first button).
    let x_of = |h: &Harness<'static, Window>, l: &str| h.get_by_label(l).rect().center().x;
    let y_of = |h: &Harness<'static, Window>, l: &str| h.get_by_label(l).rect().center().y;
    let titles = ["File", "Edit", "View", "Help"];
    let xs: Vec<f32> = titles.iter().map(|t| x_of(&h, t)).collect();
    let in_order = xs.windows(2).all(|w| w[0] < w[1]);
    let bar_y = y_of(&h, "File");
    let on_one_line = titles.iter().all(|t| (y_of(&h, t) - bar_y).abs() < 1.0);
    let above_toolbar = bar_y < y_of(&h, "Undo");
    // No fifth menu: nothing else of the bar sits on its line.
    let toolbar_has_no_menus =
        h.query_all_by_label("File").count() == 1 && h.query_all_by_label("View").count() == 1;
    let strings = quadrule_i18n::Strings::english();
    let mut per_menu = Vec::new();
    for menu in menus(Platform::Other) {
        let title = match menu.label {
            "app-file" => "File",
            "menu-edit" => "Edit",
            "app-view" => "View",
            _ => "Help",
        };
        open_menu(&mut h, title);
        let mut last = f32::MIN;
        let mut ok = true;
        for e in &menu.entries {
            let words = match e {
                Entry::Item(i) => Some(strings.get(i.label)),
                Entry::Sub(s) => Some(strings.get(s.label)),
                Entry::Recent => Some(strings.get("app-file-open-recent")),
                _ => None,
            };
            if let Some(w) = words {
                match menu_y(&h, &w) {
                    Some(y) if y > last => last = y,
                    _ => ok = false,
                }
            }
        }
        per_menu.push(ok);
        h.key_press(Key::Escape);
        h.run();
    }
    let items_ok = per_menu.iter().all(|b| *b);
    // The File menu is exactly what it was: the same eight items, the keys shown beside them.
    open_menu(&mut h, "File");
    let file = [
        "New…",
        "Open…",
        "Open Recent",
        "Save",
        "Save As…",
        "Save as template…",
        "Export…",
        "Close",
    ]
    .iter()
    .all(|i| in_menu(&h, i).len() == 1);
    // The keys as this system writes them (Ctrl+N here, ⌘N on the Mac).
    let cmd = Modifiers::COMMAND;
    let keys = [
        (cmd, Key::N),
        (cmd, Key::O),
        (cmd, Key::S),
        (cmd | Modifiers::SHIFT, Key::S),
        (cmd, Key::E),
        (cmd, Key::W),
    ]
    .iter()
    .all(|(m, k)| {
        has_part(
            &h,
            &h.ctx.format_shortcut(&egui::KeyboardShortcut::new(*m, *k)),
        )
    });
    let ok = in_order
        && on_one_line
        && above_toolbar
        && toolbar_has_no_menus
        && items_ok
        && file
        && keys;
    println!(
        "\nCHECK crate=ui name=menu_bar_order result={} four_in_order={in_order} one_line={on_one_line} above_toolbar={above_toolbar} toolbar_has_no_file_or_view={toolbar_has_no_menus} each_menu_items_in_order={items_ok} file_items={file} file_keys={keys}",
        pass(ok)
    );
    assert!(ok);
}

/// Whether the menu item `label` shows greyed.
fn greyed(h: &Harness<'static, Window>, label: &str) -> bool {
    in_menu(h, label)
        .first()
        .unwrap_or_else(|| panic!("no menu item {label}"))
        .accesskit_node()
        .is_disabled()
}

#[test]
fn edit_items_are_greyed_exactly_when_they_cannot_act() {
    let edit = [
        (Cmd::Undo, "Undo"),
        (Cmd::Redo, "Redo"),
        (Cmd::Cut, "Cut"),
        (Cmd::Copy, "Copy"),
        (Cmd::Paste, "Paste"),
        (Cmd::Duplicate, "Duplicate"),
        (Cmd::SelectAll, "Select all"),
    ];
    // The states the app reports, and the same read off what is drawn.
    let report = |h: &Harness<'static, Window>| -> Vec<bool> {
        edit.iter()
            .map(|(c, _)| h.state().ui.cmd_state(*c).enabled)
            .collect()
    };
    let drawn = |h: &mut Harness<'static, Window>| -> Vec<bool> {
        open_menu(h, "Edit");
        let v = edit.iter().map(|(_, l)| !greyed(h, l)).collect();
        h.key_press(Key::Escape);
        h.run();
        v
    };
    // Nothing open: the welcome screen. Everything is greyed.
    let mut h = window("menus-edit-none", egui::vec2(1400.0, 1000.0));
    let none = report(&h);
    let none_drawn = drawn(&mut h);
    // The sample page, nothing picked: Paste and Select all only.
    let mut h = sample_window("menus-edit-page");
    let page = report(&h);
    let page_drawn = drawn(&mut h);
    // A rule picked and moved: Undo, Cut, Copy and Duplicate too; Redo only after an Undo.
    let mut h = picked("menus-edit-picked");
    let pick = report(&h);
    let pick_drawn = drawn(&mut h);
    // Esc closed the menu in `drawn` and nothing else: the rule is still picked.
    let esc_keeps_pick = h.state().ui.viewer.selection == vec![RULE];
    h.state_mut().ui.viewer.undo();
    h.run();
    let undone = report(&h);
    let undone_drawn = drawn(&mut h);
    let want_none = vec![false; 7];
    let want_page = vec![false, false, false, false, true, false, true];
    let want_pick = vec![true, false, true, true, true, true, true];
    let want_undone = vec![false, true, true, true, true, true, true];
    let ok = esc_keeps_pick
        && none == want_none
        && none_drawn == want_none
        && page == want_page
        && page_drawn == want_page
        && pick == want_pick
        && pick_drawn == want_pick
        && undone == want_undone
        && undone_drawn == want_undone;
    println!(
        "\nCHECK crate=ui name=menu_edit_greyed result={} esc_closing_the_menu_keeps_the_pick={esc_keeps_pick} nothing_open={} page_only={} picked_and_moved={} after_undo={} drawn_matches_reported={}",
        pass(ok),
        none == want_none && none_drawn == want_none,
        page == want_page && page_drawn == want_page,
        pick == want_pick && pick_drawn == want_pick,
        undone == want_undone && undone_drawn == want_undone,
        none == none_drawn && page == page_drawn && pick == pick_drawn && undone == undone_drawn
    );
    assert!(ok);
}

#[test]
fn view_snapping_flips_the_same_settings_the_status_bar_switches_did() {
    let mut h = sample_window("menus-snapping");
    let flags = |h: &Harness<'static, Window>| {
        let s = &h.state().ui.viewer;
        [
            s.snap.pixel_grid,
            s.snap.rhythm,
            s.snap.margins_safe,
            s.snap.points,
            s.snap.smart_guides,
            s.snap.equal_spacing,
            s.show_rhythm,
        ]
    };
    let names = [
        "Pixel grid",
        "Rhythm",
        "Margins and safe zones",
        "Point snap",
        "Smart guides",
        "Equal spacing",
        "Rhythm overlay",
    ];
    let start = flags(&h);
    let mut flipped = Vec::new();
    for (i, name) in names.iter().enumerate() {
        open_menu(&mut h, "View");
        h.get_by_label("Snapping").hover();
        h.run();
        h.run();
        let before = flags(&h);
        // The item says what the setting is: ticked when on.
        let shown_on = in_menu(&h, name)
            .first()
            .and_then(|n| n.accesskit_node().toggled())
            == Some(egui::accesskit::Toggled::from(before[i]));
        in_menu(&h, name)[0].click();
        h.run();
        h.run();
        let after = flags(&h);
        let mut want = before;
        want[i] = !want[i];
        flipped.push(after == want && shown_on);
    }
    let all_flipped = flipped.iter().all(|b| *b);
    // The same settings, so the Page tab's switches and the commands agree, and flip back.
    let via_cmd = {
        let ctx = h.ctx.clone();
        let _ = h.state_mut().ui.run_cmd(&ctx, Cmd::Snap(Snap::Pixel));
        flags(&h)[0] == start[0]
    };
    let ok = all_flipped && via_cmd && start == [true, true, true, true, true, true, false];
    println!(
        "\nCHECK crate=ui name=menu_view_snapping result={} seven_items_flip_their_setting={all_flipped} same_flag_as_command={via_cmd} defaults_as_before={}",
        pass(ok),
        start == [true, true, true, true, true, true, false]
    );
    assert!(ok);
}

#[test]
fn view_holds_zoom_the_strip_and_what_the_toolbars_view_menu_held() {
    let mut h = sample_window("menus-view");
    // Zoom in, out, Fit and Actual size do what the + and − and Fit and 1:1 controls do.
    let mut zooms = Vec::new();
    for item in ["Actual size (1:1)", "Zoom in", "Zoom out", "Fit"] {
        open_menu(&mut h, "View");
        in_menu(&h, item)[0].click();
        h.run();
        zooms.push(h.state().ui.viewer.zoom);
    }
    let by_menu = zooms
        == [
            quadrule_ui::Zoom::Scale(1.0),
            quadrule_ui::Zoom::Scale(2.0),
            quadrule_ui::Zoom::Scale(1.0),
            quadrule_ui::Zoom::Fit,
        ];
    // The toolbar's own buttons, as they were, reach the same places.
    h.get_by_label("1:1").click();
    h.run();
    let one = h.state().ui.viewer.zoom == quadrule_ui::Zoom::Scale(1.0);
    h.get_by_label("+").click();
    h.run();
    let two = h.state().ui.viewer.zoom == quadrule_ui::Zoom::Scale(2.0);
    // The device strip: the item is ticked while it shows, and hides it.
    let strip_was = h.state().ui.viewer.show_strip;
    open_menu(&mut h, "View");
    let ticked = in_menu(&h, "Device previews")
        .first()
        .and_then(|n| n.accesskit_node().toggled())
        == Some(egui::accesskit::Toggled::True);
    in_menu(&h, "Device previews")[0].click();
    h.run();
    let hidden =
        strip_was && !h.state().ui.viewer.show_strip && h.query_by_label("Show Manta").is_none();
    // The theme moved to the Preferences window (rung 17): it is not in View any more.
    open_menu(&mut h, "View");
    let theme_gone = h.query_by_label("Theme").is_none();
    h.key_press(egui::Key::Escape);
    h.run();
    let dark = theme_gone;
    let no_toolbar_view = h.query_all_by_label("View").count() == 1;
    let ok = by_menu && one && two && ticked && hidden && dark && no_toolbar_view;
    println!(
        "\nCHECK crate=ui name=menu_view_zoom_and_strip result={} zoom_items={by_menu} toolbar_buttons_agree={} strip_ticked={ticked} strip_hides={hidden} theme_left_view={dark} one_view={no_toolbar_view}",
        pass(ok),
        one && two
    );
    assert!(ok);
}

#[test]
fn help_has_about_with_the_version_and_the_project_page_opens_in_the_browser() {
    let mut h = sample_window("menus-help");
    open_menu(&mut h, "Help");
    let both =
        in_menu(&h, "About quadrule").len() == 1 && in_menu(&h, "quadrule on GitHub").len() == 1;
    in_menu(&h, "About quadrule")[0].click();
    h.run();
    h.run();
    let version = env!("CARGO_PKG_VERSION");
    let shows_version = has_part(&h, &format!("Version {version}"));
    let licence = has_part(&h, "Apache-2.0");
    let address = has_part(&h, quadrule_ui::PROJECT_URL);
    let name = h.query_all_by_label("quadrule").count() >= 1;
    h.get_by_label("Close").click();
    h.run();
    let closes = !has_part(&h, "Apache-2.0");
    // The project page: the window asks the system to open the address.
    open_menu(&mut h, "Help");
    in_menu(&h, "quadrule on GitHub")[0].click();
    h.step();
    let opened =
        h.output().platform_output.commands.iter().any(
            |c| matches!(c, egui::OutputCommand::OpenUrl(u) if u.url == quadrule_ui::PROJECT_URL),
        );
    let ok = both && shows_version && licence && address && name && closes && opened;
    println!(
        "\nCHECK crate=ui name=menu_help_about result={} both_items={both} version_shown={shows_version} licence_shown={licence} address_shown={address} name_shown={name} close_closes={closes} project_page_opens_url={opened}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_status_bar_no_longer_holds_the_seven_switches() {
    let h = sample_window("menus-status");
    // The bottom line still tells the zoom and the lint count; the switches (Q3) live in View and the Page tab.
    let zoom = h.query_all_by_label_contains("zoom ").count() >= 1;
    let lint = h.query_all_by_label_contains("⚠").count() >= 1;
    let names = [
        "Pixel grid",
        "Rhythm",
        "Margins and safe zones",
        "Point snap",
        "Smart guides",
        "Equal spacing",
        "Rhythm overlay",
    ];
    let status_y = h.get_by_label_contains("zoom ").rect().center().y;
    let none_in_status = names.iter().all(|n| {
        h.query_all_by_label(n)
            .all(|x| (x.rect().center().y - status_y).abs() > 2.0)
    });
    let ok = zoom && lint && none_in_status;
    println!(
        "\nCHECK crate=ui name=status_bar_without_switches result={} zoom_and_lint_kept={} none_on_the_status_line={none_in_status}",
        pass(ok),
        zoom && lint
    );
    assert!(ok);
}

/// What a command leaves behind, for comparing a key with its menu item.
fn fingerprint(h: &Harness<'static, Window>, asked: &[Request]) -> String {
    let u = &h.state().ui;
    format!(
        "{:?}|prefs={}|keys={}|dirty={}|picked={}|kept_pick={}|welcome={}|export={}|new={}|prompt={}|zoom={:?}|doc={:?}",
        asked,
        u.preferences_showing(),
        u.shortcuts_showing(),
        u.viewer.is_dirty(),
        // How many, and whether the rule is still among them: a paste or a duplicate picks new objects, whose ids
        // differ from window to window.
        u.viewer.selection.len(),
        u.viewer.selection.contains(&RULE),
        u.welcome_showing(),
        u.export_sheet_open(),
        u.new_dialog_open(),
        u.prompt_open(),
        u.viewer.zoom,
        u.viewer.doc().map(|d| d.objects().count()),
    )
}

/// The text a frame sent to the clipboard.
fn copied(h: &Harness<'static, Window>) -> Option<String> {
    h.output()
        .platform_output
        .commands
        .iter()
        .find_map(|c| match c {
            egui::OutputCommand::CopyText(t) => Some(t.clone()),
            _ => None,
        })
}

/// One window gets the key of `cmd`'s item, another the item itself; `None` when they end the same and the item did
/// something, else what differed.
fn key_against_item(cmd: Cmd) -> Option<String> {
    let item = items_of(&menus(Platform::Other))
        .into_iter()
        .find(|i| i.cmd == cmd)
        .unwrap();
    let shortcut = item.shortcut.unwrap();
    let id = cmd.id();
    let setup = |name: &str| {
        let mut h = picked(name);
        if cmd == Cmd::Redo {
            h.state_mut().ui.viewer.undo();
            h.run();
        }
        h
    };
    // The key.
    let mut k = setup(&format!("menus-key-{id}"));
    k.state_mut().requests.clear();
    match cmd {
        Cmd::Copy => k.input_mut().events.push(Event::Copy),
        Cmd::Cut => k.input_mut().events.push(Event::Cut),
        Cmd::Paste => {
            // What Ctrl+V gives egui: the clipboard's text, here the same objects, copied first.
            let text = k.state_mut().ui.viewer.copy_text().unwrap();
            k.input_mut().events.push(Event::Paste(text));
        }
        _ => k.key_combination_modifiers(shortcut.modifiers, &[shortcut.logical_key]),
    }
    k.step();
    let key_copied = copied(&k);
    k.run();
    let key_asked = k.state().requests.clone();
    let by_key = fingerprint(&k, &key_asked);
    // The item.
    let mut m = setup(&format!("menus-item-{id}"));
    m.state_mut().requests.clear();
    let ctx = m.ctx.clone();
    let paste_text = (cmd == Cmd::Paste).then(|| m.state_mut().ui.viewer.copy_text().unwrap());
    let asked = m.state_mut().ui.run_cmd(&ctx, cmd);
    if let Some(t) = paste_text {
        // Eframe answers the window's request for the clipboard with this event on the next frame.
        m.input_mut().events.push(Event::Paste(t));
    }
    m.step();
    let item_copied = copied(&m);
    m.run();
    let by_item = fingerprint(&m, &asked);
    // Something happened: the item's window differs from one that did nothing, or it copied.
    let idle = fingerprint(&setup(&format!("menus-none-{id}")), &[]);
    let acted = idle != by_item || item_copied.is_some();
    let mut wrong = Vec::new();
    if by_key != by_item || key_copied != item_copied {
        wrong.push(format!("key[{by_key}]item[{by_item}]"));
    }
    if !acted {
        wrong.push("did_nothing".into());
    }
    // Redo's other key (Ctrl+Shift+Z beside Ctrl+Y; the Mac shows ⇧⌘Z) does the same.
    if cmd == Cmd::Redo {
        let mut k2 = setup(&format!("menus-key2-{id}"));
        k2.key_combination_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, &[Key::Z]);
        k2.run();
        let asked2 = k2.state().requests.clone();
        if fingerprint(&k2, &asked2) != by_item {
            wrong.push("second_key".into());
        }
    }
    (!wrong.is_empty()).then(|| format!("{id}:{}", wrong.join("+")))
}

/// Checks every item of `menu` that shows a key and prints its CHECK line. The Mac's items with keys are these same
/// commands (the table test checks it), so each is tried once.
fn keys_do_what_items_do(menu: &str, name: &str) {
    let menu = menus(Platform::Other)
        .into_iter()
        .find(|m| m.label == menu)
        .unwrap();
    let cmds: Vec<Cmd> = items_of(std::slice::from_ref(&menu))
        .into_iter()
        .filter(|i| i.shortcut.is_some())
        .map(|i| i.cmd)
        .collect();
    let wrong: Vec<String> = cmds.iter().filter_map(|c| key_against_item(*c)).collect();
    let ok = wrong.is_empty() && cmds.len() >= 6;
    println!(
        "\nCHECK crate=ui name={name} result={} items_with_keys={} wrong={wrong:?}",
        pass(ok),
        cmds.len()
    );
    assert!(ok);
}

#[test]
fn every_file_item_with_a_key_does_what_that_key_does() {
    keys_do_what_items_do("app-file", "menu_file_keys_do_what_items_do");
}

#[test]
fn every_edit_item_with_a_key_does_what_that_key_does() {
    keys_do_what_items_do("menu-edit", "menu_edit_keys_do_what_items_do");
}

#[test]
fn the_macs_menus_come_from_the_same_table() {
    let mac = menus(Platform::Mac);
    let other = menus(Platform::Other);
    let titles = |m: &[quadrule_ui::menus::Menu]| m.iter().map(|m| m.label).collect::<Vec<_>>();
    let mac_order = titles(&mac)
        == [
            "menu-app",
            "app-file",
            "menu-edit",
            "app-view",
            "menu-window",
            "menu-help",
        ];
    let other_order = titles(&other) == ["app-file", "menu-edit", "app-view", "menu-help"];
    // The Quadrule menu: About, Preferences… (⌘,), Hide, Hide Others, Show All, Quit (⌘Q); Window: Minimize and Zoom.
    let app = app_menu();
    let cmds: Vec<Cmd> = items_of(std::slice::from_ref(&app))
        .iter()
        .map(|i| i.cmd)
        .collect();
    let natives: Vec<Native> = app
        .entries
        .iter()
        .chain(window_menu().entries.iter())
        .filter_map(|e| match e {
            Entry::Native(n, _) => Some(*n),
            _ => None,
        })
        .collect();
    let app_ok = cmds == [Cmd::About, Cmd::Preferences, Cmd::Quit]
        && natives
            == [
                Native::Hide,
                Native::HideOthers,
                Native::ShowAll,
                Native::Minimize,
                Native::ZoomWindow,
            ];
    // File, Edit and View hold the same commands on both; Help differs only by About; every id round-trips.
    let cmds_of = |m: &[quadrule_ui::menus::Menu], label: &str| -> Vec<Cmd> {
        items_of(&[m.iter().find(|x| x.label == label).unwrap().clone()])
            .iter()
            .map(|i| i.cmd)
            .collect()
    };
    // File differs by Preferences… (the Mac has it in the Quadrule menu).
    let file_mac = cmds_of(&mac, "app-file");
    let file_bar: Vec<Cmd> = cmds_of(&other, "app-file")
        .into_iter()
        .filter(|c| *c != Cmd::Preferences)
        .collect();
    let same = file_mac == file_bar
        && cmds_of(&other, "app-file").contains(&Cmd::Preferences)
        && ["menu-edit", "app-view"]
            .iter()
            .all(|l| cmds_of(&mac, l) == cmds_of(&other, l));
    let help_ok = cmds_of(&mac, "menu-help")
        == [Cmd::ShortcutsSheet, Cmd::ProjectPage, Cmd::ExportJson]
        && cmds_of(&other, "menu-help")
            == [
                Cmd::About,
                Cmd::ShortcutsSheet,
                Cmd::ProjectPage,
                Cmd::ExportJson,
            ];
    let all: Vec<_> = [&mac[..], &other[..]]
        .into_iter()
        .flat_map(items_of)
        .collect();
    let round_trip = all.iter().all(|i| Cmd::from_id(&i.cmd.id()) == Some(i.cmd))
        && Cmd::from_id(&Cmd::OpenRecent(3).id()) == Some(Cmd::OpenRecent(3));
    // Every word is in the Fluent file.
    let strings = quadrule_i18n::Strings::english();
    let words = all.iter().all(|i| strings.has(i.label))
        && mac.iter().chain(other.iter()).all(|m| strings.has(m.label));
    let kinds = all
        .iter()
        .filter(|i| matches!(i.cmd, Cmd::Snap(_) | Cmd::SafeZones | Cmd::Strip))
        .all(|i| i.kind == Kind::Check);
    // The Mac asks for ⌘ keys; the Redo key differs by system, as each system's own programs show it.
    let redo = |m: &[quadrule_ui::menus::Menu]| {
        items_of(m)
            .iter()
            .find(|i| i.cmd == Cmd::Redo)
            .and_then(|i| i.shortcut)
    };
    let redo_ok = redo(&mac).map(|s| (s.modifiers, s.logical_key))
        == Some((Modifiers::COMMAND | Modifiers::SHIFT, Key::Z))
        && redo(&other).map(|s| (s.modifiers, s.logical_key)) == Some((Modifiers::COMMAND, Key::Y));
    // The Mac's items with keys are the other systems' plus Preferences… and Quit, so testing the keys once covers both.
    let keyed = |m: &[quadrule_ui::menus::Menu]| {
        items_of(m)
            .iter()
            .filter(|i| i.shortcut.is_some() && !matches!(i.cmd, Cmd::Preferences | Cmd::Quit))
            .map(|i| i.cmd)
            .collect::<Vec<_>>()
    };
    let keyed_same = keyed(&mac) == keyed(&other);
    let ok = keyed_same
        && mac_order
        && other_order
        && app_ok
        && same
        && help_ok
        && round_trip
        && words
        && kinds
        && redo_ok;
    println!(
        "\nCHECK crate=ui name=menu_table_for_the_mac result={} mac_order={mac_order} bar_order={other_order} app_menu_and_native_items={app_ok} file_edit_view_shared={same} help_differs_by_about={help_ok} ids_round_trip={round_trip} every_word_in_fluent={words} switches_are_checks={kinds} redo_keys={redo_ok} mac_keyed_items_same={keyed_same}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn quit_goes_through_the_unsaved_changes_prompt_and_preferences_opens() {
    // Quit with unsaved work asks first, exactly as closing the window does; clean, it closes.
    let mut h = picked("menus-quit");
    let ctx = h.ctx.clone();
    let asked = h.state_mut().ui.run_cmd(&ctx, Cmd::Quit);
    h.run();
    let prompt = asked.is_empty() && h.state().ui.prompt_open();
    let mut c = sample_window("menus-quit-clean");
    let ctx = c.ctx.clone();
    let closes = c.state_mut().ui.run_cmd(&ctx, Cmd::Quit) == vec![Request::Close];
    // Preferences… opens the Preferences window (it was the Profiles window until there was one, chris, 2026-11-20).
    let ctx = c.ctx.clone();
    let _ = c.state_mut().ui.run_cmd(&ctx, Cmd::Preferences);
    c.run();
    let profiles = c.state().ui.preferences_showing() && !c.state().ui.profiles_window_open();
    // While the question shows, the other commands wait, as the keys do.
    let ctx = h.ctx.clone();
    let held = h.state_mut().ui.run_cmd(&ctx, Cmd::Open).is_empty();
    let ok = prompt && closes && profiles && held;
    println!(
        "\nCHECK crate=ui name=menu_quit_and_settings result={} quit_asks_first={prompt} quit_clean_closes={closes} preferences_opens={profiles} commands_wait_for_the_prompt={held}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn edit_commands_reach_typing_on_the_page_as_their_keys_would() {
    // On the Mac the system menu takes the Edit keys before egui sees them; the item then hands egui the event the key
    // would have made, so typing on the page keeps its own select all, copy, cut and undo, and each acts once.
    use quadrule_text::edit::Op;
    let mut h = window("menus-text", egui::vec2(1400.0, 1000.0));
    h.state_mut().ui.viewer.open_textbox();
    h.run();
    let id = quadrule_model::sample::textbox_ids::TITLE;
    let began = h
        .state_mut()
        .ui
        .viewer
        .begin_typing(id, quadrule_ui::Start::End);
    for c in "abc".chars() {
        h.state_mut().ui.viewer.type_op(Op::Insert(c));
    }
    h.run();
    let ctx = h.ctx.clone();
    let text = h.state().ui.viewer.typing_text().unwrap();
    let doc_before = format!(
        "{:?}",
        h.state().ui.viewer.doc().map(|d| d.objects().count())
    );
    // Select all, then Copy: the text on the clipboard is all of it, the page's objects untouched.
    let a = h.state_mut().ui.run_cmd(&ctx, Cmd::SelectAll);
    h.step();
    let b = h.state_mut().ui.run_cmd(&ctx, Cmd::Copy);
    h.step();
    let all_copied = copied(&h).as_deref() == Some(text.as_str());
    // Cut takes it out of the box, once; Undo (typing's own) brings it back, once; Paste puts the clipboard back.
    let _ = h.state_mut().ui.run_cmd(&ctx, Cmd::Cut);
    h.step();
    let cut_gone = h.state().ui.viewer.typing_text().as_deref() == Some("");
    let _ = h.state_mut().ui.run_cmd(&ctx, Cmd::Undo);
    h.step();
    let undone = h.state().ui.viewer.typing_text().as_deref() == Some(text.as_str());
    let _ = h.state_mut().ui.run_cmd(&ctx, Cmd::Cut);
    h.step();
    let _ = h.state_mut().ui.run_cmd(&ctx, Cmd::Paste);
    h.input_mut().events.push(Event::Paste("zz".into()));
    h.step();
    let pasted = h.state().ui.viewer.typing_text().as_deref() == Some("zz");
    let doc_after = format!(
        "{:?}",
        h.state().ui.viewer.doc().map(|d| d.objects().count())
    );
    let untouched = a.is_empty() && b.is_empty() && doc_before == doc_after;
    let still_typing = h.state().ui.viewer.is_typing();
    let ok = began && all_copied && cut_gone && undone && pasted && untouched && still_typing;
    println!(
        "\nCHECK crate=ui name=menu_edit_while_typing result={} typing_began={began} select_all_then_copy_copies_all={all_copied} cut_takes_it_out={cut_gone} undo_brings_it_back={undone} paste_goes_to_the_box={pasted} page_objects_untouched={untouched} still_typing={still_typing}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_window_still_lays_out_at_1440_by_720() {
    let mut h = window("menus-layout", egui::vec2(1440.0, 720.0));
    h.get_by_label("Open sample page").click();
    h.run();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1440.0, 720.0));
    let rect = |l: &str| h.get_by_label(l).rect();
    let bar = rect("File");
    let tool = rect("Undo");
    let status = h.get_by_label_contains("zoom ").rect();
    let inside = [bar, tool, status].iter().all(|r| screen.contains_rect(*r));
    // The menu bar, then the toolbar, then (far below) the status line; the bars do not overlap.
    let stacked = bar.bottom() <= tool.top() + 1.0 && tool.bottom() < status.top();
    // The panels still show: the tools bar on the left, the inspector tabs and the layers on the right.
    let panels = h.query_by_label("Layers").is_some()
        && h.query_all_by_role(Role::Button).count() > 10
        && h.query_all_by_label("Devices").count() >= 1;
    // The toolbar still fits on its line or wraps within the window, never past its edge.
    let right_edge = h
        .query_all_by_role(Role::Button)
        .filter(|n| n.rect().center().y < 70.0)
        .all(|n| n.rect().right() <= 1440.0);
    let ok = inside && stacked && panels && right_edge;
    println!(
        "\nCHECK crate=ui name=menu_layout_1440x720 result={} bars_inside={inside} bar_over_toolbar_over_status={stacked} panels_show={panels} toolbar_within_width={right_edge}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_file_the_system_hands_over_opens_as_a_dropped_one_does() {
    // The Mac's Finder hands a double-clicked file to the program (quadrule-macos); the window opens it through the
    // unsaved-changes prompt, as a file dropped on the window.
    let dir = tmp("menus-system-open");
    let file = dir.join("handed.quad");
    let mut v = Viewer::default();
    v.open_sample();
    v.save_to(&file).unwrap();
    // Unsaved work: it asks first and opens nothing yet.
    let mut h = picked("menus-system-open-dirty");
    let asked = h.state_mut().ui.open_from_system(file.clone());
    h.run();
    let is_file = |u: &ViewerUi| {
        u.viewer
            .open
            .as_ref()
            .is_some_and(|o| o.source == Source::File(file.clone()))
    };
    let waits = asked.is_empty() && h.state().ui.prompt_open() && !is_file(&h.state().ui);
    // Nothing unsaved: it opens at once.
    let mut c = window("menus-system-open-clean", egui::vec2(1400.0, 1000.0));
    let asked = c.state_mut().ui.open_from_system(file.clone());
    c.run();
    let opened = asked.is_empty() && is_file(&c.state().ui);
    let ok = waits && opened;
    println!(
        "\nCHECK crate=ui name=menu_system_open result={} unsaved_asks_first={waits} clean_opens={opened}",
        pass(ok)
    );
    assert!(ok);
}
