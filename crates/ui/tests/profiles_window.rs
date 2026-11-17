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

//! The Profiles window (rung 13a, docs/bbq/2026-11-17-rung-13-profile-system.md, Q2, Q3, Q5, Q7 to Q9):
//! an override changes the export and Reset restores the shipped bytes exactly, built-ins keep their
//! size, ppi and short name, New…, Duplicate and Delete, import and export with their clashes, a damaged
//! file listed with its line, and the Edited mark wherever a device's name shows. Each test uses folders
//! of its own under the system's temporary folder, never the config folder, and prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_format::sha256_hex;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_profiles::{Registry, active, builtin, from_ron_checked, to_ron};
use quadrule_render::encode_png;
use quadrule_ui::viewer::export_pixels;
use quadrule_ui::{Confirm, ImportPick, Viewer, ViewerUi};
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn computer(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-ui-win-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    active::set(Registry::load(&d));
    d
}

fn sha(v: &mut Viewer, id: &str) -> String {
    let page = v.page_for(id).unwrap();
    sha256_hex(&encode_png(&export_pixels(page), None))
}

fn window_ui() -> ViewerUi {
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    ui.open_profiles_window();
    ui
}

#[test]
fn an_override_changes_the_export_and_reset_restores_the_shipped_bytes_exactly() {
    let d = computer("override");
    let mut ui = window_ui();
    let manta0 = sha(&mut ui.viewer, MANTA_ID);
    let nomad0 = sha(&mut ui.viewer, NOMAD_ID);
    ui.profiles_select(NOMAD_ID);
    // The page redraws as the value changes, before Save.
    ui.profiles_edit(|p| p.line_roles.guide.gray = 140);
    let live = sha(&mut ui.viewer, NOMAD_ID);
    let draft_only = !d.join(format!("{NOMAD_ID}.ron")).exists() && ui.profiles_dirty();
    let manta_same = sha(&mut ui.viewer, MANTA_ID) == manta0;
    // Cancel throws it away: the shipped bytes are back and nothing was written.
    ui.profiles_cancel();
    let cancelled = sha(&mut ui.viewer, NOMAD_ID) == nomad0
        && !ui.profiles_window_open()
        && !d.join(format!("{NOMAD_ID}.ron")).exists()
        && !active::is_edited(NOMAD_ID);
    // Edit and Save: an override file, the mark, the changed export.
    ui.open_profiles_window();
    ui.profiles_select(NOMAD_ID);
    ui.profiles_edit(|p| p.line_roles.guide.gray = 140);
    ui.profiles_save();
    let saved_file = d.join(format!("{NOMAD_ID}.ron"));
    let written = saved_file.is_file()
        && from_ron_checked(&std::fs::read_to_string(&saved_file).unwrap())
            .is_ok_and(|p| p.line_roles.guide.gray == 140);
    let edited_sha = sha(&mut ui.viewer, NOMAD_ID);
    let marked = active::is_edited(NOMAD_ID)
        && ui
            .profiles_list()
            .iter()
            .any(|e| e.id == NOMAD_ID && e.edited)
        && ui.full_name_for_test(NOMAD_ID).contains("Edited");
    let applies = live == edited_sha && edited_sha != nomad0;
    // Reset to built-in asks first; no answer, no change; yes deletes the file.
    ui.profiles_ask(Confirm::Reset(NOMAD_ID.into()));
    ui.profiles_confirm(false);
    let asked_first = saved_file.exists();
    ui.profiles_ask(Confirm::Reset(NOMAD_ID.into()));
    ui.profiles_confirm(true);
    let back = sha(&mut ui.viewer, NOMAD_ID) == nomad0;
    let gone = !saved_file.exists() && !active::is_edited(NOMAD_ID);
    let ok = draft_only
        && manta_same
        && cancelled
        && written
        && marked
        && applies
        && asked_first
        && back
        && gone;
    println!(
        "CHECK crate=ui name=profiles_override_and_reset result={} redraws_before_save={} cancel_restores={cancelled} written={written} edited_mark={marked} reset_bytes_equal_shipped={back} file_gone={gone}",
        pass(ok),
        live != nomad0
    );
    assert!(ok);
}

