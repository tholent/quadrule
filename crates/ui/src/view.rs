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

//! The viewer window's widgets (napkin §6.1, rung 2 bbq Q4): toolbar, canvas, lint panel and
//! status bar. Everything a person reads comes from `quadrule_i18n` (Q5).
//!
//! The widgets never open a system dialog themselves: they return a [`Request`], and the app
//! answers it (the file dialog lives in `quadrule-app`), so this crate runs headless in tests.

use crate::viewer::{
    CARD_WIDTH_MM, ExportWhich, PIXEL_GRID_FROM, TOOLBARS_IN_USE, Tile, Viewer, Zoom,
    ruler_px_per_mm, shrink, tiles,
};
use egui::{
    Align2, Color32, ColorImage, CornerRadius, FontId, Rect, ScrollArea, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, ThemePreference, pos2, vec2,
};
use quadrule_i18n::{Message, Strings};
use quadrule_lint::{Severity, Toolbar};
use std::path::PathBuf;

/// Something the window asks the app to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Show the system dialog to pick a `.quad` file.
    OpenDialog,
    /// Show the system dialog to pick a folder, then export.
    ExportDialog(ExportWhich),
}

/// Texture tiles are at most this many pixels a side.
const TILE: u32 = 1024;

/// What the page texture shows: which document, which device, at what size.
#[derive(Clone, Debug, PartialEq)]
struct TextureKey {
    generation: u64,
    profile: String,
    size: (u32, u32),
}

/// The ruler check's references.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Reference {
    Card,
    Ruler100,
}

impl Reference {
    fn mm(self) -> f64 {
        match self {
            Reference::Card => CARD_WIDTH_MM,
            Reference::Ruler100 => 100.0,
        }
    }
}

/// The ruler check while it is open.
#[derive(Clone, Copy, Debug)]
struct Ruler {
    reference: Reference,
    /// The bar's length in points.
    bar: f32,
}

/// The viewer window: state plus what only the screen needs (the page texture, the dialogs).
pub struct ViewerUi {
    /// The viewer's state.
    pub viewer: Viewer,
    /// The words on screen.
    pub strings: Strings,
    /// Light, dark or follow the system (rung 2 bbq, Q11).
    pub theme: ThemePreference,
    /// Whether the lint panel shows.
    pub show_lint: bool,
    texture: Option<(TextureKey, Vec<(Tile, TextureHandle)>)>,
    ruler: Option<Ruler>,
    scroll_to_selected: bool,
    cursor_px: Option<(f32, f32)>,
    canvas_size: Option<egui::Vec2>,
}

impl ViewerUi {
    /// A window on the empty start screen.
    pub fn new(viewer: Viewer) -> Self {
        ViewerUi {
            viewer,
            strings: Strings::english(),
            theme: ThemePreference::System,
            show_lint: true,
            texture: None,
            ruler: None,
            scroll_to_selected: false,
            cursor_px: None,
            canvas_size: None,
        }
    }

    fn t(&self, id: &str) -> String {
        self.strings.get(id)
    }

