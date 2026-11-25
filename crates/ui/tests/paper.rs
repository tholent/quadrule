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

//! Rung 13b, paper sizes in the editor (docs/bbq/2026-11-17-rung-13-profile-system.md, Q12 to Q14, Defaulted
//! "Paper in the UI", "Export"): the paper page exports a PNG per paper size at the PPI the page picked, under
//! the profile's short name and the PNG's own size, byte for byte the golden images; a landscape paper page
//! exports wide; the PPI choice is one undo step; New… on a paper size gives a page of that paper alone with
//! 10 mm margins; the PPI survives making another size primary and turning the page. Headless. Each test
//! prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::newpage::NewPage;
use quadrule_model::Orientation;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::{NOMAD_ID, PAPER_A4_ID, PAPER_A5_ID, PAPER_LETTER_ID};
use quadrule_render::encode_png;
use quadrule_ui::Viewer;
use quadrule_ui::viewer::{ExportWhich, export_pixels};
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

fn golden(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../render/tests/golden")
            .join(name),
    )
    .unwrap_or_default()
}

fn names(files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn the_paper_page_exports_each_size_at_its_ppi_as_the_golden_images() {
    let mut v = Viewer::default();
    v.open_paper();
    // The bytes are the goldens', which carry no text note.
    v.settings.note = false;
    let dir = tmp("ui-export-paper");
    let all = v.export(&dir, ExportWhich::AllDevices).unwrap();
    let got = names(&all);
    let want = ["rung-13b-paper-a4.png", "rung-13b-paper-a5.png"];
    let goldens = [
        "paper_paper.a4_2480x3508.png",
        "paper_paper.a5_3496x4961.png",
    ];
    let bytes = all
        .iter()
        .zip(goldens)
        .all(|(p, g)| std::fs::read(p).unwrap() == golden(g));
    let mut l = Viewer::default();
    l.open_paper_landscape();
    l.settings.note = false;
    let ldir = tmp("ui-export-paper-landscape");
    let lall = l.export(&ldir, ExportWhich::ThisDevice).unwrap();
    let lgot = names(&lall);
    let page = l.page_for(PAPER_LETTER_ID).unwrap();
    let wide = !page.profile.turned && page.image().width > page.image().height;
    let not_turned = export_pixels(page).as_ref() == page.image();
    let lbytes = std::fs::read(&lall[0]).unwrap()
        == golden("paper-landscape_paper.letter_3300x2550.png")
        && encode_png(&export_pixels(page), None) == std::fs::read(&lall[0]).unwrap();
    let ok = got == want
        && bytes
        && lgot == ["rung-13b-paper-landscape-letter.png"]
        && wide
        && not_turned
        && lbytes;
    println!(
        "CHECK crate=ui name=paper_export result={} names={got:?} landscape={lgot:?} bytes_match_golden={bytes} wide_not_turned={}",
        pass(ok),
        wide && not_turned && lbytes
    );
    assert!(ok);
}

#[test]
fn the_ppi_choice_is_one_undo_step_and_only_paper_has_one() {
    let mut v = Viewer::default();
    v.open_paper();
    let size = |v: &mut Viewer| {
        let p = &v.page_for(PAPER_A5_ID).unwrap().profile;
        (p.width_px, p.height_px)
    };
    let start = size(&mut v);
    let history_before = v.doc().unwrap().clone();
    v.set_paper_ppi(PAPER_A5_ID, 150).unwrap();
    let after = size(&mut v);
    let only_ppi_changed = {
        let d = v.doc().unwrap();
        d.ppi_of(PAPER_A5_ID) == 150
            && d.layers == history_before.layers
            && d.profiles[0] == history_before.profiles[0]
    };
    v.undo();
    let undone = size(&mut v) == start && v.doc().unwrap() == &history_before;
    v.redo();
    let redone = size(&mut v) == after;
    // Not offered: a PPI that is not one of the four, a screen, a size the page does not target.
    v.set_paper_ppi(PAPER_A5_ID, 123).unwrap();
    let odd_refused = v.doc().unwrap().ppi_of(PAPER_A5_ID) == 150;
    v.set_paper_ppi(PAPER_LETTER_ID, 600).unwrap();
    let not_target =
        v.doc().unwrap().ppi_of(PAPER_LETTER_ID) == 300 && v.doc().unwrap().profiles.len() == 2;
    let mut m = Viewer::default();
    m.open_sample();
    m.set_paper_ppi(NOMAD_ID, 600).unwrap();
    let screen_refused = m.doc().unwrap().profiles.iter().all(|c| c.ppi == 300);
    let ok = start == (3496, 4961)
        && after == (874, 1240)
        && only_ppi_changed
        && undone
        && redone
        && odd_refused
        && not_target
        && screen_refused;
    println!(
        "CHECK crate=ui name=paper_ppi_undo result={} start={start:?} after={after:?} one_step={undone} redo={redone} guards={}",
        pass(ok),
        odd_refused && not_target && screen_refused
    );
    assert!(ok);
}

#[test]
fn new_on_a_paper_size_makes_that_paper_alone_with_ten_millimetre_margins() {
    let mut portrait = Viewer::default();
    let spec = |o| NewPage {
        title: "Notebook".into(),
        device_id: PAPER_A4_ID.into(),
        rhythm: RhythmPreset::Grid5mm,
        orientation: o,
    };
    portrait.open_new(&spec(Orientation::Portrait)).unwrap();
    let d = portrait.doc().unwrap().clone();
    let m = d.page.margins_mm;
    let p = &portrait.page_for(PAPER_A4_ID).unwrap().profile;
    let alone = d.profile_ids() == [PAPER_A4_ID] && d.ppi_of(PAPER_A4_ID) == 300;
    let margins = [m.top, m.right, m.bottom, m.left] == [10.0; 4];
    let size = (p.width_px, p.height_px) == (2480, 3508);
    let mut wide = Viewer::default();
    wide.open_new(&spec(Orientation::Landscape)).unwrap();
    // The size tokens give the PNG's own size: the paper page is held wide, not turned.
    let name = {
        let mut req = wide.export_request(Some(ExportWhich::ThisDevice), PathBuf::new());
        req.devices = vec![PAPER_A4_ID.into()];
        req.template = "{title}_{device}_{width}x{height}.png".into();
        wide.plan_export(&req).unwrap().remove(0).1
    };
    let landscape = name == "notebook_a4_3508x2480.png";
    // A rhythm whose step is 6 mm rounds the 10 mm up to a whole step, 12.
    let mut six = Viewer::default();
    six.open_new(&NewPage {
        rhythm: RhythmPreset::Handwriting,
        ..spec(Orientation::Portrait)
    })
    .unwrap();
    let m6 = six.doc().unwrap().page.margins_mm;
    let rounded = [m6.top, m6.right, m6.bottom, m6.left] == [12.0; 4];
    let ok = alone && margins && size && landscape && rounded;
    println!(
        "CHECK crate=ui name=paper_new result={} alone={alone} margins_10={margins} size={size} landscape_name={name} six_rounds_to_12={rounded}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_page_keeps_each_ppi_when_another_size_becomes_primary_and_when_it_is_turned() {
    let mut v = Viewer::default();
    v.open_paper();
    v.set_targets(&[PAPER_A4_ID, PAPER_A5_ID, NOMAD_ID])
        .unwrap();
    v.make_primary(PAPER_A5_ID).unwrap();
    let primary = v.doc().unwrap().primary_profile_id.clone();
    let kept =
        v.doc().unwrap().ppi_of(PAPER_A5_ID) == 600 && v.doc().unwrap().ppi_of(PAPER_A4_ID) == 300;
    v.set_orientation(Orientation::Landscape).unwrap();
    let turned = v.doc().unwrap().ppi_of(PAPER_A5_ID) == 600;
    let a5 = v.page_for(PAPER_A5_ID).unwrap().profile.clone();
    let nomad = v.page_for(NOMAD_ID).unwrap().profile.clone();
    // Paper is wide and not turned; the screen next to it is the device's turned page.
    let sizes = (a5.width_px, a5.height_px) == (4961, 3496) && !a5.turned && nomad.turned;
    // Untick the paper: its PPI is forgotten with it, and ticking it again starts at 300.
    v.set_targets(&[PAPER_A5_ID, NOMAD_ID]).unwrap();
    v.set_targets(&[PAPER_A5_ID, NOMAD_ID, PAPER_A4_ID])
        .unwrap();
    let again_300 = v.doc().unwrap().ppi_of(PAPER_A4_ID) == 300;
    let ok = primary == PAPER_A5_ID && kept && turned && sizes && again_300;
    println!(
        "CHECK crate=ui name=paper_ppi_kept result={} primary={primary} kept={kept} turned={turned} sizes={sizes} again_300={again_300}",
        pass(ok)
    );
    assert!(ok);
}
