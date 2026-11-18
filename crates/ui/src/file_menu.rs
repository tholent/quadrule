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

//! The File menu (rung 16 bbq, Q6, Q9 and Defaulted "The File menu, in detail"): leftmost in the top bar. New…
//! Ctrl+N, Open… Ctrl+O, Open Recent, Save Ctrl+S, Save As… Ctrl+Shift+S, Save as template…, Export… Ctrl+E and Close
//! Ctrl+W. The bar no longer has New, Open, Save, Save As or Export of its own. The keys themselves are read in
//! `ViewerUi::shortcuts`; every item here does what its key does.

use crate::view::{After, Request, ViewerUi};
use egui::{Button, Key, KeyboardShortcut, Modifiers};
use quadrule_i18n::Message;

impl ViewerUi {
    /// Whether a document is on screen: one is open and the welcome screen is not over it.
    pub fn in_document(&self) -> bool {
        self.viewer.open.is_some() && !self.welcome.over_document
    }

    /// Opens the File menu on the next frame (`--look file-menu`, for a screenshot).
    pub fn look_file_menu(&mut self) {
        self.open_file_menu = true;
    }

    /// The File menu button and what it holds.
    pub(crate) fn file_menu(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        let cmd = Modifiers::COMMAND;
        let shift_cmd = Modifiers::COMMAND | Modifiers::SHIFT;
        let keys = |ui: &egui::Ui, m: Modifiers, k: Key| {
            ui.ctx().format_shortcut(&KeyboardShortcut::new(m, k))
        };
        let doc = self.in_document();
        let has_doc = self.viewer.open.is_some();
        let can_template = doc && self.viewer.templates_dir.is_some();
        let can_export = doc && !self.export_sheets_open();
        let menu = ui.menu_button(self.t("app-file"), |ui| {
            if ui
                .add(Button::new(self.t("app-new")).shortcut_text(keys(ui, cmd, Key::N)))
                .clicked()
            {
                self.ask_new();
            }
            if ui
                .add(Button::new(self.t("app-open")).shortcut_text(keys(ui, cmd, Key::O)))
                .clicked()
            {
                requests.extend(self.guard(After::OpenDialog));
            }
            self.open_recent_menu(ui, requests);
            ui.separator();
            if ui
                .add_enabled(
                    doc,
                    Button::new(self.t("app-save")).shortcut_text(keys(ui, cmd, Key::S)),
                )
                .clicked()
            {
                requests.extend(self.save());
            }
            if ui
                .add_enabled(
                    doc,
                    Button::new(self.t("app-save-as")).shortcut_text(keys(ui, shift_cmd, Key::S)),
                )
                .clicked()
            {
                self.ask(Request::SaveAsDialog(None), requests);
            }
            if ui
                .add_enabled(can_template, Button::new(self.t("app-file-save-template")))
                .clicked()
            {
                self.save_as_template();
            }
            ui.separator();
            if ui
                .add_enabled(
                    can_export,
                    Button::new(self.t("app-export-sheet")).shortcut_text(keys(ui, cmd, Key::E)),
                )
                .clicked()
            {
                self.open_export_sheet();
            }
            ui.separator();
            if ui
                .add_enabled(
                    has_doc,
                    Button::new(self.t("app-file-close")).shortcut_text(keys(ui, cmd, Key::W)),
                )
                .clicked()
            {
                requests.extend(self.ask_close_document());
            }
        });
        if std::mem::take(&mut self.open_file_menu) {
            egui::Popup::open_id(ui.ctx(), egui::Popup::default_response_id(&menu.response));
            ui.ctx().request_repaint();
        }
    }

    /// Open Recent: the recent list, a file that is gone greyed and marked Not found.
    fn open_recent_menu(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.menu_button(self.t("app-file-open-recent"), |ui| {
            let recent = self.viewer.recent_files();
            if recent.is_empty() {
                ui.add_enabled(false, Button::new(self.t("app-file-recent-none")));
            }
            for r in recent {
                let text = if r.exists {
                    r.name.clone()
                } else {
                    self.strings
                        .format(&Message::new("app-file-recent-gone").text("name", r.name.clone()))
                };
                let button = ui
                    .add_enabled(r.exists, Button::new(text))
                    .on_hover_text(r.path.display().to_string());
                if button.clicked() {
                    requests.extend(self.guard(After::OpenPath(r.path.clone())));
                }
            }
        });
    }
}
