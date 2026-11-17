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

//! The Style tab's gray swatches (rung 14b, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q8): under Custom
//! the gray is the profile's clean gray levels as swatches (two rows of eight on the Nomad and Manta; however many
//! the profile lists), the chosen one outlined, with Show all grays opening the 0 to 255 slider; a profile with
//! no gray levels (paper) shows the slider only. Stroke, fill and text grays use the same field. A "matches
//! <Role>" badge shows when the stroke's width and gray equal a role on the device shown; otherwise Use
//! recommended sets the nearest role by width.

use crate::view::ViewerUi;
use egui::{Color32, CornerRadius, Role, Sense, Stroke, StrokeKind, WidgetInfo};
use quadrule_i18n::Message;
use quadrule_model::{LineRole, Object, ObjectKind};
use quadrule_profiles::Profile;

/// Swatches in a row.
pub const SWATCHES_PER_ROW: usize = 8;

/// The line roles an object's stroke may be matched to: the dot grid's for dots, the gray-only roles for text, and the
/// four line roles for anything else. Guide comes first, so it is the pick when two are as near.
pub fn candidate_roles(o: &Object) -> &'static [LineRole] {
    match &o.kind {
        ObjectKind::Text(_) => &[LineRole::Text, LineRole::Label],
        ObjectKind::Pattern(p) if p.pattern.is_dotted() => &[LineRole::DotGrid],
        _ => &[
            LineRole::Guide,
            LineRole::Whisper,
            LineRole::Structure,
            LineRole::Emphasis,
        ],
    }
}

/// The role whose width and gray are exactly what the stroke draws on `shown`, if any.
pub fn matching_role(o: &Object, shown: &Profile, width_px: u32, gray: u8) -> Option<LineRole> {
    let text = matches!(o.kind, ObjectKind::Text(_));
    candidate_roles(o).iter().copied().find(|r| {
        let s = shown.role(*r);
        s.gray == gray && (text || s.width_px.max(1) == width_px)
    })
}

/// The role to recommend: the nearest by width (by gray for text), Guide first when two are as near.
pub fn recommended_role(o: &Object, shown: &Profile, width_px: u32, gray: u8) -> LineRole {
    let text = matches!(o.kind, ObjectKind::Text(_));
    let roles = candidate_roles(o);
    roles
        .iter()
        .copied()
        .min_by_key(|r| {
            let s = shown.role(*r);
            if text {
                i32::from(s.gray).abs_diff(i32::from(gray))
            } else {
                s.width_px.max(1).abs_diff(width_px)
            }
        })
        .unwrap_or(roles[0])
}

/// The id of a role's name.
pub fn role_name_id(r: LineRole) -> &'static str {
    match r {
        LineRole::Whisper => "app-role-whisper",
        LineRole::Guide => "app-role-guide",
        LineRole::Structure => "app-role-structure",
        LineRole::Emphasis => "app-role-emphasis",
        LineRole::DotGrid => "app-role-dotgrid",
        LineRole::Text => "app-role-text",
        LineRole::Label => "app-role-label",
    }
}

impl ViewerUi {
    /// Whether the 0 to 255 slider is open for the gray field `key` ("stroke" or "fill"): the person asked for it,
    /// or the current gray is not one of the clean levels.
    pub fn all_grays_open(&self, key: &str) -> bool {
        self.show_all_grays.contains(key)
    }

    /// Opens or closes the slider of the gray field `key`.
    pub fn set_all_grays(&mut self, key: &'static str, open: bool) {
        if open {
            self.show_all_grays.insert(key);
        } else {
            self.show_all_grays.remove(key);
        }
    }

