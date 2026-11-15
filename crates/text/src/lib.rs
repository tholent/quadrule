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

//! Text (napkin §7) as ADR-002 decides it: cosmic-text alone, with fontdb and skrifa reached
//! through its re-exports (item 1); paragraphs laid out once in millimetres (item 2); each
//! letter drawn by skrifa as a plain (unhinted) outline at cosmic-text's position, so kerning
//! carries into the export (items 3–4).
//!
//! The result is a [`ProfileOutlines`]: each distinct letter shape once, plus each box's letter
//! positions (ADR-001 item 1). Live text and saved text are both drawn from that one structure.
//!
//! Rung 11a added the bundled set with italics, optical size, installed fonts with the re-scan on
//! focus (item 8), and a fallback that uses bundled fonts only ([`registry`]).
//!
//! Not built yet: fallback to any installed font with its warning (ADR-002 item 6, rung 12),
//! substitution by font signature (item 7), hinting (item 4's flag), paragraph spacing, vertical
//! alignment and the overflow signal (item 3, rung 11b).

#![forbid(unsafe_code)]

use cosmic_text::fontdb;
use cosmic_text::skrifa::{
    self, GlyphId, MetadataProvider, Tag,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    string::StringId,
};
use cosmic_text::{Align, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight, Wrap};
use std::collections::HashMap;

pub mod edit;
pub mod opsz;
pub mod registry;
use quadrule_model::outlines::{
    FaceKey, GlyphShape, GlyphTable, PlacedGlyph, ProfileOutlines, TextOutlines,
};
use quadrule_model::{Extra, TextAlign, TextBox, Uuid, VerticalAlign, pt_to_mm};
pub use registry::{
    FALLBACK_ORDER, FamilyInfo, Finished, FontRegistry, PendingScan, Scan, Weights, scans_started,
};
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
    /// True for the family's italic file.
    pub italic: bool,
    /// The file name of the family's SIL Open Font License text under `crates/text/fonts/`, which
    /// the packages ship (`packaging/packager.json`); a test checks both.
    pub ofl: &'static str,
}

/// One bundled font file: the variable files of the record's Defaulted "Font files".
macro_rules! font {
    ($id:literal, $family:literal, $file:literal, $sha:literal, $italic:literal, $ofl:literal) => {
        BundledFont {
            id: $id,
            family: $family,
            file: $file,
            data: include_bytes!(concat!("../fonts/", $file)),
            sha256: $sha,
            italic: $italic,
            ofl: $ofl,
        }
    };
}

