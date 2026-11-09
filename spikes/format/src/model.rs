//! A minimal subset of the napkin's document model (§5.1), at format version 0.2.
//! `extra` maps keep fields this build does not know, so they survive a save (§12.2).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Fields from a newer minor version that this build does not understand.
pub type Extra = BTreeMap<String, ciborium::Value>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Document {
    pub metadata: Metadata,
    pub document_type: DocumentType,
    pub primary_profile_id: String,
    pub profiles: Vec<ProfileContext>,
    pub units: Unit,
    pub page: PageSettings,
    pub rhythm: Rhythm,
    pub layers: Vec<Layer>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub license: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created: String,
    pub modified: String,
    pub app_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum DocumentType {
    Template,
    Wallpaper,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px,
    Mm,
    Pt,
    In,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileContext {
    pub profile_id: String,
    pub mode: ProfileMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ProfileMode {
    Linked,
    Forked,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PageSettings {
    pub background_gray: u8,
    /// top, right, bottom, left in mm
    pub margins_mm: [f64; 4],
    pub landscape: bool,
}

/// Version 0.2 rhythm. Version 0.1 had a single `unit_mm` (see `migrate`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Rhythm {
    pub kind: RhythmKind,
    pub h_unit_mm: f64,
    pub v_unit_mm: f64,
    pub subdivisions: Option<u8>,
    pub major_every: Option<u8>,
    pub pixel_locked: bool,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RhythmKind {
    Rectangular,
    Isometric,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub objects: Vec<Object>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Object {
    pub id: Uuid,
    pub name: String,
    pub transform: Transform,
    pub constraints: Constraints,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub style: Style,
    pub overrides: BTreeMap<String, PropertyPatch>,
    pub shape: Shape,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Transform {
    pub x_mm: f64,
    pub y_mm: f64,
    pub w_mm: f64,
    pub h_mm: f64,
    pub rotation_deg: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum HConstraint {
    Left,
    Right,
    LeftRight,
    Center,
    Scale,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum VConstraint {
    Top,
    Bottom,
    TopBottom,
    Center,
    Scale,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Constraints {
    pub h: HConstraint,
    pub v: VConstraint,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Style {
    Role(LineRole),
    Custom { stroke_mm: f64, gray: u8, fill_gray: Option<u8> },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LineRole {
    Whisper,
    Guide,
    Structure,
    Emphasis,
    Text,
}

/// A per-profile override: only the fields that differ.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PropertyPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size_pt: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Shape {
    Line { x2_mm: f64, y2_mm: f64 },
    Rect { corner_radius_mm: f64 },
    Text(TextBox),
    PatternRegion(Pattern),
    Group { children: Vec<Object> },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextBox {
    pub content: String,
    pub font_id: String,
    pub font_size_pt: f64,
    pub line_height: f64,
    pub auto_height: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Pattern {
    pub kind: PatternKind,
    /// None = bound to the document rhythm
    pub spacing_mm: Option<f64>,
    pub major_every: Option<u8>,
    pub major_role: Option<LineRole>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum PatternKind {
    Lines,
    Dots,
    Square,
    Isometric,
    Hex,
    Cornell,
    MusicStaff,
}

impl Object {
    pub fn new(name: &str, t: Transform, style: Style, shape: Shape) -> Self {
        Object {
            id: Uuid::new_v4(),
            name: name.into(),
            transform: t,
            constraints: Constraints { h: HConstraint::Scale, v: VConstraint::Scale },
            visible: true,
            locked: false,
            opacity: 1.0,
            style,
            overrides: BTreeMap::new(),
            shape,
            extra: Extra::new(),
        }
    }
}

pub fn t(x: f64, y: f64, w: f64, h: f64) -> Transform {
    Transform { x_mm: x, y_mm: y, w_mm: w, h_mm: h, rotation_deg: 0.0 }
}

/// A small but representative document: one of each object kind, a group, an override.
pub fn sample_document() -> Document {
    let mut header = Object::new(
        "Title",
        t(10.0, 8.0, 120.0, 10.0),
        Style::Role(LineRole::Text),
        Shape::Text(TextBox {
            content: "Isometric 5 mm".into(),
            font_id: "atkinson-hyperlegible-next@2.0".into(),
            font_size_pt: 14.0,
            line_height: 1.2,
            auto_height: true,
        }),
    );
    header.overrides.insert(
        "supernote.a5x".into(),
        PropertyPatch { font_size_pt: Some(12.0), ..Default::default() },
    );
    let rule = Object::new(
        "Header rule",
        t(10.0, 20.0, 0.0, 0.0),
        Style::Role(LineRole::Structure),
        Shape::Line { x2_mm: 152.0, y2_mm: 20.0 },
    );
    let grid = Object::new(
        "Iso grid",
        t(10.0, 25.0, 142.0, 170.0),
        Style::Role(LineRole::Whisper),
        Shape::PatternRegion(Pattern {
            kind: PatternKind::Isometric,
            spacing_mm: None,
            major_every: Some(5),
            major_role: Some(LineRole::Guide),
        }),
    );
    let frame = Object::new(
        "Frame",
        t(8.0, 6.0, 146.0, 192.0),
        Style::Custom { stroke_mm: 0.17, gray: 0x80, fill_gray: None },
        Shape::Rect { corner_radius_mm: 2.0 },
    );
    let group = Object::new(
        "Header group",
        t(10.0, 8.0, 142.0, 12.0),
        Style::Role(LineRole::Guide),
        Shape::Group { children: vec![header, rule] },
    );
    Document {
        metadata: Metadata {
            title: "Isometric 5mm".into(),
            author: "spike".into(),
            license: "CC-BY-4.0".into(),
            description: "format spike sample".into(),
            tags: vec!["grid".into(), "isometric".into()],
            created: "2026-11-09T00:00:00Z".into(),
            modified: "2026-11-09T00:00:00Z".into(),
            app_version: "0.0.0-spike".into(),
        },
        document_type: DocumentType::Template,
        primary_profile_id: "supernote.a5x2".into(),
        profiles: vec![
            ProfileContext { profile_id: "supernote.a5x2".into(), mode: ProfileMode::Linked },
            ProfileContext { profile_id: "supernote.a6x2".into(), mode: ProfileMode::Linked },
        ],
        units: Unit::Mm,
        page: PageSettings { background_gray: 255, margins_mm: [6.0, 6.0, 6.0, 6.0], landscape: false },
        rhythm: Rhythm {
            kind: RhythmKind::Isometric,
            h_unit_mm: 5.0,
            v_unit_mm: 5.0,
            subdivisions: None,
            major_every: Some(5),
            pixel_locked: true,
            extra: Extra::new(),
        },
        layers: vec![
            Layer { name: "Grid".into(), visible: true, locked: false, objects: vec![grid, frame], extra: Extra::new() },
            Layer { name: "Header".into(), visible: true, locked: false, objects: vec![group], extra: Extra::new() },
        ],
        extra: Extra::new(),
    }
}
