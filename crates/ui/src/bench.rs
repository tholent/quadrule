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
    /// Opens the rung 10 repeat page when nothing is open yet (`--repeats-page`); the drag then
    /// moves its habit tracker.
    pub repeats: bool,
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
            devices: false,
            landscape: false,
            patterns: false,
            repeats: false,
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

    /// The same benchmark over the rung 10 repeat page: the drag moves the habit tracker.
    pub fn with_repeats(self) -> Self {
        DragBench {
            repeats: true,
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
