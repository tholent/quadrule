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

//! The rung 10 repeat page (docs/bbq/2026-11-13-rung-10-repeat-tool.md, Defaulted "Golden pages"): a
//! row and a column of Auto pitch, a pitch in mm, a turned line repeated into a hatch, a habit
//! tracker (every 7th column heavier, a skipped row), a group as the source and a Fill repeat with
//! Centre (a different count on each device), drawn on each device in portrait and compared with
//! golden images in `tests/golden/`, whose sums sit after the twenty that were there before (the
//! first twelve, devices and landscape, and the patterns' four and four), which are unchanged.
//! Then: every copy is the source's pixels moved by whole pixels, a landscape repeat page exports
//! as the exact quarter turn, and a saved and reopened file exports the same. Each step prints one
//! `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test repeats`, then commit `tests/golden/`.
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("repeats");
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
fn repeat_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let (portrait, ok_p) = goldens(&repeats_document(), "repeats", "repeats_golden");
    if bless {
        bless_sums("repeats_", &portrait);
    }
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = portrait.lines().all(|l| lines.contains(&l));
    let firsts = [
        "slice_",
        "shapes_",
        "transforms_",
        "groups_",
        "devices_",
        "landscape_",
        "patterns_",
        "patterns-landscape_",
    ];
    let old_sixteen = lines.len() >= 16
        && firsts
            .iter()
            .enumerate()
            .all(|(i, k)| lines[2 * i..2 * i + 2].iter().all(|l| l.contains(k)));
    println!(
        "\nCHECK crate=render name=repeats_sums_listed result={} listed={listed} old_sixteen_first={old_sixteen}",
        pass(listed && old_sixteen)
    );
    assert!(ok_p && listed && old_sixteen, "see the CHECK lines");
}

/// The page with only the repeat `id` kept, so its pixels can be compared.
fn only(doc: &Document, id: Uuid) -> Document {
    let mut d = doc.clone();
    d.layers[0].objects.retain(|o| o.id == id);
    d
}

#[test]
fn every_copy_is_the_source_moved_by_whole_pixels() {
    // The habit tracker (a grid with a skipped row), a row of boxes and the group source (rounded
    // box, ring and star, anti-aliased), on each device. Cells are compared inside a 3 px border, so a stroke's
    // overhang into the next cell is not mistaken for a difference.
    let mut ok = true;
    let mut parts = Vec::new();
    for (name, rep) in [
        ("habit", repeat_ids::HABIT),
        ("badges", repeat_ids::GROUP),
        ("across", repeat_ids::ACROSS),
    ] {
        for id in ["supernote.a5x2", "supernote.a6x2"] {
            let doc = only(&repeats_document(), rep);
            let o = doc.page.orientation;
            let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
            let target = builtin_for(id, o).unwrap();
            let nodes = quadrule_layout::tree::resolve_nodes_env(
                &doc,
                primary.page_mm(),
                target.page_mm(),
                quadrule_layout::override_key(&primary, &target),
                Some(quadrule_layout::device_scale(&doc, &target)),
            );
            let n = nodes.iter().find(|n| n.repeat).unwrap();
            let plan = n.plan.unwrap();
            let img = design(&doc, id);
            let (sx, sy) = (plan.stride.0 as i64, plan.stride.1 as i64);
            // Cell (0, 0) starts where the source's box starts.
            let src = nodes.iter().find(|m| m.depth == 1).unwrap();
            let f = src.frame;
            let ox = (f.x_mm * target.mm_to_px(1.0)).round() as i64;
            let oy = (f.y_mm * target.mm_to_px(1.0)).round() as i64;
            let at = |x: i64, y: i64| img.data[(y as usize) * img.width as usize + x as usize];
            let rep_obj = doc.object(rep).unwrap().repeat().unwrap();
            let mut same = true;
            let mut compared = 0;
            let (mut diffs, mut worst) = (0, 0);
            for c in plan
                .copies(rep_obj)
                .filter(|c| !c.effect.skip && c.effect.line.is_none())
            {
                for dy in 3..(sy - 3) {
                    for dx in 3..(sx - 3) {
                        let a = at(ox + dx, oy + dy);
                        let b = at(ox + c.d.0 as i64 + dx, oy + c.d.1 as i64 + dy);
                        if a != b {
                            diffs += 1;
                            worst = worst.max((i32::from(a) - i32::from(b)).abs());
                        }
                        // Hard-edged boxes match exactly; the rasteriser's anti-aliasing of a curve
                        // may differ by one gray level where a path starts on another fraction.
                        same &= (i32::from(a) - i32::from(b)).abs() <= 1;
                    }
                }
                compared += 1;
            }
            // Hard-edged copies (boxes on whole pixels) are exact.
            let exact_needed = rep != repeat_ids::GROUP;
            ok &= same && compared > 1 && (!exact_needed || diffs == 0);
            parts.push(format!(
                "{name}:{id}:cells={compared}:same={same}:diff_px={diffs}:worst={worst}"
            ));
        }
    }
    println!(
        "\nCHECK crate=render name=repeat_copies_pixel_identical result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_landscape_repeat_page_exports_as_the_exact_quarter_turn() {
    let doc = repeats_landscape_document();
    let mut ok = true;
    let mut parts = Vec::new();
    for id in doc.profile_ids() {
        let upright = design(&doc, id);
        let exported = export(&doc, id);
        let turned = turn_clockwise(&upright);
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let (dw, dh) = target.device_size();
        let sizes = (exported.width, exported.height) == (dw, dh)
            && (upright.width, upright.height) == (dh, dw);
        let exact = exported == turned;
        // The design's top row lands on the device's right column.
        let top_to_right = (0..upright.width as usize)
            .all(|x| exported.at(dw - 1, x as u32) == upright.at(x as u32, 0));
        // The Fill repeat runs the longer way: more copies than on the portrait page.
        let count = |d: &Document| {
            let o = d.page.orientation;
            let primary = builtin_for(&d.primary_profile_id, o).unwrap();
            let t = builtin_for(id, o).unwrap();
            let placed = quadrule_layout::placed_objects(
                d,
                &primary,
                &t,
                &quadrule_model::outlines::ProfileOutlines {
                    profile_id: id.into(),
                    tables: vec![],
                    texts: vec![],
                    extra: Extra::new(),
                },
            );
            placed
                .iter()
                .find(|p| p.node.repeat && p.id == repeat_ids::FILL)
                .and_then(|p| p.node.plan)
                .map(|p| p.cols)
                .unwrap_or(0)
        };
        let (cl, cp) = (count(&doc), count(&repeats_document()));
        let longer = cl > cp;
        ok &= sizes && exact && top_to_right && longer;
        parts.push(format!(
            "{id}:sizes={sizes}:exact={exact}:top_to_right={top_to_right}:fill_cols={cl}>{cp}={longer}"
        ));
    }
    println!(
        "\nCHECK crate=render name=repeats_landscape_exact_turn result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_repeat_page_exports_the_same_bytes_after_a_save() {
    let mut ok = true;
    let mut parts = Vec::new();
    for (name, doc) in [
        ("portrait", repeats_document()),
        ("landscape", repeats_landscape_document()),
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
            .to_bytes("repeats-test")
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
        "\nCHECK crate=render name=repeats_save_round_trip result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
