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

//! The keyboard (rung 17 bbq, Q4 and Defaulted 5): one table of keys (`quadrule_ui::keys`) that the window answers, the
//! menus show and the Keyboard shortcuts sheet lists; align, distribute and zoom keys; Ctrl+A; the sheet under Help
//! (Ctrl+/); none of it while a text field has the keyboard; arrow keys in the font and emoji lists (D7). Headless.
//! Each test prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::ids;
use quadrule_ui::keys::{Act, Group, table};
use quadrule_ui::menus::{items_of, menus};
use quadrule_ui::{ArrangeOp, Platform, Tab, Tool, Viewer, ViewerUi, Zoom};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn window(select: &[quadrule_model::Uuid]) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    v.selection = select.to_vec();
    framed(v)
}

/// The shapes page with everything on it picked: boxes spread unevenly, so align and distribute both move some.
fn shapes_window() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_shapes();
    // One object moved sideways first, so the boxes are not evenly spread.
    v.select_all();
    let first = v.selection[0];
    v.selection = vec![first];
    v.nudge(37.0, 0.0, false).unwrap();
    v.select_all();
    framed(v)
}

fn framed(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(24)
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

fn doc_text(h: &Harness<'static, Window>) -> String {
    format!("{:?}", h.state().ui.viewer.doc().unwrap().layers)
}

const ALT: Modifiers = Modifiers::ALT;
const ALT_SHIFT: Modifiers = Modifiers {
    alt: true,
    shift: true,
    ..Modifiers::NONE
};

#[test]
fn the_sheet_lists_every_row_of_the_table_with_the_keys_the_window_answers() {
    let mut h = window(&[]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    h.run();
    let open = h.state().ui.shortcuts_showing();
    let ctx = h.ctx.clone();
    let rows: Vec<_> = table()
        .into_iter()
        .filter(|r| !r.mac_only || cfg!(target_os = "macos"))
        .collect();
    let mut missing = Vec::new();
    for r in &rows {
        let words = h.state().ui.text(r.words);
        if h.query_all_by_label(&words).count() == 0 {
            missing.push(words);
            continue;
        }
        // Each key, as this system writes it (the nudge rows and the lists' row say theirs in words).
        if !matches!(r.act, Act::Nudge | Act::NudgeRhythm | Act::Picker) {
            for (m, k) in r.keys.iter().take(r.shown) {
                let text = quadrule_ui::keys::key_text(&ctx, *m, *k);
                if h.query_all_by_label_contains(&text).count() == 0 {
                    missing.push(text);
                }
            }
        }
    }
    let groups = Group::ALL.iter().all(|g| {
        h.query_all_by_label(&h.state().ui.text(g.message_id()))
            .count()
            >= 1
    });
    // The new keys are in it.
    let shown = |m: Modifiers, k: Key| {
        let text = quadrule_ui::keys::key_text(&ctx, m, k);
        h.query_all_by_label_contains(&text).count() >= 1
    };
    let new_keys = shown(ALT, Key::A)
        && shown(ALT_SHIFT, Key::H)
        && shown(Modifiers::COMMAND, Key::Equals)
        && shown(Modifiers::SHIFT, Key::Num1)
        && shown(Modifiers::COMMAND, Key::Slash);
    // Ctrl+/ again closes it.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    h.run();
    let toggles = !h.state().ui.shortcuts_showing();
    let ok = open && missing.is_empty() && groups && new_keys && toggles;
    println!(
        "\nCHECK crate=ui name=keys_sheet result={} rows={} missing={missing:?} groups={groups} new_keys_listed={new_keys} ctrl_slash_toggles={toggles}",
        pass(ok),
        rows.len()
    );
    assert!(ok);
}

#[test]
fn the_menus_show_the_keys_of_the_table_and_the_sheet_is_under_help() {
    let mut agree = true;
    let mut wrong = Vec::new();
    for platform in [Platform::Mac, Platform::Other] {
        for item in items_of(&menus(platform)) {
            let want = item.cmd.act().and_then(quadrule_ui::keys::shortcut);
            let redo = item.cmd == quadrule_ui::Cmd::Redo;
            // The Redo item shows the system's own key; the table has both.
            let good = if redo {
                item.shortcut.is_some_and(|s| {
                    quadrule_ui::keys::row(Act::Redo)
                        .keys
                        .iter()
                        .any(|(m, k)| *m == s.modifiers && *k == s.logical_key)
                })
            } else {
                item.shortcut == want
            };
            if !good {
                agree = false;
                wrong.push(format!("{:?}", item.cmd));
            }
        }
    }
    // Every menu item that has a row shows its key (Select all, the zoom items and Preferences included).
    let zoom_in = items_of(&menus(Platform::Other))
        .iter()
        .find(|i| i.cmd == quadrule_ui::Cmd::ZoomIn)
        .and_then(|i| i.shortcut)
        .is_some();
    let help: Vec<_> = menus(Platform::Other)
        .into_iter()
        .find(|m| m.label == "menu-help")
        .map(|m| items_of(&[m]).iter().map(|i| i.cmd).collect())
        .unwrap_or_default();
    let in_help = help.contains(&quadrule_ui::Cmd::ShortcutsSheet);
    // Every row that has a menu item is shown there; the table's act for a menu command round-trips.
    let ok = agree && zoom_in && in_help;
    println!(
        "\nCHECK crate=ui name=keys_menus result={} items_agree_with_table={agree} wrong={wrong:?} zoom_in_has_key={zoom_in} sheet_in_help={in_help}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn no_code_but_the_table_reads_a_key_by_itself() {
    // The drift check: a key the window answers is a row of `keys::table`, read through `keys::consume`. Another place
    // that asks egui for a shortcut or a key press on its own would be a binding the sheet and the menus do not know.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stray = Vec::new();
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !name.ends_with(".rs") || name == "keys.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if [
                "consume_shortcut(",
                "consume_key(",
                "count_and_consume_key(",
            ]
            .iter()
            .any(|p| code.contains(p))
            {
                stray.push(format!("{name}:{}", n + 1));
            }
        }
    }
    println!(
        "\nCHECK crate=ui name=keys_no_strays result={} strays={stray:?}",
        pass(stray.is_empty())
    );
    assert!(stray.is_empty());
}

