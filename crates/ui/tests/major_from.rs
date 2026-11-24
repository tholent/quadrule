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

//! "Major rows from" and "Major columns from" in the inspector (ADR-023 as amended 2026-11-23;
//! docs/bbq/2026-11-23-major-lines-from-top.md, Q5 and Q6): a pattern region's Style tab has the two rows under Major
//! lines, each with Top, Centre and Rhythm origin; picking one changes that axis only, in one undo step; Lines has no
//! major columns, so its columns row is greyed; on an isometric region Top is greyed in both, and one a file sets to
//! Top shows Rhythm origin, which is what it draws. Headless. Each test prints one structured `CHECK` line starting
//! with a newline (run with `-- --nocapture`).

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

const ROWS: &str = "Major rows from";
const COLS: &str = "Major columns from";

/// The region's (rows, columns) settings as stored.
fn major_from(h: &Harness<'static, Window>, id: Uuid) -> (MajorFrom, MajorFrom) {
    match &h.state().ui.viewer.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Pattern(p) => (p.major_rows_from, p.major_columns_from),
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

fn open_choices(h: &mut Harness<'static, Window>, row: &str) {
    h.get_by_role_and_label(Role::ComboBox, row).click();
    h.run();
    h.run();
}

/// Whether the open list's `label` is greyed.
fn greyed(h: &Harness<'static, Window>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().is_disabled()
}

/// The choice a row shows.
fn shows(h: &Harness<'static, Window>, row: &str) -> Option<String> {
    h.get_by_role_and_label(Role::ComboBox, row)
        .accesskit_node()
        .value()
        .map(|v| v.to_string())
}

/// Picks `choice` in `row`, reopening the list if a click on a greyed choice closed it.
fn pick(h: &mut Harness<'static, Window>, row: &str, choice: &str) {
    if h.query_by_label("Centre").is_none() {
        open_choices(h, row);
    }
    h.get_by_label(choice).click();
    h.run();
    h.run();
}

#[test]
fn picking_centre_is_one_undo_step() {
    use MajorFrom::{Centre, Top};
    // Ruled lines from their top: the columns row is greyed (Lines has rows only).
    let lines = majors_ids::FIRST;
    let h = page(lines);
    let both_rows = h.query_by_role_and_label(Role::ComboBox, ROWS).is_some()
        && h.query_by_role_and_label(Role::ComboBox, COLS).is_some();
    let lines_cols_greyed = h
        .get_by_role_and_label(Role::ComboBox, COLS)
        .accesskit_node()
        .is_disabled()
        && !h
            .get_by_role_and_label(Role::ComboBox, ROWS)
            .accesskit_node()
            .is_disabled();
    // A grid from its top (row 2 of the page, the Top column): rows to Centre, then columns to Centre, each its own
    // undo step, the other axis left as it was.
    let id = majors_ids::nth(4);
    let mut h = page(id);
    let was_top = major_from(&h, id) == (Top, Top);
    let n0 = steps(&h);
    open_choices(&mut h, ROWS);
    let top_open = !greyed(&h, "Top") && !greyed(&h, "Rhythm origin");
    pick(&mut h, ROWS, "Centre");
    let rows_only = major_from(&h, id) == (Centre, Top) && steps(&h) == n0 + 1;
    open_choices(&mut h, COLS);
    pick(&mut h, COLS, "Centre");
    let cols_only = major_from(&h, id) == (Centre, Centre) && steps(&h) == n0 + 2;
    h.state_mut().ui.viewer.undo();
    h.run();
    let undo_cols = major_from(&h, id) == (Centre, Top);
    h.state_mut().ui.viewer.undo();
    h.run();
    let undo_rows = major_from(&h, id) == (Top, Top);
    let ok = both_rows
        && lines_cols_greyed
        && was_top
        && top_open
        && rows_only
        && cols_only
        && undo_cols
        && undo_rows;
    println!(
        "\nCHECK crate=ui name=major_from_row result={} two_rows_shown={both_rows} lines_columns_greyed={lines_cols_greyed} starts_top={was_top} top_and_origin_offered={top_open} rows_centre_one_step={rows_only} columns_centre_one_step={cols_only} undo_columns={undo_cols} undo_rows={undo_rows}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn isometric_greys_top_and_shows_what_it_draws() {
    use MajorFrom::{Centre, RhythmOrigin, Top};
    // Row 5 of the page, the Top column: an isometric region set to Top, which draws from the rhythm origin.
    let id = majors_ids::nth(13);
    let mut h = page(id);
    let stored_top = major_from(&h, id) == (Top, Top);
    let n0 = steps(&h);
    let origin = Some("Rhythm origin".to_string());
    let shows_origin = shows(&h, ROWS) == origin && shows(&h, COLS) == origin;
    let mut top_greyed = true;
    for row in [ROWS, COLS] {
        open_choices(&mut h, row);
        top_greyed &= greyed(&h, "Top") && !greyed(&h, "Centre");
        h.get_by_label("Top").click();
        h.run();
        h.run();
    }
    let unchanged = major_from(&h, id) == (Top, Top) && steps(&h) == n0;
    pick(&mut h, COLS, "Centre");
    let centre = major_from(&h, id) == (Top, Centre)
        && steps(&h) == n0 + 1
        && shows(&h, ROWS) == origin
        && h.state()
            .ui
            .viewer
            .doc()
            .is_some_and(|d| match &d.object(id).unwrap().kind {
                ObjectKind::Pattern(p) => p.major_rows_origin() == RhythmOrigin,
                _ => false,
            });
    let ok = stored_top && shows_origin && top_greyed && unchanged && centre;
    println!(
        "\nCHECK crate=ui name=major_from_isometric_row result={} file_top={stored_top} both_show_rhythm_origin={shows_origin} top_greyed_in_both={top_greyed} greyed_top_does_nothing={unchanged} columns_centre_rows_stay={centre}",
        pass(ok)
    );
    assert!(ok);
}
