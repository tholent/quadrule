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

//! Lint checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;
use std::collections::BTreeSet;

/// A finding's message in English; fails the test if a message id or argument is missing.
fn text(f: &Finding) -> String {
    quadrule_i18n::Strings::english()
        .try_format(&f.message)
        .unwrap()
}

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn run(doc: &Document, target: &Profile) -> Vec<Finding> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let laid = lay_out_text(doc, &primary, target, &mut FontRegistry::bundled_only()).unwrap();
    lint(doc, &primary, target, &laid.outlines)
}

fn rules(f: &[Finding], object: Option<Uuid>) -> BTreeSet<&'static str> {
    f.iter()
        .filter(|x| object.is_none() || x.object == object)
        .map(|x| x.rule)
        .collect()
}

fn line(id: u128, frame: Frame, style: Style) -> Object {
    Object {
        id: Uuid::from_u128(id),
        name: format!("line {id}"),
        frame,
        constraints: Constraints::default(),
        visible: true,
        locked: false,
        style,
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        kind: ObjectKind::Line(Line::default()),
        extra: Extra::new(),
    }
}

#[test]
fn slice_page_findings() {
    // The slice's 8 mm margin is narrower than a docked toolbar, so its header sits under the
    // left, right and top toolbars (L06); the title rule at 22 mm is off the 6 mm rhythm (L16).
    let doc = slice_document();
    let manta = builtin(MANTA_ID).unwrap();
    let f = run(&doc, &manta);
    let title = rules(&f, Some(ids::TITLE));
    let rule = rules(&f, Some(ids::RULE));
    let zones: BTreeSet<String> = f
        .iter()
        .filter(|x| x.object == Some(ids::TITLE) && x.rule == "L06")
        .map(text)
        .collect();
    let ok = title == BTreeSet::from(["L06"])
        && rule.contains("L16")
        && rule.contains("L06")
        && f.iter().all(|x| x.profile == MANTA_ID)
        && zones
            .iter()
            .any(|m| m.contains("docked left") && m.contains("docked top"));
    println!(
        "CHECK crate=lint name=slice_findings result={} findings={} title={title:?} rule={rule:?}",
        pass(ok),
        f.len()
    );
    assert!(ok);
}

#[test]
fn stroke_rules() {
    // L01 below a pixel, L02 for a width that rounds or a diagonal; a role stroke is clean.
    let mut doc = slice_document();
    let thin = line(
        10,
        Frame::new(20.0, 30.0, 50.0, 0.0),
        Style::Custom {
            width_mm: 0.05,
            gray: 0,
        },
    );
    let fractional = line(
        11,
        Frame::new(20.0, 36.0, 50.0, 0.0),
        Style::Custom {
            width_mm: 0.12,
            gray: 0,
        },
    );
    let diagonal = line(
        12,
        Frame::new(20.0, 42.0, 50.0, 10.0),
        Style::Role(LineRole::Guide),
    );
    let clean = line(
        13,
        Frame::new(20.0, 56.0, 50.0, 0.0),
        Style::Role(LineRole::Guide),
    );
    doc.layers[0].objects = vec![thin, fractional, diagonal, clean];
    let f = run(&doc, &builtin(MANTA_ID).unwrap());
    let r = |id| rules(&f, Some(Uuid::from_u128(id)));
    let ok = r(10).contains("L01")
        && r(11).contains("L02")
        && r(12).contains("L02")
        && !r(13).contains("L01")
        && !r(13).contains("L02");
    println!(
        "CHECK crate=lint name=stroke_rules result={} thin={:?} fractional={:?} diagonal={:?} clean={:?}",
        pass(ok),
        r(10),
        r(11),
        r(12),
        r(13)
    );
    assert!(ok);
}

