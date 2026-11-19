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

//! The layers panel's seven buttons as line icons (fixes batch 1, docs/bbq/2026-11-18-fixes-before-16b.md, Q5 and the
//! Defaulted list): each is a button named for screen readers, all seven sit in one row on the Pi's 1440 by 720 screen,
//! a hover shows the name and the button's note, and they grey as the text buttons did. What each button does is tested
//! where it was before (`layers.rs`, `groups.rs`, `repeat.rs`). Headless. Each test prints one structured `CHECK` line
//! starting with a newline (run with `-- --nocapture`).

use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_ui::layer_icons::LayerIcon;
use quadrule_ui::{Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

/// The sample page (layers Grid and Header) in a window the size of the Pi's screen.
fn sample() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1440.0, 720.0))
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

const NAMES: [&str; 7] = [
    "Move up",
    "Move down",
    "New layer",
    "Delete layer",
    "Group",
    "Ungroup",
    "Repeat",
];

fn disabled(h: &Harness<'static, Window>, label: &str) -> bool {
    h.get_by_role_and_label(Role::Button, label)
        .accesskit_node()
        .is_disabled()
}

#[test]
fn the_seven_icons_are_named_buttons_in_one_row_left_to_right() {
    let h = sample();
    let rects: Vec<egui::Rect> = NAMES
        .iter()
        .map(|n| h.get_by_role_and_label(Role::Button, n).rect())
        .collect();
    let seven = rects.len() == LayerIcon::ALL.len();
    let one_row = rects
        .iter()
        .all(|r| (r.center().y - rects[0].center().y).abs() < 0.5);
    let in_order = rects.windows(2).all(|w| w[0].max.x <= w[1].min.x);
    let square = rects.iter().all(|r| {
        (r.width() - quadrule_ui::layer_icons::BUTTON_SIDE).abs() < 0.5
            && (r.height() - quadrule_ui::layer_icons::BUTTON_SIDE).abs() < 0.5
    });
    let width = rects[6].max.x - rects[0].min.x;
    let ok = seven && one_row && in_order && square;
    println!(
        "\nCHECK crate=ui name=layer_icons_one_row result={} seven={seven} one_row={one_row} in_order={in_order} square={square} row_width_pt={width:.0} window=1440x720",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_hover_shows_the_name_and_the_note_and_the_icons_grey_as_the_buttons_did() {
    let mut h = sample();
    // Nothing picked: only New layer works.
    let greyed_none = [
        "Move up",
        "Move down",
        "Delete layer",
        "Group",
        "Ungroup",
        "Repeat",
    ]
    .iter()
    .all(|n| disabled(&h, n));
    let new_on = !disabled(&h, "New layer");
    // Hover New layer: its name and its note show.
    h.get_by_role_and_label(Role::Button, "New layer").hover();
    h.run();
    let name_shown = h.query_all_by_label("New layer").count() == 2;
    let note_shown = h
        .query_by_label("Adds an empty layer above the picked one, or on top.")
        .is_some();
    // Hover a greyed one: its note says why.
    h.get_by_role_and_label(Role::Button, "Delete layer")
        .hover();
    h.run();
    let why_shown = h.query_by_label("Pick a layer first.").is_some();
    // Pick a layer: Move up, Move down and Delete layer wake.
    h.get_by_label("Grid").click();
    h.run();
    h.run();
    let woke = ["Move up", "Move down", "Delete layer"]
        .iter()
        .all(|n| !disabled(&h, n));
    let ok = greyed_none && new_on && name_shown && note_shown && why_shown && woke;
    println!(
        "\nCHECK crate=ui name=layer_icons_hover_and_grey result={} greyed_with_nothing_picked={greyed_none} new_layer_on={new_on} hover_name={name_shown} hover_note={note_shown} greyed_note={why_shown} layer_pick_wakes_three={woke}",
        pass(ok)
    );
    assert!(ok);
}
