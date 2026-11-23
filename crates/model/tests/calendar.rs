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

//! The calendar pattern and the mini calendar in a file (docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md;
//! format 0.19): the defaults (Monday, no date, the title on) are left out of a file and read back, a set value
//! round-trips, a mini calendar keeps a newer file's unknown field, and an undated month's days are clamped to 28
//! to 31. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn cbor<T: serde::Serialize>(v: &T) -> ciborium::Value {
    ciborium::Value::serialized(v).unwrap()
}

/// Every text key anywhere in `v`.
fn all_keys(v: &ciborium::Value, out: &mut Vec<String>) {
    match v {
        ciborium::Value::Map(m) => {
            for (k, x) in m {
                if let Some(t) = k.as_text() {
                    out.push(t.to_string());
                }
                all_keys(x, out);
            }
        }
        ciborium::Value::Array(a) => a.iter().for_each(|x| all_keys(x, out)),
        _ => {}
    }
}

fn keys(v: &ciborium::Value) -> Vec<String> {
    let mut out = Vec::new();
    all_keys(v, &mut out);
    out
}

#[test]
fn calendar_fields_default_round_trip_and_keep_the_unknown() {
    // A new calendar region writes only its days; Monday and no date are left out.
    let fresh = PatternRegion::new(PatternKind::Calendar.default_pattern());
    let k = keys(&cbor(&fresh));
    let bare = k.contains(&"Calendar".to_string())
        && k.contains(&"days".to_string())
        && !k
            .iter()
            .any(|x| ["month", "week_start", "first_day"].contains(&x.as_str()));
    let back: PatternRegion = cbor(&fresh).deserialized().unwrap();
    // A dated one, weeks from Sunday, round-trips.
    let dated = PatternRegion::new(Pattern::Calendar {
        month: Some(YearMonth::new(2027, 2)),
        week_start: Weekday::Sunday,
        first_day: Weekday::Friday,
        days: 30,
    });
    let dated_back: PatternRegion = cbor(&dated).deserialized().unwrap();
    // A mini calendar: the title and Monday are left out; an unknown field from a newer file is kept.
    let mini = MiniCalendarShape::new(YearMonth::new(2026, 11));
    let mk = keys(&cbor(&mini));
    let mini_bare = mk == ["month", "year", "month"];
    let mut with_extra = cbor(&mini);
    if let ciborium::Value::Map(m) = &mut with_extra {
        m.push((
            ciborium::Value::Text("x_mini".into()),
            ciborium::Value::Integer(7.into()),
        ));
    }
    let extra: MiniCalendarShape = with_extra.deserialized().unwrap();
    let kept =
        extra.extra.contains_key("x_mini") && keys(&cbor(&extra)).contains(&"x_mini".to_string());
    let ok = bare && back == fresh && dated_back == dated && mini_bare && kept;
    println!(
        "CHECK crate=model name=calendar_fields result={} default_writes_only_days={bare} default_reads_back={} dated_round_trip={} mini_keys={mk:?} unknown_kept={kept}",
        pass(ok),
        back == fresh,
        dated_back == dated
    );
    assert!(ok);
}

#[test]
fn an_undated_months_days_are_clamped() {
    let shape = |days: u8| {
        Pattern::Calendar {
            month: None,
            week_start: Weekday::Monday,
            first_day: Weekday::Tuesday,
            days,
        }
        .month_shape()
        .unwrap()
        .days
    };
    let (low, high, mid) = (shape(3), shape(200), shape(29));
    // Dated, its own two settings are kept but unused.
    let dated = Pattern::Calendar {
        month: Some(YearMonth::new(2026, 11)),
        week_start: Weekday::Monday,
        first_day: Weekday::Tuesday,
        days: 28,
    }
    .month_shape()
    .unwrap();
    let ok = (low, high, mid) == (28, 31, 29)
        && dated.days == 30
        && dated.first == Weekday::Sunday
        && Pattern::Lines.month_shape().is_none();
    println!(
        "CHECK crate=model name=calendar_days_clamped result={} low={low} high={high} mid={mid} dated_days={} dated_first={:?}",
        pass(ok),
        dated.days,
        dated.first
    );
    assert!(ok);
}
