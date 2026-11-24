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

//! The gallery's live thumbnails, the Device filter's effect and the built-in starters loader (rung 16 bbq, Q3, Q5 and
//! the Defaulted "Thumbnails, in detail", "The device filter" and "Where starters live"), headless with
//! egui_kittest. The starters are made from the sample documents (`quadrule_ui::test_starters`); every file is in a
//! scratch folder. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_ui::starters;
use quadrule_ui::test_starters::test_starters;
use quadrule_ui::welcome::Category;
use quadrule_ui::{Request, Source, Viewer, ViewerUi};
use std::time::{Duration, Instant};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
    requests: Vec<Request>,
}

fn harness(v: Viewer, starters_made: Vec<(String, String, Vec<u8>)>) -> Harness<'static, Window> {
    let mut ui = ViewerUi::new(v);
    let list: Vec<(&str, &str, &[u8])> = starters_made
        .iter()
        .map(|(f, s, b)| (f.as_str(), s.as_str(), b.as_slice()))
        .collect();
    ui.set_starters(starters::load(&list));
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let r = w.ui.show(ui);
                w.requests.extend(r);
            },
            Window {
                ui,
                requests: Vec::new(),
            },
        );
    h.run();
    h
}

fn has(h: &Harness<'static, Window>, label: &str) -> bool {
    h.query_all_by_label(label).count() > 0
}

fn has_part(h: &Harness<'static, Window>, part: &str) -> bool {
    h.query_all_by_label_contains(part).count() > 0
}

