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

//! Missing fonts in the window's state (rung 12a, docs/bbq/2026-11-16-rung-12-missing-fonts.md):
//! the bar and what it names, read-only boxes, Substitute as one undo step, Remember, Save never
//! writing changed text with stale outlines (the rung 3 debt), embedded fonts round trip, the Save
//! sheet and the strings. The "missing" font is a bundled file filed under a test-only family name,
//! so nothing depends on this computer's fonts. Each test prints one structured `CHECK` line (run
//! with `-- --nocapture`).

use quadrule_format::QuadFile;
use quadrule_layout::lay_out_text;
use quadrule_model::edit::Command;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_text::{BUNDLED, FontRegistry};
use quadrule_ui::Source;
use quadrule_ui::{Viewer, ViewerUi};
use std::sync::Arc;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const FAMILY: &str = "Test Carried Font";

fn caveat() -> Arc<Vec<u8>> {
    Arc::new(
        BUNDLED
            .iter()
            .find(|b| b.id == "caveat")
            .unwrap()
            .data
            .to_vec(),
    )
}

fn texts(doc: &Document) -> Vec<Uuid> {
    doc.objects()
        .filter(|o| matches!(o.kind, ObjectKind::Text(_)))
        .map(|o| o.id)
        .collect()
}

/// A saved file of the text-box page set in the test font, made where the font is carried.
fn file(embed: bool) -> Vec<u8> {
    let mut doc = textbox_document();
    for id in texts(&doc) {
        if let ObjectKind::Text(t) = &mut doc.object_mut(id).unwrap().kind {
            t.family = FAMILY.into();
            t.weight = 400;
        }
    }
    let mut reg = FontRegistry::bundled_only();
    reg.set_embedded(&[(FAMILY.into(), caveat())]);
    let primary = builtin_for(&doc.primary_profile_id, doc.page.orientation).unwrap();
    let outlines = doc
        .profile_ids()
        .iter()
        .map(|id| {
            let t = builtin_for(id, doc.page.orientation).unwrap();
            lay_out_text(&doc, &primary, &t, &mut reg).unwrap().outlines
        })
        .collect();
    let mut f = QuadFile::new(doc, outlines, vec![]);
    if embed {
        let bytes = caveat();
        let path = quadrule_format::font_path(&bytes);
        let mut r = quadrule_format::FontRef::new(FAMILY, None, None);
        r.files.push(quadrule_format::FontFile {
            path: path.clone(),
            sha256: quadrule_format::sha256_hex(&bytes),
            weight: 400,
            italic: false,
            extra: Default::default(),
        });
        f.manifest.fonts = vec![r];
        f.font_files.insert(path, bytes.to_vec());
    }
    f.to_bytes("t").unwrap()
}

fn open(bytes: &[u8]) -> Viewer {
    let mut v = Viewer::default();
    v.open_bytes(Source::Untitled, bytes).unwrap();
    v
}

fn family_of(v: &Viewer, id: Uuid) -> String {
    match &v.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Text(t) => t.family.clone(),
        _ => String::new(),
    }
}

#[test]
fn a_page_with_a_missing_font_opens_with_a_bar_and_read_only_boxes() {
    let mut v = open(&file(false));
    let boxes = texts(v.doc().unwrap());
    let missing = v.missing_fonts().to_vec();
    let page = v.page().map(|p| (p.saved_outlines, p.findings.clone()));
    let (saved, findings) = page.unwrap();
    let l11 = findings.iter().filter(|f| f.rule == "L11").count();
    let all_ro = boxes.iter().all(|b| v.is_read_only_text(*b));
    // Typing on the canvas does not start; a reflowing edit is refused with a notice.
    let typing = v.begin_typing(boxes[0], quadrule_ui::Start::End);
    let refused = v
        .edit(
            Command::SetObject({
                let mut o = v.doc().unwrap().object(boxes[0]).unwrap().clone();
                o.frame.w_mm += 3.0;
                o
            }),
            None,
        )
        .is_err()
        && v.notice.is_some();
    let ok = missing.len() == 1
        && missing[0].family == FAMILY
        && missing[0].boxes.len() == boxes.len()
        && v.bar_fonts().len() == 1
        && saved
        && l11 == boxes.len()
        && all_ro
        && !typing
        && refused;
    // The bar closes with its ×.
    v.close_font_bar();
    let closed = v.bar_fonts().is_empty() && v.missing_fonts().len() == 1;
    println!(
        "\nCHECK crate=ui name=missing_bar_and_read_only result={} families={} boxes={} l11={l11} typing_started={typing} edit_refused={refused} bar_closes={closed}",
        pass(ok && closed),
        missing.len(),
        boxes.len()
    );
    assert!(ok && closed);
}

