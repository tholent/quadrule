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

//! Quadrule's document model (napkin §5). Geometry is stored in millimetres as `f64` (§5.3);
//! each profile turns it into device pixels at layout time.
//!
//! Rung 1 holds only what the headless slice draws. A field that changes the output goes in
//! only when layout honours it, so no stored setting is ever silently ignored. Every record
//! that may grow carries an `extra` bucket that keeps fields from newer versions (ADR-001
//! item 4); each record type has a test that an unknown field survives a save.

#![forbid(unsafe_code)]

pub mod edit;
pub mod outlines;
pub mod sample;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use uuid::Uuid;

/// Fields from a newer version that this build does not understand, kept so they survive a
/// save (napkin §12.2, ADR-001 item 4).
pub type Extra = BTreeMap<String, ciborium::Value>;

/// Millimetres per inch.
pub const MM_PER_IN: f64 = 25.4;
/// Typographic points per inch.
pub const PT_PER_IN: f64 = 72.0;

/// Converts typographic points to millimetres.
pub fn pt_to_mm(pt: f64) -> f64 {
    pt * MM_PER_IN / PT_PER_IN
}

/// The whole design (napkin §5.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Document {
    /// Title, author and the like.
    pub metadata: Metadata,
    /// Template or wallpaper (§4). Changes lint rules only, not layout.
    pub document_type: DocumentType,
    /// The profile the user designs on; others derive from it through constraints (§3.4).
    pub primary_profile_id: String,
    /// Primary first, then secondaries. Text outlines are saved for exactly these (ADR-001 item 2).
    pub profiles: Vec<ProfileContext>,
    /// Display unit preference. Has no effect on output.
    pub units: Unit,
    /// Background and margins.
    pub page: PageSettings,
    /// The page's rhythm (§6.3).
    pub rhythm: Rhythm,
    /// Drawn in order: the first layer is the bottom one.
    pub layers: Vec<Layer>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

impl Document {
    /// The ids of the document's profiles, primary first.
    pub fn profile_ids(&self) -> Vec<&str> {
        self.profiles
            .iter()
            .map(|p| p.profile_id.as_str())
            .collect()
    }

    /// Every object in the document, in drawing order.
    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        self.layers.iter().flat_map(|l| l.objects.iter())
    }
}

/// Descriptive fields (napkin §5.1, §12.3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Metadata {
    /// The document's title; also used in export file names.
    pub title: String,
    /// Who made it.
    pub author: String,
    /// Licence for sharing, e.g. `CC-BY-4.0`.
    pub license: String,
    /// Free text.
    pub description: String,
    /// Search tags.
    pub tags: Vec<String>,
    /// RFC 3339 date-time.
    pub created: String,
    /// RFC 3339 date-time.
    pub modified: String,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// What the page is for (napkin §4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    /// Background under handwriting.
    Template,
    /// Lock or sleep screen art (v1.1).
    Wallpaper,
}

/// Display unit preference (napkin §5.1).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Device pixels.
    Px,
    /// Millimetres.
    Mm,
    /// Points.
    Pt,
    /// Inches.
    In,
}