#[test]
fn align_and_distribute_keys_do_what_the_arrange_buttons_do() {
    // Alt+A D W S H V and Alt+Shift+H V: the same one undo step as the button, on the same objects.
    let mut parts = Vec::new();
    let mut ok = true;
    let keys: [(Modifiers, Key, ArrangeOp); 8] = {
        use quadrule_layout::align::{Edge, Spread};
        use quadrule_layout::units::Axis;
        [
            (ALT, Key::A, ArrangeOp::Align(Axis::X, Edge::Start)),
            (ALT, Key::D, ArrangeOp::Align(Axis::X, Edge::End)),
            (ALT, Key::W, ArrangeOp::Align(Axis::Y, Edge::Start)),
            (ALT, Key::S, ArrangeOp::Align(Axis::Y, Edge::End)),
            (ALT, Key::H, ArrangeOp::Align(Axis::X, Edge::Centre)),
            (ALT, Key::V, ArrangeOp::Align(Axis::Y, Edge::Centre)),
            (
                ALT_SHIFT,
                Key::H,
                ArrangeOp::Distribute(Axis::X, Spread::Spacing),
            ),
            (
                ALT_SHIFT,
                Key::V,
                ArrangeOp::Distribute(Axis::Y, Spread::Spacing),
            ),
        ]
    };
    for (mods, key, op) in keys {
        let mut by_key = shapes_window();
        let before = doc_text(&by_key);
        by_key.key_press_modifiers(mods, key);
        by_key.run();
        let mut by_button = shapes_window();
        by_button.state_mut().ui.viewer.arrange(op).unwrap();
        by_button.run();
        let same = doc_text(&by_key) == doc_text(&by_button);
        let moved = doc_text(&by_key) != before;
        // And not the V tool: the tool stays Select after Alt+V.
        let tool = by_key.state().ui.viewer.tool == Tool::Select;
        let good = same && moved && tool;
        ok &= good;
        parts.push(format!("{op:?}={good}"));
    }
    println!(
        "\nCHECK crate=ui name=keys_align result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn zoom_keys_step_fit_and_actual_and_the_whole_interface_keeps_its_size() {
    let mut h = window(&[]);
    h.get_by_label("Fit").click();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    h.run();
    let zin = matches!(h.state().ui.viewer.zoom, Zoom::Scale(s) if s > 0.0);
    let after_in = h.state().ui.viewer.zoom;
    h.key_press_modifiers(Modifiers::COMMAND, Key::Minus);
    h.run();
    let zout = h.state().ui.viewer.zoom != after_in;
    // Shift+0 is 1:1, Shift+1 is Fit; the keys are found by their place too (Shift+1 types `!` on some layouts).
    h.key_press_modifiers(Modifiers::SHIFT, Key::Num0);
    h.run();
    let actual = h.state().ui.viewer.zoom == Zoom::Scale(1.0);
    h.event(Event::Key {
        key: Key::Exclamationmark,
        physical_key: Some(Key::Num1),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::SHIFT,
    });
    h.run();
    let fit = h.state().ui.viewer.zoom == Zoom::Fit;
    // egui's own Ctrl+= resizes the whole interface; it is off (the page zooms instead).
    let steady = h.ctx.zoom_factor() == 1.0;
    let ok = zin && zout && actual && fit && steady;
    println!(
        "\nCHECK crate=ui name=keys_zoom result={} ctrl_equals_in={zin} ctrl_minus_out={zout} shift_0_actual={actual} shift_1_fit_by_place={fit} interface_size_unchanged={steady}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn select_all_and_alt_v_are_not_the_tool() {
    let mut h = window(&[]);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    let all = h.state().ui.viewer.selection.len() >= 3;
    h.state_mut().ui.viewer.set_tool(Tool::Line);
    h.key_press_modifiers(ALT, Key::V);
    h.run();
    let alt_v_not_tool = h.state().ui.viewer.tool == Tool::Line;
    h.key_press(Key::V);
    h.run();
    let plain_v_tool = h.state().ui.viewer.tool == Tool::Select;
    let ok = all && alt_v_not_tool && plain_v_tool;
    println!(
        "\nCHECK crate=ui name=keys_select_all result={} ctrl_a_selects_all={all} alt_v_leaves_the_tool={alt_v_not_tool} plain_v_picks_select={plain_v_tool}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn no_key_fires_while_a_text_field_has_the_keyboard() {
    // The Page tab's title field has the keyboard: Ctrl+A selects its text, not the objects; Alt+A and Ctrl+= do nothing
    // to the page.
    let mut h = window(&[]);
    h.state_mut().ui.tab = Tab::Page;
    h.run();
    h.get_by_role_and_label(egui::Role::TextInput, "Title")
        .click();
    h.run();
    let focus = h.ctx.egui_wants_keyboard_input();
    let zoom_before = h.state().ui.viewer.zoom;
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    let none_picked = h.state().ui.viewer.selection.is_empty();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    h.run();
    let zoom_same = h.state().ui.viewer.zoom == zoom_before;
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run();
    let no_prefs = !h.state().ui.preferences_showing();
    let ok = focus && none_picked && zoom_same && no_prefs;
    println!(
        "\nCHECK crate=ui name=keys_text_field result={} field_has_keys={focus} ctrl_a_leaves_the_page={none_picked} zoom_key_ignored={zoom_same} preferences_key_ignored={no_prefs}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn arrow_keys_walk_the_font_list_and_enter_picks() {
    let mut h = window(&[ids::TITLE]);
    h.state_mut().ui.tab = Tab::Text;
    h.run();
    h.get_by_label("Font").click();
    h.run();
    let shown = h.query_by_label("Search fonts").is_some();
    // Down, Down, Enter: the second family of the list (the first is where Down begins).
    let first_two: Vec<String> = h.state().ui.font_picker_names();
    h.key_press(Key::ArrowDown);
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    h.key_press(Key::Enter);
    h.run();
    let family = match &h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(ids::TITLE)
        .unwrap()
        .kind
    {
        quadrule_model::ObjectKind::Text(t) => t.family.clone(),
        _ => String::new(),
    };
    let picked = first_two.len() >= 2 && family == first_two[1];
    let closed = h.query_by_label("Search fonts").is_none();
    // The text box did not move while the list had the arrows.
    let ok = shown && picked && closed;
    println!(
        "\nCHECK crate=ui name=keys_font_list result={} opened={shown} picked_second={picked} (got {family:?}, want {:?}) closed={closed}",
        pass(ok),
        first_two.get(1)
    );
    assert!(ok);
}

#[test]
fn arrow_keys_walk_the_emoji_grid_and_enter_puts_one_in() {
    // Nothing picked, so Enter makes a new text box on the page (a picked box's field is drawn by egui, whose own fonts
    // lack the newest emoji).
    let mut h = window(&[]);
    h.get_all_by_label("Emoji").next().unwrap().click();
    h.run();
    h.run();
    let shown = h.query_by_label("Search emoji by name").is_some();
    let before = h.state().ui.viewer.doc().unwrap().objects().count();
    // Right, Right, Down, Left: the cursor starts on the first emoji, so this is cell 1 + cols - 1... checked by the
    // state the picker reports.
    h.key_press(Key::ArrowRight);
    h.run();
    let a = h.state().ui.emoji_picker_cursor();
    h.key_press(Key::ArrowRight);
    h.run();
    let b = h.state().ui.emoji_picker_cursor();
    h.key_press(Key::ArrowDown);
    h.run();
    let c = h.state().ui.emoji_picker_cursor();
    h.key_press(Key::ArrowLeft);
    h.run();
    let d = h.state().ui.emoji_picker_cursor();
    let list = h.state().ui.emoji_picker_shown();
    let at = |s: &Option<String>| s.as_ref().and_then(|e| list.iter().position(|x| x == e));
    let (ia, ib, ic, id) = (at(&a), at(&b), at(&c), at(&d));
    let walked = ia == Some(0)
        && ib == Some(1)
        && ic.is_some_and(|i| i > 1)
        && id.is_some_and(|i| ic.is_some_and(|c| c == i + 1));
    h.key_press(Key::Enter);
    h.run();
    let after = h.state().ui.viewer.doc().unwrap().objects().count();
    let chosen = d.clone().unwrap_or_default();
    let made_box = h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .objects()
        .any(|o| matches!(&o.kind, quadrule_model::ObjectKind::Text(t) if t.content == chosen));
    let inserted = after == before + 1 && made_box;
    let ok = shown && walked && inserted;
    println!(
        "\nCHECK crate=ui name=keys_emoji_grid result={} opened={shown} walked={walked} cells={ia:?},{ib:?},{ic:?},{id:?} enter_inserts={inserted}",
        pass(ok)
    );
    assert!(ok);
}
