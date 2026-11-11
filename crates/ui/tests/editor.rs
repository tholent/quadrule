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

//! The editor's state, headless (rung 3 bbq, Defaulted "Tests"). Each test prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use quadrule_model::MM_PER_IN;
use quadrule_model::sample::*;
use quadrule_ui::{Autosave, ExportWhich, Recovery, Source, Toward, Viewer};
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn sample() -> Viewer {
    let mut v = Viewer::default();
    v.open_sample();
    v
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Manta pixels for a length in mm.
fn px(mm: f64) -> f64 {
    mm * 300.0 / MM_PER_IN
}

fn x_of(v: &Viewer, id: quadrule_model::Uuid) -> f64 {
    v.doc().unwrap().object(id).unwrap().frame.x_mm
}

#[test]
fn click_selects_the_topmost_pickable_object() {
    let mut v = sample();
    // The rule under the title, at y = 22 mm; 3 px off still picks it (slack).
    v.click(px(50.0), px(22.0) + 3.0, 4.0, false);
    let rule = v.selection == vec![ids::RULE];
    // Shift-click adds the dot grid, Shift-click again takes it out.
    v.click(px(30.0), px(60.0), 4.0, true);
    let added = v.selection == vec![ids::RULE, ids::DOTS];
    v.click(px(30.0), px(60.0), 4.0, true);
    let removed = v.selection == vec![ids::RULE];
    // A click on nothing (the left margin, 2 mm in) clears it.
    v.click(px(2.0), px(60.0), 4.0, false);
    let cleared = v.selection.is_empty();
    // A hidden or locked object is passed over.
    v.set_object_panel(ids::DOTS, |o| o.locked = true).unwrap();
    let locked_skipped = v.pick(px(30.0), px(60.0), 4.0).is_none();
    v.set_object_panel(ids::DOTS, |o| {
        o.locked = false;
        o.visible = false;
    })
    .unwrap();
    let hidden_skipped = v.pick(px(30.0), px(60.0), 4.0).is_none();
    let ok = rule && added && removed && cleared && locked_skipped && hidden_skipped;
    println!(
        "CHECK crate=ui name=click_selects result={} line_with_slack={rule} shift_adds={added} shift_removes={removed} empty_clears={cleared} locked_skipped={locked_skipped} hidden_skipped={hidden_skipped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn nudge_moves_by_a_device_pixel_or_a_rhythm_unit() {
    let mut v = sample();
    v.selection = vec![ids::RULE];
    let x0 = x_of(&v, ids::RULE);
    v.nudge(1.0, 0.0, false).unwrap();
    let one_px = ((x_of(&v, ids::RULE) - x0) - MM_PER_IN / 300.0).abs() < 1e-9;
    // Shift: one rhythm unit as the Manta resolves 6 mm (71 px).
    v.nudge(-1.0, 0.0, true).unwrap();
    let unit =
        ((x_of(&v, ids::RULE) - x0) - (MM_PER_IN / 300.0 - 71.0 * MM_PER_IN / 300.0)).abs() < 1e-9;
    // On the Nomad the pixel is the Nomad's.
    let nomad_ppi = quadrule_profiles::builtin(NOMAD_ID).unwrap().ppi;
    v.set_profile(NOMAD_ID);
    let before = x_of(&v, ids::RULE);
    v.nudge(1.0, 0.0, false).unwrap();
    let nomad_px = ((x_of(&v, ids::RULE) - before) - MM_PER_IN / nomad_ppi).abs() < 1e-9;
    // Three steps, three undos.
    let three = v.undo() && v.undo() && v.undo() && x_of(&v, ids::RULE) == x0;
    // Locked: refused, with a notice.
    v.set_object_panel(ids::RULE, |o| o.locked = true).unwrap();
    v.nudge(1.0, 0.0, false).unwrap();
    let locked = x_of(&v, ids::RULE) == x0 && v.notice.as_ref().is_some_and(|n| n.error);
    let ok = one_px && unit && nomad_px && three && locked;
    println!(
        "CHECK crate=ui name=nudge result={} one_device_px={one_px} rhythm_unit={unit} shown_device_px={nomad_px} undo_each={three} locked_refused={locked}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn delete_duplicate_and_reorder() {
    let mut v = sample();
    v.selection = vec![ids::DOTS];
    v.duplicate_selection().unwrap();
    let copy = v.selection.clone();
    let doc = v.doc().unwrap();
    let dup = copy.len() == 1
        && copy[0] != ids::DOTS
        && doc.layers[0]
            .objects
            .iter()
            .map(|o| o.id)
            .collect::<Vec<_>>()
            == vec![ids::DOTS, copy[0], ids::LINES];
    // Move the copy up past the ruled lines, then back down.
    v.reorder_selection(Toward::Front).unwrap();
    let up = v.doc().unwrap().layers[0].objects[2].id == copy[0];
    v.reorder_selection(Toward::Front).unwrap();
    let stays_at_top = v.doc().unwrap().layers[0].objects[2].id == copy[0];
    v.delete_selection().unwrap();
    let deleted = v.doc().unwrap().object(copy[0]).is_none() && v.selection.is_empty();
    // Layers move too.
    v.reorder_layer(0, Toward::Front).unwrap();
    let layer = v.doc().unwrap().layers[1].name == "Grid";
    // Undo it all: the sample again.
    while v.undo() {}
    let back = *v.doc().unwrap() == slice_document() && !v.is_dirty();
    let ok = dup && up && stays_at_top && deleted && layer && back;
    println!(
        "CHECK crate=ui name=delete_duplicate_reorder result={} duplicate_above={dup} move_up={up} top_stays={stays_at_top} delete={deleted} layer_moves={layer} undo_all_clean={back}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn save_round_trip() {
    // Open → edit → save → reopen → same document (rung 3 bbq, Defaulted "Tests").
    let dir = tmp("save_round_trip");
    let path = dir.join("edited.quad");
    let mut v = sample();
    let no_file = v.file_path().is_none();
    v.selection = vec![ids::TITLE];
    v.edit_selected(None, |o| {
        if let quadrule_model::ObjectKind::Text(t) = &mut o.kind {
            t.content = "Edited title".into();
        }
    })
    .unwrap();
    v.nudge(0.0, 5.0, false).unwrap();
    let dirty = v.is_dirty();
    let saved = v.save_to(&path).is_ok();
    let clean = !v.is_dirty() && v.file_path() == Some(path.as_path());
    let edited = v.doc().unwrap().clone();
    let mut w = Viewer::default();
    let reopened = w.open_path(&path).is_ok() && *w.doc().unwrap() == edited;
    // The text outlines saved for both devices draw the edited title.
    let outlines = quadrule_format::QuadFile::from_bytes(&std::fs::read(&path).unwrap())
        .is_ok_and(|f| f.outlines.len() == 2);
    // No half-written file is left beside it.
    let leftovers = std::fs::read_dir(&dir).unwrap().count();
    let ok = no_file && dirty && saved && clean && reopened && outlines && leftovers == 1;
    println!(
        "CHECK crate=ui name=save_round_trip result={} sample_has_no_file={no_file} dirty_before={dirty} saved={saved} clean_after={clean} reopened_same={reopened} outlines_for_both={outlines} files_in_folder={leftovers}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn export_bytes_unchanged_after_edit_and_undo() {
    // Rung 3 bbq Q12: the export goldens stay fixed. Edit, undo, export: the PNGs are rung 1's.
    let sums = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../render/tests/golden/SHA256SUMS"),
    )
    .unwrap();
    let golden: Vec<&str> = sums
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    let mut v = sample();
    v.selection = vec![ids::DOTS, ids::TITLE];
    v.nudge(3.0, -2.0, true).unwrap();
    v.delete_selection().unwrap();
    v.undo();
    v.undo();
    let dir = tmp("export_after_undo");
    let files = v.export(&dir, ExportWhich::AllDevices).unwrap();
    let shas: Vec<String> = files
        .iter()
        .map(|f| quadrule_format::sha256_hex(&std::fs::read(f).unwrap()))
        .collect();
    let ok = golden.len() >= 2 && shas.iter().all(|s| golden.contains(&s.as_str()));
    println!(
        "CHECK crate=ui name=export_unchanged_after_edit_undo result={} files={} matches_goldens={ok}",
        pass(ok),
        files.len()
    );
    assert!(ok);
}

#[test]
fn autosave_and_recovery() {
    let dir = tmp("recovery");
    let rec = Recovery::new(dir.join("recovery"));
    let mut v = sample();
    let mut auto = Autosave::default();
    let clean_never = !auto.due(&v, 0.0, true);
    v.selection = vec![ids::RULE];
    v.nudge(0.0, 1.0, false).unwrap();
    let not_yet = !auto.due(&v, 10.0, false);
    let after_30 = auto.due(&v, 31.0, false);
    let written = after_30 && rec.write(&mut v).is_ok();
    // Nothing new since: no second write, even on focus loss.
    let no_repeat = !auto.due(&v, 70.0, true);
    v.nudge(0.0, 1.0, false).unwrap();
    let focus_loss = auto.due(&v, 71.0, true) && rec.write(&mut v).is_ok();
    let edited = v.doc().unwrap().clone();

    // "Crash": a new window finds the recovery file and restores it, with unsaved changes.
    let found = rec.list();
    let one = found.len() == 1 && found[0].source == Source::Sample;
    let mut w = Viewer::default();
    let restored =
        rec.restore(&found[0], &mut w).is_ok() && *w.doc().unwrap() == edited && w.is_dirty();
    // A clean save deletes it.
    let path = dir.join("saved.quad");
    w.save_to(&path).unwrap();
    rec.discard(&Source::Sample);
    let gone = rec.list().is_empty();
    // A file's recovery note names its file.
    w.nudge(0.0, 1.0, false).unwrap();
    rec.write(&mut w).unwrap();
    let names_file = rec.list().first().map(|r| r.source.clone()) == Some(Source::File(path));
    let ok = clean_never
        && not_yet
        && written
        && no_repeat
        && focus_loss
        && one
        && restored
        && gone
        && names_file;
    println!(
        "CHECK crate=ui name=autosave_and_recovery result={} clean_never={clean_never} not_before_30s={not_yet} at_30s={written} no_repeat={no_repeat} on_focus_loss={focus_loss} found={one} restored_dirty={restored} discarded={gone} note_names_file={names_file}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn shift_arrow_moves_one_subdivision_when_they_are_on() {
    // Napkin §6.2: Shift+arrow is one rhythm unit, or one subdivision if subdivisions are on.
    let mut v = sample();
    v.selection = vec![ids::RULE];
    let x0 = x_of(&v, ids::RULE);
    v.nudge(1.0, 0.0, true).unwrap();
    let unit = x_of(&v, ids::RULE) - x0;
    v.undo();
    let mut r = v.doc().unwrap().rhythm.clone();
    r.subdivisions = Some(4);
    v.edit(quadrule_model::edit::Command::SetRhythm(r), None)
        .unwrap();
    v.nudge(1.0, 0.0, true).unwrap();
    let sub = x_of(&v, ids::RULE) - x0;
    // 6 mm on the Manta is 71 px (pixel-locked): 71 px, and a quarter of it.
    let ok = (unit - 71.0 * 25.4 / 300.0).abs() < 1e-9 && (sub - unit / 4.0).abs() < 1e-9;
    println!(
        "\nCHECK crate=ui name=nudge_subdivision result={} unit_mm={unit:.4} sub_mm={sub:.4}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn copy_cut_and_paste() {
    // Rung 4b (bbq Q8, Defaulted "Copy and paste"): the clipboard is text; a paste into the same
    // document lands one rhythm unit down and right (6 mm on the sample page), each further paste
    // one more; into another document at the same place; a cut puts back where it was.
    let mut a = Viewer::default();
    a.open_sample();
    let objects_before = a
        .doc()
        .unwrap()
        .layers
        .iter()
        .map(|l| l.objects.len())
        .sum::<usize>();
    a.selection = vec![ids::TITLE, ids::RULE];
    let text = a.copy_text().expect("two objects to copy");
    let copy_changed_nothing = !a.is_dirty() && a.open.as_ref().unwrap().history.undo_len() == 0;

    assert!(a.paste_text(&text).unwrap());
    let first = a.selection.clone();
    let count = |v: &Viewer| {
        v.doc()
            .unwrap()
            .layers
            .iter()
            .map(|l| l.objects.len())
            .sum::<usize>()
    };
    let title0 = a.doc().unwrap().object(ids::TITLE).unwrap().frame;
    let p1 = a.doc().unwrap().object(first[0]).unwrap().frame;
    let one_unit =
        (p1.x_mm - title0.x_mm - 6.0).abs() < 1e-9 && (p1.y_mm - title0.y_mm - 6.0).abs() < 1e-9;
    let selected_new =
        first.len() == 2 && first.iter().all(|id| *id != ids::TITLE && *id != ids::RULE);
    assert!(a.paste_text(&text).unwrap());
    let p2 = a.doc().unwrap().object(a.selection[0]).unwrap().frame;
    let two_units = (p2.x_mm - title0.x_mm - 12.0).abs() < 1e-9;
    let steps = a.open.as_ref().unwrap().history.undo_len();
    a.undo();
    a.undo();
    let undone = count(&a) == objects_before;

    // Text that is not quadrule's does nothing.
    let ignored = !a.paste_text("just some words").unwrap() && count(&a) == objects_before;

    // Another document (one that does not hold the copied objects): the first paste is in the
    // same place.
    let mut b = Viewer::default();
    b.open_sample();
    b.edit(quadrule_model::edit::remove(&[ids::TITLE, ids::RULE]), None)
        .unwrap();
    assert!(b.paste_text(&text).unwrap());
    let q = b.doc().unwrap().object(b.selection[0]).unwrap().frame;
    let same_place = q == title0;

    // A cut takes the objects away, and pasting puts them back in the same place.
    let mut c = Viewer::default();
    c.open_sample();
    c.selection = vec![ids::RULE];
    let cut = c.cut_text().unwrap().expect("cut text");
    let gone = c.doc().unwrap().object(ids::RULE).is_none() && c.selection.is_empty();
    assert!(c.paste_text(&cut).unwrap());
    let back = c.doc().unwrap().object(c.selection[0]).unwrap().frame
        == a.doc().unwrap().object(ids::RULE).unwrap().frame;

    // A locked object is copied but not cut.
    let mut d = Viewer::default();
    d.open_sample();
    let (l, o) = d.doc().unwrap().find(ids::RULE).unwrap();
    let mut locked = d.doc().unwrap().layers[l].objects[o].clone();
    locked.locked = true;
    d.edit(quadrule_model::edit::Command::SetObject(locked), None)
        .unwrap();
    d.selection = vec![ids::RULE];
    let copied_locked = d.copy_text().is_some();
    let not_cut = d.cut_text().unwrap().is_none() && d.doc().unwrap().object(ids::RULE).is_some();

    let ok = copy_changed_nothing
        && one_unit
        && selected_new
        && two_units
        && steps == 2
        && undone
        && ignored
        && same_place
        && gone
        && back
        && copied_locked
        && not_cut;
    println!(
        "\nCHECK crate=ui name=copy_cut_paste result={} copy_changes_nothing={copy_changed_nothing} one_unit_down_right={one_unit} copies_selected={selected_new} second_paste_two_units={two_units} one_step_each={} undone={undone} other_text_ignored={ignored} other_document_same_place={same_place} cut_removes={gone} cut_then_paste_in_place={back} locked_copied={copied_locked} locked_not_cut={not_cut}",
        pass(ok),
        steps == 2
    );
    assert!(ok);
}
