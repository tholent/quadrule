//! Text (napkin §7) as ADR-002 decides it: cosmic-text alone, with fontdb and skrifa reached
//! through its re-exports (item 1); paragraphs laid out once in millimetres (item 2); each
//! letter drawn by skrifa as a plain (unhinted) outline at cosmic-text's position, so kerning
//! carries into the export (items 3–4).
//!
//! The result is a [`ProfileOutlines`]: each distinct letter shape once, plus each box's letter
//! positions (ADR-001 item 1). Live text and saved text are both drawn from that one structure.
//!
//! Not built yet: system fonts and fallback to them (ADR-002 item 6), substitution by font
//! signature (item 7), re-scan on focus (item 8), hinting (item 4's flag), paragraph spacing,
//! vertical alignment and the overflow signal (item 3).

#![forbid(unsafe_code)]

use cosmic_text::fontdb;
use cosmic_text::skrifa::{
    self, GlyphId, MetadataProvider, Tag,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    string::StringId,
};
use cosmic_text::{Align, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight, Wrap};
use quadrule_model::outlines::{
    FaceKey, GlyphShape, GlyphTable, PlacedGlyph, ProfileOutlines, TextOutlines,
};
use quadrule_model::{Extra, TextAlign, TextBox, Uuid, pt_to_mm};
use std::collections::BTreeMap;

/// A font that ships inside quadrule (napkin §7.2).
#[derive(Debug)]
pub struct BundledFont {
    /// Stable id, used in file references.
    pub id: &'static str,
    /// Family name as the font reports it.
    pub family: &'static str,
    /// File name under `crates/text/fonts/`.
    pub file: &'static str,
    /// The font file.
    pub data: &'static [u8],
    /// sha256 of `data`, checked by a test.
    pub sha256: &'static str,
}

/// The bundled fonts. Rung 1 bundles Atkinson Hyperlegible Next only (rung 1 record,
/// "Fonts"); the rest of napkin §7.2's list arrives with the rungs that use them.
pub const BUNDLED: &[BundledFont] = &[BundledFont {
    id: "atkinson-hyperlegible-next",
    family: "Atkinson Hyperlegible Next",
    file: "AtkinsonHyperlegibleNext[wght].ttf",
    data: include_bytes!("../fonts/AtkinsonHyperlegibleNext[wght].ttf"),
    sha256: "5a455d1cfa099b601ab70751bb9673e8fe1854dc4500c80e1a220d0d75e31745",
}];

impl BundledFont {
    /// The bundled font of a family, if there is one.
    pub fn for_family(family: &str) -> Option<&'static BundledFont> {
        BUNDLED.iter().find(|f| f.family == family)
    }

    /// The font's own version string (name ID 5), e.g. `Version 2.001`.
    pub fn version(&self) -> Option<String> {
        let f = skrifa::FontRef::new(self.data).ok()?;
        let s = f
            .localized_strings(StringId::VERSION_STRING)
            .english_or_first()?;
        Some(s.chars().collect())
    }
}

/// The fonts text can be laid out with.
pub struct FontRegistry {
    fs: FontSystem,
}

impl FontRegistry {
    /// Bundled fonts only, no system fonts: the same on every machine. Used by tests and by the
    /// rung 1 slice.
    pub fn bundled_only() -> Self {
        let mut db = fontdb::Database::new();
        for f in BUNDLED {
            db.load_font_data(f.data.to_vec());
        }
        FontRegistry {
            fs: FontSystem::new_with_locale_and_db("en-US".into(), db),
        }
    }

    /// True when some loaded face belongs to this family.
    pub fn has_family(&self, family: &str) -> bool {
        self.fs
            .db()
            .faces()
            .any(|f| f.families.iter().any(|(n, _)| n == family))
    }
}

/// One text box to lay out on a profile.
#[derive(Debug, Clone, Copy)]
pub struct TextRequest<'a> {
    /// The text object's id.
    pub object_id: Uuid,
    /// Its text and style.
    pub text: &'a TextBox,
    /// The box width on this profile, in mm.
    pub width_mm: f64,
}

