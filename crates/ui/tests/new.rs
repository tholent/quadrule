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

//! Rung 5 step 1: File → New (docs/bbq/2026-11-10-rung-5-new-and-insert.md, Defaulted "Tests").
//! Headless. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::RhythmPx;
use quadrule_layout::newpage::{NewPage, clearing_margins, new_document, zone_thickness_px};
use quadrule_model::blank::{RhythmPreset, rfc3339_utc};
use quadrule_model::edit::Command;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_model::{
    Constraints, DashStyle, Extra, Frame, HConstraint, LineCap, LineRole, Object, ObjectKind,
    Pattern, PatternRegion, Style, Uuid, VConstraint,
};
use quadrule_profiles::builtin;
use quadrule_ui::{Source, Viewer};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn spec(device: &str, rhythm: RhythmPreset) -> NewPage {
    NewPage {
        title: "Test page".into(),
        device_id: device.into(),
        rhythm,
        orientation: quadrule_model::Orientation::Portrait,
    }
}

/// A pattern region that fills the page inside its margins.
fn fill_region(doc: &quadrule_model::Document) -> Object {
    let p = builtin(&doc.primary_profile_id).unwrap();
    let (w, h) = p.page_mm();
    let m = doc.page.margins_mm;
    Object {
        id: Uuid::from_u128(1),
        name: "Fill".into(),
        frame: Frame::new(m.left, m.top, w - m.left - m.right, h - m.top - m.bottom),
        // Keeps its distance from every edge on every device, so the margins stay what New made.
        constraints: Constraints {
            h: HConstraint::LeftRight,
            v: VConstraint::TopBottom,
        },
        visible: true,
        locked: false,
        style: Style::Role(LineRole::Guide),
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        kind: ObjectKind::Pattern(PatternRegion::new(Pattern::Lines)),
        extra: Extra::new(),
    }
}

#[test]
fn new_gives_the_chosen_device_rhythm_and_margins() {
    let mut all = true;
    let mut seen = String::new();
    for device in [MANTA_ID, NOMAD_ID] {
        let doc = new_document(
            &spec(device, RhythmPreset::Handwriting),
            "2026-11-10T00:00:00Z",
        )
        .unwrap();
        let other = if device == MANTA_ID {
            NOMAD_ID
        } else {
            MANTA_ID
        };
        let m = doc.page.margins_mm;
        // Margins are whole rhythm units, and at least as thick as every docked toolbar and footer.
        let whole = [m.top, m.right, m.bottom, m.left]
            .iter()
            .all(|x| (x / 6.0 - (x / 6.0).round()).abs() < 1e-9);
        let clears = [device, other].iter().all(|id| {
            let p = builtin(id).unwrap();
            let need = zone_thickness_px(&p);
            p.mm_to_px(m.left) >= need.left
                && p.mm_to_px(m.right) >= need.right
                && p.mm_to_px(m.top) >= need.top
                && p.mm_to_px(m.bottom) >= need.bottom
        });
        let ok = doc.primary_profile_id == device
            && doc.profile_ids() == vec![device, other]
            && doc.rhythm.h_unit_mm == 6.0
            && doc.rhythm.v_unit_mm == 6.0
            && doc.layers.len() == 1
            && doc.layers[0].objects.is_empty()
            && doc.metadata.title == "Test page"
            && whole
            && clears;
        seen.push_str(&format!(
            " {}_margin_mm={:.1}",
            builtin(device).unwrap().short_name,
            m.left
        ));
        all &= ok;
    }
    println!(
        "CHECK crate=ui name=new_gives_device_rhythm_margins result={}{seen}",
        pass(all)
    );
    assert!(all);
}

