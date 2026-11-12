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

//! Format 0.7.0's migration (rung 7a, docs/bbq/2026-11-11-rung-7-groups-and-align.md, Defaulted "File
//! format 0.7.0 (7a)"). The two 0.6 files in `tests/fixtures/v0.6/` (the transforms page, and the
//! slice page with its text) were written by the 0.6 code, before any 0.7 change, together with the
//! sha256 of the PNG each device's export gave (`tests/fixtures/v0.6/SHA256SUMS`). Every one must
//! open, migrate and export the same bytes now. Each test prints one structured `CHECK` line (run
//! with `-- --nocapture`).

use quadrule_format::{FORMAT_VERSION, QuadFile, migrate, sha256_hex};
use quadrule_layout::display_list;
use quadrule_profiles::builtin;
use quadrule_render::*;
use semver::Version;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0.6")
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
fn v06_files_open_migrate_and_export_the_same_bytes() {
    let sums = saved_sums();
    let mut ok = !sums.is_empty() && FORMAT_VERSION == "0.7.0";
    let mut parts = Vec::new();
    let mut matched = 0;
    for name in ["transforms", "slice"] {
        let bytes = std::fs::read(dir().join(format!("{name}.quad"))).unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        let was_06 = file.migrated_from == Some(Version::parse("0.6.0").unwrap())
            && file.manifest.format_version == "0.6.0";
        let editable = file.read_only.is_none();
        // No object of an old file is a group.
        let plain = file.doc.objects().all(|o| !o.is_group());
        ok &= was_06 && editable && plain;
        for id in file.doc.profile_ids() {
            let want = sums.iter().find(|(_, n)| *n == format!("{name}_{id}.png"));
            let got = sha256_hex(&export(&file, id));
            let same = want.is_some_and(|(s, _)| *s == got);
            ok &= same;
            matched += usize::from(same);
            parts.push(format!("{name}:{id}:same={same}"));
        }
        // Saved again, the file is the current version and exports the same; no group is written
        // for an old page (so a 0.6 build reads it as it was).
        let again = QuadFile::from_bytes(&file.to_bytes("migrate-test").unwrap()).unwrap();
        let resaved = again.manifest.format_version == FORMAT_VERSION
            && again.migrated_from.is_none()
            && again.doc == file.doc;
        let scene: ciborium::Value = ciborium::Value::serialized(&file.doc).unwrap();
        let mut keys = Vec::new();
        collect_keys(&scene, &mut keys);
        let no_new_keys = !keys.is_empty() && !keys.iter().any(|k| k == "children" || k == "Group");
        ok &= resaved && no_new_keys;
        parts.push(format!(
            "{name}:resaved_070={resaved}:no_new_keys={no_new_keys}"
        ));
        for id in again.doc.profile_ids() {
            ok &= export(&again, id) == export(&file, id);
        }
    }
    println!(
        "\nCHECK crate=render name=migrate_v06 result={} format_version={FORMAT_VERSION} fixtures=2 export_hashes_matched={matched} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_chain_has_the_0_6_to_0_7_step() {
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
    let has_06_to_07 = migrate::CHAIN
        .iter()
        .any(|(f, t, _)| *f == "0.6.0" && *t == "0.7.0");
    let ok = at == current && has_06_to_07;
    println!(
        "\nCHECK crate=render name=migrate_chain_070 result={} from=0.1.0 to={at} steps={steps} has_0_6_to_0_7={has_06_to_07}",
        pass(ok)
    );
    assert!(ok);
}
