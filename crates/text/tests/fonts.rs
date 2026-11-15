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

//! Rung 11a's font checks (docs/bbq/2026-11-14-rung-11-text-and-fonts.md): the bundled set and its
//! licences, optical size, italics, the bundled-only fallback, weight clamping and the system
//! fonts. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use cosmic_text::skrifa::{FontRef, MetadataProvider, Tag};
use quadrule_model::{Extra, FontStyle, TextAlign, TextBox, Uuid};
use quadrule_text::opsz::{pin_opsz, wght_range};
use quadrule_text::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn text_box(family: &str, content: &str, weight: u16, style: FontStyle, size_pt: f64) -> TextBox {
    TextBox {
        content: content.into(),
        family: family.into(),
        weight,
        style,
        size_pt,
        line_height: 1.25,
        letter_spacing_em: 0.0,
        align: TextAlign::Left,
        extra: Extra::new(),
    }
}

fn lay_in(reg: &mut FontRegistry, tb: &TextBox) -> Laid {
    let req = TextRequest {
        object_id: Uuid::from_u128(1),
        text: tb,
        width_mm: 5000.0,
    };
    lay_out(reg, "test", &[req]).expect("lay out")
}

fn last_x_per_em(l: &Laid) -> f32 {
    let t = &l.outlines.texts[0];
    t.glyphs.last().unwrap().x_mm() / t.size_mm
}

#[test]
fn every_bundled_font_has_its_licence_in_the_font_folder_and_the_packages() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts");
    let packager = std::fs::read_to_string(dir.join("../../../packaging/packager.json"))
        .expect("packager.json");
    let mut bad = Vec::new();
    for f in BUNDLED {
        let text = std::fs::read_to_string(dir.join(f.ofl)).unwrap_or_default();
        let is_ofl = text.contains("SIL OPEN FONT LICENSE Version 1.1");
        let packed = packager.contains(&format!("\"../crates/text/fonts/{}\"", f.ofl))
            && packager.contains(&format!("\"licenses/{}\"", f.ofl));
        if !(is_ofl && packed && dir.join(f.file).exists()) {
            bad.push(f.id);
        }
    }
    let ok = bad.is_empty();
    println!(
        "CHECK crate=text name=bundled_licences result={} fonts={} bad={bad:?}",
        pass(ok),
        BUNDLED.len()
    );
    assert!(ok);
}

#[test]
fn fallback_order_names_every_bundled_family_atkinson_then_emoji_first() {
    let mut families: Vec<&str> = BUNDLED.iter().map(|b| b.family).collect();
    families.sort();
    families.dedup();
    let mut order = FALLBACK_ORDER.to_vec();
    order.sort();
    let ok = order == families
        && FALLBACK_ORDER[0] == "Atkinson Hyperlegible Next"
        && FALLBACK_ORDER[1] == "Noto Emoji";
    println!(
        "CHECK crate=text name=fallback_order result={} families={}",
        pass(ok),
        families.len()
    );
    assert!(ok);
}

