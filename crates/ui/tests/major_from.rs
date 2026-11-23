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

//! "Major lines from" in the inspector (ADR-023; docs/bbq/2026-11-23-major-lines-from-top.md, Q5 and Q6): a pattern
//! region's Style tab has the row under Major lines, with Top, Centre and Rhythm origin; picking one is one undo step;
//! on an isometric region Top is greyed, and one a file sets to Top shows Rhythm origin, which is what it draws.
//! Headless. Each test prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

/// The major-lines-from page with `pick` picked and the Style tab showing.
fn page(pick: Uuid) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_majors();
    v.selection = vec![pick];
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window {
                ui: ViewerUi::new(v),
            },
        );
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    h
}

fn major_from(h: &Harness<'static, Window>, id: Uuid) -> MajorFrom {
    match &h.state().ui.viewer.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Pattern(p) => p.major_from,
        _ => panic!("not a pattern"),
    }
}

fn steps(h: &Harness<'static, Window>) -> usize {
    h.state()
        .ui
        .viewer
        .open
        .as_ref()
        .unwrap()
        .history
        .undo_len()
}

fn open_choices(h: &mut Harness<'static, Window>) {
    h.get_by_role_and_label(Role::ComboBox, "Major lines from")
        .click();
    h.run();
    h.run();
}

/// Whether the open list's `label` is greyed.
fn greyed(h: &Harness<'static, Window>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().is_disabled()
}

#[test]
fn picking_centre_is_one_undo_step() {
    let id = majors_ids::FIRST;
    let mut h = page(id);
    let row = h
        .query_by_role_and_label(Role::ComboBox, "Major lines from")
        .is_some();
    let was_top = major_from(&h, id) == MajorFrom::Top;
    let n0 = steps(&h);
    open_choices(&mut h);
    let top_open = !greyed(&h, "Top") && !greyed(&h, "Rhythm origin");
    h.get_by_label("Centre").click();
    h.run();
    h.run();
    let centre = major_from(&h, id) == MajorFrom::Centre;
    let one_step = steps(&h) == n0 + 1;
    h.state_mut().ui.viewer.undo();
    h.run();
    let undone = major_from(&h, id) == MajorFrom::Top;
    let ok = row && was_top && top_open && centre && one_step && undone;
    println!(
        "\nCHECK crate=ui name=major_from_row result={} row_shown={row} starts_top={was_top} top_and_origin_offered={top_open} picks_centre={centre} one_step={one_step} undo_back_to_top={undone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn isometric_greys_top_and_shows_what_it_draws() {
    // Row 5 of the page, the Top column: an isometric region set to Top, which draws from the rhythm origin.
    let id = majors_ids::nth(13);
    let mut h = page(id);
    let stored_top = major_from(&h, id) == MajorFrom::Top;
    let n0 = steps(&h);
    let shows_origin = h
        .get_by_role_and_label(Role::ComboBox, "Major lines from")
        .accesskit_node()
        .value()
        .as_deref()
        == Some("Rhythm origin");
    open_choices(&mut h);
    let top_greyed = greyed(&h, "Top") && !greyed(&h, "Centre");
    h.get_by_label("Top").click();
    h.run();
    h.run();
    let unchanged = major_from(&h, id) == MajorFrom::Top && steps(&h) == n0;
    // The click on the greyed choice closes the list.
    if h.query_by_label("Centre").is_none() {
        open_choices(&mut h);
    }
    h.get_by_label("Centre").click();
    h.run();
    h.run();
    let centre = major_from(&h, id) == MajorFrom::Centre && steps(&h) == n0 + 1;
    let ok = stored_top && shows_origin && top_greyed && unchanged && centre;
    println!(
        "\nCHECK crate=ui name=major_from_isometric_row result={} file_top={stored_top} shows_rhythm_origin={shows_origin} top_greyed={top_greyed} greyed_top_does_nothing={unchanged} picks_centre={centre}",
        pass(ok)
    );
    assert!(ok);
}
