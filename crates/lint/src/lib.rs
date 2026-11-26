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

//! The linter (napkin §8.3): checks a document on one profile and explains what may not look
//! right on that device. It recommends and never blocks (§8.1).
//!
//! Rung 14a adds the rules that read the profile's `lint` block ([`judge`]): L03 gray too light, L05 gray off
//! the clean levels, L07 text too small, L09 handwriting step, L13 dark fills and L14 soft thin lines; the
//! document's ignored rules and an object's Free placement are honoured, and [`skipped_checks`] says which
//! checks did not run on a device and why. Built before that, the rules that need no calibration values:
//! L01 stroke thinner than 1 px, L02 stroke not pixel-aligned, L06 content in a safe zone,
//! L08 a Fixed text box leaves lines out (rung 11b),
//! L10 off the page, L15 constraints cause trouble on a secondary device (rung 8a), L16 position
//! off the rhythm, L18 rhythm fits the margins unevenly, L19 uneven physical-locked spacing. L04 (ink) is retired (ADR-027) and L12 waits for the feature it checks. Rung 12a
//! adds L11 (a text box drawn from saved outlines because its font is missing) and L20 (fonts
//! installed on this computer are used). Rung 10: a repeat's source is judged once, L06 and L10 judge every drawn
//! copy (one finding per repeat and device, with the count), and L16 and L17 judge its pitch.

#![forbid(unsafe_code)]

mod fix;
mod judge;

pub use fix::{Edit, Fix, NoFix, RoleChoice, offered, plan, plan_snap_all};

use quadrule_i18n::{Arg, Message};
use quadrule_layout::tree::{Heights, Node, resolve_nodes_with};
use quadrule_layout::{RhythmPx, Shape, placed_objects};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{Document, Frame, ObjectKind, RhythmResolution, Style, Uuid};
use quadrule_profiles::Profile;
use std::collections::HashMap;

/// How much a finding matters (napkin §8.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Worth knowing.
    Info,
    /// Likely to look wrong on the device.
    Warn,
}

/// One thing the linter noticed.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// The rule, e.g. `L06`.
    pub rule: &'static str,
    /// How much it matters.
    pub severity: Severity,
    /// The profile it was found on.
    pub profile: String,
    /// The object, when the finding is about one.
    pub object: Option<Uuid>,
    /// What is wrong and what to do, as a message for `quadrule_i18n::Strings::format`.
    pub message: Message,
    /// Where on the page, in device pixels of `profile`, when the finding is about a place: the
    /// object's laid-out box, half its stroke included. It may reach past the page edge (L10).
    pub area: Option<PxRect>,
}

/// A box in device pixels: left, top, right, bottom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PxRect {
    /// Left edge.
    pub x0: f64,
    /// Top edge.
    pub y0: f64,
    /// Right edge.
    pub x1: f64,
    /// Bottom edge.
    pub y1: f64,
}

/// An object's box in device pixels: (x0, y0, x1, y1).
type Bounds = (f64, f64, f64, f64);

fn px_rect(b: Bounds) -> PxRect {
    PxRect {
        x0: b.0,
        y0: b.1,
        x1: b.2,
        y1: b.3,
    }
}

/// Where the user puts the device's toolbar. L06 checks only the positions in use; the footers
/// are checked always.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toolbar {
    /// Docked along the left edge (`toolbar_left`).
    Left,
    /// Docked along the right edge (`toolbar_right`).
    Right,
    /// Docked along the top edge (`toolbar_top`).
    Top,
    /// Floating, where it first appears (`toolbar_floating_default`).
    Floating,
}

impl Toolbar {
    /// The docked positions: what L06 checks unless told otherwise. The floating toolbar moves,
    /// so it is left out (chris, 2026-11-10: the linter "still checks for collisions with the
    /// floating menu bar, which isn't desired").
    pub const DOCKED: [Toolbar; 3] = [Toolbar::Left, Toolbar::Right, Toolbar::Top];

    /// Every position, the floating one included (opt-in).
    pub const ALL: [Toolbar; 4] = [
        Toolbar::Left,
        Toolbar::Right,
        Toolbar::Top,
        Toolbar::Floating,
    ];

    /// The position a profile's safe zone belongs to: `None` for a zone that is not a toolbar
    /// (the footers), `Err` for a toolbar zone this list does not know.
    pub fn of_zone(name: &str) -> Result<Option<Toolbar>, String> {
        match name.strip_prefix("toolbar_") {
            None => Ok(None),
            Some("left") => Ok(Some(Toolbar::Left)),
            Some("right") => Ok(Some(Toolbar::Right)),
            Some("top") => Ok(Some(Toolbar::Top)),
            Some("floating_default") => Ok(Some(Toolbar::Floating)),
            Some(_) => Err(format!("unknown toolbar zone `{name}`")),
        }
    }

    /// Parses `left`, `right`, `top` or `floating`.
    pub fn parse(s: &str) -> Option<Toolbar> {
        match s {
            "left" => Some(Toolbar::Left),
            "right" => Some(Toolbar::Right),
            "top" => Some(Toolbar::Top),
            "floating" => Some(Toolbar::Floating),
            _ => None,
        }
    }
}

/// Choices that change what the linter checks.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    /// Toolbar positions the user uses; L06 ignores the others. Default: the docked ones.
    pub toolbars: Vec<Toolbar>,
    /// What the window knows about fonts (rung 12a), which the linter cannot look up for itself.
    pub fonts: FontFacts,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            toolbars: Toolbar::DOCKED.to_vec(),
            fonts: FontFacts::default(),
        }
    }
}

/// What the window knows about the document's fonts on this computer (rung 12a): which text boxes are
/// drawn from saved outlines on the device being linted (L11), and which fonts installed here the
/// page uses (L20).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FontFacts {
    /// The text boxes whose font is missing here, as drawn on this device.
    pub missing: Vec<MissingBox>,
    /// The fonts the page uses that come from this computer's installed fonts or from the file's
    /// embedded ones (not bundled ones), chosen or as fallback.
    pub installed: Vec<InstalledFont>,
}

/// A text box whose font is missing on this computer.
#[derive(Clone, Debug, PartialEq)]
pub struct MissingBox {
    /// The text box.
    pub id: Uuid,
    /// The font it names.
    pub family: String,
    /// True when the file saved no shapes for it on this device and none could be borrowed from
    /// another device (ADR-001 item 7): it draws nothing.
    pub needs_font: bool,
}

