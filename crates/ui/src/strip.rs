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

//! The device previews strip (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md, Q5): along the
//! bottom of the canvas, above the status bar, a small live picture of each device's page side by
//! side, the device on screen outlined, and under each the number of objects changed on it.
//! Clicking a picture puts that device on screen. The pictures are the export pixels shrunk, and are
//! made again when an edit ends (a gesture's end, an undo, a field change), never on a frame of a
//! drag: they show the last picture meanwhile, so a drag costs no more than it did.

use crate::view::ViewerUi;
use crate::viewer::shrink;
use egui::{
    ColorImage, Role, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, WidgetInfo,
};
use quadrule_i18n::Message;

/// How tall a picture is, in screen points.
pub const STRIP_PICTURE_POINTS: f32 = 96.0;

/// One device's picture and what it was made from.
pub(crate) struct StripPic {
    generation: u64,
    revision: u64,
    texture: TextureHandle,
    /// The picture's size in screen points.
    size: egui::Vec2,
}

impl ViewerUi {
    /// Makes the pictures for devices whose picture is out of date, unless a gesture is running
    /// (the last picture shows meanwhile). Does nothing while the strip is folded.
    pub(crate) fn refresh_strip(&mut self, ctx: &egui::Context) {
        if !self.viewer.show_strip || self.viewer.dragging() {
            return;
        }
        let ppp = ctx.pixels_per_point();
        let (generation, revision) = (self.viewer.generation, self.viewer.revision);
        for id in self.viewer.profile_ids() {
            let fresh = self
                .strip_pics
                .get(&id)
                .is_some_and(|p| p.generation == generation && p.revision == revision);
            if fresh {
                continue;
            }
            let Ok(page) = self.viewer.page_for(&id) else {
                continue;
            };
            let h_pts = STRIP_PICTURE_POINTS;
            let k = h_pts * ppp / page.image.height as f32;
            let (tw, th) = (
                ((page.image.width as f32 * k).round() as u32).max(1),
                ((page.image.height as f32 * k).round() as u32).max(1),
            );
            let small = shrink(&page.image, tw, th);
            let img = ColorImage::from_gray([tw as usize, th as usize], &small.data);
            let texture = ctx.load_texture(format!("strip-{id}"), img, TextureOptions::LINEAR);
            self.strip_pics.insert(
                id,
                StripPic {
                    generation,
                    revision,
                    texture,
                    size: egui::vec2(tw as f32 / ppp, th as f32 / ppp),
                },
            );
        }
    }

    /// Draws the strip: a fold button, then a picture of each device with its name and how many
    /// objects are changed on it. A click puts that device on screen.
    pub(crate) fn strip(&mut self, ui: &mut egui::Ui) {
        let fold = self.t(if self.viewer.show_strip {
            "app-strip-fold"
        } else {
            "app-strip-unfold"
        });
        ui.horizontal(|ui| {
            // The fold arrow is drawn, as in the layers tree: the fonts may not hold a triangle.
            let arrow = ui.add(
                egui::Button::new("")
                    .min_size(egui::vec2(16.0, 16.0))
                    .frame(false),
            );
            arrow.widget_info(|| {
                WidgetInfo::selected(Role::CheckBox, true, self.viewer.show_strip, &fold)
            });
            egui::collapsing_header::paint_default_icon(
                ui,
                if self.viewer.show_strip { 1.0 } else { 0.0 },
                &arrow,
            );
            let label = ui.add(egui::Label::new(self.t("app-strip")).sense(Sense::click()));
            if arrow.on_hover_text(&fold).clicked() || label.clicked() {
                self.viewer.show_strip = !self.viewer.show_strip;
            }
        });
        if !self.viewer.show_strip {
            return;
        }
        let ids = self.viewer.profile_ids();
        let on_screen = self.viewer.profile_id.clone();
        ui.horizontal(|ui| {
            for id in &ids {
                let name = self.device_name(id);
                let changed = self.viewer.doc().map_or(0, |d| d.overridden_count(id));
                let go = self
                    .strings
                    .format(&Message::new("app-strip-device").text("device", name.clone()));
                ui.vertical(|ui| {
                    let (size, tex) = match self.strip_pics.get(id) {
                        Some(p) => (p.size, Some(p.texture.id())),
                        None => (
                            egui::vec2(STRIP_PICTURE_POINTS * 0.75, STRIP_PICTURE_POINTS),
                            None,
                        ),
                    };
                    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                    if let Some(tex) = tex {
                        ui.painter().image(
                            tex,
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    }
                    let here = *id == on_screen;
                    let stroke = if here {
                        Stroke::new(2.5, ui.visuals().selection.stroke.color)
                    } else {
                        Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
                    };
                    ui.painter()
                        .rect_stroke(rect, 0.0, stroke, StrokeKind::Outside);
                    resp.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, here, &go));
                    if resp.on_hover_text(&go).clicked() {
                        self.viewer.set_profile(id);
                    }
                    let mut line = name.clone();
                    if changed > 0 {
                        line = format!(
                            "{line} · {}",
                            self.strings.format(&Message::new("app-strip-count").num(
                                "count",
                                changed as f64,
                                0
                            ))
                        );
                    }
                    ui.label(egui::RichText::new(line).small());
                });
                ui.add_space(8.0);
            }
        });
    }
}
