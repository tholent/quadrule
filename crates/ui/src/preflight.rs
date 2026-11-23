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

//! The preflight sheet (rung 14b, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q5): Export opens one
//! sheet when there are Warn findings on the devices being exported, or L20 (installed fonts are used) applies.
//! Findings are grouped by device, then a Document group that holds L20; each row has Jump to object, Ignore and
//! its quick fix if it has one; Info findings are folded under "N notes"; a line says why a paper page has
//! fewer checks. Buttons: Cancel and Export anyway. With no Warn finding and no L20, Export goes straight on.
//! Save and Save As keep rung 12's font sheet ([`crate::missing_ui`]) exactly as it was.

use crate::export::ExportRequest;
use crate::lint_ui::{Action, RowStyle};
use crate::view::ViewerUi;
use crate::viewer::{ExportWhich, Viewer};
use quadrule_i18n::{Arg, Message};
use quadrule_lint::{Finding, Severity, SkipReason, skipped_checks};
use quadrule_profiles::active::lookup_in;

/// What Export anyway goes on to do.
#[derive(Clone, Debug)]
pub enum Then {
    /// Write the files the Export sheet asked for, then show the completion sheet.
    Run(ExportRequest),
}

/// The sheet while it shows: the export it holds back.
#[derive(Clone, Debug)]
pub struct Preflight {
    /// What happens when Export anyway is pressed.
    pub then: Then,
    /// Which devices are being exported.
    pub which: ExportWhich,
    /// The "N notes" fold is open.
    pub notes_open: bool,
}

/// What the sheet lists, worked out from the document as it is now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// The Warn findings of each device being exported, in the document's order; devices with none are left out.
    pub devices: Vec<(String, Vec<Finding>)>,
    /// The Warn findings about the whole document (L20).
    pub document: Vec<Finding>,
    /// The Info findings, folded under "N notes".
    pub notes: Vec<Finding>,
    /// The paper devices being exported, which skip checks.
    pub paper: Vec<String>,
}

impl Report {
    /// Whether the sheet is wanted: some Warn finding, or L20.
    pub fn needs_sheet(&self) -> bool {
        !self.devices.is_empty() || !self.document.is_empty()
    }

    /// Every Warn finding the sheet lists.
    pub fn warnings(&self) -> usize {
        self.devices.iter().map(|(_, f)| f.len()).sum::<usize>() + self.document.len()
    }
}

impl Viewer {
    /// The devices an export of `which` writes: the one on screen, or every device of the page.
    pub fn export_devices(&self, which: ExportWhich) -> Vec<String> {
        match which {
            ExportWhich::ThisDevice => vec![self.profile_id.clone()],
            ExportWhich::AllDevices => self.profile_ids(),
        }
    }

    /// What the preflight sheet lists for an export of `which`.
    pub fn preflight_report(&mut self, which: ExportWhich) -> Report {
        let mut report = Report::default();
        let Some(doc) = self.doc() else {
            return report;
        };
        let primary = doc.primary_profile_id.clone();
        let dont_warn = doc.page.fonts.dont_warn;
        let paper: Vec<String> = self
            .export_devices(which)
            .into_iter()
            .filter(|id| {
                self.doc()
                    .and_then(|d| lookup_in(d, id).ok())
                    .is_some_and(|p| p.is_paper())
            })
            .collect();
        // L20 is about the document: it is said once, on the primary device, whichever device is exported.
        if let Ok(page) = self.page_for(&primary)
            && !dont_warn
        {
            report.document = page
                .findings
                .iter()
                .filter(|f| f.rule == "L20")
                .cloned()
                .collect();
        }
        for (dev, findings) in self.findings_on(&self.export_devices(which)) {
            let mut warns = Vec::new();
            for f in findings {
                if f.rule == "L20" {
                    continue;
                }
                match f.severity {
                    Severity::Warn => warns.push(f),
                    Severity::Info => report.notes.push(f),
                }
            }
            if !warns.is_empty() {
                report.devices.push((dev, warns));
            }
        }
        report.paper = paper;
        report
    }
}

impl ViewerUi {
    /// Whether the preflight sheet is showing.
    pub fn preflight_open(&self) -> bool {
        self.preflight.is_some()
    }

