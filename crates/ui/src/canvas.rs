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

//! The two canvases (rung 3 bbq Q4, Q7). **Export pixels**, the default, shows the PNG's own
//! pixels as rung 2 did. **Graphics chip** draws the visible part of the page at screen
//! resolution with vello on the window's wgpu device, smooth at every zoom; it never makes an
//! export (ADR-003 item 3).
//!
//! The fallback rule (Defaulted "The chip canvas on the Pi will be slow"): if the chip canvas
//! cannot start, or a measured frame after the first takes over [`SLOW_FRAME_MS`], it draws with
//! vello_cpu at screen resolution instead and the status bar says so.
//!
//! The chip renderer itself lives in the app, which alone has the wgpu device; this crate sees it
//! only through [`ChipRenderer`], so it stays free of vello and runs headless.

use kurbo::{Affine, BezPath, Shape};
use quadrule_layout::{DisplayList, Item};

/// A chip frame slower than this switches the canvas to vello_cpu (milliseconds).
pub const SLOW_FRAME_MS: f64 = 100.0;

/// Which canvas shows the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasMode {
    /// The export's own pixels (vello_cpu at device resolution).
    ExportPixels,
    /// Vello on the graphics chip at screen resolution, or its vello_cpu fallback.
    Chip,
}

/// Draws a display list on the graphics chip into a texture egui can show.
pub trait ChipRenderer {
    /// Renders `dl` (already at screen resolution) and returns its texture and how long the
    /// frame took, GPU work included, in milliseconds.
    fn render(&mut self, dl: &DisplayList) -> Result<(egui::TextureId, f64), String>;
}

/// Why the chip canvas is drawing with vello_cpu.
#[derive(Clone, Debug, PartialEq)]
pub enum Fallback {
    /// No renderer, or it failed: the reason.
    CannotStart(String),
    /// A frame took this many milliseconds.
    Slow(f64),
}

/// What the chip canvas has done so far this session.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChipState {
    /// Set once the canvas has fallen back; it stays on vello_cpu for the session.
    pub fallback: Option<Fallback>,
    /// Chip frames drawn.
    pub frames: u32,
    /// The last chip frame's time in milliseconds.
    pub last_ms: Option<f64>,
    /// The last vello_cpu fallback frame's time in milliseconds.
    pub cpu_ms: Option<f64>,
}

impl ChipState {
    /// Records a chip frame's time; the first frame (shaders being built) is not judged.
    pub fn record_chip(&mut self, ms: f64) {
        self.frames += 1;
        self.last_ms = Some(ms);
        if self.frames > 1 && ms > SLOW_FRAME_MS {
            self.fallback = Some(Fallback::Slow(ms));
        }
    }
}

/// The part of the page on screen: `scale` screen pixels per device pixel, the visible area's
/// top-left corner in device pixels, and its size in screen pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenView {
    /// Physical screen pixels per device pixel.
    pub scale: f64,
    /// Left edge of the visible area, device pixels.
    pub x: f64,
    /// Top edge of the visible area, device pixels.
    pub y: f64,
    /// Visible width in screen pixels.
    pub width: u32,
    /// Visible height in screen pixels.
    pub height: u32,
}

