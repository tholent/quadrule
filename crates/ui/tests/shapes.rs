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

//! Rung 6a step 3 and 4: drawing the new shapes with the R, E and Y tools, and the inspector's
//! fields for them (docs/bbq/2026-11-10-rung-6-shapes.md, Q4, Defaulted "Drawing", "New shape
//! style", "Inspector", "Snapping"). Headless. Each test prints one structured `CHECK` line (run
//! with `-- --nocapture`).
//!
//! Coordinates are Manta device pixels. A new Handwriting page has 12 mm margins: 142 px, so the
//! rhythm origin is (142, 142) and its unit is 71 px.

use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_layout::newpage::NewPage;
use quadrule_layout::shapes::natural_ratio;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_model::{
    DashStyle, Fill, FillRole, LineCap, LineRole, Object, ObjectKind, Style, Uuid,
};
use quadrule_ui::{DragMods, Tab, Tool, Viewer, ViewerUi};

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
        title: "Shapes".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
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

fn obj(v: &Viewer, id: Uuid) -> Object {
    v.doc().unwrap().object(id).unwrap().clone()
}

/// Each tool and the kind it makes.
fn kind_ok(o: &Object, tool: Tool) -> bool {
    match (&o.kind, tool) {
        (ObjectKind::Rect(r), Tool::Rect) => r.corner_radius_mm == 0.0,
        (ObjectKind::Ellipse(_), Tool::Ellipse) => true,
        (ObjectKind::Polygon(p), Tool::Polygon) => p.sides == 6 && p.star.is_none(),
        _ => false,
    }
}

