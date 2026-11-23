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

//! `PatternRegion::major_from` (ADR-023; format 0.20): a new region starts at Top, an isometric one at Rhythm origin;
//! switching a region to isometric while it is at Top sets Rhythm origin; a region without the field reads as Rhythm
//! origin, and the field is written only when it is not that. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn has_key(pr: &PatternRegion) -> bool {
    let v = ciborium::Value::serialized(pr).unwrap();
    matches!(&v, ciborium::Value::Map(m) if m.iter().any(|(k, _)| k.as_text() == Some("major_from")))
}

#[test]
fn a_new_region_starts_at_top_and_isometric_at_the_rhythm_origin() {
    let mut tops = true;
    for kind in PatternKind::ALL {
        let pr = PatternRegion::new(kind.default_pattern());
        let want = if kind == PatternKind::Isometric {
            MajorFrom::RhythmOrigin
        } else {
            MajorFrom::Top
        };
        tops &= pr.major_from == want && pr.major_origin() == want;
    }
    // Switching to isometric while at Top sets Rhythm origin; Centre is kept; back to a grid stays where it is.
    let mut o = slice_document().layers[0].objects[0].clone();
    o.kind = ObjectKind::Pattern(PatternRegion::new(Pattern::Grid));
    let from = |o: &Object| match &o.kind {
        ObjectKind::Pattern(p) => p.major_from,
        _ => panic!("not a pattern"),
    };
    o.set_pattern_kind(PatternKind::Isometric);
    let switched = from(&o) == MajorFrom::RhythmOrigin;
    o.set_pattern_kind(PatternKind::Grid);
    let back = from(&o) == MajorFrom::RhythmOrigin;
    if let ObjectKind::Pattern(p) = &mut o.kind {
        p.major_from = MajorFrom::Centre;
    }
    o.set_pattern_kind(PatternKind::Isometric);
    let centre_kept = from(&o) == MajorFrom::Centre;
    // An isometric region a file sets to Top draws from the rhythm origin.
    let iso_top = PatternRegion {
        major_from: MajorFrom::Top,
        ..PatternRegion::new(PatternKind::Isometric.default_pattern())
    };
    let iso_draws_origin = iso_top.major_origin() == MajorFrom::RhythmOrigin;
    let ok = tops && switched && back && centre_kept && iso_draws_origin;
    println!(
        "\nCHECK crate=model name=major_from_new_regions result={} new_top_iso_origin={tops} to_isometric_sets_origin={switched} back_stays={back} centre_kept={centre_kept} iso_top_draws_origin={iso_draws_origin}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_file_without_the_field_reads_as_the_rhythm_origin_and_only_others_are_written() {
    // The 0.19 shape: a region with no `major_from`.
    let old = ciborium::Value::Map(vec![(
        ciborium::Value::Text("pattern".into()),
        ciborium::Value::Text("Lines".into()),
    )]);
    let r: PatternRegion = old.deserialized().unwrap();
    let reads_origin = r.major_from == MajorFrom::RhythmOrigin && r.extra.is_empty();
    let mut written = true;
    let mut round_trip = true;
    for from in [MajorFrom::Top, MajorFrom::Centre, MajorFrom::RhythmOrigin] {
        let pr = PatternRegion {
            major_from: from,
            ..PatternRegion::new(Pattern::Grid)
        };
        written &= has_key(&pr) == (from != MajorFrom::RhythmOrigin);
        let back: PatternRegion = ciborium::Value::serialized(&pr)
            .unwrap()
            .deserialized()
            .unwrap();
        round_trip &= back == pr;
    }
    let ok = reads_origin && written && round_trip;
    println!(
        "\nCHECK crate=model name=major_from_file result={} absent_reads_origin={reads_origin} written_unless_origin={written} round_trip={round_trip}",
        pass(ok)
    );
    assert!(ok);
}