/// A font used that is not bundled.
#[derive(Clone, Debug, PartialEq)]
pub struct InstalledFont {
    /// The family.
    pub family: String,
    /// True when it is used only as fallback for a letter another font lacks (ADR-002 item 6).
    pub fallback: bool,
    /// Whether the file carries it.
    pub embed: EmbedStatus,
}

/// Whether an installed font is saved inside the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbedStatus {
    /// It is embedded.
    Embedded,
    /// It is not: "Embed fonts" is off for this document.
    Off,
    /// It cannot be: the id of the message that gives the licence reason.
    Cannot(&'static str),
}

/// Lints the document on `target` with the default options (every toolbar position).
/// `outlines` is the text laid out for `target` (live or saved); it gives text boxes their
/// height.
pub fn lint(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    outlines: &ProfileOutlines,
) -> Vec<Finding> {
    lint_with(doc, primary, target, outlines, &Options::default())
}

/// Lints the document on `target` with `opts`.
pub fn lint_with(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    outlines: &ProfileOutlines,
    opts: &Options,
) -> Vec<Finding> {
    lint_against(doc, primary, target, outlines, None, opts)
}

/// [`lint_with`] with the text laid out for the primary profile too (`primary_outlines`), so L15
/// compares a text box that is as tall as its text with the height it has on the primary page, not
/// the one it is stored with (rung 11b, paying the rung 8a debt). Without it the stored height is
/// used on the primary.
pub fn lint_against(
    doc: &Document,
    primary: &Profile,
    target: &Profile,
    outlines: &ProfileOutlines,
    primary_outlines: Option<&ProfileOutlines>,
    opts: &Options,
) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut push = |rule, severity, object, area: Option<Bounds>, message: Message| {
        out.push(Finding {
            rule,
            severity,
            profile: target.id.clone(),
            object,
            message,
            area: area.map(px_rect),
        })
    };
    let rhythm = RhythmPx::new(doc, target);
    let (pw, ph) = (target.width_px as f64, target.height_px as f64);

    // Page-wide rhythm rules.
    if doc.rhythm.resolution == RhythmResolution::PhysicalLocked
        && (rhythm.unit_x.fract() != 0.0 || rhythm.unit_y.fract() != 0.0)
    {
        push(
            "L19",
            Severity::Info,
            None,
            None,
            Message::new("lint-l19")
                .num("x", rhythm.unit_x, 2)
                .num("y", rhythm.unit_y, 2),
        );
    }
    // L20: fonts installed on this computer are used (rung 12a bbq, Q7). Per document, so it is said
    // once, on the primary device.
    if target.id == primary.id && !opts.fonts.installed.is_empty() {
        let items: Vec<Message> = opts
            .fonts
            .installed
            .iter()
            .map(|f| {
                let status = match f.embed {
                    EmbedStatus::Embedded => Message::new("lint-l20-embedded"),
                    EmbedStatus::Off => Message::new("lint-l20-off"),
                    EmbedStatus::Cannot(reason) => Message::new("lint-l20-cannot")
                        .arg("reason", Arg::Msg(Box::new(Message::new(reason)))),
                };
                Message::new(if f.fallback {
                    "lint-l20-item-fallback"
                } else {
                    "lint-l20-item"
                })
                .text("font", f.family.clone())
                .arg("status", Arg::Msg(Box::new(status)))
            })
            .collect();
        push(
            "L20",
            Severity::Warn,
            None,
            None,
            Message::new("lint-l20").arg(
                "fonts",
                Arg::List {
                    items,
                    sep: "lint-l20-sep",
                },
            ),
        );
    }
    // L18: the rhythm fits unevenly when the strip it leaves at one side of the margin box is
    // more than half a unit wider than the strip at the other side.
    let m = &doc.page.margins_mm;
    let edge = |mm: f64| target.mm_to_px(mm).round();
    let axes = [
        (
            "lint-side-left",
            "lint-side-right",
            rhythm.origin_x,
            rhythm.unit_x,
            edge(m.left),
            pw - edge(m.right),
        ),
        (
            "lint-side-top",
            "lint-side-bottom",
            rhythm.origin_y,
            rhythm.unit_y,
            edge(m.top),
            ph - edge(m.bottom),
        ),
    ];
    for (near, far, origin, unit, start, end) in axes {
        // `+ 0.0` turns a negative zero into zero, which would otherwise read "-0 px".
        let (a, b) = (
            (origin - start).rem_euclid(unit) + 0.0,
            (end - origin).rem_euclid(unit) + 0.0,
        );
        if (a - b).abs() > unit / 2.0 {
            push(
                "L18",
                Severity::Info,
                None,
                None,
                Message::new("lint-l18")
                    .num("a", a, 0)
                    .arg("near", Arg::Msg(Box::new(Message::new(near))))
                    .num("b", b, 0)
                    .arg("far", Arg::Msg(Box::new(Message::new(far))))
                    .num("unit", unit, 0),
            );
        }
    }

    // L09 and L13 are for a Template; a Wallpaper is not handwritten on (rung 14 bbq, Defaulted).
    let judged_for_template = judge::template_only(doc);
    // L09: the rhythm's own vertical step is under the profile's smallest comfortable handwriting step. An isometric
    // rhythm is for drawing, not for writing between lines (ADR-021, item 3).
    if judged_for_template
        && doc.rhythm.rhythm_type != quadrule_model::RhythmType::Isometric
        && let Some((min_mm, min_px)) = judge::handwriting_min_px(target)
        && rhythm.unit_y + 1e-9 < min_px
    {
        push(
            "L09",
            Severity::Info,
            None,
            None,
            Message::new("lint-l09")
                .num("step", doc.rhythm.steps_mm().1, 1)
                .num("min", min_mm, 1),
        );
    }

    // Every leaf object at its resolved, turned place (rung 7a bbq, Defaulted "Lint"): a group has
    // no rules of its own, and what it holds is judged where it falls on the page. The places come
    // from layout, the one copy of the maths.
    // L15 compares with the primary page (rung 8a, Q7): the stored design, laid out against itself.
    let on_secondary = target.id != primary.id;
    let primary_heights = primary_outlines.map(|o| quadrule_layout::text_heights(doc, o));
    let primary_nodes: HashMap<Uuid, Node> = if on_secondary {
        resolve_nodes_with(
            doc,
            primary.page_mm(),
            primary.page_mm(),
            None,
            None,
            primary_heights.as_ref().map_or(Heights::NONE, Heights::of),
        )
        .into_iter()
        .map(|n| (n.id, n))
        .collect()
    } else {
        HashMap::new()
    };
    let placed_all = placed_objects(doc, primary, target, outlines);
    // What is inside a repeat's source is judged once, at its first cell; the rules that look at
    // where it falls (L06, L10) are judged over every copy below, one finding per repeat.
    let in_repeat = |id: Uuid| {
        doc.ancestors(id)
            .iter()
            .any(|a| doc.object(*a).is_some_and(|o| o.is_repeat()))
    };
    for (index, placed) in placed_all.iter().enumerate() {
        let node = placed.node;
        let Some(o) = doc.object(placed.id) else {
            continue;
        };
        if (node.group && !node.repeat) || node.hidden || placed.copy {
            continue;
        }
        let inside_repeat = !node.repeat && in_repeat(o.id);
        let f = node.frame;
        let px = |mm: f64| target.mm_to_px(mm);
        let rotation = node.angle;
        let half_stroke = quadrule_layout::stroke(o.style, target).0 as f64 / 2.0;
        let b = bounds(&placed.shape, o.kind.is_lettered(), half_stroke);
        let frame_px = (
            px(f.x_mm).min(px(f.x_mm + f.w_mm)),
            px(f.y_mm).min(px(f.y_mm + f.h_mm)),
            px(f.x_mm).max(px(f.x_mm + f.w_mm)),
            px(f.y_mm).max(px(f.y_mm + f.h_mm)),
        );
        // A turned shape, text box or pattern region meets a zone by its turned corners (rung 6b;
        // text, rung 11b; patterns, rung 12b).
        let is_text = o.kind.is_lettered();
        let is_pattern = matches!(o.kind, ObjectKind::Pattern(_));
        let turned = rotation != 0.0 && (o.kind.is_shape() || is_text || is_pattern);
        let at = |id| Message::new(id).text("name", o.name.clone());
        let mut push = |rule, severity, message| push(rule, severity, Some(o.id), Some(b), message);

        // L01 and L02: strokes. A shape reads its own stroke the same way a line does (rung 6 bbq,
        // Q8). L02 asks only about straight, unrotated edges: a curve or a slanted edge is soft by
        // design (Q2), so a stroke width that is not whole pixels does not matter there.
        let stroked = !o.kind.is_lettered();
        let hard_edges = quadrule_layout::shapes::has_axis_aligned_edge(&o.kind, rotation);
        if stroked && let Style::Custom { width_mm, .. } = o.style {
            let exact = px(width_mm);
            if exact < 1.0 {
                push(
                    "L01",
                    Severity::Warn,
                    at("lint-l01-thin").num("px", exact, 2).num(
                        "min_mm",
                        target.mm_to_px(1.0).recip(),
                        3,
                    ),
                );
            } else if hard_edges && (exact - exact.round()).abs() > 1e-6 {
                push(
                    "L02",
                    Severity::Info,
                    at("lint-l02-fraction")
                        .num("px", exact, 2)
                        .num("drawn", exact.round(), 0),
                );
            }
        }
        // L01 and L02 for the border of a pattern region, which has a stroke of its own.
        if let ObjectKind::Pattern(pr) = &o.kind
            && let Some(b) = &pr.border
            && b.sides.any()
            && let Style::Custom { width_mm, .. } = b.style
        {
            let exact = px(width_mm);
            if exact < 1.0 {
                push(
                    "L01",
                    Severity::Warn,
                    at("lint-l01-border-thin").num("px", exact, 2).num(
                        "min_mm",
                        target.mm_to_px(1.0).recip(),
                        3,
                    ),
                );
            } else if b.radius_mm() == 0.0
                && quadrule_layout::rotate::is_quarter_turn(rotation)
                && (exact - exact.round()).abs() > 1e-6
            {
                push(
                    "L02",
                    Severity::Info,
                    at("lint-l02-border-fraction").num("px", exact, 2).num(
                        "drawn",
                        exact.round(),
                        0,
                    ),
                );
            }
        }
        // A line is diagonal when its two ends, turned as the line is, differ in both x and y. That is a
        // soft thin line (L14, below), no longer an L02 note: L02 keeps only straight edges (rung 14 bbq, Q4).
        let mut diagonal = false;
        if let ObjectKind::Line(line) = &o.kind {
            let ((x0, y0), (x1, y1)) = quadrule_layout::rotate::rotated_line_ends(
                (
                    px(f.x_mm),
                    px(f.y_mm),
                    px(f.x_mm + f.w_mm),
                    px(f.y_mm + f.h_mm),
                ),
                line.direction,
                rotation,
            );
            diagonal = (x1 - x0).abs() > 1e-6 && (y1 - y0).abs() > 1e-6;
        }

        // L03, L05, L07 and L14: gray, text size and thin soft lines, read from this profile's lint block.
        let size_pt = match &o.kind {
            ObjectKind::Text(t) if on_secondary => Some(
                o.override_on(&target.id)
                    .and_then(|x| x.font_size_pt)
                    .filter(|s| s.is_finite() && *s > 0.0)
                    .unwrap_or(t.size_pt),
            ),
            ObjectKind::Text(t) => Some(t.size_pt),
            _ => None,
        };
        for h in judge::object_hits(doc, o, target, size_pt, rotation, diagonal) {
            push(h.rule, h.severity, h.message);
        }

        // L07 for a calendar's words (calendar bbq, Defaulted "Text drawn by the pattern"): they shrink to fit their
        // cells, and a kind of word that would be smaller than the smallest text size here is left out.
        if let Some(t_min) = target.lint.smallest_text_pt {
            let left_out = match &o.kind {
                ObjectKind::Pattern(pr) => {
                    let env = quadrule_layout::pattern::Env {
                        target,
                        rhythm: &doc.rhythm,
                        rhythm_px: rhythm,
                        labels: None,
                    };
                    let region = quadrule_layout::region_px(&f, target);
                    let g = quadrule_layout::pattern::region_grid(pr, &env, region);
                    quadrule_layout::calendar::month_plan(&pr.pattern, region, (g.ux, g.uy), target)
                        .map(|p| p.left_out)
                        .unwrap_or_default()
                }
                ObjectKind::MiniCalendar(m) => {
                    quadrule_layout::calendar::mini_plan(
                        m,
                        quadrule_layout::region_px(&f, target),
                        0,
                        target,
                    )
                    .left_out
                }
                _ => Vec::new(),
            };
            for l in left_out {
                use quadrule_layout::calendar::LabelKind;
                let what = match l.kind {
                    LabelKind::Weekday => "lint-l07-calendar-weekdays",
                    LabelKind::Day => "lint-l07-calendar-days",
                    LabelKind::Title => "lint-l07-calendar-title",
                };
                push(
                    "L07",
                    Severity::Warn,
                    at("lint-l07-calendar")
                        .arg(
                            "what",
                            quadrule_i18n::Arg::Msg(Box::new(Message::new(what))),
                        )
                        .num("pt", l.pt, 1)
                        .num("min", t_min, 1),
                );
            }
        }

        // L09 for a ruled region: its own vertical step is under the handwriting step. A region bound to
        // the rhythm with nothing of its own is already said by the rhythm's finding.
        if judged_for_template
            && let Some((min_mm, min_px)) = judge::handwriting_min_px(target)
            && let ObjectKind::Pattern(pr) = &o.kind
            && matches!(pr.pattern, quadrule_model::Pattern::Lines)
        {
            let env = quadrule_layout::pattern::Env {
                target,
                rhythm: &doc.rhythm,
                rhythm_px: rhythm,
                labels: None,
            };
            let (_, step_px) = quadrule_layout::pattern::region_spacing_px(pr, &env);
            let same_as_rhythm = pr.bound && (step_px - rhythm.unit_y).abs() < 1e-9;
            if step_px + 1e-9 < min_px && !(same_as_rhythm && rhythm.unit_y + 1e-9 < min_px) {
                push(
                    "L09",
                    Severity::Info,
                    at("lint-l09-region")
                        .num("step", step_px / target.mm_to_px(1.0), 1)
                        .num("min", min_mm, 1),
                );
            }
        }

        // L02: a pattern region whose frame, or whose ruling inside it, is turned off a quarter turn
        // (rung 12b, Q8 as amended): its lines and dots no longer sit on whole pixels, so they come
        // out softer. A quarter turn keeps them sharp.
        if let ObjectKind::Pattern(pr) = &o.kind
            && !(quadrule_layout::rotate::is_quarter_turn(rotation)
                && quadrule_layout::rotate::is_quarter_turn(pr.pattern_angle()))
        {
            push("L02", Severity::Info, at("lint-l02-turned-pattern"));
        }

        // L08: a Fixed text box leaves whole lines out on this device (rung 11b, Q5). Judged per
        // device, since the font size may be overridden on one.
        if let ObjectKind::Text(t) = &o.kind
            && t.sizing == quadrule_model::TextSizing::Fixed
            && let Some(laid) = outlines.text(o.id)
            && laid.lines_hidden > 0
        {
            let device = {
                let mut c = target.short_name.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            };
            push(
                "L08",
                Severity::Warn,
                at("lint-l08").arg("device", Arg::Text(device)).num(
                    "count",
                    f64::from(laid.lines_hidden),
                    0,
                ),
            );
        }

        // L11: the font is missing here, so the box is drawn from the shapes the file saved (or, where
        // it saved none for this device, not at all). Judged per box and per device (rung 12a, Q6).
        if let Some(m) = opts.fonts.missing.iter().find(|m| m.id == o.id) {
            push(
                "L11",
                Severity::Warn,
                at(if m.needs_font {
                    "lint-l11-needs"
                } else {
                    "lint-l11-saved"
                })
                .text("font", m.family.clone()),
            );
        }

        // L10: off the page. A repeat and what is in its source are judged over all copies below. The frame is
        // judged, not the stroke: half a stroke past an edge the object sits on is not off the page (ADR-019,
        // item 2). An object marked Full bleed is left alone (item 1).
        let bleed = full_bleed(doc, o);
        let slack = if is_text { 0.0 } else { half_stroke };
        let inside = b.0 >= -slack && b.1 >= -slack && b.2 <= pw + slack && b.3 <= ph + slack;
        let touches = b.2 > 0.0 && b.3 > 0.0 && b.0 < pw && b.1 < ph;
        let over_copies = node.repeat || inside_repeat;
        if !bleed && !over_copies && !touches {
            push("L10", Severity::Info, at("lint-l10-gone"));
        } else if !bleed && !over_copies && !inside {
            push("L10", Severity::Info, at("lint-l10-partly"));
        }

        // L15: where the constraints, and nothing the person set by hand on this device, put the
        // object badly (Q7): off the page there while it is on the primary one, newly on top of
        // something it does not touch on the primary, or a side squeezed under one device pixel.
        if on_secondary
            && !inside_repeat
            && !frame_overridden(doc, o, &target.id)
            && let Some(pn) = primary_nodes.get(&o.id)
            && !pn.hidden
        {
            let line = matches!(o.kind, ObjectKind::Line(_));
            let outer_of = |n: &Node| {
                if n.angle == 0.0 && !line {
                    n.frame
                } else {
                    let dir = match &o.kind {
                        ObjectKind::Line(l) => Some(l.direction),
                        _ => None,
                    };
                    n.frame.turned_bounds(n.angle, dir)
                }
            };
            let (p_box, t_box) = (outer_of(pn), outer_of(&node));
            let (pp, tp) = (primary.page_mm(), target.page_mm());
            let on_page = |f: &Frame, page: (f64, f64)| {
                f.x_mm >= -EPS_MM
                    && f.y_mm >= -EPS_MM
                    && f.right() <= page.0 + EPS_MM
                    && f.bottom() <= page.1 + EPS_MM
            };
            let short = |p: &Profile| {
                let mut c = p.short_name.chars();
                c.next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default()
            };
            let device = Arg::Text(short(target));
            let primary_name = Arg::Text(short(primary));
            let l15 = |id: &'static str| {
                at(id)
                    .arg("device", device.clone())
                    .arg("primary", primary_name.clone())
            };
            if on_page(&p_box, pp) && !on_page(&t_box, tp) {
                push("L15", Severity::Info, l15("lint-l15-off"));
            }
            // A side squeezed below one device pixel here that was a pixel or more on the primary
            // (a line has no thickness to lose).
            for (p_len, t_len, p_px, t_px, id) in [
                (
                    pn.frame.w_mm,
                    f.w_mm,
                    primary.mm_to_px(pn.frame.w_mm),
                    target.mm_to_px(f.w_mm),
                    "lint-l15-narrow",
                ),
                (
                    pn.frame.h_mm,
                    f.h_mm,
                    primary.mm_to_px(pn.frame.h_mm),
                    target.mm_to_px(f.h_mm),
                    "lint-l15-short",
                ),
            ] {
                // A box as tall as its text has the height its text gives it on each device; no
                // constraint squeezes that (rung 11b, paying the rung 8a debt).
                if id == "lint-l15-short" && o.keeps_height() {
                    continue;
                }
                if p_len > 0.0 && p_px >= 1.0 && t_len >= 0.0 && t_px < 1.0 {
                    push("L15", Severity::Info, l15(id).num("px", t_px, 2));
                }
            }
            // Newly on top of an object it does not touch on the primary. Each pair is said once,
            // about the lower of the two.
            for other in placed_all.iter().skip(index + 1) {
                let on = other.node;
                if on.group || on.hidden || on.framed || other.copy || in_repeat(other.id) {
                    continue;
                }
                let Some(q) = doc.object(other.id) else {
                    continue;
                };
                if frame_overridden(doc, q, &target.id) {
                    continue;
                }
                let Some(pq) = primary_nodes.get(&q.id).filter(|n| !n.hidden) else {
                    continue;
                };
                let q_line = match &q.kind {
                    ObjectKind::Line(l) => Some(l.direction),
                    _ => None,
                };
                let q_outer = |n: &Node| {
                    if n.angle == 0.0 && q_line.is_none() {
                        n.frame
                    } else {
                        n.frame.turned_bounds(n.angle, q_line)
                    }
                };
                let touch_primary = boxes_touch(&p_box, &q_outer(pq));
                if !touch_primary && overlaps(&t_box, &q_outer(&on)) {
                    push(
                        "L15",
                        Severity::Info,
                        l15("lint-l15-overlap").text("other", q.name.clone()),
                    );
                }
            }
        }

        // L06: safe zones, one finding per object for the fixed zones and one for the movable.
        // (A repeat and what is in its source are judged over all copies below.) An object marked Full bleed is
        // meant to run under them (ADR-019).
        let mut fixed = Vec::new();
        let mut movable = Vec::new();
        for z in target
            .safe_zones
            .iter()
            .filter(|_| !target.is_paper() && !bleed)
        {
            // A toolbar position the user does not use is skipped; a toolbar zone the linter
            // does not know is checked, so a new profile zone is never silently ignored.
            if let Ok(Some(t)) = Toolbar::of_zone(&z.name)
                && !opts.toolbars.contains(&t)
            {
                continue;
            }
            let (zx, zy, zw, zh) = (
                z.rect.0 as f64,
                z.rect.1 as f64,
                z.rect.2 as f64,
                z.rect.3 as f64,
            );
            // A turned shape meets a zone by its turned corners (rung 6b), not by the box round
            // them; everything else by its box, as before.
            let hit = if over_copies {
                false
            } else if turned {
                quadrule_layout::rotate::rotated_touches(
                    frame_px,
                    rotation,
                    if is_text { 0.0 } else { half_stroke },
                    (zx, zy, zx + zw, zy + zh),
                )
            } else {
                b.0 < zx + zw && b.2 > zx && b.1 < zy + zh && b.3 > zy
            };
            if hit {
                if z.advisory {
                    movable.push(zone_name(&z.name))
                } else {
                    fixed.push(zone_name(&z.name))
                }
            }
        }
        let zones = |items| Arg::List {
            items,
            sep: "lint-zone-sep",
        };
        if !fixed.is_empty() {
            push(
                "L06",
                Severity::Warn,
                at("lint-l06-fixed").arg("zones", zones(fixed)),
            );
        }
        if !movable.is_empty() {
            push(
                "L06",
                Severity::Info,
                at("lint-l06-movable").arg("zones", zones(movable)),
            );
        }

        // L16: position on the rhythm (checked where the design is made, on the primary).
        let free = free_placement(doc, o);
        if target.id == primary.id
            && !matches!(o.kind, ObjectKind::Pattern(_))
            && !inside_repeat
            && !free
        {
            let on = |pos: f64, origin: f64, unit: f64| {
                ((pos - origin).rem_euclid(unit)).min(unit - (pos - origin).rem_euclid(unit)) < 0.5
            };
            let (x, y) = (px(o.frame.x_mm).round(), px(o.frame.y_mm).round());
            // Flush on a page edge, near or far, counts as on the rhythm on that axis (ADR-019, item 3).
            let flush =
                |lo: f64, hi: f64, page: f64| lo.round() == 0.0 || hi.round() == page.round();
            // A right-aligned text box ends on the rhythm instead: its right edge counts (ADR-019, amendment of
            // 2026-11-23).
            let ends_on = matches!(&o.kind, ObjectKind::Text(t) if t.align == quadrule_model::TextAlign::Right)
                && on(
                    px(o.frame.x_mm + o.frame.w_mm).round(),
                    rhythm.origin_x,
                    rhythm.unit_x,
                );
            let off_x = !on(x, rhythm.origin_x, rhythm.unit_x)
                && !ends_on
                && !flush(frame_px.0, frame_px.2, pw);
            let off_y =
                !on(y, rhythm.origin_y, rhythm.unit_y) && !flush(frame_px.1, frame_px.3, ph);
            if off_x || off_y {
                let axes = match (off_x, off_y) {
                    (true, true) => "lint-axes-both",
                    (true, false) => "lint-axes-across",
                    _ => "lint-axes-down",
                };
                push(
                    "L16",
                    Severity::Warn,
                    at("lint-l16").arg("axes", Arg::Msg(Box::new(Message::new(axes)))),
                );
            }
        }

        // L17: a region unbound from the rhythm whose own spacing is not a clean multiple or
        // fraction of it, as this device resolves both (rung 9 bbq, Defaulted "L17"). It judges
        // the spacing the pattern uses; L16 leaves an unbound region's spacing to this rule.
        if let ObjectKind::Pattern(pr) = &o.kind
            && !pr.bound
        {
            let env = quadrule_layout::pattern::Env {
                target,
                rhythm: &doc.rhythm,
                rhythm_px: rhythm,
                labels: None,
            };
            let (sx, sy) = quadrule_layout::pattern::region_spacing_px(pr, &env);
            let (use_x, use_y) = pattern_axes(&pr.pattern);
            let clean = |spacing: f64, unit: f64| {
                (1..=10).any(|m| (spacing - f64::from(m) * unit).abs() <= 0.5 + 1e-9)
                    || (2..=10).any(|m| (spacing - unit / f64::from(m)).abs() <= 0.5 + 1e-9)
            };
            let bad_x = use_x && !clean(sx, rhythm.unit_x);
            let bad_y = use_y && !clean(sy, rhythm.unit_y);
            if bad_x || bad_y {
                let (axes, spacing, unit) = match (bad_x, bad_y) {
                    (true, true) => (
                        "lint-axes-both",
                        format!("{} × {}", trim(sx), trim(sy)),
                        format!("{} × {}", trim(rhythm.unit_x), trim(rhythm.unit_y)),
                    ),
                    (true, false) => ("lint-axes-across", trim(sx), trim(rhythm.unit_x)),
                    _ => ("lint-axes-down", trim(sy), trim(rhythm.unit_y)),
                };
                push(
                    "L17",
                    Severity::Info,
                    at("lint-l17")
                        .arg("axes", Arg::Msg(Box::new(Message::new(axes))))
                        .text("spacing", spacing)
                        .text("unit", unit),
                );
            }
        }

        // A repeat's pitch (rung 10 bbq, Defaulted "L16 and L17 learn repeats"). L16 (Warn, on the
        // primary): a pitch bound to the rhythm that is not a whole multiple of a step or
        // subdivision. L17 (Info, on each device): a pitch in mm that is not within half a pixel of
        // a whole multiple (1 to 10) or a whole fraction (a half to a tenth) of the rhythm step.
        if let (Some(r), Some(plan)) = (o.repeat(), node.plan) {
            let (ax, ay) = r.axes();
            let steps = doc.rhythm.steps_mm();
            let subs = f64::from(doc.rhythm.subdivisions_on().unwrap_or(1));
            let stride_mm = r.stride_mm(plan.size_mm, steps);
            let off = |used: bool, bound: bool, stride: f64, step: f64| {
                let k = stride / step * subs;
                used && bound && (k - k.round()).abs() > 1e-6
            };
            let (off_x, off_y) = (
                off(ax, r.h_spacing.is_bound(), stride_mm.0, steps.0),
                off(ay, r.v_spacing.is_bound(), stride_mm.1, steps.1),
            );
            let axes_of = |x: bool, y: bool| {
                Arg::Msg(Box::new(Message::new(match (x, y) {
                    (true, true) => "lint-axes-both",
                    (true, false) => "lint-axes-across",
                    _ => "lint-axes-down",
                })))
            };
            if target.id == primary.id && (off_x || off_y) && !free {
                push(
                    "L16",
                    Severity::Warn,
                    at("lint-l16-repeat-pitch").arg("axes", axes_of(off_x, off_y)),
                );
            }
            let clean = |spacing: f64, unit: f64| {
                (1..=10).any(|m| (spacing - f64::from(m) * unit).abs() <= 0.5 + 1e-9)
                    || (2..=10).any(|m| (spacing - unit / f64::from(m)).abs() <= 0.5 + 1e-9)
            };
            let bad_x = ax && !r.h_spacing.is_bound() && !clean(plan.stride.0, rhythm.unit_x);
            let bad_y = ay && !r.v_spacing.is_bound() && !clean(plan.stride.1, rhythm.unit_y);
            if bad_x || bad_y {
                let (spacing, unit) = match (bad_x, bad_y) {
                    (true, true) => (
                        format!("{} × {}", trim(plan.stride.0), trim(plan.stride.1)),
                        format!("{} × {}", trim(rhythm.unit_x), trim(rhythm.unit_y)),
                    ),
                    (true, false) => (trim(plan.stride.0), trim(rhythm.unit_x)),
                    _ => (trim(plan.stride.1), trim(rhythm.unit_y)),
                };
                push(
                    "L17",
                    Severity::Info,
                    at("lint-l17-repeat")
                        .arg("axes", axes_of(bad_x, bad_y))
                        .text("spacing", spacing)
                        .text("unit", unit),
                );
            }
        }
    }

    // L06 and L10 over every drawn copy of each repeat (rung 10 bbq, Defaulted "Other lint rules on
    // copies"): one finding per repeat and device, saying how many copies. The first cell's pieces
    // are moved to each cell; a turned shape is judged by its box here.
    for (i, rep) in placed_all.iter().enumerate() {
        let (Some(r), Some(plan)) = (doc.object(rep.id).and_then(|o| o.repeat()), rep.node.plan)
        else {
            continue;
        };
        if rep.node.hidden || rep.copy {
            continue;
        }
        let depth = rep.node.depth;
        let range = &placed_all[i + 1..];
        let range = &range[..range.iter().take_while(|q| q.node.depth > depth).count()];
        let cells: Vec<_> = plan.copies(r).filter(|c| !c.effect.skip).collect();
        let blocks = 1 + cells.iter().filter(|c| c.index > 0).count();
        if cells.is_empty() || blocks == 0 || !range.len().is_multiple_of(blocks) {
            continue;
        }
        let first = &range[..range.len() / blocks];
        // The pieces that are drawn: not the containers, not what an eye hides, not what is marked Full bleed
        // (ADR-019; a marked repeat covers every piece). Each with the half stroke L10 lets past an edge.
        let pieces: Vec<(Bounds, f64)> = first
            .iter()
            .filter(|q| !q.node.group && !doc.effectively_hidden(q.id))
            .filter_map(|q| {
                let o = doc.object(q.id)?;
                if full_bleed(doc, o) {
                    return None;
                }
                let text = o.kind.is_lettered();
                let half = quadrule_layout::stroke(o.style, target).0 as f64 / 2.0;
                Some((bounds(&q.shape, text, half), if text { 0.0 } else { half }))
            })
            .collect();
        if pieces.is_empty() {
            continue;
        }
        let total = cells.len();
        let (mut gone, mut partly) = (0usize, 0usize);
        let (mut fixed_n, mut movable_n) = (0usize, 0usize);
        let (mut fixed, mut movable): (Vec<Message>, Vec<Message>) = (Vec::new(), Vec::new());
        for c in &cells {
            let moved = |b: &Bounds| (b.0 + c.d.0, b.1 + c.d.1, b.2 + c.d.0, b.3 + c.d.1);
            let boxes: Vec<Bounds> = pieces.iter().map(|(b, _)| moved(b)).collect();
            let all_gone = boxes
                .iter()
                .all(|b| !(b.2 > 0.0 && b.3 > 0.0 && b.0 < pw && b.1 < ph));
            let all_in = boxes
                .iter()
                .zip(&pieces)
                .all(|(b, (_, h))| b.0 >= -h && b.1 >= -h && b.2 <= pw + h && b.3 <= ph + h);
            if all_gone {
                gone += 1;
            } else if !all_in {
                partly += 1;
            }
            let (mut cell_fixed, mut cell_movable) = (false, false);
            for z in target.safe_zones.iter().filter(|_| !target.is_paper()) {
                if let Ok(Some(t)) = Toolbar::of_zone(&z.name)
                    && !opts.toolbars.contains(&t)
                {
                    continue;
                }
                let (zx, zy, zw, zh) = (
                    z.rect.0 as f64,
                    z.rect.1 as f64,
                    z.rect.2 as f64,
                    z.rect.3 as f64,
                );
                if boxes
                    .iter()
                    .any(|b| b.0 < zx + zw && b.2 > zx && b.1 < zy + zh && b.3 > zy)
                {
                    let name = zone_name(&z.name);
                    let list = if z.advisory {
                        cell_movable = true;
                        &mut movable
                    } else {
                        cell_fixed = true;
                        &mut fixed
                    };
                    if !list.contains(&name) {
                        list.push(name);
                    }
                }
            }
            fixed_n += usize::from(cell_fixed);
            movable_n += usize::from(cell_movable);
        }
        let name = doc
            .object(rep.id)
            .map(|o| o.name.clone())
            .unwrap_or_default();
        let at = |id| {
            Message::new(id)
                .text("name", name.clone())
                .num("total", total as f64, 0)
        };
        let area = Some(bounds(&rep.shape, false, 0.0));
        if gone + partly > 0 {
            let (id, n) = if partly == 0 {
                ("lint-l10-repeat-gone", gone)
            } else {
                ("lint-l10-repeat-partly", gone + partly)
            };
            push(
                "L10",
                Severity::Info,
                Some(rep.id),
                area,
                at(id).num("count", n as f64, 0),
            );
        }
        let zones = |items| Arg::List {
            items,
            sep: "lint-zone-sep",
        };
        if fixed_n > 0 {
            push(
                "L06",
                Severity::Warn,
                Some(rep.id),
                area,
                at("lint-l06-repeat-fixed")
                    .num("count", fixed_n as f64, 0)
                    .arg("zones", zones(fixed)),
            );
        }
        if movable_n > 0 {
            push(
                "L06",
                Severity::Info,
                Some(rep.id),
                area,
                at("lint-l06-repeat-movable")
                    .num("count", movable_n as f64, 0)
                    .arg("zones", zones(movable)),
            );
        }
    }
    // L13: dark fills (gray 102 or darker on the Supernotes) cover more of the page than the profile allows.
    // Each fill's own laid-out area counts, every copy of a repeat as well, not the union of overlaps.
    if judged_for_template
        && let (Some(dark), Some(share)) =
            (target.lint.dark_fill_gray, target.lint.dark_fill_page_share)
    {
        let mut total = 0.0;
        let mut count = 0usize;
        let mut largest: Option<(f64, Uuid, Bounds)> = None;
        for p in &placed_all {
            let hidden = if p.copy {
                doc.effectively_hidden(p.id)
            } else {
                p.node.hidden
            };
            let Some(o) = doc.object(p.id) else { continue };
            if hidden || p.node.group || !o.kind.is_shape() {
                continue;
            }
            let Some(fill) = o.fill else { continue };
            let g = judge::gray_on_page(
                target.fill_gray(fill),
                judge::opacity_on_page(doc, o),
                doc.page.background_gray,
            );
            if g > dark {
                continue;
            }
            let area = judge::fill_area_px(o, &p.shape, (pw, ph));
            if area <= 0.0 {
                continue;
            }
            total += area;
            count += 1;
            if largest.is_none_or(|(a, _, _)| area > a) {
                largest = Some((area, o.id, bounds(&p.shape, false, 0.0)));
            }
        }
        let covered = total / (pw * ph);
        if let Some((_, id, b)) = largest
            && covered > share + 1e-9
        {
            push(
                "L13",
                Severity::Info,
                Some(id),
                Some(b),
                Message::new("lint-l13")
                    .num("count", count as f64, 0)
                    .num("percent", covered * 100.0, 0)
                    .num("limit", share * 100.0, 0)
                    .num("gray", f64::from(dark), 0),
            );
        }
    }

    // A paper size has no device data to judge against (napkin §3.3.1; rung 13b): the rules that
    // read safe zones, gray levels or pen grays do not run on it, whatever the profile file holds.
    if target.is_paper() {
        out.retain(|f| !SKIPPED_ON_PAPER.contains(&f.rule));
    }
    // The rules this page turns off, on every device (rung 14 bbq, Q7).
    if !doc.ignored_rules.is_empty() {
        out.retain(|f| !doc.ignores_rule(f.rule));
    }
    out
}

