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

//! Quick fixes (rung 14b bbq, Q6): the one button a finding may carry, worked out with no screen in it.
//! [`offered`] says which fixes a finding has; [`plan`] says what each would change, as [`Edit`]s the window
//! turns into one undo step. A fix acts on the device the finding is for; a change to the look (a width, a
//! gray, a role, a binding) is the shared design, and a change to a place or a text size on a secondary
//! device goes through that device's overrides, which the window does (rung 8).
//!
//! No fix for L08, L09, L10, L11, L13, L14, L15, L19 or L20 (the record's Q6), nor for the pitch of a repeat
//! (L16's other message) nor for L06 on a repeat's copies.

use crate::{Finding, Toolbar, free_placement, judge};
use quadrule_i18n::{Arg, Message};
use quadrule_layout::{RhythmPx, stroke_opt};
use quadrule_model::repeat::Spacing;
use quadrule_model::{
    Document, Fill, FillRole, LineRole, Object, ObjectKind, Rhythm, RhythmOrigin, Style, Uuid,
};
use quadrule_profiles::Profile;

/// A role a fix may offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleChoice {
    /// A line (or text) role.
    Line(LineRole),
    /// A fill role.
    Fill(FillRole),
}

/// One quick fix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// L01, L02: the stroke's width to whole device pixels.
    SnapToPixelGrid,
    /// L03, L05: the nearest clean gray that also shows.
    NearestCleanGray,
    /// L03, L05: a role that fits the object.
    UseRole(RoleChoice),
    /// L06: moved the least that clears the safe zones.
    MoveInsideSafeArea,
    /// L07: the profile's smallest legible size.
    UseMinTextSize,
    /// L16: this object onto the rhythm.
    SnapToRhythm,
    /// L16: every off-rhythm object onto the rhythm.
    SnapAllToRhythm,
    /// L17: the spacing bound to the rhythm.
    BindToRhythm,
    /// L18: the rhythm's origin centred.
    CenterRhythmOrigin,
}

/// One change a fix makes. The window turns these into commands (one undo step for the whole fix).
#[derive(Clone, Debug, PartialEq)]
// A short-lived list of a few edits: the one large variant (a whole object) is not worth boxing.
#[allow(clippy::large_enum_variant)]
pub enum Edit {
    /// The object as it should be, a change to the shared design.
    Object(Object),
    /// Move the object by this much in its own space on the primary page (L16: no turn is carried back).
    Translate {
        /// The object.
        id: Uuid,
        /// Across, in mm.
        dx_mm: f64,
        /// Down, in mm.
        dy_mm: f64,
    },
    /// Move the object by this much on the page of the finding's device, in that device's pixels (L06).
    MovePage {
        /// The object.
        id: Uuid,
        /// Across, in device pixels.
        dx_px: f64,
        /// Down, in device pixels.
        dy_px: f64,
    },
    /// Set a text box's size on the finding's device: the design's on the primary, an override elsewhere.
    TextSize {
        /// The text box.
        id: Uuid,
        /// The size in points.
        pt: f64,
    },
    /// The rhythm as it should be.
    Rhythm(Rhythm),
}

impl Fix {
    /// The button's words, as a message.
    pub fn label(self, doc: &Document, target: &Profile) -> Message {
        match self {
            Fix::SnapToPixelGrid => Message::new("lint-fix-pixel"),
            Fix::NearestCleanGray => Message::new("lint-fix-gray"),
            Fix::UseRole(r) => Message::new("lint-fix-role").arg(
                "role",
                Arg::Msg(Box::new(Message::new(match r {
                    RoleChoice::Line(LineRole::Whisper) => "app-role-whisper",
                    RoleChoice::Line(LineRole::Guide) => "app-role-guide",
                    RoleChoice::Line(LineRole::Structure) => "app-role-structure",
                    RoleChoice::Line(LineRole::Emphasis) => "app-role-emphasis",
                    RoleChoice::Line(LineRole::DotGrid) => "app-role-dotgrid",
                    RoleChoice::Line(LineRole::Text) => "app-role-text",
                    RoleChoice::Line(LineRole::Label) => "app-role-label",
                    RoleChoice::Fill(FillRole::Shade) => "app-fill-shade",
                    RoleChoice::Fill(FillRole::Block) => "app-fill-block",
                }))),
            ),
            Fix::MoveInsideSafeArea => Message::new("lint-fix-safe"),
            Fix::UseMinTextSize => {
                let pt = target.lint.smallest_text_pt.unwrap_or_default();
                Message::new("lint-fix-pt").num("pt", pt, if pt.fract() == 0.0 { 0 } else { 1 })
            }
            Fix::SnapToRhythm => Message::new("lint-fix-rhythm"),
            Fix::SnapAllToRhythm => Message::new("lint-fix-rhythm-all"),
            Fix::BindToRhythm => Message::new("lint-fix-bind"),
            Fix::CenterRhythmOrigin => {
                let _ = doc;
                Message::new("lint-fix-center")
            }
        }
    }
}

