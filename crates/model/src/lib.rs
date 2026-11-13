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

pub mod blank;
pub mod clipboard;
pub mod edit;
pub mod geom;
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

    /// Every object in the document, groups and everything inside them (format 0.7) included, in
    /// drawing order: a group comes before its children.
    pub fn objects(&self) -> impl Iterator<Item = &Object> {
        let mut out = Vec::new();
        for l in &self.layers {
            collect_objects(&l.objects, &mut out);
        }
        out.into_iter()
    }

    /// Every object that is not a group, in drawing order: what is actually drawn.
    pub fn leaves(&self) -> impl Iterator<Item = &Object> {
        self.objects().filter(|o| !o.is_group())
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

/// One of the document's target profiles (napkin §3.4). Every secondary profile is linked to the
/// primary one (it adapts through constraints, and per-object overrides, format 0.8, change single
/// things on it); forking a profile's design off arrives in v1.1 (rung 8 bbq, Q2).
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
    /// Which way the page is held (format 0.8; rung 8 bbq, Defaulted "File format 0.8.0"). Portrait
    /// unless the file says otherwise; only portrait is offered until rung 8b.
    #[serde(default, skip_serializing_if = "Orientation::is_portrait")]
    pub orientation: Orientation,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Which way the page is held (format 0.8).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Taller than wide: the device's own page (the default, and every file before 0.8).
    #[default]
    Portrait,
    /// Wider than tall (rung 8b): exported turned into the device's portrait pixels.
    Landscape,
}

impl Orientation {
    /// Whether this is the default; used to leave the field out of a file when it is.
    pub fn is_portrait(&self) -> bool {
        matches!(self, Orientation::Portrait)
    }
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
    /// Rectangular (the default, and every file before 0.9) or isometric (format 0.9; rung 9 bbq,
    /// Q7). Under an isometric rhythm the units are derived from the side length
    /// ([`Rhythm::steps_mm`]), and `h_unit_mm` and `v_unit_mm` are kept for when it is switched back.
    #[serde(default, skip_serializing_if = "RhythmType::is_rectangular")]
    pub rhythm_type: RhythmType,
    /// The side of an isometric rhythm's triangles in mm (format 0.9); `None` reads as
    /// [`ISOMETRIC_SIDE_DEFAULT_MM`]. Unused by a rectangular rhythm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_mm: Option<f64>,
    /// "Major every N": every Nth rhythm unit counts as major, for pattern regions and the overlay
    /// (format 0.9; rung 9 bbq, Q6). `None` means off, which is what every file before 0.9 reads as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub major_every: Option<u8>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The kind of a rhythm (napkin §6.3, format 0.9).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RhythmType {
    /// Independent horizontal and vertical units (the default).
    #[default]
    Rectangular,
    /// One side length; axes at 0, 60 and 120 degrees. Snapping uses half a side across and one
    /// row (side times the square root of three over two) down.
    Isometric,
}

impl RhythmType {
    /// Whether this is the default; used to leave the field out of a file when it is.
    pub fn is_rectangular(&self) -> bool {
        *self == RhythmType::Rectangular
    }
}

