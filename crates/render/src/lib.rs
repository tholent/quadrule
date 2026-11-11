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

/// Which arithmetic vello_cpu uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Math {
    /// The fastest the CPU offers (SIMD). The default.
    Fastest,
    /// Portable scalar maths, the lever ADR-003 item 2 keeps in case machines disagree.
    Portable,
}

/// Draws a display list with vello_cpu.
///
/// # Panics
/// If the page is larger than 65535 pixels on a side (vello_cpu's limit); no profile is.
pub fn render(dl: &DisplayList, math: Math) -> Gray {
    let w = u16::try_from(dl.width).expect("page width fits vello_cpu");
    let h = u16::try_from(dl.height).expect("page height fits vello_cpu");
    let mut settings = RenderSettings::default();
    if math == Math::Portable {
        settings.level = Level::baseline();
    }
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
    /// The app and its version, e.g. `Quadrule 0.1.0`.
    pub software: &'a str,
    /// The source document's title.
    pub title: &'a str,
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
            enc.add_text_chunk("Software".into(), m.software.into())
                .expect("text chunk");
            enc.add_text_chunk("Title".into(), m.title.into())
                .expect("text chunk");
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