    /// Starts with one part of the window showing, for a look at it (`quadrule --look lint|style|preflight|welcome|file-menu|gallery-mine`):
    /// the Lint tab; the Style tab with the first object that has a Custom stroke selected; or the sheet
    /// before exporting this device.
    pub fn show_look(&mut self, look: &str) {
        match look {
            "lint" => {
                self.show_lint = true;
                self.tab = crate::panels::Tab::Lint;
            }
            "style" => {
                let pick = self.viewer.doc().and_then(|d| {
                    d.objects()
                        .find(|o| matches!(o.style, quadrule_model::Style::Custom { .. }))
                        // The pattern-borders page has none of those: its Border section is the look.
                        .or_else(|| {
                            d.objects().find(|o| {
                                matches!(&o.kind, quadrule_model::ObjectKind::Pattern(p)
                                    if p.border.as_ref().is_some_and(|b| b.corner_radius_mm > 0.0))
                            })
                        })
                        .map(|o| o.id)
                });
                if let Some(id) = pick {
                    self.viewer.select_row(id, false);
                    self.tab = crate::panels::Tab::Style;
                    self.look_border = self
                        .viewer
                        .doc()
                        .and_then(|d| d.object(id))
                        .is_some_and(|o| matches!(&o.kind, quadrule_model::ObjectKind::Pattern(p) if p.border.is_some()));
                }
            }
            // The Style tab on the grand-staff page's first grand staff: Line gap, Space between grand staves, Grand staff.
            "grand-staff" => {
                let id = quadrule_model::sample::grand_ids::FIRST;
                if self.viewer.doc().is_some_and(|d| d.object(id).is_some()) {
                    self.viewer.select_row(id, false);
                    self.tab = crate::panels::Tab::Style;
                }
            }
            // The Style tab on the calendar page's dated calendar, or its first mini calendar.
            "calendar" | "mini-calendar" => {
                let id = if look == "calendar" {
                    quadrule_model::sample::calendar_ids::DATED
                } else {
                    quadrule_model::sample::calendar_ids::MINI
                };
                if self.viewer.doc().is_some_and(|d| d.object(id).is_some()) {
                    self.viewer.select_row(id, false);
                    self.tab = crate::panels::Tab::Style;
                }
            }
            "preflight" => {
                let which = ExportWhich::ThisDevice;
                let req = self.sheet_request(which);
                self.begin_export(req, which);
            }
            // The welcome screen with nothing open; the File menu open over the page.
            "welcome" => self.show_welcome(),
            "file-menu" => self.look_file_menu(),
            // The other menus of the menu bar, and the About sheet (fixes batch 2). Edit opens with an object picked and
            // moved, so Undo, Copy, Cut and Duplicate show as they do when they can act; View opens Snapping too.
            "menu-edit" => {
                let id = quadrule_model::sample::ids::RULE;
                if self.viewer.doc().is_some_and(|d| d.object(id).is_some()) {
                    self.viewer.select_row(id, false);
                    let _ = self.viewer.nudge(1.0, 0.0, false);
                }
                self.look_menu = Some(crate::menu_bar::LookMenu::top("menu-edit"));
            }
            "menu-view" => {
                self.look_menu = Some(crate::menu_bar::LookMenu::with_sub(
                    "app-view",
                    "menu-snapping",
                ));
            }
            "menu-help" => self.look_menu = Some(crate::menu_bar::LookMenu::top("menu-help")),
            "about" => self.about_open = true,
            // The layers panel: New layer's name field, the ask before Delete layer, a row's right-click menu.
            "layers" | "layers-new" | "layers-ask" | "layers-menu" => self.look_layers(look),
            // My templates, the tab the person's own pages are on.
            "gallery-mine" => {
                self.show_welcome();
                self.set_welcome_category(crate::welcome::Category::MyTemplates);
            }
            other => self.show_export_look(other),
        }
    }

    /// Answers the sheet: Export anyway sends the request on, Cancel drops it.
    pub fn answer_preflight(&mut self, export: bool) {
        match self.preflight.take() {
            Some(p) if export => match p.then {
                Then::Run(req) => self.run_sheet_export(req),
            },
            _ => {}
        }
    }

