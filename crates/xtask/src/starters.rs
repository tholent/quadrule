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

//! `cargo xtask starters`: builds every starter page in code and writes
//! `crates/ui/starters/<category>/<slug>.quad` (rung 16b bbq, Defaulted "How starters are made"). The files are
//! the same bytes every time (fixed dates, fixed ids, fixed order), and `tests/starters.rs` fails when a checked-in
//! file differs from what is written here, so a format change re-makes every starter and none drifts.
//!
//! To add a starter: write one function that returns a [`Starter`] (copy the nearest one), and add it to [`all`].
//! Frames are written in device pixels of the Nomad (the design device; both tablets are 300 PPI, so a pixel is the
//! same length on each), through [`px`]. Every frame uses constraints, so the page adapts to any tablet.
//!
//! How the frames are chosen (chris, rung 16b Q3: "Constraints now"): a pattern is laid out on the rhythm, and each
//! tablet centres the rhythm in the margin box differently, so a frame edge lands on a rhythm line on one tablet
//! and between two on the other. Each frame edge is set so that, on both known tablets, it lands on a line or leaves
//! a clearly part cell, never a sliver a millimetre or so tall that reads as a double line (rung 16b review: the
//! square grid on the Nomad). The numbers are worked out in each starter's own comment.

use quadrule_format::{PREVIEW_SIDE, QuadFile, preview_path};
use quadrule_layout::{display_list, lay_out_text};
use quadrule_model::*;
use quadrule_render::{Gray, encode_png, render, shrink, turn_clockwise};
use quadrule_text::FontRegistry;
use std::path::{Path, PathBuf};

/// The design device, which every frame below is measured on.
const NOMAD: &str = "supernote.a6x2";
/// The other tablet every starter targets.
const MANTA: &str = "supernote.a5x2";
/// The Nomad's page in pixels (1404 by 1872 at 300 PPI).
const PAGE_W: i64 = 1404;
const PAGE_H: i64 = 1872;
/// What a starter's files say made them, and when (the same bytes on every run).
const SAVED_BY: &str = "quadrule starters";
const WHEN: &str = "2026-11-24T12:00:00Z";

/// One starter page: the folder it goes in (a category), its file name, and its document.
pub struct Starter {
    /// The category folder under `crates/ui/starters/`: `paper`, `notes`, `planning`, `bullet-journal`, `creative`.
    pub category: &'static str,
    /// The file name without `.quad`, in hyphens.
    pub slug: &'static str,
    /// The page.
    pub doc: Document,
}

/// Every starter, in the order the files are written.
pub fn all() -> Vec<Starter> {
    vec![
        narrow_ruled(),
        narrow_ruled_rtl(),
        square_grid(),
        hex_grid(),
        isometric_grid(),
        cornell(),
        rapid_log(),
        college_ruled(),
        wide_ruled(),
        dot_grid(),
        blank_with_margins(),
        music_staff(),
        meeting_notes(),
        lecture_notes(),
        daily_planner(),
        to_do_list(),
        habit_tracker(),
        index(),
        storyboard(),
        handwriting_practice(),
    ]
}

/// The folder the starters are written to: `crates/ui/starters/`.
pub fn starters_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/starters")
}

/// A length in Nomad pixels as millimetres.
fn px(n: i64) -> f64 {
    n as f64 / 300.0 * MM_PER_IN
}

/// A fixed id: the same object has the same id on every run (`starter` numbers the page, `n` its object).
fn id(starter: u16, n: u16) -> Uuid {
    Uuid::from_u128(
        0x5155_4144_5354_4000_8000_0000_0000_0000 | (u128::from(starter) << 16) | u128::from(n),
    )
}

/// What a starter chooses; the rest is the same on every starter (Template, white, both Supernote tablets, the
/// Nomad first, one layer named Paper holding `objects`, and the page's margins, spacing and title marked Show in
/// Basic).
struct Page {
    title: &'static str,
    description: &'static str,
    tags: &'static [&'static str],
    margins: Margins,
    rhythm: Rhythm,
    objects: Vec<Object>,
}

impl Page {
    fn document(self) -> Document {
        Document {
            metadata: Metadata {
                title: self.title.into(),
                author: String::new(),
                license: String::new(),
                description: self.description.into(),
                tags: self.tags.iter().map(|t| (*t).to_string()).collect(),
                created: WHEN.into(),
                modified: WHEN.into(),
                extra: Extra::new(),
            },
            document_type: DocumentType::Template,
            primary_profile_id: NOMAD.into(),
            profiles: vec![ProfileContext::new(NOMAD), ProfileContext::new(MANTA)],
            units: Unit::Mm,
            page: PageSettings {
                background_gray: 255,
                margins_mm: self.margins,
                orientation: Orientation::Portrait,
                fonts: Default::default(),
                // Defaulted 7: every starter shows its margins, its spacing and its title in Basic.
                basic: BasicPage {
                    margins: true,
                    rhythm: true,
                    title: true,
                },
                extra: Extra::new(),
            },
            rhythm: self.rhythm,
            layers: vec![Layer {
                name: "Paper".into(),
                visible: true,
                locked: false,
                objects: self.objects,
                extra: Extra::new(),
            }],
            ignored_rules: Vec::new(),
            extra: Extra::new(),
        }
    }
}

/// Square units of `unit_mm`, centred in the margin box and pixel-locked (the stock grid's placement).
fn square_rhythm(unit_mm: f64) -> Rhythm {
    Rhythm {
        h_unit_mm: unit_mm,
        v_unit_mm: unit_mm,
        origin: RhythmOrigin::Centered,
        resolution: RhythmResolution::PixelLocked,
        subdivisions: None,
        rhythm_type: RhythmType::Rectangular,
        side_mm: None,
        major_every: None,
        extra: Extra::new(),
    }
}

/// An isometric rhythm of triangles `side_mm` a side, centred in the margin box.
fn isometric_rhythm(side_mm: f64) -> Rhythm {
    Rhythm {
        rhythm_type: RhythmType::Isometric,
        side_mm: Some(side_mm),
        ..square_rhythm(5.0)
    }
}

/// A frame in Nomad pixels: left, top, width, height.
fn frame(x: i64, y: i64, w: i64, h: i64) -> Frame {
    Frame::new(px(x), px(y), px(w), px(h))
}

