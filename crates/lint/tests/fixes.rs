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

//! Quick fixes (rung 14b, docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q6), worked out with no screen: each
//! fix is offered on the finding it is for, and applying what it plans makes that finding go away without making a
//! new one of the same rule; the rules the record gives no button have none. Each test prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::repeat::Spacing;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn manta() -> Profile {
    builtin(MANTA_ID).unwrap()
}

fn nomad() -> Profile {
    builtin(NOMAD_ID).unwrap()
}

fn run(doc: &Document, target: &Profile) -> Vec<Finding> {
    let primary = manta();
    let laid = lay_out_text(doc, &primary, target, &mut FontRegistry::bundled_only()).unwrap();
    lint(doc, &primary, target, &laid.outlines)
}

fn obj(id: u128, name: &str, frame: Frame, style: Style, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(id),
        name: name.into(),
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
        free_placement: false,
        full_bleed: false,
        kind,
        extra: Extra::new(),
    }
}

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

fn mm(px: f64) -> f64 {
    px * 25.4 / 300.0
}

/// Applies planned edits to a copy of the document, the way the window would on the primary device.
fn apply(doc: &Document, target: &Profile, edits: &[Edit]) -> Document {
    let mut d = doc.clone();
    for e in edits {
        match e {
            Edit::Object(o) => *d.object_mut(o.id).unwrap() = o.clone(),
            Edit::Translate { id, dx_mm, dy_mm } => {
                d.object_mut(*id).unwrap().translate(*dx_mm, *dy_mm)
            }
            Edit::MovePage { id, dx_px, dy_px } => {
                let k = target.mm_to_px(1.0);
                d.object_mut(*id).unwrap().translate(dx_px / k, dy_px / k)
            }
            Edit::TextSize { id, pt } => {
                if let ObjectKind::Text(t) = &mut d.object_mut(*id).unwrap().kind {
                    t.size_pt = *pt;
                }
            }
            Edit::Rhythm(r) => d.rhythm = r.clone(),
        }
    }
    d
}

fn of<'a>(f: &'a [Finding], rule: &str, id: u128) -> Option<&'a Finding> {
    f.iter()
        .find(|x| x.rule == rule && x.object == Some(Uuid::from_u128(id)))
}

fn gone(f: &[Finding], rule: &str, id: u128) -> bool {
    of(f, rule, id).is_none()
}

/// The fix `want` is offered on the finding; applying it silences the rule for that object.
fn fixes(doc: &Document, target: &Profile, rule: &str, id: u128, want: Fix) -> (bool, bool) {
    let primary = manta();
    let f = run(doc, target);
    let Some(found) = of(&f, rule, id) else {
        return (false, false);
    };
    let offered = offered(doc, &primary, target, found);
    let edits = plan(doc, &primary, target, found, want);
    let after = edits
        .map(|e| gone(&run(&apply(doc, target, &e), target), rule, id))
        .unwrap_or(false);
    (offered.contains(&want), after)
}

#[test]
fn snap_to_pixel_grid_fixes_l01_and_l02_on_either_device() {
    let thin = obj(
        1,
        "thin",
        Frame::new(20.0, 20.0, 40.0, 0.0),
        Style::Custom {
            width_mm: mm(0.4),
            gray: 51,
        },
        ObjectKind::Line(Line::default()),
    );
    let frac = obj(
        2,
        "frac",
        Frame::new(20.0, 30.0, 40.0, 0.0),
        Style::Custom {
            width_mm: mm(2.5),
            gray: 51,
        },
        ObjectKind::Line(Line::default()),
    );
    let doc = page(vec![thin, frac]);
    let mut ok = true;
    let mut detail = Vec::new();
    for (name, target) in [("manta", manta()), ("nomad", nomad())] {
        let a = fixes(&doc, &target, "L01", 1, Fix::SnapToPixelGrid);
        let b = fixes(&doc, &target, "L02", 2, Fix::SnapToPixelGrid);
        ok &= a == (true, true) && b == (true, true);
        detail.push(format!("{name}_l01={a:?} {name}_l02={b:?}"));
    }
    // A role-styled line has nothing to snap.
    println!(
        "CHECK crate=lint name=fix_snap_to_pixel_grid result={} {}",
        pass(ok),
        detail.join(" ")
    );
    assert!(ok);
}

