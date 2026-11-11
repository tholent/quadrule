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

//! Calibration sheets (napkin §8.4): pages of labelled samples, drawn by the app's own export
//! renderer at each profile's native resolution, to copy to a device and judge by eye and pen.
//!
//! Five sheets, each one or more pages:
//! - **lines**: line widths 1–6 px × 15 grays, then dashed and dotted lines × 5 grays;
//! - **dots**: square dots 1–4 px and round dots 2–6 px × 8 grays;
//! - **text**: the bundled font at 6–18 pt, regular and bold, × 4 grays;
//! - **writing**: wide strips of ruled lines and dot rows to write across with each pen;
//! - **grays**: flat patches at 30 grays and a smooth ramp, to find the grays the panel shows
//!   clean (flat, no speckle or banding), which fills will need.
//!
//! Every sample has a code (e.g. `LB7`); the code and what it means are on the sheet and in
//! `codes.txt`. Codes mean the same on every profile, so results can be compared.
//!
//! The sheets draw display-list items directly: round dots and dashes exist here only, as
//! samples to judge, and are not part of the document model.

use kurbo::{Affine, BezPath, Circle, Point, Shape};
use quadrule_layout::{DisplayList, Item, snap_center, text_path};
use quadrule_model::sample::ATKINSON;
use quadrule_model::{Extra, MM_PER_IN, TextAlign, TextBox, Uuid};
use quadrule_profiles::Profile;
use quadrule_text::{FontRegistry, TextRequest, lay_out};

/// Line widths in the lines matrix, rows A–F.
pub const LINE_WIDTHS: [u32; 6] = [1, 2, 3, 4, 5, 6];
/// Grays in the lines matrix, columns 1–15: 0 to 238 in steps of 17.
pub const LINE_GRAYS: [u8; 15] = [
    0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238,
];
/// Dash patterns (dash, gap) in pixels, rows A–E; all 2 px wide.
pub const DASHES: [(u32, u32); 5] = [(2, 2), (2, 4), (6, 6), (12, 6), (24, 12)];
/// Width of the dashed lines.
pub const DASH_WIDTH: u32 = 2;
/// Grays of the dashed lines, columns 1–5.
pub const DASH_GRAYS: [u8; 5] = [51, 102, 153, 187, 221];

/// A dot's shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DotShape {
    /// A hard-edged square on whole pixels.
    Square,
    /// An anti-aliased circle.
    Round,
}

/// Dot samples, rows A–H: shape and size in pixels (side or diameter).
pub const DOTS: [(DotShape, u32); 8] = [
    (DotShape::Square, 1),
    (DotShape::Square, 2),
    (DotShape::Square, 3),
    (DotShape::Square, 4),
    (DotShape::Round, 2),
    (DotShape::Round, 3),
    (DotShape::Round, 4),
    (DotShape::Round, 6),
];
/// Dot grays, columns 1–8.
pub const DOT_GRAYS: [u8; 8] = [0, 51, 102, 136, 170, 187, 204, 221];
/// Text sizes in points; rows A–H regular, I–P bold.
pub const TEXT_SIZES_PT: [f64; 8] = [6.0, 7.0, 8.0, 9.0, 10.0, 12.0, 14.0, 18.0];
/// Text weights.
pub const TEXT_WEIGHTS: [u16; 2] = [400, 700];
/// Text grays, columns 1–4. Column 4 (gray 153) added 2026-11-09 for labels (chris: "I'd like to consider a
/// lighter shade of gray for labels, maybe 153?"); columns 1–3 keep their codes.
pub const TEXT_GRAYS: [u8; 4] = [0, 51, 102, 153];
/// Writing strips W1–W12: ruled lines, width × gray. Set 2 (2026-11-09, chris: "let's redo the
/// calibration sheets to include 51"): grays 51 and 102 added around the Nomad favourites (LA4,
/// PD2), gray 221 dropped (invisible on the Nomad).
pub const STRIP_LINES: [(u32, u8); 12] = [
    (1, 51),
    (1, 102),
    (1, 153),
    (1, 187),
    (2, 51),
    (2, 102),
    (2, 153),
    (2, 187),
    (3, 51),
    (3, 102),
    (3, 153),
    (3, 187),
];
/// Writing strips W13–W18: square dot rows, size × gray (set 2: the sizes that showed on the
/// Nomad, at the two darkest favourites).
pub const STRIP_DOTS: [(u32, u8); 6] = [(2, 51), (2, 102), (3, 51), (3, 102), (4, 51), (4, 102)];

