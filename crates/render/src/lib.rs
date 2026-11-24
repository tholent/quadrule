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

//! The export renderer (napkin §10.1, §11.1) as ADR-003 decides it: vello_cpu draws the
//! display list on one thread, and the result is written as an 8-bit grayscale PNG with no
//! alpha. The GPU never makes export files (item 3).
//!
//! Not built yet: the vello canvas renderer (napkin §10.1; a later rung with the window) and
//! batch export with the naming scheme (§11.2).

#![forbid(unsafe_code)]

use quadrule_layout::{DisplayList, Item};
use sha2::{Digest, Sha256};
use vello_cpu::color::{AlphaColor, Srgb};
use vello_cpu::kurbo::{Cap, Join, Line, Rect, Shape, Stroke};
use vello_cpu::{Level, Pixmap, RenderContext, RenderSettings, Resources};

/// An 8-bit grayscale image: 0 = black, 255 = white, row by row, no alpha.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Gray {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height` bytes.
    pub data: Vec<u8>,
}

impl Gray {
    /// The pixel at (x, y).
    pub fn at(&self, x: u32, y: u32) -> u8 {
        self.data[(y * self.width + x) as usize]
    }
}

/// The image turned a quarter turn clockwise, exactly: no resampling, only a permutation of the
/// pixels (rung 8b, Q6). The pixel at (x, y) of an image of height `H` goes to (`H - 1 - y`, x) of
/// the result, whose width is `H` and whose height is the old width. A landscape design is exported
/// this way into the device's portrait PNG: the design's top edge lands on the device's right edge.
pub fn turn_clockwise(img: &Gray) -> Gray {
    let (w, h) = (img.width as usize, img.height as usize);
    let mut data = vec![0u8; w * h];
    // The result has `h` columns and `w` rows; result row `x` is the old column `x`, read from the
    // bottom row up.
    for y in 0..h {
        let src = &img.data[y * w..(y + 1) * w];
        let col = h - 1 - y;
        for (x, &v) in src.iter().enumerate() {
            data[x * h + col] = v;
        }
    }
    Gray {
        width: img.height,
        height: img.width,
        data,
    }
}

/// Shrinks `img` to `out_w` × `out_h` by averaging: each output pixel is the mean of the device
/// pixels it covers, weighted by how much of each it covers. Zoomed out, a 1 px line or a small dot
/// turns lighter gray but never disappears, as it would if every n-th pixel were picked.
pub fn shrink(img: &Gray, out_w: u32, out_h: u32) -> Gray {
    let (out_w, out_h) = (out_w.clamp(1, img.width), out_h.clamp(1, img.height));
    // Across, then down; each pass is a box filter with fractional edges.
    let across = box_pass(&img.data, img.width, img.height, out_w, true);
    let down = box_pass(&across, out_w, img.height, out_h, false);
    Gray {
        width: out_w,
        height: out_h,
        data: down
            .iter()
            .map(|v| v.round().clamp(0.0, 255.0) as u8)
            .collect(),
    }
}

/// One box-filter pass over a `w` × `h` image, shrinking along x (`across`) or y to `n`.
fn box_pass<T: Copy + Into<f64>>(src: &[T], w: u32, h: u32, n: u32, across: bool) -> Vec<f64> {
    let (len, lines) = if across { (w, h) } else { (h, w) };
    let ratio = len as f64 / n as f64;
    let (ow, oh) = if across { (n, h) } else { (w, n) };
    let mut out = vec![0.0; (ow * oh) as usize];
    for i in 0..n {
        let (a, b) = (i as f64 * ratio, (i + 1) as f64 * ratio);
        let (first, last) = (a.floor() as u32, (b.ceil() as u32).min(len));
        for line in 0..lines {
            let mut sum = 0.0;
            for k in first..last {
                let cover = (b.min(k as f64 + 1.0) - a.max(k as f64)).max(0.0);
                let idx = if across { line * w + k } else { k * w + line };
                sum += src[idx as usize].into() * cover;
            }
            let o = if across { line * n + i } else { i * w + line };
            out[o as usize] = sum / ratio;
        }
    }
    out
}

/// The arithmetic vello_cpu draws with, on every computer (ADR-018): fearless_simd's scalar
/// fallback. Its multiply-add is a multiply then an add, as at the SSE levels; AVX2 and NEON
/// fuse the two into one rounding, so a detected level gave different edge pixels on a
/// computer without FMA. Any other level here is a build error (below).
pub const LEVEL: Level = Level::fallback();

const _: () = assert!(
    matches!(LEVEL, Level::Fallback(_)),
    "ADR-018: export draws at the fallback level on every computer"
);

