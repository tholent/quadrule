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

//! Quadrule's window parts (napkin §15.1). Rung 2 made a viewer (docs/bbq/2026-11-09-rung-2-viewer.md):
//! open a `.quad` or the sample page, show it exactly as it exports, switch devices, zoom to
//! pixels, show lint findings, export PNGs. Rung 3 makes it the first editor
//! (docs/bbq/2026-11-10-rung-3-first-editor.md): select, layers, inspector, nudge, undo/redo,
//! Save, autosave.
//!
//! [`viewer`], [`editor`] and [`recovery`] hold the state and run headless; [`view`] draws it
//! with egui.

#![forbid(unsafe_code)]

pub mod bench;
pub mod canvas;
pub mod drag;
pub mod draw;
pub mod editor;
pub mod panels;
pub mod recovery;
pub mod tools;
pub mod view;
pub mod viewer;

pub use bench::DragBench;
pub use canvas::{CanvasMode, ChipRenderer, ChipState, DragClock, Fallback};
pub use drag::{Drag, DragKind, DragMods, HandleHit};
pub use draw::{Draw, DrawKind, NewNames};
pub use editor::{DropOn, Toward, layer_drop_place, object_drop_place};
pub use panels::Tab;
pub use recovery::{Autosave, Recovered, Recovery};
pub use tools::Tool;
pub use view::{After, Request, UiFont, ViewerUi, install_fonts};
pub use viewer::{ExportWhich, Source, Viewer, Zoom};
