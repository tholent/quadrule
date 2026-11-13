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

//! A new blank document (rung 5, docs/bbq/2026-11-10-rung-5-new-and-insert.md, Defaulted
//! "New dialog fields"): a page with a rhythm, margins and one empty layer, and nothing on it.
//! The margins are worked out in `quadrule-layout` (it knows each device's safe zones); this
//! module only builds the document from the choices.

use crate::*;

/// The name of the one layer a new document starts with.
pub const FIRST_LAYER_NAME: &str = "Layer 1";

/// A rhythm to start from (napkin §6.3, "Suggest rhythm": narrow, college and wide ruled, 5 mm
/// grid and 1/4 in grid, and the 6 mm handwriting default). The inch values are US ruled-paper
/// sizes (the agent's choice, for chris to veto).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RhythmPreset {
    /// 6 mm, roughly college ruled: the napkin's default for a blank page.
    Handwriting,
    /// Narrow ruled, 1/4 in.
    NarrowRuled,
    /// College ruled, 9/32 in.
    CollegeRuled,
    /// Wide ruled, 11/32 in.
    WideRuled,
    /// A 5 mm grid.
    Grid5mm,
    /// A 1/4 in grid.
    GridQuarterInch,
    /// An isometric rhythm with a 5 mm side (rung 9 bbq, Defaulted "New… presets").
    Isometric5mm,
}

impl RhythmPreset {
    /// Every preset, the default first.
    pub const ALL: [RhythmPreset; 7] = [
        RhythmPreset::Handwriting,
        RhythmPreset::NarrowRuled,
        RhythmPreset::CollegeRuled,
        RhythmPreset::WideRuled,
        RhythmPreset::Grid5mm,
        RhythmPreset::GridQuarterInch,
        RhythmPreset::Isometric5mm,
    ];

    /// The unit in mm; the same horizontally and vertically. For the isometric preset, its side.
    pub fn unit_mm(self) -> f64 {
        match self {
            RhythmPreset::Handwriting => 6.0,
            RhythmPreset::NarrowRuled | RhythmPreset::GridQuarterInch => 0.25 * MM_PER_IN,
            RhythmPreset::CollegeRuled => 9.0 / 32.0 * MM_PER_IN,
            RhythmPreset::WideRuled => 11.0 / 32.0 * MM_PER_IN,
            RhythmPreset::Grid5mm | RhythmPreset::Isometric5mm => 5.0,
        }
    }

    /// Whether the preset is an isometric rhythm.
    pub fn is_isometric(self) -> bool {
        self == RhythmPreset::Isometric5mm
    }

    /// The steps snapping uses, in mm (across, down): the unit twice over, or for the isometric
    /// preset half a side across and one row down.
    pub fn steps_mm(self) -> (f64, f64) {
        let u = self.unit_mm();
        if self.is_isometric() {
            (u / 2.0, u * HALF_SQRT_3)
        } else {
            (u, u)
        }
    }

    /// The Fluent message that names it (`crates/i18n/locales/en-US/app.ftl`).
    pub fn message_id(self) -> &'static str {
        match self {
            RhythmPreset::Handwriting => "new-rhythm-handwriting",
            RhythmPreset::NarrowRuled => "new-rhythm-narrow",
            RhythmPreset::CollegeRuled => "new-rhythm-college",
            RhythmPreset::WideRuled => "new-rhythm-wide",
            RhythmPreset::Grid5mm => "new-rhythm-grid-5mm",
            RhythmPreset::GridQuarterInch => "new-rhythm-grid-quarter-inch",
            RhythmPreset::Isometric5mm => "new-rhythm-isometric-5mm",
        }
    }
}

/// A blank template for `primary_profile_id`, also targeting `other_profile_ids`: no objects,
/// one empty layer, the rhythm at `unit_mm` both ways from the top-left margin corner and
/// pixel-locked (napkin §6.3 defaults), margins as given, a white page. `now` is an RFC 3339
/// time for the created and modified dates.
pub fn blank_document(
    title: &str,
    primary_profile_id: &str,
    other_profile_ids: &[&str],
    unit_mm: f64,
    margins: Margins,
    now: &str,
) -> Document {
    let profile = |id: &str| ProfileContext {
        profile_id: id.into(),
        extra: Extra::new(),
    };
    Document {
        metadata: Metadata {
            title: title.into(),
            author: String::new(),
            license: String::new(),
            description: String::new(),
            tags: Vec::new(),
            created: now.into(),
            modified: now.into(),
            extra: Extra::new(),
        },
        document_type: DocumentType::Template,
        primary_profile_id: primary_profile_id.into(),
        profiles: std::iter::once(primary_profile_id)
            .chain(other_profile_ids.iter().copied())
            .map(profile)
            .collect(),
        units: Unit::Mm,
        page: PageSettings {
            background_gray: 255,
            margins_mm: margins,
            orientation: Default::default(),
            extra: Extra::new(),
        },
        rhythm: Rhythm {
            h_unit_mm: unit_mm,
            v_unit_mm: unit_mm,
            origin: RhythmOrigin::MarginCorner,
            resolution: RhythmResolution::PixelLocked,
            subdivisions: None,
            rhythm_type: RhythmType::Rectangular,
            side_mm: None,
            major_every: None,
            extra: Extra::new(),
        },
        layers: vec![Layer {
            name: FIRST_LAYER_NAME.into(),
            visible: true,
            locked: false,
            objects: Vec::new(),
            extra: Extra::new(),
        }],
        extra: Extra::new(),
    }
}

/// An RFC 3339 UTC time (`2026-11-10T14:03:09Z`) for `secs` seconds after 1970-01-01.
pub fn rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from a day count (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

/// The time now, as [`rfc3339_utc`] writes it.
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    rfc3339_utc(secs)
}
