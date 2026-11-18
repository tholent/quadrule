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

//! The border of a pattern region (docs/bbq/2026-11-18-pattern-borders.md; format 0.17): left out of a file when
//! there is none, every field with a default, unknown fields kept. Each test prints one structured `CHECK` line (run
//! with `-- --nocapture`).

use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn cbor<T: serde::Serialize>(v: &T) -> ciborium::Value {
    ciborium::Value::serialized(v).unwrap()
}

fn keys(v: &ciborium::Value) -> Vec<String> {
    match v {
        ciborium::Value::Map(m) => m
            .iter()
            .filter_map(|(k, _)| k.as_text().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

#[test]
fn a_region_with_no_border_writes_no_border_field_and_reads_back() {
    let pr = PatternRegion::new(Pattern::Grid);
    let k = keys(&cbor(&pr));
    let none = !k.contains(&"border".to_string());
    // A pattern written the old way reads as no border.
    let old = ciborium::Value::Map(vec![(
        ciborium::Value::Text("pattern".into()),
        ciborium::Value::Text("Grid".into()),
    )]);
    let back: PatternRegion = old.deserialized().unwrap();
    let read_none = back.border.is_none();
    let doc = slice_document();
    let doc_back: Document = cbor(&doc).deserialized().unwrap();
    let ok = none && read_none && doc_back == doc;
    println!(
        "\nCHECK crate=model name=border_absent_by_default result={} no_key={none} old_reads_none={read_none}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_border_writes_only_what_differs_and_every_field_has_a_default() {
    let mut pr = PatternRegion::new(Pattern::Dots);
    pr.border = Some(Border::new());
    let v = cbor(&pr);
    let border_keys = match &v {
        ciborium::Value::Map(m) => m
            .iter()
            .find(|(k, _)| k.as_text() == Some("border"))
            .map(|(_, b)| keys(b))
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let bare = border_keys.is_empty();
    let back: PatternRegion = v.deserialized().unwrap();
    let default_back = back.border == Some(Border::new());
    // Everything set round-trips.
    let mut b = Border::new();
    b.sides = BorderSides {
        top: true,
        right: false,
        bottom: true,
        left: false,
    };
    b.style = Style::Custom {
        width_mm: 0.4,
        gray: 64,
    };
    b.corner_radius_mm = 3.5;
    pr.border = Some(b.clone());
    let set_back: PatternRegion = cbor(&pr).deserialized().unwrap();
    let round_trip = set_back.border == Some(b);
    // A newer file's unknown field inside a border is kept.
    let mut with_extra = cbor(&pr);
    if let ciborium::Value::Map(m) = &mut with_extra
        && let Some((_, ciborium::Value::Map(bm))) =
            m.iter_mut().find(|(k, _)| k.as_text() == Some("border"))
    {
        bm.push((
            ciborium::Value::Text("x_border".into()),
            ciborium::Value::Integer(7.into()),
        ));
    }
    let extra: PatternRegion = with_extra.deserialized().unwrap();
    let kept = extra
        .border
        .as_ref()
        .is_some_and(|b| b.extra.contains_key("x_border"))
        && keys(&cbor(&extra)).contains(&"border".to_string());
    let ok = bare && default_back && round_trip && kept;
    println!(
        "\nCHECK crate=model name=border_fields result={} default_border_writes_no_inner_key={bare} default_reads_back={default_back} set_round_trip={round_trip} unknown_kept={kept}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_radius_that_counts_is_never_negative_and_not_a_number_is_square() {
    let at = |mm: f64| {
        let mut b = Border::new();
        b.corner_radius_mm = mm;
        b.radius_mm()
    };
    let ok = at(3.0) == 3.0
        && at(-2.0) == 0.0
        && at(f64::NAN) == 0.0
        && at(f64::INFINITY) == 0.0
        && Border::new().sides.any()
        && !BorderSides {
            top: false,
            right: false,
            bottom: false,
            left: false,
        }
        .any();
    println!(
        "\nCHECK crate=model name=border_radius_and_sides result={}",
        pass(ok)
    );
    assert!(ok);
}
