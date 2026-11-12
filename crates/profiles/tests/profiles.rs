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

//! Profile checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::LineRole;
use quadrule_profiles::*;

#[test]
fn builtins_parse_and_match_napkin() {
    // Napkin §3.3 (napkins/quadrule-spec.md:99-100): Manta 1920 × 2560 at 300 PPI,
    // Nomad 1404 × 1872 at 300 PPI.
    let want = [
        ("supernote.a5x2", 1920, 2560, 300.0),
        ("supernote.a6x2", 1404, 1872, 300.0),
    ];
    let mut fails = Vec::new();
    for (id, w, h, ppi) in want {
        match builtin(id) {
            Ok(p) => {
                if (p.id.as_str(), p.width_px, p.height_px, p.ppi) != (id, w, h, ppi) {
                    fails.push(format!("{id}:values"));
                }
            }
            Err(e) => fails.push(format!("{id}:{e}")),
        }
    }
    let ids: Vec<&str> = builtin_ids().collect();
    let ok = fails.is_empty() && ids == ["supernote.a5x2", "supernote.a6x2"];
    println!(
        "CHECK crate=profiles name=builtins_parse result={} ids={ids:?} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn verified_profiles_have_a_record() {
    // Rung 1 record Q6: estimates are marked `verified: false`. A profile marked verified must
    // say when, and its calibration record must exist; an unverified one has no date.
    let docs =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/device-measurements");
    let mut fails = Vec::new();
    let mut verified = Vec::new();
    for id in builtin_ids() {
        let m = builtin(id).unwrap().calibration_meta;
        let record = docs.join(id).join("calibration.md");
        match (m.verified, &m.date) {
            (true, Some(_)) if record.is_file() => verified.push(id),
            (true, _) => fails.push(format!("{id}:verified_without_date_or_record")),
            (false, Some(_)) => fails.push(format!("{id}:unverified_with_date")),
            (false, None) => {}
        }
    }
    let ok = fails.is_empty();
    println!(
        "CHECK crate=profiles name=verified_have_record result={} verified={verified:?} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn role_grays_are_clean_levels() {
    // Every role gray should be one of the profile's own clean gray levels.
    let mut fails = Vec::new();
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        for role in LineRole::ALL {
            let g = p.role(role).gray;
            if !p.gray_levels.contains(&g) {
                fails.push(format!("{id}:{role:?}:{g}"));
            }
        }
    }
    let ok = fails.is_empty();
    println!(
        "CHECK crate=profiles name=role_grays_clean result={} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn page_size_in_mm() {
    let p = builtin("supernote.a5x2").unwrap();
    let (w, h) = p.page_mm();
    let ok = (w - 162.56).abs() < 1e-9
        && (h - 216.746_666_666_666_7).abs() < 1e-9
        && (p.mm_to_px(w) - 1920.0).abs() < 1e-9;
    println!(
        "CHECK crate=profiles name=page_size_mm result={} w_mm={w:.4} h_mm={h:.4}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn unknown_profile_is_an_error() {
    let ok = matches!(builtin("supernote.nope"), Err(ProfileError::Unknown(_)));
    println!(
        "CHECK crate=profiles name=unknown_profile_error result={}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn short_names_are_file_safe_and_unique() {
    // The `{profile}` token in export file names (napkin §11.2, §11.3): ASCII lowercase letters,
    // digits and hyphens, and no two devices alike, or one export would overwrite another.
    let names: Vec<String> = quadrule_profiles::builtin_ids()
        .map(|id| quadrule_profiles::builtin(id).unwrap().short_name)
        .collect();
    let safe = names.iter().all(|n| {
        !n.is_empty()
            && n.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    });
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    let unique = sorted.len() == names.len();
    let ok = safe && unique;
    println!(
        "CHECK crate=profiles name=short_names result={} names={names:?} safe={safe} unique={unique}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn fill_roles_resolve_per_device() {
    // Rung 6 bbq Q3 and Q7: Shade 204 and Block 102 on both devices (grays sheet, 2026-11-12: Block
    // moved from 68, which looked black; both on the clean levels);
    // a custom gray is itself.
    use quadrule_model::{Fill, FillRole};
    let mut parts = Vec::new();
    let mut ok = true;
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        let (shade, block) = (
            p.fill_gray(Fill::Role(FillRole::Shade)),
            p.fill_gray(Fill::Role(FillRole::Block)),
        );
        let clean = p.gray_levels.contains(&shade) && p.gray_levels.contains(&block);
        let custom = p.fill_gray(Fill::Gray(119)) == 119;
        ok &= shade == 204 && block == 102 && clean && custom && shade > block;
        parts.push(format!("{id}:shade={shade}:block={block}"));
    }
    println!(
        "\nCHECK crate=profiles name=fill_roles result={} {}",
        if ok { "pass" } else { "fail" },
        parts.join(" ")
    );
    assert!(ok);
}
