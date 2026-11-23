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

//! The calendar in the editor (docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md): the Mini calendar tool
//! (M) places a mini calendar of this month, seven rhythm units by eight, in the Text role, as one undo step; a
//! pattern region turned into a Calendar starts undated, 31 days, the 1st on a Monday, weeks starting on Monday,
//! and dating it is one edit. Headless. Each test prints one structured `CHECK` line (run with `-- --nocapture`).
//!
//! Coordinates are Manta device pixels. A new Handwriting page has 12 mm margins: 142 px, so the rhythm origin is
//! (142, 142) and its unit is 71 px.

use quadrule_layout::newpage::NewPage;
use quadrule_model::blank::RhythmPreset;
use quadrule_model::sample::MANTA_ID;
use quadrule_model::{LineRole, ObjectKind, Pattern, PatternKind, Style, Weekday, YearMonth};
use quadrule_ui::{DragMods, Tool, Viewer};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const NONE: DragMods = DragMods {
    shift: false,
    alt: false,
    snapping_off: false,
};

fn page() -> Viewer {
    let mut v = Viewer::default();
    v.open_new(&NewPage {
        title: "Calendar".into(),
        device_id: MANTA_ID.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation: quadrule_model::Orientation::Portrait,
    })
    .unwrap();
    v
}

#[test]
fn the_mini_calendar_tool_places_this_month() {
    let mut v = page();
    let before = v.doc().unwrap().objects().count();
    v.set_tool(Tool::MiniCalendar);
    let key = Tool::MiniCalendar.key() == egui::Key::M
        && Tool::from_name("mini-calendar") == Some(Tool::MiniCalendar);
    let id = v
        .place_click(213.0, 213.0, NONE, 6.0)
        .expect("a mini calendar");
    let doc = v.doc().unwrap();
    let o = doc.object(id).unwrap();
    let now = quadrule_ui::draw::this_month();
    let month = matches!(&o.kind, ObjectKind::MiniCalendar(m) if m.month == now && m.week_start == Weekday::Monday && m.title);
    let role = o.style == Style::Role(LineRole::Text);
    // Seven units by eight: 497 by 568 px.
    let k = 300.0 / quadrule_model::MM_PER_IN;
    let size =
        ((o.frame.w_mm * k - 497.0).abs() < 0.01) && ((o.frame.h_mm * k - 568.0).abs() < 0.01);
    let one = doc.objects().count() == before + 1 && v.selection == [id] && v.tool == Tool::Select;
    v.undo();
    let undone = v.doc().unwrap().object(id).is_none();
    let ok = key && month && role && size && one && undone;
    println!(
        "CHECK crate=ui name=mini_calendar_tool result={} key_m={key} this_month={month} text_role={role} seven_by_eight={size} selected={one} one_undo={undone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_pattern_region_becomes_a_calendar_and_is_dated_in_one_edit() {
    let mut v = page();
    v.set_tool(Tool::Pattern);
    let id = v.place_click(300.0, 300.0, NONE, 6.0).expect("a region");
    let listed = PatternKind::ALL.contains(&PatternKind::Calendar);
    v.edit_selected(None, |o| o.set_pattern_kind(PatternKind::Calendar))
        .unwrap();
    let doc = v.doc().unwrap();
    let o = doc.object(id).unwrap();
    let fresh = matches!(&o.kind, ObjectKind::Pattern(pr) if pr.pattern == Pattern::Calendar {
        month: None,
        week_start: Weekday::Monday,
        first_day: Weekday::Monday,
        days: 31,
    }) && o.style == Style::Role(LineRole::Guide);
    v.edit_selected(None, |o| {
        if let ObjectKind::Pattern(pr) = &mut o.kind
            && let Pattern::Calendar { month, .. } = &mut pr.pattern
        {
            *month = Some(YearMonth::new(2026, 11));
        }
    })
    .unwrap();
    let dated = matches!(
        &v.doc().unwrap().object(id).unwrap().kind,
        ObjectKind::Pattern(pr) if pr.pattern.month_shape().is_some_and(|s| s.first == Weekday::Sunday && s.days == 30)
    );
    v.undo();
    let back = matches!(
        &v.doc().unwrap().object(id).unwrap().kind,
        ObjectKind::Pattern(pr) if matches!(pr.pattern, Pattern::Calendar { month: None, .. })
    );
    let ok = listed && fresh && dated && back;
    println!(
        "CHECK crate=ui name=calendar_pattern_kind result={} listed={listed} undated_31_monday={fresh} dated_nov_2026={dated} undo_one_step={back}",
        pass(ok)
    );
    assert!(ok);
}
