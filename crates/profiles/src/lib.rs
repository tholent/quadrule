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

//! Device profiles (napkin §3): one target screen each, stored as data. Built-in profiles are
//! RON files embedded in the binary (§3.2). Rung 1 ships the Manta and the Nomad (§3.3).

#![forbid(unsafe_code)]

use quadrule_model::{LineRole, MM_PER_IN, Margins};
use serde::{Deserialize, Serialize};

/// One target screen (napkin §3.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Profile {
    /// Stable id, e.g. `supernote.a5x2`.
    pub id: String,
    /// Name shown to the user.
    pub display_name: String,
    /// Native portrait width in pixels.
    pub width_px: u32,
    /// Native portrait height in pixels.
    pub height_px: u32,
    /// Pixels per inch, for mm ↔ px.
    pub ppi: f64,
    /// Screen diagonal in inches (informational).
    pub diagonal_in: f64,
    /// Panel name (informational).
    pub panel: String,
    /// Gray values the panel renders cleanly.
    pub gray_levels: Vec<u8>,
    /// Gray values pens use, for "competes with ink" checks.
    pub ink_values: Vec<u8>,
    /// Areas covered by system UI.
    pub safe_zones: Vec<SafeZone>,
    /// Suggested page margins.
    pub default_margins_mm: Margins,
    /// Supported template orientations.
    pub orientations: Vec<Orientation>,
    /// Width and gray per line role (§8.2).
    pub line_roles: LineRoles,
    /// Where the preset values came from.
    pub calibration_meta: CalibrationMeta,
    /// Where to copy files on the device; shown as instructions only.
    pub export_hints: Vec<String>,
}

/// An area covered by system UI, in device pixels (napkin §3.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SafeZone {
    /// e.g. `toolbar_left`.
    pub name: String,
    /// Which orientation it applies to.
    pub orientation: Orientation,
    /// (x, y, width, height) in device pixels.
    pub rect: (u32, u32, u32, u32),
    /// A movable zone: shown, but linted as Info only.
    pub advisory: bool,
}

/// Page orientation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Taller than wide.
    Portrait,
    /// Wider than tall.
    Landscape,
}

/// A role's stroke on this profile.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleStyle {
    /// Stroke width in whole device pixels.
    pub width_px: u32,
    /// 0 = black, 255 = white.
    pub gray: u8,
}

/// Each role's values on this profile (napkin §8.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LineRoles {
    /// Faint guides.
    pub whisper: RoleStyle,
    /// Standard ruling.
    pub guide: RoleStyle,
    /// Dividers and borders.
    pub structure: RoleStyle,
    /// Headers and strong separators.
    pub emphasis: RoleStyle,
    /// A dot grid's dots: `width_px` is the dot's side.
    pub dot_grid: RoleStyle,
    /// The Text role (headings, reading text) sets gray only.
    pub text_gray: u8,
    /// The Label role (quiet labels) sets gray only.
    pub label_gray: u8,
}

/// Provenance of the preset values (napkin §3.2, §8.4).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CalibrationMeta {
    /// When calibrated, if ever.
    pub date: Option<String>,
    /// Device firmware at calibration.
    pub firmware: Option<String>,
    /// How the values were found.
    pub method: String,
    /// True only after calibration on real hardware.
    pub verified: bool,
}

impl Profile {
    /// Millimetres to device pixels (not rounded).
    pub fn mm_to_px(&self, mm: f64) -> f64 {
        mm * self.ppi / MM_PER_IN
    }

    /// Page size in mm (portrait).
    pub fn page_mm(&self) -> (f64, f64) {
        (
            self.width_px as f64 * MM_PER_IN / self.ppi,
            self.height_px as f64 * MM_PER_IN / self.ppi,
        )
    }

    /// A role's width and gray. The Text and Label roles have no width of its own; a line using
    /// one is 1 px.
    pub fn role(&self, role: LineRole) -> RoleStyle {
        let r = &self.line_roles;
        match role {
            LineRole::Whisper => r.whisper,
            LineRole::Guide => r.guide,
            LineRole::Structure => r.structure,
            LineRole::Emphasis => r.emphasis,
            LineRole::DotGrid => r.dot_grid,
            LineRole::Text => RoleStyle {
                width_px: 1,
                gray: r.text_gray,
            },
            LineRole::Label => RoleStyle {
                width_px: 1,
                gray: r.label_gray,
            },
        }
    }
}

/// The RON source of each built-in profile, by id.
const BUILTIN_RON: &[(&str, &str)] = &[
    ("supernote.a5x2", include_str!("../data/supernote.a5x2.ron")),
    ("supernote.a6x2", include_str!("../data/supernote.a6x2.ron")),
];

/// Why a profile could not be loaded.
#[derive(Debug)]
pub enum ProfileError {
    /// No built-in profile has this id.
    Unknown(String),
    /// The RON did not parse.
    Parse(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::Unknown(id) => write!(f, "no built-in profile `{id}`"),
            ProfileError::Parse(e) => write!(f, "profile does not parse: {e}"),
        }
    }
}

impl std::error::Error for ProfileError {}

/// Parses a profile from RON text.
pub fn from_ron(src: &str) -> Result<Profile, ProfileError> {
    ron::from_str(src).map_err(|e| ProfileError::Parse(e.to_string()))
}

/// The ids of the built-in profiles.
pub fn builtin_ids() -> impl Iterator<Item = &'static str> {
    BUILTIN_RON.iter().map(|(id, _)| *id)
}

/// A built-in profile by id.
pub fn builtin(id: &str) -> Result<Profile, ProfileError> {
    let (_, src) = BUILTIN_RON
        .iter()
        .find(|(i, _)| *i == id)
        .ok_or_else(|| ProfileError::Unknown(id.into()))?;
    from_ron(src)
}
