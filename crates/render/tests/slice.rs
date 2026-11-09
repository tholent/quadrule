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

//! The rung 1 headless slice (docs/bbq/2026-11-09-rung-1-headless-slice.md, Q2): build the
//! slice document, save it as `.quad`, reopen it, export Manta and Nomad PNGs and compare them
//! with the golden images in `tests/golden/`. Proves model → format → layout → text → render
//! join up.
//!
//! Each step prints one structured `CHECK` line (run with `-- --nocapture`). The exported
//! files are also written to `target/tmp/slice/` for viewing.
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test slice`, then commit `tests/golden/`.

use quadrule_format::{FontRef, QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::Extra;
use quadrule_model::sample::*;
use quadrule_profiles::builtin;
use quadrule_render::*;
use quadrule_text::{BundledFont, FontRegistry};
use std::path::PathBuf;

const SAVED_BY: &str = "quadrule-slice-test";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("slice");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

fn decode(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes.to_vec()))
        .read_info()
        .ok()?;
    let mut buf = vec![0; dec.output_buffer_size()?];
    dec.next_frame(&mut buf).ok()?;
    Some(buf)
}

#[test]
fn slice_end_to_end() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let mut all_ok = true;

    // 1. Build the document and lay its text out on each of its profiles.
    let doc = slice_document();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let mut outlines = Vec::new();
    let mut missing = 0;
    for id in doc.profile_ids() {
        let laid = lay_out_text(&doc, &primary, &builtin(id).unwrap(), &mut reg).unwrap();
        missing += laid.missing_glyphs;
        outlines.push(laid.outlines);
    }
    let font = BundledFont::for_family(ATKINSON).unwrap();
    let fonts = vec![FontRef {
        family: font.family.into(),
        bundled_id: Some(font.id.into()),
        version: font.version(),
        extra: Extra::new(),
    }];

    // 2. Save, then reopen.
    let bytes = QuadFile::new(doc.clone(), outlines.clone(), fonts)
        .to_bytes(SAVED_BY)
        .unwrap();
    std::fs::write(out_dir().join("slice.quad"), &bytes).unwrap();
    let file = QuadFile::from_bytes(&bytes).unwrap();
    let same_doc =
        file.doc == doc && file.outlines.values().cloned().collect::<Vec<_>>() == outlines;
    let ok = same_doc && missing == 0 && file.read_only.is_none();
    all_ok &= ok;
    println!(
        "CHECK crate=render name=slice_save_reopen result={} quad_bytes={} quad_sha256={} missing_glyphs={missing}",
        pass(ok),
        bytes.len(),
        sha256_hex(&bytes)
    );

    // 3. Export each profile from the reopened file and compare with its golden image.
    let mut sums = String::new();
    for id in file.doc.profile_ids() {
        let target = builtin(id).unwrap();
        // All fonts are available, so the text is laid out live (napkin §7.3 step 1).
        let live = lay_out_text(&file.doc, &primary, &target, &mut reg)
            .unwrap()
            .outlines;
        let png_live = encode_png(
            &render(
                &display_list(&file.doc, &primary, &target, &live).unwrap(),
                Math::Fastest,
            ),
            None,
        );
        // The same page drawn from the outlines saved in the file (ADR-002 item 5).
        let saved = &file.outlines[id];
        let png_saved = encode_png(
            &render(
                &display_list(&file.doc, &primary, &target, saved).unwrap(),
                Math::Fastest,
            ),
            None,
        );
        let identical = png_live == png_saved;
        all_ok &= identical;
        println!(
            "CHECK crate=render name=live_vs_saved_outlines profile={id} result={} identical={identical}",
            pass(identical)
        );

        let name = format!("slice_{id}_{}x{}.png", target.width_px, target.height_px);
        std::fs::write(out_dir().join(&name), &png_live).unwrap();
        let sha = sha256_hex(&png_live);
        sums.push_str(&format!("{sha}  {name}\n"));
        let golden_path = golden_dir().join(&name);
        if bless {
            std::fs::create_dir_all(golden_dir()).unwrap();
            std::fs::write(&golden_path, &png_live).unwrap();
        }
        let golden = std::fs::read(&golden_path).unwrap_or_default();
        let matches = golden == png_live;
        // On a mismatch, say how many pixels differ, so a machine difference is easy to judge.
        let diff = match (matches, decode(&golden), decode(&png_live)) {
            (true, _, _) => "0".to_string(),
            (false, Some(g), Some(l)) if g.len() == l.len() => {
                let n = g.iter().zip(&l).filter(|(a, b)| a != b).count();
                let max = g
                    .iter()
                    .zip(&l)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap_or(0);
                format!("{n}:max={max}")
            }
            _ => "no-golden".to_string(),
        };
        all_ok &= matches;
        println!(
            "CHECK crate=render name=slice_golden profile={id} result={} sha256={sha} golden_sha256={} differing_px={diff} os={} arch={}",
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
    if bless {
        std::fs::write(golden_dir().join("SHA256SUMS"), &sums).unwrap();
    }
    assert!(
        all_ok,
        "slice failed; see the CHECK lines (run with -- --nocapture)"
    );
}
