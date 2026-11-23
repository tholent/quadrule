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

//! Format 0.12.0's migration (rung 11b, docs/bbq/2026-11-14-rung-11-text-and-fonts.md, Defaulted
//! "File format 0.12.0"). The 0.11 file in `tests/fixtures/v0.11/` (the fonts page) was written with
//! format 0.11.0 (the model and text changes of 11b only add fields that are left out of a file when
//! they hold their defaults, so a page that uses none of them is the file 0.11 wrote; its manifest is
//! the 0.11 one), together with the sha256 of the PNG each device's export gave
//! (`tests/fixtures/v0.11/SHA256SUMS`, the fonts goldens of rung 11a). It must open, migrate and export
//! the same bytes now: nothing old changes, so the 0.11 to 0.12 step rewrites nothing, and a resaved
//! file holds none of 0.12's choices. Each test prints one structured `CHECK` line (run with
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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v0.11")
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
    let dl = display_list(
        &file.doc,
        &primary,
        &builtin_for(id, o).unwrap(),
        &file.outlines[id],
    )
    .unwrap();
    let img = render(&dl);
    // A landscape page is exported turned into the device's portrait PNG (rung 8b).
    let img = if o == Orientation::Landscape {
        turn_clockwise(&img)
    } else {
        img
    };
    encode_png(&img, None)
}

#[test]
fn v011_files_open_migrate_and_export_the_same_bytes() {
    let sums = saved_sums();
    let mut ok = !sums.is_empty() && FORMAT_VERSION == "0.20.0";
    let mut parts = Vec::new();
    let mut matched = 0;
    for name in ["fonts"] {
        let bytes = std::fs::read(dir().join(format!("{name}.quad"))).unwrap();
        let file = QuadFile::from_bytes(&bytes).unwrap();
        let was_011 = file.migrated_from == Some(Version::parse("0.11.0").unwrap())
            && file.manifest.format_version == "0.11.0";
        let editable = file.read_only.is_none();
        // A 0.11 file holds none of 0.12's choices: every text box is as tall as its text, with no
        // paragraph space, at the top, baselines off; and the outlines left no line out.
        let plain = file.doc.objects().all(|o| match &o.kind {
            quadrule_model::ObjectKind::Text(t) => {
                t.sizing == quadrule_model::TextSizing::Auto
                    && t.paragraph_spacing.is_none()
                    && t.v_align == quadrule_model::VerticalAlign::Top
                    && !t.baselines
                    && !t.baseline_gap
                    && o.rotation_deg == 0.0
            }
            _ => true,
        }) && file
            .outlines
            .values()
            .all(|p| p.texts.iter().all(|t| t.lines_hidden == 0));
        ok &= was_011 && editable && plain;
        for id in file.doc.profile_ids() {
            let want = sums.iter().find(|(_, n)| *n == format!("{name}_{id}.png"));
            let got = sha256_hex(&export(&file, id));
            let same = want.is_some_and(|(s, _)| *s == got);
            ok &= same;
            matched += usize::from(same);
            parts.push(format!("{name}:{id}:same={same}"));
        }
        // Saved again, the file is the current version and exports the same.
        let again = QuadFile::from_bytes(&file.to_bytes("migrate-test").unwrap()).unwrap();
        let resaved = again.manifest.format_version == FORMAT_VERSION
            && again.migrated_from.is_none()
            && again.doc == file.doc;
        ok &= resaved;
        parts.push(format!("{name}:resaved_0120={resaved}"));
        for id in again.doc.profile_ids() {
            ok &= export(&again, id) == export(&file, id);
        }
    }
    println!(
        "\nCHECK crate=render name=migrate_v011 result={} format_version={FORMAT_VERSION} fixtures=1 export_hashes_matched={matched} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_chain_has_the_0_11_to_0_12_step() {
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
        .any(|(f, t, _)| *f == "0.11.0" && *t == "0.12.0");
    let ok = at == current && has_step;
    println!(
        "\nCHECK crate=render name=migrate_chain_0120 result={} from=0.1.0 to={at} steps={steps} has_0_11_to_0_12={has_step}",
        pass(ok)
    );
    assert!(ok);
}