/// Flat gray patches G1–G15: the even steps of 17 from 0 to 238, where a 16-level panel's own
/// levels are expected to fall (added 2026-11-10 for rung 6's fills; arc v1 board, chris's queue).
pub const PATCH_STEP_GRAYS: [u8; 15] = [
    0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238,
];
/// Flat gray patches G16–G30: halfway between the steps, to show what an off-step gray becomes.
pub const PATCH_BETWEEN_GRAYS: [u8; 15] = [
    8, 25, 42, 59, 76, 93, 110, 127, 144, 161, 178, 195, 212, 229, 246,
];
/// Patches per row on the grays sheet.
const PATCH_COLUMNS: usize = 5;

const LABEL_GRAY: u8 = 0;
/// Height of one writing strip: label, three lines 7 mm apart, and room before the next.
const STRIP_MM: f64 = 24.0;
const ROWS: &str = "ABCDEFGHIJKLMNOP";

/// One labelled sample on a page.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    /// e.g. `LB7`.
    pub code: String,
    /// What the sample is, e.g. `line 2 px, gray 102`.
    pub what: String,
    /// The area holding the sample's marks (not its label), in pixels: x0, y0, x1, y1.
    pub area: (f64, f64, f64, f64),
}

/// One page of a sheet.
pub struct Page {
    /// `lines`, `dots`, `text` or `writing`.
    pub sheet: &'static str,
    /// 1-based page number within the sheet.
    pub number: usize,
    /// Pages in the sheet.
    pub of: usize,
    /// What to draw.
    pub dl: DisplayList,
    /// The samples on this page.
    pub samples: Vec<Sample>,
}

impl Page {
    /// A file name with only ASCII letters, digits, hyphens and underscores (napkin §11.3).
    pub fn file_name(&self, p: &Profile) -> String {
        format!(
            "quadrule-calib_{}-{}_{}_{}x{}.png",
            self.sheet,
            self.number,
            slug(&p.id),
            p.width_px,
            p.height_px
        )
    }
}

/// A profile id as a safe file or folder name.
pub fn slug(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[derive(Default)]
struct Draft {
    items: Vec<Item>,
    samples: Vec<Sample>,
}

/// Places rows down the page and starts a new page when one is full.
struct Pager<'a> {
    p: &'a Profile,
    reg: &'a mut FontRegistry,
    left: f64,
    right: f64,
    top: f64,
    content_top: f64,
    bottom: f64,
    y: f64,
    drafts: Vec<Draft>,
}

type Result<T> = std::result::Result<T, String>;

impl<'a> Pager<'a> {
    fn new(p: &'a Profile, reg: &'a mut FontRegistry) -> Self {
        let m = p.default_margins_mm;
        let px = |mm: f64| p.mm_to_px(mm).round();
        // Keep clear of every docked toolbar and the footer at once, plus 2 mm, so the sheet
        // reads the same wherever the toolbar is docked.
        let [l, r, t, b] = edge_insets(p);
        let clear = |inset: f64, margin_mm: f64| {
            if inset > 0.0 {
                (inset + px(2.0)).max(px(margin_mm))
            } else {
                px(margin_mm)
            }
        };
        let top = clear(t, m.top);
        let content_top = top + px(16.0);
        Pager {
            p,
            reg,
            left: clear(l, m.left),
            right: p.width_px as f64 - clear(r, m.right),
            top,
            content_top,
            bottom: p.height_px as f64 - clear(b, m.bottom),
            y: content_top,
            drafts: vec![Draft::default()],
        }
    }

    fn mm(&self, mm: f64) -> f64 {
        self.p.mm_to_px(mm).round()
    }

    fn width(&self) -> f64 {
        self.right - self.left
    }

    fn width_mm(&self) -> f64 {
        self.width() * MM_PER_IN / self.p.ppi
    }

