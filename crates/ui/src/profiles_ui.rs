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

//! The widgets for the profiles a file carries (rung 13a bbq, Q4, Q6, Q9): the bar above the canvas
//! ("Uses a profile you don't have: X. [Add to my profiles]") and the dialog that asks Merge, Rename or
//! Not now when a carried profile clashes with one here. What they decide is in [`crate::profile_use`].

use crate::profile_use::ProfileChoice;
use crate::view::ViewerUi;
use egui::{Color32, RichText};
use quadrule_i18n::{Arg, Message};
use quadrule_profiles::Incoming;

/// What a row of the clash dialog will do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// Merge: the file's values update the profile here.
    Merge,
    /// Rename the file's copy.
    Rename,
    /// Not now.
    NotNow,
}

/// One row: a carried profile that clashes.
#[derive(Clone, Debug)]
pub struct ClashRow {
    /// The id the file gives it.
    pub id: String,
    /// Its name in the file.
    pub name: String,
    /// The name of the profile here it clashes with.
    pub mine: String,
    /// Same id with other values, rather than the same name.
    pub same_id: bool,
    /// The choice.
    pub pick: Pick,
    /// The new name, when Rename is picked.
    pub new_name: String,
}

/// The clash dialog while it shows.
#[derive(Clone, Debug)]
pub struct ClashDialog {
    /// One row for each clash.
    pub rows: Vec<ClashRow>,
}

impl ViewerUi {
    /// Opens the clash dialog on the clashes the open file has, each set to Not now.
    pub fn open_clash_dialog(&mut self) {
        let Some(u) = self.viewer.profile_use() else {
            return;
        };
        let rows: Vec<ClashRow> = u
            .clashes
            .iter()
            .map(|c| ClashRow {
                id: c.incoming.id.clone(),
                name: c.incoming.display_name.clone(),
                mine: c.mine.clone(),
                same_id: c.kind == Incoming::IdClash,
                pick: Pick::NotNow,
                new_name: self.strings.format(
                    &Message::new("profiles-clash-new-name")
                        .text("name", c.incoming.display_name.clone()),
                ),
            })
            .collect();
        if !rows.is_empty() {
            self.clash_dialog = Some(ClashDialog { rows });
        }
    }

    /// Whether the clash dialog is open.
    pub fn clash_dialog_open(&self) -> bool {
        self.clash_dialog.is_some()
    }

    /// Sets what row `i` of the open clash dialog will do (for tests and the widgets).
    pub fn set_clash_pick(&mut self, i: usize, pick: Pick, new_name: Option<&str>) {
        if let Some(r) = self.clash_dialog.as_mut().and_then(|d| d.rows.get_mut(i)) {
            r.pick = pick;
            if let Some(n) = new_name {
                r.new_name = n.to_string();
            }
        }
    }

    /// Applies the clash dialog's choices and closes it.
    pub fn answer_clash_dialog(&mut self) {
        let Some(d) = self.clash_dialog.take() else {
            return;
        };
        let choices: Vec<(String, ProfileChoice)> = d
            .rows
            .into_iter()
            .map(|r| {
                let c = match r.pick {
                    Pick::Merge => ProfileChoice::Merge,
                    Pick::Rename => ProfileChoice::Rename(r.new_name),
                    Pick::NotNow => ProfileChoice::NotNow,
                };
                (r.id, c)
            })
            .collect();
        self.viewer.resolve_profile_clashes(&choices);
    }