#[test]
fn a_copy_at_an_optical_size_gives_the_coordinates_the_font_gives_when_asked_by_name() {
    let (opsz, wght) = (Tag::new(b"opsz"), Tag::new(b"wght"));
    let mut cases = 0;
    let mut fails = Vec::new();
    for b in BUNDLED.iter().filter(|b| b.file.contains("opsz")) {
        let orig = FontRef::new(b.data).unwrap();
        for size in [1.0f32, 7.0, 12.0, 14.0, 17.5, 24.0, 32.0, 72.0, 200.0] {
            for w in [200.0f32, 400.0, 700.0, 900.0] {
                let patched = pin_opsz(b.data, size).expect("patched");
                let p = FontRef::new(&patched).unwrap();
                let want = orig.axes().location([(opsz, size), (wght, w)]);
                let got = p.axes().location([(wght, w)]);
                cases += 1;
                if want.coords() != got.coords() {
                    fails.push(format!("{}@{size}/{w}", b.id));
                }
            }
        }
    }
    let ok = cases > 0 && fails.is_empty();
    println!(
        "CHECK crate=text name=opsz_copy_matches result={} cases={cases} fails={fails:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn optical_size_follows_the_point_size_and_is_clamped() {
    let mut reg = FontRegistry::bundled_only();
    let text = "Hamburgefonstiv rag";
    let mut widths = Vec::new();
    let mut keys = Vec::new();
    for pt in [6.0, 14.0, 32.0, 100.0] {
        let l = lay_in(
            &mut reg,
            &text_box("Inter", text, 400, FontStyle::Normal, pt),
        );
        widths.push(last_x_per_em(&l));
        keys.push(l.outlines.tables[0].face.opsz);
    }
    // Inter's axis runs 14 to 32: small sizes sit at 14, big ones at 32; in between it is the size.
    let same = |a: f32, b: f32| (a - b).abs() < 1e-3;
    let ok = keys == [14, 14, 32, 32]
        && same(widths[0], widths[1])
        && widths[2] < widths[1] - 0.1
        && same(widths[3], widths[2]);
    // Literata runs 7 to 72 with 12 as its default.
    let mut lit = Vec::new();
    for pt in [4.0, 12.0, 30.0, 90.0] {
        let l = lay_in(
            &mut reg,
            &text_box("Literata", text, 400, FontStyle::Normal, pt),
        );
        lit.push(l.outlines.tables[0].face.opsz);
    }
    let ok = ok && lit == [7, 12, 30, 72];
    // A font without the axis keeps 0.
    let a = lay_in(
        &mut reg,
        &text_box(
            "Atkinson Hyperlegible Next",
            text,
            400,
            FontStyle::Normal,
            30.0,
        ),
    );
    let ok = ok && a.outlines.tables[0].face.opsz == 0;
    println!(
        "CHECK crate=text name=opsz_follows_size result={} inter_opsz={keys:?} inter_width_per_em={widths:.3?} literata_opsz={lit:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn italic_picks_the_italic_face_and_a_family_without_one_stays_upright() {
    let mut reg = FontRegistry::bundled_only();
    let mut ok = true;
    let mut seen = Vec::new();
    for family in [
        "Atkinson Hyperlegible Next",
        "Atkinson Hyperlegible Mono",
        "Inter",
        "Literata",
    ] {
        let up = lay_in(
            &mut reg,
            &text_box(family, "Hamburg", 400, FontStyle::Normal, 12.0),
        );
        let it = lay_in(
            &mut reg,
            &text_box(family, "Hamburg", 400, FontStyle::Italic, 12.0),
        );
        let (u, i) = (&up.outlines.tables[0], &it.outlines.tables[0]);
        let good =
            !u.face.italic && i.face.italic && u.glyphs != i.glyphs && reg.has_italic(family);
        ok &= good;
        seen.push(format!("{family}:{good}"));
    }
    for (family, text) in [("Caveat", "Hamburg"), ("Noto Emoji", "\u{1F600}\u{2764}")] {
        let up = lay_in(
            &mut reg,
            &text_box(family, text, 400, FontStyle::Normal, 12.0),
        );
        let it = lay_in(
            &mut reg,
            &text_box(family, text, 400, FontStyle::Italic, 12.0),
        );
        let good = up.outlines == it.outlines && !reg.has_italic(family);
        ok &= good;
        seen.push(format!("{family}_upright:{good}"));
    }
    println!(
        "CHECK crate=text name=italic_faces result={} {}",
        pass(ok),
        seen.join(" ")
    );
    assert!(ok);
}

#[test]
fn weight_is_clamped_to_the_family_range_on_layout() {
    let mut reg = FontRegistry::bundled_only();
    let range = |reg: &mut FontRegistry, f: &str| reg.weights_of(f);
    let atk = range(&mut reg, "Atkinson Hyperlegible Next");
    let data = BundledFont::for_family("Atkinson Hyperlegible Next")
        .unwrap()
        .data;
    let (lo, _, hi) = wght_range(data).unwrap();
    let want = Weights::Range(lo.round() as u16, hi.round() as u16);
    let heavy = lay_in(
        &mut reg,
        &text_box(
            "Atkinson Hyperlegible Next",
            "Hamburg",
            900,
            FontStyle::Normal,
            12.0,
        ),
    );
    let top = lay_in(
        &mut reg,
        &text_box(
            "Atkinson Hyperlegible Next",
            "Hamburg",
            hi as u16,
            FontStyle::Normal,
            12.0,
        ),
    );
    let ok = atk == Some(want)
        && hi < 900.0
        && heavy.outlines.tables[0].face.weight == hi as u16
        && heavy.outlines == top.outlines;
    println!(
        "CHECK crate=text name=weight_clamped result={} range={lo}..{hi} asked=900 drawn={}",
        pass(ok),
        heavy.outlines.tables[0].face.weight
    );
    assert!(ok);
}

#[test]
fn a_missing_character_comes_from_the_bundled_fonts_only() {
    let mut reg = FontRegistry::bundled_only();
    let tb = text_box(
        "Atkinson Hyperlegible Next",
        "Hi \u{1F600} \u{2764}",
        400,
        FontStyle::Normal,
        12.0,
    );
    let l = lay_in(&mut reg, &tb);
    let families: Vec<&str> = l
        .outlines
        .tables
        .iter()
        .map(|t| t.face.family.as_str())
        .collect();
    // A character in no bundled font is the chosen font's missing-glyph box.
    let none = lay_in(
        &mut reg,
        &text_box("Inter", "\u{4E00}", 400, FontStyle::Normal, 12.0),
    );
    let ok = families.contains(&"Noto Emoji")
        && families[0] == "Atkinson Hyperlegible Next"
        && none.missing_glyphs == 1;
    println!(
        "CHECK crate=text name=bundled_fallback result={} tables={families:?} missing_for_cjk={}",
        pass(ok),
        none.missing_glyphs
    );
    assert!(ok);
}

#[test]
fn installed_fonts_are_listed_and_a_bundled_name_is_never_shadowed() {
    let t = std::time::Instant::now();
    let mut reg = FontRegistry::with_system();
    let scan_ms = t.elapsed().as_secs_f64() * 1000.0;
    let families = reg.families();
    let installed: Vec<&FamilyInfo> = families.iter().filter(|f| !f.bundled).collect();
    let faces = reg.system_face_count();
    // Bundled first, then installed sorted by name; no installed family has a bundled name.
    let order_ok = families.iter().take_while(|f| f.bundled).count() == 6
        && families.iter().skip(6).all(|f| !f.bundled);
    let no_shadow = installed
        .iter()
        .all(|f| BundledFont::for_family(&f.name).is_none());
    // The OS-level promise: macOS and Windows always have fonts installed (CI proves it).
    let must_have = cfg!(any(target_os = "macos", target_os = "windows"));
    let found_ok = !must_have || !installed.is_empty();
    // An installed family lays out, and its missing characters come from bundled fonts only.
    let mut laid = String::from("none");
    let mut lay_ok = true;
    if let Some(f) = installed.first() {
        let tb = text_box(&f.name, "Hamburg \u{1F600}", 400, FontStyle::Normal, 12.0);
        match lay_out(
            &mut reg,
            "test",
            &[TextRequest {
                object_id: Uuid::from_u128(1),
                text: &tb,
                width_mm: 200.0,
            }],
        ) {
            Ok(l) => {
                laid = f.name.clone();
                lay_ok = !l.outlines.texts[0].glyphs.is_empty()
                    && l.outlines.tables.iter().all(|t| {
                        t.face.family == f.name || BundledFont::for_family(&t.face.family).is_some()
                    });
            }
            Err(e) => {
                laid = format!("error {e}");
                lay_ok = false;
            }
        }
    }
    let ok = order_ok && no_shadow && found_ok && lay_ok;
    println!(
        "CHECK crate=text name=system_faces result={} os={} faces={faces} families={} scan_ms={scan_ms:.1} must_find={must_have} laid={laid:?}",
        pass(ok),
        std::env::consts::OS,
        installed.len()
    );
    assert!(ok);
}

#[test]
fn rescan_reads_the_fonts_again_and_keeps_the_open_ones() {
    let mut reg = FontRegistry::with_system();
    let before = reg.families();
    let a = reg.rescan();
    let b = reg.rescan();
    let same = reg.families() == before;
    let bundled = FontRegistry::bundled_only().rescan();
    let ok = same && !a.changed && !b.changed && bundled.faces == 0;
    println!(
        "CHECK crate=text name=rescan result={} faces={} families={} first_ms={:.1} second_ms={:.1} bundled_only_faces={}",
        pass(ok),
        b.faces,
        b.families,
        a.elapsed.as_secs_f64() * 1000.0,
        b.elapsed.as_secs_f64() * 1000.0,
        bundled.faces
    );
    assert!(ok);
}
