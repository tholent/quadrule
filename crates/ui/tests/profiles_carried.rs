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

//! Profiles a `.quad` carries, in the window's state (rung 13a, docs/bbq/2026-11-17-rung-13-profile-system.md,
//! Q4, Q6, Q9): saving carries what is not an unedited built-in, opening where a profile is missing uses the
//! carried copy and offers to add it, and a profile that clashes asks Merge, Rename or Not now. Each test
//! uses folders of its own under the system's temporary folder, never the config folder, and prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::{QuadFile, sha256_hex};
use quadrule_profiles::{Registry, active, builtin};
use quadrule_render::encode_png;
use quadrule_ui::viewer::export_pixels;
use quadrule_ui::{ProfileChoice, Source, Viewer};
use std::path::PathBuf;

const MANTA: &str = "supernote.a5x2";
const NOMAD: &str = "supernote.a6x2";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-ui-carried-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// One computer: a registry in its own folder, made this thread's.
fn computer(name: &str) -> PathBuf {
    let d = folder(name);
    active::set(Registry::at(&d));
    d
}

fn sha(v: &mut Viewer, id: &str) -> String {
    let page = v.page_for(id).unwrap();
    sha256_hex(&encode_png(&export_pixels(page), None))
}

/// The sample page's file as the current computer saves it.
fn saved_sample(targets: &[&str]) -> Vec<u8> {
    let mut v = Viewer::default();
    v.open_sample();
    if !targets.is_empty() {
        v.set_targets(targets).unwrap();
    }
    v.file_bytes().unwrap()
}

fn edit_nomad(guide_gray: u8) {
    let mut p = builtin(NOMAD).unwrap();
    p.line_roles.guide.gray = guide_gray;
    active::with_mut(|r| r.save_override(p)).unwrap();
}

#[test]
fn saving_carries_what_is_not_an_unedited_built_in() {
    let d = computer("carry");
    // Unedited: nothing travels, as before.
    let plain = QuadFile::from_bytes(&saved_sample(&[])).unwrap();
    // An edited Nomad and a profile of the user's own travel.
    edit_nomad(80);
    let id = active::with_mut(|r| r.create(NOMAD, "Kindle Scribe")).unwrap();
    let edited = QuadFile::from_bytes(&saved_sample(&[MANTA, NOMAD, &id])).unwrap();
    let mut ids: Vec<&str> = edited.profiles.keys().map(String::as_str).collect();
    ids.sort();
    let want = {
        let mut w = vec![NOMAD, id.as_str()];
        w.sort();
        w
    };
    let listed: Vec<&str> = edited
        .manifest
        .carried_profiles
        .iter()
        .map(|p| p.id.as_str())
        .collect();
    // The Profiles window's untried values never travel.
    active::with_mut(|r| {
        let mut p = builtin(MANTA).unwrap();
        p.line_roles.guide.gray = 9;
        r.set_preview(p);
    });
    let tried = QuadFile::from_bytes(&saved_sample(&[])).unwrap();
    let ok = plain.profiles.is_empty()
        && plain.manifest.carried_profiles.is_empty()
        && ids == want
        && listed.len() == 2
        && edited.profiles[NOMAD].line_roles.guide.gray == 80
        && !tried.profiles.contains_key(MANTA)
        && d.join(format!("{id}.ron")).is_file();
    println!(
        "CHECK crate=ui name=profiles_carried_on_save result={} plain={} edited={ids:?}",
        pass(ok),
        plain.profiles.len()
    );
    assert!(ok);
}

#[test]
fn a_missing_profile_opens_from_the_carried_copy_and_can_be_added() {
    computer("miss-a");
    let id = active::with_mut(|r| r.create(NOMAD, "Kindle Scribe")).unwrap();
    let bytes = saved_sample(&[MANTA, NOMAD, &id]);
    let want = {
        let mut v = Viewer::default();
        v.open_bytes(Source::Untitled, &bytes).unwrap();
        sha(&mut v, &id)
    };
    // Another computer, which has never heard of it.
    let d = computer("miss-b");
    let mut v = Viewer::default();
    v.open_bytes(Source::Untitled, &bytes).unwrap();
    let u = v.profile_use().unwrap().clone();
    let bar = v.profile_bar();
    let shows = sha(&mut v, &id) == want;
    let names = active::display_name(&id);
    let not_listed = active::device_ids() == vec![MANTA.to_string(), NOMAD.to_string()];
    let added = v.add_missing_profiles().unwrap();
    let after = v.profile_bar().is_none() && v.profile_use().unwrap().missing.is_empty();
    let saved = d.join(format!("{id}.ron")).is_file()
        && active::device_ids().contains(&id)
        && Registry::load(&d).get(&id).is_some();
    let still = sha(&mut v, &id) == want;
    let clean = !v.is_dirty();
    let ok = u.missing == [id.clone()]
        && u.clashes.is_empty()
        && bar.is_some_and(|(m, _)| m == ["Kindle Scribe"])
        && shows
        && names.as_deref() == Some("Kindle Scribe")
        && not_listed
        && added == 1
        && after
        && saved
        && still
        && clean;
    println!(
        "CHECK crate=ui name=profiles_missing_then_added result={} missing={} bar_shown=true added={added} saved_file={saved}",
        pass(ok),
        u.missing.len()
    );
    assert!(ok);
}

