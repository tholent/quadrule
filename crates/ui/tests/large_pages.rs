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

//! Large pages (rung 17, Q7; docs/bbq/2026-11-24-rung-17-polish.md): while the page is zoomed out the export-pixels
//! canvas draws it at the screen's own pixels and never makes the 70 M pixels of an A3 page at 600 PPI; the strip's
//! pictures and the Save preview are drawn at their own size too; the export is unchanged. Headless. Each test
//! prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui_kittest::Harness;
use quadrule_model::sample::{PAPER_A3_ID, PAPER_TABLOID_ID};
use quadrule_render::{Gray, shrink};
use quadrule_ui::viewer::export_pixels;
use quadrule_ui::{Viewer, ViewerUi, Zoom};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn harness(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(40)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window {
                ui: ViewerUi::new(v),
            },
        );
    h.run();
    h
}

fn large() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_large_paper();
    v.set_profile(PAPER_A3_ID);
    harness(v)
}

#[test]
fn a_large_page_zoomed_out_never_makes_its_export_pixels() {
    let mut h = large();
    // Fit, then a scale below 1:1, then the strip's pictures: none needs the full-size picture.
    h.run();
    h.state_mut().ui.viewer.zoom = Zoom::Scale(0.4);
    h.run();
    h.run();
    let ids = [PAPER_A3_ID, PAPER_TABLOID_ID];
    let (size, a3_drawn, tabloid_drawn) = {
        let v = &mut h.state_mut().ui.viewer;
        let size = v
            .page_for(PAPER_A3_ID)
            .map(|p| (p.profile.width_px, p.profile.height_px))
            .unwrap_or_default();
        let a3 = v.page_for(PAPER_A3_ID).is_ok_and(|p| p.image_drawn());
        let tab = v.page_for(PAPER_TABLOID_ID).is_ok_and(|p| p.image_drawn());
        (size, a3, tab)
    };
    // The Save preview is drawn at 480 px without it too.
    let (preview, drawn_after_preview) = {
        let v = &mut h.state_mut().ui.viewer;
        let p = v.preview_png().map(|(_, b)| b.len()).unwrap_or(0);
        let d = ids
            .iter()
            .any(|id| v.page_for(id).is_ok_and(|p| p.image_drawn()));
        (p, d)
    };
    let ok =
        size == (7016, 9921) && !a3_drawn && !tabloid_drawn && preview > 0 && !drawn_after_preview;
    println!(
        "\nCHECK crate=ui name=large_page_zoomed_out result={} a3_px={}x{} a3_full_picture_made={a3_drawn} tabloid_full_picture_made={tabloid_drawn} preview_bytes={preview} full_picture_made_by_preview={drawn_after_preview}",
        pass(ok),
        size.0,
        size.1
    );
    assert!(ok);
}

#[test]
fn zoomed_in_past_one_to_one_the_export_pixels_are_made_and_are_the_export() {
    let mut v = Viewer::default();
    v.open_paper();
    v.set_profile(quadrule_model::sample::PAPER_A4_ID);
    let mut h = harness(v);
    h.state_mut().ui.viewer.zoom = Zoom::Scale(2.0);
    h.run();
    h.run();
    let v = &mut h.state_mut().ui.viewer;
    let made = v
        .page_for(quadrule_model::sample::PAPER_A4_ID)
        .is_ok_and(|p| p.image_drawn());
    println!(
        "\nCHECK crate=ui name=large_page_zoomed_in result={} full_picture_made={made}",
        pass(made)
    );
    assert!(made);
}

#[test]
fn a_page_drawn_at_its_own_small_size_is_close_to_the_export_shrunk() {
    // The strip and preview pictures: the page drawn straight at the small size, against the export pixels shrunk
    // by averaging (what they were before rung 17).
    let mut v = Viewer::default();
    v.open_sample();
    let id = v.doc().unwrap().primary_profile_id.clone();
    let page = v.page_for(&id).unwrap().clone();
    let (w, h) = (360, 480);
    let direct: Gray = page.image_at(w, h);
    let shrunk = shrink(&export_pixels(&page), w, h);
    let sum: u64 = direct
        .data
        .iter()
        .zip(&shrunk.data)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    let mean = sum as f64 / direct.data.len() as f64;
    let ok = (direct.width, direct.height) == (shrunk.width, shrunk.height) && mean < 3.0;
    println!(
        "\nCHECK crate=ui name=small_picture_close_to_shrunk_export result={} mean_diff={mean:.2}",
        pass(ok)
    );
    assert!(ok);
}
