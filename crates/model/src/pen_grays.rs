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

//! The gray scale built on the Supernote pen grays (ADR-027). The tablets' 16 steps move onto the black, dark grey and
//! light grey pens' grays (1, 157, 201), the page white stays 255, and the other steps are spaced evenly between them.
//! Behind `--pen-grays` until chris's look (ADR-007, amendment of 2026-11-10): with the switch off nothing changes.
//! One switch for the whole process, set once at start by the app and by `cargo xtask`, before any profile is read.

use std::sync::atomic::{AtomicBool, Ordering};

static ON: AtomicBool = AtomicBool::new(false);

/// The old scale: even steps of 17 from 0 to 255.
pub const OLD_STEPS: [u8; 16] = [
    0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255,
];

/// The new scale (ADR-027 item 2): each entry replaces the old step at the same place.
pub const NEW_STEPS: [u8; 16] = [
    1, 18, 36, 53, 70, 88, 105, 122, 140, 157, 172, 186, 201, 219, 237, 255,
];

/// Turns the pen-gray scale on or off for this process.
pub fn set(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// Whether the pen-gray scale is on (`--pen-grays`).
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Where gray `g` goes on the new scale: an old step moves to the new step in its place; any other gray stays as it is
/// (ADR-027 item 5, chris: "move only on-step grays").
pub fn moved(g: u8) -> u8 {
    OLD_STEPS
        .iter()
        .position(|&s| s == g)
        .map_or(g, |i| NEW_STEPS[i])
}
