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

//! ADR-027 item 4: the Style tab's swatches a Supernote pen writes are named after the pen, "Gray 157 · dark grey pen",
//! on hover and for screen readers, and the chosen one is named under the swatches; other swatches keep their number.
//! A profile with no pens (paper) names none. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_ui::swatches::pen_name_id;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn style_tab(gray: u8) -> Harness<'static, ViewerUi> {
    let mut d = slice_document();
    let id = d.objects().next().unwrap().id;
    d.object_mut(id).unwrap().style = Style::Custom {
        width_mm: 25.4 / 300.0,
        gray,
    };
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_document_for_test(d);
    ui.viewer.select_row(id, false);
    ui.tab = Tab::Style;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    h
}

#[test]
fn pen_swatches_are_named_after_their_pen() {
    let h = style_tab(157);
    let pens = [
        "Gray 1 · black pen",
        "Gray 157 · dark grey pen",
        "Gray 201 · light grey pen",
    ]
    .iter()
    .all(|n| h.query_by_label(n).is_some());
    let plain = [18, 53, 122, 255]
        .iter()
        .all(|g| h.query_by_label(&format!("Gray {g}")).is_some());
    let no_white_pen = h.query_by_label("Gray 254 · white pen").is_none();
    let caption = h.query_by_label("157 · dark grey pen").is_some();
    let ok = pens && plain && no_white_pen && caption;
    println!(
        "CHECK crate=ui name=pen_swatch_names result={} pens_named={pens} others_numbered={plain} no_white_pen={no_white_pen} chosen_caption={caption}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_gray_no_pen_writes_has_no_caption_and_paper_names_no_pen() {
    let h = style_tab(53);
    let no_caption = h.query_by_label("157 · dark grey pen").is_none()
        && h.query_by_label("53 · dark grey pen").is_none();
    let paper = pen_name_id(157, &[]).is_none();
    let tablet = pen_name_id(157, &[1, 157, 201, 254]) == Some("app-pen-dark-grey");
    let ok = no_caption && paper && tablet;
    println!(
        "CHECK crate=ui name=pen_swatch_no_pen result={} no_caption={no_caption} paper_none={paper} tablet_dark_grey={tablet}",
        pass(ok)
    );
    assert!(ok);
}