#[test]
fn off_page_and_rhythm_rules() {
    let mut doc = slice_document();
    let partly = line(
        20,
        Frame::new(150.0, 50.0, 30.0, 0.0),
        Style::Role(LineRole::Guide),
    );
    let gone = line(
        21,
        Frame::new(300.0, 50.0, 10.0, 0.0),
        Style::Role(LineRole::Guide),
    );
    doc.layers[0].objects = vec![partly, gone];
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    let f = run(&doc, &builtin(MANTA_ID).unwrap());
    let off: Vec<String> = f.iter().filter(|x| x.rule == "L10").map(text).collect();
    let ok = off.len() == 2
        && off.iter().any(|m| m.contains("runs off"))
        && off.iter().any(|m| m.contains("entirely off"))
        && rules(&f, None).contains("L19");
    println!(
        "CHECK crate=lint name=off_page_and_physical result={} l10={} l19={}",
        pass(ok),
        off.len(),
        rules(&f, None).contains("L19")
    );
    assert!(ok);
}

#[test]
fn wide_strip_and_centred_rhythm() {
    // A 7 mm rhythm from an 8 mm margin leaves a wide strip at the right edge of the Manta (L18);
    // centring the rhythm splits it evenly and the finding goes away.
    let mut doc = slice_document();
    doc.layers.clear();
    doc.rhythm.h_unit_mm = 7.0;
    doc.rhythm.v_unit_mm = 7.0;
    let manta = builtin(MANTA_ID).unwrap();
    let before = run(&doc, &manta).iter().filter(|x| x.rule == "L18").count();
    doc.rhythm.origin = RhythmOrigin::Centered;
    let after = run(&doc, &manta).iter().filter(|x| x.rule == "L18").count();
    let ok = before > 0 && after == 0;
    println!(
        "CHECK crate=lint name=wide_strip_l18 result={} margin_corner={before} centred={after}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn floating_toolbar_is_not_linted_unless_asked() {
    let mut doc = slice_document();
    // A line through the middle of the Manta, where the floating toolbar starts (x 848-964).
    doc.layers[0].objects = vec![line(
        30,
        Frame::new(60.0, 60.0, 40.0, 0.0),
        Style::Role(LineRole::Guide),
    )];
    doc.layers[1].objects.clear();
    let manta = builtin(MANTA_ID).unwrap();
    // By default (chris, 2026-11-10: checking the floating menu bar "isn't desired"): nothing.
    let by_default = run(&doc, &manta).iter().filter(|x| x.rule == "L06").count();
    // Asked for with Toolbar::ALL (`--toolbars all`), it is Info only and names the floating toolbar.
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let laid = lay_out_text(&doc, &primary, &manta, &mut FontRegistry::bundled_only()).unwrap();
    let opts = Options {
        toolbars: Toolbar::ALL.to_vec(),
    };
    let f = lint_with(&doc, &primary, &manta, &laid.outlines, &opts);
    let l06: Vec<_> = f.iter().filter(|x| x.rule == "L06").collect();
    let ok = by_default == 0
        && !l06.is_empty()
        && l06.iter().all(|x| x.severity == Severity::Info)
        && l06.iter().all(|x| text(x).contains("floating toolbar"));
    println!(
        "CHECK crate=lint name=floating_toolbar_opt_in result={} l06_by_default={by_default} l06_opted_in={}",
        pass(ok),
        l06.len()
    );
    assert!(ok);
}

#[test]
fn every_toolbar_zone_is_known() {
    // A toolbar zone the linter cannot name would be checked whatever the user chose; adding one
    // to a profile must come with a `Toolbar` position.
    let mut unknown = Vec::new();
    for id in quadrule_profiles::builtin_ids() {
        for z in builtin(id).unwrap().safe_zones {
            if Toolbar::of_zone(&z.name).is_err() {
                unknown.push(format!("{id}/{}", z.name));
            }
        }
    }
    let ok = unknown.is_empty();
    println!(
        "CHECK crate=lint name=toolbar_zones_known result={} unknown={unknown:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn only_chosen_toolbars() {
    // chris docks left and top (2026-11-09): the slice header is then flagged for those two and
    // the footers only; the right toolbar is not mentioned (the default now has it, not the floating one).
    let doc = slice_document();
    let manta = builtin(MANTA_ID).unwrap();
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let laid = lay_out_text(&doc, &primary, &manta, &mut FontRegistry::bundled_only()).unwrap();
    let opts = Options {
        toolbars: vec![Toolbar::Left, Toolbar::Top],
    };
    let chosen = lint_with(&doc, &primary, &manta, &laid.outlines, &opts);
    let msgs: Vec<String> = chosen
        .iter()
        .filter(|x| x.rule == "L06")
        .map(text)
        .collect();
    let all = run(&doc, &manta);
    let mentions = |m: &[String], s: &str| m.iter().any(|x| x.contains(s));
    let all_msgs: Vec<String> = all.iter().filter(|x| x.rule == "L06").map(text).collect();
    let ok = mentions(&msgs, "docked left")
        && mentions(&msgs, "docked top")
        && !mentions(&msgs, "docked right")
        && !mentions(&msgs, "floating")
        && mentions(&all_msgs, "docked right");
    println!(
        "CHECK crate=lint name=only_chosen_toolbars result={} l06_chosen={} l06_all={}",
        pass(ok),
        msgs.len(),
        all_msgs.len()
    );
    assert!(ok);
}

#[test]
fn english_wording_unchanged() {
    // Moving the messages into Fluent (rung 2 bbq, Q5) must not change a word or a digit. The
    // expected text is built with the format strings the linter used before the move.
    let manta = builtin(MANTA_ID).unwrap();
    let mut doc = slice_document();
    let thin = line(
        10,
        Frame::new(20.0, 30.0, 50.0, 0.0),
        Style::Custom {
            width_mm: 0.05,
            gray: 0,
        },
    );
    let fractional = line(
        11,
        Frame::new(20.0, 36.0, 50.0, 0.0),
        Style::Custom {
            width_mm: 0.12,
            gray: 0,
        },
    );
    let diagonal = line(
        12,
        Frame::new(20.0, 42.0, 50.0, 10.0),
        Style::Role(LineRole::Guide),
    );
    let gone = line(
        21,
        Frame::new(300.0, 50.0, 10.0, 0.0),
        Style::Role(LineRole::Guide),
    );
    doc.layers[0].objects = vec![thin, fractional, diagonal, gone];
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    let f = run(&doc, &manta);
    let got = |rule: &str, id: Option<u128>| -> Vec<String> {
        f.iter()
            .filter(|x| x.rule == rule && x.object == id.map(Uuid::from_u128))
            .map(text)
            .collect()
    };
    let exact = |mm: f64| manta.mm_to_px(mm);
    let rhythm = quadrule_layout::RhythmPx::new(&doc, &manta);
    let (e01, e02) = (exact(0.05), exact(0.12));
    let expected = [
        (
            got("L01", Some(10)),
            format!(
                "\"line 10\" has a {e01:.2} px stroke on this device, thinner than one pixel: it is drawn 1 px wide and may look heavier than meant. Use a role, or at least {:.3} mm.",
                manta.mm_to_px(1.0).recip()
            ),
        ),
        (
            got("L02", Some(11)),
            format!(
                "\"line 11\" has a {e02:.2} px stroke on this device; it is drawn {:.0} px wide. A role, or a width of whole pixels, keeps it exact.",
                e02.round()
            ),
        ),
        (
            got("L02", Some(12)),
            "\"line 12\" is a diagonal line, so its edges cannot sit on whole pixels and are soft."
                .to_string(),
        ),
        (
            got("L10", Some(21)),
            "\"line 21\" is entirely off the page on this device.".to_string(),
        ),
        (
            got("L19", None),
            format!(
                "The rhythm keeps exact millimetres, so on this device its lines are {:.2} × {:.2} px apart and some gaps are 1 px wider than others. Pixel-locked rhythm keeps every gap the same.",
                rhythm.unit_x, rhythm.unit_y
            ),
        ),
    ];
    let wrong: Vec<_> = expected
        .iter()
        .filter(|(g, e)| g.as_slice() != std::slice::from_ref(e))
        .map(|(g, e)| format!("got={g:?} want={e:?}"))
        .collect();
    let ok = wrong.is_empty();
    println!(
        "CHECK crate=lint name=english_wording result={} checked={} wrong={wrong:?}",
        pass(ok),
        expected.len()
    );
    assert!(ok);
}

#[test]
fn every_message_id_exists() {
    // Every `"lint-…"` id in the linter's source is a message in the English strings, so a typo
    // shows here and never as `⟨id⟩` on screen.
    let src = include_str!("../src/lib.rs");
    let strings = quadrule_i18n::Strings::english();
    let ids: BTreeSet<&str> = src
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|s| s.starts_with("lint-"))
        .collect();
    let missing: Vec<_> = ids.iter().filter(|id| !strings.has(id)).collect();
    let ok = missing.is_empty() && ids.len() > 20;
    println!(
        "CHECK crate=lint name=message_ids_exist result={} ids={} missing={missing:?}",
        pass(ok),
        ids.len()
    );
    assert!(ok);
}

#[test]
fn findings_about_objects_have_areas() {
    // The viewer outlines a finding's area (rung 2 bbq, Q6): every finding about an object
    // carries the object's box on that device; page-wide findings (L18, L19) carry none.
    let doc = slice_document();
    let nomad = builtin(NOMAD_ID).unwrap();
    let f = run(&doc, &nomad);
    let title = f
        .iter()
        .find(|x| x.object == Some(ids::TITLE))
        .and_then(|x| x.area);
    let objects_ok = f.iter().all(|x| x.object.is_some() == x.area.is_some());
    let sane = f
        .iter()
        .filter_map(|x| x.area)
        .all(|a| a.x0 < a.x1 && a.y0 <= a.y1);
    // The slice title sits at the 8 mm margin, so its box starts near 8 mm on the Nomad.
    let margin = nomad.mm_to_px(8.0);
    let title_ok =
        title.is_some_and(|a| (a.x0 - margin).abs() < 2.0 && (a.y0 - margin).abs() < 2.0);
    let ok = objects_ok && sane && title_ok;
    println!(
        "CHECK crate=lint name=finding_areas result={} findings={} with_area={} title={title:?}",
        pass(ok),
        f.len(),
        f.iter().filter(|x| x.area.is_some()).count()
    );
    assert!(ok);
}

#[test]
fn shape_stroke_rules() {
    // Rung 6 bbq, Q8: L01 and L02 read each stroke of the new kinds; L02 only where a straight,
    // unrotated edge is (a rectangle, a triangle's base), never on an ellipse or a slanted-only
    // polygon; L06 uses the shape's frame; a shape with no stroke has nothing to lint.
    let fractional = Style::Custom {
        width_mm: 0.12,
        gray: 0,
    };
    let thin = Style::Custom {
        width_mm: 0.05,
        gray: 0,
    };
    let shape = |id: u128, style: Style, kind: ObjectKind| {
        let mut o = line(id, Frame::new(30.0, 60.0, 30.0, 20.0), style);
        o.kind = kind;
        o
    };
    let rect = || ObjectKind::Rect(RectShape::default());
    let ellipse = || ObjectKind::Ellipse(EllipseShape::default());
    let poly = |n| ObjectKind::Polygon(PolygonShape::polygon(n));
    let mut doc = slice_document();
    doc.layers[0].objects = vec![
        shape(20, thin, rect()),
        shape(21, thin, ellipse()),
        shape(22, thin, poly(5)),
        shape(23, fractional, rect()),
        shape(24, fractional, ellipse()),
        shape(25, fractional, poly(5)),
        // Eight sides, a point up: every edge is slanted.
        shape(26, fractional, poly(8)),
        shape(27, Style::None, rect()),
        shape(28, Style::Role(LineRole::Structure), rect()),
        // On the left toolbar's strip, and beyond the page's right edge.
        {
            let mut o = shape(29, Style::Role(LineRole::Structure), ellipse());
            o.frame = Frame::new(1.0, 100.0, 20.0, 20.0);
            o
        },
    ];
    let f = run(&doc, &builtin(MANTA_ID).unwrap());
    let r = |id| rules(&f, Some(Uuid::from_u128(id)));
    let ok = ["L01"]
        .iter()
        .all(|x| r(20).contains(x) && r(21).contains(x) && r(22).contains(x))
        && r(23).contains("L02")
        && !r(24).contains("L02")
        && r(25).contains("L02")
        && !r(26).contains("L02")
        && r(27).is_empty_of(&["L01", "L02"])
        && r(28).is_empty_of(&["L01", "L02"])
        && r(29).contains("L06");
    println!(
        "\nCHECK crate=lint name=shape_stroke_rules result={} rect_thin={:?} ellipse_frac={:?} pentagon_frac={:?} octagon_frac={:?} ellipse_in_toolbar={:?}",
        pass(ok),
        r(20),
        r(24),
        r(25),
        r(26),
        r(29)
    );
    assert!(ok);
}

trait EmptyOf {
    fn is_empty_of(&self, rules: &[&str]) -> bool;
}

impl EmptyOf for BTreeSet<&'static str> {
    fn is_empty_of(&self, rules: &[&str]) -> bool {
        !rules.iter().any(|r| self.contains(r))
    }
}

