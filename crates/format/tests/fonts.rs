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

//! Format 0.13's records (rung 12a, docs/bbq/2026-11-16-rung-12-missing-fonts.md, Defaulted "File
//! format 0.13.0" and "Embedding in detail"): the font signature and embedded files in the manifest,
//! the document's font choices, a damaged embedded font that never stops a file opening, and the
//! migration that sets a pattern region's stored turn to 0. Each test prints one structured `CHECK`
//! line (run with `-- --nocapture`).

use ciborium::Value;
use quadrule_format::migrate;
use quadrule_format::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_text::BUNDLED;
use std::io::{Cursor, Read, Write};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn signature() -> FontSignature {
    FontSignature {
        class: FontClass::Serif,
        weight: 400,
        width: 100,
        italic: false,
        avg_width: 480,
        x_height: 500,
        mono: false,
        extra: Extra::new(),
    }
}

/// A file whose one font is embedded: Caveat's file stands in for an installed font.
fn file_with_embedded_font() -> (QuadFile, String) {
    let doc = textbox_document();
    let caveat = BUNDLED.iter().find(|b| b.id == "caveat").unwrap();
    let path = font_path(caveat.data);
    let mut font = FontRef::new("Test Family", None, None);
    font.signature = Some(signature());
    font.files = vec![FontFile {
        path: path.clone(),
        sha256: sha256_hex(caveat.data),
        weight: 400,
        italic: false,
        extra: Extra::new(),
    }];
    let outlines = vec![];
    let mut f = QuadFile::new(doc, outlines, vec![font]);
    f.doc.page.fonts.keep_outlines = vec!["Some Font".into()];
    f.font_files.insert(path.clone(), caveat.data.to_vec());
    (f, path)
}

fn with_outlines(mut f: QuadFile) -> QuadFile {
    let mut reg = quadrule_text::FontRegistry::bundled_only();
    let primary = quadrule_profiles::builtin(&f.doc.primary_profile_id).unwrap();
    for id in f.doc.profile_ids().iter().map(|s| s.to_string()) {
        let target = quadrule_profiles::builtin(&id).unwrap();
        let laid = quadrule_layout::lay_out_text(&f.doc, &primary, &target, &mut reg).unwrap();
        f.outlines.insert(id, laid.outlines);
    }
    f
}

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

fn zip_raw(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for (n, b) in files {
        z.start_file(n.as_str(), opts).unwrap();
        z.write_all(b).unwrap();
    }
    z.finish().unwrap().into_inner()
}

fn put(v: &mut Value, key: &str) {
    if let Value::Map(m) = v {
        m.push((Value::Text(key.into()), Value::Bool(true)));
    }
}

fn has(v: &Value, key: &str) -> bool {
    matches!(v, Value::Map(m) if m.iter().any(|(k, _)| k.as_text() == Some(key)))
}

#[test]
fn embedded_fonts_round_trip_and_the_signature_is_kept() {
    let (f, path) = file_with_embedded_font();
    let f = with_outlines(f);
    let bytes = f.to_bytes("t").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let names: Vec<String> = entries(&bytes).into_iter().map(|(n, _)| n).collect();
    let ok = back.font_files == f.font_files
        && back.manifest.fonts == f.manifest.fonts
        && back.manifest.fonts[0].signature == Some(signature())
        && back.doc == f.doc
        && back.damaged_fonts.is_empty()
        && names.contains(&path)
        && back.manifest.entries.contains_key(&path)
        && path.starts_with("assets/fonts/")
        && path.ends_with(".ttf");
    println!(
        "\nCHECK crate=format name=embedded_font_round_trip result={} path={path} bytes={}",
        pass(ok),
        bytes.len()
    );
    assert!(ok);
}

