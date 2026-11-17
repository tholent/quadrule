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

//! The profile registry and its storage (rung 13a bbq, Q6 to Q9 and Defaulted "Limits", "Where they
//! live"). Each test prints one structured `CHECK` line (run with `-- --nocapture`). Tests use a folder of
//! their own under the system's temporary folder, never the user's config folder.

use quadrule_model::Margins;
use quadrule_profiles::*;
use std::path::PathBuf;

const MANTA: &str = "supernote.a5x2";
const NOMAD: &str = "supernote.a6x2";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// A fresh empty folder for one test.
fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-profiles-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn nomad() -> Profile {
    builtin(NOMAD).unwrap()
}

#[test]
fn every_damaged_shape_is_refused_without_a_panic() {
    let good = to_ron(&nomad());
    let mut cases: Vec<(&str, String)> = Vec::new();
    cases.push(("empty", String::new()));
    cases.push(("cut_short", good[..good.len() / 2].to_string()));
    cases.push(("cut_short_tiny", "(".to_string()));
    cases.push(("typo_field", good.replace("width_px", "widht_px")));
    cases.push(("typo_value", good.replace("ppi: 300.0", "ppi: three")));
    cases.push(("missing_field", good.replace("fill_roles", "fill_roless")));
    cases.push((
        "wrong_type",
        good.replace("width_px: 1404", "width_px: \"wide\""),
    ));
    cases.push(("negative", good.replace("width_px: 1404", "width_px: -5")));
    cases.push(("gray_over_255", good.replace("shade: 204", "shade: 300")));
    cases.push(("not_ron", "just some words".to_string()));
    cases.push((
        "binary",
        String::from_utf8_lossy(&[0u8, 159, 146, 150, 255]).into_owned(),
    ));
    cases.push((
        "deep",
        format!("{}{}", "(".repeat(100_000), ")".repeat(100_000)),
    ));
    cases.push((
        "deep_array",
        format!("{}{}", "[".repeat(100_000), "]".repeat(100_000)),
    ));
    cases.push((
        "large",
        format!("{good}\n// {}", "x".repeat(2 * 1024 * 1024)),
    ));
    let mut bad = Vec::new();
    for (name, src) in &cases {
        let a = parse_checked(src);
        let b = from_ron_checked(src);
        if a.is_ok() || b.is_ok() {
            bad.push(*name);
        }
    }
    let ok = bad.is_empty();
    println!(
        "CHECK crate=profiles name=damaged_shapes result={} cases={} accepted_wrongly={bad:?}",
        pass(ok),
        cases.len()
    );
    assert!(ok);
}

#[test]
fn a_typo_error_names_its_line() {
    let good = to_ron(&nomad());
    let src = good.replace("ppi: 300.0", "ppi: three");
    let e = parse_checked(&src).unwrap_err();
    let line = e.line.unwrap_or(0);
    let want = src.lines().position(|l| l.contains("ppi: three")).unwrap() + 1;
    let ok = line == want;
    println!(
        "CHECK crate=profiles name=error_has_line result={} line={line} want={want} error={:?}",
        pass(ok),
        e.message
    );
    assert!(ok);
}

