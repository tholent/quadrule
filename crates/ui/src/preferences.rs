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

//! The Preferences window (rung 17 bbq, Q3 and Defaulted 4): Interface size, and the theme, which moved here from the
//! View menu. Both are app settings (kept between runs by the app, never in a document). Opened by Ctrl+, , the File
//! menu's Preferences…, and the Mac's Quadrule menu.
//!
//! Interface size is applied on top of the system's own scale (egui's zoom factor multiplies the screen's), so
//! **Follow system** and **100 %** give the same size; the choice is kept as it was made.

use crate::view::ViewerUi;
use egui::ThemePreference;

/// How big the interface is drawn, on top of the system's own scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterfaceSize {
    /// The system's scale, as it is.
    Follow,
    /// This many per cent of it (one of [`InterfaceSize::PERCENTS`]).
    Percent(u16),
}

impl InterfaceSize {
    /// The percentages the window offers.
    pub const PERCENTS: [u16; 6] = [75, 100, 125, 150, 175, 200];

    /// Every choice, in the order the window lists them.
    pub fn all() -> Vec<InterfaceSize> {
        std::iter::once(InterfaceSize::Follow)
            .chain(Self::PERCENTS.into_iter().map(InterfaceSize::Percent))
            .collect()
    }

    /// egui's zoom factor for it.
    pub fn factor(self) -> f32 {
        match self {
            InterfaceSize::Follow => 1.0,
            InterfaceSize::Percent(p) => f32::from(p) / 100.0,
        }
    }

    /// The text the app keeps between runs.
    pub fn key(self) -> String {
        match self {
            InterfaceSize::Follow => "follow".to_string(),
            InterfaceSize::Percent(p) => p.to_string(),
        }
    }

    /// The choice [`InterfaceSize::key`] kept, if it is one the window offers.
    pub fn from_key(key: &str) -> Option<InterfaceSize> {
        if key == "follow" {
            return Some(InterfaceSize::Follow);
        }
        let p: u16 = key.parse().ok()?;
        Self::PERCENTS
            .contains(&p)
            .then_some(InterfaceSize::Percent(p))
    }
}

impl ViewerUi {
    /// The words on one Interface size choice.
    pub(crate) fn interface_size_words(&self, size: InterfaceSize) -> String {
        match size {
            InterfaceSize::Follow => self.t("prefs-size-follow"),
            InterfaceSize::Percent(p) => {
                self.strings
                    .format(&quadrule_i18n::Message::new("prefs-size-percent").num(
                        "percent",
                        f64::from(p),
                        0,
                    ))
            }
        }
    }

    /// Opens the Preferences window.
    pub fn open_preferences(&mut self) {
        self.preferences_open = true;
    }

    /// Whether the Preferences window shows.
    pub fn preferences_showing(&self) -> bool {
        self.preferences_open
    }

    /// Sets the theme (Preferences, and what the app read back at start).
    pub fn set_theme(&mut self, ctx: &egui::Context, theme: ThemePreference) {
        self.theme = theme;
        ctx.set_theme(theme);
    }

    /// Draws the interface at the size chosen, each frame (egui does nothing when it is already so), and takes
    /// egui's own Ctrl + plus, minus and zero for the page's zoom (they would resize the whole interface).
    pub(crate) fn apply_interface_size(&self, ctx: &egui::Context) {
        if ctx.options(|o| o.zoom_with_keyboard) {
            ctx.options_mut(|o| o.zoom_with_keyboard = false);
        }
        let want = self.interface_size.factor();
        if (ctx.zoom_factor() - want).abs() > 1e-4 {
            ctx.set_zoom_factor(want);
        }
    }

    /// The Preferences window.
    pub(crate) fn preferences_window(&mut self, ctx: &egui::Context) {
        if !self.preferences_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        let mut size = self.interface_size;
        let mut theme = self.theme;
        let mut profiles = false;
        egui::Window::new(self.t("prefs-title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                egui::Grid::new("prefs-grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        let label = ui.label(self.t("prefs-size"));
                        egui::ComboBox::from_id_salt("prefs-size")
                            .selected_text(self.interface_size_words(size))
                            .show_ui(ui, |ui| {
                                for choice in InterfaceSize::all() {
                                    ui.selectable_value(
                                        &mut size,
                                        choice,
                                        self.interface_size_words(choice),
                                    );
                                }
                            })
                            .response
                            .labelled_by(label.id);
                        ui.end_row();
                        ui.label("");
                        ui.label(
                            egui::RichText::new(self.t("prefs-size-note"))
                                .small()
                                .weak(),
                        );
                        ui.end_row();

                        let label = ui.label(self.t("prefs-theme"));
                        ui.vertical(|ui| {
                            ui.radio_value(
                                &mut theme,
                                ThemePreference::System,
                                self.t("app-theme-system"),
                            );
                            ui.radio_value(
                                &mut theme,
                                ThemePreference::Light,
                                self.t("app-theme-light"),
                            );
                            ui.radio_value(
                                &mut theme,
                                ThemePreference::Dark,
                                self.t("app-theme-dark"),
                            );
                        })
                        .response
                        .labelled_by(label.id);
                        ui.end_row();
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t("prefs-profiles")).clicked() {
                        profiles = true;
                    }
                    if ui.button(self.t("about-close")).clicked() {
                        close = true;
                    }
                });
            });
        if size != self.interface_size {
            self.interface_size = size;
            ctx.request_repaint();
        }
        if theme != self.theme {
            self.set_theme(ctx, theme);
        }
        if profiles {
            self.open_profiles_window();
        }
        if !open || close {
            self.preferences_open = false;
        }
    }
}
