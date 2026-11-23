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

//! The rules that read the profile's lint block (rung 14a, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md,
//! Q3, Q4 and the Defaulted list): L03 gray too light, L05 gray off the clean levels, L07 text too small, L09
//! handwriting step, L13 dark fills and L14 soft thin lines. Each rule fires at the first value past its
//! threshold and not at the threshold, on both Supernotes; follows an edited override's lint block; judges the
//! gray as it lands on the page (opacity, groups, roles); and is skipped on paper, on a Wallpaper (L09, L13), when
//! the profile has no value, and when the page ignores it. Free placement silences L16 only. Each test prints
//! one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, Registry, active, builtin, builtin_at};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// Lints `doc` on `target`, the design made on the document's primary profile.
fn run(doc: &Document, target: &Profile) -> Vec<Finding> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let primary = if primary.is_paper() {
        target.clone()
    } else {
        primary
    };
    let laid = lay_out_text(doc, &primary, target, &mut FontRegistry::bundled_only()).unwrap();
    lint(doc, &primary, target, &laid.outlines)
}

fn has(f: &[Finding], rule: &str, id: u128) -> bool {
    f.iter()
        .any(|x| x.rule == rule && x.object == Some(Uuid::from_u128(id)))
}

fn any(f: &[Finding], rule: &str) -> bool {
    f.iter().any(|x| x.rule == rule)
}

fn english(f: &Finding) -> String {
    quadrule_i18n::Strings::english()
        .try_format(&f.message)
        .unwrap()
}

