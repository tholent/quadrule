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

//! The Style tab's gray swatches, the "matches <Role>" badge and Use recommended driven with the pointer (rung
//! 14b, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q8). Each swatch has an accessible name, "Gray 51" or
//! "Fill gray 51". Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn mm(px: f64) -> f64 {
    px * 25.4 / 300.0
}

fn obj(id: u128, name: &str, style: Style, fill: Option<Fill>, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(id),
        name: name.into(),
        frame: Frame::new(mm(300.0), mm(300.0 + 100.0 * id as f64), 40.0, 10.0),
        constraints: Constraints::default(),
        visible: true,
        locked: false,
        style,
        fill,
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

fn custom(px: f64, gray: u8) -> Style {
    Style::Custom {
        width_mm: mm(px),
        gray,
    }
}

fn doc(objects: Vec<Object>) -> Document {
    let mut d = slice_document();
    d.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    d
}

fn style_tab(d: Document, select: u128) -> Harness<'static, ViewerUi> {
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_document_for_test(d);
    ui.viewer.select_row(Uuid::from_u128(select), false);
    ui.tab = Tab::Style;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    h
}

fn style_of(h: &Harness<'static, ViewerUi>, id: u128) -> Style {
    h.state()
        .viewer
        .doc()
        .unwrap()
        .object(Uuid::from_u128(id))
        .unwrap()
        .style
}

fn fill_of(h: &Harness<'static, ViewerUi>, id: u128) -> Option<Fill> {
    h.state()
        .viewer
        .doc()
        .unwrap()
        .object(Uuid::from_u128(id))
        .unwrap()
        .fill
}

fn steps(h: &Harness<'static, ViewerUi>) -> usize {
    h.state().viewer.open.as_ref().unwrap().history.undo_len()
}

fn swatch_count(h: &Harness<'static, ViewerUi>, prefix: &str) -> usize {
    (0..=255u32)
        .filter(|g| h.query_by_label(&format!("{prefix} {g}")).is_some())
        .count()
}

fn sliders(h: &Harness<'static, ViewerUi>) -> usize {
    h.query_all_by_role(egui::accesskit::Role::Slider).count()
}

#[test]
fn custom_gray_is_the_profiles_clean_levels_as_swatches_with_a_slider_behind_a_tick() {
    let line = obj(
        1,
        "line",
        custom(1.0, 51),
        None,
        ObjectKind::Line(Line::default()),
    );
    let mut h = style_tab(doc(vec![line]), 1);
    // Sixteen swatches (the Manta lists sixteen), each named by its gray; no slider at first.
    let sixteen = swatch_count(&h, "Gray") == 16;
    let named = [0, 51, 119, 255]
        .iter()
        .all(|g| h.query_by_label(&format!("Gray {g}")).is_some());
    let no_slider = sliders(&h) == 0;
    // The chosen one is marked (toggled) and no other is.
    let chosen = on(&h, 51);
    let only = (0..=255u32).filter(|g| *g != 51).all(|g| !on(&h, g));
    // A click on a swatch sets the gray in one undo step.
    let n0 = steps(&h);
    h.get_by_label("Gray 119").click();
    h.run();
    let set = style_of(&h, 1) == custom(1.0, 119) && steps(&h) == n0 + 1;
    let moved = on(&h, 119);
    // Show all grays opens the slider; ticking again closes it.
    h.get_by_label("Show all grays").click();
    h.run();
    let opened = sliders(&h) == 1 && h.state().all_grays_open("stroke");
    h.get_by_label("Show all grays").click();
    h.run();
    let closed = sliders(&h) == 0;
    // A gray that is not a clean level shows the slider by itself and outlines no swatch.
    h.state_mut()
        .viewer
        .edit_selected(None, |x| x.style = custom(1.0, 120))
        .unwrap();
    h.run();
    let off_level = sliders(&h) == 1 && (0..=255u32).all(|g| !on(&h, g));
    let ok = sixteen
        && named
        && no_slider
        && chosen
        && only
        && set
        && moved
        && opened
        && closed
        && off_level;
    println!(
        "CHECK crate=ui name=swatches_pointer_stroke result={} sixteen_named={} slider_hidden_at_first={no_slider} chosen_outlined={} click_sets_in_one_step={set} show_all_opens_and_closes={} off_level_forces_slider={off_level}",
        pass(ok),
        sixteen && named,
        chosen && only && moved,
        opened && closed
    );
    assert!(ok);
}

