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
fn floating_toolbar_is_info_only() {
    let mut doc = slice_document();
    // A line through the middle of the Manta, where the floating toolbar starts (x 848-964).
    doc.layers[0].objects = vec![line(
        30,
        Frame::new(60.0, 60.0, 40.0, 0.0),
        Style::Role(LineRole::Guide),
    )];
    doc.layers[1].objects.clear();
    let f = run(&doc, &builtin(MANTA_ID).unwrap());
    let l06: Vec<_> = f.iter().filter(|x| x.rule == "L06").collect();
    let ok = !l06.is_empty()
        && l06.iter().all(|x| x.severity == Severity::Info)
        && l06.iter().all(|x| text(x).contains("floating toolbar"));
    println!(
        "CHECK crate=lint name=floating_toolbar_info result={} l06={}",
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
    // the footers only; the right and floating toolbars are not mentioned.
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
