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

//! The drag benchmark's state (rung 4 bbq, Defaulted "Pi-smooth, the number"). The driver is
//! [`ViewerUi`](crate::ViewerUi): it moves the sample page's dot grid along a path, one step per
//! frame, through the same canvas a person's drag uses, and prints one `CHECK` line. Rung 5 adds
//! the same for drawing: it drags a pattern region out with the Pattern tool. Rung 6a lets either
//! run over the shapes page ([`quadrule_model::sample::shapes_document`]) and the draw run use any
//! drawing tool, so a page holding the new kinds can be timed (ADR-009).

use crate::tools::Tool;

/// Frames to let the first full render and the shaders settle before the drag starts.
pub const WARM_FRAMES: u32 = 5;

/// Where the benchmark is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for the page to be drawn; counts frames.
    Warm(u32),
    /// About to press on the dot grid.
    Begin,
    /// Dragging; the number of steps made.
    Run(u32),
    /// Finished.
    Done,
}

/// A drag benchmark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragBench {
    /// How many pointer steps the drag makes.
    pub frames: u32,
    /// Where it is.
    pub phase: Phase,
    /// Where the drag started, in device pixels.
    pub centre: (f64, f64),
    /// Draws a pattern region with the Pattern tool instead of moving the dot grid (rung 5:
    /// `--bench-draw`).
    pub draw: bool,
    /// Opens the shapes page, not the sample page, when nothing is open yet (`--shapes-page`).
    pub shapes: bool,
    /// The tool the draw run drags out (`--bench-tool`); the Pattern tool unless told otherwise.
    pub tool: Tool,
    /// Opens the rung 6b transforms page (rotated shapes, lines every way, opacity) when nothing
    /// is open yet (`--transforms-page`).
    pub transforms: bool,
    /// Opens the rung 7a groups page (nested, turned and faded groups, a heavy group of dozens of
    /// shapes) when nothing is open yet (`--groups-page`); the drag then moves the heavy group.
    pub groups: bool,
    /// Turns the big rotated rectangle of the transforms page with its rotate handle (rung 6b:
    /// `--bench-rotate`), so the handle's own drag path is timed.
    pub rotate: bool,
    /// Opens the rung 8a overrides page when nothing is open yet (`--devices-page`).
    pub devices: bool,
    /// Opens the rung 8b landscape page when nothing is open yet (`--landscape-page`).
    pub landscape: bool,
    /// Opens the rung 9 pattern page when nothing is open yet (`--patterns-page`).
    pub patterns: bool,
    /// Opens the rung 12b turned-patterns page when nothing is open yet (`--turned-patterns-page`); the
    /// drag then moves its turned dot region, and the rotate run turns its 45 degree grid.
    pub turned_patterns: bool,
    /// Opens the pattern-borders page when nothing is open yet (`--bordered-page`); the drag then moves its
    /// bordered dot region.
    pub bordered: bool,
    /// Opens the rung 12b pattern-angle page when nothing is open yet (`--pattern-angle-page`); the
    /// drag then moves its dot region.
    pub pattern_angle: bool,
    /// Opens the rung 13b paper page when nothing is open yet (`--paper-page`); the drag then moves
    /// its dot region.
    pub paper: bool,
    /// Opens the rung 14a lint page when nothing is open yet (`--lint-page`); the drag then moves its
    /// dot grid.
    pub lint: bool,
    /// Opens the rung 10 repeat page when nothing is open yet (`--repeats-page`); the drag then
    /// moves its habit tracker.
    pub repeats: bool,
    /// Opens the rung 11a fonts page when nothing is open yet (`--fonts-page`); a page that is
    /// mostly text, whose dot grid the drag moves.
    pub fonts: bool,
    /// Opens the rung 11b text-box page when nothing is open yet (`--textbox-page`).
    pub textbox: bool,
    /// Opens the rung 12a missing-fonts page when nothing is open yet (`--missing-fonts-page`): the
    /// text-box page with every text box in a font this computer lacks, drawn from saved outlines.
    pub missing_fonts: bool,
    /// Zooms the page in and out like a pinch instead of dragging (fixes batch 1: `--bench-zoom`): each
    /// step feeds the canvas a zoom factor and a pointer, through the path a pinch takes, and the run is
    /// timed like a drag.
    pub zoom: bool,
    /// Types into a long text box instead of dragging (rung 11b: `--bench-type`): a box of a page's
    /// worth of text is made, a letter is typed into it at each step (a new paragraph now and then),
    /// and the run is timed like a drag. The box is taken away again after.
    pub typing: bool,
    /// Runs on the document's first secondary device, not the primary one (`--bench-secondary`), so
    /// a drag writes overrides (rung 8a).
    pub secondary: bool,
    /// Folds the previews strip shut for the run (`--no-strip`); it is open otherwise, in a bench
    /// whatever the settings say (rung 8a, Defaulted "Speed").
    pub no_strip: bool,
}

impl DragBench {
    /// A benchmark of `frames` steps.
    pub fn new(frames: u32) -> Self {
        DragBench {
            frames,
            phase: Phase::Warm(0),
            centre: (0.0, 0.0),
            draw: false,
            shapes: false,
            tool: Tool::Pattern,
            transforms: false,
            groups: false,
            rotate: false,
            zoom: false,
            devices: false,
            landscape: false,
            patterns: false,
            turned_patterns: false,
            pattern_angle: false,
            bordered: false,
            lint: false,
            paper: false,
            repeats: false,
            fonts: false,
            textbox: false,
            missing_fonts: false,
            typing: false,
            secondary: false,
            no_strip: false,
        }
    }