#[test]
fn each_limit_is_enforced_and_the_edge_is_accepted() {
    type Edit = Box<dyn Fn(&mut Profile)>;
    let edits: Vec<(&str, Edit, Edit)> = vec![
        (
            "width",
            Box::new(|p| p.width_px = 15),
            Box::new(|p| {
                p.width_px = 16;
                p.safe_zones.clear();
                p.default_margins_mm = Margins::all(0.0);
            }),
        ),
        (
            "height_big",
            Box::new(|p| p.height_px = 16_385),
            Box::new(|p| p.height_px = 16_384),
        ),
        (
            "ppi_low",
            Box::new(|p| p.ppi = 49.9),
            Box::new(|p| p.ppi = 50.0),
        ),
        (
            "ppi_high",
            Box::new(|p| p.ppi = 1_200.1),
            Box::new(|p| p.ppi = 1_200.0),
        ),
        (
            "ppi_nan",
            Box::new(|p| p.ppi = f64::NAN),
            Box::new(|p| p.ppi = 72.0),
        ),
        (
            "role_zero",
            Box::new(|p| p.line_roles.guide.width_px = 0),
            Box::new(|p| p.line_roles.guide.width_px = 1),
        ),
        (
            "role_wide",
            Box::new(|p| p.line_roles.dot_grid.width_px = 33),
            Box::new(|p| p.line_roles.dot_grid.width_px = 32),
        ),
        (
            "zone_outside",
            Box::new(|p| p.safe_zones[0].rect = (1400, 0, 10, 10)),
            Box::new(|p| p.safe_zones[0].rect = (1394, 0, 10, 10)),
        ),
        (
            "zone_empty",
            Box::new(|p| p.safe_zones[0].rect.2 = 0),
            Box::new(|p| p.safe_zones[0].rect.2 = 1),
        ),
        (
            "zones_many",
            Box::new(|p| {
                let z = p.safe_zones[0].clone();
                p.safe_zones = vec![z; 65];
            }),
            Box::new(|p| {
                let z = p.safe_zones[0].clone();
                p.safe_zones = vec![z; 64];
            }),
        ),
        (
            "name_long",
            Box::new(|p| p.display_name = "n".repeat(81)),
            Box::new(|p| p.display_name = "n".repeat(80)),
        ),
        (
            "name_empty",
            Box::new(|p| p.display_name = "  ".into()),
            Box::new(|p| p.display_name = "a".into()),
        ),
        (
            "short_upper",
            Box::new(|p| p.short_name = "Nomad".into()),
            Box::new(|p| p.short_name = "nomad-2".into()),
        ),
        (
            "short_empty",
            Box::new(|p| p.short_name = String::new()),
            Box::new(|p| p.short_name = "a".into()),
        ),
        (
            "id_slash",
            Box::new(|p| p.id = "../x".into()),
            Box::new(|p| p.id = "x.y_z-1".into()),
        ),
        (
            "margin_negative",
            Box::new(|p| p.default_margins_mm.top = -1.0),
            Box::new(|p| p.default_margins_mm.top = 0.0),
        ),
        (
            "margin_whole_page",
            Box::new(|p| {
                p.default_margins_mm.left = 100.0;
                p.default_margins_mm.right = 100.0;
            }),
            Box::new(|p| {
                p.default_margins_mm.left = 10.0;
                p.default_margins_mm.right = 10.0;
            }),
        ),
        (
            "orientations_none",
            Box::new(|p| p.orientations.clear()),
            Box::new(|p| p.orientations = vec![Orientation::Portrait]),
        ),
        (
            "notes_long",
            Box::new(|p| p.calibration_meta.method = "m".repeat(2_001)),
            Box::new(|p| p.calibration_meta.method = "m".repeat(2_000)),
        ),
    ];
    let mut bad = Vec::new();
    for (name, wrong, right) in &edits {
        let mut p = nomad();
        wrong(&mut p);
        // Out-of-limit values are damage whether they come in a file or from the form.
        if limits::check(&p).is_ok() || from_ron_checked(&to_ron(&p)).is_ok() {
            bad.push(format!("{name}:wrong_accepted"));
        }
        let mut p = nomad();
        right(&mut p);
        if limits::check(&p).is_err() || from_ron_checked(&to_ron(&p)).is_err() {
            bad.push(format!("{name}:edge_refused"));
        }
    }
    let shipped_ok = builtin_ids().all(|id| limits::check(&builtin(id).unwrap()).is_ok());
    let ok = bad.is_empty() && shipped_ok;
    println!(
        "CHECK crate=profiles name=limits result={} cases={} shipped_inside_limits={shipped_ok} bad={bad:?}",
        pass(ok),
        edits.len()
    );
    assert!(ok);
}

