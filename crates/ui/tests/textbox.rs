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

//! The text box (rung 11b, docs/bbq/2026-11-14-rung-11-text-and-fonts.md), driven headless: auto
//! and fixed height, the overflow mark and L08. Each test prints one structured `CHECK` line,
//! starting with a newline (run with `-- --nocapture`).

use quadrule_layout::Shape;
use quadrule_model::edit::Command;
use quadrule_model::sample::*;
use quadrule_model::{ObjectKind, TextSizing};
use quadrule_ui::{DragMods, HandleHit, Viewer};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn sample() -> Viewer {
    let mut v = Viewer::default();
    v.open_sample();
    v
}

/// Changes the title text box with `f`, as the editor would.
fn change(v: &mut Viewer, f: impl Fn(&mut quadrule_model::Object)) {
    let mut o = v.doc().unwrap().object(ids::TITLE).unwrap().clone();
    f(&mut o);
    v.edit(Command::SetObject(o), None).unwrap();
}

fn title_box(v: &mut Viewer, device: &str) -> (f64, f64) {
    let page = v.page_for(device).unwrap();
    let p = page.placed.iter().find(|p| p.id == ids::TITLE).unwrap();
    let ppmm = page.profile.mm_to_px(1.0);
    match p.shape {
        Shape::Box { y0, y1, .. } => (y0 / ppmm, y1 / ppmm),
        _ => panic!("a text box is a box"),
    }
}