/// One object at id `id`, with its own box and no constraint that moves it between devices.
fn obj(id: u128, frame: Frame, style: Style, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(id),
        name: format!("probe {id}"),
        frame,
        constraints: Constraints {
            h: HConstraint::Left,
            v: VConstraint::Top,
        },
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

/// A Template page with just `objects`, on the slice's two Supernotes with a 6 mm rhythm.
fn page(objects: Vec<Object>) -> Document {
    let mut doc = slice_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc
}

fn px_mm(px: u32) -> f64 {
    f64::from(px) * 25.4 / 300.0
}

fn custom(px: u32, gray: u8) -> Style {
    Style::Custom {
        width_mm: px_mm(px),
        gray,
    }
}

fn devices() -> [Profile; 2] {
    [builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap()]
}

/// A rule that fires for the first value past `threshold` and not at it, on both devices.
fn edge(
    rule: &str,
    threshold: u8,
    make: impl Fn(u8) -> Object,
    devices: &[Profile; 2],
) -> (bool, bool, bool) {
    let (mut at, mut past, mut named) = (true, true, true);
    for d in devices {
        let at_doc = page(vec![make(threshold)]);
        let past_doc = page(vec![make(threshold + 1)]);
        at &= !has(&run(&at_doc, d), rule, 1);
        let f = run(&past_doc, d);
        past &= has(&f, rule, 1);
        named &= f
            .iter()
            .filter(|x| x.rule == rule)
            .all(|x| english(x).contains(&format!("{}", threshold + 1)) || rule == "L05");
    }
    (at, past, named)
}

type Maker = Box<dyn Fn(u8) -> Object>;

#[test]
fn l03_fires_one_gray_past_each_threshold_on_both_devices() {
    let d = devices();
    let frame = Frame::new(20.0, 30.0, 40.0, 0.0);
    let rect_frame = Frame::new(20.0, 30.0, 20.0, 10.0);
    let line =
        |px: u32| move |g: u8| obj(1, frame, custom(px, g), ObjectKind::Line(Line::default()));
    let dots = |px: u32| {
        move |g: u8| {
            obj(
                1,
                Frame::new(20.0, 30.0, 40.0, 30.0),
                custom(px, g),
                ObjectKind::Pattern(PatternRegion::new(Pattern::Dots)),
            )
        }
    };
    let fill = move |g: u8| {
        let mut o = obj(
            1,
            rect_frame,
            Style::None,
            ObjectKind::Rect(RectShape::default()),
        );
        o.fill = Some(Fill::Gray(g));
        o
    };
    let text = |g: u8| {
        obj(
            1,
            Frame::new(20.0, 30.0, 60.0, 8.0),
            Style::Custom {
                width_mm: 0.0,
                gray: g,
            },
            ObjectKind::Text(TextBox {
                content: "Legible".into(),
                size_pt: 12.0,
                ..TextBox::default()
            }),
        )
    };
    let cases: Vec<(&str, u8, Maker)> = vec![
        ("line_1px", 187, Box::new(line(1))),
        ("line_2px", 204, Box::new(line(2))),
        ("line_5px", 204, Box::new(line(5))),
        ("dot_2px", 102, Box::new(dots(2))),
        ("dot_3px", 170, Box::new(dots(3))),
        ("dot_4px", 187, Box::new(dots(4))),
        ("dot_6px", 187, Box::new(dots(6))),
        ("fill", 204, Box::new(fill)),
        ("text", 153, Box::new(text)),
    ];
    let mut ok = true;
    let mut parts = Vec::new();
    for (name, threshold, make) in &cases {
        let (at, past, named) = edge("L03", *threshold, make, &d);
        ok &= at && past && named;
        parts.push(format!("{name}@{threshold}:ok={}", at && past && named));
    }
    println!(
        "CHECK crate=lint name=l03_thresholds result={} devices=2 {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn l05_fires_for_a_gray_off_the_clean_levels_only() {
    let d = devices();
    let mk = |g: u8| {
        obj(
            1,
            Frame::new(20.0, 30.0, 40.0, 0.0),
            custom(2, g),
            ObjectKind::Line(Line::default()),
        )
    };
    let (mut clean_ok, mut off_ok, mut sev) = (true, true, true);
    for p in &d {
        for g in &p.gray_levels {
            clean_ok &= !has(&run(&page(vec![mk(*g)]), p), "L05", 1);
        }
        let f = run(&page(vec![mk(120)]), p);
        off_ok &= has(&f, "L05", 1);
        sev &= f
            .iter()
            .filter(|x| x.rule == "L05")
            .all(|x| x.severity == Severity::Info);
    }
    // The shipped roles are calibrated, so they are clean; an edited role is not.
    let role = obj(
        1,
        Frame::new(20.0, 30.0, 40.0, 0.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Line(Line::default()),
    );
    let roles_clean = d
        .iter()
        .all(|p| !any(&run(&page(vec![role.clone()]), p), "L05"));
    let ok = clean_ok && off_ok && sev && roles_clean;
    println!(
        "CHECK crate=lint name=l05_clean_levels result={} every_clean_level_passes={clean_ok} off_level_fires={off_ok} info={sev} shipped_roles_clean={roles_clean}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l07_fires_below_the_smallest_size_and_follows_a_device_override() {
    let d = devices();
    let mk = |pt: f64| {
        obj(
            1,
            Frame::new(20.0, 30.0, 60.0, 8.0),
            Style::Role(LineRole::Text),
            ObjectKind::Text(TextBox {
                content: "Small".into(),
                size_pt: pt,
                ..TextBox::default()
            }),
        )
    };
    let (mut at, mut under) = (true, true);
    for p in &d {
        at &= !any(&run(&page(vec![mk(6.0)]), p), "L07");
        let f = run(&page(vec![mk(5.9)]), p);
        under &= has(&f, "L07", 1)
            && f.iter()
                .all(|x| x.rule != "L07" || x.severity == Severity::Warn);
    }
    // A font-size override on the Nomad only: 5 pt there is small, the Manta keeps 12 pt.
    let mut o = mk(12.0);
    o.overrides.insert(
        NOMAD_ID.to_string(),
        Override {
            font_size_pt: Some(5.0),
            ..Override::default()
        },
    );
    let doc = page(vec![o]);
    let [manta, nomad] = &d;
    let by_device = !any(&run(&doc, manta), "L07") && has(&run(&doc, nomad), "L07", 1);
    let ok = at && under && by_device;
    println!(
        "CHECK crate=lint name=l07_text_size result={} at_6pt_quiet={at} under_6pt_fires={under} override_judged_per_device={by_device}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l09_fires_for_a_step_under_the_handwriting_size_and_only_on_a_template() {
    let d = devices();
    let with_step = |mm: f64| {
        let mut doc = page(vec![]);
        doc.rhythm.h_unit_mm = mm;
        doc.rhythm.v_unit_mm = mm;
        doc
    };
    let (mut at, mut under) = (true, true);
    for p in &d {
        at &= !any(&run(&with_step(5.0), p), "L09");
        let f = run(&with_step(4.9), p);
        under &= any(&f, "L09")
            && f.iter()
                .all(|x| x.rule != "L09" || x.severity == Severity::Info);
    }
    // A ruled region of its own spacing: 4.9 mm is under, 5 mm is not; one bound to a 4.9 mm rhythm is said
    // once, by the rhythm.
    let ruled = |bound: bool, mm: f64| {
        let mut pr = PatternRegion::new(Pattern::Lines);
        pr.bound = bound;
        pr.v_spacing_mm = Some(mm);
        obj(
            1,
            Frame::new(10.0, 20.0, 60.0, 60.0),
            Style::Role(LineRole::Guide),
            ObjectKind::Pattern(pr),
        )
    };
    let own = |mm: f64| page(vec![ruled(false, mm)]);
    let manta = &d[0];
    let region_at = !any(&run(&own(5.0), manta), "L09");
    let region_under = has(&run(&own(4.9), manta), "L09", 1);
    let mut bound_doc = with_step(4.9);
    bound_doc.layers = page(vec![ruled(true, 4.9)]).layers;
    let once = run(&bound_doc, manta)
        .iter()
        .filter(|x| x.rule == "L09")
        .count()
        == 1;
    // A Wallpaper is not handwritten on.
    let mut wall = with_step(4.0);
    wall.document_type = DocumentType::Wallpaper;
    let wallpaper = d.iter().all(|p| !any(&run(&wall, p), "L09"));
    let ok = at && under && region_at && region_under && once && wallpaper;
    println!(
        "CHECK crate=lint name=l09_handwriting_step result={} rhythm_at_5mm_quiet={at} rhythm_4_9_fires={under} region_at_5mm_quiet={region_at} region_4_9_fires={region_under} bound_region_said_once={once} wallpaper_skips={wallpaper}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l13_fires_for_dark_fills_over_the_share_and_not_for_lighter_or_smaller() {
    let d = devices();
    // The Manta and Nomad pages have the same shape, so a box a share of the width and height is the same
    // share on both.
    let share = |pct: f64, gray: u8, opacity: u8| {
        let (w, h) = (MANTA_W_MM, MANTA_H_MM);
        let mut o = obj(
            1,
            Frame::new(0.0, 0.0, w, h * pct / 100.0),
            Style::None,
            ObjectKind::Rect(RectShape::default()),
        );
        o.constraints = Constraints {
            h: HConstraint::Scale,
            v: VConstraint::Scale,
        };
        o.fill = Some(Fill::Gray(gray));
        o.opacity_pct = opacity;
        page(vec![o])
    };
    let (mut over, mut under, mut lighter, mut faded) = (true, true, true, true);
    for p in &d {
        over &= has(&run(&share(12.0, 102, 100), p), "L13", 1);
        under &= !any(&run(&share(8.0, 102, 100), p), "L13");
        lighter &= !any(&run(&share(30.0, 103, 100), p), "L13");
        // 102 at half opacity over white lands at 115: lighter than the dark gray.
        faded &= !any(&run(&share(30.0, 102, 50), p), "L13");
    }
    // Two 6 percent fills together are 12 percent.
    let two = {
        let mut doc = share(6.0, 0, 100);
        let mut second = doc.layers[0].objects[0].clone();
        second.id = Uuid::from_u128(2);
        second.frame.y_mm = MANTA_H_MM * 0.5;
        doc.layers[0].objects.push(second);
        doc
    };
    let together = d.iter().all(|p| any(&run(&two, p), "L13"));
    // The Block role is dark; the Shade role is not. A Wallpaper skips it.
    let block = {
        let mut doc = share(30.0, 0, 100);
        doc.layers[0].objects[0].fill = Some(Fill::Role(FillRole::Block));
        doc
    };
    let shade = {
        let mut doc = share(30.0, 0, 100);
        doc.layers[0].objects[0].fill = Some(Fill::Role(FillRole::Shade));
        doc
    };
    let roles = d
        .iter()
        .all(|p| any(&run(&block, p), "L13") && !any(&run(&shade, p), "L13"));
    let mut wall = share(30.0, 0, 100);
    wall.document_type = DocumentType::Wallpaper;
    let wallpaper = d.iter().all(|p| !any(&run(&wall, p), "L13"));
    let ok = over && under && lighter && faded && together && roles && wallpaper;
    println!(
        "CHECK crate=lint name=l13_dark_fills result={} over_share_fires={over} under_share_quiet={under} gray_103_quiet={lighter} faded_quiet={faded} fills_add_up={together} roles={roles} wallpaper_skips={wallpaper}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn l14_fires_for_a_thin_slanted_or_curved_stroke_and_not_for_straight_or_thick_ones() {
    let d = devices();
    let diag = |px: u32| {
        obj(
            1,
            Frame::new(20.0, 30.0, 40.0, 20.0),
            custom(px, 51),
            ObjectKind::Line(Line::default()),
        )
    };
    let flat = obj(
        1,
        Frame::new(20.0, 30.0, 40.0, 0.0),
        custom(1, 51),
        ObjectKind::Line(Line::default()),
    );
    let ellipse = |px: u32| {
        obj(
            1,
            Frame::new(20.0, 30.0, 30.0, 20.0),
            custom(px, 51),
            ObjectKind::Ellipse(EllipseShape::default()),
        )
    };
    let rect = |deg: f64| {
        let mut o = obj(
            1,
            Frame::new(20.0, 30.0, 30.0, 20.0),
            custom(1, 51),
            ObjectKind::Rect(RectShape::default()),
        );
        o.rotation_deg = deg;
        o
    };
    let role_diag = |role: LineRole| {
        let mut o = diag(1);
        o.style = Style::Role(role);
        o
    };
    let fires = |o: Object, p: &Profile| has(&run(&page(vec![o]), p), "L14", 1);
    let mut checks = Vec::new();
    for p in &d {
        checks.push(("diag_2px", fires(diag(2), p), true));
        checks.push(("diag_3px", fires(diag(3), p), false));
        checks.push(("flat_1px", fires(flat.clone(), p), false));
        checks.push(("ellipse_1px", fires(ellipse(1), p), true));
        checks.push(("ellipse_3px", fires(ellipse(3), p), false));
        checks.push(("rect_straight", fires(rect(0.0), p), false));
        checks.push(("rect_quarter_turn", fires(rect(90.0), p), false));
        checks.push(("rect_30deg", fires(rect(30.0), p), true));
        checks.push((
            "structure_diag_2px",
            fires(role_diag(LineRole::Structure), p),
            true,
        ));
        checks.push((
            "emphasis_diag_3px",
            fires(role_diag(LineRole::Emphasis), p),
            false,
        ));
    }
    // L02 keeps the straight edges: a diagonal line no longer says L02.
    let no_l02 = d.iter().all(|p| !any(&run(&page(vec![diag(1)]), p), "L02"));
    let wrong: Vec<&str> = checks
        .iter()
        .filter(|(_, got, want)| got != want)
        .map(|(n, _, _)| *n)
        .collect();
    let ok = wrong.is_empty() && no_l02;
    println!(
        "CHECK crate=lint name=l14_soft_thin_lines result={} cases={} wrong={wrong:?} diagonal_no_longer_l02={no_l02}",
        pass(ok),
        checks.len()
    );
    assert!(ok);
}

#[test]
fn the_gray_judged_is_the_gray_put_on_the_page() {
    let d = devices();
    let [manta, nomad] = &d;
    let faint = |opacity: u8| {
        let mut o = obj(
            1,
            Frame::new(20.0, 30.0, 40.0, 0.0),
            custom(2, 100),
            ObjectKind::Line(Line::default()),
        );
        o.opacity_pct = opacity;
        o
    };
    // Gray 100 at 100 percent is dark enough; at 30 percent it lands at 208, past 204; at 50, 178.
    let full = !any(&run(&page(vec![faint(100)]), manta), "L03");
    let half = !any(&run(&page(vec![faint(50)]), manta), "L03");
    let third = has(&run(&page(vec![faint(30)]), manta), "L03", 1);
    // A group's opacity counts too.
    let grouped = {
        let mut g = obj(
            9,
            Frame::new(0.0, 0.0, 100.0, 100.0),
            Style::None,
            ObjectKind::Group(GroupShape::default()),
        );
        g.opacity_pct = 30;
        if let ObjectKind::Group(s) = &mut g.kind {
            s.children.push(faint(100));
        }
        page(vec![g])
    };
    let group_counts = has(&run(&grouped, nomad), "L03", 1);
    // A background that is not white: the same line over a dark page lands darker.
    let mut dark = page(vec![faint(30)]);
    dark.page.background_gray = 0;
    let background = !any(&run(&dark, manta), "L03");
    // A role is judged like any other style; a calibrated one passes.
    let guide = obj(
        1,
        Frame::new(20.0, 30.0, 40.0, 0.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Line(Line::default()),
    );
    let role_passes = d
        .iter()
        .all(|p| !any(&run(&page(vec![guide.clone()]), p), "L03"));
    let ok = full && half && third && group_counts && background && role_passes;
    println!(
        "CHECK crate=lint name=gray_on_the_page result={} full_ok={full} half_opacity_ok={half} thirty_percent_fires={third} group_opacity_counts={group_counts} dark_background_ok={background} shipped_role_passes={role_passes}",
        pass(ok)
    );
    assert!(ok);
}

fn scratch(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-lint-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn a_rule_follows_an_edited_overrides_lint_block_and_an_edited_role() {
    let d = scratch("override");
    active::set(Registry::at(&d));
    let line = |style: Style| {
        obj(
            1,
            Frame::new(20.0, 30.0, 40.0, 0.0),
            style,
            ObjectKind::Line(Line::default()),
        )
    };
    let doc = |style: Style| page(vec![line(style)]);
    let lint_on = |doc: &Document| {
        let t = active::lookup_in(doc, NOMAD_ID).unwrap();
        run(doc, &t)
    };
    // A 1 px line of gray 100: fine on the shipped Nomad.
    let custom_100 = doc(custom(1, 100));
    let shipped = !has(&lint_on(&custom_100), "L03", 1);
    // The person lowers the 1 px limit to 60 in the Profiles window: the same line is now too light.
    let mut edited = builtin(NOMAD_ID).unwrap();
    edited.lint.lightest_gray_line_1px = Some(60);
    active::with_mut(|r| r.save(edited.clone())).unwrap();
    let follows = has(&lint_on(&custom_100), "L03", 1);
    // They edit the Guide role's gray to 230: a role-styled line is judged, and is too light.
    edited.lint.lightest_gray_line_1px = Some(187);
    edited.line_roles.guide.gray = 230;
    active::with_mut(|r| r.save(edited.clone())).unwrap();
    let role_judged = has(&lint_on(&doc(Style::Role(LineRole::Guide))), "L03", 1);
    // They clear every gray value: the rule is skipped on the Nomad and the list says why.
    edited.lint.lightest_gray_line_1px = None;
    edited.lint.lightest_gray_line_2px_up = None;
    edited.lint.lightest_gray_dot_2px = None;
    edited.lint.lightest_gray_dot_3px = None;
    edited.lint.lightest_gray_dot_4px_up = None;
    edited.lint.lightest_gray_fill = None;
    edited.lint.lightest_gray_text = None;
    active::with_mut(|r| r.save(edited.clone())).unwrap();
    let skipped = !has(&lint_on(&doc(Style::Role(LineRole::Guide))), "L03", 1);
    let no_value = skipped_checks(
        &custom_100,
        &active::lookup_in(&custom_100, NOMAD_ID).unwrap(),
    )
    .contains(&Skipped {
        rule: "L03",
        reason: SkipReason::NoValue,
    });
    // The Manta, untouched, still judges by the shipped values.
    let manta_same = {
        let t = active::lookup_in(&custom_100, MANTA_ID).unwrap();
        !has(&run(&custom_100, &t), "L03", 1)
    };
    active::set(Registry::builtin_only());
    let ok = shipped && follows && role_judged && skipped && no_value && manta_same;
    println!(
        "CHECK crate=lint name=rules_follow_the_profile_in_use result={} shipped_quiet={shipped} edited_limit_followed={follows} edited_role_judged={role_judged} cleared_values_skip={skipped} skip_reason_no_value={no_value} other_device_unchanged={manta_same}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn paper_skips_gray_and_ink_rules_and_runs_text_size_and_handwriting() {
    let a4 = builtin_at(PAPER_A4_ID, Orientation::Portrait, 300).unwrap();
    let mut doc = paper_document();
    doc.profiles.truncate(1);
    let mut objects = Vec::new();
    // A faint line, a slanted thin line, a big dark fill, 5 pt text.
    objects.push(obj(
        1,
        Frame::new(20.0, 30.0, 40.0, 0.0),
        custom(2, 230),
        ObjectKind::Line(Line::default()),
    ));
    objects.push(obj(
        2,
        Frame::new(20.0, 50.0, 40.0, 20.0),
        custom(1, 51),
        ObjectKind::Line(Line::default()),
    ));
    let mut dark = obj(
        3,
        Frame::new(0.0, 0.0, 210.0, 297.0),
        Style::None,
        ObjectKind::Rect(RectShape::default()),
    );
    dark.fill = Some(Fill::Gray(0));
    objects.push(dark);
    objects.push(obj(
        4,
        Frame::new(20.0, 100.0, 60.0, 8.0),
        Style::Role(LineRole::Text),
        ObjectKind::Text(TextBox {
            content: "Tiny".into(),
            size_pt: 5.0,
            ..TextBox::default()
        }),
    ));
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc.rhythm.v_unit_mm = 4.0;
    doc.rhythm.h_unit_mm = 4.0;
    let f = run(&doc, &a4);
    let skipped = ["L03", "L05", "L13", "L14"].iter().all(|r| !any(&f, r));
    let runs = has(&f, "L07", 4) && any(&f, "L09");
    let listed = skipped_checks(&doc, &a4);
    let reasons = ["L03", "L05", "L06", "L13", "L14"].iter().all(|r| {
        listed.contains(&Skipped {
            rule: r,
            reason: SkipReason::Paper,
        })
    }) && !listed.iter().any(|s| s.rule == "L07" || s.rule == "L09");
    // The same page held as a Nomad judges all of them.
    let nomad_f = {
        let mut screen = doc.clone();
        screen.primary_profile_id = NOMAD_ID.into();
        screen.profiles = vec![ProfileContext::new(NOMAD_ID)];
        run(&screen, &builtin(NOMAD_ID).unwrap())
    };
    let screen_judges = any(&nomad_f, "L03") && any(&nomad_f, "L13") && any(&nomad_f, "L14");
    let ok = skipped && runs && reasons && screen_judges;
    println!(
        "CHECK crate=lint name=paper_skips_gray_rules result={} l03_l05_l13_l14_skipped={skipped} l07_l09_run={runs} skip_list_says_paper={reasons} same_page_as_screen_judges={screen_judges}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn ignored_rules_and_free_placement_silence_what_they_should() {
    let manta = builtin(MANTA_ID).unwrap();
    let nomad = builtin(NOMAD_ID).unwrap();
    // A faint line, off the 6 mm rhythm (x = 21.7 mm; the rhythm is on 8, 14, 20, 26 ...).
    let faint = obj(
        1,
        Frame::new(21.7, 31.7, 40.0, 0.0),
        custom(2, 230),
        ObjectKind::Line(Line::default()),
    );
    let base = page(vec![faint.clone()]);
    let f = run(&base, &manta);
    let both = has(&f, "L03", 1) && has(&f, "L16", 1);
    // Free placement silences L16 and nothing else.
    let mut free = base.clone();
    free.layers[0].objects[0].free_placement = true;
    let f = run(&free, &manta);
    let free_ok = !has(&f, "L16", 1) && has(&f, "L03", 1);
    // A group's flag covers what is inside it.
    let mut group = obj(
        9,
        Frame::new(0.0, 0.0, 100.0, 100.0),
        Style::None,
        ObjectKind::Group(GroupShape::default()),
    );
    group.free_placement = true;
    if let ObjectKind::Group(g) = &mut group.kind {
        g.children.push(faint.clone());
    }
    let f = run(&page(vec![group]), &manta);
    let group_ok = !has(&f, "L16", 1) && has(&f, "L03", 1);
    // Ignoring a rule turns it off on every device.
    let mut ignoring = base.clone();
    ignoring.ignored_rules = vec!["L03".into()];
    let on_all = [&manta, &nomad].iter().all(|p| {
        !any(&run(&ignoring, p), "L03")
            && any(&run(&ignoring, p), "L10") == any(&run(&base, p), "L10")
    });
    let l16_stays = has(&run(&ignoring, &manta), "L16", 1);
    // The skip list names it.
    let listed = skipped_checks(&ignoring, &nomad).contains(&Skipped {
        rule: "L03",
        reason: SkipReason::Ignored,
    });
    // An ignored rule that is not one of the six is listed as ignored too.
    let mut other = base.clone();
    other.ignored_rules = vec!["L16".into(), "Lxx".into()];
    let listed_other = skipped_checks(&other, &manta).contains(&Skipped {
        rule: "L16",
        reason: SkipReason::Ignored,
    }) && !skipped_checks(&other, &manta)
        .iter()
        .any(|s| s.rule == "Lxx");
    let ok = both && free_ok && group_ok && on_all && l16_stays && listed && listed_other;
    println!(
        "CHECK crate=lint name=ignored_and_free_placement result={} both_fire={both} free_silences_l16_only={free_ok} group_flag_covers_children={group_ok} ignored_off_on_every_device={on_all} other_rules_stay={l16_stays} skip_list_ignored={listed} ignored_l16_listed={listed_other}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn skip_reasons_cover_paper_wallpaper_no_grays_and_no_value() {
    let nomad = builtin(NOMAD_ID).unwrap();
    let doc = page(vec![]);
    let nothing = skipped_checks(&doc, &nomad).is_empty();
    let mut wall = doc.clone();
    wall.document_type = DocumentType::Wallpaper;
    let w = skipped_checks(&wall, &nomad);
    let wallpaper = ["L09", "L13"].iter().all(|r| {
        w.contains(&Skipped {
            rule: r,
            reason: SkipReason::Wallpaper,
        })
    }) && w.len() == 2;
    let mut bare = nomad.clone();
    bare.gray_levels.clear();
    bare.lint.smallest_text_pt = None;
    bare.lint.thin_line_px = None;
    let b = skipped_checks(&doc, &bare);
    let no_grays = b.contains(&Skipped {
        rule: "L05",
        reason: SkipReason::NoGrayLevels,
    });
    let no_value = ["L07", "L14"].iter().all(|r| {
        b.contains(&Skipped {
            rule: r,
            reason: SkipReason::NoValue,
        })
    });
    // And the rules really are off: no L05 with no levels, no L07 with no size, no L14 with no width.
    let probe = page(vec![
        obj(
            1,
            Frame::new(20.0, 30.0, 40.0, 20.0),
            custom(1, 51),
            ObjectKind::Line(Line::default()),
        ),
        obj(
            2,
            Frame::new(20.0, 60.0, 60.0, 8.0),
            Style::Role(LineRole::Text),
            ObjectKind::Text(TextBox {
                content: "Tiny".into(),
                size_pt: 3.0,
                ..TextBox::default()
            }),
        ),
    ]);
    let f = run(&probe, &bare);
    let really_off = !any(&f, "L05") && !any(&f, "L07") && !any(&f, "L14");
    let on = {
        let g = run(&probe, &nomad);
        any(&g, "L07") && any(&g, "L14")
    };
    let ok = nothing && wallpaper && no_grays && no_value && really_off && on;
    println!(
        "CHECK crate=lint name=skip_reasons result={} nothing_skipped_on_the_nomad={nothing} wallpaper_l09_l13={wallpaper} no_gray_levels_l05={no_grays} no_value_l07_l14={no_value} rules_off_when_skipped={really_off}",
        pass(ok)
    );
    assert!(ok);
}
