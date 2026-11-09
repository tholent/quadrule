//! Throwaway text spike for quadrule (napkin §7, §16 spike 2). Not product code.
//!
//! Questions answered here:
//! - fontdb system font enumeration cost (T01–T02)
//! - cosmic-text paragraph layout in physical units (T03–T08)
//! - skrifa outline extraction placed where cosmic-text put each glyph (T09–T10)
//! - bundled Atkinson Hyperlegible Next and Noto Emoji (T11–T12)
//! - missing-font substitution by classification (T13)

use cosmic_text::fontdb;
use cosmic_text::skrifa::{
    self, GlyphId, MetadataProvider, Tag,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    raw::TableProvider,
};
use cosmic_text::{
    Align, Attrs, Buffer, Family, FeatureTag, FontFeatures, FontSystem, Metrics, Shaping, Weight,
    Wrap,
};
use kurbo::{Affine, BezPath, Point};
use std::path::PathBuf;

// ---------------------------------------------------------------- units

pub const MM_PER_IN: f64 = 25.4;
pub const PT_PER_IN: f64 = 72.0;

pub fn mm_to_px(mm: f64, ppi: f64) -> f64 {
    mm * ppi / MM_PER_IN
}
pub fn pt_to_mm(pt: f64) -> f64 {
    pt * MM_PER_IN / PT_PER_IN
}

// ---------------------------------------------------------------- bundled fonts

pub fn fonts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts")
}
pub const ATKINSON_FILE: &str = "AtkinsonHyperlegibleNext[wght].ttf";
pub const NOTO_EMOJI_FILE: &str = "NotoEmoji[wght].ttf";
pub const ATKINSON: &str = "Atkinson Hyperlegible Next";
pub const NOTO_EMOJI: &str = "Noto Emoji";

/// Loads a bundled font file into the font system's database.
/// Panics with a clear message if `fetch-fonts.sh` has not been run.
pub fn load_bundled(fs: &mut FontSystem, file: &str) {
    let path = fonts_dir().join(file);
    let data = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run spikes/text/fetch-fonts.sh first",
            path.display()
        )
    });
    fs.db_mut().load_font_data(data);
}

/// A font system with system fonts plus both bundled fonts.
pub fn font_system_with_bundled() -> FontSystem {
    let mut fs = FontSystem::new();
    load_bundled(&mut fs, ATKINSON_FILE);
    load_bundled(&mut fs, NOTO_EMOJI_FILE);
    fs
}

// ---------------------------------------------------------------- paragraph box

/// A paragraph text box described in physical units, as quadrule's model would store it.
#[derive(Clone, Debug)]
pub struct TextBoxSpec<'a> {
    pub text: &'a str,
    pub family: &'a str,
    pub weight: u16,
    pub size_pt: f64,
    /// Line height as a multiple of the font size.
    pub line_height: f64,
    /// Letter spacing in EM (cosmic-text's own unit).
    pub letter_spacing_em: f64,
    pub width_mm: f64,
    pub align: Align,
    /// Switch off kerning, ligatures and contextual alternates (used only to compare raw
    /// advances between cosmic-text and skrifa).
    pub plain_shaping: bool,
}

pub const SAMPLE: &str = "Meeting notes: agenda, decisions and action items. \
Each attendee writes in the margin; the ruled area below stays clear for handwriting. \
AVATAR To Wa Yo: office waffles keep labels legible on e-ink at every device resolution.";

/// 10 pt Atkinson, 1.4 line height, in an 80 mm wide box.
pub fn sample_spec(align: Align, letter_spacing_em: f64, weight: u16) -> TextBoxSpec<'static> {
    TextBoxSpec {
        text: SAMPLE,
        family: ATKINSON,
        weight,
        size_pt: 10.0,
        line_height: 1.4,
        letter_spacing_em,
        width_mm: 80.0,
        align,
        plain_shaping: false,
    }
}

