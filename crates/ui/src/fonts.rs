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

//! The font picker (rung 11a, docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted "The font
//! picker"): one list in two sections, Bundled then System, each family name drawn in its own font; a
//! search box that filters both; and a preview line showing the box's own text (or the family name
//! when it has none) at the box's point size, drawn as export pixels for the device on screen at
//! 1:1. Names and preview are made by the same layout and renderer the export uses
//! ([`quadrule_text::lay_out`], [`quadrule_layout::text_path`], [`quadrule_render::render`]), then
//! shown as pictures, so the window needs no font of its own for each family.

use crate::view::ViewerUi;
use egui::{
    Color32, ColorImage, Role, ScrollArea, Sense, TextureHandle, TextureOptions, WidgetInfo,
};
use kurbo::{Point, Shape};
use quadrule_i18n::Message;
use quadrule_layout::{DisplayList, Item, text_path};
use quadrule_model::{FontStyle, MM_PER_IN, ObjectKind, TextAlign, TextBox, Uuid};
use quadrule_render::{Math, render};
use quadrule_text::{FamilyInfo, FontRegistry, TextRequest, lay_out};
use std::collections::HashMap;

/// Point size the family names in the list are drawn at.
const NAME_PT: f64 = 11.0;
/// Height of a row of the list, in screen points.
const ROW_HEIGHT: f32 = 30.0;
/// The widest and tallest picture made, in pixels (vello_cpu's limit is 65535).
const MAX_PX: (u32, u32) = (4096, 1200);
/// Pictures kept; past this the cache starts again.
const MAX_PICTURES: usize = 600;

