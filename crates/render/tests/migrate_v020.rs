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

//! Format 0.21.0's migration (ADR-026: **Show in Basic**, the marks that say what the Basic tab lists). The 0.20
//! files in `tests/fixtures/v0.20/` are the major-lines-from, major-lines-per-axis and calendar pages saved by the
//! 0.20.0 build before the format changed, together with the sha256 of the PNG each device's export gave
//! (`tests/fixtures/v0.20/SHA256SUMS`, the same as those pages' goldens). They must open, migrate and export the same
//! bytes now: the 0.20 to 0.21 step rewrites nothing, no object and no page setting reads as marked, and a resave
//! writes no `show_in_basic` and no `basic` field. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_format::{FORMAT_VERSION, QuadFile, migrate, sha256_hex};
use quadrule_layout::display_list;
use quadrule_model::Orientation;
use quadrule_profiles::builtin_for;
use quadrule_render::*;
use semver::Version;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0.20")
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

/// The PNG a file's page gives on one device, drawn from the outlines saved in the file; a device the
/// file carries is read from the file's copy.
fn export(file: &QuadFile, id: &str) -> Vec<u8> {
    let o = file.doc.page.orientation;
    let primary = builtin_for(&file.doc.primary_profile_id, o).unwrap();
    let target = match file.profiles.get(id) {
        Some(p) => p.oriented(o),
        None => builtin_for(id, o).unwrap(),
    };
    let dl = display_list(&file.doc, &primary, &target, &file.outlines[id]).unwrap();
    let img = render(&dl);
    let img = if o == Orientation::Landscape {
        turn_clockwise(&img)
    } else {
        img
    };
    encode_png(&img, None)
}

/// Whether any map in `v` has a key called `name`.
fn has_key(v: &ciborium::Value, name: &str) -> bool {
    use ciborium::Value;
    match v {
        Value::Map(m) => m
            .iter()
            .any(|(k, x)| k.as_text() == Some(name) || has_key(x, name)),
        Value::Array(a) => a.iter().any(|x| has_key(x, name)),
        _ => false,
    }
}

#[test]
fn v020_files_open_migrate_and_export_the_same_bytes() {
    let sums = saved_sums();
    let mut ok = !sums.is_empty() && FORMAT_VERSION == "0.22.0";
    let mut parts = Vec::new();
    let mut matched = 0;
    let mut objects = 0;
    for name in ["major-lines-from", "major-lines-per-axis", "calendar"] {
        let bytes = std::fs::read(dir().join(format!("{name}.quad"))).unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        let was_020 = file.migrated_from == Some(Version::parse("0.20.0").unwrap())
            && file.manifest.format_version == "0.20.0";
        let editable = file.read_only.is_none();
        // Nothing reads as marked, and a save writes no mark.
        let unmarked = !file.doc.page.basic.any() && file.doc.objects().all(|o| !o.show_in_basic);
        objects += file.doc.objects().count();
        ok &= was_020 && editable && unmarked;
        parts.push(format!("{name}:unmarked={unmarked}"));
        for id in file.doc.profile_ids() {
            let want = sums.iter().find(|(_, n)| *n == format!("{name}_{id}.png"));
            let got = sha256_hex(&export(&file, id));
            let same = want.is_some_and(|(s, _)| *s == got);
            ok &= same;
            matched += usize::from(same);
            parts.push(format!("{name}:{id}:same={same}"));
        }
        // Saved again, the file is the current version, writes no mark, and exports the same.
        let again = QuadFile::from_bytes(&file.to_bytes("migrate-test").unwrap()).unwrap();
        let scene = ciborium::Value::serialized(&again.doc).unwrap();
        let no_key = !has_key(&scene, "show_in_basic") && !has_key(&scene, "basic");
        let resaved = again.manifest.format_version == FORMAT_VERSION
            && again.migrated_from.is_none()
            && again.doc == file.doc
            && again.profiles == file.profiles
            && no_key;
        ok &= resaved;
        parts.push(format!("{name}:resaved_0210={resaved}"));
        for id in again.doc.profile_ids() {
            ok &= export(&again, id) == export(&file, id);
        }
    }
    println!(
        "\nCHECK crate=render name=migrate_v020 result={} format_version={FORMAT_VERSION} fixtures=3 objects={objects} export_hashes_matched={matched} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_chain_has_the_0_20_to_0_21_step() {
    let current = Version::parse(FORMAT_VERSION).unwrap();
    let mut at = Version::parse("0.1.0").unwrap();
    let mut steps = 0;
    while at < current {
        let step = migrate::CHAIN
            .iter()
            .find(|(f, _, _)| Version::parse(f).unwrap() == at);
        let Some((_, next, _)) = step else { break };
        at = Version::parse(next).unwrap();
        steps += 1;
    }
    let has_step = migrate::CHAIN
        .iter()
        .any(|(f, t, _)| *f == "0.20.0" && *t == "0.21.0");
    let ok = at == current && has_step;
    println!(
        "\nCHECK crate=render name=migrate_chain_0210 result={} from=0.1.0 to={at} steps={steps} has_0_20_to_0_21={has_step}",
        pass(ok)
    );
    assert!(ok);
}
