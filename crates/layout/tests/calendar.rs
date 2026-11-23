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

//! The calendar pattern and the mini calendar laid out (docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md):
//! a month's cells and which are boxed, the day squares, the weekday names by the room there is, the week's first
//! day, words too small left out, and the words laid out as text the page saves. Each test prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::calendar::{LabelKind, MonthPlan, label_id, mini_plan, month_plan};
use quadrule_layout::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn manta() -> Profile {
    builtin(MANTA_ID).unwrap()
}

/// A calendar pattern of `month` (or undated, the 1st on `first`, with `days`), weeks starting on `start`.
fn calendar(month: Option<YearMonth>, start: Weekday, first: Weekday, days: u8) -> Pattern {
    Pattern::Calendar {
        month,
        week_start: start,
        first_day: first,
        days,
    }
}

/// A page holding one calendar region of `p` at (10, 20) mm, `w` by `h` mm, on a 5 mm rhythm (59 px on the Manta).
fn page_with(p: Pattern, w: f64, h: f64) -> Document {
    let mut doc = slice_document();
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    let mut o = doc.layers[0].objects[0].clone();
    o.frame = Frame::new(10.0, 20.0, w, h);
    o.constraints = Constraints::default();
    o.style = Style::Role(LineRole::Guide);
    o.kind = ObjectKind::Pattern(PatternRegion::new(p));
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects: vec![o],
        extra: Extra::new(),
    }];
    doc
}

/// The plan of the page's one region on the Manta.
fn plan_of(doc: &Document) -> MonthPlan {
    let m = manta();
    let o = &doc.layers[0].objects[0];
    let ObjectKind::Pattern(pr) = &o.kind else {
        panic!("a pattern region");
    };
    let r = RhythmPx::new(doc, &m);
    month_plan(
        &pr.pattern,
        region_px(&o.frame, &m),
        (r.unit_x, r.unit_y),
        &m,
    )
    .unwrap()
}

fn items_of(doc: &Document) -> Vec<Item> {
    let m = manta();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &m, &m, &mut reg).unwrap();
    display_list(doc, &m, &m, &laid.outlines).unwrap().items
}

/// Hard-edged rectangles: (x0, y0, x1, y1).
fn rects(items: &[Item]) -> Vec<(f64, f64, f64, f64)> {
    items
        .iter()
        .filter_map(|i| match *i {
            Item::Rect { x0, y0, x1, y1, .. } => Some((x0, y0, x1, y1)),
            _ => None,
        })
        .collect()
}

/// Whether some drawn horizontal line covers (x, y).
fn hline_at(rs: &[(f64, f64, f64, f64)], x: f64, y: f64) -> bool {
    rs.iter()
        .any(|&(x0, y0, x1, y1)| x1 - x0 > y1 - y0 && x0 <= x && x <= x1 && y0 <= y && y <= y1)
}

/// Whether some drawn vertical line covers (x, y).
fn vline_at(rs: &[(f64, f64, f64, f64)], x: f64, y: f64) -> bool {
    rs.iter()
        .any(|&(x0, y0, x1, y1)| y1 - y0 > x1 - x0 && x0 <= x && x <= x1 && y0 <= y && y <= y1)
}