/// Why text could not be laid out.
#[derive(Debug, Clone, PartialEq)]
pub enum TextError {
    /// The family is not loaded. (The missing-font flow, napkin §7.3, is a later rung.)
    MissingFamily(String),
    /// A glyph's outline could not be read from its font.
    Outline(String),
}

impl std::fmt::Display for TextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextError::MissingFamily(n) => write!(f, "font family not available: {n}"),
            TextError::Outline(e) => write!(f, "glyph outline: {e}"),
        }
    }
}

impl std::error::Error for TextError {}

/// Laid-out text for one profile, plus what to report about it.
#[derive(Debug, Clone, PartialEq)]
pub struct Laid {
    /// Shapes and positions, ready to draw or save.
    pub outlines: ProfileOutlines,
    /// Letters drawn as the font's "missing" box (glyph 0).
    pub missing_glyphs: usize,
    /// Shape coordinates that were not a whole number of half font units and were rounded to
    /// the nearest half (at most 0.25 font units; happens at variable-font weights between
    /// masters). Live and saved text both use the rounded shape.
    pub rounded_coords: usize,
}

/// Lays out every request for one profile and collects their outlines.
pub fn lay_out(
    reg: &mut FontRegistry,
    profile_id: &str,
    requests: &[TextRequest],
) -> Result<Laid, TextError> {
    let mut tables: Vec<GlyphTable> = Vec::new();
    let mut texts = Vec::with_capacity(requests.len());
    let mut missing = 0;
    let mut rounded = 0;
    for req in requests {
        if !reg.has_family(&req.text.family) {
            return Err(TextError::MissingFamily(req.text.family.clone()));
        }
        let size_mm = pt_to_mm(req.text.size_pt) as f32;
        let buf = paragraph(&mut reg.fs, req.text, size_mm, req.width_mm as f32);
        let mut glyphs = Vec::new();
        let mut height_mm = 0.0f32;
        for run in buf.layout_runs() {
            height_mm += run.line_height;
            for g in run.glyphs {
                if g.glyph_id == 0 {
                    missing += 1;
                }
                let family = family_of(&reg.fs, g.font_id);
                let key = FaceKey {
                    family,
                    weight: g.font_weight.0,
                };
                let table = match tables.iter().position(|t| t.face == key) {
                    Some(i) => i,
                    None => {
                        let upem = units_per_em(&mut reg.fs, g.font_id, g.font_weight)?;
                        tables.push(GlyphTable {
                            face: key,
                            units_per_em: upem,
                            glyphs: BTreeMap::new(),
                            extra: Extra::new(),
                        });
                        tables.len() - 1
                    }
                };
                let shape = match tables[table].glyphs.entry(g.glyph_id) {
                    std::collections::btree_map::Entry::Occupied(e) => e.into_mut(),
                    std::collections::btree_map::Entry::Vacant(e) => {
                        let (shape, r) =
                            glyph_shape(&mut reg.fs, g.font_id, g.font_weight, g.glyph_id)?;
                        rounded += r;
                        e.insert(shape)
                    }
                };
                if shape.verbs.is_empty() {
                    continue; // a space: nothing to draw
                }
                let x = g.x + g.font_size * g.x_offset;
                let y = run.line_y + g.y - g.font_size * g.y_offset;
                glyphs.push(PlacedGlyph(table as u16, g.glyph_id, x, y));
            }
        }
        texts.push(TextOutlines {
            object_id: req.object_id,
            size_mm,
            height_mm,
            glyphs,
            extra: Extra::new(),
        });
    }
    // Drop the empty shapes of spaces: nothing refers to them.
    for t in &mut tables {
        t.glyphs.retain(|_, s| !s.verbs.is_empty());
    }
    let outlines = ProfileOutlines {
        profile_id: profile_id.into(),
        tables,
        texts,
        extra: Extra::new(),
    };
    Ok(Laid {
        outlines,
        missing_glyphs: missing,
        rounded_coords: rounded,
    })
}

