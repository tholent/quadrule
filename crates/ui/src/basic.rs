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
//! The Basic section (rung 16b, Q5; ADR-026, proposed): a **Show in Basic** switch on each object and on the page's
//! title, margins and rhythm, and a **Basic** tab, first in the inspector, that lists what is marked with its main
//! settings. The settings are drawn by the very widgets the Style, Text and Page tabs use, so a setting is the same
//! wherever it is changed; every setting stays in those tabs too.
//!
//! The section shows only when the app is started with `--basic-section` (`ViewerUi::basic_section`), until chris
//! approves its look. To remove the flag afterwards: delete that one field and `show_switches` here, and the
//! `show_switches()` checks in `inspector` (the tab list), `arrange_tab` and `page_tab`. The file format reads and
//! writes the marks whether or not the flag is on.

use crate::panels::StrokeMixed;
use crate::view::ViewerUi;
use egui::{RichText, Ui};
use quadrule_layout::units::Axis;
use quadrule_model::edit::{Command, EditError};
use quadrule_model::{BasicPage, Document, Object, ObjectKind, Uuid};

/// One of the page's three things that can be marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageMark {
    /// The page's title.
    Title,
    /// The page's margins.
    Margins,
    /// The page's rhythm.
    Rhythm,
}

impl PageMark {
    /// Whether this one is marked.
    pub fn is_on(self, b: &BasicPage) -> bool {
        match self {
            PageMark::Title => b.title,
            PageMark::Margins => b.margins,
            PageMark::Rhythm => b.rhythm,
        }
    }

    fn set(self, b: &mut BasicPage, on: bool) {
        match self {
            PageMark::Title => b.title = on,
            PageMark::Margins => b.margins = on,
            PageMark::Rhythm => b.rhythm = on,
        }
    }
}

impl crate::viewer::Viewer {
    /// Marks the editable selected objects **Show in Basic**, or takes the mark off (one undo step): the Basic tab
    /// lists a marked object, and nothing else changes (ADR-026).
    pub fn set_show_in_basic(&mut self, on: bool) -> Result<(), EditError> {
        self.edit_selected(None, |x| x.show_in_basic = on)
    }

