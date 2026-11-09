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

//! Viewer checks, headless. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_profiles::builtin;
use quadrule_render::{Gray, encode_png};
use quadrule_ui::viewer::*;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden(id: &str) -> Vec<u8> {
    let p = builtin(id).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../render/tests/golden")
        .join(format!("slice_{id}_{}x{}.png", p.width_px, p.height_px));
    std::fs::read(&path).unwrap_or_default()
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn shown_pixels_are_the_export() {
    // Rung 2 bbq, Q3: the viewer shows exactly the PNG's pixels. Encoded, the sample page on each
    // device is byte for byte rung 1's golden image.
    let mut v = Viewer::default();
    let empty = v.page().is_none();
    v.open_sample();
    let mut results = Vec::new();
    for id in v.profile_ids() {
        v.set_profile(&id);
        let png = encode_png(&v.page().unwrap().image, None);
        results.push((id.clone(), png == golden(&id)));
    }
    let ok = empty && results.len() == 2 && results.iter().all(|r| r.1);
    println!(
        "CHECK crate=ui name=shown_pixels_are_export result={} empty_start={empty} matches={results:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn devices_have_their_own_findings() {
    let mut v = Viewer::default();
    v.open_sample();
    v.set_profile("supernote.a5x2");
    let manta = v.page().unwrap().findings.clone();
    v.selected = Some(0);
    v.set_profile("supernote.a6x2");
    let cleared = v.selected.is_none();
    let nomad = v.page().unwrap().findings.clone();
    let ok = !manta.is_empty()
        && !nomad.is_empty()
        && manta.iter().all(|f| f.profile == "supernote.a5x2")
        && nomad.iter().all(|f| f.profile == "supernote.a6x2")
        && cleared;
    println!(
        "CHECK crate=ui name=devices_own_findings result={} manta={} nomad={} selection_cleared={cleared}",
        pass(ok),
        manta.len(),
        nomad.len()
    );
    assert!(ok);
}

#[test]
fn bad_files_leave_the_page_alone() {
    let mut v = Viewer::default();
    v.open_sample();
    let before = v.generation;
    let garbage = v
        .open_bytes(Source::File("x.quad".into()), b"not a zip")
        .is_err();
    let missing = v
        .open_path(std::path::Path::new("/nonexistent/x.quad"))
        .is_err();
    let notice = v.notice.as_ref().is_some_and(|n| n.error);
    let still = v.generation == before && v.open.as_ref().unwrap().source == Source::Sample;
    // A real .quad opens: the sample, saved the way rung 1 saves it.
    let dir = tmp("ui-open");
    let path = dir.join("page.quad");
    let doc = quadrule_model::sample::slice_document();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut fonts = quadrule_text::FontRegistry::bundled_only();
    let outlines = doc
        .profile_ids()
        .iter()
        .map(|id| {
            quadrule_layout::lay_out_text(&doc, &primary, &builtin(id).unwrap(), &mut fonts)
                .unwrap()
                .outlines
        })
        .collect();
    std::fs::write(
        &path,
        quadrule_format::QuadFile::new(doc, outlines, vec![])
            .to_bytes("viewer-test")
            .unwrap(),
    )
    .unwrap();
    let opened = v.open_path(&path).is_ok() && v.generation == before + 1;
    let png = encode_png(&v.page().unwrap().image, None);
    let same = png == golden(&v.profile_id.clone());
    let ok = garbage && missing && notice && still && opened && same;
    println!(
        "CHECK crate=ui name=open_files result={} garbage_refused={garbage} missing_refused={missing} error_notice={notice} page_kept={still} quad_opened={opened} quad_matches_golden={same}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn export_writes_the_golden_bytes() {
    // Rung 2 bbq, Q8: this device or all devices, named by the napkin's scheme (§11.2), with the
    // export renderer's bytes.
    let mut v = Viewer::default();
    v.open_sample();
    v.set_profile("supernote.a6x2");
    let one_dir = tmp("ui-export-one");
    let one = v.export(&one_dir, ExportWhich::ThisDevice).unwrap();
    let all_dir = tmp("ui-export-all");
    let all = v.export(&all_dir, ExportWhich::AllDevices).unwrap();
    let names: Vec<String> = all
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let bytes_ok = all
        .iter()
        .zip(["supernote.a5x2", "supernote.a6x2"])
        .all(|(p, id)| std::fs::read(p).unwrap() == golden(id));
    let want = [
        "rung-1-slice_manta_1920x2560.png",
        "rung-1-slice_nomad_1404x1872.png",
    ];
    let notice = v
        .notice
        .as_ref()
        .is_some_and(|n| !n.error && n.message.id == "notice-exported");
    let failed = v
        .export(
            std::path::Path::new("/nonexistent/dir"),
            ExportWhich::ThisDevice,
        )
        .is_err()
        && v.notice.as_ref().is_some_and(|n| n.error);
    let ok = one.len() == 1
        && one[0].ends_with(want[1])
        && names == want
        && bytes_ok
        && notice
        && failed
        && v.export_dir.as_deref() == Some(all_dir.as_path());
    println!(
        "CHECK crate=ui name=export_golden_bytes result={} this_device={} all={names:?} bytes_match={bytes_ok} bad_folder_reported={failed}",
        pass(ok),
        one.len()
    );
    assert!(ok);
}

#[test]
fn file_names_are_safe() {
    // Napkin §11.3: ASCII letters, digits, hyphens and underscores only.
    let cases = [
        ("Today’s Notes — 5 mm", "today-s-notes-5-mm"),
        ("  isometric 5mm  ", "isometric-5mm"),
        ("a_b", "a_b"),
        ("日本語", "untitled"),
        ("", "untitled"),
    ];
    let wrong: Vec<_> = cases
        .iter()
        .filter(|(i, o)| slug(i) != *o)
        .map(|(i, o)| format!("{i:?}->{:?} want {o:?}", slug(i)))
        .collect();
    let tokens: Vec<String> = ["supernote.a5x2", "supernote.a6x2"]
        .iter()
        .map(|id| profile_token(&builtin(id).unwrap()))
        .collect();
    let ok = wrong.is_empty() && tokens == ["manta", "nomad"];
    println!(
        "CHECK crate=ui name=safe_file_names result={} cases={} wrong={wrong:?} tokens={tokens:?}",
        pass(ok),
        cases.len()
    );
    assert!(ok);
}

#[test]
fn zoom_maths() {
    // Rung 2 bbq, Q10. Scale(s) is s physical screen pixels per device pixel; ppp is egui's
    // physical pixels per point.
    let page = (1920, 2560);
    let fit = points_per_device_px(Zoom::Fit, page, 300.0, (1000.0, 832.0), 1.0, None);
    let fits = fit * 2560.0 <= 800.0 + 1e-3 && fit * 1920.0 <= 968.0;
    let one = points_per_device_px(Zoom::Scale(1.0), page, 300.0, (1.0, 1.0), 2.0, None);
    // A screen of 4 px/mm showing a 300 PPI page: 4 * 25.4 / 300 screen pixels per device pixel.
    let real = points_per_device_px(Zoom::RealSize, page, 300.0, (1.0, 1.0), 1.0, Some(4.0));
    let real_ok = (real - 4.0 * 25.4 / 300.0).abs() < 1e-6;
    let unset = points_per_device_px(Zoom::RealSize, page, 300.0, (1.0, 1.0), 1.0, None);
    let ruler = ruler_px_per_mm(342.4, 1.0, CARD_WIDTH_MM);
    let mut v = Viewer::default();
    let mut ins = Vec::new();
    let mut now = 0.32;
    for _ in 0..9 {
        v.zoom_step(now, 1);
        if let Zoom::Scale(s) = v.zoom {
            now = s;
        }
        ins.push(now);
    }
    v.zoom_step(0.2, -1);
    let out = v.zoom;
    v.zoom_step(0.125, -1);
    let floor = v.zoom;
    let ok = fits
        && one == 0.5
        && real_ok
        && unset == 1.0
        && (ruler - 4.0).abs() < 1e-4
        && ins == [0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 16.0, 16.0, 16.0]
        && out == Zoom::Scale(0.125)
        && floor == Zoom::Scale(0.125);
    println!(
        "CHECK crate=ui name=zoom_maths result={} fit={fit:.4} one_at_ppp2={one} real={real:.4} ruler_px_per_mm={ruler} steps_in={ins:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn shrinking_keeps_thin_lines() {
    // Zoomed out, a 1 px black line every 6 px must still show in every place it was, as gray,
    // and the page's overall brightness must stay the same.
    let (w, h) = (600u32, 60u32);
    let data: Vec<u8> = (0..w * h)
        .map(|i| if (i % w) % 6 == 0 { 0 } else { 255 })
        .collect();
    let img = Gray {
        width: w,
        height: h,
        data,
    };
    let small = shrink(&img, 187, 19);
    let mean = |g: &Gray| g.data.iter().map(|&v| v as f64).sum::<f64>() / g.data.len() as f64;
    // Every line lands in one or two small pixels; the darker of them holds at least half the line,
    // so it is at most 255 - 255 / 2 / 3.21 ≈ 215, clearly darker than the paper.
    let row = &small.data[..small.width as usize];
    let r = 600.0 / 187.0;
    let lost = (0..w)
        .step_by(6)
        .filter(|&x| {
            let a = (x as f64 / r) as usize;
            let b = (((x + 1) as f64 / r) as usize).min(row.len() - 1);
            row[a].min(row[b]) > 216
        })
        .count();
    let same = shrink(&img, w, h) == img;
    let ok = lost == 0 && (mean(&img) - mean(&small)).abs() < 1.0 && same;
    println!(
        "CHECK crate=ui name=shrink_keeps_lines result={} lines=100 lost={lost} mean_before={:.2} mean_after={:.2} identity={same}",
        pass(ok),
        mean(&img),
        mean(&small)
    );
    assert!(ok);
}

#[test]
fn tiles_cover_the_page_once() {
    // The page texture is cut into tiles no larger than some graphics chips allow; every pixel
    // must be in exactly one tile.
    let mut results = Vec::new();
    for (w, h) in [
        (1920, 2560),
        (1404, 1872),
        (1024, 1024),
        (1, 1),
        (600, 3000),
    ] {
        let t = tiles(w, h, 1024);
        let mut seen = vec![0u8; (w * h) as usize];
        for &(x, y, tw, th) in &t {
            for yy in y..y + th {
                for xx in x..x + tw {
                    seen[(yy * w + xx) as usize] += 1;
                }
            }
        }
        let ok = seen.iter().all(|&n| n == 1) && t.iter().all(|t| t.2 <= 1024 && t.3 <= 1024);
        results.push((w, h, t.len(), ok));
    }
    let ok = results.iter().all(|r| r.3);
    println!(
        "CHECK crate=ui name=tiles_cover_once result={} cases={results:?}",
        pass(ok)
    );
    assert!(ok);
}
