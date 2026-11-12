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

//! Align and distribute, and "mixed" (rung 7b, docs/bbq/2026-11-11-rung-7-groups-and-align.md,
//! Q5, Q6 and Defaulted "7b pays the rung 3 debt"). The maths is in `quadrule_layout::align`; this
//! turns it into one undo step on the open document. Like the Arrange fields, it works on the
//! stored design, on the primary profile's page, whichever device is on screen.

use crate::viewer::Viewer;
use egui::{Color32, Pos2, Stroke, pos2};
use quadrule_layout::align::{
    AlignTo, Boxed, Edge, Grid, Spread, align_deltas, boxes_of, distribute_deltas, target_box,
    union_of,
};
use quadrule_layout::units::Axis;
use quadrule_model::Frame;
use quadrule_model::edit::{Command, EditError};

/// One of the ten buttons of the Arrange tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrangeOp {
    /// Line up the left edges, the middles or the right edges (X), or the tops, the middles or the
    /// bottoms (Y).
    Align(Axis, Edge),
    /// Spread three or more objects evenly along an axis.
    Distribute(Axis, Spread),
}

impl ArrangeOp {
    /// The six align buttons, then the four distribute buttons, in the order they show.
    pub const ALL: [ArrangeOp; 10] = [
        ArrangeOp::Align(Axis::X, Edge::Start),
        ArrangeOp::Align(Axis::X, Edge::Centre),
        ArrangeOp::Align(Axis::X, Edge::End),
        ArrangeOp::Align(Axis::Y, Edge::Start),
        ArrangeOp::Align(Axis::Y, Edge::Centre),
        ArrangeOp::Align(Axis::Y, Edge::End),
        ArrangeOp::Distribute(Axis::X, Spread::Spacing),
        ArrangeOp::Distribute(Axis::X, Spread::Centres),
        ArrangeOp::Distribute(Axis::Y, Spread::Spacing),
        ArrangeOp::Distribute(Axis::Y, Spread::Centres),
    ];

    /// The message that names the button (for screen readers and its tooltip).
    pub fn message_id(self) -> &'static str {
        match self {
            ArrangeOp::Align(Axis::X, Edge::Start) => "app-arrange-align-left",
            ArrangeOp::Align(Axis::X, Edge::Centre) => "app-arrange-align-centre",
            ArrangeOp::Align(Axis::X, Edge::End) => "app-arrange-align-right",
            ArrangeOp::Align(Axis::Y, Edge::Start) => "app-arrange-align-top",
            ArrangeOp::Align(Axis::Y, Edge::Centre) => "app-arrange-align-middle",
            ArrangeOp::Align(Axis::Y, Edge::End) => "app-arrange-align-bottom",
            ArrangeOp::Distribute(Axis::X, Spread::Spacing) => "app-distribute-h-spacing",
            ArrangeOp::Distribute(Axis::X, Spread::Centres) => "app-distribute-h-centres",
            ArrangeOp::Distribute(Axis::Y, Spread::Spacing) => "app-distribute-v-spacing",
            ArrangeOp::Distribute(Axis::Y, Spread::Centres) => "app-distribute-v-centres",
        }
    }

    /// Whether this is a distribute button.
    pub fn distributes(self) -> bool {
        matches!(self, ArrangeOp::Distribute(..))
    }
}

/// Which of the Arrange fields hold different values across the selected objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mixed {
    /// X differs.
    pub x: bool,
    /// Y differs.
    pub y: bool,
    /// Width differs.
    pub w: bool,
    /// Height differs.
    pub h: bool,
    /// Rotation differs (among the objects that can turn).
    pub rotation: bool,
}

/// Two lengths in mm that are the same to a thousandth of a millimetre.
fn same(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}

/// Two angles in degrees that are the same to a thousandth of a degree.
fn same_deg(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}

impl Viewer {
    /// What the align buttons align to now: the choice made for this many objects picked (the
    /// selection for two or more, the page for one, until the user picks otherwise).
    pub fn align_to(&self) -> AlignTo {
        if self.selection.len() >= 2 {
            self.align_multi
        } else {
            self.align_single
        }
    }

    /// The user's pick of what to align to, kept for as many objects as are picked now (one, or
    /// two or more), so each case remembers its own.
    pub fn set_align_to(&mut self, to: AlignTo) {
        if self.selection.len() >= 2 {
            self.align_multi = to;
        } else {
            self.align_single = to;
        }
    }

    /// Whether a button can do something now: align needs two or more objects that can be edited
    /// when it aligns to the selection and one when it aligns to the page or the margins;
    /// distribute needs three or more.
    pub fn arrange_enabled(&self, op: ArrangeOp) -> bool {
        let n = self.editable_selection().len();
        match op {
            ArrangeOp::Align(..) => {
                n >= if self.align_to() == AlignTo::Selection {
                    2
                } else {
                    1
                }
            }
            ArrangeOp::Distribute(..) => n >= 3,
        }
    }

