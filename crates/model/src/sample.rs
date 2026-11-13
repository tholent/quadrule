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
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
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
        ObjectKind::Pattern(PatternRegion::new(Pattern::Dots)),
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
        ObjectKind::Pattern(PatternRegion::new(Pattern::Lines)),
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
            orientation: Default::default(),
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
        ObjectKind::Pattern(PatternRegion::new(Pattern::Lines)),
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
    let region = |p: Pattern| PatternRegion::new(p);
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
        Frame::new(30.0, 46.0, 14.0, 0.0),
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
