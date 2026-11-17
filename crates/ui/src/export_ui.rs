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

//! The Export sheet and the completion sheet (rung 15 bbq, Q2, Q7 and Q9). The engine is [`crate::export`]; this is
//! only what is drawn and what a press does. One sheet at a time, never stacked: the Export sheet closes when
//! Export is pressed, then the one preflight sheet shows if the rules call for it, then the files are written
//! and the completion sheet shows (not after Export again, which only says it in a one-line notice).
//!
//! Every choice on the Export sheet (devices, path, Set, note) is kept in the settings the moment it changes;
//! the folder is kept when an export writes into it.

use crate::export::{Again, ExportRequest, ExportResult, THUMB_SIDE};
use crate::export_path::PRESETS;
use crate::preflight::{Preflight, Then};
use crate::settings::DevicesChoice;
use crate::view::{Request, ViewerUi};
use crate::viewer::{ExportWhich, Notice};
use quadrule_i18n::{Arg, Message};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The Export sheet while it shows.
#[derive(Clone, Debug)]
pub struct ExportSheet {
    /// The folder the files go into, if one is known yet.
    pub folder: Option<PathBuf>,
}

/// The completion sheet while it shows.
pub struct Completion {
    /// What the export did.
    pub result: ExportResult,
    /// One picture per file, made when the sheet is first drawn.
    textures: Vec<egui::TextureHandle>,
}

/// The program and arguments that show `folder` in the system's file manager.
pub fn reveal_command(folder: &Path) -> (&'static str, Vec<OsString>) {
    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    (program, vec![folder.as_os_str().to_os_string()])
}

impl ViewerUi {
    /// Whether the Export sheet is showing.
    pub fn export_sheet_open(&self) -> bool {
        self.export_sheet.is_some()
    }

    /// Whether the completion sheet is showing.
    pub fn completion_open(&self) -> bool {
        self.completion.is_some()
    }

    /// Whether any of the three export sheets is showing (the shortcuts wait while one is).
    pub fn export_sheets_open(&self) -> bool {
        self.export_sheet.is_some() || self.completion.is_some() || self.preflight.is_some()
    }

    /// Opens the Export sheet (the toolbar's Export…, Ctrl+Shift+E). Folder: where this document last exported to,
    /// else the last folder used, else its own folder, else home.
    pub fn open_export_sheet(&mut self) {
        if self.viewer.open.is_none() {
            return;
        }
        self.preflight = None;
        self.completion = None;
        let folder = self
            .viewer
            .export_again_folder()
            .or_else(|| self.viewer.export_start_folder());
        self.export_sheet = Some(ExportSheet { folder });
    }

    /// The folder the system dialog picked for the Export sheet (Change…).
    pub fn export_set_folder(&mut self, folder: PathBuf) {
        if let Some(s) = self.export_sheet.as_mut() {
            s.folder = Some(folder);
        }
    }

    /// Where the folder dialog for the Export sheet starts.
    pub fn export_dialog_start(&self) -> Option<PathBuf> {
        self.export_sheet
            .as_ref()
            .and_then(|s| s.folder.clone())
            .or_else(|| self.viewer.export_start_folder())
    }

    /// The request the sheet would make for `which` devices, from the remembered choices.
    pub(crate) fn sheet_request(&self, which: ExportWhich) -> ExportRequest {
        let folder = self.export_dialog_start().unwrap_or_default();
        self.viewer.export_request(Some(which), folder)
    }

    /// Goes on from the Export sheet: it closes; the one preflight sheet follows when the rules call for it,
    /// else the files are written at once.
    pub(crate) fn begin_export(&mut self, req: ExportRequest, which: ExportWhich) {
        self.export_sheet = None;
        if self.viewer.preflight_report(which).needs_sheet() {
            self.preflight = Some(Preflight {
                then: Then::Run(req),
                which,
                notes_open: false,
            });
        } else {
            self.run_sheet_export(req);
        }
    }

    /// Writes the files of an export that came from the sheet and shows the completion sheet.
    pub(crate) fn run_sheet_export(&mut self, req: ExportRequest) {
        if let Ok(result) = self.viewer.run_export(&req) {
            self.completion = Some(Completion {
                result,
                textures: Vec::new(),
            });
        }
    }

    /// Export again (the menu, Ctrl+E): the remembered choices, no sheet and no completion sheet, the one-line
    /// notice instead. With nothing exported from this document yet, the Export sheet opens. The one preflight
    /// sheet still comes first when the rules call for it.
    pub fn ask_export_again(&mut self) {
        if self.viewer.open.is_none() {
            return;
        }
        if !self.viewer.can_export_again() {
            self.open_export_sheet();
            return;
        }
        self.export_sheet = None;
        self.completion = None;
        let which: ExportWhich = self.viewer.settings.devices.into();
        if self.viewer.preflight_report(which).needs_sheet() {
            self.preflight = Some(Preflight {
                then: Then::Again,
                which,
                notes_open: false,
            });
        } else {
            self.do_export_again();
        }
    }

