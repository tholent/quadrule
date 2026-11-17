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

//! The Profiles window's widgets (rung 13a bbq, Q5): the list on the left (Built in, Mine), the form on
//! the right (Screen, then the collapsible Line roles, Fill roles, Gray levels and pen grays, Safe zones
//! and About), and the buttons. What the buttons do is in [`crate::profiles_win`] and tested there; the
//! widgets edit a copy of the profile and hand the whole change back, so one frame is one draft.

use crate::profiles_win::{Confirm, ImportPick, NewForm};
use crate::view::{Request, ViewerUi};
use egui::{RichText, Role, WidgetInfo};
use quadrule_i18n::Message;
use quadrule_model::Orientation;
use quadrule_profiles::{Profile, SafeZone, limits};

/// Parses "0, 17, 34" into grays; `None` when anything is not 0 to 255 or there are too many.
fn parse_grays(text: &str) -> Option<Vec<u8>> {
    let out: Option<Vec<u8>> = text
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u8>().ok())
        .collect();
    out.filter(|v| v.len() <= limits::MAX_LIST)
}

/// A paper size's side in millimetres (rung 13b), held to the paper limits.
fn mm_field(v: &mut f64) -> egui::DragValue<'_> {
    egui::DragValue::new(v)
        .range(limits::PAPER_MM)
        .max_decimals(2)
        .speed(0.5)
}

fn join_grays(v: &[u8]) -> String {
    v.iter().map(u8::to_string).collect::<Vec<_>>().join(", ")
}

