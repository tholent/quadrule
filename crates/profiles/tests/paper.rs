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

//! The paper profiles (rung 13b; docs/bbq/2026-11-17-rung-13-profile-system.md, Q12 to Q14 and the Defaulted "Paper
//! ids and names", "Paper margins", "Paper lint"). Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`). Tests use a folder of their own under the system's temporary folder.

use quadrule_model::{DEFAULT_PAPER_PPI, Document, PAPER_PPIS};
use quadrule_profiles::*;
use std::path::PathBuf;

const NOMAD: &str = "supernote.a6x2";

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("quadrule-paper-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The record's eleven (Q12): id, display name, short name, width and height in mm.
const ELEVEN: [(&str, &str, &str, f64, f64); 11] = [
    ("paper.a3", "ISO A3", "a3", 297.0, 420.0),
    ("paper.a4", "ISO A4", "a4", 210.0, 297.0),
    ("paper.a5", "ISO A5", "a5", 148.0, 210.0),
    ("paper.a6", "ISO A6", "a6", 105.0, 148.0),
    ("paper.b5-iso", "ISO B5", "b5-iso", 176.0, 250.0),
    ("paper.b5-jis", "JIS B5", "b5-jis", 182.0, 257.0),
    ("paper.letter", "US Letter", "letter", 215.9, 279.4),
    ("paper.legal", "US Legal", "legal", 215.9, 355.6),
    ("paper.tabloid", "Tabloid", "tabloid", 279.4, 431.8),
    (
        "paper.half-letter",
        "Half Letter",
        "half-letter",
        139.7,
        215.9,
    ),
    // 7.25 x 10.5 in; the record writes 184.2 (one place), and napkin §3.3.1's 2175 px needs 184.15.
    ("paper.executive", "Executive", "executive", 184.15, 266.7),
];

#[test]
fn the_eleven_paper_sizes_ship_unverified_with_no_device_data() {
    let mut fails = Vec::new();
    for (id, name, short, w, h) in ELEVEN {
        let Ok(p) = builtin(id) else {
            fails.push(format!("{id}:missing"));
            continue;
        };
        let size = p.paper.map(|s| (s.width_mm, s.height_mm));
        let m = &p.default_margins_mm;
        let ok = p.id == id
            && p.display_name == name
            && p.short_name == short
            && size == Some((w, h))
            && p.is_paper()
            && !p.calibration_meta.verified
            && p.calibration_meta.date.is_none()
            && p.safe_zones.is_empty()
            && p.gray_levels.is_empty()
            && p.ink_values.is_empty()
            && (m.top, m.right, m.bottom, m.left) == (10.0, 10.0, 10.0, 10.0)
            && p.orientations == [Orientation::Portrait, Orientation::Landscape]
            && p.ppi == 300.0
            && limits::check(&p).is_ok()
            && from_ron_checked(&to_ron(&p)).is_ok_and(|q| q == p);
        if !ok {
            fails.push(id.to_string());
        }
    }
    let ids: Vec<&str> = builtin_paper_ids().collect();
    let order = ids == ELEVEN.map(|e| e.0);
    // The devices are still the two screens, and the paper ids are not among them.
    let devices = builtin_ids().collect::<Vec<_>>() == ["supernote.a5x2", "supernote.a6x2"];
    let ok = fails.is_empty() && order && devices;
    println!(
        "CHECK crate=profiles name=paper_builtins result={} count={} order={order} devices_unchanged={devices} fails={fails:?}",
        pass(ok),
        ids.len()
    );
    assert!(ok);
}

#[test]
fn paper_line_roles_are_the_nomads_by_physical_size_at_every_ppi() {
    let nomad = builtin(NOMAD).unwrap();
    let scale = |w: u32, ppi: u32| ((f64::from(w) * f64::from(ppi) / 300.0).round() as u32).max(1);
    let mut fails = Vec::new();
    for id in builtin_paper_ids() {
        for ppi in PAPER_PPIS {
            let p = builtin_at(id, Orientation::Portrait, ppi).unwrap();
            let (n, r) = (&nomad.line_roles, &p.line_roles);
            let ok = [
                (n.whisper, r.whisper),
                (n.guide, r.guide),
                (n.structure, r.structure),
                (n.emphasis, r.emphasis),
                (n.dot_grid, r.dot_grid),
            ]
            .iter()
            .all(|(a, b)| b.width_px == scale(a.width_px, ppi) && b.gray == a.gray)
                && r.text_gray == n.text_gray
                && r.label_gray == n.label_gray
                && p.fill_roles == nomad.fill_roles
                && p.ppi == f64::from(ppi);
            if !ok {
                fails.push(format!("{id}@{ppi}"));
            }
        }
    }
    // At 300 the widths are the Nomad's own, whole; at 600 they double.
    let a4 = builtin_at("paper.a4", Orientation::Portrait, 300).unwrap();
    let a4_600 = builtin_at("paper.a4", Orientation::Portrait, 600).unwrap();
    let same_at_300 = a4.line_roles == nomad.line_roles;
    let twice_at_600 = a4_600.line_roles.guide.width_px == 2 * nomad.line_roles.guide.width_px
        && a4_600.line_roles.emphasis.width_px == 2 * nomad.line_roles.emphasis.width_px;
    let ok = fails.is_empty() && same_at_300 && twice_at_600;
    println!(
        "CHECK crate=profiles name=paper_roles_follow_the_nomad result={} same_at_300={same_at_300} twice_at_600={twice_at_600} fails={fails:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn pixel_size_is_mm_times_ppi_over_25_4_rounded_for_every_size_and_ppi() {
    let mut fails = Vec::new();
    for (id, _, _, w, h) in ELEVEN {
        for ppi in PAPER_PPIS {
            let p = builtin_at(id, Orientation::Portrait, ppi).unwrap();
            let want = (
                (w * f64::from(ppi) / 25.4).round() as u32,
                (h * f64::from(ppi) / 25.4).round() as u32,
            );
            if (p.width_px, p.height_px) != want {
                fails.push(format!("{id}@{ppi}:{}x{}", p.width_px, p.height_px));
            }
        }
    }
    // Spot values from napkin §3.3.1's table at 300 PPI, and the biggest page at 600.
    let a4 = builtin_at("paper.a4", Orientation::Portrait, 300).unwrap();
    let a4_150 = builtin_at("paper.a4", Orientation::Portrait, 150).unwrap();
    let tab_600 = builtin_at("paper.tabloid", Orientation::Portrait, 600).unwrap();
    let spot = (a4.width_px, a4.height_px) == (2480, 3508)
        && (a4_150.width_px, a4_150.height_px) == (1240, 1754)
        && (tab_600.width_px, tab_600.height_px) == (6600, 10200);
    let ok = fails.is_empty() && spot;
    println!(
        "CHECK crate=profiles name=paper_pixel_size result={} spot={spot} fails={fails:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn landscape_paper_is_wide_and_not_turned_and_asking_twice_changes_nothing() {
    let a4 = builtin("paper.a4").unwrap();
    let wide = a4.at_ppi(300).oriented(Orientation::Landscape);
    let twice = wide.oriented(Orientation::Landscape);
    let back = wide.oriented(Orientation::Portrait);
    let wide_600 = wide.at_ppi(600);
    let alt = a4.at_ppi(600).oriented(Orientation::Landscape);
    let ok = (wide.width_px, wide.height_px) == (3508, 2480)
        && !wide.turned
        && wide.device_size() == (3508, 2480)
        && twice == wide
        && back == a4
        && (wide_600.width_px, wide_600.height_px) == (7016, 4961)
        && wide_600 == alt
        && wide.safe_zones.is_empty();
    // A screen turns as before (it is the device's portrait page that exports).
    let nomad = builtin(NOMAD).unwrap().oriented(Orientation::Landscape);
    let screen = nomad.turned && nomad.device_size() == (1404, 1872);
    println!(
        "CHECK crate=profiles name=paper_landscape result={} screen_still_turns={screen}",
        pass(ok && screen)
    );
    assert!(ok && screen);
}

#[test]
fn a_screen_ignores_the_ppi_and_an_odd_ppi_reads_as_300() {
    let nomad = builtin(NOMAD).unwrap();
    let a4 = builtin("paper.a4").unwrap();
    let ok = nomad.at_ppi(600) == nomad
        && a4.at_ppi(123) == a4.at_ppi(300)
        && a4.at_ppi(0) == a4
        && a4.at_ppi(u32::MAX) == a4;
    println!(
        "CHECK crate=profiles name=paper_ppi_guards result={}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_page_holds_a_ppi_per_paper_size_and_a_screen_page_is_as_before() {
    let mut doc: Document = quadrule_model::sample::paper_document();
    let (a4, a5) = (
        quadrule_model::sample::PAPER_A4_ID,
        quadrule_model::sample::PAPER_A5_ID,
    );
    let was = (doc.ppi_of(a4), doc.ppi_of(a5));
    let on_a5 = active::lookup_in(&doc, a5).unwrap();
    doc.profiles[1].ppi = 123;
    let odd = doc.ppi_of(a5);
    let strange = active::lookup_in(&doc, a5).unwrap();
    doc.profiles[1].ppi = 150;
    let on_a5_150 = active::lookup_in(&doc, a5).unwrap();
    let manta = quadrule_model::sample::slice_document();
    let screen = active::lookup_in(&manta, "supernote.a5x2").unwrap()
        == builtin("supernote.a5x2").unwrap()
        && manta.ppi_of("supernote.a5x2") == DEFAULT_PAPER_PPI;
    let ok = was == (300, 600)
        && (on_a5.width_px, on_a5.height_px) == (3496, 4961)
        && odd == 300
        && (strange.width_px, strange.height_px) == (1748, 2480)
        && (on_a5_150.width_px, on_a5_150.height_px) == (874, 1240)
        && screen;
    println!(
        "CHECK crate=profiles name=lookup_in_reads_the_pages_ppi result={} odd_reads_300={}",
        pass(ok),
        odd == 300
    );
    assert!(ok);
}

fn custom(w: f64, h: f64) -> Profile {
    let mut p = builtin("paper.a4").unwrap();
    p.id = "mine".into();
    p.paper = Some(PaperSize {
        width_mm: w,
        height_mm: h,
    });
    p.sync_paper();
    p.default_margins_mm = quadrule_model::Margins::all(0.0);
    p
}

#[test]
fn custom_paper_sizes_are_held_to_the_screen_limits_at_every_ppi() {
    let good = [
        (100.0, 150.0),
        (216.0, 279.0),
        (5.0, 100.0),
        (431.8, 279.4),
        (440.0, 440.0),
        (*limits::PAPER_MM.start(), *limits::PAPER_MM.end()),
    ];
    let bad = [
        (2.0, 100.0),
        (100.0, 700.0),
        (100.0, 441.0),
        (693.0, 693.0),
        (0.0, 100.0),
        (-5.0, 100.0),
        (f64::NAN, 100.0),
        (f64::INFINITY, 100.0),
        (1.0e9, 1.0e9),
    ];
    let mut fails = Vec::new();
    for (w, h) in good {
        if let Err(e) = limits::check(&custom(w, h)) {
            fails.push(format!("good {w}x{h}: {e}"));
        }
    }
    for (w, h) in bad {
        if limits::check(&custom(w, h)).is_ok() {
            fails.push(format!("bad {w}x{h} accepted"));
        }
    }
    // A hand-edited file whose pixels disagree with its mm is read with the mm as the truth.
    let src = to_ron(&builtin("paper.a4").unwrap()).replace("width_px: 2480", "width_px: 99");
    let healed = parse_checked(&src).map(|p| p.width_px);
    // A paper file with safe zones keeps none.
    let with_zone = to_ron(&builtin(NOMAD).unwrap()).replace(
        "ppi: 300.0,",
        "ppi: 300.0, paper: Some((width_mm: 148.0, height_mm: 210.0)),",
    );
    let zoneless = parse_checked(&with_zone)
        .map(|p| p.safe_zones.is_empty() && p.gray_levels.is_empty() && p.ink_values.is_empty());
    let ok = fails.is_empty() && healed == Ok(2480) && zoneless == Ok(true);
    println!(
        "CHECK crate=profiles name=custom_paper_limits result={} healed={healed:?} zoneless={zoneless:?} fails={fails:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_profile_file_without_the_paper_field_is_a_screen_and_a_screen_writes_none() {
    let nomad = builtin(NOMAD).unwrap();
    let text = to_ron(&nomad);
    let again = from_ron_checked(&text).unwrap();
    let ok = !text.contains("paper") && again == nomad && !again.is_paper();
    println!(
        "CHECK crate=profiles name=screen_files_unchanged result={}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_registry_lists_paper_apart_and_a_copy_of_paper_stays_paper() {
    let dir = folder("registry");
    let mut r = Registry::at(&dir);
    let devices_only = r.device_ids() == ["supernote.a5x2", "supernote.a6x2"];
    let papers = r.paper_ids() == ELEVEN.map(|e| e.0);
    // New… from a paper profile: a user's paper profile with its own GUID, Unverified, size editable.
    let id = r.create("paper.a4", "My notebook").unwrap();
    let mine = r.get(&id).unwrap().clone();
    let stays = mine.is_paper() && mine.id != "paper.a4" && !mine.calibration_meta.verified;
    let mut bigger = mine.clone();
    bigger.paper = Some(PaperSize {
        width_mm: 150.0,
        height_mm: 230.0,
    });
    bigger.sync_paper();
    let edited = r.save_mine(bigger).is_ok()
        && r.get(&id).unwrap().paper.map(|s| s.width_mm) == Some(150.0)
        && (r.get(&id).unwrap().width_px, r.get(&id).unwrap().height_px) == (1772, 2717);
    let listed = r.paper_ids().last() == Some(&id) && !r.device_ids().contains(&id);
    // It carries the way any profile of the user's does: written, read back, the same.
    let reloaded = Registry::load(&dir);
    let kept = reloaded.get(&id) == r.get(&id) && reloaded.damaged().is_empty();
    // A built-in paper's override keeps id, size and short name.
    let mut over = r.get("paper.a4").unwrap().clone();
    over.default_margins_mm = quadrule_model::Margins::all(15.0);
    over.paper = Some(PaperSize {
        width_mm: 1.0,
        height_mm: 2.0,
    });
    over.short_name = "changed".into();
    let saved = r.save(over).is_ok();
    let shown = r.get("paper.a4").unwrap();
    let locked = saved
        && r.is_edited("paper.a4")
        && shown.short_name == "a4"
        && shown.paper == builtin("paper.a4").unwrap().paper
        && (shown.width_px, shown.height_px) == (2480, 3508)
        && shown.default_margins_mm.top == 15.0;
    // And a file that tries to change the locked size is damaged, not loaded.
    let mut sneaky = builtin("paper.a5").unwrap();
    sneaky.paper = Some(PaperSize {
        width_mm: 100.0,
        height_mm: 200.0,
    });
    sneaky.sync_paper();
    std::fs::write(dir.join("paper.a5.ron"), to_ron(&sneaky)).unwrap();
    let again = Registry::load(&dir);
    let refused = again.damaged().iter().any(|d| d.file == "paper.a5.ron")
        && again.get("paper.a5") == builtin("paper.a5").ok().as_ref();
    let ok = devices_only && papers && stays && edited && listed && kept && locked && refused;
    println!(
        "CHECK crate=profiles name=registry_paper result={} devices_only={devices_only} papers={papers} copy_stays_paper={stays} size_editable={edited} listed_after={listed} kept={kept} override_locks_size={locked} locked_file_refused={refused}",
        pass(ok)
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(ok);
}
