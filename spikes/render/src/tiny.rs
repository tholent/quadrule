//! CPU export backend: tiny-skia.

use crate::layout::{DisplayList, Item};
use crate::Gray;
use tiny_skia::{Color, FillRule, LineCap, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

fn paint(gray: u8, aa: bool) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(Color::from_rgba8(gray, gray, gray, 255));
    p.anti_alias = aa;
    p
}

fn stroke(width: f64) -> Stroke {
    Stroke { width: width as f32, line_cap: LineCap::Butt, ..Stroke::default() }
}

pub fn render(dl: &DisplayList) -> Gray {
    let mut pm = Pixmap::new(dl.width, dl.height).expect("pixmap");
    let bg = dl.background;
    pm.fill(Color::from_rgba8(bg, bg, bg, 255));
    let t = Transform::identity();
    for it in &dl.items {
        match *it {
            Item::Rect { x0, y0, x1, y1, gray, aa } => {
                if let Some(r) = Rect::from_ltrb(x0 as f32, y0 as f32, x1 as f32, y1 as f32) {
                    pm.fill_rect(r, &paint(gray, aa), t, None);
                }
            }
            Item::Line { x0, y0, x1, y1, width, gray } => {
                let mut pb = PathBuilder::new();
                pb.move_to(x0 as f32, y0 as f32);
                pb.line_to(x1 as f32, y1 as f32);
                if let Some(path) = pb.finish() {
                    pm.stroke_path(&path, &paint(gray, true), &stroke(width), t, None);
                }
            }
            Item::Dot { cx, cy, r, gray } => {
                if let Some(path) = PathBuilder::from_circle(cx as f32, cy as f32, r as f32) {
                    pm.fill_path(&path, &paint(gray, true), FillRule::Winding, t, None);
                }
            }
            Item::RoundRect { x, y, w, h, radius, width, gray } => {
                if let Some(path) = round_rect(x as f32, y as f32, w as f32, h as f32, radius as f32) {
                    pm.stroke_path(&path, &paint(gray, true), &stroke(width), t, None);
                }
            }
            Item::Ellipse { cx, cy, rx, ry, width, gray } => {
                let r = Rect::from_ltrb((cx - rx) as f32, (cy - ry) as f32, (cx + rx) as f32, (cy + ry) as f32);
                if let Some(path) = r.and_then(PathBuilder::from_oval) {
                    pm.stroke_path(&path, &paint(gray, true), &stroke(width), t, None);
                }
            }
        }
    }
    // Opaque gray in -> R == G == B; take R. Alpha is 255 everywhere (background flattened).
    let data = pm.data().as_chunks::<4>().0.iter().map(|px| px[0]).collect();
    Gray { width: dl.width, height: dl.height, data }
}

/// Rounded rect from four quarter arcs (cubic approximation, k = 0.5523).
fn round_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = 0.552_284_8 * r;
    let (x1, y1) = (x + w, y + h);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x1 - r, y);
    pb.cubic_to(x1 - r + k, y, x1, y + r - k, x1, y + r);
    pb.line_to(x1, y1 - r);
    pb.cubic_to(x1, y1 - r + k, x1 - r + k, y1, x1 - r, y1);
    pb.line_to(x + r, y1);
    pb.cubic_to(x + r - k, y1, x, y1 - r + k, x, y1 - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
    pb.finish()
}
