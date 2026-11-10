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

//! The tools on the left bar (rung 5 step 2, docs/bbq/2026-11-10-rung-5-new-and-insert.md,
//! Defaulted "The tools bar"). The bar is a list, so a later tool (rectangle, ellipse, polygon,
//! repeat; Q12) is one more entry in [`Tool::ALL`] and one more icon.

use crate::draw::DrawKind;
use crate::view::ViewerUi;
use egui::{Color32, Key, Pos2, Role, Sense, Shape, Stroke, WidgetInfo, pos2, vec2};

/// A tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Tool {
    /// Click, drag and resize what is on the page.
    #[default]
    Select,
    /// Draw a line.
    Line,
    /// Draw a text box.
    Text,
    /// Draw a pattern region.
    Pattern,
}

impl Tool {
    /// Every tool, in the order of the bar.
    pub const ALL: [Tool; 4] = [Tool::Select, Tool::Line, Tool::Text, Tool::Pattern];

    /// The Fluent message that names it for screen readers (`crates/i18n/locales/en-US/app.ftl`).
    pub fn message_id(self) -> &'static str {
        match self {
            Tool::Select => "tool-select",
            Tool::Line => "tool-line",
            Tool::Text => "tool-text",
            Tool::Pattern => "tool-pattern",
        }
    }

    /// The Fluent message of its tooltip, which gives its shortcut.
    pub fn tip_id(self) -> &'static str {
        match self {
            Tool::Select => "tool-select-tip",
            Tool::Line => "tool-line-tip",
            Tool::Text => "tool-text-tip",
            Tool::Pattern => "tool-pattern-tip",
        }
    }

    /// Its shortcut key (V, L, T, P).
    pub fn key(self) -> Key {
        match self {
            Tool::Select => Key::V,
            Tool::Line => Key::L,
            Tool::Text => Key::T,
            Tool::Pattern => Key::P,
        }
    }

    /// What it draws, or `None` for Select.
    pub fn draws(self) -> Option<DrawKind> {
        match self {
            Tool::Select => None,
            Tool::Line => Some(DrawKind::Line),
            Tool::Text => Some(DrawKind::Text),
            Tool::Pattern => Some(DrawKind::Pattern),
        }
    }
}

/// The width of the bar in screen points.
pub const BAR_WIDTH: f32 = 48.0;

/// A tool's icon, drawn from lines, in a box of side `s` centred on `c`.
fn paint_icon(painter: &egui::Painter, tool: Tool, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let at = |x: f32, y: f32| pos2(c.x + x, c.y + y);
    match tool {
        Tool::Select => {
            // An arrow pointer.
            let pts = vec![
                at(-5.0, -9.0),
                at(-5.0, 7.0),
                at(-1.5, 3.5),
                at(1.5, 10.0),
                at(4.0, 9.0),
                at(1.0, 2.5),
                at(6.0, 2.5),
            ];
            painter.add(Shape::closed_line(pts, stroke));
        }
        Tool::Line => {
            painter.line_segment([at(-9.0, -6.0), at(9.0, 6.0)], stroke);
            painter.circle_filled(at(-9.0, -6.0), 2.2, color);
            painter.circle_filled(at(9.0, 6.0), 2.2, color);
        }
        Tool::Text => {
            // A capital T.
            painter.line_segment([at(-7.0, -8.0), at(7.0, -8.0)], stroke);
            painter.line_segment([at(0.0, -8.0), at(0.0, 9.0)], stroke);
        }
        Tool::Pattern => {
            // A square cut by ruled lines.
            let r = egui::Rect::from_center_size(c, vec2(18.0, 18.0));
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Middle);
            for y in [-3.0, 3.0] {
                painter.line_segment([at(-9.0, y), at(9.0, y)], stroke);
            }
        }
    }
}

impl ViewerUi {
    /// The bar: one button per tool, stacked from the top. A click picks the tool; a
    /// double-click keeps it on for several objects, shown by a small dot on the button.
    pub(crate) fn tools_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            for tool in Tool::ALL {
                let on = self.viewer.tool == tool;
                let kept = on && self.viewer.tool_kept;
                let name = self.t(tool.message_id());
                let (rect, response) = ui
                    .allocate_exact_size(vec2(BAR_WIDTH - 12.0, BAR_WIDTH - 12.0), Sense::click());
                // Named for screen readers: a radio button in the group of tools.
                response.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, on, &name));
                let v = ui.visuals();
                let fill = if on {
                    v.selection.bg_fill
                } else if response.hovered() {
                    v.widgets.hovered.weak_bg_fill
                } else {
                    Color32::TRANSPARENT
                };
                let color = if on {
                    v.selection.stroke.color
                } else {
                    v.widgets.inactive.fg_stroke.color
                };
                let painter = ui.painter();
                painter.rect_filled(rect, 4.0, fill);
                paint_icon(painter, tool, rect.center(), color);
                if kept {
                    painter.circle_filled(rect.right_top() + vec2(-6.0, 6.0), 3.0, color);
                }
                let tip = self.t(tool.tip_id());
                let hint = self.t("tools-keep-hint");
                let response = response.on_hover_ui(|ui| {
                    ui.label(tip);
                    if tool != Tool::Select {
                        ui.weak(hint);
                    }
                });
                if response.double_clicked() {
                    self.viewer.keep_tool(tool);
                } else if response.clicked() {
                    self.viewer.set_tool(tool);
                }
                ui.add_space(2.0);
            }
        });
    }
}