#[test]
fn an_edited_builtin_clashes_and_each_choice_does_what_it_says() {
    computer("clash-a");
    edit_nomad(80);
    let bytes = saved_sample(&[]);
    let edited_sha = {
        let mut v = Viewer::default();
        v.open_bytes(Source::Untitled, &bytes).unwrap();
        sha(&mut v, NOMAD)
    };
    let plain_sha = {
        active::set(Registry::builtin_only());
        let mut v = Viewer::default();
        v.open_sample();
        sha(&mut v, NOMAD)
    };
    let differs = edited_sha != plain_sha;

    // Not now: the page shows the file's copy; the bar offers to decide.
    let d = computer("clash-b");
    let mut v = Viewer::default();
    v.open_bytes(Source::Untitled, &bytes).unwrap();
    let asked = v.profile_use().unwrap().clashes.len() == 1;
    let first = sha(&mut v, NOMAD) == edited_sha;
    v.resolve_profile_clashes(&[(NOMAD.into(), ProfileChoice::NotNow)]);
    let deferred = v.profile_use().unwrap().deferred.len() == 1
        && v.profile_bar()
            .is_some_and(|(m, df)| m.is_empty() && df.len() == 1)
        && sha(&mut v, NOMAD) == edited_sha
        && !active::is_edited(NOMAD)
        && !d.join(format!("{NOMAD}.ron")).exists();
    // Asked again from the bar; Merge: the file's values become the local override.
    v.ask_deferred_clashes();
    let again = v.profile_use().unwrap().clashes.len() == 1;
    v.resolve_profile_clashes(&[(NOMAD.into(), ProfileChoice::Merge)]);
    let merged = active::is_edited(NOMAD)
        && d.join(format!("{NOMAD}.ron")).is_file()
        && sha(&mut v, NOMAD) == edited_sha
        && v.profile_bar().is_none()
        && !v.is_dirty();
    // Rename: the file's copy joins mine as a new profile under a new GUID; the page follows it.
    let d2 = computer("clash-c");
    let mut v = Viewer::default();
    v.open_bytes(Source::Untitled, &bytes).unwrap();
    v.resolve_profile_clashes(&[(
        NOMAD.into(),
        ProfileChoice::Rename("Nomad (from file)".into()),
    )]);
    let ids: Vec<String> = v
        .doc()
        .unwrap()
        .profile_ids()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let new_id = ids.iter().find(|i| *i != MANTA && *i != NOMAD).cloned();
    let renamed = new_id.as_ref().is_some_and(|n| {
        uuid_like(n)
            && active::display_name(n).as_deref() == Some("Nomad (from file)")
            && d2.join(format!("{n}.ron")).is_file()
    }) && !ids.contains(&NOMAD.to_string())
        && !active::is_edited(NOMAD)
        && v.is_dirty();
    let renamed_shows = new_id.as_ref().is_some_and(|n| {
        let mut w = v;
        sha(&mut w, n) == edited_sha
    });
    let ok = differs && asked && first && deferred && again && merged && renamed && renamed_shows;
    println!(
        "CHECK crate=ui name=profiles_clash_choices result={} export_differs={differs} asked={asked} shows_copy_first={first} not_now={deferred} merge={merged} rename={renamed} rename_shows_copy={renamed_shows}",
        pass(ok)
    );
    assert!(ok);
}

fn uuid_like(s: &str) -> bool {
    s.len() == 36 && s.matches('-').count() == 4
}