fn shape(id: u128, frame: Frame, style: Style, kind: ObjectKind, rotation: f64) -> Object {
    let mut o = line(id, frame, style);
    o.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    o.kind = kind;
    o.rotation_deg = rotation;
    o
}

fn one_object_page(o: Object) -> Document {
    let mut doc = slice_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects: vec![o],
        extra: Extra::new(),
    }];
    doc
}

#[test]
fn l02_reads_straight_edges_only_and_a_quarter_turn_keeps_them() {
    // Rung 6b: a stroke of 0.45 mm (5.3 px) is not whole pixels. On a rectangle with straight
    // edges L02 says so; turned by 30 degrees every edge is slanted and soft by design, so it does
    // not; turned by 90, 180 or 270 the edges are straight still, so it does again.
    let manta = builtin(MANTA_ID).unwrap();
    let style = Style::Custom {
        width_mm: 0.45,
        gray: 0,
    };
    let rect = || ObjectKind::Rect(RectShape::default());
    let mut parts = Vec::new();
    let mut ok = true;
    for (deg, want) in [
        (0.0, true),
        (30.0, false),
        (90.0, true),
        (180.0, true),
        (270.0, true),
        (45.0, false),
    ] {
        let doc = one_object_page(shape(
            20,
            Frame::new(40.0, 60.0, 50.0, 30.0),
            style,
            rect(),
            deg,
        ));
        let got = rules(&run(&doc, &manta), Some(Uuid::from_u128(20))).contains("L02");
        ok &= got == want;
        parts.push(format!("rect_{deg}={got}"));
    }
    // An ellipse is never judged, turned or not; a hexagon (vertical edges) is judged at 0 and 90
    // (its edges turn horizontal) and not at 30, where its edges are slanted.
    let ellipse = ObjectKind::Ellipse(EllipseShape::default());
    let hex = ObjectKind::Polygon(PolygonShape::polygon(6));
    for (name, kind, deg, want) in [
        ("ellipse_0", ellipse.clone(), 0.0, false),
        ("ellipse_90", ellipse, 90.0, false),
        ("hexagon_0", hex.clone(), 0.0, true),
        ("hexagon_90", hex.clone(), 90.0, true),
        ("hexagon_30", hex, 30.0, false),
    ] {
        let doc = one_object_page(shape(
            21,
            Frame::new(40.0, 60.0, 40.0, 40.0),
            style,
            kind,
            deg,
        ));
        let got = rules(&run(&doc, &manta), Some(Uuid::from_u128(21))).contains("L02");
        ok &= got == want;
        parts.push(format!("{name}={got}"));
    }
    println!(
        "\nCHECK crate=lint name=rotated_l02 result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_diagonal_line_is_judged_by_its_real_ends() {
    // L02's "diagonal" finding: a line in a box with width and height is diagonal; turned by 90
    // a horizontal line is vertical (no finding), and a line pointing up is as diagonal as one
    // pointing down.
    let manta = builtin(MANTA_ID).unwrap();
    let mk = |frame: Frame, dir: LineDir, deg: f64| {
        let mut o = line(30, frame, Style::Role(LineRole::Structure));
        o.rotation_deg = deg;
        o.kind = ObjectKind::Line(Line {
            direction: dir,
            ..Line::default()
        });
        o
    };
    let diag = |o: Object| {
        let doc = one_object_page(o);
        run(&doc, &manta)
            .iter()
            .any(|f| f.rule == "L02" && f.object == Some(Uuid::from_u128(30)))
    };
    let down = diag(mk(Frame::new(40.0, 60.0, 30.0, 20.0), LineDir::Down, 0.0));
    let up = diag(mk(Frame::new(40.0, 60.0, 30.0, 20.0), LineDir::Up, 0.0));
    let flat = diag(mk(Frame::new(40.0, 60.0, 30.0, 0.0), LineDir::Down, 0.0));
    let flat_turned_90 = diag(mk(Frame::new(40.0, 60.0, 30.0, 0.0), LineDir::Down, 90.0));
    let flat_turned_30 = diag(mk(Frame::new(40.0, 60.0, 30.0, 0.0), LineDir::Down, 30.0));
    let ok = down && up && !flat && !flat_turned_90 && flat_turned_30;
    println!(
        "\nCHECK crate=lint name=rotated_line_diagonal result={} down={down} up={up} flat={flat} flat_turned_90={flat_turned_90} flat_turned_30={flat_turned_30}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l06_and_l10_use_the_turned_corners() {
    // A square turned 45 degrees (a diamond, half-diagonal 100 px) centred 60 px up and right of
    // the Manta footer's top-right corner (497, 2444): the box round its corners overlaps the
    // footer, the diamond does not. Unturned (a square 141 px across) it does.
    let manta = builtin(MANTA_ID).unwrap();
    let k = manta.mm_to_px(1.0);
    let side = 100.0 * std::f64::consts::SQRT_2;
    let (cx, cy) = (497.0 + 60.0, 2444.0 - 60.0);
    let frame = Frame::new(
        (cx - side / 2.0) / k,
        (cy - side / 2.0) / k,
        side / k,
        side / k,
    );
    let style = Style::Role(LineRole::Whisper);
    let rect = ObjectKind::Rect(RectShape::default());
    let l06 = |deg: f64| {
        let doc = one_object_page(shape(40, frame, style, rect.clone(), deg));
        run(&doc, &manta)
            .iter()
            .any(|f| f.rule == "L06" && f.object == Some(Uuid::from_u128(40)))
    };
    let (flat, diamond) = (l06(0.0), l06(45.0));
    // L10: a bar 20 px wide and 300 tall near the right edge, turned 90, lies across it and
    // reaches past the page.
    let bar = Frame::new(1840.0 / k, 850.0 / k, 20.0 / k, 300.0 / k);
    let l10 = |deg: f64| {
        let doc = one_object_page(shape(41, bar, style, rect.clone(), deg));
        run(&doc, &manta)
            .iter()
            .any(|f| f.rule == "L10" && f.object == Some(Uuid::from_u128(41)))
    };
    let (bar_flat, bar_turned) = (l10(0.0), l10(90.0));
    let ok = flat && !diamond && !bar_flat && bar_turned;
    println!(
        "\nCHECK crate=lint name=rotated_l06_l10 result={} unturned_in_footer_zone={flat} diamond_clear_of_zone={} bar_on_page={} bar_turned_runs_off={bar_turned}",
        pass(ok),
        !diamond,
        !bar_flat
    );
    assert!(ok);
}