/// One of the document's target profiles (napkin §3.4). Rung 1 has linked profiles only;
/// forking arrives in v1.1.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileContext {
    /// A profile id such as `supernote.a5x2`.
    pub profile_id: String,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Page-wide settings (napkin §5.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PageSettings {
    /// 0 = black, 255 = white. The export is flattened onto it (§11.1).
    pub background_gray: u8,
    /// Page margins, physical millimetres on every profile.
    pub margins_mm: Margins,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Four margins in millimetres.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Margins {
    /// Top margin.
    pub top: f64,
    /// Right margin.
    pub right: f64,
    /// Bottom margin.
    pub bottom: f64,
    /// Left margin.
    pub left: f64,
}

impl Margins {
    /// The same margin on all four sides.
    pub fn all(mm: f64) -> Self {
        Margins {
            top: mm,
            right: mm,
            bottom: mm,
            left: mm,
        }
    }
}

/// The page's rhythm (napkin §6.3). Rung 1 has the rectangular kind only.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Rhythm {
    /// Horizontal unit in mm.
    pub h_unit_mm: f64,
    /// Vertical unit in mm; the line spacing of ruled templates.
    pub v_unit_mm: f64,
    /// Where the rhythm starts.
    pub origin: RhythmOrigin,
    /// How each profile turns the unit into pixels.
    pub resolution: RhythmResolution,
    /// Optional finer steps for snapping and Shift+arrow: this many per unit, 2 to 10 (napkin
    /// §6.3). `None` means off. Format 0.4; a file without it reads as off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdivisions: Option<u8>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The fewest subdivisions per rhythm unit (napkin §6.3).
pub const SUBDIVISIONS_MIN: u8 = 2;
/// The most subdivisions per rhythm unit (napkin §6.3).
pub const SUBDIVISIONS_MAX: u8 = 10;

impl Rhythm {
    /// The subdivisions per unit that count: `Some(n)` for 2 to 10, otherwise `None`. A file
    /// holding another number is read as if subdivisions were off, not rejected.
    pub fn subdivisions_on(&self) -> Option<u8> {
        self.subdivisions
            .filter(|n| (SUBDIVISIONS_MIN..=SUBDIVISIONS_MAX).contains(n))
    }
}

/// Where the rhythm starts (napkin §6.3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmOrigin {
    /// The top-left margin corner (the napkin's default).
    MarginCorner,
    /// The top-left page corner.
    PageCorner,
    /// Balanced in the margin box (the page, when margins are 0): as many units as leave at least
    /// half a unit at each side, the leftover split evenly, an odd pixel to the right and bottom
    /// (format 0.2). Supernote's stock grid on the Manta is placed this way.
    Centered,
}

/// How a profile resolves the rhythm unit to device pixels (napkin §6.3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmResolution {
    /// The unit rounds to a whole number of pixels: even spacing, no drift (the default).
    PixelLocked,
    /// Exact millimetres; each line snaps to its nearest pixel, so gaps can vary by 1 px.
    PhysicalLocked,
}

/// A layer: a named, ordered list of objects (napkin §5.5).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Layer {
    /// Shown in the layers panel.
    pub name: String,
    /// Hidden layers are not drawn or exported.
    pub visible: bool,
    /// Locked layers cannot be edited (no effect on output).
    pub locked: bool,
    /// Drawn in order: the first object is the bottom one.
    pub objects: Vec<Object>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// One drawable object (napkin §5.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Object {
    /// Stable identity; text outlines refer to it.
    pub id: Uuid,
    /// Shown in the layers panel.
    pub name: String,
    /// The object's box on the primary profile, in mm.
    pub frame: Frame,
    /// How the box adapts to other profiles (§5.4).
    pub constraints: Constraints,
    /// Hidden objects are not drawn or exported.
    pub visible: bool,
    /// Locked objects cannot be edited (no effect on output).
    pub locked: bool,
    /// Line role or explicit stroke and gray (§8.2).
    pub style: Style,
    /// What the object is.
    pub kind: ObjectKind,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An axis-aligned box in mm, from the page's top-left corner, y down.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Left edge.
    pub x_mm: f64,
    /// Top edge.
    pub y_mm: f64,
    /// Width; 0 for a vertical line.
    pub w_mm: f64,
    /// Height; 0 for a horizontal line.
    pub h_mm: f64,
}

impl Frame {
    /// A frame from its four values.
    pub fn new(x_mm: f64, y_mm: f64, w_mm: f64, h_mm: f64) -> Self {
        Frame {
            x_mm,
            y_mm,
            w_mm,
            h_mm,
        }
    }
}

/// Horizontal and vertical constraints (napkin §5.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constraints {
    /// Horizontal rule.
    pub h: HConstraint,
    /// Vertical rule.
    pub v: VConstraint,
}

impl Default for Constraints {
    /// Scale on both axes, the napkin's default.
    fn default() -> Self {
        Constraints {
            h: HConstraint::Scale,
            v: VConstraint::Scale,
        }
    }
}

