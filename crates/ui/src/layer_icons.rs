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

//! The layers panel's seven buttons as line icons (fixes batch 1 bbq, docs/bbq/2026-11-18-fixes-before-16b.md, Q5), drawn
//! from lines the way the tools bar and the Arrange tab draw theirs (`tools::paint_icon`, `arrange::paint_icon`): no
//! icon font. Each is a 26-point button in one row, named for screen readers and on hover, greyed when it cannot work
//! (as the Arrange tab's buttons are), with the note the text button had on hover under its name.

use crate::view::ViewerUi;
use egui::{
    Color32, Painter, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, WidgetInfo, pos2, vec2,
};

/// One of the layers panel's buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerIcon {
    /// Move up (toward the front).
    MoveUp,
    /// Move down (toward the back).
    MoveDown,
    /// New layer.
    NewLayer,
    /// Delete layer.
    DeleteLayer,
    /// Group the picked objects.
    Group,
    /// Ungroup.
    Ungroup,
    /// Repeat the picked objects.
    Repeat,
}

impl LayerIcon {
    /// The seven, in the order they show.
    pub const ALL: [LayerIcon; 7] = [
        LayerIcon::MoveUp,
        LayerIcon::MoveDown,
        LayerIcon::NewLayer,
        LayerIcon::DeleteLayer,
        LayerIcon::Group,
        LayerIcon::Ungroup,
        LayerIcon::Repeat,
    ];

    /// The message that names the button (its words are the text button's they replace).
    pub fn message_id(self) -> &'static str {
        match self {
            LayerIcon::MoveUp => "app-move-up",
            LayerIcon::MoveDown => "app-move-down",
            LayerIcon::NewLayer => "app-layer-new",
            LayerIcon::DeleteLayer => "app-layer-delete",
            LayerIcon::Group => "app-group",
            LayerIcon::Ungroup => "app-ungroup",
            LayerIcon::Repeat => "app-repeat",
        }
    }
}

/// The side of a button, in points.
pub const BUTTON_SIDE: f32 = 26.0;

/// Paints `icon` centred at `c`, in `color`.
pub(crate) fn paint(painter: &Painter, icon: LayerIcon, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let thin = Stroke::new(1.2, color);
    let at = |x: f32, y: f32| pos2(c.x + x, c.y + y);
    // A layer seen from a little above: a diamond, with a second one's edge showing under it.
    let layer = |dx: f32, dy: f32| {
        painter.add(egui::Shape::closed_line(
            vec![
                at(dx, dy - 5.0),
                at(dx + 8.5, dy - 1.0),
                at(dx, dy + 3.0),
                at(dx - 8.5, dy - 1.0),
            ],
            stroke,
        ));
        painter.add(egui::Shape::line(
            vec![
                at(dx - 8.5, dy + 3.0),
                at(dx, dy + 7.0),
                at(dx + 8.5, dy + 3.0),
            ],
            thin,
        ));
    };
    // The two small boxes a group holds.
    let pair = |gap: f32| {
        painter.rect_stroke(
            Rect::from_min_max(at(-6.0 - gap, -5.0 - gap), at(0.0 - gap, 1.0 - gap)),
            1.0,
            thin,
            StrokeKind::Middle,
        );
        painter.rect_stroke(
            Rect::from_min_max(at(0.0 + gap, -1.0 + gap), at(6.0 + gap, 5.0 + gap)),
            1.0,
            thin,
            StrokeKind::Middle,
        );
    };
    match icon {
        LayerIcon::MoveUp | LayerIcon::MoveDown => {
            let s = if icon == LayerIcon::MoveUp { 1.0 } else { -1.0 };
            painter.line_segment([at(0.0, s * 8.0), at(0.0, s * -8.0)], stroke);
            painter.add(egui::Shape::line(
                vec![at(-6.0, s * -2.0), at(0.0, s * -8.0), at(6.0, s * -2.0)],
                stroke,
            ));
        }
        LayerIcon::NewLayer => {
            layer(-2.0, 2.0);
            // A plus at the top right.
            painter.line_segment([at(7.0, -10.0), at(7.0, -3.0)], stroke);
            painter.line_segment([at(3.5, -6.5), at(10.5, -6.5)], stroke);
        }
        LayerIcon::DeleteLayer => {
            layer(-2.0, 2.0);
            // A cross at the top right.
            painter.line_segment([at(4.0, -10.0), at(10.0, -4.0)], stroke);
            painter.line_segment([at(10.0, -10.0), at(4.0, -4.0)], stroke);
        }
        LayerIcon::Group => {
            // One frame round two boxes.
            painter.rect_stroke(
                Rect::from_min_max(at(-10.0, -9.0), at(10.0, 9.0)),
                2.0,
                stroke,
                StrokeKind::Middle,
            );
            pair(0.0);
        }
        LayerIcon::Ungroup => {
            // The frame gone but its corners, the boxes apart.
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                painter.line_segment([at(sx * 10.0, sy * 9.0), at(sx * 10.0, sy * 5.0)], stroke);
                painter.line_segment([at(sx * 10.0, sy * 9.0), at(sx * 5.0, sy * 9.0)], stroke);
            }
            pair(1.5);
        }
        LayerIcon::Repeat => {
            // A box and its copies in a row: the first filled.
            for (i, x) in [-8.5_f32, -2.5, 3.5].into_iter().enumerate() {
                let r = Rect::from_min_max(at(x - 1.0, -4.5), at(x + 5.0, 4.5));
                if i == 0 {
                    painter.rect_filled(r, 1.0, color);
                } else {
                    painter.rect_stroke(r, 1.0, thin, StrokeKind::Middle);
                }
            }
            painter.line_segment([at(9.5, -4.5), at(9.5, 4.5)], thin);
        }
    }
}

impl ViewerUi {
    /// One icon button: named for screen readers and on hover (the name, then `note` under it), drawn greyed
    /// when `on` is false. Returns whether it was clicked (never when it is off).
    pub(crate) fn layer_icon_button(
        &self,
        ui: &mut egui::Ui,
        icon: LayerIcon,
        on: bool,
        note: Option<String>,
    ) -> bool {
        let name = self.t(icon.message_id());
        let (rect, response) =
            ui.allocate_exact_size(vec2(BUTTON_SIDE, BUTTON_SIDE), Sense::click());
        response.widget_info(|| WidgetInfo::labeled(Role::Button, on, &name));
        let v = ui.visuals();
        let (fill, color) = if !on {
            (
                Color32::TRANSPARENT,
                v.widgets
                    .noninteractive
                    .fg_stroke
                    .color
                    .gamma_multiply(0.45),
            )
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
        paint(painter, icon, rect.center(), color);
        let response: Response = response.on_hover_ui(|ui| {
            ui.label(&name);
            if let Some(n) = &note {
                ui.weak(n);
            }
        });
        on && response.clicked()
    }
}