/// The bundled fonts (rung 11a, bbq record Q2): Atkinson Hyperlegible Next and Mono, Inter,
/// Literata, Caveat and Noto Emoji, each with its italic where one exists, as the families' variable
/// files from github.com/google/fonts. The first is rung 1's file, unchanged.
pub const BUNDLED: &[BundledFont] = &[
    font!(
        "atkinson-hyperlegible-next",
        "Atkinson Hyperlegible Next",
        "AtkinsonHyperlegibleNext[wght].ttf",
        "5a455d1cfa099b601ab70751bb9673e8fe1854dc4500c80e1a220d0d75e31745",
        false,
        "OFL-Atkinson.txt"
    ),
    font!(
        "atkinson-hyperlegible-next-italic",
        "Atkinson Hyperlegible Next",
        "AtkinsonHyperlegibleNext-Italic[wght].ttf",
        "ce9cffed32742ad2d9238c561a93220385e5934cdc02b8eb4097a50efa957dc6",
        true,
        "OFL-Atkinson.txt"
    ),
    font!(
        "atkinson-hyperlegible-mono",
        "Atkinson Hyperlegible Mono",
        "AtkinsonHyperlegibleMono[wght].ttf",
        "5ce8b1698d1ded7dff2178c1a3ad159470085a58ea239e8b2cb88f4fb4a6f646",
        false,
        "OFL-AtkinsonMono.txt"
    ),
    font!(
        "atkinson-hyperlegible-mono-italic",
        "Atkinson Hyperlegible Mono",
        "AtkinsonHyperlegibleMono-Italic[wght].ttf",
        "e18f523877530e4abe229df6a36d394326254da488a7f5abb9b7b5ea8f780cd4",
        true,
        "OFL-AtkinsonMono.txt"
    ),
    font!(
        "inter",
        "Inter",
        "Inter[opsz,wght].ttf",
        "29160a80ff49ddcab2c97711247e08b1fab27a484a329ce8b813d820dc559031",
        false,
        "OFL-Inter.txt"
    ),
    font!(
        "inter-italic",
        "Inter",
        "Inter-Italic[opsz,wght].ttf",
        "acd98e64795781b2058f07b18475e0ecee2a0fe2b42a49e2f9e37d0d6bf66ce6",
        true,
        "OFL-Inter.txt"
    ),
    font!(
        "literata",
        "Literata",
        "Literata[opsz,wght].ttf",
        "b41138c9373112f32abb589cc22e8674b06ed4048b0c513be922bdd26f274440",
        false,
        "OFL-Literata.txt"
    ),
    font!(
        "literata-italic",
        "Literata",
        "Literata-Italic[opsz,wght].ttf",
        "d483dfaeba9cbf4ce71d32a52ee65df82f7e35b15fff8d1011cdb242d1fcd465",
        true,
        "OFL-Literata.txt"
    ),
    font!(
        "caveat",
        "Caveat",
        "Caveat[wght].ttf",
        "0bdb6b660482d31531b3945849fba5916b3ef8695da7024a9e6b9ee3c4157988",
        false,
        "OFL-Caveat.txt"
    ),
    font!(
        "noto-emoji",
        "Noto Emoji",
        "NotoEmoji[wght].ttf",
        "de6c18832938afc99caf132b39d6a30a19bac7f2e812e28db2535b4608d27551",
        false,
        "OFL-NotoEmoji.txt"
    ),
];

impl BundledFont {
    /// The bundled upright font of a family, if there is one.
    pub fn for_family(family: &str) -> Option<&'static BundledFont> {
        BUNDLED.iter().find(|f| f.family == family && !f.italic)
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

/// One text box to lay out on a profile.
#[derive(Debug, Clone, Copy)]
pub struct TextRequest<'a> {
    /// The text object's id.
    pub object_id: Uuid,
    /// Its text and style.
    pub text: &'a TextBox,
    /// The box width on this profile, in mm.
    pub width_mm: f64,
    /// The box height on this profile, in mm, when the box has a height of its own (a Fixed box,
    /// rung 11b): lines that do not fully fit are left out. `None` for a box as tall as its text.
    pub height_mm: Option<f64>,
    /// The space after each paragraph but the last, in mm (rung 11b).
    pub paragraph_mm: f64,
    /// The rhythm the baselines stand on, when the box asks for it (`TextBox::baselines`).
    pub grid: Option<BaselineGrid>,
}

/// The rhythm lines of a device, down the page, as a box whose top is at `top_px` sees them (rung
/// 11b): a line `k` is where the rhythm puts it, rounded to a pixel as a ruled pattern region does,
/// so a baseline on it lands on the very pixel row the region's line does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaselineGrid {
    /// The box's top edge on the device, in whole pixels (where the text is drawn from).
    pub top_px: f64,
    /// Device pixels to the millimetre.
    pub px_per_mm: f64,
    /// The rhythm's first horizontal line, in pixels from the page's top.
    pub origin_px: f64,
    /// One rhythm step down, in pixels.
    pub unit_px: f64,
}

impl BaselineGrid {
    /// One rhythm step down, in mm.
    pub fn step_mm(&self) -> f64 {
        self.unit_px / self.px_per_mm
    }

    /// Rhythm line `k` on the page, in pixels (whole).
    pub fn line_px(&self, k: i64) -> f64 {
        (self.origin_px + k as f64 * self.unit_px).round()
    }

    /// Rhythm line `k`, in mm below the box's top.
    pub fn line_mm(&self, k: i64) -> f64 {
        (self.line_px(k) - self.top_px) / self.px_per_mm
    }

    /// The first rhythm line at least `min_mm` below the box's top: its number.
    pub fn first_at_or_below(&self, min_mm: f64) -> i64 {
        const EPS: f64 = 1e-6;
        let want = self.top_px + min_mm * self.px_per_mm;
        let mut k = ((want - self.origin_px) / self.unit_px).floor() as i64 - 1;
        while self.line_mm(k) < min_mm - EPS {
            k += 1;
        }
        k
    }
}

