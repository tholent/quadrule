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

//! The repeat tool's record (rung 10, docs/bbq/2026-11-13-rung-10-repeat-tool.md; napkin §6.4): a
//! live object that holds one source (an object, a group, or another repeat) and draws it again in
//! a row, a column or a grid. Copies are never stored, only drawn: editing the source changes every
//! copy, and Expand turns the repeat into plain objects.
//!
//! This file holds the settings and the maths that needs no pixels (millimetres on the primary
//! page: the count, the pitch, the alternation rules). `quadrule_layout::repeat` resolves the same
//! settings to whole device pixels on each device.

use crate::{Extra, FillRole, Frame, LineRole, Object, ObjectKind};
use serde::{Deserialize, Serialize};

/// The most copies a repeat draws, nested repeats multiplied (rung 10 bbq, Defaulted "Groups and
/// nesting"); past this the first copies are drawn and the status bar says so.
pub const REPEAT_COPIES_MAX: u64 = 5000;
/// The most copies a count field may hold along one axis.
pub const REPEAT_COUNT_MAX: u32 = 999;
/// The count a new repeat starts with.
pub const REPEAT_COUNT_DEFAULT: u32 = 4;
/// The smallest pitch a repeat steps by, in mm (so a typed zero cannot pile every copy up).
pub const REPEAT_STRIDE_MIN_MM: f64 = 0.1;

/// How the copies are laid out (rung 10 bbq, Defaulted "Modes").
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    /// A row: the copies go to the right.
    #[default]
    Across,
    /// A column: the copies go down.
    Down,
    /// Columns by rows.
    Grid,
}

/// The step between copies along one axis (rung 10 bbq, Q4 and Defaulted "Spacing storage and the
/// rhythm").
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum Spacing {
    /// The source's size on that axis rounded up to whole rhythm steps; follows the source and the
    /// rhythm. Copies never overlap and land on the rhythm.
    #[default]
    Auto,
    /// This many rhythm steps (`u`): bound to the rhythm, so it follows a change of rhythm.
    Units(f64),
    /// This many mm: not bound to the rhythm.
    Mm(f64),
}

impl Spacing {
    /// Whether the spacing follows the rhythm (Auto and rhythm units).
    pub fn is_bound(&self) -> bool {
        !matches!(self, Spacing::Mm(_))
    }
}

/// What a typed spacing means (rung 10 bbq, Q4): centre to centre, or edge to edge.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpacingBasis {
    /// From one copy's centre to the next one's (the default).
    #[default]
    Pitch,
    /// The space between one copy's edge and the next one's.
    Gap,
}

/// Where the copies sit in a box with room to spare (rung 10 bbq, Q6).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Leftover {
    /// From the box's top-left corner (the default).
    #[default]
    Start,
    /// The spare space split to both sides.
    Centre,
}

/// What an alternation rule counts (rung 10 bbq, Q5). A linear repeat has one row or one column,
/// so for it Cells is the plain copy count.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuleCounts {
    /// Columns, from 1 at the left.
    Columns,
    /// Rows, from 1 at the top.
    Rows,
    /// Cells in reading order: left to right, then down (the default).
    #[default]
    Cells,
}

/// Which numbers a rule hits (rung 10 bbq, Q5 and Defaulted "Alternation details").
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RuleWhen {
    /// Every Nth, from `start` (which reads as N when it is left out): S, S+N, S+2N and so on.
    Every {
        /// The step, at least 1.
        n: u32,
        /// The first number hit; `None` reads as `n`.
        #[serde(default)]
        start: Option<u32>,
    },
    /// Exactly these numbers.
    List(Vec<u32>),
}

/// What a rule does to the copies it hits (rung 10 bbq, Q5).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    /// The copy is not drawn.
    Skip,
    /// Every stroke in the copy takes this line role (text keeps its own).
    LineRole(LineRole),
    /// The fill of every shape in the copy that has one takes this fill role.
    FillRole(FillRole),
}

/// One alternation rule: it counts something, hits some numbers and does one thing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AlternationRule {
    /// What it counts.
    #[serde(default)]
    pub counts: RuleCounts,
    /// Which numbers it hits.
    pub when: RuleWhen,
    /// What it does there.
    pub action: RuleAction,
}

impl AlternationRule {
    /// A new rule: every 2nd cell, skipped.
    pub fn new() -> Self {
        AlternationRule {
            counts: RuleCounts::Cells,
            when: RuleWhen::Every { n: 2, start: None },
            action: RuleAction::Skip,
        }
    }

    /// Whether the rule hits number `k` (counted from 1).
    pub fn hits(&self, k: u32) -> bool {
        match &self.when {
            RuleWhen::Every { n, start } => {
                let n = (*n).max(1);
                let s = start.unwrap_or(n).max(1);
                k >= s && (k - s).is_multiple_of(n)
            }
            RuleWhen::List(l) => l.contains(&k),
        }
    }
}

impl Default for AlternationRule {
    fn default() -> Self {
        Self::new()
    }
}

