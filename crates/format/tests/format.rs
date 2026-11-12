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

//! `.quad` checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use ciborium::Value;
use quadrule_format::migrate::{self, Step};
use quadrule_format::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_text::FontRegistry;
use semver::Version;
use std::io::{Cursor, Read, Write};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// The slice document's real outlines, one per profile.
fn outlines(doc: &Document) -> Vec<ProfileOutlines> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    doc.profile_ids()
        .iter()
        .map(|id| {
            quadrule_layout::lay_out_text(doc, &primary, &builtin(id).unwrap(), &mut reg)
                .unwrap()
                .outlines
        })
        .collect()
}

fn slice_file() -> QuadFile {
    let doc = slice_document();
    let o = outlines(&doc);
    let font = FontRef {
        family: ATKINSON.into(),
        bundled_id: Some("atkinson-hyperlegible-next".into()),
        version: None,
        extra: Extra::new(),
    };
    QuadFile::new(doc, o, vec![font])
}

/// Every entry of a .quad, uncompressed, in archive order.
fn entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    (0..z.len())
        .map(|i| {
            let mut f = z.by_index(i).unwrap();
            let mut b = Vec::new();
            f.read_to_end(&mut b).unwrap();
            (f.name().unwrap().to_string(), b)
        })
        .collect()
}

/// Writes a .quad from a manifest value and raw entries, as given.
fn zip_raw(manifest: &Value, files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    z.start_file("manifest.cbor", opts).unwrap();
    z.write_all(&to_cbor(manifest).unwrap()).unwrap();
    for (n, b) in files {
        z.start_file(n.as_str(), opts).unwrap();
        z.write_all(b).unwrap();
    }
    z.finish().unwrap().into_inner()
}

/// Writes a .quad from a manifest value and raw entries, fixing the manifest's checksums.
fn raw_quad(mut manifest: Value, files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let sums: Vec<(Value, Value)> = files
        .iter()
        .map(|(n, b)| {
            let info = Value::Map(vec![
                (Value::Text("sha256".into()), Value::Text(sha256_hex(b))),
                (
                    Value::Text("size".into()),
                    Value::Integer((b.len() as u64).into()),
                ),
            ]);
            (Value::Text(n.clone()), info)
        })
        .collect();
    *migrate::map_get_mut(&mut manifest, "entries").unwrap() = Value::Map(sums);
    zip_raw(&manifest, files)
}

/// Splits a .quad into its manifest (as a value) and its other entries.
fn split(bytes: &[u8]) -> (Value, Vec<(String, Vec<u8>)>) {
    let mut all = entries(bytes);
    let m = all.remove(0);
    assert_eq!(m.0, "manifest.cbor");
    (from_cbor(&m.1).unwrap(), all)
}

fn put(v: &mut Value, key: &str, val: Value) {
    if let Value::Map(m) = v {
        m.push((Value::Text(key.into()), val));
    }
}

fn has(v: &Value, key: &str) -> bool {
    matches!(v, Value::Map(m) if m.iter().any(|(k, _)| k.as_text() == Some(key)))
}

#[test]
fn round_trip() {
    let f = slice_file();
    let bytes = f.to_bytes("0.1.0-test").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let names: Vec<String> = entries(&bytes).into_iter().map(|(n, _)| n).collect();
    let ok = back.doc == f.doc
        && back.outlines == f.outlines
        && back.read_only.is_none()
        && back.manifest.saved_by == "0.1.0-test"
        && back.manifest.format_version == FORMAT_VERSION
        && back.manifest.profiles == [MANTA_ID, NOMAD_ID]
        && names
            == [
                "manifest.cbor",
                "scene.cbor",
                "outlines/supernote.a5x2.cbor",
                "outlines/supernote.a6x2.cbor",
            ];
    println!(
        "CHECK crate=format name=round_trip result={} bytes={} entries={names:?}",
        pass(ok),
        bytes.len()
    );
    assert!(ok);
}

#[test]
fn same_document_same_bytes() {
    // Compared across machines in CI by the printed sha256.
    let a = slice_file().to_bytes("0.1.0-test").unwrap();
    let b = slice_file().to_bytes("0.1.0-test").unwrap();
    let again = QuadFile::from_bytes(&a)
        .unwrap()
        .to_bytes("0.1.0-test")
        .unwrap();
    let ok = a == b && a == again;
    println!(
        "CHECK crate=format name=deterministic_bytes result={} sha256={} bytes={}",
        pass(ok),
        sha256_hex(&a),
        a.len()
    );
    assert!(ok);
}