/// The side of an isometric rhythm when the file gives none (the "Isometric 5 mm" preset).
pub const ISOMETRIC_SIDE_DEFAULT_MM: f64 = 5.0;
/// Half the square root of three: a row of an isometric grid is its side times this.
pub const HALF_SQRT_3: f64 = 0.866_025_403_784_438_6;
/// The fewest rhythm units between major lines (2: every other one).
pub const MAJOR_MIN: u8 = 2;
/// The most rhythm units between major lines.
pub const MAJOR_MAX: u8 = 50;

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

    /// "Major every N" when it is on: `Some(n)` for 2 to 50, otherwise `None`. A file holding
    /// another number is read as off, not rejected.
    pub fn major_on(&self) -> Option<u8> {
        self.major_every
            .filter(|n| (MAJOR_MIN..=MAJOR_MAX).contains(n))
    }

    /// Whether the rhythm is isometric.
    pub fn is_isometric(&self) -> bool {
        self.rhythm_type == RhythmType::Isometric
    }

    /// The isometric side in mm: the one in the file, or the default.
    pub fn side(&self) -> f64 {
        self.side_mm
            .filter(|s| s.is_finite() && *s > 0.0)
            .unwrap_or(ISOMETRIC_SIDE_DEFAULT_MM)
    }

    /// The steps snapping, Shift+arrow and drawn sizes use, in mm: (across, down). A rectangular
    /// rhythm gives its two units; an isometric one half a side across and one row down, the
    /// rectangular grid that holds every isometric crossing (rung 9 bbq, Q7).
    pub fn steps_mm(&self) -> (f64, f64) {
        match self.rhythm_type {
            RhythmType::Rectangular => (self.h_unit_mm, self.v_unit_mm),
            RhythmType::Isometric => (self.side() / 2.0, self.side() * HALF_SQRT_3),
        }
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
    /// How the box adapts to other profiles (§5.4), against the page. A group has none of its own:
    /// this is ignored for a group (its box is always round its children's, which each keep their
    /// own constraints against the page; rung 7a look change, docs/bbq/2026-11-11-rung-7-groups-and-align.md,
    /// Q3 as amended). It stays in the file so every object has the same shape.
    pub constraints: Constraints,
    /// Hidden objects are not drawn or exported.
    pub visible: bool,
    /// Locked objects cannot be edited (no effect on output).
    pub locked: bool,
    /// Line role or explicit stroke and gray (§8.2).
    pub style: Style,
    /// The fill of a shape (rectangle, ellipse, polygon); `None` for every other object. Format
    /// 0.5; a file without it reads as no fill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<Fill>,
    /// The stroke's dash. Format 0.5; a file without it reads as solid.
    #[serde(default, skip_serializing_if = "DashStyle::is_solid")]
    pub dash: DashStyle,
    /// The stroke's end caps. Format 0.5; a file without it reads as butt.
    #[serde(default, skip_serializing_if = "LineCap::is_butt")]
    pub cap: LineCap,
    /// Rotation in degrees, clockwise on the page, about the centre of the frame; the frame stays
    /// the unrotated box (rung 6 bbq, Defaulted "6b, settled now for its builder"). Format 0.6; a
    /// file without it reads as 0. Only lines and the shapes rotate: text boxes and pattern
    /// regions ignore it, so use [`Object::rotation`] to read it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rotation_deg: f64,
    /// How opaque the object is, 0 to 100 percent; the export is flattened onto what lies beneath
    /// (napkin §5.2). Format 0.6; a file without it reads as 100.
    #[serde(default = "full_opacity", skip_serializing_if = "is_full_opacity")]
    pub opacity_pct: u8,
    /// What differs on one secondary device, by profile id (format 0.8; rung 8 bbq, Q3): its place
    /// and size, its font size and whether it shows. Empty for most objects, and left out of the
    /// file when it is. An override of the primary profile is never read (the primary shows the
    /// stored design); one for a device the document does not target is kept, unused.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub overrides: BTreeMap<String, Override>,
    /// What the object is.
    pub kind: ObjectKind,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

fn collect_objects<'a>(objs: &'a [Object], out: &mut Vec<&'a Object>) {
    for o in objs {
        out.push(o);
        if let ObjectKind::Group(g) = &o.kind {
            collect_objects(&g.children, out);
        }
    }
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

fn full_opacity() -> u8 {
    OPACITY_MAX
}

fn is_full_opacity(v: &u8) -> bool {
    *v >= OPACITY_MAX
}

/// The least opaque an object can be (fully see-through, so it draws nothing).
pub const OPACITY_MIN: u8 = 0;
/// Fully opaque, what every object was before format 0.6.
pub const OPACITY_MAX: u8 = 100;

/// An angle in degrees brought into 0 up to (not including) 360; not-a-number and infinity give 0.
pub fn normalize_deg(deg: f64) -> f64 {
    if !deg.is_finite() {
        return 0.0;
    }
    let d = deg.rem_euclid(360.0);
    // `rem_euclid` can give 360.0 for a tiny negative number.
    if d >= 360.0 { 0.0 } else { d }
}

impl Object {
    /// The rotation that counts, in degrees from 0 up to (not including) 360: the stored one for a
    /// line or a shape, always 0 for a text box or a pattern region, which do not rotate (rung 6b).
    pub fn rotation(&self) -> f64 {
        if self.kind.can_rotate() {
            normalize_deg(self.rotation_deg)
        } else {
            0.0
        }
    }

    /// The opacity that counts, 0 to 100 percent.
    pub fn opacity(&self) -> u8 {
        self.opacity_pct.min(OPACITY_MAX)
    }

    /// What differs on device `profile`, if anything.
    pub fn override_on(&self, profile: &str) -> Option<&Override> {
        self.overrides.get(profile)
    }

    /// Whether the object shows on device `profile`: its own eye, unless the device says
    /// otherwise.
    pub fn visible_on(&self, profile: &str) -> bool {
        self.override_on(profile)
            .and_then(|o| o.visible)
            .unwrap_or(self.visible)
    }

    /// Whether this is a group (format 0.7).
    pub fn is_group(&self) -> bool {
        matches!(self.kind, ObjectKind::Group(_))
    }

    /// A group's children, bottom first; none for any other object.
    pub fn children(&self) -> &[Object] {
        match &self.kind {
            ObjectKind::Group(g) => &g.children,
            _ => &[],
        }
    }

    /// Moves the object by (`dx`, `dy`) mm, and, for a group, everything inside it: children are
    /// stored in the group's unturned space, which moves with it.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        self.frame.x_mm += dx;
        self.frame.y_mm += dy;
        if let ObjectKind::Group(g) = &mut self.kind {
            for c in &mut g.children {
                c.translate(dx, dy);
            }
        }
    }

    /// The box round the object as it is placed in its parent's space: its frame turned by its
    /// rotation (a line: round its two ends). A group's box is its frame turned about its centre.
    pub fn outer_box(&self) -> Frame {
        self.outer_box_of(self.frame)
    }

    /// [`Object::outer_box`] for the object if its frame were `frame` (for instance as resolved on
    /// another device's page).
    pub fn outer_box_of(&self, frame: Frame) -> Frame {
        let line = match &self.kind {
            ObjectKind::Line(l) => Some(l.direction),
            _ => None,
        };
        if self.rotation() == 0.0 && line.is_none() {
            frame
        } else {
            frame.turned_bounds(self.rotation(), line)
        }
    }
}

