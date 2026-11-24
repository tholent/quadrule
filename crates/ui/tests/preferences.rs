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

//! The Preferences window (rung 17 bbq, Q3 and Defaulted 4): Ctrl+, and File > Preferences… open it; Interface size
//! (Follow system, 75 to 200 %) draws the whole window larger or smaller on top of the system's scale and is kept as
//! a key the app saves; the theme is chosen there. Headless. Each test prints one structured `CHECK` line.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_ui::{InterfaceSize, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn window() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
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
fn ctrl_comma_and_the_file_menu_open_it_and_it_closes() {
    let mut h = window();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    h.run();
    let by_key = h.state().ui.preferences_showing();
    h.get_by_label("Close").click();
    h.run();
    let closed = !h.state().ui.preferences_showing();
    h.get_by_label("File").click();
    h.run();
    let item = h.get_all_by_label_contains("Preferences…").next().is_some();
    h.get_all_by_label_contains("Preferences…")
        .next()
        .unwrap()
        .click();
    h.run();
    let by_menu = h.state().ui.preferences_showing();
    let ok = by_key && closed && item && by_menu;
    println!(
        "\nCHECK crate=ui name=preferences_open result={} ctrl_comma={by_key} closes={closed} file_item={item} file_item_opens={by_menu}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn interface_size_scales_the_window_and_follow_system_is_the_default() {
    let mut h = window();
    let default =
        h.state().ui.interface_size == InterfaceSize::Follow && h.ctx.zoom_factor() == 1.0;
    h.state_mut().ui.open_preferences();
    h.run();
    // Pick 150 % in the combo.
    h.get_by_label("Interface size").click();
    h.run();
    h.get_by_label("150 %").click();
    h.run();
    h.run();
    let size = h.state().ui.interface_size == InterfaceSize::Percent(150);
    let scaled = (h.ctx.zoom_factor() - 1.5).abs() < 1e-4;
    // Every choice has a key the app keeps, and nothing else reads back.
    let keys = InterfaceSize::all()
        .into_iter()
        .all(|s| InterfaceSize::from_key(&s.key()) == Some(s))
        && InterfaceSize::from_key("300").is_none()
        && InterfaceSize::from_key("").is_none()
        && InterfaceSize::all().len() == 7;
    h.state_mut().ui.interface_size = InterfaceSize::Follow;
    h.run();
    let back = h.ctx.zoom_factor() == 1.0;
    let ok = default && size && scaled && keys && back;
    println!(
        "\nCHECK crate=ui name=preferences_size result={} default_follow={default} picked_150={size} zoom_factor_1_5={scaled} keys_round_trip={keys} follow_is_1_0={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_theme_is_chosen_in_preferences_not_in_view() {
    let mut h = window();
    h.state_mut().ui.open_preferences();
    h.run();
    h.get_by_label("Dark").click();
    h.run();
    let dark = h.state().ui.theme == egui::ThemePreference::Dark;
    h.get_by_label("Light").click();
    h.run();
    let light = h.state().ui.theme == egui::ThemePreference::Light;
    let ok = dark && light;
    println!(
        "\nCHECK crate=ui name=preferences_theme result={} dark={dark} light={light}",
        pass(ok)
    );
    assert!(ok);
}
