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

//! Format 0.14's carried profiles (rung 13a, docs/bbq/2026-11-17-rung-13-profile-system.md, Q4 and
//! Defaulted "Carrying in the file", "The `.quad` reader"). Each test prints one structured `CHECK`
//! line (run with `-- --nocapture`).

use quadrule_format::*;
use quadrule_model::sample::*;
use quadrule_profiles::{builtin, to_ron};
use std::io::{Cursor, Read, Write};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn mine() -> quadrule_profiles::Profile {
    let mut p = builtin("supernote.a6x2").unwrap();
    p.id = "11111111-2222-4333-8444-555555555555".into();
    p.display_name = "Kindle Scribe".into();
    p.short_name = "scribe".into();
    p.calibration_meta.verified = false;
    p
}

fn file_with(p: &quadrule_profiles::Profile) -> QuadFile {
    let doc = slice_document();
    let mut f = QuadFile::new(doc, vec![], vec![]);
    let mut reg = quadrule_text::FontRegistry::bundled_only();
    let primary = builtin(&f.doc.primary_profile_id).unwrap();
    for id in f.doc.profile_ids().iter().map(|s| s.to_string()) {
        let t = builtin(&id).unwrap();
        let laid = quadrule_layout::lay_out_text(&f.doc, &primary, &t, &mut reg).unwrap();
        f.outlines.insert(id, laid.outlines);
    }
    f.profiles.insert(p.id.clone(), p.clone());
    f
}

/// The archive's entries.
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

/// The archive with entry `path` holding `text`, and the manifest's checksum and size for it made
/// to match: the file is whole, only the profile in it is bad.
fn with_text(bytes: &[u8], path: &str, text: &[u8]) -> Vec<u8> {
    let mut all = entries(bytes);
    let mut manifest: Manifest =
        from_cbor(&all.iter().find(|(n, _)| n == "manifest.cbor").unwrap().1).unwrap();
    let info = manifest.entries.get_mut(path).unwrap();
    info.sha256 = sha256_hex(text);
    info.size = text.len() as u64;
    for (n, b) in &mut all {
        if n == path {
            *b = text.to_vec();
        } else if n == "manifest.cbor" {
            *b = to_cbor(&manifest).unwrap();
        }
    }
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (n, b) in &all {
        z.start_file(n.as_str(), zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(b).unwrap();
    }
    z.finish().unwrap().into_inner()
}

#[test]
fn a_carried_profile_round_trips_and_is_listed_in_the_manifest() {
    let p = mine();
    let f = file_with(&p);
    let bytes = f.to_bytes("t").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let path = profile_path(&p.id);
    let names: Vec<String> = entries(&bytes).into_iter().map(|(n, _)| n).collect();
    let text = entries(&bytes)
        .into_iter()
        .find(|(n, _)| *n == path)
        .map(|(_, b)| b);
    let again = back.to_bytes("t").unwrap();
    let ok = back.profiles.get(&p.id) == Some(&p)
        && back.damaged_profiles.is_empty()
        && back.manifest.carried_profiles.len() == 1
        && back.manifest.carried_profiles[0].id == p.id
        && back.manifest.carried_profiles[0].name == "Kindle Scribe"
        && back.manifest.carried_profiles[0].path == path
        && back.manifest.entries.contains_key(&path)
        && names.contains(&path)
        && text.as_deref() == Some(to_ron(&p).as_bytes())
        && back.manifest.format_version == "0.20.0"
        && again == bytes
        && back.doc == f.doc;
    println!(
        "\nCHECK crate=format name=carried_profile_round_trip result={} path={path} bytes={}",
        pass(ok),
        bytes.len()
    );
    assert!(ok);
}

#[test]
fn a_file_with_no_carried_profile_has_no_profile_entries() {
    let mut f = file_with(&mine());
    f.profiles.clear();
    let bytes = f.to_bytes("t").unwrap();
    let names: Vec<String> = entries(&bytes).into_iter().map(|(n, _)| n).collect();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let ok = !names.iter().any(|n| n.starts_with("profiles/"))
        && back.manifest.carried_profiles.is_empty()
        && back.profiles.is_empty();
    println!(
        "\nCHECK crate=format name=no_carried_profile result={}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_broken_carried_profile_is_skipped_and_the_file_opens() {
    let p = mine();
    let bytes = file_with(&p).to_bytes("t").unwrap();
    let path = profile_path(&p.id);
    let good = to_ron(&p);
    let mut other = p.clone();
    other.id = "someone-else".into();
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("cut_short", good.as_bytes()[..good.len() / 2].to_vec()),
        (
            "typo",
            good.replace("ppi: 300.0", "ppi: three").into_bytes(),
        ),
        (
            "huge_size",
            good.replace("width_px: 1404", "width_px: 99999999")
                .into_bytes(),
        ),
        (
            "zero_ppi",
            good.replace("ppi: 300.0", "ppi: 0.0").into_bytes(),
        ),
        ("not_text", vec![0, 159, 146, 150, 255]),
        ("wrong_id", to_ron(&other).into_bytes()),
        ("large", vec![b' '; 2 * 1024 * 1024]),
        ("empty", Vec::new()),
    ];
    let mut bad = Vec::new();
    for (name, text) in &cases {
        let broken = with_text(&bytes, &path, text);
        match QuadFile::from_bytes(&broken) {
            Ok(o) => {
                let skipped = o.profiles.is_empty()
                    && o.damaged_profiles.len() == 1
                    && o.damaged_profiles[0].0 == path
                    && o.doc == file_with(&p).doc
                    && o.read_only.is_none();
                // Saved again, the damaged profile is not written back (nothing broken is kept).
                let resaved = o
                    .to_bytes("t")
                    .ok()
                    .and_then(|b| QuadFile::from_bytes(&b).ok());
                if !skipped || !resaved.is_some_and(|r| r.damaged_profiles.is_empty()) {
                    bad.push(format!("{name}:not_skipped"));
                }
            }
            Err(e) => bad.push(format!("{name}:{e}")),
        }
    }
    // A broken profile can also not be written: the writer refuses it rather than make a file the
    // reader would skip.
    let mut worse = file_with(&p);
    worse.profiles.get_mut(&p.id).unwrap().ppi = 0.0;
    let refused = worse.to_bytes("t").is_err();
    let ok = bad.is_empty() && refused;
    println!(
        "\nCHECK crate=format name=broken_carried_profile result={} cases={} bad={bad:?} writer_refuses={refused}",
        pass(ok),
        cases.len()
    );
    assert!(ok);
}

#[test]
fn the_chain_reaches_0_14() {
    let f = file_with(&mine());
    let bytes = f.to_bytes("t").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let ok = back.migrated_from.is_none()
        && quadrule_format::migrate::CHAIN
            .iter()
            .any(|(a, b, _)| *a == "0.13.0" && *b == "0.14.0");
    println!(
        "\nCHECK crate=format name=chain_has_0_14 result={}",
        pass(ok)
    );
    assert!(ok);
}

fn seed_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz/seeds/open_quad/carried-profile.quad")
}

