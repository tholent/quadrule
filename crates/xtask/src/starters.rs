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
/// odd columns, on the Manta each leaves a 23 or 25 px strip, so no horizontal edge sits a few pixels from the border.
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