/// What the rules together do to one copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CellEffect {
    /// Not drawn.
    pub skip: bool,
    /// Every stroke in it takes this role.
    pub line: Option<LineRole>,
    /// Its shapes' fills take this role.
    pub fill: Option<FillRole>,
}

impl CellEffect {
    /// Whether the copy is drawn exactly as the source is.
    pub fn is_plain(&self) -> bool {
        !self.skip && self.line.is_none() && self.fill.is_none()
    }
}

/// A repeat (format 0.10, rung 10 bbq). It uses the object's `frame` (its box: the copies' outer
/// box in Count mode, the room they fill in Fill mode), `constraints`, `overrides`, `visible`,
/// `locked`, `opacity_pct` and `name`; its style, fill, dash, cap and rotation are ignored. The
/// source is laid out as designed on the primary device, its top-left corner at the box's, and
/// moved rigidly into each cell on every device.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RepeatShape {
    /// What is repeated: the only thing inside. It may be a group or another repeat.
    pub source: Box<Object>,
    /// A row, a column or a grid.
    #[serde(default)]
    pub mode: RepeatMode,
    /// Fill the box with as many copies as fit, instead of a fixed count (rung 10 bbq, Q3).
    #[serde(default)]
    pub fill: bool,
    /// The number of copies in a row or a column (Count mode).
    #[serde(default = "count_default")]
    pub count: u32,
    /// A grid's columns (Count mode).
    #[serde(default = "grid_default")]
    pub columns: u32,
    /// A grid's rows (Count mode).
    #[serde(default = "grid_default")]
    pub rows: u32,
    /// The step across.
    #[serde(default)]
    pub h_spacing: Spacing,
    /// The step down.
    #[serde(default)]
    pub v_spacing: Spacing,
    /// Whether a typed spacing is centre to centre or edge to edge.
    #[serde(default)]
    pub basis: SpacingBasis,
    /// Where the copies sit when the box has room to spare.
    #[serde(default)]
    pub leftover: Leftover,
    /// Alternation rules, applied in order, a later one winning.
    #[serde(default)]
    pub rules: Vec<AlternationRule>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

fn count_default() -> u32 {
    REPEAT_COUNT_DEFAULT
}

fn grid_default() -> u32 {
    3
}

impl RepeatShape {
    /// A new repeat of `source`: across, a count of 4, Auto pitch, from the start.
    pub fn new(source: Object) -> Self {
        RepeatShape {
            source: Box::new(source),
            mode: RepeatMode::Across,
            fill: false,
            count: REPEAT_COUNT_DEFAULT,
            columns: 3,
            rows: 3,
            h_spacing: Spacing::Auto,
            v_spacing: Spacing::Auto,
            basis: SpacingBasis::Pitch,
            leftover: Leftover::Start,
            rules: Vec::new(),
            extra: Extra::new(),
        }
    }

    /// Whether the repeat is a grid.
    pub fn is_grid(&self) -> bool {
        self.mode == RepeatMode::Grid
    }

    /// The counts (columns, rows) of Count mode, each 1 up to [`REPEAT_COUNT_MAX`].
    pub fn fixed_counts(&self) -> (u32, u32) {
        let c = |n: u32| n.clamp(1, REPEAT_COUNT_MAX);
        match self.mode {
            RepeatMode::Across => (c(self.count), 1),
            RepeatMode::Down => (1, c(self.count)),
            RepeatMode::Grid => (c(self.columns), c(self.rows)),
        }
    }

    /// Which axes the copies run along: (across, down).
    pub fn axes(&self) -> (bool, bool) {
        match self.mode {
            RepeatMode::Across => (true, false),
            RepeatMode::Down => (false, true),
            RepeatMode::Grid => (true, true),
        }
    }

    /// The step between copies in mm, (across, down), for a source of outer size `size` under a
    /// rhythm of `steps`. Never less than [`REPEAT_STRIDE_MIN_MM`].
    pub fn stride_mm(&self, size: (f64, f64), steps: (f64, f64)) -> (f64, f64) {
        let one = |sp: Spacing, size: f64, step: f64| {
            let step = if step.is_finite() && step > 0.0 {
                step
            } else {
                1.0
            };
            let v = match sp {
                Spacing::Auto => auto_steps(size, step) as f64 * step,
                Spacing::Units(u) => match self.basis {
                    SpacingBasis::Pitch => u * step,
                    SpacingBasis::Gap => size + u * step,
                },
                Spacing::Mm(m) => match self.basis {
                    SpacingBasis::Pitch => m,
                    SpacingBasis::Gap => size + m,
                },
            };
            if v.is_finite() {
                v.max(REPEAT_STRIDE_MIN_MM)
            } else {
                step
            }
        };
        (
            one(self.h_spacing, size.0, steps.0),
            one(self.v_spacing, size.1, steps.1),
        )
    }

