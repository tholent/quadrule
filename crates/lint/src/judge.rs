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

//! The gray, text-size, handwriting, dark-fill and thin-line rules (rung 14a bbq, Q3 and Q4): L03, L05,
//! L07, L09, L13 and L14. Each reads the numbers in the profile in use (`Profile::lint`); a number the
//! profile leaves empty skips the rule. The gray judged is the gray an object puts on the page: its
//! gray blended by its own opacity and its groups' over the page background (Defaulted "The gray a
//! rule judges"). Roles are judged like any other style.

use crate::Severity;
use quadrule_i18n::{Arg, Message};
use quadrule_layout::stroke_opt;
use quadrule_model::{Document, DocumentType, IsoStyle, Object, ObjectKind, Pattern};
use quadrule_profiles::Profile;

/// One finding about one object, before it is given its place on the page.
pub(crate) struct Hit {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: Message,
}

fn hit(rule: &'static str, severity: Severity, message: Message) -> Hit {
    Hit {
        rule,
        severity,
        message,
    }
}

/// How opaque an object is on the page, 0 to 1: its own opacity times that of every group round it.
pub(crate) fn opacity_on_page(doc: &Document, o: &Object) -> f64 {
    let mut a = f64::from(o.opacity()) / 100.0;
    for id in doc.ancestors(o.id) {
        if let Some(g) = doc.object(id) {
            a *= f64::from(g.opacity()) / 100.0;
        }
    }
    a
}

/// `gray` as it lands on the page: blended by `alpha` over the page background, to a whole gray.
pub(crate) fn gray_on_page(gray: u8, alpha: f64, background: u8) -> u8 {
    let g = f64::from(gray) * alpha + f64::from(background) * (1.0 - alpha);
    g.round().clamp(0.0, 255.0) as u8
}

fn num(m: Message, name: &'static str, v: f64) -> Message {
    m.num(name, v, 0)
}

/// What a gray belongs to, as L05 names it.
fn what(id: &'static str) -> Arg {
    Arg::Msg(Box::new(Message::new(id)))
}

/// Whether the object's own ruling or outline is slanted or curved, so that most of a thin stroke's pixels
/// come out as in-between grays (L14). `diagonal` is true for a line whose two ends differ in x and y.
fn soft_stroke(o: &Object, rotation: f64, diagonal: bool) -> bool {
    use quadrule_layout::rotate::is_quarter_turn;
    match &o.kind {
        ObjectKind::Line(_) => diagonal,
        ObjectKind::Rect(_) => !is_quarter_turn(rotation),
        ObjectKind::Ellipse(_) => true,
        ObjectKind::Polygon(_) => {
            !is_quarter_turn(rotation)
                || !quadrule_layout::shapes::has_only_axis_aligned_edges(&o.kind)
        }
        ObjectKind::Pattern(pr) => match &pr.pattern {
            Pattern::Dots
            | Pattern::Isometric {
                style: IsoStyle::Dots,
            } => false,
            Pattern::Isometric { .. } | Pattern::Hex { .. } => true,
            _ => !(is_quarter_turn(rotation) && is_quarter_turn(pr.pattern_angle())),
        },
        _ => false,
    }
}