/// An object on the page, Full bleed (chris's seven all run under the toolbars and footers), marked Show in Basic.
fn object(
    starter: u16,
    n: u16,
    name: &str,
    frame: Frame,
    constraints: (HConstraint, VConstraint),
    style: Style,
    kind: ObjectKind,
) -> Object {
    Object {
        id: id(starter, n),
        name: name.into(),
        frame,
        constraints: Constraints {
            h: constraints.0,
            v: constraints.1,
        },
        visible: true,
        locked: false,
        style,
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        free_placement: false,
        full_bleed: true,
        show_in_basic: true,
        kind,
        extra: Extra::new(),
    }
}

/// A pattern region that is `frame` on the page, stretched to it on both axes.
fn pattern(
    starter: u16,
    n: u16,
    name: &str,
    frame: Frame,
    style: LineRole,
    region: PatternRegion,
) -> Object {
    object(
        starter,
        n,
        name,
        frame,
        (HConstraint::LeftRight, VConstraint::TopBottom),
        Style::Role(style),
        ObjectKind::Pattern(region),
    )
}

/// A line along `frame` (a vertical line has no width, a horizontal one no height).
fn line(
    starter: u16,
    n: u16,
    name: &str,
    frame: Frame,
    constraints: (HConstraint, VConstraint),
    role: LineRole,
) -> Object {
    object(
        starter,
        n,
        name,
        frame,
        constraints,
        Style::Role(role),
        ObjectKind::Line(Line::default()),
    )
}

/// A region with the border on all four sides in the Structure role.
fn bordered(mut region: PatternRegion) -> PatternRegion {
    region.border = Some(Border::new());
    region
}

/// A region with a major line every `n`, counted from the middle of the region on both axes (ADR-023).
fn major_every_from_centre(mut region: PatternRegion, n: u8) -> PatternRegion {
    region.major = MajorLines::Every(n);
    region.major_rows_from = MajorFrom::Centre;
    region.major_columns_from = MajorFrom::Centre;
    region
}

// ---------------------------------------------------------------------------------------------------------------
// The starters. Each has the rhythm's origin worked out for both tablets (the Nomad then the Manta), in pixels:
// the rhythm is centred in the margin box, so on a page with margin `m` and unit `u`, `n = floor(box / u)` lines
// are spread over `(n - 1) * u`, and the leftover is split in half before the first line.
// ---------------------------------------------------------------------------------------------------------------

