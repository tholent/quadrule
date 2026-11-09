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

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_ui::{ExportWhich, Request, Viewer, ViewerUi, Zoom};

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
    // Clicking a finding selects it, for the outline on the page (Q6).
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
    h.get_by_label("Save").click();
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
