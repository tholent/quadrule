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

//! The graphics-chip canvas's rules, headless (rung 3 bbq Q4, Defaulted "The chip canvas on the
//! Pi will be slow"). Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_render::{Math, render};
use quadrule_ui::Viewer;
use quadrule_ui::canvas::{ChipState, Fallback, SLOW_FRAME_MS, ScreenView, screen_list};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

#[test]
fn slow_chip_frames_fall_back() {
    let mut s = ChipState::default();
    // The first frame builds shaders and is not judged.
    s.record_chip(900.0);
    let first_free = s.fallback.is_none();
    s.record_chip(SLOW_FRAME_MS);
    let at_line_ok = s.fallback.is_none();
    s.record_chip(392.0);
    let slow = s.fallback == Some(Fallback::Slow(392.0));
    let ok = first_free && at_line_ok && slow;
    println!(
        "CHECK crate=ui name=chip_fallback_rule result={} first_frame_not_judged={first_free} at_100ms_kept={at_line_ok} over_100ms_falls_back={slow}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn screen_list_matches_the_export_at_one_to_one_and_keeps_thin_lines() {
    let mut v = Viewer::default();
    v.open_sample();
    let page = v.page().unwrap().clone();
    let dl = &page.display_list;
    // At 1:1 with no offset, the screen-resolution canvas draws the export's own pixels.
    let whole = ScreenView {
        scale: 1.0,
        x: 0.0,
        y: 0.0,
        width: dl.width,
        height: dl.height,
    };
    let same = render(&screen_list(dl, &whole), Math::Fastest).data == page.image.data;
    // Zoomed out to a quarter, the 1 px ruled lines still show (lighter, not gone).
    let quarter = ScreenView {
        scale: 0.25,
        x: 0.0,
        y: 0.0,
        width: dl.width / 4,
        height: dl.height / 4,
    };
    let small = render(&screen_list(dl, &quarter), Math::Fastest);
    // Below 116 mm only the ruled lines are drawn: count runs of inked rows, in the export and
    // in the quarter-size view. Each line must still show.
    let runs = |img: &quadrule_render::Gray, from: u32| {
        let inked: Vec<bool> = (from..img.height)
            .map(|y| (0..img.width).any(|x| img.at(x, y) < 240))
            .collect();
        inked.windows(2).filter(|w| w[1] && !w[0]).count()
    };
    let from = (116.0 * 300.0 / 25.4) as u32 + 8;
    let lines_export = runs(&page.image, from);
    let lines_quarter = runs(&small, from / 4);
    let inked_rows = lines_quarter;
    let lines_show = lines_export > 10 && lines_quarter == lines_export;
    // An offset view starts where it says: 2× over the top-left of the dot grid (dots are
    // whole-pixel squares, so each becomes exact 2 × 2 blocks).
    let part = ScreenView {
        scale: 2.0,
        x: 90.0,
        y: 300.0,
        width: 64,
        height: 64,
    };
    let crop = render(&screen_list(dl, &part), Math::Fastest);
    let offset_ok =
        (0..64).all(|y| (0..64).all(|x| crop.at(x, y) == page.image.at(90 + x / 2, 300 + y / 2)));
    let dots_in_crop = crop.data.iter().any(|&p| p < 200);
    let ok = same && lines_show && offset_ok && dots_in_crop;
    println!(
        "CHECK crate=ui name=screen_canvas_list result={} one_to_one_is_export={same} lines_export={lines_export} lines_at_quarter={inked_rows} offset_2x_matches={offset_ok} dots_in_crop={dots_in_crop}",
        pass(ok)
    );
    assert!(ok);
}