/// Whether the object, or a group round it, is marked Free placement: L16 leaves it (rung 14 bbq, Q7).
/// Marked Full bleed, itself or by a group or repeat it is in (ADR-019): L06 and L10 leave it.
pub(crate) fn full_bleed(doc: &Document, o: &quadrule_model::Object) -> bool {
    o.full_bleed
        || doc
            .ancestors(o.id)
            .iter()
            .filter_map(|a| doc.object(*a))
            .any(|a| a.full_bleed)
}

fn free_placement(doc: &Document, o: &quadrule_model::Object) -> bool {
    o.free_placement
        || doc
            .ancestors(o.id)
            .iter()
            .filter_map(|a| doc.object(*a))
            .any(|a| a.free_placement)
}

/// A rule's short name, as a message: "Gray too light" for `L03` (the Ignored rules list and the preflight sheet).
/// A rule this build does not judge gets its id as the name.
pub fn rule_title(rule: &str) -> Message {
    Message::new(match rule {
        "L01" => "lint-rule-l01",
        "L02" => "lint-rule-l02",
        "L03" => "lint-rule-l03",
        "L05" => "lint-rule-l05",
        "L06" => "lint-rule-l06",
        "L07" => "lint-rule-l07",
        "L08" => "lint-rule-l08",
        "L09" => "lint-rule-l09",
        "L10" => "lint-rule-l10",
        "L11" => "lint-rule-l11",
        "L13" => "lint-rule-l13",
        "L14" => "lint-rule-l14",
        "L15" => "lint-rule-l15",
        "L16" => "lint-rule-l16",
        "L17" => "lint-rule-l17",
        "L18" => "lint-rule-l18",
        "L19" => "lint-rule-l19",
        "L20" => "lint-rule-l20",
        _ => return Message::new("lint-rule-other").text("rule", rule),
    })
}