#[test]
fn substitute_rewrites_every_box_in_one_undo_step_and_unlocks_them() {
    let mut v = open(&file(false));
    let boxes = texts(v.doc().unwrap());
    let best = v.font_matches(FAMILY);
    let pick = best[0].family.clone();
    let list_ok = best.len() == 5 && best.iter().all(|m| m.family != FAMILY);
    let r = v.apply_font_choices(&[(FAMILY.into(), pick.clone())], &[]);
    let rewritten = r.is_ok() && boxes.iter().all(|b| family_of(&v, *b) == pick);
    let free =
        v.missing_fonts().is_empty() && !v.is_read_only_text(boxes[0]) && v.bar_fonts().is_empty();
    // Now editable.
    let edited = v.edit_selected_for_test(boxes[0]).is_ok();
    v.undo();
    v.undo();
    let undone = boxes.iter().all(|b| family_of(&v, *b) == FAMILY) && v.is_read_only_text(boxes[0]);
    let ok = list_ok && rewritten && free && edited && undone;
    println!(
        "\nCHECK crate=ui name=substitute_one_undo result={} pick={pick} list_of_five={list_ok} rewritten={rewritten} editable_after={edited} two_undos_restore={undone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn remember_keeps_the_font_out_of_the_bar_and_survives_a_save() {
    let mut v = open(&file(false));
    let r = v.apply_font_choices(&[], &[FAMILY.into()]);
    let quiet = r.is_ok() && v.bar_fonts().is_empty() && v.missing_fonts()[0].kept;
    let bytes = v.file_bytes().unwrap();
    let again = open(&bytes);
    let kept = again.bar_fonts().is_empty()
        && again.missing_fonts().len() == 1
        && again.doc().unwrap().page.fonts.keeps(FAMILY);
    let ok = quiet && kept;
    println!(
        "\nCHECK crate=ui name=remember_for_document result={} quiet={quiet} after_reopen={kept}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn save_cannot_write_changed_text_with_stale_outlines() {
    // The rung 3 debt: with a font missing, Save used to keep the file's saved outlines even if the
    // text had changed. Now the text cannot change, and a changed box in a font that is here is laid
    // out fresh. Both are in one file.
    let mut v = open(&file(false));
    let boxes = texts(v.doc().unwrap());
    let (stuck, other) = (boxes[0], boxes[1]);
    let before = v.doc().unwrap().clone();
    // The stuck box's text cannot be changed.
    let change = |v: &mut Viewer, id: Uuid, f: &dyn Fn(&mut TextBox)| {
        let mut o = v.doc().unwrap().object(id).unwrap().clone();
        if let ObjectKind::Text(t) = &mut o.kind {
            f(t);
        }
        v.edit(Command::SetObject(o), None)
    };
    let refused = change(&mut v, stuck, &|t| t.content = "different words".into()).is_err();
    // The other box is given a font that is here and new words: it is laid out live.
    let live = {
        let r = v.apply_font_choices(&[], &[]);
        r.is_ok()
            && v.apply_font_choices(&[(FAMILY.into(), "Inter".into())], &[])
                .is_ok()
            && change(&mut v, other, &|t| t.content = "fresh words".into()).is_ok()
    };
    let bytes = v.file_bytes().unwrap();
    let saved = QuadFile::from_bytes(&bytes).unwrap();
    let words = |id: Uuid| match &saved.doc.object(id).unwrap().kind {
        ObjectKind::Text(t) => t.content.clone(),
        _ => String::new(),
    };
    let fresh_glyphs = |id: Uuid| {
        saved
            .outlines
            .values()
            .next()
            .and_then(|o| o.text(id))
            .map(|t| t.glyphs.len())
            .unwrap_or(0)
    };
    let was_glyphs = |id: Uuid| {
        let old = QuadFile::from_bytes(&file(false)).unwrap();
        old.outlines
            .values()
            .next()
            .and_then(|o| o.text(id))
            .map(|t| t.glyphs.len())
            .unwrap_or(0)
    };
    let ok = refused
        && live
        && words(other) == "fresh words"
        && fresh_glyphs(other) != was_glyphs(other)
        && !before.object(stuck).is_none();
    println!(
        "\nCHECK crate=ui name=save_no_stale_outlines result={} stuck_text_change_refused={refused} live_box_resaved_fresh={}",
        pass(ok),
        fresh_glyphs(other) != was_glyphs(other)
    );
    assert!(ok);
}

#[test]
fn a_still_missing_box_is_saved_with_the_outlines_it_had() {
    let mut v = open(&file(false));
    let boxes = texts(v.doc().unwrap());
    let old = QuadFile::from_bytes(&file(false)).unwrap();
    let bytes = v.file_bytes().unwrap();
    let saved = QuadFile::from_bytes(&bytes).unwrap();
    let same = saved.outlines.iter().all(|(id, o)| {
        boxes
            .iter()
            .all(|b| o.text(*b).map(|t| &t.glyphs) == old.outlines[id].text(*b).map(|t| &t.glyphs))
    });
    // The manifest keeps the font by name, so a computer that has it can match it.
    let named = saved.manifest.fonts.iter().any(|f| f.family == FAMILY);
    let ok = same && named && saved.doc == old.doc;
    println!(
        "\nCHECK crate=ui name=save_keeps_saved_outlines result={} same_glyphs={same} font_named={named}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_embedded_font_makes_the_page_live_and_a_damaged_one_does_not_stop_it() {
    let good = file(true);
    let mut v = open(&good);
    let none_missing = v.missing_fonts().is_empty() && v.bar_fonts().is_empty();
    let page_ok = v.page().is_some_and(|p| !p.saved_outlines);
    // Embedded fonts are only this document's: opening another forgets them.
    v.open_sample();
    let gone = !v.fonts_has_family_for_test(FAMILY);
    // Saved again with Embed fonts on, the font file stays in the file.
    let mut v = open(&good);
    let again = QuadFile::from_bytes(&v.file_bytes().unwrap()).unwrap();
    let kept = again.font_files.len() == 1
        && again
            .manifest
            .fonts
            .iter()
            .any(|f| f.family == FAMILY && f.files.len() == 1 && f.signature.is_some());
    // With Embed fonts off it is dropped.
    v.set_embed_fonts(false).unwrap();
    let off = QuadFile::from_bytes(&v.file_bytes().unwrap()).unwrap();
    let dropped = off.font_files.is_empty();
    // Damage the embedded file in the archive: the page opens, with the font missing.
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(&good)).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let name = f.name().unwrap().to_string();
        let mut b = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut b).unwrap();
        if name.starts_with("assets/fonts/") {
            b[10] ^= 0xff;
        }
        out.start_file(name, opts).unwrap();
        std::io::Write::write_all(&mut out, &b).unwrap();
    }
    let damaged = out.finish().unwrap().into_inner();
    let opened = open_result(&damaged);
    let ok = none_missing && page_ok && gone && kept && dropped && opened;
    println!(
        "\nCHECK crate=ui name=embedded_font_in_the_window result={} live={none_missing} forgotten_with_the_file={gone} saved_again={kept} dropped_when_off={dropped} damaged_opens_missing={opened}",
        pass(ok)
    );
    assert!(ok);
}