#[test]
fn each_shape_is_drawn_by_drag_and_by_click() {
    let mut v = page();
    let mut parts = Vec::new();
    let mut ok = true;
    for (tool, name) in [
        (Tool::Rect, "rect"),
        (Tool::Ellipse, "ellipse"),
        (Tool::Polygon, "polygon"),
    ] {
        // A drag from near the rhythm origin out to about (4, 3) units: both corners land on the
        // rhythm, the new shape is a Structure stroke with no fill, and it is selected.
        let id = draw(
            &mut v,
            tool,
            (145.0, 144.0),
            (142.0 + 4.0 * 71.0 + 2.0, 142.0 + 3.0 * 71.0 - 3.0),
            NONE,
        );
        let (x0, y0, x1, y1) = px(&v, id);
        let o = obj(&v, id);
        let by_drag = near(x0, 142.0)
            && near(y0, 142.0)
            && near(x1 - x0, 4.0 * 71.0)
            && near(y1 - y0, 3.0 * 71.0)
            && kind_ok(&o, tool)
            && o.style == Style::Role(LineRole::Structure)
            && o.fill.is_none()
            && o.dash == DashStyle::Solid
            && o.cap == LineCap::Butt
            && v.selection == vec![id]
            && v.tool == Tool::Select;
        // A click: a 10-unit square (circle, polygon) at the click point, 710 px across.
        v.set_tool(tool);
        let click = v.place_click(300.0, 1200.0, NONE, REACH).unwrap();
        let (cx0, cy0, cx1, cy1) = px(&v, click);
        let ratio = if tool == Tool::Polygon {
            natural_ratio(&quadrule_ui::new_polygon())
        } else {
            1.0
        };
        let by_click = near(cx0, 300.0)
            && near(cy0, 1200.0)
            && near(cx1 - cx0, 710.0)
            && near((cx1 - cx0) / (cy1 - cy0), ratio)
            && kind_ok(&obj(&v, click), tool);
        ok &= by_drag && by_click;
        parts.push(format!("{name}:drag={by_drag}:click={by_click}"));
    }
    println!(
        "\nCHECK crate=ui name=draw_shapes_drag_and_click result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn shift_draws_a_square_a_circle_and_a_regular_polygon() {
    let mut v = page();
    let mut parts = Vec::new();
    let mut ok = true;
    for (tool, name) in [
        (Tool::Rect, "square"),
        (Tool::Ellipse, "circle"),
        (Tool::Polygon, "regular_polygon"),
    ] {
        let ratio = if tool == Tool::Polygon {
            natural_ratio(&quadrule_ui::new_polygon())
        } else {
            1.0
        };
        // A wide drag: the width wins and the height follows, both ways round, up and left too.
        let a = draw(
            &mut v,
            tool,
            (400.0, 400.0),
            (400.0 + 301.0, 400.0 + 90.0),
            SHIFT,
        );
        let (x0, y0, x1, y1) = px(&v, a);
        let wide = ((x1 - x0) / (y1 - y0) - ratio).abs() < 1e-6 && (x1 - x0) > 250.0;
        let b = draw(
            &mut v,
            tool,
            (900.0, 900.0),
            (900.0 - 80.0, 900.0 - 300.0),
            SHIFT,
        );
        let (u0, w0, u1, w1) = px(&v, b);
        let tall = ((u1 - u0) / (w1 - w0) - ratio).abs() < 1e-6
            && (w1 - w0) > 250.0
            && near(u1, 900.0)
            && near(w1, 900.0);
        // Without Shift the shape is whatever was dragged.
        let c = draw(
            &mut v,
            tool,
            (400.0, 1400.0),
            (400.0 + 301.0, 1400.0 + 90.0),
            CTRL,
        );
        let (p0, q0, p1, q1) = px(&v, c);
        let free = near(p1 - p0, 301.0) && near(q1 - q0, 90.0);
        ok &= wide && tall && free;
        parts.push(format!(
            "{name}:wide={wide}:tall_up_left={tall}:free_without_shift={free}"
        ));
    }
    println!(
        "\nCHECK crate=ui name=draw_shapes_shift result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_shape_is_one_undo_step_and_the_tool_goes_back_to_select_unless_kept() {
    let mut v = page();
    let before = v.doc().unwrap().clone();
    v.set_tool(Tool::Ellipse);
    assert!(v.draw_begin(145.0, 144.0, NONE, REACH));
    v.draw_update(600.0, 500.0, NONE, REACH);
    let untouched = v.doc().unwrap() == &before && !v.is_dirty();
    let live = v.page().unwrap().placed.len() == 1 && v.drawing();
    let id = v.draw_end().unwrap();
    let one_step = v.open.as_ref().unwrap().history.undo_len() == 1;
    let back = v.tool == Tool::Select;
    let undone = v.undo() && v.doc().unwrap() == &before;
    let redone = v.redo() && v.doc().unwrap().object(id).is_some();
    // A double-click on the button keeps the tool on for several shapes.
    v.keep_tool(Tool::Polygon);
    v.place_click(200.0, 300.0, NONE, REACH).unwrap();
    v.place_click(200.0, 1000.0, NONE, REACH).unwrap();
    let kept = v.tool == Tool::Polygon && v.tool_kept && v.doc().unwrap().objects().count() == 3;
    let ended = v.end_tool() && v.tool == Tool::Select;
    // Escape drops a shape being drawn.
    v.set_tool(Tool::Rect);
    v.draw_begin(300.0, 300.0, NONE, REACH);
    v.draw_update(700.0, 700.0, NONE, REACH);
    v.drag_cancel();
    let dropped = !v.drawing() && v.doc().unwrap().objects().count() == 3;
    let ok = untouched && live && one_step && back && undone && redone && kept && ended && dropped;
    println!(
        "\nCHECK crate=ui name=draw_shapes_undo_and_tool result={} untouched_while_down={untouched} live_preview={live} one_undo_step={one_step} back_to_select={back} undo_redo={} kept_for_two={kept} escape_drops={dropped}",
        pass(ok),
        undone && redone
    );
    assert!(ok);
}

#[test]
fn shapes_snap_by_their_frames() {
    // Rung 4's helpers act on the new kinds' frames: a moved rectangle's top-left catches the
    // rhythm; its edge catches another shape's edge (smart guides); Ctrl turns it all off.
    let mut v = page();
    let anchor = draw(
        &mut v,
        Tool::Ellipse,
        (142.0, 142.0 + 7.0 * 71.0),
        (142.0 + 3.0 * 71.0, 142.0 + 10.0 * 71.0),
        NONE,
    );
    let moving = draw(
        &mut v,
        Tool::Rect,
        (145.0, 144.0),
        (142.0 + 2.0 * 71.0, 142.0 + 2.0 * 71.0),
        NONE,
    );
    // Press inside the rectangle and pull it by (+3 units - 3 px, +2 units + 2 px): the frame
    // lands on the rhythm.
    let (x0, y0, _, _) = px(&v, moving);
    v.selection.clear();
    assert!(v.drag_begin(x0 + 40.0, y0 + 40.0, NONE, 4.0, 6.0));
    v.drag_update(
        x0 + 40.0 + 3.0 * 71.0 - 3.0,
        y0 + 40.0 + 2.0 * 71.0 + 2.0,
        NONE,
        REACH,
    );
    v.drag_end().unwrap();
    let (a0, b0, a1, b1) = px(&v, moving);
    let on_rhythm = near(a0, 142.0 + 3.0 * 71.0)
        && near(b0, 142.0 + 2.0 * 71.0)
        && near(a1 - a0, 2.0 * 71.0)
        && near(b1 - b0, 2.0 * 71.0 - 3.0 + 3.0);
    // Ctrl: the frame follows the pointer exactly.
    v.selection.clear();
    assert!(v.drag_begin(a0 + 40.0, b0 + 40.0, CTRL, 4.0, 6.0));
    v.drag_update(a0 + 40.0 + 7.0, b0 + 40.0 + 5.0, CTRL, REACH);
    v.drag_end().unwrap();
    let (c0, d0, _, _) = px(&v, moving);
    let free = near(c0, a0 + 7.0) && near(d0, b0 + 5.0);
    // The rectangle's bottom edge against the ellipse's top edge: pulled to within reach, it
    // catches (smart guides or points, whichever is on).
    let (_, ey0, _, _) = px(&v, anchor);
    let (_, my0, _, my1) = px(&v, moving);
    v.selection.clear();
    let press = (c0 + 40.0, my0 + 40.0);
    assert!(v.drag_begin(press.0, press.1, NONE, 4.0, 6.0));
    let gap = ey0 - my1;
    v.drag_update(press.0, press.1 + gap - 3.0, NONE, REACH);
    v.drag_end().unwrap();
    let (_, _, _, fy1) = px(&v, moving);
    let caught = near(fy1, ey0);
    let ok = on_rhythm && free && caught;
    println!(
        "\nCHECK crate=ui name=shapes_snap_by_frame result={} top_left_on_rhythm={on_rhythm} ctrl_is_free={free} edge_catches_a_neighbour={caught}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn shapes_resolve_on_both_devices_and_export_differs_from_the_empty_page() {
    // A shape drawn on the Manta shows on the Nomad too (its frame scales), and drawing it and
    // undoing it puts both devices' pages back byte for byte.
    let mut v = page();
    let before = v.page_for(MANTA_ID).unwrap().image.clone();
    let before_nomad = v.page_for(NOMAD_ID).unwrap().image.clone();
    let id = draw(
        &mut v,
        Tool::Rect,
        (145.0, 144.0),
        (142.0 + 5.0 * 71.0, 142.0 + 4.0 * 71.0),
        NONE,
    );
    v.edit_selected(None, |o| {
        o.fill = Some(Fill::Role(FillRole::Block));
    })
    .unwrap();
    let manta = v.page_for(MANTA_ID).unwrap().image.clone();
    let nomad = v.page_for(NOMAD_ID).unwrap().image.clone();
    let drew = manta != before && nomad != before_nomad;
    // The Block fill's gray (102; it was 68 until the grays sheet, 2026-11-12) is in the Manta's image.
    let has_block = manta.data.contains(&102);
    let picked = v
        .page_for(NOMAD_ID)
        .unwrap()
        .placed
        .iter()
        .any(|p| p.id == id);
    while v.undo() {}
    let same = v.page_for(MANTA_ID).unwrap().image == before
        && v.page_for(NOMAD_ID).unwrap().image == before_nomad;
    let ok = drew && has_block && picked && same;
    println!(
        "\nCHECK crate=ui name=shapes_both_devices result={} changes_both_pages={drew} block_gray_102_drawn={has_block} pickable_on_nomad={picked} undo_restores_bytes={same}",
        pass(ok)
    );
    assert!(ok);
}

// ---------------------------------------------------------------- the window

struct Window {
    ui: ViewerUi,
}

fn harness() -> Harness<'static, Window> {
    harness_with(page())
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
fn the_tools_bar_has_r_e_y_with_names_and_keys() {
    let mut h = harness();
    let named = ["Rectangle tool", "Ellipse tool", "Polygon tool"]
        .iter()
        .all(|l| h.query_by_label(l).is_some());
    let mut keys = true;
    for (k, t) in [
        (Key::R, Tool::Rect),
        (Key::E, Tool::Ellipse),
        (Key::Y, Tool::Polygon),
    ] {
        h.key_press(k);
        h.run();
        keys &= h.state().ui.viewer.tool == t;
        h.key_press(Key::V);
        h.run();
    }
    // Ctrl+Y is redo, not the polygon tool.
    h.key_press_modifiers(Modifiers::COMMAND, Key::Y);
    h.run();
    let ctrl_y_is_not_a_tool = h.state().ui.viewer.tool == Tool::Select;
    h.get_by_label("Ellipse tool").click();
    h.run();
    let by_click = h.state().ui.viewer.tool == Tool::Ellipse;
    let tips: Vec<String> = Tool::ALL
        .iter()
        .map(|t| quadrule_i18n::Strings::english().get(t.tip_id()))
        .collect();
    let tips_ok = tips[4].contains("(R)") && tips[5].contains("(E)") && tips[6].contains("(Y)");
    let ok = named && keys && ctrl_y_is_not_a_tool && by_click && tips_ok;
    println!(
        "\nCHECK crate=ui name=window_shape_tools result={} named_for_screen_readers={named} keys_r_e_y={keys} ctrl_y_not_a_tool={ctrl_y_is_not_a_tool} click_picks={by_click} tooltips_name_shortcuts={tips_ok}",
        pass(ok)
    );
    assert!(ok);
}

fn select_new(h: &mut Harness<'static, Window>, tool: Tool) -> Uuid {
    let v = &mut h.state_mut().ui.viewer;
    v.set_tool(tool);
    let id = v.place_click(300.0, 300.0, NONE, REACH).unwrap();
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    id
}

#[test]
fn the_style_tab_has_named_stroke_fill_dash_cap_and_shape_controls() {
    let mut h = harness();
    let rect = select_new(&mut h, Tool::Rect);
    let rect_labels = [
        "Stroke",
        "None",
        "Role",
        "Custom",
        "Fill",
        "Dash",
        "Cap",
        "Shape",
        "Corner radius",
    ]
    .iter()
    .all(|l| h.query_all_by_label(l).count() >= 1);
    // Typing a radius in rhythm units: 1u is one rhythm unit across (71 px = 6.01 mm).
    h.get_by_label("Corner radius").focus();
    h.run();
    h.key_combination_modifiers(Modifiers::COMMAND, &[Key::A]);
    h.run();
    h.get_by_label("Corner radius").type_text("1u");
    h.run();
    let radius = match &obj(&h.state().ui.viewer, rect).kind {
        ObjectKind::Rect(r) => r.corner_radius_mm,
        _ => -1.0,
    };
    let unit_mm = 71.0 * quadrule_model::MM_PER_IN / 300.0;
    let radius_in_units = (radius - unit_mm).abs() < 1e-6;
    // Stroke None through its radio button (a stroke-less shape needs a fill to be seen).
    h.get_by_label("None").click();
    h.run();
    let none = obj(&h.state().ui.viewer, rect).style == Style::None;
    let dash_hidden = h.query_by_label("Dash").is_none();
    // The polygon's own fields; ticking Star shows the inner radius.
    let poly = select_new(&mut h, Tool::Polygon);
    let poly_labels = ["Sides", "Star"]
        .iter()
        .all(|l| h.query_by_label(l).is_some())
        && h.query_by_label("Inner radius").is_none();
    h.get_by_label("Star").click();
    h.run();
    let star = matches!(&obj(&h.state().ui.viewer, poly).kind, ObjectKind::Polygon(p) if p.star == Some(0.5));
    let inner = h.query_by_label("Inner radius").is_some();
    let ellipse = select_new(&mut h, Tool::Ellipse);
    let ellipse_labels = h.query_by_label("Shape").is_some()
        && h.query_by_label("Corner radius").is_none()
        && h.query_by_label("Sides").is_none();
    let _ = ellipse;
    let ok = rect_labels
        && radius_in_units
        && none
        && dash_hidden
        && poly_labels
        && star
        && inner
        && ellipse_labels;
    println!(
        "\nCHECK crate=ui name=window_shape_inspector result={} rect_controls_named={rect_labels} radius_typed_in_u={radius_in_units} stroke_none={none} dash_hidden_without_stroke={dash_hidden} polygon_controls_named={poly_labels} star_ticks={star} inner_radius_shows={inner} ellipse_has_no_radius_or_sides={ellipse_labels}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn fill_and_dash_and_cap_are_set_from_the_inspector() {
    let mut h = harness();
    let id = select_new(&mut h, Tool::Rect);
    // The combos open on a click and pick on a click.
    let pick = |h: &mut Harness<'static, Window>, combo: &str, item: &str| {
        h.get_by_label(combo).click();
        h.run();
        h.get_all_by_label(item).last().unwrap().click();
        h.run();
        h.run();
    };
    pick(&mut h, "Fill", "Shade");
    let shade = obj(&h.state().ui.viewer, id).fill == Some(Fill::Role(FillRole::Shade));
    pick(&mut h, "Fill", "Custom gray");
    let custom = matches!(obj(&h.state().ui.viewer, id).fill, Some(Fill::Gray(g)) if g % 17 == 0);
    let slider = h.query_all_by_label("Fill gray").count() >= 1;
    pick(&mut h, "Dash", "Long dash");
    let dash = obj(&h.state().ui.viewer, id).dash == DashStyle::Long;
    pick(&mut h, "Dash", "Custom");
    let custom_dash = matches!(obj(&h.state().ui.viewer, id).dash, DashStyle::Custom { dash_mm, gap_mm } if dash_mm == 2.0 && gap_mm == 1.0);
    let lengths =
        h.query_by_label("Dash length").is_some() && h.query_by_label("Gap length").is_some();
    pick(&mut h, "Cap", "Round");
    let cap = obj(&h.state().ui.viewer, id).cap == LineCap::Round;
    // Each is one undoable step, and undo puts the old value back.
    let steps_before = h
        .state()
        .ui
        .viewer
        .open
        .as_ref()
        .unwrap()
        .history
        .undo_len();
    h.state_mut().ui.viewer.undo();
    let undo_cap = obj(&h.state().ui.viewer, id).cap == LineCap::Butt;
    let ok = shade
        && custom
        && slider
        && dash
        && custom_dash
        && lengths
        && cap
        && undo_cap
        && steps_before >= 6;
    println!(
        "\nCHECK crate=ui name=window_fill_dash_cap result={} shade={shade} custom_gray_step_17={custom} gray_slider_named={slider} long_dash={dash} custom_dash_starts_from_preset={custom_dash} length_fields_named={lengths} round_cap={cap} undo_cap={undo_cap}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_benchmarks_run_over_the_shapes_page_and_draw_each_new_shape() {
    // `quadrule --shapes-page --bench-drag` and `--bench-draw --bench-tool rect|ellipse|polygon`
    // (rung 6a step 7): headless they finish, time their frames and leave the shapes page as it
    // was (the timing itself is judged in a release build on the Pi, ADR-009).
    let want = quadrule_model::sample::shapes_document();
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 4] = [
        ("drag", quadrule_ui::DragBench::new(12).with_shapes()),
        (
            "draw_rect",
            quadrule_ui::DragBench::draw(12)
                .with_shapes()
                .with_tool(Tool::Rect),
        ),
        (
            "draw_ellipse",
            quadrule_ui::DragBench::draw(12)
                .with_shapes()
                .with_tool(Tool::Ellipse),
        ),
        (
            "draw_polygon",
            quadrule_ui::DragBench::draw(12)
                .with_shapes()
                .with_tool(Tool::Polygon),
        ),
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
        "\nCHECK crate=ui name=window_shapes_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
