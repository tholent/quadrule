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

//! Saved text outlines (napkin §7.3, ADR-001 items 1–3): one file per profile holding each
//! distinct letter shape once, plus each text box's letter positions in mm.
//!
//! The same structure is used for live text: the `text` crate builds it from the fonts, and
//! layout draws from it. Drawing saved text and live text therefore goes through one code path,
//! which is what lets the identity test (ADR-002 item 5) demand byte-identical PNGs.

use crate::{Extra, Uuid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Everything needed to draw one profile's text without its fonts (`outlines/<profile_id>.cbor`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileOutlines {
    /// The profile these were laid out for.
    pub profile_id: String,
    /// One table per font face (family + weight) the profile's text uses.
    pub tables: Vec<GlyphTable>,
    /// One entry per text object, in document order.
    pub texts: Vec<TextOutlines>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

impl ProfileOutlines {
    /// The laid-out text of one object, if present.
    pub fn text(&self, object_id: Uuid) -> Option<&TextOutlines> {
        self.texts.iter().find(|t| t.object_id == object_id)
    }
}

/// Which face a table holds.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FaceKey {
    /// Family name.
    pub family: String,
    /// CSS weight the outlines were drawn at.
    pub weight: u16,
}

/// Each distinct letter shape of one face, once (ADR-001 item 1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GlyphTable {
    /// The face.
    pub face: FaceKey,
    /// Font units per em, to scale shapes to a font size.
    pub units_per_em: u16,
    /// Glyph id → plain (unhinted) shape.
    pub glyphs: BTreeMap<u16, GlyphShape>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// One letter shape, in font units, y up (as fonts store it).
///
/// `verbs` is one ASCII letter per path command: `M` move, `L` line, `Q` quadratic, `C` cubic,
/// `Z` close. `coords` holds their points as half font units (a stored 3 means 1.5 font units),
/// so TrueType shapes, whose implied points fall on halves, are stored exactly. M and L take
/// one point (2 numbers), Q two, C three, Z none.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct GlyphShape {
    /// Path commands.
    pub verbs: String,
    /// Points, in half font units.
    pub coords: Vec<i16>,
}

impl GlyphShape {
    /// How many numbers each verb takes; `None` for an unknown verb.
    pub fn arity(verb: u8) -> Option<usize> {
        match verb {
            b'M' | b'L' => Some(2),
            b'Q' => Some(4),
            b'C' => Some(6),
            b'Z' => Some(0),
            _ => None,
        }
    }

    /// True when every verb is known and `coords` holds exactly the numbers they need.
    pub fn is_well_formed(&self) -> bool {
        let mut need = 0;
        for v in self.verbs.bytes() {
            match Self::arity(v) {
                Some(n) => need += n,
                None => return false,
            }
        }
        need == self.coords.len()
    }
}

/// One text object's letters as laid out on one profile.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TextOutlines {
    /// The text object.
    pub object_id: Uuid,
    /// Font size in mm (one style per box in v1).
    pub size_mm: f32,
    /// Height of all lines, in mm, from the box's top.
    pub height_mm: f32,
    /// Each drawn letter, in reading order.
    pub glyphs: Vec<PlacedGlyph>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// One letter: (index into `tables`, glyph id, x mm, y mm). The point is the baseline origin,
/// relative to the text box's top-left corner, y down. Stored as a 4-item array, not a map,
/// to keep files small.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PlacedGlyph(pub u16, pub u16, pub f32, pub f32);

impl PlacedGlyph {
    /// Index into the profile's `tables`.
    pub fn table(&self) -> usize {
        self.0 as usize
    }
    /// Glyph id within that table.
    pub fn glyph(&self) -> u16 {
        self.1
    }
    /// Baseline origin x, mm from the box's left edge.
    pub fn x_mm(&self) -> f32 {
        self.2
    }
    /// Baseline origin y, mm from the box's top edge.
    pub fn y_mm(&self) -> f32 {
        self.3
    }
}
