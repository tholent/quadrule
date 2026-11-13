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

//! Where the copies of a repeat fall on one device (rung 10, docs/bbq/2026-11-13-rung-10-repeat-tool.md).
//!
//! The source is laid out as designed on the primary device and then moved rigidly into each cell,
//! on every device, so every copy is the same shape everywhere; only the box and the count change
//! from device to device (Defaulted "The source's place"). The step between copies is worked out in
//! whole device pixels on each device (Defaulted "Whole pixels"), so every copy is the source's
//! pixels moved by a whole number of pixels: they are pixel-identical on e-ink.
//!
//! A [`Plan`] is small and `Copy`: it holds the counts, the step and where the first cell is.
//! [`Plan::copies`] walks the cells in reading order.

use quadrule_model::repeat::{REPEAT_STRIDE_MIN_MM, auto_steps};
use quadrule_model::{
    CellEffect, Frame, REPEAT_COPIES_MAX, REPEAT_COUNT_MAX, RepeatShape, Spacing, SpacingBasis,
};

/// What a device's pixels are: pixels per mm and the rhythm's steps in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PxScale {
    /// Device pixels per millimetre.
    pub ppmm: f64,
    /// The rhythm's step across and down in pixels (whole when pixel-locked).
    pub step: (f64, f64),
}

/// What a repeat needs to know about the rhythm and the device: the rhythm's steps in mm, and, when
/// the layout is for a device's pixels, those. Without pixels everything is worked out in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RepeatEnv {
    /// The rhythm's steps (across, down) in mm.
    pub steps_mm: (f64, f64),
    /// The device's pixels, when the layout is for drawing or picking on one.
    pub px: Option<PxScale>,
}

impl RepeatEnv {
    /// A layout in millimetres only.
    pub fn mm(steps_mm: (f64, f64)) -> Self {
        RepeatEnv { steps_mm, px: None }
    }
}

/// The cells of one repeat on one device.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    /// Columns.
    pub cols: u32,
    /// Rows.
    pub rows: u32,
    /// The most copies drawn (nested repeats share [`REPEAT_COPIES_MAX`]).
    pub budget: u64,
    /// The step between cells in device units: whole pixels, or mm when there are no pixels.
    pub stride: (f64, f64),
    /// Device units per mm (1 when there are no pixels).
    pub k: f64,
    /// Whether the units are pixels.
    pub px: bool,
    /// How far the source moves from where it is stored to the first cell, in mm.
    pub shift0: (f64, f64),
    /// The source's outer size in mm.
    pub size_mm: (f64, f64),
}

/// One cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CopyAt {
    /// Its place in reading order, from 0.
    pub index: u32,
    /// Its column, from 0.
    pub col: u32,
    /// Its row, from 0.
    pub row: u32,
    /// What the alternation rules do to it.
    pub effect: CellEffect,
    /// How far it is from the first cell, in device units (whole pixels when `px`).
    pub d: (f64, f64),
    /// The same in mm.
    pub mm: (f64, f64),
}

impl Plan {
    /// Every cell the counts give, skipped ones included.
    pub fn total(&self) -> u64 {
        u64::from(self.cols) * u64::from(self.rows)
    }

    /// The cells walked (the total, up to the budget).
    pub fn walked(&self) -> u64 {
        self.total().min(self.budget)
    }

    /// Whether the budget cut the copies short.
    pub fn capped(&self) -> bool {
        self.total() > self.budget
    }

    /// The cells in reading order (left to right, then down), up to the budget, each with what the
    /// rules do to it.
    pub fn copies<'a>(&'a self, r: &'a RepeatShape) -> impl Iterator<Item = CopyAt> + 'a {
        let cols = self.cols.max(1);
        (0..self.walked() as u32).map(move |i| {
            let (col, row) = (i % cols, i / cols);
            let d = (
                f64::from(col) * self.stride.0,
                f64::from(row) * self.stride.1,
            );
            CopyAt {
                index: i,
                col,
                row,
                effect: r.effect(col + 1, row + 1, cols),
                d,
                mm: (d.0 / self.k, d.1 / self.k),
            }
        })
    }

    /// How many copies are actually drawn: the walked cells the rules do not skip.
    pub fn drawn(&self, r: &RepeatShape) -> u64 {
        self.copies(r).filter(|c| !c.effect.skip).count() as u64
    }
}

