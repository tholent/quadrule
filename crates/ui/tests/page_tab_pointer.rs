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

//! The Page tab's list of papers (rung 14b, paying the rung 13 debt line "the PPI choice wraps onto a second
//! line"): a ticked secondary paper shows its tick, Make primary and its PPI choice on one line, at the
//! inspector's default width. Prints one structured `CHECK` line (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_ui::{Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

#[test]
fn a_secondary_paper_keeps_its_ppi_choice_on_the_line_with_make_primary() {
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_paper();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    // The paper page targets ISO A4 (primary) and ISO A5 at 600 PPI; A5 is the secondary one.
    let tick = h.get_by_label("Target ISO A5").rect();
    let make = h.get_by_label("Make ISO A5 the primary device").rect();
    let ppi = h
        .get_by_role_and_label(egui::accesskit::Role::ComboBox, "Export PPI for ISO A5")
        .rect();
    let same_line = (tick.center().y - make.center().y).abs() < 4.0
        && (tick.center().y - ppi.center().y).abs() < 4.0;
    let in_order = tick.max.x <= make.min.x + 1.0 && make.max.x <= ppi.min.x + 1.0;
    let ok = same_line && in_order;
    println!(
        "CHECK crate=ui name=page_tab_ppi_one_line result={} same_line={same_line} in_order={in_order} tick_y={:.0} make_y={:.0} ppi_y={:.0}",
        pass(ok),
        tick.center().y,
        make.center().y,
        ppi.center().y
    );
    assert!(ok);
}
