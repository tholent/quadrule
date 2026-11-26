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

//! The major-lines-from page (ADR-023; docs/bbq/2026-11-23-major-lines-from-top.md, Defaulted): Top, Centre and
//! Rhythm origin side by side on lines, a grid, dots, Cornell, isometric lines and a bordered grid; and the
//! major-lines-per-axis page (ADR-023 as amended 2026-11-23): a grid, dots and isometric lines with their rows and
//! columns counted from different places. Each is drawn on each device in portrait and compared with golden images
//! in `tests/golden/` (`major-lines-from_*`, `major-lines-per-axis_*`), whose sums sit after the thirty-six that were
//! there before, which are unchanged. Then: the same document drawn again is the same bytes, the page with every
//! region set to Rhythm origin draws differently (the choice is drawn), and a saved and reopened file exports the
//! same and writes `major_rows_from` and `major_columns_from` each only where it is not Rhythm origin. Each step
//! prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test major_lines_from`, then commit `tests/golden/`.
//! Blessing only replaces the blessed page's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use quadrule_text::FontRegistry;
use std::path::PathBuf;

/// Each page, by the prefix of its golden images.
fn pages() -> [(&'static str, Document); 2] {
    [
        ("major-lines-from", majors_document()),
        ("major-lines-per-axis", majors_axes_document()),
    ]
}

/// The sha256 of the thirty-six lines of `SHA256SUMS` that were there before this page (each ending in a newline).
const OLD_36_SHA256: &str = "6e8d15a865d0d192f196816d9ae8f2b47948f11e7646ebbf83cbdfe91d196b6a";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("major_lines_from");
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

/// The same page with every region counting from the rhythm origin.
fn all_from_origin(doc: &Document) -> Document {
    let mut d = doc.clone();
    let ids: Vec<Uuid> = d.objects().map(|o| o.id).collect();
    for id in ids {
        if let Some(o) = d.object_mut(id)
            && let ObjectKind::Pattern(pr) = &mut o.kind
        {
            pr.major_rows_from = MajorFrom::RhythmOrigin;
            pr.major_columns_from = MajorFrom::RhythmOrigin;
        }
    }
    d
}

#[test]
fn major_lines_from_page_goldens() {
    page_goldens("major-lines-from", &majors_document());
}

#[test]
fn major_lines_per_axis_page_goldens() {
    page_goldens("major-lines-per-axis", &majors_axes_document());
}

/// Draws `doc` on each device and compares it with its golden images, named from `prefix`.
fn page_goldens(prefix: &str, doc: &Document) {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let mut sums = String::new();
    let mut all_ok = true;
    for id in doc.profile_ids() {
        let img = design(doc, id);
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
            "\nCHECK crate=render name=major_lines_from_golden page={prefix} profile={id} result={} sizes={sizes} sha256={sha} golden_sha256={} os={} arch={}",
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
                    .is_some_and(|n| n.starts_with(&format!("{prefix}_")))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    let file = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    // The thirty-six lines that were there come first, byte for byte.
    let first: String = lines.iter().take(36).map(|l| format!("{l}\n")).collect();
    let old_36 = lines.len() >= 36 && sha256_hex(first.as_bytes()) == OLD_36_SHA256;
    println!(
        "\nCHECK crate=render name=major_lines_from_sums_listed page={prefix} result={} listed={listed} old_36_first_and_unchanged={old_36}",
        pass(listed && old_36)
    );
    assert!(all_ok && listed && old_36, "see the CHECK lines");
}

#[test]
fn the_major_lines_from_page_draws_the_same_twice_and_draws_the_choice() {
    let mut same = true;
    let mut differs = true;
    for (_, doc) in pages() {
        let origin = all_from_origin(&doc);
        for id in doc.profile_ids() {
            let a = design(&doc, id);
            same &= a == design(&doc, id);
            differs &= a != design(&origin, id);
        }
    }
    println!(
        "\nCHECK crate=render name=major_lines_from_repeatable result={} same_twice={same} differs_from_all_rhythm_origin={differs}",
        pass(same && differs)
    );
    assert!(same && differs);
}

#[test]
fn the_major_lines_from_page_exports_the_same_bytes_after_a_save() {
    let (mut same_doc, mut current, mut bytes_stable, mut same, mut key_right) =
        (true, true, true, true, true);
    // Each field is written for Top and Centre, and left out for Rhythm origin (what a file without it reads as).
    let (mut written, mut left_out) = (0, 0);
    for (prefix, doc) in pages() {
        let laid: Vec<_> = doc
            .profile_ids()
            .into_iter()
            .map(|id| outlines(&doc, id))
            .collect();
        let bytes = QuadFile::new(doc.clone(), laid, vec![])
            .to_bytes(prefix)
            .unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        same_doc &= file.doc == doc;
        current &= file.manifest.format_version == quadrule_format::FORMAT_VERSION
            && file.migrated_from.is_none();
        bytes_stable &= file.to_bytes(prefix).unwrap() == bytes;
        for id in doc.profile_ids() {
            same &= draw(&file.doc, id, &file.outlines[id]) == design(&doc, id);
        }
        for o in doc.objects() {
            if let ObjectKind::Pattern(pr) = &o.kind {
                let v = ciborium::Value::serialized(pr).unwrap();
                for (key, from) in [
                    ("major_rows_from", pr.major_rows_from),
                    ("major_columns_from", pr.major_columns_from),
                ] {
                    let has = matches!(&v, ciborium::Value::Map(m) if m.iter().any(|(k, _)| k.as_text() == Some(key)));
                    key_right &= has != from.is_rhythm_origin();
                    if has {
                        written += 1;
                    } else {
                        left_out += 1;
                    }
                }
            }
        }
    }
    let ok =
        same_doc && current && bytes_stable && same && key_right && written > 0 && left_out > 0;
    println!(
        "\nCHECK crate=render name=major_lines_from_save_round_trip result={} doc_equal={same_doc} current={current} export_equal={same} bytes_stable={bytes_stable} key_only_when_not_origin={key_right} written={written} left_out={left_out}",
        pass(ok)
    );
    assert!(ok);
}
