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

//! The rung 9 pattern page (docs/bbq/2026-11-13-rung-9-pattern-library.md, Defaulted "Golden
//! pages"): every new pattern and style, major lines and an unbound region, drawn on each device in
//! portrait, and the same page held sideways (turned a quarter turn clockwise into the device's
//! portrait PNG, as rung 8b exports a landscape page), compared with golden images in
//! `tests/golden/`, whose sums sit after the twelve that were there before (slice, shapes,
//! transforms, groups, devices and landscape, two each), which are unchanged. Then: the same
//! document drawn again is the same bytes, and a saved and reopened file exports the same. Each
//! step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test patterns`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use quadrule_text::FontRegistry;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("patterns");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// The design on one device, upright (the turned size for a landscape page).
fn design(doc: &Document, id: &str) -> Gray {
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(doc, &primary, &target, &laid.outlines).unwrap();
    render(&dl, Math::Fastest)
}

/// The exported image: a landscape design turned into the device's portrait pixels.
fn export(doc: &Document, id: &str) -> Gray {
    let img = design(doc, id);
    if doc.page.orientation == Orientation::Landscape {
        turn_clockwise(&img)
    } else {
        img
    }
}

/// Checks (and with `QUADRULE_BLESS=1` writes) one page's goldens on each device. `prefix` starts
/// the file names. Returns the lines for `SHA256SUMS` and whether every image matched.
fn goldens(doc: &Document, prefix: &str, check: &str) -> (String, bool) {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let mut sums = String::new();
    let mut all_ok = true;
    for id in doc.profile_ids() {
        let img = export(doc, id);
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let (dw, dh) = target.device_size();
        let sizes = (img.width, img.height) == (dw, dh);
        let png = encode_png(&img, None);
        let name = format!("{prefix}_{id}_{dw}x{dh}.png");
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
            "\nCHECK crate=render name={check} profile={id} result={} sizes={sizes} sha256={sha} golden_sha256={} os={} arch={}",
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
    (sums, all_ok)
}

/// Replaces this page's own lines of `SHA256SUMS` (those whose name starts with `prefix`) with
/// `sums`, after the lines that were there.
fn bless_sums(prefix: &str, sums: &str) {
    let path = golden_dir().join("SHA256SUMS");
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let mut merged: String = old
        .lines()
        .filter(|l| {
            !l.split_whitespace()
                .nth(1)
                .is_some_and(|n| n.starts_with(prefix))
        })
        .map(|l| format!("{l}\n"))
        .collect();
    merged.push_str(sums);
    std::fs::write(&path, merged).unwrap();
}

#[test]
fn pattern_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let (portrait, ok_p) = goldens(&patterns_document(), "patterns", "patterns_golden");
    let (landscape, ok_l) = goldens(
        &patterns_landscape_document(),
        "patterns-landscape",
        "patterns_landscape_golden",
    );
    if bless {
        bless_sums("patterns_", &portrait);
        bless_sums("patterns-landscape_", &landscape);
    }
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = portrait
        .lines()
        .chain(landscape.lines())
        .all(|l| lines.contains(&l));
    let firsts = [
        "slice_",
        "shapes_",
        "transforms_",
        "groups_",
        "devices_",
        "landscape_",
    ];
    let old_twelve = lines.len() >= 12
        && firsts
            .iter()
            .enumerate()
            .all(|(i, k)| lines[2 * i..2 * i + 2].iter().all(|l| l.contains(k)));
    println!(
        "\nCHECK crate=render name=patterns_sums_listed result={} listed={listed} old_twelve_first={old_twelve}",
        pass(listed && old_twelve)
    );
    assert!(ok_p && ok_l && listed && old_twelve, "see the CHECK lines");
}

#[test]
fn the_pattern_page_exports_the_same_bytes_after_a_save() {
    let mut ok = true;
    let mut parts = Vec::new();
    for (name, doc) in [
        ("portrait", patterns_document()),
        ("landscape", patterns_landscape_document()),
    ] {
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
            .to_bytes("patterns-test")
            .unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        let same_doc = file.doc == doc;
        let current = file.manifest.format_version == quadrule_format::FORMAT_VERSION
            && file.migrated_from.is_none();
        let mut same = true;
        for id in doc.profile_ids() {
            let target = builtin_for(id, o).unwrap();
            let dl = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
            same &= render(&dl, Math::Fastest) == design(&doc, id);
        }
        ok &= same_doc && current && same;
        parts.push(format!("{name}:doc_equal={same_doc}:export_equal={same}"));
    }
    println!(
        "\nCHECK crate=render name=patterns_save_round_trip result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
