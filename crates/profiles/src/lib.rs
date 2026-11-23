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

pub mod active;
pub mod limits;
pub mod registry;

pub use quadrule_model::Orientation;
use quadrule_model::{DEFAULT_PAPER_PPI, Fill, FillRole, LineRole, MM_PER_IN, Margins, PAPER_PPIS};
pub use registry::{Damaged, Incoming, Registry, Source};
use serde::{Deserialize, Serialize};

/// One target screen (napkin §3.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Profile {
    /// Stable id, e.g. `supernote.a5x2`.
    pub id: String,
    /// Name shown to the user.
    pub display_name: String,
    /// Short name for file names, the `{profile}` token of napkin §11.2 (`manta`, `nomad`; chris,
    /// 2026-11-09: "Just 'manta' / 'nomad'"). ASCII lowercase letters, digits and hyphens.
    pub short_name: String,
    /// Native portrait width in pixels.
    pub width_px: u32,
    /// Native portrait height in pixels.
    pub height_px: u32,
    /// Pixels per inch, for mm ↔ px.
    pub ppi: f64,
    /// Set on a paper profile (napkin §3.3.1; rung 13b): its physical size. The pixel size and
    /// the line widths are then those at 300 PPI, the default; [`Profile::at_ppi`] gives them at
    /// the PPI a page picks. A screen has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paper: Option<PaperSize>,
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
    /// The gray of each fill role (rung 6, format 0.5).
    pub fill_roles: FillRoles,
    /// Where the preset values came from.
    pub calibration_meta: CalibrationMeta,
    /// What the linter judges a page against on this profile (format 0.15; rung 14a bbq, Q2). Every
    /// value is optional: a file without the block reads as all empty, and a rule whose value is empty
    /// is skipped on this profile.
    #[serde(default)]
    pub lint: LintThresholds,
    /// Where to copy files on the device; shown as instructions only.
    pub export_hints: Vec<String>,
    /// Set on the copy [`Profile::oriented`] makes for a landscape design: `width_px` and
    /// `height_px` are then the design's (the device's turned), the safe zones are in the design's
    /// frame, and [`Profile::device_size`] still gives the device's own size. Never in a profile
    /// file.
    #[serde(skip)]
    pub turned: bool,
}

/// A paper profile's physical size in millimetres (napkin §3.3.1; rung 13b bbq, Q12).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PaperSize {
    /// The short side or the width, in mm.
    pub width_mm: f64,
    /// The long side or the height, in mm.
    pub height_mm: f64,
}

impl PaperSize {
    /// The size in whole pixels at `ppi`: mm × PPI ÷ 25.4, rounded, as napkin §3.3.1's table has
    /// it (A4 at 300 PPI is 2480 × 3508).
    pub fn px_at(self, ppi: u32) -> (u32, u32) {
        let px = |mm: f64| (mm * f64::from(ppi) / MM_PER_IN).round().clamp(1.0, 1.0e6) as u32;
        (px(self.width_mm), px(self.height_mm))
    }
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

impl SafeZone {
    /// The zone in a landscape design's frame, from the device's own (portrait) frame: a device
    /// pixel (px, py) is the design pixel (x, y) with px = H - 1 - y and py = x, where H is the
    /// design's height, the device's width `device_w`. So the rectangle (x, y, w, h) covers the
    /// design's (y, device_w - x - w, h, w).
    fn turned_into_design(&self, device_w: u32) -> SafeZone {
        let (x, y, w, h) = self.rect;
        SafeZone {
            rect: (y, device_w.saturating_sub(x + w), h, w),
            ..self.clone()
        }
    }

    /// The inverse of [`SafeZone::turned_into_design`].
    fn turned_back(&self, device_w: u32) -> SafeZone {
        let (x, y, w, h) = self.rect;
        SafeZone {
            rect: (device_w.saturating_sub(y + h), x, h, w),
            ..self.clone()
        }
    }
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
    /// Wide, light lines such as a notebook's margin rule (ADR-020). A profile saved before the role existed reads
    /// as [`default_margin`].
    #[serde(default = "default_margin")]
    pub margin: RoleStyle,
}

/// The Margin role before it is measured: chris's drawn 0.45 mm at 300 PPI, to whole pixels, and gray 204 (ADR-020,
/// item 3; chosen, not measured).
pub fn default_margin() -> RoleStyle {
    RoleStyle {
        width_px: 5,
        gray: 204,
    }
}

/// Each fill role's gray on this profile (rung 6 bbq, Q3 and Q7). A fill has no width, so a role
/// is a gray only.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillRoles {
    /// The Shade role: a light tint. 0 = black, 255 = white.
    pub shade: u8,
    /// The Block role: a dark solid.
    pub block: u8,
}

