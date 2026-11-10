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

//! Dragging, headless (rung 4a, docs/bbq/2026-11-10-rung-4-drag-and-snap.md, Defaulted "Tests").
//! Each test prints one structured `CHECK` line, starting with a newline so `grep '^CHECK'` finds
//! it (run with `-- --nocapture`).
//!
//! The sample page on the Manta: 300 ppi, margins 8 mm = 94 px, rhythm 6 mm = 71 px.

use quadrule_layout::Shape;
use quadrule_layout::snap::SnapToggles;
use quadrule_model::sample::*;
use quadrule_model::{MM_PER_IN, Uuid};
use quadrule_ui::{DragKind, DragMods, HandleHit, Viewer};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn sample() -> Viewer {
    let mut v = Viewer::default();
    v.open_sample();
    v
}

/// Manta pixels for a length in mm.
fn px(mm: f64) -> f64 {
    mm * 300.0 / MM_PER_IN
}

fn mm(px: f64) -> f64 {
    px * MM_PER_IN / 300.0
}

const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: false,
};
const FREE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};
const REACH: f64 = 6.0;

fn grab(v: &mut Viewer, x: f64, y: f64, mods: DragMods) -> bool {
    v.drag_begin(x, y, mods, 4.0, 6.0)
}

fn frame(v: &Viewer, id: Uuid) -> quadrule_model::Frame {
    v.doc().unwrap().object(id).unwrap().frame
}