/// What part of an object a gray finding is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Stroke,
    Dots,
    Fill,
    Text,
}

/// The part an L03 or L05 finding names, from its message.
fn part_of(f: &Finding) -> Option<Part> {
    match f.message.id {
        "lint-l03-stroke" => Some(Part::Stroke),
        "lint-l03-dots" => Some(Part::Dots),
        "lint-l03-fill" => Some(Part::Fill),
        "lint-l03-text" => Some(Part::Text),
        "lint-l05" => f.message.args.iter().find_map(|(n, a)| match (*n, a) {
            ("what", Arg::Msg(m)) => match m.id {
                "lint-l05-line" => Some(Part::Stroke),
                "lint-l05-dots" => Some(Part::Dots),
                "lint-l05-fill" => Some(Part::Fill),
                "lint-l05-text" => Some(Part::Text),
                _ => None,
            },
            _ => None,
        }),
        _ => None,
    }
}

/// How an object's part is judged on `target`: its gray as laid down, the blend it makes on the page, and the
/// lightest gray the profile lets it have.
struct Judged {
    /// The part's own gray (before opacity).
    own: u8,
    /// The stroke width in device pixels (1 for fills and text).
    width_px: u32,
    /// The lightest gray that shows (None: no limit).
    lightest: Option<u8>,
    /// How opaque it is on the page, 0 to 1.
    alpha: f64,
}

fn judged(doc: &Document, o: &Object, target: &Profile, part: Part) -> Option<Judged> {
    let alpha = judge::opacity_on_page(doc, o);
    let t = &target.lint;
    match part {
        Part::Fill => {
            let own = target.fill_gray(o.fill?);
            Some(Judged {
                own,
                width_px: 1,
                lightest: t.lightest_gray_fill,
                alpha,
            })
        }
        Part::Text => {
            let (w, own) = stroke_opt(o.style, target)?;
            Some(Judged {
                own,
                width_px: w,
                lightest: t.lightest_gray_text,
                alpha,
            })
        }
        Part::Stroke | Part::Dots => {
            let (w, own) = stroke_opt(o.style, target)?;
            let lightest = if part == Part::Dots {
                t.lightest_gray_dot(w)
            } else {
                t.lightest_gray_line(w)
            };
            Some(Judged {
                own,
                width_px: w,
                lightest,
                alpha,
            })
        }
    }
}

/// Whether a gray, laid down as `own` over the page, passes L03 (not too light) and L05 (a clean level).
fn passes(doc: &Document, target: &Profile, j: &Judged, own: u8) -> bool {
    let g = judge::gray_on_page(own, j.alpha, doc.page.background_gray);
    j.lightest.is_none_or(|l| g <= l)
        && (target.gray_levels.is_empty() || target.gray_levels.contains(&g))
}

/// The gray nearest the current one that passes both rules, darker when two are as near.
fn nearest_gray(doc: &Document, target: &Profile, j: &Judged) -> Option<u8> {
    (0..=255u8)
        .filter(|g| passes(doc, target, j, *g))
        .min_by_key(|g| (i32::from(*g) - i32::from(j.own)).abs() * 2 + i32::from(*g > j.own))
}

