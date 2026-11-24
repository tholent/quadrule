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

//! The Lint tab (rung 14b, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q6 and Q7): a device picker at the
//! top (All devices, or one; view state, not saved), each finding with its quick fixes and Ignore this rule for
//! this document, and at the end the list of ignored rules, each with a button to turn it back on. The row's
//! buttons are shared with the preflight sheet ([`crate::preflight`]).

use crate::view::ViewerUi;
use egui::RichText;
use quadrule_i18n::Message;
use quadrule_lint::{Finding, Fix, Severity, rule_title};
use quadrule_profiles::active::lookup_in;

/// What a button on a finding's row asks for.
#[derive(Clone, Debug)]
pub(crate) enum Action {
    /// Outline the finding on its device and, when it is about an object, select the object (rung 17, Defaulted 2).
    Select(String, usize, Finding),
    /// Do a quick fix.
    Fix(Finding, Fix),
    /// Turn the rule off for this document.
    Ignore(&'static str),
    /// Turn the rule back on.
    Restore(String),
    /// Jump to the finding's object (the preflight sheet).
    Jump(Finding),
}

/// How a row's buttons are worded.
#[derive(Clone, Copy)]
pub(crate) struct RowStyle {
    /// Name the device on the row.
    pub show_device: bool,
    /// Offer Jump to object.
    pub jump: bool,
    /// Say just "Ignore" (the sheet), with the full words as the tip.
    pub short_ignore: bool,
}

impl ViewerUi {
    /// The device the Lint tab lists (`None`: all devices). View state, not saved in the file.
    pub fn lint_device(&self) -> Option<&str> {
        self.lint_device.as_deref()
    }

    /// Picks the device the Lint tab lists; `None` is All devices.
    pub fn set_lint_device(&mut self, device: Option<&str>) {
        self.lint_device = device.map(str::to_string);
    }

    /// The devices the Lint tab lists now: the picked one if the document still has it, else all.
    pub fn lint_scope(&self) -> Vec<String> {
        let all = self.viewer.profile_ids();
        match &self.lint_device {
            Some(d) if all.contains(d) => vec![d.clone()],
            _ => all,
        }
    }

    /// A finding as one line of words.
    pub(crate) fn finding_text(&self, f: &Finding) -> String {
        let icon = match f.severity {
            Severity::Warn => "⚠",
            Severity::Info => "ℹ",
        };
        format!("{icon} {}  {}", f.rule, self.strings.format(&f.message))
    }

    /// What a finding is called to a screen reader beside others of the same words: its rule, the object it is
    /// about, and the device it is on (`L05, L03 line, Manta`).
    pub(crate) fn finding_tag(&self, f: &Finding) -> String {
        let object = f
            .object
            .and_then(|id| self.viewer.doc().and_then(|d| d.object(id)))
            .map(|o| o.name.clone());
        let device = self.device_name(&f.profile);
        let m = match object {
            Some(o) => Message::new("app-finding-tag").text("object", o),
            None => Message::new("app-finding-tag-page"),
        };
        self.strings
            .format(&m.text("rule", f.rule).text("device", device))
    }

    /// A quick fix's button words.
    pub(crate) fn fix_label(&self, f: &Finding, fix: Fix) -> String {
        let Some(doc) = self.viewer.doc() else {
            return String::new();
        };
        match lookup_in(doc, &f.profile) {
            Ok(target) => self.strings.format(&fix.label(doc, &target)),
            Err(_) => String::new(),
        }
    }

    /// The buttons of one finding: Jump to object (the sheet), its quick fixes and Ignore. Snap all shows only
    /// when `l16_count` says there is more than one off-rhythm object to snap.
    pub(crate) fn finding_buttons(
        &mut self,
        ui: &mut egui::Ui,
        f: &Finding,
        style: RowStyle,
        l16_count: usize,
    ) -> Option<Action> {
        let mut action = None;
        let tag = self.finding_tag(f);
        let fixes = self.viewer.fixes_for(f);
        ui.horizontal_wrapped(|ui| {
            if style.show_device {
                let name = self.device_name(&f.profile);
                ui.label(RichText::new(name).small().strong());
            }
            if style.jump && f.object.is_some() {
                let words = self.t("app-pre-jump");
                let r = ui.small_button(&words);
                crate::a11y::name_button(&r, true, &format!("{words}: {tag}"));
                if r.clicked() {
                    action = Some(Action::Jump(f.clone()));
                }
            }
            for fix in fixes {
                if fix == Fix::SnapAllToRhythm && l16_count < 2 {
                    continue;
                }
                let label = self.fix_label(f, fix);
                let r = ui.small_button(&label);
                crate::a11y::name_button(&r, true, &format!("{label}: {tag}"));
                if r.clicked() {
                    action = Some(Action::Fix(f.clone(), fix));
                }
            }
            let (words, tip) = if style.short_ignore {
                (self.t("lint-ignore-short"), Some(self.t("lint-ignore")))
            } else {
                (self.t("lint-ignore"), None)
            };
            let mut b = ui.small_button(&words);
            crate::a11y::name_button(&b, true, &format!("{words}: {tag}"));
            if let Some(tip) = tip {
                b = b.on_hover_text(tip);
            } else {
                b = b.on_hover_text(self.t("lint-ignore-tip"));
            }
            if b.clicked() {
                action = Some(Action::Ignore(f.rule));
            }
        });
        action
    }