    /// Reserves `h` pixels for a row, on a new page if this one is full; returns its top.
    fn row(&mut self, h: f64) -> f64 {
        let used = !self.drafts.last().is_some_and(|d| d.items.is_empty());
        if self.y + h > self.bottom && used {
            self.drafts.push(Draft::default());
            self.y = self.content_top;
        }
        let y0 = self.y;
        self.y += h;
        y0
    }

    /// Starts a new page unless `mm` more fits on this one; keeps a section's legend and column
    /// heading with its first row of samples.
    fn keep(&mut self, mm: f64) {
        if self.y + self.mm(mm) > self.bottom {
            self.drafts.push(Draft::default());
            self.y = self.content_top;
        }
    }

    fn gap(&mut self, mm: f64) {
        self.y += self.mm(mm);
    }

    fn draft(&mut self) -> &mut Draft {
        self.drafts.last_mut().expect("at least one page")
    }

    /// Lays out text in a box `width_mm` wide; returns its path at (0, 0) and its height in px.
    fn measure(
        &mut self,
        s: &str,
        size_pt: f64,
        weight: u16,
        width_mm: f64,
    ) -> Result<(BezPath, f64)> {
        let tb = TextBox {
            content: s.into(),
            family: ATKINSON.into(),
            weight,
            size_pt,
            line_height: 1.2,
            letter_spacing_em: 0.0,
            align: TextAlign::Left,
            extra: Extra::new(),
        };
        let id = Uuid::from_u128(1);
        let laid = lay_out(
            self.reg,
            &self.p.id,
            &[TextRequest {
                object_id: id,
                text: &tb,
                width_mm,
            }],
        )
        .map_err(|e| e.to_string())?;
        if laid.missing_glyphs > 0 {
            return Err(format!("missing glyphs in {s:?}"));
        }
        let t = laid.outlines.text(id).ok_or("no text laid out")?;
        let path = text_path(&laid.outlines, t, Point::ORIGIN, self.p.ppi / MM_PER_IN)
            .map_err(|e| e.to_string())?;
        Ok((path, self.p.mm_to_px(t.height_mm as f64).ceil()))
    }

    /// Draws already-measured text with its top-left corner at (x, y).
    fn put(&mut self, mut path: BezPath, x: f64, y: f64, gray: u8) {
        path.apply_affine(Affine::translate((x, y)));
        self.draft().items.push(Item::Fill { path, gray });
    }

    /// Text at (x, y) in a box `width_mm` wide; returns its height in px.
    fn text(
        &mut self,
        s: &str,
        size_pt: f64,
        weight: u16,
        gray: u8,
        at: (f64, f64),
        width_mm: f64,
    ) -> Result<f64> {
        let (path, h) = self.measure(s, size_pt, weight, width_mm)?;
        self.put(path, at.0, at.1, gray);
        Ok(h)
    }

    /// A paragraph across the full width, as its own row.
    fn paragraph(&mut self, s: &str, size_pt: f64, weight: u16) -> Result<()> {
        let w = self.width_mm();
        let (path, h) = self.measure(s, size_pt, weight, w)?;
        let y = self.row(h + self.mm(1.5));
        let x = self.left;
        self.put(path, x, y, LABEL_GRAY);
        Ok(())
    }

    /// A row naming each column: its number and the gray it holds.
    fn column_header(&mut self, grays: &[u8]) -> Result<()> {
        let cw = (self.width() / grays.len() as f64).floor();
        let y = self.row(self.mm(6.0));
        // Each heading wraps inside its own column, so it can never spill into the next one.
        let col_mm = (cw - 2.0 * self.mm(1.0)) * MM_PER_IN / self.p.ppi;
        for (i, g) in grays.iter().enumerate() {
            let x = self.left + i as f64 * cw + self.mm(1.0);
            self.text(
                &format!("{}\n{g}", i + 1),
                6.0,
                400,
                LABEL_GRAY,
                (x, y),
                col_mm,
            )?;
        }
        Ok(())
    }

    fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, gray: u8) {
        self.draft().items.push(Item::Rect {
            x0,
            y0,
            x1,
            y1,
            gray,
        });
    }

    /// A horizontal line `w` px wide whose centre snaps near `y`, from x0 to x1.
    fn hline(&mut self, x0: f64, x1: f64, y: f64, w: u32, gray: u8) {
        let c = snap_center(y, w);
        let half = w as f64 / 2.0;
        self.rect(x0, c - half, x1, c + half, gray);
    }

    fn dot(&mut self, shape: DotShape, size: u32, x: f64, y: f64, gray: u8) {
        let (cx, cy) = (snap_center(x, size), snap_center(y, size));
        let half = size as f64 / 2.0;
        match shape {
            DotShape::Square => self.rect(cx - half, cy - half, cx + half, cy + half, gray),
            DotShape::Round => {
                let path = Circle::new((cx, cy), half).to_path(0.01);
                self.draft().items.push(Item::Fill { path, gray });
            }
        }
    }

    fn sample(&mut self, code: String, what: String, area: (f64, f64, f64, f64)) {
        self.draft().samples.push(Sample { code, what, area });
    }

    /// Adds each page's header and turns the drafts into pages.
    fn finish(self, sheet: &'static str, title: &str, version: &str) -> Result<Vec<Page>> {
        let Pager {
            p,
            reg,
            left,
            right,
            top,
            drafts,
            ..
        } = self;
        let of = drafts.len();
        let mut pages = Vec::with_capacity(of);
        let width_mm = (right - left) * MM_PER_IN / p.ppi;
        let mut header = Pager::new(p, reg);
        for (i, d) in drafts.into_iter().enumerate() {
            header.drafts = vec![Draft::default()];
            let h1 = header.text(
                &format!("Quadrule calibration · {title} · page {} of {of}", i + 1),
                10.0,
                700,
                LABEL_GRAY,
                (left, top),
                width_mm,
            )?;
            let roles = match (&p.calibration_meta.date, p.calibration_meta.verified) {
                (Some(d), true) => format!("role values calibrated on this device {d}"),
                _ => "role values on this device are estimates until calibrated".into(),
            };
            let info = format!(
                "{} · {} · {} × {} px · {} PPI · quadrule {version} · {roles}",
                p.display_name, p.id, p.width_px, p.height_px, p.ppi
            );
            let h2 = header.text(&info, 6.0, 400, LABEL_GRAY, (left, top + h1), width_mm)?;
            let rule_y = top + h1 + h2 + header.mm(1.5);
            if rule_y + header.mm(2.0) > header.content_top {
                return Err(format!("{}: page header is too tall for its space", p.id));
            }
            header.hline(left, right, rule_y, 2, 51);
            let mut items = std::mem::take(&mut header.drafts[0].items);
            items.extend(d.items);
            let dl = DisplayList {
                width: p.width_px,
                height: p.height_px,
                background: 255,
                items,
            };
            pages.push(Page {
                sheet,
                number: i + 1,
                of,
                dl,
                samples: d.samples,
            });
        }
        Ok(pages)
    }
}

/// How far the profile's fixed (non-advisory) safe zones reach in from each edge, in pixels:
/// [left, right, top, bottom]. A zone hugs the edge along its long side.
pub fn edge_insets(p: &Profile) -> [f64; 4] {
    let (w, h) = (p.width_px, p.height_px);
    let mut ins = [0u32; 4];
    for z in p.safe_zones.iter().filter(|z| !z.advisory) {
        let (x, y, zw, zh) = z.rect;
        if zw < zh {
            if x == 0 {
                ins[0] = ins[0].max(zw);
            } else if x + zw == w {
                ins[1] = ins[1].max(zw);
            }
        } else if y == 0 {
            ins[2] = ins[2].max(zh);
        } else if y + zh == h {
            ins[3] = ins[3].max(zh);
        }
    }
    ins.map(f64::from)
}

fn row_letter(i: usize) -> char {
    ROWS.as_bytes()[i] as char
}

/// Every calibration page for one profile, in order: lines, dots, text, writing, grays.
pub fn sheets(p: &Profile, reg: &mut FontRegistry, version: &str) -> Result<Vec<Page>> {
    let mut all = lines_sheet(p, reg, version)?;
    all.extend(dots_sheet(p, reg, version)?);
    all.extend(text_sheet(p, reg, version)?);
    all.extend(writing_sheet(p, reg, version)?);
    all.extend(grays_sheet(p, reg, version)?);
    Ok(all)
}

