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

//! The rung 8a overrides page (docs/bbq/2026-11-12-rung-8-several-devices.md, Defaulted "Goldens
//! (8a)"): a box moved on the Nomad, an ellipse sized there, a star hidden there, text set smaller
//! there, a group with an override holding a child with its own, and a box that runs off the Nomad's
//! page by its constraints, exported on both devices and compared with golden images in
//! `tests/golden/`, whose sums sit after the eight that were there before (slice, shapes, transforms
//! and groups, two each), which are unchanged. Then pixel checks on what overrides promise: the
//! primary device never changes because of one, the Nomad shows each, and they survive a save. Each
//! step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test devices`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("devices");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

/// The page on one device, text laid out from the bundled fonts.
fn image(doc: &Document, id: &str) -> Gray {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &primary, &target, &mut reg).unwrap();
    let dl = display_list(doc, &primary, &target, &laid.outlines).unwrap();
    render(&dl, Math::Fastest)
}

fn bare(doc: &Document) -> Document {
    let mut d = doc.clone();
    quadrule_model::edit::strip_overrides(&mut d.layers, NOMAD_ID);
    d
}

#[test]
fn devices_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = devices_document();
    let mut all_ok = true;
    let mut sums = String::new();
    for id in doc.profile_ids() {
        let target = builtin(id).unwrap();
        let img = image(&doc, id);
        let png = encode_png(&img, None);
        let name = format!("devices_{id}_{}x{}.png", target.width_px, target.height_px);
        std::fs::write(out_dir().join(&name), &png).unwrap();
        let sha = sha256_hex(&png);
        sums.push_str(&format!("{sha}  {name}\n"));
        let golden_path = golden_dir().join(&name);
        if bless {
            std::fs::write(&golden_path, &png).unwrap();
        }
        let golden = std::fs::read(&golden_path).unwrap_or_default();
        let matches = golden == png;
        all_ok &= matches;
        println!(
            "\nCHECK crate=render name=devices_golden profile={id} result={} sha256={sha} golden_sha256={} os={} arch={}",
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
        // Keep every line that is not this test's, in its place; add this test's after them.
        let path = golden_dir().join("SHA256SUMS");
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with("devices_"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    // The sums file holds them too, after the eight that were there before, which are unchanged.
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    let old_eight = lines.len() >= 8
        && lines[..2].iter().all(|l| l.contains("slice_"))
        && lines[2..4].iter().all(|l| l.contains("shapes_"))
        && lines[4..6].iter().all(|l| l.contains("transforms_"))
        && lines[6..8].iter().all(|l| l.contains("groups_"));
    println!(
        "\nCHECK crate=render name=devices_sums_listed result={} listed={listed} old_eight_first={old_eight}",
        pass(listed && old_eight)
    );
    assert!(all_ok && listed && old_eight, "see the CHECK lines");
}

#[test]
fn the_primary_export_never_changes_because_of_an_override() {
    let doc = devices_document();
    let plain = bare(&doc);
    // Even an override keyed to the primary itself (a hand-made file) is not read.
    let mut odd = plain.clone();
    odd.object_mut(device_ids::MOVED).unwrap().overrides.insert(
        MANTA_ID.into(),
        Override {
            position: Some(Position {
                x_mm: 1.0,
                y_mm: 1.0,
            }),
            size: Some(Size {
                w_mm: 3.0,
                h_mm: 3.0,
            }),
            font_size_pt: Some(40.0),
            visible: Some(false),
            ..Override::default()
        },
    );
    let manta = encode_png(&image(&doc, MANTA_ID), None);
    let same = manta == encode_png(&image(&plain, MANTA_ID), None);
    let ignored = manta == encode_png(&image(&odd, MANTA_ID), None);
    // The Nomad does change.
    let nomad_differs = image(&doc, NOMAD_ID).data != image(&plain, NOMAD_ID).data;
    println!(
        "\nCHECK crate=render name=primary_export_ignores_overrides result={} manta_equals_bare_page={same} primary_keyed_override_ignored={ignored} nomad_differs={nomad_differs}",
        pass(same && ignored && nomad_differs)
    );
    assert!(same && ignored && nomad_differs);
}

#[test]
fn the_nomad_shows_what_the_overrides_say() {
    let doc = devices_document();
    let plain = bare(&doc);
    let nomad = builtin(NOMAD_ID).unwrap();
    let (with, without) = (image(&doc, NOMAD_ID), image(&plain, NOMAD_ID));
    let at = |img: &Gray, x_mm: f64, y_mm: f64| {
        let k = nomad.mm_to_px(1.0);
        img.at((x_mm * k) as u32, (y_mm * k) as u32)
    };
    // The star hidden on the Nomad: where the design puts it (112,34 .. 142,64 on the Manta, scaled)
    // is dark without the override and white with it.
    let (cx, cy) = (127.0 * 0.7312, 49.0 * 0.7312);
    let star_hidden = at(&without, cx, cy) < 200 && at(&with, cx, cy) == 255;
    // The box moved to (20, 62) mm: its left edge is there with the override, and not without.
    let moved_edge = (0..30).any(|dx| at(&with, 20.0 + f64::from(dx) * 0.05, 70.0) < 200)
        && at(&without, 20.0, 70.0) == 255;
    // The ellipse sized to 36 x 26 mm at (56, 24): its bottom edge (50 mm) is inked at its middle
    // (x = 74 mm) with the override, and without it the design's ellipse (ending near 42 mm) leaves
    // that place white.
    let sized_reaches = (0..20).any(|dy| at(&with, 74.0, 49.0 + f64::from(dy) * 0.1) < 200)
        && at(&without, 74.0, 50.0) == 255;
    let ok = star_hidden && moved_edge && sized_reaches;
    println!(
        "\nCHECK crate=render name=nomad_shows_overrides result={} star_hidden={star_hidden} box_moved={moved_edge} ellipse_sized={sized_reaches}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn overrides_round_trip_through_save() {
    let doc = devices_document();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    let outlines: Vec<_> = doc
        .profile_ids()
        .into_iter()
        .map(|id| {
            lay_out_text(&doc, &primary, &builtin(id).unwrap(), &mut reg)
                .unwrap()
                .outlines
        })
        .collect();
    let bytes = QuadFile::new(doc.clone(), outlines.clone(), vec![])
        .to_bytes("devices-test")
        .unwrap();
    let file = QuadFile::from_bytes(&bytes).unwrap();
    let same_doc = file.doc == doc;
    let count = file.doc.overridden_count(NOMAD_ID);
    let current = file.manifest.format_version == quadrule_format::FORMAT_VERSION
        && file.migrated_from.is_none();
    // An export from the reopened file, drawn from its saved outlines, is the export from the
    // live document.
    let mut export_same = true;
    for id in doc.profile_ids() {
        let target = builtin(id).unwrap();
        let live = {
            let laid = lay_out_text(&doc, &primary, &target, &mut reg).unwrap();
            display_list(&doc, &primary, &target, &laid.outlines).unwrap()
        };
        let saved = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
        export_same &= encode_png(&render(&live, Math::Fastest), None)
            == encode_png(&render(&saved, Math::Fastest), None);
    }
    // A page without overrides writes no `overrides` and no `orientation` key.
    let plain_bytes = QuadFile::new(bare(&doc), outlines, vec![])
        .to_bytes("devices-test")
        .unwrap();
    let plain = QuadFile::from_bytes(&plain_bytes).unwrap();
    let scene: ciborium::Value = ciborium::Value::serialized(&plain.doc).unwrap();
    let mut keys = Vec::new();
    fn collect(v: &ciborium::Value, out: &mut Vec<String>) {
        match v {
            ciborium::Value::Map(m) => {
                for (k, x) in m {
                    if let Some(k) = k.as_text() {
                        out.push(k.to_string());
                    }
                    collect(x, out);
                }
            }
            ciborium::Value::Array(a) => a.iter().for_each(|x| collect(x, out)),
            _ => {}
        }
    }
    collect(&scene, &mut keys);
    let quiet = !keys.iter().any(|k| k == "overrides" || k == "orientation");
    let ok = same_doc && count == 6 && current && export_same && quiet;
    println!(
        "\nCHECK crate=render name=overrides_round_trip result={} document_equal={same_doc} overridden_on_nomad={count} saved_as_current={current} export_equal={export_same} no_keys_when_empty={quiet}",
        pass(ok)
    );
    assert!(ok);
}
