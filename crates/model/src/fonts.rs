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

//! What the file remembers about fonts (rung 12a, docs/bbq/2026-11-16-rung-12-missing-fonts.md):
//! the measured signature of a font (ADR-001 item 6), which a later computer uses to find a good
//! substitute when the font itself is missing, and the document's choices about fonts (format 0.13).

use crate::Extra;
use serde::{Deserialize, Serialize};

/// The kind of a typeface, found by measurement first, then the font's own labels, then the words
/// in its name (ADR-002 item 7).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontClass {
    /// Letters with serifs.
    Serif,
    /// Plain strokes, no serifs.
    Sans,
    /// Every letter the same width.
    Mono,
    /// Handwriting or brush lettering.
    Script,
}

/// A small measured record of a font (ADR-001 item 6), kept in the manifest for every font a
/// document uses so that a computer without the font can still pick a close substitute.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FontSignature {
    /// Serif, sans, mono or script.
    pub class: FontClass,
    /// The weight of the face measured, 100 to 900 (400 regular).
    pub weight: u16,
    /// How wide the letters are against normal, in percent (100 normal, 75 condensed).
    pub width: u16,
    /// True when the measured face is italic or oblique.
    pub italic: bool,
    /// The average width of the lowercase letters, in thousandths of the em.
    pub avg_width: u16,
    /// The height of a lowercase x, in thousandths of the em.
    pub x_height: u16,
    /// True when every letter measured has the same width.
    pub mono: bool,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The document's choices about fonts (format 0.13; rung 12a bbq, Q3, Q5, Q7 and the Defaulted
/// "Remember for this document"). Left out of the file when it holds the defaults.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FontSettings {
    /// "Embed fonts": installed fonts whose licence allows it are saved inside the file. On by default.
    #[serde(default = "yes")]
    pub embed: bool,
    /// "Don't warn again for this document": the sheet on Save, Save As and Export is not shown.
    #[serde(default)]
    pub dont_warn: bool,
    /// The families chris chose to keep as saved outlines ("Remember for this document"): the bar
    /// does not name them on the next open.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keep_outlines: Vec<String>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

fn yes() -> bool {
    true
}

impl Default for FontSettings {
    fn default() -> Self {
        FontSettings {
            embed: true,
            dont_warn: false,
            keep_outlines: Vec::new(),
            extra: Extra::new(),
        }
    }
}

impl FontSettings {
    /// Whether everything holds its default; such a record is left out of the file.
    pub fn is_default(&self) -> bool {
        *self == FontSettings::default()
    }

    /// True when `family` was kept as saved outlines.
    pub fn keeps(&self, family: &str) -> bool {
        self.keep_outlines.iter().any(|f| f == family)
    }
}
