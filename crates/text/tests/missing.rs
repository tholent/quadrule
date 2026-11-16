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

//! Rung 12a's font checks (docs/bbq/2026-11-16-rung-12-missing-fonts.md): the measured signature of
//! a font, the pick of a substitute, a box whose font is missing, fonts carried inside a file, and
//! fallback to an installed font. Tests use the bundled fonts only, some filed under test-only family
//! names, so no result depends on this computer's fonts. Each test prints one structured `CHECK` line
//! (run with `-- --nocapture`).

use quadrule_model::{FontClass, FontSignature, FontStyle, TextAlign, TextBox, Uuid};
use quadrule_text::*;
use std::sync::Arc;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn bundled_bytes(id: &str) -> Arc<Vec<u8>> {
    Arc::new(BUNDLED.iter().find(|b| b.id == id).unwrap().data.to_vec())
}

fn text_box(family: &str, content: &str) -> TextBox {
    TextBox {
        content: content.into(),
        family: family.into(),
        weight: 400,
        style: FontStyle::Normal,
        size_pt: 12.0,
        line_height: 1.25,
        letter_spacing_em: 0.0,
        align: TextAlign::Left,
        ..TextBox::default()
    }
}

fn sig(class: FontClass, weight: u16, avg_width: u16, x_height: u16) -> FontSignature {
    FontSignature {
        class,
        weight,
        width: 100,
        italic: false,
        avg_width,
        x_height,
        mono: class == FontClass::Mono,
        extra: Default::default(),
    }
}