/// The gray a ramp column shows: 0 at the left edge to 255 at the right, linear.
pub fn ramp_gray(i: u32, width: u32) -> u8 {
    if width < 2 {
        return 0;
    }
    ((i as f64 * 255.0 / (width - 1) as f64).round()) as u8
}

fn grays_sheet(p: &Profile, reg: &mut FontRegistry, version: &str) -> Result<Vec<Page>> {
    let mut pg = Pager::new(p, reg);
    pg.paragraph(
        "Flat grays. G1–G15: the even steps 0, 17, 34 … 238. G16–G30: halfway between them (8, 25, 42 … 246). \
         Each patch is one gray, the same in every pixel. Note which patches look flat and even (clean), and which \
         show speckle, dots, stripes or blotches. GR: a smooth ramp from black (left) to white (right); note \
         whether it shows steps or bands, and about how many.",
        8.0,
        400,
    )?;
    let cw = (pg.width() / PATCH_COLUMNS as f64).floor();
    let (pad, label_h, patch_h) = (pg.mm(1.0), pg.mm(3.5), pg.mm(14.0));
    let mut n = 0;
    for (set, grays) in [
        ("an even step", &PATCH_STEP_GRAYS),
        ("between steps", &PATCH_BETWEEN_GRAYS),
    ] {
        for row in grays.chunks(PATCH_COLUMNS) {
            let y0 = pg.row(label_h + patch_h + pg.mm(2.0));
            for (c, &g) in row.iter().enumerate() {
                n += 1;
                let code = format!("G{n}");
                let x0 = pg.left + c as f64 * cw;
                let what = format!("flat patch, gray {g} ({set})");
                pg.text(
                    &format!("{code} · {g}"),
                    6.0,
                    700,
                    LABEL_GRAY,
                    (x0 + pad, y0),
                    20.0,
                )?;
                let (a, b) = (x0 + pad, x0 + cw - pad);
                let (top, bottom) = (y0 + label_h, y0 + label_h + patch_h);
                pg.rect(a, top, b, bottom, g);
                pg.sample(code, what, (a, top, b, bottom));
            }
        }
        pg.gap(2.0);
    }
    let ramp_h = pg.mm(14.0);
    let y0 = pg.row(label_h + ramp_h + pg.mm(2.0));
    let (left, right) = (pg.left, pg.right);
    pg.text(
        "GR · ramp, 0 (black) left to 255 (white) right",
        6.0,
        700,
        LABEL_GRAY,
        (left, y0),
        80.0,
    )?;
    let (top, bottom) = (y0 + label_h, y0 + label_h + ramp_h);
    let w = (right - left) as u32;
    for i in 0..w {
        let x = left + i as f64;
        pg.rect(x, top, x + 1.0, bottom, ramp_gray(i, w));
    }
    // The ramp's last columns are white; a thin frame shows where it ends.
    pg.hline(left, right, top - 1.0, 1, 0);
    pg.hline(left, right, bottom, 1, 0);
    pg.rect(right, top - 1.0, right + 1.0, bottom + 1.0, 0);
    pg.sample(
        "GR".into(),
        "ramp, gray 0 at the left to 255 at the right".into(),
        (left, top, right, bottom),
    );
    pg.finish("grays", "Grays", version)
}

