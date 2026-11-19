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

//! The rung 13b paper pages (docs/bbq/2026-11-17-rung-13-profile-system.md, Q12 to Q14, Defaulted "13b goldens"):
//! ISO A4 portrait at 300 PPI and, on the same page, ISO A5 at 600 PPI; and US Letter held sideways at 300 PPI,
//! exported wide, not turned. Compared with golden images in `tests/golden/` (`paper_*`, `paper-landscape_*`),
//! whose sums sit after the twenty-six that were there before, which are unchanged. Then: the pixel size of
//! every row of napkin §3.3.1's table, a page's PPI changes its pixel size and not its design, a landscape paper
//! page is a wide PNG with no turn, and a saved and reopened file keeps each page's PPI and exports the same. Each
//! step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test paper`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{builtin_at, builtin_paper_ids};
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("paper");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// Paper profile `id` as `doc`'s page sees it: at the PPI the page picked for it, held the way the page is held.
fn profile(doc: &Document, id: &str) -> quadrule_profiles::Profile {
    builtin_at(id, doc.page.orientation, doc.ppi_of(id)).unwrap()
}

/// The design on one paper size, drawn at the size it exports at (a paper page is not turned).
fn design(doc: &Document, id: &str) -> Gray {
    let primary = profile(doc, &doc.primary_profile_id);
    let target = profile(doc, id);
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(doc, &primary, &target, &laid.outlines).unwrap();
    render(&dl)
}

