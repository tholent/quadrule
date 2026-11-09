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

//! Quadrule draws Supernote's stock 5 mm grid template and compares it, pixel by pixel, with the
//! device screenshots in `docs/device-measurements/` everywhere the toolbar and footer do not
//! cover. Prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::sample::{MANTA_H_MM, MANTA_W_MM, slice_document};
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_render::{Math, render};
use quadrule_text::FontRegistry;
use quadrule_xtask::safezones::load_dir;
use std::path::Path;

/// The stock template as a quadrule document: one full-page square grid, 5 mm centred and
/// pixel-locked, 1 px lines at gray 1 (docs/device-measurements/README.md).
fn stock_grid() -> Document {
    let mut doc = slice_document();
    doc.page.margins_mm = Margins::all(0.0);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.origin = RhythmOrigin::Centered;
    doc.rhythm.resolution = RhythmResolution::PixelLocked;
    let mut grid = doc.layers[0].objects[0].clone();
    grid.frame = Frame::new(0.0, 0.0, MANTA_W_MM, MANTA_H_MM);
    grid.constraints = Constraints::default();
    grid.style = Style::Custom {
        width_mm: MM_PER_IN / 300.0,
        gray: 1,
    };
    grid.kind = ObjectKind::Pattern(PatternRegion {
        pattern: Pattern::Grid,
        extra: Extra::new(),
    });
    doc.layers = vec![Layer {
        name: "Grid".into(),
        visible: true,
        locked: false,
        objects: vec![grid],
        extra: Extra::new(),
    }];
    doc
}

/// Pixels that differ from a screenshot, outside the footer and that screenshot's toolbar.
fn differing(p: &Profile, ours: &[u8], shot: &quadrule_xtask::safezones::Shot) -> usize {
    let toolbar = if shot.position == "float" {
        "toolbar_floating_default".to_string()
    } else {
        format!("toolbar_{}", shot.position)
    };
    let hidden: Vec<_> = p
        .safe_zones
        .iter()
        .filter(|z| z.name.starts_with("footer_") || z.name == toolbar)
        .map(|z| z.rect)
        .collect();
    let w = shot.width;
    (0..ours.len())
        .filter(|&i| {
            let (x, y) = ((i % w) as u32, (i / w) as u32);
            let covered = hidden
                .iter()
                .any(|&(zx, zy, zw, zh)| x >= zx && x < zx + zw && y >= zy && y < zy + zh);
            !covered && ours[i] != shot.gray[i]
        })
        .count()
}

#[test]
fn quadrule_draws_supernotes_stock_grid() {
    let doc = stock_grid();
    let manta = builtin("supernote.a5x2").unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/device-measurements");
    let mut parts = Vec::new();
    let mut manta_exact = true;
    let mut nomad_off_by_one = true;
    for id in ["supernote.a5x2", "supernote.a6x2"] {
        let p = builtin(id).unwrap();
        let laid = lay_out_text(&doc, &manta, &p, &mut FontRegistry::bundled_only()).unwrap();
        let ours = render(
            &display_list(&doc, &manta, &p, &laid.outlines).unwrap(),
            Math::Fastest,
        )
        .data;
        for shot in load_dir(&root.join(id)).unwrap() {
            let diff = differing(&p, &ours, &shot);
            // The Nomad's stock grid breaks its odd-pixel tie the other way: one pixel up and left.
            let (sw, sh) = (shot.width, shot.height);
            let shifted: Vec<u8> = (0..ours.len())
                .map(|i| ours[((i / sw) + 1).min(sh - 1) * sw + ((i % sw) + 1).min(sw - 1)])
                .collect();
            let diff_shifted = differing(&p, &shifted, &shot);
            if id == "supernote.a5x2" {
                manta_exact &= diff == 0;
            } else {
                nomad_off_by_one &= diff > 0 && diff_shifted == 0;
            }
            parts.push(format!(
                "{id}/{}:diff_px={diff}:diff_px_shifted_up_left={diff_shifted}",
                shot.position
            ));
        }
    }
    let ok = manta_exact && nomad_off_by_one;
    println!(
        "CHECK crate=xtask name=stock_grid_match result={} manta_exact={manta_exact} nomad_one_px_down_right={nomad_off_by_one} {}",
        if ok { "pass" } else { "fail" },
        parts.join(" ")
    );
    assert!(ok);
}
