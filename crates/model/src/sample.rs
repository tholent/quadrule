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

/// A region of `pattern` as a page made before format 0.20 holds it: its major lines count from the
/// rhythm origin. Every sample page here was made then and its golden image was blessed then, so none
/// of them changes (ADR-023, item 2); a region made in the app now starts at Top
/// ([`PatternRegion::new`]).
fn region_0_19(pattern: Pattern) -> PatternRegion {
    PatternRegion {
        major_rows_from: MajorFrom::RhythmOrigin,
        major_columns_from: MajorFrom::RhythmOrigin,
        ..PatternRegion::new(pattern)
    }
}

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
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        free_placement: false,
        full_bleed: false,
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
            style: FontStyle::Normal,
            size_pt: 12.0,
            line_height: 1.25,
            letter_spacing_em: 0.0,
            align: TextAlign::Left,
            ..TextBox::default()
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
        ObjectKind::Pattern(region_0_19(Pattern::Dots)),
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
        ObjectKind::Pattern(region_0_19(Pattern::Lines)),
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
        profiles: vec![ProfileContext::new(MANTA_ID), ProfileContext::new(NOMAD_ID)],
        units: Unit::Mm,
        page: PageSettings {
            background_gray: 255,
            margins_mm: Margins::all(margin),
            orientation: Default::default(),
            fonts: Default::default(),
            extra: Extra::new(),
        },
        rhythm: Rhythm {
            h_unit_mm: 6.0,
            v_unit_mm: 6.0,
            origin: RhythmOrigin::MarginCorner,
            resolution: RhythmResolution::PixelLocked,
            subdivisions: None,
            rhythm_type: RhythmType::Rectangular,
            side_mm: None,
            major_every: None,
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
        ignored_rules: Vec::new(),
        extra: Extra::new(),
    }
}

/// Fixed ids for the shapes page's objects.
pub mod shape_ids {
    use crate::Uuid;
    /// The `n`th object of the shapes page (1-based); the dot grid is [`super::ids::DOTS`].
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0006_4000_8000_0000_0000_0000 + n)
    }
}

/// The shapes page (rung 6a, docs/bbq/2026-11-10-rung-6-shapes.md, Defaulted "Goldens"): every
/// new kind (rectangle square and rounded, ellipse, circle, polygons of several sides, stars), both
/// fill roles, a custom fill, a shape with no stroke, each dash preset and a custom one on lines
/// and on closed shapes, and each cap. It holds the sample's dot grid ([`ids::DOTS`]) too, so
/// the drag benchmark can run over it. Designed on the Manta, also targeting the Nomad, with no
/// text, so it needs no fonts.
pub fn shapes_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 6a shapes".into();
    doc.metadata.description = "The rung 6a shapes page: every shape, fill, dash and cap.".into();
    let rule = doc.layers[1].objects[1].clone();
    let dots = doc.layers[0].objects[0].clone();
    let mk = |n: u128, name: &str, frame: Frame, style: Style, kind: ObjectKind| {
        let mut o = rule.clone();
        o.id = shape_ids::nth(n);
        o.name = name.into();
        o.frame = frame;
        o.constraints = Constraints::default();
        o.style = style;
        o.kind = kind;
        o
    };
    let structure = Style::Role(LineRole::Structure);
    let rect = |r: f64| {
        ObjectKind::Rect(RectShape {
            corner_radius_mm: r,
            extra: Extra::new(),
        })
    };
    let ellipse = || ObjectKind::Ellipse(EllipseShape::default());
    let poly = |n: u8| ObjectKind::Polygon(PolygonShape::polygon(n));
    let star = |n: u8, inner: f64| {
        ObjectKind::Polygon(PolygonShape {
            sides: n,
            star: Some(inner),
            extra: Extra::new(),
        })
    };
    let mut objs: Vec<Object> = Vec::new();
    let mut n = 0u128;
    let mut add = |name: &str,
                   frame: Frame,
                   style: Style,
                   fill: Option<Fill>,
                   dash: DashStyle,
                   cap: LineCap,
                   kind: ObjectKind| {
        n += 1;
        let mut o = mk(n, name, frame, style, kind);
        o.fill = fill;
        o.dash = dash;
        o.cap = cap;
        objs.push(o);
    };
    let (shade, block) = (
        Some(Fill::Role(FillRole::Shade)),
        Some(Fill::Role(FillRole::Block)),
    );
    let (solid, butt) = (DashStyle::Solid, LineCap::Butt);
    let cell = |col: f64, row: f64, w: f64, h: f64| Frame::new(8.0 + col * 24.0, row, w, h);

    // Row 1: the kinds, fills and no stroke.
    add(
        "Rectangle",
        cell(0.0, 32.0, 20.0, 20.0),
        structure,
        None,
        solid,
        butt,
        rect(0.0),
    );
    add(
        "Rounded rectangle, Shade",
        cell(1.0, 32.0, 20.0, 20.0),
        structure,
        shade,
        solid,
        butt,
        rect(4.0),
    );
    add(
        "Rectangle, Block, no stroke",
        cell(2.0, 32.0, 20.0, 20.0),
        Style::None,
        block,
        solid,
        butt,
        rect(0.0),
    );
    add(
        "Ellipse, custom fill",
        cell(3.0, 32.0, 20.0, 14.0),
        structure,
        Some(Fill::Gray(153)),
        solid,
        butt,
        ellipse(),
    );
    add(
        "Hexagon, Shade",
        cell(4.0, 32.0, 20.0, 20.0),
        structure,
        shade,
        solid,
        butt,
        poly(6),
    );
    add(
        "Star, Block",
        cell(5.0, 32.0, 20.0, 20.0),
        Style::Role(LineRole::Emphasis),
        block,
        solid,
        butt,
        star(5, 0.5),
    );
    // Row 2: other strokes, sides and stars.
    add(
        "Rectangle, Emphasis, Shade",
        cell(0.0, 58.0, 20.0, 14.0),
        Style::Role(LineRole::Emphasis),
        shade,
        solid,
        butt,
        rect(0.0),
    );
    add(
        "Circle",
        cell(1.0, 58.0, 14.0, 14.0),
        structure,
        None,
        solid,
        butt,
        ellipse(),
    );
    add(
        "Triangle",
        cell(2.0, 58.0, 20.0, 18.0),
        Style::Role(LineRole::Guide),
        None,
        solid,
        butt,
        poly(3),
    );
    add(
        "Heptagon, custom stroke",
        cell(3.0, 58.0, 20.0, 20.0),
        Style::Custom {
            width_mm: 0.6,
            gray: 0,
        },
        None,
        solid,
        butt,
        poly(7),
    );
    add(
        "Octagon, Block",
        cell(4.0, 58.0, 20.0, 20.0),
        Style::None,
        block,
        solid,
        butt,
        poly(8),
    );
    add(
        "Star, 8 points",
        cell(5.0, 58.0, 20.0, 20.0),
        structure,
        shade,
        solid,
        butt,
        star(8, 0.3),
    );
    // Rows 3-4: each dash preset on a line (Structure, 2 px).
    let presets = [
        ("Solid", DashStyle::Solid),
        ("Dotted", DashStyle::Dotted),
        ("Short dash", DashStyle::Short),
        ("Dash", DashStyle::Dash),
        ("Long dash", DashStyle::Long),
        (
            "Custom dash",
            DashStyle::Custom {
                dash_mm: 1.5,
                gap_mm: 1.0,
            },
        ),
    ];
    for (i, (name, d)) in presets.iter().enumerate() {
        add(
            &format!("Line, {name}"),
            Frame::new(8.0, 86.0 + 6.0 * i as f64, 70.0, 0.0),
            structure,
            None,
            *d,
            butt,
            ObjectKind::Line(Line::default()),
        );
    }
    // Each cap on a thick dashed line. The gap is longer than the stroke is wide, so the caps show
    // (a cap reaches half the width into the gap at each end of every dash).
    let thick = Style::Custom {
        width_mm: 1.0,
        gray: 51,
    };
    let spaced = DashStyle::Custom {
        dash_mm: 2.0,
        gap_mm: 3.0,
    };
    for (i, cap) in LineCap::ALL.iter().enumerate() {
        add(
            &format!("Line, {cap:?} cap"),
            Frame::new(86.0, 88.0 + 8.0 * i as f64, 66.0, 0.0),
            thick,
            None,
            spaced,
            *cap,
            ObjectKind::Line(Line::default()),
        );
    }
    // A slanted dashed line, and a vertical one with square caps.
    add(
        "Slanted line, Dash",
        Frame::new(86.0, 114.0, 30.0, 20.0),
        structure,
        None,
        DashStyle::Dash,
        butt,
        ObjectKind::Line(Line::default()),
    );
    add(
        "Vertical line, Short dash, square caps",
        Frame::new(124.0, 114.0, 0.0, 20.0),
        structure,
        None,
        DashStyle::Short,
        LineCap::Square,
        ObjectKind::Line(Line::default()),
    );
    // Row 5: dashed closed shapes, each cap.
    for (i, cap) in LineCap::ALL.iter().enumerate() {
        add(
            &format!("Dashed rectangle, {cap:?}"),
            cell(i as f64, 142.0, 20.0, 16.0),
            structure,
            None,
            DashStyle::Dash,
            *cap,
            rect(0.0),
        );
    }
    add(
        "Dashed rounded rectangle",
        cell(3.0, 142.0, 20.0, 16.0),
        structure,
        shade,
        DashStyle::Short,
        butt,
        rect(5.0),
    );
    add(
        "Dotted ellipse",
        cell(4.0, 142.0, 20.0, 16.0),
        structure,
        None,
        DashStyle::Dotted,
        butt,
        ellipse(),
    );
    add(
        "Dashed star",
        cell(5.0, 142.0, 20.0, 20.0),
        structure,
        None,
        DashStyle::Dash,
        LineCap::Round,
        star(5, 0.5),
    );
    // Row 6: dashed polygons and a thick rectangle, filled.
    add(
        "Dashed hexagon, round caps",
        cell(0.0, 168.0, 20.0, 20.0),
        thick,
        None,
        DashStyle::Long,
        LineCap::Round,
        poly(6),
    );
    add(
        "Dashed triangle, square caps",
        cell(1.0, 168.0, 20.0, 18.0),
        Style::Role(LineRole::Emphasis),
        shade,
        DashStyle::Dash,
        LineCap::Square,
        poly(3),
    );
    add(
        "Thick rectangle, Block fill",
        cell(2.0, 168.0, 20.0, 18.0),
        thick,
        block,
        solid,
        butt,
        rect(0.0),
    );
    add(
        "Thick rounded rectangle",
        cell(3.0, 168.0, 20.0, 18.0),
        thick,
        shade,
        solid,
        butt,
        rect(8.0),
    );

    let mut dot_strip = dots.clone();
    dot_strip.frame = Frame::new(8.0, 8.0, 146.0, 18.0);
    // Scales with the page like everything else on it, so no shape lands on the dots.
    dot_strip.constraints = Constraints::default();
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: vec![dot_strip],
            extra: Extra::new(),
        },
        Layer {
            name: "Shapes".into(),
            visible: true,
            locked: false,
            objects: objs,
            extra: Extra::new(),
        },
    ];
    doc
}

/// Fixed ids for the transforms page's objects.
pub mod transform_ids {
    use crate::Uuid;
    /// The `n`th object of the transforms page (1-based); the dot grid is [`super::ids::DOTS`].
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0008_4000_8000_0000_0000_0000 + n)
    }
    /// The large turned rectangle that the rotate benchmark drives.
    pub const TURNED: Uuid = nth(1000);
}