/// Lays one box out with cosmic-text, in millimetres.
fn paragraph(fs: &mut FontSystem, tb: &TextBox, size_mm: f32, width_mm: f32) -> Buffer {
    let mut buf = Buffer::new(fs, Metrics::new(size_mm, size_mm * tb.line_height as f32));
    buf.set_wrap(Wrap::WordOrGlyph);
    buf.set_size(Some(width_mm), None);
    let mut attrs = Attrs::new()
        .family(Family::Name(&tb.family))
        .weight(Weight(tb.weight));
    if tb.letter_spacing_em != 0.0 {
        attrs = attrs.letter_spacing(tb.letter_spacing_em as f32);
    }
    let align = match tb.align {
        TextAlign::Left => Align::Left,
        TextAlign::Center => Align::Center,
        TextAlign::Right => Align::Right,
        TextAlign::Justify => Align::Justified,
    };
    buf.set_text(&tb.content, &attrs, Shaping::Advanced, Some(align));
    buf.shape_until_scroll(fs, false);
    buf
}

fn family_of(fs: &FontSystem, id: fontdb::ID) -> String {
    fs.db()
        .face(id)
        .and_then(|f| f.families.first().map(|(n, _)| n.clone()))
        .unwrap_or_default()
}

fn font_ref<'a>(data: &'a [u8], index: u32) -> Result<skrifa::FontRef<'a>, TextError> {
    skrifa::FontRef::from_index(data, index).map_err(|e| TextError::Outline(e.to_string()))
}

fn units_per_em(fs: &mut FontSystem, id: fontdb::ID, weight: Weight) -> Result<u16, TextError> {
    let index = fs.db().face(id).map(|f| f.index).unwrap_or(0);
    let font = fs
        .get_font(id, weight)
        .ok_or_else(|| TextError::Outline("font not loadable".into()))?;
    let f = font_ref(font.data(), index)?;
    Ok(f.metrics(Size::unscaled(), LocationRef::default())
        .units_per_em)
}

/// Draws one glyph in font units at the face's weight and stores it in half font units.
/// Returns the shape and how many coordinates had to be rounded.
fn glyph_shape(
    fs: &mut FontSystem,
    id: fontdb::ID,
    weight: Weight,
    glyph: u16,
) -> Result<(GlyphShape, usize), TextError> {
    let index = fs.db().face(id).map(|f| f.index).unwrap_or(0);
    let font = fs
        .get_font(id, weight)
        .ok_or_else(|| TextError::Outline("font not loadable".into()))?;
    let f = font_ref(font.data(), index)?;
    let loc = f.axes().location([(Tag::new(b"wght"), weight.0 as f32)]);
    let mut pen = HalfUnitPen::default();
    if let Some(og) = f.outline_glyphs().get(GlyphId::new(glyph as u32)) {
        og.draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::from(&loc)),
            &mut pen,
        )
        .map_err(|e| TextError::Outline(e.to_string()))?;
    }
    Ok((pen.shape, pen.rounded))
}

/// Collects skrifa's pen commands as a [`GlyphShape`] in half font units.
#[derive(Default)]
struct HalfUnitPen {
    shape: GlyphShape,
    rounded: usize,
}

impl HalfUnitPen {
    fn push(&mut self, verb: char, pts: &[f32]) {
        self.shape.verbs.push(verb);
        for &p in pts {
            let half = p * 2.0;
            let r = half.round();
            if r != half {
                self.rounded += 1;
            }
            self.shape
                .coords
                .push(r.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
        }
    }
}

impl OutlinePen for HalfUnitPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.push('M', &[x, y]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.push('L', &[x, y]);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.push('Q', &[cx, cy, x, y]);
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        self.push('C', &[c0x, c0y, c1x, c1y, x, y]);
    }
    fn close(&mut self) {
        self.push('Z', &[]);
    }
}