/// Paper: ruled lines a quarter inch apart and a margin line at the left.
///
/// Margins 12.7 mm (150 px), unit 6.35 mm (75 px). Rows start at y 223 on the Nomad and 192 on the Manta. The lines
/// start at y 148 (a row on the Nomad; the first row, 192, on the Manta) and run to the page's bottom edge. The
/// margin line is on the Nomad's first column (x 214) and Scales, so it sits about 15 % across on any tablet
/// (292.6 px on the Manta; chris's hand-set 285 px was a column there).
fn narrow_ruled() -> Starter {
    const N: u16 = 1;
    Starter {
        category: "paper",
        slug: "narrow-ruled",
        doc: Page {
            title: "Narrow ruled",
            description: "Ruled lines a quarter inch apart with a margin line at the left, for small handwriting.",
            tags: &["ruled", "lined", "narrow", "notebook", "margin"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(0.25 * MM_PER_IN),
            objects: vec![
                pattern(
                    N,
                    1,
                    "Lines",
                    frame(0, 148, PAGE_W, PAGE_H - 148),
                    LineRole::Guide,
                    PatternRegion::new(Pattern::Lines),
                ),
                line(
                    N,
                    2,
                    "Margin",
                    frame(214, 0, 0, PAGE_H),
                    (HConstraint::Scale, VConstraint::Scale),
                    LineRole::Margin,
                ),
            ],
        }
        .document(),
    }
}

/// Paper: the narrow ruled page for writing right to left: the margin line is on the Nomad's last column (x 1189).
fn narrow_ruled_rtl() -> Starter {
    const N: u16 = 2;
    Starter {
        category: "paper",
        slug: "narrow-ruled-rtl",
        doc: Page {
            title: "Narrow ruled, right-to-left",
            description: "The narrow ruled page with its margin line at the right, for writing from right to left.",
            tags: &["ruled", "lined", "narrow", "right-to-left", "rtl", "arabic", "hebrew"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(0.25 * MM_PER_IN),
            objects: vec![
                pattern(
                    N,
                    1,
                    "Lines",
                    frame(0, 148, PAGE_W, PAGE_H - 148),
                    LineRole::Guide,
                    PatternRegion::new(Pattern::Lines),
                ),
                line(
                    N,
                    2,
                    "Margin",
                    frame(1189, 0, 0, PAGE_H),
                    (HConstraint::Scale, VConstraint::Scale),
                    LineRole::Margin,
                ),
            ],
        }
        .document(),
    }
}

/// Paper: a 6 mm square grid with a heavier line every fourth, in a border.
///
/// Margins 10 mm (118 px), unit 6 mm (71 px). Rows start at y 155 on the Nomad and 179 on the Manta. The frame runs
/// from y 108 to 109 px above the bottom: on the Manta both edges are on a row (179 = 108 + 71 and the last row is
/// 109 px up), on the Nomad each edge leaves a part cell of 47 and 46 px (4 mm of 6), so neither tablet has a
/// sliver. (chris's frame ran to the page's bottom edge, 13 px under the Nomad's last row.) Across, the frame is the
/// page's width: the columns are centred, leaving 27 and 28 px (Nomad) and 37 and 37 px (Manta) at the edges.
fn square_grid() -> Starter {
    const N: u16 = 3;
    Starter {
        category: "paper",
        slug: "square-grid",
        doc: Page {
            title: "Square grid",
            description: "A square grid, 6 mm a side, with a heavier line every fourth one and a border.",
            tags: &["grid", "graph", "squares", "math", "6 mm"],
            margins: Margins::all(10.0),
            rhythm: square_rhythm(6.0),
            objects: vec![pattern(
                N,
                1,
                "Grid",
                frame(0, 108, PAGE_W, PAGE_H - 108 - 109),
                LineRole::Guide,
                bordered(major_every_from_centre(PatternRegion::new(Pattern::Grid), 4)),
            )],
        }
        .document(),
    }
}

/// Paper: a honeycomb of flat-topped hexagons 6 mm across the flats, in a border.
///
/// Margins 10 mm, unit 6 mm (71 px): a hexagon is 71 px tall, and its horizontal edges fall every 35.5 px (the
/// two columns of hexagons are half a hexagon apart). Edges are at y 155 + 35.5 m on the Nomad and 179 + 35.5 m on
/// the Manta. The frame runs from y 120 to 119 px above the bottom: on the Nomad both edges are on an edge of the
/// odd columns, on the Manta each leaves a 24 px strip (measured from the goldens), so no horizontal edge sits a few pixels from the border.
fn hex_grid() -> Starter {
    const N: u16 = 4;
    Starter {
        category: "paper",
        slug: "hex",
        doc: Page {
            title: "Hex grid",
            description: "A honeycomb of hexagons, 6 mm across the flats, inside a border.",
            tags: &["hex", "hexagon", "honeycomb", "grid", "games", "chemistry"],
            margins: Margins::all(10.0),
            rhythm: square_rhythm(6.0),
            objects: vec![pattern(
                N,
                1,
                "Hex grid",
                frame(0, 120, PAGE_W, PAGE_H - 120 - 119),
                LineRole::Guide,
                bordered(PatternRegion::new(Pattern::Hex {
                    orientation: HexOrientation::Flat,
                    height_units: 1,
                })),
            )],
        }
        .document(),
    }
}

/// Paper: an isometric grid (6 mm triangles) with a heavier line every fourth one, in a border.
///
/// Side 6 mm: a column is 61 px, a half side 35 px. Margins 10 mm top and bottom and 12.99 mm (three columns) at
/// the sides. Columns are at x 61 k on the Nomad (the page's two edges are both columns) and 14 + 61 k on the Manta,
/// where chris's page-wide frame left a 14 and a 15 px sliver beside the border. The frame is inset 14 px at the
/// left and 15 at the right: on the Manta both edges are columns, on the Nomad each leaves a 47 px part column.
/// Down, the frame is 148 px from the top and the bottom (chris's ran past the page's bottom edge, so no bottom
/// border showed); no line is parallel to the top or bottom edge, so any height is clean.
fn isometric_grid() -> Starter {
    const N: u16 = 5;
    Starter {
        category: "paper",
        slug: "isometric",
        doc: Page {
            title: "Isometric grid",
            description: "An isometric grid of triangles with a heavier line every fourth one, for drawing in three dimensions.",
            tags: &["isometric", "triangle", "grid", "3d", "drafting", "technical drawing"],
            margins: Margins {
                top: 10.0,
                right: 12.990_381_056_766_578,
                bottom: 10.0,
                left: 12.990_381_056_766_578,
            },
            rhythm: isometric_rhythm(6.0),
            objects: vec![pattern(
                N,
                1,
                "Isometric grid",
                frame(14, 148, PAGE_W - 14 - 15, PAGE_H - 148 - 148),
                LineRole::Whisper,
                bordered(major_every_from_centre(
                    PatternRegion::new(Pattern::Isometric {
                        style: IsoStyle::Lines,
                    }),
                    4,
                )),
            )],
        }
        .document(),
    }
}

/// Notes: Cornell notes, a cue column of five columns at the left, ruled notes and a summary band of five rows.
///
/// Margins 12.7 mm (150 px), unit 6.35 mm (75 px). Rows start at y 223 on the Nomad and 192 on the Manta. The frame
/// runs from y 148 to 149 px above the bottom, which is 21 rows exactly on the Nomad (both edges on a row) and, on
/// the Manta, a 44 px part row at the top and at the bottom (chris's hand-set Manta frame began on a row at the top
/// but left a 33 px sliver at the bottom). The frame is the page's width, as chris made it.
fn cornell() -> Starter {
    const N: u16 = 6;
    Starter {
        category: "notes",
        slug: "cornell",
        doc: Page {
            title: "Cornell notes",
            description: "Cornell notes: a cue column at the left, a ruled notes area and a summary band at the bottom.",
            tags: &["cornell", "notes", "study", "lecture", "cue", "summary"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(0.25 * MM_PER_IN),
            objects: vec![pattern(
                N,
                1,
                "Cornell",
                frame(0, 148, PAGE_W, PAGE_H - 148 - 149),
                LineRole::Guide,
                bordered(PatternRegion::new(Pattern::Cornell {
                    cue_units: Some(5),
                    summary_units: Some(5),
                })),
            )],
        }
        .document(),
    }
}

/// Bullet journal: the rapid log, a header line over a dot grid.
///
/// Margins 12.7 mm, unit 6.35 mm (75 px). The dots fill the margin box across (columns x 214 to 1189 on the Nomad,
/// 14 columns with 214 and 215 px clear at the sides; 210 to 1710 on the Manta, 21 columns with 210 px clear each
/// side), so the field is centred and clear of the docked toolbars. Down, they run from y 100 to the page's bottom:
/// the first dot row is 148 on the Nomad and 117 on the Manta, and the header line (y 73) is above it on both. The
/// header spans the first and last dot column: x 214 to 1191 on the Nomad (2 px past the last dot), and 214 to 1707
/// on the Manta (4 px inside the first dot and 3 inside the last).
fn rapid_log() -> Starter {
    const N: u16 = 7;
    Starter {
        category: "bullet-journal",
        slug: "rapid-log",
        doc: Page {
            title: "Rapid log",
            description: "A dot grid with a header line, for the rapid log of a bullet journal.",
            tags: &["bullet journal", "rapid log", "dots", "dot grid", "journal"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(0.25 * MM_PER_IN),
            objects: vec![
                pattern(
                    N,
                    1,
                    "Dots",
                    frame(150, 100, PAGE_W - 150 - 150, PAGE_H - 100),
                    LineRole::DotGrid,
                    PatternRegion::new(Pattern::Dots),
                ),
                line(
                    N,
                    2,
                    "Header",
                    frame(214, 73, 1191 - 214, 0),
                    (HConstraint::LeftRight, VConstraint::Top),
                    LineRole::Guide,
                ),
            ],
        }
        .document(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Leg 3: the thirteen starters by the builder (rung 16b, Q4; every word, spacing and size is the builder's proposal
// for chris's look). Paper first.
// ---------------------------------------------------------------------------------------------------------------

/// Paper: ruled lines 9/32 inch (7.1 mm) apart with a margin line at the left, the common college notebook ruling.
///
/// Margins 12.7 mm (150 px), unit 9/32 in = 84 px. Rows start at y 222 on the Nomad and 230 on the Manta; columns at
/// x 198 and 204. The lines start at y 138, one row above the first (as the narrow ruled page's start one row above
/// its first) and run to the page's bottom edge. The margin line is on the Nomad's first column (x 198) and Scales.
fn college_ruled() -> Starter {
    const N: u16 = 8;
    Starter {
        category: "paper",
        slug: "college-ruled",
        doc: Page {
            title: "College ruled",
            description: "Ruled lines 7.1 mm (9/32 inch) apart with a margin line at the left, the common college notebook ruling.",
            tags: &["ruled", "lined", "college", "notebook", "margin", "7 mm"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(9.0 / 32.0 * MM_PER_IN),
            objects: vec![
                pattern(
                    N,
                    1,
                    "Lines",
                    frame(0, 138, PAGE_W, PAGE_H - 138),
                    LineRole::Guide,
                    PatternRegion::new(Pattern::Lines),
                ),
                line(
                    N,
                    2,
                    "Margin",
                    frame(198, 0, 0, PAGE_H),
                    (HConstraint::Scale, VConstraint::Scale),
                    LineRole::Margin,
                ),
            ],
        }
        .document(),
    }
}

/// Paper: ruled lines 11/32 inch (8.7 mm) apart with a margin line at the left, for larger handwriting.
///
/// Margins 12.7 mm, unit 11/32 in = 103 px. Rows start at y 215 on the Nomad and 250 on the Manta; columns at x 238
/// and 239. The lines start at y 112 (one row above the first) and run to the page's bottom edge; the margin line is on
/// the Nomad's first column (x 238) and Scales.
fn wide_ruled() -> Starter {
    const N: u16 = 9;
    Starter {
        category: "paper",
        slug: "wide-ruled",
        doc: Page {
            title: "Wide ruled",
            description: "Ruled lines 8.7 mm (11/32 inch) apart with a margin line at the left, for larger handwriting.",
            tags: &["ruled", "lined", "wide", "notebook", "margin", "8.7 mm"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(11.0 / 32.0 * MM_PER_IN),
            objects: vec![
                pattern(
                    N,
                    1,
                    "Lines",
                    frame(0, 112, PAGE_W, PAGE_H - 112),
                    LineRole::Guide,
                    PatternRegion::new(Pattern::Lines),
                ),
                line(
                    N,
                    2,
                    "Margin",
                    frame(238, 0, 0, PAGE_H),
                    (HConstraint::Scale, VConstraint::Scale),
                    LineRole::Margin,
                ),
            ],
        }
        .document(),
    }
}

/// Paper: a dot grid, dots 5 mm apart, with no header (the rapid log's plain cousin).
///
/// Margins 12.7 mm, unit 5 mm = 59 px. Dot rows start at y 198 on the Nomad and 188 on the Manta, columns at x 200 and
/// 193. As the rapid log's, the dots fill the margin box across (columns x 200 to 1203 on the Nomad, clear of the
/// docked toolbars) and run from y 100 (the first row is 139 on the Nomad and 129 on the Manta, below the top toolbar)
/// to the page's bottom.
fn dot_grid() -> Starter {
    const N: u16 = 10;
    Starter {
        category: "paper",
        slug: "dot-grid",
        doc: Page {
            title: "Dot grid",
            description: "A dot grid with the dots 5 mm apart and no header, for sketching, lettering and notes.",
            tags: &["dots", "dot grid", "5 mm", "sketch", "journal"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(5.0),
            objects: vec![pattern(
                N,
                1,
                "Dots",
                frame(150, 100, PAGE_W - 150 - 150, PAGE_H - 100),
                LineRole::DotGrid,
                PatternRegion::new(Pattern::Dots),
            )],
        }
        .document(),
    }
}

/// Paper: an empty writing area inside a faint box drawn on the margins.
///
/// The box is the margin box itself, 12.7 mm (150 px) in from each edge, in the Whisper role, and keeps those
/// distances on every tablet. It is not on the rhythm's lines (the rhythm is centred in the margin box, so its first
/// line is further in), so it is marked Free placement; it is clear of the toolbars and the footers on both tablets.
fn blank_with_margins() -> Starter {
    const N: u16 = 11;
    let mut margin_box = object(
        N,
        1,
        "Margin box",
        frame(150, 150, PAGE_W - 300, PAGE_H - 300),
        (HConstraint::LeftRight, VConstraint::TopBottom),
        Style::Role(LineRole::Whisper),
        ObjectKind::Rect(RectShape::default()),
    );
    margin_box.full_bleed = false;
    margin_box.free_placement = true;
    Starter {
        category: "paper",
        slug: "blank-with-margins",
        doc: Page {
            title: "Blank with margins",
            description: "An empty writing area inside a faint margin box.",
            tags: &["blank", "margins", "plain", "box", "empty"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(0.25 * MM_PER_IN),
            objects: vec![margin_box],
        }
        .document(),
    }
}

/// Creative: staves of five lines for writing music.
fn music_staff() -> Starter {
    const N: u16 = 12;
    Starter {
        category: "creative",
        slug: "music-staff",
        doc: Page {
            title: "Music staff",
            description: "Staves of five lines, one every 19 mm, for writing music.",
            tags: &["music", "staff", "stave", "notation", "manuscript"],
            margins: Margins::all(12.7),
            rhythm: square_rhythm(0.25 * MM_PER_IN),
            objects: vec![pattern(
                N,
                1,
                "Staves",
                frame(150, 148, PAGE_W - 300, PAGE_H - 148 - 150),
                LineRole::Guide,
                PatternRegion::new(Pattern::MusicStaff {
                    gap_mm: STAFF_GAP_DEFAULT_MM,
                    pitch_units: 3,
                    grand: false,
                    grand_space_units: None,
                }),
            )],
        }
        .document(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The form pages (meeting notes, lecture notes, daily planner, to-do list, habit tracker, index, storyboard,
// handwriting practice). They share one lattice, chosen so that every edge lands on a rhythm line on both tablets:
//
// - The rhythm unit is 86 px (7.28 mm). The Manta's page is 688 px taller (8 units) and 516 px wider (6 units) than
//   the Nomad's, so a distance measured from any edge is a whole number of units on both tablets.
// - The rhythm starts at the margin box's corner (not centred) and the margin box is a whole number of units on both
//   tablets: margins 162 px top and bottom and 143 px at the sides leave 18 by 13 units on the Nomad and 26 by 19 on
//   the Manta, so the rhythm's first and last lines are on the margin box's edges and the linter's L18 is quiet.
// - So a frame kept from the top or left edge (Top, Left), or from the bottom or right edge (Bottom, Right), or
//   both (TopBottom, LeftRight), has its edges on rhythm lines on both tablets: no part row at a border, and text
//   boxes whose baselines stand on rhythm lines stand on the same rows as the ruling on each tablet.
//
// Row `k` is at y 162 + 86 k on the Nomad (the Manta's rows are the same from the top), column `k` at x 143 + 86 k.
// ---------------------------------------------------------------------------------------------------------------

/// One rhythm step of the form pages, in pixels.
const U: i64 = 86;
/// The width of the form pages' margin box: 13 steps on the Nomad (19 on the Manta).
const FORM_W: i64 = 13 * U;

/// The y of rhythm row `k` on the form pages (from the top, on both tablets).
fn row(k: i64) -> i64 {
    162 + U * k
}

/// The x of rhythm column `k` on the form pages.
fn col(k: i64) -> i64 {
    143 + U * k
}

/// The margins that make the form pages' margin box whole steps on both tablets (see above).
fn form_margins() -> Margins {
    Margins {
        top: px(162),
        right: px(143),
        bottom: px(162),
        left: px(143),
    }
}

/// The form pages' rhythm: one step square, counted from the margin box's corner.
fn form_rhythm() -> Rhythm {
    Rhythm {
        origin: RhythmOrigin::MarginCorner,
        ..square_rhythm(px(U))
    }
}

const LEFT_TOP: (HConstraint, VConstraint) = (HConstraint::Left, VConstraint::Top);
const LEFT_BOTTOM: (HConstraint, VConstraint) = (HConstraint::Left, VConstraint::Bottom);
const RIGHT_TOP: (HConstraint, VConstraint) = (HConstraint::Right, VConstraint::Top);
const RIGHT_BOTTOM: (HConstraint, VConstraint) = (HConstraint::Right, VConstraint::Bottom);
const STRETCH: (HConstraint, VConstraint) = (HConstraint::LeftRight, VConstraint::TopBottom);
const STRETCH_TOP: (HConstraint, VConstraint) = (HConstraint::LeftRight, VConstraint::Top);
const STRETCH_BOTTOM: (HConstraint, VConstraint) = (HConstraint::LeftRight, VConstraint::Bottom);
const LEFT_STRETCH: (HConstraint, VConstraint) = (HConstraint::Left, VConstraint::TopBottom);
const RIGHT_STRETCH: (HConstraint, VConstraint) = (HConstraint::Right, VConstraint::TopBottom);

/// An object inside the margin box: it does not run under the toolbars, so it is not Full bleed.
fn inside(mut o: Object) -> Object {
    o.full_bleed = false;
    o
}

/// An object not marked Show in Basic (the page's labels and its minor lines).
fn quiet(mut o: Object) -> Object {
    o.show_in_basic = false;
    o
}

/// A pattern region inside the margin box, marked Show in Basic when `basic`.
fn region(
    starter: u16,
    n: u16,
    name: &str,
    frame: Frame,
    constraints: (HConstraint, VConstraint),
    region: PatternRegion,
    basic: bool,
) -> Object {
    let o = inside(object(
        starter,
        n,
        name,
        frame,
        constraints,
        Style::Role(if region.pattern.is_dotted() {
            LineRole::DotGrid
        } else {
            LineRole::Guide
        }),
        ObjectKind::Pattern(region),
    ));
    if basic { o } else { quiet(o) }
}

/// A region with the Structure border on the top and bottom only (no sides: a form with open ends).
fn ruled_between(mut region: PatternRegion, top: bool, bottom: bool) -> PatternRegion {
    let mut b = Border::new();
    b.sides = BorderSides {
        top,
        right: false,
        bottom,
        left: false,
    };
    region.border = Some(b);
    region
}

/// A region with its own spacing (px across, px down), counted from its own top-left corner, not the rhythm's.
fn own_spacing(mut region: PatternRegion, across: i64, down: i64) -> PatternRegion {
    region.bound = false;
    region.h_spacing_mm = Some(px(across));
    region.v_spacing_mm = Some(px(down));
    region
}

/// A line inside the margin box in `role`, marked Show in Basic when `basic`.
fn rule(
    starter: u16,
    n: u16,
    name: &str,
    frame: Frame,
    constraints: (HConstraint, VConstraint),
    role: LineRole,
    basic: bool,
) -> Object {
    let o = inside(line(starter, n, name, frame, constraints, role));
    if basic { o } else { quiet(o) }
}

/// The size of the labels in points (the smallest the linter accepts is 6; this reads comfortably).
const LABEL_PT: f64 = 9.0;

/// A label in the Label role, in the bundled Atkinson Hyperlegible Next. With `on_line` its letters stand a small
/// gap above the rhythm line below the box's top (the box is one step tall, so that is the next line down); without,
/// the text hangs from the box's top.
#[allow(clippy::too_many_arguments)]
fn label(
    starter: u16,
    n: u16,
    name: &str,
    content: &str,
    x: i64,
    y: i64,
    constraints: (HConstraint, VConstraint),
    on_line: bool,
) -> Object {
    let tb = TextBox {
        content: content.into(),
        size_pt: LABEL_PT,
        weight: 600,
        baselines: on_line,
        baseline_gap: on_line,
        ..TextBox::default()
    };
    quiet(inside(object(
        starter,
        n,
        name,
        frame(x, y, 3 * U, if on_line { U } else { 48 }),
        constraints,
        Style::Role(LineRole::Label),
        ObjectKind::Text(tb),
    )))
}

/// A group of `children` (a box round them), not marked and not Full bleed.
fn group(starter: u16, n: u16, name: &str, children: Vec<Object>) -> Object {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for c in &children {
        let b = c.outer_box();
        x0 = x0.min(b.x_mm);
        y0 = y0.min(b.y_mm);
        x1 = x1.max(b.x_mm + b.w_mm);
        y1 = y1.max(b.y_mm + b.h_mm);
    }
    quiet(inside(object(
        starter,
        n,
        name,
        Frame::new(x0, y0, x1 - x0, y1 - y0),
        (HConstraint::Scale, VConstraint::Scale),
        Style::None,
        ObjectKind::Group(GroupShape {
            children,
            extra: Extra::new(),
        }),
    )))
}

/// Notes: a meeting page. A header of four ruled rows (meeting, date, attendees), a ruled notes area, and a box of
/// action items at the bottom with a tick box on each row and an owner column.
///
/// Header rows 0 to 4, notes rows 4 to 13, action items rows 13 to 18 (the Manta's notes area is taller: the box is
/// kept from the bottom). The labels' letters stand on the line at the foot of their row. The tick boxes are a
/// one-column grid on the rhythm, so they line up with the ruling on both tablets.
fn meeting_notes() -> Starter {
    const N: u16 = 13;
    Starter {
        category: "notes",
        slug: "meeting-notes",
        doc: Page {
            title: "Meeting notes",
            description: "Meeting notes: a header for the meeting, date and attendees, a ruled notes area and an action-items box with tick boxes.",
            tags: &["meeting", "notes", "agenda", "action items", "work"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![
                region(
                    N,
                    1,
                    "Header",
                    frame(col(0), row(0), FORM_W, 4 * U),
                    STRETCH_TOP,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    false,
                ),
                label(N, 2, "Meeting label", "Meeting", col(0), row(0), LEFT_TOP, true),
                label(N, 3, "Date label", "Date", col(0), row(1), LEFT_TOP, true),
                label(N, 4, "Attendees label", "Attendees", col(0), row(2), LEFT_TOP, true),
                region(
                    N,
                    5,
                    "Notes",
                    frame(col(0), row(4), FORM_W, 9 * U),
                    STRETCH,
                    PatternRegion::new(Pattern::Lines),
                    true,
                ),
                label(N, 6, "Notes label", "Notes", col(0), row(4), LEFT_TOP, true),
                region(
                    N,
                    7,
                    "Action items",
                    frame(col(0), row(13), FORM_W, 5 * U),
                    STRETCH_BOTTOM,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    true,
                ),
                region(
                    N,
                    8,
                    "Tick boxes",
                    frame(col(0), row(14), U, 4 * U),
                    LEFT_BOTTOM,
                    PatternRegion::new(Pattern::Grid),
                    false,
                ),
                rule(
                    N,
                    9,
                    "Owner divider",
                    frame(col(9), row(14), 0, 4 * U),
                    RIGHT_BOTTOM,
                    LineRole::Structure,
                    false,
                ),
                label(N, 10, "Action items label", "Action items", col(1), row(13), LEFT_BOTTOM, true),
                label(N, 11, "Owner label", "Owner", col(9), row(13), RIGHT_BOTTOM, true),
            ],
        }
        .document(),
    }
}

/// Notes: a lecture page. A header of three ruled rows (course, date, topic), ruled lines below and a margin line
/// at column 5 with a label for keywords above it.
///
/// Header rows 0 to 3, ruled lines rows 3 to 18 (the Manta's run to its row 26) closed by a border at the bottom.
fn lecture_notes() -> Starter {
    const N: u16 = 14;
    Starter {
        category: "notes",
        slug: "lecture-notes",
        doc: Page {
            title: "Lecture notes",
            description: "Lecture notes: a header for course, date and topic, ruled lines and a margin line for keywords.",
            tags: &["lecture", "notes", "class", "study", "keywords", "course"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![
                region(
                    N,
                    1,
                    "Header",
                    frame(col(0), row(0), FORM_W, 3 * U),
                    STRETCH_TOP,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    false,
                ),
                label(N, 2, "Course label", "Course", col(0), row(0), LEFT_TOP, true),
                label(N, 3, "Date label", "Date", col(0), row(1), LEFT_TOP, true),
                label(N, 4, "Topic label", "Topic", col(0), row(2), LEFT_TOP, true),
                region(
                    N,
                    5,
                    "Lines",
                    frame(col(0), row(3), FORM_W, 15 * U),
                    STRETCH,
                    ruled_between(PatternRegion::new(Pattern::Lines), false, true),
                    true,
                ),
                rule(
                    N,
                    6,
                    "Margin",
                    frame(col(5), row(3), 0, 15 * U),
                    LEFT_STRETCH,
                    LineRole::Margin,
                    true,
                ),
                label(N, 7, "Keywords label", "Keywords", col(0), row(3), LEFT_TOP, true),
                label(N, 8, "Notes label", "Notes", col(6), row(3), LEFT_TOP, true),
            ],
        }
        .document(),
    }
}

/// Planning: a day on one page. An hourly schedule column (6:00 to 22:00) at the left, and beside it a to-do list
/// with a tick box on each row over a notes area.
///
/// The schedule column is six steps wide and kept from the left; the right column takes the rest of the width. Rows 0
/// to 10 are the to-do list (nine tick boxes), rows 10 to the bottom the notes. Each hour's label hangs from the top
/// line of its row.
fn daily_planner() -> Starter {
    const N: u16 = 15;
    let hours: Vec<Object> = (0..17)
        .map(|i| {
            label(
                N,
                30 + i as u16,
                &format!("Hour {}:00", 6 + i),
                &format!("{}:00", 6 + i),
                col(0),
                row(1 + i),
                LEFT_TOP,
                false,
            )
        })
        .collect();
    Starter {
        category: "planning",
        slug: "daily-planner",
        doc: Page {
            title: "Daily planner",
            description: "A day on one page: an hourly schedule beside a to-do list with tick boxes and a notes area.",
            tags: &["planner", "daily", "schedule", "hourly", "to-do", "planning"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![
                region(
                    N,
                    1,
                    "Schedule",
                    frame(col(0), row(0), 6 * U, 18 * U),
                    LEFT_STRETCH,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    true,
                ),
                label(N, 2, "Schedule label", "Schedule", col(0), row(0), LEFT_TOP, true),
                group(N, 3, "Hours", hours),
                rule(
                    N,
                    4,
                    "Divider",
                    frame(col(6), row(0), 0, 18 * U),
                    LEFT_STRETCH,
                    LineRole::Structure,
                    false,
                ),
                region(
                    N,
                    5,
                    "To do",
                    frame(col(6), row(0), 7 * U, 10 * U),
                    STRETCH_TOP,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    true,
                ),
                region(
                    N,
                    6,
                    "Tick boxes",
                    frame(col(6), row(1), U, 9 * U),
                    LEFT_TOP,
                    PatternRegion::new(Pattern::Grid),
                    false,
                ),
                label(N, 7, "To do label", "To do", col(7), row(0), LEFT_TOP, true),
                region(
                    N,
                    8,
                    "Notes",
                    frame(col(6), row(10), 7 * U, 8 * U),
                    STRETCH,
                    ruled_between(PatternRegion::new(Pattern::Lines), false, true),
                    false,
                ),
                label(N, 9, "Notes label", "Notes", col(7), row(10), LEFT_TOP, true),
            ],
        }
        .document(),
    }
}

/// Planning: ruled rows, each with a tick box. A header row says "To do"; below it a column of square tick boxes (a
/// one-column grid on the rhythm) runs the page's height beside the ruled rows.
fn to_do_list() -> Starter {
    const N: u16 = 16;
    Starter {
        category: "planning",
        slug: "to-do-list",
        doc: Page {
            title: "To-do list",
            description: "Ruled rows, each with a tick box.",
            tags: &["to-do", "checklist", "tasks", "tick boxes", "planning"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![
                region(
                    N,
                    1,
                    "Rows",
                    frame(col(0), row(0), FORM_W, 18 * U),
                    STRETCH,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    true,
                ),
                region(
                    N,
                    2,
                    "Tick boxes",
                    frame(col(0), row(1), U, 17 * U),
                    LEFT_STRETCH,
                    PatternRegion::new(Pattern::Grid),
                    true,
                ),
                label(N, 3, "To do label", "To do", col(0), row(0), LEFT_TOP, true),
            ],
        }
        .document(),
    }
}

/// Bullet journal: a habit tracker on a landscape page: a column for the habit's name, then 31 day columns numbered
/// 1 to 31 (a portrait page is too narrow: 31 columns a writable width wide would not fit beside the names).
///
/// Landscape, so the design page is 1872 by 1404 px on the Nomad (2560 by 1920 on the Manta). Rows are 86 px (13 on
/// the Nomad, 19 on the Manta: the first is the header with the day numbers); the margins are 143 px top and bottom
/// and 152 at the sides. The 31 columns are 38 px each, 1178 px together, with their own spacing (not the rhythm's)
/// and kept from the right edge, so every tablet draws the same cells and the names column takes the width that is
/// left (the day numbers are Free placement: they sit in the cells, not on the rhythm). The rhythm is 38 px across
/// and 86 down.
fn habit_tracker() -> Starter {
    const N: u16 = 18;
    const GRID_X: i64 = 532;
    const CELL: i64 = 38;
    let top = 143;
    let rows = 13 * U;
    let numbers: Vec<Object> = (0..31)
        .map(|i| {
            let tb = TextBox {
                content: format!("{}", i + 1),
                size_pt: 6.5,
                weight: 600,
                align: TextAlign::Center,
                sizing: TextSizing::Fixed,
                v_align: VerticalAlign::Middle,
                ..TextBox::default()
            };
            quiet(inside(object(
                N,
                40 + i as u16,
                &format!("Day {}", i + 1),
                frame(GRID_X + CELL * i, top, CELL, U),
                RIGHT_TOP,
                Style::Role(LineRole::Label),
                ObjectKind::Text(tb),
            )))
        })
        .collect();
    let mut numbers = group(N, 4, "Day numbers", numbers);
    numbers.free_placement = true;
    let mut names = ruled_between(PatternRegion::new(Pattern::Lines), true, true);
    if let Some(b) = names.border.as_mut() {
        b.sides.left = true;
    }
    let mut doc = Page {
        title: "Habit tracker",
        description: "A habit-name column and 31 day columns numbered 1 to 31, on a landscape page.",
        tags: &["habit", "tracker", "bullet journal", "days", "31 days", "landscape"],
        margins: Margins {
            top: px(143),
            right: px(152),
            bottom: px(143),
            left: px(152),
        },
        rhythm: Rhythm {
            h_unit_mm: px(CELL),
            v_unit_mm: px(U),
            origin: RhythmOrigin::MarginCorner,
            ..square_rhythm(px(U))
        },
        objects: vec![
            region(
                N,
                1,
                "Habits",
                frame(152, top, GRID_X - 152, rows),
                STRETCH,
                names,
                true,
            ),
            region(
                N,
                2,
                "Days",
                frame(GRID_X, top, 31 * CELL, rows),
                RIGHT_STRETCH,
                bordered(own_spacing(PatternRegion::new(Pattern::Grid), CELL, U)),
                true,
            ),
            label(N, 3, "Habit label", "Habit", 152 + CELL, top, LEFT_TOP, true),
            numbers,
            rule(
                N,
                5,
                "Header rule",
                frame(152, top + U, GRID_X + 31 * CELL - 152, 0),
                STRETCH_TOP,
                LineRole::Structure,
                false,
            ),
        ],
    }
    .document();
    doc.page.orientation = Orientation::Landscape;
    Starter {
        category: "bullet-journal",
        slug: "habit-tracker",
        doc,
    }
}

/// Bullet journal: an index. Two columns, topic and page numbers: ruled rows under a header row, and a divider
/// 2 steps from the right edge between the two.
fn index() -> Starter {
    const N: u16 = 17;
    Starter {
        category: "bullet-journal",
        slug: "index",
        doc: Page {
            title: "Index",
            description: "A bullet journal index: a topic column and a page-number column, ruled.",
            tags: &["index", "bullet journal", "contents", "topics", "pages"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![
                region(
                    N,
                    1,
                    "Rows",
                    frame(col(0), row(0), FORM_W, 18 * U),
                    STRETCH,
                    ruled_between(PatternRegion::new(Pattern::Lines), true, true),
                    true,
                ),
                rule(
                    N,
                    2,
                    "Header rule",
                    frame(col(0), row(1), FORM_W, 0),
                    STRETCH_TOP,
                    LineRole::Structure,
                    false,
                ),
                rule(
                    N,
                    3,
                    "Page divider",
                    frame(col(11), row(1), 0, 17 * U),
                    RIGHT_STRETCH,
                    LineRole::Structure,
                    true,
                ),
                label(N, 4, "Topic label", "Topic", col(0), row(0), LEFT_TOP, true),
                {
                    // Two steps wide, so the box ends on the margin box's edge (three would reach the toolbar).
                    let mut page =
                        label(N, 5, "Page label", "Page", col(11), row(0), RIGHT_TOP, true);
                    page.frame.w_mm = px(2 * U);
                    page
                },
            ],
        }
        .document(),
    }
}

/// Creative: a storyboard. A block of a 16:9 frame (516 by 290 px) with two ruled rows under it for notes, repeated
/// in as many columns and rows as fit (2 by 3 on the Nomad, 2 by 4 on the Manta), centred in the margin box.
///
/// A block is 516 px wide and 462 tall; the repeat steps 7 units across and 6 down, so the blocks are a unit apart
/// across and 54 px down. The notes lines have their own spacing (86 px) from the frame's bottom edge, so they
/// start on it whatever the frame's height.
fn storyboard() -> Starter {
    const N: u16 = 19;
    let frame_box = inside(quiet(object(
        N,
        1,
        "Frame",
        frame(col(0), row(0), 6 * U, 290),
        STRETCH,
        Style::Role(LineRole::Structure),
        ObjectKind::Rect(RectShape::default()),
    )));
    let notes = region(
        N,
        2,
        "Notes lines",
        frame(col(0), row(0) + 290, 6 * U, 2 * U),
        STRETCH,
        own_spacing(PatternRegion::new(Pattern::Lines), 6 * U, U),
        false,
    );
    let block = group(N, 3, "Block", vec![frame_box, notes]);
    let mut shape = RepeatShape::new(block);
    shape.mode = RepeatMode::Grid;
    shape.fill = true;
    shape.h_spacing = Spacing::Units(7.0);
    shape.v_spacing = Spacing::Units(6.0);
    shape.leftover = Leftover::Centre;
    Starter {
        category: "creative",
        slug: "storyboard",
        doc: Page {
            title: "Storyboard",
            description: "Frames in the 16:9 shape, each with ruled lines under it for notes.",
            tags: &["storyboard", "frames", "16:9", "film", "video", "comics"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![inside(object(
                N,
                4,
                "Frames",
                frame(col(0), row(0), FORM_W, 18 * U),
                STRETCH,
                Style::None,
                ObjectKind::Repeat(shape),
            ))],
        }
        .document(),
    }
}

/// Creative: handwriting practice. Rows of four lines: an ascender line, a dashed midline, a baseline (heavier) and
/// a descender line, repeated down the page.
///
/// The app's ruled pattern draws solid lines at one spacing, so each row is a repeat: a one-step-wide piece of the
/// four lines (a Whisper top line 0 px down, a dashed Whisper midline at 64, the Structure baseline at 128 and a
/// Whisper descender line at 172) repeated across and down. The margin box is a whole number of steps wide on both
/// tablets, so the pieces meet exactly, and the dash (22 px, gap 21) is two periods in a piece, so it runs on across
/// the joins. Rows are 3 steps apart: 6 on the Nomad, 9 on the Manta.
fn handwriting_practice() -> Starter {
    const N: u16 = 20;
    let piece = |n: u16, name: &str, y: i64, role: LineRole| {
        rule(
            N,
            n,
            name,
            frame(col(0), row(0) + y, U, 0),
            LEFT_TOP,
            role,
            false,
        )
    };
    let mut midline = piece(2, "Midline", 64, LineRole::Whisper);
    midline.dash = DashStyle::Custom {
        dash_mm: px(22),
        gap_mm: px(21),
    };
    let block = group(
        N,
        1,
        "Row piece",
        vec![
            piece(1, "Ascender line", 0, LineRole::Whisper),
            midline,
            piece(3, "Baseline", 128, LineRole::Structure),
            piece(4, "Descender line", 172, LineRole::Whisper),
        ],
    );
    let mut shape = RepeatShape::new(block);
    shape.mode = RepeatMode::Grid;
    shape.fill = true;
    shape.h_spacing = Spacing::Units(1.0);
    shape.v_spacing = Spacing::Units(3.0);
    shape.leftover = Leftover::Centre;
    Starter {
        category: "creative",
        slug: "handwriting-practice",
        doc: Page {
            title: "Handwriting practice",
            description: "Rows with an ascender line, a dashed midline, a baseline and a descender line, for practising letters.",
            tags: &["handwriting", "penmanship", "practice", "letters", "calligraphy", "lettering"],
            margins: form_margins(),
            rhythm: form_rhythm(),
            objects: vec![inside(object(
                N,
                5,
                "Rows",
                frame(col(0), row(0), FORM_W, 18 * U),
                STRETCH,
                Style::None,
                ObjectKind::Repeat(shape),
            ))],
        }
        .document(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Making the files.
// ---------------------------------------------------------------------------------------------------------------

/// The `.quad` bytes for `doc`: its text outlines on each device (none of the starters has text yet) and the
/// Nomad's picture, as the app's Save writes them (`Viewer::preview_png`).
pub fn build(doc: &Document) -> Result<Vec<u8>, String> {
    let o = doc.page.orientation;
    let primary =
        quadrule_profiles::builtin_for(&doc.primary_profile_id, o).map_err(|e| e.to_string())?;
    let mut reg = FontRegistry::bundled_only();
    let mut outlines = Vec::new();
    let mut preview = None;
    for id in doc.profile_ids() {
        let target = quadrule_profiles::builtin_for(id, o).map_err(|e| e.to_string())?;
        let laid = lay_out_text(doc, &primary, &target, &mut reg).map_err(|e| e.to_string())?;
        if id == doc.primary_profile_id {
            let dl =
                display_list(doc, &primary, &target, &laid.outlines).map_err(|e| e.to_string())?;
            preview = Some(preview_png(&render(&dl), o == Orientation::Landscape));
        }
        outlines.push(laid.outlines);
    }
    let mut file = QuadFile::new(doc.clone(), outlines, vec![]);
    if let Some(png) = preview {
        file.previews
            .insert(preview_path(&doc.primary_profile_id), png);
    }
    file.to_bytes(SAVED_BY).map_err(|e| e.to_string())
}

/// The picture a file keeps of its main device: 8-bit gray, longest side [`PREVIEW_SIDE`] px, shrunk by averaging.
fn preview_png(page: &Gray, landscape: bool) -> Vec<u8> {
    let turned;
    let px = if landscape {
        turned = turn_clockwise(page);
        &turned
    } else {
        page
    };
    let scale = (f64::from(PREVIEW_SIDE) / f64::from(px.width.max(px.height))).min(1.0);
    let w = ((f64::from(px.width) * scale).round() as u32).max(1);
    let h = ((f64::from(px.height) * scale).round() as u32).max(1);
    encode_png(&shrink(px, w, h), None)
}

/// Every starter's path under [`starters_dir`] (`<category>/<slug>.quad`) with its bytes, in the order of [`all`].
pub fn files() -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    all()
        .into_iter()
        .map(|s| {
            let path = PathBuf::from(s.category).join(format!("{}.quad", s.slug));
            build(&s.doc).map(|bytes| (path, bytes))
        })
        .collect()
}

/// Writes every starter under `root`, and removes any `.quad` there that no starter makes. Returns the paths
/// written, relative to `root`.
pub fn write_all(root: &Path) -> Result<Vec<PathBuf>, String> {
    let files = files()?;
    for (rel, bytes) in &files {
        let path = root.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    for stale in stale_files(root, &files) {
        std::fs::remove_file(&stale).map_err(|e| format!("{}: {e}", stale.display()))?;
    }
    Ok(files.into_iter().map(|(p, _)| p).collect())
}

/// The `.quad` files under `root` (one folder deep) that no starter makes.
pub fn stale_files(root: &Path, made: &[(PathBuf, Vec<u8>)]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in std::fs::read_dir(root).into_iter().flatten().flatten() {
        if !dir.path().is_dir() {
            continue;
        }
        for file in std::fs::read_dir(dir.path())
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = file.path();
            if path.extension().is_some_and(|x| x == "quad")
                && let Ok(rel) = path.strip_prefix(root)
                && !made.iter().any(|(p, _)| p == rel)
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}
