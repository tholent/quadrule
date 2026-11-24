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

use crate::arrange::ArrangeOp;
use crate::editor::{DropOn, RowZone, Toward, layer_drop_place, object_drop_place, row_drop_place};
use crate::layer_icons::LayerIcon;
use crate::view::ViewerUi;
use egui::{Color32, RichText, Role, ScrollArea, Sense, WidgetInfo};
use quadrule_i18n::Message;
use quadrule_layout::RhythmPx;
use quadrule_layout::align::AlignTo;
use quadrule_layout::units::{Axis, UnitContext, format_length, parse_length, trim_number};
use quadrule_model::edit::{Command, OverrideField, Place, reframe};
use quadrule_model::{
    DashStyle, Document, Fill, FillRole, Frame, HConstraint, HexOrientation, IsoStyle, LineCap,
    LineRole, MajorFrom, MajorLines, Object, ObjectKind, Orientation, POLYGON_SIDES_MAX,
    POLYGON_SIDES_MIN, Pattern, PatternKind, PatternRegion, RhythmOrigin, RhythmResolution,
    RhythmType, STAR_INNER_DEFAULT, STAR_INNER_MAX, STAR_INNER_MIN, Style, TextAlign, Unit, Uuid,
    VConstraint,
};

/// The inspector's tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    /// What is marked Show in Basic, with its main settings; first in the inspector (rung 16b, ADR-026).
    Basic,
    /// Nothing selected: the page's settings.
    Page,
    /// Position and size.
    Arrange,
    /// Line role or custom stroke; a pattern region's pattern.
    Style,
    /// A text box's text and type.
    Text,
    /// A repeat's settings and Expand (rung 10).
    Repeat,
    /// How the object follows other devices' pages (napkin §5.4).
    Constraints,
    /// What differs on the secondary device on screen (rung 8a, spec §6.1).
    Overrides,
    /// Lint findings on the device on screen.
    Lint,
}

/// Which parts of the stroke differ across the objects picked (they read "mixed").
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct StrokeMixed {
    pub(crate) kind: bool,
    pub(crate) role: bool,
    pub(crate) width: bool,
    pub(crate) gray: bool,
}

/// What a double-click in the layers panel is renaming.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Renaming {
    Layer(usize, String),
    Object(Uuid, String),
}

/// The line roles as the Repeat tab's rules offer them (the same words as the Style tab's).
pub(crate) const LINE_ROLE_OPTIONS: [(LineRole, &str); 8] = ROLES;

/// The fill roles as the Repeat tab's rules offer them.
pub(crate) const FILL_ROLE_OPTIONS: [(FillRole, &str); 2] = [
    (FillRole::Shade, "app-fill-shade"),
    (FillRole::Block, "app-fill-block"),
];

const ROLES: [(LineRole, &str); 8] = [
    (LineRole::Whisper, "app-role-whisper"),
    (LineRole::Guide, "app-role-guide"),
    (LineRole::Structure, "app-role-structure"),
    (LineRole::Emphasis, "app-role-emphasis"),
    (LineRole::Margin, "app-role-margin"),
    (LineRole::DotGrid, "app-role-dotgrid"),
    (LineRole::Text, "app-role-text"),
    (LineRole::Label, "app-role-label"),
];

