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

//! Readable JSON (rung 17 bbq, Q6 and Defaulted 6): Help > Developer > Export as readable JSON… writes the page
//! with `quadrule_format::to_debug_json`; File > Open (and a file dropped on the window) reads a `.json` with
//! `from_debug_json` and opens it as an unsaved document, so Save asks for a `.quad` name and never writes over the
//! JSON. Nothing about a `.quad` or its bytes changes. Headless. Each test prints one structured `CHECK` line.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_ui::{Cmd, Request, Source, Viewer, ViewerUi};
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn window(dir: &std::path::Path) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.use_settings_file(dir.join("settings.ron"));
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
fn help_developer_exports_the_page_as_readable_json_and_changes_nothing() {
    let dir = tmp("readable-json-export");
    let mut h = window(&dir);
    let before = h.state().ui.viewer.doc().unwrap().clone();
    let dirty = h.state().ui.viewer.is_dirty();
    // The item is in the menu: Help, then Developer, then the export.
    h.get_by_label("Help").click();
    h.run();
    h.get_by_label("Developer").hover();
    h.run();
    h.run();
    let item = h.query_by_label("Export as readable JSON…").is_some();
    egui::Popup::close_all(&h.ctx);
    h.run();
    // The command asks the app for a save dialog with a `.json` name; the app answers with the path.
    let ctx = h.ctx.clone();
    let asked = h.state_mut().ui.run_cmd(&ctx, Cmd::ExportJson);
    let named = matches!(asked.as_slice(), [Request::ExportJsonDialog(n)] if n.ends_with(".json"));
    let path = dir.join("page.json");
    h.state_mut().ui.export_json_to(&path);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let readable = text.contains('\n') && text.contains("\"rhythm\"");
    let back = quadrule_format::from_debug_json(&text).ok();
    let same = back.as_ref() == Some(&before);
    let unchanged =
        h.state().ui.viewer.doc() == Some(&before) && h.state().ui.viewer.is_dirty() == dirty;
    let ok = item && named && readable && same && unchanged;
    println!(
        "\nCHECK crate=ui name=readable_json_export result={} menu_item={item} asks_for_a_json_name={named} pretty_text={readable} reads_back_equal={same} page_unchanged={unchanged}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_json_opens_as_an_unsaved_document_and_save_asks_for_a_quad_name() {
    let dir = tmp("readable-json-open");
    let mut h = window(&dir);
    let doc = h.state().ui.viewer.doc().unwrap().clone();
    let path = dir.join("page.json");
    h.state_mut().ui.export_json_to(&path);
    // Another page is open; the JSON replaces it.
    h.state_mut().ui.viewer.open_shapes();
    h.run();
    h.state_mut().ui.open_file(&path);
    h.run();
    let ui = &h.state().ui;
    let opened = ui.viewer.doc() == Some(&doc);
    let untitled = matches!(
        ui.viewer.open.as_ref().map(|o| &o.source),
        Some(Source::Untitled)
    ) && ui.viewer.file_path().is_none();
    // Save asks where, as for a new page: it cannot write to the JSON.
    let asked = h.state_mut().ui.save();
    let asks = matches!(asked.as_slice(), [Request::SaveAsDialog(None)]);
    let json_untouched =
        quadrule_format::from_debug_json(&std::fs::read_to_string(&path).unwrap()).is_ok();
    // A dropped file goes the same way (the unsaved-changes prompt first).
    h.state_mut().ui.viewer.open_shapes();
    let _ = h.state_mut().ui.open_from_system(path.clone());
    h.run();
    let dropped = h.state().ui.viewer.doc() == Some(&doc);
    // Not in the recent files: it is not a `.quad`.
    let recent = h
        .state()
        .ui
        .viewer
        .recent_files()
        .iter()
        .all(|r| r.path != path);
    let ok = opened && untitled && asks && json_untouched && dropped && recent;
    println!(
        "\nCHECK crate=ui name=readable_json_open result={} opens_equal={opened} unsaved_untitled={untitled} save_asks_for_a_name={asks} json_file_untouched={json_untouched} drop_opens_it={dropped} not_in_recent={recent}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_damaged_json_says_so_and_leaves_the_open_page_alone() {
    let dir = tmp("readable-json-damaged");
    let mut h = window(&dir);
    let before = h.state().ui.viewer.doc().unwrap().clone();
    let path = dir.join("bad.json");
    std::fs::write(&path, "{ \"not\": \"a page\" }").unwrap();
    h.state_mut().ui.open_file(&path);
    h.run();
    let kept = h.state().ui.viewer.doc() == Some(&before);
    let said = h.state().ui.viewer.notice.as_ref().is_some_and(|n| n.error);
    let ok = kept && said;
    println!(
        "\nCHECK crate=ui name=readable_json_damaged result={} page_kept={kept} says_why={said}",
        pass(ok)
    );
    assert!(ok);
}
