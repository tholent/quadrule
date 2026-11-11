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

//! The editor's right-hand panel (rung 3): the inspector's tabs on top (bbq Q8, Defaulted
//! "Inspector fields": Page when nothing is selected; Arrange, Style, Text and Constraints for an
//! object; Lint always), and the layers panel below them (bbq Q9, Defaulted "Layers panel"). chris
//! moved the layers there from the left on 2026-11-10, keeping the left side for a future tools
//! bar.

use crate::editor::{DropOn, Toward, layer_drop_place, object_drop_place};
use crate::view::ViewerUi;
use egui::{RichText, Role, ScrollArea, Sense, WidgetInfo};
use quadrule_i18n::Message;
use quadrule_layout::RhythmPx;
use quadrule_layout::units::{Axis, UnitContext, format_length, parse_length, trim_number};
use quadrule_lint::Severity;
use quadrule_model::edit::Command;
use quadrule_model::{
    DashStyle, Document, Fill, FillRole, HConstraint, LineCap, LineRole, Object, ObjectKind,
    POLYGON_SIDES_MAX, POLYGON_SIDES_MIN, Pattern, RhythmOrigin, RhythmResolution,
    STAR_INNER_DEFAULT, STAR_INNER_MAX, STAR_INNER_MIN, Style, TextAlign, Unit, Uuid, VConstraint,
};

/// The inspector's tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    /// Nothing selected: the page's settings.
    Page,
    /// Position and size.
    Arrange,
    /// Line role or custom stroke; a pattern region's pattern.
    Style,
    /// A text box's text and type.
    Text,
    /// How the object follows other devices' pages (napkin §5.4).
    Constraints,
    /// Lint findings on the device on screen.
    Lint,
}

/// What a double-click in the layers panel is renaming.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Renaming {
    Layer(usize, String),
    Object(Uuid, String),
}

const ROLES: [(LineRole, &str); 7] = [
    (LineRole::Whisper, "app-role-whisper"),
    (LineRole::Guide, "app-role-guide"),
    (LineRole::Structure, "app-role-structure"),
    (LineRole::Emphasis, "app-role-emphasis"),
    (LineRole::DotGrid, "app-role-dotgrid"),
    (LineRole::Text, "app-role-text"),
    (LineRole::Label, "app-role-label"),
];

/// A row of the layers panel as a place to drop.
#[derive(Clone, Copy, Debug)]
enum Drop {
    /// A layer's row, by index.
    Layer(usize),
    /// An object's row: its layer and its index in it.
    Object(usize, usize),
}

/// What a row of the layers panel carries while it is dragged (rung 4 bbq Q6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayerDrag {
    /// An object's row.
    Object(Uuid),
    /// A layer's row, by its index.
    Layer(usize),
}

impl ViewerUi {
    // ------------------------------------------------------------------ layers panel

