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

//! Calibration-sheet checks. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_layout::Item;
use quadrule_profiles::{Profile, builtin, builtin_ids};
use quadrule_render::{Math, render};
use quadrule_text::FontRegistry;
use quadrule_xtask::calib::*;
use std::collections::{BTreeMap, BTreeSet};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn pages(p: &Profile) -> Vec<Page> {
    sheets(p, &mut FontRegistry::bundled_only(), "test").expect("sheets build")
}

/// The (width, gray) a line-like sample claims, from its code: L (matrix) or W1–W12 (strips).
fn claimed_line(code: &str) -> Option<(u32, u8, usize)> {
    if let Some(rest) = code.strip_prefix('L') {
        let row = (rest.as_bytes()[0] - b'A') as usize;
        let col: usize = rest[1..].parse().ok()?;
        Some((LINE_WIDTHS[row], LINE_GRAYS[col - 1], 3))
    } else if let Some(n) = code.strip_prefix('W') {
        let n: usize = n.parse().ok()?;
        STRIP_LINES.get(n - 1).map(|&(w, g)| (w, g, 3))
    } else {
        None
    }
}

#[test]
fn codes_are_unique_and_mean_the_same_everywhere() {
    let mut per_profile: Vec<BTreeMap<String, String>> = Vec::new();
    let mut parts = Vec::new();
    let mut ok = true;
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        let ps = pages(&p);
        let all: Vec<(String, String)> = ps
            .iter()
            .flat_map(|pg| pg.samples.iter().map(|s| (s.code.clone(), s.what.clone())))
            .collect();
        let map: BTreeMap<String, String> = all.iter().cloned().collect();
        let unique = map.len() == all.len();
        let whole = ps.iter().flat_map(|pg| &pg.dl.items).all(|i| match i {
            Item::Rect { x0, y0, x1, y1, .. } => [x0, y0, x1, y1].iter().all(|v| v.fract() == 0.0),
            _ => true,
        });
        let inside = ps.iter().flat_map(|pg| &pg.samples).all(|s| {
            s.area.0 >= 0.0 && s.area.2 <= p.width_px as f64 && s.area.3 <= p.height_px as f64
        });
        ok &= unique && whole && inside;
        parts.push(format!(
            "{id}:pages={}:codes={}:unique={unique}:whole_px={whole}:inside={inside}",
            ps.len(),
            all.len()
        ));
        per_profile.push(map);
    }
    let same = per_profile.windows(2).all(|w| w[0] == w[1]);
    let expected = LINE_WIDTHS.len() * LINE_GRAYS.len()
        + DASHES.len() * DASH_GRAYS.len()
        + DOTS.len() * DOT_GRAYS.len()
        + TEXT_SIZES_PT.len() * TEXT_WEIGHTS.len() * TEXT_GRAYS.len()
        + STRIP_LINES.len()
        + STRIP_DOTS.len();
    ok &= same && per_profile.iter().all(|m| m.len() == expected);
    println!(
        "CHECK crate=xtask name=calib_codes result={} same_on_all_profiles={same} expected={expected} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn line_samples_print_what_their_codes_say() {
    // Down the middle of every line sample: exactly the claimed number of lines, each exactly the
    // claimed width in pixels, every inked pixel exactly the claimed gray.
    let mut checked = 0;
    let mut fails = Vec::new();
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        for page in pages(&p)
            .iter()
            .filter(|pg| pg.sheet == "lines" || pg.sheet == "writing")
        {
            let lines: Vec<_> = page
                .samples
                .iter()
                .filter_map(|s| claimed_line(&s.code).map(|c| (s, c)))
                .collect();
            if lines.is_empty() {
                continue;
            }
            let img = render(&page.dl, Math::Fastest);
            for (s, (w, g, n)) in lines {
                let x = ((s.area.0 + s.area.2) / 2.0) as u32;
                let col: Vec<u8> = (s.area.1 as u32..s.area.3 as u32)
                    .map(|y| img.at(x, y))
                    .collect();
                let mut runs = Vec::new();
                let mut run = 0;
                for &v in col.iter().chain(std::iter::once(&255)) {
                    if v != 255 {
                        run += 1;
                    } else if run > 0 {
                        runs.push(run);
                        run = 0;
                    }
                }
                let wrong_gray = col.iter().filter(|&&v| v != 255 && v != g).count();
                checked += 1;
                if runs.len() != n || runs.iter().any(|&r| r != w) || wrong_gray > 0 {
                    fails.push(format!(
                        "{id}:{}:runs={runs:?}:wrong_gray={wrong_gray}",
                        s.code
                    ));
                }
            }
        }
    }
    let ok = fails.is_empty() && checked > 0;
    println!(
        "CHECK crate=xtask name=calib_lines_exact result={} samples={checked} fails={fails:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn sheets_are_repeatable() {
    let p = builtin("supernote.a6x2").unwrap();
    let (a, b) = (pages(&p), pages(&p));
    let same_lists = a.len() == b.len()
        && a.iter()
            .zip(&b)
            .all(|(x, y)| x.dl == y.dl && x.samples == y.samples);
    let same_pixels = render(&a[0].dl, Math::Fastest) == render(&b[0].dl, Math::Fastest);
    let names: BTreeSet<String> = a.iter().map(|pg| pg.file_name(&p)).collect();
    let safe = names.iter().all(|n| {
        n.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
    });
    let ok = same_lists && same_pixels && safe && names.len() == a.len();
    println!(
        "CHECK crate=xtask name=calib_repeatable result={} pages={} safe_names={safe}",
        pass(ok),
        a.len()
    );
    assert!(ok);
}

#[test]
fn sheets_keep_clear_of_docked_toolbars_and_footer() {
    // Nothing drawn and no sample may reach into a fixed safe zone, whichever way the toolbar
    // is docked (napkin §3.5). The floating toolbar moves, so it is not avoided.
    let mut parts = Vec::new();
    let mut ok = true;
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        let zones: Vec<_> = p.safe_zones.iter().filter(|z| !z.advisory).collect();
        let hits = |x0: f64, y0: f64, x1: f64, y1: f64| {
            zones.iter().any(|z| {
                let (zx, zy, zw, zh) = (
                    z.rect.0 as f64,
                    z.rect.1 as f64,
                    z.rect.2 as f64,
                    z.rect.3 as f64,
                );
                x0 < zx + zw && x1 > zx && y0 < zy + zh && y1 > zy
            })
        };
        let mut overlaps = 0;
        for page in pages(&p) {
            for item in &page.dl.items {
                let (x0, y0, x1, y1) = match item {
                    Item::Rect { x0, y0, x1, y1, .. } => (*x0, *y0, *x1, *y1),
                    Item::Fill { path, .. } => {
                        let b = kurbo::Shape::bounding_box(path);
                        (b.x0, b.y0, b.x1, b.y1)
                    }
                    Item::Stroke {
                        x0,
                        y0,
                        x1,
                        y1,
                        width,
                        ..
                    } => (
                        x0.min(*x1) - width,
                        y0.min(*y1) - width,
                        x0.max(*x1) + width,
                        y0.max(*y1) + width,
                    ),
                };
                if hits(x0, y0, x1, y1) {
                    overlaps += 1;
                }
            }
        }
        ok &= overlaps == 0 && !zones.is_empty();
        parts.push(format!(
            "{id}:fixed_zones={}:overlaps={overlaps}",
            zones.len()
        ));
    }
    println!(
        "CHECK crate=xtask name=calib_clear_of_safe_zones result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
