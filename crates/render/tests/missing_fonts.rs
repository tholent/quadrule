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

//! Missing fonts (rung 12a, docs/bbq/2026-11-16-rung-12-missing-fonts.md, Defaulted "12a draws nothing
//! new"): a page whose font is not on this computer draws from the outlines the file saved, and its
//! export is byte for byte the export of the live text. The live text is made by a bundled font filed
//! under a test-only family name, carried as a document font; the same page on a registry without it
//! is the "missing" case. No golden image is added: this compares the page with itself. Each test
//! prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::QuadFile;
use quadrule_layout::{display_list, lay_out_text, lay_out_text_saved};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use quadrule_text::{BUNDLED, FontRegistry};
use std::sync::Arc;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const FAMILY: &str = "Test Carried Font";

/// The text-box page with every text box set in the carried test font.
fn page() -> Document {
    let mut doc = textbox_document();
    let ids: Vec<Uuid> = doc
        .objects()
        .filter(|o| matches!(o.kind, ObjectKind::Text(_)))
        .map(|o| o.id)
        .collect();
    for id in &ids {
        if let Some(o) = doc.object_mut(*id)
            && let ObjectKind::Text(t) = &mut o.kind
        {
            t.family = FAMILY.into();
            t.weight = 400;
        }
    }
    doc
}

fn carried() -> FontRegistry {
    let caveat = BUNDLED.iter().find(|b| b.id == "caveat").unwrap();
    let mut reg = FontRegistry::bundled_only();
    reg.set_embedded(&[(FAMILY.into(), Arc::new(caveat.data.to_vec()))]);
    reg
}

fn png(doc: &Document, id: &str, outlines: &quadrule_model::outlines::ProfileOutlines) -> Vec<u8> {
    let primary = builtin_for(&doc.primary_profile_id, doc.page.orientation).unwrap();
    let target = builtin_for(id, doc.page.orientation).unwrap();
    let dl = display_list(doc, &primary, &target, outlines).unwrap();
    encode_png(&render(&dl), None)
}

#[test]
fn a_page_with_a_missing_font_exports_the_live_bytes_from_saved_outlines() {
    let doc = page();
    let primary = builtin_for(&doc.primary_profile_id, doc.page.orientation).unwrap();
    // Live: the font is here.
    let mut reg = carried();
    let mut live = Vec::new();
    for id in doc.profile_ids() {
        let target = builtin_for(id, doc.page.orientation).unwrap();
        let laid = lay_out_text(&doc, &primary, &target, &mut reg).unwrap();
        assert!(laid.unavailable.is_empty());
        live.push(laid.outlines);
    }
    let live_pngs: Vec<Vec<u8>> = doc
        .profile_ids()
        .iter()
        .zip(&live)
        .map(|(id, o)| png(&doc, id, o))
        .collect();
    // Saved, then opened where the font is missing.
    let file = QuadFile::new(doc.clone(), live, vec![]);
    let opened = QuadFile::from_bytes(&file.to_bytes("t").unwrap()).unwrap();
    let mut bare = FontRegistry::bundled_only();
    let mut same = 0;
    let mut from_saved = 0;
    let mut needs = 0;
    for (i, id) in doc.profile_ids().into_iter().enumerate() {
        let target = builtin_for(id, doc.page.orientation).unwrap();
        let (laid, how) =
            lay_out_text_saved(&doc, &primary, &target, &mut bare, &opened.outlines).unwrap();
        from_saved += how.from_saved.len();
        needs += how.needs_font.len();
        // The page is drawn from the merged outlines; so are the ones Save would write.
        let drawn = png(&doc, id, &laid.outlines);
        same += usize::from(drawn == live_pngs[i]);
        // The merged outlines are the saved ones, text for text.
        assert_eq!(laid.outlines.texts.len(), opened.outlines[id].texts.len());
    }
    let ok = same == doc.profile_ids().len() && from_saved > 0 && needs == 0;
    println!(
        "\nCHECK crate=render name=missing_font_exports_live_bytes result={} devices_same={same} boxes_from_saved={from_saved} needs_font={needs}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_device_with_no_saved_outlines_reuses_another_or_needs_the_font() {
    // ADR-001 item 7: a box is drawn from another device's shapes where it is the same size in mm;
    // otherwise it draws nothing and says it needs the font. Either way the page still draws.
    let mut doc = page();
    let secondary = doc.profile_ids()[1].to_string();
    // One box is a different size on the secondary device, so nothing can be reused for it there.
    let wide = doc
        .objects()
        .find(|o| matches!(o.kind, ObjectKind::Text(_)))
        .map(|o| o.id)
        .unwrap();
    {
        let o = doc.object_mut(wide).unwrap();
        let f = o.frame;
        o.overrides.insert(
            secondary.clone(),
            Override {
                size: Some(Size {
                    w_mm: f.w_mm * 0.5,
                    h_mm: f.h_mm,
                }),
                ..Default::default()
            },
        );
    }
    let primary = builtin_for(&doc.primary_profile_id, doc.page.orientation).unwrap();
    let mut reg = carried();
    let mut saved = std::collections::BTreeMap::new();
    for id in doc.profile_ids() {
        let target = builtin_for(id, doc.page.orientation).unwrap();
        saved.insert(
            id.to_string(),
            lay_out_text(&doc, &primary, &target, &mut reg)
                .unwrap()
                .outlines,
        );
    }
    saved.remove(&secondary);
    let target = builtin_for(&secondary, doc.page.orientation).unwrap();
    let mut bare = FontRegistry::bundled_only();
    let (laid, how) = lay_out_text_saved(&doc, &primary, &target, &mut bare, &saved).unwrap();
    let boxes = doc
        .objects()
        .filter(|o| matches!(o.kind, ObjectKind::Text(_)))
        .count();
    let drew = display_list(&doc, &primary, &target, &laid.outlines).is_ok();
    let empty = how
        .needs_font
        .iter()
        .all(|id| laid.outlines.text(*id).is_some_and(|t| t.glyphs.is_empty()));
    let ok = how.from_saved.is_empty()
        && how.reused.len() + how.needs_font.len() == boxes
        && how.needs_font == [wide]
        && drew
        && empty
        && laid.outlines.texts.len() == boxes;
    println!(
        "\nCHECK crate=render name=missing_font_new_device result={} boxes={boxes} reused={} needs_font={}",
        pass(ok),
        how.reused.len(),
        how.needs_font.len()
    );
    assert!(ok);
}
