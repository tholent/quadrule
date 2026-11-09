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
//! L10 off the page, L16 position off the rhythm, L18 rhythm fits the margins unevenly, L19 uneven
//! physical-locked spacing. The gray, ink, text-size and handwriting rules (L03, L04, L05, L07,
//! L09, L13, L14) wait for calibrated profile values; L08, L11, L12, L15 and L17 wait for the
//! features they check.

#![forbid(unsafe_code)]

use quadrule_i18n::{Arg, Message};
use quadrule_layout::{RhythmPx, resolve_frame};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{Document, Object, ObjectKind, RhythmResolution, Style, Uuid};
use quadrule_profiles::Profile;

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
    /// Every position.
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
    /// Toolbar positions the user uses; L06 ignores the others. Default: all of them.
    pub toolbars: Vec<Toolbar>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            toolbars: Toolbar::ALL.to_vec(),
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
    let (from, to) = (primary.page_mm(), target.page_mm());
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

    for o in doc
        .layers
        .iter()
        .filter(|l| l.visible)
        .flat_map(|l| l.objects.iter())
        .filter(|o| o.visible)
    {
        let f = resolve_frame(o.frame, o.constraints, from, to);
        let px = |mm: f64| target.mm_to_px(mm);
        let b = bounds(o, f, target, outlines);
        let at = |id| Message::new(id).text("name", o.name.clone());
        let mut push = |rule, severity, message| push(rule, severity, Some(o.id), Some(b), message);

        // L01 and L02: strokes.
        let stroked = !matches!(o.kind, ObjectKind::Text(_));
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
            } else if (exact - exact.round()).abs() > 1e-6 {
                push(
                    "L02",
                    Severity::Info,
                    at("lint-l02-fraction")
                        .num("px", exact, 2)
                        .num("drawn", exact.round(), 0),
                );
            }
        }
        if matches!(o.kind, ObjectKind::Line(_)) && f.w_mm != 0.0 && f.h_mm != 0.0 {
            push("L02", Severity::Info, at("lint-l02-diagonal"));
        }

        // L10: off the page.
        let inside = b.0 >= 0.0 && b.1 >= 0.0 && b.2 <= pw && b.3 <= ph;
        let touches = b.2 > 0.0 && b.3 > 0.0 && b.0 < pw && b.1 < ph;
        if !touches {
            push("L10", Severity::Info, at("lint-l10-gone"));
        } else if !inside {
            push("L10", Severity::Info, at("lint-l10-partly"));
        }

        // L06: safe zones, one finding per object for the fixed zones and one for the movable.
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
            if b.0 < zx + zw && b.2 > zx && b.1 < zy + zh && b.3 > zy {
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
        if target.id == primary.id && !matches!(o.kind, ObjectKind::Pattern(_)) {
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
    }
    out
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

/// An object's box on `target`, in pixels, including half its stroke.
fn bounds(
    o: &Object,
    f: quadrule_model::Frame,
    target: &Profile,
    outlines: &ProfileOutlines,
) -> Bounds {
    let px = |mm: f64| target.mm_to_px(mm);
    let (x0, y0, x1, y1) = (
        px(f.x_mm),
        px(f.y_mm),
        px(f.x_mm + f.w_mm),
        px(f.y_mm + f.h_mm),
    );
    match &o.kind {
        ObjectKind::Text(_) => {
            let h = outlines
                .text(o.id)
                .map(|t| px(t.height_mm as f64))
                .unwrap_or(y1 - y0);
            (x0, y0, x1, y0 + h)
        }
        _ => {
            let half = quadrule_layout::stroke(o.style, target).0 as f64 / 2.0;
            (
                x0.min(x1) - half,
                y0.min(y1) - half,
                x0.max(x1) + half,
                y0.max(y1) + half,
            )
        }
    }
}
