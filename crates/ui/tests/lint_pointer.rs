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

//! The Lint tab, quick fixes, Ignore, Free placement and the preflight sheet driven with the pointer (rung 14b,
//! docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q5 to Q7). Each test prints one structured `CHECK` line
//! (run with `-- --nocapture`). Documents are small pages with exactly the findings under test, so a button's
//! label names one thing.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_ui::{DevicesChoice, Request, Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
    requests: Vec<Request>,
}

fn window(doc: Document) -> Harness<'static, Window> {
    let mut ui = ViewerUi::new(Viewer::default());
    scratch_folder(&mut ui);
    ui.viewer.open_document_for_test(doc);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let r = w.ui.show(ui);
                w.requests.extend(r);
            },
            Window {
                ui,
                requests: Vec::new(),
            },
        );
    h.run();
    h
}

/// An export from these tests goes into a scratch folder, never the home folder.
fn scratch_folder(ui: &mut ViewerUi) {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("lint-pointer-export");
    std::fs::create_dir_all(&dir).unwrap();
    ui.viewer.settings.last_folder = Some(dir.to_string_lossy().into_owned());
}

/// Opens the Export sheet from the File menu.
fn open_export_sheet(h: &mut Harness<'static, Window>) {
    if h.state().ui.completion_open() {
        press(h, "Done");
    }
    h.get_by_label("File").click();
    h.run();
    h.get_by_label_contains("Export…").click();
    h.run();
}

/// Presses the Export sheet's own Export button .
fn export_from_sheet(h: &mut Harness<'static, Window>) {
    for _ in 0..6 {
        h.step();
    }
    let all: Vec<_> = h
        .get_all_by_role_and_label(egui::accesskit::Role::Button, "Export")
        .collect();
    all.last().unwrap().hover();
    h.run();
    let all: Vec<_> = h
        .get_all_by_role_and_label(egui::accesskit::Role::Button, "Export")
        .collect();
    all.last().unwrap().click();
    h.run();
}

fn lint_page_window() -> Harness<'static, Window> {
    let mut ui = ViewerUi::new(Viewer::default());
    scratch_folder(&mut ui);
    ui.viewer.open_lint();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let r = w.ui.show(ui);
                w.requests.extend(r);
            },
            Window {
                ui,
                requests: Vec::new(),
            },
        );
    h.run();
    h
}

fn mm(px: f64) -> f64 {
    px * 25.4 / 300.0
}

fn obj(id: u128, name: &str, frame: Frame, style: Style, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(id),
        name: name.into(),
        frame,
        constraints: Constraints::default(),
        visible: true,
        locked: false,
        style,
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        free_placement: false,
        full_bleed: false,
        show_in_basic: false,
        kind,
        extra: Extra::new(),
    }
}

fn page(objects: Vec<Object>) -> Document {
    let mut doc = slice_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc
}

fn line(id: u128, name: &str, x: f64, y: f64, style: Style) -> Object {
    obj(
        id,
        name,
        Frame::new(x, y, 40.0, 0.0),
        style,
        ObjectKind::Line(Line::default()),
    )
}

fn text(id: u128, name: &str, size_pt: f64) -> Object {
    obj(
        id,
        name,
        Frame::new(mm(300.0), mm(600.0), 60.0, 8.0),
        Style::Role(LineRole::Text),
        ObjectKind::Text(TextBox {
            content: "words".into(),
            family: ATKINSON.into(),
            size_pt,
            ..TextBox::default()
        }),
    )
}

/// The Manta rhythm: origin 94 px, step 71 px.
fn at(k: f64) -> f64 {
    mm(94.0 + 71.0 * k)
}

/// A click on a button of the sheet: the pointer arrives first, as a hand's does, then presses.
fn press(h: &mut Harness<'static, Window>, label: &str) {
    // The sheet fades in and settles over a few frames; the pointer goes to where the button ends up.
    for _ in 0..6 {
        h.step();
    }
    h.get_by_label(label).hover();
    h.run();
    h.get_by_label(label).click();
    h.run();
}

/// [`press`] on the first button of that name.
fn press_first(h: &mut Harness<'static, Window>, label: &str) {
    for _ in 0..6 {
        h.step();
    }
    h.get_all_by_label(label).next().unwrap().hover();
    h.run();
    h.get_all_by_label(label).next().unwrap().click();
    h.run();
}

fn open_lint_tab(h: &mut Harness<'static, Window>) {
    h.get_by_label_contains("Lint (").click();
    h.run();
}