/// What one object does differently on one secondary device (format 0.8, rung 8 bbq, Q3 and
/// Defaulted "File format 0.8.0"). Each part is optional and replaces what the object's constraints
/// give on that device; the rest still follows the design. Millimetres on that device's page, in
/// the space the object is laid out in (the page, unless it is inside a turned group).
///
/// A group's override is its box on that device (Q4): its children resolve as usual, and what they
/// make is then scaled into that box.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Override {
    /// The top-left corner of the object's box.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// The width and height of the object's box.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,
    /// A text box's font size in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size_pt: Option<f64>,
    /// Whether the object shows on this device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A place on a page in mm (format 0.8).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// Distance from the left edge.
    pub x_mm: f64,
    /// Distance from the top edge.
    pub y_mm: f64,
}

/// A width and height in mm (format 0.8).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// Width.
    pub w_mm: f64,
    /// Height.
    pub h_mm: f64,
}

impl Override {
    /// Whether nothing is overridden (an override like this is dropped, not saved).
    pub fn is_empty(&self) -> bool {
        self.position.is_none()
            && self.size.is_none()
            && self.font_size_pt.is_none()
            && self.visible.is_none()
            && self.extra.is_empty()
    }

    /// Whether the place or the size is overridden: what Q7's lint rule (L15) leaves alone.
    pub fn has_frame(&self) -> bool {
        self.position.is_some() || self.size.is_some()
    }

    /// How many parts are overridden (place, size, font size, eye).
    pub fn count(&self) -> usize {
        usize::from(self.position.is_some())
            + usize::from(self.size.is_some())
            + usize::from(self.font_size_pt.is_some())
            + usize::from(self.visible.is_some())
    }

    /// `frame` with the place and size replaced where they are overridden.
    pub fn apply_to(&self, frame: Frame) -> Frame {
        let mut f = frame;
        if let Some(p) = self.position {
            f.x_mm = p.x_mm;
            f.y_mm = p.y_mm;
        }
        if let Some(s) = self.size {
            f.w_mm = s.w_mm;
            f.h_mm = s.h_mm;
        }
        f
    }
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

/// A semantic fill role; each profile resolves it to a gray, and only a gray: a fill has no
/// width (rung 6 bbq, Q3 and Defaulted "File format 0.5.0"). Format 0.5.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillRole {
    /// A light tint for boxes and highlights.
    Shade,
    /// A dark solid, for header bars and the like.
    Block,
}