/// The rung 6b transforms page (docs/bbq/2026-11-10-rung-6-shapes.md, 6b "Goldens"): shapes
/// rotated by several angles (90, 180 and 270 included, which keep their straight edges), lines
/// pointing every way (alone, turned, dashed), and opacity over fills and lines (a half-clear
/// shape over a Block bar, a faint line across a Shade box, a clear pattern region, a shape at
/// 0 percent that draws nothing). It holds the sample's dot grid ([`ids::DOTS`]) so the drag
/// benchmark can run over it, and no text, so it needs no fonts. Designed on the Manta, also
/// targeting the Nomad.
pub fn transforms_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 6b transforms".into();
    doc.metadata.description =
        "The rung 6b transforms page: rotation, lines pointing every way, opacity.".into();
    let rule = doc.layers[1].objects[1].clone();
    let dots = doc.layers[0].objects[0].clone();
    let structure = Style::Role(LineRole::Structure);
    let (shade, block) = (
        Some(Fill::Role(FillRole::Shade)),
        Some(Fill::Role(FillRole::Block)),
    );
    let rect = |r: f64| {
        ObjectKind::Rect(RectShape {
            corner_radius_mm: r,
            extra: Extra::new(),
        })
    };
    let ellipse = || ObjectKind::Ellipse(EllipseShape::default());
    let poly = |n: u8| ObjectKind::Polygon(PolygonShape::polygon(n));
    let star = |n: u8| ObjectKind::Polygon(PolygonShape::star(n));
    let line = |dir: LineDir| {
        ObjectKind::Line(Line {
            direction: dir,
            ..Line::default()
        })
    };
    let mut objs: Vec<Object> = Vec::new();
    let mut n = 0u128;
    // (name, frame, style, fill, dash, rotation, opacity, kind)
    #[allow(clippy::too_many_arguments)]
    let mut add = |name: &str,
                   frame: Frame,
                   style: Style,
                   fill: Option<Fill>,
                   dash: DashStyle,
                   rotation: f64,
                   opacity: u8,
                   kind: ObjectKind| {
        n += 1;
        let mut o = rule.clone();
        o.id = transform_ids::nth(n);
        o.name = name.into();
        o.frame = frame;
        o.constraints = Constraints::default();
        o.style = style;
        o.fill = fill;
        o.dash = dash;
        o.rotation_deg = rotation;
        o.opacity_pct = opacity;
        o.kind = kind;
        objs.push(o);
    };
    let solid = DashStyle::Solid;
    let cell = |col: f64, row: f64, w: f64, h: f64| Frame::new(8.0 + col * 24.0, row, w, h);

    // Row 1: a rectangle turned by 0, 15, 30, 45 and 90 degrees, and a rounded one at 20.
    for (i, deg) in [0.0, 15.0, 30.0, 45.0, 90.0].iter().enumerate() {
        add(
            &format!("Rectangle, {deg} degrees"),
            cell(i as f64, 32.0, 20.0, 12.0),
            structure,
            if i == 0 { None } else { shade },
            solid,
            *deg,
            100,
            rect(0.0),
        );
    }
    add(
        "Rounded rectangle, 20 degrees",
        cell(5.0, 32.0, 20.0, 12.0),
        structure,
        shade,
        solid,
        20.0,
        100,
        rect(3.0),
    );
    // Row 2: other shapes turned, the quarter turns among them.
    add(
        "Ellipse, 30 degrees",
        cell(0.0, 56.0, 22.0, 12.0),
        structure,
        shade,
        solid,
        30.0,
        100,
        ellipse(),
    );
    add(
        "Ellipse, 90 degrees",
        cell(1.0, 56.0, 22.0, 12.0),
        structure,
        shade,
        solid,
        90.0,
        100,
        ellipse(),
    );
    add(
        "Hexagon, 90 degrees",
        cell(2.0, 54.0, 18.0, 18.0),
        structure,
        shade,
        solid,
        90.0,
        100,
        poly(6),
    );
    add(
        "Triangle, 180 degrees",
        cell(3.0, 54.0, 18.0, 16.0),
        structure,
        shade,
        solid,
        180.0,
        100,
        poly(3),
    );
    add(
        "Triangle, 270 degrees",
        cell(4.0, 54.0, 18.0, 16.0),
        structure,
        None,
        solid,
        270.0,
        100,
        poly(3),
    );
    add(
        "Star, 36 degrees",
        cell(5.0, 52.0, 20.0, 20.0),
        Style::Role(LineRole::Emphasis),
        block,
        solid,
        36.0,
        100,
        star(5),
    );
    // Row 3: lines pointing every way, then turned and dashed.
    let guide = Style::Role(LineRole::Guide);
    let emphasis = Style::Role(LineRole::Emphasis);
    let lines = [
        (
            "Line, down right",
            Frame::new(8.0, 84.0, 24.0, 16.0),
            LineDir::Down,
            0.0,
            solid,
            structure,
        ),
        (
            "Line, up right",
            Frame::new(38.0, 84.0, 24.0, 16.0),
            LineDir::Up,
            0.0,
            solid,
            structure,
        ),
        (
            "Line, steep up right",
            Frame::new(68.0, 84.0, 10.0, 24.0),
            LineDir::Up,
            0.0,
            solid,
            emphasis,
        ),
        (
            "Line, up right, dashed",
            Frame::new(84.0, 84.0, 24.0, 16.0),
            LineDir::Up,
            0.0,
            DashStyle::Dash,
            structure,
        ),
        (
            "Line, 90 degrees",
            Frame::new(122.0, 94.0, 20.0, 0.0),
            LineDir::Down,
            90.0,
            solid,
            structure,
        ),
        (
            "Line, 30 degrees",
            Frame::new(10.0, 116.0, 30.0, 0.0),
            LineDir::Down,
            30.0,
            solid,
            guide,
        ),
        (
            "Line, up right, 20 degrees",
            Frame::new(50.0, 110.0, 20.0, 14.0),
            LineDir::Up,
            20.0,
            solid,
            structure,
        ),
        (
            "Line, 45 degrees, dotted",
            Frame::new(90.0, 116.0, 30.0, 0.0),
            LineDir::Down,
            45.0,
            DashStyle::Dotted,
            emphasis,
        ),
    ];
    for (name, f, dir, deg, dash, style) in lines {
        add(name, f, style, None, dash, deg, 100, line(dir));
    }
    // Row 4: opacity. A Block bar with a half-clear shape across it, a Shade box with a faint
    // line over it, a clear pattern region over a bar, and a shape at 0 percent.
    add(
        "Block bar",
        Frame::new(8.0, 138.0, 70.0, 10.0),
        Style::None,
        block,
        solid,
        0.0,
        100,
        rect(0.0),
    );
    add(
        "Shade over the bar, 50 percent",
        Frame::new(20.0, 133.0, 30.0, 20.0),
        structure,
        shade,
        solid,
        0.0,
        50,
        rect(0.0),
    );
    add(
        "Circle over the bar, 25 percent",
        Frame::new(56.0, 132.0, 22.0, 22.0),
        emphasis,
        shade,
        solid,
        0.0,
        25,
        ellipse(),
    );
    add(
        "Shade box",
        Frame::new(90.0, 136.0, 40.0, 16.0),
        structure,
        shade,
        solid,
        0.0,
        100,
        rect(0.0),
    );
    add(
        "Line across the box, 40 percent",
        Frame::new(84.0, 132.0, 52.0, 24.0),
        emphasis,
        None,
        solid,
        0.0,
        40,
        line(LineDir::Up),
    );
    add(
        "Turned rectangle, 60 percent",
        Frame::new(8.0, 162.0, 40.0, 14.0),
        structure,
        block,
        solid,
        20.0,
        60,
        rect(0.0),
    );
    add(
        "Turned star, 50 percent",
        Frame::new(56.0, 160.0, 26.0, 26.0),
        structure,
        shade,
        solid,
        15.0,
        50,
        star(6),
    );
    add(
        "Pattern, 30 percent",
        Frame::new(90.0, 162.0, 40.0, 16.0),
        Style::Role(LineRole::Emphasis),
        None,
        solid,
        0.0,
        30,
        ObjectKind::Pattern(region_0_19(Pattern::Lines)),
    );
    add(
        "Circle, 0 percent",
        Frame::new(8.0, 188.0, 20.0, 20.0),
        structure,
        block,
        solid,
        0.0,
        0,
        ellipse(),
    );
    // The big turned rectangle the rotate benchmark drives, across the empty lower right.
    add(
        "Turned rectangle",
        Frame::new(90.0, 184.0, 56.0, 28.0),
        structure,
        shade,
        solid,
        10.0,
        100,
        rect(0.0),
    );
    if let Some(o) = objs.last_mut() {
        o.id = transform_ids::TURNED;
    }

    let mut dot_strip = dots.clone();
    dot_strip.frame = Frame::new(8.0, 8.0, 146.0, 18.0);
    dot_strip.constraints = Constraints::default();
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: vec![dot_strip],
            extra: Extra::new(),
        },
        Layer {
            name: "Transforms".into(),
            visible: true,
            locked: false,
            objects: objs,
            extra: Extra::new(),
        },
    ];
    doc
}

/// Fixed ids for the groups page's objects.
pub mod group_ids {
    use crate::Uuid;
    /// The `n`th object of the groups page (1-based, groups and children alike); the dot grid is
    /// [`super::ids::DOTS`].
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0009_4000_8000_0000_0000_0000 + n)
    }
    /// The outermost of the three nested groups.
    pub const NESTED: Uuid = nth(1);
    /// The turned group (holding a turned child).
    pub const TURNED: Uuid = nth(20);
    /// The group at 50 percent with children that overlap.
    pub const HALF: Uuid = nth(30);
    /// The group whose child is pinned to the top and right.
    pub const PINNED: Uuid = nth(40);
    /// The pinned child of [`PINNED`].
    pub const PINNED_CHILD: Uuid = nth(41);
    /// The heavy group the drag benchmark moves: dozens of children in two levels.
    pub const HEAVY: Uuid = nth(100);
}

/// The rung 7a groups page (docs/bbq/2026-11-11-rung-7-groups-and-align.md, Defaulted "Goldens
/// (7a)"): groups nested three deep, a turned group with a turned child, a group at 50 percent
/// whose children overlap (so the overlap must not show through), a group whose child is pinned
/// to the top and the right (it resolves against the page, not the group, so it keeps its size and
/// its distance from those edges of the page on the other device), and a heavy group of dozens of shapes in two levels for the drag benchmark. It holds the
/// sample's dot grid ([`ids::DOTS`]) so the benchmarks can run over it, and no text, so it needs
/// no fonts. Designed on the Manta, also targeting the Nomad.
pub fn groups_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 7a groups".into();
    doc.metadata.description =
        "The rung 7a groups page: nesting, a turned group, group opacity, a pinned child.".into();
    let rule = doc.layers[1].objects[1].clone();
    let dots = doc.layers[0].objects[0].clone();
    let structure = Style::Role(LineRole::Structure);
    let emphasis = Style::Role(LineRole::Emphasis);
    let (shade, block) = (
        Some(Fill::Role(FillRole::Shade)),
        Some(Fill::Role(FillRole::Block)),
    );
    let mk =
        |n: u128, name: &str, frame: Frame, style: Style, fill: Option<Fill>, kind: ObjectKind| {
            let mut o = rule.clone();
            o.id = group_ids::nth(n);
            o.name = name.into();
            o.frame = frame;
            o.constraints = Constraints::default();
            o.style = style;
            o.fill = fill;
            o.kind = kind;
            o
        };
    let rect = || ObjectKind::Rect(RectShape::default());
    let ellipse = || ObjectKind::Ellipse(EllipseShape::default());
    let star = |n: u8| ObjectKind::Polygon(PolygonShape::star(n));
    let line = |dir: LineDir| {
        ObjectKind::Line(Line {
            direction: dir,
            ..Line::default()
        })
    };
    // A group whose box is round its children, as the editor leaves it.
    let group = |n: u128, name: &str, children: Vec<Object>| {
        let frame = children
            .iter()
            .map(Object::outer_box)
            .reduce(|a, b| a.union(&b))
            .unwrap_or(Frame::new(0.0, 0.0, 0.0, 0.0));
        let mut g = mk(
            n,
            name,
            frame,
            Style::None,
            None,
            ObjectKind::Group(GroupShape {
                children,
                extra: Extra::new(),
            }),
        );
        g.style = Style::None;
        g
    };

    // 1. Three levels: Outer holds a box and Middle; Middle holds an ellipse and Inner; Inner holds
    //    a star and a line.
    let inner = group(
        4,
        "Inner group",
        vec![
            mk(
                5,
                "Star",
                Frame::new(34.0, 40.0, 18.0, 18.0),
                emphasis,
                block,
                star(5),
            ),
            mk(
                6,
                "Line",
                Frame::new(32.0, 62.0, 22.0, 6.0),
                structure,
                None,
                line(LineDir::Up),
            ),
        ],
    );
    let middle = group(
        3,
        "Middle group",
        vec![
            mk(
                7,
                "Ellipse",
                Frame::new(22.0, 36.0, 34.0, 26.0),
                structure,
                shade,
                ellipse(),
            ),
            inner,
        ],
    );
    let nested = group(
        1,
        "Outer group",
        vec![
            mk(
                2,
                "Box",
                Frame::new(12.0, 32.0, 54.0, 40.0),
                structure,
                None,
                rect(),
            ),
            middle,
        ],
    );

    // 2. A group turned by 25 degrees holding a rectangle turned by 15, an ellipse and a line.
    let mut turned_child = mk(
        21,
        "Turned child",
        Frame::new(80.0, 40.0, 24.0, 12.0),
        structure,
        shade,
        rect(),
    );
    turned_child.rotation_deg = 15.0;
    let mut turned = group(
        20,
        "Turned group",
        vec![
            turned_child,
            mk(
                22,
                "Disc",
                Frame::new(98.0, 52.0, 16.0, 16.0),
                structure,
                block,
                ellipse(),
            ),
            mk(
                23,
                "Rule",
                Frame::new(80.0, 58.0, 34.0, 0.0),
                emphasis,
                None,
                line(LineDir::Down),
            ),
        ],
    );
    turned.rotation_deg = 25.0;

    // 3. A group at 50 percent: a Block bar and a Shade disc that overlap, and a line across both.
    let mut half = group(
        30,
        "Half group",
        vec![
            mk(
                31,
                "Bar",
                Frame::new(12.0, 96.0, 40.0, 14.0),
                Style::None,
                block,
                rect(),
            ),
            mk(
                32,
                "Disc",
                Frame::new(34.0, 90.0, 26.0, 26.0),
                emphasis,
                shade,
                ellipse(),
            ),
            mk(
                33,
                "Slash",
                Frame::new(14.0, 92.0, 44.0, 22.0),
                emphasis,
                None,
                line(LineDir::Up),
            ),
        ],
    );
    half.opacity_pct = 50;

    // 4. A group whose child is pinned to the top and right: on a wider or taller page it keeps its
    //    size and its distance from those edges, while the child that scales grows with the group.
    let mut pinned_child = mk(
        41,
        "Pinned top right",
        Frame::new(124.0, 92.0, 18.0, 12.0),
        structure,
        block,
        rect(),
    );
    pinned_child.constraints = Constraints {
        h: HConstraint::Right,
        v: VConstraint::Top,
    };
    let pinned = group(
        40,
        "Pinned group",
        vec![
            mk(
                42,
                "Frame",
                Frame::new(76.0, 88.0, 66.0, 38.0),
                structure,
                None,
                rect(),
            ),
            pinned_child,
            mk(
                43,
                "Scaled disc",
                Frame::new(84.0, 100.0, 22.0, 20.0),
                structure,
                shade,
                ellipse(),
            ),
        ],
    );

    // 5. The heavy group: a 8 x 5 grid of small shapes in two sub-groups, to move in the benchmark.
    let mut cells: Vec<Object> = Vec::new();
    let mut n = 101u128;
    for row in 0..5 {
        for col in 0..8 {
            let frame = Frame::new(
                10.0 + f64::from(col) * 16.0,
                150.0 + f64::from(row) * 11.0,
                12.0,
                8.0,
            );
            let kind = match (row + col) % 3 {
                0 => rect(),
                1 => ellipse(),
                _ => star(5),
            };
            let fill = if (row + col) % 2 == 0 { shade } else { None };
            cells.push(mk(n, "Cell", frame, structure, fill, kind));
            n += 1;
        }
    }
    let right = cells.split_off(20);
    let left = group(n, "Heavy left", cells);
    let mut faded = group(n + 1, "Heavy right", right);
    faded.opacity_pct = 80;
    let mut heavy = group(n + 2, "Heavy group", vec![left, faded]);
    heavy.id = group_ids::HEAVY;

    let mut dot_strip = dots.clone();
    dot_strip.frame = Frame::new(8.0, 8.0, 146.0, 18.0);
    dot_strip.constraints = Constraints::default();
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: vec![dot_strip],
            extra: Extra::new(),
        },
        Layer {
            name: "Groups".into(),
            visible: true,
            locked: false,
            objects: vec![nested, turned, half, pinned, heavy],
            extra: Extra::new(),
        },
    ];
    doc
}

/// Fixed ids for the overrides page's objects.
pub mod device_ids {
    use crate::Uuid;
    /// The `n`th object of the overrides page (1-based, groups and children alike); the dot grid is
    /// [`super::ids::DOTS`].
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_000A_4000_8000_0000_0000_0000 + n)
    }
    /// A box moved on the Nomad (a position override).
    pub const MOVED: Uuid = nth(1);
    /// An ellipse sized on the Nomad (a size override).
    pub const SIZED: Uuid = nth(2);
    /// A star hidden on the Nomad (an eye override).
    pub const HIDDEN: Uuid = nth(3);
    /// A text box set smaller on the Nomad (a font-size override).
    pub const SMALL_TEXT: Uuid = nth(4);
    /// A group with an override on the Nomad (its box there).
    pub const GROUP: Uuid = nth(5);
    /// A child of [`GROUP`] with an override of its own.
    pub const GROUP_CHILD: Uuid = nth(7);
    /// A box that runs off the Nomad's page by its constraints (the L15 case).
    pub const OFF_PAGE: Uuid = nth(8);
}