#[test]
fn a_profile_round_trips_through_text() {
    let mut fails = Vec::new();
    for id in builtin_ids() {
        let p = builtin(id).unwrap();
        match from_ron_checked(&to_ron(&p)) {
            Ok(back) if back == p => {}
            _ => fails.push(id),
        }
        if to_ron(&p) != to_ron(&p) {
            fails.push("not repeatable");
        }
    }
    let ok = fails.is_empty();
    println!(
        "CHECK crate=profiles name=round_trip result={} fails={fails:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn load_lists_damaged_files_and_keeps_the_good_ones() {
    let d = folder("load");
    let mut mine = nomad();
    mine.id = "11111111-2222-4333-8444-555555555555".into();
    mine.display_name = "Kindle Scribe".into();
    mine.short_name = "scribe".into();
    std::fs::write(d.join(format!("{}.ron", mine.id)), to_ron(&mine)).unwrap();
    let good = to_ron(&nomad());
    std::fs::write(d.join("a-cut.ron"), &good[..good.len() / 3]).unwrap();
    std::fs::write(
        d.join("b-typo.ron"),
        good.replace("ppi: 300.0", "ppi: three"),
    )
    .unwrap();
    std::fs::write(
        d.join("c-limit.ron"),
        good.replace("width_px: 1404", "width_px: 9"),
    )
    .unwrap();
    std::fs::write(d.join("d-binary.ron"), [0u8, 159, 146, 150, 255]).unwrap();
    std::fs::write(d.join("e-big.ron"), vec![b' '; 3 * 1024 * 1024]).unwrap();
    std::fs::write(d.join("f-wrongname.ron"), &good).unwrap();
    std::fs::write(d.join("notes.txt"), "not a profile").unwrap();
    std::fs::create_dir(d.join("sub.ron")).unwrap();
    let r = Registry::load(&d);
    let files: Vec<&str> = r.damaged().iter().map(|x| x.file.as_str()).collect();
    let with_line = r.damaged().iter().filter(|x| x.line.is_some()).count();
    let ok = files
        == [
            "a-cut.ron",
            "b-typo.ron",
            "c-limit.ron",
            "d-binary.ron",
            "e-big.ron",
            "f-wrongname.ron",
        ]
        && with_line >= 2
        && r.get(&mine.id) == Some(&mine)
        && r.device_ids() == vec![MANTA.to_string(), NOMAD.to_string(), mine.id.clone()]
        && !r.is_edited(NOMAD);
    println!(
        "CHECK crate=profiles name=load_damaged result={} damaged={} with_line={with_line} mine_loaded={}",
        pass(ok),
        files.len(),
        r.get(&mine.id).is_some()
    );
    assert!(ok, "{files:?}");
    // A folder that is not there, or is a file, is an empty one.
    assert!(Registry::load(&d.join("nope")).damaged().is_empty());
    assert!(Registry::load(&d.join("notes.txt")).damaged().is_empty());
}

#[test]
fn an_override_wins_and_a_broken_one_falls_back_marked() {
    let d = folder("override");
    let mut edited = nomad();
    edited.line_roles.guide.gray = 80;
    std::fs::write(d.join(format!("{NOMAD}.ron")), to_ron(&edited)).unwrap();
    let mut broken = builtin(MANTA).unwrap();
    broken.width_px = 1000; // a built-in stays the same device
    std::fs::write(d.join(format!("{MANTA}.ron")), to_ron(&broken)).unwrap();
    let r = Registry::load(&d);
    let ok = r.get(NOMAD) == Some(&edited)
        && r.is_edited(NOMAD)
        && r.source(NOMAD) == Some(Source::Edited)
        && r.get(MANTA) == Some(&builtin(MANTA).unwrap())
        && r.has_broken_override(MANTA)
        && !r.is_edited(MANTA)
        && r.damaged().len() == 1;
    // builtin() itself never looks at the folder.
    let shipped_unchanged =
        builtin(NOMAD).unwrap() == nomad() && nomad().line_roles.guide.gray == 51;
    println!(
        "CHECK crate=profiles name=override_and_fallback result={} edited={} broken_marked={} builtin_unchanged={shipped_unchanged}",
        pass(ok && shipped_unchanged),
        r.is_edited(NOMAD),
        r.has_broken_override(MANTA)
    );
    assert!(ok && shipped_unchanged);
}

#[test]
fn save_override_reset_and_equal_values() {
    let d = folder("save");
    let mut r = Registry::at(&d);
    let mut p = nomad();
    p.line_roles.guide.gray = 80;
    p.width_px = 5; // locked: put back
    r.save_override(p).unwrap();
    let file = d.join(format!("{NOMAD}.ron"));
    let written = std::fs::read_to_string(&file).unwrap();
    let back = from_ron_checked(&written).unwrap();
    let on_disk = Registry::load(&d);
    let leftovers = std::fs::read_dir(&d)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".saving")
        })
        .count();
    let ok1 = back.line_roles.guide.gray == 80
        && back.width_px == 1404
        && r.is_edited(NOMAD)
        && on_disk.get(NOMAD) == r.get(NOMAD)
        && leftovers == 0;
    // Saving the shipped values again leaves no override.
    r.save_override(nomad()).unwrap();
    let ok2 = !r.is_edited(NOMAD) && !file.exists();
    // Reset removes the file, even a damaged one.
    std::fs::write(&file, "(").unwrap();
    let mut r = Registry::load(&d);
    let ok3 = r.has_broken_override(NOMAD) && r.damaged().len() == 1;
    r.reset_builtin(NOMAD).unwrap();
    let ok4 = !file.exists() && r.damaged().is_empty() && !r.has_broken_override(NOMAD);
    let ok = ok1 && ok2 && ok3 && ok4;
    println!(
        "CHECK crate=profiles name=save_override_reset result={} written={ok1} equal_removes={ok2} broken_listed={ok3} reset_removes={ok4}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn my_profiles_save_delete_and_keep_short_names_unique() {
    let d = folder("mine");
    let mut r = Registry::at(&d);
    let id = r.create(NOMAD, "Kindle Scribe").unwrap();
    let id2 = r.create(NOMAD, "Kindle Scribe").unwrap();
    let a = r.get(&id).unwrap().clone();
    let b = r.get(&id2).unwrap().clone();
    let uuid_ok = uuid::Uuid::parse_str(&id).is_ok_and(|u| u.get_version_num() == 4);
    let unverified = !a.calibration_meta.verified && a.calibration_meta.date.is_none();
    let names_ok = a.short_name == "kindle-scribe" && b.short_name == "kindle-scribe-2";
    let n3 = r.suggest_short_name("Supernote Nomad!", None);
    let taken_refused = {
        let mut c = a.clone();
        c.short_name = b.short_name.clone();
        r.save_mine(c).is_err()
            && r.save_mine({
                let mut c = a.clone();
                c.short_name = "nomad".into();
                c
            })
            .is_err()
    };
    let nomad_name = r.suggest_short_name("Nomad", None) == "nomad-2";
    let reloaded = Registry::load(&d);
    let persisted = reloaded.get(&id) == Some(&a) && reloaded.get(&id2) == Some(&b);
    r.delete_mine(&id).unwrap();
    let deleted = r.get(&id).is_none() && !d.join(format!("{id}.ron")).exists();
    let builtin_refused = r.delete_mine(NOMAD).is_err() && r.save_mine(nomad()).is_err();
    let ok = uuid_ok
        && unverified
        && names_ok
        && taken_refused
        && nomad_name
        && persisted
        && deleted
        && builtin_refused
        && n3 == "supernote-nomad";
    println!(
        "CHECK crate=profiles name=my_profiles result={} uuid_v4={uuid_ok} unverified={unverified} short_names={names_ok} taken_refused={taken_refused} persisted={persisted} deleted={deleted}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn no_folder_means_a_clear_error_not_a_panic() {
    let mut r = Registry::builtin_only();
    let e = r.create(NOMAD, "X").unwrap_err();
    let ok = e.contains("no folder") && r.mine().is_empty();
    println!(
        "CHECK crate=profiles name=no_folder result={} error={e:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn incoming_profiles_are_classified_and_resolved() {
    let d = folder("incoming");
    let mut r = Registry::at(&d);
    let id = r.create(NOMAD, "Kindle Scribe").unwrap();
    let mine = r.get(&id).unwrap().clone();
    // Same id, same values.
    let same = r.classify(&mine) == Incoming::Same;
    // Same id, other values: Replace mine / Keep both.
    let mut changed = mine.clone();
    changed.line_roles.guide.gray = 10;
    let idc = r.classify(&changed) == Incoming::IdClash;
    // Another id, same name (ignoring case and outer spaces).
    let mut theirs = mine.clone();
    theirs.id = "other-id".into();
    theirs.display_name = "  kindle SCRIBE ".into();
    let namec = r.classify(&theirs) == Incoming::NameClash(id.clone());
    let mut fresh = mine.clone();
    fresh.id = "fresh".into();
    fresh.display_name = "Boox".into();
    let new = r.classify(&fresh) == Incoming::New;
    // A built-in's id with other values is an id clash; its own values are the same.
    let mut e = nomad();
    e.line_roles.guide.gray = 9;
    let builtin_clash =
        r.classify(&e) == Incoming::IdClash && r.classify(&nomad()) == Incoming::Same;
    // Another id named like a built-in is a name clash with it.
    let mut like = nomad();
    like.id = "like-nomad".into();
    like.short_name = "like".into();
    let builtin_name = r.classify(&like) == Incoming::NameClash(NOMAD.into());

    // Replace mine.
    r.replace(changed.clone()).unwrap();
    let replaced = r.get(&id).unwrap().line_roles.guide.gray == 10;
    // Keep both: a new GUID, the same values.
    let both = r.keep_both(mine.clone()).unwrap();
    let kept = both != id
        && r.get(&both).unwrap().line_roles == mine.line_roles
        && uuid::Uuid::parse_str(&both).is_ok();
    // Merge: the incoming values update mine; id and short name stay.
    let mut inc = mine.clone();
    inc.id = "elsewhere".into();
    inc.short_name = "else".into();
    inc.line_roles.whisper.gray = 99;
    let target = r.merge(&inc, &id).unwrap();
    let merged = target == id
        && r.get(&id).unwrap().line_roles.whisper.gray == 99
        && r.get(&id).unwrap().short_name == mine.short_name
        && r.get("elsewhere").is_none();
    // Merge into a built-in: it becomes the override; Reset brings the shipped values back.
    let mut inc2 = nomad();
    inc2.id = "somebody".into();
    inc2.width_px = 100; // locked: ignored
    inc2.line_roles.emphasis.gray = 20;
    r.merge(&inc2, NOMAD).unwrap();
    let into_builtin = r.is_edited(NOMAD)
        && r.get(NOMAD).unwrap().line_roles.emphasis.gray == 20
        && r.get(NOMAD).unwrap().width_px == 1404;
    r.reset_builtin(NOMAD).unwrap();
    let reset = !r.is_edited(NOMAD) && r.get(NOMAD) == Some(&nomad());
    // Rename: keeps a free id, takes a new one when the id is known.
    let mut free = fresh.clone();
    free.id = "free-id".into();
    let fid = r.rename_incoming(&free, "Boox Note").unwrap();
    let kept_id = fid == "free-id" && r.get(&fid).unwrap().display_name == "Boox Note";
    let nid = r.rename_incoming(&mine, "Scribe 2").unwrap();
    let new_id = nid != id
        && r.get(&nid).unwrap().display_name == "Scribe 2"
        && r.get(&id).unwrap().display_name == "Kindle Scribe";
    let ok = same
        && idc
        && namec
        && new
        && builtin_clash
        && builtin_name
        && replaced
        && kept
        && merged
        && into_builtin
        && reset
        && kept_id
        && new_id;
    println!(
        "CHECK crate=profiles name=incoming result={} same={same} id_clash={idc} name_clash={namec} new={new} replace={replaced} keep_both={kept} merge={merged} merge_builtin={into_builtin} reset={reset} rename_free={kept_id} rename_taken={new_id}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn lookup_order_and_carried_copies() {
    let mut r = Registry::builtin_only();
    let mut c = nomad();
    c.id = "carried-one".into();
    c.display_name = "Carried".into();
    let mut c2 = nomad();
    c2.line_roles.guide.gray = 1;
    r.set_carried(vec![c.clone(), c2.clone()], Default::default());
    let a = r.get("carried-one") == Some(&c) && r.source("carried-one") == Some(Source::Carried);
    // A carried copy of a built-in's id loses to what the computer has, unless put first (Not now).
    let b = r.get(NOMAD) == Some(&nomad());
    r.set_carried(vec![c2.clone()], [NOMAD.to_string()].into());
    let c_ = r.get(NOMAD) == Some(&c2);
    r.clear_carried();
    let d_ = r.get("carried-one").is_none() && r.get(NOMAD) == Some(&nomad());
    let ok = a && b && c_ && d_;
    println!(
        "CHECK crate=profiles name=lookup_order result={} carried={a} stored_wins={b} not_now_first={c_} cleared={d_}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_active_registry_is_per_thread_and_defaults_to_the_shipped_values() {
    let start = active::lookup(NOMAD).unwrap() == nomad() && active::device_ids() == [MANTA, NOMAD];
    let mut r = Registry::builtin_only();
    let mut e = nomad();
    e.line_roles.guide.gray = 7;
    r.set_carried(vec![e.clone()], [NOMAD.to_string()].into());
    active::set(r);
    let here = active::lookup(NOMAD).unwrap() == e;
    let other = std::thread::spawn(|| active::lookup(NOMAD).unwrap() == nomad())
        .join()
        .unwrap();
    let landscape = active::lookup_for(NOMAD, Orientation::Landscape)
        .unwrap()
        .turned;
    let unknown = matches!(active::lookup("nope"), Err(ProfileError::Unknown(_)));
    let ok = start && here && other && landscape && unknown;
    println!(
        "CHECK crate=profiles name=active_per_thread result={} default={start} set={here} other_thread_default={other}",
        pass(ok)
    );
    assert!(ok);
}
