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

//! Rung 6b: rotation, lines that point any way, opacity (docs/bbq/2026-11-10-rung-6-shapes.md, Q5,
//! Defaulted "6b, settled now for its builder"). Headless. Each test prints one structured `CHECK`
//! line starting with a newline (run with `-- --nocapture`).
//!
//! Coordinates are Manta device pixels. A new Handwriting page has 12 mm margins: 142 px, so the
//! rhythm origin is (142, 142) and its unit is 71 px.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_layout::Shape;
use quadrule_layout::newpage::NewPage;
use quadrule_layout::snap::Handle;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::MANTA_ID;
use quadrule_model::{LineDir, Object, ObjectKind, Uuid};
use quadrule_ui::{DragMods, HandleHit, Tab, Tool, Viewer, ViewerUi, turned_angle};

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
const FREE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn page() -> Viewer {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Rotate".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
    v.rotate_stalk_px = 60.0;
    v
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

fn obj(v: &Viewer, id: Uuid) -> Object {
    v.doc().unwrap().object(id).unwrap().clone()
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

/// A rectangle 300 by 200 px with its top-left at (400, 500), drawn and selected.
fn rect(v: &mut Viewer) -> Uuid {
    v.set_tool(Tool::Rect);
    assert!(v.draw_begin(400.0, 500.0, FREE, REACH));
    v.draw_update(700.0, 700.0, FREE, REACH);
    v.draw_end().expect("drawn")
}

/// Where the pointer must be to hold the object's `deg` angle when the handle was grabbed at its
/// tip (angle 0 is straight up, clockwise): a point on a circle round the centre.
fn at_angle(c: (f64, f64), deg: f64) -> (f64, f64) {
    let r = deg.to_radians();
    (c.0 + 500.0 * r.sin(), c.1 - 500.0 * r.cos())
}

fn turn(v: &mut Viewer, c: (f64, f64), deg: f64, mods: DragMods) -> f64 {
    let tip = v.rotate_handle().expect("a rotate handle").tip;
    assert!(v.drag_begin(tip.0, tip.1, NONE, 4.0, 6.0));
    assert_eq!(
        v.drag.as_ref().map(|d| d.kind),
        Some(quadrule_ui::DragKind::Handle(HandleHit::Rotate))
    );
    let p = at_angle(c, deg);
    // The pointer swings round from the stalk to the new angle.
    v.drag_update(p.0, p.1, mods, REACH);
    v.drag_end().unwrap();
    let id = v.selection[0];
    obj(v, id).rotation_deg
}

#[test]
fn the_rotate_handle_turns_catches_steps_and_is_one_undo_step() {
    let mut v = page();
    let id = rect(&mut v);
    let c = (550.0, 600.0);
    let before = steps(&v);
    let frame0 = obj(&v, id).frame;
    // The handle stands above the middle of the top edge, on a stalk.
    let h = v.rotate_handle().unwrap();
    let above = near(h.base.0, 550.0) && near(h.base.1, 500.0) && near(h.tip.1, 440.0);
    // 87 degrees is within 4 of a quarter turn: it catches at 90. Held with Ctrl it does not.
    let caught = turn(&mut v, c, 87.0, NONE);
    let one_step = steps(&v) == before + 1;
    v.undo();
    let free = turn(&mut v, c, 87.0, FREE);
    v.undo();
    // Shift holds 15-degree steps: 37 gives 30, 38 gives 45.
    let s30 = turn(&mut v, c, 37.0, SHIFT);
    v.undo();
    let s45 = turn(&mut v, c, 38.0, SHIFT);
    v.undo();
    // Past 180 it wraps into 0..360, and 0 stays 0.
    let back = turn(&mut v, c, 267.0, NONE);
    v.undo();
    let frame_same = obj(&v, id).frame == frame0 && obj(&v, id).rotation_deg == 0.0;
    let helper = near(turned_angle(358.0, false, true), 0.0)
        && near(turned_angle(44.0, true, true), 45.0)
        && near(turned_angle(10.0, false, true), 10.0);
    let ok = above
        && near(caught, 90.0)
        && one_step
        && near(free, 87.0)
        && near(s30, 30.0)
        && near(s45, 45.0)
        && near(back, 270.0)
        && frame_same
        && helper;
    println!(
        "\nCHECK crate=ui name=rotate_handle result={} handle_above_top_middle={above} catches_90={} ctrl_does_not_catch={} shift_37_gives={s30} shift_38_gives={s45} wraps_to_270={} one_undo_step={one_step} frame_unchanged={frame_same} turned_angle_helper={helper}",
        pass(ok),
        near(caught, 90.0),
        near(free, 87.0),
        near(back, 270.0)
    );
    assert!(ok);
}

#[test]
fn only_one_line_or_shape_has_a_rotate_handle() {
    let mut v = page();
    let a = rect(&mut v);
    let one = v.rotate_handle().is_some();
    // A text box and a pattern region do not turn.
    v.set_tool(Tool::Text);
    let t = v.place_click(300.0, 900.0, NONE, REACH).unwrap();
    let text = v.rotate_handle().is_none();
    v.set_tool(Tool::Pattern);
    let p = v.place_click(300.0, 1200.0, NONE, REACH).unwrap();
    let pattern = v.rotate_handle().is_none();
    v.selection = vec![a, t];
    let several = v.rotate_handle().is_none();
    // Locked: no handle.
    v.selection = vec![a];
    let mut locked = obj(&v, a);
    locked.locked = true;
    v.edit(quadrule_model::edit::Command::SetObject(locked), None)
        .unwrap();
    let locked_none = v.rotate_handle().is_none();
    // The field's setter refuses a kind that cannot turn.
    v.selection = vec![t, p];
    let _ = v.edit_selected(None, |o| {
        if o.kind.can_rotate() {
            o.rotation_deg = 40.0;
        }
    });
    let refused = obj(&v, t).rotation() == 0.0 && obj(&v, p).rotation() == 0.0;
    let ok = one && text && pattern && several && locked_none && refused;
    println!(
        "\nCHECK crate=ui name=rotate_handle_kinds result={} shape_has_one={one} text_none={text} pattern_none={pattern} several_none={several} locked_none={locked_none} text_and_pattern_ignore_rotation={refused}",
        pass(ok)
    );
    assert!(ok);
}

/// The world position of a handle of the selection.
fn handle_pos(v: &mut Viewer, h: Handle) -> (f64, f64) {
    v.handles()
        .into_iter()
        .find(|x| x.0 == HandleHit::Box(h))
        .map(|x| (x.1, x.2))
        .expect("handle")
}

#[test]
fn a_turned_shape_is_picked_outlined_and_resized_by_its_turned_corners() {
    let mut v = page();
    let id = rect(&mut v);
    // Turned 90: 200 wide by 300 tall on the page, about the centre (550, 600).
    v.edit_selected(None, |o| o.rotation_deg = 90.0).unwrap();
    let corners = v.page().unwrap().placed.last().unwrap().shape.corners();
    let shape_ok = corners.is_some();
    // Inside the turned box but outside the unturned one: picked. The reverse: missed.
    let inside_turned = v.pick(550.0, 470.0, 0.0) == Some(id);
    v.selection.clear();
    let outside_unturned_box = v.pick(430.0, 600.0, 0.0).is_none();
    // Handles stand on the turned corners: the north-west one (top-left before turning) is the
    // top-right corner now.
    v.selection = vec![id];
    let nw = handle_pos(&mut v, Handle::NW);
    let at_corner = near(nw.0, 650.0) && near(nw.1, 450.0);
    // The marquee takes it by the turned box: a box around a corner the turn left empty misses.
    v.selection.clear();
    v.set_tool(Tool::Select);
    assert!(v.drag_begin(410.0, 420.0, NONE, 4.0, 6.0));
    v.drag_update(440.0, 460.0, NONE, REACH);
    let empty_corner_misses = v.selection.is_empty();
    v.drag_update(560.0, 520.0, NONE, REACH);
    let touching_hits = v.selection == vec![id];
    v.drag_end().unwrap();
    // Resize: pull the south-east handle (bottom-right before turning, bottom-left now) 40 px away
    // along its own axes; the opposite (north-west) corner stays where it was on the page.
    v.selection = vec![id];
    let se = handle_pos(&mut v, Handle::SE);
    let before = steps(&v);
    assert!(v.drag_begin(se.0, se.1, NONE, 4.0, 6.0));
    // Turned 90 clockwise, the local x axis points down the page and the local y axis left.
    v.drag_update(se.0 - 30.0, se.1 + 40.0, FREE, REACH);
    v.drag_end().unwrap();
    let f = obj(&v, id);
    let nw_after = handle_pos(&mut v, Handle::NW);
    let anchored = near(nw_after.0, nw.0) && near(nw_after.1, nw.1);
    let k = 300.0 / quadrule_model::MM_PER_IN;
    let grew = near(f.frame.w_mm * k, 300.0 + 40.0) && near(f.frame.h_mm * k, 200.0 + 30.0);
    let kept = near(f.rotation_deg, 90.0) && steps(&v) == before + 1;
    // Never inside out: a pull far past the opposite corner stops at the least size.
    let se = handle_pos(&mut v, Handle::SE);
    assert!(v.drag_begin(se.0, se.1, NONE, 4.0, 6.0));
    v.drag_update(se.0 + 2000.0, se.1 - 2000.0, FREE, REACH);
    v.drag_end().unwrap();
    let f = obj(&v, id);
    let floor = f.frame.w_mm > 0.0 && f.frame.h_mm > 0.0;
    let ok = shape_ok
        && inside_turned
        && outside_unturned_box
        && at_corner
        && empty_corner_misses
        && touching_hits
        && anchored
        && grew
        && kept
        && floor;
    println!(
        "\nCHECK crate=ui name=rotated_pick_handles_resize result={} turned_shape_has_corners={shape_ok} picked_inside_turned_box={inside_turned} missed_outside_turned_box={outside_unturned_box} handle_on_turned_corner={at_corner} marquee_empty_corner_misses={empty_corner_misses} marquee_touch_hits={touching_hits} opposite_corner_stays={anchored} grew_along_own_axes={grew} angle_kept_one_step={kept} no_flip_past_zero={floor}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_turned_shape_snaps_by_the_box_round_its_turned_corners() {
    // Turned 90, the 300 x 200 rectangle at (400, 500) covers x 450..650, y 450..750. Moved so
    // that box's left edge is 3 px short of the rhythm line at x = 484 (142 + 71 * 4.8: take
    // 142 + 71 * 5 = 497): it lands on the line.
    let mut v = page();
    let id = rect(&mut v);
    v.edit_selected(None, |o| o.rotation_deg = 90.0).unwrap();
    v.snap = quadrule_layout::snap::SnapToggles {
        pixel_grid: false,
        rhythm: true,
        margins_safe: false,
        points: false,
        smart_guides: false,
        equal_spacing: false,
    };
    let Shape::Rotated { .. } = v.page().unwrap().placed.last().unwrap().shape else {
        panic!("a turned rectangle is a rotated shape")
    };
    let b0 = v.page().unwrap().placed.last().unwrap().shape.bounds();
    let aabb = near(b0.0, 450.0) && near(b0.2, 650.0) && near(b0.1, 450.0) && near(b0.3, 750.0);
    assert!(v.drag_begin(550.0, 600.0, NONE, 4.0, 6.0));
    v.drag_update(550.0 + 44.0, 600.0, NONE, REACH);
    v.drag_end().unwrap();
    let b1 = v.page().unwrap().placed.last().unwrap().shape.bounds();
    let on_line = near(b1.0, 142.0 + 71.0 * 5.0);
    let _ = id;
    let ok = aabb && on_line;
    println!(
        "\nCHECK crate=ui name=rotated_snaps_by_turned_box result={} box_is_round_turned_corners={aabb} left_edge_on_rhythm_line={on_line}",
        pass(ok)
    );
    assert!(ok);
}

fn line_ends(v: &mut Viewer, id: Uuid) -> ((f64, f64), (f64, f64)) {
    match v
        .page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .shape
    {
        Shape::Segment { x0, y0, x1, y1, .. } => ((x0, y0), (x1, y1)),
        s => panic!("not a segment: {s:?}"),
    }
}

#[test]
fn a_lines_ends_can_be_pulled_past_each_other_any_way() {
    let mut v = page();
    v.set_tool(Tool::Line);
    assert!(v.draw_begin(400.0, 400.0, FREE, REACH));
    v.draw_update(700.0, 600.0, FREE, REACH);
    let id = v.draw_end().unwrap();
    let dir = |v: &Viewer| match &obj(v, id).kind {
        ObjectKind::Line(l) => l.direction,
        _ => panic!("a line"),
    };
    let mut ok = dir(&v) == LineDir::Down;
    let mut parts = Vec::new();
    // Each pull moves the right end (the end handle) to a place; the left end stays at (400, 400).
    for (name, to, want) in [
        ("past_left_and_up", (300.0, 200.0), LineDir::Down),
        ("past_left_and_down", (300.0, 700.0), LineDir::Up),
        ("up_and_right", (800.0, 100.0), LineDir::Up),
        ("down_and_right", (900.0, 900.0), LineDir::Down),
    ] {
        let before = steps(&v);
        let ends = v.handles();
        let end = ends
            .iter()
            .find(|h| h.0 == HandleHit::LineEnd)
            .copied()
            .unwrap();
        assert!(v.drag_begin(end.1, end.2, NONE, 4.0, 6.0));
        v.drag_update(to.0, to.1, FREE, REACH);
        v.drag_end().unwrap();
        let (a, b) = line_ends(&mut v, id);
        // The two ends are the fixed point and the pointer, whichever is on the left.
        let want_ends = if to.0 < 400.0 {
            (to, (400.0, 400.0))
        } else {
            ((400.0, 400.0), to)
        };
        let good = near(a.0, want_ends.0.0)
            && near(a.1, want_ends.0.1)
            && near(b.0, want_ends.1.0)
            && near(b.1, want_ends.1.1)
            && dir(&v) == want
            && steps(&v) == before + 1;
        ok &= good;
        parts.push(format!("{name}={good}"));
        v.undo();
        // Put the line back where it was for the next pull.
    }
    // A turned line: pulling an end sets the line as it is and clears the turn.
    v.edit_selected(None, |o| o.rotation_deg = 30.0).unwrap();
    let (a0, _) = line_ends(&mut v, id);
    let start = v
        .handles()
        .into_iter()
        .find(|h| h.0 == HandleHit::LineStart)
        .unwrap();
    assert!(v.drag_begin(start.1, start.2, NONE, 4.0, 6.0));
    v.drag_update(start.1 + 10.0, start.2 + 10.0, FREE, REACH);
    v.drag_end().unwrap();
    let cleared = near(obj(&v, id).rotation_deg, 0.0);
    let _ = a0;
    ok &= cleared;
    println!(
        "\nCHECK crate=ui name=line_ends_any_way result={} {} pulled_turned_line_clears_turn={cleared}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_turned_line_has_turned_ends_and_a_rotate_handle() {
    let mut v = page();
    v.set_tool(Tool::Line);
    assert!(v.draw_begin(400.0, 500.0, FREE, REACH));
    v.draw_update(600.0, 500.0, FREE, REACH);
    let id = v.draw_end().unwrap();
    // A horizontal line from (400, 500) to (600, 500), turned 90 about (500, 500): vertical.
    let c = (500.0, 500.0);
    let got = turn(&mut v, c, 90.0, NONE);
    let (a, b) = line_ends(&mut v, id);
    let vertical = near(a.0, 500.0) && near(b.0, 500.0) && near(a.1.min(b.1), 400.0);
    let ok = near(got, 90.0) && vertical;
    println!(
        "\nCHECK crate=ui name=turned_line result={} turned_by_handle={} ends_vertical_about_centre={vertical}",
        pass(ok),
        near(got, 90.0)
    );
    assert!(ok);
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

#[test]
fn the_rotation_and_opacity_fields_are_named_and_work() {
    let mut h = harness_with(page());
    let id = rect(&mut h.state_mut().ui.viewer);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let rotation_named = h.query_by_label("Rotation").is_some();
    // The field is a drag value: click it to type, select all, type an angle, Enter.
    h.get_by_label("Rotation").click();
    h.run();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::A]);
    h.run();
    h.get_by_label("Rotation").type_text("30");
    h.key_press(Key::Enter);
    h.run();
    let by_field = near(obj(&h.state().ui.viewer, id).rotation_deg, 30.0);
    let one_step_undo = {
        h.state_mut().ui.viewer.undo();
        near(obj(&h.state().ui.viewer, id).rotation_deg, 0.0)
    };
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let opacity_named = h.query_by_label("Opacity").is_some();
    h.get_by_label("Opacity").click();
    h.run();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::A]);
    h.run();
    h.get_by_label("Opacity").type_text("40");
    h.key_press(Key::Enter);
    h.run();
    let opacity = obj(&h.state().ui.viewer, id).opacity_pct == 40;
    // A text box has no Rotation field.
    let t = {
        let v = &mut h.state_mut().ui.viewer;
        v.set_tool(Tool::Text);
        v.place_click(300.0, 1000.0, NONE, REACH).unwrap()
    };
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let none_for_text = h.query_by_label("Rotation").is_none();
    let _ = t;
    let ok =
        rotation_named && by_field && one_step_undo && opacity_named && opacity && none_for_text;
    println!(
        "\nCHECK crate=ui name=window_rotation_opacity_fields result={} rotation_named={rotation_named} rotation_typed={by_field} undone={one_step_undo} opacity_named={opacity_named} opacity_typed={opacity} text_has_no_rotation_field={none_for_text}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_benchmarks_run_over_the_transforms_page_and_turn_its_rectangle() {
    // `quadrule --transforms-page --bench-drag`, `--bench-draw --bench-tool line` and
    // `--bench-rotate`: headless they finish, time their frames and leave the page as it was.
    let want = quadrule_model::sample::transforms_document();
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 3] = [
        ("drag", quadrule_ui::DragBench::new(12).with_transforms()),
        (
            "draw_line",
            quadrule_ui::DragBench::draw(12)
                .with_transforms()
                .with_tool(Tool::Line),
        ),
        ("rotate", quadrule_ui::DragBench::rotate(12)),
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
        "\nCHECK crate=ui name=window_transforms_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

// ---- Look change (chris, 2026-11-11): the box and its handles turn with the object ----

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

fn mid(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

fn close2(a: (f64, f64), b: (f64, f64)) -> bool {
    dist(a, b) < 1e-6
}

/// The turned corners of the selected shape as the page shows it now (the preview during a drag).
fn shown_corners(v: &mut Viewer, id: Uuid) -> [(f64, f64); 4] {
    v.page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| p.shape.corners())
        .expect("a turned shape has corners")
}

/// Whether the eight handles, the rotate handle and the handle squares of the selected shape sit
/// on the turned corners it is drawn with, with the angle `deg`.
fn handles_follow(v: &mut Viewer, id: Uuid, deg: f64) -> bool {
    let c = shown_corners(v, id);
    let pos = |v: &mut Viewer, h: Handle| handle_pos(v, h);
    let on_corners = close2(pos(v, Handle::NW), c[0])
        && close2(pos(v, Handle::NE), c[1])
        && close2(pos(v, Handle::SE), c[2])
        && close2(pos(v, Handle::SW), c[3])
        && close2(pos(v, Handle::N), mid(c[0], c[1]))
        && close2(pos(v, Handle::E), mid(c[1], c[2]))
        && close2(pos(v, Handle::S), mid(c[2], c[3]))
        && close2(pos(v, Handle::W), mid(c[3], c[0]));
    let centre = mid(c[0], c[2]);
    let r = v.rotate_handle().expect("a rotate handle");
    // The stalk leaves the middle of the top edge straight outward, 60 px long.
    let out = dist(r.base, centre);
    let want_tip = (
        r.base.0 + 60.0 * (r.base.0 - centre.0) / out,
        r.base.1 + 60.0 * (r.base.1 - centre.1) / out,
    );
    let stalk = close2(r.base, mid(c[0], c[1])) && close2(r.tip, want_tip);
    // Each handle's square is turned with the box: its top side runs at the object's angle.
    let squares = v.handle_squares(5.0);
    let turned = squares.len() == 8
        && squares.iter().all(|(_, q)| {
            let side = (q[1].0 - q[0].0, q[1].1 - q[0].1);
            let a = side.1.atan2(side.0).to_degrees().rem_euclid(360.0);
            (side.0.hypot(side.1) - 10.0).abs() < 1e-6
                && ((a - deg).rem_euclid(360.0) < 1e-6 || (deg - a).rem_euclid(360.0) < 1e-6)
        });
    let angle = (v.selection_angle() - deg).abs() < 1e-6;
    on_corners && stalk && turned && angle
}

#[test]
fn the_box_and_its_handles_turn_with_the_object_while_it_turns() {
    let mut v = page();
    let id = rect(&mut v);
    let c = (550.0, 600.0);
    let tip = v.rotate_handle().unwrap().tip;
    let upright_at_start =
        v.selection_angle() == 0.0 && v.handle_squares(5.0).iter().all(|(_, q)| q[0].1 == q[1].1);
    assert!(v.drag_begin(tip.0, tip.1, NONE, 4.0, 6.0));
    let mut parts = Vec::new();
    let mut ok = upright_at_start;
    // Free of the catch, so the angles are what the pointer asks for. Each step is checked while
    // the pointer is still down, on the preview.
    for deg in [20.0, 45.0, 77.0, 133.0, 200.0, 310.0] {
        let p = at_angle(c, deg);
        v.drag_update(p.0, p.1, FREE, REACH);
        let good = handles_follow(&mut v, id, deg);
        // The committed document is still the old one: it was the cause of the stale handles.
        let uncommitted = obj(&v, id).rotation_deg == 0.0;
        ok &= good && uncommitted;
        parts.push(format!("{deg}={good}"));
    }
    v.drag_end().unwrap();
    let after = handles_follow(&mut v, id, 310.0);
    ok &= after;
    println!(
        "\nCHECK crate=ui name=handles_turn_during_rotate_drag result={} upright_at_start={upright_at_start} {} after_release={after}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_turned_shapes_handles_follow_it_through_a_move_and_a_resize() {
    let mut v = page();
    let id = rect(&mut v);
    v.edit_selected(None, |o| o.rotation_deg = 30.0).unwrap();
    let start = handle_pos(&mut v, Handle::SE);
    // Move: grab the middle, pull 100 right and 50 down; the handles are on the moved corners.
    assert!(v.drag_begin(550.0, 600.0, FREE, 4.0, 6.0));
    v.drag_update(650.0, 650.0, FREE, REACH);
    let moved = handles_follow(&mut v, id, 30.0)
        && close2(
            handle_pos(&mut v, Handle::SE),
            (start.0 + 100.0, start.1 + 50.0),
        );
    v.drag_end().unwrap();
    // Resize: pull the south-east handle outward along the turned sides; handles are on the new
    // corners at every step.
    let se = handle_pos(&mut v, Handle::SE);
    assert!(v.drag_begin(se.0, se.1, FREE, 4.0, 6.0));
    let mut resized = true;
    for k in 1..=3 {
        let d = 20.0 * f64::from(k);
        v.drag_update(se.0 + d, se.1 + d / 2.0, FREE, REACH);
        resized &= handles_follow(&mut v, id, 30.0);
    }
    v.drag_end().unwrap();
    resized &= handles_follow(&mut v, id, 30.0);
    let ok = moved && resized;
    println!(
        "\nCHECK crate=ui name=handles_follow_turned_move_and_resize result={} move={moved} resize={resized}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_turned_line_is_outlined_along_its_turned_frame() {
    let mut v = page();
    v.set_tool(Tool::Line);
    assert!(v.draw_begin(400.0, 500.0, FREE, REACH));
    v.draw_update(600.0, 500.0, FREE, REACH);
    let id = v.draw_end().unwrap();
    let upright = v.turned_line_outline(id, 3.0).is_none();
    // Turning it by the handle: while the pointer is down the outline, the selection angle and
    // the handle squares are those of the turned line.
    let c = (500.0, 500.0);
    let tip = v.rotate_handle().unwrap().tip;
    assert!(v.drag_begin(tip.0, tip.1, NONE, 4.0, 6.0));
    let p = at_angle(c, 90.0);
    v.drag_update(p.0, p.1, FREE, REACH);
    let o = v
        .turned_line_outline(id, 3.0)
        .expect("a turned line has an outline");
    // The 200 by 0 frame turned 90 about (500, 500) is 0 by 200 up and down, grown by 3.
    let xs = o.iter().map(|q| q.0);
    let ys = o.iter().map(|q| q.1);
    let (x0, x1) = (
        xs.clone().fold(f64::MAX, f64::min),
        xs.fold(f64::MIN, f64::max),
    );
    let (y0, y1) = (
        ys.clone().fold(f64::MAX, f64::min),
        ys.fold(f64::MIN, f64::max),
    );
    let box_ok = (x0 - 497.0).abs() < 1e-6
        && (x1 - 503.0).abs() < 1e-6
        && (y0 - 397.0).abs() < 1e-6
        && (y1 - 603.0).abs() < 1e-6;
    let angle = (v.selection_angle() - 90.0).abs() < 1e-6;
    // The two end handles are on the turned ends, their squares turned.
    let ends = v.handle_squares(4.0);
    let turned_ends = ends.len() == 2
        && ends
            .iter()
            .all(|(_, q)| (q[1].1 - q[0].1).abs() > 7.9 && (q[1].0 - q[0].0).abs() < 1e-6);
    v.drag_end().unwrap();
    let ok = upright && box_ok && angle && turned_ends;
    println!(
        "\nCHECK crate=ui name=turned_line_outline result={} upright_has_none={upright} outline_is_turned_frame={box_ok} angle_follows={angle} end_squares_turned={turned_ends}",
        pass(ok)
    );
    assert!(ok);
}
