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

//! Format 0.21.0 files (ADR-026: **Show in Basic**). The two starters in `tests/fixtures/v0.21/` (narrow ruled and the
//! rapid log) were written by `cargo xtask starters` in format 0.21.0 and carry the marks: the page's margins, rhythm
//! and title, and the objects that make the page what it is. Together with the sha256 of the PNG each device's export
//! gave (`tests/fixtures/v0.21/SHA256SUMS`, the same as those starters' goldens), they must open, read the marks as
//! they were written, export the same bytes now and after a save, and write the marks again, only where they are on.
//! Since format 0.22.0 (ADR-027, 2026-11-26) they open through the 0.22 step, which moves on-step hand-picked grays;
//! their recorded exports were made again then, with every golden page. Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::{FORMAT_VERSION, QuadFile, sha256_hex};
use quadrule_layout::display_list;
use quadrule_model::Orientation;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0.21")
}

fn saved_sums() -> Vec<(String, String)> {
    std::fs::read_to_string(dir().join("SHA256SUMS"))
        .unwrap()
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            Some((it.next()?.to_string(), it.next()?.to_string()))
        })
        .collect()
}

/// The PNG a file's page gives on one device, drawn from the outlines saved in the file.
fn export(file: &QuadFile, id: &str) -> Vec<u8> {
    let o = file.doc.page.orientation;
    let primary = builtin_for(&file.doc.primary_profile_id, o).unwrap();
    let target = builtin_for(id, o).unwrap();
    let dl = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
    let img = render(&dl);
    let img = if o == Orientation::Landscape {
        turn_clockwise(&img)
    } else {
        img
    };
    encode_png(&img, None)
}

/// How many maps in `v` have a key called `name`.
fn count_key(v: &ciborium::Value, name: &str) -> usize {
    use ciborium::Value;
    match v {
        Value::Map(m) => m
            .iter()
            .map(|(k, x)| usize::from(k.as_text() == Some(name)) + count_key(x, name))
            .sum(),
        Value::Array(a) => a.iter().map(|x| count_key(x, name)).sum(),
        _ => 0,
    }
}

#[test]
fn v021_files_open_as_they_are_keep_their_marks_and_export_the_same_bytes() {
    let sums = saved_sums();
    let mut ok = !sums.is_empty() && FORMAT_VERSION == "0.22.0";
    let mut parts = Vec::new();
    let mut matched = 0;
    let (mut marked_objects, mut objects) = (0, 0);
    for name in ["narrow-ruled", "rapid-log"] {
        let bytes = std::fs::read(dir().join(format!("{name}.quad"))).unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        // Written by 0.21.0: opens through the 0.22 step, editable.
        let current = file.manifest.format_version == "0.21.0"
            && file.migrated_from.as_ref().map(|v| v.to_string()) == Some("0.21.0".into())
            && file.read_only.is_none();
        // The page's three marks, and at least one object, were written as on.
        let page = file.doc.page.basic;
        let marks = page.margins && page.rhythm && page.title;
        let on = file.doc.objects().filter(|o| o.show_in_basic).count();
        marked_objects += on;
        objects += file.doc.objects().count();
        ok &= current && marks && on > 0;
        parts.push(format!(
            "{name}:current={current}:page_marks={marks}:marked_objects={on}"
        ));
        for id in file.doc.profile_ids() {
            let want = sums.iter().find(|(_, n)| *n == format!("{name}_{id}.png"));
            let got = sha256_hex(&export(&file, id));
            let same = want.is_some_and(|(s, _)| *s == got);
            ok &= same;
            matched += usize::from(same);
            parts.push(format!("{name}:{id}:same={same}"));
        }
        // Saved again: the same document with the same marks, written as keys only where on, the same exports.
        let again = QuadFile::from_bytes(&file.to_bytes("migrate-test").unwrap()).unwrap();
        let scene = ciborium::Value::serialized(&again.doc).unwrap();
        let keys = count_key(&scene, "show_in_basic");
        let basic_key = count_key(&scene, "basic");
        let resaved = again.doc == file.doc
            && again.manifest.format_version == FORMAT_VERSION
            && again.migrated_from.is_none()
            && keys == on
            && basic_key == 1;
        ok &= resaved;
        parts.push(format!("{name}:resaved_0210={resaved}"));
        for id in again.doc.profile_ids() {
            ok &= export(&again, id) == export(&file, id);
        }
    }
    println!(
        "\nCHECK crate=render name=migrate_v021 result={} format_version={FORMAT_VERSION} fixtures=2 objects={objects} marked_objects={marked_objects} export_hashes_matched={matched} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