impl ViewerUi {
    /// The Profiles window, if it is open.
    pub(crate) fn profiles_window_ui(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        if self.profiles_win.is_none() {
            return;
        }
        let list = self.profiles_list();
        let selected = self
            .profiles_win
            .as_ref()
            .map(|w| w.selected.clone())
            .unwrap_or_default();
        let builtin = self.profiles_selected_is_builtin();
        let current = self.profiles_current();
        let dirty = self.profiles_dirty();
        let can_reset = self.profiles_can_reset();
        let damaged = self.profiles_damaged();
        let message = self
            .profiles_win
            .as_ref()
            .and_then(|w| w.message.clone())
            .map(|m| self.strings.format(&m));
        let mut pick: Option<String> = None;
        let mut delete_damaged: Option<String> = None;
        let mut edited = current.clone();
        let (mut new, mut dup, mut import, mut export) = (false, false, false, false);
        let (mut reset, mut delete, mut cancel, mut save) = (false, false, false, false);
        let mut open = true;
        // The lists and the form scroll inside what the screen allows, so the buttons stay in view.
        let body_h = (ctx.content_rect().height() - 220.0).clamp(150.0, 440.0);
        egui::Window::new(self.t("profiles-title"))
            .open(&mut open)
            .collapsible(false)
            .default_size([780.0, body_h + 90.0])
            .show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(210.0, body_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("profiles-list")
                                .max_height(body_h)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(self.t("profiles-group-builtin"))
                                            .small()
                                            .strong(),
                                    );
                                    for e in list.iter().filter(|e| e.built_in && !e.paper) {
                                        self.list_row(ui, e, &selected, &mut pick);
                                    }
                                    // The paper sizes sit under Built in, as their own group (rung 13b).
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(self.t("profiles-group-paper"))
                                            .small()
                                            .weak(),
                                    );
                                    for e in list.iter().filter(|e| e.built_in && e.paper) {
                                        self.list_row(ui, e, &selected, &mut pick);
                                    }
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new(self.t("profiles-group-mine"))
                                            .small()
                                            .strong(),
                                    );
                                    if list.iter().all(|e| e.built_in) {
                                        ui.label(
                                            RichText::new(self.t("profiles-none-mine"))
                                                .small()
                                                .weak(),
                                        );
                                    }
                                    for e in list.iter().filter(|e| !e.built_in) {
                                        self.list_row(ui, e, &selected, &mut pick);
                                    }
                                    if !damaged.is_empty() {
                                        ui.add_space(6.0);
                                        ui.label(
                                            RichText::new(self.t("profiles-group-damaged"))
                                                .small()
                                                .strong(),
                                        );
                                        for d in &damaged {
                                            let text = self.strings.format(
                                                &Message::new(if d.line.is_some() {
                                                    "profiles-damaged-line"
                                                } else {
                                                    "profiles-damaged"
                                                })
                                                .text("file", d.file.clone())
                                                .text("error", d.error.clone())
                                                .num("line", d.line.unwrap_or(0) as f64, 0),
                                            );
                                            ui.label(
                                                RichText::new(text)
                                                    .small()
                                                    .color(ui.visuals().warn_fg_color),
                                            );
                                            let tip = self.strings.format(
                                                &Message::new("profiles-damaged-delete-tip")
                                                    .text("file", d.file.clone()),
                                            );
                                            let del = ui
                                                .small_button(self.t("profiles-damaged-delete"))
                                                .on_hover_text(tip.clone());
                                            del.widget_info(|| {
                                                WidgetInfo::labeled(Role::Button, true, &tip)
                                            });
                                            if del.clicked() {
                                                delete_damaged = Some(d.file.clone());
                                            }
                                        }
                                    }
                                });
                        },
                    );
                    // A rule between the list and the form, as tall as they are (a plain separator would
                    // take the whole height the window may have).
                    let (rule, _) =
                        ui.allocate_exact_size(egui::vec2(1.0, body_h), egui::Sense::hover());
                    ui.painter().vline(
                        rule.center().x,
                        rule.y_range(),
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                    ui.vertical(|ui| {
                        if let Some(p) = edited.as_mut() {
                            egui::ScrollArea::vertical()
                                .id_salt("profiles-form")
                                .max_height(body_h)
                                .auto_shrink([false, false])
                                .show(ui, |ui| self.profile_form(ui, p, builtin));
                        }
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    new = ui.button(self.t("profiles-new")).clicked();
                    dup = ui.button(self.t("profiles-duplicate")).clicked();
                    import = ui.button(self.t("profiles-import")).clicked();
                    export = ui.button(self.t("profiles-export")).clicked();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        save = ui
                            .add_enabled(dirty, egui::Button::new(self.t("profiles-save")))
                            .clicked();
                        cancel = ui.button(self.t("profiles-cancel")).clicked();
                        if !builtin {
                            delete = ui.button(self.t("profiles-delete")).clicked();
                        }
                        if builtin {
                            reset = ui
                                .add_enabled(can_reset, egui::Button::new(self.t("profiles-reset")))
                                .clicked();
                        }
                    });
                });
                if let Some(m) = &message {
                    ui.label(RichText::new(m).small());
                }
            });
        // What the form changed, as one draft.
        if let (Some(new_p), Some(old)) = (edited, current)
            && new_p != old
        {
            self.profiles_edit(|d| *d = new_p);
        }
        if let Some(id) = pick {
            self.profiles_select(&id);
        }
        if let Some(file) = delete_damaged {
            self.profiles_ask(Confirm::DeleteDamaged(file));
        }
        if new {
            let from = if selected.is_empty() {
                quadrule_model::sample::NOMAD_ID.to_string()
            } else {
                selected.clone()
            };
            let name = self
                .strings
                .format(&Message::new("profiles-new-default-name"));
            if let Some(w) = &mut self.profiles_win {
                w.new_form = Some(NewForm { name, from });
            }
        }
        if dup {
            self.profiles_duplicate();
        }
        if import {
            requests.push(Request::ProfileImportDialog);
        }
        if export && let Some(name) = self.profiles_export_name() {
            requests.push(Request::ProfileExportDialog(name));
        }
        if reset {
            self.profiles_ask(Confirm::Reset(selected.clone()));
        }
        if delete {
            self.profiles_ask(Confirm::Delete(selected));
        }
        if save {
            self.profiles_save();
        }
        self.profiles_modals(ctx);
        if cancel || !open {
            self.profiles_cancel();
        }
    }

    fn list_row(
        &self,
        ui: &mut egui::Ui,
        e: &crate::profiles_win::ListEntry,
        selected: &str,
        pick: &mut Option<String>,
    ) {
        let mut name = e.name.clone();
        if e.edited {
            name = self
                .strings
                .format(&Message::new("profiles-name-edited").text("name", name));
        }
        if e.unverified {
            name = self
                .strings
                .format(&Message::new("profiles-name-unverified").text("name", name));
        }
        let row = ui.selectable_label(e.id == selected, name);
        let row = match &e.detail {
            Some(d) => row.on_hover_text(d),
            None => row,
        };
        if row.clicked() {
            *pick = Some(e.id.clone());
        }
    }

    /// The questions on top of the window: Reset and Delete, New…, and an import that clashes.
    fn profiles_modals(&mut self, ctx: &egui::Context) {
        let Some(w) = self.profiles_win.as_ref() else {
            return;
        };
        let confirm = w.confirm.clone();
        let new_form = w.new_form.clone();
        let import = w.import.clone();
        if let Some(c) = confirm {
            let name = match &c {
                Confirm::Reset(id) | Confirm::Delete(id) => {
                    quadrule_profiles::active::display_name(id).unwrap_or_default()
                }
                Confirm::DeleteDamaged(file) => file.clone(),
            };
            let (title, body, go) = match &c {
                Confirm::Reset(_) => (
                    "profiles-reset-title",
                    "profiles-reset-body",
                    "profiles-reset-yes",
                ),
                Confirm::Delete(_) => (
                    "profiles-delete-title",
                    "profiles-delete-body",
                    "profiles-delete-yes",
                ),
                Confirm::DeleteDamaged(_) => (
                    "profiles-damaged-title",
                    "profiles-damaged-body",
                    "profiles-delete-yes",
                ),
            };
            let mut answer = None;
            egui::Modal::new(egui::Id::unique("profiles-confirm")).show(ctx, |ui| {
                ui.set_width(340.0);
                ui.heading(self.t(title));
                ui.label(
                    self.strings
                        .format(&Message::new(body).text("name", name.clone())),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t(go)).clicked() {
                        answer = Some(true);
                    }
                    if ui.button(self.t("profiles-keep")).clicked() {
                        answer = Some(false);
                    }
                });
            });
            if let Some(yes) = answer {
                self.profiles_confirm(yes);
            }
        }
        if let Some(mut form) = new_form {
            let mut create = false;
            let mut cancel = false;
            let mut ids = quadrule_profiles::active::device_ids();
            ids.extend(quadrule_profiles::active::paper_ids());
            egui::Modal::new(egui::Id::unique("profiles-new")).show(ctx, |ui| {
                ui.set_width(340.0);
                ui.heading(self.t("profiles-new-title"));
                egui::Grid::new("profiles-new-grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        let label = ui.label(self.t("profiles-name"));
                        ui.add(
                            egui::TextEdit::singleline(&mut form.name)
                                .char_limit(limits::MAX_NAME_CHARS),
                        )
                        .labelled_by(label.id);
                        ui.end_row();
                        let label = ui.label(self.t("profiles-start-from"));
                        egui::ComboBox::from_id_salt("profiles-new-from")
                            .selected_text(self.full_name(&form.from))
                            .show_ui(ui, |ui| {
                                for id in &ids {
                                    ui.selectable_value(
                                        &mut form.from,
                                        id.clone(),
                                        self.full_name(id),
                                    );
                                }
                            })
                            .response
                            .labelled_by(label.id);
                        ui.end_row();
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    create = ui
                        .add_enabled(
                            !form.name.trim().is_empty(),
                            egui::Button::new(self.t("profiles-create")),
                        )
                        .clicked();
                    cancel = ui.button(self.t("profiles-cancel")).clicked();
                });
            });
            if create {
                self.profiles_new(&form.name, &form.from);
            } else if cancel {
                if let Some(w) = &mut self.profiles_win {
                    w.new_form = None;
                }
            } else if let Some(w) = &mut self.profiles_win {
                w.new_form = Some(form);
            }
        }
        if let Some(ask) = import {
            let mut pick: Option<ImportPick> = None;
            let mut cancel = false;
            let mut new_name = ask.new_name.clone();
            let same_id = ask.kind == quadrule_profiles::Incoming::IdClash;
            egui::Modal::new(egui::Id::unique("profiles-import")).show(ctx, |ui| {
                ui.set_width(380.0);
                ui.heading(self.t("profiles-import-title"));
                let body = if same_id {
                    "profiles-import-row-id"
                } else {
                    "profiles-import-row-name"
                };
                ui.label(
                    self.strings.format(
                        &Message::new(body)
                            .text("name", ask.incoming.display_name.clone())
                            .text("mine", ask.mine.clone()),
                    ),
                );
                ui.add_space(8.0);
                if same_id {
                    ui.horizontal(|ui| {
                        if ui.button(self.t("profiles-import-replace")).clicked() {
                            pick = Some(ImportPick::Replace);
                        }
                        if ui.button(self.t("profiles-import-both")).clicked() {
                            pick = Some(ImportPick::KeepBoth);
                        }
                        cancel = ui.button(self.t("profiles-cancel")).clicked();
                    });
                } else {
                    if ui
                        .button(self.strings.format(
                            &Message::new("profiles-clash-merge").text("mine", ask.mine.clone()),
                        ))
                        .clicked()
                    {
                        pick = Some(ImportPick::Merge);
                    }
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut new_name)
                                .id_salt("profiles-import-name")
                                .char_limit(limits::MAX_NAME_CHARS),
                        );
                        if ui.button(self.t("profiles-import-rename")).clicked() {
                            pick = Some(ImportPick::Rename(new_name.clone()));
                        }
                    });
                    cancel = ui.button(self.t("profiles-cancel")).clicked();
                }
            });
            if pick.is_some() {
                self.profiles_import_answer(pick);
            } else if cancel {
                self.profiles_import_answer(None);
            } else if let Some(w) = &mut self.profiles_win
                && let Some(a) = &mut w.import
            {
                a.new_name = new_name;
            }
        }
    }

    /// The form for one profile: Screen, then the collapsible sections.
    fn profile_form(&mut self, ui: &mut egui::Ui, p: &mut Profile, builtin: bool) {
        let title = self.marked(&p.id, p.display_name.clone());
        ui.heading(title);
        ui.add_space(4.0);
        ui.strong(self.t("profiles-screen"));
        egui::Grid::new("profiles-screen")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                let label = ui.label(self.t("profiles-name"));
                ui.add(
                    egui::TextEdit::singleline(&mut p.display_name)
                        .char_limit(limits::MAX_NAME_CHARS),
                )
                .labelled_by(label.id);
                ui.end_row();

                let label = ui.label(self.t("profiles-size"));
                if let Some(mut size) = p.paper {
                    // A paper size is a physical size; its pixels follow it at 300 PPI (rung 13b).
                    let (pw, ph) = size.px_at(quadrule_model::DEFAULT_PAPER_PPI);
                    let mm = |v: f64| quadrule_layout::units::trim_number(v, 2);
                    if builtin {
                        let text = self.strings.format(
                            &Message::new("profiles-size-paper-locked")
                                .text("w", mm(size.width_mm))
                                .text("h", mm(size.height_mm))
                                .num("pw", f64::from(pw), 0)
                                .num("ph", f64::from(ph), 0),
                        );
                        ui.label(text).labelled_by(label.id);
                    } else {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let tip = self.t("profiles-size-mm");
                                ui.add(mm_field(&mut size.width_mm))
                                    .on_hover_text(tip.clone())
                                    .labelled_by(label.id);
                                ui.label("×");
                                ui.add(mm_field(&mut size.height_mm))
                                    .on_hover_text(tip)
                                    .labelled_by(label.id);
                                ui.label("mm");
                            });
                            ui.label(
                                RichText::new(
                                    self.strings.format(
                                        &Message::new("profiles-size-paper-px")
                                            .num("pw", f64::from(pw), 0)
                                            .num("ph", f64::from(ph), 0),
                                    ),
                                )
                                .small()
                                .weak(),
                            );
                        });
                        if Some(size) != p.paper {
                            p.paper = Some(size);
                            p.sync_paper();
                        }
                    }
                } else if builtin {
                    let text = self.strings.format(
                        &Message::new("profiles-size-locked")
                            .num("w", p.width_px as f64, 0)
                            .num("h", p.height_px as f64, 0),
                    );
                    ui.label(text).labelled_by(label.id);
                } else {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut p.width_px)
                                .range(limits::MIN_PX..=limits::MAX_PX),
                        )
                        .labelled_by(label.id);
                        ui.label("×");
                        ui.add(
                            egui::DragValue::new(&mut p.height_px)
                                .range(limits::MIN_PX..=limits::MAX_PX),
                        )
                        .labelled_by(label.id);
                        ui.label("px");
                    });
                }
                ui.end_row();

                let label = ui.label(self.t("profiles-ppi"));
                if p.is_paper() {
                    ui.label(self.t("profiles-ppi-per-page"))
                        .labelled_by(label.id);
                } else if builtin {
                    ui.label(
                        self.strings
                            .format(&Message::new("profiles-ppi-locked").num("ppi", p.ppi, 0)),
                    )
                    .labelled_by(label.id);
                } else {
                    ui.add(
                        egui::DragValue::new(&mut p.ppi)
                            .range(limits::MIN_PPI..=limits::MAX_PPI)
                            .max_decimals(1),
                    )
                    .labelled_by(label.id);
                }
                ui.end_row();

                let label = ui.label(self.t("profiles-short-name"));
                if builtin {
                    ui.label(RichText::new(p.short_name.clone()).monospace())
                        .labelled_by(label.id);
                } else {
                    ui.add(
                        egui::TextEdit::singleline(&mut p.short_name)
                            .char_limit(limits::MAX_SHORT_CHARS),
                    )
                    .on_hover_text(self.t("profiles-short-name-tip"))
                    .labelled_by(label.id);
                }
                ui.end_row();

                let label = ui.label(self.t("profiles-margins"));
                ui.horizontal(|ui| {
                    let m = &mut p.default_margins_mm;
                    for (v, tip) in [
                        (&mut m.top, "profiles-margin-top"),
                        (&mut m.right, "profiles-margin-right"),
                        (&mut m.bottom, "profiles-margin-bottom"),
                        (&mut m.left, "profiles-margin-left"),
                    ] {
                        ui.add(egui::DragValue::new(v).range(0.0..=100.0).max_decimals(1))
                            .on_hover_text(self.t(tip))
                            .labelled_by(label.id);
                    }
                    ui.label("mm");
                });
                ui.end_row();

                let label = ui.label(self.t("profiles-orientations"));
                ui.horizontal(|ui| {
                    for (o, id) in [
                        (Orientation::Portrait, "app-orientation-portrait"),
                        (Orientation::Landscape, "app-orientation-landscape"),
                    ] {
                        let mut on = p.orientations.contains(&o);
                        let was = on;
                        let only = p.orientations.len() == 1 && on;
                        if ui
                            .add_enabled(!only, egui::Checkbox::new(&mut on, self.t(id)))
                            .changed()
                            && on != was
                        {
                            if on {
                                p.orientations.push(o);
                            } else {
                                p.orientations.retain(|x| *x != o);
                            }
                        }
                    }
                })
                .response
                .labelled_by(label.id);
                ui.end_row();
            });
        ui.add_space(6.0);

        let paper = p.is_paper();
        egui::CollapsingHeader::new(self.t("profiles-line-roles"))
            .id_salt("profiles-line-roles")
            .show(ui, |ui| {
                if paper {
                    ui.label(
                        RichText::new(self.t("profiles-paper-roles-note"))
                            .small()
                            .weak(),
                    );
                } else {
                    ui.label(RichText::new(self.t("profiles-calibrated")).small().weak());
                }
                egui::Grid::new("profiles-lines")
                    .num_columns(3)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("");
                        ui.label(RichText::new(self.t("profiles-width-px")).small());
                        ui.label(RichText::new(self.t("profiles-gray")).small());
                        ui.end_row();
                        let r = &mut p.line_roles;
                        for (st, id) in [
                            (&mut r.whisper, "app-role-whisper"),
                            (&mut r.guide, "app-role-guide"),
                            (&mut r.structure, "app-role-structure"),
                            (&mut r.emphasis, "app-role-emphasis"),
                            (&mut r.dot_grid, "app-role-dotgrid"),
                        ] {
                            let label = ui.label(self.t(id));
                            ui.add(egui::DragValue::new(&mut st.width_px).range(limits::ROLE_PX))
                                .on_hover_text(self.t("profiles-width-px"))
                                .labelled_by(label.id);
                            ui.add(egui::DragValue::new(&mut st.gray).range(0..=255))
                                .on_hover_text(self.t("profiles-gray"))
                                .labelled_by(label.id);
                            ui.end_row();
                        }
                        for (g, id) in [
                            (&mut r.text_gray, "app-role-text"),
                            (&mut r.label_gray, "app-role-label"),
                        ] {
                            let label = ui.label(self.t(id));
                            ui.label("");
                            ui.add(egui::DragValue::new(g).range(0..=255))
                                .on_hover_text(self.t("profiles-gray"))
                                .labelled_by(label.id);
                            ui.end_row();
                        }
                    });
            });
        egui::CollapsingHeader::new(self.t("profiles-fill-roles"))
            .id_salt("profiles-fill-roles")
            .show(ui, |ui| {
                egui::Grid::new("profiles-fills")
                    .num_columns(2)
                    .show(ui, |ui| {
                        for (g, id) in [
                            (&mut p.fill_roles.shade, "app-fill-shade"),
                            (&mut p.fill_roles.block, "app-fill-block"),
                        ] {
                            let label = ui.label(self.t(id));
                            ui.add(egui::DragValue::new(g).range(0..=255))
                                .on_hover_text(self.t("profiles-gray"))
                                .labelled_by(label.id);
                            ui.end_row();
                        }
                    });
            });
        egui::CollapsingHeader::new(self.t("profiles-lint"))
            .id_salt("profiles-lint")
            .show(ui, |ui| self.lint_section(ui, p));
        // A paper size has no gray levels, pen grays or safe zones (napkin §3.3.1).
        if !paper {
            egui::CollapsingHeader::new(self.t("profiles-grays"))
                .id_salt("profiles-grays")
                .show(ui, |ui| {
                    self.gray_list(
                        ui,
                        "profiles-gray-levels",
                        "profiles-gray-levels-label",
                        &mut p.gray_levels,
                    );
                    self.gray_list(
                        ui,
                        "profiles-pen-grays",
                        "profiles-pen-grays-label",
                        &mut p.ink_values,
                    );
                });
        }
        let zones = self
            .strings
            .format(&Message::new("profiles-safe-zones").num(
                "count",
                p.safe_zones.len() as f64,
                0,
            ));
        if !paper {
            egui::CollapsingHeader::new(zones)
                .id_salt("profiles-safe-zones")
                .show(ui, |ui| self.zone_table(ui, p));
        }
        egui::CollapsingHeader::new(self.t("profiles-about"))
            .id_salt("profiles-about")
            .show(ui, |ui| {
                ui.checkbox(
                    &mut p.calibration_meta.verified,
                    self.t("profiles-verified"),
                );
                let mut date = p.calibration_meta.date.clone().unwrap_or_default();
                ui.horizontal(|ui| {
                    let label = ui.label(self.t("profiles-date"));
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut date)
                                .char_limit(40)
                                .desired_width(120.0),
                        )
                        .labelled_by(label.id)
                        .changed()
                    {
                        p.calibration_meta.date = if date.trim().is_empty() {
                            None
                        } else {
                            Some(date.clone())
                        };
                    }
                });
                let label = ui.label(self.t("profiles-notes"));
                ui.add(
                    egui::TextEdit::multiline(&mut p.calibration_meta.method)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                        .char_limit(limits::MAX_TEXT_CHARS),
                )
                .labelled_by(label.id);
                ui.label(
                    RichText::new(
                        self.strings
                            .format(&Message::new("profiles-id").text("id", p.id.clone())),
                    )
                    .small()
                    .weak()
                    .monospace(),
                );
            });
    }

    /// The lint numbers (rung 14a bbq, Q2): each can be left empty, which skips the check that reads it.
    fn lint_section(&self, ui: &mut egui::Ui, p: &mut Profile) {
        ui.label(RichText::new(self.t("profiles-lint-note")).small().weak());
        egui::Grid::new("profiles-lint-grid")
            .num_columns(3)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                let l = &mut p.lint;
                for (v, id) in [
                    (&mut l.lightest_gray_line_1px, "profiles-lint-line-1px"),
                    (&mut l.lightest_gray_line_2px_up, "profiles-lint-line-2px"),
                    (&mut l.lightest_gray_dot_2px, "profiles-lint-dot-2px"),
                    (&mut l.lightest_gray_dot_3px, "profiles-lint-dot-3px"),
                    (&mut l.lightest_gray_dot_4px_up, "profiles-lint-dot-4px"),
                    (&mut l.lightest_gray_fill, "profiles-lint-fill"),
                    (&mut l.lightest_gray_text, "profiles-lint-text-gray"),
                    (&mut l.dark_fill_gray, "profiles-lint-dark-gray"),
                ] {
                    self.lint_row(ui, id, v, 128, 0..=255, 0, 1.0);
                }
                self.lint_row(
                    ui,
                    "profiles-lint-text-pt",
                    &mut l.smallest_text_pt,
                    6.0,
                    1.0..=1_000.0,
                    1,
                    0.1,
                );
                self.lint_row(
                    ui,
                    "profiles-lint-step-mm",
                    &mut l.smallest_handwriting_step_mm,
                    5.0,
                    0.5..=1_000.0,
                    1,
                    0.1,
                );
                // The share is held as 0 to 1 and shown as a percentage.
                let mut pct = l
                    .dark_fill_page_share
                    .map(|s| (s * 100.0 * 1e6).round() / 1e6);
                let before = pct;
                self.lint_row(
                    ui,
                    "profiles-lint-dark-share",
                    &mut pct,
                    10.0,
                    0.0..=100.0,
                    1,
                    0.5,
                );
                if pct != before {
                    l.dark_fill_page_share = pct.map(|v| v / 100.0);
                }
                self.lint_row(
                    ui,
                    "profiles-lint-thin-px",
                    &mut l.thin_line_px,
                    2,
                    0..=*limits::ROLE_PX.end(),
                    0,
                    1.0,
                );
            });
    }

    /// One lint number: a field with a Clear button, or "skipped" with a Set button when it is empty.
    #[allow(clippy::too_many_arguments)]
    fn lint_row<T: egui::emath::Numeric>(
        &self,
        ui: &mut egui::Ui,
        label_id: &str,
        value: &mut Option<T>,
        default: T,
        range: std::ops::RangeInclusive<T>,
        decimals: usize,
        speed: f64,
    ) {
        let label = ui.label(self.t(label_id));
        match value {
            Some(v) => {
                ui.add(
                    egui::DragValue::new(v)
                        .range(range)
                        .max_decimals(decimals)
                        .speed(speed),
                )
                .labelled_by(label.id);
                let name = self.t(label_id);
                let clear = ui
                    .button(self.t("profiles-lint-clear"))
                    .on_hover_text(self.t("profiles-lint-clear-tip"));
                clear.widget_info(|| {
                    WidgetInfo::labeled(
                        Role::Button,
                        true,
                        format!("{} {name}", self.t("profiles-lint-clear")),
                    )
                });
                if clear.clicked() {
                    *value = None;
                }
            }
            None => {
                ui.label(RichText::new(self.t("profiles-lint-skipped")).weak())
                    .labelled_by(label.id);
                let name = self.t(label_id);
                let set = ui
                    .button(self.t("profiles-lint-set"))
                    .on_hover_text(self.t("profiles-lint-set-tip"));
                set.widget_info(|| {
                    WidgetInfo::labeled(
                        Role::Button,
                        true,
                        format!("{} {name}", self.t("profiles-lint-set")),
                    )
                });
                if set.clicked() {
                    *value = Some(default);
                }
            }
        }
        ui.end_row();
    }

    /// A list of grays typed as "0, 17, 34", kept in the form until it parses.
    fn gray_list(&mut self, ui: &mut egui::Ui, key: &str, label_id: &str, values: &mut Vec<u8>) {
        let label = ui.label(self.t(label_id));
        let current = join_grays(values);
        let mut text = self
            .profiles_win
            .as_ref()
            .and_then(|w| w.texts.get(key).cloned())
            .unwrap_or_else(|| current.clone());
        let resp = ui
            .add(egui::TextEdit::singleline(&mut text).desired_width(f32::INFINITY))
            .labelled_by(label.id);
        if resp.changed() {
            if let Some(v) = parse_grays(&text) {
                *values = v;
            }
            if let Some(w) = &mut self.profiles_win {
                w.texts.insert(key.to_string(), text.clone());
            }
        }
        if parse_grays(&text).is_none() {
            ui.label(
                RichText::new(self.t("profiles-grays-bad"))
                    .small()
                    .color(ui.visuals().warn_fg_color),
            );
        } else if !resp.has_focus()
            && let Some(w) = &mut self.profiles_win
            && parse_grays(&text).as_deref() == Some(values.as_slice())
        {
            // Once it is read, the list shows as the profile holds it.
            w.texts.remove(key);
        }
    }

    /// Safe zones as a table of pixel boxes (Q5).
    fn zone_table(&mut self, ui: &mut egui::Ui, p: &mut Profile) {
        let (w, h) = (p.width_px, p.height_px);
        let mut remove: Option<usize> = None;
        egui::Grid::new("profiles-zones")
            .num_columns(8)
            .spacing([6.0, 4.0])
            .show(ui, |ui| {
                for head in [
                    "profiles-zone-name",
                    "profiles-zone-orientation",
                    "x",
                    "y",
                    "profiles-zone-w",
                    "profiles-zone-h",
                    "profiles-zone-movable",
                    "",
                ] {
                    let text = if head.starts_with("profiles") {
                        self.t(head)
                    } else {
                        head.to_string()
                    };
                    ui.label(RichText::new(text).small());
                }
                ui.end_row();
                for (i, z) in p.safe_zones.iter_mut().enumerate() {
                    ui.add(
                        egui::TextEdit::singleline(&mut z.name)
                            .id_salt(("zone-name", i))
                            .desired_width(130.0)
                            .char_limit(limits::MAX_NAME_CHARS),
                    )
                    .on_hover_text(self.t("profiles-zone-name"));
                    egui::ComboBox::from_id_salt(("zone-orientation", i))
                        .selected_text(self.t(match z.orientation {
                            Orientation::Portrait => "app-orientation-portrait",
                            Orientation::Landscape => "app-orientation-landscape",
                        }))
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut z.orientation,
                                Orientation::Portrait,
                                self.t("app-orientation-portrait"),
                            );
                            ui.selectable_value(
                                &mut z.orientation,
                                Orientation::Landscape,
                                self.t("app-orientation-landscape"),
                            );
                        });
                    let (x, y, zw, zh) = &mut z.rect;
                    ui.add(egui::DragValue::new(x).range(0..=w.saturating_sub(1)))
                        .on_hover_text("x");
                    ui.add(egui::DragValue::new(y).range(0..=h.saturating_sub(1)))
                        .on_hover_text("y");
                    let (xv, yv) = (*x, *y);
                    ui.add(egui::DragValue::new(zw).range(1..=w.saturating_sub(xv).max(1)))
                        .on_hover_text(self.t("profiles-zone-w"));
                    ui.add(egui::DragValue::new(zh).range(1..=h.saturating_sub(yv).max(1)))
                        .on_hover_text(self.t("profiles-zone-h"));
                    ui.checkbox(&mut z.advisory, "")
                        .on_hover_text(self.t("profiles-zone-movable"));
                    let del = ui.button("×");
                    let name = self
                        .strings
                        .format(&Message::new("profiles-zone-remove").text("name", z.name.clone()));
                    del.widget_info(|| WidgetInfo::labeled(Role::Button, true, &name));
                    if del.on_hover_text(name.clone()).clicked() {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
            });
        if let Some(i) = remove {
            p.safe_zones.remove(i);
        }
        if p.safe_zones.len() < limits::MAX_ZONES
            && ui.button(self.t("profiles-zone-add")).clicked()
        {
            p.safe_zones.push(SafeZone {
                name: self.t("profiles-zone-new"),
                orientation: Orientation::Portrait,
                rect: (0, 0, (w / 10).max(1), (h / 10).max(1)),
                advisory: false,
            });
        }
    }
}