fn open_result(bytes: &[u8]) -> bool {
    let mut v = Viewer::default();
    v.open_bytes(Source::Untitled, bytes).is_ok() && v.missing_fonts().len() == 1
}

#[test]
fn the_sheet_comes_before_save_and_never_for_autosave() {
    // Installed fonts are needed for the sheet; stand them in with a registry whose installed fonts
    // are the test font.
    let mut v = Viewer::default();
    v.use_installed_for_test(&[(FAMILY.into(), caveat())]);
    v.open_bytes(Source::Untitled, &file(false)).unwrap();
    let none_missing = v.missing_fonts().is_empty();
    let listed = v.font_sheet_fonts();
    let mut ui = ViewerUi::new(v);
    let held = ui.save().is_empty() && ui.font_sheet_open();
    let cancelled = ui.answer_font_sheet(false, false).is_empty() && !ui.font_sheet_open();
    let held_again = ui.save().is_empty() && ui.font_sheet_open();
    // Continue with the tick: the document remembers, and the next Save goes straight through.
    ui.answer_font_sheet(true, true);
    let quiet = ui.viewer.doc().unwrap().page.fonts.dont_warn;
    let direct = {
        let _ = ui.save();
        !ui.font_sheet_open()
    };
    let l20 = ui
        .viewer
        .page()
        .map(|p| p.findings.iter().any(|f| f.rule == "L20"))
        .unwrap_or(false);
    let ok = none_missing
        && listed.len() == 1
        && held
        && cancelled
        && held_again
        && quiet
        && direct
        && l20;
    println!(
        "\nCHECK crate=ui name=font_sheet result={} fonts={} held={held} cancel_stops={cancelled} dont_warn_saved={quiet} next_save_direct={direct} l20={l20}",
        pass(ok),
        listed.len()
    );
    assert!(ok);
}

#[test]
fn every_font_message_id_exists_in_the_strings() {
    let s = quadrule_i18n::Strings::english();
    let ids = [
        quadrule_text::embed::NoEmbed::Restricted,
        quadrule_text::embed::NoEmbed::PreviewAndPrint,
        quadrule_text::embed::NoEmbed::BitmapOnly,
        quadrule_text::embed::NoEmbed::Collection,
        quadrule_text::embed::NoEmbed::NoLicence,
    ];
    let missing: Vec<_> = ids.iter().filter(|r| !s.has(r.message_id())).collect();
    let ok = missing.is_empty();
    println!(
        "\nCHECK crate=ui name=embed_reason_strings result={} missing={missing:?}",
        pass(ok)
    );
    assert!(ok);
}