    /// Draws the whole window into `ui` and returns what the app must do.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Request> {
        let mut requests = Vec::new();
        self.take_dropped_files(ui.ctx());
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui, &mut requests));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        if self.show_lint && self.viewer.open.is_some() {
            egui::Panel::right("lint")
                .default_size(320.0)
                .show(ui, |ui| self.lint_panel(ui));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(backdrop(ui.visuals())))
            .show(ui, |ui| {
                if self.viewer.open.is_some() {
                    self.canvas(ui);
                } else {
                    self.empty(ui, &mut requests);
                }
            });
        self.ruler_window(ui.ctx());
        requests
    }

    fn take_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if let Some(path) = dropped.first() {
            let _ = self.viewer.open_path(path);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.horizontal_wrapped(|ui| {
            if ui.button(self.t("app-open")).clicked() {
                requests.push(Request::OpenDialog);
            }
            ui.separator();

            // Device.
            let ids = self.viewer.profile_ids();
            let name = |id: &str| {
                quadrule_profiles::builtin(id)
                    .map(|p| p.display_name)
                    .unwrap_or_else(|_| id.to_string())
            };
            let current = self.viewer.profile_id.clone();
            let mut chosen = current.clone();
            // Named for screen readers (napkin §14, AccessKit) by the label beside it.
            let label = ui.label(self.t("app-device"));
            egui::ComboBox::from_id_salt("profile")
                .selected_text(name(&current))
                .show_ui(ui, |ui| {
                    for id in &ids {
                        ui.selectable_value(&mut chosen, id.clone(), name(id));
                    }
                })
                .response
                .labelled_by(label.id);
            if chosen != current {
                self.viewer.set_profile(&chosen);
            }
            ui.separator();

            // Zoom (rung 2 bbq, Q10).
            let open = self.viewer.open.is_some();
            ui.add_enabled_ui(open, |ui| {
                if ui.button(self.t("app-zoom-fit")).clicked() {
                    self.viewer.zoom = Zoom::Fit;
                }
                if ui.button(self.t("app-zoom-actual")).clicked() {
                    self.viewer.zoom = Zoom::Scale(1.0);
                }
                let now = self.shown_scale(ui);
                if ui
                    .button("−")
                    .on_hover_text(self.t("app-zoom-out"))
                    .clicked()
                {
                    self.viewer.zoom_step(now, -1);
                }
                if ui
                    .button("+")
                    .on_hover_text(self.t("app-zoom-in"))
                    .clicked()
                {
                    self.viewer.zoom_step(now, 1);
                }
                if ui.button(self.t("app-zoom-real")).clicked() {
                    if self.viewer.screen_px_per_mm.is_some() {
                        self.viewer.zoom = Zoom::RealSize;
                    } else {
                        self.open_ruler(ui.ctx());
                    }
                }
            });
            ui.separator();

            // Lint count, which shows or hides the panel.
            let count = self.lint_count();
            let label =
                self.strings
                    .format(&Message::new("app-lint-count").num("count", count as f64, 0));
            if ui
                .add_enabled(open, egui::Button::selectable(self.show_lint, label))
                .clicked()
            {
                self.show_lint = !self.show_lint;
            }
            ui.separator();

            // Export (rung 2 bbq, Q8).
            ui.add_enabled_ui(open, |ui| {
                ui.menu_button(self.t("app-export"), |ui| {
                    if ui.button(self.t("app-export-this")).clicked() {
                        requests.push(Request::ExportDialog(ExportWhich::ThisDevice));
                    }
                    if ui.button(self.t("app-export-all")).clicked() {
                        requests.push(Request::ExportDialog(ExportWhich::AllDevices));
                    }
                });
            });

            // View.
            ui.menu_button(self.t("app-view"), |ui| {
                ui.label(self.t("app-theme"));
                for (pref, id) in [
                    (ThemePreference::System, "app-theme-system"),
                    (ThemePreference::Light, "app-theme-light"),
                    (ThemePreference::Dark, "app-theme-dark"),
                ] {
                    if ui.radio(self.theme == pref, self.t(id)).clicked() {
                        self.theme = pref;
                        ui.ctx().set_theme(pref);
                    }
                }
                ui.separator();
                let label = self.t("app-safe-zones");
                ui.checkbox(&mut self.viewer.show_safe_zones, label);
                if ui.button(self.t("app-ruler-open")).clicked() {
                    self.open_ruler(ui.ctx());
                }
            });
        });
    }

    fn open_ruler(&mut self, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point();
        // Start from what the system claims (96 points per inch), so the bar is near right.
        let bar = match self.viewer.screen_px_per_mm {
            Some(r) => (r * CARD_WIDTH_MM) as f32 / ppp,
            None => (CARD_WIDTH_MM / quadrule_model::MM_PER_IN * 96.0) as f32,
        };
        self.ruler = Some(Ruler {
            reference: Reference::Card,
            bar,
        });
    }

    fn lint_count(&mut self) -> usize {
        self.viewer.page().map_or(0, |p| p.findings.len())
    }

    /// Physical screen pixels per device pixel now showing.
    fn shown_scale(&mut self, ui: &egui::Ui) -> f32 {
        // The canvas's own size is known once it has been drawn (the toolbar comes first in a
        // frame); until then, the window's.
        let ppp = ui.ctx().pixels_per_point();
        let canvas = self
            .canvas_size
            .unwrap_or_else(|| ui.ctx().content_rect().size());
        self.viewer
            .scale((canvas.x, canvas.y), ppp)
            .map_or(1.0, |s| s * ppp)
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some((x, y)) = self.cursor_px
                && let Some(page) = self.viewer.page()
            {
                let mm = |px: f32| px as f64 * quadrule_model::MM_PER_IN / page.profile.ppi;
                let msg = Message::new("app-status-cursor")
                    .num("x", x.floor() as f64, 0)
                    .num("y", y.floor() as f64, 0)
                    .num("xmm", mm(x), 1)
                    .num("ymm", mm(y), 1);
                ui.label(self.strings.format(&msg));
                ui.separator();
            }
            if self.viewer.open.is_some() {
                let pct = self.shown_scale(ui) * 100.0;
                let msg = Message::new("app-status-zoom").num("percent", pct as f64, 0);
                ui.label(self.strings.format(&msg));
                ui.separator();
                let count = self.lint_count();
                let msg = Message::new("app-lint-count").num("count", count as f64, 0);
                ui.label(self.strings.format(&msg));
            }
            if let Some(n) = &self.viewer.notice {
                ui.separator();
                let text = self.strings.format(&n.message);
                if n.error {
                    ui.colored_label(ui.visuals().error_fg_color, text);
                } else {
                    ui.label(text);
                }
            }
        });
    }

    fn lint_panel(&mut self, ui: &mut egui::Ui) {
        let strings = &self.strings;
        let Some(page) = self.viewer.page() else {
            return;
        };
        let findings = page.findings.clone();
        let saved = page.saved_outlines;
        ui.heading(strings.format(&Message::new("app-lint-heading").num(
            "count",
            findings.len() as f64,
            0,
        )));
        ui.label(strings.get("app-lint-toolbars"));
        if saved {
            ui.label(strings.get("app-saved-outlines"));
        }
        ui.separator();
        ScrollArea::vertical().show(ui, |ui| {
            if findings.is_empty() {
                ui.label(strings.get("app-lint-none"));
            }
            for (i, f) in findings.iter().enumerate() {
                let icon = match f.severity {
                    Severity::Warn => "⚠",
                    Severity::Info => "ℹ",
                };
                let text = format!("{icon} {}  {}", f.rule, strings.format(&f.message));
                let selected = self.viewer.selected == Some(i);
                let r = ui.add(egui::Button::selectable(selected, text).wrap());
                if r.clicked() {
                    self.viewer.selected = if selected { None } else { Some(i) };
                    self.scroll_to_selected = self.viewer.selected.is_some();
                }
            }
        });
    }

    fn empty(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.heading(self.t("app-empty-title"));
            ui.add_space(12.0);
            if ui.button(self.t("app-open")).clicked() {
                requests.push(Request::OpenDialog);
            }
            if ui.button(self.t("app-open-sample")).clicked() {
                self.viewer.open_sample();
            }
            ui.add_space(8.0);
            ui.weak(self.t("app-empty-drop"));
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let avail = ui.available_size();
        self.canvas_size = Some(avail);
        let Some(scale) = self.viewer.scale((avail.x, avail.y), ppp) else {
            return;
        };
        let generation = self.viewer.generation;
        let selected = self.viewer.selected;
        let Some(page) = self.viewer.page() else {
            return;
        };
        let (w, h) = (page.profile.width_px, page.profile.height_px);
        // Zoomed in or at 1:1, the export pixels unchanged, drawn as squares (Q3, Q10). Zoomed
        // out, the page shrunk by averaging to exactly the screen pixels it covers (`shrink`).
        let phys = scale * ppp;
        let (tw, th) = if phys < 1.0 {
            (
                ((w as f32 * phys).round() as u32).max(1),
                ((h as f32 * phys).round() as u32).max(1),
            )
        } else {
            (w, h)
        };
        let key = TextureKey {
            generation,
            profile: page.profile.id.clone(),
            size: (tw, th),
        };
        if self.texture.as_ref().is_none_or(|(k, _)| *k != key) {
            let image = if (tw, th) == (w, h) {
                page.image.clone()
            } else {
                shrink(&page.image, tw, th)
            };
            // In tiles, because a Manta page (1920 × 2560) is larger than some graphics chips
            // take in one texture (egui's test harness: 2048).
            let tiles = tiles(tw, th, TILE)
                .into_iter()
                .map(|t| {
                    let (x, y, tw_, th_) = t;
                    let mut data = Vec::with_capacity((tw_ * th_) as usize);
                    for row in y..y + th_ {
                        let start = (row * tw + x) as usize;
                        data.extend_from_slice(&image.data[start..start + tw_ as usize]);
                    }
                    let img = ColorImage::from_gray([tw_ as usize, th_ as usize], &data);
                    let tex =
                        ctx.load_texture(format!("page-{x}-{y}"), img, TextureOptions::NEAREST);
                    (t, tex)
                })
                .collect();
            self.texture = Some((key, tiles));
        }
        let Some((_, tiles)) = &self.texture else {
            return;
        };
        let tiles: Vec<_> = tiles.iter().map(|(t, tex)| (*t, tex.id())).collect();
        let zones: Vec<_> = page.profile.safe_zones.clone();
        let area = selected
            .and_then(|i| page.findings.get(i))
            .and_then(|f| f.area);

        let size = if phys < 1.0 {
            vec2(tw as f32 / ppp, th as f32 / ppp)
        } else {
            vec2(w as f32 * scale, h as f32 * scale)
        };
        let margin = 16.0;
        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            let total = vec2(
                (size.x + 2.0 * margin).max(avail.x),
                (size.y + 2.0 * margin).max(avail.y),
            );
            let (rect, response) = ui.allocate_exact_size(total, Sense::hover());
            // The page's corner sits on a whole screen pixel, so 1:1 and the zoom steps stay
            // sharp.
            let min = rect.center() - size / 2.0;
            let min = pos2((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
            let page_rect = Rect::from_min_size(min, size);
            let painter = ui.painter_at(rect);
            let to_screen =
                |x: f64, y: f64| pos2(min.x + x as f32 * scale, min.y + y as f32 * scale);

            painter.rect_stroke(
                page_rect.expand(1.0),
                CornerRadius::ZERO,
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                StrokeKind::Outside,
            );
            for ((x, y, tw_, th_), id) in &tiles {
                let at = |px: u32, py: u32| {
                    min + vec2(
                        px as f32 / tw as f32 * size.x,
                        py as f32 / th as f32 * size.y,
                    )
                };
                painter.image(
                    *id,
                    Rect::from_min_max(at(*x, *y), at(x + tw_, y + th_)),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }

            // A grid between device pixels from 8× (Q10), only where it shows.
            if scale * ppp >= PIXEL_GRID_FROM {
                let clip = painter.clip_rect().intersect(page_rect);
                let stroke = Stroke::new(
                    1.0 / ppp,
                    Color32::from_rgba_unmultiplied(128, 128, 255, 90),
                );
                let x0 = ((clip.min.x - min.x) / scale).floor().max(0.0) as u32;
                let x1 = (((clip.max.x - min.x) / scale).ceil() as u32).min(w);
                let y0 = ((clip.min.y - min.y) / scale).floor().max(0.0) as u32;
                let y1 = (((clip.max.y - min.y) / scale).ceil() as u32).min(h);
                for x in x0..=x1 {
                    let sx = min.x + x as f32 * scale;
                    painter.line_segment([pos2(sx, clip.min.y), pos2(sx, clip.max.y)], stroke);
                }
                for y in y0..=y1 {
                    let sy = min.y + y as f32 * scale;
                    painter.line_segment([pos2(clip.min.x, sy), pos2(clip.max.x, sy)], stroke);
                }
            }

            // Safe zones the linter checks (Q6): fixed ones red, movable ones amber.
            if self.viewer.show_safe_zones {
                for z in &zones {
                    if let Ok(Some(t)) = Toolbar::of_zone(&z.name)
                        && !TOOLBARS_IN_USE.contains(&t)
                    {
                        continue;
                    }
                    let (x, y, zw, zh) = z.rect;
                    let r = Rect::from_min_max(
                        to_screen(x as f64, y as f64),
                        to_screen((x + zw) as f64, (y + zh) as f64),
                    );
                    let color = if z.advisory {
                        Color32::from_rgb(230, 150, 0)
                    } else {
                        Color32::from_rgb(220, 40, 40)
                    };
                    painter.rect_filled(r, CornerRadius::ZERO, color.gamma_multiply(0.18));
                    painter.rect_stroke(
                        r,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, color),
                        StrokeKind::Inside,
                    );
                    painter.text(
                        r.left_top() + vec2(4.0, 4.0),
                        Align2::LEFT_TOP,
                        self.strings.get(&format!("lint-zone-{}", z.name)),
                        FontId::proportional(12.0),
                        color,
                    );
                }
            }

            // The selected finding's area (Q6).
            if let Some(a) = area {
                let r =
                    Rect::from_min_max(to_screen(a.x0, a.y0), to_screen(a.x1, a.y1)).expand(3.0);
                let accent = ui.visuals().selection.stroke.color;
                painter.rect_stroke(
                    r,
                    CornerRadius::same(2),
                    Stroke::new(2.0, accent),
                    StrokeKind::Outside,
                );
                if self.scroll_to_selected {
                    ui.scroll_to_rect(r, Some(egui::Align::Center));
                    self.scroll_to_selected = false;
                }
            }

            // Where the pointer is, in device pixels, for the status bar.
            self.cursor_px = response.hover_pos().and_then(|p| {
                let x = (p.x - min.x) / scale;
                let y = (p.y - min.y) / scale;
                (x >= 0.0 && y >= 0.0 && x < w as f32 && y < h as f32).then_some((x, y))
            });

            // Ctrl + scroll (or a pinch) zooms by one step.
            if response.hovered() {
                let z = ui.input(|i| i.zoom_delta());
                if z > 1.0 {
                    self.viewer.zoom_step(scale * ppp, 1);
                } else if z < 1.0 {
                    self.viewer.zoom_step(scale * ppp, -1);
                }
            }
        });
    }

    fn ruler_window(&mut self, ctx: &egui::Context) {
        let Some(mut ruler) = self.ruler else {
            return;
        };
        let ppp = ctx.pixels_per_point();
        let mut open = true;
        let mut done = None;
        egui::Window::new(self.t("app-ruler-title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(self.t("app-ruler-help"));
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut ruler.reference,
                        Reference::Card,
                        self.t("app-ruler-card"),
                    );
                    ui.radio_value(
                        &mut ruler.reference,
                        Reference::Ruler100,
                        self.t("app-ruler-100"),
                    );
                });
                ui.add(
                    egui::Slider::new(&mut ruler.bar, 100.0..=900.0)
                        .step_by(0.25)
                        .show_value(false)
                        .text(self.t("app-ruler-bar")),
                );
                ui.horizontal(|ui| {
                    if ui.small_button("−").clicked() {
                        ruler.bar -= 1.0 / ppp;
                    }
                    if ui.small_button("+").clicked() {
                        ruler.bar += 1.0 / ppp;
                    }
                });
                let (r, _) = ui.allocate_exact_size(vec2(ruler.bar, 28.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(r, CornerRadius::ZERO, ui.visuals().strong_text_color());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t("app-ruler-save")).clicked() {
                        done = Some(ruler_px_per_mm(ruler.bar, ppp, ruler.reference.mm()));
                    }
                });
            });
        if let Some(r) = done {
            self.viewer.screen_px_per_mm = Some(r);
            self.viewer.zoom = Zoom::RealSize;
            self.ruler = None;
        } else if open {
            self.ruler = Some(ruler);
        } else {
            self.ruler = None;
        }
    }

    /// Whether the ruler check is open.
    pub fn ruler_open(&self) -> bool {
        self.ruler.is_some()
    }
}

