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
    // In the toolbar and the status bar. The Manta has seven findings with every toolbar position
    // checked (chris, 2026-11-10); with only left and top it had five.
    let count = h.query_all_by_label("⚠ 7").count() == 2;
    let panel = h.query_by_label("Lint (7)").is_some();
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
    h.get_by_label("Lint (7)").click();
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

/// A stand-in chip renderer that takes `ms` a frame.
struct FakeChip {
    ms: f64,
    frames: std::rc::Rc<std::cell::Cell<u32>>,
}

impl quadrule_ui::ChipRenderer for FakeChip {
    fn render(
        &mut self,
        _dl: &quadrule_layout::DisplayList,
    ) -> Result<(egui::TextureId, f64), String> {
        self.frames.set(self.frames.get() + 1);
        Ok((egui::TextureId::User(1), self.ms))
    }
}

#[test]
fn chip_canvas_switch_and_fallback() {
    // Rung 3 bbq Q7: View → Canvas switches to the graphics chip; a slow chip (over 100 ms a
    // frame after the first) falls back to vello_cpu and the status bar says so.
    let frames = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut h = harness();
    h.state_mut().ui.chip = Ok(Box::new(FakeChip {
        ms: 150.0,
        frames: frames.clone(),
    }));
    h.get_by_label("Open sample page").click();
    h.run();
    let export_default = h.state().ui.canvas_mode == quadrule_ui::CanvasMode::ExportPixels;
    h.get_by_label("View").click();
    h.run();
    h.get_by_label("Graphics chip").click();
    h.run();
    // The warm-up frame is followed at once by a judged one, which is too slow.
    let first = frames.get() == 2;
    let fell_back = h.state().ui.chip_state.fallback == Some(quadrule_ui::Fallback::Slow(150.0))
        && h.query_by_label_contains("too slow").is_some();
    let after = frames.get();
    h.get_by_label("+").click();
    h.run();
    let stays_on_cpu = frames.get() == after && h.state().ui.chip_state.cpu_ms.is_some();

    // No renderer at all: says it could not start.
    let mut h2 = harness();
    h2.state_mut().ui.chip = Err("no adapter".into());
    h2.state_mut().ui.canvas_mode = quadrule_ui::CanvasMode::Chip;
    h2.get_by_label("Open sample page").click();
    h2.run();
    let cannot = h2
        .query_by_label_contains("could not start (no adapter)")
        .is_some();
    let ok = export_default && first && fell_back && stays_on_cpu && cannot;
    println!(
        "CHECK crate=ui name=window_chip_canvas result={} export_pixels_default={export_default} warm_up_then_judged={first} slow_falls_back={fell_back} stays_on_cpu={stays_on_cpu} cannot_start_said={cannot}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn layers_sit_under_the_inspector_on_the_right() {
    // chris, 2026-11-10: the layers pane goes at the bottom of the right-hand panel; the left
    // side is kept for a future tools bar.
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    let layers = h.get_by_label("Layers").rect();
    let page_tab = h.get_by_label("Page").rect();
    let canvas_mid = 1280.0 / 2.0;
    let right = layers.min.x > canvas_mid && page_tab.min.x > canvas_mid;
    let below = layers.min.y > page_tab.max.y;
    // Nothing on the left of the page: the leftmost control below the toolbar is past the canvas.
    let left_free = h.get_by_label("Title rule").rect().min.x > canvas_mid;
    let ok = right && below && left_free;
    println!(
        "CHECK crate=ui name=layers_under_inspector result={} right_panel={right} below_tabs={below} left_side_free={left_free} layers_y={} tabs_y={}",
        pass(ok),
        layers.min.y,
        page_tab.max.y
    );
    assert!(ok);
}

#[test]
fn dont_save_then_close_leaves_no_recovery_file() {
    // Found 2026-11-10 on the Pi: a recovery file was left after the window closed. "Don't save"
    // deletes it, but the window loses focus as it closes, and autosave on focus loss wrote it
    // again.
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("window_close_recovery");
    let _ = std::fs::remove_dir_all(&dir);
    let rec = Recovery::new(&dir);
    let mut h = harness();
    h.state_mut().ui.recovery = Some(rec.clone());
    h.get_by_label("Open sample page").click();
    h.run();
    h.state_mut().ui.viewer.selection = vec![ids::RULE];
    h.state_mut().ui.viewer.nudge(1.0, 0.0, false).unwrap();
    h.run();
    let asks = !h.state_mut().ui.may_close();
    h.run();
    h.get_by_label("Don’t save").click();
    h.run();
    let close = h.state().requests.contains(&Request::Close);
    // The window loses focus while it closes.
    let id = egui::ViewportId::ROOT;
    h.input_mut().viewports.entry(id).or_default().focused = Some(false);
    h.input_mut().focused = false;
    h.run();
    let left = rec.list().len();
    let ok = asks && close && left == 0;
    println!(
        "CHECK crate=ui name=dont_save_close_no_recovery result={} prompt={asks} close_requested={close} recovery_files_left={left}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn snap_toggles_and_subdivisions_are_named_controls() {
    // Rung 4a: each helper has a switch in the status bar and in the Page tab, named for screen
    // readers by its label; the Page tab has a Subdivisions field.
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    let names = [
        "Pixel grid",
        "Rhythm",
        "Margins and safe zones",
        "Point snap",
        "Smart guides",
        "Equal spacing",
        "Rhythm overlay",
    ];
    let twice = names.iter().all(|n| h.get_all_by_label(n).count() >= 2);
    let on_at_start = {
        let s = &h.state().ui.viewer;
        s.snap.pixel_grid
            && s.snap.rhythm
            && s.snap.margins_safe
            && s.snap.points
            && s.snap.smart_guides
            && s.snap.equal_spacing
            && !s.show_rhythm
    };
    // The harness drops a first click that no hover came before; hover, then click.
    h.get_all_by_label("Pixel grid").next().unwrap().hover();
    h.run();
    h.get_all_by_label("Pixel grid").next().unwrap().click();
    h.run();
    let grid_off = !h.state().ui.viewer.snap.pixel_grid;
    h.get_all_by_label("Rhythm overlay").next().unwrap().click();
    h.run();
    let overlay_on = h.state().ui.viewer.show_rhythm;
    h.get_all_by_label("Smart guides").next().unwrap().click();
    h.run();
    let smart_off = !h.state().ui.viewer.snap.smart_guides;
    let sub_field = h.query_by_label("Subdivisions").is_some();
    let ok = twice && on_at_start && overlay_on && grid_off && smart_off && sub_field;
    println!(
        "\nCHECK crate=ui name=window_snap_toggles result={} in_status_and_page_tab={twice} on_at_start={on_at_start} overlay_toggles={overlay_on} pixel_grid_toggles={grid_off} smart_guides_toggles={smart_off} subdivisions_field={sub_field}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn escape_drops_a_drag_and_other_keys_wait() {
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    let doc0 = h.state().ui.viewer.doc().unwrap().clone();
    // A drag begun and moved through the viewer's own calls, as the canvas makes them.
    {
        let v = &mut h.state_mut().ui.viewer;
        let px = |mm: f64| mm * 300.0 / 25.4;
        assert!(v.drag_begin(px(50.0), px(22.0), Default::default(), 4.0, 6.0));
        v.drag_update(px(60.0), px(40.0), Default::default(), 6.0);
    }
    h.run();
    let dragging = h.state().ui.viewer.dragging();
    // Ctrl+Z means nothing mid-drag.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::Z]);
    h.run();
    let still = h.state().ui.viewer.dragging();
    h.key_press(Key::Escape);
    h.run();
    let v = &h.state().ui.viewer;
    let dropped = !v.dragging() && v.doc().unwrap() == &doc0 && !v.is_dirty();
    let ok = dragging && still && dropped;
    println!(
        "\nCHECK crate=ui name=window_escape_drag result={} was_dragging={dragging} undo_key_waits={still} escape_drops_it={dropped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_drag_benchmark_runs_and_leaves_the_document_alone() {
    // `quadrule --bench-drag` replays a drag through the real frame loop; here, headless, to
    // show it finishes, times its frames and undoes itself.
    let mut h = harness();
    h.state_mut().ui.bench = Some(quadrule_ui::DragBench::new(12));
    let mut steps = 0;
    while !h.state().ui.bench_finished() && steps < 80 {
        h.step();
        steps += 1;
    }
    let ui = &h.state().ui;
    let done = ui.bench_finished();
    let timed = ui.drag_clock.frames > 0 && ui.drag_clock.mean_ms() > 0.0;
    let clean = ui
        .viewer
        .doc()
        .is_some_and(|d| *d == quadrule_model::sample::slice_document())
        && !ui.viewer.is_dirty()
        && !ui.viewer.dragging();
    let ok = done && timed && clean;
    println!(
        "\nCHECK crate=ui name=window_drag_bench result={} finished={done} frames_timed={} clean_after={clean}",
        pass(ok),
        ui.drag_clock.frames
    );
    assert!(ok);
}

#[test]
fn the_canvas_turns_pointer_drags_into_moves_marquees_and_escape() {
    // The real wiring: pointer events on the canvas, through egui, into the viewer's drag.
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    h.run();
    let px = |mm: f64| mm * 300.0 / 25.4;
    let at = |h: &Harness<'static, Window>, x_mm: f64, y_mm: f64| {
        h.state().ui.screen_point(px(x_mm), px(y_mm)).unwrap()
    };
    let doc0 = h.state().ui.viewer.doc().unwrap().clone();

    // Press on the title rule, move 40 points down, let go: the rule moves, as one step.
    let p0 = at(&h, 50.0, 22.0);
    h.hover_at(p0);
    h.step();
    h.drag_at(p0);
    h.step();
    h.hover_at(p0 + egui::vec2(0.0, 12.0));
    h.step();
    h.hover_at(p0 + egui::vec2(0.0, 40.0));
    h.step();
    h.step();
    let dragging = h.state().ui.viewer.dragging();
    let untouched = h.state().ui.viewer.doc().unwrap() == &doc0;
    h.drop_at(p0 + egui::vec2(0.0, 40.0));
    h.step();
    h.step();
    let v = &h.state().ui.viewer;
    let one_step = !v.dragging() && v.open.as_ref().unwrap().history.undo_len() == 1;
    let moved = v.doc().unwrap().object(ids::RULE).unwrap().frame.y_mm > 22.0 + 1.0;
    let selected = v.selection == vec![ids::RULE];

    // Press on empty page and sweep over the title: a marquee selects it.
    h.state_mut().ui.viewer.selection.clear();
    let a = at(&h, 2.0, 2.0);
    let b = at(&h, 100.0, 20.0);
    h.hover_at(a);
    h.step();
    h.drag_at(a);
    h.step();
    h.hover_at(a + egui::vec2(10.0, 10.0));
    h.step();
    h.hover_at(b);
    h.step();
    h.step();
    let box_drawn = h.state().ui.viewer.drag_marquee().is_some();
    h.drop_at(b);
    h.step();
    h.step();
    let marquee = h.state().ui.viewer.selection == vec![ids::TITLE];

    // Escape mid-drag drops it.
    let steps_before = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .unwrap()
        .history
        .undo_len();
    let doc1 = h.state().ui.viewer.doc().unwrap().clone();
    let t = at(&h, 50.0, 12.0);
    h.hover_at(t);
    h.step();
    h.drag_at(t);
    h.step();
    h.hover_at(t + egui::vec2(10.0, 10.0));
    h.step();
    h.hover_at(t + egui::vec2(30.0, 30.0));
    h.step();
    h.step();
    let moving = h.state().ui.viewer.dragging();
    h.key_press(Key::Escape);
    h.step();
    h.step();
    h.drop_at(t + egui::vec2(30.0, 30.0));
    h.step();
    h.step();
    let v = &h.state().ui.viewer;
    let escaped = moving
        && !v.dragging()
        && v.doc().unwrap() == &doc1
        && v.open.as_ref().unwrap().history.undo_len() == steps_before;
    let ok =
        dragging && untouched && one_step && moved && selected && box_drawn && marquee && escaped;
    println!(
        "\nCHECK crate=ui name=window_pointer_drag result={} dragging={dragging} doc_untouched_midway={untouched} one_step={one_step} rule_moved={moved} selected={selected} marquee_box={box_drawn} marquee_selects_title={marquee} escape_drops={escaped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn copy_cut_and_paste_keys() {
    // Rung 4b: Ctrl+C, Ctrl+X and Ctrl+V reach the viewer as egui's copy, cut and paste events;
    // a copy puts quadrule's text on the clipboard, a paste of that text makes new objects.
    let mut h = harness();
    h.get_by_label("Open sample page").click();
    h.run();
    h.state_mut().ui.viewer.selection = vec![ids::RULE];
    let objects = |h: &Harness<'_, Window>| {
        h.state()
            .ui
            .viewer
            .doc()
            .unwrap()
            .layers
            .iter()
            .map(|l| l.objects.len())
            .sum::<usize>()
    };
    let before = objects(&h);
    h.event(egui::Event::Copy);
    h.step();
    let on_clipboard = h
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|c| match c {
            egui::OutputCommand::CopyText(t) => Some(t.clone()),
            _ => None,
        });
    let copied = on_clipboard
        .as_ref()
        .is_some_and(|t| t.contains("quadrule-clipboard"));
    let text = on_clipboard.unwrap_or_default();
    h.event(egui::Event::Paste(text.clone()));
    h.run();
    let pasted = objects(&h) == before + 1;
    h.event(egui::Event::Paste("not ours".into()));
    h.run();
    let ignored = objects(&h) == before + 1;
    h.state_mut().ui.viewer.selection = vec![ids::RULE];
    h.event(egui::Event::Cut);
    h.run();
    let cut = objects(&h) == before;
    let ok = copied && pasted && ignored && cut;
    println!(
        "\nCHECK crate=ui name=window_copy_paste_keys result={} copy_on_clipboard={copied} paste_adds={pasted} other_text_ignored={ignored} cut_removes={cut}",
        pass(ok)
    );
    assert!(ok);
}
