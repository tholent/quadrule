//! Shared pixel checks, usable with any backend.

use crate::layout::{layout, role_px, DisplayList};
use crate::scene::{test_scene, Role};
use crate::{Gray, Profile, A5X, MANTA};

pub const ROLES: [Role; 4] = [Role::Whisper, Role::Guide, Role::Structure, Role::Emphasis];

pub fn mm_y(p: &Profile, mm: f64) -> f64 {
    mm * p.height_px as f64 / test_scene().height_mm
}
pub fn mm_x(p: &Profile, mm: f64) -> f64 {
    mm * p.width_px as f64 / test_scene().width_mm
}

/// Pixels along a column (vertical=true) or row, within +/- 12 px of `centre`.
pub fn window(img: &Gray, fixed: u32, centre: f64, vertical: bool) -> Vec<u8> {
    let c = centre as i64;
    (c - 12..=c + 12).map(|v| if vertical { img.at(fixed, v as u32) } else { img.at(v as u32, fixed) }).collect()
}

/// For each role line of the given width parity, on both profiles, horizontal and vertical:
/// exactly `role_px` pixels of the role's gray and no in-between grays.
/// Returns (lines checked, failures).
pub fn check_lines(render: &dyn Fn(&DisplayList) -> Gray, parity_odd: bool) -> (usize, Vec<String>) {
    let mut checked = 0;
    let mut fails = Vec::new();
    for p in [MANTA, A5X] {
        let img = render(&layout(&test_scene(), &p, true));
        let mid_x = p.width_px / 2;
        let row_y = mm_y(&p, 65.0) as u32;
        for (i, r) in ROLES.iter().enumerate() {
            let w = role_px(*r, &p);
            if (w % 2 == 1) != parity_odd {
                continue;
            }
            for (vertical, centre, fixed) in [(true, mm_y(&p, 20.0 + 6.0 * i as f64), mid_x), (false, mm_x(&p, 20.0 + 6.0 * i as f64), row_y)] {
                checked += 1;
                let px = window(&img, fixed, centre, vertical);
                let ink = px.iter().filter(|&&v| v == r.gray()).count() as u32;
                let stray = px.iter().filter(|&&v| v != r.gray() && v != 0xFF).count();
                if ink != w || stray != 0 {
                    fails.push(format!("{}:{:?}:{}:w={w}:ink={ink}:stray={stray}", p.id, r, if vertical { "h" } else { "v" }));
                }
            }
        }
    }
    (checked, fails)
}

/// (pixels that differ, largest difference)
pub fn diff(a: &Gray, b: &Gray) -> (usize, u8) {
    let mut n = 0;
    let mut m = 0;
    for (x, y) in a.data.iter().zip(&b.data) {
        let d = x.abs_diff(*y);
        if d > 0 {
            n += 1;
            m = m.max(d);
        }
    }
    (n, m)
}
