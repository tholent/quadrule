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

//! The inspector's tab buttons as one row of line icons (rung 17 bbq, Q8 and Defaulted 3), drawn from lines the way
//! the layers panel's icons are (`layer_icons`): no icon font. Each is a 26-point button named for screen readers and
//! on hover; the open one is filled as a selected button is; Lint keeps its count beside its icon.

use crate::layer_icons::{self, BUTTON_SIDE, LayerIcon};
use crate::panels::Tab;
use crate::view::ViewerUi;
use egui::{
    Color32, Painter, Pos2, Rect, Response, Role, Sense, Shape, Stroke, StrokeKind, WidgetInfo,
    pos2, vec2,
};

/// Paints the icon for `tab` centred at `c`, in `color`.
pub(crate) fn paint(painter: &Painter, tab: Tab, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let thin = Stroke::new(1.2, color);
    let at = |x: f32, y: f32| pos2(c.x + x, c.y + y);
    let line = |a: (f32, f32), b: (f32, f32), s: Stroke| {
        painter.line_segment([at(a.0, a.1), at(b.0, b.1)], s);
    };
    match tab {
        Tab::Basic => {
            // A star: what is marked to show first.
            let pts: Vec<Pos2> = (0..10)
                .map(|i| {
                    let r = if i % 2 == 0 { 9.5 } else { 4.0 };
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                    at(r * a.cos(), r * a.sin() + 0.5)
                })
                .collect();
            painter.add(Shape::closed_line(pts, thin));
        }
        Tab::Page => {
            // A sheet with its corner turned down.
            painter.add(Shape::closed_line(
                vec![
                    at(-7.0, -9.0),
                    at(2.0, -9.0),
                    at(7.0, -4.0),
                    at(7.0, 9.0),
                    at(-7.0, 9.0),
                ],
                stroke,
            ));
            painter.add(Shape::line(
                vec![at(2.0, -9.0), at(2.0, -4.0), at(7.0, -4.0)],
                thin,
            ));
        }
        Tab::Style => {
            // A drop of ink: a point on top, a round belly.
            let (cy, r) = (3.5_f32, 6.0_f32);
            let tangent = (r / (cy + 9.0)).acos();
            let from = -std::f32::consts::FRAC_PI_2 + tangent;
            let to = from + 2.0 * (std::f32::consts::PI - tangent);
            let mut pts = vec![at(0.0, -9.0)];
            let steps = 14;
            for i in 0..=steps {
                let a = from + (to - from) * i as f32 / steps as f32;
                pts.push(at(r * a.cos(), cy + r * a.sin()));
            }
            painter.add(Shape::closed_line(pts, stroke));
        }
        Tab::Arrange => {
            // Four arrows out from the middle: position and size.
            line((-9.0, 0.0), (9.0, 0.0), thin);
            line((0.0, -9.0), (0.0, 9.0), thin);
            for (sx, sy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                // An arrowhead at (9 sx, 9 sy), turned to point along (sx, sy).
                let tip = (9.0 * sx, 9.0 * sy);
                let back = |along: f32, across: f32| {
                    (
                        tip.0 - sx * along - sy * across,
                        tip.1 - sy * along + sx * across,
                    )
                };
                painter.add(Shape::line(
                    vec![
                        at(back(4.0, -3.5).0, back(4.0, -3.5).1),
                        at(tip.0, tip.1),
                        at(back(4.0, 3.5).0, back(4.0, 3.5).1),
                    ],
                    stroke,
                ));
            }
        }
        Tab::Text => {
            // A capital T.
            line((-7.0, -8.0), (7.0, -8.0), stroke);
            line((0.0, -8.0), (0.0, 8.0), stroke);
            line((-3.5, 8.0), (3.5, 8.0), thin);
        }
        Tab::Repeat => layer_icons::paint(painter, LayerIcon::Repeat, c, color),
        Tab::Constraints => {
            // A box held to the page's edges by two lines.
            painter.rect_stroke(
                Rect::from_min_max(at(-10.0, -9.0), at(10.0, 9.0)),
                1.5,
                thin,
                StrokeKind::Middle,
            );
            painter.rect_stroke(
                Rect::from_min_max(at(-2.0, -2.0), at(6.0, 5.0)),
                1.0,
                stroke,
                StrokeKind::Middle,
            );
            line((-10.0, 1.5), (-2.0, 1.5), thin);
            line((2.0, -9.0), (2.0, -2.0), thin);
        }
        Tab::Overrides => {
            // Two screens side by side, the second one marked as set apart.
            painter.rect_stroke(
                Rect::from_min_max(at(-10.0, -7.0), at(-2.0, 7.0)),
                1.5,
                thin,
                StrokeKind::Middle,
            );
            painter.rect_stroke(
                Rect::from_min_max(at(2.0, -7.0), at(10.0, 7.0)),
                1.5,
                stroke,
                StrokeKind::Middle,
            );
            painter.circle_filled(at(6.0, 0.0), 2.2, color);
        }
        Tab::Lint => {
            // A warning triangle.
            painter.add(Shape::closed_line(
                vec![at(0.0, -9.0), at(9.5, 7.5), at(-9.5, 7.5)],
                stroke,
            ));
            line((0.0, -3.0), (0.0, 2.0), stroke);
            painter.circle_filled(at(0.0, 5.0), 1.0, color);
        }
    }
}

impl ViewerUi {
    /// One tab button: named `name` for screen readers and on hover, filled when `selected`; `count` shows
    /// beside the icon (Lint's findings). Returns whether it was clicked.
    pub(crate) fn tab_icon_button(
        &self,
        ui: &mut egui::Ui,
        tab: Tab,
        name: &str,
        selected: bool,
        count: Option<usize>,
    ) -> bool {
        let galley = count.map(|n| {
            ui.painter().layout_no_wrap(
                n.to_string(),
                egui::TextStyle::Small.resolve(ui.style()),
                Color32::PLACEHOLDER,
            )
        });
        let width = BUTTON_SIDE + galley.as_ref().map_or(0.0, |g| g.size().x + 2.0);
        let (rect, response) = ui.allocate_exact_size(vec2(width, BUTTON_SIDE), Sense::click());
        response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, name));
        let v = ui.visuals();
        let (fill, color) = if selected {
            (v.selection.bg_fill, v.selection.stroke.color)
        } else if response.hovered() {
            (
                v.widgets.hovered.weak_bg_fill,
                v.widgets.hovered.fg_stroke.color,
            )
        } else {
            (Color32::TRANSPARENT, v.widgets.inactive.fg_stroke.color)
        };
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, fill);
        let icon_centre = pos2(rect.min.x + BUTTON_SIDE / 2.0, rect.center().y);
        paint(painter, tab, icon_centre, color);
        if let Some(g) = galley {
            let at = pos2(
                rect.min.x + BUTTON_SIDE - 2.0,
                rect.center().y - g.size().y / 2.0,
            );
            painter.galley(at, g, color);
        }
        let response: Response = response.on_hover_text(name);
        response.clicked()
    }
}
