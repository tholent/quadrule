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

//! ADR-027 item 7 with `--pen-grays` on: the grays sheet shows the new steps (N1–N15, the pens' grays among them) and
//! the points halfway between them (N16–N30), each patch exactly its gray, and no G code. This file is its own test
//! program, so the switch is on for it alone. Prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_profiles::{builtin, builtin_ids};
use quadrule_render::render;
use quadrule_text::FontRegistry;
use quadrule_xtask::calib::*;

#[test]
fn the_grays_sheet_shows_the_new_steps_and_the_points_between() {
    quadrule_model::pen_grays::set(true);
    let claimed: Vec<u8> = pen_patch_step_grays()
        .iter()
        .chain(&pen_patch_between_grays())
        .copied()
        .collect();
    let pens_on_sheet = [1, 157, 201].iter().all(|g| claimed[..15].contains(g));
    let between_ok = claimed[15] == 9 && claimed[29] == 246;
    let (mut patches, mut fails) = (0, Vec::new());
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        let pages = sheets(&p, &mut FontRegistry::bundled_only(), "test").expect("sheets build");
        for page in pages.iter().filter(|pg| pg.sheet == "grays") {
            let img = render(&page.dl);
            for s in &page.samples {
                if s.code == "GR" {
                    continue;
                }
                let Some(n) = s
                    .code
                    .strip_prefix('N')
                    .and_then(|n| n.parse::<usize>().ok())
                else {
                    fails.push(format!("{id}:{}:not_an_N_code", s.code));
                    continue;
                };
                let g = claimed[n - 1];
                let (x0, y0, x1, y1) = (
                    s.area.0 as u32,
                    s.area.1 as u32,
                    s.area.2 as u32,
                    s.area.3 as u32,
                );
                patches += 1;
                let wrong = (y0..y1)
                    .flat_map(|y| (x0..x1).map(move |x| (x, y)))
                    .filter(|&(x, y)| img.at(x, y) != g)
                    .count();
                if wrong > 0 {
                    fails.push(format!("{id}:{}:wrong={wrong}", s.code));
                }
            }
        }
    }
    let ok = fails.is_empty() && patches == 2 * claimed.len() && pens_on_sheet && between_ok;
    println!(
        "CHECK crate=xtask name=pen_grays_sheet result={} patches={patches} pens_on_sheet={pens_on_sheet} between_ok={between_ok} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
