//! The design, in millimetres (napkin §5.3: geometry stored in mm, f64).

/// Line roles (napkin §8.2) with placeholder widths and grays. Real values come from
/// maintainer calibration; these are only here to exercise 1-4 px strokes.
#[derive(Clone, Copy, Debug)]
pub enum Role {
    Whisper,
    Guide,
    Structure,
    Emphasis,
}

impl Role {
    pub fn width_mm(self) -> f64 {
        match self {
            Role::Whisper => 0.085,   // 1 px @300, 1 px @226
            Role::Guide => 0.17,      // 2 px @300, 2 px @226
            Role::Structure => 0.254, // 3 px @300, 2 px @226
            Role::Emphasis => 0.34,   // 4 px @300, 3 px @226
        }
    }
    pub fn gray(self) -> u8 {
        match self {
            Role::Whisper => 0xC0,
            Role::Guide => 0x90,
            Role::Structure => 0x60,
            Role::Emphasis => 0x30,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    /// Horizontal line from x0 to x1 at y.
    HLine { y: f64, x0: f64, x1: f64, role: Role },
    /// Vertical line from y0 to y1 at x.
    VLine { x: f64, y0: f64, y1: f64, role: Role },
    /// Any-angle line (always anti-aliased).
    Line { x0: f64, y0: f64, x1: f64, y1: f64, role: Role },
    /// Filled dot.
    Dot { cx: f64, cy: f64, r: f64, gray: u8 },
    /// Stroked rounded rectangle.
    RoundRect { x: f64, y: f64, w: f64, h: f64, radius: f64, role: Role },
    /// Stroked ellipse.
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64, role: Role },
}

pub struct Scene {
    pub width_mm: f64,
    pub height_mm: f64,
    pub background: u8,
    pub shapes: Vec<Shape>,
}

/// The spike's test page, in a 3:4 page of 162.6 x 216.8 mm (Manta's size at 300 PPI).
/// Laid out with Scale constraints: every coordinate is a fraction of the page, so the
/// same scene fits A5 X too.
pub fn test_scene() -> Scene {
    let (pw, ph) = (162.56, 216.746_666);
    let mut s = Vec::new();

    // Ruled band: one line per role, spaced 6 mm, so each role's width is easy to measure.
    let roles = [Role::Whisper, Role::Guide, Role::Structure, Role::Emphasis];
    for (i, r) in roles.iter().enumerate() {
        s.push(Shape::HLine { y: 20.0 + 6.0 * i as f64, x0: 10.0, x1: pw - 10.0, role: *r });
    }
    // Vertical lines, one per role.
    for (i, r) in roles.iter().enumerate() {
        s.push(Shape::VLine { x: 20.0 + 6.0 * i as f64, y0: 50.0, y1: 80.0, role: *r });
    }

    // Ruled region: 10 Guide lines at a 6 mm rhythm (pixel-locked spacing tested separately).
    for i in 0..10 {
        s.push(Shape::HLine { y: 90.0 + 6.0 * i as f64, x0: 50.0, x1: pw - 10.0, role: Role::Guide });
    }

    // Dot grid, 5 mm, in the lower-left.
    for gy in 0..8 {
        for gx in 0..8 {
            s.push(Shape::Dot { cx: 12.0 + 5.0 * gx as f64, cy: 160.0 + 5.0 * gy as f64, r: 0.25, gray: Role::Guide.gray() });
        }
    }

    // Isometric grid, 5 mm side, lower-right: horizontals + 60/120 degree lines, clipped to a box.
    let (bx0, by0, bx1, by1) = (60.0, 155.0, pw - 10.0, ph - 15.0);
    let side = 5.0_f64;
    let row = side * (3.0_f64).sqrt() / 2.0;
    let mut y = by0;
    while y <= by1 + 1e-9 {
        s.push(Shape::HLine { y, x0: bx0, x1: bx1, role: Role::Whisper });
        y += row;
    }
    let slope = (3.0_f64).sqrt(); // 60 degrees
    let span = (by1 - by0) / slope;
    let mut x = bx0 - span;
    while x <= bx1 + 1e-9 {
        for dir in [1.0, -1.0] {
            // Line through (x, by1) going up at +/-60 degrees.
            let (ax, ay) = (x, by1);
            let (bx, byy) = (x + dir * span, by0);
            if let Some((p, q)) = clip(ax, ay, bx, byy, bx0, by0, bx1, by1) {
                s.push(Shape::Line { x0: p.0, y0: p.1, x1: q.0, y1: q.1, role: Role::Whisper });
            }
        }
        x += side;
    }

    // A rounded rect and an ellipse.
    s.push(Shape::RoundRect { x: 90.0, y: 40.0, w: 55.0, h: 30.0, radius: 4.0, role: Role::Structure });
    s.push(Shape::Ellipse { cx: 117.5, cy: 55.0, rx: 20.0, ry: 10.0, role: Role::Guide });

    Scene { width_mm: pw, height_mm: ph, background: 0xFF, shapes: s }
}

/// Liang-Barsky clip of a segment to a box.
#[allow(clippy::too_many_arguments)]
fn clip(x0: f64, y0: f64, x1: f64, y1: f64, bx0: f64, by0: f64, bx1: f64, by1: f64) -> Option<((f64, f64), (f64, f64))> {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
    for (p, q) in [(-dx, x0 - bx0), (dx, bx1 - x0), (-dy, y0 - by0), (dy, by1 - y0)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    if t0 > t1 - 1e-9 {
        return None;
    }
    Some(((x0 + t0 * dx, y0 + t0 * dy), (x0 + t1 * dx, y0 + t1 * dy)))
}