    /// The line that says why a paper page has fewer checks, one per paper device being exported.
    pub fn preflight_paper_lines(&self, report: &Report) -> Vec<String> {
        let Some(doc) = self.viewer.doc() else {
            return Vec::new();
        };
        report
            .paper
            .iter()
            .filter_map(|id| {
                let target = lookup_in(doc, id).ok()?;
                let rules: Vec<Message> = skipped_checks(doc, &target)
                    .iter()
                    .filter(|k| k.reason == SkipReason::Paper)
                    .map(|k| Message::new("app-pre-rule").text("rule", k.rule))
                    .collect();
                if rules.is_empty() {
                    return None;
                }
                Some(
                    self.strings.format(
                        &Message::new("app-pre-paper")
                            .text("device", self.device_name(id))
                            .arg(
                                "rules",
                                Arg::List {
                                    items: rules,
                                    sep: "lint-skipped-sep",
                                },
                            ),
                    ),
                )
            })
            .collect()
    }

    /// The sheet shown before Export when there is something to say (Q5).
    pub(crate) fn preflight_window(&mut self, ctx: &egui::Context) {
        let Some(sheet) = self.preflight.clone() else {
            return;
        };
        let report = self.viewer.preflight_report(sheet.which);
        let primary = self.primary_id();
        let l16: Vec<Finding> = self
            .viewer
            .findings_on(std::slice::from_ref(&primary))
            .into_iter()
            .flat_map(|(_, f)| f)
            .filter(|f| f.rule == "L16" && f.message.id == "lint-l16")
            .collect();
        let paper_lines = self.preflight_paper_lines(&report);
        let mut action: Option<Action> = None;
        let mut answer: Option<bool> = None;
        let mut notes_open = sheet.notes_open;
        let rows = RowStyle {
            show_device: false,
            jump: true,
            short_ignore: true,
        };
        egui::Modal::new(egui::Id::unique("preflight-sheet")).show(ctx, |ui| {
            ui.set_max_width(520.0);
            ui.heading(self.t("app-pre-title"));
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .max_height(ctx.content_rect().height() * 0.6)
                .show(ui, |ui| {
                    if report.warnings() == 0 {
                        ui.label(self.t("app-pre-clear"));
                    }
                    for (dev, findings) in &report.devices {
                        ui.strong(self.device_name(dev));
                        for f in findings {
                            ui.label(self.finding_text(f));
                            if let Some(a) = self.finding_buttons(ui, f, rows, l16.len()) {
                                action = Some(a);
                            }
                            ui.add_space(4.0);
                        }
                    }
                    if !report.document.is_empty() {
                        ui.strong(self.t("app-pre-document"));
                        for f in &report.document {
                            ui.label(self.finding_text(f));
                            if let Some(a) = self.finding_buttons(ui, f, rows, l16.len()) {
                                action = Some(a);
                            }
                            ui.add_space(4.0);
                        }
                    }
                    if !report.notes.is_empty() {
                        let label = self.strings.format(&Message::new("app-pre-notes").num(
                            "count",
                            report.notes.len() as f64,
                            0,
                        ));
                        let head = egui::CollapsingHeader::new(label)
                            .id_salt("preflight-notes")
                            .open(Some(notes_open))
                            .show(ui, |ui| {
                                let notes_rows = RowStyle {
                                    show_device: report.devices.len() > 1 || report.notes.len() > 1,
                                    ..rows
                                };
                                for f in &report.notes {
                                    ui.label(self.finding_text(f));
                                    if let Some(a) =
                                        self.finding_buttons(ui, f, notes_rows, l16.len())
                                    {
                                        action = Some(a);
                                    }
                                    ui.add_space(4.0);
                                }
                            });
                        if head.header_response.clicked() {
                            notes_open = !notes_open;
                        }
                    }
                    for line in &paper_lines {
                        ui.label(egui::RichText::new(line).small().weak());
                    }
                });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-pre-cancel")).clicked() {
                    answer = Some(false);
                }
                if ui.button(self.t("app-pre-export")).clicked() {
                    answer = Some(true);
                }
            });
        });
        if let Some(p) = self.preflight.as_mut() {
            p.notes_open = notes_open;
        }
        if let Some(a) = action {
            // Jump closes the sheet: the person goes to the object.
            if matches!(a, Action::Jump(_)) {
                self.preflight = None;
            }
            self.do_action(a, &l16);
        }
        if let Some(go) = answer {
            self.answer_preflight(go);
        }
    }
}