/// Whether the swatch for gray `g` is marked as the chosen one.
fn on(h: &Harness<'static, ViewerUi>, g: u32) -> bool {
    h.query_by_label(&format!("Gray {g}"))
        .is_some_and(|n| n.accesskit_node().toggled() == Some(egui::accesskit::Toggled::True))
}

#[test]
fn fill_and_text_grays_get_the_same_swatches_and_paper_gets_the_slider_only() {
    let rect = obj(
        1,
        "box",
        Style::None,
        Some(Fill::Gray(204)),
        ObjectKind::Rect(RectShape::default()),
    );
    let words = obj(
        2,
        "words",
        custom(1.0, 68),
        None,
        ObjectKind::Text(TextBox {
            content: "words".into(),
            family: ATKINSON.into(),
            size_pt: 12.0,
            ..TextBox::default()
        }),
    );
    let mut h = style_tab(doc(vec![rect, words]), 1);
    let fill_sixteen = swatch_count(&h, "Fill gray") == 16;
    let n0 = steps(&h);
    h.get_by_label("Fill gray 85").click();
    h.run();
    let fill_set = fill_of(&h, 1) == Some(Fill::Gray(85)) && steps(&h) == n0 + 1;
    // The text box: its gray is a stroke gray field.
    h.state_mut().viewer.select_row(Uuid::from_u128(2), false);
    h.run();
    let text_sixteen = swatch_count(&h, "Gray") == 16;
    h.get_by_label("Gray 34").click();
    h.run();
    let text_set = style_of(&h, 2) == custom(1.0, 34);
    // Paper has no gray levels: the slider only, no swatch and no tick.
    let mut paper = paper_document();
    let id = paper
        .objects()
        .find(|o| o.kind.is_shape() || matches!(o.kind, ObjectKind::Line(_)))
        .map(|o| o.id)
        .unwrap();
    paper.object_mut(id).unwrap().style = custom(2.0, 100);
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_document_for_test(paper);
    ui.viewer.select_row(id, false);
    ui.tab = Tab::Style;
    let mut p = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    p.run();
    let paper_only_slider = swatch_count(&p, "Gray") == 0
        && sliders(&p) == 1
        && p.query_by_label("Show all grays").is_none();
    let ok = fill_sixteen && fill_set && text_sixteen && text_set && paper_only_slider;
    println!(
        "CHECK crate=ui name=swatches_pointer_fill_text_paper result={} fill_swatches={fill_sixteen} fill_click_sets={fill_set} text_swatches={text_sixteen} text_click_sets={text_set} paper_slider_only={paper_only_slider}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_badge_names_the_role_a_stroke_matches_and_use_recommended_sets_the_nearest() {
    let line = obj(
        1,
        "line",
        custom(1.0, 51),
        None,
        ObjectKind::Line(Line::default()),
    );
    let mut h = style_tab(doc(vec![line]), 1);
    // 1 px at gray 51 is exactly Guide on the Manta.
    let guide = h.query_by_label("matches Guide").is_some()
        && h.query_by_label("Use recommended").is_none();
    // 1 px at 119 is Whisper.
    h.state_mut()
        .viewer
        .edit_selected(None, |x| x.style = custom(1.0, 119))
        .unwrap();
    h.run();
    let whisper = h.query_by_label("matches Whisper").is_some();
    // 3 px at gray 10 matches no role: Use recommended, and the nearest by width is Emphasis (3 px).
    h.state_mut()
        .viewer
        .edit_selected(None, |x| x.style = custom(3.0, 10))
        .unwrap();
    h.run();
    let none = h.query_by_label_contains("matches").is_none()
        && h.query_by_label("Use recommended").is_some();
    let n0 = steps(&h);
    h.get_by_label("Use recommended").click();
    h.run();
    let set = style_of(&h, 1) == Style::Role(LineRole::Emphasis) && steps(&h) == n0 + 1;
    h.state_mut().viewer.undo();
    let undone = style_of(&h, 1) == custom(3.0, 10);
    // A pale 1 px line: the tie between Whisper and Guide goes to Guide.
    h.state_mut()
        .viewer
        .edit_selected(None, |x| x.style = custom(1.0, 230))
        .unwrap();
    h.run();
    h.get_by_label("Use recommended").click();
    h.run();
    let tie = style_of(&h, 1) == Style::Role(LineRole::Guide);
    let ok = guide && whisper && none && set && undone && tie;
    println!(
        "CHECK crate=ui name=swatches_pointer_badge result={} matches_guide={guide} matches_whisper={whisper} no_match_offers_button={none} use_recommended_one_step={set} undo={undone} tie_goes_to_guide={tie}",
        pass(ok)
    );
    assert!(ok);
}
