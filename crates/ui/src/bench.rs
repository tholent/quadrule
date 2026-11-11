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
        }
    }

    /// The same benchmark over the shapes page, which holds every new kind.
    pub fn with_shapes(self) -> Self {
        DragBench {
            shapes: true,
            ..self
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