impl FillRole {
    /// Every fill role, in order.
    pub const ALL: [FillRole; 2] = [FillRole::Shade, FillRole::Block];
}

/// What fills a shape. Format 0.5.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// The gray the profile gives the role.
    Role(FillRole),
    /// An explicit gray: 0 = black, 255 = white.
    Gray(u8),
}

/// How a stroke is broken into dashes (rung 6 bbq, Q6). Lengths are rounded to whole device
/// pixels on each profile, at least 1. Format 0.5.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum DashStyle {
    /// An unbroken stroke (the default, and what every 0.4 object is).
    #[default]
    Solid,
    /// Dash and gap each as long as the stroke is wide.
    Dotted,
    /// 0.5 mm dash, 0.5 mm gap.
    Short,
    /// 1 mm dash, 0.5 mm gap.
    Dash,
    /// 2 mm dash, 1 mm gap.
    Long,
    /// Explicit lengths in mm.
    Custom {
        /// The dash.
        dash_mm: f64,
        /// The gap.
        gap_mm: f64,
    },
}

impl DashStyle {
    /// Every named preset, solid first (not Custom).
    pub const PRESETS: [DashStyle; 5] = [
        DashStyle::Solid,
        DashStyle::Dotted,
        DashStyle::Short,
        DashStyle::Dash,
        DashStyle::Long,
    ];

    /// Whether the stroke is unbroken; used to leave the field out of a file when it is.
    pub fn is_solid(&self) -> bool {
        matches!(self, DashStyle::Solid)
    }

    /// The dash and gap in mm, or `None` for a solid stroke or for Dotted, which depends on the
    /// stroke's width in pixels.
    pub fn lengths_mm(&self) -> Option<(f64, f64)> {
        match *self {
            DashStyle::Solid | DashStyle::Dotted => None,
            DashStyle::Short => Some((0.5, 0.5)),
            DashStyle::Dash => Some((1.0, 0.5)),
            DashStyle::Long => Some((2.0, 1.0)),
            DashStyle::Custom { dash_mm, gap_mm } => Some((dash_mm, gap_mm)),
        }
    }
}

/// How the ends of a stroke (and of each dash) are finished. Format 0.5.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    /// Cut square at the end point (the default).
    #[default]
    Butt,
    /// Rounded, reaching half the width past the end point.
    Round,
    /// Square, reaching half the width past the end point.
    Square,
}

impl LineCap {
    /// Every cap, the default first.
    pub const ALL: [LineCap; 3] = [LineCap::Butt, LineCap::Round, LineCap::Square];

    /// Whether this is the default cap; used to leave the field out of a file when it is.
    pub fn is_butt(&self) -> bool {
        matches!(self, LineCap::Butt)
    }
}

/// An object's look (napkin §8.2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Style {
    /// No stroke at all: a shape that is only a fill. Format 0.5.
    None,
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

/// What an object is (napkin §5.2). Lines, text boxes and pattern regions from the start;
/// rectangles, ellipses and polygons (stars included) from format 0.5.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObjectKind {
    /// A straight line along one diagonal of its frame (format 0.6; before, always the
    /// top-left to bottom-right one).
    Line(Line),
    /// A paragraph text box (§7.1).
    Text(TextBox),
    /// A parametric fill clipped to the frame (§5.2).
    Pattern(PatternRegion),
    /// A rectangle with optional rounded corners, filling its frame (format 0.5).
    Rect(RectShape),
    /// An ellipse inscribed in its frame (format 0.5).
    Ellipse(EllipseShape),
    /// A regular polygon or star, stretched to its frame (format 0.5).
    Polygon(PolygonShape),
    /// Objects kept together (format 0.7): moved, turned, resized and faded as one.
    Group(GroupShape),
}