    pub(crate) fn layers_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(self.t("app-layers"));
        });
        ui.horizontal(|ui| {
            let can = !self.viewer.selection.is_empty() || self.selected_layer.is_some();
            ui.add_enabled_ui(can, |ui| {
                if ui.button(self.t("app-move-up")).clicked() {
                    self.move_selected(Toward::Front);
                }
                if ui.button(self.t("app-move-down")).clicked() {
                    self.move_selected(Toward::Back);
                }
            });
        });
        ui.separator();
        let Some(doc) = self.viewer.doc().cloned() else {
            return;
        };
        let add = ui.input(|i| i.modifiers.shift);
        ScrollArea::vertical()
            .id_salt("layers-scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Top of the stack first, as it is drawn last.
                for (li, layer) in doc.layers.iter().enumerate().rev() {
                    let row = ui.horizontal(|ui| {
                        if let Some(v) =
                            self.eye_and_lock(ui, &layer.name, layer.visible, layer.locked)
                        {
                            let _ = self.viewer.set_layer(li, |p| {
                                p.visible = v.0;
                                p.locked = v.1;
                            });
                        }
                        let selected =
                            self.selected_layer == Some(li) && self.viewer.selection.is_empty();
                        if let Some(name) = self.name_cell(
                            ui,
                            Renaming::Layer(li, String::new()),
                            &layer.name,
                            selected,
                            true,
                            Some(LayerDrag::Layer(li)),
                        ) {
                            match name {
                                NameEvent::Click => {
                                    self.selected_layer = Some(li);
                                    self.viewer.selection.clear();
                                }
                                NameEvent::Renamed(n) => {
                                    let _ = self.viewer.set_layer(li, |p| p.name.clone_from(&n));
                                }
                            }
                        }
                    });
                    self.row_drop(ui, &row.response, &doc, Drop::Layer(li));
                    for (oi, o) in layer.objects.iter().enumerate().rev() {
                        let row = ui.horizontal(|ui| {
                            ui.add_space(18.0);
                            if let Some(v) = self.eye_and_lock(ui, &o.name, o.visible, o.locked) {
                                let _ = self.viewer.set_object_panel(o.id, |obj| {
                                    obj.visible = v.0;
                                    obj.locked = v.1;
                                });
                            }
                            let selected = self.viewer.selection.contains(&o.id);
                            if let Some(ev) = self.name_cell(
                                ui,
                                Renaming::Object(o.id, String::new()),
                                &o.name,
                                selected,
                                false,
                                Some(LayerDrag::Object(o.id)),
                            ) {
                                match ev {
                                    NameEvent::Click if add => self.viewer.toggle(o.id),
                                    NameEvent::Click => {
                                        self.viewer.selection = vec![o.id];
                                        self.selected_layer = Some(li);
                                    }
                                    NameEvent::Renamed(n) => {
                                        let _ = self
                                            .viewer
                                            .set_object_panel(o.id, |obj| obj.name.clone_from(&n));
                                    }
                                }
                            }
                        });
                        self.row_drop(ui, &row.response, &doc, Drop::Object(li, oi));
                    }
                }
            });
    }

    /// A row of the layers panel as a place to drop: draws where the dragged row would land and,
    /// on release, moves it there as one undo step (rung 4 bbq Q6). Objects go among objects, or
    /// to the top of a layer when dropped on its row; layers go among layers.
    fn row_drop(&mut self, ui: &mut egui::Ui, row: &egui::Response, doc: &Document, at: Drop) {
        let Some(payload) = row.dnd_hover_payload::<LayerDrag>() else {
            return;
        };
        let above = ui
            .input(|i| i.pointer.latest_pos())
            .is_some_and(|p| p.y < row.rect.center().y);
        // Where it would go: None when the row is no place for this payload.
        enum Plan {
            Object(Uuid, usize, usize),
            Layer(usize, usize),
        }
        let plan = match (*payload, at) {
            (LayerDrag::Object(id), Drop::Object(layer, index)) => {
                object_drop_place(doc, id, DropOn::Object { layer, index }, above)
                    .map(|(l, i)| Plan::Object(id, l, i))
            }
            (LayerDrag::Object(id), Drop::Layer(layer)) => {
                object_drop_place(doc, id, DropOn::LayerTop(layer), above)
                    .map(|(l, i)| Plan::Object(id, l, i))
            }
            (LayerDrag::Layer(from), Drop::Layer(target)) => {
                Some(Plan::Layer(from, layer_drop_place(from, target, above)))
            }
            (LayerDrag::Layer(_), Drop::Object(..)) => None,
        };
        if plan.is_none() {
            return;
        }
        // The line where it lands: the row's top edge for above, bottom for below; a drop on a
        // layer's own row goes inside it, so that row is boxed instead.
        let accent = ui.visuals().selection.stroke.color;
        let stroke = egui::Stroke::new(2.0, accent);
        match (*payload, at) {
            (LayerDrag::Object(_), Drop::Layer(_)) => {
                ui.painter()
                    .rect_stroke(row.rect, 2.0, stroke, egui::StrokeKind::Inside);
            }
            _ => {
                let y = if above {
                    row.rect.top()
                } else {
                    row.rect.bottom()
                };
                ui.painter().hline(row.rect.x_range(), y, stroke);
            }
        }
        if row.dnd_release_payload::<LayerDrag>().is_some() {
            match plan {
                Some(Plan::Object(id, layer, index)) => {
                    let _ = self.viewer.move_object_to(id, layer, index);
                }
                Some(Plan::Layer(from, to)) => {
                    // The selected layer goes with its row.
                    let moved = self.viewer.move_layer_to(from, to).is_ok();
                    if moved && self.selected_layer == Some(from) {
                        self.selected_layer = Some(to);
                    }
                }
                None => {}
            }
        }
    }

    /// Move up / Move down: the selected objects, or else the selected layer.
    pub(crate) fn move_selected(&mut self, toward: Toward) {
        if !self.viewer.selection.is_empty() {
            let _ = self.viewer.reorder_selection(toward);
        } else if let Some(li) = self.selected_layer {
            let n = self.viewer.doc().map_or(0, |d| d.layers.len());
            if self.viewer.reorder_layer(li, toward).is_ok() {
                self.selected_layer = Some(match toward {
                    Toward::Front => (li + 1).min(n.saturating_sub(1)),
                    Toward::Back => li.saturating_sub(1),
                });
            }
        }
    }

    /// The eye and lock toggles of one row. Returns the new (visible, locked) when one is clicked.
    fn eye_and_lock(
        &self,
        ui: &mut egui::Ui,
        name: &str,
        visible: bool,
        locked: bool,
    ) -> Option<(bool, bool)> {
        let show_name = self
            .strings
            .format(&Message::new("app-show-item").text("name", name));
        let lock_name = self
            .strings
            .format(&Message::new("app-lock-item").text("name", name));
        let eye = ui.add(egui::Button::selectable(visible, "👁").small());
        eye.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, visible, &show_name));
        let eye = eye.on_hover_text(&show_name);
        let lock = ui.add(egui::Button::selectable(locked, "🔒").small());
        lock.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, locked, &lock_name));
        let lock = lock.on_hover_text(&lock_name);
        if eye.clicked() {
            Some((!visible, locked))
        } else if lock.clicked() {
            Some((visible, !locked))
        } else {
            None
        }
    }

    /// A row's name: a selectable label, or a text field while it is being renamed
    /// (double-click; Enter or clicking away keeps the name, Escape drops it).
    fn name_cell(
        &mut self,
        ui: &mut egui::Ui,
        target: Renaming,
        name: &str,
        selected: bool,
        strong: bool,
        drag: Option<LayerDrag>,
    ) -> Option<NameEvent> {
        let same = |r: &Renaming| match (r, &target) {
            (Renaming::Layer(a, _), Renaming::Layer(b, _)) => a == b,
            (Renaming::Object(a, _), Renaming::Object(b, _)) => a == b,
            _ => false,
        };
        if let Some(r) = self.renaming.as_mut().filter(|r| same(r)) {
            let buf = match r {
                Renaming::Layer(_, b) | Renaming::Object(_, b) => b,
            };
            let label = self.strings.get("app-rename");
            let resp = ui.add(egui::TextEdit::singleline(buf).desired_width(150.0));
            resp.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, &label));
            resp.request_focus();
            let text = buf.clone();
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.renaming = None;
            } else if resp.lost_focus() {
                self.renaming = None;
                if !text.trim().is_empty() && text != name {
                    return Some(NameEvent::Renamed(text));
                }
            }
            return None;
        }
        let text = if strong {
            RichText::new(name).strong()
        } else {
            RichText::new(name)
        };
        // A row can be dragged to a new place (rung 4 bbq Q6) as well as clicked.
        let resp = ui.add(egui::Button::selectable(selected, text).sense(Sense::click_and_drag()));
        if let Some(d) = drag {
            resp.dnd_set_drag_payload(d);
        }
        if resp.double_clicked() {
            self.renaming = Some(match target {
                Renaming::Layer(i, _) => Renaming::Layer(i, name.to_string()),
                Renaming::Object(id, _) => Renaming::Object(id, name.to_string()),
            });
            None
        } else if resp.clicked() {
            Some(NameEvent::Click)
        } else {
            None
        }
    }

    // ------------------------------------------------------------------ inspector

    pub(crate) fn inspector(&mut self, ui: &mut egui::Ui) {
        let has_selection = !self.viewer.selection.is_empty();
        let text_selected = self
            .first_selected()
            .is_some_and(|o| matches!(o.kind, ObjectKind::Text(_)));
        let mut tabs = if has_selection {
            vec![
                (Tab::Arrange, self.t("app-tab-arrange")),
                (Tab::Style, self.t("app-tab-style")),
            ]
        } else {
            vec![(Tab::Page, self.t("app-tab-page"))]
        };
        if text_selected {
            tabs.push((Tab::Text, self.t("app-tab-text")));
        }
        if has_selection {
            tabs.push((Tab::Constraints, self.t("app-tab-constraints")));
        }
        let count = self.lint_count();
        tabs.push((
            Tab::Lint,
            self.strings
                .format(&Message::new("app-lint-heading").num("count", count as f64, 0)),
        ));
        // A tab that does not apply now falls back to the first one.
        if !tabs.iter().any(|(t, _)| *t == self.tab) {
            self.tab = tabs[0].0;
        }
        ui.horizontal_wrapped(|ui| {
            for (t, label) in &tabs {
                if ui
                    .add(egui::Button::selectable(self.tab == *t, label))
                    .clicked()
                {
                    self.tab = *t;
                }
            }
        });
        ui.separator();
        if self.tab != Tab::Lint && self.viewer.profile_id != self.primary_id() {
            let device = self.device_name(&self.viewer.profile_id.clone());
            ui.label(
                RichText::new(
                    self.strings
                        .format(&Message::new("app-every-device").text("device", device)),
                )
                .small(),
            );
        }
        if has_selection && self.tab != Tab::Lint {
            let n = self.viewer.selection.len();
            ui.label(
                self.strings
                    .format(&Message::new("app-selected-count").num("count", n as f64, 0)),
            );
            if self.viewer.editable_selection().is_empty() {
                ui.colored_label(ui.visuals().warn_fg_color, self.t("app-selection-locked"));
            }
            ui.add_space(4.0);
        }
        ScrollArea::vertical()
            .id_salt("inspector-scroll")
            .show(ui, |ui| match self.tab {
                Tab::Page => self.page_tab(ui),
                Tab::Arrange => self.arrange_tab(ui),
                Tab::Style => self.style_tab(ui),
                Tab::Text => self.text_tab(ui),
                Tab::Constraints => self.constraints_tab(ui),
                Tab::Lint => self.lint_tab(ui),
            });
    }

    fn primary_id(&self) -> String {
        self.viewer
            .doc()
            .map_or_else(String::new, |d| d.primary_profile_id.clone())
    }

    pub(crate) fn device_name(&self, id: &str) -> String {
        quadrule_profiles::builtin(id)
            .map(|p| p.display_name)
            .unwrap_or_else(|_| id.to_string())
    }

    fn first_selected(&self) -> Option<Object> {
        let id = *self.viewer.selection.first()?;
        self.viewer.doc()?.object(id).cloned()
    }

    fn page_tab(&mut self, ui: &mut egui::Ui) {
        let Some(doc) = self.viewer.doc().cloned() else {
            return;
        };
        let (Some(cx), Some(cy)) = (
            self.viewer.unit_context(Axis::X),
            self.viewer.unit_context(Axis::Y),
        ) else {
            return;
        };
        egui::Grid::new("page-grid").num_columns(2).show(ui, |ui| {
            // Title.
            let label = ui.label(self.t("app-page-title"));
            let mut title = doc.metadata.title.clone();
            if ui
                .add(egui::TextEdit::singleline(&mut title).desired_width(180.0))
                .labelled_by(label.id)
                .changed()
            {
                let mut m = doc.metadata.clone();
                m.title = title;
                let _ = self
                    .viewer
                    .edit(Command::SetMetadata(m), Some("page-title"));
            }
            ui.end_row();

            // Display unit.
            let label = ui.label(self.t("app-page-units"));
            let mut unit = doc.units;
            egui::ComboBox::from_id_salt("units")
                .selected_text(unit_symbol(unit))
                .show_ui(ui, |ui| {
                    for u in [Unit::Mm, Unit::Pt, Unit::Px, Unit::In] {
                        ui.selectable_value(&mut unit, u, unit_symbol(u));
                    }
                })
                .response
                .labelled_by(label.id);
            if unit != doc.units {
                let _ = self.viewer.edit(Command::SetUnits(unit), None);
            }
            ui.end_row();

            // Background.
            let label = ui.label(self.t("app-page-background"));
            let mut gray = doc.page.background_gray;
            if ui
                .add(egui::DragValue::new(&mut gray).range(0..=255))
                .labelled_by(label.id)
                .changed()
            {
                let mut p = doc.page.clone();
                p.background_gray = gray;
                let _ = self
                    .viewer
                    .edit(Command::SetPage(p), Some("page-background"));
            }
            ui.end_row();
        });

        // Margins, linked by default.
        ui.add_space(6.0);
        ui.strong(self.t("app-page-margins"));
        let link = self.t("app-link-margins");
        ui.checkbox(&mut self.link_margins, link);
        let m = doc.page.margins_mm;
        egui::Grid::new("margins-grid")
            .num_columns(2)
            .show(ui, |ui| {
                let sides = [
                    ("app-margin-top", m.top, cy, 0),
                    ("app-margin-right", m.right, cx, 1),
                    ("app-margin-bottom", m.bottom, cy, 2),
                    ("app-margin-left", m.left, cx, 3),
                ];
                for (id, v, ctx, side) in sides {
                    if let Some(mm) =
                        self.length_row(ui, &format!("margin-{side}"), id, v, &ctx, true)
                    {
                        let mut p = doc.page.clone();
                        let mm = mm.max(0.0);
                        if self.link_margins {
                            p.margins_mm = quadrule_model::Margins::all(mm);
                        } else {
                            match side {
                                0 => p.margins_mm.top = mm,
                                1 => p.margins_mm.right = mm,
                                2 => p.margins_mm.bottom = mm,
                                _ => p.margins_mm.left = mm,
                            }
                        }
                        let _ = self
                            .viewer
                            .edit(Command::SetPage(p), Some(&format!("margin-{side}")));
                    }
                    ui.end_row();
                }
            });

        // Rhythm.
        ui.add_space(6.0);
        ui.strong(self.t("app-rhythm"));
        let link = self.t("app-link-rhythm");
        ui.checkbox(&mut self.link_rhythm, link);
        let r = doc.rhythm.clone();
        // Rhythm units are lengths, but `u` would be circular here: read them without it.
        let plain = |c: UnitContext| UnitContext {
            rhythm_mm: f64::NAN,
            ..c
        };
        egui::Grid::new("rhythm-grid")
            .num_columns(2)
            .show(ui, |ui| {
                for (key, id, v, ctx, across) in [
                    (
                        "rhythm-h",
                        "app-rhythm-across",
                        r.h_unit_mm,
                        plain(cx),
                        true,
                    ),
                    ("rhythm-v", "app-rhythm-down", r.v_unit_mm, plain(cy), false),
                ] {
                    if let Some(mm) = self.length_row(ui, key, id, v, &ctx, true)
                        && mm.is_finite()
                        && mm > 0.0
                    {
                        let mut nr = r.clone();
                        if self.link_rhythm || across {
                            nr.h_unit_mm = mm;
                        }
                        if self.link_rhythm || !across {
                            nr.v_unit_mm = mm;
                        }
                        let _ = self.viewer.edit(Command::SetRhythm(nr), Some(key));
                    }
                    ui.end_row();
                }

                let label = ui.label(self.t("app-rhythm-origin"));
                let mut origin = r.origin;
                let origins = [
                    (RhythmOrigin::MarginCorner, "app-origin-margin"),
                    (RhythmOrigin::PageCorner, "app-origin-page"),
                    (RhythmOrigin::Centered, "app-origin-centered"),
                ];
                self.combo(ui, "origin", label.id, &mut origin, &origins);
                if origin != r.origin {
                    let mut nr = r.clone();
                    nr.origin = origin;
                    let _ = self.viewer.edit(Command::SetRhythm(nr), None);
                }
                ui.end_row();

                let label = ui.label(self.t("app-rhythm-resolution"));
                let mut res = r.resolution;
                let options = [
                    (RhythmResolution::PixelLocked, "app-res-pixel"),
                    (RhythmResolution::PhysicalLocked, "app-res-physical"),
                ];
                self.combo(ui, "resolution", label.id, &mut res, &options);
                if res != r.resolution {
                    let mut nr = r.clone();
                    nr.resolution = res;
                    let _ = self.viewer.edit(Command::SetRhythm(nr), None);
                }
                ui.end_row();

                // Subdivisions: finer steps for snapping and Shift+arrow (napkin §6.3).
                let label = ui.label(self.t("app-rhythm-subdivisions"));
                let mut sub = r.subdivisions_on();
                let name = |n: Option<u8>| match n {
                    None => self.t("app-subdivisions-off"),
                    Some(n) => self.strings.format(&Message::new("app-subdivisions-n").num(
                        "n",
                        f64::from(n),
                        0,
                    )),
                };
                egui::ComboBox::from_id_salt("subdivisions")
                    .selected_text(name(sub))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut sub, None, name(None));
                        for n in quadrule_model::SUBDIVISIONS_MIN..=quadrule_model::SUBDIVISIONS_MAX
                        {
                            ui.selectable_value(&mut sub, Some(n), name(Some(n)));
                        }
                    })
                    .response
                    .labelled_by(label.id);
                if sub != r.subdivisions_on() {
                    let mut nr = r.clone();
                    nr.subdivisions = sub;
                    let _ = self.viewer.edit(Command::SetRhythm(nr), None);
                }
                ui.end_row();
            });
        // The unit as each device resolves it (napkin §6.3: "5 mm → 59 px (Manta)").
        for id in doc.profile_ids() {
            if let Ok(p) = quadrule_profiles::builtin(id) {
                let rp = RhythmPx::new(&doc, &p);
                let px = if rp.unit_x == rp.unit_y {
                    trim_number(rp.unit_x, 2)
                } else {
                    format!(
                        "{} × {}",
                        trim_number(rp.unit_x, 2),
                        trim_number(rp.unit_y, 2)
                    )
                };
                ui.label(
                    RichText::new(
                        self.strings.format(
                            &Message::new("app-rhythm-on-device")
                                .text("px", px)
                                .text("device", p.display_name.clone()),
                        ),
                    )
                    .small(),
                );
            }
        }
        ui.add_space(6.0);
        let label = self.t("app-safe-zones");
        ui.checkbox(&mut self.viewer.show_safe_zones, label);

        // The helpers that catch a dragged object, each on or off (napkin §6.2). Holding Ctrl
        // (Cmd on macOS) while dragging turns them all off for that drag.
        ui.add_space(6.0);
        ui.strong(self.t("app-snapping"));
        self.snap_toggles(ui);
    }

    fn arrange_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.first_selected() else {
            return;
        };
        let (Some(cx), Some(cy)) = (
            self.viewer.unit_context(Axis::X),
            self.viewer.unit_context(Axis::Y),
        ) else {
            return;
        };
        let enabled = !self.viewer.editable_selection().is_empty();
        let f = o.frame;
        egui::Grid::new("arrange-grid")
            .num_columns(2)
            .show(ui, |ui| {
                type Set = fn(&mut Object, f64);
                let rows: [(&str, &str, f64, &UnitContext, Set); 4] = [
                    ("arrange-x", "app-x", f.x_mm, &cx, |o, v| o.frame.x_mm = v),
                    ("arrange-y", "app-y", f.y_mm, &cy, |o, v| o.frame.y_mm = v),
                    ("arrange-w", "app-width", f.w_mm, &cx, |o, v| {
                        o.frame.w_mm = v.max(0.0)
                    }),
                    ("arrange-h", "app-height", f.h_mm, &cy, |o, v| {
                        o.frame.h_mm = v.max(0.0)
                    }),
                ];
                for (key, id, v, ctx, set) in rows {
                    if let Some(mm) = self.length_row(ui, key, id, v, ctx, enabled) {
                        let _ = self.viewer.edit_selected(Some(key), |o| set(o, mm));
                    }
                    ui.end_row();
                }
            });
    }

    fn style_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.first_selected() else {
            return;
        };
        let Some(cx) = self.viewer.unit_context(Axis::X) else {
            return;
        };
        let enabled = !self.viewer.editable_selection().is_empty();
        let shape = o.kind.is_shape();
        // Dash and cap are for anything drawn as a stroke: lines and the shapes.
        let stroked = shape || matches!(o.kind, ObjectKind::Line(_));
        ui.add_enabled_ui(enabled, |ui| {
            egui::Grid::new("style-grid").num_columns(2).show(ui, |ui| {
                // Stroke: None (shapes only), a role or custom values.
                let label = ui.label(self.t(if shape {
                    "app-style-stroke"
                } else {
                    "app-style-kind"
                }));
                let was = StrokeChoice::of(o.style);
                let mut want = was;
                ui.horizontal(|ui| {
                    if shape {
                        ui.radio_value(&mut want, StrokeChoice::None, self.t("app-stroke-none"));
                    }
                    ui.radio_value(&mut want, StrokeChoice::Role, self.t("app-style-role"));
                    ui.radio_value(&mut want, StrokeChoice::Custom, self.t("app-style-custom"));
                })
                .response
                .labelled_by(label.id);
                if want != was {
                    let device = quadrule_profiles::builtin(&self.viewer.profile_id).ok();
                    // Start from what the role draws on the device on screen.
                    let from_role = |r: LineRole| {
                        let (w, g) = device.as_ref().map_or((1, 0), |p| {
                            let s = p.role(r);
                            (s.width_px.max(1), s.gray)
                        });
                        Style::Custom {
                            width_mm: w as f64 / cx.px_per_mm,
                            gray: g,
                        }
                    };
                    let style = match (want, o.style) {
                        (StrokeChoice::None, _) => Style::None,
                        (StrokeChoice::Custom, Style::Role(r)) => from_role(r),
                        (StrokeChoice::Custom, _) => from_role(default_role(&o)),
                        (StrokeChoice::Role, _) => Style::Role(default_role(&o)),
                    };
                    let _ = self.viewer.edit_selected(None, |x| {
                        if style != Style::None || x.kind.is_shape() {
                            x.style = style;
                        }
                    });
                }
                ui.end_row();

                match o.style {
                    Style::None => {}
                    Style::Role(role) => {
                        let label = ui.label(self.t("app-style-role"));
                        let mut r = role;
                        self.combo(ui, "role", label.id, &mut r, &ROLES);
                        if r != role {
                            let _ = self
                                .viewer
                                .edit_selected(None, |x| x.style = Style::Role(r));
                        }
                        ui.end_row();
                    }
                    Style::Custom { width_mm, gray } => {
                        if let Some(mm) = self.length_row(
                            ui,
                            "style-width",
                            "app-stroke-width",
                            width_mm,
                            &cx,
                            enabled,
                        ) {
                            let _ = self.viewer.edit_selected(Some("style-width"), |x| {
                                if let Style::Custom { width_mm, .. } = &mut x.style {
                                    *width_mm = mm.max(0.0);
                                }
                            });
                        }
                        ui.end_row();
                        let label = ui.label(self.t("app-stroke-gray"));
                        let mut g = gray;
                        if ui
                            .add(egui::DragValue::new(&mut g).range(0..=255))
                            .labelled_by(label.id)
                            .changed()
                        {
                            let _ = self.viewer.edit_selected(Some("style-gray"), |x| {
                                if let Style::Custom { gray, .. } = &mut x.style {
                                    *gray = g;
                                }
                            });
                        }
                        ui.end_row();
                    }
                }

                if shape {
                    self.fill_rows(ui, &o, enabled);
                }

                if stroked && o.style != Style::None {
                    self.dash_rows(ui, &o, &cx, enabled);
                }

                if let ObjectKind::Pattern(pr) = &o.kind {
                    let label = ui.label(self.t("app-pattern"));
                    let mut p = pr.pattern;
                    let options = [
                        (Pattern::Lines, "app-pattern-lines"),
                        (Pattern::Dots, "app-pattern-dots"),
                        (Pattern::Grid, "app-pattern-grid"),
                    ];
                    self.combo(ui, "pattern", label.id, &mut p, &options);
                    if p != pr.pattern {
                        let _ = self.viewer.edit_selected(None, |x| {
                            if let ObjectKind::Pattern(pr) = &mut x.kind {
                                pr.pattern = p;
                            }
                        });
                    }
                    ui.end_row();
                }
            });
            if shape {
                ui.add_space(6.0);
                ui.strong(self.t("app-shape"));
                self.shape_rows(ui, &o, &cx, enabled);
            }
        });
    }

    /// The fill: none, a role (each device gives it a gray) or a gray of its own.
    fn fill_rows(&mut self, ui: &mut egui::Ui, o: &Object, _enabled: bool) {
        let label = ui.label(self.t("app-fill"));
        let was = FillChoice::of(o.fill);
        let mut want = was;
        let options = [
            (FillChoice::None, "app-fill-none"),
            (FillChoice::Shade, "app-fill-shade"),
            (FillChoice::Block, "app-fill-block"),
            (FillChoice::Custom, "app-fill-custom"),
        ];
        self.combo(ui, "fill", label.id, &mut want, &options);
        if want != was {
            // A custom fill starts from the gray the role gave on the device on screen.
            let device = quadrule_profiles::builtin(&self.viewer.profile_id).ok();
            let start = match (o.fill, &device) {
                (Some(f), Some(p)) => p.fill_gray(f),
                _ => 204,
            };
            let fill = match want {
                FillChoice::None => None,
                FillChoice::Shade => Some(Fill::Role(FillRole::Shade)),
                FillChoice::Block => Some(Fill::Role(FillRole::Block)),
                FillChoice::Custom => Some(Fill::Gray(snap_gray(start))),
            };
            let _ = self.viewer.edit_selected(None, |x| {
                if x.kind.is_shape() {
                    x.fill = fill;
                }
            });
        }
        ui.end_row();
        if let Some(Fill::Gray(gray)) = o.fill {
            let label = ui.label(self.t("app-fill-gray"));
            let mut g = gray;
            // Steps of 17 (the sixteen levels 0, 17 ... 255) until the clean grays are measured.
            if ui
                .add(egui::Slider::new(&mut g, 0..=255).step_by(f64::from(GRAY_STEP)))
                .labelled_by(label.id)
                .changed()
            {
                let _ = self.viewer.edit_selected(Some("fill-gray"), |x| {
                    if x.kind.is_shape() {
                        x.fill = Some(Fill::Gray(g));
                    }
                });
            }
            ui.end_row();
        }
    }

    /// The dash and the cap.
    fn dash_rows(&mut self, ui: &mut egui::Ui, o: &Object, cx: &UnitContext, enabled: bool) {
        let label = ui.label(self.t("app-dash"));
        let was = DashChoice::of(o.dash);
        let mut want = was;
        let options = [
            (DashChoice::Solid, "app-dash-solid"),
            (DashChoice::Dotted, "app-dash-dotted"),
            (DashChoice::Short, "app-dash-short"),
            (DashChoice::Dash, "app-dash-dash"),
            (DashChoice::Long, "app-dash-long"),
            (DashChoice::Custom, "app-dash-custom"),
        ];
        self.combo(ui, "dash", label.id, &mut want, &options);
        if want != was {
            // A custom dash starts from the preset's own lengths.
            let (dash_mm, gap_mm) = o.dash.lengths_mm().unwrap_or((1.0, 0.5));
            let style = match want {
                DashChoice::Solid => DashStyle::Solid,
                DashChoice::Dotted => DashStyle::Dotted,
                DashChoice::Short => DashStyle::Short,
                DashChoice::Dash => DashStyle::Dash,
                DashChoice::Long => DashStyle::Long,
                DashChoice::Custom => DashStyle::Custom { dash_mm, gap_mm },
            };
            let _ = self.viewer.edit_selected(None, |x| {
                if x.kind.is_shape() || matches!(x.kind, ObjectKind::Line(_)) {
                    x.dash = style;
                }
            });
        }
        ui.end_row();
        if let DashStyle::Custom { dash_mm, gap_mm } = o.dash {
            if let Some(mm) =
                self.length_row(ui, "dash-length", "app-dash-length", dash_mm, cx, enabled)
            {
                let _ = self.viewer.edit_selected(Some("dash-length"), |x| {
                    if let DashStyle::Custom { dash_mm, .. } = &mut x.dash {
                        *dash_mm = mm.max(0.0);
                    }
                });
            }
            ui.end_row();
            if let Some(mm) =
                self.length_row(ui, "gap-length", "app-gap-length", gap_mm, cx, enabled)
            {
                let _ = self.viewer.edit_selected(Some("gap-length"), |x| {
                    if let DashStyle::Custom { gap_mm, .. } = &mut x.dash {
                        *gap_mm = mm.max(0.0);
                    }
                });
            }
            ui.end_row();
        }
        let label = ui.label(self.t("app-cap"));
        let mut cap = o.cap;
        let options = [
            (LineCap::Butt, "app-cap-butt"),
            (LineCap::Round, "app-cap-round"),
            (LineCap::Square, "app-cap-square"),
        ];
        self.combo(ui, "cap", label.id, &mut cap, &options);
        if cap != o.cap {
            let _ = self.viewer.edit_selected(None, |x| {
                if x.kind.is_shape() || matches!(x.kind, ObjectKind::Line(_)) {
                    x.cap = cap;
                }
            });
        }
        ui.end_row();
    }

    /// The Shape section: a rectangle's corner radius; a polygon's sides, star and inner radius.
    fn shape_rows(&mut self, ui: &mut egui::Ui, o: &Object, cx: &UnitContext, enabled: bool) {
        egui::Grid::new("shape-grid")
            .num_columns(2)
            .show(ui, |ui| match &o.kind {
                ObjectKind::Rect(r) => {
                    if let Some(mm) = self.length_row(
                        ui,
                        "shape-radius",
                        "app-corner-radius",
                        r.corner_radius_mm,
                        cx,
                        enabled,
                    ) {
                        let _ = self.viewer.edit_selected(Some("shape-radius"), |x| {
                            if let ObjectKind::Rect(r) = &mut x.kind {
                                r.corner_radius_mm = mm.max(0.0);
                            }
                        });
                    }
                    ui.end_row();
                }
                ObjectKind::Polygon(pg) => {
                    let label = ui.label(self.t("app-sides"));
                    let mut sides = pg.sides_clamped();
                    if ui
                        .add(
                            egui::DragValue::new(&mut sides)
                                .range(POLYGON_SIDES_MIN..=POLYGON_SIDES_MAX),
                        )
                        .labelled_by(label.id)
                        .changed()
                    {
                        let _ = self.viewer.edit_selected(Some("shape-sides"), |x| {
                            if let ObjectKind::Polygon(pg) = &mut x.kind {
                                pg.sides = sides;
                            }
                        });
                    }
                    ui.end_row();
                    let label = ui.label(self.t("app-star"));
                    let mut star = pg.star.is_some();
                    if ui.checkbox(&mut star, "").labelled_by(label.id).changed() {
                        let _ = self.viewer.edit_selected(None, |x| {
                            if let ObjectKind::Polygon(pg) = &mut x.kind {
                                pg.star = star.then_some(STAR_INNER_DEFAULT);
                            }
                        });
                    }
                    ui.end_row();
                    if let Some(inner) = pg.star_clamped() {
                        let label = ui.label(self.t("app-star-inner"));
                        let mut r = inner;
                        if ui
                            .add(
                                egui::DragValue::new(&mut r)
                                    .range(STAR_INNER_MIN..=STAR_INNER_MAX)
                                    .speed(0.01)
                                    .fixed_decimals(2),
                            )
                            .labelled_by(label.id)
                            .changed()
                        {
                            let _ = self.viewer.edit_selected(Some("shape-inner"), |x| {
                                if let ObjectKind::Polygon(pg) = &mut x.kind {
                                    pg.star = Some(r);
                                }
                            });
                        }
                        ui.end_row();
                    }
                }
                _ => {}
            });
    }

    fn text_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.first_selected() else {
            return;
        };
        let ObjectKind::Text(t) = &o.kind else {
            return;
        };
        let Some(cx) = self.viewer.unit_context(Axis::Y) else {
            return;
        };
        let enabled = !self.viewer.editable_selection().is_empty();
        let edit_text =
            |v: &mut crate::viewer::Viewer, key: &str, f: &dyn Fn(&mut quadrule_model::TextBox)| {
                let _ = v.edit_selected(Some(key), |x| {
                    if let ObjectKind::Text(t) = &mut x.kind {
                        f(t);
                    }
                });
            };
        ui.add_enabled_ui(enabled, |ui| {
            let label = ui.label(self.t("app-text-content"));
            let mut content = t.content.clone();
            let out = egui::TextEdit::multiline(&mut content)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .show(ui);
            let field = egui::Response::clone(&out.response).labelled_by(label.id);
            // A text box was just drawn: the cursor goes in with all its text selected, so typing
            // replaces it (rung 5 bbq, Defaulted "Per kind").
            if std::mem::take(&mut self.focus_text_content) {
                field.request_focus();
                let mut state = out.state;
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(0),
                        egui::text::CCursor::new(content.chars().count()),
                    )));
                state.store(ui.ctx(), field.id);
            }
            if field.changed() {
                edit_text(&mut self.viewer, "text-content", &|t| {
                    t.content.clone_from(&content)
                });
            }
            ui.add_space(4.0);
            egui::Grid::new("text-grid").num_columns(2).show(ui, |ui| {
                // Font: the families this build can lay out (bundled ones), and the one in use.
                let label = ui.label(self.t("app-font"));
                let mut fam = t.family.clone();
                let mut families: Vec<String> = quadrule_text::BUNDLED
                    .iter()
                    .map(|b| b.family.to_string())
                    .collect();
                if !families.contains(&t.family) {
                    families.push(t.family.clone());
                }
                egui::ComboBox::from_id_salt("family")
                    .selected_text(&fam)
                    .show_ui(ui, |ui| {
                        for f in &families {
                            ui.selectable_value(&mut fam, f.clone(), f);
                        }
                    })
                    .response
                    .labelled_by(label.id);
                if fam != t.family {
                    edit_text(&mut self.viewer, "text-family", &|t| {
                        t.family.clone_from(&fam)
                    });
                }
                ui.end_row();

                let label = ui.label(self.t("app-weight"));
                let mut w = t.weight;
                if ui
                    .add(egui::DragValue::new(&mut w).range(100..=900).speed(5))
                    .labelled_by(label.id)
                    .changed()
                {
                    edit_text(&mut self.viewer, "text-weight", &|t| t.weight = w);
                }
                ui.end_row();

                // Size is in points, whatever the display unit; any unit can be typed.
                let pt = UnitContext {
                    display: Unit::Pt,
                    ..cx
                };
                let size_mm = quadrule_model::pt_to_mm(t.size_pt);
                if let Some(mm) =
                    self.length_row(ui, "text-size", "app-size", size_mm, &pt, enabled)
                    && mm > 0.0
                {
                    let size = mm / quadrule_model::pt_to_mm(1.0);
                    edit_text(&mut self.viewer, "text-size", &|t| t.size_pt = size);
                }
                ui.end_row();

                let label = ui.label(self.t("app-line-height"));
                let mut lh = t.line_height;
                if ui
                    .add(egui::DragValue::new(&mut lh).range(0.5..=4.0).speed(0.01))
                    .labelled_by(label.id)
                    .changed()
                {
                    edit_text(&mut self.viewer, "text-line-height", &|t| {
                        t.line_height = lh
                    });
                }
                ui.end_row();

                let label = ui.label(self.t("app-letter-spacing"));
                let mut ls = t.letter_spacing_em;
                if ui
                    .add(egui::DragValue::new(&mut ls).range(-0.5..=1.0).speed(0.005))
                    .labelled_by(label.id)
                    .changed()
                {
                    edit_text(&mut self.viewer, "text-letter-spacing", &|t| {
                        t.letter_spacing_em = ls
                    });
                }
                ui.end_row();

                let label = ui.label(self.t("app-align"));
                let mut a = t.align;
                let options = [
                    (TextAlign::Left, "app-align-left"),
                    (TextAlign::Center, "app-align-center"),
                    (TextAlign::Right, "app-align-right"),
                    (TextAlign::Justify, "app-align-justify"),
                ];
                self.combo(ui, "align", label.id, &mut a, &options);
                if a != t.align {
                    edit_text(&mut self.viewer, "text-align", &|t| t.align = a);
                }
                ui.end_row();
            });
        });
    }

    fn constraints_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.first_selected() else {
            return;
        };
        let enabled = !self.viewer.editable_selection().is_empty();
        ui.add_enabled_ui(enabled, |ui| {
            egui::Grid::new("constraints-grid")
                .num_columns(2)
                .show(ui, |ui| {
                    let label = ui.label(self.t("app-constraint-h"));
                    let mut h = o.constraints.h;
                    let hs = [
                        (HConstraint::Left, "app-c-left"),
                        (HConstraint::Right, "app-c-right"),
                        (HConstraint::LeftRight, "app-c-left-right"),
                        (HConstraint::Center, "app-c-center"),
                        (HConstraint::Scale, "app-c-scale"),
                    ];
                    self.combo(ui, "constraint-h", label.id, &mut h, &hs);
                    if h != o.constraints.h {
                        let _ = self.viewer.edit_selected(None, |x| x.constraints.h = h);
                    }
                    ui.end_row();
                    let label = ui.label(self.t("app-constraint-v"));
                    let mut v = o.constraints.v;
                    let vs = [
                        (VConstraint::Top, "app-c-top"),
                        (VConstraint::Bottom, "app-c-bottom"),
                        (VConstraint::TopBottom, "app-c-top-bottom"),
                        (VConstraint::Center, "app-c-center"),
                        (VConstraint::Scale, "app-c-scale"),
                    ];
                    self.combo(ui, "constraint-v", label.id, &mut v, &vs);
                    if v != o.constraints.v {
                        let _ = self.viewer.edit_selected(None, |x| x.constraints.v = v);
                    }
                    ui.end_row();
                });
        });
    }

    fn lint_tab(&mut self, ui: &mut egui::Ui) {
        let strings = &self.strings;
        let Some(page) = self.viewer.page() else {
            return;
        };
        let findings = page.findings.clone();
        let saved = page.saved_outlines;
        ui.label(strings.get("app-lint-toolbars"));
        if saved {
            ui.label(strings.get("app-saved-outlines"));
        }
        ui.separator();
        if findings.is_empty() {
            ui.label(strings.get("app-lint-none"));
        }
        for (i, f) in findings.iter().enumerate() {
            let icon = match f.severity {
                Severity::Warn => "⚠",
                Severity::Info => "ℹ",
            };
            let text = format!("{icon} {}  {}", f.rule, strings.format(&f.message));
            let selected = self.viewer.selected == Some(i);
            let r = ui.add(egui::Button::selectable(selected, text).wrap());
            if r.clicked() {
                self.viewer.selected = if selected { None } else { Some(i) };
                self.scroll_to_selected = self.viewer.selected.is_some();
            }
        }
    }

    // ------------------------------------------------------------------ fields

    /// A labelled length field (rung 3 bbq Q10): shows `mm` in the display unit; while focused it
    /// keeps what is typed, and returns the value in mm each time the text reads as a length.
    /// Text that does not read is outlined in red and changes nothing.
    pub(crate) fn length_row(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        label_id: &str,
        mm: f64,
        ctx: &UnitContext,
        enabled: bool,
    ) -> Option<f64> {
        let label = ui.label(self.t(label_id));
        let id = ui.make_persistent_id(("length", key));
        let focused = ui.memory(|m| m.has_focus(id));
        let mut text = match (focused, self.field_text.get(key)) {
            (true, Some(t)) => t.clone(),
            _ => format_length(mm, ctx),
        };
        let parsed = parse_length(&text, ctx);
        let bad = focused && parsed.is_err();
        let mut edit = egui::TextEdit::singleline(&mut text)
            .id(id)
            .desired_width(110.0);
        if bad {
            edit = edit.background_color(ui.visuals().error_fg_color.gamma_multiply(0.25));
        }
        let resp = ui.add_enabled(enabled, edit).labelled_by(label.id);
        let resp = if bad {
            resp.on_hover_text(self.t("app-field-invalid"))
        } else {
            resp
        };
        if resp.lost_focus() || !resp.has_focus() {
            self.field_text.remove(key);
        }
        if resp.changed() {
            self.field_text.insert(key.to_string(), text.clone());
            if let Ok(v) = parse_length(&text, ctx)
                && v.is_finite()
            {
                return Some(v);
            }
        }
        None
    }

    /// A labelled choice from `options` (value, message id).
    fn combo<T: Copy + PartialEq>(
        &self,
        ui: &mut egui::Ui,
        salt: &str,
        label: egui::Id,
        value: &mut T,
        options: &[(T, &str)],
    ) {
        let shown = options
            .iter()
            .find(|(v, _)| v == value)
            .map(|(_, id)| self.t(id))
            .unwrap_or_default();
        egui::ComboBox::from_id_salt(salt)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for (v, id) in options {
                    ui.selectable_value(value, *v, self.t(id));
                }
            })
            .response
            .labelled_by(label);
    }
}