/// `dl` moved and scaled so the visible area fills a `view.width` × `view.height` image at screen
/// resolution, every shape anti-aliased. Lines are no longer on whole pixels, which is right for a
/// smooth preview and why this is never used for export.
pub fn screen_list(dl: &DisplayList, view: &ScreenView) -> DisplayList {
    let s = view.scale;
    let tx = |x: f64| (x - view.x) * s;
    let ty = |y: f64| (y - view.y) * s;
    let affine = Affine::translate((-view.x, -view.y)).then_scale(s);
    let items = dl
        .items
        .iter()
        .map(|it| match it {
            // As an anti-aliased fill: the export's hard-edged rectangles would drop a line
            // thinner than half a screen pixel when zoomed out.
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } => Item::Fill {
                path: kurbo::Rect::new(tx(*x0), ty(*y0), tx(*x1), ty(*y1)).to_path(0.1),
                gray: *gray,
            },
            Item::Stroke {
                x0,
                y0,
                x1,
                y1,
                width,
                gray,
            } => Item::Stroke {
                x0: tx(*x0),
                y0: ty(*y0),
                x1: tx(*x1),
                y1: ty(*y1),
                width: width * s,
                gray: *gray,
            },
            Item::Fill { path, gray } => {
                let mut p: BezPath = path.clone();
                p.apply_affine(affine);
                Item::Fill {
                    path: p,
                    gray: *gray,
                }
            }
            Item::StrokePath {
                path,
                width,
                gray,
                cap,
            } => {
                let mut p: BezPath = path.clone();
                p.apply_affine(affine);
                Item::StrokePath {
                    path: p,
                    width: width * s,
                    gray: *gray,
                    cap: *cap,
                }
            }
        })
        .collect();
    DisplayList {
        width: view.width.max(1),
        height: view.height.max(1),
        background: dl.background,
        items,
    }
}

/// How long the frames of a drag took (rung 4 bbq, Defaulted "Pi-smooth, the number"): each frame
/// in which the page was made again, from the start of the canvas to its end, so the page's
/// layout and render, the texture and (on the chip canvas) the chip frame are all in it.
///
/// chris ruled (2026-11-10, "let's use a percentile") that the 50 ms line is judged by a
/// percentile of the frames, not by the worst one: [`DragClock::PERCENTILE`], chosen here for
/// chris to veto.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DragClock {
    /// Frames that made the page again.
    pub frames: u32,
    /// All their times added, in milliseconds.
    pub total_ms: f64,
    /// The first frame's time: it builds the shaders on the chip canvas, so it is reported but
    /// not judged (as in [`ChipState::record_chip`]).
    pub first_ms: f64,
    /// The slowest frame after the first, in milliseconds.
    pub worst_ms: f64,
    /// How many frames after the first took [`DragClock::BUDGET_MS`] or more.
    pub over_budget: u32,
    /// The times of the frames after the first, in the order they came.
    later_ms: Vec<f64>,
}

impl DragClock {
    /// The line a drag frame must stay under on the Pi, in milliseconds (20 frames a second).
    pub const BUDGET_MS: f64 = 50.0;

    /// The share of frames (after the first) that must be under the line: the 90th percentile
    /// (ADR-009, ratified by chris 2026-11-10).
    pub const PERCENTILE: f64 = 90.0;

    /// Adds one frame's time.
    pub fn record(&mut self, ms: f64) {
        if self.frames == 0 {
            self.first_ms = ms;
        } else {
            self.worst_ms = self.worst_ms.max(ms);
            if ms >= Self::BUDGET_MS {
                self.over_budget += 1;
            }
            self.later_ms.push(ms);
        }
        self.frames += 1;
        self.total_ms += ms;
    }

    /// The mean frame time, 0 before any frame.
    pub fn mean_ms(&self) -> f64 {
        if self.frames == 0 {
            0.0
        } else {
            self.total_ms / f64::from(self.frames)
        }
    }

    /// The [`DragClock::PERCENTILE`]th percentile of the frames after the first (nearest rank),
    /// in milliseconds; 0 before there are any.
    pub fn percentile_ms(&self) -> f64 {
        if self.later_ms.is_empty() {
            return 0.0;
        }
        let mut v = self.later_ms.clone();
        v.sort_by(f64::total_cmp);
        let rank = (Self::PERCENTILE / 100.0 * v.len() as f64).ceil() as usize;
        v[rank.clamp(1, v.len()) - 1]
    }

    /// Whether the percentile of the frames after the first is under [`DragClock::BUDGET_MS`].
    pub fn within_budget(&self) -> bool {
        !self.later_ms.is_empty() && self.percentile_ms() < Self::BUDGET_MS
    }
}
