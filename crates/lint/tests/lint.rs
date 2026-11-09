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
        .map(|x| x.message.clone())
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
    let off: Vec<&str> = f
        .iter()
        .filter(|x| x.rule == "L10")
        .map(|x| x.message.as_str())
        .collect();
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
        && l06.iter().all(|x| x.message.contains("floating toolbar"));
    println!(
        "CHECK crate=lint name=floating_toolbar_info result={} l06={}",
        pass(ok),
        l06.len()
    );
    assert!(ok);
}
