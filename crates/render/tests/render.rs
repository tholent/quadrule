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

//! Renderer checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::{DisplayList, Item, display_list, lay_out_text};
use quadrule_model::LineRole;
use quadrule_model::sample::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_render::*;
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn slice_page(id: &str) -> (Profile, DisplayList) {
    let doc = slice_document();
    let primary = builtin(MANTA_ID).unwrap();
    let target = builtin(id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(&doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(&doc, &primary, &target, &laid.outlines).unwrap();
    (target, dl)
}

#[test]
fn snapped_lines_are_exact() {
    // Down one column through the ruled lines: each line is exactly its role's width in
    // pixels, all of its role's gray, with no soft gray edge (napkin §10.2).
    let mut parts = Vec::new();
    let mut ok = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let (p, dl) = slice_page(id);
        let img = render(&dl, Math::Fastest);
        let guide = p.role(LineRole::Guide);
        let (x, y_start) = match dl
            .items
            .iter()
            .find(|i| matches!(i, Item::Rect { gray, .. } if *gray == guide.gray))
        {
            Some(Item::Rect { x0, x1, y0, .. }) => (((x0 + x1) / 2.0) as u32, *y0 as u32 - 5),
            _ => panic!("no ruled line"),
        };
        let column: Vec<u8> = (y_start..img.height).map(|y| img.at(x, y)).collect();
        let mut runs = Vec::new();
        let mut run = 0;
        for &v in &column {
            if v == guide.gray {
                run += 1;
            } else if run > 0 {
                runs.push(run);
                run = 0;
            }
        }
        let stray = column
            .iter()
            .filter(|&&v| v != guide.gray && v != 255)
            .count();
        let exact = !runs.is_empty() && runs.iter().all(|&r| r == guide.width_px) && stray == 0;
        ok &= exact;
        parts.push(format!(
            "{id}:lines={}:width_px={}:stray={stray}",
            runs.len(),
            guide.width_px
        ));
    }
    println!(
        "CHECK crate=render name=snapped_lines_exact result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn simd_and_portable_maths_agree() {
    // ADR-003 item 2's lever: if this ever fails, export must switch to portable maths.
    let mut parts = Vec::new();
    let mut ok = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let (_, dl) = slice_page(id);
        let fast = render(&dl, Math::Fastest);
        let portable = render(&dl, Math::Portable);
        let again = render(&dl, Math::Fastest);
        let diff = fast
            .data
            .iter()
            .zip(&portable.data)
            .filter(|(a, b)| a != b)
            .count();
        ok &= diff == 0 && fast == again;
        parts.push(format!(
            "{id}:simd_vs_portable_diff_px={diff}:repeat_identical={}",
            fast == again
        ));
    }
    println!(
        "CHECK crate=render name=simd_vs_portable result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn png_is_8_bit_gray_with_optional_note() {
    let (_, dl) = slice_page(NOMAD_ID);
    let img = render(&dl, Math::Fastest);
    let read = |bytes: &[u8]| {
        let mut dec = png::Decoder::new(std::io::Cursor::new(bytes.to_vec()))
            .read_info()
            .unwrap();
        let mut buf = vec![0; dec.output_buffer_size().unwrap()];
        let info = dec.next_frame(&mut buf).unwrap();
        let texts: Vec<String> = dec
            .info()
            .uncompressed_latin1_text
            .iter()
            .map(|t| format!("{}={}", t.keyword, t.text))
            .collect();
        (info.color_type, info.bit_depth, buf, texts)
    };
    let plain = read(&encode_png(&img, None));
    let noted = read(&encode_png(
        &img,
        Some(PngMeta {
            software: "Quadrule 0.1.0",
            title: "Rung 1 slice",
        }),
    ));
    let ok = plain.0 == png::ColorType::Grayscale
        && plain.1 == png::BitDepth::Eight
        && plain.2 == img.data
        && plain.3.is_empty()
        && noted.2 == img.data
        && noted.3 == ["Software=Quadrule 0.1.0", "Title=Rung 1 slice"];
    println!(
        "CHECK crate=render name=png_gray8_no_alpha result={} color={:?} depth={:?} note={:?}",
        pass(ok),
        plain.0,
        plain.1,
        noted.3
    );
    assert!(ok);
}
