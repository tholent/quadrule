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
//! Built so far, the rules that need no calibration values:
//! L01 stroke thinner than 1 px, L02 stroke not pixel-aligned, L06 content in a safe zone,
//! L10 off the page, L15 constraints cause trouble on a secondary device (rung 8a), L16 position
//! off the rhythm, L18 rhythm fits the margins unevenly, L19 uneven physical-locked spacing. The gray, ink, text-size and handwriting rules (L03, L04, L05, L07,
//! L09, L13, L14) wait for calibrated profile values; L08, L11 and L12 wait for the
//! features they check. Rung 10: a repeat's source is judged once, L06 and L10 judge every drawn
//! copy (one finding per repeat and device, with the count), and L16 and L17 judge its pitch.

#![forbid(unsafe_code)]

use quadrule_i18n::{Arg, Message};
use quadrule_layout::tree::{Node, resolve_nodes};
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
}

impl Default for Options {
    fn default() -> Self {
        Options {
            toolbars: Toolbar::DOCKED.to_vec(),
        }
    }
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
        let (a, b) = (
            (origin - start).rem_euclid(unit),
            (end - origin).rem_euclid(unit),
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

    // Every leaf object at its resolved, turned place (rung 7a bbq, Defaulted "Lint"): a group has
    // no rules of its own, and what it holds is judged where it falls on the page. The places come
    // from layout, the one copy of the maths.
    // L15 compares with the primary page (rung 8a, Q7): the stored design, laid out against itself.
    let on_secondary = target.id != primary.id;
    let primary_nodes: HashMap<Uuid, Node> = if on_secondary {
        resolve_nodes(doc, primary.page_mm(), primary.page_mm())
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
        let b = bounds(
            &placed.shape,
            matches!(o.kind, ObjectKind::Text(_)),
            half_stroke,
        );
        let frame_px = (
            px(f.x_mm).min(px(f.x_mm + f.w_mm)),
            px(f.y_mm).min(px(f.y_mm + f.h_mm)),
            px(f.x_mm).max(px(f.x_mm + f.w_mm)),
            px(f.y_mm).max(px(f.y_mm + f.h_mm)),
        );
        let turned = rotation != 0.0 && o.kind.is_shape();
        let at = |id| Message::new(id).text("name", o.name.clone());
        let mut push = |rule, severity, message| push(rule, severity, Some(o.id), Some(b), message);

        // L01 and L02: strokes. A shape reads its own stroke the same way a line does (rung 6 bbq,
        // Q8). L02 asks only about straight, unrotated edges: a curve or a slanted edge is soft by
        // design (Q2), so a stroke width that is not whole pixels does not matter there.
        let stroked = !matches!(o.kind, ObjectKind::Text(_));
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
        // A line is diagonal when its two ends, turned as the line is, differ in both x and y.
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
            if (x1 - x0).abs() > 1e-6 && (y1 - y0).abs() > 1e-6 {
                push("L02", Severity::Info, at("lint-l02-diagonal"));
            }
        }

        // L10: off the page. A repeat and what is in its source are judged over all copies below.
        let inside = b.0 >= 0.0 && b.1 >= 0.0 && b.2 <= pw && b.3 <= ph;
        let touches = b.2 > 0.0 && b.3 > 0.0 && b.0 < pw && b.1 < ph;
        let over_copies = node.repeat || inside_repeat;
        if !over_copies && !touches {
            push("L10", Severity::Info, at("lint-l10-gone"));
        } else if !over_copies && !inside {
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
        // (A repeat and what is in its source are judged over all copies below.)
        let mut fixed = Vec::new();
        let mut movable = Vec::new();
        for z in &target.safe_zones {
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
                    half_stroke,
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
        if target.id == primary.id && !matches!(o.kind, ObjectKind::Pattern(_)) && !inside_repeat {
            let on = |pos: f64, origin: f64, unit: f64| {
                ((pos - origin).rem_euclid(unit)).min(unit - (pos - origin).rem_euclid(unit)) < 0.5
            };
            let (x, y) = (px(o.frame.x_mm).round(), px(o.frame.y_mm).round());
            let off_x = !on(x, rhythm.origin_x, rhythm.unit_x);
            let off_y = !on(y, rhythm.origin_y, rhythm.unit_y);
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
            if target.id == primary.id && (off_x || off_y) {
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
        // The pieces that are drawn: not the containers, not what an eye hides.
        let pieces: Vec<Bounds> = first
            .iter()
            .filter(|q| !q.node.group && !doc.effectively_hidden(q.id))
            .filter_map(|q| {
                let o = doc.object(q.id)?;
                let half = quadrule_layout::stroke(o.style, target).0 as f64 / 2.0;
                Some(bounds(
                    &q.shape,
                    matches!(o.kind, ObjectKind::Text(_)),
                    half,
                ))
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
            let boxes: Vec<Bounds> = pieces.iter().map(moved).collect();
            let all_gone = boxes
                .iter()
                .all(|b| !(b.2 > 0.0 && b.3 > 0.0 && b.0 < pw && b.1 < ph));
            let all_in = boxes
                .iter()
                .all(|b| b.0 >= 0.0 && b.1 >= 0.0 && b.2 <= pw && b.3 <= ph);
            if all_gone {
                gone += 1;
            } else if !all_in {
                partly += 1;
            }
            let (mut cell_fixed, mut cell_movable) = (false, false);
            for z in &target.safe_zones {
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
    out
}

/// Which of a pattern's two spacings it uses: (across, down). Rows alone for ruled lines, an
/// isometric grid and a staff; columns alone for pointy hexagons.
fn pattern_axes(p: &quadrule_model::Pattern) -> (bool, bool) {
    use quadrule_model::{HexOrientation, Pattern};
    match p {
        Pattern::Lines | Pattern::Isometric { .. } | Pattern::MusicStaff { .. } => (false, true),
        Pattern::Dots | Pattern::Grid | Pattern::Cornell { .. } => (true, true),
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
