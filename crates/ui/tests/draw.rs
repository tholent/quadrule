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

//! Rung 5 steps 2 and 3: drawing with the tools (docs/bbq/2026-11-10-rung-5-new-and-insert.md,
//! Defaulted "Drawing (Q9)", "Per kind", "After drawing (Q10)", "Tests"). Headless. Each test
//! prints one structured `CHECK` line (run with `-- --nocapture`).
//!
//! Coordinates are Manta device pixels. A new Handwriting page has 12 mm margins: 142 px, so the
//! rhythm origin is (142, 142) and its unit is 71 px.

use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::edit::Command;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_model::{HConstraint, LineRole, ObjectKind, Pattern, Style, Uuid, VConstraint};
use quadrule_ui::{DragMods, Tool, Viewer};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: false,
};
const SHIFT: DragMods = DragMods {
    shift: true,
    alt: false,
    snapping_off: false,
};
const CTRL: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn page() -> Viewer {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Draw".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
    })
    .unwrap();
    v
}

/// An object's frame in Manta device pixels: (x0, y0, x1, y1).
fn px(v: &Viewer, id: Uuid) -> (f64, f64, f64, f64) {
    let f = v.doc().unwrap().object(id).unwrap().frame;
    let k = 300.0 / quadrule_model::MM_PER_IN;
    (
        f.x_mm * k,
        f.y_mm * k,
        (f.x_mm + f.w_mm) * k,
        (f.y_mm + f.h_mm) * k,
    )
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

fn draw(v: &mut Viewer, tool: Tool, from: (f64, f64), to: (f64, f64), mods: DragMods) -> Uuid {
    v.set_tool(tool);
    assert!(v.draw_begin(from.0, from.1, mods, REACH));
    v.draw_update(to.0, to.1, mods, REACH);
    v.draw_end().expect("an object was drawn")
}

fn count(v: &Viewer) -> usize {
    v.doc().unwrap().objects().count()
}

#[test]
fn a_line_is_drawn_by_drag_and_by_click() {
    let mut v = page();
    // A drag from near the rhythm origin to near four units across, with Shift: horizontal, both
    // ends on the rhythm.
    let id = draw(&mut v, Tool::Line, (145.0, 144.0), (430.0, 150.0), SHIFT);
    let (x0, y0, x1, y1) = px(&v, id);
    let o = v.doc().unwrap().object(id).unwrap().clone();
    let by_drag = near(x0, 142.0)
        && near(y0, 142.0)
        && near(x1, 142.0 + 4.0 * 71.0)
        && near(y1, 142.0)
        && matches!(o.kind, ObjectKind::Line(_))
        && o.style == Style::Role(LineRole::Guide)
        && v.selection == vec![id]
        && v.doc().unwrap().find(id) == Some((0, 0))
        && v.tool == Tool::Select;
    // A click: a horizontal line 10 units long from the click point (no catch within 6 px).
    v.set_tool(Tool::Line);
    let click = v.place_click(300.0, 600.0, NONE, REACH).unwrap();
    let (cx0, cy0, cx1, cy1) = px(&v, click);
    let by_click =
        near(cx0, 300.0) && near(cy0, 600.0) && near(cx1, 300.0 + 710.0) && near(cy1, 600.0);
    let ok = by_drag && by_click;
    println!(
        "CHECK crate=ui name=draw_line result={} drag_shift_horizontal_on_rhythm={by_drag} click_ten_units={by_click}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_line_that_would_point_up_and_right_runs_along_an_axis() {
    // The model's line is its box's top-left to bottom-right diagonal (docs/STATUS-debt.md), so a
    // drag up and to the right cannot be drawn as it was dragged: it runs along the axis the
    // pointer went further on.
    let mut v = page();
    let id = draw(&mut v, Tool::Line, (200.0, 800.0), (500.0, 600.0), CTRL);
    let (x0, y0, x1, y1) = px(&v, id);
    let horizontal = near(y0, 800.0) && near(y1, 800.0) && near(x0, 200.0) && near(x1, 500.0);
    let id2 = draw(&mut v, Tool::Line, (200.0, 800.0), (300.0, 500.0), CTRL);
    let (a0, b0, a1, b1) = px(&v, id2);
    let vertical = near(a0, 200.0) && near(a1, 200.0) && near(b0, 500.0) && near(b1, 800.0);
    let ok = horizontal && vertical;
    println!(
        "CHECK crate=ui name=draw_line_up_right_axis result={} wide_runs_horizontal={horizontal} tall_runs_vertical={vertical}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_text_box_is_drawn_by_drag_and_by_click() {
    let mut v = page();
    // Width from the drag, snapped to five units; one unit tall; role Text; says "Text".
    let id = draw(
        &mut v,
        Tool::Text,
        (145.0, 144.0),
        (142.0 + 5.0 * 71.0 + 2.0, 500.0),
        NONE,
    );
    let (x0, y0, x1, y1) = px(&v, id);
    let o = v.doc().unwrap().object(id).unwrap().clone();
    let by_drag = near(x0, 142.0)
        && near(y0, 142.0)
        && near(x1 - x0, 5.0 * 71.0)
        && near(y1 - y0, 71.0)
        && o.style == Style::Role(LineRole::Text)
        && matches!(&o.kind, ObjectKind::Text(t) if t.content == "Text" && t.family == "Atkinson Hyperlegible Next")
        && v.selection == vec![id]
        && v.focus_new_text;
    v.focus_new_text = false;
    v.set_tool(Tool::Text);
    let click = v.place_click(300.0, 900.0, NONE, REACH).unwrap();
    let (cx0, cy0, cx1, _) = px(&v, click);
    let by_click =
        near(cx0, 300.0) && near(cy0, 900.0) && near(cx1 - cx0, 710.0) && v.focus_new_text;
    let ok = by_drag && by_click;
    println!(
        "CHECK crate=ui name=draw_text result={} drag_width_snapped_and_focused={by_drag} click_ten_units={by_click}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_pattern_region_is_drawn_by_drag_and_fills_the_margins_by_click() {
    let mut v = page();
    let id = draw(
        &mut v,
        Tool::Pattern,
        (145.0, 144.0),
        (142.0 + 6.0 * 71.0 + 2.0, 142.0 + 3.0 * 71.0 - 3.0),
        NONE,
    );
    let (x0, y0, x1, y1) = px(&v, id);
    let o = v.doc().unwrap().object(id).unwrap().clone();
    let by_drag = near(x0, 142.0)
        && near(y0, 142.0)
        && near(x1 - x0, 6.0 * 71.0)
        && near(y1 - y0, 3.0 * 71.0)
        && matches!(&o.kind, ObjectKind::Pattern(p) if p.pattern == Pattern::Lines)
        && o.style == Style::Role(LineRole::Guide);
    // A click fills the page margins, pinned to every edge so it fills them on the Nomad too.
    v.set_tool(Tool::Pattern);
    let click = v.place_click(900.0, 900.0, NONE, REACH).unwrap();
    let c = v.doc().unwrap().object(click).unwrap().clone();
    let m = v.doc().unwrap().page.margins_mm;
    let (pw, ph) = quadrule_profiles::builtin(MANTA_ID).unwrap().page_mm();
    let fills = near(c.frame.x_mm, m.left)
        && near(c.frame.y_mm, m.top)
        && near(c.frame.w_mm, pw - m.left - m.right)
        && near(c.frame.h_mm, ph - m.top - m.bottom)
        && c.constraints.h == HConstraint::LeftRight
        && c.constraints.v == VConstraint::TopBottom;
    // On the Nomad it keeps the same distances from every edge.
    let nomad = quadrule_profiles::builtin(NOMAD_ID).unwrap();
    let placed = v.page_for(NOMAD_ID).unwrap().placed.clone();
    let p = placed.iter().find(|p| p.id == click).unwrap();
    let (b0, b1, b2, b3) = p.shape.bounds();
    let want = nomad.mm_to_px(m.left);
    let on_nomad = (b0 - want).abs() < 1.0
        && (b1 - nomad.mm_to_px(m.top)).abs() < 1.0
        && (b2 - (f64::from(nomad.width_px) - nomad.mm_to_px(m.right))).abs() < 1.0
        && (b3 - (f64::from(nomad.height_px) - nomad.mm_to_px(m.bottom))).abs() < 1.0;
    let ok = by_drag && fills && on_nomad;
    println!(
        "CHECK crate=ui name=draw_pattern result={} drag_snapped_to_units={by_drag} click_fills_margins={fills} same_margins_on_nomad={on_nomad}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_draw_is_one_undo_step_and_the_page_shows_it_live() {
    let mut v = page();
    let before = v.doc().unwrap().clone();
    v.set_tool(Tool::Pattern);
    assert!(v.draw_begin(145.0, 144.0, NONE, REACH));
    v.draw_update(600.0, 500.0, NONE, REACH);
    // While the pointer is down the document is untouched and the page shows the new object.
    let untouched = v.doc().unwrap() == &before && !v.is_dirty();
    let live = v.page().unwrap().placed.len() == 1 && v.drawing() && v.dragging();
    let id = v.draw_end().unwrap();
    let one_step = v.doc().unwrap().object(id).is_some() && v.is_dirty();
    let undone = v.undo() && v.doc().unwrap() == &before && !v.is_dirty();
    let redone = v.redo() && v.doc().unwrap().object(id).is_some();
    let nothing_more = !v.undo() || v.doc().unwrap() == &before;
    let ok = untouched && live && one_step && undone && redone && nothing_more;
    println!(
        "CHECK crate=ui name=draw_one_undo_step result={} doc_untouched_while_down={untouched} page_shows_it_live={live} lands_on_release={one_step} one_undo_removes_it={undone} redo_restores={redone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn escape_drops_a_draw_and_the_tool_stays() {
    let mut v = page();
    let before = v.doc().unwrap().clone();
    v.set_tool(Tool::Line);
    assert!(v.draw_begin(200.0, 300.0, NONE, REACH));
    v.draw_update(500.0, 300.0, NONE, REACH);
    v.drag_cancel();
    let dropped = !v.drawing()
        && v.doc().unwrap() == &before
        && !v.is_dirty()
        && v.page().unwrap().placed.is_empty()
        && v.tool == Tool::Line;
    // Escape with the tool on and nothing being drawn ends the tool.
    let ended = v.end_tool() && v.tool == Tool::Select && !v.end_tool();
    let ok = dropped && ended;
    println!(
        "CHECK crate=ui name=draw_escape result={} drop_leaves_no_trace_tool_stays={dropped} escape_again_ends_tool={ended}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_locked_layer_refuses_a_draw() {
    let mut v = page();
    let props = quadrule_model::edit::LayerProps {
        name: "Layer 1".into(),
        visible: true,
        locked: true,
    };
    v.edit(Command::SetLayer { index: 0, props }, None).unwrap();
    v.set_tool(Tool::Line);
    let began = v.draw_begin(200.0, 300.0, NONE, REACH);
    let placed = v.place_click(200.0, 300.0, NONE, REACH);
    let ok = !began
        && placed.is_none()
        && count(&v) == 0
        && !v.drawing()
        && v.notice.as_ref().is_some_and(|n| n.error)
        && v.tool == Tool::Line;
    println!(
        "CHECK crate=ui name=draw_locked_layer_refused result={} no_begin_no_object_error_notice={ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_tool_returns_to_select_unless_kept() {
    let mut v = page();
    v.set_tool(Tool::Line);
    v.place_click(200.0, 300.0, NONE, REACH).unwrap();
    let back = v.tool == Tool::Select && !v.tool_kept;
    v.keep_tool(Tool::Line);
    v.place_click(200.0, 400.0, NONE, REACH).unwrap();
    v.place_click(200.0, 500.0, NONE, REACH).unwrap();
    let kept = v.tool == Tool::Line && v.tool_kept && count(&v) == 3;
    let ended = v.end_tool() && v.tool == Tool::Select && !v.tool_kept;
    // Picking a tool by a single click is not keeping it; Select cannot be kept.
    v.set_tool(Tool::Text);
    let single = !v.tool_kept;
    v.keep_tool(Tool::Select);
    let select_never_kept = !v.tool_kept && v.tool == Tool::Select;
    let ok = back && kept && ended && single && select_never_kept;
    println!(
        "CHECK crate=ui name=draw_tool_returns_to_select result={} one_object_then_select={back} kept_tool_stays_for_three={kept} escape_ends_kept={ended} select_is_never_kept={select_never_kept}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_drag_shorter_than_the_reach_is_a_click() {
    let mut v = page();
    v.set_tool(Tool::Line);
    assert!(v.draw_begin(300.0, 600.0, NONE, REACH));
    v.draw_update(302.0, 601.0, NONE, REACH);
    let still_nothing = v.page().unwrap().placed.is_empty();
    let id = v.draw_end().unwrap();
    let (x0, y0, x1, y1) = px(&v, id);
    let ok = still_nothing && near(x1 - x0, 710.0) && near(y1, y0);
    println!(
        "CHECK crate=ui name=draw_short_drag_is_a_click result={} nothing_shown_before_reach={still_nothing} default_length={ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn drawing_snaps_unless_ctrl_is_held() {
    let mut v = page();
    // 3 px off the rhythm: caught with snapping on, left alone with Ctrl.
    let on = draw(&mut v, Tool::Pattern, (145.0, 145.0), (429.0, 358.0), NONE);
    let off = draw(&mut v, Tool::Pattern, (145.0, 145.0), (429.0, 358.0), CTRL);
    let (a0, b0, a1, b1) = px(&v, on);
    let (c0, d0, c1, d1) = px(&v, off);
    let snapped = near(a0, 142.0) && near(b0, 142.0) && near(a1, 426.0) && near(b1, 355.0);
    let free = near(c0, 145.0) && near(d0, 145.0) && near(c1, 429.0) && near(d1, 358.0);
    let ok = snapped && free;
    println!(
        "CHECK crate=ui name=draw_snapping result={} snaps_to_rhythm={snapped} ctrl_leaves_pointer_pixels={free}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn export_bytes_are_unchanged_after_draw_and_undo() {
    let mut v = page();
    let before = v.page_for(MANTA_ID).unwrap().image.clone();
    let before_nomad = v.page_for(NOMAD_ID).unwrap().image.clone();
    draw(&mut v, Tool::Pattern, (145.0, 144.0), (900.0, 700.0), NONE);
    draw(&mut v, Tool::Text, (145.0, 800.0), (900.0, 850.0), NONE);
    draw(&mut v, Tool::Line, (145.0, 900.0), (900.0, 900.0), SHIFT);
    let drew = v.page_for(MANTA_ID).unwrap().image != before;
    while v.undo() {}
    let same = v.page_for(MANTA_ID).unwrap().image == before
        && v.page_for(NOMAD_ID).unwrap().image == before_nomad;
    let ok = drew && same;
    println!(
        "CHECK crate=ui name=draw_export_bytes_unchanged result={} drawing_changes_the_page={drew} undo_restores_both_devices_byte_for_byte={same}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_new_object_goes_into_the_selected_objects_layer_else_the_top_unlocked_one() {
    // The sample page has two layers: Grid (0) and Header (1, the top one).
    let mut v = Viewer::default();
    v.open_sample();
    // Nothing selected: the top unlocked layer, on top of what is in it.
    let top = draw(&mut v, Tool::Line, (145.0, 144.0), (500.0, 144.0), SHIFT);
    let n = v.doc().unwrap().layers[1].objects.len();
    let on_top = v.doc().unwrap().find(top) == Some((1, n - 1));
    // A selected object in the Grid layer: the new one goes into that layer.
    v.selection = vec![quadrule_model::sample::ids::DOTS];
    let low = draw(&mut v, Tool::Line, (145.0, 300.0), (500.0, 300.0), SHIFT);
    let m = v.doc().unwrap().layers[0].objects.len();
    let in_selection_layer = v.doc().unwrap().find(low) == Some((0, m - 1));
    // With the top layer locked and nothing selected, the top unlocked layer is the next down.
    v.edit(
        Command::SetLayer {
            index: 1,
            props: quadrule_model::edit::LayerProps {
                name: "Header".into(),
                visible: true,
                locked: true,
            },
        },
        None,
    )
    .unwrap();
    v.selection.clear();
    let skipped = draw(&mut v, Tool::Line, (145.0, 500.0), (500.0, 500.0), SHIFT);
    let skips_locked = v.doc().unwrap().find(skipped).map(|p| p.0) == Some(0);
    // The selection's own layer locked: refused, not redirected.
    v.selection = vec![quadrule_model::sample::ids::TITLE];
    v.set_tool(Tool::Line);
    let refused = !v.draw_begin(145.0, 600.0, NONE, REACH);
    let ok = on_top
        && in_selection_layer
        && skips_locked
        && refused
        && v.selection == vec![quadrule_model::sample::ids::TITLE];
    println!(
        "CHECK crate=ui name=draw_layer_choice result={} top_layer_by_default={on_top} selection_layer_when_selected={in_selection_layer} skips_a_locked_top_layer={skips_locked} locked_selection_layer_refused={refused}",
        pass(ok)
    );
    assert!(ok);
}
