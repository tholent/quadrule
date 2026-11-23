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

//! The measured font signature (ADR-002 item 7, ADR-001 item 6; rung 12a) and the ranking that
//! picks the closest installed font when one is missing (spike `spikes/text/FINDINGS.md`, T13).
//!
//! A font's class (serif, sans, mono, script) is found by measuring its letters first: whether
//! every letter has the same width (mono), whether the lowercase letters sit on the baseline and
//! under the x-height the way a typeface's do (a handwriting face wobbles), and whether the feet of
//! an "n" are wider than its stem (serif). Only when the measurement cannot say (a font with no
//! Latin letters) do the font's own labels and then the words in its name decide, because the
//! labels are often empty: none of our bundled faces but Inter sets them.
//!
//! The file is read through `cosmic_text::skrifa`, as ADR-002 item 1 asks.

use cosmic_text::skrifa::{
    self, GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    raw::TableProvider,
};
use quadrule_model::{FontClass, FontSignature};

/// Lowercase letters that stand flat on the baseline in a typeface.
const FLAT_BOTTOM: &str = "xzkhmnril";
/// Lowercase letters with a flat top at the x-height.
const FLAT_TOP: &str = "xzvwu";
/// A wobble in the baseline or the x-height bigger than this many thousandths of the em marks a
/// handwritten face (a typeface's own letters stray by under 15; Caveat's baseline strays by 58).
const WOBBLE_MILLI: f32 = 25.0;
/// Feet at least this much wider than the stem mark a serif face.
const FOOT_RATIO: f32 = 1.3;

/// A face to measure, set at one weight.
struct Face<'a> {
    f: skrifa::FontRef<'a>,
    loc: skrifa::instance::Location,
    upem: f32,
    /// The weight the variable axis was set to (clamped to its range), if the font has one.
    set_weight: Option<f32>,
}

impl<'a> Face<'a> {
    fn new(data: &'a [u8], index: u32, weight: Option<u16>) -> Option<Self> {
        let f = skrifa::FontRef::from_index(data, index).ok()?;
        let axis = f.axes().get_by_tag(skrifa::Tag::new(b"wght"));
        let set_weight = match (axis, weight) {
            (Some(a), Some(w)) => Some(f32::from(w).clamp(a.min_value(), a.max_value())),
            _ => None,
        };
        let loc = f
            .axes()
            .location(set_weight.map(|w| (skrifa::Tag::new(b"wght"), w)));
        let upem = f32::from(
            f.metrics(Size::unscaled(), LocationRef::default())
                .units_per_em,
        );
        (upem > 0.0).then_some(Face {
            f,
            loc,
            upem,
            set_weight,
        })
    }

    fn at(&self) -> LocationRef<'_> {
        LocationRef::from(&self.loc)
    }
}

/// How wide `text` is in the bundled upright face of `family` at `weight`, in em: the sum of its letters'
/// advances, without kerning (calendar bbq: a calendar picks "Monday", "Mon" or "M" by the room there is).
/// `None` when the family is not bundled or a letter is not in it.
pub fn bundled_advance_em(family: &str, weight: u16, text: &str) -> Option<f64> {
    let font = crate::BundledFont::for_family(family)?;
    let face = Face::new(font.data, 0, Some(weight))?;
    let gm = face.f.glyph_metrics(Size::unscaled(), face.at());
    let cmap = face.f.charmap();
    let mut sum = 0.0_f64;
    for c in text.chars() {
        sum += f64::from(gm.advance_width(cmap.map(c)?)?);
    }
    Some(sum / f64::from(face.upem))
}

/// The capital height of the bundled upright face of `family` at `weight`, in em (the height of a figure too,
/// in a face with lining figures), from the font's own metrics or else from the letter H. `None` when the family
/// is not bundled.
pub fn bundled_cap_em(family: &str, weight: u16) -> Option<f64> {
    let font = crate::BundledFont::for_family(family)?;
    let face = Face::new(font.data, 0, Some(weight))?;
    let m = face.f.metrics(Size::unscaled(), face.at());
    let cap = match m.cap_height {
        Some(c) if c > 0.0 => c,
        _ => bounds(&face, 'H')?.3,
    };
    Some(f64::from(cap / face.upem))
}

