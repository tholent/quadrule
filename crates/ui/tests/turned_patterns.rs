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

//! Rung 12b: pattern regions turn whole (docs/bbq/2026-11-16-rung-12-missing-fonts.md, Q8) in the
//! editor: the rotate handle, the Arrange tab's Rotation field, picking by the turned box, and the
//! benchmarks over the turned-patterns page. Headless. Each test prints one structured `CHECK` line
//! starting with a newline (run with `-- --nocapture`). Coordinates are Manta device pixels.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::*;
use quadrule_ui::{DragMods, Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: false,
};

fn open() -> Viewer {
    let mut v = Viewer::default();
    v.open_turned_patterns();
    v.rotate_stalk_px = 60.0;
    v
}

fn px_per_mm() -> f64 {
    quadrule_profiles::builtin(MANTA_ID).unwrap().mm_to_px(1.0)
}

struct Window {
    ui: ViewerUi,
}

fn harness_with(v: Viewer) -> Harness<'static, Window> {
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
    h
}

/// The middle of the grid turned by 45 degrees, in pixels.
fn centre(v: &mut Viewer) -> (f64, f64) {
    let b = v
        .page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == turned_ids::TURNED)
        .unwrap()
        .shape
        .bounds();
    ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0)
}

#[test]
fn the_rotate_handle_turns_a_pattern_region_and_catches_at_a_quarter_turn() {
    let mut v = open();
    v.selection = vec![turned_ids::TURNED];
    let handle = v.rotate_handle();
    let has = handle.is_some();
    let h = handle.expect("a pattern region has a rotate handle");
    let c = centre(&mut v);
    let before_steps = v.open.as_ref().unwrap().history.undo_len();
    assert!(v.drag_begin(h.tip.0, h.tip.1, NONE, 4.0, 6.0));
    // The pointer goes to the right of the middle: close to 90 degrees, which the handle catches.
    v.drag_update(c.0 + 500.0, c.1 + 6.0, NONE, REACH);
    v.drag_end().unwrap();
    let turned = v
        .doc()
        .unwrap()
        .object(turned_ids::TURNED)
        .unwrap()
        .rotation();
    let caught = (turned - 90.0).abs() < 1e-6;
    let one_step = v.open.as_ref().unwrap().history.undo_len() == before_steps + 1;
    v.undo();
    let back = (v
        .doc()
        .unwrap()
        .object(turned_ids::TURNED)
        .unwrap()
        .rotation()
        - 45.0)
        .abs()
        < 1e-6;
    let ok = has && caught && one_step && back;
    println!(
        "\nCHECK crate=ui name=turn_pattern_handle result={} has_handle={has} caught_at_90={caught} one_undo_step={one_step} undo_restores_45={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_rotation_field_is_there_for_a_pattern_region_and_a_turned_one_is_picked_by_its_turned_box() {
    let mut v = open();
    v.selection = vec![turned_ids::TURNED];
    let mut h = harness_with(v);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let field = h.query_by_label("Rotation").is_some();
    let mut v = open();
    let k = px_per_mm();
    let c = centre(&mut v);
    // The grid is 30 by 20 mm turned by 45: a point 14 mm along its long axis is on it, though
    // outside the unturned frame's height (10.25 mm up); a corner of the unturned frame (14, 9 mm
    // from the middle) is off the turned one (16.3 mm along its long axis).
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let on = (c.0 + 14.5 * k * s, c.1 + 14.5 * k * s);
    let off = (c.0 + 14.0 * k, c.1 + 9.0 * k);
    v.click(on.0, on.1, 1.0, false);
    let picked_on = v.selection == vec![turned_ids::TURNED];
    v.selection.clear();
    v.click(off.0, off.1, 1.0, false);
    let picked_off = v.selection == vec![turned_ids::TURNED];
    let ok = field && picked_on && !picked_off;
    println!(
        "\nCHECK crate=ui name=turn_pattern_field_and_pick result={} rotation_field={field} turned_corner_picks={picked_on} unturned_corner_does_not={}",
        pass(ok),
        !picked_off
    );
    assert!(ok);
}

#[test]
fn the_benchmarks_run_over_the_turned_patterns_page() {
    // `quadrule --turned-patterns-page --bench-drag`, `--bench-draw` and `--bench-rotate`: headless they
    // finish, time their frames and leave the page as it was.
    let want = turned_patterns_document();
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 3] = [
        (
            "drag",
            quadrule_ui::DragBench::new(12).with_turned_patterns(),
        ),
        (
            "draw",
            quadrule_ui::DragBench::draw(12).with_turned_patterns(),
        ),
        (
            "rotate",
            quadrule_ui::DragBench::rotate(12).with_turned_patterns(),
        ),
    ];
    for (name, bench) in runs {
        let mut h = harness_with(Viewer::default());
        h.state_mut().ui.bench = Some(bench);
        let mut steps = 0;
        while !h.state().ui.bench_finished() && steps < 80 {
            h.step();
            steps += 1;
        }
        let ui = &h.state().ui;
        let done = ui.bench_finished();
        let timed = ui.drag_clock.frames > 0 && ui.drag_clock.mean_ms() > 0.0;
        let clean = ui.viewer.doc().is_some_and(|d| *d == want)
            && !ui.viewer.is_dirty()
            && !ui.viewer.dragging();
        ok &= done && timed && clean;
        parts.push(format!(
            "{name}:finished={done}:frames={}:clean={clean}",
            ui.drag_clock.frames
        ));
    }
    println!(
        "\nCHECK crate=ui name=window_turned_patterns_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_pattern_tab_has_a_pattern_angle_field_and_its_edits_merge_into_one_undo_step() {
    // Rung 12b, Q11: a "Pattern angle" field beside the spacing in the Style tab (the pattern's tab); no handle.
    let mut v = Viewer::default();
    v.open_pattern_angle();
    v.rotate_stalk_px = 60.0;
    v.selection = vec![angle_ids::GRID];
    let mut h = harness_with(v);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let field = h.query_all_by_label("Pattern angle").count() >= 1;
    let v = &mut h.state_mut().ui.viewer;
    let before = v.open.as_ref().unwrap().history.undo_len();
    for deg in [31.0, 32.0, 33.0] {
        let _ = v.edit_selected(Some("pattern-angle"), |o| {
            if let quadrule_model::ObjectKind::Pattern(p) = &mut o.kind {
                p.angle_deg = quadrule_model::normalize_deg(deg);
            }
        });
    }
    let set = match &v.doc().unwrap().object(angle_ids::GRID).unwrap().kind {
        quadrule_model::ObjectKind::Pattern(p) => p.pattern_angle(),
        _ => -1.0,
    };
    let one_step = v.open.as_ref().unwrap().history.undo_len() == before + 1;
    v.undo();
    let back = match &v.doc().unwrap().object(angle_ids::GRID).unwrap().kind {
        quadrule_model::ObjectKind::Pattern(p) => p.pattern_angle(),
        _ => -1.0,
    };
    // The frame's own rotation is a separate field and stays: 0 here.
    let frame_turn = v.doc().unwrap().object(angle_ids::GRID).unwrap().rotation();
    let handle = {
        v.selection = vec![angle_ids::GRID];
        v.rotate_handle().is_some()
    };
    let ok = field && set == 33.0 && one_step && back == 30.0 && frame_turn == 0.0 && handle;
    println!(
        "\nCHECK crate=ui name=pattern_angle_field result={} field_named={field} set_33={} edits_merge_to_one_step={one_step} undo_back_to_30={} frame_turn_untouched={}",
        pass(ok),
        set == 33.0,
        back == 30.0,
        frame_turn == 0.0
    );
    assert!(ok);
}

#[test]
fn the_benchmarks_run_over_the_pattern_angle_page() {
    // `quadrule --pattern-angle-page --bench-drag` and `--bench-draw`: headless they finish and leave the page as it was.
    let want = pattern_angle_document();
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 2] = [
        ("drag", quadrule_ui::DragBench::new(12).with_pattern_angle()),
        (
            "draw",
            quadrule_ui::DragBench::draw(12).with_pattern_angle(),
        ),
    ];
    for (name, bench) in runs {
        let mut h = harness_with(Viewer::default());
        h.state_mut().ui.bench = Some(bench);
        let mut steps = 0;
        while !h.state().ui.bench_finished() && steps < 80 {
            h.step();
            steps += 1;
        }
        let ui = &h.state().ui;
        let done = ui.bench_finished();
        let clean = ui.viewer.doc().is_some_and(|d| *d == want)
            && !ui.viewer.is_dirty()
            && !ui.viewer.dragging();
        ok &= done && clean;
        parts.push(format!("{name}:finished={done}:clean={clean}"));
    }
    println!(
        "\nCHECK crate=ui name=window_pattern_angle_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
