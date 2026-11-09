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

//! Text checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::outlines::GlyphShape;
use quadrule_model::sample::{ATKINSON, TITLE_TEXT};
use quadrule_model::{Extra, TextAlign, TextBox, Uuid};
use quadrule_text::*;
use sha2::{Digest, Sha256};

fn text_box(content: &str, weight: u16) -> TextBox {
    TextBox {
        content: content.into(),
        family: ATKINSON.into(),
        weight,
        size_pt: 12.0,
        line_height: 1.25,
        letter_spacing_em: 0.0,
        align: TextAlign::Left,
        extra: Extra::new(),
    }
}

fn lay(tb: &TextBox, width_mm: f64) -> Laid {
    let mut reg = FontRegistry::bundled_only();
    let req = TextRequest {
        object_id: Uuid::from_u128(1),
        text: tb,
        width_mm,
    };
    lay_out(&mut reg, "test", &[req]).expect("lay out")
}

fn lines(l: &Laid) -> usize {
    let t = &l.outlines.texts[0];
    let mut ys: Vec<f32> = t.glyphs.iter().map(|g| g.y_mm()).collect();
    ys.dedup();
    ys.len()
}

#[test]
fn bundled_font_is_the_recorded_file() {
    let mut fails = Vec::new();
    for f in BUNDLED {
        let got: String = Sha256::digest(f.data)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if got != f.sha256 {
            fails.push(f.id);
        }
    }
    let version = BUNDLED[0].version().unwrap_or_default();
    let ok = fails.is_empty() && !version.is_empty();
    println!(
        "CHECK crate=text name=bundled_font_sha256 result={} fonts={} version={version:?} fails={fails:?}",
        if ok { "pass" } else { "fail" },
        BUNDLED.len()
    );
    assert!(ok);
}

#[test]
fn title_lays_out_with_no_missing_glyphs() {
    let l = lay(&text_box(TITLE_TEXT, 700), 146.56);
    let t = &l.outlines.texts[0];
    let shapes_ok = l
        .outlines
        .tables
        .iter()
        .all(|tb| tb.glyphs.values().all(GlyphShape::is_well_formed));
    let ok =
        l.missing_glyphs == 0 && !t.glyphs.is_empty() && shapes_ok && l.outlines.tables.len() == 1;
    println!(
        "CHECK crate=text name=title_no_missing_glyphs result={} glyphs={} distinct={} missing={} rounded_coords={}",
        if ok { "pass" } else { "fail" },
        t.glyphs.len(),
        l.outlines.tables[0].glyphs.len(),
        l.missing_glyphs,
        l.rounded_coords
    );
    assert!(ok);
}

#[test]
fn regular_weight_shapes_are_exact() {
    // At the font's default weight, TrueType points are whole or half font units, so storing
    // half units loses nothing (ADR-001 item 1).
    let l = lay(&text_box(TITLE_TEXT, 400), 146.56);
    let ok = l.rounded_coords == 0;
    println!(
        "CHECK crate=text name=regular_weight_exact result={} rounded_coords={}",
        if ok { "pass" } else { "fail" },
        l.rounded_coords
    );
    assert!(ok);
}

#[test]
fn bold_draws_bolder_shapes_and_wider_text() {
    // Weight drives the variable font's weight axis in layout (wider advances) and in the
    // outlines (different shapes for the same glyph).
    let regular = lay(&text_box("Hamburgefonstiv", 400), 200.0);
    let bold = lay(&text_box("Hamburgefonstiv", 700), 200.0);
    let last_x = |l: &Laid| l.outlines.texts[0].glyphs.last().unwrap().x_mm();
    let (r, b) = (&regular.outlines.tables[0], &bold.outlines.tables[0]);
    let differ = r
        .glyphs
        .iter()
        .filter(|(id, s)| b.glyphs.get(id).is_some_and(|bs| bs != *s))
        .count();
    let ok = last_x(&bold) > last_x(&regular) && differ == r.glyphs.len() && b.face.weight == 700;
    println!(
        "CHECK crate=text name=bold_differs result={} last_x_400={:.3} last_x_700={:.3} shapes_differ={differ}/{} rounded_700={}",
        if ok { "pass" } else { "fail" },
        last_x(&regular),
        last_x(&bold),
        r.glyphs.len(),
        bold.rounded_coords
    );
    assert!(ok);
}

#[test]
fn narrower_box_wraps_more() {
    // The title wraps differently on the Manta (146.56 mm box) and the Nomad (102.87 mm box).
    let wide = lines(&lay(&text_box(TITLE_TEXT, 700), 146.56));
    let narrow = lines(&lay(&text_box(TITLE_TEXT, 700), 102.872));
    let ok = wide >= 1 && narrow > wide;
    println!(
        "CHECK crate=text name=narrower_box_wraps_more result={} lines_wide={wide} lines_narrow={narrow}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn height_is_lines_times_line_height() {
    let l = lay(&text_box("one\ntwo\nthree", 400), 100.0);
    let t = &l.outlines.texts[0];
    let want = 3.0 * t.size_mm * 1.25;
    let ok = (t.height_mm - want).abs() < 1e-4 && lines(&l) == 3;
    println!(
        "CHECK crate=text name=height_lines_x_line_height result={} height_mm={:.4} want_mm={want:.4}",
        if ok { "pass" } else { "fail" },
        t.height_mm
    );
    assert!(ok);
}

#[test]
fn kerning_moves_letters() {
    // "To" is a kerning pair: the o sits closer to the T than in "Tx" when both start at 0.
    let to = lay(&text_box("To", 400), 100.0);
    let tx = lay(&text_box("Tx", 400), 100.0);
    let (o, x) = (
        to.outlines.texts[0].glyphs[1].x_mm(),
        tx.outlines.texts[0].glyphs[1].x_mm(),
    );
    let ok = o < x;
    println!(
        "CHECK crate=text name=kerning_in_layout result={} o_x_mm={o:.4} x_x_mm={x:.4}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn layout_is_repeatable() {
    let a = lay(&text_box(TITLE_TEXT, 700), 146.56);
    let b = lay(&text_box(TITLE_TEXT, 700), 146.56);
    let ok = a == b;
    println!(
        "CHECK crate=text name=layout_repeatable result={}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn missing_family_is_an_error() {
    let mut tb = text_box("x", 400);
    tb.family = "No Such Family".into();
    let mut reg = FontRegistry::bundled_only();
    let r = lay_out(
        &mut reg,
        "test",
        &[TextRequest {
            object_id: Uuid::from_u128(1),
            text: &tb,
            width_mm: 50.0,
        }],
    );
    let ok = matches!(r, Err(TextError::MissingFamily(_)));
    println!(
        "CHECK crate=text name=missing_family_error result={}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