/// Why a check did not run on a device (rung 14a bbq, Q2; shown by the Lint tab and the preflight sheet).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipReason {
    /// A paper size has no screen to judge against (napkin §3.3.1).
    Paper,
    /// A Wallpaper is not handwritten on.
    Wallpaper,
    /// The document turned this rule off, on every device.
    Ignored,
    /// The profile lists no clean gray levels.
    NoGrayLevels,
    /// The profile has no value for the numbers this rule reads.
    NoValue,
}

/// One check that did not run on a device, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skipped {
    /// The rule, e.g. `L03`.
    pub rule: &'static str,
    /// Why it did not run.
    pub reason: SkipReason,
}

/// The rules that read the profile's numbers or its kind of page, in the order they are listed.
const JUDGED: [&str; 7] = ["L03", "L05", "L06", "L07", "L09", "L13", "L14"];

/// The checks that did not run on `target` for `doc`, with the reason for each (rung 14a bbq, Q2): a paper
/// size's missing device data, a Wallpaper, a rule the document ignores, no clean gray levels, or no value
/// for the rule in the profile. Rules the document ignores that this list does not judge are named too.
pub fn skipped_checks(doc: &Document, target: &Profile) -> Vec<Skipped> {
    let t = &target.lint;
    let mut out = Vec::new();
    let mut ids: Vec<&'static str> = JUDGED.to_vec();
    for r in &doc.ignored_rules {
        if let Some(known) = KNOWN_RULES.iter().find(|k| **k == r.as_str())
            && !ids.contains(known)
        {
            ids.push(known);
        }
    }
    for rule in ids {
        let has_value = match rule {
            "L03" => [
                t.lightest_gray_line_1px,
                t.lightest_gray_line_2px_up,
                t.lightest_gray_dot_2px,
                t.lightest_gray_dot_3px,
                t.lightest_gray_dot_4px_up,
                t.lightest_gray_fill,
                t.lightest_gray_text,
            ]
            .iter()
            .any(Option::is_some),
            "L07" => t.smallest_text_pt.is_some(),
            "L09" => t.smallest_handwriting_step_mm.is_some(),
            "L13" => t.dark_fill_gray.is_some() && t.dark_fill_page_share.is_some(),
            "L14" => t.thin_line_px.is_some(),
            _ => true,
        };
        let reason = if target.is_paper() && SKIPPED_ON_PAPER.contains(&rule) {
            Some(SkipReason::Paper)
        } else if !judge::template_only(doc) && matches!(rule, "L09" | "L13") {
            Some(SkipReason::Wallpaper)
        } else if doc.ignores_rule(rule) {
            Some(SkipReason::Ignored)
        } else if rule == "L05" && target.gray_levels.is_empty() {
            Some(SkipReason::NoGrayLevels)
        } else if !has_value {
            Some(SkipReason::NoValue)
        } else {
            None
        };
        if let Some(reason) = reason {
            out.push(Skipped { rule, reason });
        }
    }
    out
}

