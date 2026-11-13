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

//! Rung 9 in the editor (docs/bbq/2026-11-13-rung-9-pattern-library.md): an isometric rhythm
//! snaps, nudges and draws on half a side across and a row down, and its overlay draws the
//! triangles with major lines stronger; each pattern setting and each rhythm setting is one undo
//! step. Headless. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::RhythmPx;
use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::edit::Command;
use quadrule_model::sample::{MANTA_ID, patterns_document};
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_ui::drag::OverlayLine;
use quadrule_ui::{DragMods, Tool, Viewer};

const REACH: f64 = 6.0;
const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: false,
};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn page(rhythm: RhythmPreset) -> Viewer {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Patterns".into(),
        device_id: MANTA_ID.into(),
        rhythm,
        orientation: Orientation::Portrait,
    })
    .unwrap();
    v
}

fn px(v: &Viewer, id: Uuid) -> (f64, f64, f64, f64) {
    let f = v.doc().unwrap().object(id).unwrap().frame;
    let k = 300.0 / MM_PER_IN;
    (
        f.x_mm * k,
        f.y_mm * k,
        (f.x_mm + f.w_mm) * k,
        (f.y_mm + f.h_mm) * k,
    )
}

fn draw(v: &mut Viewer, tool: Tool, from: (f64, f64), to: (f64, f64)) -> Uuid {
    v.set_tool(tool);
    assert!(v.draw_begin(from.0, from.1, NONE, REACH));
    v.draw_update(to.0, to.1, NONE, REACH);
    v.draw_end().expect("an object was drawn")
}

fn region(v: &mut Viewer) -> Uuid {
    let id = draw(v, Tool::Pattern, (200.0, 200.0), (600.0, 500.0));
    v.selection = vec![id];
    id
}

fn pattern_of(v: &Viewer, id: Uuid) -> PatternRegion {
    match &v.doc().unwrap().object(id).unwrap().kind {
        ObjectKind::Pattern(p) => p.clone(),
        _ => panic!("not a pattern"),
    }
}

