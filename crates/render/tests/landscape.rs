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

//! The rung 8b landscape page (docs/bbq/2026-11-12-rung-8-several-devices.md, Q6 and Defaulted
//! "8b, orientation", "Goldens (8b)", "Tests"): designed held sideways, drawn at the turned size and
//! turned a quarter turn clockwise into each device's portrait PNG, compared with golden images in
//! `tests/golden/`, whose sums sit after the ten that were there before (slice, shapes, transforms,
//! groups and devices, two each), which are unchanged. Then: the turn is an exact, clockwise pixel
//! turn (the design's top lands on the device's right edge), and orientation survives a save. Each
//! step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test landscape`, then commit `tests/golden/`.
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("landscape");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// The design on one device, upright (the turned size), text laid out from the bundled fonts.
fn design(doc: &Document, id: &str) -> Gray {
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(doc, &primary, &target, &laid.outlines).unwrap();
    render(&dl, Math::Fastest)
}

#[test]
fn landscape_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = landscape_document();
    let mut all_ok = true;
    let mut sums = String::new();
    for id in doc.profile_ids() {
        let target = builtin_for(id, Orientation::Landscape).unwrap();
        let upright = design(&doc, id);
        let img = turn_clockwise(&upright);
        let (dw, dh) = target.device_size();
        // The PNG is the device's own portrait size; the design was the turned one.
        let sizes = (img.width, img.height) == (dw, dh)
            && (upright.width, upright.height) == (target.width_px, target.height_px)
            && upright.width > upright.height;
        let png = encode_png(&img, None);
        let name = format!("landscape_{id}_{dw}x{dh}.png");
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
            "\nCHECK crate=render name=landscape_golden profile={id} result={} sizes={sizes} sha256={sha} golden_sha256={} os={} arch={}",
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
    if bless {
        let path = golden_dir().join("SHA256SUMS");
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with("landscape_"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    let firsts = ["slice_", "shapes_", "transforms_", "groups_", "devices_"];
    let old_ten = lines.len() >= 10
        && firsts
            .iter()
            .enumerate()
            .all(|(i, k)| lines[2 * i..2 * i + 2].iter().all(|l| l.contains(k)));
    println!(
        "\nCHECK crate=render name=landscape_sums_listed result={} listed={listed} old_ten_first={old_ten}",
        pass(listed && old_ten)
    );
    assert!(all_ok && listed && old_ten, "see the CHECK lines");
}

#[test]
fn the_turn_is_an_exact_clockwise_quarter_turn() {
    // A 3 by 2 image with every pixel different.
    let img = Gray {
        width: 3,
        height: 2,
        data: vec![10, 20, 30, 40, 50, 60],
    };
    let t = turn_clockwise(&img);
    // Turned clockwise: the old left column, read bottom up, is the new top row.
    let exact = (t.width, t.height) == (2, 3) && t.data == vec![40, 10, 50, 20, 60, 30];
    // The design's top row lands on the device's right column, left to right becoming top to
    // bottom; its top-left corner is the device's top-right corner.
    let top_to_right = (0..3).all(|x| t.at(1, x) == img.at(x, 0));
    // Four turns give the image back; nothing is resampled.
    let four = turn_clockwise(&turn_clockwise(&turn_clockwise(&t)));
    let identity = four == img;
    // On the real page: the histogram is unchanged (a permutation of the pixels).
    let doc = landscape_document();
    let upright = design(&doc, MANTA_ID);
    let turned = turn_clockwise(&upright);
    let mut a = upright.data.clone();
    let mut b = turned.data.clone();
    a.sort_unstable();
    b.sort_unstable();
    let permutation = a == b;
    let ok = exact && top_to_right && identity && permutation;
    println!(
        "\nCHECK crate=render name=turn_clockwise_exact result={} small_exact={exact} top_to_right_edge={top_to_right} four_turns_identity={identity} page_permutation={permutation}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn orientation_round_trips_through_save() {
    let doc = landscape_document();
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
        .to_bytes("landscape-test")
        .unwrap();
    let file = QuadFile::from_bytes(&bytes).unwrap();
    let same_doc = file.doc == doc;
    let landscape = file.doc.page.orientation == Orientation::Landscape;
    let current = file.manifest.format_version == quadrule_format::FORMAT_VERSION
        && file.migrated_from.is_none();
    let mut export_same = true;
    for id in doc.profile_ids() {
        let target = builtin_for(id, o).unwrap();
        let saved = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
        export_same &= render(&saved, Math::Fastest) == design(&doc, id);
    }
    let ok = same_doc && landscape && current && export_same;
    println!(
        "\nCHECK crate=render name=orientation_round_trip result={} document_equal={same_doc} landscape={landscape} saved_as_current={current} export_equal={export_same}",
        pass(ok)
    );
    assert!(ok);
}