/// The numbers the linter judges a page against on one profile (rung 14a bbq, Q2 to Q4). Each is
/// optional; an empty one means "no value here", and the rule that reads it is skipped on this
/// profile (the Lint tab says so). Grays are 0 = black .. 255 = white; "lightest" is the lightest
/// gray that still shows, so a gray above it is too light.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct LintThresholds {
    /// L03: the lightest gray a 1 px stroke may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_line_1px: Option<u8>,
    /// L03: the lightest gray a stroke 2 px or wider may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_line_2px_up: Option<u8>,
    /// L03: the lightest gray a square dot 2 px wide may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_dot_2px: Option<u8>,
    /// L03: the lightest gray a square dot 3 px wide may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_dot_3px: Option<u8>,
    /// L03: the lightest gray a square dot 4 px or wider may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_dot_4px_up: Option<u8>,
    /// L03: the lightest gray a fill may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_fill: Option<u8>,
    /// L03: the lightest gray text may have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightest_gray_text: Option<u8>,
    /// L07: the smallest text size that reads, in points.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smallest_text_pt: Option<f64>,
    /// L09: the smallest comfortable handwriting step, in millimetres.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smallest_handwriting_step_mm: Option<f64>,
    /// L13: a fill at this gray or darker counts as a dark fill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_fill_gray: Option<u8>,
    /// L13: the share of the page (0 to 1) dark fills may cover before it is noted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_fill_page_share: Option<f64>,
    /// L14: a stroke this many device pixels wide or thinner counts as a thin line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thin_line_px: Option<u32>,
}

impl LintThresholds {
    /// Whether every value is empty.
    pub fn is_empty(&self) -> bool {
        *self == LintThresholds::default()
    }

    /// The lightest gray a 1 px stroke may have, or a wider one, by the stroke's width in pixels.
    pub fn lightest_gray_line(&self, width_px: u32) -> Option<u8> {
        if width_px <= 1 {
            self.lightest_gray_line_1px
        } else {
            self.lightest_gray_line_2px_up
        }
    }

    /// The lightest gray a square dot `side_px` wide may have: the value for the nearest listed size at
    /// or below it, and for a dot under 2 px the 2 px value (a smaller dot is judged at least as hard).
    pub fn lightest_gray_dot(&self, side_px: u32) -> Option<u8> {
        match side_px {
            0..=2 => self.lightest_gray_dot_2px,
            3 => self.lightest_gray_dot_3px,
            _ => self.lightest_gray_dot_4px_up,
        }
    }
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
    /// Whether this is a paper profile (a physical size), not a screen.
    pub fn is_paper(&self) -> bool {
        self.paper.is_some()
    }

    /// A paper profile made consistent: its pixel size is the physical size at 300 PPI, its ppi 300
    /// and its page held upright. A screen is left alone. Reading a file runs it, so the pixels
    /// and the size in mm never disagree. Its safe zones, gray levels and pen grays are emptied: a
    /// paper size has none (napkin §3.3.1).
    pub fn sync_paper(&mut self) {
        if let Some(p) = self.paper {
            let (w, h) = p.px_at(DEFAULT_PAPER_PPI);
            self.width_px = w.min(h);
            self.height_px = w.max(h);
            self.ppi = f64::from(DEFAULT_PAPER_PPI);
            self.turned = false;
            // No device-specific data (napkin §3.3.1).
            self.safe_zones.clear();
            self.gray_levels.clear();
            self.ink_values.clear();
        }
    }

    /// This paper profile at `ppi` (one of [`PAPER_PPIS`], else 300): the physical size in whole
    /// pixels, and each line width scaled by `ppi / 300`, rounded, at least 1 (rung 13b bbq, Q13:
    /// the role values are the Nomad's by physical size). Grays stay. A screen comes back as it
    /// is, and so does a paper profile asked for the PPI it already has. The page keeps the way it
    /// is held (wide stays wide).
    pub fn at_ppi(&self, ppi: u32) -> Profile {
        let Some(size) = self.paper else {
            return self.clone();
        };
        let ppi = if PAPER_PPIS.contains(&ppi) {
            ppi
        } else {
            DEFAULT_PAPER_PPI
        };
        let (a, b) = size.px_at(ppi);
        let wide = self.width_px > self.height_px;
        let mut out = self.clone();
        (out.width_px, out.height_px) = if wide {
            (a.max(b), a.min(b))
        } else {
            (a.min(b), a.max(b))
        };
        out.ppi = f64::from(ppi);
        // The stored widths are those at 300 PPI.
        let scale = |s: RoleStyle| RoleStyle {
            width_px: ((f64::from(s.width_px) * f64::from(ppi) / f64::from(DEFAULT_PAPER_PPI))
                .round() as u32)
                .max(1),
            gray: s.gray,
        };
        let r = &mut out.line_roles;
        r.whisper = scale(r.whisper);
        r.guide = scale(r.guide);
        r.structure = scale(r.structure);
        r.emphasis = scale(r.emphasis);
        r.dot_grid = scale(r.dot_grid);
        r.margin = scale(r.margin);
        out
    }