    /// The same benchmark over the rung 8a overrides page.
    pub fn with_devices(self) -> Self {
        DragBench {
            devices: true,
            ..self
        }
    }

    /// The same benchmark over the rung 9 pattern page.
    pub fn with_patterns(self) -> Self {
        DragBench {
            patterns: true,
            ..self
        }
    }

    /// The same benchmark over the rung 12b turned-patterns page.
    pub fn with_turned_patterns(self) -> Self {
        DragBench {
            turned_patterns: true,
            ..self
        }
    }

    /// The same benchmark over the rung 13b paper page.
    pub fn with_paper(self) -> Self {
        DragBench {
            paper: true,
            ..self
        }
    }

    /// The same benchmark over the rung 14a lint page.
    pub fn with_lint(self) -> Self {
        DragBench { lint: true, ..self }
    }

    /// The same benchmark over the pattern-borders page.
    pub fn with_bordered(self) -> Self {
        DragBench {
            bordered: true,
            ..self
        }
    }

    /// The same benchmark over the rung 12b pattern-angle page.
    pub fn with_pattern_angle(self) -> Self {
        DragBench {
            pattern_angle: true,
            ..self
        }
    }

    /// The same benchmark over the rung 10 repeat page: the drag moves the habit tracker.
    pub fn with_repeats(self) -> Self {
        DragBench {
            repeats: true,
            ..self
        }
    }

    /// The same benchmark over the rung 11a fonts page.
    pub fn with_fonts(self) -> Self {
        DragBench {
            fonts: true,
            ..self
        }
    }

    /// The same benchmark over the rung 11b text-box page.
    pub fn with_textbox(self) -> Self {
        DragBench {
            textbox: true,
            ..self
        }
    }

    /// The same benchmark over the rung 12a missing-fonts page.
    pub fn with_missing_fonts(self) -> Self {
        DragBench {
            missing_fonts: true,
            ..self
        }
    }

    /// The same benchmark over the rung 8b landscape page.
    pub fn with_landscape(self) -> Self {
        DragBench {
            landscape: true,
            ..self
        }
    }

    /// The same benchmark on the first secondary device: a drag then writes overrides there.
    pub fn on_secondary(self) -> Self {
        DragBench {
            secondary: true,
            ..self
        }
    }

    /// The same benchmark with the previews strip folded shut.
    pub fn without_strip(self) -> Self {
        DragBench {
            no_strip: true,
            ..self
        }
    }

    /// The same benchmark over the shapes page, which holds every new kind.
    pub fn with_shapes(self) -> Self {
        DragBench {
            shapes: true,
            ..self
        }
    }

    /// The same benchmark over the transforms page, which holds rotated objects, lines pointing
    /// every way and opacity (rung 6b).
    pub fn with_transforms(self) -> Self {
        DragBench {
            transforms: true,
            ..self
        }
    }

    /// The same benchmark over the groups page (rung 7a): the drag moves its heavy group.
    pub fn with_groups(self) -> Self {
        DragBench {
            groups: true,
            ..self
        }
    }

    /// A benchmark of `frames` steps that zooms the page between a quarter and twelve times, twice over
    /// (fixes batch 1). [`DragBench::with_groups`] and the other `with_` changes pick the page.
    pub fn zoom(frames: u32) -> Self {
        DragBench {
            zoom: true,
            ..DragBench::new(frames)
        }
    }

    /// A benchmark of `frames` steps that types into a long text box (rung 11b).
    pub fn typing(frames: u32) -> Self {
        DragBench {
            typing: true,
            ..DragBench::new(frames)
        }
    }

    /// A benchmark of `frames` steps that turns the transforms page's big rotated rectangle with
    /// its rotate handle, sweeping the pointer round it, and lets go (then undoes it).
    pub fn rotate(frames: u32) -> Self {
        DragBench {
            rotate: true,
            transforms: true,
            ..DragBench::new(frames)
        }
    }

    /// The same drawing benchmark with another tool (a rectangle, ellipse or polygon).
    pub fn with_tool(self, tool: Tool) -> Self {
        DragBench { tool, ..self }
    }

    /// A benchmark of `frames` steps that drags a pattern region out with the Pattern tool, over
    /// the sample page, and lets it go (the object is then undone). [`DragBench::with_tool`] and
    /// [`DragBench::with_shapes`] change the tool and the page.
    pub fn draw(frames: u32) -> Self {
        DragBench {
            draw: true,
            ..DragBench::new(frames)
        }
    }
}

/// The scale the zoom bench has reached after `step` of `frames` steps (fixes batch 1): it swings
/// between a quarter and twelve screen pixels per device pixel, twice over, on a cosine so the factor
/// a step gives changes smoothly (the biggest is about 6 % a step at 150 steps).
pub fn zoom_target(step: u32, frames: u32) -> f32 {
    let (lo, hi) = (0.25_f64.ln(), 12.0_f64.ln());
    let phase = f64::from(step) / f64::from(frames.max(1));
    let mix = 0.5 - 0.5 * (4.0 * std::f64::consts::PI * phase).cos();
    (lo + (hi - lo) * mix).exp() as f32
}
