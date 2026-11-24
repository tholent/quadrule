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

//! The inspector's tabs as one row of line icons (rung 17 bbq, Q8 and Defaulted 3): Style comes before Arrange, each
//! tab is a named button in one row, Lint keeps its count beside its icon, and the open tab's name stands under the
//! row. Headless. Each test prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::sample::ids;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn sample(select: bool) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    if select {
        v.selection = vec![ids::RULE];
    }
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 860.0))
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
    h.run();
    h
}

fn button(h: &Harness<'static, Window>, name: &str) -> egui::Rect {
    h.get_by_role_and_label(Role::Button, name).rect()
}

#[test]
fn the_tabs_are_named_icon_buttons_in_one_row_with_style_before_arrange() {
    let mut h = sample(true);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    let names = ["Basic", "Style", "Arrange", "Lint (5)"];
    let rects: Vec<egui::Rect> = names.iter().map(|n| button(&h, n)).collect();
    let one_row = rects
        .iter()
        .all(|r| (r.center().y - rects[0].center().y).abs() < 0.5);
    let order = rects.windows(2).all(|w| w[0].max.x <= w[1].min.x);
    // Icons, not text buttons: a tab button is a square 26 points wide (Lint's is wider, for its count).
    let squares = rects[..3].iter().all(|r| (r.width() - 26.0).abs() < 0.5);
    let lint_wider = rects[3].width() > 26.0;
    // The open tab's name stands under the row, small, as a label of its own (not a button).
    let heading = h
        .query_all_by_label("Style")
        .any(|n| n.accesskit_node().role() != Role::Button);
    let under = h
        .query_all_by_label("Style")
        .filter(|n| n.accesskit_node().role() != Role::Button)
        .any(|n| n.rect().min.y >= rects[0].max.y);
    let ok = one_row && order && squares && lint_wider && heading && under;
    println!(
        "\nCHECK crate=ui name=tab_icons result={} one_row={one_row} style_before_arrange={order} squares={squares} lint_wider={lint_wider} heading={heading} under_row={under}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_tab_button_is_picked_by_its_icon_and_the_open_one_is_selected() {
    let mut h = sample(true);
    h.get_by_role_and_label(Role::Button, "Arrange").click();
    h.run();
    let arrange = h.state().ui.tab == Tab::Arrange;
    h.get_by_role_and_label(Role::Button, "Style").click();
    h.run();
    let style = h.state().ui.tab == Tab::Style;
    let selected = h
        .get_by_role_and_label(Role::Button, "Style")
        .accesskit_node()
        .toggled()
        == Some(egui::accesskit::Toggled::True)
        && h.get_by_role_and_label(Role::Button, "Arrange")
            .accesskit_node()
            .toggled()
            == Some(egui::accesskit::Toggled::False);
    let ok = arrange && style && selected;
    println!(
        "\nCHECK crate=ui name=tab_icons_pick result={} arrange={arrange} style={style} selected_state={selected}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn nothing_selected_the_row_is_basic_page_and_lint() {
    let h = sample(false);
    let r: Vec<_> = ["Basic", "Page", "Lint (5)"]
        .iter()
        .map(|n| button(&h, n))
        .collect();
    let row = r
        .windows(2)
        .all(|w| w[0].max.x <= w[1].min.x && (w[0].center().y - w[1].center().y).abs() < 0.5);
    println!(
        "\nCHECK crate=ui name=tab_icons_page result={} basic_page_lint_in_a_row={row}",
        pass(row)
    );
    assert!(row);
}
