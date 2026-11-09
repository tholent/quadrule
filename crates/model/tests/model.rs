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

//! Model checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use ciborium::Value;
use quadrule_model::outlines::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

fn to_value<T: Serialize>(v: &T) -> Value {
    Value::serialized(v).expect("to cbor value")
}

/// Adds an unknown field at the record's top level, reads it back as `T`, writes it again and
/// returns whether the field is still there with the same value.
fn unknown_field_survives<T: Serialize + DeserializeOwned>(record: &T) -> bool {
    let mut v = to_value(record);
    let unknown = Value::Map(vec![(
        Value::Text("nested".into()),
        Value::Integer(7.into()),
    )]);
    let Value::Map(m) = &mut v else { return false };
    m.push((Value::Text("from_a_newer_version".into()), unknown.clone()));
    let read: T = match v.deserialized() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let Value::Map(again) = to_value(&read) else {
        return false;
    };
    again
        .iter()
        .any(|(k, val)| k.as_text() == Some("from_a_newer_version") && *val == unknown)
}

#[test]
fn unknown_fields_survive_per_record_type() {
    let doc = slice_document();
    let obj = doc.layers[1].objects[0].clone();
    let ObjectKind::Text(tb) = obj.kind.clone() else {
        panic!("title is text")
    };
    let ObjectKind::Pattern(pr) = doc.layers[0].objects[0].kind.clone() else {
        panic!("dots is a pattern")
    };
    let shape = GlyphShape {
        verbs: "MLZ".into(),
        coords: vec![0, 0, 2, 2],
    };
    let table = GlyphTable {
        face: FaceKey {
            family: ATKINSON.into(),
            weight: 400,
        },
        units_per_em: 1000,
        glyphs: BTreeMap::from([(5, shape)]),
        extra: Extra::new(),
    };
    let text = TextOutlines {
        object_id: ids::TITLE,
        size_mm: 4.2,
        height_mm: 5.3,
        glyphs: vec![PlacedGlyph(0, 5, 1.0, 2.0)],
        extra: Extra::new(),
    };
    let outlines = ProfileOutlines {
        profile_id: MANTA_ID.into(),
        tables: vec![table.clone()],
        texts: vec![text.clone()],
        extra: Extra::new(),
    };

    let results = [
        ("Document", unknown_field_survives(&doc)),
        ("Metadata", unknown_field_survives(&doc.metadata)),
        ("ProfileContext", unknown_field_survives(&doc.profiles[0])),
        ("PageSettings", unknown_field_survives(&doc.page)),
        ("Rhythm", unknown_field_survives(&doc.rhythm)),
        ("Layer", unknown_field_survives(&doc.layers[0])),
        ("Object", unknown_field_survives(&obj)),
        ("Line", unknown_field_survives(&Line::default())),
        ("TextBox", unknown_field_survives(&tb)),
        ("PatternRegion", unknown_field_survives(&pr)),
        ("ProfileOutlines", unknown_field_survives(&outlines)),
        ("GlyphTable", unknown_field_survives(&table)),
        ("TextOutlines", unknown_field_survives(&text)),
    ];
    let failed: Vec<&str> = results
        .iter()
        .filter(|(_, ok)| !ok)
        .map(|(n, _)| *n)
        .collect();
    println!(
        "CHECK crate=model name=unknown_fields_survive result={} types={} failed={:?}",
        if failed.is_empty() { "pass" } else { "fail" },
        results.len(),
        failed
    );
    assert!(failed.is_empty());
}

#[test]
fn slice_document_round_trips_through_cbor_and_json() {
    let doc = slice_document();
    let cbor: Document = to_value(&doc).deserialized().expect("cbor");
    let json: Document = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).expect("json");
    let ok = cbor == doc && json == doc;
    println!(
        "CHECK crate=model name=slice_document_round_trip result={} formats=cbor,json",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn glyph_shape_well_formed() {
    let good = GlyphShape {
        verbs: "MLQCZ".into(),
        coords: vec![0; 2 + 2 + 4 + 6],
    };
    let short = GlyphShape {
        verbs: "MQ".into(),
        coords: vec![0; 4],
    };
    let bad_verb = GlyphShape {
        verbs: "MX".into(),
        coords: vec![0; 2],
    };
    let ok = good.is_well_formed() && !short.is_well_formed() && !bad_verb.is_well_formed();
    println!(
        "CHECK crate=model name=glyph_shape_well_formed result={} cases=3",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
