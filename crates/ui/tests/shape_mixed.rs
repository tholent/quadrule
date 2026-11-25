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

//! Rung 17, D11: with several shapes picked, the Style tab's fill, dash, cap and shape rows read "mixed" where the
//! objects differ (as opacity, stroke kind, role, width and gray already did), and only there. Headless. Each test
//! prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::ObjectKind;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn harness(v: Viewer) -> Harness<'static, Window> {
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

/// The Style tab with `pick` (a filter on kinds) of the shapes page picked: the words on screen that read "mixed",
/// as the labels of what holds them.
fn mixed_rows(pick: &dyn Fn(&ObjectKind) -> bool) -> Vec<String> {
    let mut v = Viewer::default();
    v.open_shapes();
    let ids: Vec<_> = v
        .doc()
        .unwrap()
        .objects()
        .filter(|o| pick(&o.kind))
        .map(|o| o.id)
        .collect();
    v.selection = ids;
    let mut h = harness(v);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let mut out: Vec<String> = h
        .query_all_by_value("mixed")
        .filter_map(|n| n.accesskit_node().label())
        .map(|l| l.to_string())
        .collect();
    out.sort();
    out
}

#[test]
fn several_shapes_that_differ_read_mixed_in_the_fill_dash_cap_and_shape_rows() {
    let rects = mixed_rows(&|k| matches!(k, ObjectKind::Rect(_)));
    let polys = mixed_rows(&|k| matches!(k, ObjectKind::Polygon(_)));
    let has = |list: &[String], names: &[&str]| names.iter().all(|n| list.iter().any(|l| l == n));
    // One object picked reads no "mixed" at all.
    let single = {
        let mut v = Viewer::default();
        v.open_shapes();
        let first = v
            .doc()
            .unwrap()
            .objects()
            .find(|o| matches!(o.kind, ObjectKind::Rect(_)))
            .unwrap()
            .id;
        v.selection = vec![first];
        let mut h = harness(v);
        h.state_mut().ui.tab = Tab::Style;
        h.run();
        h.run();
        h.query_all_by_value("mixed").count()
    };
    let rects_ok = has(&rects, &["Fill", "Dash", "Cap", "Corner radius"]);
    let polys_ok = has(&polys, &["Fill", "Dash", "Cap", "Sides"]);
    let ok = rects_ok && polys_ok && single == 0;
    println!(
        "\nCHECK crate=ui name=shape_rows_read_mixed result={} rects={rects:?} polygons={polys:?} single_reads_mixed={single}",
        pass(ok)
    );
    assert!(ok);
}
