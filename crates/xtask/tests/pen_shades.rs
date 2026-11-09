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

//! The Nomad's pen shades, re-measured from chris's screenshot on every run: the profile's
//! `ink_values` must be exactly the grays the four pens wrote. Prints one `CHECK` line.

use std::collections::BTreeMap;

#[test]
fn ink_values_match_the_pen_screenshot() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/device-measurements/supernote.a6x2/nomad-pen-shades.png");
    let mut dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(&path).unwrap()));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    let ch = info.color_type.samples();
    let (w, h) = (info.width as usize, info.height as usize);
    // Count grays on the page between the toolbar and the footer (y 120..1700). A pen stroke is
    // one flat gray over thousands of pixels; its soft edges and the handwriting's are a few
    // hundred of any one gray, so the flat grays stand out.
    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    let mut gray_only = true;
    for y in 120..1700.min(h) {
        for x in 0..w {
            let px = &buf[(y * w + x) * ch..(y * w + x) * ch + ch];
            if ch >= 3 && (px[0] != px[1] || px[1] != px[2]) {
                gray_only = false;
            }
            *counts.entry(px[0]).or_default() += 1;
        }
    }
    let shades: Vec<u8> = counts
        .iter()
        .filter(|&(&g, &n)| g != 255 && n > 5000)
        .map(|(&g, _)| g)
        .collect();
    let mut fails = Vec::new();
    for id in quadrule_profiles::builtin_ids() {
        let ink = quadrule_profiles::builtin(id).unwrap().ink_values;
        if ink != shades {
            fails.push(format!("{id}:{ink:?}"));
        }
    }
    let ok = gray_only && fails.is_empty() && shades.len() == 4;
    println!(
        "CHECK crate=xtask name=pen_shades result={} measured={shades:?} gray_only={gray_only} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
