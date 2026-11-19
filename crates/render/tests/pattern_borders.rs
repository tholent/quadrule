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

//! The pattern-borders page (docs/bbq/2026-11-18-pattern-borders.md): a border round a pattern region in twelve
//! labelled cells (all four sides, one side off, rounded corners, two sides, a custom line, a turned frame, turned
//! rulings), drawn on each device in portrait and compared with golden images in `tests/golden/`
//! (`pattern-borders_*`), whose sums sit after the thirty that were there before, which are unchanged. Then: the
//! same document drawn again is the same bytes, a saved and reopened file exports the same, a page with the borders
//! taken off draws differently, and a region with no border draws the same bytes as before the feature. Each step
//! prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test pattern_borders`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use quadrule_text::FontRegistry;
use std::path::PathBuf;

const PREFIX: &str = "pattern-borders";

/// The sha256 of the thirty lines of `SHA256SUMS` that were there before this page (each ending in a newline),
/// as re-made at the fallback level (ADR-018).
const OLD_THIRTY_SHA256: &str = "70eb24a2b3c793191f24b94928f685d0c92a47cd0d7f23335125373828b6af12";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("pattern_borders");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// The design on one device.
fn design(doc: &Document, id: &str) -> Gray {
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(doc, &primary, &target, &laid.outlines).unwrap();
    render(&dl)
}

/// The same page with every border taken off.
fn unbordered(doc: &Document) -> Document {
    let mut d = doc.clone();
    let ids: Vec<Uuid> = d.objects().map(|o| o.id).collect();
    for id in ids {
        if let Some(o) = d.object_mut(id)
            && let ObjectKind::Pattern(pr) = &mut o.kind
        {
            pr.border = None;
        }
    }
    d
}

#[test]
fn pattern_borders_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = pattern_borders_document();
    let mut sums = String::new();
    let mut all_ok = true;
    for id in doc.profile_ids() {
        let img = design(&doc, id);
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let (dw, dh) = target.device_size();
        let sizes = (img.width, img.height) == (dw, dh);
        let png = encode_png(&img, None);
        let name = format!("{PREFIX}_{id}_{dw}x{dh}.png");
        std::fs::write(out_dir().join(&name), &png).unwrap();
        let sha = sha256_hex(&png);
        sums.push_str(&format!("{sha}  {name}\n"));
        let golden_path = golden_dir().join(&name);
        if bless {
            std::fs::write(&golden_path, &png).unwrap();
        }
        let golden = std::fs::read(&golden_path).unwrap_or_default();
        let matches = golden == png;
        all_ok &= matches && sizes;
        println!(
            "\nCHECK crate=render name=pattern_borders_golden profile={id} result={} sizes={sizes} sha256={sha} golden_sha256={} os={} arch={}",
            pass(matches && sizes),
            if golden.is_empty() {
                "none".to_string()
            } else {
                sha256_hex(&golden)
            },
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }
    let path = golden_dir().join("SHA256SUMS");
    if bless {
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with(&format!("{PREFIX}_")))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    let file = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    // The thirty lines that were there come first, byte for byte.
    let first: String = lines.iter().take(30).map(|l| format!("{l}\n")).collect();
    let old_thirty = lines.len() >= 30 && sha256_hex(first.as_bytes()) == OLD_THIRTY_SHA256;
    println!(
        "\nCHECK crate=render name=pattern_borders_sums_listed result={} listed={listed} old_thirty_first_and_unchanged={old_thirty}",
        pass(listed && old_thirty)
    );
    assert!(all_ok && listed && old_thirty, "see the CHECK lines");
}

#[test]
fn the_pattern_borders_page_draws_the_same_twice_and_differs_from_no_border() {
    let doc = pattern_borders_document();
    let flat = unbordered(&doc);
    let mut same = true;
    let mut differs = true;
    for id in doc.profile_ids() {
        let a = design(&doc, id);
        same &= a == design(&doc, id);
        differs &= a != design(&flat, id);
    }
    println!(
        "\nCHECK crate=render name=pattern_borders_repeatable result={} same_twice={same} differs_from_no_border={differs}",
        pass(same && differs)
    );
    assert!(same && differs);
}

#[test]
fn a_region_with_no_border_draws_what_it_drew_before_the_feature() {
    // The pattern-angle page has no border anywhere: its export is the golden that was blessed before the border
    // existed, so a layout that added anything for a region without one would show here.
    let doc = pattern_angle_document();
    let mut same = true;
    for id in doc.profile_ids() {
        let img = design(&doc, id);
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let (dw, dh) = target.device_size();
        let name = format!("pattern-angle_{id}_{dw}x{dh}.png");
        let golden = std::fs::read(golden_dir().join(name)).unwrap();
        same &= encode_png(&img, None) == golden;
    }
    println!(
        "\nCHECK crate=render name=pattern_borders_none_unchanged result={} angle_page_matches_its_golden={same}",
        pass(same)
    );
    assert!(same);
}

#[test]
fn the_pattern_borders_page_exports_the_same_bytes_after_a_save() {
    let doc = pattern_borders_document();
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let outlines: Vec<_> = doc
        .profile_ids()
        .into_iter()
        .map(|id| {
            lay_out_text(&doc, &primary, &builtin_for(id, o).unwrap(), &mut reg)
                .unwrap()
                .outlines
        })
        .collect();
    let bytes = QuadFile::new(doc.clone(), outlines, vec![])
        .to_bytes("pattern-borders-test")
        .unwrap();
    let file = QuadFile::from_bytes(&bytes).unwrap();
    let same_doc = file.doc == doc;
    let current = file.manifest.format_version == quadrule_format::FORMAT_VERSION
        && file.migrated_from.is_none();
    let again = file.to_bytes("pattern-borders-test").unwrap();
    let bytes_stable = again == bytes;
    let mut same = true;
    for id in doc.profile_ids() {
        let target = builtin_for(id, o).unwrap();
        let dl = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
        same &= render(&dl) == design(&doc, id);
    }
    // A page with no border writes no `border` field at all.
    let mut no_key = true;
    for o in pattern_angle_document().objects() {
        if let ObjectKind::Pattern(pr) = &o.kind {
            let v = ciborium::Value::serialized(pr).unwrap();
            no_key &= !matches!(&v, ciborium::Value::Map(m) if m.iter().any(|(k, _)| k.as_text() == Some("border")));
        }
    }
    let ok = same_doc && current && bytes_stable && same && no_key;
    println!(
        "\nCHECK crate=render name=pattern_borders_save_round_trip result={} doc_equal={same_doc} export_equal={same} bytes_stable={bytes_stable} no_border_no_key={no_key}",
        pass(ok)
    );
    assert!(ok);
}