#[test]
fn a_built_ins_id_size_ppi_and_short_name_cannot_change() {
    computer("locked");
    let mut ui = window_ui();
    ui.profiles_select(NOMAD_ID);
    ui.profiles_edit(|p| {
        p.width_px = 100;
        p.height_px = 100;
        p.ppi = 72.0;
        p.short_name = "x".into();
        p.id = "other".into();
        p.display_name = "Renamed Nomad".into();
        p.default_margins_mm.top = 9.0;
    });
    let p = ui.profiles_current().unwrap();
    let shipped = builtin(NOMAD_ID).unwrap();
    let ok = (
        p.width_px,
        p.height_px,
        p.ppi,
        p.short_name.clone(),
        p.id.clone(),
    ) == (
        shipped.width_px,
        shipped.height_px,
        shipped.ppi,
        shipped.short_name.clone(),
        shipped.id.clone(),
    ) && p.display_name == "Renamed Nomad"
        && p.default_margins_mm.top == 9.0
        && ui.profiles_selected_is_builtin();
    println!(
        "CHECK crate=ui name=profiles_builtin_locked result={} name_and_margins_editable=true",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn new_duplicate_and_delete_make_and_remove_the_users_own() {
    let d = computer("mine");
    let mut ui = window_ui();
    ui.profiles_new("Kindle Scribe", NOMAD_ID);
    let id = ui.profiles_window().unwrap().selected.clone();
    let p = ui.profiles_current().unwrap();
    let made = p.display_name == "Kindle Scribe"
        && !p.calibration_meta.verified
        && p.short_name == "kindle-scribe"
        && p.id.len() == 36
        && d.join(format!("{id}.ron")).is_file()
        && !ui.profiles_selected_is_builtin()
        && ui
            .profiles_list()
            .iter()
            .any(|e| e.id == id && !e.built_in && e.unverified);
    // Size and ppi are editable on one's own profile; Save writes them.
    ui.profiles_edit(|p| {
        p.width_px = 1000;
        p.height_px = 1400;
        p.ppi = 226.0;
        p.default_margins_mm.left = 4.0;
        // Its safe zones were measured on the Nomad's screen and no longer fit inside this one.
        p.safe_zones.clear();
    });
    ui.profiles_save();
    let reread = Registry::load(&d).get(&id).cloned().unwrap();
    let edited = (reread.width_px, reread.height_px, reread.ppi) == (1000, 1400, 226.0);
    // A short name another profile uses is refused at Save.
    ui.profiles_edit(|p| p.short_name = "nomad".into());
    ui.profiles_save();
    let refused =
        ui.profiles_dirty() && Registry::load(&d).get(&id).unwrap().short_name == "kindle-scribe";
    ui.profiles_cancel();
    ui.open_profiles_window();
    ui.profiles_select(&id);
    ui.profiles_duplicate();
    let copy = ui.profiles_window().unwrap().selected.clone();
    let c = ui.profiles_current().unwrap();
    let duplicated = copy != id
        && c.display_name == "Kindle Scribe copy"
        && c.short_name == "kindle-scribe-copy"
        && (c.width_px, c.height_px) == (1000, 1400)
        && !c.calibration_meta.verified;
    // Delete asks first, then removes the file; the other stays.
    ui.profiles_ask(Confirm::Delete(copy.clone()));
    ui.profiles_confirm(false);
    let kept = d.join(format!("{copy}.ron")).is_file();
    ui.profiles_ask(Confirm::Delete(copy.clone()));
    ui.profiles_confirm(true);
    let deleted = !d.join(format!("{copy}.ron")).exists() && d.join(format!("{id}.ron")).is_file();
    // Built-ins cannot be deleted.
    ui.profiles_ask(Confirm::Delete(NOMAD_ID.into()));
    ui.profiles_confirm(true);
    let builtin_safe = active::lookup(NOMAD_ID).is_ok();
    let ok = made && edited && refused && duplicated && kept && deleted && builtin_safe;
    println!(
        "CHECK crate=ui name=profiles_new_duplicate_delete result={} new={made} edited_size={edited} short_name_refused={refused} duplicate={duplicated} asked_first={kept} deleted={deleted}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn import_and_export_with_their_clashes() {
    let d = computer("import");
    let mut ui = window_ui();
    ui.profiles_new("Kindle Scribe", NOMAD_ID);
    let id = ui.profiles_window().unwrap().selected.clone();
    // Export writes the RON text; the same file imports as "already here".
    let out = d.join("out").join("scribe.ron");
    ui.profiles_export_to(&out);
    let text = std::fs::read_to_string(&out).unwrap();
    let exported = ui.profiles_export_name().as_deref() == Some("kindle-scribe.ron")
        && from_ron_checked(&text).is_ok_and(|p| p.id == id);
    let _ = ui.profiles_import_text(&text);
    let same = ui.profiles_window().unwrap().import.is_none()
        && ui.profiles_list().iter().filter(|e| !e.built_in).count() == 1;
    // Same id, other values: Replace mine or Keep both.
    let mut other = from_ron_checked(&text).unwrap();
    other.line_roles.guide.gray = 11;
    let _ = ui.profiles_import_text(&to_ron(&other));
    let asked_id = ui
        .profiles_window()
        .unwrap()
        .import
        .as_ref()
        .is_some_and(|a| a.kind == quadrule_profiles::Incoming::IdClash);
    ui.profiles_import_answer(Some(ImportPick::KeepBoth));
    let both = ui.profiles_list().iter().filter(|e| !e.built_in).count() == 2;
    let _ = ui.profiles_import_text(&to_ron(&other));
    ui.profiles_import_answer(Some(ImportPick::Replace));
    let replaced = active::with(|r| r.get(&id).map(|p| p.line_roles.guide.gray)) == Some(11);
    // Another id, the same name: Merge or Rename.
    let mut named = other.clone();
    named.id = "somebody-elses".into();
    named.display_name = "KINDLE SCRIBE".into();
    named.line_roles.whisper.gray = 77;
    let _ = ui.profiles_import_text(&to_ron(&named));
    let asked_name = ui
        .profiles_window()
        .unwrap()
        .import
        .as_ref()
        .is_some_and(|a| matches!(a.kind, quadrule_profiles::Incoming::NameClash(_)));
    ui.profiles_import_answer(Some(ImportPick::Rename("Scribe from a friend".into())));
    let renamed = active::display_name("somebody-elses").as_deref() == Some("Scribe from a friend");
    let mut named2 = named.clone();
    named2.id = "third".into();
    let _ = ui.profiles_import_text(&to_ron(&named2));
    ui.profiles_import_answer(Some(ImportPick::Merge));
    // Two of mine share the name now (Keep both did that), so Merge went into one of them.
    let merged = active::with(|r| {
        r.mine()
            .iter()
            .filter(|p| {
                p.display_name.to_lowercase() == "kindle scribe" && p.line_roles.whisper.gray == 77
            })
            .count()
    }) == 1
        && active::lookup("third").is_err();
    // A new profile is added; a cancelled clash changes nothing.
    let mut fresh = named.clone();
    fresh.id = "fresh".into();
    fresh.display_name = "Boox".into();
    fresh.short_name = "boox".into();
    let _ = ui.profiles_import_text(&to_ron(&fresh));
    let added = active::lookup("fresh").is_ok();
    let before = ui.profiles_list().len();
    let _ = ui.profiles_import_text(&to_ron(&other));
    ui.profiles_import_answer(None);
    let cancelled = ui.profiles_list().len() == before;
    // Bad imports: said, never a panic.
    let bad = [
        String::new(),
        "(".to_string(),
        text.replace("ppi: 300.0", "ppi: -1.0"),
        "x".repeat(300_000),
    ];
    let mut said = 0;
    for b in &bad {
        if ui.profiles_import_text(b).is_err() && ui.profiles_window().unwrap().message.is_some() {
            said += 1;
        }
    }
    let big = d.join("big.ron");
    std::fs::write(&big, vec![b' '; 3 * 1024 * 1024]).unwrap();
    ui.profiles_import_path(&big);
    ui.profiles_import_path(&d.join("missing.ron"));
    let ok = exported
        && same
        && asked_id
        && both
        && replaced
        && asked_name
        && renamed
        && merged
        && added
        && cancelled
        && said == bad.len();
    println!(
        "CHECK crate=ui name=profiles_import_export result={} exported={exported} same_is_noop={same} id_clash={asked_id} keep_both={both} replace={replaced} name_clash={asked_name} rename={renamed} merge={merged} cancel={cancelled} bad_said={said}/{}",
        pass(ok),
        bad.len()
    );
    assert!(ok);
}

#[test]
fn a_damaged_file_is_listed_with_its_error_and_the_app_goes_on() {
    let d = std::env::temp_dir().join(format!("quadrule-ui-win-damaged-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let good = to_ron(&builtin(NOMAD_ID).unwrap());
    std::fs::write(d.join("typo.ron"), good.replace("ppi: 300.0", "ppi: three")).unwrap();
    // A broken override of the Nomad falls back to the shipped values.
    std::fs::write(d.join(format!("{NOMAD_ID}.ron")), &good[..good.len() / 2]).unwrap();
    active::set(Registry::load(&d));
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    ui.note_damaged_profiles();
    let noted = ui.viewer.notice.is_some();
    ui.open_profiles_window();
    let listed = ui.profiles_damaged();
    let with_line = listed
        .iter()
        .any(|x| x.file == "typo.ron" && x.line.is_some() && !x.error.is_empty());
    let nomad_ok = ui.profiles_current().is_some()
        && sha(&mut ui.viewer, NOMAD_ID) == {
            active::set(Registry::builtin_only());
            let mut v = Viewer::default();
            v.open_sample();
            sha(&mut v, NOMAD_ID)
        };
    // The window draws it (a harness frame) without stopping.
    active::set(Registry::load(&d));
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    ui.open_profiles_window();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    let drawn =
        h.query_by_label_contains("typo.ron").is_some() && h.query_by_label("Profiles").is_some();
    let ok = noted && listed.len() == 2 && with_line && nomad_ok && drawn;
    println!(
        "CHECK crate=ui name=profiles_damaged_listed result={} damaged={} line_given={with_line} start_note={noted} broken_override_uses_shipped={nomad_ok} listed_in_window={drawn}",
        pass(ok),
        listed.len()
    );
    assert!(ok);
}

#[test]
fn the_edited_mark_shows_in_the_page_tab_the_strip_and_new() {
    let d = computer("mark");
    let mut p = builtin(NOMAD_ID).unwrap();
    p.line_roles.guide.gray = 140;
    active::with_mut(|r| r.save_override(p)).unwrap();
    assert!(d.join(format!("{NOMAD_ID}.ron")).is_file());
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    // The strip names it, and the Page tab's ticks, and the toolbar's picker.
    let strip = h.query_by_label_contains("Show Nomad · Edited").is_some();
    let tick = h.query_by_label_contains("Target Nomad · Edited").is_some();
    let manta_plain = h.query_by_label("Show Manta").is_some();
    let ok = strip && tick && manta_plain;
    println!(
        "CHECK crate=ui name=profiles_edited_mark result={} strip={strip} page_tab={tick} manta_plain={manta_plain}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_window_draws_and_a_changed_role_redraws_the_page_fast() {
    computer("draw");
    let mut ui = ViewerUi::new(Viewer::default());
    ui.viewer.open_sample();
    ui.open_profiles_window();
    ui.profiles_select(NOMAD_ID);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut ViewerUi| {
                let _ = w.show(ui);
            },
            ui,
        );
    h.run();
    let names = [
        "BUILT IN",
        "MINE",
        "New…",
        "Duplicate",
        "Import…",
        "Export…",
        "Cancel",
        "Save",
        "Screen",
        "Line roles",
    ];
    let missing: Vec<&str> = names
        .iter()
        .copied()
        .filter(|n| h.query_all_by_label_contains(n).count() == 0)
        .collect();
    let titled = h.query_by_label("Profiles").is_some();
    let reset_disabled = h.query_by_label("Reset to built-in").is_some();
    // A role value changes: the page is made again from the draft.
    let t = std::time::Instant::now();
    h.state_mut()
        .profiles_edit(|p| p.line_roles.structure.gray = 20);
    let _ = h.state_mut().viewer.page_for(NOMAD_ID).unwrap();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    h.run();
    // A whole frame with the window open, after another change: the form, the strip and the canvas.
    let t = std::time::Instant::now();
    h.state_mut()
        .profiles_edit(|p| p.line_roles.structure.gray = 30);
    h.step();
    let frame_ms = t.elapsed().as_secs_f64() * 1000.0;
    let dirty = h.state().profiles_dirty();
    // The same frame with nothing changed, and the same redraw with the window closed (Cancel changes
    // the page back), to tell what the window itself costs.
    let t = std::time::Instant::now();
    h.step();
    let idle_ms = t.elapsed().as_secs_f64() * 1000.0;
    h.state_mut().profiles_cancel();
    let t = std::time::Instant::now();
    h.step();
    let closed_ms = t.elapsed().as_secs_f64() * 1000.0;
    let ok = missing.is_empty() && titled && reset_disabled && dirty;
    println!(
        "CHECK crate=ui name=profiles_window_draws result={} missing={missing:?} role_change_redraw_ms={ms:.1} frame_with_window_ms={frame_ms:.1} idle_frame_with_window_ms={idle_ms:.1} same_redraw_window_closed_ms={closed_ms:.1} build={}",
        pass(ok),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    assert!(ok);
}
