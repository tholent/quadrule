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

//! The rung 9 pattern model (docs/bbq/2026-11-13-rung-9-pattern-library.md): the new patterns and
//! the new rhythm and region fields, each with a default, so an old document reads as it was and
//! writes the same fields it did. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn cbor<T: serde::Serialize>(v: &T) -> ciborium::Value {
    ciborium::Value::serialized(v).unwrap()
}

fn keys(v: &ciborium::Value, out: &mut Vec<String>) {
    match v {
        ciborium::Value::Map(m) => {
            for (k, x) in m {
                if let Some(k) = k.as_text() {
                    out.push(k.to_string());
                }
                keys(x, out);
            }
        }
        ciborium::Value::Array(a) => a.iter().for_each(|x| keys(x, out)),
        _ => {}
    }
}

#[test]
fn an_old_document_writes_none_of_the_new_fields() {
    let doc = slice_document();
    let mut k = Vec::new();
    keys(&cbor(&doc), &mut k);
    let new = [
        "rhythm_type",
        "side_mm",
        "major_every",
        "bound",
        "h_spacing_mm",
        "v_spacing_mm",
        "major",
        "major_role",
    ];
    let none = !k.iter().any(|x| new.contains(&x.as_str()));
    // A pattern written the old way is a bare name, and reads back the same.
    let back: Document = cbor(&doc).deserialized().unwrap();
    let same = back == doc;
    let ok = none && same && !k.is_empty();
    println!(
        "\nCHECK crate=model name=old_document_writes_no_new_field result={} no_new_key={none} round_trip={same}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_region_missing_the_new_fields_reads_with_their_defaults() {
    // The 0.8 shape of a region: only the pattern (a bare name).
    let old = ciborium::Value::Map(vec![(
        ciborium::Value::Text("pattern".into()),
        ciborium::Value::Text("Grid".into()),
    )]);
    let r: PatternRegion = old.deserialized().unwrap();
    let defaults = r == PatternRegion::new(Pattern::Grid);
    // An isometric pattern with no settings reads as lines; a hex one as flat top, one unit high.
    let iso: Pattern = ciborium::Value::Map(vec![(
        ciborium::Value::Text("Isometric".into()),
        ciborium::Value::Map(vec![]),
    )])
    .deserialized()
    .unwrap();
    let hex: Pattern = ciborium::Value::Map(vec![(
        ciborium::Value::Text("Hex".into()),
        ciborium::Value::Map(vec![]),
    )])
    .deserialized()
    .unwrap();
    let staff: Pattern = ciborium::Value::Map(vec![(
        ciborium::Value::Text("MusicStaff".into()),
        ciborium::Value::Map(vec![]),
    )])
    .deserialized()
    .unwrap();
    let iso_ok = iso == PatternKind::Isometric.default_pattern();
    let hex_ok = hex == PatternKind::Hex.default_pattern();
    let staff_ok = staff == PatternKind::MusicStaff.default_pattern();
    let ok = defaults && iso_ok && hex_ok && staff_ok;
    println!(
        "\nCHECK crate=model name=new_fields_default result={} region={defaults} isometric={iso_ok} hex={hex_ok} staff={staff_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn every_new_setting_round_trips() {
    let mut doc = slice_document();
    doc.rhythm.rhythm_type = RhythmType::Isometric;
    doc.rhythm.side_mm = Some(6.0);
    doc.rhythm.major_every = Some(5);
    let mut region = PatternRegion::new(Pattern::MusicStaff {
        gap_mm: 1.5,
        pitch_units: 4,
        grand: true,
    });
    region.bound = false;
    region.h_spacing_mm = Some(3.0);
    region.v_spacing_mm = Some(4.5);
    region.major = MajorLines::Every(4);
    region.major_role = LineRole::Emphasis;
    doc.layers[0].objects[0].kind = ObjectKind::Pattern(region.clone());
    let back: Document = cbor(&doc).deserialized().unwrap();
    let same = back == doc;
    let json = serde_json::to_string(&doc).unwrap();
    let back_json: Document = serde_json::from_str(&json).unwrap();
    let same_json = back_json == doc;
    let ok = same && same_json;
    println!(
        "\nCHECK crate=model name=new_settings_round_trip result={} cbor={same} json={same_json}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_isometric_rhythm_steps_are_half_a_side_across_and_a_row_down() {
    let mut r = slice_document().rhythm;
    let rect = r.steps_mm();
    r.rhythm_type = RhythmType::Isometric;
    r.side_mm = Some(5.0);
    let (across, down) = r.steps_mm();
    let ok_iso = (across - 2.5).abs() < 1e-12 && (down - 5.0 * 3f64.sqrt() / 2.0).abs() < 1e-12;
    let ok_rect = rect == (6.0, 6.0);
    // Without a side, the default; a nonsense side reads as the default too.
    r.side_mm = None;
    let dflt = r.steps_mm().0 == ISOMETRIC_SIDE_DEFAULT_MM / 2.0;
    r.side_mm = Some(-2.0);
    let bad = r.steps_mm().0 == ISOMETRIC_SIDE_DEFAULT_MM / 2.0;
    let ok = ok_iso && ok_rect && dflt && bad;
    println!(
        "\nCHECK crate=model name=isometric_rhythm_steps result={} across_mm={across:.4} down_mm={down:.4} rectangular_unchanged={ok_rect} default_side={dflt} bad_side_default={bad}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn major_lines_follow_the_rhythm_unless_the_region_says_otherwise() {
    let mut r = slice_document().rhythm;
    let mut region = PatternRegion::new(Pattern::Grid);
    let off_by_default = region.major_every(&r).is_none();
    r.major_every = Some(5);
    let follows = region.major_every(&r) == Some(5);
    region.major = MajorLines::Off;
    let off = region.major_every(&r).is_none();
    region.major = MajorLines::Every(3);
    let own = region.major_every(&r) == Some(3);
    // Hex and the music staff have none, whatever is asked.
    let mut hex = PatternRegion::new(PatternKind::Hex.default_pattern());
    hex.major = MajorLines::Every(3);
    let hex_none = hex.major_every(&r).is_none();
    // Out of range is off.
    r.major_every = Some(1);
    let follow_bad = PatternRegion::new(Pattern::Lines).major_every(&r).is_none();
    let ok = off_by_default && follows && off && own && hex_none && follow_bad;
    println!(
        "\nCHECK crate=model name=major_lines_resolve result={} off_by_default={off_by_default} follows_rhythm={follows} region_off={off} region_own_n={own} hex_none={hex_none} n_of_1_is_off={follow_bad}",
        pass(ok)
    );
    assert!(ok);
}
