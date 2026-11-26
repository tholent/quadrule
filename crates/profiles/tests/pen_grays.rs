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

//! ADR-027 with `--pen-grays` on: the tablets' steps are built on the pen grays, every role and lint gray moves to the
//! new step in its place, and paper profiles' roles move too. Every test here turns the switch on first; this file is
//! its own test program, so no test elsewhere sees it on.

use quadrule_model::LineRole;
use quadrule_model::pen_grays::{self, NEW_STEPS, OLD_STEPS, moved};
use quadrule_profiles::{Registry, builtin};

const TABLETS: [&str; 2] = ["supernote.a5x2", "supernote.a6x2"];

fn on() {
    pen_grays::set(true);
}

#[test]
fn the_scale_moves_only_the_old_steps() {
    on();
    let steps_move = OLD_STEPS.iter().zip(NEW_STEPS).all(|(&o, n)| moved(o) == n);
    let close = OLD_STEPS
        .iter()
        .zip(NEW_STEPS)
        .all(|(&o, n)| o.abs_diff(n) <= 4);
    let others_stay = (0..=255u8)
        .filter(|g| !OLD_STEPS.contains(g))
        .all(|g| moved(g) == g);
    let ok = steps_move && close && others_stay && NEW_STEPS[15] == 255;
    println!(
        "CHECK crate=profiles name=pen_scale_moves_old_steps result={} steps_move={steps_move} within_4={close} others_stay={others_stay} top={}",
        if ok { "pass" } else { "fail" },
        NEW_STEPS[15]
    );
    assert!(ok);
}

#[test]
fn the_tablets_steps_end_on_255_and_hold_every_pen_but_white() {
    on();
    let mut fails = Vec::new();
    for id in TABLETS {
        let p = builtin(id).unwrap();
        if p.gray_levels != NEW_STEPS {
            fails.push(format!("{id}:steps:{:?}", p.gray_levels));
        }
        if p.ink_values != [1, 157, 201, 254] {
            fails.push(format!("{id}:inks:{:?}", p.ink_values));
        }
        for &ink in &p.ink_values[..3] {
            if !p.gray_levels.contains(&ink) {
                fails.push(format!("{id}:pen_not_a_step:{ink}"));
            }
        }
        if p.gray_levels.contains(&254) {
            fails.push(format!("{id}:white_pen_is_a_step"));
        }
    }
    let ok = fails.is_empty();
    println!(
        "CHECK crate=profiles name=pen_scale_tablet_steps result={} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn roles_and_lint_grays_move_to_the_new_steps() {
    on();
    let mut fails = Vec::new();
    for id in TABLETS {
        let p = builtin(id).unwrap();
        let want = [
            (LineRole::Emphasis, 1),
            (LineRole::Guide, 53),
            (LineRole::Structure, 53),
            (LineRole::DotGrid, 53),
            (LineRole::Text, 53),
            (LineRole::Whisper, 122),
            (LineRole::Label, 157),
            (LineRole::Margin, 201),
        ];
        for (role, g) in want {
            if p.role(role).gray != g {
                fails.push(format!("{id}:{role:?}:{}", p.role(role).gray));
            }
        }
        if (p.fill_roles.shade, p.fill_roles.block) != (201, 105) {
            fails.push(format!("{id}:fills:{:?}", p.fill_roles));
        }
        for role in LineRole::ALL {
            if !p.gray_levels.contains(&p.role(role).gray) {
                fails.push(format!("{id}:{role:?}:not_a_step"));
            }
        }
        let l = &p.lint;
        let lint = [
            l.lightest_gray_line_1px,
            l.lightest_gray_line_2px_up,
            l.lightest_gray_dot_2px,
            l.lightest_gray_dot_3px,
            l.lightest_gray_dot_4px_up,
            l.lightest_gray_fill,
            l.lightest_gray_text,
            l.dark_fill_gray,
        ];
        for g in lint.into_iter().flatten() {
            if !p.gray_levels.contains(&g) {
                fails.push(format!("{id}:lint_gray_not_a_step:{g}"));
            }
        }
        if l.lightest_gray_text != Some(157) {
            fails.push(format!("{id}:lightest_text:{:?}", l.lightest_gray_text));
        }
    }
    let ok = fails.is_empty();
    println!(
        "CHECK crate=profiles name=pen_scale_roles_and_lint result={} fails={fails:?}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}

#[test]
fn paper_roles_move_and_the_registry_ships_the_new_values() {
    on();
    let r = Registry::builtin_only();
    let mut fails = Vec::new();
    for id in r.paper_ids() {
        let p = r.get(&id).unwrap();
        if !p.gray_levels.is_empty() || !p.ink_values.is_empty() {
            fails.push(format!("{id}:has_steps_or_pens"));
        }
        if p.role(LineRole::Label).gray != 157 || p.fill_roles.shade != 201 {
            fails.push(format!("{id}:label_or_shade"));
        }
    }
    for id in TABLETS {
        if r.get(id).map(|p| p.gray_levels.clone()) != Some(NEW_STEPS.to_vec()) {
            fails.push(format!("{id}:registry_steps"));
        }
    }
    let ok = fails.is_empty();
    println!(
        "CHECK crate=profiles name=pen_scale_paper_and_registry result={} papers={} fails={fails:?}",
        if ok { "pass" } else { "fail" },
        r.paper_ids().len()
    );
    assert!(ok);
}
