//! Text outlines (§7.3): a text-heavy synthetic planner, its glyph outlines taken from a real
//! font with skrifa, and four ways to store them, so their sizes can be compared.

use crate::model::*;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const FONT_PATH: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

/// A profile's page size in mm (from its pixels and PPI).
pub struct ProfileGeom {
    pub id: &'static str,
    pub w_mm: f64,
    pub h_mm: f64,
}

pub const MANTA: ProfileGeom = ProfileGeom { id: "supernote.a5x2", w_mm: 1920.0 / 300.0 * 25.4, h_mm: 2560.0 / 300.0 * 25.4 };
pub const A5X: ProfileGeom = ProfileGeom { id: "supernote.a5x", w_mm: 1404.0 / 226.0 * 25.4, h_mm: 1872.0 / 226.0 * 25.4 };

// ---------- the planner ----------

/// A weekly spread with about 1500 non-space characters of text, laid out on the Manta page.
pub fn planner_document() -> Document {
    let mut doc = sample_document();
    doc.metadata.title = "Weekly spread".into();
    doc.profiles[1].profile_id = A5X.id.into();
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    let mut objects = Vec::new();
    let col_w = (MANTA.w_mm - 12.0) / 2.0;
    for (i, day) in days.iter().enumerate() {
        let x = 6.0 + (i % 2) as f64 * col_w;
        let y0 = 8.0 + (i / 2) as f64 * 52.0;
        let mut lines = vec![(format!("{day} {} November 2026", 10 + i), 11.0)];
        for h in 8..=18 {
            lines.push((format!("{h:02}:00  ________"), 7.0));
        }
        lines.push(("Priorities: deep work, email, review".into(), 7.0));
        lines.push(("Habits: water · walk · read · sleep".into(), 7.0));
        for (j, (text, pt)) in lines.into_iter().enumerate() {
            objects.push(Object::new(
                &format!("{day} line {j}"),
                t(x, y0 + j as f64 * 3.6, col_w - 4.0, 3.6),
                Style::Role(LineRole::Text),
                Shape::Text(TextBox {
                    content: text,
                    font_id: "dejavu-sans@2.37".into(),
                    font_size_pt: pt,
                    line_height: 1.2,
                    auto_height: true,
                }),
            ));
        }
    }
    doc.layers.push(Layer { name: "Labels".into(), visible: true, locked: false, objects, extra: Extra::new() });
    doc
}

