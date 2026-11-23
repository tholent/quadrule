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

//! Months and weekdays for the calendar pattern and the mini calendar (format 0.19; calendar bbq,
//! docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md). The proleptic Gregorian calendar, worked out
//! here so a page draws the same on every computer whatever its clock or time zone says.

use crate::Extra;
use serde::{Deserialize, Serialize};

/// A day of the week (format 0.19).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Weekday {
    /// The first day of a new calendar's week (calendar bbq, Q3).
    #[default]
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    /// Every day, Monday first.
    pub const ALL: [Weekday; 7] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    /// Its place in the week counted from Monday: 0 for Monday, 6 for Sunday.
    pub fn index(self) -> u32 {
        self as u32
    }

    /// The day `i` days after Monday (wrapping round the week).
    pub fn from_index(i: u32) -> Weekday {
        Weekday::ALL[(i % 7) as usize]
    }

    /// Whether this is Monday; used to leave the field out of a file when it is.
    pub fn is_monday(&self) -> bool {
        *self == Weekday::Monday
    }

    /// How many days after `start` this day comes, 0 to 6: its column in a week that starts on `start`.
    pub fn column_from(self, start: Weekday) -> u32 {
        (self.index() + 7 - start.index()) % 7
    }

    /// The short English key of the day (`mon` to `sun`), for message ids such as `calendar-day-mon`.
    pub fn key(self) -> &'static str {
        ["mon", "tue", "wed", "thu", "fri", "sat", "sun"][self as usize]
    }
}

/// Whether `year` is a leap year in the Gregorian calendar.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// The fewest days a month has.
pub const MONTH_DAYS_MIN: u8 = 28;
/// The most days a month has.
pub const MONTH_DAYS_MAX: u8 = 31;
/// The earliest year a calendar is offered.
pub const YEAR_MIN: i32 = 1;
/// The latest year a calendar is offered.
pub const YEAR_MAX: i32 = 9999;

/// A month of a year (format 0.19): what a dated calendar shows.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YearMonth {
    /// The year, such as 2026.
    pub year: i32,
    /// The month, 1 for January to 12 for December. A file holding another number is read as the
    /// nearest month ([`YearMonth::month_clamped`]).
    pub month: u8,
}

impl YearMonth {
    /// The month `month` (1 to 12) of `year`.
    pub fn new(year: i32, month: u8) -> Self {
        YearMonth { year, month }
    }

    /// The month that counts, 1 to 12.
    pub fn month_clamped(&self) -> u8 {
        self.month.clamp(1, 12)
    }

    /// The year that counts, [`YEAR_MIN`] to [`YEAR_MAX`].
    pub fn year_clamped(&self) -> i32 {
        self.year.clamp(YEAR_MIN, YEAR_MAX)
    }

    /// How many days the month has, 28 to 31.
    pub fn days(&self) -> u8 {
        match self.month_clamped() {
            2 if is_leap_year(self.year_clamped()) => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    /// The weekday the month's 1st falls on.
    pub fn first_weekday(&self) -> Weekday {
        weekday_of(self.year_clamped(), self.month_clamped(), 1)
    }

    /// The month after this one.
    pub fn next(&self) -> YearMonth {
        match self.month_clamped() {
            12 => YearMonth::new((self.year_clamped() + 1).min(YEAR_MAX), 1),
            m => YearMonth::new(self.year_clamped(), m + 1),
        }
    }

    /// The month before this one.
    pub fn previous(&self) -> YearMonth {
        match self.month_clamped() {
            1 => YearMonth::new((self.year_clamped() - 1).max(YEAR_MIN), 12),
            m => YearMonth::new(self.year_clamped(), m - 1),
        }
    }

    /// The month holding the day `days` days after 1 January 1970 (a Unix day number), as a clock gives it.
    pub fn from_unix_days(days: i64) -> YearMonth {
        // Howard Hinnant's civil_from_days: count in 400-year eras starting 1 March 0000.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        YearMonth::new(
            year.clamp(i64::from(YEAR_MIN), i64::from(YEAR_MAX)) as i32,
            month as u8,
        )
    }
}

/// The weekday of `day` `month` `year` (month 1 to 12), by Sakamoto's method.
pub fn weekday_of(year: i32, month: u8, day: u8) -> Weekday {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let m = month.clamp(1, 12);
    let y = if m < 3 { year - 1 } else { year };
    let sunday_first = (y + y.div_euclid(4) - y.div_euclid(100)
        + y.div_euclid(400)
        + T[usize::from(m - 1)]
        + i32::from(day))
    .rem_euclid(7);
    // 0 is Sunday here; Monday is 0 in [`Weekday`].
    Weekday::from_index((sunday_first as u32 + 6) % 7)
}

/// The shape of a month on a calendar: the weekday its 1st falls on and how many days it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthShape {
    /// The weekday of the 1st.
    pub first: Weekday,
    /// How many days, 28 to 31.
    pub days: u8,
}