    /// The counts (columns, rows) when the box has room `room` (w, h) in mm and the copies are
    /// `size` big and `stride` apart: those of Count mode, or, in Fill mode, as many as fit along
    /// each axis the copies run along (at least one).
    pub fn counts_mm(&self, room: (f64, f64), size: (f64, f64), stride: (f64, f64)) -> (u32, u32) {
        let (cols, rows) = self.fixed_counts();
        if !self.fill {
            return (cols, rows);
        }
        let (ax, ay) = self.axes();
        let fit = |room: f64, size: f64, stride: f64| {
            if room < size {
                1
            } else {
                (((room - size) / stride + 1e-9).floor() as u32 + 1).min(REPEAT_COUNT_MAX)
            }
        };
        (
            if ax { fit(room.0, size.0, stride.0) } else { 1 },
            if ay { fit(room.1, size.1, stride.1) } else { 1 },
        )
    }

    /// What the rules do to the copy at column `col`, row `row` (both from 1) of a block
    /// `cols` wide.
    pub fn effect(&self, col: u32, row: u32, cols: u32) -> CellEffect {
        let cell = (row - 1) * cols + col;
        let mut e = CellEffect::default();
        for r in &self.rules {
            let k = match r.counts {
                RuleCounts::Columns => col,
                RuleCounts::Rows => row,
                RuleCounts::Cells => cell,
            };
            if !r.hits(k) {
                continue;
            }
            match r.action {
                RuleAction::Skip => e.skip = true,
                RuleAction::LineRole(l) => e.line = Some(l),
                RuleAction::FillRole(f) => e.fill = Some(f),
            }
        }
        e
    }

    /// The box round a block of `counts` copies of `size` that are `stride` apart, in mm.
    pub fn block_mm(counts: (u32, u32), size: (f64, f64), stride: (f64, f64)) -> (f64, f64) {
        (
            (counts.0.saturating_sub(1)) as f64 * stride.0 + size.0,
            (counts.1.saturating_sub(1)) as f64 * stride.1 + size.1,
        )
    }

    /// Whether the repeat's step is bound to the rhythm on both axes (Auto and rhythm units).
    pub fn is_bound(&self) -> bool {
        self.h_spacing.is_bound() && self.v_spacing.is_bound()
    }
}

/// How many whole rhythm steps hold `size` (at least one): the source's size rounded up.
pub fn auto_steps(size: f64, step: f64) -> u32 {
    if !(size.is_finite() && step > 0.0) || size <= 0.0 {
        return 1;
    }
    ((size / step - 1e-6).ceil().max(1.0)) as u32
}

impl Object {
    /// Whether this is a repeat (format 0.10).
    pub fn is_repeat(&self) -> bool {
        matches!(self.kind, ObjectKind::Repeat(_))
    }

    /// Whether the object holds other objects: a group or a repeat.
    pub fn is_container(&self) -> bool {
        matches!(self.kind, ObjectKind::Group(_) | ObjectKind::Repeat(_))
    }

    /// The repeat's settings, if this is one.
    pub fn repeat(&self) -> Option<&RepeatShape> {
        match &self.kind {
            ObjectKind::Repeat(r) => Some(r),
            _ => None,
        }
    }

    /// The repeat's settings to change, if this is one.
    pub fn repeat_mut(&mut self) -> Option<&mut RepeatShape> {
        match &mut self.kind {
            ObjectKind::Repeat(r) => Some(r),
            _ => None,
        }
    }

    /// How many times this object is drawn over, as the cap on a repeat's copies counts it (rung
    /// 10 bbq, Defaulted "Groups and nesting"): 1 for a plain object, the sum for a group, and for a
    /// repeat its copies (as it would draw them on the primary) times what its source weighs.
    pub fn copy_weight(&self, steps: (f64, f64)) -> u64 {
        match &self.kind {
            ObjectKind::Group(g) => g
                .children
                .iter()
                .map(|c| c.copy_weight(steps))
                .sum::<u64>()
                .max(1),
            ObjectKind::Repeat(r) => {
                let size = r.source.outer_box();
                let size = (size.w_mm, size.h_mm);
                let stride = r.stride_mm(size, steps);
                let (c, rw) = r.counts_mm((self.frame.w_mm, self.frame.h_mm), size, stride);
                (u64::from(c) * u64::from(rw)).saturating_mul(r.source.copy_weight(steps))
            }
            _ => 1,
        }
    }

    /// The box the repeat's frame must be, from its source as it stands: the source's top-left
    /// corner is the box's; in Count mode the size is the copies' outer box, in Fill mode it is
    /// kept. `None` for an object that is not a repeat.
    pub fn repeat_fit(&self, steps: (f64, f64)) -> Option<Frame> {
        let r = self.repeat()?;
        let sb = r.source.outer_box();
        if r.fill {
            return Some(Frame::new(
                sb.x_mm,
                sb.y_mm,
                self.frame.w_mm,
                self.frame.h_mm,
            ));
        }
        let size = (sb.w_mm, sb.h_mm);
        let stride = r.stride_mm(size, steps);
        let (w, h) = RepeatShape::block_mm(r.fixed_counts(), size, stride);
        Some(Frame::new(sb.x_mm, sb.y_mm, w, h))
    }
}