/// The overrides page (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md, Defaulted "Goldens
/// (8a)"): a box with a position override on the Nomad, an ellipse with a size override, a star
/// hidden on the Nomad, a text box with a font-size override, a group with an override whose child
/// has one of its own, and a box that runs off the Nomad's page by its constraints (the L15 case).
/// It holds the sample's dot grid ([`ids::DOTS`]) so the benchmarks can run over it. Designed on
/// the Manta, also targeting the Nomad; the Manta draws the design alone.
pub fn devices_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 8a overrides".into();
    doc.metadata.description =
        "The rung 8a overrides page: what differs on the Nomad, and one thing the constraints get wrong."
            .into();
    let rule = doc.layers[1].objects[1].clone();
    let dots = doc.layers[0].objects[0].clone();
    let title = doc.layers[1].objects[0].clone();
    let structure = Style::Role(LineRole::Structure);
    let emphasis = Style::Role(LineRole::Emphasis);
    let (shade, block) = (
        Some(Fill::Role(FillRole::Shade)),
        Some(Fill::Role(FillRole::Block)),
    );
    let mk =
        |n: u128, name: &str, frame: Frame, style: Style, fill: Option<Fill>, kind: ObjectKind| {
            let mut o = rule.clone();
            o.id = device_ids::nth(n);
            o.name = name.into();
            o.frame = frame;
            o.constraints = Constraints::default();
            o.style = style;
            o.fill = fill;
            o.kind = kind;
            o
        };
    let on_nomad = |o: &mut Object, ov: Override| {
        o.overrides.insert(NOMAD_ID.to_string(), ov);
    };

    // 1. A box moved on the Nomad: only its place is overridden, so it keeps the size it scales to.
    let mut moved = mk(
        1,
        "Moved on Nomad",
        Frame::new(12.0, 34.0, 40.0, 24.0),
        structure,
        shade,
        ObjectKind::Rect(RectShape::default()),
    );
    on_nomad(
        &mut moved,
        Override {
            position: Some(Position {
                x_mm: 20.0,
                y_mm: 62.0,
            }),
            ..Override::default()
        },
    );
    // 2. An ellipse sized on the Nomad: place and size both overridden.
    let mut sized = mk(
        2,
        "Sized on Nomad",
        Frame::new(62.0, 34.0, 40.0, 24.0),
        structure,
        None,
        ObjectKind::Ellipse(EllipseShape::default()),
    );
    on_nomad(
        &mut sized,
        Override {
            position: Some(Position {
                x_mm: 56.0,
                y_mm: 24.0,
            }),
            size: Some(Size {
                w_mm: 36.0,
                h_mm: 26.0,
            }),
            ..Override::default()
        },
    );
    // 3. A star that is not there on the Nomad.
    let mut hidden = mk(
        3,
        "Hidden on Nomad",
        Frame::new(112.0, 34.0, 30.0, 30.0),
        emphasis,
        block,
        ObjectKind::Polygon(PolygonShape::star(5)),
    );
    on_nomad(
        &mut hidden,
        Override {
            visible: Some(false),
            ..Override::default()
        },
    );
    // 4. Text set smaller on the Nomad.
    let mut small = title.clone();
    small.id = device_ids::SMALL_TEXT;
    small.name = "Small on Nomad".into();
    small.frame = Frame::new(12.0, 76.0, 138.0, 18.0);
    small.constraints = Constraints::default();
    if let ObjectKind::Text(t) = &mut small.kind {
        t.content = "Same words, smaller type on the Nomad".into();
        t.size_pt = 16.0;
        t.weight = 500;
    }
    on_nomad(
        &mut small,
        Override {
            font_size_pt: Some(11.0),
            ..Override::default()
        },
    );
    // 5. A group with an override on the Nomad, holding a child with one of its own.
    let mut child = mk(
        7,
        "Group child",
        Frame::new(44.0, 120.0, 30.0, 26.0),
        structure,
        shade,
        ObjectKind::Ellipse(EllipseShape::default()),
    );
    on_nomad(
        &mut child,
        Override {
            position: Some(Position {
                x_mm: 40.0,
                y_mm: 118.0,
            }),
            ..Override::default()
        },
    );
    let kids = vec![
        mk(
            6,
            "Group box",
            Frame::new(12.0, 108.0, 40.0, 30.0),
            structure,
            None,
            ObjectKind::Rect(RectShape::default()),
        ),
        child,
    ];
    let frame = kids
        .iter()
        .map(Object::outer_box)
        .reduce(|a, b| a.union(&b))
        .unwrap_or(Frame::new(0.0, 0.0, 0.0, 0.0));
    let mut group = mk(
        5,
        "Group with override",
        frame,
        Style::None,
        None,
        ObjectKind::Group(GroupShape {
            children: kids,
            extra: Extra::new(),
        }),
    );
    on_nomad(
        &mut group,
        Override {
            position: Some(Position {
                x_mm: 10.0,
                y_mm: 100.0,
            }),
            size: Some(Size {
                w_mm: 66.0,
                h_mm: 36.0,
            }),
            ..Override::default()
        },
    );
    // 6. A box pinned to the left that fits the Manta's page and runs off the Nomad's: the one
    //    thing here that the constraints get wrong, for L15.
    let mut off = mk(
        8,
        "Runs off on Nomad",
        Frame::new(92.0, 150.0, 62.0, 14.0),
        structure,
        block,
        ObjectKind::Rect(RectShape::default()),
    );
    off.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };

    let mut dot_strip = dots;
    dot_strip.frame = Frame::new(8.0, 8.0, 146.0, 18.0);
    dot_strip.constraints = Constraints::default();
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: vec![dot_strip],
            extra: Extra::new(),
        },
        Layer {
            name: "Examples".into(),
            visible: true,
            locked: false,
            objects: vec![moved, sized, hidden, small, group, off],
            extra: Extra::new(),
        },
    ];
    doc
}

/// Fixed ids for the landscape page's objects.
pub mod landscape_ids {
    use crate::Uuid;
    /// The `n`th object of the landscape page (1-based); the dot grid is [`super::ids::DOTS`].
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_000B_4000_8000_0000_0000_0000 + n)
    }
    /// The band across the design's top (the device's right edge, once exported).
    pub const HEADER: Uuid = nth(1);
    /// The title in the band.
    pub const HEADLINE: Uuid = nth(2);
    /// The ruled region.
    pub const RULED: Uuid = nth(3);
    /// The star near the top-right corner: the one shape that shows which way is up.
    pub const STAR: Uuid = nth(4);
    /// A marker along the design's left edge (the device's top toolbar, once exported).
    pub const MARK_LEFT: Uuid = nth(5);
    /// A marker along the design's right edge (the device's footers).
    pub const MARK_RIGHT: Uuid = nth(6);
    /// A marker along the design's bottom edge (the device's left toolbar).
    pub const MARK_BOTTOM: Uuid = nth(7);
}

/// The landscape page (rung 8b, docs/bbq/2026-11-12-rung-8-several-devices.md, Defaulted "8b,
/// orientation" and "Goldens (8b)"): a design that shows clearly which way is up, for the look, the
/// benchmarks (`--landscape-page`) and the goldens. A band with a headline across the top, a dot
/// grid under it, a ruled region, a star near the top-right corner, and a marker just inside each
/// of the left, right and bottom edges, where the device's top toolbar, footers and left toolbar
/// fall once the page is turned (the band is by the right toolbar). Designed on the Manta, also
/// targeting the Nomad, held landscape; the margins (12 mm) clear every turned zone on both.
pub fn landscape_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 8b landscape".into();
    doc.metadata.description =
        "The rung 8b landscape page: a band and headline at the top, a ruled region, a star near the top-right corner and a marker by three of the edges."
            .into();
    doc.page.orientation = Orientation::Landscape;
    let margin = 12.0;
    doc.page.margins_mm = Margins::all(margin);
    // The design's page is the Manta's turned: its height by its width.
    let (w, h) = (MANTA_H_MM, MANTA_W_MM);
    let rule = doc.layers[1].objects[1].clone();
    let dots = doc.layers[0].objects[0].clone();
    let lines = doc.layers[0].objects[1].clone();
    let title = doc.layers[1].objects[0].clone();
    let structure = Style::Role(LineRole::Structure);
    let top_wide = Constraints {
        h: HConstraint::LeftRight,
        v: VConstraint::Top,
    };
    let mk =
        |n: u128, name: &str, frame: Frame, style: Style, fill: Option<Fill>, kind: ObjectKind| {
            let mut o = rule.clone();
            o.id = landscape_ids::nth(n);
            o.name = name.into();
            o.frame = frame;
            o.constraints = Constraints::default();
            o.style = style;
            o.fill = fill;
            o.kind = kind;
            o
        };
    let mut band = mk(
        1,
        "Header band",
        Frame::new(margin, margin, w - 2.0 * margin, 12.0),
        structure,
        Some(Fill::Role(FillRole::Shade)),
        ObjectKind::Rect(RectShape::default()),
    );
    band.constraints = top_wide;
    let mut headline = title;
    headline.id = landscape_ids::HEADLINE;
    headline.name = "Headline".into();
    headline.frame = Frame::new(margin + 3.0, margin + 2.5, w - 2.0 * margin - 6.0, 8.0);
    headline.constraints = top_wide;
    if let ObjectKind::Text(t) = &mut headline.kind {
        t.content = "Landscape: this edge is the top".into();
        t.size_pt = 14.0;
    }
    let mut ruled = lines;
    ruled.id = landscape_ids::RULED;
    ruled.name = "Ruled region".into();
    ruled.frame = Frame::new(
        margin + 12.0,
        62.0,
        w - 2.0 * margin - 24.0,
        h - 62.0 - margin - 12.0,
    );
    ruled.constraints = Constraints {
        h: HConstraint::LeftRight,
        v: VConstraint::TopBottom,
    };
    let star = mk(
        4,
        "Star",
        Frame::new(w - margin - 6.0 - 4.0 - 18.0, margin + 16.0, 18.0, 18.0),
        Style::Role(LineRole::Emphasis),
        Some(Fill::Role(FillRole::Block)),
        ObjectKind::Polygon(PolygonShape::star(5)),
    );
    // Each piece is pinned to the edge it marks, so on the Nomad's smaller page it keeps its
    // distance from that edge and clear of the band, instead of scaling up into it.
    let pinned = |h, v| Constraints { h, v };
    let mut star = star;
    star.constraints = pinned(HConstraint::Right, VConstraint::Top);
    let marker = |n: u128, name: &str, x: f64, y: f64, c: Constraints| {
        let mut o = mk(
            n,
            name,
            Frame::new(x, y, 6.0, 6.0),
            structure,
            Some(Fill::Role(FillRole::Block)),
            ObjectKind::Rect(RectShape::default()),
        );
        o.constraints = c;
        o
    };
    // Footers sit at the device's bottom: along the design's right edge, near its top-right corner.
    let mark_right = marker(
        6,
        "Marker, right edge",
        w - margin - 6.0,
        margin + 18.0,
        pinned(HConstraint::Right, VConstraint::Top),
    );
    let mark_left = marker(
        5,
        "Marker, left edge",
        margin,
        h / 2.0 - 3.0,
        pinned(HConstraint::Left, VConstraint::Center),
    );
    let mark_bottom = marker(
        7,
        "Marker, bottom edge",
        w / 2.0 - 3.0,
        h - margin - 6.0,
        pinned(HConstraint::Center, VConstraint::Bottom),
    );
    let mut dot_strip = dots;
    dot_strip.frame = Frame::new(margin, margin + 16.0, 120.0, 30.0);
    dot_strip.constraints = pinned(HConstraint::Left, VConstraint::Top);
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: vec![dot_strip, ruled],
            extra: Extra::new(),
        },
        Layer {
            name: "Marks".into(),
            visible: true,
            locked: false,
            objects: vec![band, headline, star, mark_left, mark_right, mark_bottom],
            extra: Extra::new(),
        },
    ];
    doc
}

/// Fixed ids for the pattern page's objects.
pub mod pattern_ids {
    use crate::Uuid;
    /// The `n`th object of the pattern page (1-based).
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_000C_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
}

/// What the pattern page's cells hold, in order (left to right, then down): the label and the
/// region.
fn pattern_cells() -> Vec<(&'static str, PatternRegion, LineRole)> {
    let region = |p: Pattern| region_0_19(p);
    let guide = LineRole::Guide;
    let mut unbound = region(Pattern::Grid);
    unbound.bound = false;
    unbound.h_spacing_mm = Some(4.3);
    unbound.v_spacing_mm = Some(7.0);
    let mut own_major = region(Pattern::Dots);
    own_major.major = MajorLines::Every(3);
    own_major.major_role = LineRole::Emphasis;
    let hex = |orientation| Pattern::Hex {
        orientation,
        height_units: 2,
    };
    vec![
        (
            "Isometric, major every 5",
            region(Pattern::Isometric {
                style: IsoStyle::Lines,
            }),
            guide,
        ),
        (
            "Isometric dots, major 5",
            region(Pattern::Isometric {
                style: IsoStyle::Dots,
            }),
            LineRole::DotGrid,
        ),
        ("Hex, flat top", region(hex(HexOrientation::Flat)), guide),
        (
            "Hex, pointy top",
            region(hex(HexOrientation::Pointy)),
            guide,
        ),
        (
            "Cornell",
            region(Pattern::Cornell {
                cue_units: None,
                summary_units: None,
            }),
            guide,
        ),
        (
            "Music staff",
            region(PatternKind::MusicStaff.default_pattern()),
            guide,
        ),
        (
            "Grand staff",
            // Closer lines, so a pair still fits the cell on the smaller page.
            region(Pattern::MusicStaff {
                gap_mm: 1.5,
                pitch_units: 5,
                grand: true,
                grand_space_units: None,
            }),
            guide,
        ),
        ("Grid, own 4.3 x 7 mm", unbound, guide),
        ("Grid, major 5", region(Pattern::Grid), guide),
        ("Dots, own major 3", own_major, LineRole::DotGrid),
    ]
}

/// The rung 9 pattern page (docs/bbq/2026-11-13-rung-9-pattern-library.md, Defaulted "Golden
/// pages"): every new pattern and style in a labelled cell (isometric lines and dots, hexagons
/// flat and pointy, Cornell, a music staff and a grand staff), major lines from the rhythm and
/// from a region of its own, and a region unbound from the rhythm. The rhythm is 5 mm with a major
/// line every 5. It holds a dot region with the sample's id ([`ids::DOTS`]), so the benchmarks run
/// over it. Designed on the Manta, also targeting the Nomad.
pub fn patterns_document() -> Document {
    patterns_page(false)
}

/// The pattern page held sideways (rung 9, one landscape page per device): the same cells in three
/// columns on the Manta's page turned.
pub fn patterns_landscape_document() -> Document {
    patterns_page(true)
}