/// Lays the box out at `px_per_mm` (1.0 = lay out directly in millimetres).
pub fn layout(fs: &mut FontSystem, spec: &TextBoxSpec, px_per_mm: f64) -> Buffer {
    let size = (pt_to_mm(spec.size_pt) * px_per_mm) as f32;
    let mut buf = Buffer::new(fs, Metrics::new(size, size * spec.line_height as f32));
    buf.set_wrap(Wrap::WordOrGlyph);
    buf.set_size(Some((spec.width_mm * px_per_mm) as f32), None);
    let mut attrs = Attrs::new()
        .family(Family::Name(spec.family))
        .weight(Weight(spec.weight));
    if spec.letter_spacing_em != 0.0 {
        attrs = attrs.letter_spacing(spec.letter_spacing_em as f32);
    }
    if spec.plain_shaping {
        let mut ff = FontFeatures::new();
        for t in [b"kern", b"liga", b"clig", b"calt"] {
            ff.disable(FeatureTag::new(t));
        }
        attrs = attrs.font_features(ff);
    }
    buf.set_text(spec.text, &attrs, Shaping::Advanced, Some(spec.align));
    buf.shape_until_scroll(fs, false);
    buf
}

/// The text of each laid-out line, so line breaks can be compared across scales.
pub fn line_breaks(buf: &Buffer) -> Vec<(usize, usize)> {
    buf.layout_runs()
        .map(|r| {
            let s = r.glyphs.iter().map(|g| g.start).min().unwrap_or(0);
            let e = r.glyphs.iter().map(|g| g.end).max().unwrap_or(0);
            (s, e)
        })
        .collect()
}

/// Total height of all lines (for overflow and vertical alignment, which cosmic-text
/// leaves to the caller).
pub fn total_height(buf: &Buffer) -> f32 {
    buf.layout_runs().map(|r| r.line_height).sum()
}

// ---------------------------------------------------------------- outlines

/// Collects skrifa's pen commands into a kurbo path, flipping font y-up into page y-down
/// and moving the glyph to its laid-out origin.
struct PathPen {
    path: BezPath,
    to_page: Affine,
}
impl PathPen {
    fn p(&self, x: f32, y: f32) -> Point {
        self.to_page * Point::new(x as f64, y as f64)
    }
}
impl OutlinePen for PathPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.path.move_to(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.path.line_to(p);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (c, p) = (self.p(cx, cy), self.p(x, y));
        self.path.quad_to(c, p);
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let (a, b, p) = (self.p(c0x, c0y), self.p(c1x, c1y), self.p(x, y));
        self.path.curve_to(a, b, p);
    }
    fn close(&mut self) {
        self.path.close_path();
    }
}

#[derive(Clone, Debug)]
pub struct GlyphOutline {
    pub font_id: fontdb::ID,
    pub glyph_id: u16,
    /// Pen position (baseline origin) in layout units.
    pub origin: Point,
    /// cosmic-text's advance for this glyph (layout units).
    pub layout_advance: f32,
    /// skrifa's advance for the same glyph, size and weight (layout units).
    pub skrifa_advance: f32,
    pub path: BezPath,
}

fn face_index(fs: &FontSystem, id: fontdb::ID) -> u32 {
    fs.db().face(id).map(|f| f.index).unwrap_or(0)
}