/// Measures the face at `index` of a font file of the family `family` (the name is the last resort for
/// the class). `weight` sets a variable font's weight axis (clamped
/// to its range); `None` leaves its default. `None` when the file cannot be read or has no letters.
pub fn measure(
    data: &[u8],
    index: u32,
    weight: Option<u16>,
    family: &str,
) -> Option<FontSignature> {
    let face = Face::new(data, index, weight)?;
    let f = &face.f;
    let m = f.metrics(Size::unscaled(), face.at());
    let upem = face.upem;
    let gm = f.glyph_metrics(Size::unscaled(), face.at());
    let cmap = f.charmap();
    let adv = |c: char| cmap.map(c).and_then(|g| gm.advance_width(g));
    let lower: Vec<f32> = ('a'..='z').filter_map(adv).collect();
    if lower.is_empty() {
        return None;
    }
    let avg = lower.iter().sum::<f32>() / lower.len() as f32 / upem;
    let attrs = f.attributes();
    let x_height = match m.x_height {
        Some(x) if x > 0.0 => x / upem,
        _ => bounds(&face, 'x').map_or(0.0, |b| b.3 / upem),
    };
    let class = classify(&face, family, &lower, adv('i'), adv('M'), adv('w'));
    Some(FontSignature {
        class,
        weight: face
            .set_weight
            .unwrap_or(attrs.weight.value())
            .round()
            .clamp(1.0, 1000.0) as u16,
        width: (attrs.stretch.ratio() * 100.0).round().clamp(1.0, 400.0) as u16,
        italic: attrs.style != skrifa::attribute::Style::Normal,
        avg_width: (avg * 1000.0).round().clamp(0.0, 65535.0) as u16,
        x_height: (x_height * 1000.0).round().clamp(0.0, 65535.0) as u16,
        mono: class == FontClass::Mono,
        extra: Default::default(),
    })
}

/// The class of a font: measurement, then labels, then name words (the name is added by
/// [`with_name`], which knows the family).
fn classify(
    face: &Face,
    family: &str,

    lower: &[f32],
    adv_i: Option<f32>,
    adv_m: Option<f32>,
    adv_w: Option<f32>,
) -> FontClass {
    let (f, upem) = (&face.f, face.upem);
    // Mono: the narrow letter is as wide as the wide ones, and the lowercase letters all share a width.
    let same = |a: f32, b: f32| (a - b).abs() <= upem * 0.004;
    if let (Some(i), Some(m), Some(w)) = (adv_i, adv_m, adv_w)
        && same(i, m)
        && same(i, w)
        && lower.windows(2).all(|p| same(p[0], p[1]))
    {
        return FontClass::Mono;
    }
    // Script: the lowercase letters wobble on the baseline and at the x-height.
    let (below, above) = wobble(face);
    if below > WOBBLE_MILLI || above > WOBBLE_MILLI {
        return FontClass::Script;
    }
    // A font that says it is a script (OS/2 family class 10, PANOSE kind 3) is believed over feet.
    let label = label_class(f);
    if label == Some(FontClass::Script) {
        return FontClass::Script;
    }
    // Serif or sans: the feet of an "n" against its stem.
    if let Some(r) = foot_ratio(face) {
        return if r >= FOOT_RATIO {
            FontClass::Serif
        } else {
            FontClass::Sans
        };
    }
    label
        .or_else(|| name_class(family))
        .unwrap_or(FontClass::Sans)
}

/// How far the bottoms of the letters that stand flat on the baseline, and the tops of those with a
/// flat top at the x-height, stray from one another, in thousandths of the em: (bottoms, tops).
fn wobble(face: &Face) -> (f32, f32) {
    let upem = face.upem;
    let spread = |letters: &str, top: bool| -> f32 {
        let v: Vec<f32> = letters
            .chars()
            .filter_map(|c| bounds(face, c).map(|b| if top { b.3 } else { b.1 }))
            .collect();
        if v.len() < 4 {
            return 0.0;
        }
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        (hi - lo) / upem * 1000.0
    };
    (spread(FLAT_BOTTOM, false), spread(FLAT_TOP, true))
}

/// The numbers the class is judged by, for calibrating the thresholds in a test: (bottom wobble, top
/// wobble, foot ratio).
#[doc(hidden)]
pub fn measurements(data: &[u8], index: u32) -> Option<(f32, f32, f32)> {
    let face = Face::new(data, index, Some(400))?;
    let (a, b) = wobble(&face);
    Some((a, b, foot_ratio(&face).unwrap_or(0.0)))
}

/// What the font's own OS/2 labels say, if anything: family class, then PANOSE.
fn label_class(f: &skrifa::FontRef) -> Option<FontClass> {
    let os2 = f.os2().ok()?;
    match (os2.s_family_class() >> 8) as u8 {
        1..=5 | 7 => return Some(FontClass::Serif),
        8 => return Some(FontClass::Sans),
        10 => return Some(FontClass::Script),
        _ => {}
    }
    let p = os2.panose_10();
    match (p.first(), p.get(1), p.get(3)) {
        (_, _, Some(9)) if p.first() == Some(&2) => Some(FontClass::Mono),
        (Some(3), _, _) => Some(FontClass::Script),
        (Some(2), Some(11..=13), _) => Some(FontClass::Sans),
        (Some(2), Some(2..=10), _) => Some(FontClass::Serif),
        _ => None,
    }
}

