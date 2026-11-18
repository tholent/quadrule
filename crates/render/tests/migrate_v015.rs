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

//! Format 0.16.0's migration (rung 15 bbq, Q8). The 0.15 file in `tests/fixtures/v0.15/` (the overrides page with a
//! third device and a profile of the user's own that the file carries, as `v0.14/` has) was written by the 0.15.0
//! build, before the format changed, together with the sha256 of the PNG each device's export gave
//! (`tests/fixtures/v0.15/SHA256SUMS`). It must open, migrate and export the same bytes now: the 0.15 to 0.16 step
//! rewrites nothing, and the file has no preview (it stays valid). A preview put in survives a save and a read,
//! and the exported pixels do not move. Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::{FORMAT_VERSION, QuadFile, migrate, preview_path, sha256_hex};
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0.15")
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
    let img = render(&dl, Math::Fastest);
    let img = if o == Orientation::Landscape {
        turn_clockwise(&img)
    } else {
        img
    };
    encode_png(&img, None)
}

#[test]
fn v015_files_open_migrate_and_export_the_same_bytes() {
    let sums = saved_sums();
    let mut ok = !sums.is_empty() && FORMAT_VERSION == "0.17.0";
    let mut parts = Vec::new();
    let mut matched = 0;
    for name in ["devices"] {
        let bytes = std::fs::read(dir().join(format!("{name}.quad"))).unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        let was_015 = file.migrated_from == Some(Version::parse("0.15.0").unwrap())
            && file.manifest.format_version == "0.15.0";
        let editable = file.read_only.is_none();
        let carried = file.profiles.len() == 1
            && file.damaged_profiles.is_empty()
            && file.manifest.carried_profiles.len() == 1;
        // A 0.15 file has no preview and is no less valid for it.
        let no_preview = file.previews.is_empty();
        ok &= was_015 && editable && carried && no_preview;
        for id in file.doc.profile_ids() {
            let want = sums.iter().find(|(_, n)| *n == format!("{name}_{id}.png"));
            let got = sha256_hex(&export(&file, id));
            let same = want.is_some_and(|(s, _)| *s == got);
            ok &= same;
            matched += usize::from(same);
            parts.push(format!("{name}:{id}:same={same}"));
        }
        // Saved again, the file is the current version, still has no preview, and exports the same.
        let again = QuadFile::from_bytes(&file.to_bytes("migrate-test").unwrap()).unwrap();
        let resaved = again.manifest.format_version == FORMAT_VERSION
            && again.migrated_from.is_none()
            && again.doc == file.doc
            && again.profiles == file.profiles
            && again.previews.is_empty();
        ok &= resaved;
        parts.push(format!("{name}:resaved_0160={resaved}"));
        for id in again.doc.profile_ids() {
            ok &= export(&again, id) == export(&file, id);
        }
    }
    println!(
        "\nCHECK crate=render name=migrate_v015 result={} format_version={FORMAT_VERSION} fixtures=1 export_hashes_matched={matched} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_preview_is_saved_read_back_and_never_changes_the_page() {
    let bytes = std::fs::read(dir().join("devices.quad")).unwrap();
    let mut file = QuadFile::from_bytes(&bytes).unwrap();
    let primary = file.doc.primary_profile_id.clone();
    let before = sha256_hex(&export(&file, &primary));
    // A stand-in picture: 3 x 4 gray.
    let thumb = encode_png(
        &Gray {
            width: 3,
            height: 4,
            data: (0..12).map(|i| i * 20).collect(),
        },
        None,
    );
    file.previews.insert(preview_path(&primary), thumb.clone());
    let back = QuadFile::from_bytes(&file.to_bytes("t").unwrap()).unwrap();
    let kept = back.previews.get(&preview_path(&primary)) == Some(&thumb)
        && back.manifest.entries.contains_key(&preview_path(&primary));
    let same_pixels = sha256_hex(&export(&back, &primary)) == before && back.doc == file.doc;

    let ok = kept && same_pixels;
    println!(
        "\nCHECK crate=render name=preview_saved result={} survives_save={kept} export_unchanged={same_pixels}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_chain_has_the_0_15_to_0_16_step() {
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
        .any(|(f, t, _)| *f == "0.15.0" && *t == "0.16.0");
    let ok = at == current && has_step;
    println!(
        "\nCHECK crate=render name=migrate_chain_0160 result={} from=0.1.0 to={at} steps={steps} has_0_15_to_0_16={has_step}",
        pass(ok)
    );
    assert!(ok);
}
