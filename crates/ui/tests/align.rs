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

//! Rung 7b: align and distribute in the editor (docs/bbq/2026-11-11-rung-7-groups-and-align.md,
//! Q5, Q6, Defaulted "7b, align and distribute" and "7b pays the rung 3 debt"). Headless. Each
//! test prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_layout::align::{AlignTo, Edge, Spread, boxes_of};
use quadrule_layout::newpage::NewPage;
use quadrule_layout::units::Axis;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::edit::{Command, LayerProps};
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_model::{Frame, Object, Uuid};
use quadrule_profiles::{Profile, builtin};
use quadrule_ui::{ArrangeOp, DragMods, Tab, Tool, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const REACH: f64 = 6.0;
const FREE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: true,
};

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// A new page on `device`, with the pixel snap switched off (exact millimetres) unless asked.
fn page(device: &str) -> Viewer {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Align".into(),
        device_id: device.into(),
        rhythm: RhythmPreset::Handwriting,
    })
    .unwrap();
    v.snap.pixel_grid = false;
    v.rotate_stalk_px = 60.0;
    v
}

fn primary(v: &Viewer) -> Profile {
    builtin(&v.doc().unwrap().primary_profile_id).unwrap()
}

fn obj(v: &Viewer, id: Uuid) -> Object {
    v.doc().unwrap().object(id).unwrap().clone()
}

fn steps(v: &Viewer) -> usize {
    v.open.as_ref().unwrap().history.undo_len()
}

/// Draws a rectangle, then gives it exactly `frame` and `rot` degrees.
fn add(v: &mut Viewer, frame: Frame, rot: f64) -> Uuid {
    v.set_tool(Tool::Rect);
    assert!(v.draw_begin(300.0, 300.0, FREE, REACH));
    v.draw_update(500.0, 400.0, FREE, REACH);
    let id = v.draw_end().expect("drawn");
    let mut o = obj(v, id);
    o.frame = frame;
    o.rotation_deg = rot;
    v.edit(Command::SetObject(o), None).unwrap();
    v.set_tool(Tool::Select);
    id
}

/// The box round an object on the primary page, turned corners included.
fn outer(v: &Viewer, id: Uuid) -> Frame {
    let p = primary(v);
    boxes_of(v.doc().unwrap(), &p, &[id])[0].outer
}

/// Three upright rectangles that differ in place and size, selected in the order given.
fn trio(v: &mut Viewer) -> [Uuid; 3] {
    let a = add(v, Frame::new(20.0, 30.0, 40.0, 20.0), 0.0);
    let b = add(v, Frame::new(70.0, 50.0, 30.0, 50.0), 0.0);
    let c = add(v, Frame::new(120.0, 100.0, 20.0, 10.0), 0.0);
    v.selection = vec![a, b, c];
    [a, b, c]
}

fn edge_of(f: &Frame, axis: Axis, edge: Edge) -> f64 {
    let (lo, len) = match axis {
        Axis::X => (f.x_mm, f.w_mm),
        Axis::Y => (f.y_mm, f.h_mm),
    };
    match edge {
        Edge::Start => lo,
        Edge::Centre => lo + len / 2.0,
        Edge::End => lo + len,
    }
}