/// Every glyph in the buffer as a vector path, placed where cosmic-text laid it out.
pub fn outlines(fs: &mut FontSystem, buf: &Buffer) -> Vec<GlyphOutline> {
    let mut out = Vec::new();
    for run in buf.layout_runs() {
        for g in run.glyphs {
            let font = fs
                .get_font(g.font_id, g.font_weight)
                .expect("font for laid-out glyph");
            let index = face_index(fs, g.font_id);
            let fref = skrifa::FontRef::from_index(font.data(), index).expect("parse font");
            let loc = fref
                .axes()
                .location([(Tag::new(b"wght"), g.font_weight.0 as f32)]);
            let size = Size::new(g.font_size);
            let gid = GlyphId::new(g.glyph_id as u32);
            let ox = g.x + g.font_size * g.x_offset;
            let oy = run.line_y + g.y - g.font_size * g.y_offset;
            let mut pen = PathPen {
                path: BezPath::new(),
                to_page: Affine::new([1.0, 0.0, 0.0, -1.0, ox as f64, oy as f64]),
            };
            if let Some(og) = fref.outline_glyphs().get(gid) {
                og.draw(
                    DrawSettings::unhinted(size, LocationRef::from(&loc)),
                    &mut pen,
                )
                .expect("draw outline");
            }
            let skrifa_advance = fref
                .glyph_metrics(size, LocationRef::from(&loc))
                .advance_width(gid)
                .unwrap_or(0.0);
            out.push(GlyphOutline {
                font_id: g.font_id,
                glyph_id: g.glyph_id,
                origin: Point::new(ox as f64, oy as f64),
                layout_advance: g.w,
                skrifa_advance,
                path: pen.path,
            });
        }
    }
    out
}

/// One font unit at this glyph's size: the shaper (harfrust) positions glyphs in whole
/// font units, so cosmic-text advances can differ from skrifa's by up to half of this.
pub fn font_unit(fs: &mut FontSystem, id: fontdb::ID, weight: fontdb::Weight, size: f32) -> f32 {
    let upem = fs
        .get_font(id, weight)
        .map(|f| f.metrics().units_per_em)
        .unwrap_or(1000);
    size / upem as f32
}

pub fn family_of(fs: &FontSystem, id: fontdb::ID) -> String {
    fs.db()
        .face(id)
        .and_then(|f| f.families.first().map(|(n, _)| n.clone()))
        .unwrap_or_default()
}

// ---------------------------------------------------------------- substitution

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Serif,
    Sans,
    Mono,
    Script,
    Unknown,
}

/// What quadrule would store in a document for each font it uses, so a good substitute
/// can be found when the font is missing on another machine (napkin §7.3 :375).
/// Measured from the font itself at save time, because font metadata is often empty.
#[derive(Clone, Debug, PartialEq)]
pub struct FontSignature {
    pub family: String,
    pub class: Class,
    pub weight: u16,
    pub stretch: fontdb::Stretch,
    pub italic: bool,
    /// Average advance of a–z, in em.
    pub avg_width_em: f32,
    /// x-height in em (0 when the font does not say).
    pub x_height_em: f32,
}

fn os2_class(f: &skrifa::FontRef) -> Class {
    let Ok(os2) = f.os2() else {
        return Class::Unknown;
    };
    let class = (os2.s_family_class() >> 8) as u8;
    let panose = os2.panose_10();
    match class {
        1..=5 | 7 => Class::Serif,
        8 => Class::Sans,
        10 => Class::Script,
        _ => match (panose.first(), panose.get(1), panose.get(3)) {
            (_, _, Some(9)) => Class::Mono, // PANOSE proportion: monospaced
            (Some(3), _, _) => Class::Script,
            (Some(2), Some(11..=13), _) => Class::Sans,
            (Some(2), Some(2..=10), _) => Class::Serif,
            _ => Class::Unknown,
        },
    }
}

fn name_class(family: &str) -> Class {
    let f = family.to_lowercase();
    let has = |ws: &[&str]| ws.iter().any(|w| f.contains(w));
    if has(&["mono", "code", "courier", "console"]) {
        Class::Mono
    } else if has(&["sans", "gothic", "grotesk", "hyperlegible"]) {
        Class::Sans
    } else if has(&[
        "serif", "roman", "times", "garamond", "bookman", "didot", "century",
    ]) {
        Class::Serif
    } else if has(&["hand", "script", "caveat", "chancery"]) {
        Class::Script
    } else {
        Class::Unknown
    }
}

