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

//! Safe zones from device screenshots (napkin §3.5): the areas system UI covers.
//!
//! Input: native-resolution screenshots of one note with a plain grid template, one per toolbar
//! position, named `<anything>-<position>.png` (`left`, `right`, `top`, `float`), in
//! `docs/device-measurements/<profile_id>/`.
//!
//! Method: the grid's line positions are read from the middle of the page, where at most one
//! screenshot is covered at any point, and the bare template is rebuilt from them. Every pixel
//! that differs from the bare template is covered. What is covered in **every** screenshot is
//! the footer (page navigation, note name); the rest of each screenshot is its toolbar. Cover
//! boxes hide the grid lines under them, so their edges show where the lines stop.

use quadrule_profiles::{Orientation, SafeZone};
use std::path::Path;

/// One screenshot as 8-bit gray.
pub struct Shot {
    /// Toolbar position, from the file name: `left`, `right`, `top` or `float`.
    pub position: String,
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Row by row.
    pub gray: Vec<u8>,
}

type Result<T> = std::result::Result<T, String>;

/// Toolbar positions a file name may end in.
pub const POSITIONS: [&str; 4] = ["left", "right", "top", "float"];

/// Pixels further than this from the bare template count as covered.
const DIFF: u8 = 40;
/// Pieces closer than this many pixels belong to the same zone.
const JOIN_PX: usize = 30;

/// Reads every `*-<position>.png` in a folder, sorted by file name.
pub fn load_dir(dir: &Path) -> Result<Vec<Shot>> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .collect();
    paths.sort();
    let mut shots = Vec::new();
    for p in paths {
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        let Some(pos) = POSITIONS
            .iter()
            .find(|pos| stem.ends_with(&format!("-{pos}")))
        else {
            continue;
        };
        shots.push(load(&p, pos)?);
    }
    Ok(shots)
}

fn load(path: &Path, position: &str) -> Result<Shot> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut dec = png::Decoder::new(std::io::BufReader::new(file));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut r = dec
        .read_info()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut buf = vec![0; r.output_buffer_size().ok_or("PNG too large")?];
    let info = r
        .next_frame(&mut buf)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let channels = info.color_type.samples();
    let (w, h) = (info.width as usize, info.height as usize);
    let gray = buf[..w * h * channels]
        .chunks(channels)
        .map(|px| match channels {
            1 | 2 => px[0],
            _ => ((px[0] as u16 + px[1] as u16 + px[2] as u16) / 3) as u8,
        })
        .collect();
    Ok(Shot {
        position: position.into(),
        width: w,
        height: h,
        gray,
    })
}

/// The template grid: lines `width` px wide at `offset + k * pitch`, on each axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    /// Vertical lines: first column, spacing, width.
    pub x: (usize, usize, usize),
    /// Horizontal lines: first row, spacing, width.
    pub y: (usize, usize, usize),
}

fn median(v: &mut [u8]) -> f64 {
    v.sort_unstable();
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2] as f64
    } else {
        (v[n / 2 - 1] as f64 + v[n / 2] as f64) / 2.0
    }
}

/// Line starts in a profile (dark where lines are): (offset, pitch, width).
fn lines(profile: &[f64]) -> Result<(usize, usize, usize)> {
    let mut starts = Vec::new();
    let mut widths = Vec::new();
    let mut i = 0;
    while i < profile.len() {
        if profile[i] < 200.0 {
            let s = i;
            while i < profile.len() && profile[i] < 200.0 {
                i += 1;
            }
            starts.push(s);
            widths.push(i - s);
        } else {
            i += 1;
        }
    }
    if starts.len() < 3 {
        return Err("no grid found: use a note with a plain grid template".into());
    }
    let mut gaps: Vec<usize> = starts.windows(2).map(|w| w[1] - w[0]).collect();
    gaps.sort_unstable();
    widths.sort_unstable();
    let pitch = gaps[gaps.len() / 2];
    if gaps.iter().any(|g| g % pitch != 0) {
        return Err("grid lines are not evenly spaced".into());
    }
    Ok((starts[0] % pitch, pitch, widths[widths.len() / 2]))
}

/// Reads the grid from the middle third of the page, using the per-pixel median of the shots.
pub fn find_grid(shots: &[Shot]) -> Result<Grid> {
    let (w, h) = (shots[0].width, shots[0].height);
    let mut px = vec![0u8; shots.len()];
    let mut at = |x: usize, y: usize| {
        for (i, s) in shots.iter().enumerate() {
            px[i] = s.gray[y * w + x];
        }
        median(&mut px)
    };
    let (y0, y1, x0, x1) = (h / 3, 2 * h / 3, w / 3, 2 * w / 3);
    let cols: Vec<f64> = (0..w)
        .map(|x| (y0..y1).map(|y| at(x, y)).sum::<f64>() / (y1 - y0) as f64)
        .collect();
    let rows: Vec<f64> = (0..h)
        .map(|y| (x0..x1).map(|x| at(x, y)).sum::<f64>() / (x1 - x0) as f64)
        .collect();
    Ok(Grid {
        x: lines(&cols)?,
        y: lines(&rows)?,
    })
}

