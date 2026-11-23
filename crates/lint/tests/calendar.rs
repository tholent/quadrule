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

//! L07 on calendars (docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md, Defaulted "Text drawn by the
//! pattern"): a calendar's words shrink to fit their cells, and a kind of word that would be smaller than the
//! profile's smallest text size is left out and L07 says so; the calendar page, whose words all fit, has no L07.
//! Prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// The L07 findings on `doc`, on both Supernotes, with their messages in English.
fn l07(doc: &Document) -> Vec<(Option<Uuid>, String)> {
    let primary = builtin(MANTA_ID).unwrap();
    let strings = quadrule_i18n::Strings::english();
    let mut out = Vec::new();
    for id in [MANTA_ID, NOMAD_ID] {
        let target = builtin(id).unwrap();
        let laid = lay_out_text(doc, &primary, &target, &mut FontRegistry::bundled_only()).unwrap();
        for f in lint(doc, &primary, &target, &laid.outlines) {
            if f.rule == "L07" {
                out.push((f.object, strings.format(&f.message)));
            }
        }
    }
    out
}

#[test]
fn l07_says_when_a_calendar_leaves_its_words_out() {
    let page = calendar_document();
    let quiet = l07(&page).is_empty();
    // The dated calendar squeezed to 35 by 14 mm, the November mini calendar to 10 by 8 mm.
    let mut small = page.clone();
    small.object_mut(calendar_ids::DATED).unwrap().frame.w_mm = 35.0;
    small.object_mut(calendar_ids::DATED).unwrap().frame.h_mm = 14.0;
    small.object_mut(calendar_ids::MINI).unwrap().frame.w_mm = 10.0;
    small.object_mut(calendar_ids::MINI).unwrap().frame.h_mm = 8.0;
    let found = l07(&small);
    let on = |id: Uuid, what: &str| {
        found
            .iter()
            .any(|(o, m)| *o == Some(id) && m.contains(what) && m.contains("left out"))
    };
    let dated = on(calendar_ids::DATED, "day numbers");
    let mini = on(calendar_ids::MINI, "day numbers") && on(calendar_ids::MINI, "weekday names");
    let others = found
        .iter()
        .all(|(o, _)| [Some(calendar_ids::DATED), Some(calendar_ids::MINI)].contains(o));
    let ok = quiet && dated && mini && others;
    println!(
        "CHECK crate=lint name=calendar_l07 result={} page_quiet={quiet} dated_days={dated} mini_words={mini} only_these={others} findings={}",
        pass(ok),
        found.len()
    );
    assert!(ok, "{found:#?}");
}
