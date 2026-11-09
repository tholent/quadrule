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

//! Safe-zone measurement checks. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_profiles::{builtin, builtin_ids};
use quadrule_xtask::safezones::*;
use std::path::Path;

/// A zone as (name, rect, advisory), for comparing.
type Zone = (String, (u32, u32, u32, u32), bool);

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// A grid page (1 px black lines every 20 px from 7) with white boxes painted over it.
fn shot(position: &str, boxes: &[(usize, usize, usize, usize)]) -> Shot {
    let (w, h) = (600, 800);
    let mut gray = vec![255u8; w * h];
    for y in 0..h {
        for x in 0..w {
            if (x >= 7 && (x - 7) % 20 == 0) || (y >= 7 && (y - 7) % 20 == 0) {
                gray[y * w + x] = 0;
            }
        }
    }
    for &(bx, by, bw, bh) in boxes {
        for y in by..by + bh {
            for x in bx..bx + bw {
                gray[y * w + x] = 255;
            }
        }
    }
    Shot {
        position: position.into(),
        width: w,
        height: h,
        gray,
    }
}

#[test]
fn finds_known_boxes_on_a_synthetic_grid() {
    // The side toolbars are drawn shorter than the page, as a trimmed toolbar is; their zones must
    // still run the full height.
    let footer = [(50, 760, 100, 40), (400, 760, 120, 40)];
    let with = |b: (usize, usize, usize, usize)| [footer[0], footer[1], b];
    let shots = [
        shot("left", &with((0, 0, 40, 750))),
        shot("right", &with((560, 0, 40, 750))),
        shot("top", &with((0, 0, 600, 40))),
        shot("float", &with((250, 100, 40, 300))),
    ];
    let got: Vec<Zone> = measure(&shots)
        .unwrap()
        .into_iter()
        .map(|z| (z.name, z.rect, z.advisory))
        .collect();
    let want: Vec<Zone> = vec![
        ("footer_left".to_string(), (50, 760, 100, 40), false),
        ("footer_right".to_string(), (400, 760, 120, 40), false),
        (
            "toolbar_floating_default".to_string(),
            (250, 100, 40, 300),
            true,
        ),
        ("toolbar_left".to_string(), (0, 0, 40, 800), false),
        ("toolbar_right".to_string(), (560, 0, 40, 800), false),
        ("toolbar_top".to_string(), (0, 0, 600, 40), false),
    ];
    let ok = got == want;
    println!(
        "CHECK crate=xtask name=safe_zones_synthetic result={} zones={} got={got:?}",
        pass(ok),
        got.len()
    );
    assert!(ok);
}

#[test]
fn profiles_match_their_screenshots() {
    // Re-measures docs/device-measurements/<id>/ and requires the profile's safe zones to match,
    // so new screenshots and the RON files cannot drift apart.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/device-measurements");
    let mut parts = Vec::new();
    let mut ok = true;
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        let shots = load_dir(&root.join(id)).unwrap();
        let same = measure(&shots).map(|z| z == p.safe_zones).unwrap_or(false);
        ok &= same;
        parts.push(format!(
            "{id}:screenshots={}:zones={}:match={same}",
            shots.len(),
            p.safe_zones.len()
        ));
    }
    println!(
        "CHECK crate=xtask name=safe_zones_match_screenshots result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