#[test]
fn a_dated_month_boxes_only_its_own_days() {
    // November 2026 starts on a Sunday: Monday first, the 1st is alone in the first row's last column.
    let doc = page_with(
        calendar(
            Some(YearMonth::new(2026, 11)),
            Weekday::Monday,
            Weekday::Monday,
            31,
        ),
        105.0,
        70.0,
    );
    let plan = plan_of(&doc);
    let rs = rects(&items_of(&doc));
    let mid = |c: usize| (plan.cols[c] + plan.cols[c + 1]) / 2.0;
    let vmid = |r: usize| (plan.rows[r] + plan.rows[r + 1]) / 2.0;
    let shape = plan.weeks() == 6
        && plan.day_at(0, 6) == Some(1)
        && plan.day_at(0, 5).is_none()
        && plan.day_at(5, 0) == Some(30)
        && plan.day_at(5, 1).is_none();
    // The first row's top edge is drawn over Sunday only; the last row's bottom under Monday only.
    let top =
        hline_at(&rs, mid(6), plan.rows[0]) && !(0..6).any(|c| hline_at(&rs, mid(c), plan.rows[0]));
    let bottom =
        hline_at(&rs, mid(0), plan.rows[6]) && !(1..7).any(|c| hline_at(&rs, mid(c), plan.rows[6]));
    // The left edge steps round: nothing left of Monday in the first row, a line from the second row on.
    let left = !vline_at(&rs, plan.cols[0], vmid(0)) && vline_at(&rs, plan.cols[0], vmid(1));
    // Dated: no squares, but every day's number (30) and the seven names.
    let days = plan
        .labels
        .iter()
        .filter(|l| l.kind == LabelKind::Day)
        .count();
    let names = plan
        .labels
        .iter()
        .filter(|l| l.kind == LabelKind::Weekday)
        .count();
    let ok = shape && top && bottom && left && days == 30 && names == 7 && plan.left_out.is_empty();
    println!(
        "CHECK crate=layout name=calendar_boxes_its_days result={} weeks={} first_row_top={top} last_row_bottom={bottom} left_steps={left} numbers={days} names={names}",
        pass(ok),
        plan.weeks()
    );
    assert!(ok);
}

#[test]
fn an_undated_month_has_a_square_in_each_day() {
    // 31 days, the 1st on a Wednesday, Sunday first: five rows, the 1st in the fourth column.
    let doc = page_with(
        calendar(None, Weekday::Sunday, Weekday::Wednesday, 31),
        105.0,
        70.0,
    );
    let plan = plan_of(&doc);
    let rs = rects(&items_of(&doc));
    let s = plan.square;
    // A square's left side: a short vertical line `s` in from a cell's right edge, from its top.
    let mut squares = 0;
    for r in 0..plan.weeks() {
        for c in 0..7 {
            let (x, y) = (plan.cols[c + 1] - s, plan.rows[r] + s / 2.0);
            if vline_at(&rs, x, y) {
                squares += 1;
                assert!(
                    plan.day_at(r as i64, c as i64).is_some(),
                    "a square outside the month"
                );
            }
        }
    }
    let no_numbers = plan.labels.iter().all(|l| l.kind == LabelKind::Weekday);
    // One rhythm unit across (59 px), unless a third of the cell is less.
    let cell_h = plan.rows[1] - plan.rows[0];
    let size = s
        == 59.0_f64
            .min(((plan.cols[1] - plan.cols[0]) / 3.0).min(cell_h / 3.0))
            .floor();
    let ok =
        plan.weeks() == 5 && plan.day_at(0, 3) == Some(1) && squares == 31 && no_numbers && size;
    println!(
        "CHECK crate=layout name=calendar_undated_squares result={} weeks={} squares={squares} square_px={s} numbers={}",
        pass(ok),
        plan.weeks(),
        !no_numbers
    );
    assert!(ok);
}

