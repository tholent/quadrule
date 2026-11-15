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

//! The font picker and the Text tab's font, weight and style fields (rung 11a,
//! docs/bbq/2026-11-14-rung-11-text-and-fonts.md), driven headless. Each test prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::ids;
use quadrule_model::{FontStyle, ObjectKind};
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

/// The sample page with its title picked and the Text tab showing.
fn harness() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    v.selection = vec![ids::TITLE];
    let mut ui = ViewerUi::new(v);
    ui.tab = Tab::Text;
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
    h
}

fn title(h: &Harness<'static, Window>) -> (String, FontStyle, u16) {
    let doc = h.state().ui.viewer.doc().unwrap();
    match &doc.object(ids::TITLE).unwrap().kind {
        ObjectKind::Text(t) => (t.family.clone(), t.style, t.weight),
        _ => panic!("the title is text"),
    }
}

#[test]
fn the_picker_lists_bundled_then_system_and_a_pick_is_one_undo_step() {
    let mut h = harness();
    let before = title(&h);
    h.get_by_label("Font").click();
    h.run();
    let opened = h.query_by_label("Search fonts").is_some()
        && h.query_by_label("Bundled").is_some()
        && h.query_by_label("Inter").is_some()
        && h.query_by_label("Noto Emoji").is_some();
    h.get_by_label("Inter").click();
    h.run();
    let after = title(&h);
    let picked = after.0 == "Inter" && before.0 == "Atkinson Hyperlegible Next";
    let closed = h.query_by_label("Search fonts").is_none();
    // One undo step puts it back.
    h.state_mut().ui.viewer.undo();
    h.run();
    let undone = title(&h) == before;
    let ok = opened && picked && closed && undone;
    println!(
        "CHECK crate=ui name=font_picker result={} opened={opened} picked={picked} closed={closed} one_undo={undone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_search_box_filters_the_list() {
    let mut h = harness();
    h.get_by_label("Font").click();
    h.run();
    h.get_by_label("Search fonts").click();
    h.get_by_label("Search fonts").type_text("lit");
    h.run();
    let only_literata = h.query_by_label("Literata").is_some()
        && h.query_by_label("Inter").is_none()
        && h.query_by_label("Caveat").is_none();
    println!(
        "CHECK crate=ui name=font_picker_search result={} only_literata={only_literata}",
        pass(only_literata)
    );
    assert!(only_literata);
}

#[test]
fn a_family_without_italic_greys_the_style_and_says_why() {
    let mut h = harness();
    let italic_ok = h
        .query_by_label("This font has no italic, so the text stays upright.")
        .is_none();
    // Caveat has no italic face.
    {
        let v = &mut h.state_mut().ui.viewer;
        let doc = v.doc().unwrap().clone();
        let mut o = doc.object(ids::TITLE).unwrap().clone();
        if let ObjectKind::Text(t) = &mut o.kind {
            t.family = "Caveat".into();
        }
        v.edit(quadrule_model::edit::Command::SetObject(o), None)
            .unwrap();
    }
    h.run();
    let note = h
        .query_by_label("This font has no italic, so the text stays upright.")
        .is_some();
    // A weight outside Caveat's own range (400 to 700) is said to be drawn at the nearest.
    {
        let v = &mut h.state_mut().ui.viewer;
        let doc = v.doc().unwrap().clone();
        let mut o = doc.object(ids::TITLE).unwrap().clone();
        if let ObjectKind::Text(t) = &mut o.kind {
            t.weight = 900;
        }
        v.edit(quadrule_model::edit::Command::SetObject(o), None)
            .unwrap();
    }
    h.run();
    let clamped = h
        .query_by_label("This font offers weights 400 to 700; the text is drawn at 700.")
        .is_some();
    let ok = italic_ok && note && clamped;
    println!(
        "CHECK crate=ui name=style_and_weight_notes result={} upright_note_for_caveat={note} weight_note={clamped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_rescan_lays_pages_out_again_only_when_the_list_changed() {
    let mut v = Viewer::default();
    v.open_sample();
    v.use_system_fonts();
    let _ = v.page().map(|p| p.image.width);
    let first = v.rescan_fonts();
    let second = v.rescan_fonts();
    let ok = !first.changed && !second.changed && v.last_font_scan == Some(second);
    println!(
        "CHECK crate=ui name=font_rescan result={} faces={} families={} ms={:.1}",
        pass(ok),
        second.faces,
        second.families,
        second.elapsed.as_secs_f64() * 1000.0
    );
    assert!(ok);
}

#[test]
fn names_and_preview_are_drawn_in_their_own_fonts_as_export_pixels() {
    use quadrule_model::{Extra, TextAlign, TextBox};
    use quadrule_text::FontRegistry;
    use quadrule_ui::fonts::raster_text;
    let mut reg = FontRegistry::bundled_only();
    let px_per_mm = 300.0 / quadrule_model::MM_PER_IN;
    let make = |family: &str, text: &str, size_pt: f64| TextBox {
        content: text.into(),
        family: family.into(),
        weight: 400,
        style: FontStyle::Normal,
        size_pt,
        line_height: 1.25,
        letter_spacing_em: 0.0,
        align: TextAlign::Left,
        extra: Extra::new(),
    };
    let mut rasters = Vec::new();
    for f in reg.families() {
        let r = raster_text(&mut reg, &make(&f.name, &f.name, 11.0), px_per_mm);
        rasters.push((f.name, r));
    }
    let inked = rasters
        .iter()
        .all(|(_, r)| r.as_ref().is_some_and(|r| r.gray.iter().any(|&g| g < 128)));
    // The same text in two families gives two different pictures.
    let a = raster_text(&mut reg, &make("Inter", "Hamburg", 12.0), px_per_mm).unwrap();
    let b = raster_text(&mut reg, &make("Literata", "Hamburg", 12.0), px_per_mm).unwrap();
    let differ = a.gray != b.gray;
    // Twelve points at 300 pixels to the inch is 50 pixels to the em: the line is about that tall.
    let tall = (45..=90).contains(&a.height);
    // Nothing to draw gives no picture.
    let blank = raster_text(&mut reg, &make("Inter", "   ", 12.0), px_per_mm).is_none();
    let ok = inked && differ && tall && blank;
    println!(
        "CHECK crate=ui name=font_pictures result={} families={} inked={inked} families_differ={differ} height_px={} blank_has_no_picture={blank}",
        pass(ok),
        rasters.len(),
        a.height
    );
    assert!(ok);
    // For looking at by eye: every family's name above Inter and Literata at 12 pt.
    let mut rows: Vec<quadrule_ui::fonts::Raster> =
        rasters.into_iter().filter_map(|(_, r)| r).collect();
    rows.push(a);
    rows.push(b);
    let w = rows.iter().map(|r| r.width).max().unwrap();
    let h: u32 = rows.iter().map(|r| r.height).sum();
    let mut gray = vec![255u8; (w * h) as usize];
    let mut y0 = 0;
    for r in &rows {
        for y in 0..r.height {
            let d = ((y0 + y) * w) as usize;
            gray[d..d + r.width as usize]
                .copy_from_slice(&r.gray[(y * r.width) as usize..((y + 1) * r.width) as usize]);
        }
        y0 += r.height;
    }
    let png = quadrule_render::encode_png(
        &quadrule_render::Gray {
            width: w,
            height: h,
            data: gray,
        },
        None,
    );
    let _ = std::fs::write(
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("font-names.png"),
        png,
    );
}