#[test]
fn the_bundled_fonts_are_classed_by_measurement() {
    let mut reg = FontRegistry::bundled_only();
    let want = [
        ("Literata", FontClass::Serif),
        ("Inter", FontClass::Sans),
        ("Atkinson Hyperlegible Next", FontClass::Sans),
        ("Atkinson Hyperlegible Mono", FontClass::Mono),
        ("Caveat", FontClass::Script),
    ];
    let mut wrong = Vec::new();
    let mut parts = Vec::new();
    for (family, class) in want {
        let s = reg.signature_of(family);
        parts.push(format!("{family}={:?}", s.as_ref().map(|s| s.class)));
        let ok = s.is_some_and(|s| {
            s.class == class
                && s.weight == 400
                && s.width == 100
                && !s.italic
                && s.avg_width > 300
                && s.x_height > 300
                && s.mono == (class == FontClass::Mono)
        });
        if !ok {
            wrong.push(family);
        }
    }
    // Noto Emoji has no letters to measure: no signature, so it is never offered as a substitute.
    let emoji = reg.signature_of("Noto Emoji").is_none();
    let ok = wrong.is_empty() && emoji;
    println!(
        "CHECK crate=text name=signature_classes result={} {} emoji_unmeasured={emoji} wrong={wrong:?}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_substitute_is_picked_by_signature() {
    let mut reg = FontRegistry::bundled_only();
    let top = |reg: &mut FontRegistry, w: &Want| reg.best_matches(w, 5);
    // A serif the file measured: Literata is the only serif here.
    let serif = Want {
        family: "Futura Missing Serif".into(),
        signature: Some(sig(FontClass::Serif, 400, 470, 450)),
    };
    // A heavy sans: a sans comes first, not the serif or the script.
    let sans = Want {
        family: "Gill Missing".into(),
        signature: Some(sig(FontClass::Sans, 700, 500, 520)),
    };
    let mono = Want {
        family: "Courier Missing".into(),
        signature: Some(sig(FontClass::Mono, 400, 600, 520)),
    };
    let script = Want {
        family: "Pacifico Missing".into(),
        signature: Some(sig(FontClass::Script, 400, 400, 420)),
    };
    // A file from before 0.13 holds no signature: the name's own words decide the class.
    let by_name = Want {
        family: "Garamond Missing Serif".into(),
        signature: None,
    };
    // An exact name (any capitals) wins whatever the signature says.
    let exact = Want {
        family: "LITERATA".into(),
        signature: Some(sig(FontClass::Sans, 400, 500, 500)),
    };
    let picks: Vec<String> = [&serif, &sans, &mono, &script, &by_name, &exact]
        .iter()
        .map(|w| top(&mut reg, w)[0].family.clone())
        .collect();
    let list = top(&mut reg, &sans);
    let sorted = list.windows(2).all(|p| p[0].distance <= p[1].distance);
    let ok = picks[0] == "Literata"
        && ["Inter", "Atkinson Hyperlegible Next"].contains(&picks[1].as_str())
        && picks[2] == "Atkinson Hyperlegible Mono"
        && picks[3] == "Caveat"
        && picks[4] == "Literata"
        && picks[5] == "Literata"
        && list.len() == 5
        && sorted
        && list.iter().all(|m| m.bundled);
    println!(
        "CHECK crate=text name=substitution_picks result={} picks={picks:?} list={:?}",
        pass(ok),
        list.iter().map(|m| m.family.as_str()).collect::<Vec<_>>()
    );
    assert!(ok);
}

#[test]
fn a_box_with_a_missing_font_is_left_out_not_an_error() {
    let mut reg = FontRegistry::bundled_only();
    let tb = text_box("Not Installed Anywhere", "hello");
    let id = Uuid::from_u128(7);
    let l = lay_out(&mut reg, "p", &[TextRequest::new(id, &tb, 50.0)]).unwrap();
    // Typing needs the font itself and is refused.
    let refused = quadrule_text::edit::EditBuffer::new(
        &mut reg,
        &TextRequest::new(id, &tb, 50.0),
        &quadrule_text::edit::Caret::at_end("hello"),
    )
    .is_err();
    let ok = l.unavailable == [id] && l.outlines.texts.is_empty() && refused;
    println!(
        "CHECK crate=text name=missing_family_unavailable result={} refused_typing={refused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_font_carried_in_the_file_serves_only_that_file() {
    let mut reg = FontRegistry::bundled_only();
    let tb = text_box("Test Embedded", "hello world");
    let id = Uuid::from_u128(1);
    let before = lay_out(&mut reg, "p", &[TextRequest::new(id, &tb, 50.0)]).unwrap();
    let changed = reg.set_embedded(&[("Test Embedded".into(), bundled_bytes("caveat"))]);
    let with = lay_out(&mut reg, "p", &[TextRequest::new(id, &tb, 50.0)]).unwrap();
    let listed = reg
        .families()
        .iter()
        .any(|f| f.name == "Test Embedded" && f.embedded && !f.bundled);
    let weights = reg.weights_of("Test Embedded").is_some();
    let sig = reg.signature_of("Test Embedded").map(|s| s.class);
    let installed = reg.is_installed("Test Embedded");
    let gone = reg.set_embedded(&[]);
    let after = lay_out(&mut reg, "p", &[TextRequest::new(id, &tb, 50.0)]).unwrap();
    let ok = before.unavailable == [id]
        && changed
        && with.unavailable.is_empty()
        && with
            .outlines
            .tables
            .iter()
            .all(|t| t.face.family == "Test Embedded")
        && with.missing_glyphs == 0
        && listed
        && weights
        && sig == Some(FontClass::Script)
        && !installed
        && gone
        && after.unavailable == [id];
    println!(
        "CHECK crate=text name=embedded_font_serves_this_file result={} listed={listed} class={sig:?} gone_after={}",
        pass(ok),
        after.unavailable == [id]
    );
    assert!(ok);
}

#[test]
fn a_bundled_only_registry_never_looks_in_installed_fonts() {
    // Goldens and tests keep `bundled_only()`: whatever is on this computer, a letter no bundled font
    // has stays the font's "missing" box, and nothing is reported as fallback.
    let mut reg = FontRegistry::bundled_only();
    let tb = text_box("Inter", "\u{6f22}\u{5b57}");
    let l = lay_out(
        &mut reg,
        "p",
        &[TextRequest::new(Uuid::from_u128(1), &tb, 50.0)],
    )
    .unwrap();
    let ok = l.missing_glyphs > 0 && !reg.falls_back_to_installed();
    println!(
        "CHECK crate=text name=bundled_only_no_installed_fallback result={} missing_glyphs={}",
        pass(ok),
        l.missing_glyphs
    );
    assert!(ok);
}

#[test]
fn an_installed_font_is_opened_as_fallback_for_a_letter_the_bundled_fonts_lack() {
    // The installed fonts here are two stand-ins under test-only names. Atkinson Mono's file stands
    // in for a font with nothing special; the test asks the registry to look for Latin letters, which
    // both have, and checks it picks one family, the same one every time, and opens it.
    let files = [
        ("Test Installed B".to_string(), bundled_bytes("caveat")),
        ("Test Installed A".to_string(), bundled_bytes("caveat")),
    ];
    let mut reg = FontRegistry::with_installed_for_test(&files);
    let opened = reg.open_fallbacks_for_test("abc");
    let again = reg.open_fallbacks_for_test("abc");
    let ok = reg.falls_back_to_installed()
        && opened == ["Test Installed A"]
        && again == ["Test Installed A"]
        && reg.is_installed("Test Installed B");
    println!(
        "CHECK crate=text name=installed_fallback_opened result={} family={opened:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_system_registry_reports_the_installed_font_that_drew_a_letter() {
    // Uses this computer's own fonts, so it checks only what it can: if some installed font has the
    // letter U+6F22, a box that needs it is drawn with that font and the layout names it as fallback;
    // if none does, the check is skipped (and says so).
    let mut reg = FontRegistry::with_system();
    let tb = text_box("Inter", "\u{6f22}");
    let l = lay_out(
        &mut reg,
        "p",
        &[TextRequest::new(Uuid::from_u128(1), &tb, 50.0)],
    )
    .unwrap();
    let found = l.missing_glyphs == 0;
    let installed: Vec<&String> = l
        .fallback
        .iter()
        .filter(|f| BundledFont::for_family(f).is_none())
        .collect();
    let ok = !found || (!installed.is_empty() && installed.iter().all(|f| reg.is_installed(f)));
    println!(
        "CHECK crate=text name=system_fallback_reported result={} drawn={found} installed_fallback={installed:?}{}",
        pass(ok),
        if found {
            ""
        } else {
            " skipped=no_installed_font_has_the_letter"
        }
    );
    assert!(ok);
}

#[test]
fn the_embed_rule_follows_the_fs_type_bits() {
    use quadrule_text::embed::{NoEmbed, verdict, verdict_for_fs_type};
    // (fsType, may embed): installable and editable yes; restricted, preview-and-print and
    // bitmap-only no; "no subsetting" (0x0100) is fine because whole files are embedded.
    let cases: [(u16, bool); 9] = [
        (0x0000, true),
        (0x0008, true),
        (0x0100, true),
        (0x0108, true),
        (0x0002, false),
        (0x0004, false),
        (0x0200, false),
        (0x0208, false),
        (0x000A, false),
    ];
    let wrong: Vec<u16> = cases
        .iter()
        .filter(|(fs, yes)| verdict_for_fs_type(*fs).is_ok() != *yes)
        .map(|(fs, _)| *fs)
        .collect();
    // The measured values: every bundled font is installable (the record measured 10 of 10).
    let bundled_ok = BUNDLED.iter().all(|b| verdict(b.data).is_ok());
    let collection = verdict(b"ttcf\0\x01\0\0 rest") == Err(NoEmbed::Collection);
    let unreadable = verdict(b"not a font at all") == Err(NoEmbed::NoLicence);
    let reasons = [
        verdict_for_fs_type(0x0002),
        verdict_for_fs_type(0x0004),
        verdict_for_fs_type(0x0200),
    ] == [
        Err(NoEmbed::Restricted),
        Err(NoEmbed::PreviewAndPrint),
        Err(NoEmbed::BitmapOnly),
    ];
    let ok = wrong.is_empty() && bundled_ok && collection && unreadable && reasons;
    println!(
        "CHECK crate=text name=embed_rule result={} cases={} bundled_embeddable={bundled_ok} wrong={wrong:?}",
        pass(ok),
        cases.len()
    );
    assert!(ok);
}

#[test]
fn an_embedded_family_gives_back_its_whole_file() {
    let bytes = bundled_bytes("caveat");
    let mut reg = FontRegistry::bundled_only();
    reg.set_embedded(&[("Test Embedded".into(), bytes.clone())]);
    let files = reg.font_files("Test Embedded", &[(400, false), (700, false)]);
    let bundled = reg.font_files("Inter", &[(400, false)]);
    let ok = files
        .as_ref()
        .is_ok_and(|f| f.len() == 1 && f[0].bytes == *bytes && !f[0].italic)
        && bundled == Ok(Vec::new());
    println!(
        "CHECK crate=text name=font_files_whole result={} files={}",
        pass(ok),
        files.map_or(0, |f| f.len())
    );
    assert!(ok);
}