/// A group (format 0.7, rung 7a bbq, Defaulted "File format 0.7.0 (7a)"): its children, bottom
/// first. A group uses the object's `frame`, `visible`, `locked`, `rotation_deg`, `opacity_pct`
/// and `name`; its `constraints` (kept so every object has the same shape) and its `style`, `fill`,
/// `dash` and `cap` are ignored. The children's frames are in page millimetres in the group's
/// unturned space, so grouping moves nothing, and each child resolves against the page by its own
/// constraints (groups are see-through to constraints, as in Figma). The group's box is the
/// children's outer box on every device, and is refitted to it after an edit inside it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GroupShape {
    /// What is in the group, drawn in order: the first is the bottom one. Groups may nest.
    pub children: Vec<Object>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A rectangle (format 0.5). Its geometry is the frame.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RectShape {
    /// Corner radius in mm; 0 is square corners. Drawn at most half the shorter side.
    pub corner_radius_mm: f64,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// An ellipse (format 0.5). Its geometry is the frame's inscribed ellipse; a circle when the
/// frame is square.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct EllipseShape {
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A regular polygon, or a star (format 0.5): the corners are placed on a circle with the first
/// one straight up, then the whole shape is stretched to fill the frame, so constraints apply
/// unchanged.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PolygonShape {
    /// Corners (or star points), [`POLYGON_SIDES_MIN`] to [`POLYGON_SIDES_MAX`].
    pub sides: u8,
    /// `Some(r)` makes a star: between the points the outline dips to `r` times the outer radius
    /// (0 to 1, exclusive of both ends in practice). `None` is a plain polygon.
    pub star: Option<f64>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The fewest sides a polygon has.
pub const POLYGON_SIDES_MIN: u8 = 3;
/// The most sides a polygon has.
pub const POLYGON_SIDES_MAX: u8 = 32;
/// The inner radius a new star starts with, as a fraction of the outer one.
pub const STAR_INNER_DEFAULT: f64 = 0.5;
/// The inner radius a star can be set to, at least and at most (fractions of the outer radius).
pub const STAR_INNER_MIN: f64 = 0.1;
/// See [`STAR_INNER_MIN`].
pub const STAR_INNER_MAX: f64 = 0.95;

impl PolygonShape {
    /// A new polygon of `sides`, not a star.
    pub fn polygon(sides: u8) -> Self {
        PolygonShape {
            sides,
            star: None,
            extra: Extra::new(),
        }
    }

    /// A new star of `points` points with the default inner radius.
    pub fn star(points: u8) -> Self {
        PolygonShape {
            sides: points,
            star: Some(STAR_INNER_DEFAULT),
            extra: Extra::new(),
        }
    }

    /// The side count that counts: clamped to the allowed range, so a file holding another number
    /// still draws.
    pub fn sides_clamped(&self) -> u8 {
        self.sides.clamp(POLYGON_SIDES_MIN, POLYGON_SIDES_MAX)
    }

    /// The star's inner radius that counts (clamped), or `None` for a plain polygon.
    pub fn star_clamped(&self) -> Option<f64> {
        self.star
            .map(|r| if r.is_finite() { r } else { STAR_INNER_DEFAULT })
            .map(|r| r.clamp(STAR_INNER_MIN, STAR_INNER_MAX))
    }
}

impl ObjectKind {
    /// Whether the kind can rotate (format 0.6): lines, the shapes and groups of such. A text box and
    /// a pattern region cannot, in 6b (rung 6 bbq, "Things I chose" of the 6b builder).
    pub fn can_rotate(&self) -> bool {
        match self {
            ObjectKind::Text(_) | ObjectKind::Pattern(_) => false,
            // A group turns only if nothing in it, at any depth, is a text box or a pattern region
            // (rung 7a bbq, Q2).
            ObjectKind::Group(g) => g.children.iter().all(|c| c.kind.can_rotate()),
            _ => true,
        }
    }

    /// Whether this is one of the shapes (rectangle, ellipse, polygon): the kinds that have a fill
    /// and a stroke that can be switched off.
    pub fn is_shape(&self) -> bool {
        matches!(
            self,
            ObjectKind::Rect(_) | ObjectKind::Ellipse(_) | ObjectKind::Polygon(_)
        )
    }
}

/// Which diagonal of its frame a line runs along (format 0.6). A line is always its frame's
/// diagonal, so constraints apply unchanged; this makes it point any way.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineDir {
    /// From the top-left corner to the bottom-right one (the default, and every 0.5 line).
    #[default]
    Down,
    /// From the bottom-left corner to the top-right one.
    Up,
}

impl LineDir {
    /// Whether this is the default direction; used to leave the field out of a file when it is.
    pub fn is_down(&self) -> bool {
        matches!(self, LineDir::Down)
    }

    /// The other diagonal.
    pub fn flipped(self) -> Self {
        match self {
            LineDir::Down => LineDir::Up,
            LineDir::Up => LineDir::Down,
        }
    }
}