/// What the stroke of a shape is set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StrokeChoice {
    None,
    Role,
    Custom,
}

impl StrokeChoice {
    fn of(style: Style) -> Self {
        match style {
            Style::None => StrokeChoice::None,
            Style::Role(_) => StrokeChoice::Role,
            Style::Custom { .. } => StrokeChoice::Custom,
        }
    }
}

/// What the fill is set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FillChoice {
    None,
    Shade,
    Block,
    Custom,
}

impl FillChoice {
    fn of(fill: Option<Fill>) -> Self {
        match fill {
            None => FillChoice::None,
            Some(Fill::Role(FillRole::Shade)) => FillChoice::Shade,
            Some(Fill::Role(FillRole::Block)) => FillChoice::Block,
            Some(Fill::Gray(_)) => FillChoice::Custom,
        }
    }
}

/// What the dash is set to: the presets and Custom, which carries its two lengths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DashChoice {
    Solid,
    Dotted,
    Short,
    Dash,
    Long,
    Custom,
}

impl DashChoice {
    fn of(d: DashStyle) -> Self {
        match d {
            DashStyle::Solid => DashChoice::Solid,
            DashStyle::Dotted => DashChoice::Dotted,
            DashStyle::Short => DashChoice::Short,
            DashStyle::Dash => DashChoice::Dash,
            DashStyle::Long => DashChoice::Long,
            DashStyle::Custom { .. } => DashChoice::Custom,
        }
    }
}