#[test]
fn a_damaged_embedded_font_is_skipped_and_the_file_opens() {
    let (f, path) = file_with_embedded_font();
    let f = with_outlines(f);
    let bytes = f.to_bytes("t").unwrap();
    // Flip bytes inside the font entry, leaving the manifest (and its checksum) as it was.
    let mut all = entries(&bytes);
    let at = all.iter().position(|(n, _)| *n == path).unwrap();
    for b in all[at].1.iter_mut().take(64) {
        *b ^= 0xff;
    }
    let damaged = zip_raw(&all);
    let opened = QuadFile::from_bytes(&damaged);
    let skipped = opened.as_ref().is_ok_and(|o| {
        o.damaged_fonts == [path.clone()] && o.font_files.is_empty() && o.doc == f.doc
    });
    // And one that is missing from the archive altogether.
    let mut all = entries(&bytes);
    all.retain(|(n, _)| *n != path);
    let missing =
        QuadFile::from_bytes(&zip_raw(&all)).is_ok_and(|o| o.damaged_fonts == [path.clone()]);
    // A damaged scene still stops the file: only fonts are forgiven.
    let mut all = entries(&bytes);
    let s = all.iter().position(|(n, _)| n == "scene.cbor").unwrap();
    all[s].1[0] ^= 0xff;
    let scene_stops = QuadFile::from_bytes(&zip_raw(&all)).is_err();
    // The skipped file saves again, and reopens clean.
    let resaved = opened
        .and_then(|o| QuadFile::from_bytes(&o.to_bytes("t").unwrap()))
        .is_ok_and(|o| o.damaged_fonts.is_empty());
    let ok = skipped && missing && scene_stops && resaved;
    println!(
        "\nCHECK crate=format name=damaged_embedded_font_skipped result={} skipped={skipped} missing={missing} scene_still_stops={scene_stops} resaved_clean={resaved}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn font_records_keep_unknown_fields() {
    // ADR-001 item 4 for the four records 0.13 adds or grows: FontRef, FontFile, FontSignature and
    // FontSettings. A field from a newer version survives open, save and open.
    let (f, _) = file_with_embedded_font();
    let f = with_outlines(f);
    let bytes = f.to_bytes("t").unwrap();
    let mut all = entries(&bytes);
    let mut manifest: Value = from_cbor(&all.remove(0).1).unwrap();
    if let Some(Value::Array(fonts)) = migrate::map_get_mut(&mut manifest, "fonts") {
        put(&mut fonts[0], "x_fontref");
        put(
            migrate::map_get_mut(&mut fonts[0], "signature").unwrap(),
            "x_signature",
        );
        let Some(Value::Array(files)) = migrate::map_get_mut(&mut fonts[0], "files") else {
            panic!("files")
        };
        put(&mut files[0], "x_fontfile");
    }
    let s = all.iter().position(|(n, _)| n == "scene.cbor").unwrap();
    let mut scene: Value = from_cbor(&all[s].1).unwrap();
    let page = migrate::map_get_mut(&mut scene, "page").unwrap();
    put(migrate::map_get_mut(page, "fonts").unwrap(), "x_settings");
    all[s].1 = to_cbor(&scene).unwrap();
    // Rebuild the manifest's checksums, then write.
    let entries_v: Vec<(Value, Value)> = all
        .iter()
        .map(|(n, b)| {
            (
                Value::Text(n.clone()),
                Value::Map(vec![
                    (Value::Text("sha256".into()), Value::Text(sha256_hex(b))),
                    (
                        Value::Text("size".into()),
                        Value::Integer((b.len() as u64).into()),
                    ),
                ]),
            )
        })
        .collect();
    *migrate::map_get_mut(&mut manifest, "entries").unwrap() = Value::Map(entries_v);
    let mut files = vec![("manifest.cbor".to_string(), to_cbor(&manifest).unwrap())];
    files.extend(all);
    let opened = QuadFile::from_bytes(&zip_raw(&files)).unwrap();
    let again = entries(&opened.to_bytes("t").unwrap());
    let m2: Value = from_cbor(&again[0].1).unwrap();
    let s2: Value = from_cbor(&again.iter().find(|(n, _)| n == "scene.cbor").unwrap().1).unwrap();
    let Some(Value::Array(fonts)) = migrate::map_get_mut(&mut m2.clone(), "fonts").cloned() else {
        panic!("fonts")
    };
    let sig = migrate::map_get_mut(&mut fonts[0].clone(), "signature")
        .cloned()
        .unwrap();
    let Some(Value::Array(ffiles)) = migrate::map_get_mut(&mut fonts[0].clone(), "files").cloned()
    else {
        panic!("files")
    };
    let page = migrate::map_get_mut(&mut s2.clone(), "page")
        .cloned()
        .unwrap();
    let kept = [
        ("fontref", has(&fonts[0], "x_fontref")),
        ("signature", has(&sig, "x_signature")),
        ("fontfile", has(&ffiles[0], "x_fontfile")),
        (
            "settings",
            has(
                &migrate::map_get_mut(&mut page.clone(), "fonts")
                    .cloned()
                    .unwrap(),
                "x_settings",
            ),
        ),
    ];
    let lost: Vec<&str> = kept.iter().filter(|(_, k)| !k).map(|(n, _)| *n).collect();
    println!(
        "\nCHECK crate=format name=font_records_keep_unknown_fields result={} records={} lost={lost:?}",
        pass(lost.is_empty()),
        kept.len()
    );
    assert!(lost.is_empty());
}

#[test]
fn default_font_settings_are_left_out_of_the_file() {
    // A page that holds the defaults writes no `fonts` field, so a file with no font choices has the
    // very scene bytes the 0.12 build wrote.
    let doc = textbox_document();
    let v: Value = {
        let bytes = to_cbor(&doc).unwrap();
        from_cbor(&bytes).unwrap()
    };
    let page = migrate::map_get_mut(&mut v.clone(), "page")
        .cloned()
        .unwrap();
    let left_out = !has(&page, "fonts") && doc.page.fonts.is_default() && doc.page.fonts.embed;
    let mut changed = doc.clone();
    changed.page.fonts.embed = false;
    let bytes = to_cbor(&changed).unwrap();
    let back: Document = from_cbor(&bytes).unwrap();
    let kept = back.page.fonts == changed.page.fonts && !back.page.fonts.embed;
    let ok = left_out && kept;
    println!(
        "\nCHECK crate=format name=default_font_settings_left_out result={} left_out={left_out} kept_when_changed={kept}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn migration_sets_a_pattern_regions_stored_turn_to_zero() {
    // A 0.12 file may hold a `rotation_deg` on a pattern region (the app ignored it): 0.13 turns
    // pattern regions, so the step drops it, in a group and in a repeat's source too.
    let mut doc = patterns_document();
    let mut turned = 0;
    fn turn(objs: &mut [Object], n: &mut usize) {
        for o in objs {
            if matches!(o.kind, ObjectKind::Pattern(_)) {
                o.rotation_deg = 30.0;
                *n += 1;
            }
            turn(o.children_mut(), n);
        }
    }
    for l in &mut doc.layers {
        turn(&mut l.objects, &mut turned);
    }
    let mut scene: Value = from_cbor(&to_cbor(&doc).unwrap()).unwrap();
    let before = count_rotations(&scene);
    migrate::CHAIN
        .iter()
        .find(|(f, t, _)| *f == "0.12.0" && *t == "0.13.0")
        .map(|(_, _, step)| step(&mut scene).unwrap())
        .unwrap();
    let after = count_rotations(&scene);
    let back: Document = scene.deserialized().unwrap();
    let all_zero = back
        .objects()
        .filter(|o| matches!(o.kind, ObjectKind::Pattern(_)))
        .all(|o| o.rotation_deg == 0.0 && o.rotation() == 0.0);
    let ok = turned > 0 && before == turned && after == 0 && all_zero;
    println!(
        "\nCHECK crate=format name=migrate_pattern_turn_zeroed result={} regions={turned} stored_before={before} stored_after={after}",
        pass(ok)
    );
    assert!(ok);
}

/// How many pattern regions in a scene tree hold a `rotation_deg`.
fn count_rotations(v: &Value) -> usize {
    match v {
        Value::Map(m) => {
            let here = usize::from(
                m.iter().any(|(k, val)| {
                    k.as_text() == Some("kind")
                        && matches!(val, Value::Map(k) if k.first().and_then(|(n, _)| n.as_text()) == Some("Pattern"))
                }) && m.iter().any(|(k, _)| k.as_text() == Some("rotation_deg")),
            );
            here + m.iter().map(|(_, x)| count_rotations(x)).sum::<usize>()
        }
        Value::Array(a) => a.iter().map(count_rotations).sum(),
        _ => 0,
    }
}