fn lines_sheet(p: &Profile, reg: &mut FontRegistry, version: &str) -> Result<Vec<Page>> {
    let mut pg = Pager::new(p, reg);
    pg.paragraph(
        "Line widths × grays. Row = width: A 1 px, B 2 px, C 3 px, D 4 px, E 5 px, F 6 px. Column = gray \
         (0 black … 255 white): above each column, its number and then its gray. Each cell holds three lines. Code = L + row + column, \
         e.g. LB7 is 2 px at gray 102. For each line role, note the code that looks best.",
        8.0,
        400,
    )?;
    pg.column_header(&LINE_GRAYS)?;
    let cw = (pg.width() / LINE_GRAYS.len() as f64).floor();
    let (pad, label_h, step) = (pg.mm(1.0), pg.mm(3.5), pg.mm(2.5));
    for (r, &w) in LINE_WIDTHS.iter().enumerate() {
        let y0 = pg.row(pg.mm(13.0));
        for (c, &g) in LINE_GRAYS.iter().enumerate() {
            let code = format!("L{}{}", row_letter(r), c + 1);
            let x0 = pg.left + c as f64 * cw;
            pg.text(&code, 6.0, 700, LABEL_GRAY, (x0 + pad, y0), 20.0)?;
            let (a, b) = (x0 + pad, x0 + cw - pad);
            for k in 0..3 {
                pg.hline(a, b, y0 + label_h + pg.mm(1.5) + k as f64 * step, w, g);
            }
            pg.sample(
                code,
                format!("line {w} px, gray {g}"),
                (a, y0 + label_h, b, y0 + pg.mm(13.0)),
            );
        }
    }
    pg.gap(4.0);
    // Legend (up to 3 lines), column heading and the first row of dashes stay together.
    pg.keep(18.0 + 6.0 + 10.0);
    pg.paragraph(
        "Dashed and dotted lines, 2 px wide. Row = dash and gap in pixels: A 2/2, B 2/4, C 6/6, D 12/6, \
         E 24/12. Column = gray (number, then gray, above it). Code = D + row + column.",
        8.0,
        400,
    )?;
    pg.column_header(&DASH_GRAYS)?;
    let cw = (pg.width() / DASH_GRAYS.len() as f64).floor();
    for (r, &(dash, gap)) in DASHES.iter().enumerate() {
        let y0 = pg.row(pg.mm(10.0));
        for (c, &g) in DASH_GRAYS.iter().enumerate() {
            let code = format!("D{}{}", row_letter(r), c + 1);
            let x0 = pg.left + c as f64 * cw;
            pg.text(&code, 6.0, 700, LABEL_GRAY, (x0 + pad, y0), 20.0)?;
            let (a, b) = (x0 + pad, x0 + cw - pad);
            for k in 0..2 {
                let y = y0 + label_h + pg.mm(1.5) + k as f64 * step;
                let mut x = a;
                while x < b {
                    let end = (x + dash as f64).min(b);
                    pg.hline(x, end, y, DASH_WIDTH, g);
                    x += (dash + gap) as f64;
                }
            }
            pg.sample(
                code,
                format!("dashes {dash}/{gap} px, 2 px wide, gray {g}"),
                (a, y0 + label_h, b, y0 + pg.mm(10.0)),
            );
        }
    }
    pg.finish("lines", "Lines", version)
}

fn dots_sheet(p: &Profile, reg: &mut FontRegistry, version: &str) -> Result<Vec<Page>> {
    let mut pg = Pager::new(p, reg);
    pg.paragraph(
        "Dots, 4 mm apart. Rows A–D: square dots 1, 2, 3, 4 px. Rows E–H: round dots 2, 3, 4, 6 px across. \
         Column = gray (number, then gray, above it). Code = P + row + column, e.g. PB6 is a 2 px square dot at gray 187.",
        8.0,
        400,
    )?;
    pg.column_header(&DOT_GRAYS)?;
    let cw = (pg.width() / DOT_GRAYS.len() as f64).floor();
    let (pad, label_h, step) = (pg.mm(1.0), pg.mm(3.5), pg.mm(4.0));
    for (r, &(shape, size)) in DOTS.iter().enumerate() {
        let y0 = pg.row(pg.mm(15.0));
        for (c, &g) in DOT_GRAYS.iter().enumerate() {
            let code = format!("P{}{}", row_letter(r), c + 1);
            let x0 = pg.left + c as f64 * cw;
            pg.text(&code, 6.0, 700, LABEL_GRAY, (x0 + pad, y0), 20.0)?;
            let (dx, dy) = (x0 + pad + pg.mm(1.5), y0 + label_h + pg.mm(1.5));
            for j in 0..3 {
                for i in 0..3 {
                    pg.dot(shape, size, dx + i as f64 * step, dy + j as f64 * step, g);
                }
            }
            let kind = if shape == DotShape::Square {
                "square"
            } else {
                "round"
            };
            pg.sample(
                code,
                format!("{kind} dot {size} px, gray {g}"),
                (x0 + pad, y0 + label_h, x0 + cw - pad, y0 + pg.mm(15.0)),
            );
        }
    }
    pg.finish("dots", "Dots", version)
}

