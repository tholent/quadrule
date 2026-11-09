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
fn estimates_are_marked_unverified() {
    // Rung 1 record Q6: role and gray values are estimates, marked `verified: false`.
    let unverified: Vec<bool> = builtin_ids()
        .map(|id| !builtin(id).unwrap().calibration_meta.verified)
        .collect();
    let ok = unverified.iter().all(|u| *u);
    println!(
        "CHECK crate=profiles name=estimates_unverified result={} profiles={}",
        if ok { "pass" } else { "fail" },
        unverified.len()
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