/// Draws each page, compares it with its golden image and returns the sums lines.
fn goldens(prefix: &str, doc: &Document, bless: bool) -> (bool, String) {
    let mut all_ok = true;
    let mut sums = String::new();
    for id in doc.profile_ids() {
        let img = design(doc, id);
        let target = profile(doc, id);
        let (dw, dh) = target.device_size();
        let sizes = (img.width, img.height) == (dw, dh) && !target.turned;
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
            "\nCHECK crate=render name=paper_golden page={prefix} profile={id} ppi={} result={} sizes={sizes} px={dw}x{dh} sha256={sha} golden_sha256={} os={} arch={}",
            doc.ppi_of(id),
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
    (all_ok, sums)
}

#[test]
fn paper_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let (ok_a, sums_a) = goldens("paper", &paper_document(), bless);
    let (ok_b, sums_b) = goldens("paper-landscape", &paper_landscape_document(), bless);
    let sums = format!("{sums_a}{sums_b}");
    let path = golden_dir().join("SHA256SUMS");
    if bless {
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with("paper_") || n.starts_with("paper-landscape_"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    let file = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    // The twenty-six lines that were there come first, in their order.
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
        "textbox_",
        "patterns-turned_",
        "pattern-angle_",
    ];
    let old_twenty_six = lines.len() >= 26
        && firsts
            .iter()
            .enumerate()
            .all(|(i, k)| lines[2 * i..2 * i + 2].iter().all(|l| l.contains(k)));
    let three = sums.lines().count() == 3;
    println!(
        "\nCHECK crate=render name=paper_sums_listed result={} listed={listed} new_lines={} old_twenty_six_first={old_twenty_six}",
        pass(listed && old_twenty_six && three),
        sums.lines().count()
    );
    assert!(
        ok_a && ok_b && listed && old_twenty_six && three,
        "see the CHECK lines"
    );
}

#[test]
fn every_row_of_the_papers_table_has_its_pixel_size_at_300_ppi() {
    // Napkin §3.3.1's table (and Tabloid and A3, which the rung 13 record adds).
    let table = [
        ("paper.a4", 2480, 3508),
        ("paper.a5", 1748, 2480),
        ("paper.a6", 1240, 1748),
        ("paper.b5-iso", 2079, 2953),
        ("paper.b5-jis", 2150, 3035),
        ("paper.letter", 2550, 3300),
        ("paper.legal", 2550, 4200),
        ("paper.half-letter", 1650, 2550),
        ("paper.executive", 2175, 3150),
        ("paper.tabloid", 3300, 5100),
        ("paper.a3", 3508, 4961),
    ];
    let mut fails = Vec::new();
    for (id, w, h) in table {
        let p = builtin_at(id, Orientation::Portrait, 300).unwrap();
        if (p.width_px, p.height_px) != (w, h) || p.ppi != 300.0 {
            fails.push(format!("{id}:{}x{}", p.width_px, p.height_px));
        }
    }
    let all = builtin_paper_ids().count() == table.len()
        && table
            .iter()
            .all(|(id, _, _)| builtin_paper_ids().any(|b| b == *id));
    let ok = fails.is_empty() && all;
    println!(
        "CHECK crate=render name=paper_table_300 result={} rows={} fails={fails:?}",
        pass(ok),
        table.len()
    );
    assert!(ok);
}

#[test]
fn a_pages_ppi_changes_the_pixels_and_the_line_widths_and_not_the_design() {
    let mut doc = paper_document();
    let a5_600 = design(&doc, PAPER_A5_ID);
    doc.profiles
        .iter_mut()
        .find(|c| c.profile_id == PAPER_A5_ID)
        .unwrap()
        .ppi = 300;
    let a5_300 = design(&doc, PAPER_A5_ID);
    let sizes = (a5_600.width, a5_600.height) == (3496, 4961)
        && (a5_300.width, a5_300.height) == (1748, 2480);
    // The same physical page: the 600 PPI image is about twice the 300 one on each side.
    let twice = a5_600.width.abs_diff(2 * a5_300.width) <= 1
        && a5_600.height.abs_diff(2 * a5_300.height) <= 1;
    // The guide role is 1 px at 300 and 2 at 600; the emphasis role 3 and 6.
    let w300 = builtin_at(PAPER_A5_ID, Orientation::Portrait, 300).unwrap();
    let w600 = builtin_at(PAPER_A5_ID, Orientation::Portrait, 600).unwrap();
    let w150 = builtin_at(PAPER_A5_ID, Orientation::Portrait, 150).unwrap();
    let w200 = builtin_at(PAPER_A5_ID, Orientation::Portrait, 200).unwrap();
    let widths = |p: &quadrule_profiles::Profile| {
        let r = &p.line_roles;
        [
            r.whisper.width_px,
            r.guide.width_px,
            r.structure.width_px,
            r.emphasis.width_px,
            r.dot_grid.width_px,
        ]
    };
    let roles = widths(&w300) == [1, 1, 2, 3, 3]
        && widths(&w600) == [2, 2, 4, 6, 6]
        && widths(&w150) == [1, 1, 1, 2, 2]
        && widths(&w200) == [1, 1, 1, 2, 2];
    let ok = sizes && twice && roles;
    println!(
        "CHECK crate=render name=paper_ppi_per_page result={} sizes={sizes} twice={twice} roles={roles}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_landscape_paper_page_is_a_wide_png_not_turned() {
    let doc = paper_landscape_document();
    let p = profile(&doc, PAPER_LETTER_ID);
    let img = design(&doc, PAPER_LETTER_ID);
    let png = encode_png(&img, None);
    // The PNG header (signature, then IHDR) holds the width and height as big-endian numbers.
    let be = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
    let header = (be(16), be(20));
    let a4 = builtin_at("paper.a4", Orientation::Landscape, 300).unwrap();
    let ok = !p.turned
        && p.device_size() == (3300, 2550)
        && (img.width, img.height) == (3300, 2550)
        && header == (3300, 2550)
        && (a4.width_px, a4.height_px) == (3508, 2480)
        && a4.device_size() == (3508, 2480);
    println!(
        "CHECK crate=render name=paper_landscape_wide result={} png={}x{} a4_landscape={}x{}",
        pass(ok),
        header.0,
        header.1,
        a4.width_px,
        a4.height_px
    );
    assert!(ok);
}

#[test]
fn the_paper_pages_draw_the_same_twice_and_a_saved_file_keeps_each_ppi() {
    let mut same = true;
    let mut kept = true;
    for doc in [paper_document(), paper_landscape_document()] {
        for id in doc.profile_ids() {
            same &= design(&doc, id) == design(&doc, id);
        }
        let primary = profile(&doc, &doc.primary_profile_id);
        let mut reg = FontRegistry::bundled_only();
        let outlines: Vec<_> = doc
            .profile_ids()
            .into_iter()
            .map(|id| {
                lay_out_text(&doc, &primary, &profile(&doc, id), &mut reg)
                    .unwrap()
                    .outlines
            })
            .collect();
        let bytes = QuadFile::new(doc.clone(), outlines, vec![])
            .to_bytes("paper-test")
            .unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        kept &= file.doc == doc
            && file.migrated_from.is_none()
            && file.manifest.format_version == quadrule_format::FORMAT_VERSION
            // Unedited built-in papers are not carried.
            && file.profiles.is_empty();
        for id in doc.profile_ids() {
            kept &= file.doc.ppi_of(id) == doc.ppi_of(id);
            let dl = display_list(
                &file.doc,
                &primary,
                &profile(&file.doc, id),
                &file.outlines[id],
            )
            .unwrap();
            kept &= render(&dl) == design(&doc, id);
        }
    }
    let a5_600 = paper_document().ppi_of(PAPER_A5_ID) == 600;
    let ok = same && kept && a5_600;
    println!(
        "CHECK crate=render name=paper_repeatable_and_saved result={} same_twice={same} saved_keeps_ppi_and_export={kept} a5_at_600={a5_600}",
        pass(ok)
    );
    assert!(ok);
}
