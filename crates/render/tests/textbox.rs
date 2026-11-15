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

//! The rung 11a fonts page (docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted "Golden page 11a"):
//! every bundled family upright and italic at two sizes, a weight ramp in Inter and Literata, Atkinson
//! Hyperlegible Mono and a line of emoji, drawn on each device in portrait with the bundled fonts only
//! and compared with golden images in `tests/golden/`, whose sums sit after the twenty-two that were
//! there before (the eighteen of rung 10 and its repeats), which are unchanged. Then: the page uses
//! bundled faces only, shows italic and optical sizes, and exports the same bytes after a save.
//! Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test fonts`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

//! The rung 11b text-box page (docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted "Golden page 11b"):
//! a Fixed box that leaves lines out, the three vertical alignments, a paragraph's space, a box whose
//! baselines stand on a ruled region, and text turned 15, 90 and 180 degrees, drawn on each device in
//! portrait with the bundled fonts only and compared with golden images in `tests/golden/`, whose sums
//! sit after the twenty that were there before (rung 11a's included), which are unchanged. Then: the
//! baselines land on the very pixel rows the ruled region's lines do, on both devices; a landscape
//! page with turned text exports as the exact quarter turn of its upright drawing; and the page
//! exports the same bytes after a save. Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test textbox`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{Item, display_list, lay_out_text, text_box_origin};
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("textbox");
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
fn textbox_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let (portrait, ok_p) = goldens(&textbox_document(), "textbox", "textbox_golden");
    if bless {
        bless_sums("textbox_", &portrait);
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
        "repeats_",
        "fonts_",
    ];
    let old_twenty = lines.len() >= 20
        && firsts
            .iter()
            .enumerate()
            .all(|(i, k)| lines[2 * i..2 * i + 2].iter().all(|l| l.contains(k)));
    println!(
        "\nCHECK crate=render name=textbox_sums_listed result={} listed={listed} old_twenty_first={old_twenty}",
        pass(listed && old_twenty)
    );
    assert!(ok_p && listed && old_twenty, "see the CHECK lines");
}