    /// Does what the button `op` says to the selected objects that can be edited, as one undo
    /// step. A locked object, or one on a locked layer, stays put and is left out of the
    /// selection's box. With the pixel-grid snap on the result lands on whole pixels of the
    /// primary profile (Q6). Nothing to move is not a step.
    pub fn arrange(&mut self, op: ArrangeOp) -> Result<(), EditError> {
        let ids = self.editable_selection();
        if ids.is_empty() {
            self.locked_notice_if_none_editable();
            return Ok(());
        }
        let Some((primary, _)) = self.profiles() else {
            return Ok(());
        };
        let to = self.align_to();
        let grid = Grid::for_profile(&primary, self.snap.pixel_grid);
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let boxed = boxes_of(doc, &primary, &ids);
        let frames: Vec<Frame> = boxed.iter().map(|b| b.outer).collect();
        let steps: Vec<f64> = match op {
            ArrangeOp::Align(axis, edge) => {
                let Some(target) = target_box(doc, &primary, to).or_else(|| union_of(&frames))
                else {
                    return Ok(());
                };
                align_deltas(&frames, axis, edge, target, grid)
            }
            ArrangeOp::Distribute(axis, by) => distribute_deltas(&frames, axis, by, grid),
        };
        let axis = match op {
            ArrangeOp::Align(a, _) | ArrangeOp::Distribute(a, _) => a,
        };
        let cmd = move_command(doc, &boxed, axis, &steps);
        match cmd {
            Some(cmd) => self.edit(cmd, None),
            None => Ok(()),
        }
    }

    /// Which Arrange fields differ across the selected objects (Defaulted "7b pays the rung 3
    /// debt"). One object is never mixed.
    pub fn arrange_mixed(&self) -> Mixed {
        let Some(doc) = self.doc() else {
            return Mixed::default();
        };
        let objects: Vec<_> = self
            .selection
            .iter()
            .filter_map(|id| doc.object(*id))
            .collect();
        let mut m = Mixed::default();
        let Some(first) = objects.first() else {
            return m;
        };
        for o in &objects[1..] {
            m.x |= !same(o.frame.x_mm, first.frame.x_mm);
            m.y |= !same(o.frame.y_mm, first.frame.y_mm);
            m.w |= !same(o.frame.w_mm, first.frame.w_mm);
            m.h |= !same(o.frame.h_mm, first.frame.h_mm);
        }
        let mut turning = objects.iter().filter(|o| o.kind.can_rotate());
        if let Some(a) = turning.next() {
            m.rotation = turning.any(|b| !same_deg(a.rotation(), b.rotation()));
        }
        m
    }
}

/// The command that moves each box by its step along `axis` (page mm), as the objects are stored
/// (undone as the turn of the groups they are in); `None` when no one moves.
fn move_command(
    doc: &quadrule_model::Document,
    boxed: &[Boxed],
    axis: Axis,
    steps: &[f64],
) -> Option<Command> {
    let cmds: Vec<Command> = boxed
        .iter()
        .zip(steps)
        .filter(|(_, d)| **d != 0.0)
        .filter_map(|(b, d)| {
            let (dx, dy) = match axis {
                Axis::X => (*d, 0.0),
                Axis::Y => (0.0, *d),
            };
            let (sx, sy) = b.stored_step(dx, dy);
            let mut o = doc.object(b.id)?.clone();
            o.translate(sx, sy);
            Some(Command::SetObject(o))
        })
        .collect();
    (!cmds.is_empty()).then_some(Command::Batch(cmds))
}

/// Paints the icon of a button centred at `c` (a 26-point square holds it): a bar or two
/// against a line for align, a row of boxes between two lines for distribute.
pub(crate) fn paint_icon(painter: &egui::Painter, op: ArrangeOp, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let thin = Stroke::new(1.2, color);
    // Draw as if horizontal (X) and turn the points for Y.
    let at = |axis: Axis, a: f32, b: f32| match axis {
        Axis::X => pos2(c.x + a, c.y + b),
        Axis::Y => pos2(c.x + b, c.y + a),
    };
    let bar = |axis: Axis, a0: f32, a1: f32, b0: f32, b1: f32| {
        egui::Rect::from_two_pos(at(axis, a0, b0), at(axis, a1, b1))
    };
    match op {
        ArrangeOp::Align(axis, edge) => {
            // A line across (at a), and two bars of different lengths hung on it.
            let (line, b1, b2) = match edge {
                Edge::Start => (-9.0, (-9.0, 3.0), (-9.0, 8.0)),
                Edge::Centre => (0.0, (-6.0, 6.0), (-3.0, 3.0)),
                Edge::End => (9.0, (-3.0, 9.0), (-8.0, 9.0)),
            };
            painter.line_segment([at(axis, line, -10.0), at(axis, line, 10.0)], thin);
            painter.rect_filled(bar(axis, b1.0, b1.1, -6.0, -1.0), 1.0, color);
            painter.rect_filled(bar(axis, b2.0, b2.1, 2.0, 7.0), 1.0, color);
        }
        ArrangeOp::Distribute(axis, by) => {
            painter.line_segment([at(axis, -10.0, -9.0), at(axis, -10.0, 9.0)], thin);
            painter.line_segment([at(axis, 10.0, -9.0), at(axis, 10.0, 9.0)], thin);
            match by {
                Spread::Spacing => {
                    painter.rect_filled(bar(axis, -3.5, 3.5, -5.0, 5.0), 1.0, color);
                }
                Spread::Centres => {
                    painter.circle_filled(at(axis, 0.0, 0.0), 2.4, color);
                    painter.line_segment([at(axis, 0.0, -7.0), at(axis, 0.0, 7.0)], stroke);
                }
            }
        }
    }
}
