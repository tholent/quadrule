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

//! The widgets for missing fonts and installed fonts (rung 12a, docs/bbq/2026-11-16-rung-12-missing-fonts.md):
//! the yellow bar above the canvas (Q5), the Missing fonts dialog it opens, and the sheet shown before
//! Save, Save As and Export when the page uses installed fonts (Q7). What they decide is in
//! [`crate::missing`] and tested there.

use crate::view::{After, Request, ViewerUi};
use egui::{Color32, RichText};
use quadrule_i18n::{Arg, Message};
use quadrule_lint::{EmbedStatus, InstalledFont};

/// What a row of the Missing fonts dialog will do with its font.
#[derive(Clone, Debug, PartialEq)]
pub enum Choice {
    /// Keep drawing it from the saved outlines (the default).
    UseSaved,
    /// Rewrite the font of its boxes to this family.
    Substitute(String),
}

/// One row of the dialog: a missing font and what to do about it.
#[derive(Clone, Debug)]
pub struct Row {
    /// The missing family.
    pub family: String,
    /// How many text boxes use it.
    pub boxes: usize,
    /// What is chosen.
    pub choice: Choice,
    /// The best matches here, best first (at most five).
    pub matches: Vec<String>,
}

/// The Missing fonts dialog while it shows.
#[derive(Clone, Debug)]
pub struct MissingDialog {
    /// One row for each missing font.
    pub rows: Vec<Row>,
    /// "Remember for this document" (on by default).
    pub remember: bool,
}

/// What the sheet asks permission for.
#[derive(Clone, Debug, PartialEq)]
pub enum Sheet {
    /// Ask where to save, or export: the request goes on when Continue is pressed.
    Request(Request),
    /// Save to the document's file.
    Save,
    /// Save, then do this (from the unsaved-changes prompt).
    SaveThen(After),
}

impl ViewerUi {
    /// Opens the Missing fonts dialog on the fonts missing now, each row set to use the saved
    /// outlines, with the best matches worked out.
    pub fn open_missing_dialog(&mut self) {
        let missing: Vec<(String, usize)> = self
            .viewer
            .missing_fonts()
            .iter()
            .map(|m| (m.family.clone(), m.boxes.len()))
            .collect();
        let rows = missing
            .into_iter()
            .map(|(family, boxes)| {
                let matches = self
                    .viewer
                    .font_matches(&family)
                    .into_iter()
                    .map(|m| m.family)
                    .collect();
                Row {
                    family,
                    boxes,
                    choice: Choice::UseSaved,
                    matches,
                }
            })
            .collect();
        self.missing_dialog = Some(MissingDialog {
            rows,
            remember: true,
        });
    }

    /// Whether the Missing fonts dialog is open.
    pub fn missing_dialog_open(&self) -> bool {
        self.missing_dialog.is_some()
    }

    /// Whether the Save sheet is showing.
    pub fn font_sheet_open(&self) -> bool {
        self.font_sheet.is_some()
    }

    /// Does what the sheet was holding back, or drops it.
    pub fn answer_font_sheet(&mut self, continue_on: bool, dont_warn: bool) -> Vec<Request> {
        let Some(sheet) = self.font_sheet.take() else {
            return Vec::new();
        };
        if !continue_on {
            return Vec::new();
        }
        if dont_warn {
            let _ = self.viewer.set_dont_warn(true);
        }
        match sheet {
            Sheet::Request(r) => vec![r],
            Sheet::Save => self.save_now(),
            Sheet::SaveThen(after) => {
                match self.viewer.file_path().map(std::path::Path::to_path_buf) {
                    Some(p) => self.save_as(&p, Some(after)),
                    None => vec![Request::SaveAsDialog(Some(after))],
                }
            }
        }
    }

    /// Pushes `request` (the Save As dialog, or the Export one) unless the sheet has to be answered
    /// first (rung 12a bbq, Q7). Autosave never comes through here.
    pub(crate) fn ask(&mut self, request: Request, requests: &mut Vec<Request>) {
        if self.viewer.font_sheet_fonts().is_empty() {
            requests.push(request);
        } else {
            self.font_sheet = Some(Sheet::Request(request));
        }
    }