/// Measures a face. Order of trust: measured monospacing, then the OS/2 table, then
/// words in the family name.
pub fn signature(db: &fontdb::Database, id: fontdb::ID) -> Option<FontSignature> {
    let info = db.face(id)?;
    let family = info.families.first()?.0.clone();
    let (class, avg, xh) = db
        .with_face_data(id, |data, index| {
            let f = skrifa::FontRef::from_index(data, index).ok()?;
            let loc = f
                .axes()
                .location([(Tag::new(b"wght"), info.weight.0 as f32)]);
            let m = f.metrics(Size::unscaled(), LocationRef::from(&loc));
            let upem = m.units_per_em as f32;
            let gm = f.glyph_metrics(Size::unscaled(), LocationRef::from(&loc));
            let cmap = f.charmap();
            let adv = |c: char| cmap.map(c).and_then(|g| gm.advance_width(g));
            let lower: Vec<f32> = ('a'..='z').filter_map(adv).collect();
            let avg = if lower.is_empty() {
                0.0
            } else {
                lower.iter().sum::<f32>() / lower.len() as f32 / upem
            };
            let mono = info.monospaced
                || matches!((adv('i'), adv('M')), (Some(a), Some(b)) if (a - b).abs() < 0.5);
            let class = if mono {
                Class::Mono
            } else {
                match os2_class(&f) {
                    Class::Unknown => name_class(&family),
                    c => c,
                }
            };
            Some((class, avg, m.x_height.unwrap_or(0.0) / upem))
        })
        .flatten()?;
    Some(FontSignature {
        family,
        class,
        weight: info.weight.0,
        stretch: info.stretch,
        italic: info.style != fontdb::Style::Normal,
        avg_width_em: avg,
        x_height_em: xh,
    })
}

pub fn classify(db: &fontdb::Database, id: fontdb::ID) -> Class {
    signature(db, id).map(|s| s.class).unwrap_or(Class::Unknown)
}

/// Lower is closer. Weights chosen by hand for the spike, not tuned.
pub fn distance(want: &FontSignature, got: &FontSignature) -> u32 {
    let class = match (want.class, got.class) {
        (a, b) if a == b => 0,
        (_, Class::Unknown) | (Class::Unknown, _) => 400,
        _ => 1000,
    };
    let weight = (got.weight as i32 - want.weight as i32).unsigned_abs();
    let stretch =
        (got.stretch.to_number() as i32 - want.stretch.to_number() as i32).unsigned_abs() * 50;
    let style = if got.italic == want.italic { 0 } else { 300 };
    let width = ((got.avg_width_em - want.avg_width_em).abs() * 4000.0) as u32;
    let xh = if want.x_height_em > 0.0 && got.x_height_em > 0.0 {
        ((got.x_height_em - want.x_height_em).abs() * 2000.0) as u32
    } else {
        0
    };
    class + weight + stretch + style + width + xh
}

/// Picks the closest installed face: an exact family-name match wins (fontdb's own
/// CSS-style query does that part for free); otherwise the lowest `distance`.
/// `exclude` hides one installed family, to test "what if this font were missing".
pub fn substitute(
    db: &fontdb::Database,
    want: &FontSignature,
    exclude: Option<&str>,
) -> Option<(fontdb::ID, u32)> {
    let exact = db.query(&fontdb::Query {
        families: &[fontdb::Family::Name(&want.family)],
        weight: fontdb::Weight(want.weight),
        stretch: want.stretch,
        style: if want.italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        },
    });
    if let Some(id) = exact.filter(|_| exclude != Some(want.family.as_str())) {
        return Some((id, 0));
    }
    db.faces()
        .filter(|f| exclude.is_none_or(|x| f.families.iter().all(|(n, _)| n != x)))
        .filter_map(|f| signature(db, f.id).map(|s| (f.id, distance(want, &s))))
        .min_by_key(|(_, d)| *d)
}

/// Finds the face of a family at a weight, for building test signatures.
pub fn find_face(db: &fontdb::Database, family: &str, weight: u16) -> Option<fontdb::ID> {
    db.query(&fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        weight: fontdb::Weight(weight),
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    })
}