fn patterns_page(landscape: bool) -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 9 patterns".into();
    doc.metadata.description =
        "The rung 9 pattern page: isometric, hex, Cornell, music staff, major lines and an unbound region."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = Some(5);
    // Columns, the cell's width and height in 5 mm units (the label and the gap below take three more
    // rows). A hexagon at a region's edge is drawn whole, so it pokes out of it by up to half an edge.
    let (cols, cell_w, cell_h) = if landscape {
        doc.page.orientation = Orientation::Landscape;
        (4, 9, 6)
    } else {
        (3, 9, 6)
    };
    let title = doc.layers[1].objects[0].clone();
    let dots = doc.layers[0].objects[0].clone();
    let unit = 5.0;
    let mut objs: Vec<Object> = Vec::new();
    let mut head = title.clone();
    head.id = pattern_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(
        margin,
        margin,
        if landscape { MANTA_H_MM } else { MANTA_W_MM } - 2.0 * margin,
        8.0,
    );
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Rung 9: the pattern library".into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    objs.push(head);
    for (i, (label, region, role)) in pattern_cells().into_iter().enumerate() {
        let (col, row) = ((i % cols) as f64, (i / cols) as f64);
        let x = margin + col * (cell_w as f64 + 1.0) * unit;
        let y = margin + (2.0 + row * (cell_h as f64 + 3.0)) * unit;
        let mut text = title.clone();
        text.id = pattern_ids::nth(100 + i as u128);
        text.name = format!("Label {}", i + 1);
        text.frame = Frame::new(x, y, cell_w as f64 * unit, 5.0);
        if let ObjectKind::Text(t) = &mut text.kind {
            t.content = label.into();
            t.size_pt = 7.0;
            t.weight = 400;
        }
        text.constraints = Constraints::default();
        objs.push(text);
        let mut o = dots.clone();
        // The sample's own id goes to the last dot region, so the drag benchmark finds it.
        o.id = if i == 9 {
            ids::DOTS
        } else {
            pattern_ids::nth(1 + i as u128)
        };
        o.name = label.into();
        // A millimetre past the rhythm lines at each edge, so the edge lines are in whatever a
        // pixel of rounding says.
        o.frame = Frame::new(
            x - 1.0,
            y + 2.0 * unit - 1.0,
            cell_w as f64 * unit + 2.0,
            cell_h as f64 * unit + 2.0,
        );
        o.constraints = Constraints::default();
        o.style = Style::Role(role);
        o.kind = ObjectKind::Pattern(region);
        objs.push(o);
    }
    doc.layers = vec![Layer {
        name: "Patterns".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the turned-patterns page's objects.
pub mod turned_ids {
    use crate::Uuid;
    /// The `n`th object of the turned-patterns page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0010_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// The grid turned by 45 degrees, which `--bench-rotate` turns on this page.
    pub const TURNED: Uuid = nth(3);
    /// The group holding a turned pattern region.
    pub const GROUP: Uuid = nth(11);
    /// The repeat of a turned pattern region.
    pub const REPEAT: Uuid = nth(12);
}

/// The rung 12b turned-patterns page (docs/bbq/2026-11-16-rung-12-missing-fonts.md, Q8): pattern
/// regions turned whole, one in each of a dozen labelled cells: lines, dots, grid with major lines,
/// isometric lines and dots, hexagons flat and pointy, Cornell, a grand staff, a region of its own
/// spacing, a group holding a turned region (its angle and the group's add) and a repeat of one (each
/// copy turns). One is turned by a quarter turn, which stays on whole pixels. The rhythm is 5 mm with
/// a major line every 5. A dot region holds the sample's id ([`ids::DOTS`]), so the benchmarks run
/// over it. Designed on the Manta, also targeting the Nomad.
pub fn turned_patterns_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 12b turned patterns".into();
    doc.metadata.description =
        "The rung 12b turned-patterns page: every pattern turned, in a group and in a repeat."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = Some(5);
    let title = doc.layers[1].objects[0].clone();
    let unit = 5.0;
    let (cols, cell_w) = (3, 9.0);
    let region = |p: Pattern| region_0_19(p);
    let hex = |orientation| Pattern::Hex {
        orientation,
        height_units: 2,
    };
    let mut unbound = region(Pattern::Grid);
    unbound.bound = false;
    unbound.h_spacing_mm = Some(4.3);
    unbound.v_spacing_mm = Some(7.0);
    let guide = LineRole::Guide;
    let dotted = LineRole::DotGrid;
    // (label, region, role, degrees)
    let cells: Vec<(&str, Option<PatternRegion>, LineRole, f64)> = vec![
        ("Lines, 15°", Some(region(Pattern::Lines)), guide, 15.0),
        ("Dots, 30°", Some(region(Pattern::Dots)), dotted, 30.0),
        (
            "Grid, major 5, 45°",
            Some(region(Pattern::Grid)),
            guide,
            45.0,
        ),
        (
            "Isometric, 90° (sharp)",
            Some(region(Pattern::Isometric {
                style: IsoStyle::Lines,
            })),
            guide,
            90.0,
        ),
        (
            "Isometric dots, 60°",
            Some(region(Pattern::Isometric {
                style: IsoStyle::Dots,
            })),
            dotted,
            60.0,
        ),
        (
            "Hex, flat top, 20°",
            Some(region(hex(HexOrientation::Flat))),
            guide,
            20.0,
        ),
        (
            "Hex, pointy top, 335°",
            Some(region(hex(HexOrientation::Pointy))),
            guide,
            335.0,
        ),
        (
            "Cornell, 10°",
            Some(region(Pattern::Cornell {
                cue_units: None,
                summary_units: None,
            })),
            guide,
            10.0,
        ),
        (
            "Grand staff, 352°",
            Some(region(Pattern::MusicStaff {
                gap_mm: 1.5,
                pitch_units: 5,
                grand: true,
                grand_space_units: None,
            })),
            guide,
            352.0,
        ),
        ("Grid, own 4.3 x 7 mm, 12°", Some(unbound), guide, 12.0),
        ("In a group (10° + 15°)", None, guide, 10.0),
        ("In a repeat, 3 copies, 25°", None, dotted, 25.0),
    ];
    let mut head = title.clone();
    head.id = turned_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, MANTA_W_MM - 2.0 * margin, 8.0);
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Rung 12b: turned patterns".into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    let mut objs: Vec<Object> = vec![head];
    for (i, (label, pr, role, deg)) in cells.into_iter().enumerate() {
        let (col, row) = ((i % cols) as f64, (i / cols) as f64);
        let x = margin + col * (cell_w + 1.0) * unit;
        let y = margin + (2.0 + row * 9.0) * unit;
        let mut text = title.clone();
        text.id = turned_ids::nth(100 + i as u128);
        text.name = format!("Label {}", i + 1);
        text.frame = Frame::new(x, y, cell_w * unit, 5.0);
        if let ObjectKind::Text(t) = &mut text.kind {
            t.content = label.into();
            t.size_pt = 7.0;
            t.weight = 400;
        }
        text.constraints = Constraints::default();
        objs.push(text);
        // A 30 x 20 mm region round the middle of the cell's 45 x 30 mm.
        let (cx, cy) = (x + cell_w * unit / 2.0, y + 2.0 * unit + 15.0);
        let id = match i {
            1 => ids::DOTS,
            2 => turned_ids::TURNED,
            _ => turned_ids::nth(1 + i as u128),
        };
        let pattern_at = |id: Uuid, name: &str, frame: Frame, pr: PatternRegion, deg: f64| {
            let mut o = object(
                id,
                name,
                frame,
                Constraints::default(),
                Style::Role(role),
                ObjectKind::Pattern(pr),
            );
            o.rotation_deg = deg;
            o
        };
        match pr {
            Some(pr) => {
                // A staff needs room for its five lines twice over.
                let frame = if matches!(pr.pattern, Pattern::MusicStaff { .. }) {
                    Frame::new(cx - 17.0, cy - 14.0, 34.0, 28.0)
                } else {
                    Frame::new(cx - 15.0, cy - 10.0, 30.0, 20.0)
                };
                objs.push(pattern_at(id, label, frame, pr, deg));
            }
            // A group of a grid (turned by 10) and a plain frame round it; the group turns by 15.
            None if i == 10 => {
                let grid = pattern_at(
                    turned_ids::nth(20),
                    "Grid in the group",
                    Frame::new(cx - 12.0, cy - 8.0, 24.0, 16.0),
                    region(Pattern::Grid),
                    deg,
                );
                let mut frame = object(
                    turned_ids::nth(21),
                    "Frame in the group",
                    Frame::new(cx - 14.0, cy - 10.0, 28.0, 20.0),
                    Constraints::default(),
                    Style::Role(LineRole::Structure),
                    ObjectKind::Rect(RectShape::default()),
                );
                frame.constraints = Constraints::default();
                let children = vec![frame, grid];
                let bounds = children
                    .iter()
                    .map(Object::outer_box)
                    .reduce(|a, b| a.union(&b))
                    .unwrap_or(Frame::new(0.0, 0.0, 0.0, 0.0));
                let mut g = object(
                    turned_ids::GROUP,
                    "Turned group with a pattern",
                    bounds,
                    Constraints::default(),
                    Style::None,
                    ObjectKind::Group(GroupShape {
                        children,
                        extra: Extra::new(),
                    }),
                );
                g.rotation_deg = 15.0;
                objs.push(g);
            }
            // A repeat of a small dot region turned by 25: three copies, each turned.
            None => {
                let dots = pattern_at(
                    turned_ids::nth(22),
                    "Dots in the repeat",
                    Frame::new(x + 2.0, cy - 9.0, 10.0, 18.0),
                    region(Pattern::Dots),
                    deg,
                );
                let mut shape = RepeatShape::new(dots);
                shape.count = 3;
                shape.h_spacing = Spacing::Mm(14.0);
                let mut o = object(
                    turned_ids::REPEAT,
                    "Repeat of a turned region",
                    Frame::new(0.0, 0.0, 1.0, 1.0),
                    Constraints::default(),
                    Style::None,
                    ObjectKind::Repeat(shape),
                );
                if let Some(f) = o.repeat_fit((unit, unit)) {
                    o.frame = f;
                }
                objs.push(o);
            }
        }
    }
    doc.layers = vec![Layer {
        name: "Turned patterns".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the pattern-angle page's objects.
pub mod angle_ids {
    use crate::Uuid;
    /// The `n`th object of the pattern-angle page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0011_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// The grid whose ruling is turned by 30 degrees in an upright frame.
    pub const GRID: Uuid = nth(3);
    /// The group holding a region with a ruling angle.
    pub const GROUP: Uuid = nth(10);
    /// The repeat of a region with a ruling angle.
    pub const REPEAT: Uuid = nth(11);
}

/// The rung 12b pattern-angle page (docs/bbq/2026-11-16-rung-12-missing-fonts.md, Q8 as amended, Q10,
/// Q11): the ruling turned inside its frame and cut at it, in twelve labelled cells: lines, dots, a grid
/// with major lines, hexagons and isometric lines in an upright frame; a turned frame whose ruling is turned
/// back to level, a turned frame and ruling that add, a ruling turned a quarter (sharp); a Cornell page
/// and a grand staff (laid out in their frame, turned with it); a group and a repeat. The rhythm is 5 mm
/// with a major line every 5. A dot region holds the sample's id ([`ids::DOTS`]), so the benchmarks run over
/// it. Designed on the Manta, also targeting the Nomad.
pub fn pattern_angle_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 12b pattern angle".into();
    doc.metadata.description =
        "The rung 12b pattern-angle page: the ruling turned inside its frame, cut at the frame."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = Some(5);
    let title = doc.layers[1].objects[0].clone();
    let unit = 5.0;
    let (cols, cell_w) = (3, 9.0);
    let region = |p: Pattern, ruling: f64| {
        let mut pr = region_0_19(p);
        pr.angle_deg = ruling;
        pr
    };
    let hex = Pattern::Hex {
        orientation: HexOrientation::Flat,
        height_units: 2,
    };
    let guide = LineRole::Guide;
    let dotted = LineRole::DotGrid;
    // (label, region, role, frame degrees)
    let cells: Vec<(&str, Option<PatternRegion>, LineRole, f64)> = vec![
        (
            "Lines, ruling 30°",
            Some(region(Pattern::Lines, 30.0)),
            guide,
            0.0,
        ),
        (
            "Dots, ruling 45°",
            Some(region(Pattern::Dots, 45.0)),
            dotted,
            0.0,
        ),
        (
            "Grid, major 5, ruling 30°",
            Some(region(Pattern::Grid, 30.0)),
            guide,
            0.0,
        ),
        (
            "Hex, flat top, ruling 15°",
            Some(region(hex, 15.0)),
            guide,
            0.0,
        ),
        (
            "Isometric, ruling 10°",
            Some(region(
                Pattern::Isometric {
                    style: IsoStyle::Lines,
                },
                10.0,
            )),
            guide,
            0.0,
        ),
        (
            "Cornell, ruling 10°",
            Some(region(
                Pattern::Cornell {
                    cue_units: None,
                    summary_units: None,
                },
                10.0,
            )),
            guide,
            0.0,
        ),
        (
            "Frame 20°, ruling 340° (level)",
            Some(region(Pattern::Lines, 340.0)),
            guide,
            20.0,
        ),
        (
            "Frame 45° + ruling 45° (add)",
            Some(region(Pattern::Grid, 45.0)),
            guide,
            45.0,
        ),
        (
            "Lines, ruling 90° (sharp)",
            Some(region(Pattern::Lines, 90.0)),
            guide,
            0.0,
        ),
        (
            "Grand staff, ruling 8°",
            Some(region(
                Pattern::MusicStaff {
                    gap_mm: 1.5,
                    pitch_units: 5,
                    grand: true,
                    grand_space_units: None,
                },
                8.0,
            )),
            guide,
            0.0,
        ),
        ("In a group (frame 15° + ruling 30°)", None, guide, 15.0),
        ("In a repeat, 3 copies, ruling 45°", None, dotted, 0.0),
    ];
    let mut head = title.clone();
    head.id = angle_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, MANTA_W_MM - 2.0 * margin, 8.0);
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Rung 12b: pattern angle".into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    let mut objs: Vec<Object> = vec![head];
    for (i, (label, pr, role, frame_deg)) in cells.into_iter().enumerate() {
        let (col, row) = ((i % cols) as f64, (i / cols) as f64);
        let x = margin + col * (cell_w + 1.0) * unit;
        let y = margin + (2.0 + row * 9.0) * unit;
        let mut text = title.clone();
        text.id = angle_ids::nth(100 + i as u128);
        text.name = format!("Label {}", i + 1);
        text.frame = Frame::new(x, y, cell_w * unit, 5.0);
        if let ObjectKind::Text(t) = &mut text.kind {
            t.content = label.into();
            t.size_pt = 7.0;
            t.weight = 400;
        }
        text.constraints = Constraints::default();
        objs.push(text);
        let (cx, cy) = (x + cell_w * unit / 2.0, y + 2.0 * unit + 15.0);
        let id = match i {
            1 => ids::DOTS,
            2 => angle_ids::GRID,
            _ => angle_ids::nth(1 + i as u128),
        };
        let pattern_at = |id: Uuid, name: &str, frame: Frame, pr: PatternRegion, deg: f64| {
            let mut o = object(
                id,
                name,
                frame,
                Constraints::default(),
                Style::Role(role),
                ObjectKind::Pattern(pr),
            );
            o.rotation_deg = deg;
            o
        };
        match pr {
            Some(pr) => {
                let frame = if matches!(pr.pattern, Pattern::MusicStaff { .. }) {
                    Frame::new(cx - 17.0, cy - 14.0, 34.0, 28.0)
                } else {
                    Frame::new(cx - 15.0, cy - 10.0, 30.0, 20.0)
                };
                objs.push(pattern_at(id, label, frame, pr, frame_deg));
            }
            // A group of a grid with a ruling angle of 30 and a plain frame round it; the group turns by 15.
            None if i == 10 => {
                let grid = pattern_at(
                    angle_ids::nth(20),
                    "Grid in the group",
                    Frame::new(cx - 12.0, cy - 8.0, 24.0, 16.0),
                    region(Pattern::Grid, 30.0),
                    0.0,
                );
                let frame = object(
                    angle_ids::nth(21),
                    "Frame in the group",
                    Frame::new(cx - 14.0, cy - 10.0, 28.0, 20.0),
                    Constraints::default(),
                    Style::Role(LineRole::Structure),
                    ObjectKind::Rect(RectShape::default()),
                );
                let children = vec![frame, grid];
                let bounds = children
                    .iter()
                    .map(Object::outer_box)
                    .reduce(|a, b| a.union(&b))
                    .unwrap_or(Frame::new(0.0, 0.0, 0.0, 0.0));
                let mut g = object(
                    angle_ids::GROUP,
                    "Group with a ruling angle",
                    bounds,
                    Constraints::default(),
                    Style::None,
                    ObjectKind::Group(GroupShape {
                        children,
                        extra: Extra::new(),
                    }),
                );
                g.rotation_deg = frame_deg;
                objs.push(g);
            }
            // A repeat of a small dot region with a ruling angle of 45: three copies.
            None => {
                let dots = pattern_at(
                    angle_ids::nth(22),
                    "Dots in the repeat",
                    Frame::new(x + 2.0, cy - 9.0, 10.0, 18.0),
                    region(Pattern::Dots, 45.0),
                    frame_deg,
                );
                let mut shape = RepeatShape::new(dots);
                shape.count = 3;
                shape.h_spacing = Spacing::Mm(14.0);
                let mut o = object(
                    angle_ids::REPEAT,
                    "Repeat of a region with a ruling angle",
                    Frame::new(0.0, 0.0, 1.0, 1.0),
                    Constraints::default(),
                    Style::None,
                    ObjectKind::Repeat(shape),
                );
                if let Some(f) = o.repeat_fit((unit, unit)) {
                    o.frame = f;
                }
                objs.push(o);
            }
        }
    }
    doc.layers = vec![Layer {
        name: "Pattern angle".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the pattern-borders page's objects.
pub mod border_ids {
    use crate::Uuid;
    /// The `n`th object of the pattern-borders page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0015_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
}

/// The pattern-borders page (docs/bbq/2026-11-18-pattern-borders.md): a border round a pattern region, in twelve
/// labelled cells. Ruled lines with all four sides; a dot grid with its right side off (it holds the sample's id,
/// [`ids::DOTS`], so the benchmarks run over it); a grid, lines and dots with rounded corners; a grid bordered on
/// two sides only (one rounded corner); an isometric pattern and hexagons with rounded corners; a border of a
/// custom width and gray; a Cornell page; a turned frame; a ruling turned 30 degrees and one turned a quarter.
/// Each frame sits on the 5 mm rhythm, so the lines and dots on its edges are the ones the border replaces. Every
/// object keeps its millimetres on the Nomad (Left and Top), so a page of 25 by 15 mm frames fits both screens.
/// Designed on the Manta, also targeting the Nomad.
pub fn pattern_borders_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Pattern borders".into();
    doc.metadata.description =
        "A border round a pattern region: sides, line, rounded corners, turned frames and rulings."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    // Frames are set in millimetres here, so the rhythm is too: a pixel-locked step (59 px for 5 mm, where 5 mm is
    // 59.06) drifts a pixel or two from a frame's edge a hundred millimetres along, and the app's own snapping
    // would have placed such a frame on the pixel rhythm.
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = Some(5);
    let title = doc.layers[1].objects[0].clone();
    let keep = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    let (cols, pitch_x, pitch_y) = (3, 35.0, 30.0);
    let bordered = |sides: BorderSides, style: Style, radius: f64| {
        let mut b = Border::new();
        b.sides = sides;
        b.style = style;
        b.corner_radius_mm = radius;
        b
    };
    let structure = Style::Role(LineRole::Structure);
    let region = |p: Pattern, border: Border, ruling: f64| {
        let mut pr = region_0_19(p);
        pr.angle_deg = ruling;
        pr.border = Some(border);
        pr
    };
    let hex = Pattern::Hex {
        orientation: HexOrientation::Flat,
        height_units: 1,
    };
    let iso = Pattern::Isometric {
        style: IsoStyle::Lines,
    };
    let no_right = BorderSides {
        right: false,
        ..BorderSides::ALL
    };
    let top_left = BorderSides {
        top: true,
        left: true,
        right: false,
        bottom: false,
    };
    let custom = Style::Custom {
        width_mm: 0.4,
        gray: 64,
    };
    // (label, region, role, frame degrees)
    let cells: Vec<(&str, PatternRegion, LineRole, f64)> = vec![
        (
            "Lines, all four sides",
            region(Pattern::Lines, Border::new(), 0.0),
            LineRole::Guide,
            0.0,
        ),
        (
            "Dots, right side off",
            region(Pattern::Dots, bordered(no_right, structure, 0.0), 0.0),
            LineRole::DotGrid,
            0.0,
        ),
        (
            "Grid, rounded corners 3 mm",
            region(
                Pattern::Grid,
                bordered(BorderSides::ALL, structure, 3.0),
                0.0,
            ),
            LineRole::Guide,
            0.0,
        ),
        (
            "Lines, Emphasis, rounded 4 mm",
            region(
                Pattern::Lines,
                bordered(BorderSides::ALL, Style::Role(LineRole::Emphasis), 4.0),
                0.0,
            ),
            LineRole::Guide,
            0.0,
        ),
        (
            "Dots, rounded 4 mm",
            region(
                Pattern::Dots,
                bordered(BorderSides::ALL, structure, 4.0),
                0.0,
            ),
            LineRole::DotGrid,
            0.0,
        ),
        (
            "Grid, top and left, 3 mm",
            region(Pattern::Grid, bordered(top_left, structure, 3.0), 0.0),
            LineRole::Guide,
            0.0,
        ),
        (
            "Isometric, 0.4 mm gray 64, rounded 3 mm",
            region(iso, bordered(BorderSides::ALL, custom, 3.0), 0.0),
            LineRole::Guide,
            0.0,
        ),
        (
            "Hex, rounded 3 mm",
            region(hex, bordered(BorderSides::ALL, structure, 3.0), 0.0),
            LineRole::Guide,
            0.0,
        ),
        (
            "Cornell, all four sides",
            region(
                Pattern::Cornell {
                    cue_units: None,
                    summary_units: None,
                },
                Border::new(),
                0.0,
            ),
            LineRole::Guide,
            0.0,
        ),
        (
            "Frame 20°, grid, rounded 3 mm",
            region(
                Pattern::Grid,
                bordered(BorderSides::ALL, structure, 3.0),
                0.0,
            ),
            LineRole::Guide,
            20.0,
        ),
        (
            "Ruling 30°, lines, rounded 4 mm",
            region(
                Pattern::Lines,
                bordered(BorderSides::ALL, structure, 4.0),
                30.0,
            ),
            LineRole::Guide,
            0.0,
        ),
        (
            "Ruling 90°, own 5 mm, all four sides",
            {
                // Its own spacing counts from the frame's corner, so the turned lines meet the frame's edges.
                let mut pr = region(Pattern::Lines, Border::new(), 90.0);
                pr.set_bound(false, (5.0, 5.0));
                pr
            },
            LineRole::Guide,
            0.0,
        ),
    ];
    let mut head = title.clone();
    head.id = border_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, MANTA_W_MM - 2.0 * margin, 8.0);
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Pattern borders".into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    let mut objs: Vec<Object> = vec![head];
    for (i, (label, pr, role, frame_deg)) in cells.into_iter().enumerate() {
        let (col, row) = ((i % cols) as f64, (i / cols) as f64);
        let x = margin + col * pitch_x;
        let y = margin + 10.0 + row * pitch_y;
        let mut text = title.clone();
        text.id = border_ids::nth(100 + i as u128);
        text.name = format!("Label {}", i + 1);
        text.frame = Frame::new(x, y, 33.0, 6.0);
        if let ObjectKind::Text(t) = &mut text.kind {
            t.content = label.into();
            t.size_pt = 7.0;
            t.weight = 400;
        }
        text.constraints = keep;
        objs.push(text);
        let id = if i == 1 {
            ids::DOTS
        } else {
            border_ids::nth(1 + i as u128)
        };
        let mut o = object(
            id,
            label,
            Frame::new(x, y + 10.0, 25.0, 15.0),
            keep,
            Style::Role(role),
            ObjectKind::Pattern(pr),
        );
        o.rotation_deg = frame_deg;
        objs.push(o);
    }
    doc.layers = vec![Layer {
        name: "Pattern borders".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the major-lines-from page's objects.
pub mod majors_ids {
    use crate::Uuid;
    /// The `n`th object of the major-lines-from page (1-based: row by row, Top, Centre, Rhythm origin).
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0017_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// The ruled lines counted from their top: the one the look picks.
    pub const FIRST: Uuid = nth(1);
}

/// Fixed ids for the major-lines-per-axis page's objects.
pub mod majors_axes_ids {
    use crate::Uuid;
    /// The `n`th object of the major-lines-per-axis page (1-based: row by row).
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0018_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
}

/// The major-lines-from page (ADR-023; docs/bbq/2026-11-23-major-lines-from-top.md): three columns, Top, Centre and
/// Rhythm origin, rows and columns alike, of six labelled cells each: ruled lines, a grid, dots, Cornell, isometric
/// lines (which has no Top: its Top cell is set to Top and draws as Rhythm origin) and a grid with a border on all four
/// sides (the border stands on the first major line). The rhythm is 3 mm, physically locked, with a major line every 7;
/// each region holds 11 lines across and 5 down and starts one rhythm row past a major, so the three choices put their
/// majors in three different places, and Centre leaves 1 line before its first major column and 2 after its last.
/// Every object keeps its millimetres on the Nomad (Left and Top), so the page is the same on both screens. Designed on
/// the Manta, also targeting the Nomad.
pub fn majors_document() -> Document {
    let froms = [
        (MajorFrom::Top, "Top"),
        (MajorFrom::Centre, "Centre"),
        (MajorFrom::RhythmOrigin, "Rhythm origin"),
    ];
    let mut bordered = PatternRegion::new(Pattern::Grid);
    bordered.border = Some(Border::new());
    let rows: [(&str, PatternRegion, LineRole); 6] = [
        ("Lines", PatternRegion::new(Pattern::Lines), LineRole::Guide),
        ("Grid", PatternRegion::new(Pattern::Grid), LineRole::Guide),
        ("Dots", PatternRegion::new(Pattern::Dots), LineRole::DotGrid),
        (
            "Cornell",
            PatternRegion::new(Pattern::Cornell {
                cue_units: None,
                summary_units: None,
            }),
            LineRole::Guide,
        ),
        (
            "Isometric",
            PatternRegion::new(Pattern::Isometric {
                style: IsoStyle::Lines,
            }),
            LineRole::Guide,
        ),
        ("Grid, border", bordered, LineRole::Guide),
    ];
    let cells = rows
        .iter()
        .map(|(name, region, role)| {
            froms
                .iter()
                .map(|&(from, from_name)| {
                    let label = if from == MajorFrom::Top && !region.pattern.offers_major_top() {
                        format!("{name}, Top: from origin")
                    } else {
                        format!("{name}, {from_name}")
                    };
                    let pr = PatternRegion {
                        major_rows_from: from,
                        major_columns_from: from,
                        ..region.clone()
                    };
                    (label, pr, *role)
                })
                .collect()
        })
        .collect();
    majors_page(
        "Major lines from",
        "Major lines counted from a region's top, its centre or the rhythm origin, on five patterns and a border.",
        "Major lines from: Top, Centre, Rhythm origin",
        7.0,
        majors_ids::nth,
        cells,
    )
}

/// The major-lines-per-axis page (ADR-023 as amended 2026-11-23): the rows and the columns of one region counted from
/// different places, on a grid, dots and isometric lines (Lines and Cornell have rows only). Each row of the page mixes
/// two of Top, Centre and Rhythm origin a different way; isometric, which has no Top, takes Rhythm origin where the
/// others take Top. Same rhythm, cells and constraints as [`majors_document`].
pub fn majors_axes_document() -> Document {
    use MajorFrom::{Centre, RhythmOrigin as Origin, Top};
    let name = |f: MajorFrom| match f {
        Top => "Top",
        Centre => "Centre",
        Origin => "Origin",
    };
    let mixes = [(Top, Centre), (Centre, Origin), (Origin, Top)];
    let patterns = [
        ("Grid", Pattern::Grid, LineRole::Guide),
        ("Dots", Pattern::Dots, LineRole::DotGrid),
        (
            "Iso",
            Pattern::Isometric {
                style: IsoStyle::Lines,
            },
            LineRole::Guide,
        ),
    ];
    let cells = mixes
        .iter()
        .map(|&mix| {
            patterns
                .iter()
                .map(|&(pname, pattern, role)| {
                    let region = PatternRegion::new(pattern);
                    let rows = pattern.drawn_major_from(mix.0);
                    let cols = pattern.drawn_major_from(mix.1);
                    let label = format!("{pname}: rows {}, cols {}", name(rows), name(cols));
                    let pr = PatternRegion {
                        major_rows_from: rows,
                        major_columns_from: cols,
                        ..region
                    };
                    (label, pr, role)
                })
                .collect()
        })
        .collect();
    majors_page(
        "Major lines per axis",
        "A region's major rows and major columns counted from different places: top, centre or the rhythm origin.",
        "Major rows and columns, each their own",
        6.0,
        majors_axes_ids::nth,
        cells,
    )
}

/// A page of labelled pattern cells in rows, for [`majors_document`] and [`majors_axes_document`]: the rhythm is 3 mm,
/// physically locked, with a major line every 7; a cell is 10 units wide and 4 down, a unit apart across; a row is the
/// label (2 units), the cell and a unit's gap. Cell `i` (row by row, from 0) is object `nth(1 + i)` and its label
/// `nth(100 + i)`; the title is `nth(200)`.
fn majors_page(
    title_text: &str,
    description: &str,
    heading: &str,
    label_pt: f64,
    nth: fn(u128) -> Uuid,
    cells: Vec<Vec<(String, PatternRegion, LineRole)>>,
) -> Document {
    let mut doc = slice_document();
    doc.metadata.title = title_text.into();
    doc.metadata.description = description.into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    doc.rhythm.h_unit_mm = 3.0;
    doc.rhythm.v_unit_mm = 3.0;
    doc.rhythm.major_every = Some(7);
    let title = doc.layers[1].objects[0].clone();
    let keep = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    let (unit, cell_w, cell_h, gap, title_units) = (3.0, 10.0, 4.0, 1.0, 4.0);
    let mut head = title.clone();
    head.id = nth(200);
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, MANTA_W_MM - 2.0 * margin, 8.0);
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = heading.into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    let mut objs: Vec<Object> = vec![head];
    let mut i = 0;
    for (r, row) in cells.into_iter().enumerate() {
        for (c, (label, pr, role)) in row.into_iter().enumerate() {
            let x = margin + c as f64 * (cell_w + gap) * unit;
            let y = margin + (title_units + r as f64 * (cell_h + 3.0)) * unit;
            let mut text = title.clone();
            text.id = nth(100 + i as u128);
            text.name = format!("Label {}", i + 1);
            text.frame = Frame::new(x, y, cell_w * unit, 5.0);
            if let ObjectKind::Text(t) = &mut text.kind {
                t.content = label.clone();
                t.size_pt = label_pt;
                t.weight = 400;
            }
            text.constraints = keep;
            objs.push(text);
            // A millimetre past the rhythm lines at each edge, so the edge lines are in whatever a pixel of
            // rounding says; a bordered frame sits on them, so its border stands on the edge lines (Q7).
            let past = if pr.border.is_some() { 0.0 } else { 1.0 };
            objs.push(object(
                nth(1 + i as u128),
                &label,
                Frame::new(
                    x - past,
                    y + 2.0 * unit - past,
                    cell_w * unit + 2.0 * past,
                    cell_h * unit + 2.0 * past,
                ),
                keep,
                Style::Role(role),
                ObjectKind::Pattern(pr),
            ));
            i += 1;
        }
    }
    doc.layers = vec![Layer {
        name: title_text.into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the grand-staff page's objects.
pub mod grand_ids {
    use crate::Uuid;
    /// The `n`th object of the grand-staff page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0016_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// The first grand staff (space 2 units, the app's default line gap): the one the look picks.
    pub const FIRST: Uuid = nth(1);
}

/// The grand-staff page (docs/bbq/2026-11-18-fixes-before-16b.md, Q2): six labelled cells of music staves. Four grand
/// staves with a space of their own between them (2 units; 1 unit; 3 units at a 1.5 mm line gap; 2 units at a 1 mm line gap, to
/// show the space holding while the line gap changes), a grand staff from an older file with no space field (`Staff every`
/// 6 units, top to top), and a single staff every 3 units. Each frame sits on the 5 mm rhythm. Every object keeps its
/// millimetres on the Nomad (Left and Top), so a page of 31 by 60 mm frames fits both screens. Designed on the Manta,
/// also targeting the Nomad.
pub fn grand_staff_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Grand staves".into();
    doc.metadata.description =
        "Music staves: grand staves with a space between them, one from an older file, a single staff.".into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = Some(5);
    let title = doc.layers[1].objects[0].clone();
    let keep = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    let (cols, pitch_x, pitch_y) = (3, 34.0, 68.0);
    let staff = |gap_mm: f64, pitch_units: u32, grand: bool, space: Option<u32>| {
        region_0_19(Pattern::MusicStaff {
            gap_mm,
            pitch_units,
            grand,
            grand_space_units: space,
        })
    };
    let cells: Vec<(&str, PatternRegion)> = vec![
        ("Grand, space 2 u", staff(2.0, 3, true, Some(2))),
        ("Grand, space 1 u", staff(2.0, 3, true, Some(1))),
        ("Line gap 1.5, space 3 u", staff(1.5, 3, true, Some(3))),
        ("Line gap 1, space 2 u", staff(1.0, 3, true, Some(2))),
        ("Old file, every 6 u", staff(2.0, 6, true, None)),
        ("Single, every 3 u", staff(2.0, 3, false, None)),
    ];
    let mut head = title.clone();
    head.id = grand_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, MANTA_W_MM - 2.0 * margin, 8.0);
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Grand staves".into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    let mut objs: Vec<Object> = vec![head];
    for (i, (label, pr)) in cells.into_iter().enumerate() {
        let (col, row) = ((i % cols) as f64, (i / cols) as f64);
        let x = margin + col * pitch_x;
        let y = margin + 10.0 + row * pitch_y;
        let mut text = title.clone();
        text.id = grand_ids::nth(100 + i as u128);
        text.name = format!("Label {}", i + 1);
        text.frame = Frame::new(x, y, 33.0, 6.0);
        if let ObjectKind::Text(t) = &mut text.kind {
            t.content = label.into();
            t.size_pt = 7.0;
            t.weight = 400;
        }
        text.constraints = keep;
        objs.push(text);
        objs.push(object(
            grand_ids::nth(1 + i as u128),
            label,
            Frame::new(x, y + 7.0, 31.0, 60.0),
            keep,
            Style::Role(LineRole::Guide),
            ObjectKind::Pattern(pr),
        ));
    }
    doc.layers = vec![Layer {
        name: "Grand staves".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the calendar page's objects.
pub mod calendar_ids {
    use crate::Uuid;
    /// The `n`th object of the calendar page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0016_5000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// The dated calendar (November 2026, Monday first): the one the look picks.
    pub const DATED: Uuid = nth(1);
    /// The undated calendar (31 days, the 1st on a Wednesday, Sunday first).
    pub const UNDATED: Uuid = nth(2);
    /// The mini calendar of November 2026, with its title.
    pub const MINI: Uuid = nth(3);
    /// The mini calendar of February 2028, Sunday first, without its title.
    pub const MINI_PLAIN: Uuid = nth(4);
}

/// The calendar page (docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md): a dated calendar pattern
/// (November 2026, weeks starting on Monday: six rows, the 1st in the last column), an undated one (31 days, the
/// 1st on a Wednesday, weeks starting on Sunday: five rows, a square in each day), and two mini calendars
/// (November 2026 with its title; February 2028, a leap month, Sunday first, without one), each under a label.
/// Every object keeps its millimetres on the Nomad (Left and Top), on the 5 mm rhythm. Designed on the Manta, also
/// targeting the Nomad.
pub fn calendar_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Calendars".into();
    doc.metadata.description =
        "Calendar patterns, dated and undated, and two mini calendars.".into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    let title = doc.layers[1].objects[0].clone();
    let keep = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    let label = |n: u128, text: &str, x: f64, y: f64, w: f64| {
        let mut t = title.clone();
        t.id = calendar_ids::nth(100 + n);
        t.name = format!("Label {n}");
        t.frame = Frame::new(x, y, w, 4.0);
        if let ObjectKind::Text(tb) = &mut t.kind {
            tb.content = text.into();
            tb.size_pt = 7.0;
            tb.weight = 400;
        }
        t.constraints = keep;
        t
    };
    let mut head = title.clone();
    head.id = calendar_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, MANTA_W_MM - 2.0 * margin, 8.0);
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Calendars".into();
        t.size_pt = 12.0;
    }
    head.constraints = Constraints::default();
    let month = |m: Option<YearMonth>, week_start: Weekday, first_day: Weekday, days: u8| {
        ObjectKind::Pattern(region_0_19(Pattern::Calendar {
            month: m,
            week_start,
            first_day,
            days,
        }))
    };
    let mini = |m: YearMonth, week_start: Weekday, title: bool| {
        let mut c = MiniCalendarShape::new(m);
        c.week_start = week_start;
        c.title = title;
        ObjectKind::MiniCalendar(c)
    };
    let right = MANTA_W_MM.min(118.9) - margin;
    let full = 102.0;
    let (side_x, side_w) = (margin + 64.0, right - margin - 64.0);
    let objs: Vec<Object> = vec![
        head,
        label(1, "Dated: November 2026, Monday first", margin, 17.0, full),
        object(
            calendar_ids::DATED,
            "November 2026",
            Frame::new(margin, 21.0, full, 65.0),
            keep,
            Style::Role(LineRole::Guide),
            month(
                Some(YearMonth::new(2026, 11)),
                Weekday::Monday,
                Weekday::Monday,
                31,
            ),
        ),
        label(
            2,
            "Undated: 31 days, 1st on Wed, Sunday first",
            margin,
            88.0,
            62.0,
        ),
        object(
            calendar_ids::UNDATED,
            "Undated month",
            Frame::new(margin, 92.0, 62.0, 58.0),
            keep,
            Style::Role(LineRole::Guide),
            month(None, Weekday::Sunday, Weekday::Wednesday, 31),
        ),
        label(3, "Mini: November 2026", side_x, 88.0, side_w),
        object(
            calendar_ids::MINI,
            "Mini November 2026",
            Frame::new(side_x, 92.0, side_w, 27.0),
            keep,
            Style::Role(LineRole::Text),
            mini(YearMonth::new(2026, 11), Weekday::Monday, true),
        ),
        label(4, "Mini: Feb 2028, Sunday first", side_x, 121.0, side_w),
        object(
            calendar_ids::MINI_PLAIN,
            "Mini February 2028",
            Frame::new(side_x, 125.0, side_w, 25.0),
            keep,
            Style::Role(LineRole::Text),
            mini(YearMonth::new(2028, 2), Weekday::Sunday, false),
        ),
    ];
    doc.layers = vec![Layer {
        name: "Calendars".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the fonts page's objects.
pub mod fonts_ids {
    use crate::Uuid;
    /// The `n`th object of the fonts page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_000E_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
}

/// The fonts page (docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted "Golden page 11a"): every
/// bundled family upright and italic at two sizes, a weight ramp in Inter and in Literata, Atkinson
/// Hyperlegible Mono, and a line of emoji, with Atkinson Hyperlegible Next's own text falling back to
/// Noto Emoji mid-sentence. Only bundled fonts: no installed font is ever on a golden page. Designed
/// on the Manta, also targeting the Nomad. A dot grid holds the sample's id, so the benchmarks drag
/// it over a page that is mostly text.
pub fn fonts_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 11a fonts".into();
    doc.metadata.description =
        "The rung 11a fonts page: every bundled family upright and italic, weight ramps and emoji."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    let title = doc.layers[1].objects[0].clone();
    let dots = doc.layers[0].objects[0].clone();
    // Every box keeps its place and size in mm on each device, so the page reads the same on both.
    let pinned = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    // Laid out for the Nomad's smaller page (118.9 by 158.5 mm), so every box fits both devices.
    let width = 102.8;
    let col = width / 2.0;
    let mut n = 0u128;
    let mut objs: Vec<Object> = Vec::new();
    let mut text = |objs: &mut Vec<Object>,
                    name: &str,
                    family: &str,
                    content: &str,
                    weight: u16,
                    style: FontStyle,
                    pt: f64,
                    x: f64,
                    y: f64,
                    w: f64| {
        n += 1;
        let mut t = title.clone();
        t.id = fonts_ids::nth(n);
        t.name = name.into();
        t.frame = Frame::new(x, y, w, pt * 0.3528 * 1.25);
        t.constraints = pinned;
        if let ObjectKind::Text(tb) = &mut t.kind {
            tb.content = content.into();
            tb.family = family.into();
            tb.weight = weight;
            tb.style = style;
            tb.size_pt = pt;
        }
        objs.push(t);
    };
    let mut head = title.clone();
    head.id = fonts_ids::TITLE;
    head.name = "Title".into();
    head.frame = Frame::new(margin, margin, width, 7.0);
    head.constraints = pinned;
    if let ObjectKind::Text(t) = &mut head.kind {
        t.content = "Rung 11a: the bundled fonts".into();
        t.size_pt = 12.0;
    }
    objs.push(head);
    let sample = "Quartz sphinx 123";
    let atkinson = "Atkinson Hyperlegible Next";
    let mut y = margin + 9.0;
    for family in [
        atkinson,
        "Atkinson Hyperlegible Mono",
        "Inter",
        "Literata",
        "Caveat",
    ] {
        text(
            &mut objs,
            &format!("{family} label"),
            atkinson,
            &format!("{family}: upright, then italic"),
            400,
            FontStyle::Normal,
            7.0,
            margin,
            y,
            width,
        );
        y += 3.4;
        // Upright in the left column, italic in the right, at two sizes.
        for pt in [8.0, 12.0] {
            for (i, (style, tag)) in [
                (FontStyle::Normal, "upright"),
                (FontStyle::Italic, "italic"),
            ]
            .into_iter()
            .enumerate()
            {
                text(
                    &mut objs,
                    &format!("{family} {tag} {pt}"),
                    family,
                    sample,
                    400,
                    style,
                    pt,
                    margin + i as f64 * col,
                    y,
                    col,
                );
            }
            y += pt * 0.3528 * 1.25;
        }
        y += 2.0;
    }
    for family in ["Inter", "Literata"] {
        text(
            &mut objs,
            &format!("{family} ramp label"),
            atkinson,
            &format!("{family}, weight 100 to 900"),
            400,
            FontStyle::Normal,
            7.0,
            margin,
            y,
            width,
        );
        y += 3.4;
        for (i, weight) in (100..=900).step_by(100).enumerate() {
            text(
                &mut objs,
                &format!("{family} {weight}"),
                family,
                &format!("Ag {weight}"),
                weight,
                FontStyle::Normal,
                8.0,
                margin + i as f64 * 11.4,
                y,
                11.4,
            );
        }
        y += 4.5;
    }
    // Large sizes, where Inter and Literata draw their optical-size (display) shapes.
    for (i, family) in ["Inter", "Literata"].into_iter().enumerate() {
        text(
            &mut objs,
            &format!("{family} display"),
            family,
            "Quartz 24",
            400,
            FontStyle::Normal,
            24.0,
            margin + i as f64 * col,
            y,
            col,
        );
    }
    y += 24.0 * 0.3528 * 1.25;
    text(
        &mut objs,
        "Emoji label",
        atkinson,
        "Noto Emoji, and Atkinson falling back to it",
        400,
        FontStyle::Normal,
        7.0,
        margin,
        y,
        width,
    );
    y += 3.4;
    text(
        &mut objs,
        "Emoji",
        "Noto Emoji",
        "\u{1F600} \u{1F60E} \u{1F389} \u{2764} \u{2605} \u{2600} \u{2602}",
        400,
        FontStyle::Normal,
        16.0,
        margin,
        y,
        width,
    );
    y += 8.0;
    text(
        &mut objs,
        "Fallback",
        atkinson,
        "Hello \u{1F600} world \u{2764} again",
        400,
        FontStyle::Normal,
        12.0,
        margin,
        y,
        width,
    );
    let mut grid = dots.clone();
    grid.name = "Dots".into();
    grid.frame = Frame::new(margin, y + 7.0, width, 10.0);
    grid.constraints = pinned;
    objs.push(grid);
    doc.layers = vec![Layer {
        name: "Fonts".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the text-box page's objects.
pub mod textbox_ids {
    use crate::Uuid;
    /// The `n`th object of the text-box page.
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_000F_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// The Fixed box that leaves lines out.
    pub const CUT: Uuid = nth(1);
    /// The box with a paragraph's space.
    pub const SPACED: Uuid = nth(2);
    /// The Fixed box with its text at the top.
    pub const TOP: Uuid = nth(3);
    /// The Fixed box with its text in the middle.
    pub const MIDDLE: Uuid = nth(4);
    /// The Fixed box with its text at the bottom.
    pub const BOTTOM: Uuid = nth(5);
    /// The ruled region the baselines stand on.
    pub const RULED: Uuid = nth(6);
    /// The box whose baselines stand on the rhythm, on the rule's top edge.
    pub const ON_RHYTHM: Uuid = nth(7);
    /// The ruled region of the box with a small gap above the line.
    pub const RULED_GAP: Uuid = nth(11);
    /// The box whose baselines stand a small gap above the rule.
    pub const ON_RHYTHM_GAP: Uuid = nth(12);
    /// The text turned 15 degrees.
    pub const TURNED_15: Uuid = nth(8);
    /// The text turned 90 degrees.
    pub const TURNED_90: Uuid = nth(9);
    /// The text turned 180 degrees.
    pub const TURNED_180: Uuid = nth(10);
}

/// The text-box page (docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted "Golden page 11b"): a
/// Fixed box that leaves lines out, the three vertical alignments, a paragraph's space, two boxes whose
/// baselines stand on a ruled region (on the rule's top edge, and a small gap above it), and text turned 15, 90 and 180 degrees. Every box with a height
/// of its own has a hairline frame round it, so the page shows where it ends. Bundled fonts only.
/// Designed on the Manta, also targeting the Nomad (laid out for its smaller page). A dot grid holds
/// the sample's id, so the benchmarks drag it.
pub fn textbox_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 11b text boxes".into();
    doc.metadata.description =
        "The rung 11b text-box page: a Fixed box with hidden lines, vertical \
        alignment, paragraph spacing, baselines on a ruled region and turned text."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    let title = doc.layers[1].objects[0].clone();
    let rule = doc.layers[1].objects[1].clone();
    let dots = doc.layers[0].objects[0].clone();
    let pinned = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    let width = 102.8;
    let n = std::cell::Cell::new(100u128);
    let mut objs: Vec<Object> = Vec::new();
    // A text box: its place and size in mm, what it says, and how its size and style are set.
    let text = |objs: &mut Vec<Object>,
                id: Option<Uuid>,
                name: &str,
                content: &str,
                frame: Frame,
                pt: f64,
                set: &dyn Fn(&mut TextBox, &mut Object)| {
        n.set(n.get() + 1);
        let mut t = title.clone();
        t.id = id.unwrap_or_else(|| textbox_ids::nth(n.get()));
        t.name = name.into();
        t.frame = frame;
        t.constraints = pinned;
        let mut tb = TextBox {
            content: content.into(),
            size_pt: pt,
            ..TextBox::default()
        };
        set(&mut tb, &mut t);
        t.kind = ObjectKind::Text(tb);
        objs.push(t);
    };
    // A hairline frame (no fill) round a box.
    let mut frames = Vec::new();
    let frame_of = |frames: &mut Vec<Object>, name: &str, f: Frame| {
        n.set(n.get() + 1);
        let mut r = rule.clone();
        r.id = textbox_ids::nth(n.get());
        r.name = name.into();
        r.frame = f;
        r.constraints = pinned;
        r.style = Style::Role(LineRole::Structure);
        r.kind = ObjectKind::Rect(RectShape {
            corner_radius_mm: 0.0,
            extra: Extra::new(),
        });
        frames.push(r);
    };
    let plain = |_: &mut TextBox, _: &mut Object| {};
    text(
        &mut objs,
        Some(textbox_ids::TITLE),
        "Title",
        "Rung 11b: the text box",
        Frame::new(margin, margin, width, 7.0),
        12.0,
        &|tb, _| tb.weight = 700,
    );
    let small = 7.0;
    // Row 1: a Fixed box that leaves lines out, and a paragraph's space.
    let (left, right) = (margin, 62.0);
    text(
        &mut objs,
        None,
        "Cut label",
        "Fixed: 3 of 5 lines fit",
        Frame::new(left, 17.0, 48.0, 3.0),
        small,
        &plain,
    );
    let cut = Frame::new(left, 21.0, 48.0, 14.0);
    frame_of(&mut frames, "Cut frame", cut);
    text(
        &mut objs,
        Some(textbox_ids::CUT),
        "Cut",
        "Line one\nLine two\nLine three\nLine four\nLine five",
        cut,
        10.0,
        &|tb, _| tb.sizing = TextSizing::Fixed,
    );
    text(
        &mut objs,
        None,
        "Spaced label",
        "Paragraph spacing: 8 pt",
        Frame::new(right, 17.0, 48.8, 3.0),
        small,
        &plain,
    );
    text(
        &mut objs,
        Some(textbox_ids::SPACED),
        "Spaced",
        "First paragraph\nSecond paragraph\nThird paragraph",
        Frame::new(right, 21.0, 48.8, 16.0),
        10.0,
        &|tb, _| tb.paragraph_spacing = ParagraphSpacing::Pt(8.0),
    );
    // Row 2: the three vertical alignments, in Fixed boxes 22 mm tall.
    text(
        &mut objs,
        None,
        "Align label",
        "Vertical align: top, middle, bottom",
        Frame::new(margin, 42.0, width, 3.0),
        small,
        &plain,
    );
    for (i, (id, name, v)) in [
        (textbox_ids::TOP, "Top", VerticalAlign::Top),
        (textbox_ids::MIDDLE, "Middle", VerticalAlign::Middle),
        (textbox_ids::BOTTOM, "Bottom", VerticalAlign::Bottom),
    ]
    .into_iter()
    .enumerate()
    {
        let f = Frame::new(margin + i as f64 * 35.9, 46.0, 31.0, 22.0);
        frame_of(&mut frames, &format!("{name} frame"), f);
        text(
            &mut objs,
            Some(id),
            name,
            &format!("{name}\nof the box"),
            f,
            10.0,
            &move |tb, _| {
                tb.sizing = TextSizing::Fixed;
                tb.v_align = v;
                tb.align = TextAlign::Center;
            },
        );
    }
    // Row 3: baselines on a ruled region, twice: standing on the rule's top edge (B, the default), and
    // with a small gap above it (C). The boxes sit off the rhythm; their text is pulled onto it.
    for (i, (ruled_id, text_id, name, gap)) in [
        (
            textbox_ids::RULED,
            textbox_ids::ON_RHYTHM,
            "on the line (default)",
            false,
        ),
        (
            textbox_ids::RULED_GAP,
            textbox_ids::ON_RHYTHM_GAP,
            "a small gap above the line",
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let top = 72.0 + i as f64 * 22.0;
        text(
            &mut objs,
            None,
            &format!("Ruled label {i}"),
            &format!("Baselines on the rhythm: {name}"),
            Frame::new(margin, top, width, 3.0),
            small,
            &plain,
        );
        let mut ruled = dots.clone();
        ruled.id = ruled_id;
        ruled.name = format!("Ruled {i}");
        ruled.frame = Frame::new(margin, top + 4.0, width, 16.0);
        ruled.constraints = pinned;
        // The Pattern tool's own line role.
        ruled.style = Style::Role(LineRole::Guide);
        if let ObjectKind::Pattern(p) = &mut ruled.kind {
            p.pattern = Pattern::Lines;
        }
        objs.push(ruled);
        text(
            &mut objs,
            Some(text_id),
            &format!("On the rhythm {i}"),
            "Letters stand on the line.\nLlama Elf",
            Frame::new(margin + 3.0, top + 5.7, 60.0, 14.0),
            11.0,
            &move |tb, _| {
                tb.baselines = true;
                tb.baseline_gap = gap;
            },
        );
    }
    // Row 4: turned text, about the middle of each box.
    text(
        &mut objs,
        None,
        "Turned label",
        "Turned: 15, 90 and 180 degrees",
        Frame::new(margin, 118.0, width, 3.0),
        small,
        &plain,
    );
    for (i, (id, name, deg)) in [
        (textbox_ids::TURNED_15, "Turned 15", 15.0),
        (textbox_ids::TURNED_90, "Turned 90", 90.0),
        (textbox_ids::TURNED_180, "Turned 180", 180.0),
    ]
    .into_iter()
    .enumerate()
    {
        text(
            &mut objs,
            Some(id),
            name,
            "Quartz",
            Frame::new(margin + 2.0 + i as f64 * 34.0, 132.0, 28.0, 5.3),
            12.0,
            &move |tb, o| {
                tb.align = TextAlign::Center;
                o.rotation_deg = deg;
            },
        );
    }
    let mut grid = dots.clone();
    grid.name = "Dots".into();
    grid.frame = Frame::new(margin, 147.0, width, 3.0);
    grid.constraints = pinned;
    objs.push(grid);
    // Frames go under the text.
    let mut all = frames;
    all.extend(objs);
    doc.layers = vec![Layer {
        name: "Text boxes".into(),
        visible: true,
        locked: false,
        objects: all,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the repeat page's objects.
pub mod repeat_ids {
    use crate::Uuid;
    /// The `n`th object of the repeat page (1-based, repeats, sources and labels alike).
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_000D_4000_8000_0000_0000_0000 + n)
    }
    /// The page's title.
    pub const TITLE: Uuid = nth(200);
    /// A row of four boxes, Auto pitch.
    pub const ACROSS: Uuid = nth(1);
    /// The same boxes at a pitch in mm, not bound to the rhythm.
    pub const MM_PITCH: Uuid = nth(2);
    /// A column of ellipses.
    pub const DOWN: Uuid = nth(3);
    /// A turned line repeated into a hatch.
    pub const HATCH: Uuid = nth(4);
    /// The habit tracker: a grid with a heavier line at every 7th column and a skipped row. It is
    /// what the benchmarks drag.
    pub const HABIT: Uuid = nth(5);
    /// A group (a box, a ring and a star) repeated in a grid.
    pub const GROUP: Uuid = nth(6);
    /// A row of stars that fills the margins, centred: its count differs on each device.
    pub const FILL: Uuid = nth(7);
}

/// The repeat page (docs/bbq/2026-11-13-rung-10-repeat-tool.md, Defaulted "Golden pages"): a linear
/// repeat across and one down, a habit-tracker grid with every 7th column in Structure and a skipped
/// row, a Fill repeat pinned to the margins with Centre (a different count on each device), a group as
/// the source, a turned line repeated into a hatch and a pitch in mm (not bound to the rhythm).
/// Designed on the Manta, also targeting the Nomad. The rhythm is 5 mm.
pub fn repeats_document() -> Document {
    repeats_page(false)
}

/// The repeat page held sideways: the same repeats on the Manta's page turned, the Fill repeat
/// running the longer way.
pub fn repeats_landscape_document() -> Document {
    repeats_page(true)
}

fn repeats_page(landscape: bool) -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 10 repeats".into();
    doc.metadata.description =
        "The rung 10 repeat page: across, down, a habit tracker, Fill with Centre, a group and a hatch."
            .into();
    let margin = 8.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    if landscape {
        doc.page.orientation = Orientation::Landscape;
    }
    let title = doc.layers[1].objects[0].clone();
    let text_at = |id: Uuid, name: &str, content: &str, x: f64, y: f64, w: f64, pt: f64, weight| {
        let mut t = title.clone();
        t.id = id;
        t.name = name.into();
        t.frame = Frame::new(x, y, w, 5.0);
        t.constraints = Constraints {
            h: HConstraint::Left,
            v: VConstraint::Top,
        };
        if let ObjectKind::Text(tb) = &mut t.kind {
            tb.content = content.into();
            tb.size_pt = pt;
            tb.weight = weight;
        }
        t
    };
    let pinned = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    let shape = |n: u128, name: &str, frame: Frame, kind: ObjectKind, fill: Option<Fill>| {
        let mut o = object(
            repeat_ids::nth(100 + n),
            name,
            frame,
            pinned,
            Style::Role(LineRole::Structure),
            kind,
        );
        o.fill = fill;
        o
    };
    let repeat = |id: Uuid,
                  name: &str,
                  source: Object,
                  constraints: Constraints,
                  tweak: &dyn Fn(&mut RepeatShape)| {
        let mut shape = RepeatShape::new(source);
        tweak(&mut shape);
        let mut o = object(
            id,
            name,
            Frame::new(0.0, 0.0, 1.0, 1.0),
            constraints,
            Style::None,
            ObjectKind::Repeat(shape),
        );
        // The box a repeat must have, from its source and its settings.
        if let Some(f) = o.repeat_fit((5.0, 5.0)) {
            o.frame = f;
        }
        o
    };
    let (left, top) = (margin, margin);
    let mut objs: Vec<Object> = vec![text_at(
        repeat_ids::TITLE,
        "Title",
        "Rung 10: the repeat tool",
        left,
        top,
        100.0,
        12.0,
        700,
    )];
    let label = |n: u128, content: &str, x: f64, y: f64| {
        text_at(
            repeat_ids::nth(300 + n),
            &format!("Label {n}"),
            content,
            x,
            y,
            52.0,
            7.0,
            400,
        )
    };

    // Across, Auto pitch: four 10 x 5 mm boxes, a pitch of two units.
    objs.push(label(1, "Across, 4 copies, Auto pitch", left, 22.0));
    objs.push(repeat(
        repeat_ids::ACROSS,
        "Across",
        shape(
            1,
            "Box",
            Frame::new(left, 28.0, 10.0, 5.0),
            ObjectKind::Rect(RectShape::default()),
            Some(Fill::Role(FillRole::Shade)),
        ),
        pinned,
        &|r| r.count = 4,
    ));
    // The same at 12 mm: a pitch in mm does not follow the rhythm.
    objs.push(label(2, "Pitch 12 mm (not on the rhythm)", 58.0, 22.0));
    objs.push(repeat(
        repeat_ids::MM_PITCH,
        "Pitch in mm",
        shape(
            2,
            "Box",
            Frame::new(58.0, 28.0, 8.0, 5.0),
            ObjectKind::Rect(RectShape::default()),
            None,
        ),
        pinned,
        &|r| {
            r.count = 4;
            r.h_spacing = Spacing::Mm(12.0);
        },
    ));
    // Down: a column of ellipses.
    objs.push(label(3, "Down, Auto", left, 40.0));
    objs.push(repeat(
        repeat_ids::DOWN,
        "Down",
        shape(
            3,
            "Ellipse",
            Frame::new(left, 46.0, 12.0, 6.0),
            ObjectKind::Ellipse(EllipseShape::default()),
            Some(Fill::Role(FillRole::Shade)),
        ),
        pinned,
        &|r| {
            r.mode = RepeatMode::Down;
            r.count = 4;
        },
    ));
    // A hatch: one turned line, copies side by side.
    objs.push(label(4, "A turned line, 8 copies", 30.0, 40.0));
    let mut hatch_line = object(
        repeat_ids::nth(104),
        "Line",
        Frame::new(30.0, 53.0, 14.0, 0.0),
        pinned,
        Style::Role(LineRole::Structure),
        ObjectKind::Line(Line::default()),
    );
    hatch_line.rotation_deg = 60.0;
    objs.push(repeat(
        repeat_ids::HATCH,
        "Hatch",
        hatch_line,
        pinned,
        &|r| {
            r.count = 8;
            r.h_spacing = Spacing::Units(0.5);
        },
    ));
    // The habit tracker: 14 columns by 4 rows, every 7th column heavier, row 2 skipped.
    objs.push(label(
        5,
        "Every 7th column heavier, row 2 skipped",
        left,
        86.0,
    ));
    objs.push(repeat(
        repeat_ids::HABIT,
        "Habit tracker",
        shape(
            5,
            "Day",
            Frame::new(left, 92.0, 5.0, 5.0),
            ObjectKind::Rect(RectShape::default()),
            None,
        ),
        pinned,
        &|r| {
            r.mode = RepeatMode::Grid;
            r.columns = 14;
            r.rows = 4;
            r.rules = vec![
                AlternationRule {
                    counts: RuleCounts::Columns,
                    when: RuleWhen::Every { n: 7, start: None },
                    action: RuleAction::LineRole(LineRole::Emphasis),
                },
                AlternationRule {
                    counts: RuleCounts::Rows,
                    when: RuleWhen::List(vec![2]),
                    action: RuleAction::Skip,
                },
            ];
        },
    ));
    // A group as the source: a box, a ring and a star in a 10 mm cell.
    objs.push(label(6, "A group as the source", 84.0, 86.0));
    let mut group_box = shape(
        6,
        "Cell",
        Frame::new(84.0, 92.0, 10.0, 10.0),
        ObjectKind::Rect(RectShape {
            corner_radius_mm: 2.0,
            extra: Extra::new(),
        }),
        None,
    );
    group_box.style = Style::Role(LineRole::Guide);
    let ring = shape(
        7,
        "Ring",
        Frame::new(85.5, 93.5, 7.0, 7.0),
        ObjectKind::Ellipse(EllipseShape::default()),
        None,
    );
    let star = shape(
        8,
        "Star",
        Frame::new(87.5, 95.5, 3.0, 3.0),
        ObjectKind::Polygon(PolygonShape::star(5)),
        Some(Fill::Role(FillRole::Block)),
    );
    let mut group = object(
        repeat_ids::nth(109),
        "Badge",
        Frame::new(84.0, 92.0, 10.0, 10.0),
        pinned,
        Style::None,
        ObjectKind::Group(GroupShape {
            children: vec![group_box, ring, star],
            extra: Extra::new(),
        }),
    );
    if let Some(b) = crate::edit::children_box(group.children()) {
        group.frame = b;
    }
    objs.push(repeat(repeat_ids::GROUP, "Badges", group, pinned, &|r| {
        r.mode = RepeatMode::Grid;
        r.columns = 3;
        r.rows = 2;
    }));
    // Fill: stars as many as fit between the margins, the spare room split to both sides.
    let page_w = if landscape { MANTA_H_MM } else { MANTA_W_MM };
    objs.push(label(7, "Fill between the margins, centred", left, 124.0));
    let mut fill = repeat(
        repeat_ids::FILL,
        "Fill",
        shape(
            9,
            "Star",
            Frame::new(left, 130.0, 4.5, 4.5),
            ObjectKind::Polygon(PolygonShape::star(5)),
            Some(Fill::Role(FillRole::Shade)),
        ),
        Constraints {
            h: HConstraint::LeftRight,
            v: VConstraint::Top,
        },
        &|r| {
            r.fill = true;
            r.leftover = Leftover::Centre;
        },
    );
    fill.frame = Frame::new(left, 130.0, page_w - 2.0 * margin, 4.5);
    objs.push(fill);
    doc.layers = vec![Layer {
        name: "Repeats".into(),
        visible: true,
        locked: false,
        objects: objs,
        extra: Extra::new(),
    }];
    doc
}

/// Fixed ids for the paper pages' objects.
pub mod paper_ids {
    use crate::Uuid;
    /// The `n`th object of the paper pages (both have the same objects).
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0012_4000_8000_0000_0000_0000 + n)
    }
}

/// The id of the ISO A4 paper profile, the paper page's primary.
pub const PAPER_A4_ID: &str = "paper.a4";
/// The id of the ISO A5 paper profile, which the paper page also targets, at 600 PPI.
pub const PAPER_A5_ID: &str = "paper.a5";
/// The id of the US Letter paper profile, which the landscape paper page is designed on.
pub const PAPER_LETTER_ID: &str = "paper.letter";

/// A paper page (rung 13b bbq, Defaulted "13b goldens"): a band and headline, a dot grid and ruled
/// lines between 10 mm margins, a frame on the margin box, a mark in each corner and a circle in
/// the middle (a squashed or stretched page shows as an oval). `w` by `h` mm is the page as it is
/// held; `landscape` holds it sideways, wide. Designed on `primary`, also targeting `secondary`
/// at the PPI given.
fn paper_page(
    title: &str,
    headline: &str,
    primary: &str,
    secondary: Option<(&str, u32)>,
    (w, h): (f64, f64),
    landscape: bool,
) -> Document {
    let mut doc = slice_document();
    doc.metadata.title = title.into();
    doc.metadata.description =
        "The rung 13b paper page: a band, dots, ruled lines, a margin frame, corner marks and a circle."
            .into();
    let margin = 10.0;
    doc.page.margins_mm = Margins::all(margin);
    doc.page.orientation = if landscape {
        Orientation::Landscape
    } else {
        Orientation::Portrait
    };
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.rhythm.major_every = Some(5);
    doc.primary_profile_id = primary.into();
    doc.profiles = vec![ProfileContext::new(primary)];
    if let Some((id, ppi)) = secondary {
        let mut c = ProfileContext::new(id);
        c.ppi = ppi;
        doc.profiles.push(c);
    }
    let inner_w = w - 2.0 * margin;
    let top_wide = Constraints {
        h: HConstraint::LeftRight,
        v: VConstraint::Top,
    };
    let rule = doc.layers[1].objects[1].clone();
    let mut dots = doc.layers[0].objects[0].clone();
    let mut lines = doc.layers[0].objects[1].clone();
    let mut headline_box = doc.layers[1].objects[0].clone();
    let mk = |n: u128, name: &str, frame: Frame, c: Constraints, style: Style, fill, kind| {
        let mut o = rule.clone();
        o.id = paper_ids::nth(n);
        o.name = name.into();
        o.frame = frame;
        o.constraints = c;
        o.style = style;
        o.fill = fill;
        o.kind = kind;
        o
    };
    let structure = Style::Role(LineRole::Structure);
    let frame_box = mk(
        1,
        "Margin frame",
        Frame::new(margin, margin, inner_w, h - 2.0 * margin),
        Constraints {
            h: HConstraint::LeftRight,
            v: VConstraint::TopBottom,
        },
        Style::Role(LineRole::Whisper),
        None,
        ObjectKind::Rect(RectShape::default()),
    );
    let band = mk(
        2,
        "Header band",
        Frame::new(margin + 2.0, margin + 2.0, inner_w - 4.0, 12.0),
        top_wide,
        structure,
        Some(Fill::Role(FillRole::Shade)),
        ObjectKind::Rect(RectShape::default()),
    );
    headline_box.id = paper_ids::nth(3);
    headline_box.name = "Headline".into();
    headline_box.frame = Frame::new(margin + 5.0, margin + 4.5, inner_w - 10.0, 8.0);
    headline_box.constraints = top_wide;
    if let ObjectKind::Text(t) = &mut headline_box.kind {
        t.content = headline.into();
        t.size_pt = 14.0;
    }
    dots.frame = Frame::new(margin + 2.0, margin + 20.0, inner_w - 4.0, 60.0);
    dots.constraints = top_wide;
    lines.frame = Frame::new(
        margin + 2.0,
        margin + 86.0,
        inner_w - 4.0,
        // Down to just above the bottom corner marks.
        h - margin - 14.0 - (margin + 86.0),
    );
    lines.constraints = Constraints {
        h: HConstraint::LeftRight,
        v: VConstraint::TopBottom,
    };
    let corner = |n: u128, name: &str, x: f64, y: f64, hc, vc| {
        mk(
            n,
            name,
            Frame::new(x, y, 6.0, 6.0),
            Constraints { h: hc, v: vc },
            structure,
            Some(Fill::Role(FillRole::Block)),
            ObjectKind::Rect(RectShape::default()),
        )
    };
    let (far_x, far_y) = (w - margin - 3.0 - 6.0, h - margin - 3.0 - 6.0);
    let mut objs = vec![
        frame_box,
        band,
        headline_box,
        corner(
            4,
            "Corner, top left",
            margin + 3.0,
            margin + 3.0 + 14.0,
            HConstraint::Left,
            VConstraint::Top,
        ),
        corner(
            5,
            "Corner, top right",
            far_x,
            margin + 3.0 + 14.0,
            HConstraint::Right,
            VConstraint::Top,
        ),
        corner(
            6,
            "Corner, bottom left",
            margin + 3.0,
            far_y,
            HConstraint::Left,
            VConstraint::Bottom,
        ),
        corner(
            7,
            "Corner, bottom right",
            far_x,
            far_y,
            HConstraint::Right,
            VConstraint::Bottom,
        ),
    ];
    // A circle: it stays round only when the page and its pixels are not stretched.
    let circle = 36.0;
    objs.push(mk(
        8,
        "Circle",
        Frame::new((w - circle) / 2.0, margin + 24.0, circle, circle),
        Constraints {
            h: HConstraint::Center,
            v: VConstraint::Top,
        },
        Style::Role(LineRole::Emphasis),
        Some(Fill::Role(FillRole::Shade)),
        ObjectKind::Ellipse(EllipseShape::default()),
    ));
    let mut ruled_layer = vec![dots, lines];
    if landscape {
        // A star by the top-right corner: upright in the wide page, which is not turned.
        objs.push(mk(
            9,
            "Star",
            Frame::new(w - margin - 28.0, margin + 24.0, 18.0, 18.0),
            Constraints {
                h: HConstraint::Right,
                v: VConstraint::Top,
            },
            Style::Role(LineRole::Emphasis),
            Some(Fill::Role(FillRole::Block)),
            ObjectKind::Polygon(PolygonShape::star(5)),
        ));
    }
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: std::mem::take(&mut ruled_layer),
            extra: Extra::new(),
        },
        Layer {
            name: "Marks".into(),
            visible: true,
            locked: false,
            objects: objs,
            extra: Extra::new(),
        },
    ];
    doc
}

/// The paper page (rung 13b): ISO A4 portrait at 300 PPI, also targeting ISO A5 at 600 PPI (the page
/// picks its export PPI per paper size). It holds a dot region with the sample's id
/// ([`ids::DOTS`]), so the benchmarks run over it.
pub fn paper_document() -> Document {
    paper_page(
        "Rung 13b paper",
        "Paper: A4 at 300 PPI, A5 at 600 PPI",
        PAPER_A4_ID,
        Some((PAPER_A5_ID, 600)),
        (210.0, 297.0),
        false,
    )
}

/// The same page on US Letter held sideways (rung 13b bbq, Q14): a wide PNG, 3300 × 2550 px at 300
/// PPI, not turned. It holds a dot region with the sample's id ([`ids::DOTS`]).
pub fn paper_landscape_document() -> Document {
    paper_page(
        "Rung 13b paper, landscape",
        "Landscape paper: US Letter, wide, 300 PPI",
        PAPER_LETTER_ID,
        None,
        (279.4, 215.9),
        true,
    )
}

/// Fixed ids for the lint page's objects.
pub mod lint_ids {
    use crate::Uuid;
    /// The `n`th object of the lint page (1-based); the dot grid is [`super::ids::DOTS`].
    pub const fn nth(n: u128) -> Uuid {
        Uuid::from_u128(0x5155_4144_0014_4000_8000_0000_0000_0000 + n)
    }
}

/// The lint page (rung 14a, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q3 and Q4): a Template with
/// objects that make each of L03 (gray too light, as a line, a fill, text and dots), L05 (a gray between the
/// clean levels), L07 (text under 6 pt), L09 (a ruled region under 5 mm (3 mm apart, half the 6 mm rhythm)), L13 (a dark fill over a tenth of
/// the page) and L14 (a thin slanted line and a thin curve) fire, beside objects of the same kinds that pass.
/// Every object is named for what it shows, and sits on the 6 mm rhythm inside the margins and clear of the
/// Supernotes' toolbars, so the only findings are the ones meant. It holds the sample's dot grid
/// ([`ids::DOTS`]) so the benchmarks can run over it. Designed on the Manta, also targeting the Nomad.
pub fn lint_document() -> Document {
    let mut doc = slice_document();
    doc.metadata.title = "Rung 14a lint".into();
    doc.metadata.description =
        "The rung 14a lint page: objects that make L03, L05, L07, L09, L13 and L14 fire, and objects that pass."
            .into();
    let mut dots = doc.layers[0].objects[0].clone();
    let rule = doc.layers[1].objects[1].clone();
    // The pixel-locked rhythm on the Manta: its origin at the 8 mm margin is 94 px and its step 71 px (6 mm is 70.87),
    // so a place k steps along is (94 + 71 k) px, which is what L16 counts as on the rhythm.
    let at = |k: f64| (94.0 + 71.0 * k) * MM_PER_IN / 300.0;
    dots.frame = Frame::new(at(1.0), at(29.0), 132.0, 12.0);
    // Scaled with the page, so the Nomad's smaller page holds it too.
    dots.constraints = Constraints::default();
    let mut objs: Vec<Object> = Vec::new();
    let mut n = 0u128;
    let mut add = |name: &str, frame: Frame, style: Style, fill: Option<Fill>, kind: ObjectKind| {
        n += 1;
        let mut o = rule.clone();
        o.id = lint_ids::nth(n);
        o.name = name.into();
        o.frame = frame;
        o.constraints = Constraints::default();
        o.style = style;
        o.fill = fill;
        o.kind = kind;
        objs.push(o);
    };
    let px = |px: f64, gray: u8| Style::Custom {
        width_mm: px * MM_PER_IN / 300.0,
        gray,
    };
    let line = || ObjectKind::Line(Line::default());
    let text = |content: &str, size_pt: f64, weight: u16| {
        ObjectKind::Text(TextBox {
            content: content.into(),
            family: ATKINSON.into(),
            weight,
            size_pt,
            ..TextBox::default()
        })
    };
    let region = |pattern: Pattern| ObjectKind::Pattern(region_0_19(pattern));
    let ruled_3mm = {
        let mut pr = region_0_19(Pattern::Lines);
        pr.bound = false;
        pr.v_spacing_mm = Some(3.0);
        ObjectKind::Pattern(pr)
    };
    let (guide, structure, emphasis, text_role) = (
        Style::Role(LineRole::Guide),
        Style::Role(LineRole::Structure),
        Style::Role(LineRole::Emphasis),
        Style::Role(LineRole::Text),
    );

    // Left column (x = 14 mm).
    add(
        "L03 line, gray 230",
        Frame::new(at(1.0), at(1.0), 60.0, 0.0),
        px(1.0, 230),
        None,
        line(),
    );
    add(
        "Passes: 1 px line, Guide role",
        Frame::new(at(1.0), at(2.0), 60.0, 0.0),
        guide,
        None,
        line(),
    );
    add(
        "L03 fill, gray 230",
        Frame::new(at(1.0), at(3.0), 60.0, 12.0),
        Style::None,
        Some(Fill::Gray(230)),
        ObjectKind::Rect(RectShape::default()),
    );
    add(
        "L03 text, gray 200",
        Frame::new(at(1.0), at(6.0), 60.0, 6.0),
        px(1.0, 200),
        None,
        text("Pale heading", 12.0, 400),
    );
    add(
        "L07 text, 5 pt",
        Frame::new(at(1.0), at(8.0), 60.0, 6.0),
        text_role,
        None,
        text("Fine print", 5.0, 400),
    );
    add(
        "Passes: text, 9 pt",
        Frame::new(at(1.0), at(9.0), 60.0, 6.0),
        text_role,
        None,
        text("Reading text", 9.0, 400),
    );
    add(
        "L05 line, gray 120",
        Frame::new(at(1.0), at(10.0), 60.0, 0.0),
        px(1.0, 120),
        None,
        line(),
    );
    add(
        "L14 diagonal line, 1 px",
        Frame::new(at(1.0), at(11.0), 60.0, 12.0),
        px(1.0, 51),
        None,
        line(),
    );
    add(
        "L14 ellipse, 1 px",
        Frame::new(at(1.0), at(14.0), 30.0, 18.0),
        px(1.0, 51),
        None,
        ObjectKind::Ellipse(EllipseShape::default()),
    );
    add(
        "Passes: diagonal line, Emphasis (3 px)",
        Frame::new(at(1.0), at(18.0), 60.0, 12.0),
        emphasis,
        None,
        line(),
    );
    add(
        "Passes: Shade fill",
        Frame::new(at(1.0), at(21.0), 60.0, 6.0),
        Style::None,
        Some(Fill::Role(FillRole::Shade)),
        ObjectKind::Rect(RectShape::default()),
    );

    // Right column (x = 86 mm).
    add(
        "L03 dots, 2 px at gray 150",
        Frame::new(at(13.0), at(1.0), 60.0, 18.0),
        px(2.0, 150),
        None,
        region(Pattern::Dots),
    );
    add(
        "Passes: dot grid, Dot grid role",
        Frame::new(at(13.0), at(5.0), 60.0, 18.0),
        Style::Role(LineRole::DotGrid),
        None,
        region(Pattern::Dots),
    );
    add(
        "L09 ruled lines 3 mm apart",
        Frame::new(at(13.0), at(9.0), 60.0, 24.0),
        guide,
        None,
        ruled_3mm,
    );
    add(
        "Passes: ruled lines on the rhythm",
        Frame::new(at(13.0), at(14.0), 60.0, 24.0),
        guide,
        None,
        region(Pattern::Lines),
    );
    add(
        "Passes: Structure line",
        Frame::new(at(13.0), at(19.0), 60.0, 0.0),
        structure,
        None,
        line(),
    );

    // The dark fill: 132 mm by 36 mm is 13 percent of the Manta page.
    add(
        "L13 dark fill, Block role",
        Frame::new(at(1.0), at(23.0), 132.0, 30.0),
        Style::None,
        Some(Fill::Role(FillRole::Block)),
        ObjectKind::Rect(RectShape::default()),
    );
    doc.layers = vec![
        Layer {
            name: "Grid".into(),
            visible: true,
            locked: false,
            objects: vec![dots],
            extra: Extra::new(),
        },
        Layer {
            name: "Lint".into(),
            visible: true,
            locked: false,
            objects: objs,
            extra: Extra::new(),
        },
    ];
    doc
}