    /// The yellow bar above the canvas (Q5): the fonts missing, Choose… and a close ×.
    pub(crate) fn font_bar(&mut self, ui: &mut egui::Ui) {
        let names: Vec<String> = self
            .viewer
            .bar_fonts()
            .iter()
            .map(|m| m.family.clone())
            .collect();
        if names.is_empty() {
            return;
        }
        let items: Vec<Message> = names
            .iter()
            .map(|n| Message::new("app-fonts-bar-name").text("name", n.clone()))
            .collect();
        let text = self.strings.format(
            &Message::new("app-fonts-bar")
                .num("count", names.len() as f64, 0)
                .arg(
                    "fonts",
                    Arg::List {
                        items,
                        sep: "app-fonts-bar-sep",
                    },
                ),
        );
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
        let mut choose = false;
        let mut close = false;
        egui::Frame::new()
            .fill(fill)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(text).color(ink));
                    choose = ui.button(self.t("app-fonts-bar-choose")).clicked();
                    close = ui
                        .button("×")
                        .on_hover_text(self.t("app-fonts-bar-close-tip"))
                        .clicked();
                });
            });
        if choose {
            self.open_missing_dialog();
        }
        if close {
            self.viewer.close_font_bar();
        }
    }

    /// The Missing fonts dialog: one row per missing font with how many boxes use it and a choice,
    /// Locate fonts…, and Remember for this document.
    pub(crate) fn missing_dialog_window(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.missing_dialog.take() else {
            return;
        };
        // A font that turned up after a re-scan drops out of the list at once.
        let still: Vec<String> = self
            .viewer
            .missing_fonts()
            .iter()
            .map(|m| m.family.clone())
            .collect();
        dialog.rows.retain(|r| still.contains(&r.family));
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let mut locate = false;
        let mut other: Option<String> = None;
        egui::Window::new(self.t("app-missing-title"))
            .open(&mut open)
            .collapsible(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                if dialog.rows.is_empty() {
                    ui.label(self.t("app-missing-none"));
                } else {
                    ui.label(self.t("app-missing-intro"));
                    ui.add_space(6.0);
                }
                egui::Grid::new("missing-rows")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        for (i, row) in dialog.rows.iter_mut().enumerate() {
                            let row_label = ui.label(
                                self.strings.format(
                                    &Message::new("app-missing-row")
                                        .text("font", row.family.clone())
                                        .num("count", row.boxes as f64, 0),
                                ),
                            );
                            let shown = match &row.choice {
                                Choice::UseSaved => self.t("app-missing-use-saved"),
                                Choice::Substitute(f) => self.strings.format(
                                    &Message::new("app-missing-substitute").text("font", f.clone()),
                                ),
                            };
                            egui::ComboBox::from_id_salt(("missing-choice", i))
                                .selected_text(shown)
                                .width(240.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut row.choice,
                                        Choice::UseSaved,
                                        self.t("app-missing-use-saved"),
                                    );
                                    for m in &row.matches {
                                        let label = self.strings.format(
                                            &Message::new("app-missing-substitute")
                                                .text("font", m.clone()),
                                        );
                                        ui.selectable_value(
                                            &mut row.choice,
                                            Choice::Substitute(m.clone()),
                                            label,
                                        );
                                    }
                                    if ui.button(self.t("app-missing-other")).clicked() {
                                        other = Some(row.family.clone());
                                    }
                                })
                                // The choice is named by the font it is for.
                                .response
                                .labelled_by(row_label.id);
                            ui.end_row();
                        }
                    });
                ui.add_space(6.0);
                ui.checkbox(&mut dialog.remember, self.t("app-missing-remember"))
                    .on_hover_text(self.t("app-missing-remember-tip"));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    locate = ui
                        .button(self.t("app-missing-locate"))
                        .on_hover_text(self.t("app-missing-locate-tip"))
                        .clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        cancel = ui.button(self.t("app-missing-cancel")).clicked();
                        apply = ui.button(self.t("app-missing-apply")).clicked();
                    });
                });
            });
        if locate {
            let _ = self.viewer.locate_fonts();
        }
        if let Some(family) = other {
            // The picker chooses the substitute for this row; the pick comes back as the row's choice.
            self.font_picker_for = Some(family.clone());
            let fonts = &self.viewer.fonts;
            self.font_picker.open(fonts);
        }
        // A pick from the font picker (Other font…) lands in its row.
        if let Some((family, pick)) = self.picked_for_missing.take()
            && let Some(row) = dialog.rows.iter_mut().find(|r| r.family == family)
        {
            row.choice = Choice::Substitute(pick);
        }
        if apply {
            let subs: Vec<(String, String)> = dialog
                .rows
                .iter()
                .filter_map(|r| match &r.choice {
                    Choice::Substitute(to) => Some((r.family.clone(), to.clone())),
                    Choice::UseSaved => None,
                })
                .collect();
            let keep: Vec<String> = if dialog.remember {
                dialog
                    .rows
                    .iter()
                    .filter(|r| r.choice == Choice::UseSaved)
                    .map(|r| r.family.clone())
                    .collect()
            } else {
                Vec::new()
            };
            let _ = self.viewer.apply_font_choices(&subs, &keep);
            return;
        }
        if open && !cancel {
            self.missing_dialog = Some(dialog);
        }
    }

    /// The sheet shown before Save, Save As and Export when the page uses installed fonts (Q7): each
    /// font with whether the file carries it, Continue, Cancel and "Don't warn again for this
    /// document".
    pub(crate) fn font_sheet_window(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        if self.font_sheet.is_none() {
            return;
        }
        let fonts: Vec<InstalledFont> = self.viewer.font_sheet_fonts();
        let mut answer: Option<bool> = None;
        egui::Modal::new(egui::Id::unique("font-sheet")).show(ctx, |ui| {
            ui.heading(self.t("app-sheet-title"));
            ui.label(self.t("app-sheet-intro"));
            ui.add_space(6.0);
            for f in &fonts {
                let status = match f.embed {
                    EmbedStatus::Embedded => self.t("app-sheet-embedded"),
                    EmbedStatus::Off => self.t("app-sheet-off"),
                    EmbedStatus::Cannot(reason) => self.strings.format(
                        &Message::new("app-sheet-cannot")
                            .arg("reason", Arg::Msg(Box::new(Message::new(reason)))),
                    ),
                };
                let mut line = self.strings.format(
                    &Message::new("app-sheet-item")
                        .text("font", f.family.clone())
                        .text("status", status),
                );
                if f.fallback {
                    line = format!("{line} {}", self.t("app-sheet-fallback"));
                }
                ui.label(line);
            }
            ui.add_space(8.0);
            let tick = self.t("app-sheet-dont-warn");
            ui.checkbox(&mut self.sheet_dont_warn, tick);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-sheet-continue")).clicked() {
                    answer = Some(true);
                }
                if ui.button(self.t("app-sheet-cancel")).clicked() {
                    answer = Some(false);
                }
            });
        });
        if let Some(go) = answer {
            let dont = std::mem::take(&mut self.sheet_dont_warn);
            requests.extend(self.answer_font_sheet(go, dont));
        }
    }
}