/// The roles that fit the part: they pass both rules on this device and, for a stroke, come within a pixel of its
/// width. The role it already has is left out.
fn fitting_roles(doc: &Document, o: &Object, target: &Profile, part: Part) -> Vec<RoleChoice> {
    let Some(j) = judged(doc, o, target, part) else {
        return Vec::new();
    };
    match part {
        Part::Fill => FillRole::ALL
            .iter()
            .filter(|r| o.fill != Some(Fill::Role(**r)))
            .filter(|r| passes(doc, target, &j, target.fill_role_gray(**r)))
            .map(|r| RoleChoice::Fill(*r))
            .collect(),
        Part::Text => [LineRole::Text, LineRole::Label]
            .into_iter()
            .filter(|r| o.style != Style::Role(*r))
            .filter(|r| passes(doc, target, &j, target.role(*r).gray))
            .map(RoleChoice::Line)
            .collect(),
        Part::Stroke | Part::Dots => {
            let pool: &[LineRole] = if part == Part::Dots {
                &[LineRole::DotGrid]
            } else {
                &[
                    LineRole::Whisper,
                    LineRole::Guide,
                    LineRole::Structure,
                    LineRole::Emphasis,
                ]
            };
            pool.iter()
                .copied()
                .filter(|r| o.style != Style::Role(*r))
                .filter(|r| {
                    let s = target.role(*r);
                    s.width_px.abs_diff(j.width_px) <= 1 && passes(doc, target, &j, s.gray)
                })
                .map(RoleChoice::Line)
                .collect()
        }
    }
}

/// The active safe zones on `target` as pixel boxes (x0, y0, x1, y1): the toolbars docked left, right and top, and
/// the footers; none on paper. Fixed zones first when `fixed_only` asks for them alone.
fn zone_boxes(target: &Profile, fixed_only: bool) -> Vec<(f64, f64, f64, f64)> {
    if target.is_paper() {
        return Vec::new();
    }
    target
        .safe_zones
        .iter()
        .filter(|z| !(fixed_only && z.advisory))
        .filter(|z| match Toolbar::of_zone(&z.name) {
            Ok(Some(t)) => Toolbar::DOCKED.contains(&t),
            _ => true,
        })
        .map(|z| {
            let (x, y, w, h) = (
                z.rect.0 as f64,
                z.rect.1 as f64,
                z.rect.2 as f64,
                z.rect.3 as f64,
            );
            (x, y, x + w, y + h)
        })
        .collect()
}

/// The least move that takes a box (x0, y0, x1, y1) out of every zone and, when it fits, keeps it on the page.
fn clear_of(
    b: (f64, f64, f64, f64),
    zones: &[(f64, f64, f64, f64)],
    page: (f64, f64),
) -> Option<(f64, f64)> {
    let (w, h) = (b.2 - b.0, b.3 - b.1);
    let mut xs = vec![b.0];
    let mut ys = vec![b.1];
    for z in zones {
        xs.extend([z.2, z.0 - w]);
        ys.extend([z.3, z.1 - h]);
    }
    let fits_x = w <= page.0;
    let fits_y = h <= page.1;
    let mut best: Option<((f64, f64), f64)> = None;
    for &x in &xs {
        for &y in &ys {
            if (fits_x && (x < -1e-9 || x + w > page.0 + 1e-9))
                || (fits_y && (y < -1e-9 || y + h > page.1 + 1e-9))
            {
                continue;
            }
            let hit = zones.iter().any(|z| {
                x < z.2 - 1e-9 && x + w > z.0 + 1e-9 && y < z.3 - 1e-9 && y + h > z.1 + 1e-9
            });
            if hit {
                continue;
            }
            let d = (x - b.0, y - b.1);
            let cost = d.0 * d.0 + d.1 * d.1;
            if best.is_none_or(|(_, c)| cost < c - 1e-9) {
                best = Some((d, cost));
            }
        }
    }
    best.map(|(d, _)| d)
}

/// The move that takes the finding's object clear of the safe zones, in the device's pixels.
fn safe_move(doc: &Document, target: &Profile, f: &Finding) -> Option<(f64, f64)> {
    let a = f.area?;
    let _ = doc;
    let page = (f64::from(target.width_px), f64::from(target.height_px));
    let b = (a.x0, a.y0, a.x1, a.y1);
    clear_of(b, &zone_boxes(target, false), page)
        .or_else(|| clear_of(b, &zone_boxes(target, true), page))
        .filter(|d| d.0.abs() > 1e-9 || d.1.abs() > 1e-9)
}

