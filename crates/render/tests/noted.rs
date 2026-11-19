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

//! The PNG text note (rung 15 bbq, Q6 and Defaulted "Text note encoding"). One new golden, a noted PNG with the
//! version string fixed here (`noted_supernote.a6x2_1404x1872.png`, listed in `golden/SHA256SUMS` after the 29
//! frozen lines): Software is Latin-1 so it is a `tEXt` chunk, the title has other characters so it is an `iTXt`
//! chunk. The pixels are the slice page's, and `encode_png(img, None)` is untouched (the other goldens say so).
//! With `QUADRULE_BLESS=1` it writes the golden and its line. Each test prints one structured `CHECK` line (run
//! with `-- --nocapture`).

use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::*;
use quadrule_profiles::builtin;
use quadrule_render::*;
use quadrule_text::FontRegistry;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn page() -> Gray {
    let doc = slice_document();
    let primary = builtin(MANTA_ID).unwrap();
    let target = builtin(NOMAD_ID).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(&doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(&doc, &primary, &target, &laid.outlines).unwrap();
    render(&dl)
}

const SOFTWARE: &str = "quadrule 0.0.0-golden";
const TITLE: &str = "Caf\u{e9} \u{65e5}\u{672c}";

fn noted(img: &Gray) -> Vec<u8> {
    encode_png(
        img,
        Some(PngMeta {
            software: SOFTWARE,
            title: TITLE,
        }),
    )
}

/// (software chunk kind, title chunk kind, pixels) read back by the png decoder.
fn read(png_bytes: &[u8]) -> (Vec<String>, Vec<String>, Vec<u8>) {
    let mut dec = png::Decoder::new(std::io::Cursor::new(png_bytes))
        .read_info()
        .unwrap();
    let mut buf = vec![0; dec.output_buffer_size().unwrap()];
    dec.next_frame(&mut buf).unwrap();
    let latin1 = dec
        .info()
        .uncompressed_latin1_text
        .iter()
        .map(|t| format!("{}={}", t.keyword, t.text))
        .collect();
    let utf8 = dec
        .info()
        .utf8_text
        .iter()
        .map(|t| format!("{}={}", t.keyword, t.get_text().unwrap()))
        .collect();
    (latin1, utf8, buf)
}

#[test]
fn a_noted_png_matches_its_golden() {
    let img = page();
    let png_bytes = noted(&img);
    let name = "noted_supernote.a6x2_1404x1872.png";
    let sha = sha256_hex(&png_bytes);
    let path = golden_dir().join(name);
    if std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1") {
        std::fs::write(&path, &png_bytes).unwrap();
        let sums = golden_dir().join("SHA256SUMS");
        let mut lines: String = std::fs::read_to_string(&sums)
            .unwrap()
            .lines()
            .filter(|l| !l.ends_with(name))
            .map(|l| format!("{l}\n"))
            .collect();
        lines.push_str(&format!("{sha}  {name}\n"));
        std::fs::write(sums, lines).unwrap();
    }
    let golden = std::fs::read(&path).unwrap_or_default();
    let listed = std::fs::read_to_string(golden_dir().join("SHA256SUMS"))
        .unwrap()
        .lines()
        .any(|l| l == format!("{sha}  {name}"));
    let (latin1, utf8, pixels) = read(&png_bytes);
    let kinds = latin1 == [format!("Software={SOFTWARE}")] && utf8 == [format!("Title={TITLE}")];
    let same_pixels = pixels == img.data;
    let ok = golden == png_bytes && listed && kinds && same_pixels;
    println!(
        "\nCHECK crate=render name=noted_png_golden result={} sha256={sha} listed={listed} software_text_chunk={} title_itxt_chunk={} pixels_unchanged={same_pixels} os={} arch={}",
        pass(ok),
        latin1.len() == 1,
        utf8.len() == 1,
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    assert!(ok);
}

#[test]
fn the_note_picks_its_chunk_and_leaves_no_note_alone() {
    let img = Gray {
        width: 4,
        height: 4,
        data: (0..16).map(|i| i * 15).collect(),
    };
    let plain = encode_png(&img, None);
    let (pl, pu, _) = read(&plain);
    let latin = encode_png(
        &img,
        Some(PngMeta {
            software: "quadrule 1.2.3",
            title: "Caf\u{e9} planner",
        }),
    );
    let (ll, lu, _) = read(&latin);
    // No title: the Title chunk is left out. A NUL cannot be in a chunk.
    let untitled = encode_png(
        &img,
        Some(PngMeta {
            software: "quadrule 1.2.3",
            title: "",
        }),
    );
    let (ul, uu, _) = read(&untitled);
    let nul = encode_png(
        &img,
        Some(PngMeta {
            software: "a\0b",
            title: "x",
        }),
    );
    let (nl, _, _) = read(&nul);
    let ok = pl.is_empty()
        && pu.is_empty()
        && ll == ["Software=quadrule 1.2.3", "Title=Caf\u{e9} planner"]
        && lu.is_empty()
        && ul == ["Software=quadrule 1.2.3"]
        && uu.is_empty()
        && nl == ["Software=a b", "Title=x"];
    println!(
        "\nCHECK crate=render name=png_note_chunks result={} plain_has_none={} latin1_is_text={} untitled_has_no_title={} nul_to_space={}",
        pass(ok),
        pl.is_empty() && pu.is_empty(),
        ll.len() == 2 && lu.is_empty(),
        ul.len() == 1,
        nl.first().is_some_and(|s| s == "Software=a b")
    );
    assert!(ok);
}
