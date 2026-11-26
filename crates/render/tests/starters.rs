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

//! The starter pages' golden images (rung 16b bbq, Q8: "each new starter adds goldens on both tablets"). Every
//! `.quad` file in `crates/ui/starters/<category>/` is opened as a file, drawn from the outlines it saved on each of its
//! devices in portrait, and compared with `tests/golden/starter-<slug>_<profile>_<w>x<h>.png`, whose sums follow the forty
//! lines of `SHA256SUMS` that were there before the starters, which stay unchanged. A starter with no golden fails
//! here, so a new starter cannot ship without its two images. Then: every starter draws the same twice and a saved and
//! reopened copy exports the same bytes. Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye, never before the lead has looked):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test starters`, then commit `tests/golden/`.
//! Blessing only replaces the starters' own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::display_list;
use quadrule_model::Orientation;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use std::path::PathBuf;

/// The sha256 of the forty lines of `SHA256SUMS` that were there before the starters (each ending in a newline).
const OLD_40_SHA256: &str = "c4cb23ad5b965f025281726fbaac3bf6332e167f514b4d813683b1aa2dc2c9e1";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn starters_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/starters")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("starters");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// Every starter on disk: (golden prefix `starter-<slug>`, file bytes), by category folder and then by name.
fn starters() -> Vec<(String, Vec<u8>)> {
    let mut found = Vec::new();
    for category in std::fs::read_dir(starters_dir()).unwrap().flatten() {
        if !category.path().is_dir() {
            continue;
        }
        for file in std::fs::read_dir(category.path()).unwrap().flatten() {
            let path = file.path();
            if path.extension().is_some_and(|x| x == "quad") {
                let slug = path.file_stem().unwrap().to_string_lossy().into_owned();
                found.push((format!("starter-{slug}"), std::fs::read(&path).unwrap()));
            }
        }
    }
    found.sort();
    found
}

/// The PNG a file's page gives on one device, drawn from the outlines saved in the file.
fn export(file: &QuadFile, id: &str) -> Vec<u8> {
    let o = file.doc.page.orientation;
    let primary = builtin_for(&file.doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let dl = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
    let img = render(&dl);
    let img = if o == Orientation::Landscape {
        turn_clockwise(&img)
    } else {
        img
    };
    encode_png(&img, None)
}

#[test]
fn starter_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let all = starters();
    assert!(!all.is_empty(), "no starters found");
    let mut sums = String::new();
    let mut all_ok = true;
    for (prefix, bytes) in &all {
        let file = QuadFile::from_bytes(bytes).unwrap();
        for id in file.doc.profile_ids() {
            let png = export(&file, id);
            let target = builtin_for(id, Orientation::Portrait).unwrap();
            let (dw, dh) = target.device_size();
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
            all_ok &= matches;
            println!(
                "\nCHECK crate=render name=starter_golden page={prefix} profile={id} result={} sha256={sha} golden_sha256={} os={} arch={}",
                pass(matches),
                if golden.is_empty() {
                    "none".to_string()
                } else {
                    sha256_hex(&golden)
                },
                std::env::consts::OS,
                std::env::consts::ARCH
            );
        }
    }
    let path = golden_dir().join("SHA256SUMS");
    if bless {
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with("starter-"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    let file = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    // The forty lines that were there come first, byte for byte.
    let first: String = lines.iter().take(40).map(|l| format!("{l}\n")).collect();
    let old_40 = lines.len() >= 40 && sha256_hex(first.as_bytes()) == OLD_40_SHA256;
    // Nothing of a starter that is gone is left listed.
    let extra = lines.iter().skip(40).all(|l| sums.lines().any(|s| s == *l));
    println!(
        "\nCHECK crate=render name=starter_sums_listed starters={} lines={} result={} listed={listed} old_40_first_and_unchanged={old_40} no_extra_lines={extra}",
        all.len(),
        lines.len(),
        pass(listed && old_40 && extra)
    );
    assert!(all_ok && listed && old_40 && extra, "see the CHECK lines");
}

#[test]
fn every_starter_draws_the_same_twice_and_exports_the_same_after_a_save() {
    let (mut same, mut resaved, mut sized) = (true, true, true);
    let mut pages = 0;
    for (prefix, bytes) in starters() {
        let file = QuadFile::from_bytes(bytes.as_slice()).unwrap();
        let again = QuadFile::from_bytes(&file.to_bytes("starters-test").unwrap()).unwrap();
        resaved &= again.doc == file.doc && again.previews == file.previews;
        for id in file.doc.profile_ids() {
            let a = export(&file, id);
            same &= a == export(&file, id);
            resaved &= a == export(&again, id);
            let target = builtin_for(id, Orientation::Portrait).unwrap();
            let img = png_size(&a);
            sized &= img == Some(target.device_size());
            pages += 1;
        }
        let _ = prefix;
    }
    let ok = same && resaved && sized && pages > 0;
    println!(
        "\nCHECK crate=render name=starter_repeatable result={} pages={pages} same_twice={same} resaved_exports_same={resaved} device_sized={sized}",
        pass(ok)
    );
    assert!(ok);
}

/// The width and height a PNG says it has.
fn png_size(png: &[u8]) -> Option<(u32, u32)> {
    let w = u32::from_be_bytes(png.get(16..20)?.try_into().ok()?);
    let h = u32::from_be_bytes(png.get(20..24)?.try_into().ok()?);
    Some((w, h))
}