/// The plan for repeat `r` whose box is `local` on this device (in the space it sits in), for a
/// design made on the page `from`.
pub fn plan(r: &RepeatShape, local: Frame, from: Frame, env: &RepeatEnv) -> Plan {
    let sb = crate::tree::source_box(&r.source, from);
    let size_mm = (sb.w_mm, sb.h_mm);
    let (k, px) = match env.px {
        Some(p) if p.ppmm.is_finite() && p.ppmm > 0.0 => (p.ppmm, true),
        _ => (1.0, false),
    };
    let q = |v: f64| if px { v.round() } else { v };
    // The source's size and the room, in device units (whole pixels on a device).
    let size = (q(size_mm.0 * k), q(size_mm.1 * k));
    let room = (q(local.w_mm * k), q(local.h_mm * k));
    let step_d = match env.px {
        Some(p) if px => p.step,
        _ => env.steps_mm,
    };
    let min = if px { 1.0 } else { REPEAT_STRIDE_MIN_MM };
    let one = |sp: Spacing, size_mm: f64, size_d: f64, step_mm: f64, step_d: f64| {
        let step_mm = if step_mm.is_finite() && step_mm > 0.0 {
            step_mm
        } else {
            1.0
        };
        let v = match sp {
            Spacing::Auto => f64::from(auto_steps(size_mm, step_mm)) * step_d,
            Spacing::Units(u) => match r.basis {
                SpacingBasis::Pitch => u * step_d,
                SpacingBasis::Gap => size_d + u * step_d,
            },
            Spacing::Mm(m) => match r.basis {
                SpacingBasis::Pitch => m * k,
                SpacingBasis::Gap => size_d + m * k,
            },
        };
        if v.is_finite() {
            q(v).max(min)
        } else {
            q(step_d).max(min)
        }
    };
    let stride = (
        one(r.h_spacing, size_mm.0, size.0, env.steps_mm.0, step_d.0),
        one(r.v_spacing, size_mm.1, size.1, env.steps_mm.1, step_d.1),
    );
    // How many.
    let (fc, fr) = r.fixed_counts();
    let (cols, rows) = if r.fill {
        let (ax, ay) = r.axes();
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
    } else {
        (fc, fr)
    };
    // Where the first cell is.
    let block = (
        f64::from(cols - 1) * stride.0 + size.0,
        f64::from(rows - 1) * stride.1 + size.1,
    );
    let off = match r.leftover {
        quadrule_model::Leftover::Start => (0.0, 0.0),
        quadrule_model::Leftover::Centre => {
            let half = |spare: f64| {
                if px {
                    (spare / 2.0).floor()
                } else {
                    spare / 2.0
                }
            };
            (half(room.0 - block.0), half(room.1 - block.1))
        }
    };
    let mut shift0 = (
        local.x_mm + off.0 / k - sb.x_mm,
        local.y_mm + off.1 / k - sb.y_mm,
    );
    // Float noise is not a move: the source stays exactly where it is stored.
    if shift0.0.abs() < 1e-9 {
        shift0.0 = 0.0;
    }
    if shift0.1.abs() < 1e-9 {
        shift0.1 = 0.0;
    }
    let weight = r.source.copy_weight(env.steps_mm).max(1);
    Plan {
        cols,
        rows,
        budget: (REPEAT_COPIES_MAX / weight).max(1),
        stride,
        k,
        px,
        shift0,
        size_mm,
    }
}

/// What Expand did, for the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expanded {
    /// The new group's id.
    pub group: quadrule_model::Uuid,
    /// How many copies it holds.
    pub copies: usize,
    /// The other devices that drew a different number of copies before (a Fill repeat's box held
    /// more or fewer there), with that number: the expanded copies keep the primary's count.
    pub differing: Vec<(String, u64)>,
}

/// The command that expands repeat `id` into plain objects (rung 10 bbq, Q7 and Defaulted "Expand on
/// other devices"): one group `group_id` named after the repeat, holding a real copy of the source
/// for each copy the primary device draws (new ids from `new_id`, alternation baked in, skipped
/// copies left out; with a group as the source, each copy is a group inside it). Changes no pixel on
/// the primary device. One undo step.
pub fn expand(
    doc: &quadrule_model::Document,
    primary: &quadrule_profiles::Profile,
    id: quadrule_model::Uuid,
    group_id: quadrule_model::Uuid,
    mut new_id: impl FnMut() -> quadrule_model::Uuid,
) -> Result<(quadrule_model::edit::Command, Expanded), quadrule_model::edit::EditError> {
    use quadrule_model::edit::{EditError, children_box, renew_ids, replace_object};
    use quadrule_model::{GroupShape, Object, ObjectKind};
    let o = doc.object(id).ok_or(EditError::NoSuchObject(id))?;
    let r = o.repeat().ok_or(EditError::NotARepeat(id))?;
    if !doc.editable(id) {
        return Err(EditError::Locked(id));
    }
    let plans_on = |target: &quadrule_profiles::Profile| {
        crate::tree::resolve_nodes_env(
            doc,
            primary.page_mm(),
            target.page_mm(),
            crate::override_key(primary, target),
            Some(crate::device_scale(doc, target)),
        )
        .into_iter()
        .find(|n| n.id == id)
        .and_then(|n| n.plan)
    };
    let plan = plans_on(primary).ok_or(EditError::NotARepeat(id))?;
    let mut copies: Vec<Object> = Vec::new();
    for c in plan.copies(r).filter(|c| !c.effect.skip) {
        let mut copy = (*r.source).clone();
        crate::restyle(&mut copy, c.effect.line, c.effect.fill);
        renew_ids(&mut copy, &mut new_id);
        copy.translate(plan.shift0.0 + c.mm.0, plan.shift0.1 + c.mm.1);
        copies.push(copy);
    }
    let frame = children_box(&copies).ok_or(EditError::NothingToRepeat)?;
    let n = copies.len();
    let mut group = o.clone();
    group.id = group_id;
    group.frame = frame;
    group.constraints = quadrule_model::Constraints::default();
    group.overrides.clear();
    group.style = quadrule_model::Style::None;
    group.rotation_deg = 0.0;
    group.kind = ObjectKind::Group(GroupShape {
        children: copies,
        extra: Default::default(),
    });
    let cmd = replace_object(doc, id, group)?;
    // Devices that drew another number of copies.
    let mut differing = Vec::new();
    for dev in doc.profile_ids() {
        if dev == primary.id {
            continue;
        }
        let Ok(target) = quadrule_profiles::builtin_for(dev, doc.page.orientation) else {
            continue;
        };
        if let Some(p) = plans_on(&target) {
            let drawn = p.drawn(r);
            if drawn != n as u64 {
                differing.push((dev.to_string(), drawn));
            }
        }
    }
    Ok((
        cmd,
        Expanded {
            group: group_id,
            copies: n,
            differing,
        },
    ))
}