/// Rewrites the fuzz seed: `cargo test -p quadrule-format --test profiles -- --ignored`.
#[test]
#[ignore = "writes fuzz/seeds/open_quad/carried-profile.quad"]
fn write_the_fuzz_seed() {
    std::fs::write(seed_path(), file_with(&mine()).to_bytes("seed").unwrap()).unwrap();
}

#[test]
fn the_fuzz_seed_carries_a_profile() {
    // The open_quad fuzz target starts from this file, so its mutations reach the profile reader.
    let back = QuadFile::from_bytes(&std::fs::read(seed_path()).unwrap()).unwrap();
    let ok = back.profiles.len() == 1 && back.damaged_profiles.is_empty();
    println!(
        "\nCHECK crate=format name=fuzz_seed_carries_profile result={}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn renaming_a_profile_reaches_every_place_the_document_names_it() {
    let doc = devices_document();
    let from = doc.profile_ids()[1].to_string();
    let had_overrides = doc
        .objects()
        .filter(|o| o.overrides.contains_key(&from))
        .count();
    let renamed = rename_profile(&doc, &from, "new-id").unwrap();
    let still: usize = renamed
        .objects()
        .filter(|o| o.overrides.contains_key(&from))
        .count();
    let now = renamed
        .objects()
        .filter(|o| o.overrides.contains_key("new-id"))
        .count();
    let back = rename_profile(&renamed, "new-id", &from).unwrap();
    // The primary too.
    let primary = doc.primary_profile_id.clone();
    let p2 = rename_profile(&doc, &primary, "primary-2").unwrap();
    let ok = had_overrides > 0
        && still == 0
        && now == had_overrides
        && renamed.profile_ids()[1] == "new-id"
        && back == doc
        && p2.primary_profile_id == "primary-2"
        && p2.profile_ids()[0] == "primary-2";
    println!(
        "\nCHECK crate=format name=rename_profile result={} objects_with_overrides={had_overrides}",
        pass(ok)
    );
    assert!(ok);
}