impl MonthShape {
    /// The cell of the 1st in a week starting on `week_start`, counted from the top-left cell: its column.
    pub fn offset(&self, week_start: Weekday) -> u32 {
        self.first.column_from(week_start)
    }

    /// How many week rows the month spans in a week starting on `week_start`, 4 to 6.
    pub fn weeks(&self, week_start: Weekday) -> u32 {
        (self.offset(week_start) + u32::from(self.days)).div_ceil(7)
    }

    /// The day in cell `cell` (counted from the top-left, across then down), or `None` for a cell outside the
    /// month.
    pub fn day_at(&self, cell: u32, week_start: Weekday) -> Option<u8> {
        let off = self.offset(week_start);
        (cell >= off && cell < off + u32::from(self.days)).then(|| (cell - off + 1) as u8)
    }
}

/// The days a new undated calendar has (calendar bbq, Q1).
pub const UNDATED_DAYS_DEFAULT: u8 = 31;

/// A mini calendar (format 0.19; calendar bbq, Q2): a compact month block of weekday initials and day numbers,
/// with no cells to write in, placed anywhere. Always dated. It uses the object's `frame`, `style` (the gray of its
/// numbers and title: the Text role unless changed), `rotation_deg` and `opacity_pct`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MiniCalendarShape {
    /// The month shown.
    pub month: YearMonth,
    /// The first column's day (Monday unless the file says otherwise).
    #[serde(default, skip_serializing_if = "Weekday::is_monday")]
    pub week_start: Weekday,
    /// Whether the month's name and year head the block (on unless the file says otherwise).
    #[serde(default = "crate::yes", skip_serializing_if = "crate::is_true")]
    pub title: bool,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

impl MiniCalendarShape {
    /// A mini calendar of `month`, weeks starting on Monday, with its title.
    pub fn new(month: YearMonth) -> Self {
        MiniCalendarShape {
            month,
            week_start: Weekday::Monday,
            title: true,
            extra: Extra::new(),
        }
    }

    /// The shape of its month.
    pub fn shape(&self) -> MonthShape {
        MonthShape {
            first: self.month.first_weekday(),
            days: self.month.days(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_have_their_days_and_first_weekdays() {
        // November 2026 starts on a Sunday; February 2028 is a leap month starting on a Tuesday.
        let nov = YearMonth::new(2026, 11);
        assert_eq!((nov.days(), nov.first_weekday()), (30, Weekday::Sunday));
        let feb = YearMonth::new(2028, 2);
        assert_eq!((feb.days(), feb.first_weekday()), (29, Weekday::Tuesday));
        assert_eq!(YearMonth::new(1900, 2).days(), 28);
        assert_eq!(YearMonth::new(2000, 2).days(), 29);
        assert_eq!(weekday_of(1970, 1, 1), Weekday::Thursday);
        assert_eq!(YearMonth::new(2026, 12).next(), YearMonth::new(2027, 1));
        assert_eq!(YearMonth::new(2027, 1).previous(), YearMonth::new(2026, 12));
    }

    #[test]
    fn unix_days_give_their_month() {
        assert_eq!(YearMonth::from_unix_days(0), YearMonth::new(1970, 1));
        // 2026-11-23 is day 20780.
        assert_eq!(YearMonth::from_unix_days(20_780), YearMonth::new(2026, 11));
        assert_eq!(YearMonth::from_unix_days(-1), YearMonth::new(1969, 12));
    }

    #[test]
    fn a_month_spans_its_weeks() {
        let nov = MonthShape {
            first: Weekday::Sunday,
            days: 30,
        };
        // Monday first: the 1st is in the last column, and the month spans six rows.
        assert_eq!(
            (nov.offset(Weekday::Monday), nov.weeks(Weekday::Monday)),
            (6, 6)
        );
        // Sunday first: the 1st is in the first column, five rows.
        assert_eq!(
            (nov.offset(Weekday::Sunday), nov.weeks(Weekday::Sunday)),
            (0, 5)
        );
        let feb = MonthShape {
            first: Weekday::Monday,
            days: 28,
        };
        assert_eq!(feb.weeks(Weekday::Monday), 4);
        assert_eq!(nov.day_at(6, Weekday::Monday), Some(1));
        assert_eq!(nov.day_at(5, Weekday::Monday), None);
        assert_eq!(nov.day_at(35, Weekday::Monday), Some(30));
        assert_eq!(nov.day_at(36, Weekday::Monday), None);
    }
}
