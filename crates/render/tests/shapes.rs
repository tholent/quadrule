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

//! The rung 6a shapes page (docs/bbq/2026-11-10-rung-6-shapes.md, Defaulted "Goldens"): every new
//! kind, both fill roles, a custom fill, each dash preset and each cap, exported on both devices
//! and compared with golden images in `tests/golden/`, whose sums sit after rung 1's two in
//! `SHA256SUMS`. Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test shapes`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::sha256_hex;
use quadrule_layout::{Item, display_list};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_render::*;
use std::collections::BTreeSet;
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("shapes");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

fn empty_outlines(id: &str) -> quadrule_model::outlines::ProfileOutlines {
    quadrule_model::outlines::ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn page(id: &str, doc: &Document) -> quadrule_layout::DisplayList {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    // No text on the page, so empty outlines do.
    let outlines = empty_outlines(&target.id);
    display_list(doc, &primary, &target, &outlines).unwrap()
}

#[test]
fn shapes_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = shapes_document();
    let mut all_ok = true;
    let mut sums = String::new();
    for id in doc.profile_ids() {
        let target = builtin(id).unwrap();
        let dl = page(id, &doc);
        let png = encode_png(&render(&dl, Math::Fastest), None);
        let portable = encode_png(&render(&dl, Math::Portable), None);
        // ADR-003 item 2's lever, reported and not enforced here: on x86 CI the portable maths
        // differ from the fastest on the shapes page's anti-aliased edges (the exported golden,
        // made with the fastest, matches on both machines), so the count is only printed.
        let same_maths = png == portable;
        let (d_px, d_max) = {
            let (a, b) = (
                render(&dl, Math::Fastest).data,
                render(&dl, Math::Portable).data,
            );
            let n = a.iter().zip(&b).filter(|(x, y)| x != y).count();
            let m = a
                .iter()
                .zip(&b)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            (n, m)
        };
        let name = format!("shapes_{id}_{}x{}.png", target.width_px, target.height_px);
        std::fs::write(out_dir().join(&name), &png).unwrap();
        let sha = sha256_hex(&png);
        sums.push_str(&format!("{sha}  {name}\n"));
        let golden_path = golden_dir().join(&name);
        if bless {
            std::fs::write(&golden_path, &png).unwrap();
        }
        let golden = std::fs::read(&golden_path).unwrap_or_default();
        let matches = golden == png;
        all_ok &= matches;
        println!(
            "\nCHECK crate=render name=shapes_golden profile={id} result={} sha256={sha} golden_sha256={} simd_equals_portable={same_maths} portable_diff_px={d_px} portable_max_diff={d_max} items={} os={} arch={}",
            pass(matches),
            if golden.is_empty() {
                "none".to_string()
            } else {
                sha256_hex(&golden)
            },
            dl.items.len(),
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }
    if bless {
        // Keep every line that is not this test's, in its place; add this test's after them.
        let path = golden_dir().join("SHA256SUMS");
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with("shapes_"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    // The sums file holds them too, after rung 1's two.
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    let first_two = lines.iter().take(2).all(|l| l.contains("slice_"));
    println!(
        "\nCHECK crate=render name=shapes_sums_listed result={} listed={listed} rung1_lines_first={first_two}",
        pass(listed && first_two)
    );
    assert!(all_ok && listed && first_two, "see the CHECK lines");
}

#[test]
fn square_rectangles_are_hard_edged() {
    // Q2: an unrotated rectangle with square corners draws as whole-pixel rectangles, so every
    // pixel is the background, the stroke's gray or the fill's.
    let mut parts = Vec::new();
    let mut ok = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let primary = builtin(MANTA_ID).unwrap();
        let target = builtin(id).unwrap();
        let mut doc = shapes_document();
        // Only the plain rectangles: square, Shade or Block, with and without stroke.
        doc.layers[1].objects.retain(|o| {
            matches!(&o.kind, ObjectKind::Rect(r) if r.corner_radius_mm == 0.0) && o.dash.is_solid()
        });
        doc.layers.remove(0);
        let outlines = empty_outlines(&target.id);
        let dl = display_list(&doc, &primary, &target, &outlines).unwrap();
        let only_rects = dl.items.iter().all(|i| matches!(i, Item::Rect { .. }));
        let whole = dl.items.iter().all(|i| match i {
            Item::Rect { x0, y0, x1, y1, .. } => [x0, y0, x1, y1].iter().all(|v| v.fract() == 0.0),
            _ => false,
        });
        let mut grays: BTreeSet<u8> = dl
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Rect { gray, .. } => Some(*gray),
                _ => None,
            })
            .collect();
        grays.insert(dl.background);
        let img = render(&dl, Math::Fastest);
        let stray = img.data.iter().filter(|v| !grays.contains(v)).count();
        ok &= only_rects && whole && stray == 0 && !dl.items.is_empty();
        parts.push(format!(
            "{id}:items={}:only_rects={only_rects}:whole_px={whole}:stray_px={stray}",
            dl.items.len()
        ));
    }
    println!(
        "\nCHECK crate=render name=square_rect_hard_edged result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