    /// Does what a button asked.
    pub(crate) fn do_action(&mut self, action: Action, all: &[Finding]) {
        match action {
            Action::Select(dev, idx, f) => {
                let same = self.viewer.profile_id == dev && self.viewer.selected == Some(idx);
                if same {
                    self.viewer.selected = None;
                } else {
                    // The object, when there is one, is picked with its handles; the Lint tab stays open, and
                    // the canvas scrolls to it. A finding of the page itself only outlines.
                    self.viewer.show_finding(&dev, idx);
                    let _ = self.viewer.jump_to(&f);
                    self.viewer.selected = Some(idx);
                    self.scroll_to_selected = true;
                }
            }
            Action::Fix(f, fix) => {
                let _ = self.viewer.apply_fix(&f, fix, all);
            }
            Action::Ignore(rule) => {
                let _ = self.viewer.ignore_rule(rule);
            }
            Action::Restore(rule) => {
                let _ = self.viewer.restore_rule(&rule);
            }
            Action::Jump(f) => {
                if self.viewer.jump_to(&f) {
                    self.scroll_to_selected = true;
                }
            }
        }
    }

    /// The device picker: All devices, or one device of the page.
    fn lint_picker(&mut self, ui: &mut egui::Ui) {
        let ids = self.viewer.profile_ids();
        if ids.len() < 2 {
            return;
        }
        let scope_one = self.lint_scope().len() == 1;
        let shown = if scope_one {
            self.device_name(&self.lint_scope()[0])
        } else {
            self.t("app-lint-all-devices")
        };
        let mut pick: Option<Option<String>> = None;
        ui.horizontal(|ui| {
            let label = ui.label(self.t("app-lint-device"));
            egui::ComboBox::from_id_salt("lint-device")
                .selected_text(shown)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(!scope_one, self.t("app-lint-all-devices"))
                        .clicked()
                    {
                        pick = Some(None);
                    }
                    for id in &ids {
                        let selected = scope_one && self.lint_scope()[0] == *id;
                        if ui
                            .selectable_label(selected, self.device_name(id))
                            .clicked()
                        {
                            pick = Some(Some(id.clone()));
                        }
                    }
                })
                .response
                .labelled_by(label.id);
        });
        if let Some(p) = pick {
            self.set_lint_device(p.as_deref());
        }
    }

    pub(crate) fn lint_tab(&mut self, ui: &mut egui::Ui) {
        if self.viewer.page().is_none() {
            return;
        }
        let scope = self.lint_scope();
        let several = self.viewer.profile_ids().len() > 1;
        let all_devices = scope.len() > 1;
        self.lint_picker(ui);
        let saved = self.viewer.page().is_some_and(|p| p.saved_outlines);
        ui.label(self.t("app-lint-toolbars"));
        if saved {
            ui.label(self.t("app-saved-outlines"));
        }
        ui.separator();
        let groups = self.viewer.findings_on(&scope);
        let total: usize = groups.iter().map(|(_, f)| f.len()).sum();
        if total == 0 {
            ui.label(self.t("app-lint-none"));
        }
        // One plain line per device on the checks that did not run on it, and why (rung 14a bbq, Q2).
        let skipped = self.lint_skipped_lines_in(&scope);
        for line in &skipped {
            ui.label(RichText::new(line).small().weak());
        }
        // The page's off-rhythm objects, for Snap all (L16 is judged on the primary device).
        let primary = self.primary_id();
        let l16: Vec<Finding> = self
            .viewer
            .findings_on(std::slice::from_ref(&primary))
            .into_iter()
            .flat_map(|(_, f)| f)
            .filter(|f| f.rule == "L16" && f.message.id == "lint-l16")
            .collect();
        let mut action: Option<Action> = None;
        let style = RowStyle {
            show_device: all_devices && several,
            jump: false,
            short_ignore: false,
        };
        for (dev, findings) in &groups {
            for (i, f) in findings.iter().enumerate() {
                let text = self.finding_text(f);
                let selected = self.viewer.profile_id == *dev && self.viewer.selected == Some(i);
                let r = ui.add(egui::Button::selectable(selected, &text).wrap());
                // The same finding is listed once for each device when all are shown.
                crate::a11y::name_button(
                    &r,
                    true,
                    &format!("{text} ({})", self.device_name(&f.profile)),
                );
                if r.clicked() {
                    action = Some(Action::Select(dev.clone(), i, f.clone()));
                }
                if let Some(a) = self.finding_buttons(ui, f, style, l16.len()) {
                    action = Some(a);
                }
                ui.add_space(4.0);
            }
        }
        // The rules this page turns off, each with a button to turn it back on.
        let ignored = self.viewer.ignored_rules();
        if !ignored.is_empty() {
            ui.separator();
            ui.strong(
                self.strings
                    .format(&Message::new("lint-ignored-heading").num(
                        "count",
                        ignored.len() as f64,
                        0,
                    )),
            );
            for rule in &ignored {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!(
                        "{rule}  {}",
                        self.strings.format(&rule_title(rule))
                    ));
                    let b = ui.button(self.t("lint-ignored-on"));
                    if b.clicked() {
                        action = Some(Action::Restore(rule.clone()));
                    }
                });
            }
        }
        if let Some(a) = action {
            self.do_action(a, &l16);
        }
    }
}
