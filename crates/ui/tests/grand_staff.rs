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

//! The grand staff's space in the inspector (docs/bbq/2026-11-18-fixes-before-16b.md, Q2): a grand staff with a space of its
//! own shows "Space between grand staves" in place of "Staff every"; a single staff and a grand staff from an older file show
//! "Staff every"; turning Grand staff on writes the space, starting where the staves look closest to what "Staff every"
//! gave, in one undo step. Headless. Each test prints one structured `CHECK` line starting with a newline (run with
//! `-- --nocapture`).

use egui::Role;
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

fn page(pick: Uuid) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_grand_staff();
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

fn staff(h: &Harness<'static, Window>, id: Uuid) -> (bool, Option<u32>, u32) {
    match &h.state().ui.viewer.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Pattern(PatternRegion {
            pattern:
                Pattern::MusicStaff {
                    grand,
                    grand_space_units,
                    pitch_units,
                    ..
                },
            ..
        }) => (*grand, *grand_space_units, *pitch_units),
        _ => panic!("not a staff"),
    }
}

fn count(h: &Harness<'static, Window>, label: &str) -> usize {
    h.query_all_by_label(label).count()
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

#[test]
fn each_mode_shows_its_own_field() {
    // A grand staff with its own space, a grand staff from an older file, a single staff.
    let mut parts = Vec::new();
    let mut ok = true;
    for (name, id, space, every) in [
        ("grand_with_space", grand_ids::FIRST, 1, 0),
        ("grand_old_file", grand_ids::nth(5), 0, 1),
        ("single", grand_ids::nth(6), 0, 1),
    ] {
        let h = page(id);
        let (s, e) = (
            count(&h, "Space between grand staves"),
            count(&h, "Staff every"),
        );
        let right = (s >= 1) == (space == 1) && (e >= 1) == (every == 1) && s + e >= 1;
        ok &= right;
        parts.push(format!("{name}:space_field={s}:staff_every={e}"));
    }
    println!(
        "\nCHECK crate=ui name=grand_staff_fields result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn turning_grand_staff_on_writes_the_closest_space_in_one_step_and_off_keeps_it() {
    let single = grand_ids::nth(6);
    let mut h = page(single);
    let before = staff(&h, single);
    let n0 = steps(&h);
    h.get_by_role_and_label(Role::CheckBox, "Grand staff")
        .click();
    h.run();
    h.run();
    let on = staff(&h, single);
    // A 2 mm line gap and "Staff every 3" on a 5 mm rhythm overlapped as a grand staff: the nearest space that does not is 1.
    let wrote = on == (true, Some(1), 3) && before == (false, None, 3);
    let one_step = steps(&h) == n0 + 1;
    let shows = count(&h, "Space between grand staves") >= 1 && count(&h, "Staff every") == 0;
    h.get_by_role_and_label(Role::CheckBox, "Grand staff")
        .click();
    h.run();
    h.run();
    let off = staff(&h, single);
    let kept = off == (false, Some(1), 3) && count(&h, "Staff every") >= 1;
    h.state_mut().ui.viewer.undo();
    h.state_mut().ui.viewer.undo();
    h.run();
    let back = staff(&h, single) == before;
    // The value that looks closest to "Staff every": nothing to preserve where it overlapped, the same space where it did not.
    let near = (
        grand_space_closest(2.0, 3, 5.0),
        grand_space_closest(2.0, 8, 5.0),
        grand_space_closest(1.0, 12, 5.0),
        grand_space_closest(2.0, 8, 0.0),
    );
    let closest = near == (1, 4, 9, 1);
    let ok = wrote && one_step && shows && kept && back && closest;
    println!(
        "\nCHECK crate=ui name=grand_staff_on result={} writes_closest_space={wrote} one_undo_step={one_step} shows_space_field={shows} off_keeps_it={kept} undo_restores={back} closest_3_8_12={near:?}",
        pass(ok)
    );
    assert!(ok);
}