/// The nearest rhythm line to `pos` (pixels), given the origin and the unit.
fn nearest_line(pos: f64, origin: f64, unit: f64) -> f64 {
    origin + ((pos - origin) / unit).round() * unit
}

/// How far an object's top-left corner must move, in mm, to sit on the rhythm on the primary page; `None` when it
/// already does (L16's own test: within half a pixel).
fn rhythm_shift(doc: &Document, primary: &Profile, o: &Object) -> Option<(f64, f64)> {
    let r = RhythmPx::new(doc, primary);
    let ppmm = primary.mm_to_px(1.0);
    let on = |pos: f64, origin: f64, unit: f64| {
        let m = (pos - origin).rem_euclid(unit);
        m.min(unit - m) < 0.5
    };
    let (x, y) = (
        primary.mm_to_px(o.frame.x_mm).round(),
        primary.mm_to_px(o.frame.y_mm).round(),
    );
    let dx = if on(x, r.origin_x, r.unit_x) {
        0.0
    } else {
        (nearest_line(x, r.origin_x, r.unit_x) - primary.mm_to_px(o.frame.x_mm)) / ppmm
    };
    let dy = if on(y, r.origin_y, r.unit_y) {
        0.0
    } else {
        (nearest_line(y, r.origin_y, r.unit_y) - primary.mm_to_px(o.frame.y_mm)) / ppmm
    };
    (dx != 0.0 || dy != 0.0).then_some((dx, dy))
}

/// Which fixes the finding has, in the order the buttons show. `target` is the profile of the finding's device;
/// `primary` the document's.
pub fn offered(doc: &Document, primary: &Profile, target: &Profile, f: &Finding) -> Vec<Fix> {
    let object = f.object.and_then(|id| doc.object(id));
    let mut out = Vec::new();
    match f.rule {
        "L01" | "L02" => {
            if matches!(f.message.id, "lint-l01-thin" | "lint-l02-fraction")
                && object.is_some_and(|o| matches!(o.style, Style::Custom { .. }))
            {
                out.push(Fix::SnapToPixelGrid);
            }
        }
        "L03" | "L05" => {
            if let (Some(o), Some(part)) = (object, part_of(f)) {
                if let Some(j) = judged(doc, o, target, part)
                    && nearest_gray(doc, target, &j).is_some()
                {
                    out.push(Fix::NearestCleanGray);
                }
                out.extend(
                    fitting_roles(doc, o, target, part)
                        .into_iter()
                        .map(Fix::UseRole),
                );
            }
        }
        "L06" => {
            if matches!(f.message.id, "lint-l06-fixed" | "lint-l06-movable")
                && safe_move(doc, target, f).is_some()
            {
                out.push(Fix::MoveInsideSafeArea);
            }
        }
        "L07" => {
            if object.is_some_and(|o| matches!(o.kind, ObjectKind::Text(_)))
                && target.lint.smallest_text_pt.is_some()
            {
                out.push(Fix::UseMinTextSize);
            }
        }
        "L16" => {
            if f.message.id == "lint-l16"
                && target.id == primary.id
                && object.is_some_and(|o| !free_placement(doc, o))
                && object.is_some_and(|o| rhythm_shift(doc, primary, o).is_some())
            {
                out.push(Fix::SnapToRhythm);
                out.push(Fix::SnapAllToRhythm);
            }
        }
        "L17" => {
            if object.is_some_and(|o| {
                matches!(&o.kind, ObjectKind::Pattern(p) if !p.bound) || o.repeat().is_some()
            }) {
                out.push(Fix::BindToRhythm);
            }
        }
        "L18" if doc.rhythm.origin != RhythmOrigin::Centered => {
            out.push(Fix::CenterRhythmOrigin);
        }
        _ => {}
    }
    out
}

/// Why a fix could not be worked out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoFix;