    /// The gray a fill role resolves to on this profile.
    pub fn fill_role_gray(&self, role: FillRole) -> u8 {
        match role {
            FillRole::Shade => self.fill_roles.shade,
            FillRole::Block => self.fill_roles.block,
        }
    }

    /// The gray of a fill: a role resolved on this profile, or the custom gray itself.
    pub fn fill_gray(&self, fill: Fill) -> u8 {
        match fill {
            Fill::Role(r) => self.fill_role_gray(r),
            Fill::Gray(g) => g,
        }
    }

    /// Millimetres to device pixels (not rounded).
    pub fn mm_to_px(&self, mm: f64) -> f64 {
        mm * self.ppi / MM_PER_IN
    }

    /// Page size in mm: the device's portrait page, or the turned one on a profile made by
    /// [`Profile::oriented`] for a landscape design.
    pub fn page_mm(&self) -> (f64, f64) {
        (
            self.width_px as f64 * MM_PER_IN / self.ppi,
            self.height_px as f64 * MM_PER_IN / self.ppi,
        )
    }

    /// The device's own (portrait) size in pixels: what a template PNG for it measures, whichever
    /// way the design is held.
    pub fn device_size(&self) -> (u32, u32) {
        if self.turned {
            (self.height_px, self.width_px)
        } else {
            (self.width_px, self.height_px)
        }
    }

    /// This profile as a design held `orientation` sees it (rung 8b, Q6): for landscape, the page
    /// is the device's height by its width, and each safe zone is turned into the design's frame by
    /// the inverse of the export turn, so a design pixel (x, y) and the device pixel it exports to
    /// agree about what is covered. The device's left toolbar lands along the design's bottom, its
    /// top toolbar along the design's left, its right toolbar along the design's top and its
    /// footers along the design's right. Zone names stay the device's (a "toolbar_left" zone is
    /// still the left toolbar, wherever it falls in the design). For portrait this is a copy.
    ///
    /// Asked of a profile that is already turned, it works from the device's own size, so the
    /// result never depends on how many times it is called.
    pub fn oriented(&self, orientation: Orientation) -> Profile {
        let mut out = self.clone();
        if self.is_paper() {
            // Paper is not turned: a landscape page is the wide one (rung 13b bbq, Q14), exported
            // as it is drawn, so `turned` stays false and there are no safe zones to move.
            let (short, long) = (
                self.width_px.min(self.height_px),
                self.width_px.max(self.height_px),
            );
            (out.width_px, out.height_px) = match orientation {
                Orientation::Portrait => (short, long),
                Orientation::Landscape => (long, short),
            };
            return out;
        }
        let (dw, dh) = self.device_size();
        match orientation {
            Orientation::Portrait => {
                if self.turned {
                    out.width_px = dw;
                    out.height_px = dh;
                    out.turned = false;
                    out.safe_zones = self.safe_zones.iter().map(|z| z.turned_back(dw)).collect();
                }
            }
            Orientation::Landscape => {
                if !self.turned {
                    out.width_px = dh;
                    out.height_px = dw;
                    out.turned = true;
                    out.safe_zones = self
                        .safe_zones
                        .iter()
                        .map(|z| z.turned_into_design(dw))
                        .collect();
                }
            }
        }
        out
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
            LineRole::Margin => r.margin,
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
    // The paper sizes (rung 13b), in the order a list shows them.
    ("paper.a3", include_str!("../data/paper.a3.ron")),
    ("paper.a4", include_str!("../data/paper.a4.ron")),
    ("paper.a5", include_str!("../data/paper.a5.ron")),
    ("paper.a6", include_str!("../data/paper.a6.ron")),
    ("paper.b5-iso", include_str!("../data/paper.b5-iso.ron")),
    ("paper.b5-jis", include_str!("../data/paper.b5-jis.ron")),
    ("paper.letter", include_str!("../data/paper.letter.ron")),
    ("paper.legal", include_str!("../data/paper.legal.ron")),
    ("paper.tabloid", include_str!("../data/paper.tabloid.ron")),
    (
        "paper.half-letter",
        include_str!("../data/paper.half-letter.ron"),
    ),
    (
        "paper.executive",
        include_str!("../data/paper.executive.ron"),
    ),
];

/// Why a profile could not be loaded.
#[derive(Debug)]
pub enum ProfileError {
    /// No built-in profile has this id.
    Unknown(String),
    /// The RON did not parse.
    Parse(String),
    /// The RON parsed but a value is outside the limits, or the file is too large.
    Invalid(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::Unknown(id) => write!(f, "no built-in profile `{id}`"),
            ProfileError::Parse(e) => write!(f, "profile does not parse: {e}"),
            ProfileError::Invalid(e) => write!(f, "profile is not acceptable: {e}"),
        }
    }
}