/// A straight line. Its geometry is the frame's diagonal (which one: `direction`), so constraints
/// apply unchanged. Its start is its left end.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Line {
    /// Which diagonal. Format 0.6; a file without it reads as top-left to bottom-right. It does
    /// not matter for a line whose frame has no width or no height.
    #[serde(default, skip_serializing_if = "LineDir::is_down")]
    pub direction: LineDir,
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

/// A pattern region (napkin §5.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PatternRegion {
    /// Which pattern, with its own settings.
    pub pattern: Pattern,
    /// Bound to the rhythm (the default; napkin §5.2): the rhythm sets the spacing. Unticked, the
    /// region has its own spacing, counted from its own top-left corner (format 0.9; rung 9 bbq,
    /// Defaulted "Unbinding a region").
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub bound: bool,
    /// An unbound region's horizontal spacing in mm; `None` reads as the rhythm's horizontal step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h_spacing_mm: Option<f64>,
    /// An unbound region's vertical spacing in mm; `None` reads as the rhythm's vertical step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_spacing_mm: Option<f64>,
    /// Major lines: follow the rhythm's "Major every N", turn them off, or use an N of its own
    /// (rung 9 bbq, Q6).
    #[serde(default, skip_serializing_if = "MajorLines::is_follow")]
    pub major: MajorLines,
    /// The role major lines are drawn in (Structure unless changed).
    #[serde(default = "structure_role", skip_serializing_if = "is_structure_role")]
    pub major_role: LineRole,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

fn yes() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

fn structure_role() -> LineRole {
    LineRole::Structure
}

fn is_structure_role(r: &LineRole) -> bool {
    *r == LineRole::Structure
}

impl PatternRegion {
    /// A region of `pattern`, bound to the rhythm, with the rhythm's major lines in Structure.
    pub fn new(pattern: Pattern) -> Self {
        PatternRegion {
            pattern,
            bound: true,
            h_spacing_mm: None,
            v_spacing_mm: None,
            major: MajorLines::Follow,
            major_role: LineRole::Structure,
            extra: Extra::new(),
        }
    }

    /// The N for this region's major lines under `rhythm`: its own, the rhythm's, or `None` for
    /// off. Hex and music staff have none.
    pub fn major_every(&self, rhythm: &Rhythm) -> Option<u8> {
        if !self.pattern.has_major() {
            return None;
        }
        match self.major {
            MajorLines::Follow => rhythm.major_on(),
            MajorLines::Off => None,
            MajorLines::Every(n) => Some(n).filter(|n| (MAJOR_MIN..=MAJOR_MAX).contains(n)),
        }
    }
}

/// What a region does about major lines (format 0.9; rung 9 bbq, Q6).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MajorLines {
    /// Use the rhythm's "Major every N" (the default).
    #[default]
    Follow,
    /// No major lines in this region.
    Off,
    /// Every Nth line, whatever the rhythm says.
    Every(u8),
}

impl MajorLines {
    /// Whether this is the default; used to leave the field out of a file when it is.
    pub fn is_follow(&self) -> bool {
        *self == MajorLines::Follow
    }
}

/// How an isometric pattern is drawn (format 0.9; rung 9 bbq, Q2).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsoStyle {
    /// The triangle grid: horizontal rows and the two slanted families.
    #[default]
    Lines,
    /// A dot at each crossing.
    Dots,
}

/// Which way a hexagon sits (format 0.9; rung 9 bbq, Q3).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexOrientation {
    /// Top and bottom edges horizontal (the default).
    #[default]
    Flat,
    /// Two edges vertical.
    Pointy,
}