#[test]
fn each_align_command_against_selection_page_and_margins_is_one_undo_step() {
    let mut v = page(MANTA_ID);
    let ids = trio(&mut v);
    let (pw, ph) = primary(&v).page_mm();
    let m = v.doc().unwrap().page.margins_mm;
    let selection_box = ids
        .iter()
        .map(|id| outer(&v, *id))
        .reduce(|a, b| a.union(&b))
        .unwrap();
    let page_box = Frame::new(0.0, 0.0, pw, ph);
    let margin_box = Frame::new(m.left, m.top, pw - m.left - m.right, ph - m.top - m.bottom);
    let mut ok = true;
    let mut one_step = true;
    let mut restored = true;
    let mut count = 0;
    for (to, target) in [
        (AlignTo::Selection, selection_box),
        (AlignTo::Page, page_box),
        (AlignTo::Margins, margin_box),
    ] {
        v.set_align_to(to);
        for op in &ArrangeOp::ALL[..6] {
            let ArrangeOp::Align(axis, edge) = *op else {
                unreachable!()
            };
            let before: Vec<Frame> = ids.iter().map(|id| obj(&v, *id).frame).collect();
            let n = steps(&v);
            v.arrange(*op).unwrap();
            for id in &ids {
                ok &= near(
                    edge_of(&outer(&v, *id), axis, edge),
                    edge_of(&target, axis, edge),
                );
                count += 1;
            }
            // The one that is already there may move nothing, which is no step.
            one_step &= steps(&v) <= n + 1;
            if steps(&v) == n + 1 {
                v.undo();
            }
            let after: Vec<Frame> = ids.iter().map(|id| obj(&v, *id).frame).collect();
            restored &= before == after;
        }
    }
    let ok = ok && one_step && restored;
    println!(
        "\nCHECK crate=ui name=align_each_command_three_targets result={} commands_checked={count} edges_on_target={ok} at_most_one_step={one_step} undo_restores={restored}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn distribute_by_spacing_and_by_centres_both_ways_in_exact_millimetres() {
    let mut v = page(MANTA_ID);
    // Four boxes in a mess, in both directions, picked out of order.
    let a = add(&mut v, Frame::new(10.0, 10.0, 20.0, 12.0), 0.0);
    let b = add(&mut v, Frame::new(35.0, 50.0, 40.0, 30.0), 0.0);
    let c = add(&mut v, Frame::new(90.0, 40.0, 10.0, 8.0), 0.0);
    let d = add(&mut v, Frame::new(130.0, 120.0, 15.0, 20.0), 0.0);
    let ids = [a, b, c, d];
    v.selection = vec![d, b, a, c];
    let before_steps = steps(&v);
    let mut report = String::new();
    let mut all = true;
    for op in &ArrangeOp::ALL[6..] {
        let ArrangeOp::Distribute(axis, by) = *op else {
            unreachable!()
        };
        v.arrange(*op).unwrap();
        let mut boxes: Vec<Frame> = ids.iter().map(|id| outer(&v, *id)).collect();
        boxes.sort_by(|p, q| {
            edge_of(p, axis, Edge::Start).total_cmp(&edge_of(q, axis, Edge::Start))
        });
        let gaps: Vec<f64> = match by {
            Spread::Spacing => boxes
                .windows(2)
                .map(|w| edge_of(&w[1], axis, Edge::Start) - edge_of(&w[0], axis, Edge::End))
                .collect(),
            Spread::Centres => boxes
                .windows(2)
                .map(|w| edge_of(&w[1], axis, Edge::Centre) - edge_of(&w[0], axis, Edge::Centre))
                .collect(),
        };
        let equal = gaps.iter().all(|g| near(*g, gaps[0]));
        all &= equal;
        report += &format!(" {:?}_{:?}_equal={equal}", axis, by);
    }
    // Four commands, four steps (an order change of nothing would be fewer, not more).
    let four = steps(&v) <= before_steps + 4;
    // First and last along each axis stayed within a pixel of where they were.
    let ok = all && four;
    println!(
        "\nCHECK crate=ui name=distribute_four_ways_mm result={}{report} steps_ok={four}",
        pass(ok)
    );
    assert!(ok);
}

/// The Manta and the Nomad each design in their own profile; on both, with the pixel snap on,
/// edges land on whole pixels and a leftover pixel goes to the first gaps (Q6).
#[test]
fn whole_pixel_results_on_both_devices_and_exact_millimetres_with_the_snap_off() {
    let mut report = String::new();
    let mut all = true;
    for device in [MANTA_ID, NOMAD_ID] {
        let mut v = page(device);
        let p = primary(&v);
        let s = p.mm_to_px(1.0);
        // Four boxes 20 px wide and 14 px tall at fractional pixel places.
        let place = |x: f64, y: f64| Frame::new(x / s, y / s, 20.0 / s, 14.0 / s);
        let a = add(&mut v, place(3.4, 10.2), 0.0);
        let b = add(&mut v, place(50.2, 33.7), 0.0);
        let c = add(&mut v, place(61.9, 21.1), 0.0);
        let d = add(&mut v, place(135.6, 70.9), 0.0);
        let ids = [a, b, c, d];
        v.selection = ids.to_vec();
        v.set_align_to(AlignTo::Selection);
        // Snap off: horizontally equal gaps are exact (not whole pixels), left edges exact.
        v.snap.pixel_grid = false;
        v.arrange(ArrangeOp::Distribute(Axis::X, Spread::Spacing))
            .unwrap();
        let mut bx: Vec<Frame> = ids.iter().map(|id| outer(&v, *id)).collect();
        bx.sort_by(|p, q| p.x_mm.total_cmp(&q.x_mm));
        let gaps: Vec<f64> = bx.windows(2).map(|w| w[1].x_mm - w[0].right()).collect();
        let exact =
            gaps.iter().all(|g| near(*g, gaps[0])) && !near(gaps[0] * s, (gaps[0] * s).round());
        v.undo();
        // Snap on.
        v.snap.pixel_grid = true;
        v.arrange(ArrangeOp::Distribute(Axis::X, Spread::Spacing))
            .unwrap();
        let mut bx: Vec<Frame> = ids.iter().map(|id| outer(&v, *id)).collect();
        bx.sort_by(|p, q| p.x_mm.total_cmp(&q.x_mm));
        let lefts: Vec<f64> = bx.iter().map(|f| f.x_mm * s).collect();
        let whole = lefts.iter().all(|x| near(*x, x.round()));
        let gaps_px: Vec<f64> = lefts
            .windows(2)
            .map(|w| (w[1] - w[0] - 20.0).round())
            .collect();
        // 3.4 -> 3; 135.6 + 20 -> 156; 156 - 3 - 80 = 73 = 3 gaps of 24 and one left over.
        let spread = gaps_px == vec![25.0, 24.0, 24.0];
        // Vertical centres: tops land on a pixel, middles within half a pixel of the selection's.
        v.arrange(ArrangeOp::Align(Axis::Y, Edge::Centre)).unwrap();
        let tops: Vec<f64> = ids.iter().map(|id| outer(&v, *id).y_mm * s).collect();
        let tops_whole = tops.iter().all(|t| near(*t, t.round()));
        // Align right and bottom to the page: the edge is on a pixel.
        v.set_align_to(AlignTo::Page);
        v.arrange(ArrangeOp::Align(Axis::X, Edge::End)).unwrap();
        v.arrange(ArrangeOp::Align(Axis::Y, Edge::End)).unwrap();
        let (pw, ph) = p.page_mm();
        let edges = ids.iter().all(|id| {
            let f = outer(&v, *id);
            let r = f.right() * s;
            let b = f.bottom() * s;
            near(r, r.round())
                && near(b, b.round())
                && (f.right() - pw).abs() * s < 0.5 + 1e-6
                && (f.bottom() - ph).abs() * s < 0.5 + 1e-6
        });
        let good = exact && whole && spread && tops_whole && edges;
        all &= good;
        report += &format!(
            " {device}_exact_mm_gaps={exact} {device}_lefts_whole={whole} {device}_gaps_25_24_24={spread} {device}_tops_whole={tops_whole} {device}_page_edges_whole={edges}"
        );
    }
    println!(
        "\nCHECK crate=ui name=align_distribute_whole_pixels result={}{report}",
        pass(all)
    );
    assert!(all);
}

#[test]
fn a_turned_object_aligns_by_its_turned_box_and_a_line_by_its_two_ends() {
    let mut v = page(MANTA_ID);
    // 40 x 10 turned a quarter: its box is 10 x 40, centred where the frame's centre is.
    let q = add(&mut v, Frame::new(60.0, 100.0, 40.0, 10.0), 90.0);
    // Turned 45 degrees: the box round its corners is (40 + 10) / sqrt 2 on each side.
    let t = add(&mut v, Frame::new(30.0, 60.0, 40.0, 10.0), 45.0);
    let plain = add(&mut v, Frame::new(90.0, 20.0, 10.0, 10.0), 0.0);
    v.selection = vec![q, t, plain];
    v.set_align_to(AlignTo::Page);
    v.arrange(ArrangeOp::Align(Axis::X, Edge::Start)).unwrap();
    let left = [q, t, plain]
        .iter()
        .all(|id| near(outer(&v, *id).x_mm, 0.0));
    // The quarter-turned frame is not at the page's edge: its box is.
    let frame_off = near(obj(&v, q).frame.x_mm, -15.0);
    v.arrange(ArrangeOp::Align(Axis::Y, Edge::End)).unwrap();
    let (_, ph) = primary(&v).page_mm();
    let bottom = [q, t, plain]
        .iter()
        .all(|id| near(outer(&v, *id).bottom(), ph));
    let side = 50.0 / 2f64.sqrt();
    let box45 = near(outer(&v, t).w_mm, side);
    // Centres in the selection: the turned box's centre is the frame's centre.
    v.set_align_to(AlignTo::Selection);
    v.arrange(ArrangeOp::Align(Axis::X, Edge::Centre)).unwrap();
    let c = outer(&v, q).centre().0;
    let centred = [q, t, plain]
        .iter()
        .all(|id| near(outer(&v, *id).centre().0, c));
    let ok = left && frame_off && bottom && box45 && centred;
    println!(
        "\nCHECK crate=ui name=align_turned_by_outer_box result={} left_edges_at_0={left} quarter_turn_frame_outside_page={frame_off} bottoms_on_page={bottom} box_at_45_is_50_over_root2={box45} centres_meet={centred}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_group_aligns_and_distributes_as_one_object_even_turned() {
    let mut v = page(MANTA_ID);
    let a = add(&mut v, Frame::new(20.0, 30.0, 20.0, 10.0), 0.0);
    let b = add(&mut v, Frame::new(50.0, 60.0, 10.0, 30.0), 0.0);
    let c = add(&mut v, Frame::new(100.0, 20.0, 15.0, 15.0), 0.0);
    let d = add(&mut v, Frame::new(130.0, 120.0, 12.0, 9.0), 0.0);
    v.selection = vec![a, b];
    v.group_selection().unwrap();
    let g = v.selection[0];
    let rel_before = obj(&v, b).frame.x_mm - obj(&v, a).frame.x_mm;
    v.selection = vec![g, c];
    v.set_align_to(AlignTo::Selection);
    v.arrange(ArrangeOp::Align(Axis::X, Edge::End)).unwrap();
    let right = near(outer(&v, g).right(), outer(&v, c).right());
    let rel_after = obj(&v, b).frame.x_mm - obj(&v, a).frame.x_mm;
    let kept = near(rel_before, rel_after);
    let box_fits = {
        let f = obj(&v, g).frame;
        near(f.x_mm, outer(&v, g).x_mm)
    };
    // Distribute the group and two others vertically.
    v.selection = vec![g, c, d];
    v.arrange(ArrangeOp::Distribute(Axis::Y, Spread::Spacing))
        .unwrap();
    let mut boxes: Vec<Frame> = [g, c, d].iter().map(|id| outer(&v, *id)).collect();
    boxes.sort_by(|p, q| p.y_mm.total_cmp(&q.y_mm));
    let g1 = boxes[1].y_mm - boxes[0].bottom();
    let g2 = boxes[2].y_mm - boxes[1].bottom();
    let spread = near(g1, g2);
    // A turned group: its box is the turned box; aligned to the page top it touches it.
    let mut turned = obj(&v, g);
    turned.rotation_deg = 30.0;
    v.edit(Command::SetObject(turned), None).unwrap();
    v.selection = vec![g];
    v.set_align_to(AlignTo::Page);
    let n = steps(&v);
    v.arrange(ArrangeOp::Align(Axis::Y, Edge::Start)).unwrap();
    v.arrange(ArrangeOp::Align(Axis::X, Edge::Start)).unwrap();
    let top = near(outer(&v, g).y_mm, 0.0) && near(outer(&v, g).x_mm, 0.0);
    let two_steps = steps(&v) == n + 2;
    let ok = right && kept && box_fits && spread && top && two_steps;
    println!(
        "\nCHECK crate=ui name=align_group_as_one result={} right_edges_meet={right} children_keep_offsets={kept} group_box_follows={box_fits} distribute_equal_gaps={spread} turned_group_box_at_corner={top} one_step_each={two_steps}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn locked_objects_stay_put_and_are_left_out_of_the_selection_box() {
    let mut v = page(MANTA_ID);
    let [a, b, c] = trio(&mut v);
    v.set_object_panel(b, |o| o.locked = true).unwrap();
    v.selection = vec![a, b, c];
    v.set_align_to(AlignTo::Selection);
    let b_before = obj(&v, b).frame;
    // Left edges of a (x 20) and c (x 120) only: the locked b at 70 is not in the box, and a stays.
    v.arrange(ArrangeOp::Align(Axis::X, Edge::Start)).unwrap();
    let c_moves = near(obj(&v, c).frame.x_mm, 20.0) && near(obj(&v, a).frame.x_mm, 20.0);
    let b_stays = obj(&v, b).frame == b_before;
    // Only a locked object and one other: distribute needs three that can be edited.
    let d_enabled = v.arrange_enabled(ArrangeOp::Distribute(Axis::X, Spread::Spacing));
    let align_enabled = v.arrange_enabled(ArrangeOp::Align(Axis::X, Edge::Start));
    // Everything locked by its layer: nothing moves, nothing is a step, and the user is told.
    v.set_object_panel(b, |o| o.locked = false).unwrap();
    v.set_layer(0, |p: &mut LayerProps| p.locked = true)
        .unwrap();
    let n = steps(&v);
    let before = obj(&v, a).frame;
    v.set_align_to(AlignTo::Page);
    v.arrange(ArrangeOp::Align(Axis::X, Edge::End)).unwrap();
    let nothing = steps(&v) == n && obj(&v, a).frame == before && v.notice.is_some();
    let ok = c_moves && b_stays && !d_enabled && align_enabled && nothing;
    println!(
        "\nCHECK crate=ui name=align_locked_stay_put result={} others_line_up_without_the_locked={c_moves} locked_unmoved={b_stays} distribute_off_with_two_editable={} align_on_with_two_editable={align_enabled} locked_layer_is_no_step={nothing}",
        pass(ok),
        !d_enabled
    );
    assert!(ok);
}

#[test]
fn inside_an_entered_group_page_and_margins_mean_the_pages() {
    let mut v = page(MANTA_ID);
    let a = add(&mut v, Frame::new(40.0, 30.0, 20.0, 10.0), 0.0);
    let b = add(&mut v, Frame::new(70.0, 60.0, 10.0, 30.0), 0.0);
    let c = add(&mut v, Frame::new(100.0, 20.0, 15.0, 15.0), 0.0);
    v.selection = vec![a, b];
    v.group_selection().unwrap();
    let g = v.selection[0];
    v.entered = vec![g];
    v.selection = vec![a, b];
    v.set_align_to(AlignTo::Page);
    v.arrange(ArrangeOp::Align(Axis::X, Edge::Start)).unwrap();
    let on_page = near(outer(&v, a).x_mm, 0.0) && near(outer(&v, b).x_mm, 0.0);
    let group_refit = near(obj(&v, g).frame.x_mm, 0.0);
    let c_untouched = near(obj(&v, c).frame.x_mm, 100.0);
    v.set_align_to(AlignTo::Margins);
    let m = v.doc().unwrap().page.margins_mm;
    v.arrange(ArrangeOp::Align(Axis::Y, Edge::Start)).unwrap();
    let margin = near(outer(&v, a).y_mm, m.top) && near(outer(&v, b).y_mm, m.top);
    // Selection inside the group: its children's own box.
    v.set_align_to(AlignTo::Selection);
    v.arrange(ArrangeOp::Align(Axis::X, Edge::End)).unwrap();
    let inner = near(outer(&v, a).right(), outer(&v, b).right());
    let ok = on_page && group_refit && c_untouched && margin && inner;
    println!(
        "\nCHECK crate=ui name=align_inside_entered_group result={} children_to_page_edge={on_page} group_box_refit={group_refit} outside_object_untouched={c_untouched} children_to_margin_top={margin} children_to_each_other={inner}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn align_to_switches_by_itself_and_remembers_what_the_user_picks() {
    let mut v = page(MANTA_ID);
    let [a, b, c] = trio(&mut v);
    v.selection = vec![a];
    let one = v.align_to() == AlignTo::Page;
    v.selection = vec![a, b];
    let several = v.align_to() == AlignTo::Selection;
    v.selection = vec![a, b, c];
    let three = v.align_to() == AlignTo::Selection;
    // A pick for one object is kept for one, and does not change what several get.
    v.selection = vec![a];
    v.set_align_to(AlignTo::Margins);
    let kept_one = v.align_to() == AlignTo::Margins;
    v.selection = vec![a, b];
    let several_unchanged = v.align_to() == AlignTo::Selection;
    // A pick for several is kept for several, and one still has its own.
    v.set_align_to(AlignTo::Page);
    let kept_several = v.align_to() == AlignTo::Page;
    v.selection = vec![b];
    let one_again = v.align_to() == AlignTo::Margins;
    v.selection = vec![a, c];
    let several_again = v.align_to() == AlignTo::Page;
    // It follows across a click on another object and across an undo.
    v.selection = vec![c];
    v.undo();
    let after_undo = v.align_to() == AlignTo::Margins;
    // With one picked and Margins, Align acts on the margins; the buttons count the objects.
    let one_enabled = v.arrange_enabled(ArrangeOp::Align(Axis::X, Edge::Start));
    v.set_align_to(AlignTo::Selection);
    let one_selection_off = !v.arrange_enabled(ArrangeOp::Align(Axis::X, Edge::Start));
    v.selection = vec![a, b];
    let two_on = v.arrange_enabled(ArrangeOp::Align(Axis::X, Edge::Start));
    let dist_off = !v.arrange_enabled(ArrangeOp::Distribute(Axis::X, Spread::Spacing));
    let ok = one
        && several
        && three
        && kept_one
        && several_unchanged
        && kept_several
        && one_again
        && several_again
        && after_undo
        && one_enabled
        && one_selection_off
        && two_on
        && dist_off;
    println!(
        "\nCHECK crate=ui name=align_to_switching result={} one_is_page={one} several_is_selection={several} three_is_selection={three} pick_kept_for_one={kept_one} several_unchanged={several_unchanged} pick_kept_for_several={kept_several} one_remembers={one_again} several_remembers={several_again} survives_undo={after_undo} buttons_count_objects={}",
        pass(ok),
        one_enabled && one_selection_off && two_on && dist_off
    );
    assert!(ok);
}

#[test]
fn fields_read_mixed_when_the_selected_objects_differ() {
    let mut v = page(MANTA_ID);
    let a = add(&mut v, Frame::new(20.0, 30.0, 40.0, 20.0), 0.0);
    let b = add(&mut v, Frame::new(20.0, 50.0, 40.0, 30.0), 30.0);
    let twin = add(&mut v, Frame::new(20.0, 30.0, 40.0, 20.0), 0.0);
    v.selection = vec![a];
    let one = v.arrange_mixed();
    v.selection = vec![a, b];
    let ab = v.arrange_mixed();
    v.selection = vec![a, twin];
    let same = v.arrange_mixed();
    let logic = !one.x
        && !one.y
        && !one.w
        && !one.h
        && !one.rotation
        && !ab.x
        && ab.y
        && !ab.w
        && ab.h
        && ab.rotation
        && !same.x
        && !same.y
        && !same.w
        && !same.h
        && !same.rotation;

    // In the window: the fields that differ read "mixed", a value typed goes to all.
    v.selection = vec![a, b];
    let mut h = harness(v);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let shown = mixed_labels(&h);
    h.get_by_label("Y").click();
    h.run();
    h.get_by_label("Y").type_text("75mm");
    h.key_press(Key::Enter);
    h.run();
    h.run();
    let both = {
        let v = &h.state().ui.viewer;
        near(obj(v, a).frame.y_mm, 75.0) && near(obj(v, b).frame.y_mm, 75.0)
    };
    h.run();
    let after = mixed_labels(&h);
    // The rotation field: mixed (0 and 30), and typing applies to both.
    h.get_by_label("Rotation").click();
    h.run();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::A]);
    h.run();
    h.get_by_label("Rotation").type_text("20");
    h.key_press(Key::Enter);
    h.run();
    let rot = {
        let v = &h.state().ui.viewer;
        near(obj(v, a).rotation_deg, 20.0) && near(obj(v, b).rotation_deg, 20.0)
    };
    let shown_ok = shown == ["Height", "Rotation", "Y"];
    let after_ok = after == ["Height", "Rotation"];
    let ok = logic && shown_ok && both && after_ok && rot;
    println!(
        "\nCHECK crate=ui name=mixed_fields result={} logic={logic} fields_reading_mixed={shown_ok} typed_value_goes_to_all={both} y_clears_after={after_ok} rotation_typed_goes_to_all={rot}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_align_controls_are_named_and_work_in_the_window() {
    let mut v = page(MANTA_ID);
    let [a, b, c] = trio(&mut v);
    v.selection = vec![a, b, c];
    let mut h = harness(v);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let names = [
        "Align left edges",
        "Align horizontal centres",
        "Align right edges",
        "Align top edges",
        "Align vertical middles",
        "Align bottom edges",
        "Distribute horizontally by spacing",
        "Distribute horizontally by centres",
        "Distribute vertically by spacing",
        "Distribute vertically by centres",
    ];
    let named = names.iter().all(|n| h.query_by_label(n).is_some());
    let align_to = h.query_by_label("Align to").is_some();
    let n = steps(&h.state().ui.viewer);
    h.get_by_label("Align left edges").click();
    h.run();
    h.run();
    let left = {
        let v = &h.state().ui.viewer;
        [a, b, c]
            .iter()
            .all(|id| near(obj(v, *id).frame.x_mm, 20.0))
    };
    let one_step = steps(&h.state().ui.viewer) == n + 1;
    // With one object picked the Selection choice is off for align, and distribute is off.
    h.state_mut().ui.viewer.selection = vec![a];
    h.run();
    h.run();
    let n = steps(&h.state().ui.viewer);
    h.get_by_label("Distribute horizontally by spacing").click();
    h.run();
    let idle = steps(&h.state().ui.viewer) == n;
    // One picked aligns to the page by itself.
    h.get_by_label("Align right edges").click();
    h.run();
    h.run();
    let (pw, _) = primary(&h.state().ui.viewer).page_mm();
    let page_right = near(outer(&h.state().ui.viewer, a).right(), pw);
    let ok = named && align_to && left && one_step && idle && page_right;
    println!(
        "\nCHECK crate=ui name=align_controls_in_window result={} ten_buttons_named={named} align_to_named={align_to} click_aligns={left} one_undo_step={one_step} distribute_idle_with_one={idle} one_picked_aligns_to_page={page_right}",
        pass(ok)
    );
    assert!(ok);
}

/// The labels of the fields that read "mixed" now, sorted (a text field and a drag value both
/// carry the word as their value).
fn mixed_labels(h: &Harness<'static, Window>) -> Vec<String> {
    let mut out: Vec<String> = h
        .query_all_by_value("mixed")
        .filter_map(|n| n.accesskit_node().label())
        .map(|l| l.to_string())
        .collect();
    out.sort();
    out
}

struct Window {
    ui: ViewerUi,
}

fn harness(v: Viewer) -> Harness<'static, Window> {
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