/// Steps the window until nothing is waiting on the drawing thread, or `secs` pass; the thread asks for a repaint
/// when a picture is done, and the test frames stand in for the ones the window would draw.
fn until_drawn(h: &mut Harness<'static, Window>, secs: u64) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        h.step();
        if h.state().ui.thumbs_pending() == 0 && h.state().ui.thumbs_jobs_sent() > 0 {
            h.step();
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn thumbnails_are_drawn_off_the_interface_thread_once_and_an_idle_gallery_asks_for_nothing() {
    let mut h = harness(Viewer::default(), test_starters(6));
    let tiles = h.query_all_by_label_contains("Test starter").count() == 6;
    // The first frame came back with nothing drawn: the page is not rendered on the interface thread.
    let at_first = h.state().ui.thumbs_ready() == 0 && h.state().ui.thumbs_jobs_sent() == 6;
    let done = until_drawn(&mut h, 120);
    let ready = h.state().ui.thumbs_ready() == 6 && h.state().ui.thumbs_failed() == 0;
    let me = std::thread::current().id();
    let drawn_by = h.state().ui.thumbs_drawn_by();
    let elsewhere = drawn_by.len() == 6 && drawn_by.iter().all(|t| *t != me);
    // One thread drew them all.
    let one = drawn_by.iter().all(|t| *t == drawn_by[0]);
    // Idle: more frames hand the thread nothing, and the window settles (no frame is asked for).
    let sent = h.state().ui.thumbs_jobs_sent();
    for _ in 0..10 {
        h.step();
    }
    let settled = h.try_run().is_ok();
    let idle = h.state().ui.thumbs_jobs_sent() == sent && sent == 6;
    // The other device is drawn once; going back to the first costs nothing (the cache has it).
    h.state_mut().ui.set_welcome_device(NOMAD_ID);
    let nomad = until_drawn(&mut h, 120);
    let after_nomad = h.state().ui.thumbs_jobs_sent() == 12 && h.state().ui.thumbs_ready() == 12;
    h.state_mut().ui.set_welcome_device(MANTA_ID);
    h.run();
    h.step();
    let cached = h.state().ui.thumbs_jobs_sent() == 12 && h.state().ui.thumbs_pending() == 0;
    // Another tab hands over no job for tiles already drawn.
    h.get_by_label("Notes").click();
    h.run();
    let tab = h.state().ui.thumbs_jobs_sent() == 12;
    let ok = tiles
        && at_first
        && done
        && ready
        && elsewhere
        && one
        && settled
        && idle
        && nomad
        && after_nomad
        && cached
        && tab;
    println!(
        "CHECK crate=ui name=gallery_thumbnails result={} six_tiles={tiles} nothing_drawn_in_first_frame={at_first} all_drawn={done} ready={ready} not_the_interface_thread={elsewhere} one_thread={one} idle_settles={settled} idle_no_jobs={idle} nomad_drawn_once={nomad} nomad_twelve={after_nomad} back_to_manta_cached={cached} tab_no_new_jobs={tab}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_filters_device_decides_where_a_tile_is_drawn_and_a_page_that_cannot_be_drawn_is_plain() {
    // A starter made for the Manta alone, and one for both.
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Manta only".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
    v.set_targets(&[MANTA_ID]).unwrap();
    let manta_only = v.save_bytes().unwrap();
    let mut both = test_starters(1);
    both[0].0 = "paper".into();
    // A page that lists fine but names a device no profile exists for: it cannot be drawn.
    let file = quadrule_format::QuadFile::from_bytes(&both[0].2).unwrap();
    let doc = quadrule_format::rename_profile(&file.doc, MANTA_ID, "no.such.device").unwrap();
    let outlines =
        quadrule_format::rename_profile_outlines(&file.outlines, MANTA_ID, "no.such.device");
    let in_order: Vec<_> = doc
        .profile_ids()
        .iter()
        .filter_map(|id| outlines.get(*id).cloned())
        .collect();
    let mut broken = quadrule_format::QuadFile::new(doc, in_order, Vec::new());
    broken.doc.metadata.title = "Vanishing".into();
    let broken = broken.to_bytes("test").unwrap();
    let mut starters_made = vec![
        ("paper".to_string(), "manta-only".to_string(), manta_only),
        both.remove(0),
    ];
    starters_made.push(("notes".to_string(), "vanishing".to_string(), broken));
    let mut h = harness(Viewer::default(), starters_made);
    let listed = has(&h, "Vanishing") && has(&h, "Manta only");
    // The filter on the Manta: nothing says "drawn on".
    h.state_mut().ui.set_welcome_device(MANTA_ID);
    h.run();
    let manta_plain = !has_part(&h, "drawn on Manta") && !has_part(&h, "drawn on Supernote Manta");
    // On the Nomad the Manta-only page is drawn on its own device, and its tile says so; the other tile does not.
    h.state_mut().ui.set_welcome_device(NOMAD_ID);
    h.run();
    let says = has_part(&h, "drawn on Manta") || has_part(&h, "drawn on Supernote Manta");
    let finished = until_drawn(&mut h, 120);
    let failed = h.state().ui.thumbs_failed() >= 1 && has(&h, "Vanishing");
    let drawn_on = h.state().ui.thumbs_ready() >= 2;
    // The tile keeps its name and still opens an untitled copy (the file is gone, so the pick says why).
    h.get_by_label("Manta only").click();
    h.run();
    let o = h.state().ui.viewer.open.as_ref();
    let copy = o
        .is_some_and(|o| o.source == Source::Untitled && o.doc.primary_profile_id == NOMAD_ID)
        && !h.state().ui.viewer.is_dirty();
    let ok = listed && manta_plain && says && finished && failed && drawn_on && copy;
    println!(
        "CHECK crate=ui name=gallery_device_filter result={} listed={listed} manta_has_no_note={manta_plain} nomad_has_the_note={says} all_finished={finished} missing_file_plain_tile={failed} others_drawn={drawn_on} pick_makes_filter_device_main={copy}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_tile_that_does_not_target_the_device_says_which_device_it_is_drawn_on() {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Manta only".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
    v.set_targets(&[MANTA_ID]).unwrap();
    let bytes = v.save_bytes().unwrap();
    let list = starters::load(&[("paper", "manta-only", bytes.as_slice())]);
    let entry = &list[0];
    let rule = entry.drawn_on(MANTA_ID) == MANTA_ID && entry.drawn_on(NOMAD_ID) == MANTA_ID;
    let mut h = harness(
        Viewer::default(),
        vec![("paper".into(), "manta-only".into(), bytes)],
    );
    h.state_mut().ui.set_welcome_device(NOMAD_ID);
    h.run();
    h.step();
    let nomad_says = has_part(&h, "drawn on Manta") || has_part(&h, "drawn on Supernote Manta");
    h.state_mut().ui.set_welcome_device(MANTA_ID);
    h.run();
    h.step();
    let manta_silent = !has_part(&h, "drawn on");
    // The drop-down lists the devices, then the paper sizes (it hides nothing).
    h.get_by_label("Device").click();
    h.run();
    let lists = has_part(&h, "Supernote Manta")
        && has_part(&h, "Supernote Nomad")
        && has_part(&h, "ISO A4");
    let ok = rule && nomad_says && manta_silent && lists;
    println!(
        "CHECK crate=ui name=gallery_drawn_on_note result={} rule={rule} nomad_filter_says_manta={nomad_says} manta_filter_silent={manta_silent} filter_lists_devices_and_paper={lists}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_starters_loader_sorts_by_category_and_the_tabs_show_theirs_with_coming_soon_for_the_rest() {
    // What ships (rung 16b leg 2): chris's seven, in tab order and then by name. Planning and Creative have none
    // until leg 3 makes them. Each category the built-in list names is one of the five tabs.
    let shipped_list = starters::built_in();
    let shipped_names: Vec<&str> = shipped_list.iter().map(|e| e.name.as_str()).collect();
    let shipped = shipped_names
        == [
            "Hex grid",
            "Isometric grid",
            "Narrow ruled",
            "Narrow ruled, right-to-left",
            "Square grid",
            "Cornell notes",
            "Rapid log",
        ]
        && starters::BUILT_IN
            .iter()
            .all(|(folder, _, _)| Category::ALL.iter().any(|c| c.folder() == Some(folder)))
        && !starters::BUILT_IN
            .iter()
            .any(|(folder, _, _)| matches!(*folder, "planning" | "creative"));
    let made = test_starters(3);
    let mut list: Vec<(&str, &str, &[u8])> = made
        .iter()
        .map(|(f, s, b)| (f.as_str(), s.as_str(), b.as_slice()))
        .collect();
    // A folder that is not a category is left out; the order is by tab, then by name.
    list.push(("elsewhere", "stray", made[0].2.as_slice()));
    list.reverse();
    let loaded = starters::load(&list);
    let order: Vec<&str> = loaded.iter().map(|e| e.name.as_str()).collect();
    let sorted = loaded.len() == 3
        && order[0].starts_with("Test starter 01")
        && order[1].starts_with("Test starter 02")
        && order[2].starts_with("Test starter 03");
    let mut h = harness(Viewer::default(), made);
    // All lists every starter; Paper only its own; a category with none keeps its coming soon line (Q5).
    let all = h.query_all_by_label_contains("Test starter").count() == 3;
    h.get_by_label("Paper").click();
    h.run();
    let paper = has_part(&h, "Test starter 01")
        && !has_part(&h, "Test starter 02")
        && !has_part(&h, "coming soon");
    h.get_by_label("Bullet journal").click();
    h.run();
    let soon = has_part(&h, "Starter pages for this group are coming soon")
        && !has_part(&h, "Test starter");
    h.get_by_label("Creative").click();
    h.run();
    let soon2 = has_part(&h, "Starter pages for this group are coming soon");
    h.get_by_label("My templates").click();
    h.run();
    let mine = has_part(&h, "No templates yet") && !has_part(&h, "Test starter");
    // Search looks through every tab: by title, tag, description and category name.
    h.get_by_label("Search").click();
    h.get_by_label("Search").type_text("planning");
    h.run();
    let by_category = has_part(&h, "Test starter 03") && !has_part(&h, "Test starter 01");
    h.state_mut().ui.welcome_search_clear();
    h.get_by_label("Search").type_text("shapes");
    h.run();
    let by_tag = has_part(&h, "Test starter 02") && !has_part(&h, "Test starter 03");
    h.state_mut().ui.welcome_search_clear();
    h.state_mut().ui.set_welcome_category(Category::All);
    h.run();
    // Picking a starter opens an untitled copy, as a template does.
    h.get_by_label("All").click();
    h.run();
    h.get_by_label_contains("Test starter 02").click();
    h.run();
    let o = h.state().ui.viewer.open.as_ref();
    let copy = o.is_some_and(|o| {
        o.source == Source::Untitled && o.doc.metadata.title.starts_with("Test starter 02")
    }) && !h.state().ui.viewer.is_dirty()
        && !h.state().ui.welcome_showing();
    let ok =
        shipped && sorted && all && paper && soon && soon2 && mine && by_category && by_tag && copy;
    println!(
        "CHECK crate=ui name=gallery_starters result={} seven_shipped_none_in_planning_or_creative={shipped} sorted_and_stray_left_out={sorted} all_lists_three={all} paper_tab={paper} empty_tab_soon={soon} second_empty_soon={soon2} my_templates_untouched={mine} search_by_category={by_category} search_by_tag={by_tag} pick_opens_untitled_copy={copy}",
        pass(ok)
    );
    assert!(ok);
}
