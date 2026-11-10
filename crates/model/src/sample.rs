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

//! The rung 1 slice document (docs/bbq/2026-11-09-rung-1-headless-slice.md, Q2): a dot-grid
//! pattern region, ruled lines and one text box, designed on the Manta and also targeting the
//! Nomad. Shared by every crate's tests so they all check the same page.

use crate::*;

/// Primary profile of the slice document.
pub const MANTA_ID: &str = "supernote.a5x2";
/// Secondary profile of the slice document.
pub const NOMAD_ID: &str = "supernote.a6x2";

/// Manta page width in mm: 1920 px at 300 PPI (napkin §3.3).
pub const MANTA_W_MM: f64 = 1920.0 / 300.0 * MM_PER_IN;
/// Manta page height in mm: 2560 px at 300 PPI (napkin §3.3).
pub const MANTA_H_MM: f64 = 2560.0 / 300.0 * MM_PER_IN;

/// Fixed object ids, so the same document always gives the same file bytes.
pub mod ids {
    use crate::Uuid;
    /// The title text box.
    pub const TITLE: Uuid = Uuid::from_u128(0x5155_4144_0001_4000_8000_0000_0000_0001);
    /// The rule under the title.
    pub const RULE: Uuid = Uuid::from_u128(0x5155_4144_0001_4000_8000_0000_0000_0002);
    /// The dot grid.
    pub const DOTS: Uuid = Uuid::from_u128(0x5155_4144_0001_4000_8000_0000_0000_0003);
    /// The ruled lines.
    pub const LINES: Uuid = Uuid::from_u128(0x5155_4144_0001_4000_8000_0000_0000_0004);
}

/// The bundled font the slice's text uses (napkin §7.2).
pub const ATKINSON: &str = "Atkinson Hyperlegible Next";

/// The title's text: kerning pairs (To, Wa, Yo, Ar) and a curly apostrophe and em dash.
pub const TITLE_TEXT: &str = "Today’s Notes — Wave Your Arrows: a slice of text, dots and lines";

fn object(
    id: Uuid,
    name: &str,
    frame: Frame,
    constraints: Constraints,
    style: Style,
    kind: ObjectKind,
) -> Object {
    Object {
        id,
        name: name.into(),
        frame,
        constraints,
        visible: true,
        locked: false,
        style,
        kind,
        extra: Extra::new(),
    }
}

/// The slice document.
pub fn slice_document() -> Document {
    let margin = 8.0;
    let inner_w = MANTA_W_MM - 2.0 * margin;
    let pinned_top = Constraints {
        h: HConstraint::LeftRight,
        v: VConstraint::Top,
    };

    let title = object(
        ids::TITLE,
        "Title",
        Frame::new(margin, margin, inner_w, 11.0),
        pinned_top,
        Style::Role(LineRole::Text),
        ObjectKind::Text(TextBox {
            content: TITLE_TEXT.into(),
            family: ATKINSON.into(),
            weight: 700,
            size_pt: 12.0,
            line_height: 1.25,
            letter_spacing_em: 0.0,
            align: TextAlign::Left,
            extra: Extra::new(),
        }),
    );
    let rule = object(
        ids::RULE,
        "Title rule",
        Frame::new(margin, 22.0, inner_w, 0.0),
        pinned_top,
        Style::Role(LineRole::Structure),
        ObjectKind::Line(Line::default()),
    );
    let dots = object(
        ids::DOTS,
        "Dot grid",
        Frame::new(margin, 26.0, inner_w, 84.0),
        // Same physical size on every profile, under the pinned header.
        pinned_top,
        Style::Role(LineRole::DotGrid),
        ObjectKind::Pattern(PatternRegion {
            pattern: Pattern::Dots,
            extra: Extra::new(),
        }),
    );
    let lines = object(
        ids::LINES,
        "Ruled lines",
        Frame::new(margin, 116.0, inner_w, MANTA_H_MM - margin - 116.0),
        // Fills what is left down to the bottom margin: a smaller page gets fewer lines.
        Constraints {
            h: HConstraint::LeftRight,
            v: VConstraint::TopBottom,
        },
        Style::Role(LineRole::Guide),
        ObjectKind::Pattern(PatternRegion {
            pattern: Pattern::Lines,
            extra: Extra::new(),
        }),
    );

    Document {
        metadata: Metadata {
            title: "Rung 1 slice".into(),
            author: "quadrule".into(),
            license: "CC-BY-4.0".into(),
            description: "The rung 1 headless slice: dot grid, ruled lines and one text box."
                .into(),
            tags: vec!["test".into()],
            created: "2026-11-09T00:00:00Z".into(),
            modified: "2026-11-09T00:00:00Z".into(),
            extra: Extra::new(),
        },
        document_type: DocumentType::Template,
        primary_profile_id: MANTA_ID.into(),
        profiles: vec![
            ProfileContext {
                profile_id: MANTA_ID.into(),
                extra: Extra::new(),
            },
            ProfileContext {
                profile_id: NOMAD_ID.into(),
                extra: Extra::new(),
            },
        ],
        units: Unit::Mm,
        page: PageSettings {
            background_gray: 255,
            margins_mm: Margins::all(margin),
            extra: Extra::new(),
        },
        rhythm: Rhythm {
            h_unit_mm: 6.0,
            v_unit_mm: 6.0,
            origin: RhythmOrigin::MarginCorner,
            resolution: RhythmResolution::PixelLocked,
            subdivisions: None,
            extra: Extra::new(),
        },
        layers: vec![
            Layer {
                name: "Grid".into(),
                visible: true,
                locked: false,
                objects: vec![dots, lines],
                extra: Extra::new(),
            },
            Layer {
                name: "Header".into(),
                visible: true,
                locked: false,
                objects: vec![title, rule],
                extra: Extra::new(),
            },
        ],
        extra: Extra::new(),
    }
}
