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

//! The main device's thumbnail in a saved `.quad` (rung 15 bbq, Q8; format 0.16): Save and Save As write
//! `previews/<primary_profile_id>.png`, 8-bit gray, longest side 480 px, shrunk by averaging from the main
//! device's export pixels (a landscape design turned like its PNG); an autosave writes none; a 0.15 file gains one
//! at its next save. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::{PREVIEW_SIDE, QuadFile, preview_path};
use quadrule_render::{Gray, encode_png};
use quadrule_ui::viewer::{export_pixels, shrink};
use quadrule_ui::{Recovery, Source, Viewer};
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A preview read back: (width, height, color, depth, pixels).
fn decode(bytes: &[u8]) -> (u32, u32, png::ColorType, png::BitDepth, Vec<u8>) {
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut buf = vec![0; dec.output_buffer_size().unwrap()];
    let info = dec.next_frame(&mut buf).unwrap();
    (
        info.width,
        info.height,
        info.color_type,
        info.bit_depth,
        buf,
    )
}

/// What the preview should be for the open document: the main device's export pixels, shrunk.
fn expected(v: &mut Viewer) -> Gray {
    let id = v.doc().unwrap().primary_profile_id.clone();
    let page = v.page_for(&id).unwrap().clone();
    let px = export_pixels(&page);
    let scale = PREVIEW_SIDE as f64 / px.width.max(px.height) as f64;
    shrink(
        &px,
        (px.width as f64 * scale).round() as u32,
        (px.height as f64 * scale).round() as u32,
    )
}

#[test]
fn save_writes_the_main_device_thumbnail() {
    let dir = tmp("preview-save");
    let mut v = Viewer::default();
    v.open_sample();
    let path = dir.join("page.quad");
    v.save_to(&path).unwrap();
    let file = QuadFile::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let primary = v.doc().unwrap().primary_profile_id.clone();
    let key = preview_path(&primary);
    let got = file.previews.get(&key).map(|b| decode(b));
    let want = expected(&mut v);
    let (shape, same) = match &got {
        Some((w, h, c, d, px)) => (
            (*w, *h, *c, *d),
            (*w, *h) == (want.width, want.height) && *px == want.data,
        ),
        None => ((0, 0, png::ColorType::Rgb, png::BitDepth::One), false),
    };
    let sized = shape.0.max(shape.1) == PREVIEW_SIDE;
    let gray8 = shape.2 == png::ColorType::Grayscale && shape.3 == png::BitDepth::Eight;
    // Only the main device gets one.
    let only_main = file.previews.len() == 1 && file.doc.profile_ids().len() > 1;
    // The same bytes as encoding the expected thumbnail: nothing else is in the file.
    let bytes_same = file.previews.get(&key) == Some(&encode_png(&want, None));
    let ok = got.is_some() && sized && gray8 && same && only_main && bytes_same;
    println!(
        "CHECK crate=ui name=save_writes_preview result={} size={}x{} gray8={gray8} longest_side_480={sized} pixels_are_shrunk_export={same} only_main_device={only_main}",
        pass(ok),
        shape.0,
        shape.1
    );
    assert!(ok);
}

#[test]
fn a_landscape_design_gets_a_portrait_thumbnail_like_its_png() {
    let dir = tmp("preview-landscape");
    let mut v = Viewer::default();
    v.open_landscape();
    let path = dir.join("wide.quad");
    v.save_to(&path).unwrap();
    let file = QuadFile::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let key = preview_path(&v.doc().unwrap().primary_profile_id);
    let (w, h, ..) = decode(file.previews.get(&key).unwrap());
    let want = expected(&mut v);
    let ok = h == PREVIEW_SIDE && w < h && (w, h) == (want.width, want.height);
    println!(
        "CHECK crate=ui name=preview_landscape_turned result={} size={w}x{h} portrait={}",
        pass(ok),
        w < h
    );
    assert!(ok);
}

#[test]
fn autosave_writes_no_preview_and_an_old_file_gains_one_at_its_next_save() {
    let dir = tmp("preview-old");
    // The 0.15 fixture: written by the 0.15.0 build, so it has no preview.
    let old = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../render/tests/fixtures/v0.15/devices.quad"),
    )
    .unwrap();
    let path = dir.join("old.quad");
    std::fs::write(&path, &old).unwrap();
    let mut v = Viewer::default();
    v.open_bytes(Source::File(path.clone()), &old).unwrap();
    let had_none = QuadFile::from_bytes(&old).unwrap().previews.is_empty();
    // Autosave: the recovery file holds none.
    let rec = Recovery::new(dir.join("recovery"));
    let quad = rec.write(&mut v).unwrap();
    let rec_bytes = std::fs::read(&quad).unwrap();
    let auto_none = QuadFile::from_bytes(&rec_bytes)
        .unwrap()
        .previews
        .is_empty();
    // Save: it gains one, and the file is the current version.
    v.save_to(&path).unwrap();
    let saved = QuadFile::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let key = preview_path(&saved.doc.primary_profile_id);
    let gained = saved.previews.contains_key(&key) && saved.previews.len() == 1;
    let current = saved.manifest.format_version == quadrule_format::FORMAT_VERSION;
    // Saved twice, the same bytes (the picture is deterministic).
    v.save_to(&path).unwrap();
    let again = QuadFile::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let stable = again.previews == saved.previews;
    let ok = had_none && auto_none && gained && current && stable;
    println!(
        "CHECK crate=ui name=preview_old_file_and_autosave result={} old_had_none={had_none} autosave_none={auto_none} gained_at_save={gained} current_version={current} stable={stable}",
        pass(ok)
    );
    assert!(ok);
}

/// Times Save with and without the thumbnail (rung 15 leg 2): `cargo test --release -p quadrule-ui --test preview
/// save_time -- --ignored --nocapture`. Reported, not judged.
#[test]
#[ignore = "a timing run, for a release build on the Pi"]
fn save_time() {
    fn median(mut v: Vec<f64>) -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    }
    let dir = tmp("preview-time");
    for (name, open) in [
        ("sample", Viewer::open_sample as fn(&mut Viewer)),
        ("groups", Viewer::open_groups as fn(&mut Viewer)),
        ("paper", Viewer::open_paper as fn(&mut Viewer)),
    ] {
        let mut v = Viewer::default();
        open(&mut v);
        // Warm up: pages drawn, fonts loaded.
        let _ = v.save_bytes().unwrap();
        let _ = v.file_bytes().unwrap();
        let (mut plain, mut with, mut thumb, mut save) = (vec![], vec![], vec![], vec![]);
        for _ in 0..9 {
            let t = std::time::Instant::now();
            let _ = v.file_bytes().unwrap();
            plain.push(t.elapsed().as_secs_f64() * 1e3);
            let t = std::time::Instant::now();
            let _ = v.save_bytes().unwrap();
            with.push(t.elapsed().as_secs_f64() * 1e3);
            let t = std::time::Instant::now();
            let _ = v.preview_png().unwrap();
            thumb.push(t.elapsed().as_secs_f64() * 1e3);
            let t = std::time::Instant::now();
            v.save_to(&dir.join(format!("{name}.quad"))).unwrap();
            save.push(t.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "CHECK crate=ui name=save_time page={name} runs=9 median_ms_without_preview={:.1} median_ms_with_preview={:.1} median_ms_preview_alone={:.1} median_ms_save_to={:.1}",
            median(plain),
            median(with),
            median(thumb),
            median(save)
        );
    }
}