    /// One gray field as rows of the Style tab's grid: the label and the clean levels as swatches, a Show all
    /// grays tick, and the slider when it is wanted. `name` is the label's id ("app-stroke-gray" or
    /// "app-fill-gray"); the gray picked, if one was, comes back with whether it is a drag (merges) or a click.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn gray_field(
        &mut self,
        ui: &mut egui::Ui,
        key: &'static str,
        name: &str,
        current: u8,
        mixed: bool,
        levels: &[u8],
        enabled: bool,
    ) -> Option<(u8, bool)> {
        let mut picked = None;
        let label_text = self.t(name);
        let label = ui.label(&label_text);
        let on_level = levels.contains(&current) && !mixed;
        let show_slider = levels.is_empty() || self.all_grays_open(key) || (!on_level && !mixed);
        if levels.is_empty() {
            // A profile with no gray levels (paper): the slider only.
            let mut g = current;
            let r = ui
                .add_enabled(enabled, self.gray_slider(&mut g, mixed))
                .labelled_by(label.id);
            if r.changed() {
                picked = Some((g, true));
            }
            ui.end_row();
            return picked;
        }
        ui.vertical(|ui| {
            let size = egui::vec2(20.0, 20.0);
            ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
            for row in levels.chunks(SWATCHES_PER_ROW) {
                ui.horizontal(|ui| {
                    for &g in row {
                        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                        let chosen = !mixed && g == current;
                        let name = format!("{label_text} {g}");
                        resp.widget_info(|| {
                            WidgetInfo::selected(Role::Button, enabled, chosen, &name)
                        });
                        let resp = resp.on_hover_text(g.to_string());
                        let painter = ui.painter();
                        painter.rect_filled(rect, CornerRadius::same(2), Color32::from_gray(g));
                        if chosen {
                            painter.rect_stroke(
                                rect.expand(1.0),
                                CornerRadius::same(2),
                                Stroke::new(2.0, ui.visuals().selection.stroke.color),
                                StrokeKind::Outside,
                            );
                        } else {
                            painter.rect_stroke(
                                rect,
                                CornerRadius::same(2),
                                ui.visuals().widgets.noninteractive.bg_stroke,
                                StrokeKind::Inside,
                            );
                        }
                        if enabled && resp.clicked() {
                            picked = Some((g, false));
                        }
                    }
                });
            }
        });
        ui.end_row();
        // Show all grays: the slider. Held open while the current gray is off the clean levels.
        ui.label("");
        let mut open = show_slider;
        let forced = !on_level && !mixed;
        let tick = ui.add_enabled(
            enabled && !forced,
            egui::Checkbox::new(&mut open, self.t("app-style-show-grays")),
        );
        if tick.changed() {
            self.set_all_grays(key, open);
        }
        ui.end_row();
        if show_slider {
            ui.label("");
            let mut g = current;
            let r = ui
                .add_enabled(enabled, self.gray_slider(&mut g, mixed))
                .labelled_by(label.id);
            if r.changed() {
                picked = Some((g, true));
            }
            ui.end_row();
        }
        picked
    }

    /// The 0 to 255 slider, reading "mixed" while the picked objects differ.
    fn gray_slider<'a>(&self, g: &'a mut u8, mixed: bool) -> egui::Slider<'a> {
        let mut s = egui::Slider::new(g, 0..=255);
        if mixed {
            let word = self.t("app-mixed");
            s = s
                .custom_formatter(move |_, _| word.clone())
                .custom_parser(|t| {
                    t.trim()
                        .trim_start_matches(|c: char| c.is_alphabetic())
                        .trim()
                        .parse()
                        .ok()
                });
        }
        s
    }

    /// The line under a Custom stroke: "matches <Role>" when the stroke is exactly a role on the device shown,
    /// otherwise Use recommended. Returns the role to set when the button was pressed.
    pub(crate) fn role_badge(
        &mut self,
        ui: &mut egui::Ui,
        o: &Object,
        shown: &Profile,
        width_px: u32,
        gray: u8,
        enabled: bool,
    ) -> Option<LineRole> {
        let mut set = None;
        ui.label("");
        match matching_role(o, shown, width_px, gray) {
            Some(r) => {
                let text = self.strings.format(&Message::new("app-style-matches").arg(
                    "role",
                    quadrule_i18n::Arg::Msg(Box::new(Message::new(role_name_id(r)))),
                ));
                ui.label(egui::RichText::new(text).small());
            }
            None => {
                let r = recommended_role(o, shown, width_px, gray);
                let b = ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(self.t("app-style-use-recommended")),
                    )
                    .on_hover_text(self.strings.format(
                        &Message::new("app-style-use-recommended-tip").arg(
                            "role",
                            quadrule_i18n::Arg::Msg(Box::new(Message::new(role_name_id(r)))),
                        ),
                    ));
                if b.clicked() {
                    set = Some(r);
                }
            }
        }
        ui.end_row();
        set
    }
}