impl std::error::Error for ProfileError {}

/// Parses a profile from RON text.
pub fn from_ron(src: &str) -> Result<Profile, ProfileError> {
    ron::from_str(src).map_err(|e| ProfileError::Parse(e.to_string()))
}

/// Why a profile file could not be loaded, with the line it stopped at when RON said so.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadError {
    /// What is wrong, in English (the Profiles window shows it as it is).
    pub message: String,
    /// The line of the text, from 1.
    pub line: Option<usize>,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(l) => write!(f, "line {l}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

/// Parses profile text and checks it against the [`limits`]. Never panics, whatever `src` holds:
/// a file over [`limits::MAX_FILE_BYTES`] is refused before it is read.
pub fn parse_checked(src: &str) -> Result<Profile, LoadError> {
    if src.len() > limits::MAX_FILE_BYTES {
        return Err(LoadError {
            message: format!("the file is larger than {} bytes", limits::MAX_FILE_BYTES),
            line: None,
        });
    }
    let mut p: Profile = ron::from_str(src).map_err(|e| LoadError {
        message: e.code.to_string(),
        line: Some(e.span.start.line),
    })?;
    // A paper profile's size in mm is the truth; its pixels follow it.
    p.sync_paper();
    limits::check(&p).map_err(|message| LoadError {
        message,
        line: None,
    })?;
    Ok(p)
}

/// Like [`from_ron`], and checked against the [`limits`] (see [`parse_checked`]).
pub fn from_ron_checked(src: &str) -> Result<Profile, ProfileError> {
    parse_checked(src).map_err(|e| ProfileError::Invalid(e.to_string()))
}

/// A profile as RON text, the file Export… writes and a `.quad` carries. The same profile always
/// gives the same text.
pub fn to_ron(p: &Profile) -> String {
    ron::ser::to_string_pretty(p, ron::ser::PrettyConfig::default())
        .expect("a profile always serialises")
}

/// The ids of the built-in screens (the devices), in shipped order. A new page targets these; the
/// paper sizes are [`builtin_paper_ids`].
pub fn builtin_ids() -> impl Iterator<Item = &'static str> {
    BUILTIN_RON
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| !id.starts_with(PAPER_ID_PREFIX))
}

/// The ids of the built-in paper sizes (rung 13b), in shipped order.
pub fn builtin_paper_ids() -> impl Iterator<Item = &'static str> {
    BUILTIN_RON
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| id.starts_with(PAPER_ID_PREFIX))
}

/// What every built-in paper id starts with.
pub const PAPER_ID_PREFIX: &str = "paper.";

/// A built-in profile as a page sees it at export PPI `ppi` (paper only), held `orientation`: the
/// shipped values, which is what the golden images and tests use.
pub fn builtin_at(id: &str, orientation: Orientation, ppi: u32) -> Result<Profile, ProfileError> {
    Ok(builtin(id)?.at_ppi(ppi).oriented(orientation))
}

/// A built-in profile by id, as a design held `orientation` sees it ([`Profile::oriented`]).
pub fn builtin_for(id: &str, orientation: Orientation) -> Result<Profile, ProfileError> {
    Ok(builtin(id)?.oriented(orientation))
}

/// A built-in profile by id: the device's own portrait page.
pub fn builtin(id: &str) -> Result<Profile, ProfileError> {
    let (_, src) = BUILTIN_RON
        .iter()
        .find(|(i, _)| *i == id)
        .ok_or_else(|| ProfileError::Unknown(id.into()))?;
    from_ron(src)
}
