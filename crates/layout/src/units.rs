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

//! Lengths typed into a field (napkin §5.3, §6.3; rung 3 bbq Q10). Every length field reads
//! `mm`, `pt`, `px`, `in`, `%` and `u`, and shows values in the document's display unit.
//!
//! - `px` is a device pixel of the profile on screen.
//! - `%` is of the page along the field's axis (also for an object inside a group: groups are
//!   see-through to constraints, so a child is placed against the page; rung 7a look change).
//! - `u` is the rhythm unit along the field's axis, as the profile on screen resolves it: with a
//!   pixel-locked rhythm, `2u` is exactly two grid steps on that device.
//! - A number with no unit is in the display unit.

use crate::RhythmPx;
use quadrule_model::{Document, MM_PER_IN, PT_PER_IN, Unit};
use quadrule_profiles::Profile;

/// Which way a length runs, for `%` and `u`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Across: x, width.
    X,
    /// Down: y, height.
    Y,
}

impl Axis {
    /// The other axis.
    pub fn other(self) -> Axis {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}

/// What a typed length means in one field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitContext {
    /// Device pixels per mm on the profile on screen.
    pub px_per_mm: f64,
    /// The page's length along the field's axis, in mm (the primary page, where designs live).
    pub page_mm: f64,
    /// One rhythm unit along the field's axis, in mm, as the profile on screen resolves it.
    pub rhythm_mm: f64,
    /// The unit of a number typed with none, and of the values shown.
    pub display: Unit,
}

impl UnitContext {
    /// The context for a field on `axis`, with `shown` on screen.
    pub fn new(doc: &Document, primary: &Profile, shown: &Profile, axis: Axis) -> Self {
        let px_per_mm = shown.ppi / MM_PER_IN;
        let r = RhythmPx::new(doc, shown);
        let (page, unit_px) = match axis {
            Axis::X => (primary.page_mm().0, r.unit_x),
            Axis::Y => (primary.page_mm().1, r.unit_y),
        };
        UnitContext {
            px_per_mm,
            page_mm: page,
            rhythm_mm: unit_px / px_per_mm,
            display: doc.units,
        }
    }

    /// Millimetres in one of `unit`.
    pub fn mm_per(&self, unit: Unit) -> f64 {
        match unit {
            Unit::Mm => 1.0,
            Unit::Pt => MM_PER_IN / PT_PER_IN,
            Unit::In => MM_PER_IN,
            Unit::Px => 1.0 / self.px_per_mm,
        }
    }
}

/// Why a typed length could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitError {
    /// Nothing was typed.
    Empty,
    /// The number part is not a number.
    BadNumber(String),
    /// The unit is not one of mm, pt, px, in, %, u.
    UnknownUnit(String),
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitError::Empty => write!(f, "nothing typed"),
            UnitError::BadNumber(s) => write!(f, "not a number: {s}"),
            UnitError::UnknownUnit(s) => write!(f, "unknown unit: {s}"),
        }
    }
}

impl std::error::Error for UnitError {}

/// Reads a typed length (`5mm`, `12 pt`, `40px`, `0.25in`, `50%`, `2u`, `-3`) as millimetres.
/// Units are not case-sensitive; a comma works as the decimal point when there is no full stop.
pub fn parse_length(input: &str, ctx: &UnitContext) -> Result<f64, UnitError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(UnitError::Empty);
    }
    let split = s
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | ',' | '-' | '+')))
        .unwrap_or(s.len());
    let (num, unit) = (s[..split].trim(), s[split..].trim().to_ascii_lowercase());
    let num_text = if num.contains('.') {
        num.to_string()
    } else {
        num.replace(',', ".")
    };
    let value: f64 = num_text
        .parse()
        .ok()
        .filter(|v: &f64| v.is_finite())
        .ok_or_else(|| UnitError::BadNumber(num.to_string()))?;
    let mm_per = match unit.as_str() {
        "" => ctx.mm_per(ctx.display),
        "mm" => 1.0,
        "pt" => ctx.mm_per(Unit::Pt),
        "px" => ctx.mm_per(Unit::Px),
        "in" | "\"" => ctx.mm_per(Unit::In),
        "%" => ctx.page_mm / 100.0,
        "u" => ctx.rhythm_mm,
        other => return Err(UnitError::UnknownUnit(other.to_string())),
    };
    Ok(value * mm_per)
}

/// A length in mm as shown in a field: in the display unit, with its symbol, to a sensible
/// number of places (`12.5 mm`, `148 px`, `0.492 in`), trailing zeros dropped.
pub fn format_length(mm: f64, ctx: &UnitContext) -> String {
    let (places, symbol) = match ctx.display {
        Unit::Mm => (2, "mm"),
        Unit::Pt => (1, "pt"),
        Unit::Px => (1, "px"),
        Unit::In => (3, "in"),
    };
    format!(
        "{} {symbol}",
        trim_number(mm / ctx.mm_per(ctx.display), places)
    )
}

/// `v` to at most `places` decimals, without trailing zeros or a trailing point; never `-0`.
pub fn trim_number(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    if s == "-0" { "0".into() } else { s }
}