#[test]
fn a_blank_page_passes_lint_on_both_devices() {
    // A region that fills the margins touches no docked toolbar or footer: no L06 finding. (The
    // floating toolbar is not linted.)
    let mut results = Vec::new();
    let mut ok = true;
    for device in [MANTA_ID, NOMAD_ID] {
        for preset in RhythmPreset::ALL {
            let mut v = Viewer::default();
            v.open_new(&spec(device, preset)).unwrap();
            let region = fill_region(v.doc().unwrap());
            v.edit(
                Command::Insert {
                    layer: 0,
                    index: 0,
                    object: region,
                },
                None,
            )
            .unwrap();
            let mut bad = 0;
            for id in [MANTA_ID, NOMAD_ID] {
                let page = v.page_for(id).unwrap();
                bad += page
                    .findings
                    .iter()
                    .filter(|f| f.rule == "L06" && f.severity != quadrule_lint::Severity::Info)
                    .count();
            }
            ok &= bad == 0;
            results.push(bad);
        }
    }
    println!(
        "CHECK crate=ui name=new_blank_page_lints_clean result={} l06_non_info_findings={results:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn each_preset_resolves_to_pixels_on_each_device() {
    // The px per device, as the rhythm resolves it (pixel-locked: whole pixels).
    let mut line = String::new();
    let mut ok = true;
    for preset in RhythmPreset::ALL {
        let doc = new_document(&spec(MANTA_ID, preset), "2026-11-10T00:00:00Z").unwrap();
        for id in [MANTA_ID, NOMAD_ID] {
            let p = builtin(id).unwrap();
            let r = RhythmPx::new(&doc, &p);
            // The isometric preset snaps to half a side across and a row down.
            let (sx, sy) = preset.steps_mm();
            let (ex, ey) = (p.mm_to_px(sx).round(), p.mm_to_px(sy).round());
            ok &= r.unit_x == ex && r.unit_y == ey && ex >= 1.0 && ey >= 1.0;
            line.push_str(&format!(
                " {}_{}={}",
                preset.message_id().trim_start_matches("new-rhythm-"),
                p.short_name,
                r.unit_x
            ));
        }
    }
    println!(
        "CHECK crate=layout name=new_presets_resolve result={}{line}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn new_opens_untitled_with_no_unsaved_changes() {
    let mut v = Viewer::default();
    v.open_sample();
    v.open_new(&spec(NOMAD_ID, RhythmPreset::Grid5mm)).unwrap();
    let o = v.open.as_ref().unwrap();
    let ok = o.source == Source::Untitled
        && v.file_path().is_none()
        && !v.is_dirty()
        && v.profile_id == NOMAD_ID
        && v.title() == Some("Test page");
    // Insert then undo leaves nothing to save.
    let region = fill_region(v.doc().unwrap());
    v.edit(
        Command::Insert {
            layer: 0,
            index: 0,
            object: region,
        },
        None,
    )
    .unwrap();
    let dirty = v.is_dirty();
    v.undo();
    let ok = ok && dirty && !v.is_dirty();
    println!(
        "CHECK crate=ui name=new_opens_untitled result={} untitled_no_file_clean_then_dirty_then_clean={ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_new_page_saves_and_reopens() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("new-save");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("new.quad");
    let mut v = Viewer::default();
    v.open_new(&spec(MANTA_ID, RhythmPreset::CollegeRuled))
        .unwrap();
    let region = fill_region(v.doc().unwrap());
    v.edit(
        Command::Insert {
            layer: 0,
            index: 0,
            object: region,
        },
        None,
    )
    .unwrap();
    let before = v.doc().unwrap().clone();
    v.save_to(&path).unwrap();
    let mut w = Viewer::default();
    w.open_path(&path).unwrap();
    let ok =
        w.doc().unwrap() == &before && matches!(w.open.as_ref().unwrap().source, Source::File(_));
    println!(
        "CHECK crate=ui name=new_page_round_trips result={} same_document_after_reopen={ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn margins_grow_by_whole_units_until_they_clear() {
    // 116 px on the Manta is 9.8 mm: two 6 mm units (12 mm). With 1/4 in units (6.35 mm), two
    // as well; with a 20 mm unit, one.
    let manta = builtin(MANTA_ID).unwrap();
    let a = clearing_margins(std::slice::from_ref(&manta), 6.0, 6.0);
    let b = clearing_margins(std::slice::from_ref(&manta), 20.0, 20.0);
    let ok = (a.left - 12.0).abs() < 1e-9 && (b.left - 20.0).abs() < 1e-9;
    println!(
        "CHECK crate=layout name=margins_whole_units result={} six_mm_left={} twenty_mm_left={}",
        pass(ok),
        a.left,
        b.left
    );
    assert!(ok);
}

#[test]
fn rfc3339_times_are_right() {
    let ok = rfc3339_utc(0) == "1970-01-01T00:00:00Z"
        && rfc3339_utc(951_782_400) == "2000-02-29T00:00:00Z"
        && rfc3339_utc(1_762_732_800 + 3_661) == "2025-11-10T01:01:01Z";
    println!(
        "CHECK crate=model name=rfc3339_utc result={} epoch_leap_day_and_a_2025_date={ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_unsaved_new_page_is_recovered_as_untitled() {
    use quadrule_ui::Recovery;
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("new-recovery");
    let _ = std::fs::remove_dir_all(&dir);
    let rec = Recovery::new(&dir);
    let mut v = Viewer::default();
    v.open_new(&spec(MANTA_ID, RhythmPreset::WideRuled))
        .unwrap();
    let region = fill_region(v.doc().unwrap());
    v.edit(
        Command::Insert {
            layer: 0,
            index: 0,
            object: region,
        },
        None,
    )
    .unwrap();
    let before = v.doc().unwrap().clone();
    rec.write(&mut v).unwrap();
    let found = rec.list();
    let one_untitled = found.len() == 1 && found[0].source == Source::Untitled;
    let mut w = Viewer::default();
    rec.restore(&found[0], &mut w).unwrap();
    let ok = one_untitled && w.doc().unwrap() == &before && w.is_dirty();
    println!(
        "CHECK crate=ui name=new_page_recovers_untitled result={} one_untitled_file={one_untitled} restored_same_and_dirty={ok}",
        pass(ok)
    );
    assert!(ok);
}