/// The bare template: white with black grid lines.
pub fn template(g: Grid, w: usize, h: usize) -> Vec<u8> {
    let on = |v: usize, (o, p, lw): (usize, usize, usize)| v >= o && (v - o) % p < lw;
    let mut t = vec![255u8; w * h];
    for y in 0..h {
        for x in 0..w {
            if on(x, g.x) || on(y, g.y) {
                t[y * w + x] = 0;
            }
        }
    }
    t
}

/// Bounding boxes (x, y, w, h) of groups of set pixels, joining pieces closer than `JOIN_PX`,
/// sorted by x then y.
fn boxes(mask: &[bool], w: usize, h: usize) -> Vec<(u32, u32, u32, u32)> {
    let (cw, ch) = (w.div_ceil(JOIN_PX), h.div_ceil(JOIN_PX));
    let mut cell = vec![false; cw * ch];
    for (i, _) in mask.iter().enumerate().filter(|(_, m)| **m) {
        cell[(i / w) / JOIN_PX * cw + (i % w) / JOIN_PX] = true;
    }
    let mut label = vec![0usize; cw * ch];
    let mut n = 0;
    for start in 0..cell.len() {
        if !cell[start] || label[start] != 0 {
            continue;
        }
        n += 1;
        label[start] = n;
        let mut stack = vec![start];
        while let Some(c) = stack.pop() {
            let (cy, cx) = ((c / cw) as isize, (c % cw) as isize);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (y, x) = (cy + dy, cx + dx);
                    if y < 0 || x < 0 || y >= ch as isize || x >= cw as isize {
                        continue;
                    }
                    let k = y as usize * cw + x as usize;
                    if cell[k] && label[k] == 0 {
                        label[k] = n;
                        stack.push(k);
                    }
                }
            }
        }
    }
    let mut bb = vec![(usize::MAX, usize::MAX, 0usize, 0usize); n];
    for (i, _) in mask.iter().enumerate().filter(|(_, m)| **m) {
        let (y, x) = (i / w, i % w);
        let b = &mut bb[label[y / JOIN_PX * cw + x / JOIN_PX] - 1];
        *b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
    }
    let mut out: Vec<_> = bb
        .into_iter()
        .map(|(x0, y0, x1, y1)| {
            (
                x0 as u32,
                y0 as u32,
                (x1 - x0 + 1) as u32,
                (y1 - y0 + 1) as u32,
            )
        })
        .collect();
    out.sort();
    out
}

/// Measures the safe zones from one device's portrait screenshots.
pub fn measure(shots: &[Shot]) -> Result<Vec<SafeZone>> {
    if shots.len() < 3 {
        return Err(format!(
            "need at least 3 screenshots with different toolbar positions, found {}",
            shots.len()
        ));
    }
    let (w, h) = (shots[0].width, shots[0].height);
    if shots.iter().any(|s| (s.width, s.height) != (w, h)) || h < w {
        return Err("screenshots must be portrait and all the same size".into());
    }
    let t = template(find_grid(shots)?, w, h);
    let covered: Vec<Vec<bool>> = shots
        .iter()
        .map(|s| {
            s.gray
                .iter()
                .zip(&t)
                .map(|(a, b)| a.abs_diff(*b) > DIFF)
                .collect()
        })
        .collect();
    let footer: Vec<bool> = (0..w * h).map(|i| covered.iter().all(|c| c[i])).collect();
    let mut zones = Vec::new();
    for (x, y, bw, bh) in boxes(&footer, w, h) {
        let side = if (x as usize + bw as usize / 2) < w / 2 {
            "left"
        } else {
            "right"
        };
        zones.push(SafeZone {
            name: format!("footer_{side}"),
            orientation: Orientation::Portrait,
            rect: (x, y, bw, bh),
            advisory: false,
        });
    }
    for (s, c) in shots.iter().zip(&covered) {
        let own: Vec<bool> = c.iter().zip(&footer).map(|(a, f)| *a && !*f).collect();
        let bs = boxes(&own, w, h);
        let Some(first) = bs.first() else {
            return Err(format!("nothing covered in the {} screenshot", s.position));
        };
        // One toolbar per screenshot: the box around all of its pieces.
        let (mut x0, mut y0, mut x1, mut y1) =
            (first.0, first.1, first.0 + first.2, first.1 + first.3);
        for b in &bs {
            (x0, y0, x1, y1) = (
                x0.min(b.0),
                y0.min(b.1),
                x1.max(b.0 + b.2),
                y1.max(b.1 + b.3),
            );
        }
        let floating = s.position == "float";
        zones.push(SafeZone {
            name: if floating {
                "toolbar_floating_default".into()
            } else {
                format!("toolbar_{}", s.position)
            },
            orientation: Orientation::Portrait,
            rect: (x0, y0, x1 - x0, y1 - y0),
            // A floating toolbar can be moved: shown, but linted as Info (napkin §3.5).
            advisory: floating,
        });
    }
    zones.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(zones)
}