    pub(crate) fn do_export_again(&mut self) {
        if let Again::NeedsSheet = self.viewer.export_again() {
            self.open_export_sheet();
        }
    }

    /// A press on Reveal in folder failed, or the app could not start the file manager.
    pub fn reveal_failed(&mut self, error: String) {
        self.viewer.notice = Some(Notice {
            message: Message::new("notice-reveal-failed").text("error", error),
            error: true,
        });
    }

    /// Looks for a screenshot: `export` (the sheet), `export-preset` (Device then set with a Set), `export-mistake`
    /// (all devices and a path with no device in it), `exported` and `exported-open` (the completion sheet folded
    /// and with its note open). Nothing is written to the settings file.
    pub(crate) fn show_export_look(&mut self, look: &str) {
        use crate::export_path::{DEFAULT_TEMPLATE, PRESET_DEVICE_THEN_SET};
        if self.viewer.open.is_none() {
            return;
        }
        let path = self.viewer.settings_path.take();
        match look {
            "export" => {
                self.viewer.settings.template = DEFAULT_TEMPLATE.into();
                self.open_export_sheet();
            }
            "export-preset" => {
                self.viewer.settings.template = PRESET_DEVICE_THEN_SET.into();
                self.viewer.settings.set = "planner-2027".into();
                self.viewer.settings.devices = DevicesChoice::All;
                self.open_export_sheet();
            }
            "export-mistake" => {
                self.viewer.settings.template = "{title}.png".into();
                self.viewer.settings.devices = DevicesChoice::All;
                self.open_export_sheet();
            }
            "exported" | "exported-open" => {
                let dir = std::env::temp_dir().join("quadrule-look-export");
                self.viewer.settings.template = PRESET_DEVICE_THEN_SET.into();
                self.viewer.settings.set = "planner-2027".into();
                self.viewer.settings.devices = DevicesChoice::All;
                self.viewer.settings.completion_note_open = look == "exported-open";
                let req = self.viewer.export_request(None, dir);
                self.run_sheet_export(req);
            }
            _ => {}
        }
        self.viewer.settings_path = path;
    }