/// A line of text drawn as device pixels: black on white, 0 to 255.
pub struct Raster {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height` gray values.
    pub gray: Vec<u8>,
}

/// Draws `tb` on one line as the export would at `px_per_mm`, black on white, or `None` when it
/// has nothing to draw or cannot be laid out (a family missing here).
pub fn raster_text(reg: &mut FontRegistry, tb: &TextBox, px_per_mm: f64) -> Option<Raster> {
    // One line: the box is as wide as a page ever is.
    let req = TextRequest::new(Uuid::from_u128(1), tb, 100_000.0);
    let laid = lay_out(reg, "preview", &[req]).ok()?;
    let text = laid.outlines.texts.first()?;
    if text.glyphs.is_empty() {
        return None;
    }
    let pad = 2.0;
    let path = text_path(&laid.outlines, text, Point::new(pad, pad), px_per_mm).ok()?;
    let b = path.bounding_box();
    let width = ((b.x1 + pad).ceil() as u32).clamp(1, MAX_PX.0);
    let height = ((f64::from(text.height_mm) * px_per_mm + 2.0 * pad).ceil() as u32)
        .max((b.y1 + pad).ceil() as u32)
        .clamp(1, MAX_PX.1);
    let dl = DisplayList {
        width,
        height,
        background: 255,
        items: vec![Item::Fill { path, gray: 0 }],
    };
    let img = render(&dl, Math::Fastest);
    Some(Raster {
        width,
        height,
        gray: img.data,
    })
}

/// A text box for a picture of `text` in `family`.
fn sample_box(family: &str, text: &str, size_pt: f64, like: Option<&TextBox>) -> TextBox {
    TextBox {
        content: text.into(),
        family: family.into(),
        weight: like.map_or(400, |t| t.weight),
        style: like.map_or(FontStyle::Normal, |t| t.style),
        size_pt,
        line_height: like.map_or(1.25, |t| t.line_height),
        letter_spacing_em: like.map_or(0.0, |t| t.letter_spacing_em),
        align: TextAlign::Left,
        ..TextBox::default()
    }
}

/// Pictures of lines of text made with a registry of its own, kept as textures by key.
#[derive(Default)]
pub(crate) struct Pictures {
    /// A registry over the same list of installed fonts, with a layout store of its own, so the
    /// previews do not fill the page's store with every family shown.
    pub reg: Option<FontRegistry>,
    pics: HashMap<String, Option<(TextureHandle, [u32; 2])>>,
}

impl Pictures {
    /// Starts again over `fonts`' families.
    pub(crate) fn reset(&mut self, fonts: &FontRegistry) {
        self.reg = Some(fonts.sibling());
        self.pics.clear();
    }

    pub(crate) fn picture(
        &mut self,
        ctx: &egui::Context,
        key: String,
        tb: &TextBox,
        px_per_mm: f64,
        ink: bool,
    ) -> Option<(egui::TextureId, egui::Vec2)> {
        if self.pics.len() > MAX_PICTURES {
            self.pics.clear();
        }
        let entry = self.pics.entry(key.clone()).or_insert_with(|| {
            let reg = self.reg.as_mut()?;
            let r = raster_text(reg, tb, px_per_mm)?;
            // Names take the text colour of the window (a mask); the preview is the export's own
            // black on white.
            let pixels: Vec<Color32> = r
                .gray
                .iter()
                .map(|&g| {
                    if ink {
                        Color32::from_white_alpha(255 - g)
                    } else {
                        Color32::from_gray(g)
                    }
                })
                .collect();
            let img = ColorImage::new([r.width as usize, r.height as usize], pixels);
            let tex = ctx.load_texture(format!("font-{key}"), img, TextureOptions::NEAREST);
            Some((tex, [r.width, r.height]))
        });
        let ppp = ctx.pixels_per_point();
        entry
            .as_ref()
            .map(|(t, [w, h])| (t.id(), egui::vec2(*w as f32 / ppp, *h as f32 / ppp)))
    }
}

/// What a text box has to say about the picker while it is open.
#[derive(Default)]
pub(crate) struct FontPicker {
    /// The window shows.
    pub open: bool,
    search: String,
    focus_search: bool,
    /// The family under the pointer on the last frame, whose preview shows.
    hovered: Option<String>,
    families: Vec<FamilyInfo>,
    pub(crate) pictures: Pictures,
}

impl FontPicker {
    /// Opens the window with a fresh list of families.
    pub(crate) fn open(&mut self, fonts: &FontRegistry) {
        self.pictures.reset(fonts);
        self.families = self
            .pictures
            .reg
            .as_ref()
            .map(|r| r.families())
            .unwrap_or_default();
        self.search.clear();
        self.focus_search = true;
        self.open = true;
    }
}

/// One line of the list.
enum Row<'a> {
    Header(&'static str),
    Family(&'a FamilyInfo),
}

impl ViewerUi {
    /// The Font row of the Text tab: the family in use, and the button that opens the picker.
    pub(crate) fn font_button(&mut self, ui: &mut egui::Ui, label: egui::Id, family: &str) {
        let exists = self.viewer.fonts.has_family(family);
        let r = ui
            .button(family)
            .on_hover_text(self.t("app-font-pick-tip"))
            .labelled_by(label);
        if r.clicked() {
            self.font_picker_for = None;
            let fonts = &self.viewer.fonts;
            self.font_picker.open(fonts);
        }
        if !exists {
            ui.colored_label(ui.visuals().warn_fg_color, self.t("app-font-missing"));
        }
    }

    /// The picker window, while it is open and a text box is picked.
    pub(crate) fn font_picker_window(&mut self, ctx: &egui::Context) {
        if !self.font_picker.open {
            return;
        }
        // The text box the pick is for, and the device the preview is drawn for. Choosing a
        // substitute for a missing font (Other font… in the Missing fonts dialog) shows the first
        // box that uses it.
        let target = match &self.font_picker_for {
            Some(missing) => self
                .viewer
                .missing_fonts()
                .iter()
                .find(|m| &m.family == missing)
                .and_then(|m| m.boxes.first().copied()),
            None => self.viewer.selection.first().copied(),
        };
        let Some(obj) = target
            .and_then(|id| self.viewer.doc().and_then(|d| d.object(id)))
            .cloned()
        else {
            self.font_picker.open = false;
            self.font_picker_for = None;
            return;
        };
        let ObjectKind::Text(tb) = &obj.kind else {
            self.font_picker.open = false;
            self.font_picker_for = None;
            return;
        };
        let Some((_, shown)) = self.viewer.profiles() else {
            return;
        };
        let device = quadrule_profiles::builtin(&shown.id)
            .map(|p| p.display_name)
            .unwrap_or_else(|_| shown.id.clone());
        let device_px_per_mm = shown.ppi / MM_PER_IN;
        let name_px_per_mm = f64::from(ctx.pixels_per_point()) * 96.0 / MM_PER_IN;
        let mut picked: Option<String> = None;
        let mut open = true;
        egui::Window::new(self.t("app-font-picker-title"))
            .open(&mut open)
            .collapsible(false)
            .default_width(420.0)
            .default_height(520.0)
            .show(ctx, |ui| {
                let search = egui::TextEdit::singleline(&mut self.font_picker.search)
                    .hint_text(self.strings.get("app-font-search"))
                    .desired_width(f32::INFINITY);
                let r = ui.add(search);
                r.widget_info(|| {
                    WidgetInfo::labeled(
                        Role::SearchInput,
                        true,
                        self.strings.get("app-font-search"),
                    )
                });
                if std::mem::take(&mut self.font_picker.focus_search) {
                    r.request_focus();
                }
                let needle = self.font_picker.search.trim().to_lowercase();
                let families = std::mem::take(&mut self.font_picker.families);
                let shown_families: Vec<&FamilyInfo> = families
                    .iter()
                    .filter(|f| needle.is_empty() || f.name.to_lowercase().contains(&needle))
                    .collect();
                let mut rows: Vec<Row> = Vec::new();
                for (bundled, header) in [(true, "app-font-bundled"), (false, "app-font-system")] {
                    let mut section = shown_families
                        .iter()
                        .filter(|f| f.bundled == bundled)
                        .peekable();
                    if section.peek().is_some() {
                        rows.push(Row::Header(header));
                        rows.extend(section.map(|f| Row::Family(f)));
                    }
                }
                // The preview: the family under the pointer, else the one in use.
                let preview_family: String = self
                    .font_picker
                    .hovered
                    .clone()
                    .filter(|f| families.iter().any(|x| &x.name == f))
                    .unwrap_or_else(|| tb.family.clone());
                ui.add_space(4.0);
                let text = tb
                    .content
                    .lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("");
                let text = if text.is_empty() {
                    preview_family.as_str()
                } else {
                    text
                };
                let sample = sample_box(&preview_family, text, tb.size_pt, Some(tb));
                let key = format!(
                    "p|{preview_family}|{text}|{}|{:?}|{}|{}|{}",
                    tb.weight, tb.style, tb.size_pt, tb.letter_spacing_em, device_px_per_mm
                );
                ui.label(
                    self.strings.format(
                        &Message::new("app-font-preview")
                            .num("size", tb.size_pt, 1)
                            .text("device", device.clone()),
                    ),
                );
                if let Some(reg) = self.font_picker.pictures.reg.as_mut() {
                    reg.ensure_open([preview_family.as_str()]);
                }
                let pic =
                    self.font_picker
                        .pictures
                        .picture(ctx, key, &sample, device_px_per_mm, false);
                egui::Frame::new().fill(Color32::WHITE).show(ui, |ui| {
                    ScrollArea::horizontal()
                        .id_salt("font-preview")
                        .show(ui, |ui| {
                            ui.set_min_height(60.0);
                            if let Some((id, size)) = pic {
                                ui.add(egui::Image::new((id, size)));
                            }
                        });
                });
                ui.separator();
                if rows.is_empty() {
                    ui.label(self.strings.get("app-font-none-found"));
                }
                let mut hovered: Option<String> = None;
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, ROW_HEIGHT, rows.len(), |ui, range| {
                        // Open the installed families on show once, not one by one.
                        let visible: Vec<&str> = rows[range.clone()]
                            .iter()
                            .filter_map(|r| match r {
                                Row::Family(f) => Some(f.name.as_str()),
                                Row::Header(_) => None,
                            })
                            .collect();
                        if let Some(reg) = self.font_picker.pictures.reg.as_mut() {
                            reg.ensure_open(visible);
                        }
                        for row in &rows[range] {
                            match row {
                                Row::Header(id) => {
                                    ui.add_sized(
                                        [ui.available_width(), ROW_HEIGHT],
                                        egui::Label::new(
                                            egui::RichText::new(self.strings.get(id)).strong(),
                                        )
                                        .selectable(false),
                                    );
                                }
                                Row::Family(f) => {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), ROW_HEIGHT),
                                        Sense::click(),
                                    );
                                    let current = f.name == tb.family;
                                    resp.widget_info(|| {
                                        WidgetInfo::selected(Role::Button, true, current, &f.name)
                                    });
                                    if current || resp.hovered() {
                                        let fill = if current {
                                            ui.visuals().selection.bg_fill
                                        } else {
                                            ui.visuals().widgets.hovered.bg_fill
                                        };
                                        ui.painter().rect_filled(rect, 3.0, fill);
                                    }
                                    let name_box = sample_box(&f.name, &f.name, NAME_PT, None);
                                    let key = format!("n|{}|{name_px_per_mm}", f.name);
                                    let pic = self.font_picker.pictures.picture(
                                        ctx,
                                        key,
                                        &name_box,
                                        name_px_per_mm,
                                        true,
                                    );
                                    let ink = ui.visuals().text_color();
                                    let pos = rect.left_center() + egui::vec2(6.0, 0.0);
                                    match pic {
                                        Some((id, size)) => {
                                            let size = if size.y > ROW_HEIGHT - 4.0 {
                                                size * ((ROW_HEIGHT - 4.0) / size.y)
                                            } else {
                                                size
                                            };
                                            let r = egui::Rect::from_min_size(
                                                pos - egui::vec2(0.0, size.y / 2.0),
                                                size,
                                            );
                                            ui.painter().image(
                                                id,
                                                r,
                                                egui::Rect::from_min_max(
                                                    egui::pos2(0.0, 0.0),
                                                    egui::pos2(1.0, 1.0),
                                                ),
                                                ink,
                                            );
                                        }
                                        None => {
                                            ui.painter().text(
                                                pos,
                                                egui::Align2::LEFT_CENTER,
                                                &f.name,
                                                egui::FontId::proportional(14.0),
                                                ink,
                                            );
                                        }
                                    }
                                    if resp.hovered() {
                                        hovered = Some(f.name.clone());
                                    }
                                    if resp.clicked() {
                                        picked = Some(f.name.clone());
                                    }
                                }
                            }
                        }
                    });
                self.font_picker.hovered = hovered;
                self.font_picker.families = families;
            });
        self.font_picker.open = open && self.font_picker.open;
        if !self.font_picker.open {
            self.font_picker_for = None;
        }
        if let Some(family) = picked {
            if let Some(missing) = self.font_picker_for.take() {
                // A substitute for a missing font: the dialog's row takes it.
                self.picked_for_missing = Some((missing, family));
                self.font_picker.open = false;
                return;
            }
            // One undo step per pick.
            if family != tb.family {
                let _ = self.viewer.edit_selected(None, |x| {
                    if let ObjectKind::Text(t) = &mut x.kind {
                        t.family.clone_from(&family);
                    }
                });
            }
            self.font_picker.open = false;
        }
    }
}