/// The class a family name hints at, for a font that measurement and labels could not place.
pub fn name_class(family: &str) -> Option<FontClass> {
    let f = family.to_lowercase();
    let has = |ws: &[&str]| ws.iter().any(|w| f.contains(w));
    if has(&["mono", "code", "courier", "console", "typewriter"]) {
        Some(FontClass::Mono)
    } else if has(&["sans", "gothic", "grotesk", "grotesque", "hyperlegible"]) {
        Some(FontClass::Sans)
    } else if has(&[
        "serif", "roman", "times", "garamond", "bookman", "didot", "century", "georgia",
    ]) {
        Some(FontClass::Serif)
    } else if has(&[
        "hand",
        "script",
        "brush",
        "chancery",
        "cursive",
        "calligraph",
    ]) {
        Some(FontClass::Script)
    } else {
        None
    }
}

/// The bounding box of a letter's outline in font units: (x0, y0, x1, y1).
fn bounds(face: &Face, c: char) -> Option<(f32, f32, f32, f32)> {
    let g = face.f.charmap().map(c)?;
    let b = face
        .f
        .glyph_metrics(Size::unscaled(), face.at())
        .bounds(g)?;
    Some((b.x_min, b.y_min, b.x_max, b.y_max))
}

/// One straight piece of an outline: from, to.
type Seg = ((f32, f32), (f32, f32));

/// Straight pieces of an outline (curves cut into short lines), for counting ink along a line.
#[derive(Default)]
struct Flat {
    segs: Vec<Seg>,
    at: (f32, f32),
    start: (f32, f32),
}

impl Flat {
    fn line(&mut self, to: (f32, f32)) {
        self.segs.push((self.at, to));
        self.at = to;
    }
}

impl OutlinePen for Flat {
    fn move_to(&mut self, x: f32, y: f32) {
        self.at = (x, y);
        self.start = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.line((x, y));
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (p0, steps) = (self.at, 8);
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let u = 1.0 - t;
            self.line((
                u * u * p0.0 + 2.0 * u * t * cx + t * t * x,
                u * u * p0.1 + 2.0 * u * t * cy + t * t * y,
            ));
        }
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let (p0, steps) = (self.at, 8);
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let u = 1.0 - t;
            self.line((
                u * u * u * p0.0 + 3.0 * u * u * t * c0x + 3.0 * u * t * t * c1x + t * t * t * x,
                u * u * u * p0.1 + 3.0 * u * u * t * c0y + 3.0 * u * t * t * c1y + t * t * t * y,
            ));
        }
    }
    fn close(&mut self) {
        if self.at != self.start {
            self.line(self.start);
        }
    }
}

/// The length of ink a horizontal line at height `y` crosses in a letter's outline (even-odd).
fn ink_at(segs: &[Seg], y: f32) -> f32 {
    let mut xs: Vec<f32> = segs
        .iter()
        .filter(|((_, y0), (_, y1))| (*y0 <= y && y < *y1) || (*y1 <= y && y < *y0))
        .map(|((x0, y0), (x1, y1))| x0 + (y - y0) / (y1 - y0) * (x1 - x0))
        .collect();
    xs.sort_by(f32::total_cmp);
    xs.as_chunks::<2>().0.iter().map(|p| p[1] - p[0]).sum()
}

/// How much wider the ink of an "n" is just above the baseline (where a serif's foot is) than a
/// little higher up (where only the stems are). About 1 for a sans, 1.5 and more for a serif.
fn foot_ratio(face: &Face) -> Option<f32> {
    let (f, upem) = (&face.f, face.upem);
    let g = f.charmap().map('n')?;
    let mut pen = Flat::default();
    f.outline_glyphs()
        .get(GlyphId::new(g.to_u32()))?
        .draw(
            DrawSettings::unhinted(Size::unscaled(), face.at()),
            &mut pen,
        )
        .ok()?;
    if pen.segs.is_empty() {
        return None;
    }
    let foot = ink_at(&pen.segs, upem * 0.02);
    let stem = ink_at(&pen.segs, upem * 0.20);
    (stem > 0.0).then(|| foot / stem)
}

/// Lower is closer. The class counts most, then weight, width and italic, then the average letter
/// width and the x-height (the spike's weights, T13, not tuned).
pub fn distance(want: &FontSignature, got: &FontSignature) -> u32 {
    let class = if want.class == got.class { 0 } else { 1000 };
    let weight = u32::from(got.weight.abs_diff(want.weight));
    let width = u32::from(got.width.abs_diff(want.width)) * 5;
    let style = if got.italic == want.italic { 0 } else { 300 };
    let letters = u32::from(got.avg_width.abs_diff(want.avg_width)) * 4;
    let xh = if want.x_height > 0 && got.x_height > 0 {
        u32::from(got.x_height.abs_diff(want.x_height)) * 2
    } else {
        0
    };
    class + weight + width + style + letters + xh
}
