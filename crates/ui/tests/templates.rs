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

//! My templates without a window (rung 16 bbq, Q4, and Defaulted "Save as template…" and "A starter or template
//! becomes a document as an untitled copy"): Save as template… writes `templates/<slug>.quad` through Save's code
//! and leaves the open document alone; a template opens as an untitled copy on the chosen device. Every file is in a
//! scratch folder. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::{QuadFile, preview_path};
use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_ui::templates::{TemplateError, list_templates};
use quadrule_ui::{Source, Viewer};
use std::path::PathBuf;

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

#[test]
fn save_as_template_writes_the_slug_asks_before_replacing_and_leaves_the_document_alone() {
    let dir = tmp("templates-save");
    let mut v = Viewer::default();
    v.templates_dir = Some(dir.join("templates"));
    // Nothing open, and no folder, are told apart.
    let nothing = v.save_as_template(false) == Err(TemplateError::NothingOpen);
    new_page(&mut v, "Weekly Planner 2027");
    let mut bare = Viewer::default();
    new_page(&mut bare, "x");
    let no_folder = bare.save_as_template(false) == Err(TemplateError::NoFolder);
    // The first save makes the folder and writes the slug.
    let want = dir.join("templates").join("weekly-planner-2027.quad");
    let dirty_before = v.is_dirty();
    let wrote = v.save_as_template(false) == Ok(want.clone()) && want.is_file();
    // It is an ordinary .quad with the main device's picture in it (Save's code, rung 15 Q8).
    let file = QuadFile::from_bytes(&std::fs::read(&want).unwrap()).unwrap();
    let picture = file.previews.contains_key(&preview_path(MANTA_ID))
        && file.doc.metadata.title == "Weekly Planner 2027";
    // The open document is as it was: no file, no unsaved changes made, nothing in the recent list.
    let alone = v
        .open
        .as_ref()
        .is_some_and(|o| o.source == Source::Untitled)
        && v.is_dirty() == dirty_before
        && v.settings.recent.is_empty();
    let said = v
        .notice
        .as_ref()
        .is_some_and(|n| !n.error && n.message.id == "notice-template-saved");
    // A second save of the same name asks; Replace takes its place.
    let first = std::fs::read(&want).unwrap();
    let asks = v.save_as_template(false) == Err(TemplateError::Exists(want.clone()));
    let untouched = std::fs::read(&want).unwrap() == first;
    v.edit(
        quadrule_model::edit::Command::SetUnits(quadrule_model::Unit::In),
        None,
    )
    .unwrap();
    let replaced = v.save_as_template(true) == Ok(want.clone())
        && std::fs::read(&want).unwrap() != first
        && v.is_dirty();
    let ok =
        nothing && no_folder && wrote && picture && alone && said && asks && untouched && replaced;
    println!(
        "CHECK crate=ui name=template_save result={} nothing_open_told={nothing} no_folder_told={no_folder} slug_written={wrote} picture_and_title={picture} document_alone={alone} notice={said} exists_asks={asks} ask_leaves_file={untouched} replace_takes_place={replaced}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_folder_is_listed_by_name_and_only_a_template_can_be_removed() {
    let dir = tmp("templates-list");
    let t = dir.join("templates");
    let mut v = Viewer::default();
    v.templates_dir = Some(t.clone());
    new_page(&mut v, "Zebra");
    v.save_as_template(false).unwrap();
    v.open_new(&NewPage {
        title: "apple".into(),
        device_id: NOMAD_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
    v.save_as_template(false).unwrap();
    // A file that is not a .quad, a half-written one and a damaged .quad.
    std::fs::write(t.join("notes.txt"), b"x").unwrap();
    std::fs::write(t.join(".zebra.quad.saving"), b"x").unwrap();
    std::fs::write(t.join("broken.quad"), b"not a zip").unwrap();
    let list = v.templates();
    let names: Vec<&str> = list.iter().map(|e| e.name.as_str()).collect();
    let by_name = names == ["apple", "broken", "Zebra"];
    let apple = &list[0];
    let facts = apple.primary == NOMAD_ID
        && apple.devices.len() >= 2
        && apple
            .preview
            .as_ref()
            .is_some_and(|p| p.starts_with(b"\x89PNG"))
        && !apple.unreadable;
    let broken = list[1].unreadable && list[1].preview.is_none();
    let none_there = list_templates(&dir.join("nowhere")).is_empty();
    // Removal: inside the folder only.
    let outside = dir.join("outside.quad");
    std::fs::write(&outside, b"x").unwrap();
    let refused = v.remove_template(&outside).is_err()
        && outside.is_file()
        && v.remove_template(&t.join("notes.txt")).is_err();
    let removed = v.remove_template(&t.join("zebra.quad")).is_ok()
        && !t.join("zebra.quad").exists()
        && v.templates().len() == 2;
    let ok = by_name && facts && broken && none_there && refused && removed;
    println!(
        "CHECK crate=ui name=template_list_remove result={} by_name={by_name} apple_facts={facts} damaged_is_plain={broken} missing_folder_empty={none_there} outside_refused={refused} remove_inside={removed}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_template_opens_as_an_untitled_copy_on_the_chosen_device() {
    let dir = tmp("templates-copy");
    let t = dir.join("templates");
    let mut v = Viewer::default();
    v.templates_dir = Some(t.clone());
    new_page(&mut v, "Dot grid");
    // A template made for the Manta alone: the Nomad is not one of its devices.
    v.set_targets(&[MANTA_ID]).unwrap();
    let path = v.save_as_template(false).unwrap();
    let before = std::fs::read(&path).unwrap();
    let made = quadrule_ui::templates::TemplateEntry::read(&path);
    let alone = made.devices == [MANTA_ID];
    // Its own device: opens as it is, untitled, with no unsaved changes, dated now.
    let mut w = Viewer::default();
    w.open_template(&path, MANTA_ID).unwrap();
    let o = w.open.as_ref().unwrap();
    let plain = o.source == Source::Untitled
        && !w.is_dirty()
        && o.doc.metadata.title == "Dot grid"
        && o.doc.primary_profile_id == MANTA_ID
        && w.notice.is_none();
    let recent_alone = w.settings.recent.is_empty();
    // The Nomad: added to the page's devices, then made the main one; still no unsaved changes, and not undoable.
    w.open_template(&path, NOMAD_ID).unwrap();
    let o = w.open.as_ref().unwrap();
    let moved = o.doc.primary_profile_id == NOMAD_ID
        && o.doc.profile_ids().contains(&MANTA_ID)
        && w.profile_id == NOMAD_ID
        && !w.is_dirty()
        && !o.history.can_undo()
        && w.notice.is_none();
    // The Nomad's page can be made: the picture of the main device is the Nomad's.
    let page = w.page_for(NOMAD_ID).is_ok();
    // A device that is not there: the copy opens on its own main device and says why.
    w.open_template(&path, "no.such.device").unwrap();
    let o = w.open.as_ref().unwrap();
    let fallback = o.doc.primary_profile_id == MANTA_ID
        && w.profile_id == MANTA_ID
        && w.notice
            .as_ref()
            .is_some_and(|n| n.message.id == "notice-copy-device");
    // A paper size works the same way.
    w.open_template(&path, quadrule_model::sample::PAPER_A5_ID)
        .unwrap();
    let paper = w
        .open
        .as_ref()
        .is_some_and(|o| o.doc.primary_profile_id == quadrule_model::sample::PAPER_A5_ID);
    // A file that cannot be read leaves what was open alone and says so.
    let junk = dir.join("junk.quad");
    std::fs::write(&junk, b"junk").unwrap();
    let kept = w.open_template(&junk, MANTA_ID).is_err()
        && w.open
            .as_ref()
            .is_some_and(|o| o.doc.primary_profile_id == quadrule_model::sample::PAPER_A5_ID)
        && w.notice.as_ref().is_some_and(|n| n.error);
    // The template file is never changed by opening it.
    let intact = std::fs::read(&path).unwrap() == before;
    let ok = alone && plain && recent_alone && moved && page && fallback && paper && kept && intact;
    println!(
        "CHECK crate=ui name=template_open_copy result={} manta_only={alone} untitled_clean_dated={plain} not_in_recent={recent_alone} nomad_added_and_main={moved} page_made={page} bad_device_falls_back={fallback} paper_size={paper} unreadable_keeps_document={kept} template_untouched={intact}",
        pass(ok)
    );
    assert!(ok);
}
