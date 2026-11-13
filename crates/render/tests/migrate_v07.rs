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

//! Format 0.8.0's migration (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md, Defaulted "File
//! format 0.8.0"). The two 0.7 files in `tests/fixtures/v0.7/` (the groups page and the shapes page)
//! were written by the 0.7 code, before any 0.8 change, together with the sha256 of the PNG each
//! device's export gave (`tests/fixtures/v0.7/SHA256SUMS`). Every one must open, migrate and export
//! the same bytes now: nothing old changes, so the 0.7 to 0.8 step rewrites nothing. Each test prints one structured `CHECK` line (run
//! with `-- --nocapture`).

use quadrule_format::{FORMAT_VERSION, QuadFile, migrate, sha256_hex};
use quadrule_layout::display_list;
use quadrule_model::Orientation;
use quadrule_profiles::builtin;
use quadrule_render::*;
use semver::Version;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0.7")
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

/// Every key in the scene's maps (the objects and the lines' own maps).
fn collect_keys(v: &ciborium::Value, out: &mut Vec<String>) {
    match v {
        ciborium::Value::Map(m) => {
            for (k, x) in m {
                if let Some(k) = k.as_text() {
                    out.push(k.to_string());
                }
                collect_keys(x, out);
            }
        }
        ciborium::Value::Array(a) => a.iter().for_each(|x| collect_keys(x, out)),
        _ => {}
    }
}

/// The PNG a file's page gives on one device, drawn from the outlines saved in the file.
fn export(file: &QuadFile, id: &str) -> Vec<u8> {
    let primary = builtin(&file.doc.primary_profile_id).unwrap();
    let dl = display_list(
        &file.doc,
        &primary,
        &builtin(id).unwrap(),
        &file.outlines[id],
    )
    .unwrap();
    encode_png(&render(&dl, Math::Fastest), None)
}

#[test]
fn v07_files_open_migrate_and_export_the_same_bytes() {
    let sums = saved_sums();
    let mut ok = !sums.is_empty() && FORMAT_VERSION == "0.9.0";
    let mut parts = Vec::new();
    let mut matched = 0;
    for name in ["groups", "shapes"] {
        let bytes = std::fs::read(dir().join(format!("{name}.quad"))).unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        let was_07 = file.migrated_from == Some(Version::parse("0.7.0").unwrap())
            && file.manifest.format_version == "0.7.0";
        let editable = file.read_only.is_none();
        // No object of an old file carries an override, and its page is portrait.
        let plain = file.doc.objects().all(|o| o.overrides.is_empty())
            && file.doc.page.orientation == Orientation::Portrait;
        ok &= was_07 && editable && plain;
        for id in file.doc.profile_ids() {
            let want = sums.iter().find(|(_, n)| *n == format!("{name}_{id}.png"));
            let got = sha256_hex(&export(&file, id));
            let same = want.is_some_and(|(s, _)| *s == got);
            ok &= same;
            matched += usize::from(same);
            parts.push(format!("{name}:{id}:same={same}"));
        }
        // Saved again, the file is the current version and exports the same; no override and no
        // orientation is written for an old page (so a 0.7 build reads it as it was).
        let again = QuadFile::from_bytes(&file.to_bytes("migrate-test").unwrap()).unwrap();
        let resaved = again.manifest.format_version == FORMAT_VERSION
            && again.migrated_from.is_none()
            && again.doc == file.doc;
        let scene: ciborium::Value = ciborium::Value::serialized(&file.doc).unwrap();
        let mut keys = Vec::new();
        collect_keys(&scene, &mut keys);
        let no_new_keys = !keys.is_empty()
            && !keys
                .iter()
                .any(|k| k == "overrides" || k == "orientation" || k == "position");
        ok &= resaved && no_new_keys;
        parts.push(format!(
            "{name}:resaved_080={resaved}:no_new_keys={no_new_keys}"
        ));
        for id in again.doc.profile_ids() {
            ok &= export(&again, id) == export(&file, id);
        }
    }
    println!(
        "\nCHECK crate=render name=migrate_v07 result={} format_version={FORMAT_VERSION} fixtures=2 export_hashes_matched={matched} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_chain_has_the_0_7_to_0_8_step() {
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
    let has_07_to_08 = migrate::CHAIN
        .iter()
        .any(|(f, t, _)| *f == "0.7.0" && *t == "0.8.0");
    let ok = at == current && has_07_to_08;
    println!(
        "\nCHECK crate=render name=migrate_chain_080 result={} from=0.1.0 to={at} steps={steps} has_0_7_to_0_8={has_07_to_08}",
        pass(ok)
    );
    assert!(ok);
}