#[test]
fn weekday_names_follow_the_room_there_is() {
    let p = calendar(None, Weekday::Monday, Weekday::Monday, 30);
    let first = |w: f64| {
        let plan = plan_of(&page_with(p, w, 60.0));
        plan.labels
            .iter()
            .find(|l| l.kind == LabelKind::Weekday && l.slot == 2)
            .map(|l| l.text.clone())
            .unwrap_or_default()
    };
    // The third column's name (Wednesday, the longest) in a wide, a middle and a narrow calendar.
    let (wide, middle, narrow) = (first(180.0), first(105.0), first(50.0));
    let ok = wide == "Wednesday" && middle == "Wed" && narrow == "W";
    println!(
        "CHECK crate=layout name=calendar_weekday_names result={} wide={wide} middle={middle} narrow={narrow}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_week_starts_where_the_setting_says() {
    let nov = Some(YearMonth::new(2026, 11));
    let sunday = plan_of(&page_with(
        calendar(nov, Weekday::Sunday, Weekday::Monday, 31),
        105.0,
        70.0,
    ));
    let names: Vec<&str> = sunday
        .labels
        .iter()
        .filter(|l| l.kind == LabelKind::Weekday)
        .map(|l| l.text.as_str())
        .collect();
    let ok = sunday.weeks() == 5 && sunday.day_at(0, 0) == Some(1) && names.first() == Some(&"Sun");
    println!(
        "CHECK crate=layout name=calendar_week_start result={} weeks={} first_cell={:?} first_name={:?}",
        pass(ok),
        sunday.weeks(),
        sunday.day_at(0, 0),
        names.first()
    );
    assert!(ok);
}

#[test]
fn words_too_small_to_read_are_left_out() {
    // 35 by 21 mm: a header row of a third of 21 mm would do, but the numbers' corners are tiny.
    let small = plan_of(&page_with(
        calendar(
            Some(YearMonth::new(2026, 11)),
            Weekday::Monday,
            Weekday::Monday,
            31,
        ),
        35.0,
        14.0,
    ));
    let min = manta().lint.smallest_text_pt.unwrap();
    let days_out = small
        .left_out
        .iter()
        .any(|l| l.kind == LabelKind::Day && l.pt < min);
    let none_small = small.labels.iter().all(|l| l.size_pt >= min);
    let no_days = small.labels.iter().all(|l| l.kind != LabelKind::Day);
    // A mini calendar squeezed into 10 by 8 mm leaves its numbers out too.
    let m = manta();
    let mini = mini_plan(
        &MiniCalendarShape::new(YearMonth::new(2026, 11)),
        region_px(&Frame::new(10.0, 10.0, 10.0, 8.0), &m),
        0,
        &m,
    );
    let mini_out = mini.left_out.iter().any(|l| l.kind == LabelKind::Day) && mini.labels.is_empty();
    let ok = days_out && none_small && no_days && mini_out;
    println!(
        "CHECK crate=layout name=calendar_small_left_out result={} days_left_out={days_out} drawn_at_least_min={none_small} mini_left_out={mini_out} left_out={:?}",
        pass(ok),
        small.left_out
    );
    assert!(ok);
}

#[test]
fn the_words_are_laid_out_as_text_on_each_device() {
    // The calendar page: every label of every calendar has outlines under its own id, on both devices.
    let doc = calendar_document();
    let primary = manta();
    let mut reg = FontRegistry::bundled_only();
    let mut missing = 0;
    let mut labels = 0;
    for id in doc.profile_ids() {
        let target = builtin(id).unwrap();
        let laid = lay_out_text(&doc, &primary, &target, &mut reg).unwrap();
        let jobs = text_jobs(&doc, &primary, &target);
        for j in jobs.iter().filter(|j| doc.object(j.object_id).is_none()) {
            labels += 1;
            if laid
                .outlines
                .text(j.object_id)
                .is_none_or(|t| t.glyphs.is_empty())
            {
                missing += 1;
            }
        }
        // The dated calendar's 1st, by its id.
        assert!(
            laid.outlines
                .text(label_id(calendar_ids::DATED, 101))
                .is_some()
        );
        assert_eq!(laid.missing_glyphs, 0);
    }
    // Per device: 7 + 30 (dated), 7 (undated), 1 + 7 + 30 (November mini), 7 + 29 (February mini) = 118.
    let ok = missing == 0 && labels == 2 * 118;
    println!(
        "CHECK crate=layout name=calendar_words_laid_out result={} labels={labels} missing={missing}",
        pass(ok)
    );
    assert!(ok);
}