/// Every rule the linter has an id for that is built (L04 is retired by ADR-027; L12 waits for images).
const KNOWN_RULES: [&str; 18] = [
    "L01", "L02", "L03", "L05", "L06", "L07", "L08", "L09", "L10", "L11", "L13", "L14", "L15",
    "L16", "L17", "L18", "L19", "L20",
];

/// The rules that need a device's safe zones or clean gray levels, and so are skipped on a paper size
/// (napkin §3.3.1): L03 gray too light, L05 grays off the clean levels, L06 safe zones, L13 dark fills
/// and L14 soft thin lines (the last two since rung 14a; a paper size prints at a PPI the page picks).
/// L07 and L09 run on paper as on a device, like the others. L04 left the list when ADR-027 retired it.
pub const SKIPPED_ON_PAPER: [&str; 5] = ["L03", "L05", "L06", "L13", "L14"];

/// Which of a pattern's two spacings it uses: (across, down). Rows alone for ruled lines, an
/// isometric grid and a staff; columns alone for pointy hexagons.
fn pattern_axes(p: &quadrule_model::Pattern) -> (bool, bool) {
    use quadrule_model::{HexOrientation, Pattern};
    match p {
        Pattern::Lines | Pattern::Isometric { .. } | Pattern::MusicStaff { .. } => (false, true),
        // A calendar's day square is one step across, its header row one step down.
        Pattern::Dots | Pattern::Grid | Pattern::Cornell { .. } | Pattern::Calendar { .. } => {
            (true, true)
        }
        Pattern::Hex { orientation, .. } => match orientation {
            HexOrientation::Flat => (false, true),
            HexOrientation::Pointy => (true, false),
        },
    }
}

