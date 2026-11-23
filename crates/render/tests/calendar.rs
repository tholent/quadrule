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

//! The calendar page (docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md, Defaulted "New golden pages"): a
//! dated calendar pattern, an undated one and two mini calendars, drawn on each device in portrait and compared with
//! golden images in `tests/golden/` (`calendar_*`), whose sums sit after the thirty-four that were there before, which
//! are unchanged. Then: the same document drawn again is the same bytes, the page with its calendars' words taken out
//! of the outlines draws differently (the words are drawn), and a saved and reopened file exports the same (the words
//! come from the saved outlines). Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test calendar`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use quadrule_text::FontRegistry;
use std::path::PathBuf;

const PREFIX: &str = "calendar";

/// The sha256 of the thirty-four lines of `SHA256SUMS` that were there before this page (each ending in a newline).
const OLD_34_SHA256: &str = "de17f2970ebe132a473578d2ec1ab7b96eacb884d89e269c0a92f8e2ddae8d8e";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("calendar");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// The page's text outlines on one device.
fn outlines(doc: &Document, id: &str) -> ProfileOutlines {
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let mut reg = FontRegistry::bundled_only();
    lay_out_text(doc, &primary, &target, &mut reg)
        .unwrap()
        .outlines
}

/// The design on one device, drawn from `outlines`.
fn draw(doc: &Document, id: &str, outlines: &ProfileOutlines) -> Gray {
    let o = doc.page.orientation;
    let primary = builtin_for(&doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let dl = display_list(doc, &primary, &target, outlines).unwrap();
    render(&dl)
}

/// The design on one device.
fn design(doc: &Document, id: &str) -> Gray {
    draw(doc, id, &outlines(doc, id))
}

#[test]
fn calendar_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = calendar_document();
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
            "\nCHECK crate=render name=calendar_golden profile={id} result={} sizes={sizes} sha256={sha} golden_sha256={} os={} arch={}",
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
    // The thirty-four lines that were there come first, byte for byte.
    let first: String = lines.iter().take(34).map(|l| format!("{l}\n")).collect();
    let old_34 = lines.len() >= 34 && sha256_hex(first.as_bytes()) == OLD_34_SHA256;
    println!(
        "\nCHECK crate=render name=calendar_sums_listed result={} listed={listed} old_34_first_and_unchanged={old_34}",
        pass(listed && old_34)
    );
    assert!(all_ok && listed && old_34, "see the CHECK lines");
}

#[test]
fn the_calendar_page_draws_the_same_twice_and_draws_its_words() {
    let doc = calendar_document();
    let mut same = true;
    let mut words = true;
    for id in doc.profile_ids() {
        let full = outlines(&doc, id);
        let a = draw(&doc, id, &full);
        same &= a == design(&doc, id);
        // Only the page's own text boxes: the calendars' words gone.
        let mut bare = full.clone();
        bare.texts.retain(|t| doc.object(t.object_id).is_some());
        words &= bare.texts.len() < full.texts.len() && a != draw(&doc, id, &bare);
    }
    println!(
        "\nCHECK crate=render name=calendar_repeatable result={} same_twice={same} words_drawn={words}",
        pass(same && words)
    );
    assert!(same && words);
}

#[test]
fn the_calendar_page_exports_the_same_bytes_after_a_save() {
    let doc = calendar_document();
    let laid: Vec<_> = doc
        .profile_ids()
        .into_iter()
        .map(|id| outlines(&doc, id))
        .collect();
    let bytes = QuadFile::new(doc.clone(), laid, vec![])
        .to_bytes("calendar-test")
        .unwrap();
    let file = QuadFile::from_bytes(&bytes).unwrap();
    let same_doc = file.doc == doc;
    let current = file.manifest.format_version == quadrule_format::FORMAT_VERSION
        && file.migrated_from.is_none();
    let again = file.to_bytes("calendar-test").unwrap();
    let bytes_stable = again == bytes;
    let mut same = true;
    for id in doc.profile_ids() {
        // Drawn from the saved outlines alone: the calendars' words were saved with the page's text.
        same &= draw(&file.doc, id, &file.outlines[id]) == design(&doc, id);
    }
    let ok = same_doc && current && bytes_stable && same;
    println!(
        "\nCHECK crate=render name=calendar_save_round_trip result={} doc_equal={same_doc} current={current} export_equal={same} bytes_stable={bytes_stable}",
        pass(ok)
    );
    assert!(ok);
}