/// What `fix` changes for finding `f`. `target` is the profile of the finding's device. For
/// [`Fix::SnapAllToRhythm`] use [`plan_snap_all`].
pub fn plan(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    f: &Finding,
    fix: Fix,
) -> Result<Vec<Edit>, NoFix> {
    let object = f.object.and_then(|id| doc.object(id));
    match fix {
        Fix::SnapToPixelGrid => {
            let mut o = object.ok_or(NoFix)?.clone();
            let Style::Custom { width_mm, gray } = o.style else {
                return Err(NoFix);
            };
            let ppmm = target.mm_to_px(1.0);
            let px = (width_mm * ppmm).round().max(1.0);
            o.style = Style::Custom {
                width_mm: px / ppmm,
                gray,
            };
            Ok(vec![Edit::Object(o)])
        }
        Fix::NearestCleanGray => {
            let o = object.ok_or(NoFix)?;
            let part = part_of(f).ok_or(NoFix)?;
            let j = judged(doc, o, target, part).ok_or(NoFix)?;
            let g = nearest_gray(doc, target, &j).ok_or(NoFix)?;
            let mut o = o.clone();
            if part == Part::Fill {
                o.fill = Some(Fill::Gray(g));
            } else {
                o.style = Style::Custom {
                    width_mm: f64::from(j.width_px) / target.mm_to_px(1.0),
                    gray: g,
                };
            }
            Ok(vec![Edit::Object(o)])
        }
        Fix::UseRole(role) => {
            let mut o = object.ok_or(NoFix)?.clone();
            match role {
                RoleChoice::Line(r) => o.style = Style::Role(r),
                RoleChoice::Fill(r) => o.fill = Some(Fill::Role(r)),
            }
            Ok(vec![Edit::Object(o)])
        }
        Fix::MoveInsideSafeArea => {
            let id = f.object.ok_or(NoFix)?;
            let (dx_px, dy_px) = safe_move(doc, target, f).ok_or(NoFix)?;
            Ok(vec![Edit::MovePage { id, dx_px, dy_px }])
        }
        Fix::UseMinTextSize => {
            let id = f.object.ok_or(NoFix)?;
            let pt = target.lint.smallest_text_pt.ok_or(NoFix)?;
            Ok(vec![Edit::TextSize { id, pt }])
        }
        Fix::SnapToRhythm => {
            let o = object.ok_or(NoFix)?;
            let (dx_mm, dy_mm) = rhythm_shift(doc, primary, o).ok_or(NoFix)?;
            Ok(vec![Edit::Translate {
                id: o.id,
                dx_mm,
                dy_mm,
            }])
        }
        Fix::SnapAllToRhythm => plan_snap_all(doc, primary, std::slice::from_ref(f)),
        Fix::BindToRhythm => {
            let mut o = object.ok_or(NoFix)?.clone();
            if let Some(r) = o.repeat_mut() {
                let (ax, ay) = r.axes();
                if ax && !r.h_spacing.is_bound() {
                    r.h_spacing = Spacing::Auto;
                }
                if ay && !r.v_spacing.is_bound() {
                    r.v_spacing = Spacing::Auto;
                }
            } else if let ObjectKind::Pattern(p) = &mut o.kind {
                p.bound = true;
            } else {
                return Err(NoFix);
            }
            Ok(vec![Edit::Object(o)])
        }
        Fix::CenterRhythmOrigin => {
            let mut r = doc.rhythm.clone();
            r.origin = RhythmOrigin::Centered;
            Ok(vec![Edit::Rhythm(r)])
        }
    }
}

/// Every off-rhythm object onto the rhythm: the objects the `findings` name with L16's position message, once each,
/// leaving Free placement and anything locked.
pub fn plan_snap_all(
    doc: &Document,
    primary: &Profile,
    findings: &[Finding],
) -> Result<Vec<Edit>, NoFix> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for f in findings
        .iter()
        .filter(|f| f.rule == "L16" && f.message.id == "lint-l16" && f.profile == primary.id)
    {
        let Some(id) = f.object else { continue };
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let Some(o) = doc.object(id) else { continue };
        if free_placement(doc, o) || !doc.editable(id) {
            continue;
        }
        if let Some((dx_mm, dy_mm)) = rhythm_shift(doc, primary, o) {
            out.push(Edit::Translate { id, dx_mm, dy_mm });
        }
    }
    if out.is_empty() { Err(NoFix) } else { Ok(out) }
}
