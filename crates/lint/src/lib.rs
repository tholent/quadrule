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
    /// What is wrong and what to do, in plain words.
    pub message: String,
}

/// An object's box in device pixels: (x0, y0, x1, y1).
type Bounds = (f64, f64, f64, f64);

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
    let mut push = |rule, severity, object, message: String| {
        out.push(Finding {
            rule,
            severity,
            profile: target.id.clone(),
            object,
            message,
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
            format!(
                "The rhythm keeps exact millimetres, so on this device its lines are {:.2} × {:.2} px apart and some gaps are 1 px wider than others. Pixel-locked rhythm keeps every gap the same.",
                rhythm.unit_x, rhythm.unit_y
            ),
        );
    }
    // L18: the rhythm fits unevenly when the strip it leaves at one side of the margin box is
    // more than half a unit wider than the strip at the other side.
    let m = &doc.page.margins_mm;
    let edge = |mm: f64| target.mm_to_px(mm).round();
    let axes = [
        (
            "left",
            "right",
            rhythm.origin_x,
            rhythm.unit_x,
            edge(m.left),
            pw - edge(m.right),
        ),
        (
            "top",
            "bottom",
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
                format!(
                    "The rhythm leaves {a:.0} px at the {near} margin and {b:.0} px at the {far}: more than half a unit ({unit:.0} px) apart. Centring the rhythm origin splits the space evenly."
                ),
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
        let name = &o.name;

        // L01 and L02: strokes.
        let stroked = !matches!(o.kind, ObjectKind::Text(_));
        if stroked && let Style::Custom { width_mm, .. } = o.style {
            let exact = px(width_mm);
            if exact < 1.0 {
                push(
                    "L01",
                    Severity::Warn,
                    Some(o.id),
                    format!(
                        "\"{name}\" has a {exact:.2} px stroke on this device, thinner than one pixel: it is drawn 1 px wide and may look heavier than meant. Use a role, or at least {:.3} mm.",
                        target.mm_to_px(1.0).recip()
                    ),
                );
            } else if (exact - exact.round()).abs() > 1e-6 {
                push(
                    "L02",
                    Severity::Info,
                    Some(o.id),
                    format!(
                        "\"{name}\" has a {exact:.2} px stroke on this device; it is drawn {:.0} px wide. A role, or a width of whole pixels, keeps it exact.",
                        exact.round()
                    ),
                );
            }
        }
        if matches!(o.kind, ObjectKind::Line(_)) && f.w_mm != 0.0 && f.h_mm != 0.0 {
            push(
                "L02",
                Severity::Info,
                Some(o.id),
                format!(
                    "\"{name}\" is a diagonal line, so its edges cannot sit on whole pixels and are soft."
                ),
            );
        }

        // L10: off the page.
        let b = bounds(o, f, target, outlines);
        let inside = b.0 >= 0.0 && b.1 >= 0.0 && b.2 <= pw && b.3 <= ph;
        let touches = b.2 > 0.0 && b.3 > 0.0 && b.0 < pw && b.1 < ph;
        if !touches {
            push(
                "L10",
                Severity::Info,
                Some(o.id),
                format!("\"{name}\" is entirely off the page on this device."),
            );
        } else if !inside {
            push(
                "L10",
                Severity::Info,
                Some(o.id),
                format!("\"{name}\" runs off the edge of the page on this device."),
            );
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
        if !fixed.is_empty() {
            push(
                "L06",
                Severity::Warn,
                Some(o.id),
                format!(
                    "\"{name}\" is under the {} on this device. Move it inside the safe area, or widen the margins.",
                    fixed.join(", the ")
                ),
            );
        }
        if !movable.is_empty() {
            push(
                "L06",
                Severity::Info,
                Some(o.id),
                format!(
                    "\"{name}\" can be covered by the {} where it first appears; it can be moved.",
                    movable.join(", the ")
                ),
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
                    (true, true) => "across and down",
                    (true, false) => "across",
                    _ => "down",
                };
                push(
                    "L16",
                    Severity::Warn,
                    Some(o.id),
                    format!(
                        "\"{name}\" starts off the rhythm ({axes}). Snap it to the rhythm so it lines up with the grid."
                    ),
                );
            }
        }
    }
    out
}

/// A safe zone's plain name.
fn zone_name(name: &str) -> String {
    match name {
        "footer_left" => "page-number footer".into(),
        "footer_right" => "note-name footer".into(),
        "toolbar_floating_default" => "floating toolbar".into(),
        n => match n.strip_prefix("toolbar_") {
            Some(side) => format!("toolbar when docked {side}"),
            None => n.replace('_', " "),
        },
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