/// The backdrop round the page (napkin §14: it follows the theme; the page never does).
fn backdrop(v: &egui::Visuals) -> Color32 {
    if v.dark_mode {
        Color32::from_gray(40)
    } else {
        Color32::from_gray(200)
    }
}

/// The font for the controls: the system's interface font (chris, 2026-11-09: "I would like the
/// system default font for the controls"), found by the app. `weight` sets a variable font's
/// weight axis; a fixed font ignores it.
#[derive(Clone, Debug)]
pub struct UiFont {
    /// The font file's bytes.
    pub data: Vec<u8>,
    /// The face's index in the file.
    pub index: u32,
    /// 100 to 900; 400 is regular.
    pub weight: u16,
}

/// Puts `font` first for the controls, keeping egui's own fonts behind it for symbols such as ⚠
/// and for anything it lacks. With no font, egui's own fonts alone.
pub fn install_fonts(ctx: &egui::Context, font: Option<UiFont>) {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(f) = font {
        let mut data = egui::FontData::from_owned(f.data);
        data.index = f.index;
        data.tweak.coords = egui::epaint::text::VariationCoords::new([("wght", f.weight as f32)]);
        fonts
            .font_data
            .insert("system-ui".into(), std::sync::Arc::new(data));
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "system-ui".into());
        }
    }
    ctx.set_fonts(fonts);
}
