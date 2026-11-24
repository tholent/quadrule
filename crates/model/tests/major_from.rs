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

//! `PatternRegion::major_rows_from` and `major_columns_from` (ADR-023 as amended 2026-11-23; format 0.20): a new
//! region starts both at Top, an isometric one at Rhythm origin; switching a region to isometric while an axis is at
//! Top sets that axis to Rhythm origin; a region without the fields reads both as Rhythm origin, and each field is
//! written only when it is not that, on its own. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn has_key(pr: &PatternRegion, key: &str) -> bool {
    let v = ciborium::Value::serialized(pr).unwrap();
    matches!(&v, ciborium::Value::Map(m) if m.iter().any(|(k, _)| k.as_text() == Some(key)))
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
        tops &= pr.major_rows_from == want
            && pr.major_columns_from == want
            && pr.major_rows_origin() == want
            && pr.major_columns_origin() == want;
    }
    // Switching to isometric sets an axis at Top to Rhythm origin and keeps one at Centre; back to a grid, each
    // stays where it is.
    let mut o = slice_document().layers[0].objects[0].clone();
    let mut grid = PatternRegion::new(Pattern::Grid);
    grid.major_columns_from = MajorFrom::Centre;
    o.kind = ObjectKind::Pattern(grid);
    let from = |o: &Object| match &o.kind {
        ObjectKind::Pattern(p) => (p.major_rows_from, p.major_columns_from),
        _ => panic!("not a pattern"),
    };
    o.set_pattern_kind(PatternKind::Isometric);
    let switched = from(&o) == (MajorFrom::RhythmOrigin, MajorFrom::Centre);
    o.set_pattern_kind(PatternKind::Grid);
    let back = from(&o) == (MajorFrom::RhythmOrigin, MajorFrom::Centre);
    // An isometric region a file sets to Top draws from the rhythm origin, on each axis.
    let iso_top = PatternRegion {
        major_rows_from: MajorFrom::Top,
        major_columns_from: MajorFrom::Top,
        ..PatternRegion::new(PatternKind::Isometric.default_pattern())
    };
    let iso_draws_origin = iso_top.major_rows_origin() == MajorFrom::RhythmOrigin
        && iso_top.major_columns_origin() == MajorFrom::RhythmOrigin;
    // Grid, Dots and Isometric have major columns; Lines and Cornell rows only.
    let columns = PatternKind::ALL
        .iter()
        .filter(|k| k.default_pattern().has_major_columns())
        .copied()
        .collect::<Vec<_>>()
        == [PatternKind::Dots, PatternKind::Grid, PatternKind::Isometric];
    let ok = tops && switched && back && iso_draws_origin && columns;
    println!(
        "\nCHECK crate=model name=major_from_new_regions result={} new_top_iso_origin={tops} to_isometric_top_to_origin_centre_kept={switched} back_stays={back} iso_top_draws_origin={iso_draws_origin} columns_on_grid_dots_iso={columns}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_file_without_the_field_reads_as_the_rhythm_origin_and_only_others_are_written() {
    // The 0.19 shape: a region with neither field.
    let old = ciborium::Value::Map(vec![(
        ciborium::Value::Text("pattern".into()),
        ciborium::Value::Text("Lines".into()),
    )]);
    let r: PatternRegion = old.deserialized().unwrap();
    let reads_origin = r.major_rows_from == MajorFrom::RhythmOrigin
        && r.major_columns_from == MajorFrom::RhythmOrigin
        && r.extra.is_empty();
    let all = [MajorFrom::Top, MajorFrom::Centre, MajorFrom::RhythmOrigin];
    let mut written = true;
    let mut round_trip = true;
    let mut pairs = 0;
    for rows in all {
        for cols in all {
            let pr = PatternRegion {
                major_rows_from: rows,
                major_columns_from: cols,
                ..PatternRegion::new(Pattern::Grid)
            };
            written &= has_key(&pr, "major_rows_from") == !rows.is_rhythm_origin()
                && has_key(&pr, "major_columns_from") == !cols.is_rhythm_origin();
            let back: PatternRegion = ciborium::Value::serialized(&pr)
                .unwrap()
                .deserialized()
                .unwrap();
            round_trip &= back == pr;
            pairs += 1;
        }
    }
    let ok = reads_origin && written && round_trip && pairs == 9;
    println!(
        "\nCHECK crate=model name=major_from_file result={} absent_reads_origin={reads_origin} each_written_unless_origin={written} round_trip={round_trip} pairs={pairs}",
        pass(ok)
    );
    assert!(ok);
}
