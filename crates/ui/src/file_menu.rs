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

//! The File menu (rung 16 bbq, Q6, Q9 and Defaulted "The File menu, in detail"): New… Ctrl+N, Open… Ctrl+O, Open
//! Recent, Save Ctrl+S, Save As… Ctrl+Shift+S, Save as template…, Export… Ctrl+E and Close Ctrl+W. Since fixes batch 2
//! it is the first menu of the menu bar ([`crate::menus`], [`crate::menu_bar`]), and the toolbar has no File button.
//! What is left here is Open Recent and the "is a document on screen" question the items ask.

use crate::view::{Request, ViewerUi};
use egui::Button;
use quadrule_i18n::Message;

impl ViewerUi {
    /// Whether a document is on screen: one is open and the welcome screen is not over it.
    pub fn in_document(&self) -> bool {
        self.viewer.open.is_some() && !self.welcome.over_document
    }

    /// Opens the File menu on the next frame (`--look file-menu`, for a screenshot).
    pub fn look_file_menu(&mut self) {
        self.look_menu = Some(crate::menu_bar::LookMenu::top("app-file"));
    }

    /// Open Recent: the recent list, a file that is gone greyed and marked Not found.
    pub(crate) fn open_recent_menu(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.menu_button(self.t("app-file-open-recent"), |ui| {
            let recent = self.viewer.recent_files();
            if recent.is_empty() {
                ui.add_enabled(false, Button::new(self.t("app-file-recent-none")));
            }
            for (i, r) in recent.into_iter().enumerate() {
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
                    requests.extend(self.run_cmd(ui.ctx(), crate::Cmd::OpenRecent(i)));
                }
            }
        });
    }
}
