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

//! The grand-staff page (docs/bbq/2026-11-18-fixes-before-16b.md, Q2; format 0.18): six labelled cells of music staves, four
//! of them grand staves with a space of their own between them, drawn on each device in portrait and compared with golden
//! images in `tests/golden/` (`grand-staff_*`), whose sums sit after the thirty-two that were there before, which are
//! unchanged. Then: the same document drawn again is the same bytes, a saved and reopened file exports the same, the page with
//! the field taken off draws differently, and the older pages with grand staves (no field) draw what their goldens say. Each
//! step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test grand_staff`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use quadrule_text::FontRegistry;
use std::path::PathBuf;

const PREFIX: &str = "grand-staff";

/// The sha256 of the thirty-two lines of `SHA256SUMS` that were there before this page (each ending in a newline).
const OLD_32_SHA256: &str = "206e90e8660c0408bf1dbc16892f2d4c1b8f60e0be4afda4262c9c01779e2e55";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("grand_staff");
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
    render(&dl, Math::Fastest)
}

/// The same page with the grand staves' own space taken off (they are top to top again).
fn without_space(doc: &Document) -> Document {
    let mut d = doc.clone();
    let ids: Vec<Uuid> = d.objects().map(|o| o.id).collect();
    for id in ids {
        if let Some(o) = d.object_mut(id)
            && let ObjectKind::Pattern(pr) = &mut o.kind
            && let Pattern::MusicStaff {
                grand_space_units, ..
            } = &mut pr.pattern
        {
            *grand_space_units = None;
        }
    }
    d
}

#[test]
fn grand_staff_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = grand_staff_document();
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
            "\nCHECK crate=render name=grand_staff_golden profile={id} result={} sizes={sizes} sha256={sha} golden_sha256={} os={} arch={}",
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
    // The thirty-two lines that were there come first, byte for byte.
    let first: String = lines.iter().take(32).map(|l| format!("{l}\n")).collect();
    let old_32 = lines.len() >= 32 && sha256_hex(first.as_bytes()) == OLD_32_SHA256;
    println!(
        "\nCHECK crate=render name=grand_staff_sums_listed result={} listed={listed} old_32_first_and_unchanged={old_32}",
        pass(listed && old_32)
    );
    assert!(all_ok && listed && old_32, "see the CHECK lines");
}

#[test]
fn the_grand_staff_page_draws_the_same_twice_and_differs_without_the_space() {
    let doc = grand_staff_document();
    let flat = without_space(&doc);
    let mut same = true;
    let mut differs = true;
    for id in doc.profile_ids() {
        let a = design(&doc, id);
        same &= a == design(&doc, id);
        differs &= a != design(&flat, id);
    }
    println!(
        "\nCHECK crate=render name=grand_staff_repeatable result={} same_twice={same} differs_without_space={differs}",
        pass(same && differs)
    );
    assert!(same && differs);
}

#[test]
fn grand_staves_with_no_space_field_draw_what_they_drew_before() {
    // The patterns page and the turned-patterns page hold grand staves with no field: their exports are the goldens that
    // were blessed before the field existed, so a layout that changed a grand staff without one would show here.
    let mut same = true;
    let mut names = Vec::new();
    for (prefix, doc) in [
        ("patterns", patterns_document()),
        ("patterns-turned", turned_patterns_document()),
    ] {
        for id in doc.profile_ids() {
            let img = design(&doc, id);
            let target = builtin_for(id, Orientation::Portrait).unwrap();
            let (dw, dh) = target.device_size();
            let name = format!("{prefix}_{id}_{dw}x{dh}.png");
            let golden = std::fs::read(golden_dir().join(&name)).unwrap();
            let one = encode_png(&img, None) == golden;
            same &= one;
            names.push(format!("{name}:{one}"));
        }
    }
    println!(
        "\nCHECK crate=render name=grand_staff_old_unchanged result={} old_pages_match_their_goldens={same} {}",
        pass(same),
        names.join(" ")
    );
    assert!(same);
}

#[test]
fn the_grand_staff_page_exports_the_same_bytes_after_a_save() {
    let doc = grand_staff_document();
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
        .to_bytes("grand-staff-test")
        .unwrap();
    let file = QuadFile::from_bytes(&bytes).unwrap();
    let same_doc = file.doc == doc;
    let current = file.manifest.format_version == quadrule_format::FORMAT_VERSION
        && file.migrated_from.is_none();
    let again = file.to_bytes("grand-staff-test").unwrap();
    let bytes_stable = again == bytes;
    let mut same = true;
    for id in doc.profile_ids() {
        let target = builtin_for(id, o).unwrap();
        let dl = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
        same &= render(&dl, Math::Fastest) == design(&doc, id);
    }
    // A staff with no space writes no `grand_space_units` field at all; one with a space writes it.
    let has_key = |pr: &PatternRegion| {
        let v = ciborium::Value::serialized(pr).unwrap();
        format!("{v:?}").contains("grand_space_units")
    };
    let (mut with_key, mut without_key) = (0, 0);
    for o in doc.objects() {
        if let ObjectKind::Pattern(pr) = &o.kind {
            if let Pattern::MusicStaff {
                grand_space_units, ..
            } = &pr.pattern
            {
                let written = has_key(pr);
                assert_eq!(written, grand_space_units.is_some());
                if written {
                    with_key += 1;
                } else {
                    without_key += 1;
                }
            }
        }
    }
    let ok = same_doc && current && bytes_stable && same && with_key == 4 && without_key == 2;
    println!(
        "\nCHECK crate=render name=grand_staff_save_round_trip result={} doc_equal={same_doc} export_equal={same} bytes_stable={bytes_stable} staves_with_key={with_key} staves_without_key={without_key}",
        pass(ok)
    );
    assert!(ok);
}
