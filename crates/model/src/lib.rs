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
        let line = match &self.kind {
            ObjectKind::Line(l) => Some(l.direction),
            _ => None,
        };
        if self.rotation() == 0.0 && line.is_none() {
            self.frame
        } else {
            self.frame.turned_bounds(self.rotation(), line)
        }
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
/// first. A group uses the object's `frame`, `constraints`, `visible`, `locked`, `rotation_deg`,
/// `opacity_pct` and `name`; its `style`, `fill`, `dash` and `cap` are ignored. The children's
/// frames are in page millimetres in the group's unturned space, so grouping moves nothing. The
/// group's box is refitted to the children's outer box after an edit inside it.
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