/// A pixel count to two places, without trailing zeros.
fn trim(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Boxes closer than this many mm are touching (float noise, not a gap).
const EPS_MM: f64 = 1e-6;

/// Whether the object, or a group round it, has a place or size override on `device`: it is then
/// where the person put it there, and L15 leaves it alone.
fn frame_overridden(doc: &Document, o: &quadrule_model::Object, device: &str) -> bool {
    let has = |x: &quadrule_model::Object| {
        x.override_on(device)
            .is_some_and(quadrule_model::Override::has_frame)
    };
    has(o)
        || doc
            .ancestors(o.id)
            .iter()
            .filter_map(|a| doc.object(*a))
            .any(has)
}

/// Whether two boxes meet: overlap, or share an edge or a corner.
fn boxes_touch(a: &Frame, b: &Frame) -> bool {
    a.x_mm <= b.right() + EPS_MM
        && b.x_mm <= a.right() + EPS_MM
        && a.y_mm <= b.bottom() + EPS_MM
        && b.y_mm <= a.bottom() + EPS_MM
}

/// Whether two boxes share some area, not only an edge.
fn overlaps(a: &Frame, b: &Frame) -> bool {
    a.x_mm < b.right() - EPS_MM
        && b.x_mm < a.right() - EPS_MM
        && a.y_mm < b.bottom() - EPS_MM
        && b.y_mm < a.bottom() - EPS_MM
}

/// A safe zone's name, as a message. A zone the strings do not name yet shows its profile name.
fn zone_name(name: &str) -> Message {
    match name {
        "footer_left" => Message::new("lint-zone-footer_left"),
        "footer_right" => Message::new("lint-zone-footer_right"),
        "toolbar_left" => Message::new("lint-zone-toolbar_left"),
        "toolbar_right" => Message::new("lint-zone-toolbar_right"),
        "toolbar_top" => Message::new("lint-zone-toolbar_top"),
        "toolbar_floating_default" => Message::new("lint-zone-toolbar_floating_default"),
        n => Message::new("lint-zone-other").text("zone", n.replace('_', " ")),
    }
}

/// An object's box in pixels, from its placed shape: including half its stroke, except a text
/// box, which has none.
fn bounds(shape: &Shape, text: bool, half_stroke: f64) -> Bounds {
    let (x0, y0, x1, y1) = shape.bounds();
    // A line's bounds already hold half its stroke.
    if text || matches!(shape, Shape::Segment { .. }) {
        (x0, y0, x1, y1)
    } else {
        (
            x0 - half_stroke,
            y0 - half_stroke,
            x1 + half_stroke,
            y1 + half_stroke,
        )
    }
}