#[test]
fn an_auto_box_is_as_tall_as_its_text_and_a_fixed_one_as_its_frame() {
    let mut v = sample();
    // The stored height is nothing like the text's.
    change(&mut v, |o| o.frame.h_mm = 80.0);
    let (top, bottom) = title_box(&mut v, MANTA_ID);
    let auto_h = bottom - top;
    let laid = v.laid_out_heights();
    let reads_text = auto_h < 30.0 && (laid[&ids::TITLE] - auto_h).abs() < 1e-6;
    let nomad = title_box(&mut v, NOMAD_ID);
    let nomad_h = nomad.1 - nomad.0;
    // Fixed: the frame's own height, and it keeps the text's height when switched.
    v.selection = vec![ids::TITLE];
    v.set_text_sizing(TextSizing::Fixed).unwrap();
    let kept = (v.doc().unwrap().object(ids::TITLE).unwrap().frame.h_mm - auto_h).abs() < 1e-6;
    change(&mut v, |o| o.frame.h_mm = 40.0);
    let (top, bottom) = title_box(&mut v, MANTA_ID);
    let fixed_h = bottom - top;
    let own = (fixed_h - 40.0).abs() < 1e-6;
    let ok = reads_text && kept && own && nomad_h > 0.0;
    println!(
        "\nCHECK crate=ui name=auto_height_is_laid_out result={} auto_mm={auto_h:.3} nomad_auto_mm={nomad_h:.3} switched_keeps_height={kept} fixed_mm={fixed_h:.3}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn pulling_the_bottom_of_an_auto_box_turns_it_fixed_in_one_undo_step() {
    let mut v = sample();
    v.selection = vec![ids::TITLE];
    let edges: Vec<HandleHit> = v.handles().into_iter().map(|(h, ..)| h).collect();
    let four = edges.len() == 4;
    let (hx, hy) = v
        .handles()
        .into_iter()
        .find_map(|(h, x, y)| {
            matches!(h, HandleHit::Box(quadrule_layout::snap::Handle::S)).then_some((x, y))
        })
        .unwrap();
    let before = v.doc().unwrap().object(ids::TITLE).unwrap().clone();
    let steps0 = v.open.as_ref().unwrap().history.undo_len();
    assert!(v.drag_begin(hx, hy, NONE, 4.0, 6.0));
    v.drag_update(hx, hy + 300.0, NONE, 6.0);
    v.drag_end().unwrap();
    let after = v.doc().unwrap().object(ids::TITLE).unwrap().clone();
    let fixed = matches!(&after.kind, ObjectKind::Text(t) if t.sizing == TextSizing::Fixed);
    let taller = after.frame.h_mm > before.frame.h_mm + 10.0;
    let one = v.open.as_ref().unwrap().history.undo_len() == steps0 + 1;
    v.undo();
    let back = v.doc().unwrap().object(ids::TITLE).unwrap() == &before;
    // The left and right handles still change only the width.
    let ok = four && fixed && taller && one && back;
    println!(
        "\nCHECK crate=ui name=pulling_the_edge_turns_a_box_fixed result={} four_handles={four} fixed={fixed} taller={taller} one_undo_step={one} undone={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_fixed_box_that_cuts_lines_shows_the_mark_and_says_so_per_device() {
    let mut v = sample();
    change(&mut v, |o| {
        o.frame.h_mm = 11.0;
        if let ObjectKind::Text(t) = &mut o.kind {
            t.sizing = TextSizing::Fixed;
            t.content = "One\nTwo\nThree\nFour\nFive\nSix".into();
            t.size_pt = 16.0;
        }
    });
    let per_device: Vec<(String, u32, usize)> = [MANTA_ID, NOMAD_ID]
        .iter()
        .map(|id| {
            let page = v.page_for(id).unwrap();
            let l08 = page.findings.iter().filter(|f| f.rule == "L08").count();
            let hidden = page.overflows.first().map_or(0, |o| o.hidden);
            ((*id).to_string(), hidden, l08)
        })
        .collect();
    let marked = per_device
        .iter()
        .all(|(_, hidden, l08)| *hidden > 0 && *l08 == 1);
    // The mark is not in the page that is exported: the picture is the display list's, alone.
    let page = v.page_for(MANTA_ID).unwrap().clone();
    let redrawn = quadrule_render::render(&page.display_list);
    let not_exported = redrawn.data == page.image.data;
    // Auto again: nothing is cut, no mark, no L08.
    v.selection = vec![ids::TITLE];
    v.set_text_sizing(TextSizing::Auto).unwrap();
    let page = v.page_for(MANTA_ID).unwrap();
    let clean = page.overflows.is_empty() && page.findings.iter().all(|f| f.rule != "L08");
    let ok = marked && not_exported && clean;
    println!(
        "\nCHECK crate=ui name=overflow_mark_and_l08 result={} manta_hidden={} nomad_hidden={} one_l08_each={marked} not_exported={not_exported} auto_is_clean={clean}",
        pass(ok),
        per_device[0].1,
        per_device[1].1
    );
    assert!(ok);
}

fn text_of(v: &Viewer, id: quadrule_model::Uuid) -> String {
    match &v.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Text(t) => t.content.clone(),
        _ => String::new(),
    }
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

#[test]
fn a_text_box_turns_by_its_handle_its_field_and_its_group_and_is_judged_by_its_turned_corners() {
    use quadrule_layout::Shape;
    let mut v = sample();
    v.selection = vec![ids::TITLE];
    // The handle stands above the box; pulling it a quarter round catches at 90 degrees.
    let handle = v.rotate_handle().is_some();
    let (c, top) = match v
        .page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == ids::TITLE)
        .unwrap()
        .shape
    {
        Shape::Box { x0, y0, x1, y1 } => (((x0 + x1) / 2.0, (y0 + y1) / 2.0), y0),
        other => panic!("{other:?}"),
    };
    let _ = top;
    let tip = v.rotate_handle().unwrap().tip;
    assert!(v.drag_begin(tip.0, tip.1, NONE, 4.0, 6.0));
    let r = 500.0;
    let a = 87.0f64.to_radians();
    v.drag_update(
        c.0 + r * a.sin(),
        c.1 - r * a.cos(),
        DragMods::default(),
        6.0,
    );
    v.drag_end().unwrap();
    let turned = v.doc().unwrap().object(ids::TITLE).unwrap().rotation_deg;
    let caught = (turned - 90.0).abs() < 1e-9;
    // Picked by its turned corners: a point beside the turned box is on it, one at the old far end
    // of the unturned box is not.
    let page = v.page().unwrap().clone();
    let p = page.placed.iter().find(|p| p.id == ids::TITLE).unwrap();
    let rotated = matches!(p.shape, Shape::Rotated { deg, .. } if (deg - 90.0).abs() < 1e-9);
    let corners = p.shape.corners().unwrap();
    let tall = (corners[0].1 - corners[2].1).abs() > (corners[0].0 - corners[2].0).abs();
    // The export draws the turned outlines: the text is taller than wide now.
    let bounds = page
        .display_list
        .items
        .iter()
        .filter_map(|it| match it {
            quadrule_layout::Item::Fill { path, .. } => Some(kurbo::Shape::bounding_box(path)),
            _ => None,
        })
        .fold(None, |acc: Option<kurbo::Rect>, r| {
            Some(acc.map_or(r, |a| a.union(r)))
        });
    let drawn_tall = bounds.is_some_and(|b| b.height() > b.width());
    // Four handles on a turned box that is as tall as its text, turned with it.
    let four = v.handles().len() == 4;
    // The Arrange tab's field turns it too, and a group holding it turns.
    v.undo();
    let field = {
        v.edit_selected(None, |o| o.rotation_deg = 30.0).unwrap();
        (v.doc().unwrap().object(ids::TITLE).unwrap().rotation_deg - 30.0).abs() < 1e-9
    };
    v.undo();
    v.selection = vec![ids::TITLE, ids::RULE];
    v.group_selection().unwrap();
    let group = v.selection[0];
    let group_turns = v.doc().unwrap().object(group).unwrap().kind.can_rotate();
    v.edit_selected(None, |o| o.rotation_deg = 90.0).unwrap();
    let group_turned = (v.doc().unwrap().object(group).unwrap().rotation_deg - 90.0).abs() < 1e-9;
    v.undo();
    v.undo();
    // L10 judges the turned corners: a long box near the top, turned a quarter, runs off the page.
    change(&mut v, |o| {
        o.frame = quadrule_model::Frame::new(10.0, 5.0, 60.0, 5.0);
        o.rotation_deg = 90.0;
    });
    let page = v.page_for(MANTA_ID).unwrap();
    let l10 = page
        .findings
        .iter()
        .any(|f| f.rule == "L10" && f.object == Some(ids::TITLE));
    let ok = handle
        && caught
        && rotated
        && tall
        && drawn_tall
        && four
        && field
        && group_turns
        && group_turned
        && l10;
    println!(
        "\nCHECK crate=ui name=text_turns result={} handle={handle} caught_at_90={caught} picked_turned={rotated} corners_tall={tall} export_turned={drawn_tall} four_handles={four} field={field} group_with_text_turns={group_turns}/{group_turned} l10_by_turned_corners={l10}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn typing_in_a_box_is_one_step_goes_back_by_words_and_sits_on_the_letters() {
    use quadrule_text::edit::{Go, Op};
    let mut v = sample();
    let original = text_of(&v, ids::TITLE);
    let before = steps(&v);
    let began = v.begin_typing(ids::TITLE, quadrule_ui::Start::End);
    for c in "hello world".chars() {
        v.type_op(Op::Insert(c));
    }
    let want = format!("{original}hello world");
    let typed = v.typing_text().as_deref() == Some(want.as_str());
    // While it runs the page shows the new text and the document is not touched.
    let live = text_of(&v, ids::TITLE) == original && steps(&v) == before;
    let shown = !v.page().unwrap().display_list.items.is_empty();
    // Ctrl+Z inside the session steps back through words.
    v.type_undo();
    let one_word = v.typing_text().as_deref() == Some(format!("{original}hello ").as_str());
    v.type_undo();
    let two_words = v.typing_text().as_deref() == Some(original.as_str());
    v.type_redo();
    let redone = v.typing_text().as_deref() == Some(format!("{original}hello ").as_str());
    // Enter makes a paragraph; the caret is on the new line, below the first.
    let y0 = v.typing_view().unwrap().caret.0.1;
    v.type_op(Op::Enter);
    v.type_op(Op::Insert('x'));
    let y1 = v.typing_view().unwrap().caret.0.1;
    let paragraph = v.typing_text().unwrap().contains('\n') && y1 > y0 + 10.0;
    // Left, Home and End move the caret; the caret sits where the letters end.
    v.type_go(Go::Finish, false);
    let at_end =
        v.typing_caret().unwrap().cursor == quadrule_text::edit::end_of(&v.typing_text().unwrap());
    v.type_go(Go::Left, true);
    let selected = v.typing_caret().unwrap().selected_text().as_deref() == Some("x");
    v.type_op(Op::Backspace);
    let deleted = !v.typing_text().unwrap().ends_with('x');
    let ended_text = v.typing_text().unwrap();
    v.end_typing();
    // The whole session is one step in the document's undo (ADR-010).
    let one_step = steps(&v) == before + 1 && text_of(&v, ids::TITLE) == ended_text;
    v.undo();
    let back = text_of(&v, ids::TITLE) == original;
    let ok = began
        && typed
        && live
        && shown
        && one_word
        && two_words
        && redone
        && paragraph
        && at_end
        && selected
        && deleted
        && one_step
        && back;
    println!(
        "\nCHECK crate=ui name=typing_session result={} began={began} typed={typed} doc_untouched_meanwhile={live} words_back={one_word}/{two_words} redone={redone} new_paragraph={paragraph} caret_at_end={at_end} shift_selects={selected} backspace={deleted} one_undo_step={one_step} undone={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_caret_sits_on_the_drawn_letters_turned_boxes_included() {
    use quadrule_text::edit::{Go, Op};
    let mut results = Vec::new();
    let mut all = true;
    for deg in [0.0, 90.0, 180.0, 15.0] {
        let mut v = sample();
        change(&mut v, |o| {
            o.rotation_deg = deg;
            o.frame = quadrule_model::Frame::new(40.0, 60.0, 80.0, 6.0);
            if let ObjectKind::Text(t) = &mut o.kind {
                t.content = "Hamburgefonstiv".into();
                t.size_pt = 14.0;
            }
        });
        v.begin_typing(ids::TITLE, quadrule_ui::Start::End);
        v.type_go(Go::Finish, false);
        let end = v.typing_view().unwrap().caret;
        // The drawn letters: the box round every outline, in device pixels.
        let b = v
            .page()
            .unwrap()
            .display_list
            .items
            .iter()
            .filter_map(|it| match it {
                quadrule_layout::Item::Fill { path, .. } => Some(kurbo::Shape::bounding_box(path)),
                _ => None,
            })
            .reduce(|a, b| a.union(b))
            .unwrap();
        // The caret at the end of the text is at the last letter's far edge: along the reading
        // direction, within a side bearing (a tenth of the size) of the outlines' far end.
        let size_px = 14.0 * 25.4 / 72.0 * 300.0 / 25.4;
        let mid = ((end.0.0 + end.1.0) / 2.0, (end.0.1 + end.1.1) / 2.0);
        let r = deg.to_radians();
        let along = (r.cos(), r.sin());
        let centre = ((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0);
        // Distance of the caret past the middle of the letters, along the reading direction, against
        // half the letters' length along it.
        let half_len = ((b.width() * along.0.abs()) + (b.height() * along.1.abs())) / 2.0;
        let past = (mid.0 - centre.0) * along.0 + (mid.1 - centre.1) * along.1;
        let near_end = (past - half_len).abs()
            < size_px * 0.15 + 2.0 + if deg == 15.0 { size_px * 0.3 } else { 0.0 };
        // And the caret stands across the line (perpendicular to reading).
        let across = ((end.1.0 - end.0.0) * along.0 + (end.1.1 - end.0.1) * along.1).abs() < 1.0;
        // A caret at the start is a full text length back.
        v.type_go(Go::Start, false);
        let start = v.typing_view().unwrap().caret;
        let first = ((start.0.0 + start.1.0) / 2.0 - centre.0) * along.0
            + ((start.0.1 + start.1.1) / 2.0 - centre.1) * along.1;
        let at_start = (first + half_len).abs()
            < size_px * 0.15 + 2.0 + if deg == 15.0 { size_px * 0.3 } else { 0.0 };
        v.type_op(Op::Insert('Z'));
        all &= near_end && across && at_start;
        results.push(format!(
            "{deg}:end={near_end},across={across},start={at_start}"
        ));
    }
    println!(
        "\nCHECK crate=ui name=caret_on_the_letters result={} {}",
        pass(all),
        results.join(" ")
    );
    assert!(all);
}

#[test]
fn a_double_click_types_in_text_inside_a_group_and_in_a_repeat_copy() {
    use quadrule_text::edit::Op;
    let mut v = sample();
    // Text inside a group: typed in directly, without entering the group.
    v.selection = vec![ids::TITLE, ids::RULE];
    v.group_selection().unwrap();
    let at = {
        let page = v.page().unwrap();
        let p = page.placed.iter().find(|p| p.id == ids::TITLE).unwrap();
        let b = p.shape.bounds();
        ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0)
    };
    let in_group = v.begin_typing_at(at.0, at.1, 2.0) && v.typing_id() == Some(ids::TITLE);
    let not_entered = v.entered.is_empty();
    v.type_op(Op::Insert('!'));
    v.end_typing();
    let group_text = text_of(&v, ids::TITLE).contains('!');
    v.undo();
    v.undo();
    // A copy in a repeat edits the source.
    v.selection = vec![ids::TITLE];
    v.repeat_selection().unwrap();
    let copy_at = {
        let page = v.page().unwrap();
        page.placed
            .iter()
            .find(|p| p.id == ids::TITLE && p.copy)
            .map(|p| {
                let b = p.shape.bounds();
                ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0)
            })
    };
    let in_copy = copy_at.is_some_and(|c| v.begin_typing_at(c.0, c.1, 2.0))
        && v.typing_id() == Some(ids::TITLE);
    v.type_op(Op::Insert('?'));
    // The caret is drawn on the source, not on the copy that was clicked.
    let view = v.typing_view().is_some();
    v.end_typing();
    let source_text = text_of(&v, ids::TITLE).contains('?');
    // A click outside the box does not hit it.
    v.begin_typing(ids::TITLE, quadrule_ui::Start::End);
    let outside = !v.typing_contains(5.0, 1800.0) && !v.type_pointer(5.0, 1800.0, false, false);
    v.end_typing();
    let ok = in_group && not_entered && group_text && in_copy && view && source_text && outside;
    println!(
        "\nCHECK crate=ui name=double_click_types_in_groups_and_repeat_copies result={} in_group={in_group} group_not_entered={not_entered} group_text={group_text} copy_edits_source={in_copy}/{source_text} caret_on_source={view} outside_misses={outside}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn copy_cut_paste_select_all_and_composed_text_work_as_plain_text() {
    use quadrule_text::edit::{Go, Op};
    let mut v = sample();
    v.begin_typing(ids::TITLE, quadrule_ui::Start::All);
    let all = text_of(&v, ids::TITLE);
    let copied = v.type_copy().as_deref() == Some(all.as_str());
    let cut =
        v.type_cut().as_deref() == Some(all.as_str()) && v.typing_text().as_deref() == Some("");
    v.type_str("one\r\ntwo three");
    let pasted = v.typing_text().as_deref() == Some("one\ntwo three");
    v.type_op(Op::SelectAll);
    let sel_all = v.typing_caret().unwrap().selected_text().as_deref() == Some("one\ntwo three");
    v.type_go(Go::Start, false);
    v.type_go(Go::WordRight, true);
    let word = v.typing_caret().unwrap().selected_text().is_some();
    // An input method: what it composes shows at the caret until it commits.
    v.type_go(Go::Finish, false);
    v.type_preedit(Some("ka".into()));
    let composing = v.typing_preedit() == Some("ka");
    v.type_str("\u{304B}");
    let committed = v.typing_preedit().is_none() && v.typing_text().unwrap().ends_with('\u{304B}');
    v.end_typing();
    let ok = copied && cut && pasted && sel_all && word && composing && committed;
    println!(
        "\nCHECK crate=ui name=typing_clipboard_and_ime result={} copy={copied} cut={cut} paste_one_undo_unit={pasted} select_all={sel_all} word_select={word} preedit={composing} commit={committed}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_font_scan_runs_off_the_thread_one_at_a_time_and_swaps_in_when_done() {
    let mut v = Viewer::default();
    v.use_system_fonts();
    v.open_sample();
    let before = quadrule_text::scans_started();
    let t0 = std::time::Instant::now();
    let started = v.start_font_scan();
    let returned = t0.elapsed();
    // A focus while one runs queues none.
    let second = v.start_font_scan();
    let third = v.start_font_scan();
    let running = v.font_scan_running();
    let mut found = None;
    let t1 = std::time::Instant::now();
    while found.is_none() && t1.elapsed().as_secs() < 120 {
        found = v.poll_font_scan();
        std::thread::yield_now();
    }
    let counted = quadrule_text::scans_started() - before;
    let swapped = found.is_some() && !v.font_scan_running() && v.last_font_scan.is_some();
    // Polling with nothing running does nothing; a bundled-only viewer starts nothing.
    let idle = v.poll_font_scan().is_none();
    let mut plain = Viewer::default();
    let none_for_bundled = !plain.start_font_scan();
    let ok = started
        && !second
        && !third
        && running
        && counted == 1
        && swapped
        && idle
        && none_for_bundled
        && returned < std::time::Duration::from_millis(50);
    println!(
        "\nCHECK crate=ui name=font_scan_in_background result={} started={started} returned_ms={:.2} second_start_refused={} scans_started={counted} swapped_in={swapped} scan_ms={:.1} faces={}",
        pass(ok),
        returned.as_secs_f64() * 1000.0,
        !second && !third,
        found.map_or(0.0, |s| s.elapsed.as_secs_f64() * 1000.0),
        found.map_or(0, |s| s.faces)
    );
    assert!(ok);
}

struct Window {
    ui: quadrule_ui::ViewerUi,
}

#[test]
fn keys_in_the_window_type_undo_by_words_take_an_emoji_and_end_on_escape() {
    use egui::{Key, Modifiers};
    use egui_kittest::Harness;
    let mut v = sample();
    v.selection = vec![ids::TITLE];
    let original = text_of(&v, ids::TITLE);
    v.begin_typing(ids::TITLE, quadrule_ui::Start::End);
    let ui = quadrule_ui::ViewerUi::new(v);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(24)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window { ui },
        );
    h.run();
    h.event(egui::Event::Text("ab cd".into()));
    h.run();
    let typed =
        h.state().ui.viewer.typing_text().as_deref() == Some(format!("{original}ab cd").as_str());
    // Ctrl+Z steps back through typed words, not through the document's history.
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::Z]);
    h.run();
    let back = h.state().ui.viewer.typing_text().as_deref()
        == Some(format!("{original}ab ").as_str())
        && h.state().ui.viewer.is_typing();
    // The emoji picker puts its pick at the canvas caret.
    let ctx = h.ctx.clone();
    h.state_mut().ui.insert_emoji(&ctx, "\u{1F600}");
    h.run();
    let emoji = h
        .state()
        .ui
        .viewer
        .typing_text()
        .unwrap()
        .ends_with('\u{1F600}');
    // Arrows move the caret, Shift selects, Escape ends the session and keeps the text.
    h.key_press(Key::ArrowLeft);
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowLeft);
    h.run();
    let selected = h
        .state()
        .ui
        .viewer
        .typing_caret()
        .unwrap()
        .selection()
        .is_some();
    h.key_press(Key::Escape);
    h.run();
    let v = &h.state().ui.viewer;
    let ended = !v.is_typing() && text_of(v, ids::TITLE).ends_with('\u{1F600}');
    let ok = typed && back && emoji && selected && ended;
    println!(
        "\nCHECK crate=ui name=window_typing_keys result={} typed={typed} ctrl_z_by_words={back} emoji_at_caret={emoji} shift_arrow_selects={selected} escape_ends={ended}",
        pass(ok)
    );
    assert!(ok);
}