#[test]
fn json_debug_round_trip() {
    let doc = slice_document();
    let json = to_debug_json(&doc).unwrap();
    let ok = from_debug_json(&json).unwrap() == doc;
    println!(
        "CHECK crate=format name=json_debug_round_trip result={} bytes={}",
        pass(ok),
        json.len()
    );
    assert!(ok);
}

#[test]
fn outlines_must_match_profiles() {
    // ADR-001 item 2: outlines are saved for exactly the document's profiles.
    let mut missing = slice_file();
    missing.outlines.remove(NOMAD_ID);
    let mut extra = slice_file();
    let mut stray = extra.outlines[MANTA_ID].clone();
    stray.profile_id = "supernote.a5x".into();
    extra.outlines.insert("supernote.a5x".into(), stray);
    let ok = matches!(
        missing.to_bytes("t"),
        Err(FormatError::OutlinesMismatch { .. })
    ) && matches!(
        extra.to_bytes("t"),
        Err(FormatError::OutlinesMismatch { .. })
    );
    println!(
        "CHECK crate=format name=outlines_match_profiles result={} cases=2",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn unknown_fields_survive_resave() {
    // Fields from a newer version, at five levels, survive open → save → open (ADR-001 item 4).
    let (mut manifest, mut files) = split(&slice_file().to_bytes("t").unwrap());
    put(&mut manifest, "x_manifest", Value::Bool(true));
    let mut scene: Value = from_cbor(&files[0].1).unwrap();
    put(&mut scene, "x_document", Value::Bool(true));
    let layers = migrate::map_get_mut(&mut scene, "layers").unwrap();
    let Value::Array(layers) = layers else {
        panic!("layers")
    };
    put(&mut layers[0], "x_layer", Value::Bool(true));
    let Value::Array(objects) = migrate::map_get_mut(&mut layers[1], "objects").unwrap() else {
        panic!("objects")
    };
    put(&mut objects[0], "x_object", Value::Bool(true));
    files[0].1 = to_cbor(&scene).unwrap();
    let mut outl: Value = from_cbor(&files[1].1).unwrap();
    put(&mut outl, "x_outlines", Value::Bool(true));
    files[1].1 = to_cbor(&outl).unwrap();

    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let (m2, f2) = split(&opened.to_bytes("t").unwrap());
    let s2: Value = from_cbor(&f2[0].1).unwrap();
    let mut s2m = s2.clone();
    let Value::Array(l2) = migrate::map_get_mut(&mut s2m, "layers").unwrap().clone() else {
        panic!()
    };
    let mut l2b = l2[1].clone();
    let Value::Array(o2) = migrate::map_get_mut(&mut l2b, "objects").unwrap().clone() else {
        panic!()
    };
    let o2v: Value = from_cbor(&f2[1].1).unwrap();
    let kept = [
        ("manifest", has(&m2, "x_manifest")),
        ("document", has(&s2, "x_document")),
        ("layer", has(&l2[0], "x_layer")),
        ("object", has(&o2[0], "x_object")),
        ("outlines", has(&o2v, "x_outlines")),
    ];
    let lost: Vec<&str> = kept.iter().filter(|(_, k)| !k).map(|(n, _)| *n).collect();
    println!(
        "CHECK crate=format name=unknown_fields_survive_resave result={} levels={} lost={lost:?}",
        pass(lost.is_empty()),
        kept.len()
    );
    assert!(lost.is_empty());
}

#[test]
fn newer_minor_file_keeps_its_version() {
    // ADR-001 item 5: a resaved newer file keeps its own format_version; saved_by is updated.
    let (mut manifest, files) = split(&slice_file().to_bytes("0.1.0").unwrap());
    *migrate::map_get_mut(&mut manifest, "format_version").unwrap() = Value::Text("0.8.0".into());
    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let again = QuadFile::from_bytes(&opened.to_bytes("0.1.0-older").unwrap()).unwrap();
    let ok = opened.read_only.is_none()
        && again.manifest.format_version == "0.8.0"
        && again.manifest.saved_by == "0.1.0-older";
    println!(
        "CHECK crate=format name=newer_minor_keeps_version result={} version={} saved_by={}",
        pass(ok),
        again.manifest.format_version,
        again.manifest.saved_by
    );
    assert!(ok);
}

#[test]
fn unknown_required_feature_opens_read_only() {
    let (mut manifest, files) = split(&slice_file().to_bytes("t").unwrap());
    let Value::Array(feats) = migrate::map_get_mut(&mut manifest, "required_features").unwrap()
    else {
        panic!()
    };
    feats.push(Value::Text("boolean-ops".into()));
    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let ok = opened.read_only == Some(ReadOnly::UnknownFeatures(vec!["boolean-ops".into()]))
        && matches!(opened.to_bytes("t"), Err(FormatError::ReadOnly(_)));
    println!(
        "CHECK crate=format name=unknown_feature_read_only result={} feature=boolean-ops",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn newer_major_opens_read_only() {
    let (mut manifest, files) = split(&slice_file().to_bytes("t").unwrap());
    *migrate::map_get_mut(&mut manifest, "format_version").unwrap() = Value::Text("1.0.0".into());
    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let ok = opened.read_only == Some(ReadOnly::NewerMajor("1.0.0".into()))
        && opened.to_bytes("t").is_err();
    println!(
        "CHECK crate=format name=newer_major_read_only result={} version=1.0.0",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn damaged_entry_is_caught() {
    // Change one byte of the scene and keep the manifest as written: its checksum catches it.
    let (manifest, mut files) = split(&slice_file().to_bytes("t").unwrap());
    let good = zip_raw(&manifest, &files);
    files[0].1[10] ^= 0x01;
    let bad = zip_raw(&manifest, &files);
    let ok = QuadFile::from_bytes(&good).is_ok()
        && matches!(QuadFile::from_bytes(&bad), Err(FormatError::Checksum(n)) if n == "scene.cbor");
    println!(
        "CHECK crate=format name=damaged_entry_caught result={} entry=scene.cbor",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn random_damage_never_panics() {
    // A stable-Rust stand-in for cargo-fuzz (ADR-001 item 8): damage a valid file many ways.
    let good = slice_file().to_bytes("t").unwrap();
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let runs = 2000;
    let mut panics = 0;
    let mut opened = 0;
    for _ in 0..runs {
        let mut b = good.clone();
        for _ in 0..(1 + next() % 8) {
            let i = (next() % b.len() as u64) as usize;
            b[i] = next() as u8;
        }
        if next() % 4 == 0 {
            b.truncate((next() % b.len() as u64) as usize);
        }
        match std::panic::catch_unwind(|| QuadFile::from_bytes(&b).is_ok()) {
            Ok(true) => opened += 1,
            Ok(false) => {}
            Err(_) => panics += 1,
        }
    }
    let ok = panics == 0;
    println!(
        "CHECK crate=format name=random_damage_no_panic result={} runs={runs} panics={panics} still_opened={opened}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn migration_chain_walks_steps() {
    // This checks the machinery with a stand-in chain where
    // 0.0.9 had one `unit_mm` and 0.1.0 has `h_unit_mm` and `v_unit_mm`.
    fn split_unit(scene: &mut Value) -> Result<(), FormatError> {
        let r =
            migrate::map_get_mut(scene, "rhythm").ok_or(FormatError::Missing("rhythm".into()))?;
        let Value::Map(m) = r else {
            return Err(FormatError::Missing("rhythm".into()));
        };
        let i = m
            .iter()
            .position(|(k, _)| k.as_text() == Some("unit_mm"))
            .ok_or(FormatError::Missing("unit_mm".into()))?;
        let (_, unit) = m.remove(i);
        m.push((Value::Text("h_unit_mm".into()), unit.clone()));
        m.push((Value::Text("v_unit_mm".into()), unit));
        Ok(())
    }
    let chain: &[(&str, &str, Step)] = &[("0.0.9", "0.1.0", split_unit)];
    let doc = slice_document();
    let mut old: Value = Value::serialized(&doc).unwrap();
    {
        let Value::Map(m) = migrate::map_get_mut(&mut old, "rhythm").unwrap() else {
            panic!()
        };
        m.retain(|(k, _)| !matches!(k.as_text(), Some("h_unit_mm" | "v_unit_mm")));
        m.push((Value::Text("unit_mm".into()), Value::Float(6.0)));
    }
    let (v009, v010) = (Version::new(0, 0, 9), Version::new(0, 1, 0));
    let mut up = old.clone();
    let walked = migrate::upgrade(&mut up, &v009, &v010, chain).is_ok()
        && up.deserialized::<Document>().ok() == Some(doc);
    let no_route = migrate::upgrade(&mut old, &Version::new(0, 0, 5), &v010, chain).is_err();
    let ok = walked && no_route;
    println!(
        "CHECK crate=format name=migration_chain result={} steps=1 missing_link_error={no_route}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_0_1_file_opens_through_the_real_chain() {
    // A file written at format 0.1.0 (before the Centered origin and Grid pattern existed) opens in
    // this build through migrate::CHAIN, and saves again at the current version.
    let (mut manifest, files) = split(&slice_file().to_bytes("0.1.0").unwrap());
    *migrate::map_get_mut(&mut manifest, "format_version").unwrap() = Value::Text("0.1.0".into());
    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let resaved = QuadFile::from_bytes(&opened.to_bytes("t").unwrap()).unwrap();
    let ok = opened.migrated_from == Some(Version::new(0, 1, 0))
        && opened.doc == slice_document()
        && resaved.manifest.format_version == FORMAT_VERSION;
    println!(
        "CHECK crate=format name=open_0_1_file result={} migrated_from={:?} now={}",
        pass(ok),
        opened.migrated_from.map(|v| v.to_string()),
        resaved.manifest.format_version
    );
    assert!(ok);
}

#[test]
fn a_0_2_file_opens_through_the_real_chain() {
    // A file written at format 0.2.0 (before the DotGrid and Label roles existed) opens in this
    // build and saves again at the current version.
    let (mut manifest, files) = split(&slice_file().to_bytes("0.2.0").unwrap());
    *migrate::map_get_mut(&mut manifest, "format_version").unwrap() = Value::Text("0.2.0".into());
    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let resaved = QuadFile::from_bytes(&opened.to_bytes("t").unwrap()).unwrap();
    let ok = opened.migrated_from == Some(Version::new(0, 2, 0))
        && opened.doc == slice_document()
        && resaved.manifest.format_version == FORMAT_VERSION;
    println!(
        "CHECK crate=format name=open_0_2_file result={} migrated_from={:?} now={}",
        pass(ok),
        opened.migrated_from.map(|v| v.to_string()),
        resaved.manifest.format_version
    );
    assert!(ok);
}

#[test]
fn a_0_3_file_opens_and_subdivisions_round_trip() {
    // A file written at format 0.3.0 (before `rhythm.subdivisions` existed) has no such field; it
    // opens as "off" through migrate::CHAIN, and saves again at the current version without
    // inventing one. A document that sets subdivisions keeps them through a save.
    let (mut manifest, files) = split(&slice_file().to_bytes("0.3.0").unwrap());
    *migrate::map_get_mut(&mut manifest, "format_version").unwrap() = Value::Text("0.3.0".into());
    let opened = QuadFile::from_bytes(&raw_quad(manifest, &files)).unwrap();
    let resaved = QuadFile::from_bytes(&opened.to_bytes("t").unwrap()).unwrap();
    let mut with = slice_file();
    with.doc.rhythm.subdivisions = Some(4);
    let back = QuadFile::from_bytes(&with.to_bytes("t").unwrap()).unwrap();
    let mut wild = slice_file();
    wild.doc.rhythm.subdivisions = Some(40);
    let wild = QuadFile::from_bytes(&wild.to_bytes("t").unwrap()).unwrap();
    let ok = opened.migrated_from == Some(Version::new(0, 3, 0))
        && opened.doc == slice_document()
        && opened.doc.rhythm.subdivisions.is_none()
        && resaved.manifest.format_version == FORMAT_VERSION
        && back.doc.rhythm.subdivisions == Some(4)
        && back.doc.rhythm.subdivisions_on() == Some(4)
        && wild.doc.rhythm.subdivisions_on().is_none();
    println!(
        "\nCHECK crate=format name=open_0_3_file result={} migrated_from={:?} now={} subdivisions_kept={:?} out_of_range_reads_off={}",
        pass(ok),
        opened.migrated_from.map(|v| v.to_string()),
        resaved.manifest.format_version,
        back.doc.rhythm.subdivisions,
        wild.doc.rhythm.subdivisions_on().is_none()
    );
    assert!(ok);
}
