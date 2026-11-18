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

//! The border of a pattern region in the editor (docs/bbq/2026-11-18-pattern-borders.md, Defaulted "The inspector"):
//! a Border section in the Style tab (a tick; with it on, four side ticks, the line and the corner radius), one edit
//! one undo step, and the benchmarks over the bordered page. Headless. Each test prints one structured `CHECK` line
//! starting with a newline (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn harness_with(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 1400.0))
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

fn border_of(v: &Viewer, id: Uuid) -> Option<Border> {
    match &v.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Pattern(p) => p.border.clone(),
        _ => None,
    }
}

fn labels(h: &Harness<'static, Window>) -> Vec<(&'static str, usize)> {
    [
        "Show border",
        "Sides",
        "Top",
        "Right",
        "Bottom",
        "Left",
        "Line",
        "Corner radius",
    ]
    .into_iter()
    .map(|l| (l, h.query_all_by_label(l).count()))
    .collect()
}

#[test]
fn the_style_tab_has_a_border_section_whose_sides_line_and_radius_show_with_the_tick_on() {
    // A region with no border shows only the tick; one with a border shows its sides, its line and its radius.
    let mut v = Viewer::default();
    v.open_pattern_angle();
    v.selection = vec![angle_ids::GRID];
    let mut h = harness_with(v);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let off = labels(&h);
    let only_tick = off[0].1 >= 1 && off[1..].iter().all(|(l, n)| *n == 0 || *l == "Left");
    // `Left` and `Top` may also be words elsewhere in the window (the Arrange tab's alignment); the Sides label is
    // the one that tells.
    let no_sides = off[1].1 == 0;

    let mut v = Viewer::default();
    v.open_bordered();
    v.selection = vec![border_ids::nth(3)];
    let mut h = harness_with(v);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let on = labels(&h);
    let all_named = on.iter().all(|(_, n)| *n >= 1);
    let ok = only_tick && no_sides && all_named;
    println!(
        "\nCHECK crate=ui name=border_section result={} tick_only_when_off={only_tick} sides_hidden_when_off={no_sides} all_fields_when_on={all_named}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn ticking_the_border_adds_one_and_one_undo_removes_it_and_each_field_is_one_step() {
    let mut v = Viewer::default();
    v.open_pattern_angle();
    v.selection = vec![angle_ids::GRID];
    let mut h = harness_with(v);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let before = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .unwrap()
        .history
        .undo_len();
    h.get_by_label("Show border").click();
    h.run();
    h.run();
    let v = &mut h.state_mut().ui.viewer;
    let added = border_of(v, angle_ids::GRID) == Some(Border::new());
    let one_step = v.open.as_ref().unwrap().history.undo_len() == before + 1;
    // Typing into a field merges into one step; each other change is one step.
    let steps = v.open.as_ref().unwrap().history.undo_len();
    for mm in [1.0, 2.0, 3.0] {
        let _ = v.edit_selected(Some("border-radius"), |o| {
            if let ObjectKind::Pattern(p) = &mut o.kind
                && let Some(b) = &mut p.border
            {
                b.corner_radius_mm = mm;
            }
        });
    }
    let merged = v.open.as_ref().unwrap().history.undo_len() == steps + 1
        && border_of(v, angle_ids::GRID).map(|b| b.corner_radius_mm) == Some(3.0);
    v.undo();
    let radius_undone = border_of(v, angle_ids::GRID).map(|b| b.corner_radius_mm) == Some(0.0);
    v.undo();
    let gone = border_of(v, angle_ids::GRID).is_none();
    let ok = added && one_step && merged && radius_undone && gone;
    println!(
        "\nCHECK crate=ui name=border_tick_undo result={} tick_adds_default={added} one_undo_step={one_step} radius_typing_one_step={merged} undo_steps_back={radius_undone}/{gone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_benchmarks_run_over_the_bordered_page() {
    // `quadrule --bordered-page --bench-drag` and `--bench-draw`: headless they finish and leave the page as it was.
    let want = pattern_borders_document();
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 2] = [
        ("drag", quadrule_ui::DragBench::new(12).with_bordered()),
        ("draw", quadrule_ui::DragBench::draw(12).with_bordered()),
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
        "\nCHECK crate=ui name=window_bordered_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
