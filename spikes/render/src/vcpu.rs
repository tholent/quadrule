//! CPU backend: vello_cpu. Same display list as tiny-skia.

use crate::layout::{DisplayList, Item};
use crate::Gray;
use vello_cpu::color::AlphaColor;
use vello_cpu::kurbo::{BezPath, Cap, Ellipse, Line, Rect, RoundedRect, Shape, Stroke};
use vello_cpu::{Level, Pixmap, RenderContext, RenderSettings, Resources};

/// `simd: false` forces the scalar baseline (a candidate for cross-CPU determinism).
pub fn render(dl: &DisplayList, simd: bool) -> Gray {
    let (w, h) = (dl.width as u16, dl.height as u16);
    let mut settings = RenderSettings::default();
    if !simd {
        settings.level = Level::baseline();
    }
    let mut ctx = RenderContext::new_with(w, h, settings);
    let mut res = Resources::new();
    let g = |v: u8| AlphaColor::from_rgba8(v, v, v, 255);
    ctx.set_paint(g(dl.background));
    ctx.fill_rect(&Rect::new(0.0, 0.0, w as f64, h as f64));
    for it in &dl.items {
        match *it {
            Item::Rect { x0, y0, x1, y1, gray, aa } => {
                ctx.set_aliasing_threshold(if aa { None } else { Some(127) });
                ctx.set_paint(g(gray));
                ctx.fill_rect(&Rect::new(x0, y0, x1, y1));
                ctx.set_aliasing_threshold(None);
            }
            Item::Line { x0, y0, x1, y1, width, gray } => {
                stroke(&mut ctx, Line::new((x0, y0), (x1, y1)).into_path(0.1), width, g(gray));
            }
            Item::Dot { cx, cy, r, gray } => {
                ctx.set_paint(g(gray));
                ctx.fill_path(&Ellipse::new((cx, cy), (r, r), 0.0).into_path(0.1));
            }
            Item::RoundRect { x, y, w, h, radius, width, gray } => {
                stroke(&mut ctx, RoundedRect::new(x, y, x + w, y + h, radius).into_path(0.1), width, g(gray));
            }
            Item::Ellipse { cx, cy, rx, ry, width, gray } => {
                stroke(&mut ctx, Ellipse::new((cx, cy), (rx, ry), 0.0).into_path(0.1), width, g(gray));
            }
        }
    }
    let mut target = Pixmap::new(w, h);
    ctx.flush();
    ctx.render(&mut target, &mut res);
    let data = target.data().iter().map(|p| p.r).collect();
    Gray { width: dl.width, height: dl.height, data }
}

fn stroke(ctx: &mut RenderContext, path: BezPath, width: f64, c: AlphaColor<vello_cpu::color::Srgb>) {
    ctx.set_paint(c);
    ctx.set_stroke(Stroke::new(width).with_caps(Cap::Butt));
    ctx.stroke_path(&path);
}