    /// Marks one of the page's own settings **Show in Basic**, or takes the mark off (one undo step).
    pub fn set_page_basic(&mut self, which: PageMark, on: bool) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        if which.is_on(&doc.page.basic) == on {
            return Ok(());
        }
        let mut page = doc.page.clone();
        which.set(&mut page.basic, on);
        self.edit(Command::SetPage(page), None)
    }

    /// The marked objects, in the order of the page (a layer's first object first, a group before what is in it).
    pub fn basic_objects(&self) -> Vec<Uuid> {
        self.doc()
            .map(|d| {
                d.objects()
                    .filter(|o| o.show_in_basic)
                    .map(|o| o.id)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl ViewerUi {
    /// Whether the Basic tab and the **Show in Basic** switches show: only when the app was started with
    /// `--basic-section` (ADR-026, until chris approves the look).
    pub(crate) fn show_switches(&self) -> bool {
        self.basic_section
    }

    /// `--look basic`: the section on, the page's three settings marked, and the first line, text box and pattern
    /// region on the page marked, with the Basic tab showing and nothing selected.
    pub fn look_basic(&mut self) {
        self.basic_section = true;
        let Some(doc) = self.viewer.doc() else {
            return;
        };
        let pick =
            |f: &dyn Fn(&ObjectKind) -> bool| doc.objects().find(|o| f(&o.kind)).map(|o| o.id);
        let ids: Vec<Uuid> = [
            pick(&|k| matches!(k, ObjectKind::Line(_))),
            pick(&|k| matches!(k, ObjectKind::Text(_))),
            pick(&|k| matches!(k, ObjectKind::Pattern(_))),
        ]
        .into_iter()
        .flatten()
        .collect();
        for which in [PageMark::Title, PageMark::Margins, PageMark::Rhythm] {
            let _ = self.viewer.set_page_basic(which, true);
        }
        let saved = std::mem::take(&mut self.viewer.selection);
        for id in ids {
            self.viewer.selection = vec![id];
            let _ = self.viewer.set_show_in_basic(true);
        }
        self.viewer.selection = saved;
        self.tab = crate::panels::Tab::Basic;
    }

    /// The **Show in Basic** tick on the Arrange tab for the selected objects; mixed when some have it and some do
    /// not.
    pub(crate) fn show_in_basic_tick(&mut self, ui: &mut Ui, enabled: bool, mixed: bool) {
        let picked: Vec<bool> = self
            .viewer
            .selection
            .iter()
            .filter_map(|id| self.viewer.doc().and_then(|d| d.object(*id)))
            .map(|x| x.show_in_basic)
            .collect();
        let mut on = !picked.is_empty() && picked.iter().all(|b| *b);
        let tick = egui::Checkbox::new(&mut on, self.t("app-show-in-basic")).indeterminate(mixed);
        let r = ui
            .add_enabled(enabled, tick)
            .on_hover_text(self.t("app-show-in-basic-tip"));
        if r.changed() {
            let _ = self.viewer.set_show_in_basic(on);
        }
    }

    /// The **Show in Basic** tick for one of the page's own settings, beside its heading.
    pub(crate) fn page_mark_tick(&mut self, ui: &mut Ui, which: PageMark) {
        let Some(on) = self.viewer.doc().map(|d| which.is_on(&d.page.basic)) else {
            return;
        };
        let mut want = on;
        let tick = ui
            .checkbox(&mut want, self.t("app-show-in-basic"))
            .on_hover_text(self.t("app-show-in-basic-tip"));
        if tick.changed() {
            let _ = self.viewer.set_page_basic(which, want);
        }
    }

    /// The Basic tab: what is marked, each with its main settings. It is about the document, so it shows whether or
    /// not anything is selected.
    pub(crate) fn basic_tab(&mut self, ui: &mut Ui) {
        let Some(doc) = self.viewer.doc().cloned() else {
            return;
        };
        let objects = self.viewer.basic_objects();
        if !doc.page.basic.any() && objects.is_empty() {
            ui.label(self.t("app-basic-empty"));
            return;
        }
        if doc.page.basic.any() {
            self.basic_page(ui, &doc);
        }
        for id in objects {
            ui.separator();
            self.basic_object(ui, id);
        }
    }

    /// The page's marked settings: title, margins, rhythm.
    fn basic_page(&mut self, ui: &mut Ui, doc: &Document) {
        let (Some(cx), Some(cy)) = (
            self.viewer.unit_context(Axis::X),
            self.viewer.unit_context(Axis::Y),
        ) else {
            return;
        };
        ui.push_id("basic-page", |ui| {
            ui.heading(self.t("app-basic-page"));
            if doc.page.basic.title {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.strong(self.t("app-page-title"));
                    self.page_mark_tick(ui, PageMark::Title);
                });
                egui::Grid::new("page-grid")
                    .num_columns(2)
                    .show(ui, |ui| self.title_row(ui, doc));
            }
            if doc.page.basic.margins {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.strong(self.t("app-page-margins"));
                    self.page_mark_tick(ui, PageMark::Margins);
                });
                self.margins_rows(ui, doc, cx, cy);
            }
            if doc.page.basic.rhythm {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.strong(self.t("app-rhythm"));
                    self.page_mark_tick(ui, PageMark::Rhythm);
                });
                let r = doc.rhythm.clone();
                self.rhythm_link(ui, &r);
                egui::Grid::new("rhythm-grid")
                    .num_columns(2)
                    .show(ui, |ui| self.rhythm_unit_rows(ui, &r, cx, cy));
            }
        });
    }

    /// One marked object: its name, the tick that takes it off, a button that picks it on the page, and its main
    /// settings. The settings are the other tabs' widgets, which act on the selection, so the object stands in as the
    /// selection while they are drawn and the real selection comes back after.
    fn basic_object(&mut self, ui: &mut Ui, id: Uuid) {
        let Some(o) = self.viewer.doc().and_then(|d| d.object(id)).cloned() else {
            return;
        };
        let saved = std::mem::replace(&mut self.viewer.selection, vec![id]);
        let mut pick = None;
        ui.push_id(id, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&o.name);
                let mut on = true;
                let tick = ui
                    .checkbox(&mut on, self.t("app-show-in-basic"))
                    .on_hover_text(self.t("app-show-in-basic-tip"));
                if tick.changed() {
                    let _ = self.viewer.set_show_in_basic(on);
                }
                if ui
                    .small_button(self.t("app-basic-select"))
                    .on_hover_text(self.t("app-basic-select-tip"))
                    .clicked()
                {
                    // The real selection becomes this object once the widgets are done.
                    pick = Some(id);
                }
            });
            self.basic_settings(ui, &o);
        });
        self.viewer.selection = match pick {
            Some(pick) => {
                self.viewer.entered = self
                    .viewer
                    .doc()
                    .map(|d| d.ancestors(pick))
                    .unwrap_or_default();
                vec![pick]
            }
            None => saved,
        };
    }

    /// An object's main settings, by kind, drawn with the widgets of the tabs that hold them.
    fn basic_settings(&mut self, ui: &mut Ui, o: &Object) {
        let Some(cx) = self.viewer.unit_context(Axis::X) else {
            return;
        };
        let enabled = !self.viewer.editable_selection().is_empty();
        match &o.kind {
            // A line: its role (or its width and gray, when it has its own).
            ObjectKind::Line(_) => {
                egui::Grid::new("basic-grid").num_columns(2).show(ui, |ui| {
                    self.stroke_rows(ui, o, &cx, enabled, StrokeMixed::default());
                });
            }
            // A text box: its text.
            ObjectKind::Text(t) => {
                ui.add_enabled_ui(enabled, |ui| self.text_content_field(ui, t));
            }
            // A pattern region: what defines it, its spacing and its major lines.
            ObjectKind::Pattern(pr) => {
                egui::Grid::new("basic-grid").num_columns(2).show(ui, |ui| {
                    self.pattern_kind_rows(ui, o, pr, &cx, enabled);
                    self.pattern_spacing_rows(ui, pr, &cx, enabled);
                    self.pattern_major_rows(ui, pr, enabled);
                });
            }
            // A shape: its stroke and its fill.
            ObjectKind::Rect(_) | ObjectKind::Ellipse(_) | ObjectKind::Polygon(_) => {
                egui::Grid::new("basic-grid").num_columns(2).show(ui, |ui| {
                    self.stroke_rows(ui, o, &cx, enabled, StrokeMixed::default());
                    self.fill_rows(ui, o, enabled);
                });
            }
            ObjectKind::MiniCalendar(m) => self.mini_calendar_rows(ui, m, enabled),
            // A group or a repeat holds other objects: mark those, or use the other tabs.
            ObjectKind::Group(_) | ObjectKind::Repeat(_) => {
                ui.label(RichText::new(self.t("app-basic-other-tabs")).small());
            }
        }
    }
}