pub fn text_objects(doc: &Document) -> Vec<&Object> {
    fn walk<'a>(o: &'a Object, out: &mut Vec<&'a Object>) {
        match &o.shape {
            Shape::Text(_) => out.push(o),
            Shape::Group { children } => children.iter().for_each(|c| walk(c, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for l in &doc.layers {
        for o in &l.objects {
            walk(o, &mut out);
        }
    }
    out
}

pub fn glyph_count(doc: &Document) -> usize {
    text_objects(doc)
        .iter()
        .map(|o| match &o.shape {
            Shape::Text(tb) => tb.content.chars().filter(|c| !c.is_whitespace()).count(),
            _ => 0,
        })
        .sum()
}

// ---------- path storage forms ----------

/// Form A: one path per glyph instance, as a list of tagged commands, in page mm.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Cmd {
    M(f32, f32),
    L(f32, f32),
    Q(f32, f32, f32, f32),
    C(f32, f32, f32, f32, f32, f32),
    Z,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NaiveText {
    pub object_id: Uuid,
    pub glyphs: Vec<Vec<Cmd>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NaiveFile {
    pub profile_id: String,
    pub texts: Vec<NaiveText>,
}

/// Form B: same per-instance paths, packed as two byte strings: verbs (1 byte each) and
/// coordinates (little-endian f32, page mm).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PackedText {
    pub object_id: Uuid,
    pub verbs: ByteBuf,
    pub coords: ByteBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PackedFile {
    pub profile_id: String,
    pub texts: Vec<PackedText>,
}

/// Form C: each distinct glyph stored once in half font units (i16, exact for TrueType), plus a
/// list of placements (glyph id, x, y in mm) per text object.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GlyphShape {
    pub verbs: ByteBuf,
    pub coords: ByteBuf, // i16 LE, half font units (TrueType implied points fall on .5)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GlyphTable {
    pub font_id: String,
    pub units_per_em: u16,
    pub glyphs: BTreeMap<u16, GlyphShape>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlacedText {
    pub object_id: Uuid,
    pub font_id: String,
    pub size_mm: f32,
    /// per glyph: u16 glyph id, f32 x mm, f32 y mm (baseline), little-endian
    pub placements: ByteBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TableFile {
    pub profile_id: String,
    /// None when the table lives in a shared `outlines/glyphs.cbor`
    pub tables: Option<Vec<GlyphTable>>,
    pub texts: Vec<PlacedText>,
}

// ---------- extraction ----------

const V_M: u8 = 0;
const V_L: u8 = 1;
const V_Q: u8 = 2;
const V_C: u8 = 3;
const V_Z: u8 = 4;

#[derive(Default)]
struct RawPen {
    verbs: Vec<u8>,
    pts: Vec<f32>, // font units
}

impl OutlinePen for RawPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.verbs.push(V_M);
        self.pts.extend([x, y]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.verbs.push(V_L);
        self.pts.extend([x, y]);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.verbs.push(V_Q);
        self.pts.extend([cx, cy, x, y]);
    }
    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.verbs.push(V_C);
        self.pts.extend([a, b, c, d, x, y]);
    }
    fn close(&mut self) {
        self.verbs.push(V_Z);
    }
}

pub struct Font<'a> {
    pub font: FontRef<'a>,
    pub upem: f32,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let font = FontRef::new(data).expect("font parse");
        let upem = font.metrics(Size::unscaled(), LocationRef::default()).units_per_em as f32;
        Font { font, upem }
    }
    fn gid(&self, c: char) -> GlyphId {
        self.font.charmap().map(c).unwrap_or(GlyphId::NOTDEF)
    }
    fn advance(&self, g: GlyphId) -> f32 {
        self.font.glyph_metrics(Size::unscaled(), LocationRef::default()).advance_width(g).unwrap_or(0.0)
    }
    fn raw(&self, g: GlyphId) -> RawPen {
        let mut pen = RawPen::default();
        if let Some(og) = self.font.outline_glyphs().get(g) {
            og.draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::default()), &mut pen).unwrap();
        }
        pen
    }
}

/// One placed glyph: which glyph, where its baseline origin sits (mm), at what size (mm per em).
struct Placed {
    gid: GlyphId,
    x: f32,
    y: f32,
    size_mm: f32,
}

/// Simple left-to-right layout (no shaping, no kerning: enough to size the storage).
/// Positions follow the Scale constraint; the font size is fixed physical (§5.4).
fn layout(font: &Font, o: &Object, p: &ProfileGeom) -> Vec<Placed> {
    let Shape::Text(tb) = &o.shape else { return vec![] };
    let sx = p.w_mm / MANTA.w_mm;
    let sy = p.h_mm / MANTA.h_mm;
    let size_mm = (tb.font_size_pt * 25.4 / 72.0) as f32;
    let mut x = (o.transform.x_mm * sx) as f32;
    let y = ((o.transform.y_mm + o.transform.h_mm) * sy) as f32;
    let mut out = Vec::new();
    for c in tb.content.chars() {
        let g = font.gid(c);
        if !c.is_whitespace() {
            out.push(Placed { gid: g, x, y, size_mm });
        }
        x += font.advance(g) / font.upem * size_mm;
    }
    out
}

fn to_page(pen: &RawPen, pl: &Placed, upem: f32) -> Vec<f32> {
    let s = pl.size_mm / upem;
    pen.pts
        .chunks(2)
        .flat_map(|p| [pl.x + p[0] * s, pl.y - p[1] * s])
        .collect()
}

