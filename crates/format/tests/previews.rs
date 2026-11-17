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

//! Format 0.16's previews (rung 15 bbq, Q8): `previews/<id>.png` is carried as bytes, listed in the manifest with its
//! checksum, written in a fixed place in the archive, and never needed: a file without one is valid, a damaged or
//! missing one is skipped and the file opens, and the next save writes it clean. Each test prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use quadrule_format::*;
use quadrule_model::sample::*;
use std::io::{Cursor, Read, Write};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn file() -> QuadFile {
    let doc = slice_document();
    QuadFile::new(doc, vec![], vec![])
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

const PNG: &[u8] = b"\x89PNG stand-in picture bytes";

#[test]
fn a_preview_round_trips_in_a_fixed_place_and_the_bytes_are_stable() {
    let mut f = with_outlines(file());
    let path = preview_path(&f.doc.primary_profile_id);
    f.previews.insert(path.clone(), PNG.to_vec());
    let bytes = f.to_bytes("t").unwrap();
    let again = f.to_bytes("t").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let names: Vec<String> = entries(&bytes).into_iter().map(|(n, _)| n).collect();
    let listed = back
        .manifest
        .entries
        .get(&path)
        .is_some_and(|e| e.sha256 == sha256_hex(PNG) && e.size == PNG.len() as u64);
    let ok = back.previews.get(&path).map(Vec::as_slice) == Some(PNG)
        && listed
        && names.last() == Some(&path)
        && path == format!("previews/{}.png", f.doc.primary_profile_id)
        && bytes == again
        && back.doc == f.doc;
    println!(
        "\nCHECK crate=format name=preview_round_trip result={} path={path} listed={listed} stable_bytes={}",
        pass(ok),
        bytes == again
    );
    assert!(ok);
}

#[test]
fn a_file_without_previews_is_valid_and_saves_none() {
    let f = with_outlines(file());
    let bytes = f.to_bytes("t").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let no_entry = !entries(&bytes)
        .iter()
        .any(|(n, _)| n.starts_with(PREVIEW_DIR));
    let ok = back.previews.is_empty() && no_entry && back.read_only.is_none();
    println!(
        "\nCHECK crate=format name=no_preview_valid result={} previews={} entry_in_archive={}",
        pass(ok),
        back.previews.len(),
        !no_entry
    );
    assert!(ok);
}

#[test]
fn a_damaged_or_missing_preview_is_skipped_and_the_file_opens() {
    let mut f = with_outlines(file());
    let path = preview_path(&f.doc.primary_profile_id);
    f.previews.insert(path.clone(), PNG.to_vec());
    let bytes = f.to_bytes("t").unwrap();
    let mut all = entries(&bytes);
    let at = all.iter().position(|(n, _)| *n == path).unwrap();
    for b in all[at].1.iter_mut().take(8) {
        *b ^= 0xff;
    }
    let damaged = QuadFile::from_bytes(&zip_raw(&all));
    let skipped = damaged
        .as_ref()
        .is_ok_and(|o| o.previews.is_empty() && o.doc == f.doc);
    let mut all = entries(&bytes);
    all.retain(|(n, _)| *n != path);
    let missing = QuadFile::from_bytes(&zip_raw(&all)).is_ok_and(|o| o.previews.is_empty());
    let resaved = damaged
        .and_then(|o| QuadFile::from_bytes(&o.to_bytes("t").unwrap()))
        .is_ok_and(|o| o.previews.is_empty() && o.read_only.is_none());
    let ok = skipped && missing && resaved;
    println!(
        "\nCHECK crate=format name=damaged_preview_skipped result={} skipped={skipped} missing={missing} resaved_clean={resaved}",
        pass(ok)
    );
    assert!(ok);
}
