//! Per-profile layout: Scale constraints for position, role widths from physical size,
//! and pixel snapping (napkin §10.2).

use crate::scene::{Role, Scene, Shape};
use crate::Profile;

/// One drawing command in device pixels. Shared by every backend.
#[derive(Clone, Debug)]
pub enum Item {
    /// Axis-aligned filled rectangle. `aa: false` means hard edges (snapped lines).
    Rect { x0: f64, y0: f64, x1: f64, y1: f64, gray: u8, aa: bool },
    /// Anti-aliased stroked segment with butt caps.
    Line { x0: f64, y0: f64, x1: f64, y1: f64, width: f64, gray: u8 },
    /// Anti-aliased filled circle.
    Dot { cx: f64, cy: f64, r: f64, gray: u8 },
    /// Anti-aliased stroked rounded rect.
    RoundRect { x: f64, y: f64, w: f64, h: f64, radius: f64, width: f64, gray: u8 },
    /// Anti-aliased stroked ellipse.
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64, width: f64, gray: u8 },
}

pub struct DisplayList {
    pub width: u32,
    pub height: u32,
    pub background: u8,
    pub items: Vec<Item>,
}

/// Role width in whole device pixels, never below 1 (napkin §10.2).
pub fn role_px(role: Role, p: &Profile) -> u32 {
    (p.mm_to_px(role.width_mm()).round() as u32).max(1)
}

/// Snap a stroke centre: odd widths on pixel centres, even widths on pixel edges.
pub fn snap_center(pos: f64, width_px: u32) -> f64 {
    if width_px % 2 == 1 {
        pos.floor() + 0.5
    } else {
        pos.round()
    }
}

/// Pixel-locked rhythm (napkin §6.3): a unit in mm becomes a whole number of device pixels.
pub fn rhythm_px(unit_mm: f64, p: &Profile) -> u32 {
    (p.mm_to_px(unit_mm).round() as u32).max(1)
}

pub fn layout(scene: &Scene, p: &Profile, snap: bool) -> DisplayList {
    let sx = p.width_px as f64 / scene.width_mm;
    let sy = p.height_px as f64 / scene.height_mm;
    let mut items = Vec::with_capacity(scene.shapes.len());
    for s in &scene.shapes {
        items.push(match *s {
            Shape::HLine { y, x0, x1, role } => {
                let w = role_px(role, p);
                let cy = y * sy;
                let (cy, a, b) = if snap { (snap_center(cy, w), (x0 * sx).round(), (x1 * sx).round()) } else { (cy, x0 * sx, x1 * sx) };
                Item::Rect { x0: a, y0: cy - w as f64 / 2.0, x1: b, y1: cy + w as f64 / 2.0, gray: role.gray(), aa: !snap }
            }
            Shape::VLine { x, y0, y1, role } => {
                let w = role_px(role, p);
                let cx = x * sx;
                let (cx, a, b) = if snap { (snap_center(cx, w), (y0 * sy).round(), (y1 * sy).round()) } else { (cx, y0 * sy, y1 * sy) };
                Item::Rect { x0: cx - w as f64 / 2.0, y0: a, x1: cx + w as f64 / 2.0, y1: b, gray: role.gray(), aa: !snap }
            }
            Shape::Line { x0, y0, x1, y1, role } => Item::Line { x0: x0 * sx, y0: y0 * sy, x1: x1 * sx, y1: y1 * sy, width: role_px(role, p) as f64, gray: role.gray() },
            Shape::Dot { cx, cy, r, gray } => Item::Dot { cx: cx * sx, cy: cy * sy, r: p.mm_to_px(r), gray },
            Shape::RoundRect { x, y, w, h, radius, role } => {
                let sw = role_px(role, p);
                // Snap the straight edges too: rect edges land like an HLine/VLine of that width.
                let (mut x, mut y, mut w2, mut h2) = (x * sx, y * sy, w * sx, h * sy);
                if snap {
                    let (l, t) = (snap_center(x, sw), snap_center(y, sw));
                    let (r, b) = (snap_center(x + w2, sw), snap_center(y + h2, sw));
                    (x, y, w2, h2) = (l, t, r - l, b - t);
                }
                Item::RoundRect { x, y, w: w2, h: h2, radius: p.mm_to_px(radius), width: sw as f64, gray: role.gray() }
            }
            Shape::Ellipse { cx, cy, rx, ry, role } => Item::Ellipse { cx: cx * sx, cy: cy * sy, rx: rx * sx, ry: ry * sy, width: role_px(role, p) as f64, gray: role.gray() },
        });
    }
    DisplayList { width: p.width_px, height: p.height_px, background: scene.background, items }
}