/// The page's text outlines and display list on one device.
fn laid(
    doc: &Document,
    id: &str,
) -> (
    quadrule_model::outlines::ProfileOutlines,
    quadrule_layout::DisplayList,
) {
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let l = lay_out_text(doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(doc, &primary, &target, &l.outlines).unwrap();
    (l.outlines, dl)
}

/// The ruled region's rules as (top row, bottom row exclusive), from the display list.
fn rules(
    dl: &quadrule_layout::DisplayList,
    ruled: &Object,
    target: &quadrule_profiles::Profile,
) -> Vec<(f64, f64)> {
    let (top, bottom) = (
        target.mm_to_px(ruled.frame.y_mm),
        target.mm_to_px(ruled.frame.y_mm + ruled.frame.h_mm),
    );
    dl.items
        .iter()
        .filter_map(|it| match it {
            Item::Rect { x0, y0, x1, y1, .. }
                if x1 - x0 > 100.0 && *y0 >= top - 4.0 && *y1 <= bottom + 4.0 =>
            {
                Some((*y0, *y1))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn baselines_stand_on_the_rules_top_edge_or_a_small_gap_above_it_on_both_devices() {
    let doc = textbox_document();
    let mut parts = Vec::new();
    let mut all = true;
    for id in doc.profile_ids() {
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let (outlines, dl) = laid(&doc, id);
        let ppmm = target.mm_to_px(1.0);
        // B on the rule's top edge; C a gap of 0.3 mm in whole pixels, at least one, above it.
        let gap = (0.3 * ppmm).round().max(1.0);
        for (ruled_id, text_id, want_gap) in [
            (textbox_ids::RULED, textbox_ids::ON_RHYTHM, 0.0),
            (textbox_ids::RULED_GAP, textbox_ids::ON_RHYTHM_GAP, gap),
        ] {
            let o = doc.object(text_id).unwrap();
            let text = outlines.text(o.id).unwrap();
            let (origin, _) = text_box_origin(&o.frame, &target);
            let mut rows: Vec<f64> = text
                .glyphs
                .iter()
                .map(|g| origin.y + f64::from(g.y_mm()) * ppmm)
                .collect();
            rows.sort_by(f64::total_cmp);
            rows.dedup_by(|a, b| (*a - *b).abs() < 0.01);
            let lines = rules(&dl, doc.object(ruled_id).unwrap(), &target);
            let exact = rows.iter().all(|r| (r - r.round()).abs() < 1e-3);
            // The baseline is the rule's top row less the gap: the same pixel row layout draws.
            let on = !rows.is_empty()
                && rows.iter().all(|r| {
                    lines
                        .iter()
                        .any(|(y0, _)| (*y0 - want_gap - r.round()).abs() < 1e-6)
                });
            let several = rows.len() >= 2;
            all &= exact && on && several;
            parts.push(format!(
                "{id}:gap_px={want_gap}:baselines={}:rules={}:exact={exact}:on_top_edge={on}",
                rows.len(),
                lines.len()
            ));
        }
    }
    println!(
        "\nCHECK crate=render name=textbox_baselines_on_rule_top_edge result={} {}",
        pass(all),
        parts.join(" ")
    );
    assert!(all);
}

/// A ruled region with an "L" standing on it, drawn with and without the letter: the rows the letter
/// alone darkens, and the rows of the rule above which it stands.
fn pixels_of_l(id: &str, gap: bool) -> (Option<usize>, usize, f64) {
    let mut doc = textbox_document();
    let (ruled, text) = (textbox_ids::RULED, textbox_ids::ON_RHYTHM);
    let (ruled_id, text_id) = (ruled, text);
    doc.layers[0]
        .objects
        .retain(|o| o.id == ruled_id || o.id == text_id);
    if let Some(o) = doc.object_mut(text_id)
        && let ObjectKind::Text(t) = &mut o.kind
    {
        t.content = "LLL".into();
        t.baseline_gap = gap;
    }
    let with = {
        let (_, dl) = laid(&doc, id);
        (render(&dl, Math::Fastest), dl)
    };
    doc.layers[0].objects.retain(|o| o.id == ruled_id);
    let without = render(&laid(&doc, id).1, Math::Fastest);
    let (img, dl) = with;
    let w = img.width as usize;
    // Rows holding a pixel only the letters made.
    let lowest = (0..img.height as usize)
        .rev()
        .find(|y| (0..w).any(|x| img.data[y * w + x] != without.data[y * w + x]));
    let target = builtin_for(id, Orientation::Portrait).unwrap();
    // The top row of the rule the letters stand on: the first rule at or below the letters.
    let ruled_obj = doc.object(ruled_id).unwrap();
    let rule_rows = rules(&dl, ruled_obj, &target);
    let below = lowest
        .and_then(|l| {
            rule_rows
                .iter()
                .map(|(y0, _)| *y0)
                .filter(|y0| *y0 > l as f64)
                .min_by(f64::total_cmp)
        })
        .unwrap_or(f64::NAN);
    (lowest, w, below)
}

#[test]
fn a_letters_flat_bottom_ends_on_the_row_above_the_rule_or_a_clear_gap_above_that() {
    let mut parts = Vec::new();
    let mut all = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let n = (0.3 * target.mm_to_px(1.0)).round().max(1.0);
        // B: the letters' lowest dark row is the row just above the rule's top row.
        let (low_b, _, rule_b) = pixels_of_l(id, false);
        let b_ok = low_b.is_some_and(|l| (rule_b - (l as f64 + 1.0)).abs() < 1e-6);
        // C: exactly N clear rows between the letters and the rule.
        let (low_c, _, rule_c) = pixels_of_l(id, true);
        let c_ok = low_c.is_some_and(|l| (rule_c - (l as f64 + 1.0) - n).abs() < 1e-6);
        all &= b_ok && c_ok;
        parts.push(format!(
            "{id}:B_clear_rows={}:C_clear_rows={}:N={n}:b_ok={b_ok}:c_ok={c_ok}",
            low_b.map_or(-1.0, |l| rule_b - (l as f64 + 1.0)),
            low_c.map_or(-1.0, |l| rule_c - (l as f64 + 1.0)),
        ));
    }
    println!(
        "\nCHECK crate=render name=textbox_letter_bottom_rows result={} {}",
        pass(all),
        parts.join(" ")
    );
    assert!(all);
}

#[test]
fn a_landscape_page_with_turned_text_exports_as_the_exact_quarter_turn() {
    let mut doc = landscape_document();
    // Turned text on the landscape page: 15 and 90 degrees, as the portrait page has.
    let base = doc
        .objects()
        .find(|o| matches!(o.kind, ObjectKind::Text(_)))
        .unwrap()
        .clone();
    for (i, deg) in [15.0, 90.0].into_iter().enumerate() {
        let mut t = base.clone();
        t.id = textbox_ids::nth(500 + i as u128);
        t.name = format!("Turned {deg}");
        t.frame = Frame::new(40.0 + i as f64 * 45.0, 60.0, 30.0, 6.0);
        t.rotation_deg = deg;
        if let ObjectKind::Text(tb) = &mut t.kind {
            tb.content = "Quartz".into();
        }
        doc.layers.last_mut().unwrap().objects.push(t);
    }
    let mut all = true;
    let mut parts = Vec::new();
    for id in doc.profile_ids() {
        let target = builtin_for(id, Orientation::Landscape).unwrap();
        let (_, dl) = laid(&doc, id);
        let upright = render(&dl, Math::Fastest);
        let turned = turn_clockwise(&upright);
        let (w, h) = (upright.width as usize, upright.height as usize);
        let (dw, dh) = target.device_size();
        let sized = (turned.width, turned.height) == (dw, dh)
            && (turned.width as usize, turned.height as usize) == (h, w);
        // The pixel (x, y) of the design is at (h - 1 - y, x) on the device: nothing blurred, nothing lost.
        let exact = (0..h)
            .all(|y| (0..w).all(|x| turned.data[x * h + (h - 1 - y)] == upright.data[y * w + x]));
        // And the turned text is drawn at all.
        let drawn = dl
            .items
            .iter()
            .filter(|it| matches!(it, Item::Fill { .. }))
            .count()
            >= 3;
        all &= sized && exact && drawn;
        parts.push(format!("{id}:sized={sized} exact_quarter_turn={exact}"));
    }
    println!(
        "\nCHECK crate=render name=textbox_landscape_quarter_turn result={} {}",
        pass(all),
        parts.join(" ")
    );
    assert!(all);
}

#[test]
fn the_page_exports_the_same_bytes_after_a_save() {
    let doc = textbox_document();
    let mut ok = true;
    let mut outlines = Vec::new();
    for id in doc.profile_ids() {
        outlines.push(laid(&doc, id).0);
    }
    let file = QuadFile::new(doc.clone(), outlines, vec![]);
    let again = QuadFile::from_bytes(&file.to_bytes("test").unwrap()).unwrap();
    ok &= again.doc == doc;
    for id in doc.profile_ids() {
        let primary = builtin_for(&doc.primary_profile_id, Orientation::Portrait).unwrap();
        let target = builtin_for(id, Orientation::Portrait).unwrap();
        let from_file = display_list(&again.doc, &primary, &target, &again.outlines[id]).unwrap();
        let (_, live) = laid(&doc, id);
        ok &= encode_png(&render(&from_file, Math::Fastest), None)
            == encode_png(&render(&live, Math::Fastest), None);
    }
    println!(
        "\nCHECK crate=render name=textbox_saved_page_exports_the_same result={}",
        pass(ok)
    );
    assert!(ok);
}