/// The step of a custom fill gray: the sixteen levels 0, 17 ... 255 (rung 6 bbq, Defaulted
/// "Inspector"), until the clean grays are measured.
pub const GRAY_STEP: u8 = 17;

/// The step of 17 nearest `g`.
fn snap_gray(g: u8) -> u8 {
    (u16::from(g) + u16::from(GRAY_STEP) / 2) as u8 / GRAY_STEP * GRAY_STEP
}

/// What happened to a row's name.
enum NameEvent {
    Click,
    Renamed(String),
}

fn unit_symbol(u: Unit) -> &'static str {
    match u {
        Unit::Mm => "mm",
        Unit::Pt => "pt",
        Unit::Px => "px",
        Unit::In => "in",
    }
}

/// The role an object goes back to when Custom is switched off: Text for text, Guide for lines,
/// Dot grid for a dot pattern, Structure for a shape.
fn default_role(o: &Object) -> LineRole {
    match &o.kind {
        ObjectKind::Text(_) => LineRole::Text,
        ObjectKind::Pattern(p) if p.pattern == Pattern::Dots => LineRole::DotGrid,
        ObjectKind::Rect(_) | ObjectKind::Ellipse(_) | ObjectKind::Polygon(_) => {
            LineRole::Structure
        }
        _ => LineRole::Guide,
    }
}
