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
    use quadrule_model::{TextAlign, TextBox};
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
        ..TextBox::default()
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

// ---------------------------------------------------------------- the emoji picker

#[test]
fn the_unicode_notice_ships_beside_the_font_licences_and_is_credited() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let notice = std::fs::read_to_string(root.join("licenses/Unicode-3.0.txt")).unwrap_or_default();
    let packager =
        std::fs::read_to_string(root.join("../../packaging/packager.json")).unwrap_or_default();
    let readme = std::fs::read_to_string(root.join("../../README.md")).unwrap_or_default();
    let ok = notice.contains("UNICODE LICENSE V3")
        && packager.contains("\"../crates/ui/licenses/Unicode-3.0.txt\"")
        && packager.contains("\"licenses/Unicode-3.0.txt\"")
        && readme.contains("Unicode-3.0.txt");
    println!(
        "CHECK crate=ui name=unicode_notice_shipped result={} notice={} packaged={} credited={}",
        pass(ok),
        notice.contains("UNICODE LICENSE V3"),
        packager.contains("licenses/Unicode-3.0.txt"),
        readme.contains("Unicode-3.0.txt")
    );
    assert!(ok);
}

#[test]
fn the_picker_lists_emoji_noto_can_draw_in_the_nine_groups() {
    use quadrule_text::FontRegistry;
    use quadrule_ui::emoji::{GROUPS, drawable};
    let t = std::time::Instant::now();
    let list = drawable(&mut FontRegistry::bundled_only());
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let per: Vec<usize> = GROUPS
        .iter()
        .map(|(g, _)| list.iter().filter(|e| e.group() == *g).count())
        .collect();
    let all = emojis::iter()
        .filter(|e| e.skin_tone().is_none_or(|t| t == emojis::SkinTone::Default))
        .count();
    let grin = list.iter().any(|e| e.as_str() == "\u{1F600}");
    let ok = list.len() > 500 && per.iter().filter(|n| **n > 0).count() >= 8 && grin;
    println!(
        "CHECK crate=ui name=emoji_list result={} drawable={} of={all} per_group={per:?} grinning_face={grin} ms={ms:.0} build={}",
        pass(ok),
        list.len(),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    assert!(ok);
}

fn emoji_harness(select_title: bool) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    if select_title {
        v.selection = vec![ids::TITLE];
    }
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

fn count_objects(h: &Harness<'static, Window>) -> usize {
    h.state().ui.viewer.doc().unwrap().objects().count()
}

#[test]
fn an_emoji_goes_into_the_picked_text_box_and_with_none_picked_into_a_new_24_pt_box() {
    // In the picked box: the emoji is added to its text (the field's cursor is at the end when it was
    // never clicked), as one undo step.
    let mut h = emoji_harness(true);
    let before = match &h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(ids::TITLE)
        .unwrap()
        .kind
    {
        ObjectKind::Text(t) => t.content.clone(),
        _ => unreachable!(),
    };
    h.get_all_by_label("Emoji").next().unwrap().click();
    h.run();
    let opened =
        h.query_by_label("Search emoji by name").is_some() && h.query_by_label("Smileys").is_some();
    h.get_by_label("Search emoji by name").click();
    h.get_by_label("Search emoji by name")
        .type_text("grinning face");
    h.run();
    h.get_by_label("grinning face").click();
    h.run();
    let after = match &h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(ids::TITLE)
        .unwrap()
        .kind
    {
        ObjectKind::Text(t) => t.content.clone(),
        _ => unreachable!(),
    };
    let inserted = after == format!("{before}\u{1F600}");
    h.state_mut().ui.viewer.undo();
    h.run();
    let undone = match &h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(ids::TITLE)
        .unwrap()
        .kind
    {
        ObjectKind::Text(t) => t.content == before,
        _ => false,
    };

    // With nothing picked: a new box holding only the emoji, 24 pt, one undo step.
    let mut h = emoji_harness(false);
    let n = count_objects(&h);
    h.get_all_by_label("Emoji").next().unwrap().click();
    h.run();
    h.get_by_label("Search emoji by name").click();
    h.get_by_label("Search emoji by name")
        .type_text("grinning face");
    h.run();
    h.get_by_label("grinning face").click();
    h.run();
    let doc = h.state().ui.viewer.doc().unwrap().clone();
    let sel = h.state().ui.viewer.selection.clone();
    let new = sel.first().and_then(|id| doc.object(*id)).cloned();
    let (content, pt) = match new.as_ref().map(|o| &o.kind) {
        Some(ObjectKind::Text(t)) => (t.content.clone(), t.size_pt),
        _ => (String::new(), 0.0),
    };
    let made = count_objects(&h) == n + 1 && content == "\u{1F600}" && pt == 24.0;
    // In the middle of the page (the page is shown whole), give or take the box's own size.
    let middle = new.as_ref().is_some_and(|o| {
        let (cx, cy) = (
            o.frame.x_mm + o.frame.w_mm / 2.0,
            o.frame.y_mm + o.frame.h_mm / 2.0,
        );
        let (pw, ph) = (
            quadrule_model::sample::MANTA_W_MM,
            quadrule_model::sample::MANTA_H_MM,
        );
        (cx - pw / 2.0).abs() < pw * 0.15 && (cy - ph / 2.0).abs() < ph * 0.15
    });
    h.state_mut().ui.viewer.undo();
    h.run();
    let one_step = count_objects(&h) == n;
    let ok = opened && inserted && undone && made && middle && one_step;
    println!(
        "CHECK crate=ui name=emoji_insert result={} opened={opened} into_box={inserted} undone={undone} new_box={made} in_the_middle={middle} one_undo={one_step}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_emoji_goes_at_the_cursor_of_the_text_field_and_replaces_what_is_selected() {
    let mut h = emoji_harness(true);
    let ctx = h.ctx.clone();
    let id = h.state().ui.text_field_id();
    let content = |h: &Harness<'static, Window>| match &h
        .state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(ids::TITLE)
        .unwrap()
        .kind
    {
        ObjectKind::Text(t) => t.content.clone(),
        _ => unreachable!(),
    };
    let before = content(&h);
    let set = |from: usize, to: usize| {
        let id = id.expect("the Text tab shows its field");
        let mut state = egui::TextEdit::load_state(&ctx, id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(from),
                egui::text::CCursor::new(to),
            )));
        state.store(&ctx, id);
    };
    // A cursor after the third letter.
    set(3, 3);
    h.state_mut().ui.insert_emoji(&ctx, "\u{1F600}");
    h.run();
    let chars: Vec<char> = before.chars().collect();
    let want_mid: String = chars[..3].iter().collect::<String>()
        + "\u{1F600}"
        + &chars[3..].iter().collect::<String>();
    let mid = content(&h) == want_mid;
    // A selection of letters 1 to 2 is replaced.
    h.state_mut().ui.viewer.undo();
    h.run();
    set(1, 2);
    h.state_mut().ui.insert_emoji(&ctx, "\u{2764}");
    h.run();
    let want_sel: String =
        chars[..1].iter().collect::<String>() + "\u{2764}" + &chars[2..].iter().collect::<String>();
    let replaced = content(&h) == want_sel;
    let ok = mid && replaced;
    println!(
        "CHECK crate=ui name=emoji_at_cursor result={} at_cursor={mid} replaces_selection={replaced}",
        pass(ok)
    );
    assert!(ok);
}
