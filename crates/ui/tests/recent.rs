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

//! Recent files (rung 16 bbq, Defaulted "Recent files"): added at Open, Save and Save As, newest first, at most
//! ten, kept in `settings.ron`, and a file that has gone shows as missing. Headless; every file and every settings
//! file is in a scratch folder. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::MANTA_ID;
use quadrule_ui::settings::MAX_RECENT;
use quadrule_ui::{Settings, Viewer};
use std::path::{Path, PathBuf};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn new_page(v: &mut Viewer, title: &str) {
    v.open_new(&NewPage {
        title: title.into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
}

fn names(v: &Viewer) -> Vec<String> {
    v.recent_files().into_iter().map(|r| r.name).collect()
}

#[test]
fn open_save_and_save_as_add_files_newest_first_and_the_list_is_kept() {
    let dir = tmp("recent-basic");
    let cfg = dir.join("settings.ron");
    let mut v = Viewer::default();
    v.use_settings_file(cfg.clone());
    // Save As adds; so does opening.
    new_page(&mut v, "One");
    v.save_to(&dir.join("one.quad")).unwrap();
    new_page(&mut v, "Two");
    v.save_to(&dir.join("two.quad")).unwrap();
    let after_saves = names(&v) == ["two.quad", "one.quad"];
    v.open_path(&dir.join("one.quad")).unwrap();
    let opened_first = names(&v) == ["one.quad", "two.quad"];
    // Saving the open file again keeps one line for it; Save As to a new name adds a second.
    v.save_to(&dir.join("one.quad")).unwrap();
    let no_twin = names(&v) == ["one.quad", "two.quad"];
    v.save_to(&dir.join("three.quad")).unwrap();
    let as_new = names(&v) == ["three.quad", "one.quad", "two.quad"];
    // A file that does not open is not added.
    let _ = v.open_path(&dir.join("nothing.quad"));
    let failed_not_added = names(&v).len() == 3;
    // The list is in settings.ron and a new run reads it back; the line shows name and folder.
    let saved = Settings::load(&cfg).settings;
    let kept = saved.recent == v.settings.recent && saved.recent.len() == 3;
    let line = &v.recent_files()[0];
    let shape =
        line.name == "three.quad" && line.folder == dir.display().to_string() && line.exists;
    let ok = after_saves && opened_first && no_twin && as_new && failed_not_added && kept && shape;
    println!(
        "CHECK crate=ui name=recent_open_save_save_as result={} saves_add={after_saves} open_moves_first={opened_first} no_twin={no_twin} save_as_adds={as_new} failed_open_not_added={failed_not_added} kept_in_settings={kept} name_and_folder={shape}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn ten_at_most_missing_ones_stay_and_forget_takes_one_off() {
    let dir = tmp("recent-ten");
    let cfg = dir.join("settings.ron");
    let mut v = Viewer::default();
    v.use_settings_file(cfg.clone());
    new_page(&mut v, "Page");
    for i in 0..12 {
        v.save_to(&dir.join(format!("p{i:02}.quad"))).unwrap();
    }
    let list = names(&v);
    let ten = list.len() == MAX_RECENT && list[0] == "p11.quad" && list[9] == "p02.quad";
    // A file gone from the disk stays, marked as not there.
    std::fs::remove_file(dir.join("p11.quad")).unwrap();
    let r = v.recent_files();
    let missing = !r[0].exists && r[1].exists && r.len() == MAX_RECENT;
    // Forgetting takes just that one off, and the settings file follows.
    v.forget_recent(&dir.join("p11.quad"));
    let forgotten = names(&v).len() == 9 && !names(&v).contains(&"p11.quad".to_string());
    let on_disk = Settings::load(&cfg).settings.recent.len() == 9;
    let ok = ten && missing && forgotten && on_disk;
    println!(
        "CHECK crate=ui name=recent_ten_missing_forget result={} ten_newest={ten} missing_stays_marked={missing} forget_one={forgotten} forget_saved={on_disk}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_older_settings_file_without_a_recent_list_still_loads() {
    // A file written by rung 15 has no `recent` field; it takes the empty list and keeps what it had.
    let old = r#"(template: "{title}.png", set: "x", note: false, devices: This, folders: [], last_folder: None, completion_note_open: true)"#;
    let s = Settings::parse(old);
    let ok = s.as_ref().is_ok_and(|s| {
        s.recent.is_empty() && s.template == "{title}.png" && s.set == "x" && s.completion_note_open
    });
    // A relative path is kept absolute, so the list means the same wherever the app starts.
    let mut v = Viewer::default();
    v.note_recent(Path::new("some/relative.quad"));
    let absolute = Path::new(&v.settings.recent[0]).is_absolute();
    let all = ok && absolute;
    println!(
        "CHECK crate=ui name=recent_old_settings_load result={} old_file_loads={ok} relative_kept_absolute={absolute}",
        pass(all)
    );
    assert!(all);
}
