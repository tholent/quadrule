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
        ("RectShape", unknown_field_survives(&RectShape::default())),
        (
            "EllipseShape",
            unknown_field_survives(&EllipseShape::default()),
        ),
        (
            "PolygonShape",
            unknown_field_survives(&PolygonShape::star(5)),
        ),
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

#[test]
fn new_object_fields_stay_out_of_old_objects() {
    // Format 0.5 (rung 6a): an object with no fill, a solid stroke and butt caps writes none of the
    // new fields, so a 0.4 object saves exactly as before; one that has them keeps them.
    let doc = slice_document();
    let keys = |o: &Object| -> Vec<String> {
        let Value::Map(m) = to_value(o) else {
            return vec![];
        };
        m.iter()
            .filter_map(|(k, _)| k.as_text().map(String::from))
            .collect()
    };
    let plain = doc.layers[1].objects[1].clone();
    let plain_keys = keys(&plain);
    let mut shaped = plain.clone();
    shaped.fill = Some(Fill::Role(FillRole::Shade));
    shaped.dash = DashStyle::Custom {
        dash_mm: 1.5,
        gap_mm: 1.0,
    };
    shaped.cap = LineCap::Round;
    shaped.style = Style::None;
    let back: Object = to_value(&shaped).deserialized().unwrap();
    let ok = !plain_keys
        .iter()
        .any(|k| ["fill", "dash", "cap"].contains(&k.as_str()))
        && back == shaped
        && keys(&shaped)
            .iter()
            .filter(|k| ["fill", "dash", "cap"].contains(&k.as_str()))
            .count()
            == 3;
    println!(
        "\nCHECK crate=model name=new_fields_default_out result={} plain_keys={} shaped_roundtrips={}",
        if ok { "pass" } else { "fail" },
        plain_keys.len(),
        back == shaped
    );
    assert!(ok);
}

#[test]
fn polygon_numbers_stay_in_range() {
    // A file holding 2 sides, 200 sides or a wild star still draws: the numbers are clamped.
    let p = |sides, star| PolygonShape {
        sides,
        star,
        extra: Extra::new(),
    };
    let ok = p(2, None).sides_clamped() == POLYGON_SIDES_MIN
        && p(200, None).sides_clamped() == POLYGON_SIDES_MAX
        && p(5, Some(7.0)).star_clamped() == Some(STAR_INNER_MAX)
        && p(5, Some(-1.0)).star_clamped() == Some(STAR_INNER_MIN)
        && p(5, Some(f64::NAN)).star_clamped() == Some(STAR_INNER_DEFAULT)
        && p(5, None).star_clamped().is_none()
        && PolygonShape::star(5).star == Some(0.5)
        && PolygonShape::polygon(6).sides == 6;
    println!(
        "\nCHECK crate=model name=polygon_ranges result={}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn rotation_opacity_and_line_direction_are_left_out_of_the_file_at_their_defaults() {
    // Format 0.6 (rung 6b): a plain object writes none of the three new keys, so a file that
    // does not use them is what a 0.5 build wrote; set, they round-trip.
    let keys = |v: &Value| -> Vec<String> {
        let mut out = Vec::new();
        fn walk(v: &Value, out: &mut Vec<String>) {
            match v {
                Value::Map(m) => {
                    for (k, x) in m {
                        if let Some(k) = k.as_text() {
                            out.push(k.to_string());
                        }
                        walk(x, out);
                    }
                }
                Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
                _ => {}
            }
        }
        walk(v, &mut out);
        out
    };
    let doc = shapes_document();
    let plain_keys = keys(&to_value(&doc));
    let plain = !plain_keys
        .iter()
        .any(|k| ["rotation_deg", "opacity_pct", "direction"].contains(&k.as_str()));
    let mut turned = doc.clone();
    turned.layers[1].objects[0].rotation_deg = 30.0;
    turned.layers[1].objects[0].opacity_pct = 40;
    let line = turned.layers[1]
        .objects
        .iter()
        .position(|o| matches!(o.kind, ObjectKind::Line(_)))
        .unwrap();
    turned.layers[1].objects[line].kind = ObjectKind::Line(Line {
        direction: LineDir::Up,
        ..Line::default()
    });
    let back: Document = to_value(&turned).deserialized().unwrap();
    let round = back == turned;
    let set_keys = keys(&to_value(&turned));
    let written = ["rotation_deg", "opacity_pct", "direction"]
        .iter()
        .all(|k| set_keys.iter().any(|x| x == k));
    // A file with none of them reads as 0, 100 and top-left to bottom-right.
    let o = &doc.layers[1].objects[0];
    let defaults = o.rotation_deg == 0.0 && o.opacity_pct == 100;
    let ok = plain && round && written && defaults;
    println!(
        "\nCHECK crate=model name=format_060_fields result={} defaults_left_out={plain} set_values_round_trip={round} set_values_written={written} read_as_defaults={defaults}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn only_lines_and_shapes_rotate_and_angles_are_normalised() {
    let doc = shapes_document();
    let mut rect = doc.layers[1].objects[0].clone();
    rect.rotation_deg = -90.0;
    let mut text = slice_document().layers[1].objects[0].clone();
    text.rotation_deg = 45.0;
    let mut pattern = slice_document().layers[0].objects[0].clone();
    pattern.rotation_deg = 45.0;
    let norm = normalize_deg(-90.0) == 270.0
        && normalize_deg(360.0) == 0.0
        && normalize_deg(725.0) == 5.0
        && normalize_deg(f64::NAN) == 0.0
        && normalize_deg(-1e-20) == 0.0;
    let kinds = rect.rotation() == 270.0
        && text.rotation() == 0.0
        && pattern.rotation() == 0.0
        && rect.kind.can_rotate()
        && !text.kind.can_rotate();
    let mut see = rect.clone();
    see.opacity_pct = 250;
    let clamped = see.opacity() == 100;
    let ok = norm && kinds && clamped;
    println!(
        "\nCHECK crate=model name=rotation_rules result={} normalised={norm} only_lines_and_shapes_turn={kinds} opacity_clamped={clamped}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn a_placement_change_is_one_undoable_step() {
    use quadrule_model::edit::{Placement, set_placements};
    let mut doc = shapes_document();
    let before = doc.clone();
    let o = doc.layers[1].objects[0].clone();
    let line_id = doc
        .objects()
        .find(|o| matches!(o.kind, ObjectKind::Line(_)))
        .unwrap()
        .id;
    let places = vec![
        (
            o.id,
            Placement {
                rotation_deg: 33.0,
                ..Placement::of(&o)
            },
        ),
        (
            line_id,
            Placement {
                dir: Some(LineDir::Up),
                ..Placement::of(doc.object(line_id).unwrap())
            },
        ),
    ];
    let cmd = set_placements(&doc, &places);
    let undo = doc.apply(cmd).unwrap();
    let rotated = doc.object(o.id).unwrap().rotation_deg == 33.0;
    let up = matches!(&doc.object(line_id).unwrap().kind, ObjectKind::Line(l) if l.direction == LineDir::Up);
    doc.apply(undo).unwrap();
    let undone = doc == before;
    // Nothing changed: an empty step.
    let same = quadrule_model::edit::is_empty(&set_placements(&doc, &[(o.id, Placement::of(&o))]));
    let ok = rotated && up && undone && same;
    println!(
        "\nCHECK crate=model name=placement_command result={} rotated={rotated} line_up={up} undone={undone} no_change_is_empty={same}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