fn shape(v: &mut Viewer, id: Uuid) -> Shape {
    v.page()
        .unwrap()
        .placed
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .shape
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

#[test]
fn a_move_shows_live_and_is_one_undo_step() {
    let mut v = sample();
    let doc0 = v.doc().unwrap().clone();
    let img0 = v.page().unwrap().image.data.clone();
    let started = grab(&mut v, px(50.0), px(22.0), NONE);
    let picked = v.selection == vec![ids::RULE] && v.dragging();
    v.drag_update(px(50.0), px(22.0) + 100.0, FREE, REACH);
    // Mid-drag: the page shows the move, the document and its history do not.
    let live = matches!(shape(&mut v, ids::RULE), Shape::Segment { y0, .. } if close(y0, px(22.0) + 100.0));
    let untouched = v.doc().unwrap() == &doc0 && steps(&v) == 0 && !v.is_dirty();
    // Holding still draws nothing new.
    let rev = v.revision;
    v.drag_update(px(50.0), px(22.0) + 100.0, FREE, REACH);
    let still = v.revision == rev;
    v.drag_end().unwrap();
    let moved = close(frame(&v, ids::RULE).y_mm, 22.0 + mm(100.0));
    let one = steps(&v) == 1 && v.is_dirty() && !v.dragging();
    v.undo();
    let undone = v.doc().unwrap() == &doc0;
    // Export bytes: the page after the drag and its undo is the page before.
    let same_bytes = v.page().unwrap().image.data == img0;
    let ok =
        started && picked && live && untouched && still && moved && one && undone && same_bytes;
    println!(
        "\nCHECK crate=ui name=drag_move_one_step result={} started={started} selected_on_press={picked} live_preview={live} doc_untouched_midway={untouched} idle_no_redraw={still} moved={moved} one_step={one} undone={undone} export_bytes_same={same_bytes}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_move_snaps_to_the_rhythm_and_ctrl_turns_it_off() {
    // The rule runs along y = 22 mm = 259.8 px. Dragged up 20 px it is at 239.8, 3.8 px from the
    // rhythm line at 236: it snaps there. With snapping off it stays where the pointer put it.
    let mut v = sample();
    grab(&mut v, px(50.0), px(22.0), NONE);
    v.drag_update(px(50.0), px(22.0) - 20.0, NONE, REACH);
    let guides = v.drag_guides().to_vec();
    v.drag_end().unwrap();
    let snapped = close(px(frame(&v, ids::RULE).y_mm), 236.0);
    let guide = guides.iter().any(|g| g.at == 236.0);
    v.undo();
    grab(&mut v, px(50.0), px(22.0), NONE);
    v.drag_update(px(50.0), px(22.0) - 20.0, FREE, REACH);
    let no_guides = v.drag_guides().is_empty();
    v.drag_end().unwrap();
    let free = close(px(frame(&v, ids::RULE).y_mm), px(22.0) - 20.0);
    // The toggles are app settings: with the rhythm helper off nothing but the pixel grid works.
    v.undo();
    v.snap = SnapToggles {
        rhythm: false,
        margins_safe: false,
        pixel_grid: true,
    };
    grab(&mut v, px(50.0), px(22.0), NONE);
    v.drag_update(px(50.0), px(22.0) - 20.0, NONE, REACH);
    v.drag_end().unwrap();
    let grid_only = close(px(frame(&v, ids::RULE).y_mm), (px(22.0) - 20.0).round());
    let ok = snapped && guide && no_guides && free && grid_only;
    println!(
        "\nCHECK crate=ui name=drag_snaps result={} to_rhythm_line={snapped} guide_shown={guide} ctrl_off_no_guides={no_guides} ctrl_off_free={free} pixel_grid_only={grid_only}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn escape_cancels_and_locked_objects_do_not_drag() {
    let mut v = sample();
    let doc0 = v.doc().unwrap().clone();
    let img0 = v.page().unwrap().image.data.clone();
    grab(&mut v, px(50.0), px(22.0), NONE);
    v.drag_update(px(60.0), px(40.0), FREE, REACH);
    v.drag_cancel();
    let cancelled = !v.dragging()
        && v.doc().unwrap() == &doc0
        && steps(&v) == 0
        && !v.is_dirty()
        && v.page().unwrap().image.data == img0;
    // Cancelling an Alt-drag also takes the copy away and puts the selection back.
    v.selection = vec![ids::RULE];
    grab(&mut v, px(50.0), px(22.0), DragMods { alt: true, ..NONE });
    let copy_shown = v.selection.len() == 1 && v.selection[0] != ids::RULE;
    v.drag_update(px(60.0), px(40.0), FREE, REACH);
    v.drag_cancel();
    let alt_cancelled = v.doc().unwrap() == &doc0 && v.selection == vec![ids::RULE] && copy_shown;

    // A locked object in the selection stays; the unlocked one moves.
    v.set_object_panel(ids::RULE, |o| o.locked = true).unwrap();
    let locked_frame = frame(&v, ids::RULE);
    v.selection = vec![ids::RULE, ids::TITLE];
    let title_y = shape(&mut v, ids::TITLE);
    let Shape::Box { x0, y0, .. } = title_y else {
        panic!("a text box is a box")
    };
    grab(&mut v, x0 + 20.0, y0 + 5.0, NONE);
    v.drag_update(x0 + 20.0, y0 + 5.0 + 50.0, FREE, REACH);
    v.drag_end().unwrap();
    let rule_stayed = frame(&v, ids::RULE) == locked_frame;
    let title_moved = close(frame(&v, ids::TITLE).y_mm, 8.0 + mm(50.0));
    // A press on a locked object cannot pick it, so it draws a marquee and moves nothing.
    v.selection = vec![ids::RULE];
    let began = grab(&mut v, px(50.0), px(22.0), NONE);
    let refused = began && v.drag.as_ref().map(|d| d.kind) == Some(DragKind::Marquee);
    v.drag_cancel();
    let refused = refused && frame(&v, ids::RULE) == locked_frame;
    let ok = cancelled && alt_cancelled && rule_stayed && title_moved && refused;
    println!(
        "\nCHECK crate=ui name=drag_cancel_and_locks result={} cancelled={cancelled} alt_cancelled={alt_cancelled} locked_stayed={rule_stayed} unlocked_moved={title_moved} press_on_locked_is_marquee={refused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn resize_handles_with_shift_and_alt() {
    let ppm = 300.0 / MM_PER_IN;
    let mut v = sample();
    v.click(px(50.0), px(150.0), 4.0, false);
    let sel = v.selection == vec![ids::LINES];
    let hs = v.handles();
    let se = hs
        .iter()
        .find(|h| h.0 == HandleHit::Box(quadrule_layout::snap::Handle::SE))
        .copied()
        .unwrap();
    let f0 = frame(&v, ids::LINES);

    let pull = |v: &mut Viewer, d: (f64, f64), mods: DragMods| {
        assert!(grab(v, se.1, se.2, mods));
        assert_eq!(v.drag.as_ref().unwrap().kind, DragKind::Handle(se.0));
        v.drag_update(
            se.1 + d.0,
            se.2 + d.1,
            DragMods {
                snapping_off: true,
                ..mods
            },
            REACH,
        );
        v.drag_end().unwrap();
        let f = frame(v, ids::LINES);
        v.undo();
        f
    };
    // Plain: the bottom-right corner moves; the top-left stays.
    let f = pull(&mut v, (-60.0, -40.0), NONE);
    let plain = close(f.x_mm, f0.x_mm)
        && close(f.y_mm, f0.y_mm)
        && close(f.w_mm, f0.w_mm - 60.0 / ppm)
        && close(f.h_mm, f0.h_mm - 40.0 / ppm);
    // Shift: proportions kept (the larger ask, x: -10, y: -40 -> -10 on the longer side).
    let f = pull(
        &mut v,
        (-10.0, -40.0),
        DragMods {
            shift: true,
            ..NONE
        },
    );
    let ratio = close(f.w_mm / f.h_mm, f0.w_mm / f0.h_mm) && close(f.x_mm, f0.x_mm);
    // Alt: from the centre, both sides together.
    let f = pull(&mut v, (-60.0, -40.0), DragMods { alt: true, ..NONE });
    let centre = close(f.w_mm, f0.w_mm - 120.0 / ppm)
        && close(f.h_mm, f0.h_mm - 80.0 / ppm)
        && close(f.x_mm + f.w_mm / 2.0, f0.x_mm + f0.w_mm / 2.0)
        && close(f.y_mm + f.h_mm / 2.0, f0.y_mm + f0.h_mm / 2.0);
    let undone = steps(&v) == 0;
    let ok = sel && hs.len() == 8 && plain && ratio && centre && undone;
    println!(
        "\nCHECK crate=ui name=drag_resize result={} handles={} corner={plain} shift_ratio={ratio} alt_centre={centre} each_one_step_undone={undone}",
        pass(ok),
        hs.len()
    );
    assert!(ok);
}

#[test]
fn text_has_side_handles_and_a_line_has_two_ends() {
    let ppm = 300.0 / MM_PER_IN;
    let mut v = sample();
    v.selection = vec![ids::TITLE];
    let text: Vec<_> = v.handles().iter().map(|h| h.0).collect();
    let sides = text.len() == 2
        && text.iter().all(|h| {
            matches!(
                h,
                HandleHit::Box(quadrule_layout::snap::Handle::W | quadrule_layout::snap::Handle::E)
            )
        });
    let hs = v.handles();
    let e = hs.iter().find(|h| h.1 > 500.0).copied().unwrap();
    let t0 = frame(&v, ids::TITLE);
    grab(&mut v, e.1, e.2, NONE);
    v.drag_update(e.1 - 100.0, e.2 + 300.0, FREE, REACH);
    v.drag_end().unwrap();
    let t1 = frame(&v, ids::TITLE);
    let narrower = close(t1.w_mm, t0.w_mm - 100.0 / ppm)
        && close(t1.y_mm, t0.y_mm)
        && close(t1.h_mm, t0.h_mm)
        && close(t1.x_mm, t0.x_mm);
    v.undo();

    // A line: two end handles at its ends; pulling one end turns the line.
    v.selection = vec![ids::RULE];
    let ends = v.handles();
    let two =
        ends.len() == 2 && ends[0].0 == HandleHit::LineStart && ends[1].0 == HandleHit::LineEnd;
    let r0 = frame(&v, ids::RULE);
    let end = ends[1];
    grab(&mut v, end.1, end.2, NONE);
    v.drag_update(end.1, end.2 + 30.0, FREE, REACH);
    v.drag_end().unwrap();
    let r1 = frame(&v, ids::RULE);
    let turned = close(r1.x_mm, r0.x_mm)
        && close(r1.y_mm, r0.y_mm)
        && close(r1.w_mm, r0.w_mm)
        && close(r1.h_mm, 30.0 / ppm);
    v.undo();
    // The start end moves the other way round: the far end stays.
    let start = ends[0];
    grab(&mut v, start.1, start.2, NONE);
    v.drag_update(start.1 + 40.0, start.2, FREE, REACH);
    v.drag_end().unwrap();
    let r2 = frame(&v, ids::RULE);
    let shorter =
        close(r2.x_mm, r0.x_mm + 40.0 / ppm) && close(r2.x_mm + r2.w_mm, r0.x_mm + r0.w_mm);
    let ok = sides && narrower && two && turned && shorter;
    println!(
        "\nCHECK crate=ui name=drag_text_and_line_handles result={} text_side_handles={sides} text_narrower_only={narrower} line_two_ends={two} end_turns_line={turned} start_keeps_far_end={shorter}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn marquee_selects_what_it_touches() {
    let mut v = sample();
    let doc0 = v.doc().unwrap().clone();
    // From the empty top-left corner over the title only (the rule is at 22 mm, below the box).
    grab(&mut v, px(2.0), px(2.0), NONE);
    let kind = v.drag.as_ref().map(|d| d.kind);
    v.drag_update(px(100.0), px(20.0), NONE, REACH);
    let box_shown = v.drag_marquee().is_some();
    v.drag_end().unwrap();
    let title_only = v.selection == vec![ids::TITLE];
    // Shift: a second box over the dot grid adds to it.
    grab(
        &mut v,
        px(2.0),
        px(24.0),
        DragMods {
            shift: true,
            ..NONE
        },
    );
    v.drag_update(
        px(100.0),
        px(40.0),
        DragMods {
            shift: true,
            ..NONE
        },
        REACH,
    );
    v.drag_end().unwrap();
    let added = v.selection == vec![ids::TITLE, ids::DOTS];
    // Without Shift a new box replaces the selection; a box over nothing selects nothing.
    grab(&mut v, px(2.0), px(24.0), NONE);
    v.drag_update(px(5.0), px(25.0), NONE, REACH);
    v.drag_end().unwrap();
    let emptied = v.selection.is_empty();
    // A thin line is touched by a box that crosses it; locked and hidden objects are skipped.
    grab(&mut v, px(2.0), px(21.5), NONE);
    v.drag_update(px(5.0), px(22.5), NONE, REACH);
    v.drag_end().unwrap();
    let line = v.selection.is_empty(); // x 2..5 mm is left of the line (it starts at 8 mm)
    grab(&mut v, px(60.0), px(21.0), NONE);
    v.drag_update(px(61.0), px(23.0), NONE, REACH);
    v.drag_end().unwrap();
    let crossed = v.selection == vec![ids::RULE];
    v.set_object_panel(ids::RULE, |o| o.locked = true).unwrap();
    grab(&mut v, px(60.0), px(21.0), NONE);
    v.drag_update(px(61.0), px(23.0), NONE, REACH);
    v.drag_end().unwrap();
    let locked_skipped = v.selection.is_empty();
    // Selecting is not an edit.
    let no_edit = steps(&v) == 1 && {
        let mut d = v.doc().unwrap().clone();
        d.layers[1].objects[1].locked = false;
        d == doc0
    };
    let ok = kind == Some(DragKind::Marquee)
        && box_shown
        && title_only
        && added
        && emptied
        && line
        && crossed
        && locked_skipped
        && no_edit;
    println!(
        "\nCHECK crate=ui name=drag_marquee result={} box_shown={box_shown} title_only={title_only} shift_adds={added} nothing_selects_nothing={emptied} crossing_line={crossed} locked_skipped={locked_skipped} not_an_edit={no_edit}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn alt_drag_makes_a_copy_in_one_step() {
    let mut v = sample();
    let doc0 = v.doc().unwrap().clone();
    let n0 = doc0.objects().count();
    grab(&mut v, px(50.0), px(22.0), DragMods { alt: true, ..NONE });
    v.drag_update(px(50.0), px(22.0) + 100.0, FREE, REACH);
    let live_copy = v.page().unwrap().placed.len() == n0 + 1;
    v.drag_end().unwrap();
    let doc1 = v.doc().unwrap().clone();
    let copy = v.selection[0];
    let made = doc1.objects().count() == n0 + 1
        && copy != ids::RULE
        && close(frame(&v, copy).y_mm, 22.0 + mm(100.0))
        && close(frame(&v, ids::RULE).y_mm, 22.0)
        && doc1.object(copy).map(|o| o.name.clone())
            == doc0.object(ids::RULE).map(|o| o.name.clone());
    let one = steps(&v) == 1;
    v.undo();
    let undone = v.doc().unwrap() == &doc0;
    // Alt-drag that goes nowhere makes no copy.
    grab(&mut v, px(50.0), px(22.0), DragMods { alt: true, ..NONE });
    v.drag_update(px(50.0), px(22.0), FREE, REACH);
    v.drag_end().unwrap();
    let nowhere = v.doc().unwrap() == &doc0 && steps(&v) == 0 && v.selection == vec![ids::RULE];
    let ok = live_copy && made && one && undone && nowhere;
    println!(
        "\nCHECK crate=ui name=drag_alt_duplicate result={} live_copy={live_copy} copy_made={made} one_step={one} undone={undone} nowhere_no_copy={nowhere}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn dragging_on_the_nomad_lands_where_the_pointer_put_it() {
    // The frame is stored in the Manta's mm and the Nomad resolves it through the constraints;
    // the drag undoes that, so the object moves by exactly the pointer's distance on the Nomad.
    let mut v = sample();
    v.set_profile(NOMAD_ID);
    let Shape::Box { x0, y0, x1, .. } = shape(&mut v, ids::DOTS) else {
        panic!("a pattern is a box")
    };
    let (cx, cy) = ((x0 + x1) / 2.0, y0 + 20.0);
    grab(&mut v, cx, cy, NONE);
    v.drag_update(cx - 33.0, cy + 47.0, FREE, REACH);
    v.drag_end().unwrap();
    let Shape::Box { x0: nx, y0: ny, .. } = shape(&mut v, ids::DOTS) else {
        panic!("a pattern is a box")
    };
    let exact = close(nx - x0, -33.0) && close(ny - y0, 47.0);
    // The same drag on a screen-scaled object (Scale constraints) lands the same.
    let mut doc = v.doc().unwrap().clone();
    doc.layers[0].objects[1].constraints = quadrule_model::Constraints::default();
    let mut w = Viewer::default();
    w.open_sample();
    w.edit(
        quadrule_model::edit::Command::SetObject(doc.layers[0].objects[1].clone()),
        None,
    )
    .unwrap();
    w.set_profile(NOMAD_ID);
    let Shape::Box {
        x0: sx,
        y0: sy,
        x1: sx1,
        ..
    } = shape(&mut w, ids::LINES)
    else {
        panic!("a pattern is a box")
    };
    let (lx, ly) = ((sx + sx1) / 2.0, sy + 20.0);
    grab(&mut w, lx, ly, NONE);
    w.drag_update(lx + 21.0, ly - 9.0, FREE, REACH);
    w.drag_end().unwrap();
    let Shape::Box { x0: tx, y0: ty, .. } = shape(&mut w, ids::LINES) else {
        panic!("a pattern is a box")
    };
    let scaled = close(tx - sx, 21.0) && close(ty - sy, -9.0);
    let ok = exact && scaled;
    println!(
        "\nCHECK crate=ui name=drag_secondary_device result={} constrained_exact={exact} scaled_exact={scaled}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn layers_drag_to_reorder() {
    use quadrule_ui::{DropOn, layer_drop_place, object_drop_place};
    // Layer 0 is [Dot grid, Ruled lines], layer 1 is [Title, Title rule], bottom first.
    let mut v = sample();
    let doc0 = v.doc().unwrap().clone();
    let on = |layer, index| DropOn::Object { layer, index };
    let ids_of = |v: &Viewer, l: usize| -> Vec<Uuid> {
        v.doc().unwrap().layers[l]
            .objects
            .iter()
            .map(|o| o.id)
            .collect()
    };
    // The rule dropped on the upper half of the ruled lines' row goes above them, in layer 0.
    let place = object_drop_place(&doc0, ids::RULE, on(0, 1), true);
    let (l, i) = place.unwrap();
    v.move_object_to(ids::RULE, l, i).unwrap();
    let into_layer = place == Some((0, 2))
        && ids_of(&v, 0) == vec![ids::DOTS, ids::LINES, ids::RULE]
        && ids_of(&v, 1) == vec![ids::TITLE]
        && steps(&v) == 1;
    v.undo();
    let undone = v.doc().unwrap() == &doc0;
    // Within a layer the index counts after the object leaves: the dot grid dropped above the
    // ruled lines lands on top; dropped below them it stays, and a move that goes nowhere is no step.
    let up = object_drop_place(&doc0, ids::DOTS, on(0, 1), true);
    let stay = object_drop_place(&doc0, ids::DOTS, on(0, 1), false);
    v.move_object_to(ids::DOTS, 0, stay.unwrap().1).unwrap();
    let no_step = steps(&v) == 0;
    v.move_object_to(ids::DOTS, 0, up.unwrap().1).unwrap();
    let swapped = ids_of(&v, 0) == vec![ids::LINES, ids::DOTS];
    v.undo();
    // On a layer's own row, an object goes to the top of that layer.
    let top = object_drop_place(&doc0, ids::TITLE, DropOn::LayerTop(0), true) == Some((0, 2));
    // Layers: above or below the target, from either side.
    let layers = layer_drop_place(0, 1, true) == 1
        && layer_drop_place(1, 0, true) == 1
        && layer_drop_place(1, 0, false) == 0
        && layer_drop_place(0, 1, false) == 0;
    v.move_layer_to(0, 1).unwrap();
    let names: Vec<_> = v
        .doc()
        .unwrap()
        .layers
        .iter()
        .map(|l| l.name.clone())
        .collect();
    let layer_moved = names == vec!["Header", "Grid"] && steps(&v) == 1;
    v.undo();
    // A locked layer, or a locked object, refuses the drop and nothing changes.
    v.set_layer(0, |p| p.locked = true).unwrap();
    let before = v.doc().unwrap().clone();
    let refused = v.move_object_to(ids::RULE, 0, 2).is_err() && v.doc().unwrap() == &before;
    let ok = into_layer && undone && no_step && swapped && top && layers && layer_moved && refused;
    println!(
        "\nCHECK crate=ui name=layers_drag_reorder result={} into_layer={into_layer} undone={undone} same_place_no_step={no_step} within_layer={swapped} layer_row_is_top={top} layer_places={layers} layer_moved={layer_moved} locked_refused={refused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_right_toolbar_zone_is_shown_and_caught_by_default() {
    // chris, 2026-11-10: the right-hand safe area is included. A dragged object's edge
    // catches the left edge of the Manta's right toolbar zone, as it does the left and top ones.
    let manta = quadrule_profiles::builtin(MANTA_ID).unwrap();
    let right = manta
        .safe_zones
        .iter()
        .find(|z| z.name == "toolbar_right")
        .unwrap();
    let edge = f64::from(right.rect.0);
    let mut v = sample();
    // Rhythm and margins off: only the zones can catch. The rule's right edge ends 94 px in from the
    // page edge; drag it so that edge is 3 px short of the zone's left edge.
    v.snap = SnapToggles {
        pixel_grid: false,
        rhythm: false,
        margins_safe: true,
    };
    let Shape::Segment { x1, y0, .. } = shape(&mut v, ids::RULE) else {
        panic!("a line is a segment")
    };
    let start = px(50.0);
    grab(&mut v, start, y0, NONE);
    v.drag_update(start + (edge - 3.0 - x1), y0, NONE, REACH);
    let caught = v.drag_guides().iter().any(|g| g.at == edge);
    v.drag_cancel();
    let listed = quadrule_lint::Toolbar::ALL
        .iter()
        .all(|t| quadrule_ui::viewer::TOOLBARS_IN_USE.contains(t));
    let ok = caught && listed;
    println!(
        "\nCHECK crate=ui name=right_zone_shown_by_default result={} zone_edge_x={edge} caught={caught} every_toolbar_in_use={listed}",
        pass(ok)
    );
    assert!(ok);
}