fn text_sheet(p: &Profile, reg: &mut FontRegistry, version: &str) -> Result<Vec<Page>> {
    let mut pg = Pager::new(p, reg);
    pg.paragraph(
        "Text in Atkinson Hyperlegible Next. Rows A–H: regular at 6, 7, 8, 9, 10, 12, 14, 18 pt; rows I–P: \
         bold at the same sizes. Column = gray (number, then gray, above it). Code = T + row + column. Note the smallest size you can read \
         comfortably, and the gray you prefer for labels.",
        8.0,
        400,
    )?;
    pg.column_header(&TEXT_GRAYS)?;
    let cw = (pg.width() / TEXT_GRAYS.len() as f64).floor();
    let cell_mm = cw * MM_PER_IN / p.ppi - 2.0;
    let pad = pg.mm(1.0);
    for (wi, &weight) in TEXT_WEIGHTS.iter().enumerate() {
        for (si, &size) in TEXT_SIZES_PT.iter().enumerate() {
            let r = wi * TEXT_SIZES_PT.len() + si;
            let mut cells = Vec::new();
            let mut h = 0.0f64;
            for (c, &g) in TEXT_GRAYS.iter().enumerate() {
                let code = format!("T{}{}", row_letter(r), c + 1);
                let (path, th) = pg.measure(&format!("{code} Notes"), size, weight, cell_mm)?;
                h = h.max(th);
                cells.push((code, path, g));
            }
            let y0 = pg.row(h + pg.mm(1.5));
            for (c, (code, path, g)) in cells.into_iter().enumerate() {
                let x0 = pg.left + c as f64 * cw + pad;
                pg.put(path, x0, y0, g);
                let style = if weight == 700 { "bold" } else { "regular" };
                pg.sample(
                    code,
                    format!("text {size} pt {style}, gray {g}"),
                    (x0, y0, x0 + cw - 2.0 * pad, y0 + h),
                );
            }
        }
    }
    pg.finish("text", "Text", version)
}

fn writing_sheet(p: &Profile, reg: &mut FontRegistry, version: &str) -> Result<Vec<Page>> {
    let mut pg = Pager::new(p, reg);
    pg.paragraph(
        "Writing strips. Write across each strip with every pen you use. Note the strips whose lines stay \
         visible without competing with your writing. W1–W12: ruled lines 7 mm apart; W13–W18: dot rows 5 mm apart. Set 2: replaces any earlier writing pages.",
        8.0,
        400,
    )?;
    let label_h = pg.mm(3.5);
    let (left, right) = (pg.left, pg.right);
    let mut n = 0;
    for &(w, g) in &STRIP_LINES {
        n += 1;
        let code = format!("W{n}");
        let what = format!("ruled lines {w} px, gray {g}");
        let y0 = pg.row(pg.mm(STRIP_MM));
        pg.text(
            &format!("{code} · {what}"),
            7.0,
            700,
            LABEL_GRAY,
            (left, y0),
            80.0,
        )?;
        for k in 0..3 {
            pg.hline(
                left,
                right,
                y0 + label_h + pg.mm(2.0) + k as f64 * pg.mm(7.0),
                w,
                g,
            );
        }
        pg.sample(
            code,
            what,
            (left, y0 + label_h, right, y0 + pg.mm(STRIP_MM)),
        );
    }
    for &(s, g) in &STRIP_DOTS {
        n += 1;
        let code = format!("W{n}");
        let what = format!("square dots {s} px, gray {g}");
        let y0 = pg.row(pg.mm(STRIP_MM));
        pg.text(
            &format!("{code} · {what}"),
            7.0,
            700,
            LABEL_GRAY,
            (left, y0),
            80.0,
        )?;
        let step = pg.mm(5.0);
        for k in 0..3 {
            let y = y0 + label_h + pg.mm(2.0) + k as f64 * step;
            let mut x = left;
            while x <= right {
                pg.dot(DotShape::Square, s, x, y, g);
                x += step;
            }
        }
        pg.sample(
            code,
            what,
            (left, y0 + label_h, right, y0 + pg.mm(STRIP_MM)),
        );
    }
    pg.finish("writing", "Writing", version)
}