/// L03, L05, L07 and L14 for one object on `target`. `size_pt` is the size a text box is drawn at on this
/// device; `rotation` the object's turn on the page; `diagonal` whether a line runs slanted.
pub(crate) fn object_hits(
    doc: &Document,
    o: &Object,
    target: &Profile,
    size_pt: Option<f64>,
    rotation: f64,
    diagonal: bool,
) -> Vec<Hit> {
    let mut out = Vec::new();
    if o.is_container() {
        return out;
    }
    let t = &target.lint;
    let alpha = opacity_on_page(doc, o);
    let on_page = |g: u8| gray_on_page(g, alpha, doc.page.background_gray);
    let levels = &target.gray_levels;
    let at = |id| Message::new(id).text("name", o.name.clone());
    let off_levels = |g: u8| !levels.is_empty() && !levels.contains(&g);

    let stroke = stroke_opt(o.style, target);
    let is_text = matches!(o.kind, ObjectKind::Text(_));
    let dots = match &o.kind {
        ObjectKind::Pattern(pr) => matches!(
            pr.pattern,
            Pattern::Dots
                | Pattern::Isometric {
                    style: IsoStyle::Dots
                }
        ),
        _ => false,
    };

    // The stroke (or, for text, the letters).
    if let Some((w, gray)) = stroke {
        let g = on_page(gray);
        if is_text {
            if let Some(lightest) = t.lightest_gray_text
                && g > lightest
            {
                out.push(hit(
                    "L03",
                    Severity::Warn,
                    num(
                        num(at("lint-l03-text"), "gray", f64::from(g)),
                        "lightest",
                        f64::from(lightest),
                    ),
                ));
            }
            if off_levels(g) {
                out.push(hit(
                    "L05",
                    Severity::Info,
                    num(at("lint-l05"), "gray", f64::from(g)).arg("what", what("lint-l05-text")),
                ));
            }
        } else {
            let limit = if dots {
                t.lightest_gray_dot(w)
            } else {
                t.lightest_gray_line(w)
            };
            if let Some(lightest) = limit
                && g > lightest
            {
                let m = if dots {
                    num(at("lint-l03-dots"), "px", f64::from(w))
                } else {
                    at("lint-l03-stroke")
                };
                out.push(hit(
                    "L03",
                    Severity::Warn,
                    num(
                        num(m, "gray", f64::from(g)),
                        "lightest",
                        f64::from(lightest),
                    ),
                ));
            }
            if off_levels(g) {
                out.push(hit(
                    "L05",
                    Severity::Info,
                    num(at("lint-l05"), "gray", f64::from(g)).arg(
                        "what",
                        what(if dots {
                            "lint-l05-dots"
                        } else {
                            "lint-l05-line"
                        }),
                    ),
                ));
            }
            // L14: a thin stroke that is slanted or curved.
            if let Some(thin) = t.thin_line_px
                && w <= thin
                && !dots
                && soft_stroke(o, rotation, diagonal)
            {
                out.push(hit(
                    "L14",
                    Severity::Info,
                    num(at("lint-l14"), "px", f64::from(w)),
                ));
            }
        }
    }

    // The border of a pattern region is a line of its own, judged as any line is (pattern borders bbq,
    // Defaulted "Snapping and lint").
    if let ObjectKind::Pattern(pr) = &o.kind
        && let Some(b) = &pr.border
        && b.sides.any()
        && let Some((w, gray)) = stroke_opt(b.style, target)
    {
        let g = on_page(gray);
        if let Some(lightest) = t.lightest_gray_line(w)
            && g > lightest
        {
            out.push(hit(
                "L03",
                Severity::Warn,
                num(
                    num(at("lint-l03-border"), "gray", f64::from(g)),
                    "lightest",
                    f64::from(lightest),
                ),
            ));
        }
        if off_levels(g) {
            out.push(hit(
                "L05",
                Severity::Info,
                num(at("lint-l05"), "gray", f64::from(g)).arg("what", what("lint-l05-border")),
            ));
        }
        // Curved at a rounded corner, or slanted when the frame is turned off a quarter turn.
        let curved = b.radius_mm() > 0.0
            && (b.sides.top || b.sides.bottom)
            && (b.sides.left || b.sides.right);
        if let Some(thin) = t.thin_line_px
            && w <= thin
            && (curved || !quadrule_layout::rotate::is_quarter_turn(rotation))
        {
            out.push(hit(
                "L14",
                Severity::Info,
                num(at("lint-l14-border"), "px", f64::from(w)),
            ));
        }
    }

    // The fill of a shape.
    if let Some(f) = o.fill
        && o.kind.is_shape()
    {
        let g = on_page(target.fill_gray(f));
        if let Some(lightest) = t.lightest_gray_fill
            && g > lightest
        {
            out.push(hit(
                "L03",
                Severity::Warn,
                num(
                    num(at("lint-l03-fill"), "gray", f64::from(g)),
                    "lightest",
                    f64::from(lightest),
                ),
            ));
        }
        if off_levels(g) {
            out.push(hit(
                "L05",
                Severity::Info,
                num(at("lint-l05"), "gray", f64::from(g)).arg("what", what("lint-l05-fill")),
            ));
        }
    }

    // L07: text smaller than the profile's smallest legible size.
    if let (Some(pt), Some(min)) = (size_pt, t.smallest_text_pt)
        && pt < min - 1e-9
    {
        out.push(hit(
            "L07",
            Severity::Warn,
            at("lint-l07").num("pt", pt, 1).num("min", min, 1),
        ));
    }
    out
}

/// The least a handwriting step may be on `target`, in device pixels: the profile's millimetres as a
/// whole number of pixels, as the rhythm is drawn.
pub(crate) fn handwriting_min_px(target: &Profile) -> Option<(f64, f64)> {
    let mm = target.lint.smallest_handwriting_step_mm?;
    Some((mm, target.mm_to_px(mm).round().max(1.0)))
}

/// Whether L09 and L13 run on this document at all: they are for a Template (Defaulted "Template only").
pub(crate) fn template_only(doc: &Document) -> bool {
    doc.document_type == DocumentType::Template
}

/// A shape's fill area in device pixels, clipped to the page (`page` is its width and height in pixels):
/// the box of a rectangle, a quarter turn short for an ellipse, the polygon's own share of its box.
pub(crate) fn fill_area_px(o: &Object, shape: &quadrule_layout::Shape, page: (f64, f64)) -> f64 {
    let (x0, y0, x1, y1) = shape.bounds();
    let full = (x1 - x0).max(0.0) * (y1 - y0).max(0.0);
    if full <= 0.0 {
        return 0.0;
    }
    let (cx0, cy0, cx1, cy1) = (x0.max(0.0), y0.max(0.0), x1.min(page.0), y1.min(page.1));
    let visible = (cx1 - cx0).max(0.0) * (cy1 - cy0).max(0.0);
    // A turned shape's bounds are the box round it, so its own area comes from its unturned size.
    let own = match shape {
        quadrule_layout::Shape::Rotated { hw, hh, .. } => 4.0 * hw * hh,
        _ => full,
    };
    let share = match &o.kind {
        ObjectKind::Ellipse(_) => std::f64::consts::FRAC_PI_4,
        ObjectKind::Polygon(pg) => {
            let pts = quadrule_layout::shapes::polygon_unit_points(pg);
            let n = pts.len();
            (0..n)
                .map(|i| {
                    let (a, b) = (pts[i], pts[(i + 1) % n]);
                    a.0 * b.1 - b.0 * a.1
                })
                .sum::<f64>()
                .abs()
                / 2.0
        }
        _ => 1.0,
    };
    own * share * (visible / full)
}