fn naive_cmds(verbs: &[u8], pts: &[f32]) -> Vec<Cmd> {
    let mut i = 0;
    verbs
        .iter()
        .map(|v| {
            let n = match *v {
                V_M | V_L => 2,
                V_Q => 4,
                V_C => 6,
                _ => 0,
            };
            let p = &pts[i..i + n];
            i += n;
            match *v {
                V_M => Cmd::M(p[0], p[1]),
                V_L => Cmd::L(p[0], p[1]),
                V_Q => Cmd::Q(p[0], p[1], p[2], p[3]),
                V_C => Cmd::C(p[0], p[1], p[2], p[3], p[4], p[5]),
                _ => Cmd::Z,
            }
        })
        .collect()
}

pub struct AllForms {
    pub naive: NaiveFile,
    pub packed: PackedFile,
    pub table_inline: TableFile,
    pub table_shared: TableFile,
    pub shared_table: GlyphTable,
    pub instances: usize,
    pub distinct_glyphs: usize,
}

pub fn build_forms(font: &Font, doc: &Document, p: &ProfileGeom) -> AllForms {
    let mut naive = NaiveFile { profile_id: p.id.into(), texts: vec![] };
    let mut packed = PackedFile { profile_id: p.id.into(), texts: vec![] };
    let mut placed_texts = vec![];
    let mut table = GlyphTable { font_id: "dejavu-sans@2.37".into(), units_per_em: font.upem as u16, glyphs: BTreeMap::new() };
    let mut cache: BTreeMap<u16, RawPen> = BTreeMap::new();
    let mut instances = 0;

    for o in text_objects(doc) {
        let placed = layout(font, o, p);
        let mut nt = NaiveText { object_id: o.id, glyphs: vec![] };
        let (mut verbs, mut coords) = (Vec::new(), Vec::new());
        let mut placements = Vec::new();
        let mut size_mm = 0.0;
        for pl in &placed {
            instances += 1;
            let gid = pl.gid.to_u32() as u16;
            let raw = cache.entry(gid).or_insert_with(|| font.raw(pl.gid));
            let page = to_page(raw, pl, font.upem);
            nt.glyphs.push(naive_cmds(&raw.verbs, &page));
            verbs.extend_from_slice(&raw.verbs);
            for f in &page {
                coords.extend_from_slice(&f.to_le_bytes());
            }
            placements.extend_from_slice(&gid.to_le_bytes());
            placements.extend_from_slice(&pl.x.to_le_bytes());
            placements.extend_from_slice(&pl.y.to_le_bytes());
            size_mm = pl.size_mm;
            table.glyphs.entry(gid).or_insert_with(|| GlyphShape {
                verbs: ByteBuf::from(raw.verbs.clone()),
                coords: ByteBuf::from(raw.pts.iter().flat_map(|f| ((*f * 2.0) as i16).to_le_bytes()).collect::<Vec<u8>>()),
            });
        }
        naive.texts.push(nt);
        packed.texts.push(PackedText { object_id: o.id, verbs: ByteBuf::from(verbs), coords: ByteBuf::from(coords) });
        placed_texts.push(PlacedText {
            object_id: o.id,
            font_id: table.font_id.clone(),
            size_mm,
            placements: ByteBuf::from(placements),
        });
    }
    let distinct_glyphs = table.glyphs.len();
    AllForms {
        naive,
        packed,
        table_inline: TableFile { profile_id: p.id.into(), tables: Some(vec![table.clone()]), texts: placed_texts.clone() },
        table_shared: TableFile { profile_id: p.id.into(), tables: None, texts: placed_texts },
        shared_table: table,
        instances,
        distinct_glyphs,
    }
}

/// True when every glyph coordinate is a whole number of half font units, so i16 storage is exact.
pub fn table_is_exact(font: &Font, doc: &Document) -> bool {
    text_objects(doc).iter().flat_map(|o| layout(font, o, &MANTA)).all(|pl| {
        font.raw(pl.gid).pts.iter().all(|f| (f * 2.0).fract() == 0.0 && (f * 2.0).abs() < i16::MAX as f32)
    })
}