/// How an object's horizontal position and width follow the page (napkin §5.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HConstraint {
    /// Keeps its distance from the left edge and its width.
    Left,
    /// Keeps its distance from the right edge and its width.
    Right,
    /// Keeps both distances; the width stretches.
    LeftRight,
    /// Keeps its offset from the centre and its width.
    Center,
    /// Position and width scale with the page width (the default).
    Scale,
}

/// How an object's vertical position and height follow the page (napkin §5.4).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VConstraint {
    /// Keeps its distance from the top edge and its height.
    Top,
    /// Keeps its distance from the bottom edge and its height.
    Bottom,
    /// Keeps both distances; the height stretches.
    TopBottom,
    /// Keeps its offset from the centre and its height.
    Center,
    /// Position and height scale with the page height (the default).
    Scale,
}

/// A semantic line role; each profile resolves it to a width and gray (napkin §8.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineRole {
    /// Faint guides that fade under writing.
    Whisper,
    /// Standard ruling lines.
    Guide,
    /// Section dividers, box borders.
    Structure,
    /// Headers, strong separators.
    Emphasis,
    /// Headings and reading text (gray only).
    Text,
    /// The dots of a dot grid: the dot's size and gray, apart from the line roles. Format 0.3
    /// (chris, 2026-11-09: "Yes, own setting"; named "Dot grid").
    DotGrid,
    /// Quiet labels, lighter than Text (gray only). Format 0.3 (chris, 2026-11-09:
    /// "Text + new Label").
    Label,
}

impl LineRole {
    /// Every role, in order.
    pub const ALL: [LineRole; 7] = [
        LineRole::Whisper,
        LineRole::Guide,
        LineRole::Structure,
        LineRole::Emphasis,
        LineRole::DotGrid,
        LineRole::Text,
        LineRole::Label,
    ];
}

/// An object's look (napkin §8.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Style {
    /// Width and gray come from the profile's role values.
    Role(LineRole),
    /// Explicit values.
    Custom {
        /// Stroke width in mm; rounds to whole device pixels, at least 1.
        width_mm: f64,
        /// 0 = black, 255 = white.
        gray: u8,
    },
}

/// What an object is (napkin §5.2). Rung 1 has lines, text boxes and pattern regions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObjectKind {
    /// A straight line from the frame's top-left to its bottom-right corner.
    Line(Line),
    /// A paragraph text box (§7.1).
    Text(TextBox),
    /// A parametric fill clipped to the frame (§5.2).
    Pattern(PatternRegion),
}

/// A straight line. Its geometry is the frame's diagonal, so constraints apply unchanged.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Line {
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A paragraph text box with one style (napkin §7.1). Rung 1 boxes grow downward (auto
/// height); fixed boxes, paragraph spacing and vertical alignment arrive later (ADR-002 item 3).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextBox {
    /// The text. `\n` starts a new paragraph.
    pub content: String,
    /// Font family name, e.g. `Atkinson Hyperlegible Next`.
    pub family: String,
    /// CSS weight, 100 to 900; drives a variable font's weight axis.
    pub weight: u16,
    /// Font size in points, a fixed physical size on every profile (§5.4).
    pub size_pt: f64,
    /// Line height as a multiple of the font size.
    pub line_height: f64,
    /// Extra space between letters, in em.
    pub letter_spacing_em: f64,
    /// Horizontal alignment.
    pub align: TextAlign,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Horizontal text alignment.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    /// Ragged right.
    Left,
    /// Centred.
    Center,
    /// Ragged left.
    Right,
    /// Every line but the last reaches both edges.
    Justify,
}

/// A pattern region (napkin §5.2). Rung 1 regions are always bound to the rhythm.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PatternRegion {
    /// Which pattern.
    pub pattern: Pattern,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The patterns rung 1 draws.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// Horizontal ruled lines, one per vertical rhythm unit.
    Lines,
    /// A dot at every rhythm crossing.
    Dots,
    /// A square grid: horizontal and vertical lines on the rhythm (format 0.2).
    Grid,
}