    /// The yellow bar above the canvas: the profiles the file carries that this computer lacks, with
    /// Add to my profiles, and those the user put off deciding about, with Merge or rename…
    pub(crate) fn profile_bar(&mut self, ui: &mut egui::Ui) {
        let Some((missing, deferred)) = self.viewer.profile_bar() else {
            return;
        };
        let list = |names: &[String]| Arg::List {
            items: names
                .iter()
                .map(|n| Message::new("profiles-bar-name").text("name", n.clone()))
                .collect(),
            sep: "profiles-bar-sep",
        };
        let fill = if ui.visuals().dark_mode {
            Color32::from_rgb(92, 76, 12)
        } else {
            Color32::from_rgb(255, 238, 170)
        };
        let ink = if ui.visuals().dark_mode {
            Color32::from_rgb(255, 240, 190)
        } else {
            Color32::from_rgb(60, 48, 0)
        };
        let (mut add, mut decide, mut close) = (false, false, false);
        egui::Frame::new()
            .fill(fill)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if !missing.is_empty() {
                        let text = self.strings.format(
                            &Message::new("profiles-bar-missing")
                                .num("count", missing.len() as f64, 0)
                                .arg("names", list(&missing)),
                        );
                        ui.label(RichText::new(text).color(ink));
                        add = ui.button(self.t("profiles-bar-add")).clicked();
                    }
                    if !deferred.is_empty() {
                        let text = self.strings.format(
                            &Message::new("profiles-bar-deferred")
                                .num("count", deferred.len() as f64, 0)
                                .arg("names", list(&deferred)),
                        );
                        ui.label(RichText::new(text).color(ink));
                        decide = ui.button(self.t("profiles-bar-decide")).clicked();
                    }
                    close = ui
                        .button("×")
                        .on_hover_text(self.t("profiles-bar-close-tip"))
                        .clicked();
                });
            });
        if add {
            let _ = self.viewer.add_missing_profiles();
        }
        if decide {
            self.viewer.ask_deferred_clashes();
            self.open_clash_dialog();
        }
        if close {
            self.viewer.close_profile_bar();
        }
    }

    /// The clash dialog: one row for each carried profile that clashes, with Merge, Rename and Not now.
    pub(crate) fn clash_dialog_window(&mut self, ctx: &egui::Context) {
        // The file just opened with clashes: ask at once (Defaulted "When Merge/Rename is asked").
        if self.clash_dialog.is_none()
            && self
                .viewer
                .profile_use()
                .is_some_and(|u| !u.clashes.is_empty())
        {
            self.open_clash_dialog();
        }
        let Some(mut dialog) = self.clash_dialog.take() else {
            return;
        };
        let mut apply = false;
        let mut cancel = false;
        egui::Window::new(self.t("profiles-clash-title"))
            .collapsible(false)
            .resizable(false)
            .default_width(480.0)
            .show(ctx, |ui| {
                ui.label(self.t("profiles-clash-intro"));
                ui.add_space(6.0);
                for (i, row) in dialog.rows.iter_mut().enumerate() {
                    ui.separator();
                    let which = if row.same_id {
                        "profiles-clash-row-id"
                    } else {
                        "profiles-clash-row-name"
                    };
                    ui.label(
                        RichText::new(
                            self.strings.format(
                                &Message::new(which)
                                    .text("name", row.name.clone())
                                    .text("mine", row.mine.clone()),
                            ),
                        )
                        .strong(),
                    );
                    let merge = self.strings.format(
                        &Message::new("profiles-clash-merge").text("mine", row.mine.clone()),
                    );
                    ui.radio_value(&mut row.pick, Pick::Merge, merge);
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut row.pick,
                            Pick::Rename,
                            self.t("profiles-clash-rename"),
                        );
                        let name = ui.add_enabled(
                            row.pick == Pick::Rename,
                            egui::TextEdit::singleline(&mut row.new_name)
                                .id_salt(("clash-name", i))
                                .char_limit(quadrule_profiles::limits::MAX_NAME_CHARS),
                        );
                        let label = self.t("profiles-clash-rename-field");
                        name.widget_info(|| {
                            egui::WidgetInfo::text_edit(true, &label, "", &row.new_name)
                        });
                    });
                    ui.radio_value(&mut row.pick, Pick::NotNow, self.t("profiles-clash-notnow"));
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    apply = ui.button(self.t("profiles-clash-apply")).clicked();
                    cancel = ui.button(self.t("profiles-clash-later")).clicked();
                });
            });
        if cancel {
            for r in &mut dialog.rows {
                r.pick = Pick::NotNow;
            }
            apply = true;
        }
        self.clash_dialog = Some(dialog);
        if apply {
            self.answer_clash_dialog();
        }
    }
}