/// The pattern of a region, with its own settings. The first three are format 0.2 and before and
/// are written as plain names; the others (format 0.9, rung 9) carry their settings, each with a
/// default for a field a file leaves out.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Pattern {
    /// Horizontal ruled lines, one per vertical rhythm unit.
    Lines,
    /// A dot at every rhythm crossing.
    Dots,
    /// A square grid: horizontal and vertical lines on the rhythm (format 0.2).
    Grid,
    /// Horizontal rows with the two slanted families at 60 and 120 degrees, as lines or dots.
    Isometric {
        /// Lines or dots.
        #[serde(default)]
        style: IsoStyle,
    },
    /// A honeycomb of regular hexagons.
    Hex {
        /// Flat top or pointy top.
        #[serde(default)]
        orientation: HexOrientation,
        /// A hexagon's height across its flats in rhythm units (vertical for flat top, horizontal
        /// for pointy top).
        #[serde(default = "one")]
        height_units: u32,
    },
    /// Ruled lines with a cue column at the left and a summary band at the bottom.
    Cornell {
        /// The cue column's width in rhythm units; `None` is 30 percent of the region's width,
        /// rounded to whole units.
        #[serde(default)]
        cue_units: Option<u32>,
        /// The summary band's height in rhythm units; `None` is 20 percent of the region's
        /// height, rounded to whole units.
        #[serde(default)]
        summary_units: Option<u32>,
    },
    /// Staves of five lines, each starting on a rhythm row.
    MusicStaff {
        /// The gap between the five lines in mm.
        #[serde(default = "staff_gap")]
        gap_mm: f64,
        /// A new staff every this many rhythm units.
        #[serde(default = "three")]
        pitch_units: u32,
        /// Pairs the staves (treble and bass) with a brace line at the left.
        #[serde(default)]
        grand: bool,
    },
}

fn one() -> u32 {
    1
}

fn three() -> u32 {
    3
}

fn staff_gap() -> f64 {
    STAFF_GAP_DEFAULT_MM
}

/// The gap between a music staff's lines unless the region says otherwise: an 8 mm staff.
pub const STAFF_GAP_DEFAULT_MM: f64 = 2.0;
/// The most rhythm units any pattern count (hex height, staff pitch, Cornell sizes) may hold.
pub const PATTERN_UNITS_MAX: u32 = 99;

impl Pattern {
    /// Whether the pattern draws major lines (Hex and Music staff do not).
    pub fn has_major(&self) -> bool {
        !matches!(self, Pattern::Hex { .. } | Pattern::MusicStaff { .. })
    }

    /// Which choice in the inspector this is.
    pub fn kind(&self) -> PatternKind {
        match self {
            Pattern::Lines => PatternKind::Lines,
            Pattern::Dots => PatternKind::Dots,
            Pattern::Grid => PatternKind::Grid,
            Pattern::Isometric { .. } => PatternKind::Isometric,
            Pattern::Hex { .. } => PatternKind::Hex,
            Pattern::Cornell { .. } => PatternKind::Cornell,
            Pattern::MusicStaff { .. } => PatternKind::MusicStaff,
        }
    }

    /// Whether the pattern draws dots (they take the Dot grid role).
    pub fn is_dotted(&self) -> bool {
        matches!(
            self,
            Pattern::Dots
                | Pattern::Isometric {
                    style: IsoStyle::Dots
                }
        )
    }
}

/// The patterns a region can be, without their settings: the inspector's choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    /// [`Pattern::Lines`].
    Lines,
    /// [`Pattern::Dots`].
    Dots,
    /// [`Pattern::Grid`].
    Grid,
    /// [`Pattern::Isometric`].
    Isometric,
    /// [`Pattern::Hex`].
    Hex,
    /// [`Pattern::Cornell`].
    Cornell,
    /// [`Pattern::MusicStaff`].
    MusicStaff,
}

impl PatternKind {
    /// Every choice, in the order the inspector lists them.
    pub const ALL: [PatternKind; 7] = [
        PatternKind::Lines,
        PatternKind::Dots,
        PatternKind::Grid,
        PatternKind::Isometric,
        PatternKind::Hex,
        PatternKind::Cornell,
        PatternKind::MusicStaff,
    ];

    /// The pattern with its default settings.
    pub fn default_pattern(self) -> Pattern {
        match self {
            PatternKind::Lines => Pattern::Lines,
            PatternKind::Dots => Pattern::Dots,
            PatternKind::Grid => Pattern::Grid,
            PatternKind::Isometric => Pattern::Isometric {
                style: IsoStyle::Lines,
            },
            PatternKind::Hex => Pattern::Hex {
                orientation: HexOrientation::Flat,
                height_units: 1,
            },
            PatternKind::Cornell => Pattern::Cornell {
                cue_units: None,
                summary_units: None,
            },
            PatternKind::MusicStaff => Pattern::MusicStaff {
                gap_mm: STAFF_GAP_DEFAULT_MM,
                pitch_units: 3,
                grand: false,
            },
        }
    }
}