fn findings(h: &mut Harness<'static, Window>, rule: &str) -> usize {
    h.state_mut()
        .ui
        .viewer
        .page()
        .map_or(0, |p| p.findings.iter().filter(|f| f.rule == rule).count())
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

fn doc_of(h: &Harness<'static, Window>) -> Document {
    h.state().ui.viewer.doc().unwrap().clone()
}

#[test]
fn the_device_picker_lists_all_devices_or_one_and_is_not_saved() {
    let mut h = lint_page_window();
    open_lint_tab(&mut h);
    let count = |h: &mut Harness<'static, Window>| h.query_all_by_label_contains("⚠ L03").count();
    let by_device: Vec<usize> = [MANTA_ID, NOMAD_ID]
        .iter()
        .map(|id| {
            h.state_mut()
                .ui
                .viewer
                .page_for(id)
                .unwrap()
                .findings
                .iter()
                .filter(|f| f.rule == "L03")
                .count()
        })
        .collect();
    let all_shows_both =
        h.state().ui.lint_device().is_none() && count(&mut h) == by_device[0] + by_device[1];
    // Each row names its device when all are listed.
    let names = h.query_all_by_label("Nomad").count() >= by_device[1];
    let before = (steps(&h), h.state().ui.viewer.is_dirty());
    h.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Findings for")
        .click();
    h.run();
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Nomad")
        .click();
    h.run();
    let one = h.state().ui.lint_device() == Some(NOMAD_ID) && count(&mut h) == by_device[1];
    let after = (steps(&h), h.state().ui.viewer.is_dirty());
    // Back to all.
    h.get_by_role_and_label(egui::accesskit::Role::ComboBox, "Findings for")
        .click();
    h.run();
    h.get_by_label("All devices").click();
    h.run();
    let back = h.state().ui.lint_device().is_none() && count(&mut h) == by_device[0] + by_device[1];
    let ok = all_shows_both && names && one && back && before == after && by_device[1] > 0;
    println!(
        "CHECK crate=ui name=lint_pointer_device_picker result={} all_lists_both={all_shows_both} rows_name_device={names} one_device_lists_one={one} back_to_all={back} view_state_only={}",
        pass(ok),
        before == after
    );
    assert!(ok);
}

#[test]
fn ignore_turns_a_rule_off_and_the_list_at_the_end_turns_it_back_on() {
    let mut h = lint_page_window();
    open_lint_tab(&mut h);
    let l03_before = findings(&mut h, "L03");
    let ids = h.state().ui.viewer.profile_ids();
    let all_before = h.state_mut().ui.viewer.findings_on(&ids);
    let n0 = steps(&h);
    // The first row is L03 (the page's first finding): its Ignore button is the first one.
    let first_rule = all_before[0].1[0].rule;
    h.get_all_by_label("Ignore this rule for this document")
        .next()
        .unwrap()
        .click();
    h.run();
    let ignored = h.state().ui.viewer.ignored_rules() == vec![first_rule.to_string()];
    let gone = findings(&mut h, first_rule) == 0 && l03_before > 0;
    let listed = h.query_by_label("Ignored rules (1)").is_some();
    let one_step = steps(&h) == n0 + 1;
    // Turn back on.
    h.get_by_label("Turn back on").scroll_to_me();
    h.run();
    h.get_by_label("Turn back on").click();
    h.run();
    let restored = h.state().ui.viewer.ignored_rules().is_empty()
        && findings(&mut h, first_rule) > 0
        && h.query_by_label_contains("Ignored rules").is_none();
    let two_steps = steps(&h) == n0 + 2;
    // Each is one undo step: undo the restore, the rule is off again; undo the ignore, it is on.
    h.state_mut().ui.viewer.undo();
    let off_again = h.state().ui.viewer.ignored_rules() == vec![first_rule.to_string()];
    h.state_mut().ui.viewer.undo();
    let on_again = h.state().ui.viewer.ignored_rules().is_empty();
    let ok =
        ignored && gone && listed && one_step && restored && two_steps && off_again && on_again;
    println!(
        "CHECK crate=ui name=lint_pointer_ignore_and_restore result={} rule={first_rule} ignored_in_doc={ignored} findings_gone={gone} list_shows={listed} one_undo_step_each={} restored={restored} undo_walks_back={}",
        pass(ok),
        one_step && two_steps,
        off_again && on_again
    );
    assert!(ok);
}

/// One quick fix by the pointer: the button is there, a click clears the finding in one undo step, and undo
/// brings the finding and the document back.
fn fix_by_pointer(doc: Document, rule: &str, button: &str) -> (bool, bool, bool, bool) {
    let mut h = window(doc);
    open_lint_tab(&mut h);
    let before = doc_of(&h);
    let n0 = steps(&h);
    let had = findings(&mut h, rule) > 0;
    let there = h.query_all_by_label(button).count() > 0;
    if !there {
        return (had, false, false, false);
    }
    h.get_all_by_label(button).next().unwrap().click();
    h.run();
    let cleared = findings(&mut h, rule) == 0;
    let one_step = steps(&h) == n0 + 1 && doc_of(&h) != before;
    h.state_mut().ui.viewer.undo();
    let undone = doc_of(&h) == before && findings(&mut h, rule) > 0;
    (had && there, cleared, one_step, undone)
}

#[test]
fn every_quick_fix_works_by_the_pointer_in_one_undo_step() {
    let thin = page(vec![line(
        1,
        "thin",
        mm(300.0),
        mm(300.0),
        Style::Custom {
            width_mm: mm(0.4),
            gray: 51,
        },
    )]);
    let frac = page(vec![line(
        1,
        "frac",
        mm(300.0),
        mm(300.0),
        Style::Custom {
            width_mm: mm(2.5),
            gray: 51,
        },
    )]);
    let band = page(vec![line(
        1,
        "band",
        mm(300.0),
        mm(300.0),
        Style::Custom {
            width_mm: mm(1.0),
            gray: 120,
        },
    )]);
    let pale = page(vec![line(
        1,
        "pale",
        mm(300.0),
        mm(300.0),
        Style::Custom {
            width_mm: mm(1.0),
            gray: 230,
        },
    )]);
    let boxed = page(vec![obj(
        1,
        "logo",
        Frame::new(mm(40.0), mm(300.0), mm(200.0), mm(100.0)),
        Style::Role(LineRole::Structure),
        ObjectKind::Rect(RectShape::default()),
    )]);
    let small = page(vec![text(1, "fine print", 4.0)]);
    let one_off = page(vec![line(
        1,
        "off",
        at(2.0) + 1.7,
        at(3.0),
        Style::Role(LineRole::Guide),
    )]);
    let two_off = page(vec![
        line(
            1,
            "off a",
            at(2.0) + 1.7,
            at(3.0),
            Style::Role(LineRole::Guide),
        ),
        line(
            2,
            "off b",
            at(5.0),
            at(6.0) + 2.3,
            Style::Role(LineRole::Guide),
        ),
    ]);
    let mut pr = PatternRegion::new(Pattern::Grid);
    pr.bound = false;
    pr.h_spacing_mm = Some(5.3);
    pr.v_spacing_mm = Some(5.3);
    let unbound = page(vec![obj(
        1,
        "own grid",
        Frame::new(at(1.0), at(2.0), 80.0, 60.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Pattern(pr),
    )]);
    let mut uneven = page(vec![]);
    uneven.rhythm.origin = RhythmOrigin::MarginCorner;
    uneven.page.margins_mm = Margins {
        top: 8.0,
        right: 30.0,
        bottom: 8.0,
        left: 8.0,
    };

    let cases: Vec<(&str, &str, Document, &str)> = vec![
        ("pixel", "L01", thin, "Snap to pixel grid"),
        ("pixel_l02", "L02", frac, "Snap to pixel grid"),
        ("gray", "L05", band, "Use nearest clean gray"),
        ("role", "L03", pale, "Use Guide role"),
        ("safe", "L06", boxed, "Move inside safe area"),
        ("pt", "L07", small, "Use 6 pt"),
        ("rhythm", "L16", one_off, "Snap to rhythm"),
        ("rhythm_all", "L16", two_off, "Snap all to rhythm"),
        ("bind", "L17", unbound, "Bind to rhythm"),
        ("center", "L18", uneven, "Center rhythm origin"),
    ];
    let mut all = true;
    let mut detail = Vec::new();
    for (name, rule, doc, button) in cases {
        let r = fix_by_pointer(doc, rule, button);
        let ok = r == (true, true, true, true);
        all &= ok;
        detail.push(format!("{name}={}", if ok { "ok" } else { "FAIL" }));
        if !ok {
            detail.push(format!("{name}_detail={r:?}"));
        }
    }
    println!(
        "CHECK crate=ui name=lint_pointer_quick_fixes result={} {}",
        pass(all),
        detail.join(" ")
    );
    assert!(all);
}

#[test]
fn snap_all_hides_with_one_off_rhythm_object_and_a_fix_on_a_secondary_device_is_an_override() {
    // One off-rhythm object: "Snap to rhythm" only; two: both buttons.
    let one = page(vec![line(
        1,
        "off",
        at(2.0) + 1.7,
        at(3.0),
        Style::Role(LineRole::Guide),
    )]);
    let mut h = window(one);
    open_lint_tab(&mut h);
    let only_one = h.query_by_label("Snap all to rhythm").is_none()
        && h.query_by_label("Snap to rhythm").is_some();
    // A text 4 pt and a box under the toolbar, with the Nomad picked: the fix goes through the Nomad's overrides.
    let mut doc = page(vec![
        text(1, "fine print", 4.0),
        obj(
            2,
            "logo",
            Frame::new(mm(10.0), mm(300.0), mm(200.0), mm(100.0)),
            Style::Role(LineRole::Structure),
            ObjectKind::Rect(RectShape::default()),
        ),
    ]);
    doc.metadata.title = "secondary".into();
    let mut h = window(doc);
    open_lint_tab(&mut h);
    h.state_mut().ui.set_lint_device(Some(NOMAD_ID));
    h.run();
    let before = doc_of(&h);
    let l07_nomad = h
        .state_mut()
        .ui
        .viewer
        .page_for(NOMAD_ID)
        .unwrap()
        .findings
        .iter()
        .any(|f| f.rule == "L07");
    h.get_by_label("Use 6 pt").click();
    h.run();
    let d = doc_of(&h);
    let t = d.object(Uuid::from_u128(1)).unwrap();
    let design_untouched = matches!(&t.kind, ObjectKind::Text(tb) if tb.size_pt == 4.0);
    let override_set = t.override_on(NOMAD_ID).and_then(|o| o.font_size_pt) == Some(6.0);
    let nomad_l07_gone = !h
        .state_mut()
        .ui
        .viewer
        .page_for(NOMAD_ID)
        .unwrap()
        .findings
        .iter()
        .any(|f| f.rule == "L07");
    let manta_l07_stays = h
        .state_mut()
        .ui
        .viewer
        .page_for(MANTA_ID)
        .unwrap()
        .findings
        .iter()
        .any(|f| f.rule == "L07");
    // The box: Move inside safe area on the Nomad is a position override; the design box has not moved.
    let l06_nomad = h
        .state_mut()
        .ui
        .viewer
        .page_for(NOMAD_ID)
        .unwrap()
        .findings
        .iter()
        .any(|f| f.rule == "L06");
    let had_button = h.query_by_label("Move inside safe area").is_some();
    if had_button {
        h.get_by_label("Move inside safe area").click();
        h.run();
    }
    let d2 = doc_of(&h);
    let b = d2.object(Uuid::from_u128(2)).unwrap();
    let box_design_same = b.frame == before.object(Uuid::from_u128(2)).unwrap().frame;
    let box_override = b
        .override_on(NOMAD_ID)
        .is_some_and(|o| o.position.is_some());
    let nomad_l06_gone = !h
        .state_mut()
        .ui
        .viewer
        .page_for(NOMAD_ID)
        .unwrap()
        .findings
        .iter()
        .any(|f| f.rule == "L06");
    // Undo both: the document is as it was.
    h.state_mut().ui.viewer.undo();
    h.state_mut().ui.viewer.undo();
    let back = doc_of(&h) == before;
    let ok = only_one
        && l07_nomad
        && design_untouched
        && override_set
        && nomad_l07_gone
        && manta_l07_stays
        && l06_nomad
        && had_button
        && box_design_same
        && box_override
        && nomad_l06_gone
        && back;
    println!(
        "CHECK crate=ui name=lint_pointer_secondary_device result={} snap_all_hidden_for_one={only_one} pt_is_nomad_override={} design_pt_untouched={design_untouched} manta_still_flagged={manta_l07_stays} box_moved_by_override={} box_design_untouched={box_design_same} undo_restores={back}",
        pass(ok),
        override_set && nomad_l07_gone,
        box_override && nomad_l06_gone
    );
    assert!(ok);
}

#[test]
fn free_placement_is_a_checkbox_in_the_arrange_tab_with_a_mixed_state() {
    let doc = page(vec![
        line(
            1,
            "off a",
            at(2.0) + 1.7,
            at(3.0),
            Style::Role(LineRole::Guide),
        ),
        line(
            2,
            "off b",
            at(5.0),
            at(6.0) + 2.3,
            Style::Role(LineRole::Guide),
        ),
    ]);
    let mut h = window(doc);
    let l16_before = findings(&mut h, "L16");
    // One object: tick, one undo step, L16 leaves it, tick off, back.
    h.state_mut()
        .ui
        .viewer
        .select_row(Uuid::from_u128(1), false);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    let n0 = steps(&h);
    let present = h.query_by_label("Free placement").is_some();
    h.get_by_label("Free placement").click();
    h.run();
    let on = h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(Uuid::from_u128(1))
        .unwrap()
        .free_placement
        && steps(&h) == n0 + 1
        && findings(&mut h, "L16") == l16_before - 1;
    // Both selected: one has it, one does not: mixed; a click sets all and L16 is silent.
    h.state_mut().ui.viewer.select_row(Uuid::from_u128(2), true);
    h.run();
    let mixed = h.state_mut().ui.viewer.arrange_mixed().free_placement;
    h.get_by_label("Free placement").click();
    h.run();
    let all_on = [1u128, 2].iter().all(|i| {
        h.state()
            .ui
            .viewer
            .doc()
            .unwrap()
            .object(Uuid::from_u128(*i))
            .unwrap()
            .free_placement
    }) && findings(&mut h, "L16") == 0
        && steps(&h) == n0 + 2;
    // Another click turns all off; undo walks back one step at a time.
    h.get_by_label("Free placement").click();
    h.run();
    let all_off = [1u128, 2].iter().all(|i| {
        !h.state()
            .ui
            .viewer
            .doc()
            .unwrap()
            .object(Uuid::from_u128(*i))
            .unwrap()
            .free_placement
    });
    h.state_mut().ui.viewer.undo();
    let undo_one = [1u128, 2].iter().all(|i| {
        h.state()
            .ui
            .viewer
            .doc()
            .unwrap()
            .object(Uuid::from_u128(*i))
            .unwrap()
            .free_placement
    });
    let ok = present && on && mixed && all_on && all_off && undo_one && l16_before == 2;
    println!(
        "CHECK crate=ui name=lint_pointer_free_placement result={} checkbox_in_arrange={present} one_step_silences_l16={on} mixed_for_two={mixed} click_sets_all={all_on} off_and_undo={}",
        pass(ok),
        all_off && undo_one
    );
    assert!(ok);
}

#[test]
fn full_bleed_is_a_checkbox_beside_free_placement_with_a_mixed_state() {
    // ADR-019: two lines 2 mm from the top, under the top toolbar's zone on the Manta (L06 each).
    let doc = page(vec![
        line(1, "top a", 30.0, 2.0, Style::Role(LineRole::Guide)),
        line(2, "top b", 80.0, 2.0, Style::Role(LineRole::Guide)),
    ]);
    let mut h = window(doc);
    let l06_of = |h: &mut Harness<'static, Window>, id: u128| {
        h.state_mut().ui.viewer.page().map_or(0, |p| {
            p.findings
                .iter()
                .filter(|f| f.rule == "L06" && f.object == Some(Uuid::from_u128(id)))
                .count()
        })
    };
    let bleed = |h: &Harness<'static, Window>, id: u128| {
        h.state()
            .ui
            .viewer
            .doc()
            .unwrap()
            .object(Uuid::from_u128(id))
            .unwrap()
            .full_bleed
    };
    let before = (l06_of(&mut h, 1), l06_of(&mut h, 2));
    // One object: one undo step, L06 leaves it, the other is still caught.
    h.state_mut()
        .ui
        .viewer
        .select_row(Uuid::from_u128(1), false);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    let n0 = steps(&h);
    let present = h.query_by_label("Full bleed").is_some();
    h.get_by_label("Full bleed").click();
    h.run();
    let on = bleed(&h, 1)
        && steps(&h) == n0 + 1
        && l06_of(&mut h, 1) == 0
        && l06_of(&mut h, 2) == before.1;
    // Both selected: mixed; a click marks both and L06 is silent on them.
    h.state_mut().ui.viewer.select_row(Uuid::from_u128(2), true);
    h.run();
    let mixed = h.state_mut().ui.viewer.arrange_mixed().full_bleed;
    h.get_by_label("Full bleed").click();
    h.run();
    let all_on = bleed(&h, 1) && bleed(&h, 2) && findings(&mut h, "L06") == 0;
    // Undo takes back the last click only.
    h.state_mut().ui.viewer.undo();
    let undo_one = bleed(&h, 1) && !bleed(&h, 2);
    let ok = present && before.0 > 0 && before.1 > 0 && on && mixed && all_on && undo_one;
    println!(
        "CHECK crate=ui name=lint_pointer_full_bleed result={} checkbox_in_arrange={present} l06_before={before:?} one_step_silences_l06_on_it_only={on} mixed_for_two={mixed} click_sets_all={all_on} undo_one={undo_one}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn export_opens_one_sheet_for_warnings_and_goes_straight_on_without() {
    // Warnings: the lint page has L03 and L07 (Warn).
    let mut h = lint_page_window();
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let sheet = h.state().ui.preflight_open() && h.query_by_label("Before you export").is_some();
    let held_back = h.state().requests.is_empty();
    // Rows: Warn findings by device, with Jump, Ignore and a fix; Info folded under "N notes".
    let rows = h.query_all_by_label("Jump to object").count();
    let jump_ignore = rows > 0 && h.query_all_by_label("Ignore").count() == rows;
    let notes = h.query_by_label_contains(" notes").is_some()
        || h.query_by_label_contains("1 note").is_some();
    // Cancel: nothing goes on, the sheet is gone.
    press(&mut h, "Cancel");
    let cancelled = !h.state().ui.preflight_open() && h.state().requests.is_empty();
    // Export anyway sends the request on.
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::All;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let again = h.state().ui.preflight_open();
    press(&mut h, "Export anyway");
    let went = h.state().ui.completion_open() && !h.state().ui.preflight_open();
    // Nothing to warn about: straight on, no sheet.
    let clean = page(vec![line(
        1,
        "ok",
        at(2.0),
        at(3.0),
        Style::Role(LineRole::Guide),
    )]);
    let mut c = window(clean);
    c.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    open_export_sheet(&mut c);
    export_from_sheet(&mut c);
    let straight = !c.state().ui.preflight_open() && c.state().ui.completion_open();
    let ok = sheet && held_back && jump_ignore && notes && cancelled && again && went && straight;
    println!(
        "CHECK crate=ui name=preflight_pointer_sheet result={} opens_for_warnings={sheet} request_held={held_back} rows_have_jump_and_ignore={jump_ignore} notes_folded={notes} cancel_drops={cancelled} export_anyway_sends={went} clean_goes_straight={straight}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_preflight_row_jumps_to_its_object_ignores_its_rule_and_fixes_itself() {
    // A text 4 pt (L07) and a pale line (L03) on the Manta; the Nomad on screen when Export is asked.
    let doc = page(vec![
        text(1, "fine print", 4.0),
        line(
            2,
            "pale",
            mm(300.0),
            mm(300.0),
            Style::Custom {
                width_mm: mm(1.0),
                gray: 230,
            },
        ),
    ]);
    let mut h = window(doc);
    h.state_mut().ui.viewer.set_profile(NOMAD_ID);
    h.run();
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::All;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let rows0 = h.query_all_by_label("Jump to object").count();
    // The pale line's quick fix (a role) is on the sheet; Use 6 pt too: fix L07 from the sheet.
    let fixes_on_sheet = h.query_all_by_label("Use 6 pt").count() > 0;
    press_first(&mut h, "Use 6 pt");
    let after_fix = h.query_all_by_label("Jump to object").count();
    let fixed_one = after_fix < rows0 && h.state().ui.preflight_open();
    // Ignore the L03 rule from the sheet: its rows disappear (every device).
    let l03_rows = h.query_all_by_label_contains("⚠ L03").count();
    press_first(&mut h, "Ignore");
    let ignored = !h.state().ui.viewer.ignored_rules().is_empty();
    let rows_gone = h.query_all_by_label_contains("⚠ L03").count() < l03_rows.max(1)
        || h.query_all_by_label_contains("L07").count() < 2;
    // Jump to object: closes the sheet, selects the object, shows its device, outlines the finding.
    let remaining = h.query_all_by_label("Jump to object").count();
    let mut jumped = false;
    if remaining > 0 {
        press_first(&mut h, "Jump to object");
        let v = &h.state().ui.viewer;
        jumped = !h.state().ui.preflight_open() && v.selection.len() == 1 && v.selected.is_some();
    }
    let ok = rows0 >= 3 && fixes_on_sheet && fixed_one && ignored && rows_gone && jumped;
    println!(
        "CHECK crate=ui name=preflight_pointer_rows result={} rows_before={rows0} fix_from_sheet_removes_row={fixed_one} ignore_removes_rule_rows={} jump_closes_selects_and_outlines={jumped}",
        pass(ok),
        ignored && rows_gone
    );
    assert!(ok);
}

fn caveat() -> std::sync::Arc<Vec<u8>> {
    std::sync::Arc::new(
        quadrule_text::BUNDLED
            .iter()
            .find(|b| b.id == "caveat")
            .unwrap()
            .data
            .to_vec(),
    )
}

#[test]
fn the_sheet_has_a_document_row_for_installed_fonts_and_save_keeps_its_own_sheet() {
    const FAMILY: &str = "Test Carried Font";
    let mut doc = page(vec![obj(
        1,
        "words",
        Frame::new(at(2.0), at(3.0), 60.0, 8.0),
        Style::Role(LineRole::Text),
        ObjectKind::Text(TextBox {
            content: "words".into(),
            family: FAMILY.into(),
            weight: 400,
            size_pt: 12.0,
            ..TextBox::default()
        }),
    )]);
    doc.metadata.title = "fonts".into();
    let mut v = Viewer::default();
    v.use_installed_for_test(&[(FAMILY.into(), caveat())]);
    v.open_document_for_test(doc);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let r = w.ui.show(ui);
                w.requests.extend(r);
            },
            Window {
                ui: ViewerUi::new(v),
                requests: Vec::new(),
            },
        );
    h.run();
    // Export: one sheet, no font sheet; L20 is a row under Document and has no Jump.
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let sheet = h.state().ui.preflight_open() && !h.state().ui.font_sheet_open();
    let document =
        h.query_by_label("Document").is_some() && h.query_by_label_contains("L20").is_some();
    let no_jump = h.query_all_by_label("Jump to object").count() == 0;
    // Ignore removes the row; the sheet says nothing is left, and Export anyway still goes on.
    h.get_by_label("Ignore").hover();
    h.run();
    h.get_by_label("Ignore").click();
    h.run();
    let gone = h.query_by_label_contains("L20").is_none()
        && h.query_by_label("Nothing is left to warn about.").is_some()
        && h.state().ui.viewer.ignored_rules() == vec!["L20".to_string()];
    press(&mut h, "Export anyway");
    let sent = h.state().ui.completion_open();
    // With L20 ignored the next Export goes straight on.
    h.state_mut().requests.clear();
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::All;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let straight = !h.state().ui.preflight_open() && h.state().ui.completion_open();
    // Save keeps rung 12's sheet exactly: it opens, Cancel stops, "Don't warn again" is remembered; the
    // preflight sheet never opens for Save. (L20 is ignored for the preflight, not for the font sheet.)
    let held = h.state_mut().ui.save().is_empty()
        && h.state().ui.font_sheet_open()
        && !h.state().ui.preflight_open();
    let cancelled = h.state_mut().ui.answer_font_sheet(false, false).is_empty()
        && !h.state().ui.font_sheet_open();
    let again = h.state_mut().ui.save().is_empty() && h.state().ui.font_sheet_open();
    let _ = h.state_mut().ui.answer_font_sheet(true, true);
    let remembered = h.state().ui.viewer.doc().unwrap().page.fonts.dont_warn;
    // "Don't warn again" also quiets the preflight's L20 row (a document that opts out of the font warning).
    h.state_mut().ui.viewer.restore_rule("L20").unwrap();
    h.state_mut().requests.clear();
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let quiet = !h.state().ui.preflight_open() && h.state().ui.completion_open();
    let ok = sheet
        && document
        && no_jump
        && gone
        && sent
        && straight
        && held
        && cancelled
        && again
        && remembered
        && quiet;
    println!(
        "CHECK crate=ui name=preflight_pointer_fonts_and_save result={} sheet_not_font_sheet={sheet} l20_row_in_document={document} no_jump_for_l20={no_jump} ignore_removes_row={gone} export_anyway_sends={sent} ignored_goes_straight={straight} save_keeps_font_sheet={held} cancel_stops_save={cancelled} dont_warn_remembered={remembered} dont_warn_quiets_l20_row={quiet}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_paper_page_says_why_it_has_fewer_checks_and_this_device_lists_only_its_own_findings() {
    let mut paper = paper_document();
    paper.layers[0].objects.push(obj(
        900,
        "tiny",
        Frame::new(30.0, 30.0, 60.0, 8.0),
        Style::Role(LineRole::Text),
        ObjectKind::Text(TextBox {
            content: "tiny".into(),
            family: ATKINSON.into(),
            size_pt: 4.0,
            ..TextBox::default()
        }),
    ));
    let mut h = window(paper);
    h.state_mut().ui.viewer.settings.devices = DevicesChoice::All;
    open_export_sheet(&mut h);
    export_from_sheet(&mut h);
    let sheet = h.state().ui.preflight_open();
    let lines = h
        .query_all_by_label_contains("is paper, which has no screen to judge against")
        .count();
    let named = h
        .query_all_by_label_contains("L03, L05, L06, L13, L14")
        .count()
        > 0
        || lines > 0;
    // Both devices are paper, so each gets a line; the L07 row is on each device group.
    let groups = h.query_all_by_label_contains("L07").count();
    press(&mut h, "Cancel");
    // A screen page says nothing about paper.
    let mut s = lint_page_window();
    s.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    open_export_sheet(&mut s);
    export_from_sheet(&mut s);
    let none_for_screens = s.query_all_by_label_contains("is paper").count() == 0;
    // This device only: the Nomad on screen lists the Nomad's findings alone, not the Manta's.
    press(&mut s, "Cancel");
    s.state_mut().ui.viewer.set_profile(NOMAD_ID);
    s.run();
    s.state_mut().ui.viewer.settings.devices = DevicesChoice::This;
    open_export_sheet(&mut s);
    export_from_sheet(&mut s);
    let nomad_rows = s.query_all_by_label("Jump to object").count();
    let nomad_warns = s
        .state_mut()
        .ui
        .viewer
        .page_for(NOMAD_ID)
        .unwrap()
        .findings
        .iter()
        .filter(|f| f.severity == quadrule_lint::Severity::Warn && f.object.is_some())
        .count();
    let only_this = nomad_rows > 0 && nomad_rows >= nomad_warns;
    let ok = sheet && lines == 2 && named && groups >= 2 && none_for_screens && only_this;
    println!(
        "CHECK crate=ui name=preflight_pointer_paper_and_scope result={} paper_lines={lines} names_the_skipped_checks={named} screens_have_none={none_for_screens} this_device_rows={nomad_rows} warns_on_nomad={nomad_warns}",
        pass(ok)
    );
    assert!(ok);
}

/// Clicking a finding's line in the Lint tab picks the object it is about, with the Lint tab still open (rung 17,
/// Defaulted 2); a finding about no object only outlines it.
#[test]
fn clicking_a_finding_selects_its_object_and_keeps_the_lint_tab() {
    let mut h = lint_page_window();
    open_lint_tab(&mut h);
    let findings: Vec<quadrule_lint::Finding> = h
        .state_mut()
        .ui
        .viewer
        .page()
        .map(|p| p.findings.clone())
        .unwrap_or_default();
    // A finding whose rule appears once on its device, so its line is found by the rule's name.
    let unique = |f: &quadrule_lint::Finding| {
        findings
            .iter()
            .filter(|g| g.rule == f.rule && g.profile == f.profile)
            .count()
            == 1
    };
    let with_object = findings
        .iter()
        .find(|f| f.object.is_some() && unique(f))
        .cloned();
    let without = findings
        .iter()
        .find(|f| f.object.is_none() && unique(f))
        .cloned();
    let mut picked = false;
    let mut tab_kept = false;
    let mut outlined = false;
    if let Some(f) = &with_object {
        let line = format!("{}  ", f.rule);
        h.get_all_by_label_contains(&line)
            .next()
            .unwrap()
            .scroll_to_me();
        h.run();
        h.get_all_by_label_contains(&line).next().unwrap().hover();
        h.run();
        h.get_all_by_label_contains(&line).next().unwrap().click();
        h.run();
        h.run();
        let ui = &h.state().ui;
        picked = ui.viewer.selection == vec![f.object.unwrap()];
        tab_kept = ui.tab == Tab::Lint;
        outlined = ui.viewer.selected.is_some();
    }
    // A finding with no object leaves the selection alone (and still outlines).
    let mut page_only = without.is_none();
    if let Some(f) = &without {
        h.state_mut().ui.viewer.selection.clear();
        h.run();
        let line = format!("{}  ", f.rule);
        h.get_all_by_label_contains(&line)
            .next()
            .unwrap()
            .scroll_to_me();
        h.run();
        h.get_all_by_label_contains(&line).next().unwrap().hover();
        h.run();
        h.get_all_by_label_contains(&line).next().unwrap().click();
        h.run();
        h.run();
        let ui = &h.state().ui;
        page_only = ui.viewer.selection.is_empty() && ui.viewer.selected.is_some();
    }
    let ok = with_object.is_some() && picked && tab_kept && outlined && page_only;
    println!(
        "\nCHECK crate=ui name=lint_click_selects result={} has_object_finding={} object_picked={picked} lint_tab_kept={tab_kept} outlined={outlined} page_finding_only_outlines={page_only} (page finding in the lint page: {})",
        pass(ok),
        with_object.is_some(),
        without.is_some()
    );
    assert!(ok);
}