/// A row of the layers panel as a place to drop.
#[derive(Clone, Copy, Debug)]
enum Drop {
    /// A layer's row, by index.
    Layer(usize),
    /// An object's row, at any depth in the tree (rung 7a).
    Row(Uuid),
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
        // Seven line icons in one row (fixes batch 1 bbq, Q5), a little apart in three groups.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            let can = !self.viewer.selection.is_empty() || self.selected_layer.is_some();
            if self.layer_icon_button(ui, LayerIcon::MoveUp, can, None) {
                self.move_selected(Toward::Front);
            }
            if self.layer_icon_button(ui, LayerIcon::MoveDown, can, None) {
                self.move_selected(Toward::Back);
            }
            ui.add_space(6.0);
            // New layer and Delete layer (docs/bbq/2026-11-18-new-and-delete-layers.md).
            self.layer_buttons(ui);
            ui.add_space(6.0);
            self.group_icon_buttons(ui);
        });
        ui.separator();
        let Some(doc) = self.viewer.doc().cloned() else {
            return;
        };
        let add = ui.input(|i| i.modifiers.shift);
        let mut row_act = None;
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
                    row_act = self.layer_row_menu(&row.response, li).or(row_act);
                    self.object_rows(ui, &doc, &layer.objects, li, 0, add);
                }
            });
        if let Some(act) = row_act {
            self.do_row_act(act);
        }
    }

    /// The Group and Ungroup buttons (Ctrl+G, Ctrl+Shift+G), here and in the Arrange tab (rung 7a
    /// bbq, "Making and breaking").
    pub(crate) fn group_buttons(&mut self, ui: &mut egui::Ui) {
        let editable = self.viewer.editable_selection();
        let any_group = editable.iter().any(|id| {
            self.viewer
                .doc()
                .and_then(|d| d.object(*id))
                .is_some_and(Object::is_group)
        });
        if ui
            .add_enabled(editable.len() >= 2, egui::Button::new(self.t("app-group")))
            .clicked()
        {
            let _ = self.viewer.group_selection();
        }
        if ui
            .add_enabled(any_group, egui::Button::new(self.t("app-ungroup")))
            .clicked()
        {
            let _ = self.viewer.ungroup_selection();
        }
        // Repeat: the picked objects become the source of a repeat (rung 10).
        if ui
            .add_enabled(
                !editable.is_empty(),
                egui::Button::new(self.t("app-repeat")),
            )
            .on_hover_text(self.t("app-repeat-hint"))
            .clicked()
        {
            let _ = self.viewer.repeat_selection();
        }
    }

    /// Group, Ungroup and Repeat as icons in the layers panel's row (fixes batch 1 bbq, Q5); the same three
    /// conditions and hover note as [`ViewerUi::group_buttons`].
    pub(crate) fn group_icon_buttons(&mut self, ui: &mut egui::Ui) {
        let editable = self.viewer.editable_selection();
        let any_group = editable.iter().any(|id| {
            self.viewer
                .doc()
                .and_then(|d| d.object(*id))
                .is_some_and(Object::is_group)
        });
        if self.layer_icon_button(ui, LayerIcon::Group, editable.len() >= 2, None) {
            let _ = self.viewer.group_selection();
        }
        if self.layer_icon_button(ui, LayerIcon::Ungroup, any_group, None) {
            let _ = self.viewer.ungroup_selection();
        }
        let hint = Some(self.t("app-repeat-hint"));
        if self.layer_icon_button(ui, LayerIcon::Repeat, !editable.is_empty(), hint) {
            let _ = self.viewer.repeat_selection();
        }
    }

    /// The rows of one list of objects, top of the stack first; a group's row has a fold arrow
    /// and, unless folded, its children under it, one step in (rung 7a bbq, "Layers tree").
    fn object_rows(
        &mut self,
        ui: &mut egui::Ui,
        doc: &Document,
        objs: &[Object],
        li: usize,
        depth: usize,
        add: bool,
    ) {
        for o in objs.iter().rev() {
            let row = ui.horizontal(|ui| {
                ui.add_space(18.0 + 14.0 * depth as f32);
                if o.is_container() {
                    let folded = self.folded.contains(&o.id);
                    let name = self
                        .strings
                        .format(&Message::new("app-fold-item").text("name", &o.name));
                    // The fold arrow: a triangle that points right when folded, down when open.
                    let arrow = ui.add(
                        egui::Button::new("")
                            .min_size(egui::vec2(16.0, 16.0))
                            .frame(false),
                    );
                    arrow
                        .widget_info(|| WidgetInfo::selected(Role::CheckBox, true, !folded, &name));
                    egui::collapsing_header::paint_default_icon(
                        ui,
                        if folded { 0.0 } else { 1.0 },
                        &arrow,
                    );
                    if arrow.on_hover_text(&name).clicked() {
                        if folded {
                            self.folded.remove(&o.id);
                        } else {
                            self.folded.insert(o.id);
                        }
                    }
                } else {
                    ui.add_space(20.0);
                }
                // On a secondary device the eye is that device's (an override), and a dot marks an
                // object changed there (rung 8a, Defaulted "Badges").
                let device = self.viewer.override_device();
                let shown = device.as_deref().map_or(o.visible, |d| o.visible_on(d));
                if let Some(v) = self.eye_and_lock(ui, &o.name, shown, o.locked) {
                    if device.is_some() && v.0 != shown {
                        let _ = self.viewer.set_visible_on_device(o.id, v.0);
                    } else {
                        let _ = self.viewer.set_object_panel(o.id, |obj| {
                            if device.is_none() {
                                obj.visible = v.0;
                            }
                            obj.locked = v.1;
                        });
                    }
                }
                if device
                    .as_deref()
                    .is_some_and(|d| o.override_on(d).is_some_and(|x| !x.is_empty()))
                {
                    self.paint_badge(ui);
                }
                let selected = self.viewer.selection.contains(&o.id);
                // A text box drawn from saved outlines, because its font is missing (rung 12a, Q6).
                if self.viewer.is_read_only_text(o.id) {
                    ui.label(RichText::new("⚠").color(ui.visuals().warn_fg_color))
                        .on_hover_text(self.t("app-layer-saved-outlines"));
                }
                if let Some(ev) = self.name_cell(
                    ui,
                    Renaming::Object(o.id, String::new()),
                    &o.name,
                    selected,
                    o.is_container(),
                    Some(LayerDrag::Object(o.id)),
                ) {
                    match ev {
                        NameEvent::Click => {
                            self.viewer.select_row(o.id, add);
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
            self.row_drop(ui, &row.response, doc, Drop::Row(o.id));
            if o.is_container() && !self.folded.contains(&o.id) {
                self.object_rows(ui, doc, o.children(), li, depth + 1, add);
            }
        }
    }

    /// A row of the layers panel as a place to drop: draws where the dragged row would land and,
    /// on release, moves it there as one undo step (rung 4 bbq Q6). Objects go among objects, or
    /// to the top of a layer when dropped on its row; layers go among layers.
    fn row_drop(&mut self, ui: &mut egui::Ui, row: &egui::Response, doc: &Document, at: Drop) {
        let Some(payload) = row.dnd_hover_payload::<LayerDrag>() else {
            return;
        };
        let frac = ui
            .input(|i| i.pointer.latest_pos())
            .map_or(0.0, |p| (p.y - row.rect.top()) / row.rect.height().max(1.0));
        let above = frac < 0.5;
        // On a group's row the middle is "inside it"; the edges are above and below.
        let group_row = matches!(at, Drop::Row(id) if doc.object(id).is_some_and(Object::is_group));
        let zone = if group_row && (0.25..0.75).contains(&frac) {
            RowZone::Inside
        } else if above {
            RowZone::Above
        } else {
            RowZone::Below
        };
        // Where it would go: None when the row is no place for this payload.
        enum Plan {
            Object(Uuid, Place, usize),
            Layer(usize, usize),
        }
        let plan = match (*payload, at) {
            (LayerDrag::Object(id), Drop::Row(row_id)) => {
                row_drop_place(doc, id, row_id, zone).map(|(p, i)| Plan::Object(id, p, i))
            }
            (LayerDrag::Object(id), Drop::Layer(layer)) => {
                object_drop_place(doc, id, DropOn::LayerTop(layer), above)
                    .map(|(l, i)| Plan::Object(id, Place::Layer(l), i))
                    .or_else(|| {
                        // From inside a group to the top of a layer.
                        let n = doc.layers.get(layer)?.objects.len();
                        doc.locate(id)?;
                        Some(Plan::Object(id, Place::Layer(layer), n))
                    })
            }
            (LayerDrag::Layer(from), Drop::Layer(target)) => {
                Some(Plan::Layer(from, layer_drop_place(from, target, above)))
            }
            (LayerDrag::Layer(_), Drop::Row(_)) => None,
        };
        if plan.is_none() {
            return;
        }
        // The line where it lands: the row's top edge for above, bottom for below; a drop on a
        // layer's own row, or the middle of a group's, goes inside it, so that row is boxed.
        let accent = ui.visuals().selection.stroke.color;
        let stroke = egui::Stroke::new(2.0, accent);
        match (*payload, at) {
            (LayerDrag::Object(_), Drop::Layer(_)) => {
                ui.painter()
                    .rect_stroke(row.rect, 2.0, stroke, egui::StrokeKind::Inside);
            }
            (LayerDrag::Object(_), Drop::Row(_)) if zone == RowZone::Inside => {
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
                Some(Plan::Object(id, place, index)) => {
                    let _ = self.viewer.move_object_into(id, &place, index);
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
            let select_all = std::mem::take(&mut self.rename_select_all);
            let out = egui::TextEdit::singleline(buf)
                .desired_width(150.0)
                .show(ui);
            let resp = out.response;
            if select_all {
                // A new layer's name opens selected, to type over it.
                let mut state = out.state;
                let end = egui::text::CCursor::new(buf.chars().count());
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(0),
                        end,
                    )));
                state.store(ui.ctx(), resp.id);
            }
            resp.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, &label));
            let text = buf.clone();
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.renaming = None;
            } else if resp.lost_focus() {
                // Enter, or a click elsewhere, ends it. Focus is asked for only while the field still has none:
                // asking first, every frame, took it back before this could see it go.
                self.renaming = None;
                if !text.trim().is_empty() && text != name {
                    return Some(NameEvent::Renamed(text));
                }
            } else {
                resp.request_focus();
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
                (Tab::Style, self.t("app-tab-style")),
                (Tab::Arrange, self.t("app-tab-arrange")),
            ]
        } else {
            vec![(Tab::Page, self.t("app-tab-page"))]
        };
        // The Basic tab is about the document, so it is there whether or not anything is selected.
        tabs.insert(0, (Tab::Basic, self.t("app-tab-basic")));
        if text_selected {
            tabs.push((Tab::Text, self.t("app-tab-text")));
        }
        if has_selection && self.repeat_subject().is_some() {
            tabs.push((Tab::Repeat, self.t("app-tab-repeat")));
        }
        // A group has no constraints of its own: what is in it follows the page by its own (rung 7a
        // look change, Q3 as amended).
        if self.constraints_subject().is_some() {
            tabs.push((Tab::Constraints, self.t("app-tab-constraints")));
        }
        // A secondary device on screen: what differs there, with Reset (rung 8a).
        if has_selection && self.viewer.override_device().is_some() {
            tabs.push((Tab::Overrides, self.t("app-tab-overrides")));
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
        // One row of line icons, each named on hover and to screen readers; Lint keeps its count beside its
        // icon, and the open tab's name stands under the row (rung 17 bbq, Q8 and Defaulted 3).
        let open_name = tabs
            .iter()
            .find(|(t, _)| *t == self.tab)
            .map(|(_, n)| n.clone())
            .unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            for (t, label) in &tabs {
                let count = (*t == Tab::Lint).then_some(count);
                if self.tab_icon_button(ui, *t, label, self.tab == *t, count) {
                    self.tab = *t;
                }
            }
        });
        ui.label(RichText::new(open_name).small().strong());
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
        if has_selection && self.tab != Tab::Lint && self.tab != Tab::Basic {
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
                Tab::Basic => self.basic_tab(ui),
                Tab::Page => self.page_tab(ui),
                Tab::Arrange => self.arrange_tab(ui),
                Tab::Style => self.style_tab(ui),
                Tab::Text => self.text_tab(ui),
                Tab::Repeat => self.repeat_tab(ui),
                Tab::Constraints => self.constraints_tab(ui),
                Tab::Overrides => self.overrides_tab(ui),
                Tab::Lint => self.lint_tab(ui),
            });
    }

    pub(crate) fn primary_id(&self) -> String {
        self.viewer
            .doc()
            .map_or_else(String::new, |d| d.primary_profile_id.clone())
    }

    /// A device's short name (`Nomad`) for what is said about it in the inspector and the strip.
    /// A device's full name for the pickers, with the **Edited** mark when it has a local override
    /// (rung 13a bbq, Q3).
    pub(crate) fn full_name(&self, id: &str) -> String {
        let name = quadrule_profiles::active::display_name(id).unwrap_or_else(|| id.to_string());
        self.marked(id, name)
    }

    /// \`name\` with the **Edited** mark after it when built-in \`id\` has a local override.
    pub(crate) fn marked(&self, id: &str, name: String) -> String {
        if quadrule_profiles::active::is_edited(id) {
            self.strings
                .format(&Message::new("profiles-name-edited").text("name", name))
        } else {
            name
        }
    }

    pub(crate) fn device_name(&self, id: &str) -> String {
        crate::viewer::device_label(id)
    }

    /// The first selected object that has constraints: anything but a group, and nothing inside a
    /// repeat's source (it keeps the design of the primary page; rung 10).
    fn constraints_subject(&self) -> Option<Object> {
        let doc = self.viewer.doc()?;
        self.viewer
            .selection
            .iter()
            .filter(|id| {
                !doc.ancestors(**id)
                    .iter()
                    .any(|a| doc.object(*a).is_some_and(Object::is_repeat))
            })
            .filter_map(|id| doc.object(*id))
            .find(|o| !o.is_group())
            .cloned()
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
            self.title_row(ui, &doc);
            ui.label("");
            self.page_mark_tick(ui, crate::basic::PageMark::Title);
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

            // Orientation (rung 8b): one undo step; the pages turn, objects follow their own
            // constraints and override boxes scale with the page.
            let label = ui.label(self.t("app-page-orientation"));
            let mut orientation = doc.page.orientation;
            let options = [
                (Orientation::Portrait, "app-orientation-portrait"),
                (Orientation::Landscape, "app-orientation-landscape"),
            ];
            self.combo(ui, "orientation", label.id, &mut orientation, &options);
            if orientation != doc.page.orientation {
                let _ = self.viewer.set_orientation(orientation);
            }
            ui.end_row();
            if doc.page.orientation == Orientation::Landscape {
                let paper_of = |id: &&str| quadrule_profiles::active::with(|r| r.is_paper(id));
                let ids = doc.profile_ids();
                if ids.iter().any(|id| !paper_of(id)) {
                    ui.label("");
                    ui.label(RichText::new(self.t("app-orientation-landscape-tip")).small());
                    ui.end_row();
                }
                if ids.iter().any(paper_of) {
                    ui.label("");
                    ui.label(RichText::new(self.t("app-orientation-landscape-paper-tip")).small());
                    ui.end_row();
                }
            }

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

        self.devices_section(ui, &doc);

        // Margins, linked by default.
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.strong(self.t("app-page-margins"));
            self.page_mark_tick(ui, crate::basic::PageMark::Margins);
        });
        self.margins_rows(ui, &doc, cx, cy);

        // Rhythm.
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.strong(self.t("app-rhythm"));
            self.page_mark_tick(ui, crate::basic::PageMark::Rhythm);
        });
        let r = doc.rhythm.clone();
        // The type first (rung 9 bbq, Q7): rectangular units, or one side length.
        egui::Grid::new("rhythm-type-grid")
            .num_columns(2)
            .show(ui, |ui| {
                let label = ui.label(self.t("app-rhythm-type"));
                let mut ty = r.rhythm_type;
                let types = [
                    (RhythmType::Rectangular, "app-rhythm-type-rectangular"),
                    (RhythmType::Isometric, "app-rhythm-type-isometric"),
                ];
                self.combo(ui, "rhythm-type", label.id, &mut ty, &types);
                if ty != r.rhythm_type {
                    let mut nr = r.clone();
                    nr.rhythm_type = ty;
                    let _ = self.viewer.edit(Command::SetRhythm(nr), None);
                }
                ui.end_row();
            });
        self.rhythm_link(ui, &r);
        egui::Grid::new("rhythm-grid")
            .num_columns(2)
            .show(ui, |ui| {
                self.rhythm_unit_rows(ui, &r, cx, cy);

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

                // Major every N: every Nth unit is a major line, for the pattern regions that
                // follow it and for the overlay (rung 9 bbq, Q6). Off by default.
                let label = ui.label(self.t("app-rhythm-major"));
                let mut n = r.major_on().map_or(0, i32::from);
                let off = self.t("app-major-off");
                let drag = egui::DragValue::new(&mut n)
                    .range(0..=i32::from(quadrule_model::MAJOR_MAX))
                    .custom_formatter(
                        move |v, _| {
                            if v < 2.0 { off.clone() } else { format!("{v}") }
                        },
                    )
                    .custom_parser(|t| t.trim().parse::<f64>().ok());
                if ui.add(drag).labelled_by(label.id).changed() {
                    let mut nr = r.clone();
                    nr.major_every = (n >= i32::from(quadrule_model::MAJOR_MIN))
                        .then(|| u8::try_from(n).unwrap_or(quadrule_model::MAJOR_MAX));
                    let _ = self
                        .viewer
                        .edit(Command::SetRhythm(nr), Some("rhythm-major"));
                }
                ui.end_row();
            });
        // The unit as each device resolves it (napkin §6.3: "5 mm → 59 px (Manta)").
        for id in doc.profile_ids() {
            if let Ok(p) = quadrule_profiles::active::lookup_in(&doc, id) {
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

        // Whether installed fonts the page uses are saved inside the file (rung 12a bbq, Q3).
        ui.add_space(6.0);
        let mut embed = doc.page.fonts.embed;
        let label = self.t("app-embed-fonts");
        let tick = ui
            .checkbox(&mut embed, label)
            .on_hover_text(self.t("app-embed-fonts-tip"));
        if tick.changed() {
            let _ = self.viewer.set_embed_fonts(embed);
        }
    }

    /// The page's title row, inside a two-column grid: the Page tab's and the Basic tab's (ADR-026).
    pub(crate) fn title_row(&mut self, ui: &mut egui::Ui, doc: &Document) {
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
    }

    /// The margins: the link tick and the four lengths. The Page tab's heading goes above it; so does the Basic
    /// tab's (ADR-026).
    pub(crate) fn margins_rows(
        &mut self,
        ui: &mut egui::Ui,
        doc: &Document,
        cx: UnitContext,
        cy: UnitContext,
    ) {
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
    }

    /// The tick that links the rhythm's two units (a rectangular rhythm has two; an isometric one has a side).
    pub(crate) fn rhythm_link(&mut self, ui: &mut egui::Ui, r: &quadrule_model::Rhythm) {
        if !r.is_isometric() {
            let link = self.t("app-link-rhythm");
            ui.checkbox(&mut self.link_rhythm, link);
        }
    }

    /// The rhythm's spacing rows, inside a two-column grid: the side of an isometric rhythm, or the two units of a
    /// rectangular one (the Page tab's grid has more rows after them; the Basic tab's has these only; ADR-026).
    pub(crate) fn rhythm_unit_rows(
        &mut self,
        ui: &mut egui::Ui,
        r: &quadrule_model::Rhythm,
        cx: UnitContext,
        cy: UnitContext,
    ) {
        // Rhythm units are lengths, but `u` would be circular here: read them without it.
        let plain = |c: UnitContext| UnitContext {
            rhythm_mm: f64::NAN,
            ..c
        };
        if r.is_isometric() {
            // One side; snapping runs on half a side across and a row down.
            if let Some(mm) = self.length_row(
                ui,
                "rhythm-side",
                "app-rhythm-side",
                r.side(),
                &plain(cx),
                true,
            ) && mm.is_finite()
                && mm > 0.0
            {
                let mut nr = r.clone();
                nr.side_mm = Some(mm);
                let _ = self
                    .viewer
                    .edit(Command::SetRhythm(nr), Some("rhythm-side"));
            }
            ui.end_row();
        } else {
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
        }
    }

    /// The devices this page targets (rung 8a, Defaulted "Which devices a document targets" and
    /// "Switching the primary device"): a tick for each device the app has a profile for, the
    /// primary marked and fixed, and Make primary on the others.
    fn devices_section(&mut self, ui: &mut egui::Ui, doc: &Document) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.strong(self.t("app-page-devices"));
            // The Profiles window (rung 13a, Q5): beside the device ticks.
            if ui
                .button(self.t("profiles-open"))
                .on_hover_text(self.t("profiles-open-tip"))
                .clicked()
            {
                self.open_profiles_window();
            }
        });
        ui.label(RichText::new(self.t("app-device-primary-tip")).small());
        let have: Vec<String> = doc.profile_ids().into_iter().map(String::from).collect();
        let is_paper = |id: &str| quadrule_profiles::active::with(|r| r.is_paper(id));
        let mut screens: Vec<String> = quadrule_profiles::active::device_ids();
        let mut papers: Vec<String> = quadrule_profiles::active::paper_ids();
        // A device the page uses from the file's own copy (not added to this computer) still lists.
        for id in &have {
            if !screens.contains(id) && !papers.contains(id) {
                if is_paper(id) {
                    papers.push(id.clone());
                } else {
                    screens.push(id.clone());
                }
            }
        }
        for id in &screens {
            self.device_row(ui, doc, &have, id);
        }
        // The paper sizes are their own group after the devices (rung 13b bbq, Defaulted "Paper in
        // the UI").
        ui.add_space(4.0);
        ui.strong(self.t("app-page-paper"));
        for id in &papers {
            self.device_row(ui, doc, &have, id);
        }
        if let Some(id) = self.take_out.clone() {
            let name = self.device_name(&id);
            let count = doc.overridden_count(&id);
            ui.group(|ui| {
                ui.label(
                    self.strings
                        .format(&Message::new("app-device-take-out").text("device", name.clone())),
                );
                ui.label(
                    self.strings
                        .format(&Message::new("app-device-take-out-lose").num(
                            "count",
                            count as f64,
                            0,
                        )),
                );
                ui.horizontal(|ui| {
                    if ui.button(self.t("app-device-take-out-yes")).clicked() {
                        let ids: Vec<&str> = have
                            .iter()
                            .map(String::as_str)
                            .filter(|h| *h != id)
                            .collect();
                        let _ = self.viewer.set_targets(&ids);
                        self.take_out = None;
                    }
                    if ui.button(self.t("app-device-take-out-no")).clicked() {
                        self.take_out = None;
                    }
                });
            });
        }
    }

    /// One device or paper size in the Page tab's list: its tick, primary or Make primary, and for a
    /// paper size the page picks, the export PPI.
    fn device_row(&mut self, ui: &mut egui::Ui, doc: &Document, have: &[String], id: &String) {
        let name = {
            let n = self.device_name(id);
            self.marked(id, n)
        };
        let is_primary = *id == doc.primary_profile_id;
        let mut on = have.contains(id);
        let paper = quadrule_profiles::active::with(|r| r.is_paper(id));
        ui.horizontal_wrapped(|ui| {
            let label = self
                .strings
                .format(&Message::new("app-device-target").text("device", name.clone()));
            let tick = ui.add_enabled(!is_primary, egui::Checkbox::new(&mut on, label.clone()));
            tick.widget_info(|| WidgetInfo::selected(Role::CheckBox, !is_primary, on, &label));
            if tick.changed() {
                if on {
                    let mut ids: Vec<&str> = have.iter().map(String::as_str).collect();
                    ids.push(id);
                    let _ = self.viewer.set_targets(&ids);
                } else if doc.has_overrides_on(id) {
                    // Ask first: taking the device out drops what is changed on it.
                    self.take_out = Some(id.clone());
                } else {
                    let ids: Vec<&str> = have
                        .iter()
                        .map(String::as_str)
                        .filter(|h| *h != id)
                        .collect();
                    let _ = self.viewer.set_targets(&ids);
                }
            }
            if is_primary {
                ui.label(RichText::new(self.t("app-device-primary")).small());
            } else if on {
                let make = self.strings.format(
                    &Message::new("app-device-make-primary-name").text("device", name.clone()),
                );
                let btn = ui.button(self.t("app-device-make-primary"));
                btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, &make));
                if btn.on_hover_text(make).clicked() {
                    let _ = self.viewer.make_primary(id);
                }
            }
            // The export PPI this page uses on a paper size (rung 13b bbq, Q12): one undo step.
            if paper && on {
                let was = doc.ppi_of(id);
                let mut ppi = was;
                let option = |p: u32| {
                    self.strings
                        .format(&Message::new("app-paper-ppi-option").num("ppi", f64::from(p), 0))
                };
                let described = self
                    .strings
                    .format(&Message::new("app-paper-ppi-name").text("device", name.clone()));
                // Narrow enough that the tick, Make primary and this fit one line of the inspector (the rung 13
                // debt: the choice wrapped onto a second line beside Make primary).
                let combo = egui::ComboBox::from_id_salt(("paper-ppi", id.as_str()))
                    .width(PPI_COMBO_WIDTH)
                    .selected_text(option(was))
                    .show_ui(ui, |ui| {
                        for p in quadrule_model::PAPER_PPIS {
                            ui.selectable_value(&mut ppi, p, option(p));
                        }
                    });
                combo
                    .response
                    .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, &described));
                combo.response.on_hover_text(self.t("app-paper-ppi-tip"));
                if ppi != was {
                    let _ = self.viewer.set_paper_ppi(id, ppi);
                }
            }
        });
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
        let mixed = self.viewer.arrange_mixed();
        // The values are those of the device on screen; typing on a secondary one sets an override
        // there (rung 8a, Defaulted "The inspector on a secondary").
        let device = self.viewer.override_device();
        let selection = self.viewer.selection.clone();
        let f = self
            .viewer
            .shown_frames(&selection)
            .first()
            .map_or(o.frame, |(_, f)| *f);
        let ovr = device
            .as_deref()
            .and_then(|d| o.override_on(d))
            .cloned()
            .unwrap_or_default();
        self.show_in_basic_tick(ui, enabled, mixed.show_in_basic);
        ui.horizontal(|ui| self.group_buttons(ui));
        self.align_section(ui);
        egui::Grid::new("arrange-grid")
            .num_columns(2)
            .show(ui, |ui| {
                type Set = fn(&mut Object, f64);
                type OnDevice = fn(Frame, f64) -> Frame;
                #[allow(clippy::type_complexity)]
                let rows: [(
                    &str,
                    &str,
                    f64,
                    bool,
                    &UnitContext,
                    Set,
                    OnDevice,
                    bool,
                ); 4] = [
                    // A group's contents follow its frame (`reframe`).
                    (
                        "arrange-x",
                        "app-x",
                        f.x_mm,
                        mixed.x,
                        &cx,
                        |o, v| reframe(o, Frame { x_mm: v, ..o.frame }),
                        |fr, v| Frame { x_mm: v, ..fr },
                        ovr.position.is_some(),
                    ),
                    (
                        "arrange-y",
                        "app-y",
                        f.y_mm,
                        mixed.y,
                        &cy,
                        |o, v| reframe(o, Frame { y_mm: v, ..o.frame }),
                        |fr, v| Frame { y_mm: v, ..fr },
                        ovr.position.is_some(),
                    ),
                    (
                        "arrange-w",
                        "app-width",
                        f.w_mm,
                        mixed.w,
                        &cx,
                        |o, v| {
                            reframe(
                                o,
                                Frame {
                                    w_mm: v.max(0.0),
                                    ..o.frame
                                },
                            )
                        },
                        |fr, v| Frame {
                            w_mm: v.max(0.0),
                            ..fr
                        },
                        ovr.size.is_some(),
                    ),
                    (
                        "arrange-h",
                        "app-height",
                        f.h_mm,
                        mixed.h,
                        &cy,
                        |o, v| {
                            reframe(
                                o,
                                Frame {
                                    h_mm: v.max(0.0),
                                    ..o.frame
                                },
                            )
                        },
                        |fr, v| Frame {
                            h_mm: v.max(0.0),
                            ..fr
                        },
                        ovr.size.is_some(),
                    ),
                ];
                // The height of a box that is as tall as its text is its text's: shown, not typed
                // (rung 11b). Switch the box to Fixed to give it a height.
                let all_tall = !self.viewer.selection.is_empty()
                    && self.viewer.selection.iter().all(|id| {
                        self.viewer
                            .doc()
                            .and_then(|d| d.object(*id))
                            .is_some_and(|x| x.keeps_height())
                    });
                for (key, id, v, is_mixed, ctx, set, set_here, badge) in rows {
                    let enabled = enabled && !(key == "arrange-h" && all_tall);
                    if let Some(mm) = self.length_row_badged(
                        ui,
                        key,
                        id,
                        v,
                        ctx,
                        enabled,
                        is_mixed,
                        badge && device.is_some(),
                    ) {
                        if device.is_some() {
                            let _ = self
                                .viewer
                                .edit_frames_on_device(Some(key), |fr| set_here(fr, mm));
                        } else {
                            let _ = self.viewer.edit_selected(Some(key), |o| set(o, mm));
                        }
                    }
                    ui.end_row();
                }
                // Rotation in degrees, clockwise, about the frame's centre (rung 6b, Q5). Lines and
                // lines, shapes, text boxes and pattern regions (a repeat does not rotate, so no
                // field). With several selected, the first that turns gives the value, and the field reads
                // "mixed" when those that turn differ.
                let turner = self
                    .viewer
                    .selection
                    .iter()
                    .filter_map(|id| self.viewer.doc().and_then(|d| d.object(*id)))
                    .find(|x| x.kind.can_rotate())
                    .cloned();
                if let Some(turner) = turner {
                    let label = ui.label(self.t("app-rotation"));
                    let mut deg = turner.rotation();
                    let mut drag = egui::DragValue::new(&mut deg)
                        .range(-360.0..=360.0)
                        .speed(1.0)
                        .max_decimals(2);
                    if mixed.rotation {
                        // The word stands in for the number and its degree sign.
                        let word = self.t("app-mixed");
                        drag = drag
                            .custom_formatter(move |_, _| word.clone())
                            .custom_parser(|t| {
                                t.trim()
                                    .trim_start_matches(|c: char| c.is_alphabetic())
                                    .trim_end_matches('°')
                                    .trim()
                                    .parse()
                                    .ok()
                            });
                    } else {
                        drag = drag.suffix("°");
                    }
                    let changed = ui
                        .add_enabled(enabled, drag)
                        .labelled_by(label.id)
                        .changed();
                    if changed {
                        let _ = self.viewer.edit_selected(Some("arrange-rotation"), |x| {
                            if x.kind.can_rotate() {
                                x.rotation_deg = quadrule_model::normalize_deg(deg);
                            }
                        });
                    }
                    ui.end_row();
                } else if o.is_group() {
                    // A group with a repeat in it does not turn (rung 7a bbq, Q2): the field is
                    // there, greyed, with the reason.
                    let label = ui.label(self.t("app-rotation"));
                    let mut deg = 0.0;
                    ui.add_enabled(false, egui::DragValue::new(&mut deg).suffix("°"))
                        .labelled_by(label.id);
                    ui.end_row();
                    ui.label("");
                    ui.label(RichText::new(self.t("app-rotation-note")).small());
                    ui.end_row();
                }
            });
        // Free placement (rung 14b, Q7): off the rhythm on purpose; the linter's L16 leaves it alone. Mixed
        // when some of the selected objects have it and some do not.
        let picked: Vec<bool> = self
            .viewer
            .selection
            .iter()
            .filter_map(|id| self.viewer.doc().and_then(|d| d.object(*id)))
            .map(|x| x.free_placement)
            .collect();
        let mut on = !picked.is_empty() && picked.iter().all(|b| *b);
        let checkbox = egui::Checkbox::new(&mut on, self.t("app-free-placement"))
            .indeterminate(mixed.free_placement);
        let r = ui
            .add_enabled(enabled, checkbox)
            .on_hover_text(self.t("app-free-placement-tip"));
        if r.changed() {
            let _ = self.viewer.set_free_placement(on);
        }
        // Full bleed (ADR-019): meant to run under the toolbars and footers and to the page edge; the linter's L06
        // and L10 leave it alone. Mixed as Free placement is.
        let picked: Vec<bool> = self
            .viewer
            .selection
            .iter()
            .filter_map(|id| self.viewer.doc().and_then(|d| d.object(*id)))
            .map(|x| x.full_bleed)
            .collect();
        let mut on = !picked.is_empty() && picked.iter().all(|b| *b);
        let checkbox =
            egui::Checkbox::new(&mut on, self.t("app-full-bleed")).indeterminate(mixed.full_bleed);
        let r = ui
            .add_enabled(enabled, checkbox)
            .on_hover_text(self.t("app-full-bleed-tip"));
        if r.changed() {
            let _ = self.viewer.set_full_bleed(on);
        }
    }

    /// Align and distribute (rung 7b): the Align to choice, six align buttons, four distribute
    /// buttons. Icons, each named for screen readers and its tooltip.
    fn align_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.strong(self.t("app-align-heading"));
        let to = self.viewer.align_to();
        let mut picked = to;
        ui.horizontal(|ui| {
            let label = ui.label(self.t("app-align-to"));
            self.combo(
                ui,
                "align-to",
                label.id,
                &mut picked,
                &[
                    (AlignTo::Selection, "app-align-to-selection"),
                    (AlignTo::Page, "app-align-to-page"),
                    (AlignTo::Margins, "app-align-to-margins"),
                ],
            );
        });
        if picked != to {
            self.viewer.set_align_to(picked);
        }
        ui.horizontal_wrapped(|ui| {
            for (n, op) in ArrangeOp::ALL.into_iter().enumerate() {
                if n == 6 {
                    ui.add_space(8.0);
                }
                let on = self.viewer.arrange_enabled(op);
                let name = self.t(op.message_id());
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(26.0, 26.0), Sense::click());
                response.widget_info(|| WidgetInfo::labeled(Role::Button, on, &name));
                let v = ui.visuals();
                let (fill, color) = if !on {
                    (
                        Color32::TRANSPARENT,
                        v.widgets.noninteractive.fg_stroke.color,
                    )
                } else if response.hovered() {
                    (
                        v.widgets.hovered.weak_bg_fill,
                        v.widgets.hovered.fg_stroke.color,
                    )
                } else {
                    (Color32::TRANSPARENT, v.widgets.inactive.fg_stroke.color)
                };
                let color = if on {
                    color
                } else {
                    color.gamma_multiply(0.45)
                };
                let painter = ui.painter();
                painter.rect_filled(rect, 4.0, fill);
                crate::arrange::paint_icon(painter, op, rect.center(), color);
                let response = response.on_hover_text(name);
                if on && response.clicked() {
                    let _ = self.viewer.arrange(op);
                }
            }
        });
        ui.add_space(4.0);
    }

    fn style_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.first_selected() else {
            return;
        };
        let Some(cx) = self.viewer.unit_context(Axis::X) else {
            return;
        };
        let enabled = !self.viewer.editable_selection().is_empty();
        if self.viewer.override_device().is_some() {
            ui.label(RichText::new(self.t("app-style-shared")).small());
        }
        // Which parts differ across the objects picked: those fields read "mixed" (rung 6a/7b debt).
        let picked: Vec<Object> = self
            .viewer
            .selection
            .iter()
            .filter_map(|id| self.viewer.doc().and_then(|d| d.object(*id)))
            .cloned()
            .collect();
        let any_differs =
            |same: &dyn Fn(&Object, &Object) -> bool| picked.iter().any(|x| !same(x, &o));
        let kind_mixed =
            any_differs(&|a, b| StrokeChoice::of(a.style) == StrokeChoice::of(b.style));
        let role_mixed = any_differs(
            &|a, b| matches!((a.style, b.style), (Style::Role(x), Style::Role(y)) if x == y),
        );
        let width_mixed = any_differs(&|a, b| {
            matches!((a.style, b.style),
                (Style::Custom { width_mm: x, .. }, Style::Custom { width_mm: y, .. }) if (x - y).abs() < 1e-9)
        });
        let gray_mixed = any_differs(&|a, b| {
            matches!((a.style, b.style),
                (Style::Custom { gray: x, .. }, Style::Custom { gray: y, .. }) if x == y)
        });
        let opacity_mixed = any_differs(&|a, b| a.opacity() == b.opacity());
        // A group has opacity and nothing else: its stroke and fill are its children's (rung 7a).
        if o.is_group() {
            ui.add_enabled_ui(enabled, |ui| {
                egui::Grid::new("style-grid").num_columns(2).show(ui, |ui| {
                    self.opacity_row(ui, &o, opacity_mixed);
                });
            });
            return;
        }
        let shape = o.kind.is_shape();
        // Dash and cap are for anything drawn as a stroke: lines and the shapes.
        let stroked = shape || matches!(o.kind, ObjectKind::Line(_));
        ui.add_enabled_ui(enabled, |ui| {
            egui::Grid::new("style-grid").num_columns(2).show(ui, |ui| {
                self.stroke_rows(
                    ui,
                    &o,
                    &cx,
                    enabled,
                    StrokeMixed {
                        kind: kind_mixed,
                        role: role_mixed,
                        width: width_mixed,
                        gray: gray_mixed,
                    },
                );

                if shape {
                    self.fill_rows(ui, &o, enabled);
                }

                if stroked && o.style != Style::None {
                    self.dash_rows(ui, &o, &cx, enabled);
                }

                self.opacity_row(ui, &o, opacity_mixed);

                if let ObjectKind::Pattern(pr) = &o.kind {
                    let label = ui.label(self.t("app-pattern"));
                    let mut p = pr.pattern.kind();
                    let options = [
                        (PatternKind::Lines, "app-pattern-lines"),
                        (PatternKind::Dots, "app-pattern-dots"),
                        (PatternKind::Grid, "app-pattern-grid"),
                        (PatternKind::Isometric, "app-pattern-isometric"),
                        (PatternKind::Hex, "app-pattern-hex"),
                        (PatternKind::Cornell, "app-pattern-cornell"),
                        (PatternKind::MusicStaff, "app-pattern-staff"),
                        (PatternKind::Calendar, "app-pattern-calendar"),
                    ];
                    self.combo(ui, "pattern", label.id, &mut p, &options);
                    if p != pr.pattern.kind() {
                        let _ = self.viewer.edit_selected(None, |x| x.set_pattern_kind(p));
                    }
                    ui.end_row();
                }
            });
            if let ObjectKind::Pattern(pr) = &o.kind {
                ui.add_space(6.0);
                ui.strong(self.t("app-pattern-settings"));
                self.pattern_rows(ui, &o, pr, &cx, enabled);
                ui.add_space(6.0);
                let heading = ui.strong(self.t("app-border-section"));
                if std::mem::take(&mut self.look_border) {
                    heading.scroll_to_me(Some(egui::Align::TOP));
                }
                self.border_rows(ui, pr, &cx, enabled);
            }
            if shape {
                ui.add_space(6.0);
                ui.strong(self.t("app-shape"));
                self.shape_rows(ui, &o, &cx, enabled);
            }
            if let ObjectKind::MiniCalendar(m) = &o.kind {
                ui.add_space(6.0);
                ui.strong(self.t("app-mini-calendar"));
                self.mini_calendar_rows(ui, m, enabled);
            }
        });
    }

    /// The stroke rows, inside a two-column grid: None (a shape only), a role or custom values; the role, or the
    /// width, the gray and the role that matches it. The Style tab's and the Basic tab's (ADR-026).
    pub(crate) fn stroke_rows(
        &mut self,
        ui: &mut egui::Ui,
        o: &Object,
        cx: &UnitContext,
        enabled: bool,
        mixed: StrokeMixed,
    ) {
        let shape = o.kind.is_shape();
        let cx = *cx;
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
            if mixed.kind {
                ui.label(RichText::new(self.t("app-mixed")).small());
            }
        })
        .response
        .labelled_by(label.id);
        if want != was {
            let device = self.viewer.profiles().map(|(_, shown)| shown);
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
                (StrokeChoice::Custom, _) => from_role(default_role(o)),
                (StrokeChoice::Role, _) => Style::Role(default_role(o)),
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
                self.combo_mixed(ui, "role", label.id, &mut r, &ROLES, mixed.role);
                if r != role {
                    let _ = self
                        .viewer
                        .edit_selected(None, |x| x.style = Style::Role(r));
                }
                ui.end_row();
            }
            Style::Custom { width_mm, gray } => {
                if let Some(mm) = self.length_row_mixed(
                    ui,
                    "style-width",
                    "app-stroke-width",
                    width_mm,
                    &cx,
                    enabled,
                    mixed.width,
                ) {
                    let _ = self.viewer.edit_selected(Some("style-width"), |x| {
                        if let Style::Custom { width_mm, .. } = &mut x.style {
                            *width_mm = mm.max(0.0);
                        }
                    });
                }
                ui.end_row();
                // The gray: the profile's clean levels as swatches (rung 14b, Q8), then which
                // role it matches, or Use recommended.
                let shown = self.viewer.profiles().map(|(_, shown)| shown);
                let levels: Vec<u8> = shown
                    .as_ref()
                    .map(|p| p.gray_levels.clone())
                    .unwrap_or_default();
                if let Some((g, drag)) = self.gray_field(
                    ui,
                    "stroke",
                    "app-stroke-gray",
                    gray,
                    mixed.gray,
                    &levels,
                    enabled,
                ) {
                    let merge = drag.then_some("style-gray");
                    let _ = self.viewer.edit_selected(merge, |x| {
                        if let Style::Custom { gray, .. } = &mut x.style {
                            *gray = g;
                        }
                    });
                }
                if let Some(shown) = shown
                    && !mixed.width
                    && !mixed.gray
                {
                    let width_px =
                        quadrule_layout::stroke_opt(o.style, &shown).map_or(1, |(w, _)| w);
                    if let Some(role) = self.role_badge(ui, o, &shown, width_px, gray, enabled) {
                        let _ = self
                            .viewer
                            .edit_selected(None, |x| x.style = Style::Role(role));
                    }
                }
            }
        }
    }

    /// A mini calendar's settings (calendar bbq, Q2 and Q3): its month and year, the week's first day and
    /// whether the month's name heads it. Each edit is one undo step (typing the year merges).
    pub(crate) fn mini_calendar_rows(
        &mut self,
        ui: &mut egui::Ui,
        m: &quadrule_model::MiniCalendarShape,
        enabled: bool,
    ) {
        macro_rules! set {
            ($key:expr, |$mc:ident| $body:expr) => {{
                let _ = self.viewer.edit_selected($key, |x| {
                    if let ObjectKind::MiniCalendar($mc) = &mut x.kind {
                        $body;
                    }
                });
            }};
        }
        egui::Grid::new("mini-calendar-grid")
            .num_columns(2)
            .show(ui, |ui| {
                let label = ui.label(self.t("app-calendar-month"));
                let mut n = m.month.month_clamped();
                let ids: Vec<(u8, String)> = (1..=12u8)
                    .map(|k| (k, format!("calendar-month-{k}")))
                    .collect();
                let options: Vec<(u8, &str)> =
                    ids.iter().map(|(v, id)| (*v, id.as_str())).collect();
                self.combo(ui, "mini-month", label.id, &mut n, &options);
                if n != m.month.month_clamped() {
                    set!(None, |mc| mc.month.month = n);
                }
                ui.end_row();
                let label = ui.label(self.t("app-calendar-year"));
                let mut y = m.month.year_clamped();
                let drag = egui::DragValue::new(&mut y)
                    .range(quadrule_model::calendar::YEAR_MIN..=quadrule_model::calendar::YEAR_MAX);
                if ui
                    .add_enabled(enabled, drag)
                    .labelled_by(label.id)
                    .changed()
                {
                    set!(Some("mini-calendar-year"), |mc| mc.month.year = y);
                }
                ui.end_row();
                let label = ui.label(self.t("app-calendar-week-start"));
                let mut w = m.week_start;
                let options = weekday_options();
                self.combo(ui, "mini-week-start", label.id, &mut w, &options);
                if w != m.week_start {
                    set!(None, |mc| mc.week_start = w);
                }
                ui.end_row();
                let label = ui.label(self.t("app-mini-title"));
                let mut t = m.title;
                if ui
                    .add_enabled(enabled, egui::Checkbox::new(&mut t, ""))
                    .labelled_by(label.id)
                    .changed()
                {
                    set!(None, |mc| mc.title = t);
                }
                ui.end_row();
            });
    }

    /// The settings of a pattern region (rung 9 bbq, Defaulted "Where the settings live"), below the
    /// pattern choice: each pattern's own, whether it is bound to the rhythm and its own spacing
    /// when it is not, and its major lines. Each edit is one undo step (typing in a field merges).
    fn pattern_rows(
        &mut self,
        ui: &mut egui::Ui,
        o: &Object,
        pr: &PatternRegion,
        cx: &UnitContext,
        enabled: bool,
    ) {
        // Edits one pattern setting on every selected region.
        macro_rules! set {
            ($key:expr, |$pr:ident| $body:expr) => {{
                let _ = self.viewer.edit_selected($key, |x| {
                    if let ObjectKind::Pattern($pr) = &mut x.kind {
                        $body;
                    }
                });
            }};
        }
        egui::Grid::new("pattern-grid")
            .num_columns(2)
            .show(ui, |ui| {
                self.pattern_kind_rows(ui, o, pr, cx, enabled);
                self.pattern_spacing_rows(ui, pr, cx, enabled);

                // The ruling's own angle inside the frame, in degrees clockwise, about the frame's
                // middle (rung 12b bbq, Q8 as amended, Q11); it adds to the frame's Rotation.
                let label = ui.label(self.t("app-pattern-angle"));
                let mut deg = pr.pattern_angle();
                let drag = egui::DragValue::new(&mut deg)
                    .range(-360.0..=360.0)
                    .speed(1.0)
                    .max_decimals(2)
                    .suffix("°");
                if ui
                    .add_enabled(enabled, drag)
                    .labelled_by(label.id)
                    .changed()
                {
                    let deg = quadrule_model::normalize_deg(deg);
                    set!(Some("pattern-angle"), |p| p.angle_deg = deg);
                }
                ui.end_row();

                self.pattern_major_rows(ui, pr, enabled);
                let on = enabled && pr.pattern.has_major();
                // Where the major rows and the major columns count from, each on its own (ADR-023 as amended
                // 2026-11-23): Top, Centre or the rhythm origin. Isometric has no Top, so it is greyed there, and a
                // region a file sets to Top reads as what it draws: Rhythm origin. Lines and Cornell have no major
                // columns, so that row is greyed for them.
                let options = [
                    (MajorFrom::Top, "app-major-from-top"),
                    (MajorFrom::Centre, "app-major-from-centre"),
                    (MajorFrom::RhythmOrigin, "app-major-from-origin"),
                ];
                let top_ok = pr.pattern.offers_major_top();
                for (rows, label_id, salt, shown, axis_on) in [
                    (
                        true,
                        "app-pattern-major-rows-from",
                        "pattern-major-rows-from",
                        pr.major_rows_origin(),
                        on,
                    ),
                    (
                        false,
                        "app-pattern-major-columns-from",
                        "pattern-major-columns-from",
                        pr.major_columns_origin(),
                        on && pr.pattern.has_major_columns(),
                    ),
                ] {
                    let label = ui.label(self.t(label_id));
                    let mut want = shown;
                    ui.add_enabled_ui(axis_on, |ui| {
                        egui::ComboBox::from_id_salt(salt)
                            .selected_text(
                                self.t(options
                                    .iter()
                                    .find(|(v, _)| *v == shown)
                                    .map_or("app-major-from-origin", |(_, id)| id)),
                            )
                            .show_ui(ui, |ui| {
                                for (v, id) in options {
                                    ui.add_enabled_ui(v != MajorFrom::Top || top_ok, |ui| {
                                        ui.selectable_value(&mut want, v, self.t(id));
                                    });
                                }
                            })
                            .response
                            .labelled_by(label.id);
                    });
                    if want != shown {
                        set!(None, |p| if rows {
                            p.major_rows_from = want;
                        } else {
                            p.major_columns_from = want;
                        });
                    }
                    ui.end_row();
                }
                let label = ui.label(self.t("app-pattern-major-role"));
                let mut role = pr.major_role;
                ui.add_enabled_ui(on, |ui| {
                    self.combo(ui, "pattern-major-role", label.id, &mut role, &ROLES);
                });
                if role != pr.major_role {
                    set!(None, |p| p.major_role = role);
                }
                ui.end_row();
            });
    }

    /// The rows that are one pattern's own (its style, its height, its cue and summary sizes, its staff or its
    /// month), inside a two-column grid: the Style tab's and the Basic tab's (ADR-026).
    pub(crate) fn pattern_kind_rows(
        &mut self,
        ui: &mut egui::Ui,
        o: &Object,
        pr: &PatternRegion,
        cx: &UnitContext,
        enabled: bool,
    ) {
        let Some(cy) = self.viewer.unit_context(Axis::Y) else {
            return;
        };
        let Some(steps) = self.viewer.doc().map(|d| d.rhythm.steps_mm()) else {
            return;
        };
        // Edits one pattern setting on every selected region.
        macro_rules! set {
            ($key:expr, |$pr:ident| $body:expr) => {{
                let _ = self.viewer.edit_selected($key, |x| {
                    if let ObjectKind::Pattern($pr) = &mut x.kind {
                        $body;
                    }
                });
            }};
        }
        match pr.pattern {
            Pattern::Isometric { style } => {
                let label = ui.label(self.t("app-iso-style"));
                let mut st = style;
                let options = [
                    (IsoStyle::Lines, "app-iso-lines"),
                    (IsoStyle::Dots, "app-iso-dots"),
                ];
                self.combo(ui, "iso-style", label.id, &mut st, &options);
                if st != style {
                    let _ = self.viewer.edit_selected(None, |x| {
                        if let ObjectKind::Pattern(p) = &mut x.kind {
                            p.pattern = Pattern::Isometric { style: st };
                        }
                        x.settle_pattern_role();
                    });
                }
                ui.end_row();
            }
            Pattern::Hex {
                orientation,
                height_units,
            } => {
                let label = ui.label(self.t("app-hex-orientation"));
                let mut or = orientation;
                let options = [
                    (HexOrientation::Flat, "app-hex-flat"),
                    (HexOrientation::Pointy, "app-hex-pointy"),
                ];
                self.combo(ui, "hex-orientation", label.id, &mut or, &options);
                if or != orientation {
                    set!(
                        None,
                        |p| if let Pattern::Hex { orientation, .. } = &mut p.pattern {
                            *orientation = or;
                        }
                    );
                }
                ui.end_row();
                if let Some(v) = self.units_row(ui, "app-hex-height", height_units, enabled) {
                    set!(Some("pattern-hex-height"), |p| if let Pattern::Hex {
                        height_units, ..
                    } = &mut p.pattern
                    {
                        *height_units = v;
                    });
                }
                ui.end_row();
            }
            Pattern::Cornell {
                cue_units,
                summary_units,
            } => {
                // What the region has now: its own, or the default its size gives.
                let (cue, sum) = quadrule_layout::pattern::cornell_units(
                    cue_units,
                    summary_units,
                    (o.frame.w_mm, o.frame.h_mm),
                    (cx.rhythm_mm, cy.rhythm_mm),
                );
                if let Some(v) = self.units_row(ui, "app-cornell-cue", cue, enabled) {
                    set!(Some("pattern-cornell-cue"), |p| if let Pattern::Cornell {
                        cue_units,
                        ..
                    } = &mut p.pattern
                    {
                        *cue_units = Some(v);
                    });
                }
                ui.end_row();
                if let Some(v) = self.units_row(ui, "app-cornell-summary", sum, enabled) {
                    set!(
                        Some("pattern-cornell-summary"),
                        |p| if let Pattern::Cornell { summary_units, .. } = &mut p.pattern {
                            *summary_units = Some(v);
                        }
                    );
                }
                ui.end_row();
            }
            Pattern::MusicStaff {
                gap_mm,
                pitch_units,
                grand,
                grand_space_units,
            } => {
                if let Some(mm) = self.length_row(
                    ui,
                    "pattern-staff-gap",
                    "app-staff-gap",
                    gap_mm,
                    cx,
                    enabled,
                ) && mm.is_finite()
                    && mm > 0.0
                {
                    set!(Some("pattern-staff-gap"), |p| if let Pattern::MusicStaff {
                        gap_mm, ..
                    } = &mut p.pattern
                    {
                        *gap_mm = mm;
                    });
                }
                ui.end_row();
                // A grand staff with its own space shows that (bass bottom to the next top); a single
                // staff, and a grand staff from an older file with none, show "Staff every" (top to top).
                match grand_space_units.filter(|_| grand) {
                    Some(space) => {
                        if let Some(v) = self.units_row(ui, "app-staff-grand-space", space, enabled)
                        {
                            set!(Some("pattern-staff-grand-space"), |p| {
                                if let Pattern::MusicStaff {
                                    grand_space_units, ..
                                } = &mut p.pattern
                                {
                                    *grand_space_units = Some(v);
                                }
                            });
                        }
                    }
                    None => {
                        if let Some(v) = self.units_row(ui, "app-staff-pitch", pitch_units, enabled)
                        {
                            set!(Some("pattern-staff-pitch"), |p| {
                                if let Pattern::MusicStaff { pitch_units, .. } = &mut p.pattern {
                                    *pitch_units = v;
                                }
                            });
                        }
                    }
                }
                ui.end_row();
                let label = ui.label(self.t("app-staff-grand"));
                let mut g = grand;
                if ui.checkbox(&mut g, "").labelled_by(label.id).changed() {
                    // Turning it on writes the space, starting where the staves look closest to
                    // what "Staff every" gave; turning it off keeps the field for a turn back on.
                    set!(None, |p| {
                        let unit = if p.bound {
                            steps.1
                        } else {
                            p.v_spacing_mm.unwrap_or(steps.1)
                        };
                        if let Pattern::MusicStaff {
                            gap_mm,
                            pitch_units,
                            grand,
                            grand_space_units,
                        } = &mut p.pattern
                        {
                            *grand = g;
                            if g && grand_space_units.is_none() {
                                *grand_space_units = Some(quadrule_model::grand_space_closest(
                                    *gap_mm,
                                    *pitch_units,
                                    unit,
                                ));
                            }
                        }
                    });
                }
                ui.end_row();
            }
            Pattern::Calendar {
                month,
                week_start,
                first_day,
                days,
            } => {
                // The month is one field (calendar bbq, Q1): Undated, or a month, with its year below.
                let label = ui.label(self.t("app-calendar-month"));
                let mut m = month.map(|m| m.month_clamped());
                let ids: Vec<(Option<u8>, String)> =
                    std::iter::once((None, "app-calendar-undated".to_string()))
                        .chain((1..=12u8).map(|n| (Some(n), format!("calendar-month-{n}"))))
                        .collect();
                let options: Vec<(Option<u8>, &str)> =
                    ids.iter().map(|(v, id)| (*v, id.as_str())).collect();
                self.combo(ui, "calendar-month", label.id, &mut m, &options);
                if m != month.map(|m| m.month_clamped()) {
                    let year = month.map_or_else(this_month_year, |m| m.year_clamped());
                    set!(
                        None,
                        |p| if let Pattern::Calendar { month, .. } = &mut p.pattern {
                            *month = m.map(|n| quadrule_model::YearMonth::new(year, n));
                        }
                    );
                }
                ui.end_row();
                if let Some(ym) = month {
                    let label = ui.label(self.t("app-calendar-year"));
                    let mut y = ym.year_clamped();
                    let drag = egui::DragValue::new(&mut y).range(
                        quadrule_model::calendar::YEAR_MIN..=quadrule_model::calendar::YEAR_MAX,
                    );
                    if ui
                        .add_enabled(enabled, drag)
                        .labelled_by(label.id)
                        .changed()
                    {
                        set!(
                            Some("pattern-calendar-year"),
                            |p| if let Pattern::Calendar { month: Some(m), .. } = &mut p.pattern {
                                m.year = y;
                            }
                        );
                    }
                    ui.end_row();
                } else {
                    // Undated, the month's shape is its own: where the 1st falls and how many days.
                    let label = ui.label(self.t("app-calendar-first-day"));
                    let mut d = first_day;
                    let options = weekday_options();
                    self.combo(ui, "calendar-first-day", label.id, &mut d, &options);
                    if d != first_day {
                        set!(
                            None,
                            |p| if let Pattern::Calendar { first_day, .. } = &mut p.pattern {
                                *first_day = d;
                            }
                        );
                    }
                    ui.end_row();
                    let label = ui.label(self.t("app-calendar-days"));
                    let mut n = days.clamp(
                        quadrule_model::calendar::MONTH_DAYS_MIN,
                        quadrule_model::calendar::MONTH_DAYS_MAX,
                    );
                    let drag = egui::DragValue::new(&mut n).range(
                        quadrule_model::calendar::MONTH_DAYS_MIN
                            ..=quadrule_model::calendar::MONTH_DAYS_MAX,
                    );
                    if ui
                        .add_enabled(enabled, drag)
                        .labelled_by(label.id)
                        .changed()
                    {
                        set!(
                            Some("pattern-calendar-days"),
                            |p| if let Pattern::Calendar { days, .. } = &mut p.pattern {
                                *days = n;
                            }
                        );
                    }
                    ui.end_row();
                }
                let label = ui.label(self.t("app-calendar-week-start"));
                let mut w = week_start;
                let options = weekday_options();
                self.combo(ui, "calendar-week-start", label.id, &mut w, &options);
                if w != week_start {
                    set!(
                        None,
                        |p| if let Pattern::Calendar { week_start, .. } = &mut p.pattern {
                            *week_start = w;
                        }
                    );
                }
                ui.end_row();
            }
            Pattern::Lines | Pattern::Dots | Pattern::Grid => {}
        }
    }

    /// Whether a pattern region follows the rhythm, and its own spacing when it does not, inside a two-column grid:
    /// the Style tab's and the Basic tab's (ADR-026).
    pub(crate) fn pattern_spacing_rows(
        &mut self,
        ui: &mut egui::Ui,
        pr: &PatternRegion,
        cx: &UnitContext,
        enabled: bool,
    ) {
        let Some(cy) = self.viewer.unit_context(Axis::Y) else {
            return;
        };
        let Some(steps) = self.viewer.doc().map(|d| d.rhythm.steps_mm()) else {
            return;
        };
        // Edits one pattern setting on every selected region.
        macro_rules! set {
            ($key:expr, |$pr:ident| $body:expr) => {{
                let _ = self.viewer.edit_selected($key, |x| {
                    if let ObjectKind::Pattern($pr) = &mut x.kind {
                        $body;
                    }
                });
            }};
        }
        // Bound to the rhythm, or its own spacing (napkin §5.2).
        let label = ui.label(self.t("app-pattern-bound"));
        let mut bound = pr.bound;
        if ui
            .add_enabled(enabled, egui::Checkbox::new(&mut bound, ""))
            .labelled_by(label.id)
            .changed()
        {
            set!(None, |p| p.set_bound(bound, steps));
        }
        ui.end_row();
        if !pr.bound {
            let own = |v: Option<f64>, step: f64| v.unwrap_or(step);
            for (key, id, v, ctx, across) in [
                (
                    "pattern-h",
                    "app-pattern-across",
                    own(pr.h_spacing_mm, steps.0),
                    *cx,
                    true,
                ),
                (
                    "pattern-v",
                    "app-pattern-down",
                    own(pr.v_spacing_mm, steps.1),
                    cy,
                    false,
                ),
            ] {
                if let Some(mm) = self.length_row(ui, key, id, v, &ctx, enabled)
                    && mm.is_finite()
                    && mm > 0.0
                {
                    if across {
                        set!(Some(key), |p| p.h_spacing_mm = Some(mm));
                    } else {
                        set!(Some(key), |p| p.v_spacing_mm = Some(mm));
                    }
                }
                ui.end_row();
            }
        }
    }

    /// A pattern region's major lines (follow the rhythm, off, or its own N) and the N, inside a two-column grid:
    /// the Style tab's and the Basic tab's (ADR-026).
    pub(crate) fn pattern_major_rows(
        &mut self,
        ui: &mut egui::Ui,
        pr: &PatternRegion,
        enabled: bool,
    ) {
        let Some(rhythm) = self.viewer.doc().map(|d| d.rhythm.clone()) else {
            return;
        };
        // Edits one pattern setting on every selected region.
        macro_rules! set {
            ($key:expr, |$pr:ident| $body:expr) => {{
                let _ = self.viewer.edit_selected($key, |x| {
                    if let ObjectKind::Pattern($pr) = &mut x.kind {
                        $body;
                    }
                });
            }};
        }
        // Major lines: follow the rhythm, off, or its own N; hex and the music staff have
        // none, so the fields are greyed (Defaulted "Which patterns have major lines").
        let has_major = pr.pattern.has_major();
        let on = enabled && has_major;
        let label = ui.label(self.t("app-pattern-major"));
        let mode = |m: MajorLines| match m {
            MajorLines::Follow => 0,
            MajorLines::Off => 1,
            MajorLines::Every(_) => 2,
        };
        let mut want = mode(pr.major);
        let options = [
            (0, "app-major-follow"),
            (1, "app-major-off"),
            (2, "app-major-own"),
        ];
        ui.add_enabled_ui(on, |ui| {
            self.combo(ui, "pattern-major", label.id, &mut want, &options);
        });
        if want != mode(pr.major) {
            let new = match want {
                0 => MajorLines::Follow,
                1 => MajorLines::Off,
                _ => MajorLines::Every(rhythm.major_on().unwrap_or(5)),
            };
            set!(None, |p| p.major = new);
        }
        ui.end_row();
        if let MajorLines::Every(n) = pr.major {
            let label = ui.label(self.t("app-pattern-major-n"));
            let mut v = i32::from(n);
            let drag = egui::DragValue::new(&mut v)
                .range(i32::from(quadrule_model::MAJOR_MIN)..=i32::from(quadrule_model::MAJOR_MAX));
            if ui.add_enabled(on, drag).labelled_by(label.id).changed() {
                let n = u8::try_from(v).unwrap_or(quadrule_model::MAJOR_MAX);
                set!(Some("pattern-major-n"), |p| p.major = MajorLines::Every(n));
            }
            ui.end_row();
        }
    }

    /// The Border section of a pattern region's Style tab (pattern borders bbq, Defaulted "The inspector"): a tick,
    /// and, with it on, a tick for each side, the line (a role, or a width and gray of its own) and the corner
    /// radius. Each edit is one undo step (typing in a field merges), applied to every region picked.
    fn border_rows(
        &mut self,
        ui: &mut egui::Ui,
        pr: &PatternRegion,
        cx: &UnitContext,
        enabled: bool,
    ) {
        // Edits the border of every picked region that has one.
        macro_rules! set {
            ($key:expr, |$b:ident| $body:expr) => {{
                let _ = self.viewer.edit_selected($key, |x| {
                    if let ObjectKind::Pattern(p) = &mut x.kind
                        && let Some($b) = &mut p.border
                    {
                        $body;
                    }
                });
            }};
        }
        egui::Grid::new("border-grid")
            .num_columns(2)
            .show(ui, |ui| {
                let label = ui.label(self.t("app-border"));
                let mut on = pr.border.is_some();
                if ui
                    .add_enabled(enabled, egui::Checkbox::new(&mut on, ""))
                    .labelled_by(label.id)
                    .changed()
                {
                    let _ = self.viewer.edit_selected(None, |x| {
                        if let ObjectKind::Pattern(p) = &mut x.kind {
                            p.border = on.then(quadrule_model::Border::new);
                        }
                    });
                }
                ui.end_row();
                let Some(b) = &pr.border else {
                    return;
                };
                // Which sides: four ticks on one row.
                let label = ui.label(self.t("app-border-sides"));
                let sides = b.sides;
                let mut want = sides;
                ui.horizontal(|ui| {
                    for (id, flag) in [
                        ("app-border-top", &mut want.top),
                        ("app-border-right", &mut want.right),
                        ("app-border-bottom", &mut want.bottom),
                        ("app-border-left", &mut want.left),
                    ] {
                        ui.add_enabled(enabled, egui::Checkbox::new(flag, self.t(id)));
                    }
                })
                .response
                .labelled_by(label.id);
                if want != sides {
                    set!(None, |b| b.sides = want);
                }
                ui.end_row();

                // The line: a role or values of its own.
                let label = ui.label(self.t("app-border-line"));
                let was = StrokeChoice::of(b.style);
                let mut choice = was;
                ui.horizontal(|ui| {
                    ui.radio_value(&mut choice, StrokeChoice::Role, self.t("app-style-role"));
                    ui.radio_value(
                        &mut choice,
                        StrokeChoice::Custom,
                        self.t("app-style-custom"),
                    );
                })
                .response
                .labelled_by(label.id);
                if choice != was {
                    let device = self.viewer.profiles().map(|(_, shown)| shown);
                    let style = match (choice, b.style) {
                        (StrokeChoice::Custom, Style::Role(r)) => {
                            let (w, g) = device.as_ref().map_or((1, 0), |p| {
                                let s = p.role(r);
                                (s.width_px.max(1), s.gray)
                            });
                            Style::Custom {
                                width_mm: w as f64 / cx.px_per_mm,
                                gray: g,
                            }
                        }
                        _ => Style::Role(LineRole::Structure),
                    };
                    set!(None, |b| b.style = style);
                }
                ui.end_row();
                match b.style {
                    Style::Role(role) => {
                        let label = ui.label(self.t("app-style-role"));
                        let mut r = role;
                        self.combo(ui, "border-role", label.id, &mut r, &ROLES);
                        if r != role {
                            set!(None, |b| b.style = Style::Role(r));
                        }
                        ui.end_row();
                    }
                    Style::Custom { width_mm, gray } => {
                        if let Some(mm) = self.length_row(
                            ui,
                            "border-width",
                            "app-stroke-width",
                            width_mm,
                            cx,
                            enabled,
                        ) {
                            set!(Some("border-width"), |b| if let Style::Custom {
                                width_mm, ..
                            } = &mut b.style
                            {
                                *width_mm = mm.max(0.0);
                            });
                        }
                        ui.end_row();
                        let levels: Vec<u8> = self
                            .viewer
                            .profiles()
                            .map(|(_, shown)| shown.gray_levels)
                            .unwrap_or_default();
                        if let Some((g, drag)) = self.gray_field(
                            ui,
                            "border",
                            "app-stroke-gray",
                            gray,
                            false,
                            &levels,
                            enabled,
                        ) {
                            let merge = drag.then_some("border-gray");
                            set!(
                                merge,
                                |b| if let Style::Custom { gray, .. } = &mut b.style {
                                    *gray = g;
                                }
                            );
                        }
                    }
                    Style::None => {}
                }

                if let Some(mm) = self.length_row(
                    ui,
                    "border-radius",
                    "app-corner-radius",
                    b.corner_radius_mm,
                    cx,
                    enabled,
                ) {
                    set!(Some("border-radius"), |b| b.corner_radius_mm = mm.max(0.0));
                }
                ui.end_row();
            });
    }

    /// A row with a whole number of rhythm units (1 to 99) in a drag field; the new value when it
    /// changed.
    fn units_row(
        &mut self,
        ui: &mut egui::Ui,
        label_id: &str,
        value: u32,
        enabled: bool,
    ) -> Option<u32> {
        let label = ui.label(self.t(label_id));
        let mut v = value;
        let drag = egui::DragValue::new(&mut v)
            .range(1..=quadrule_model::PATTERN_UNITS_MAX)
            .suffix(" u");
        let changed = ui
            .add_enabled(enabled, drag)
            .labelled_by(label.id)
            .changed();
        changed.then_some(v)
    }

    /// The fill: none, a role (each device gives it a gray) or a gray of its own.
    /// The Opacity row: 0 to 100 percent, for every kind; the export is flattened onto what lies
    /// beneath (rung 6b). A group fades as one (rung 7a bbq, Q7).
    fn opacity_row(&mut self, ui: &mut egui::Ui, o: &Object, mixed: bool) {
        let label = ui.label(self.t("app-opacity"));
        let mut pct = o.opacity();
        let mut drag = egui::DragValue::new(&mut pct)
            .range(quadrule_model::OPACITY_MIN..=quadrule_model::OPACITY_MAX);
        if mixed {
            // The word stands in for the number (rung 8a: "mixed" where the picked objects differ).
            let word = self.t("app-mixed");
            drag = drag
                .custom_formatter(move |_, _| word.clone())
                .custom_parser(|t| {
                    t.trim()
                        .trim_start_matches(|c: char| c.is_alphabetic())
                        .trim_end_matches('%')
                        .trim()
                        .parse()
                        .ok()
                });
        } else {
            drag = drag.suffix(" %");
        }
        if ui.add(drag).labelled_by(label.id).changed() {
            let _ = self.viewer.edit_selected(Some("style-opacity"), |x| {
                x.opacity_pct = pct;
            });
        }
        ui.end_row();
    }

    pub(crate) fn fill_rows(&mut self, ui: &mut egui::Ui, o: &Object, _enabled: bool) {
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
            let device = self.viewer.profiles().map(|(_, shown)| shown);
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
            let levels: Vec<u8> = self
                .viewer
                .profiles()
                .map(|(_, shown)| shown.gray_levels)
                .unwrap_or_default();
            if let Some((g, drag)) =
                self.gray_field(ui, "fill", "app-fill-gray", gray, false, &levels, true)
            {
                let merge = drag.then_some("fill-gray");
                let _ = self.viewer.edit_selected(merge, |x| {
                    if x.kind.is_shape() {
                        x.fill = Some(Fill::Gray(g));
                    }
                });
            }
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

    /// A text box's text, in its field with the emoji button under it: the Text tab's and the Basic tab's (ADR-026).
    pub(crate) fn text_content_field(&mut self, ui: &mut egui::Ui, t: &quadrule_model::TextBox) {
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
        self.text_field_id = Some(field.id);
        if field.changed() {
            let _ = self.viewer.edit_selected(Some("text-content"), |x| {
                if let ObjectKind::Text(t) = &mut x.kind {
                    t.content.clone_from(&content);
                }
            });
        }
        self.emoji_button(ui);
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
        // A box drawn from saved outlines is read-only (rung 12a bbq, Q6): the badge says why and
        // offers the way out; the fields below are greyed.
        let read_only = self.viewer.is_read_only_text(o.id);
        if read_only {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    self.strings.format(
                        &Message::new("app-text-saved-outlines").text("font", t.family.clone()),
                    ),
                );
                if ui.button(self.t("app-choose-a-font")).clicked() {
                    self.open_missing_dialog();
                }
            });
            ui.add_space(4.0);
        }
        let enabled = !self.viewer.editable_selection().is_empty() && !read_only;
        let edit_text =
            |v: &mut crate::viewer::Viewer, key: &str, f: &dyn Fn(&mut quadrule_model::TextBox)| {
                let _ = v.edit_selected(Some(key), |x| {
                    if let ObjectKind::Text(t) = &mut x.kind {
                        f(t);
                    }
                });
            };
        ui.add_enabled_ui(enabled, |ui| {
            self.text_content_field(ui, t);
            ui.add_space(4.0);
            egui::Grid::new("text-grid").num_columns(2).show(ui, |ui| {
                // Font: the family in use, and the button that opens the picker (rung 11a).
                let label = ui.label(self.t("app-font"));
                ui.vertical(|ui| self.font_button(ui, label.id, &t.family));
                ui.end_row();

                let weights = self.viewer.fonts_weights(&t.family);
                let label = ui.label(self.t("app-weight"));
                let mut w = t.weight;
                let (lo, hi) = weights
                    .as_ref()
                    .map_or((100, 900), |w| (w.min().min(w.max()), w.max()));
                ui.vertical(|ui| {
                    if ui
                        .add(
                            egui::DragValue::new(&mut w)
                                .range(lo..=hi)
                                .clamp_existing_to_range(false)
                                .speed(5),
                        )
                        .labelled_by(label.id)
                        .changed()
                    {
                        edit_text(&mut self.viewer, "text-weight", &|t| t.weight = w);
                    }
                    // A weight the family does not offer is drawn at the nearest one it does.
                    if let Some(used) = weights.as_ref().map(|x| x.clamp(t.weight))
                        && used != t.weight
                    {
                        ui.label(
                            self.strings.format(
                                &Message::new("app-weight-clamped")
                                    .num("min", f64::from(lo), 0)
                                    .num("max", f64::from(hi), 0)
                                    .num("used", f64::from(used), 0),
                            ),
                        );
                    }
                });
                ui.end_row();

                // Style: the family's own italic face; a family without one stays upright.
                let label = ui.label(self.t("app-style"));
                let has_italic = self.viewer.fonts_has_italic(&t.family);
                let mut st = t.style;
                ui.vertical(|ui| {
                    ui.add_enabled_ui(has_italic, |ui| {
                        let options = [
                            (quadrule_model::FontStyle::Normal, "app-style-normal"),
                            (quadrule_model::FontStyle::Italic, "app-style-italic"),
                        ];
                        self.combo(ui, "font-style", label.id, &mut st, &options);
                    });
                    if !has_italic {
                        ui.label(self.t("app-style-no-italic"));
                    }
                });
                if st != t.style {
                    edit_text(&mut self.viewer, "text-style", &|t| t.style = st);
                }
                ui.end_row();

                // Size is in points, whatever the display unit; any unit can be typed.
                let pt = UnitContext {
                    display: Unit::Pt,
                    ..cx
                };
                // On a secondary device the size shown is that device's, and typing one sets an
                // override there (rung 8a, Q3).
                let device = self.viewer.override_device();
                let font_override = device
                    .as_deref()
                    .and_then(|d| o.override_on(d))
                    .and_then(|x| x.font_size_pt);
                let size_mm = quadrule_model::pt_to_mm(font_override.unwrap_or(t.size_pt));
                if let Some(mm) = self.length_row_badged(
                    ui,
                    "text-size",
                    "app-size",
                    size_mm,
                    &pt,
                    enabled,
                    false,
                    font_override.is_some(),
                ) && mm > 0.0
                {
                    let size = mm / quadrule_model::pt_to_mm(1.0);
                    if device.is_some() {
                        let _ = self.viewer.set_font_size_on_device(Some("text-size"), size);
                    } else {
                        edit_text(&mut self.viewer, "text-size", &|t| t.size_pt = size);
                    }
                }
                ui.end_row();

                let label = ui.label(self.t("app-line-height"));
                let mut lh = t.line_height;
                if t.baselines {
                    // On the rhythm the line height is a whole number of its steps: shown as the
                    // device on screen resolves it, and not typed (rung 11b).
                    let size_mm = quadrule_model::pt_to_mm(font_override.unwrap_or(t.size_pt));
                    let step = cx.rhythm_mm;
                    let n = if step > 0.0 {
                        (size_mm * t.line_height / step).round().max(1.0)
                    } else {
                        1.0
                    };
                    let mut shown = if size_mm > 0.0 {
                        n * step / size_mm
                    } else {
                        lh
                    };
                    ui.vertical(|ui| {
                        ui.add_enabled(false, egui::DragValue::new(&mut shown).fixed_decimals(2))
                            .labelled_by(label.id);
                        ui.label(
                            RichText::new(
                                self.strings
                                    .format(&Message::new("app-line-height-steps").num("n", n, 0)),
                            )
                            .small(),
                        );
                    });
                } else if ui
                    .add(egui::DragValue::new(&mut lh).range(0.5..=4.0).speed(0.01))
                    .labelled_by(label.id)
                    .changed()
                {
                    edit_text(&mut self.viewer, "text-line-height", &|t| {
                        t.line_height = lh
                    });
                }
                ui.end_row();

                // Every baseline on a rung of the rhythm (rung 11b).
                ui.label("");
                let mut on = t.baselines;
                let tick = ui.checkbox(&mut on, self.t("app-baselines"));
                if tick.changed() {
                    edit_text(&mut self.viewer, "text-baselines", &|t| t.baselines = on);
                }
                ui.end_row();

                // A small clear gap above the rule instead of standing on its top edge; greyed
                // while the baselines are off (chris, at the 11b look).
                ui.label("");
                let mut gap = t.baseline_gap;
                let tick = ui
                    .add_enabled_ui(t.baselines, |ui| {
                        ui.checkbox(&mut gap, self.t("app-baselines-gap"))
                    })
                    .inner;
                if tick.changed() {
                    edit_text(&mut self.viewer, "text-baseline-gap", &|t| {
                        t.baseline_gap = gap
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

                // The space after each paragraph but the last: any unit; a length is kept in points,
                // `u` in rhythm steps (it follows the rhythm; rung 11b).
                let spacing = match t.paragraph_spacing {
                    quadrule_model::ParagraphSpacing::Units(u) => quadrule_model::Spacing::Units(u),
                    quadrule_model::ParagraphSpacing::Pt(p) => {
                        quadrule_model::Spacing::Mm(quadrule_model::pt_to_mm(p))
                    }
                };
                if let Some(new) =
                    self.spacing_row(ui, "text-paragraph", "app-paragraph-spacing", spacing, &cx)
                {
                    let next = match new {
                        quadrule_model::Spacing::Units(u) => {
                            Some(quadrule_model::ParagraphSpacing::Units(u.max(0.0)))
                        }
                        quadrule_model::Spacing::Mm(m) => {
                            Some(quadrule_model::ParagraphSpacing::Pt(
                                (m / quadrule_model::pt_to_mm(1.0)).max(0.0),
                            ))
                        }
                        quadrule_model::Spacing::Auto => None,
                    };
                    if let Some(next) = next {
                        edit_text(&mut self.viewer, "text-paragraph", &|t| {
                            t.paragraph_spacing = next
                        });
                    }
                }
                ui.end_row();

                // Where the text sits in a Fixed box. A box as tall as its text has no room to move
                // it, so the choice is greyed there, with the reason.
                let label = ui.label(self.t("app-valign"));
                let mut va = t.v_align;
                let fixed = t.sizing == quadrule_model::TextSizing::Fixed;
                ui.vertical(|ui| {
                    ui.add_enabled_ui(fixed, |ui| {
                        self.combo(
                            ui,
                            "valign",
                            label.id,
                            &mut va,
                            &[
                                (quadrule_model::VerticalAlign::Top, "app-valign-top"),
                                (quadrule_model::VerticalAlign::Middle, "app-valign-middle"),
                                (quadrule_model::VerticalAlign::Bottom, "app-valign-bottom"),
                            ],
                        );
                    });
                    if !fixed {
                        ui.label(RichText::new(self.t("app-valign-auto-note")).small());
                    }
                });
                if va != t.v_align {
                    edit_text(&mut self.viewer, "text-valign", &|t| t.v_align = va);
                }
                ui.end_row();

                // Sizing: as tall as the text (the default) or a height of its own (rung 11b).
                let label = ui.label(self.t("app-sizing"));
                let mut sz = t.sizing;
                let options = [
                    (quadrule_model::TextSizing::Auto, "app-sizing-auto"),
                    (quadrule_model::TextSizing::Fixed, "app-sizing-fixed"),
                ];
                self.combo(ui, "sizing", label.id, &mut sz, &options);
                if sz != t.sizing {
                    let _ = self.viewer.set_text_sizing(sz);
                }
                ui.end_row();
            });
        });
    }

    fn constraints_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.constraints_subject() else {
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

    /// One plain line per device of the page: the checks that did not run on it and why (a paper size, a
    /// Wallpaper, a rule the page ignores, no clean grays, no value in the profile). A device where
    /// every check ran says so.
    pub fn lint_skipped_lines(&self) -> Vec<String> {
        self.lint_skipped_lines_in(&self.viewer.profile_ids())
    }

    /// [`ViewerUi::lint_skipped_lines`] for the devices `ids` only (the Lint tab's device picker).
    pub fn lint_skipped_lines_in(&self, ids: &[String]) -> Vec<String> {
        use quadrule_lint::{SkipReason, skipped_checks};
        let Some(doc) = self.viewer.doc() else {
            return Vec::new();
        };
        doc.profile_ids()
            .into_iter()
            .filter(|id| ids.iter().any(|i| i == id))
            .filter_map(|id| {
                let target = quadrule_profiles::active::lookup_in(doc, id).ok()?;
                let device = self.device_name(id);
                let skipped = skipped_checks(doc, &target);
                if skipped.is_empty() {
                    return Some(
                        self.strings
                            .format(&Message::new("lint-skipped-none").text("device", device)),
                    );
                }
                let items = skipped
                    .iter()
                    .map(|k| {
                        let why = match k.reason {
                            SkipReason::Paper => "lint-skip-paper",
                            SkipReason::Wallpaper => "lint-skip-wallpaper",
                            SkipReason::Ignored => "lint-skip-ignored",
                            SkipReason::NoGrayLevels => "lint-skip-no-grays",
                            SkipReason::NoValue => "lint-skip-no-value",
                        };
                        Message::new("lint-skipped-item")
                            .text("rule", k.rule)
                            .arg("why", quadrule_i18n::Arg::Msg(Box::new(Message::new(why))))
                    })
                    .collect();
                Some(
                    self.strings
                        .format(&Message::new("lint-skipped").text("device", device).arg(
                            "items",
                            quadrule_i18n::Arg::List {
                                items,
                                sep: "lint-skipped-sep",
                            },
                        )),
                )
            })
            .collect()
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
        self.length_row_mixed(ui, key, label_id, mm, ctx, enabled, false)
    }

    /// [`ViewerUi::length_row`], and with `mixed` (the selected objects differ) the field reads
    /// "mixed" until it is focused, then is empty; a length typed applies to all of them (rung 7b,
    /// Defaulted "7b pays the rung 3 debt").
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn length_row_mixed(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        label_id: &str,
        mm: f64,
        ctx: &UnitContext,
        enabled: bool,
        mixed: bool,
    ) -> Option<f64> {
        self.length_row_badged(ui, key, label_id, mm, ctx, enabled, mixed, false)
    }

    /// [`ViewerUi::length_row_mixed`], and with `badge` a small dot after the field says the value
    /// is changed on the secondary device on screen (rung 8a, Defaulted "Badges").
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn length_row_badged(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        label_id: &str,
        mm: f64,
        ctx: &UnitContext,
        enabled: bool,
        mixed: bool,
        badge: bool,
    ) -> Option<f64> {
        let label = ui.label(self.t(label_id));
        let id = ui.make_persistent_id(("length", key));
        let focused = ui.memory(|m| m.has_focus(id));
        let mut text = match (focused, self.field_text.get(key)) {
            (true, Some(t)) => t.clone(),
            (true, None) if mixed => String::new(),
            (false, _) if mixed => self.t("app-mixed"),
            _ => format_length(mm, ctx),
        };
        let parsed = parse_length(&text, ctx);
        let bad = focused && parsed.is_err() && !(mixed && text.is_empty());
        let mut edit = egui::TextEdit::singleline(&mut text)
            .id(id)
            .desired_width(110.0);
        if mixed {
            edit = edit.hint_text(self.t("app-mixed"));
        }
        if bad {
            edit = edit.background_color(ui.visuals().error_fg_color.gamma_multiply(0.25));
        }
        let resp = ui
            .horizontal(|ui| {
                let resp = ui.add_enabled(enabled, edit).labelled_by(label.id);
                if badge {
                    self.paint_badge(ui);
                }
                resp
            })
            .inner;
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

    /// The small filled dot that marks a value changed on the secondary device on screen, with the
    /// tooltip "Changed on Nomad" (and the same words for screen readers).
    pub(crate) fn paint_badge(&self, ui: &mut egui::Ui) {
        let device = self.device_name(&self.viewer.profile_id.clone());
        let tip = self
            .strings
            .format(&Message::new("app-override-badge").text("device", device));
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 3.5, ui.visuals().selection.stroke.color);
        resp.widget_info(|| WidgetInfo::labeled(Role::Image, true, &tip));
        resp.on_hover_text(tip);
    }

    /// The Overrides tab (spec §6.1; rung 8a Defaulted "The Overrides tab"): with a secondary device
    /// on screen and one object picked, each part of it changed on that device with its value
    /// there and in the design, and a Reset beside each; Reset all on this device for everything
    /// picked.
    fn overrides_tab(&mut self, ui: &mut egui::Ui) {
        let Some(dev) = self.viewer.override_device() else {
            return;
        };
        let name = self.device_name(&dev);
        ui.label(
            self.strings
                .format(&Message::new("app-overrides-intro").text("device", name.clone())),
        );
        let selection = self.viewer.editable_selection();
        let all = self.viewer.selection.clone();
        let (Some(cx), Some(cy)) = (
            self.viewer.unit_context(Axis::X),
            self.viewer.unit_context(Axis::Y),
        ) else {
            return;
        };
        if let [id] = all[..] {
            let Some(o) = self.viewer.doc().and_then(|d| d.object(id)).cloned() else {
                return;
            };
            let base = self
                .viewer
                .page()
                .and_then(|p| p.placed.iter().find(|q| q.id == id).map(|q| q.node));
            let here = |sel: &mut Self,
                        field: OverrideField,
                        what: String,
                        a: String,
                        b: String,
                        ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    ui.strong(what.clone());
                });
                ui.label(
                    sel.strings
                        .format(&Message::new("app-override-here").text("value", a)),
                );
                ui.label(
                    sel.strings
                        .format(&Message::new("app-override-linked").text("value", b)),
                );
                let reset = sel.strings.format(
                    &Message::new("app-override-reset-name")
                        .text("what", what)
                        .text("device", name.clone()),
                );
                let btn = ui.button(sel.t("app-override-reset"));
                btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, &reset));
                if btn.on_hover_text(reset).clicked() {
                    let _ = sel.viewer.reset_overrides(&[id], Some(&[field]));
                }
                ui.separator();
            };
            let ov = o.override_on(&dev).cloned().unwrap_or_default();
            if ov.is_empty() {
                ui.label(
                    self.strings
                        .format(&Message::new("app-overrides-none").text("device", name.clone())),
                );
            }
            let len = |mm: f64, c: &UnitContext| format_length(mm, c);
            if let Some(p) = ov.position {
                let linked = base.map_or((0.0, 0.0), |n| (n.base.x_mm, n.base.y_mm));
                here(
                    self,
                    OverrideField::Position,
                    self.t("app-override-position"),
                    format!("{}, {}", len(p.x_mm, &cx), len(p.y_mm, &cy)),
                    format!("{}, {}", len(linked.0, &cx), len(linked.1, &cy)),
                    ui,
                );
            }
            if let Some(sz) = ov.size {
                let linked = base.map_or((0.0, 0.0), |n| (n.base.w_mm, n.base.h_mm));
                here(
                    self,
                    OverrideField::Size,
                    self.t("app-override-size"),
                    format!("{} × {}", len(sz.w_mm, &cx), len(sz.h_mm, &cy)),
                    format!("{} × {}", len(linked.0, &cx), len(linked.1, &cy)),
                    ui,
                );
            }
            if let (Some(pt), ObjectKind::Text(t)) = (ov.font_size_pt, &o.kind) {
                here(
                    self,
                    OverrideField::FontSize,
                    self.t("app-override-font"),
                    format!("{} pt", trim_number(pt, 1)),
                    format!("{} pt", trim_number(t.size_pt, 1)),
                    ui,
                );
            }
            if let Some(v) = ov.visible {
                let word = |sel: &Self, v: bool| {
                    sel.t(if v {
                        "app-override-yes"
                    } else {
                        "app-override-no"
                    })
                };
                here(
                    self,
                    OverrideField::Visible,
                    self.t("app-override-eye"),
                    word(self, v),
                    word(self, o.visible),
                    ui,
                );
            }
        } else {
            ui.label(self.t("app-overrides-many"));
        }
        let reset_all = self
            .strings
            .format(&Message::new("app-override-reset-all-name").text("device", name));
        let btn = ui.add_enabled(
            !selection.is_empty(),
            egui::Button::new(self.t("app-override-reset-all")),
        );
        btn.widget_info(|| WidgetInfo::labeled(Role::Button, !selection.is_empty(), &reset_all));
        if btn.on_hover_text(reset_all).clicked() {
            let _ = self.viewer.reset_overrides(&selection, None);
        }
    }

    /// A labelled choice from `options` (value, message id).
    pub(crate) fn combo<T: Copy + PartialEq>(
        &self,
        ui: &mut egui::Ui,
        salt: &str,
        label: egui::Id,
        value: &mut T,
        options: &[(T, &str)],
    ) {
        self.combo_mixed(ui, salt, label, value, options, false);
    }

    /// [`ViewerUi::combo`], reading "mixed" while the picked objects differ (rung 8a, Defaulted
    /// "Debt paid in code the rung touches").
    fn combo_mixed<T: Copy + PartialEq>(
        &self,
        ui: &mut egui::Ui,
        salt: &str,
        label: egui::Id,
        value: &mut T,
        options: &[(T, &str)],
        mixed: bool,
    ) {
        let shown = if mixed {
            self.t("app-mixed")
        } else {
            options
                .iter()
                .find(|(v, _)| v == value)
                .map(|(_, id)| self.t(id))
                .unwrap_or_default()
        };
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

/// The width of the Page tab's PPI choice, in points: it must leave room for the tick and Make primary on one line.
const PPI_COMBO_WIDTH: f32 = 64.0;

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
/// The seven weekdays as a combo's choices, by their whole names, Monday first.
fn weekday_options() -> [(quadrule_model::Weekday, &'static str); 7] {
    use quadrule_model::Weekday;
    [
        (Weekday::Monday, "calendar-day-mon"),
        (Weekday::Tuesday, "calendar-day-tue"),
        (Weekday::Wednesday, "calendar-day-wed"),
        (Weekday::Thursday, "calendar-day-thu"),
        (Weekday::Friday, "calendar-day-fri"),
        (Weekday::Saturday, "calendar-day-sat"),
        (Weekday::Sunday, "calendar-day-sun"),
    ]
}

/// The year it is now, by this computer's clock: what a calendar dated for the first time takes.
fn this_month_year() -> i32 {
    crate::draw::this_month().year
}

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
        ObjectKind::Pattern(p) if p.pattern.is_dotted() => LineRole::DotGrid,
        ObjectKind::Rect(_) | ObjectKind::Ellipse(_) | ObjectKind::Polygon(_) => {
            LineRole::Structure
        }
        _ => LineRole::Guide,
    }
}