/// Draws a display list with vello_cpu, at [`LEVEL`].
///
/// # Panics
/// If the page is larger than 65535 pixels on a side (vello_cpu's limit); no profile is.
pub fn render(dl: &DisplayList) -> Gray {
    let w = u16::try_from(dl.width).expect("page width fits vello_cpu");
    let h = u16::try_from(dl.height).expect("page height fits vello_cpu");
    let settings = RenderSettings {
        level: LEVEL,
        ..RenderSettings::default()
    };
    let mut ctx = RenderContext::new_with(w, h, settings);
    ctx.set_paint(gray(dl.background));
    ctx.fill_rect(&Rect::new(0.0, 0.0, w as f64, h as f64));
    for item in &dl.items {
        match item {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray: g,
            } => {
                // Snapped shapes sit on whole pixels; the threshold keeps their edges hard.
                ctx.set_aliasing_threshold(Some(127));
                ctx.set_paint(gray(*g));
                ctx.fill_rect(&Rect::new(*x0, *y0, *x1, *y1));
                ctx.set_aliasing_threshold(None);
            }
            Item::Stroke {
                x0,
                y0,
                x1,
                y1,
                width,
                gray: g,
            } => {
                ctx.set_paint(gray(*g));
                ctx.set_stroke(Stroke::new(*width).with_caps(Cap::Butt));
                ctx.stroke_path(&Line::new((*x0, *y0), (*x1, *y1)).into_path(0.1));
            }
            Item::Fill { path, gray: g } => {
                ctx.set_paint(gray(*g));
                ctx.fill_path(path);
            }
            Item::StrokePath {
                path,
                width,
                gray: g,
                cap,
            } => {
                ctx.set_paint(gray(*g));
                ctx.set_stroke(
                    Stroke::new(*width)
                        .with_caps(quadrule_layout::kurbo_cap(*cap))
                        .with_join(Join::Miter),
                );
                ctx.stroke_path(path);
            }
            Item::BeginAlpha { alpha, bounds } => {
                // The group is drawn alone, then laid over what is beneath with its opacity: the
                // page is opaque, so the result is a gray, flattened (napkin §5.2).
                let (x0, y0, x1, y1) = *bounds;
                let clip = Rect::new(
                    x0.floor() - 1.0,
                    y0.floor() - 1.0,
                    x1.ceil() + 1.0,
                    y1.ceil() + 1.0,
                )
                .to_path(0.1);
                ctx.push_layer(
                    Some(&clip),
                    None,
                    Some(f32::from(*alpha) / 255.0),
                    None,
                    None,
                );
            }
            Item::EndAlpha => ctx.pop_layer(),
        }
    }
    ctx.flush();
    let mut target = Pixmap::new(w, h);
    ctx.render(&mut target, &mut Resources::new());
    // The page is opaque, so every pixel's alpha is 255 and red = green = blue = the gray.
    let data = target.data().iter().map(|p| p.r).collect();
    Gray {
        width: dl.width,
        height: dl.height,
        data,
    }
}

fn gray(v: u8) -> AlphaColor<Srgb> {
    AlphaColor::from_rgba8(v, v, v, 255)
}

/// The optional PNG text note (napkin §11.1).
#[derive(Clone, Copy, Debug)]
pub struct PngMeta<'a> {
    /// The app and its version, e.g. `Quadrule 0.1.0`. Left out when empty.
    pub software: &'a str,
    /// The source document's title. Left out when empty.
    pub title: &'a str,
}

/// Adds one text chunk: a `tEXt` chunk (Latin-1) when every character fits Latin-1, else an `iTXt` chunk (UTF-8)
/// (rung 15 bbq, Defaulted "Text note encoding"). A NUL cannot be in either, so it becomes a space.
fn add_note(enc: &mut png::Encoder<&mut Vec<u8>>, keyword: &str, text: &str) {
    if text.is_empty() {
        return;
    }
    let text: String = text
        .chars()
        .map(|c| if c == '\0' { ' ' } else { c })
        .collect();
    if text.chars().all(|c| (c as u32) < 256) {
        enc.add_text_chunk(keyword.into(), text)
            .expect("text chunk");
    } else {
        enc.add_itxt_chunk(keyword.into(), text)
            .expect("text chunk");
    }
}

/// Encodes an 8-bit grayscale PNG, no alpha, with an optional text note (napkin §11.1).
pub fn encode_png(img: &Gray, meta: Option<PngMeta>) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.width, img.height);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        // Fixed explicitly, so a change in the png crate's defaults cannot change export bytes.
        enc.set_compression(png::Compression::Balanced);
        enc.set_filter(png::Filter::Adaptive);
        if let Some(m) = meta {
            add_note(&mut enc, "Software", m.software);
            add_note(&mut enc, "Title", m.title);
        }
        let mut w = enc.write_header().expect("PNG header into a Vec");
        w.write_image_data(&img.data).expect("PNG data into a Vec");
    }
    out
}

/// Lowercase hex sha256.
pub fn sha256_hex(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}
