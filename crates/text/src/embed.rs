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

//! Whether a font may be saved inside a `.quad` (rung 12a bbq, Q3 and Defaulted "Embedding in
//! detail"; napkin §7.3 and §12.1: "only fonts whose licence allows embedding").
//!
//! The font's own licence bits are read from its OS/2 table (`fsType`) through skrifa. A font is
//! embedded when it is installable (0) or editable (0x0008) and not bitmap-only (0x0200). Fonts that
//! are restricted (0x0002), preview-and-print only (0x0004), bitmap-only, held in a collection
//! (`.ttc`), or that carry no OS/2 table (Type 1 and the like) are not embedded: their text travels
//! as saved outlines. Whole files are embedded, never subsets (ADR-002 item 1: no subsetter).

use cosmic_text::skrifa::{self, raw::TableProvider};

/// Why a font may not be embedded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoEmbed {
    /// The licence is restricted (`fsType` 0x0002).
    Restricted,
    /// The licence allows preview and print only (0x0004).
    PreviewAndPrint,
    /// The font holds bitmaps only (0x0200).
    BitmapOnly,
    /// The face lives in a font collection (`.ttc`/`.otc`), which cannot be written as one font file
    /// without a new dependency.
    Collection,
    /// The file carries no licence bits (no OS/2 table), or cannot be read.
    NoLicence,
}

impl NoEmbed {
    /// The Fluent id of the words for this reason (`lint.ftl`).
    pub fn message_id(self) -> &'static str {
        match self {
            NoEmbed::Restricted => "lint-l20-reason-restricted",
            NoEmbed::PreviewAndPrint => "lint-l20-reason-preview",
            NoEmbed::BitmapOnly => "lint-l20-reason-bitmap",
            NoEmbed::Collection => "lint-l20-reason-collection",
            NoEmbed::NoLicence => "lint-l20-reason-unknown",
        }
    }
}

/// The embedding rule on a font's `fsType` value.
pub fn verdict_for_fs_type(fs_type: u16) -> Result<(), NoEmbed> {
    if fs_type & 0x0200 != 0 {
        return Err(NoEmbed::BitmapOnly);
    }
    match fs_type & 0x000E {
        0 | 0x0008 => Ok(()),
        0x0004 => Err(NoEmbed::PreviewAndPrint),
        // 0x0002, and the combinations the specification does not allow.
        _ => Err(NoEmbed::Restricted),
    }
}

/// The embedding rule on a font file's bytes.
pub fn verdict(data: &[u8]) -> Result<(), NoEmbed> {
    if data.starts_with(b"ttcf") {
        return Err(NoEmbed::Collection);
    }
    let Ok(f) = skrifa::FontRef::new(data) else {
        return Err(NoEmbed::NoLicence);
    };
    match f.os2() {
        Ok(os2) => verdict_for_fs_type(os2.fs_type()),
        Err(_) => Err(NoEmbed::NoLicence),
    }
}