#[test]
fn a_same_name_profile_with_another_id_asks_and_merge_follows_mine() {
    // A: the user's own "Kindle Scribe".
    computer("name-a");
    let a_id = active::with_mut(|r| r.create(NOMAD, "Kindle Scribe")).unwrap();
    active::with_mut(|r| {
        let mut p = r.get(&a_id).unwrap().clone();
        p.line_roles.guide.gray = 100;
        r.save_mine(p)
    })
    .unwrap();
    let bytes = saved_sample(&[MANTA, NOMAD, &a_id]);
    // B: another computer with its own profile of that name (another id, other values).
    computer("name-b");
    let b_id = active::with_mut(|r| r.create(NOMAD, "kindle scribe ")).unwrap();
    let mut v = Viewer::default();
    v.open_bytes(Source::Untitled, &bytes).unwrap();
    let u = v.profile_use().unwrap().clone();
    let asked =
        u.clashes.len() == 1 && u.missing.is_empty() && u.clashes[0].mine == "kindle scribe";
    v.resolve_profile_clashes(&[(a_id.clone(), ProfileChoice::Merge)]);
    let ids: Vec<String> = v
        .doc()
        .unwrap()
        .profile_ids()
        .iter()
        .map(|s| s.to_string())
        .collect();
    // The page now uses B's profile, which took the file's values; B's id and short name stay.
    let followed = ids.contains(&b_id) && !ids.contains(&a_id);
    let mine = active::with(|r| r.get(&b_id).cloned()).unwrap();
    let updated = mine.line_roles.guide.gray == 100 && mine.short_name == "kindle-scribe";
    let ok = asked && followed && updated && v.is_dirty();
    println!(
        "CHECK crate=ui name=profiles_name_clash_merge result={} asked={asked} page_follows={followed} values_updated={updated}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_broken_carried_profile_leaves_that_device_out_and_the_page_still_opens() {
    computer("broken-a");
    let id = active::with_mut(|r| r.create(NOMAD, "Kindle Scribe")).unwrap();
    let bytes = saved_sample(&[MANTA, NOMAD, &id]);
    // The reader's side is covered in crates/format/tests/profiles.rs; here: a file that names a device
    // that is neither here nor carried (the carried copy was skipped).
    let mut stripped = QuadFile::from_bytes(&bytes).unwrap();
    stripped.profiles.clear();
    let bytes2 = stripped.to_bytes("t").unwrap();
    computer("broken-b");
    let mut v = Viewer::default();
    let opened = v.open_bytes(Source::Untitled, &bytes2).is_ok();
    let ids: Vec<String> = v
        .doc()
        .unwrap()
        .profile_ids()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let left = v.profile_use().unwrap().left_out == [id.clone()];
    let page = v.page().is_some();
    let said = v.notice.is_some();
    let unsaved = v.is_dirty();
    // The primary missing: the file is refused, with a message and no panic.
    let mut prim = Viewer::default();
    let refused = {
        computer("broken-c");
        let id2 = active::with_mut(|r| r.create(NOMAD, "Another")).unwrap();
        let mut v2 = Viewer::default();
        v2.open_sample();
        v2.set_targets(&[MANTA, NOMAD, &id2]).unwrap();
        v2.make_primary(&id2).unwrap();
        let mut f = QuadFile::from_bytes(&v2.file_bytes().unwrap()).unwrap();
        f.profiles.clear();
        computer("broken-d");
        prim.open_bytes(Source::Untitled, &f.to_bytes("t").unwrap())
            .is_err()
    };
    let ok = opened && ids == [MANTA, NOMAD] && left && page && said && unsaved && refused;
    println!(
        "CHECK crate=ui name=profiles_carried_missing_left_out result={} opened={opened} left_out={left} said={said} primary_missing_refused={refused}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_bar_and_the_page_tab_name_a_carried_profile_this_computer_lacks() {
    computer("tab-a");
    let id = active::with_mut(|r| r.create(NOMAD, "Kindle Scribe")).unwrap();
    let bytes = saved_sample(&[MANTA, NOMAD, &id]);
    computer("tab-b");
    let mut ui = quadrule_ui::ViewerUi::new(Viewer::default());
    ui.viewer.open_bytes(Source::Untitled, &bytes).unwrap();
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut quadrule_ui::ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    use egui_kittest::kittest::Queryable;
    let bar = h
        .query_all_by_label_contains("Uses a profile you don't have: Kindle Scribe")
        .count()
        > 0;
    let tick = h
        .query_all_by_label_contains("Target Kindle Scribe")
        .count()
        > 0;
    h.get_by_label("Add to my profiles").click();
    h.run();
    let gone = h
        .query_all_by_label_contains("Uses a profile you don't have")
        .count()
        == 0;
    let added = active::device_ids().contains(&id);
    let ok = bar && tick && gone && added;
    println!(
        "CHECK crate=ui name=profiles_bar_and_page_tab result={} bar={bar} page_tab_ticks_it={tick} add_clears_bar={gone}",
        pass(ok)
    );
    assert!(ok);
}