impl<'a> TextRequest<'a> {
    /// A box as tall as its text.
    pub fn new(object_id: Uuid, text: &'a TextBox, width_mm: f64) -> Self {
        TextRequest {
            object_id,
            text,
            width_mm,
            height_mm: None,
            paragraph_mm: 0.0,
            grid: None,
        }
    }
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

/// A font face found on this computer, for the app's own controls (not the page: page text
/// stays on bundled fonts until ADR-002's system-font flow is built).
#[derive(Clone, Debug, PartialEq)]
pub struct SystemFace {
    /// The family the system reports.
    pub family: String,
    /// The font file's bytes.
    pub data: Vec<u8>,
    /// The face's index in the file (font collections hold several).
    pub index: u32,
}

/// Finds an installed face of `family` closest to `weight` (400 regular, 300 light, 700 bold)
/// and reads its bytes. `None` when the family is not installed.
pub fn system_face(family: &str, weight: u16) -> Option<SystemFace> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let id = db.query(&fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        weight: fontdb::Weight(weight),
        ..Default::default()
    })?;
    let face = db.face(id)?;
    // fontdb matches a family case-insensitively and never falls back to another family here.
    let name = face.families.first().map(|(n, _)| n.clone())?;
    db.with_face_data(id, |data, index| SystemFace {
        family: name,
        data: data.to_vec(),
        index,
    })
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
    // Installed families first, so the layout store is rebuilt at most once for the page.
    reg.ensure_open(requests.iter().map(|r| r.text.family.as_str()));
    // Family, italic flag and optical size of each face met, worked out once.
    let mut identities: HashMap<fontdb::ID, (String, bool, u16)> = HashMap::new();
    for req in requests {
        let tb = req.text;
        let resolved = reg
            .resolve(&tb.family, tb.weight, tb.style, tb.size_pt)
            .ok_or_else(|| TextError::MissingFamily(tb.family.clone()))?;
        let prep = prepare(reg, req, resolved)?;
        let buf = paragraph(
            &mut reg.fs,
            tb,
            &prep.resolved,
            prep.size_mm,
            prep.line_mm,
            req.width_mm as f32,
        );
        let (_, arranged) = arrange_buffer(&buf, req, &prep);
        let mut glyphs = Vec::new();
        for (run, line) in buf.layout_runs().zip(&arranged.lines) {
            if !line.visible {
                continue;
            }
            for g in run.glyphs {
                if g.glyph_id == 0 {
                    missing += 1;
                }
                let (family, italic, opsz) = identities
                    .entry(g.font_id)
                    .or_insert_with(|| reg.face_identity(g.font_id))
                    .clone();
                let key = FaceKey {
                    family,
                    weight: g.font_weight.0,
                    italic,
                    opsz,
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
                let y = run.line_y + line.shift + g.y - g.font_size * g.y_offset;
                glyphs.push(PlacedGlyph(table as u16, g.glyph_id, x, y));
            }
        }
        let height_mm = arranged.height_mm;
        let lines_hidden = arranged.hidden as u32;
        texts.push(TextOutlines {
            object_id: req.object_id,
            size_mm: prep.size_mm,
            height_mm,
            lines_hidden,
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

/// What a box's layout needs worked out before its text is shaped (rung 11b).
pub(crate) struct Prepared {
    resolved: registry::Resolved,
    size_mm: f32,
    /// The line height in mm: the size times the box's line height, or on the rhythm a whole number
    /// of its steps.
    line_mm: f32,
    grid: Option<BaselineGrid>,
    /// A capital's height in mm, for the first baseline on the rhythm (0 when there is none).
    cap_mm: f32,
}

fn prepare(
    reg: &mut FontRegistry,
    req: &TextRequest,
    resolved: registry::Resolved,
) -> Result<Prepared, TextError> {
    let tb = req.text;
    let size_mm = pt_to_mm(tb.size_pt) as f32;
    // Baselines on the rhythm: the line height is a whole number of steps, at least one.
    let grid = req.grid.filter(|_| tb.baselines);
    let line_mm = match grid {
        Some(g) => {
            let step = g.step_mm() as f32;
            (size_mm * tb.line_height as f32 / step).round().max(1.0) * step
        }
        None => size_mm * tb.line_height as f32,
    };
    let cap_mm = if grid.is_some() {
        cap_height_mm(&mut reg.fs, &resolved, size_mm)
    } else {
        0.0
    };
    Ok(Prepared {
        resolved,
        size_mm,
        line_mm,
        grid,
        cap_mm,
    })
}

/// The lines of a shaped buffer, and where the box puts them.
fn arrange_buffer(buf: &Buffer, req: &TextRequest, prep: &Prepared) -> (Vec<RunBox>, Arranged) {
    let runs: Vec<RunBox> = buf
        .layout_runs()
        .map(|r| RunBox {
            line_i: r.line_i,
            line_top: r.line_top,
            line_y: r.line_y,
            line_height: r.line_height,
        })
        .collect();
    let arranged = arrange(&runs, req, prep.grid, prep.cap_mm);
    (runs, arranged)
}

/// One laid-out line (cosmic-text's "run") as the arrangement reads it.
#[derive(Debug, Clone, Copy)]
struct RunBox {
    /// The paragraph it belongs to.
    line_i: usize,
    line_top: f32,
    line_y: f32,
    line_height: f32,
}

/// Where one line goes in the box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSlot {
    /// Added to the line's own baseline (mm).
    pub shift: f32,
    /// Top of the line, mm from the box's top.
    pub top: f32,
    /// Bottom of the line, mm from the box's top.
    pub bottom: f32,
    /// False when a Fixed box leaves the line out because it does not fully fit.
    pub visible: bool,
}

/// How a box's lines are placed (rung 11b): where each goes, how many a Fixed box leaves out, and
/// the height of the text.
#[derive(Debug, Clone, PartialEq)]
struct Arranged {
    lines: Vec<LineSlot>,
    hidden: usize,
    height_mm: f32,
}

/// Places the lines of one box.
fn arrange(
    runs: &[RunBox],
    req: &TextRequest,
    grid: Option<BaselineGrid>,
    cap_mm: f32,
) -> Arranged {
    if let (Some(g), false) = (grid, runs.is_empty()) {
        return arrange_on_rhythm(runs, req, g, cap_mm);
    }
    let para = req.paragraph_mm as f32;
    // Each paragraph after the first starts a paragraph's space lower.
    let mut breaks = 0u32;
    let mut lines: Vec<LineSlot> = Vec::with_capacity(runs.len());
    for (j, r) in runs.iter().enumerate() {
        if j > 0 && r.line_i != runs[j - 1].line_i {
            breaks += 1;
        }
        let shift = breaks as f32 * para;
        lines.push(LineSlot {
            shift,
            top: r.line_top + shift,
            bottom: r.line_top + shift + r.line_height,
            visible: true,
        });
    }
    // The height of the text: every line's height added in turn, and the paragraphs' space.
    let height_mm: f32 = runs.iter().map(|r| r.line_height).sum::<f32>() + breaks as f32 * para;
    // Where a Fixed box puts the text between its top and bottom (never above the top).
    if let Some(h) = req.height_mm {
        let free = (h as f32 - height_mm).max(0.0);
        let down = match req.text.v_align {
            VerticalAlign::Top => 0.0,
            VerticalAlign::Middle => free / 2.0,
            VerticalAlign::Bottom => free,
        };
        if down != 0.0 {
            for l in &mut lines {
                l.shift += down;
                l.top += down;
                l.bottom += down;
            }
        }
    }
    let mut hidden = 0;
    if let Some(h) = req.height_mm {
        // Whole lines only: a line that does not fully fit is left out, and so is every line below.
        let h = h as f32 + 1e-4;
        for l in &mut lines {
            l.visible = l.bottom <= h;
            hidden += usize::from(!l.visible);
        }
    }
    Arranged {
        lines,
        hidden,
        height_mm,
    }
}

/// Places the lines of a box whose baselines stand on the rhythm (rung 11b): the first on the first
/// rhythm line at least a capital's height down, the others a whole number of steps below it (a
/// paragraph's space rounded up to whole steps), and a Fixed box's Middle and Bottom moving the
/// text by whole steps.
fn arrange_on_rhythm(runs: &[RunBox], req: &TextRequest, g: BaselineGrid, cap_mm: f32) -> Arranged {
    let step = g.step_mm();
    let per_line = (f64::from(runs[0].line_height) / step).round().max(1.0) as i64;
    let per_para = if req.paragraph_mm > 0.0 {
        (req.paragraph_mm / step - 1e-6).ceil().max(0.0) as i64
    } else {
        0
    };
    let first = g.first_at_or_below(f64::from(cap_mm));
    let mut ks = Vec::with_capacity(runs.len());
    let mut k = first;
    for (j, r) in runs.iter().enumerate() {
        if j > 0 {
            k += per_line;
            if r.line_i != runs[j - 1].line_i {
                k += per_para;
            }
        }
        ks.push(k);
    }
    // The line box round a baseline: what is above it and below it.
    let above = |r: &RunBox| r.line_y - r.line_top;
    let below = |r: &RunBox| r.line_top + r.line_height - r.line_y;
    let last = runs.len() - 1;
    let text_bottom = g.line_mm(ks[last]) as f32 + below(&runs[last]);
    let down = match req.height_mm {
        Some(h) => {
            let room = ((h as f32 - text_bottom).max(0.0)) as f64 / step;
            match req.text.v_align {
                VerticalAlign::Top => 0,
                VerticalAlign::Middle => (room / 2.0).floor() as i64,
                VerticalAlign::Bottom => room.floor() as i64,
            }
        }
        None => 0,
    };
    let mut lines = Vec::with_capacity(runs.len());
    let mut hidden = 0;
    for (r, k) in runs.iter().zip(&ks) {
        let baseline = g.line_mm(k + down) as f32;
        let top = baseline - above(r);
        let bottom = baseline + below(r);
        let visible = req.height_mm.is_none_or(|h| bottom <= h as f32 + 1e-4);
        hidden += usize::from(!visible);
        lines.push(LineSlot {
            shift: baseline - r.line_y,
            top,
            bottom,
            visible,
        });
    }
    Arranged {
        lines,
        hidden,
        height_mm: text_bottom,
    }
}

/// A capital's height in mm in the font a box uses, or 0.7 of the size when the font does not say.
fn cap_height_mm(fs: &mut FontSystem, font: &registry::Resolved, size_mm: f32) -> f32 {
    let guess = size_mm * 0.7;
    let id = fs.db().query(&fontdb::Query {
        families: &[fontdb::Family::Name(&font.family)],
        weight: fontdb::Weight(font.weight),
        style: font.style,
        ..Default::default()
    });
    let Some(id) = id else {
        return guess;
    };
    let index = fs.db().face(id).map(|f| f.index).unwrap_or(0);
    let Some(loaded) = fs.get_font(id, Weight(font.weight)) else {
        return guess;
    };
    let Ok(f) = font_ref(loaded.data(), index) else {
        return guess;
    };
    let m = f.metrics(Size::unscaled(), LocationRef::default());
    match m.cap_height {
        Some(c) if m.units_per_em > 0 && c > 0.0 => c / f32::from(m.units_per_em) * size_mm,
        _ => guess,
    }
}

/// Lays one box out with cosmic-text, in millimetres.
fn paragraph(
    fs: &mut FontSystem,
    tb: &TextBox,
    font: &registry::Resolved,
    size_mm: f32,
    line_mm: f32,
    width_mm: f32,
) -> Buffer {
    let mut buf = Buffer::new(fs, Metrics::new(size_mm, line_mm));
    buf.set_wrap(Wrap::WordOrGlyph);
    buf.set_size(Some(width_mm), None);
    let mut attrs = Attrs::new()
        .family(Family::Name(&font.family))
        .weight(Weight(font.weight))
        .style(font.style);
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

/// Draws one glyph in font units at the face's weight (and, in a copy made for an optical size,
/// at that size: see [`opsz`]) and stores it in half font units. Returns the shape and how many
/// coordinates had to be rounded.
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