#[test]
fn an_isometric_page_draws_nudges_and_snaps_on_half_a_side_and_a_row() {
    let mut v = page(RhythmPreset::Isometric5mm);
    let doc = v.doc().unwrap().clone();
    let is_iso = doc.rhythm.is_isometric() && doc.rhythm.side() == 5.0;
    let m = builtin(MANTA_ID).unwrap();
    let r = RhythmPx::new(&doc, &m);
    // 5 mm side on the Manta: 30 px across, 51 px down.
    let units = (r.unit_x, r.unit_y) == (30.0, 51.0);
    // A rectangle dragged 3 px off the grid at the start and 5 and 4 px short at the end: its
    // corner and its size land on the grid.
    let from = (r.origin_x + 3.0, r.origin_y - 2.0);
    let to = (r.origin_x + 4.0 * 30.0 - 5.0, r.origin_y + 3.0 * 51.0 - 4.0);
    let id = draw(&mut v, Tool::Rect, from, to);
    let (x0, y0, x1, y1) = px(&v, id);
    let near = |a: f64, b: f64| (a - b).abs() < 0.01;
    let drawn = near(x0, r.origin_x)
        && near(y0, r.origin_y)
        && near(x1 - x0, 120.0)
        && near(y1 - y0, 153.0);
    // Shift+arrow moves by one step: half a side across, a row down.
    v.selection = vec![id];
    v.nudge(1.0, 0.0, true).unwrap();
    v.nudge(0.0, 1.0, true).unwrap();
    let (nx0, ny0, _, _) = px(&v, id);
    let nudged = near(nx0 - x0, 30.0) && near(ny0 - y0, 51.0);
    let ok = is_iso && units && drawn && nudged;
    println!(
        "CHECK crate=ui name=isometric_editing result={} preset_isometric_5mm={is_iso} steps_px=30x51={units} drawn_on_grid={drawn} shift_arrow_one_step={nudged}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_overlay_draws_triangles_and_stronger_major_lines() {
    let mut v = page(RhythmPreset::Isometric5mm);
    let none = v.rhythm_overlay().unwrap();
    // Major every 3: rows 0, 3, 6 and slanted lines 0, 3, 6 are strong.
    let mut r = v.doc().unwrap().rhythm.clone();
    r.major_every = Some(3);
    v.edit(Command::SetRhythm(r), None).unwrap();
    let ov = v.rhythm_overlay().unwrap();
    let triangles = ov.xs.is_empty() && !ov.slanted.is_empty();
    // The slanted lines are on the 60 degree axes: dy / dx = 51 / 30 (the pixel-locked row over
    // half a side).
    let slope_ok = ov.slanted.iter().all(|((ax, ay), (bx, by), _)| {
        (((by - ay) / (bx - ax)).abs() - 51.0 / (30.0 * 1.0)).abs() < 0.01
    });
    let majors = ov
        .ys
        .iter()
        .filter(|(_, l)| *l == OverlayLine::Major)
        .count();
    let majors_slanted = ov
        .slanted
        .iter()
        .filter(|(_, _, l)| *l == OverlayLine::Major)
        .count();
    let rows_every_3 = ov
        .ys
        .iter()
        .enumerate()
        .all(|(i, (_, l))| (*l == OverlayLine::Major) == (i % 3 == 0));
    let off_before = none.ys.iter().all(|(_, l)| *l == OverlayLine::Unit)
        && none.slanted.iter().all(|(_, _, l)| *l == OverlayLine::Unit);
    // A rectangular rhythm has vertical lines and no slanted ones, majors every 3 units each way.
    let mut rect = page(RhythmPreset::Grid5mm);
    let mut rr = rect.doc().unwrap().rhythm.clone();
    rr.major_every = Some(3);
    rect.edit(Command::SetRhythm(rr), None).unwrap();
    let ro = rect.rhythm_overlay().unwrap();
    let rd = rect.doc().unwrap().clone();
    let rpx = RhythmPx::new(&rd, &builtin(MANTA_ID).unwrap());
    let is_major = |x: f64, o: f64, u: f64| (((x - o) / u).round() as i64).rem_euclid(3) == 0;
    let rect_ok = !ro.xs.is_empty()
        && ro.slanted.is_empty()
        && ro
            .xs
            .iter()
            .filter(|(_, l)| *l == OverlayLine::Major)
            .count()
            > 3
        && ro
            .xs
            .iter()
            .all(|(x, l)| (*l == OverlayLine::Major) == is_major(*x, rpx.origin_x, rpx.unit_x))
        && ro
            .ys
            .iter()
            .all(|(y, l)| (*l == OverlayLine::Major) == is_major(*y, rpx.origin_y, rpx.unit_y));
    let ok = triangles
        && slope_ok
        && majors > 3
        && majors_slanted > 3
        && rows_every_3
        && off_before
        && rect_ok;
    println!(
        "CHECK crate=ui name=rhythm_overlay result={} triangles_no_verticals={triangles} slanted_60_degrees={slope_ok} major_rows={majors} major_slanted={majors_slanted} off_by_default_all_units={off_before} rectangular_majors={rect_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn every_rhythm_setting_is_one_undo_step() {
    let mut v = page(RhythmPreset::Grid5mm);
    let start = v.doc().unwrap().clone();
    let mut results = Vec::new();
    let mut ok = true;
    type RhythmEdit = Box<dyn Fn(&mut Rhythm)>;
    let steps: Vec<(&str, RhythmEdit)> = vec![
        ("type", Box::new(|r| r.rhythm_type = RhythmType::Isometric)),
        ("side", Box::new(|r| r.side_mm = Some(6.0))),
        ("major", Box::new(|r| r.major_every = Some(4))),
    ];
    for (name, f) in steps {
        let before = v.doc().unwrap().clone();
        let mut r = before.rhythm.clone();
        f(&mut r);
        v.edit(Command::SetRhythm(r), Some(name)).unwrap();
        v.seal();
        let changed = v.doc().unwrap() != &before;
        v.undo();
        let one_step = v.doc().unwrap() == &before;
        // Redo, so the next setting starts from this one.
        v.redo();
        ok &= changed && one_step;
        results.push(format!("{name}:changed={changed}:one_undo={one_step}"));
    }
    while v.undo() {}
    let back = v.doc().unwrap() == &start;
    ok &= back;
    println!(
        "CHECK crate=ui name=rhythm_settings_undo result={} {} all_undone={back}",
        pass(ok),
        results.join(" ")
    );
    assert!(ok);
}

#[test]
fn every_pattern_setting_is_one_undo_step() {
    let mut v = page(RhythmPreset::Handwriting);
    let id = region(&mut v);
    v.seal();
    let kinds_ok = {
        let mut all = true;
        for kind in PatternKind::ALL {
            let before = v.doc().unwrap().clone();
            v.edit_selected(None, |x| x.set_pattern_kind(kind)).unwrap();
            v.seal();
            let kind_now = pattern_of(&v, id).pattern.kind() == kind;
            let changed = v.doc().unwrap() != &before || pattern_of(&v, id).pattern.kind() == kind;
            v.undo();
            all &= kind_now && changed && v.doc().unwrap() == &before;
        }
        all
    };
    // The dot patterns take the Dot grid role, and the line ones give it back.
    v.edit_selected(None, |x| x.set_pattern_kind(PatternKind::Dots))
        .unwrap();
    v.seal();
    let dots_role = v.doc().unwrap().object(id).unwrap().style == Style::Role(LineRole::DotGrid);
    v.edit_selected(None, |x| x.set_pattern_kind(PatternKind::Hex))
        .unwrap();
    v.seal();
    let lines_role = v.doc().unwrap().object(id).unwrap().style == Style::Role(LineRole::Guide);
    v.edit_selected(None, |x| {
        if let ObjectKind::Pattern(p) = &mut x.kind {
            p.pattern = Pattern::Isometric {
                style: IsoStyle::Dots,
            };
        }
        x.settle_pattern_role();
    })
    .unwrap();
    v.seal();
    let iso_dots_role =
        v.doc().unwrap().object(id).unwrap().style == Style::Role(LineRole::DotGrid);
    // Each of the other settings, one step each.
    type Edit = Box<dyn Fn(&mut PatternRegion, (f64, f64))>;
    let settings: Vec<(&str, PatternKind, Edit)> = vec![
        (
            "iso_style",
            PatternKind::Isometric,
            Box::new(|p, _| {
                p.pattern = Pattern::Isometric {
                    style: IsoStyle::Dots,
                }
            }),
        ),
        (
            "hex_orientation",
            PatternKind::Hex,
            Box::new(|p, _| {
                if let Pattern::Hex { orientation, .. } = &mut p.pattern {
                    *orientation = HexOrientation::Pointy
                }
            }),
        ),
        (
            "hex_height",
            PatternKind::Hex,
            Box::new(|p, _| {
                if let Pattern::Hex { height_units, .. } = &mut p.pattern {
                    *height_units = 3
                }
            }),
        ),
        (
            "cornell_cue",
            PatternKind::Cornell,
            Box::new(|p, _| {
                if let Pattern::Cornell { cue_units, .. } = &mut p.pattern {
                    *cue_units = Some(2)
                }
            }),
        ),
        (
            "cornell_summary",
            PatternKind::Cornell,
            Box::new(|p, _| {
                if let Pattern::Cornell { summary_units, .. } = &mut p.pattern {
                    *summary_units = Some(2)
                }
            }),
        ),
        (
            "staff_gap",
            PatternKind::MusicStaff,
            Box::new(|p, _| {
                if let Pattern::MusicStaff { gap_mm, .. } = &mut p.pattern {
                    *gap_mm = 1.5
                }
            }),
        ),
        (
            "staff_pitch",
            PatternKind::MusicStaff,
            Box::new(|p, _| {
                if let Pattern::MusicStaff { pitch_units, .. } = &mut p.pattern {
                    *pitch_units = 4
                }
            }),
        ),
        (
            "staff_grand",
            PatternKind::MusicStaff,
            Box::new(|p, _| {
                if let Pattern::MusicStaff { grand, .. } = &mut p.pattern {
                    *grand = true
                }
            }),
        ),
        (
            "unbind",
            PatternKind::Grid,
            Box::new(|p, s| p.set_bound(false, s)),
        ),
        (
            "major_mode",
            PatternKind::Grid,
            Box::new(|p, _| p.major = MajorLines::Every(4)),
        ),
        (
            "major_role",
            PatternKind::Grid,
            Box::new(|p, _| p.major_role = LineRole::Emphasis),
        ),
    ];
    let steps = v.doc().unwrap().rhythm.steps_mm();
    let mut parts = Vec::new();
    let mut all = true;
    for (name, kind, f) in settings {
        v.edit_selected(None, |x| x.set_pattern_kind(kind)).unwrap();
        v.seal();
        let before = v.doc().unwrap().clone();
        v.edit_selected(Some(name), |x| {
            if let ObjectKind::Pattern(p) = &mut x.kind {
                f(p, steps);
            }
            x.settle_pattern_role();
        })
        .unwrap();
        v.seal();
        let changed = v.doc().unwrap() != &before;
        v.undo();
        let one = v.doc().unwrap() == &before;
        all &= changed && one;
        parts.push(format!("{name}:{}", changed && one));
    }
    // Unbinding keeps the look: the region's own spacing starts as the rhythm's steps.
    v.edit_selected(None, |x| x.set_pattern_kind(PatternKind::Grid))
        .unwrap();
    v.edit_selected(None, |x| {
        if let ObjectKind::Pattern(p) = &mut x.kind {
            p.set_bound(false, steps);
        }
    })
    .unwrap();
    let p = pattern_of(&v, id);
    let seeded = !p.bound && p.h_spacing_mm == Some(steps.0) && p.v_spacing_mm == Some(steps.1);
    let ok = kinds_ok && dots_role && lines_role && iso_dots_role && all && seeded;
    println!(
        "CHECK crate=ui name=pattern_settings_undo result={} kinds={kinds_ok} dots_take_dot_grid={dots_role} lines_take_guide={lines_role} iso_dots_take_dot_grid={iso_dots_role} unbind_seeds_own_spacing={seeded} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn the_pattern_page_opens_with_the_dot_region_the_benchmarks_move() {
    let mut v = Viewer::default();
    v.open_patterns();
    let doc = v.doc().unwrap().clone();
    let kinds: std::collections::BTreeSet<_> = doc
        .leaves()
        .filter_map(|o| match &o.kind {
            ObjectKind::Pattern(p) => Some(format!("{:?}", p.pattern.kind())),
            _ => None,
        })
        .collect();
    let has_dots_id = doc
        .object(quadrule_model::sample::ids::DOTS)
        .is_some_and(|o| matches!(&o.kind, ObjectKind::Pattern(p) if p.pattern == Pattern::Dots));
    let unbound = doc
        .leaves()
        .any(|o| matches!(&o.kind, ObjectKind::Pattern(p) if !p.bound));
    let same = patterns_document() == doc;
    let ok = kinds.len() == 6 && has_dots_id && unbound && same;
    println!(
        "CHECK crate=ui name=patterns_page result={} kinds={} dots_region_has_the_sample_id={has_dots_id} unbound_region={unbound}",
        pass(ok),
        kinds.len()
    );
    assert!(ok);
}