    /// The Export sheet (Q2, Q9).
    pub(crate) fn export_sheet_window(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        let Some(sheet) = self.export_sheet.clone() else {
            return;
        };
        let mut s = self.viewer.settings.clone();
        let this_name = self.device_name(&self.viewer.profile_id.clone());
        let all_names: Vec<Message> = self
            .viewer
            .profile_ids()
            .iter()
            .map(|id| Message::new("app-exp-name").text("name", self.device_name(id)))
            .collect();
        let this_label = self
            .strings
            .format(&Message::new("app-exp-this").text("name", this_name));
        let all_label = self.strings.format(&Message::new("app-exp-all").arg(
            "names",
            Arg::List {
                items: all_names,
                sep: "lint-skipped-sep",
            },
        ));
        let mut go = false;
        let mut cancel = false;
        let mut change_folder = false;
        let mut text_done = false;
        let modal = egui::Modal::new(egui::Id::unique("export-sheet")).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.heading(self.t("app-exp-title"));
            ui.add_space(6.0);
            ui.label(self.t("app-exp-devices"));
            ui.radio_value(&mut s.devices, DevicesChoice::This, this_label);
            ui.radio_value(&mut s.devices, DevicesChoice::All, all_label);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(self.t("app-exp-folder"));
                if ui.button(self.t("app-exp-change")).clicked() {
                    change_folder = true;
                }
                let text = match &sheet.folder {
                    Some(f) => f.display().to_string(),
                    None => self.t("app-exp-no-folder"),
                };
                ui.add(egui::Label::new(text).truncate());
            });
            ui.add_space(6.0);
            let path_label = ui.label(self.t("app-exp-path"));
            ui.horizontal(|ui| {
                let r = ui
                    .add(
                        egui::TextEdit::singleline(&mut s.template)
                            .desired_width(330.0)
                            .font(egui::TextStyle::Monospace),
                    )
                    .on_hover_text(self.t("app-exp-path-tip"));
                text_done |= r.lost_focus();
                r.labelled_by(path_label.id);
                egui::ComboBox::from_id_salt("export-presets")
                    .selected_text(self.t("app-exp-presets"))
                    .width(90.0)
                    .show_ui(ui, |ui| {
                        for (id, template) in PRESETS {
                            if ui
                                .selectable_label(s.template == template, self.t(id))
                                .clicked()
                            {
                                s.template = template.to_string();
                                text_done = true;
                            }
                        }
                    })
                    .response
                    .widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::ComboBox,
                            true,
                            self.t("app-exp-presets"),
                        )
                    });
            });
            let set_label = ui.label(self.t("app-exp-set"));
            let r = ui
                .add(egui::TextEdit::singleline(&mut s.set).desired_width(430.0))
                .on_hover_text(self.t("app-exp-set-tip"));
            text_done |= r.lost_focus();
            r.labelled_by(set_label.id);
            ui.add_space(6.0);

            // What would be written, every file; a mistake shows in red and holds Export back.
            let req = ExportRequest {
                devices: self.viewer.export_devices(s.devices.into()),
                folder: sheet.folder.clone().unwrap_or_default(),
                template: s.template.clone(),
                set: s.set.clone(),
                note: s.note,
                date: self.viewer.export_date(),
            };
            let mut ok = sheet.folder.is_some();
            ui.label(egui::RichText::new(self.t("app-exp-files")).weak());
            match self.viewer.plan_export(&req) {
                Ok(files) if !files.is_empty() => {
                    for (_, rel) in files {
                        ui.label(egui::RichText::new(rel).monospace());
                    }
                }
                Ok(_) => {
                    ok = false;
                    ui.label(self.t("app-exp-no-files"));
                }
                Err(e) => {
                    ok = false;
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        self.strings.format(&e.message()),
                    );
                }
            }
            ui.add_space(6.0);
            ui.checkbox(&mut s.note, self.t("app-exp-note"));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-exp-cancel")).clicked() {
                    cancel = true;
                }
                if ui
                    .add_enabled(ok, egui::Button::new(self.t("app-exp-go")))
                    .clicked()
                {
                    go = true;
                }
            });
            req
        });
        let close_asked = modal.should_close();
        let req = modal.inner;
        let which: ExportWhich = s.devices.into();
        let changed = s != self.viewer.settings;
        if changed {
            self.viewer.settings = s;
        }
        if change_folder {
            requests.push(Request::ExportFolderDialog);
        }
        if cancel || close_asked {
            self.viewer.save_settings();
            self.export_sheet = None;
        } else if go {
            self.viewer.save_settings();
            self.begin_export(req, which);
        } else if changed && (text_done || !ctx.egui_wants_keyboard_input()) {
            self.viewer.save_settings();
        }
    }

    /// The completion sheet (Q7), with the MyStyle note folded until the person opens it.
    pub(crate) fn completion_window(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        let Some(mut done) = self.completion.take() else {
            return;
        };
        if done.textures.is_empty() {
            for (i, f) in done.result.files.iter().enumerate() {
                let img = egui::ColorImage::from_gray(
                    [f.thumbnail.width as usize, f.thumbnail.height as usize],
                    &f.thumbnail.data,
                );
                done.textures.push(ctx.load_texture(
                    format!("exported-{i}"),
                    img,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        let mut open = self.viewer.settings.completion_note_open;
        let mut close = false;
        let mut reveal = false;
        let r = &done.result;
        let names: Vec<String> = r
            .files
            .iter()
            .map(|f| {
                f.path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })
            .collect();
        let title = self.strings.format(&Message::new("app-done-title").num(
            "count",
            r.files.len() as f64,
            0,
        ));
        let modal = egui::Modal::new(egui::Id::unique("export-done")).show(ctx, |ui| {
            ui.set_max_width(520.0);
            ui.heading(title);
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for (t, f) in done.textures.iter().zip(&r.files) {
                    let scale = 120.0 / THUMB_SIDE as f32;
                    let size = egui::vec2(
                        f.thumbnail.width as f32 * scale,
                        f.thumbnail.height as f32 * scale,
                    );
                    ui.add(egui::Image::new(egui::load::SizedTexture::new(
                        t.id(),
                        size,
                    )));
                }
            });
            ui.add_space(4.0);
            ui.label(names.join(" \u{b7} "));
            ui.label(self.strings.format(
                &Message::new("app-done-in").text("folder", r.shared_folder.display().to_string()),
            ));
            if r.replaced > 0 {
                ui.label(
                    egui::RichText::new(self.strings.format(
                        &Message::new("app-done-replaced").num("count", r.replaced as f64, 0),
                    ))
                    .weak(),
                );
            }
            ui.add_space(4.0);
            let head = egui::CollapsingHeader::new(self.t("app-done-where"))
                .id_salt("export-done-where")
                .open(Some(open))
                .show(ui, |ui| {
                    ui.label(self.t("app-done-where-body"));
                });
            if head.header_response.clicked() {
                open = !open;
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-done-reveal")).clicked() {
                    reveal = true;
                }
                if ui.button(self.t("app-done-done")).clicked() {
                    close = true;
                }
            });
        });
        if open != self.viewer.settings.completion_note_open {
            self.viewer
                .change_settings(|s| s.completion_note_open = open);
        }
        if reveal {
            requests.push(Request::RevealFolder(done.result.shared_folder.clone()));
        }
        if !(close || modal.should_close()) {
            self.completion = Some(done);
        }
    }
}