#[test]
fn nearest_clean_gray_and_a_role_fix_l03_and_l05() {
    let line = obj(
        1,
        "pale line",
        Frame::new(20.0, 20.0, 40.0, 0.0),
        Style::Custom {
            width_mm: mm(1.0),
            gray: 230,
        },
        ObjectKind::Line(Line::default()),
    );
    let off = obj(
        2,
        "banded",
        Frame::new(20.0, 30.0, 40.0, 0.0),
        Style::Custom {
            width_mm: mm(1.0),
            gray: 120,
        },
        ObjectKind::Line(Line::default()),
    );
    let mut fill = obj(
        3,
        "pale box",
        Frame::new(20.0, 40.0, 30.0, 10.0),
        Style::None,
        ObjectKind::Rect(RectShape::default()),
    );
    fill.fill = Some(Fill::Gray(230));
    let text = obj(
        4,
        "pale words",
        Frame::new(20.0, 55.0, 40.0, 8.0),
        Style::Custom {
            width_mm: mm(1.0),
            gray: 200,
        },
        ObjectKind::Text(TextBox {
            content: "words".into(),
            family: ATKINSON.into(),
            size_pt: 12.0,
            ..TextBox::default()
        }),
    );
    let doc = page(vec![line, off, fill, text]);
    let t = manta();
    let a = fixes(&doc, &t, "L03", 1, Fix::NearestCleanGray);
    let b = fixes(&doc, &t, "L05", 2, Fix::NearestCleanGray);
    let c = fixes(&doc, &t, "L03", 3, Fix::NearestCleanGray);
    let d = fixes(&doc, &t, "L03", 4, Fix::NearestCleanGray);
    // The nearest clean gray to 230 that shows a 1 px line is 187 (the lightest allowed); to 120 it is 119.
    let f = run(&doc, &t);
    let g1 = plan(
        &doc,
        &manta(),
        &t,
        of(&f, "L03", 1).unwrap(),
        Fix::NearestCleanGray,
    )
    .unwrap();
    let g2 = plan(
        &doc,
        &manta(),
        &t,
        of(&f, "L05", 2).unwrap(),
        Fix::NearestCleanGray,
    )
    .unwrap();
    let gray_of = |e: &[Edit]| match &e[0] {
        Edit::Object(o) => match o.style {
            Style::Custom { gray, .. } => gray,
            _ => 255,
        },
        _ => 255,
    };
    let (n1, n2) = (gray_of(&g1), gray_of(&g2));
    // A role that fits is offered and works: Guide for the 1 px line.
    let role = fixes(
        &doc,
        &t,
        "L03",
        1,
        Fix::UseRole(RoleChoice::Line(LineRole::Guide)),
    );
    let fill_role = fixes(
        &doc,
        &t,
        "L03",
        3,
        Fix::UseRole(RoleChoice::Fill(FillRole::Shade)),
    );
    let ok = a == (true, true)
        && b == (true, true)
        && c == (true, true)
        && d == (true, true)
        && n1 == 187
        && n2 == 119
        && role == (true, true)
        && fill_role == (true, true);
    println!(
        "CHECK crate=lint name=fix_nearest_gray_and_role result={} line={a:?} banded={b:?} fill={c:?} text={d:?} nearest_to_230={n1} nearest_to_120={n2} guide_role={role:?} shade_role={fill_role:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn move_inside_safe_area_clears_the_zones_and_stays_on_the_page() {
    // Under the left toolbar on the Manta (zone x 0..99 px).
    let boxed = obj(
        1,
        "logo",
        Frame::new(mm(40.0), mm(300.0), mm(200.0), mm(100.0)),
        Style::Role(LineRole::Structure),
        ObjectKind::Rect(RectShape::default()),
    );
    let doc = page(vec![boxed]);
    let t = manta();
    let r = fixes(&doc, &t, "L06", 1, Fix::MoveInsideSafeArea);
    let f = run(&doc, &t);
    let e = plan(
        &doc,
        &manta(),
        &t,
        of(&f, "L06", 1).unwrap(),
        Fix::MoveInsideSafeArea,
    )
    .unwrap();
    let moved = match &e[0] {
        Edit::MovePage { dx_px, dy_px, .. } => (*dx_px, *dy_px),
        _ => (0.0, 0.0),
    };
    // It is the smallest of the ways out: right to the toolbar's edge (about 59 px) or down/up past
    // the zone; and nothing was asked of a box that is already clear.
    let ok = r == (true, true) && moved.0 > 0.0 && moved.0 < 120.0 && moved.1 == 0.0;
    println!(
        "CHECK crate=lint name=fix_move_inside_safe_area result={} offered_and_clears={r:?} moved_px={moved:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn use_the_smallest_text_size_fixes_l07() {
    let small = obj(
        1,
        "fine print",
        Frame::new(20.0, 30.0, 60.0, 8.0),
        Style::Role(LineRole::Text),
        ObjectKind::Text(TextBox {
            content: "tiny".into(),
            family: ATKINSON.into(),
            size_pt: 4.0,
            ..TextBox::default()
        }),
    );
    let doc = page(vec![small]);
    let t = manta();
    let r = fixes(&doc, &t, "L07", 1, Fix::UseMinTextSize);
    let label = Strings_english(Fix::UseMinTextSize.label(&doc, &t));
    let ok = r == (true, true) && label == "Use 6 pt";
    println!(
        "CHECK crate=lint name=fix_use_min_text_size result={} offered_and_clears={r:?} label={label:?}",
        pass(ok)
    );
    assert!(ok);
}

#[allow(non_snake_case)]
fn Strings_english(m: quadrule_i18n::Message) -> String {
    quadrule_i18n::Strings::english().try_format(&m).unwrap()
}

#[test]
fn snap_to_rhythm_one_or_all_and_free_placement_is_left_alone() {
    // The Manta rhythm: origin 94 px, step 71 px.
    let at = |k: f64| mm(94.0 + 71.0 * k);
    let line = |id: u128, x: f64, y: f64| {
        obj(
            id,
            &format!("o{id}"),
            Frame::new(x, y, 40.0, 0.0),
            Style::Role(LineRole::Guide),
            ObjectKind::Line(Line::default()),
        )
    };
    let on = line(1, at(2.0), at(3.0));
    let off1 = line(2, at(2.0) + 1.7, at(4.0));
    let off2 = line(3, at(5.0), at(6.0) + 2.3);
    let mut free = line(4, at(2.0) + 2.2, at(8.0) + 1.1);
    free.free_placement = true;
    let doc = page(vec![on, off1, off2, free]);
    let t = manta();
    let f = run(&doc, &t);
    let l16: Vec<u128> = f
        .iter()
        .filter(|x| x.rule == "L16")
        .map(|x| x.object.unwrap().as_u128())
        .collect();
    let one = fixes(&doc, &t, "L16", 2, Fix::SnapToRhythm);
    // Snap all: fixes both off-rhythm objects at once and leaves the free one where it is.
    let all = plan_snap_all(&doc, &t, &f).unwrap();
    let after = run(&apply(&doc, &t, &all), &t);
    let none_left = !after.iter().any(|x| x.rule == "L16");
    let free_still = doc.object(Uuid::from_u128(4)).unwrap().frame
        == apply(&doc, &t, &all)
            .object(Uuid::from_u128(4))
            .unwrap()
            .frame;
    let ok = l16 == vec![2, 3] && one == (true, true) && all.len() == 2 && none_left && free_still;
    println!(
        "CHECK crate=lint name=fix_snap_to_rhythm result={} l16_on={l16:?} snap_one={one:?} snap_all_edits={} none_left={none_left} free_left_alone={free_still}",
        pass(ok),
        all.len()
    );
    assert!(ok);
}

#[test]
fn bind_to_rhythm_and_center_the_rhythm_origin_fix_l17_and_l18() {
    let mut pr = PatternRegion::new(Pattern::Grid);
    pr.bound = false;
    pr.h_spacing_mm = Some(5.3);
    pr.v_spacing_mm = Some(5.3);
    let region = obj(
        1,
        "own grid",
        Frame::new(mm(94.0 + 71.0), mm(94.0 + 71.0 * 2.0), 80.0, 60.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Pattern(pr),
    );
    let mut doc = page(vec![region]);
    // A repeat with a spacing of its own that is not a clean multiple of the rhythm.
    let src = obj(
        2,
        "src",
        Frame::new(mm(94.0 + 71.0 * 8.0), mm(94.0 + 71.0), 6.0, 0.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Line(Line::default()),
    );
    let mut rep = quadrule_model::repeat::RepeatShape::new(src);
    rep.h_spacing = Spacing::Mm(7.7);
    let repeat = obj(
        3,
        "own pitch",
        Frame::new(mm(94.0 + 71.0 * 8.0), mm(94.0 + 71.0), 6.0, 0.0),
        Style::Role(LineRole::Guide),
        ObjectKind::Repeat(rep),
    );
    doc.layers[0].objects.push(repeat);
    let t = manta();
    let bind = fixes(&doc, &t, "L17", 1, Fix::BindToRhythm);
    let bind_repeat = fixes(&doc, &t, "L17", 3, Fix::BindToRhythm);
    // L18: a margin that leaves an uneven strip, on the margin-corner origin.
    let mut uneven = page(vec![]);
    uneven.rhythm.origin = RhythmOrigin::MarginCorner;
    uneven.page.margins_mm = Margins {
        top: 8.0,
        right: 30.0,
        bottom: 8.0,
        left: 8.0,
    };
    let f = run(&uneven, &t);
    let l18 = f.iter().find(|x| x.rule == "L18");
    let center = match l18 {
        Some(found) => {
            let offered = offered(&uneven, &t, &t, found);
            let e = plan(&uneven, &t, &t, found, Fix::CenterRhythmOrigin).unwrap();
            let after = run(&apply(&uneven, &t, &e), &t);
            (
                offered.contains(&Fix::CenterRhythmOrigin),
                !after.iter().any(|x| x.rule == "L18"),
            )
        }
        None => (false, false),
    };
    let ok = bind == (true, true) && bind_repeat == (true, true) && center == (true, true);
    println!(
        "CHECK crate=lint name=fix_bind_and_center result={} bind_region={bind:?} bind_repeat={bind_repeat:?} center_origin={center:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_rules_without_a_button_offer_no_fix() {
    // Every finding the lint page and the sample make, on both devices: only the rules of Q6 have fixes.
    let mut with = std::collections::BTreeSet::new();
    let mut without = std::collections::BTreeSet::new();
    for doc in [lint_document(), slice_document()] {
        for t in [manta(), nomad()] {
            for f in run(&doc, &t) {
                let n = offered(&doc, &manta(), &t, &f).len();
                if n > 0 {
                    with.insert(f.rule);
                } else {
                    without.insert(f.rule);
                }
            }
        }
    }
    let buttonless = [
        "L08", "L09", "L10", "L11", "L13", "L14", "L15", "L19", "L20",
    ];
    let ok = buttonless.iter().all(|r| !with.contains(r))
        && with.contains("L03")
        && with.contains("L07");
    println!(
        "CHECK crate=lint name=fix_none_for_the_rest result={} rules_with_a_fix={with:?} rules_without={without:?}",
        pass(ok)
    );
    assert!(ok);
}
