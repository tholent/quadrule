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

//! The menu bar along the top of the window, above the toolbar, on Windows and Linux (fixes batch 2, Q6): File, Edit,
//! View and Help, drawn from the table in [`crate::menus`]. The Mac does not draw it (Q7: its menus are in the system's
//! bar). Also the About quadrule sheet.

use crate::menus::{Entry, Item, Kind, Menu, bar_menus};
use crate::view::{Request, ViewerUi};
use egui::{Button, Popup, Response};
use quadrule_i18n::Message;

/// A menu to hold open for a screenshot (`--look menu-edit` and the like): the top-level menu by its words' id, and a
/// menu inside it, if one is wanted open too.
#[derive(Clone, Debug)]
pub struct LookMenu {
    top: &'static str,
    sub: Option<&'static str>,
    top_open: bool,
    sub_open: bool,
}

impl LookMenu {
    /// The menu with this title open.
    pub fn top(top: &'static str) -> Self {
        LookMenu {
            top,
            sub: None,
            top_open: false,
            sub_open: false,
        }
    }

    /// The menu with this title open and, inside it, the one with this title.
    pub fn with_sub(top: &'static str, sub: &'static str) -> Self {
        LookMenu {
            sub: Some(sub),
            ..LookMenu::top(top)
        }
    }
}

/// Opens the popup the button `response` belongs to.
fn open_popup(ctx: &egui::Context, response: &Response) {
    Popup::open_id(ctx, Popup::default_response_id(response));
    ctx.request_repaint();
}

impl ViewerUi {
    /// The menu bar's own contents.
    pub(crate) fn menu_bar_ui(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        egui::MenuBar::new().ui(ui, |ui| {
            for menu in bar_menus() {
                let button = ui.menu_button(self.t(menu.label), |ui| {
                    self.menu_entries(ui, &menu.entries, requests);
                });
                let wanted = self
                    .look_menu
                    .as_mut()
                    .filter(|l| l.top == menu.label && !l.top_open);
                if let Some(look) = wanted {
                    look.top_open = true;
                    open_popup(ui.ctx(), &button.response);
                }
            }
        });
        // Nothing more to open: the look is done.
        if self
            .look_menu
            .as_ref()
            .is_some_and(|l| l.top_open && (l.sub.is_none() || l.sub_open))
        {
            self.look_menu = None;
        }
    }

    /// The lines of a menu or of a menu inside it.
    fn menu_entries(&mut self, ui: &mut egui::Ui, entries: &[Entry], requests: &mut Vec<Request>) {
        for entry in entries {
            match entry {
                Entry::Item(item) => self.menu_item(ui, item, requests),
                Entry::Separator => {
                    ui.separator();
                }
                Entry::Sub(sub) => self.sub_menu(ui, sub, requests),
                Entry::Recent => self.open_recent_menu(ui, requests),
                // The system's own items exist on the Mac only.
                Entry::Native(..) => {}
            }
        }
    }

    fn sub_menu(&mut self, ui: &mut egui::Ui, sub: &Menu, requests: &mut Vec<Request>) {
        let button = ui.menu_button(self.t(sub.label), |ui| {
            self.menu_entries(ui, &sub.entries, requests);
        });
        let wanted = self
            .look_menu
            .as_mut()
            .filter(|l| l.top_open && l.sub == Some(sub.label) && !l.sub_open);
        if let Some(look) = wanted {
            look.sub_open = true;
            open_popup(ui.ctx(), &button.response);
        }
    }

    /// One item: its words, its key to the right, greyed when it cannot act, ticked when it is on.
    fn menu_item(&mut self, ui: &mut egui::Ui, item: &Item, requests: &mut Vec<Request>) {
        let state = self.cmd_state(item.cmd);
        let label = self.t(item.label);
        let clicked = match (item.kind, state.checked) {
            (Kind::Check, Some(on)) => {
                let mut on = on;
                ui.add_enabled_ui(state.enabled, |ui| ui.checkbox(&mut on, label))
                    .inner
                    .clicked()
            }
            (Kind::Radio, Some(on)) => ui
                .add_enabled_ui(state.enabled, |ui| ui.radio(on, label))
                .inner
                .clicked(),
            _ => {
                let mut button = Button::new(label);
                if let Some(keys) = item.shortcut {
                    button = button.shortcut_text(crate::keys::key_text(
                        ui.ctx(),
                        keys.modifiers,
                        keys.logical_key,
                    ));
                }
                ui.add_enabled(state.enabled, button).clicked()
            }
        };
        if clicked {
            requests.extend(self.run_cmd(ui.ctx(), item.cmd));
        }
    }

    /// The About quadrule sheet (Help > About quadrule): the name, the version, the licence and the project's address.
    pub(crate) fn about_window(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        egui::Window::new(self.t("about-title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading(self.t("app-title"));
                ui.label(self.strings.format(
                    &Message::new("about-version").text("version", env!("CARGO_PKG_VERSION")),
                ));
                ui.add_space(4.0);
                ui.label(self.t("about-tagline"));
                ui.label(self.t("about-licence"));
                ui.add_space(4.0);
                // The address is shown as text to read or copy; the Help menu opens it.
                ui.add(egui::Label::new(crate::menus::PROJECT_URL).selectable(true));
                ui.add_space(8.0);
                if ui.button(self.t("about-close")).clicked() {
                    close = true;
                }
            });
        if !open || close {
            self.about_open = false;
        }
    }
}
