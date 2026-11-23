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

//! The calendar pattern and the mini calendar laid out (calendar bbq,
//! docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md).
//!
//! A plan says where a calendar's lines go and what words it draws: each word is a [`Label`] with its text,
//! its size and the box it is aligned in. The words are laid out as text like a text box's, one text job per
//! label under an id made from the object's ([`label_id`]), so they are saved with the page's other text
//! outlines and drawn from them ([`label_items`]). Every word is in the bundled Atkinson Hyperlegible Next, in
//! English (the page's words go through `crates/i18n/locales/en-US/calendar.ftl`).
//!
//! Sizes follow the room there is: a word shrinks to fit its cell, but never below the profile's smallest text
//! size; a kind of word that would be smaller is left out, and the plan says so for the linter's L07.

use crate::{Item, text_path};
use kurbo::Point;
use quadrule_i18n::{Message, Strings};
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{
    LineRole, MiniCalendarShape, MonthShape, Pattern, TextAlign, TextBox, Uuid, Weekday, YearMonth,
};
use quadrule_profiles::Profile;
use std::sync::OnceLock;

/// The family every calendar word is set in (bundled, so it is on every computer).
pub const FAMILY: &str = "Atkinson Hyperlegible Next";
/// The weight every calendar word is set at.
pub const WEIGHT: u16 = 400;
/// The smallest text size, in points, when the profile gives none.
pub const MIN_PT_DEFAULT: f64 = 6.0;

/// A weekday name's capital height, as a share of the header row's height.
const HEADER_CAP_SHARE: f64 = 0.4;
/// A day number's capital height, as a share of the day square's side.
const DAY_CAP_SHARE: f64 = 0.5;
/// A mini calendar's capital height, as a share of its row's height.
const MINI_CAP_SHARE: f64 = 0.5;
/// How much of its box's width a word may take: the rest is a margin either side.
const WIDTH_SHARE: f64 = 0.8;
/// How much of a mini calendar's width its title may take.
const TITLE_WIDTH_SHARE: f64 = 0.95;

/// The first slot of the day numbers: day `d` is slot `DAY_SLOT + d`. Slots 0 to 6 are the weekday names by column.
const DAY_SLOT: u32 = 100;
/// The slot of a mini calendar's title.
const TITLE_SLOT: u32 = 200;
/// Mixed into an object's id to make its labels' ids ("calendar" in ASCII).
const LABEL_SALT: u64 = 0x6361_6c65_6e64_6172;

/// SplitMix64's finaliser: every bit of the result depends on every bit of `x`.
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// The id the text outlines of label `slot` of the calendar `object` are kept under: both halves of the object's
/// id and the slot mixed, so no two labels of any objects share one (a plain XOR would give object 1's slot 2
/// the id of object 2's slot 1). The same on every computer, as it is saved in the file.
pub fn label_id(object: Uuid, slot: u32) -> Uuid {
    let id = object.as_u128();
    let (hi, lo) = ((id >> 64) as u64, id as u64);
    let s = LABEL_SALT ^ u64::from(slot);
    let a = mix(hi ^ mix(lo ^ s));
    let b = mix(lo ^ mix(hi ^ s.rotate_left(32)) ^ a);
    Uuid::from_u128((u128::from(a) << 64) | u128::from(b))
}

/// Which of a calendar's words a label is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelKind {
    /// The weekday names (or initials) heading the columns.
    Weekday,
    /// The day numbers.
    Day,
    /// A mini calendar's month and year.
    Title,
}

/// One word or number a calendar draws, in device pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// Which of the calendar's labels (its id is [`label_id`] of it).
    pub slot: u32,
    /// What kind of word it is.
    pub kind: LabelKind,
    /// The text.
    pub text: String,
    /// The size in points (to a tenth).
    pub size_pt: f64,
    /// The left of the box it is aligned in.
    pub x: f64,
    /// The width of that box.
    pub w: f64,
    /// Where its baseline goes, on a whole pixel.
    pub baseline: f64,
    /// How it sits in its box.
    pub align: TextAlign,
    /// Its gray.
    pub gray: u8,
}

impl Label {
    /// The text box that lays the label out: one line in the calendar's font, aligned in its box.
    pub fn text_box(&self) -> TextBox {
        TextBox {
            content: self.text.clone(),
            family: FAMILY.into(),
            weight: WEIGHT,
            size_pt: self.size_pt,
            line_height: 1.0,
            align: self.align,
            ..TextBox::default()
        }
    }
}

/// A kind of word left out because it would be smaller than the profile's smallest text size: the size it
/// would have needed, in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeftOut {
    /// What was left out.
    pub kind: LabelKind,
    /// The size that would have fitted, in points (to a tenth).
    pub pt: f64,
}

/// Where a calendar pattern's lines go and what it writes, in device pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct MonthPlan {
    /// The eight column edges, left to right.
    pub cols: [f64; 8],
    /// The top of the header row (the region's top).
    pub header_top: f64,
    /// The week rows' edges, top to bottom: the first is the header row's bottom, the last the region's bottom.
    pub rows: Vec<f64>,
    /// The month's shape.
    pub shape: MonthShape,
    /// The first column's day.
    pub week_start: Weekday,
    /// Whether the days are numbered (the calendar has a date).
    pub dated: bool,
    /// The side of a day's square (or, dated, of the corner its number sits in), whole pixels; 0 for none.
    pub square: f64,
    /// The words it draws.
    pub labels: Vec<Label>,
    /// The kinds of word left out for want of room.
    pub left_out: Vec<LeftOut>,
}

impl MonthPlan {
    /// How many week rows.
    pub fn weeks(&self) -> usize {
        self.rows.len().saturating_sub(1)
    }

    /// The day in the cell at week `row` and column `col`, or `None` outside the month (and outside the grid).
    pub fn day_at(&self, row: i64, col: i64) -> Option<u8> {
        if row < 0 || col < 0 || row as usize >= self.weeks() || col >= 7 {
            return None;
        }
        self.shape
            .day_at(row as u32 * 7 + col as u32, self.week_start)
    }
}

/// What a mini calendar writes, in device pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct MiniPlan {
    /// The words it draws.
    pub labels: Vec<Label>,
    /// The kinds of word left out for want of room.
    pub left_out: Vec<LeftOut>,
}

/// The page's words, in English.
fn words() -> &'static Strings {
    static WORDS: OnceLock<Strings> = OnceLock::new();
    WORDS.get_or_init(Strings::english)
}

/// A weekday's name as long as `len` says: 0 the whole name, 1 three letters, 2 one letter.
fn weekday_name(day: Weekday, len: usize) -> String {
    let id = match len {
        0 => format!("calendar-day-{}", day.key()),
        1 => format!("calendar-day-{}-short", day.key()),
        _ => format!("calendar-day-{}-letter", day.key()),
    };
    words().get(&id)
}

/// The month's name and year, as a mini calendar's title.
fn title_of(m: YearMonth) -> String {
    let month: &'static str = [
        "calendar-month-1",
        "calendar-month-2",
        "calendar-month-3",
        "calendar-month-4",
        "calendar-month-5",
        "calendar-month-6",
        "calendar-month-7",
        "calendar-month-8",
        "calendar-month-9",
        "calendar-month-10",
        "calendar-month-11",
        "calendar-month-12",
    ][usize::from(m.month_clamped() - 1)];
    words().format(
        &Message::new("calendar-title")
            .text("month", words().get(month))
            .text("year", m.year_clamped().to_string()),
    )
}

/// The widest of `texts` in em.
fn widest_em(texts: &[String]) -> f64 {
    texts
        .iter()
        .filter_map(|t| quadrule_text::signature::bundled_advance_em(FAMILY, WEIGHT, t))
        .fold(0.0, f64::max)
}

/// The capital height of the calendar's font in em.
fn cap_em() -> f64 {
    static CAP: OnceLock<f64> = OnceLock::new();
    *CAP.get_or_init(|| quadrule_text::signature::bundled_cap_em(FAMILY, WEIGHT).unwrap_or(0.7))
}

/// Sizes on one device: pixels to points, and the smallest size there.
struct Sizes {
    px_per_pt: f64,
    min_pt: f64,
}

impl Sizes {
    fn of(target: &Profile) -> Self {
        Sizes {
            px_per_pt: target.mm_to_px(quadrule_model::pt_to_mm(1.0)),
            min_pt: target.lint.smallest_text_pt.unwrap_or(MIN_PT_DEFAULT),
        }
    }

    /// The size, in points to a tenth (rounded down), whose capitals are `cap_px` tall.
    fn pt_for_cap(&self, cap_px: f64) -> f64 {
        self.floor(cap_px / cap_em() / self.px_per_pt)
    }

    /// The size, in points to a tenth (rounded down), at which `em` em are `w_px` wide.
    fn pt_for_width(&self, em: f64, w_px: f64) -> f64 {
        if em > 0.0 {
            self.floor(w_px / em / self.px_per_pt)
        } else {
            f64::INFINITY
        }
    }

    fn floor(&self, pt: f64) -> f64 {
        if pt.is_finite() {
            (pt * 10.0 + 1e-6).floor().max(0.0) / 10.0
        } else {
            0.0
        }
    }

    /// The capital height in pixels of text `pt` points tall.
    fn cap_px(&self, pt: f64) -> f64 {
        pt * self.px_per_pt * cap_em()
    }

    /// Where a line of capitals `pt` points tall stands to sit in the middle of `top`..`bottom`: its baseline,
    /// on a whole pixel.
    fn centred_baseline(&self, top: f64, bottom: f64, pt: f64) -> f64 {
        ((top + bottom + self.cap_px(pt)) / 2.0).round()
    }
}

/// `n + 1` edges dividing `a`..`b` into `n` equal parts, on whole pixels.
fn edges(a: f64, b: f64, n: usize) -> Vec<f64> {
    (0..=n)
        .map(|k| (a + (b - a) * k as f64 / n as f64).round())
        .collect()
}

/// The plan of a calendar pattern `pattern` in `region` (left, top, right, bottom, whole pixels) whose steps
/// across and down are `steps` pixels, on `target`; `None` for any other pattern, or a region too small to hold
/// one.
pub fn month_plan(
    pattern: &Pattern,
    region: (f64, f64, f64, f64),
    steps: (f64, f64),
    target: &Profile,
) -> Option<MonthPlan> {
    let Pattern::Calendar {
        month, week_start, ..
    } = *pattern
    else {
        return None;
    };
    let shape = pattern.month_shape()?;
    let (x0, y0, x1, y1) = region;
    if !(x1 - x0 >= 7.0 && y1 - y0 >= 2.0) {
        return None;
    }
    // The header row is one step down, but never more than a third of the region.
    let header_h = steps.1.min((y1 - y0) / 3.0).max(1.0);
    let header_bottom = (y0 + header_h).round();
    let weeks = shape.weeks(week_start) as usize;
    let cols: [f64; 8] = edges(x0, x1, 7).try_into().ok()?;
    let rows = edges(header_bottom, y1, weeks);
    let cell_w = (0..7)
        .map(|c| cols[c + 1] - cols[c])
        .fold(f64::INFINITY, f64::min);
    let cell_h = (0..weeks)
        .map(|r| rows[r + 1] - rows[r])
        .fold(f64::INFINITY, f64::min);
    // One step across, at most a third of a cell.
    let square = steps.0.min(cell_w / 3.0).min(cell_h / 3.0).floor().max(0.0);
    let sizes = Sizes::of(target);
    let label_gray = target.role(LineRole::Label).gray;
    let mut labels = Vec::new();
    let mut left_out = Vec::new();

    // The weekday names: the longest that fits every column, at the size the header's height gives.
    let days: Vec<Weekday> = (0..7)
        .map(|c| Weekday::from_index(week_start.index() + c))
        .collect();
    let by_height = sizes.pt_for_cap(HEADER_CAP_SHARE * (header_bottom - y0));
    let room = WIDTH_SHARE * cell_w;
    let mut pick = None;
    for len in 0..3 {
        let names: Vec<String> = days.iter().map(|d| weekday_name(*d, len)).collect();
        let pt = by_height.min(sizes.pt_for_width(widest_em(&names), room));
        if pt >= by_height || len == 2 {
            pick = Some((names, pt));
            break;
        }
    }
    if let Some((names, pt)) = pick {
        if pt + 1e-9 < sizes.min_pt {
            left_out.push(LeftOut {
                kind: LabelKind::Weekday,
                pt,
            });
        } else {
            let baseline = sizes.centred_baseline(y0, header_bottom, pt);
            for (c, name) in names.into_iter().enumerate() {
                labels.push(Label {
                    slot: c as u32,
                    kind: LabelKind::Weekday,
                    text: name,
                    size_pt: pt,
                    x: cols[c],
                    w: cols[c + 1] - cols[c],
                    baseline,
                    align: TextAlign::Center,
                    gray: label_gray,
                });
            }
        }
    }

    // The day numbers, dated: each alone in its day's top-right corner, where the square would be.
    if month.is_some() && square > 0.0 {
        let numbers: Vec<String> = (1..=shape.days).map(|d| d.to_string()).collect();
        let pt = sizes
            .pt_for_cap(DAY_CAP_SHARE * square)
            .min(sizes.pt_for_width(widest_em(&numbers), WIDTH_SHARE * square));
        if pt + 1e-9 < sizes.min_pt {
            left_out.push(LeftOut {
                kind: LabelKind::Day,
                pt,
            });
        } else {
            for cell in 0..(weeks as u32 * 7) {
                let Some(d) = shape.day_at(cell, week_start) else {
                    continue;
                };
                let (r, c) = ((cell / 7) as usize, (cell % 7) as usize);
                let (right, top) = (cols[c + 1], rows[r]);
                labels.push(Label {
                    slot: DAY_SLOT + u32::from(d),
                    kind: LabelKind::Day,
                    text: d.to_string(),
                    size_pt: pt,
                    x: right - square,
                    w: square,
                    baseline: sizes.centred_baseline(top, top + square, pt),
                    align: TextAlign::Center,
                    gray: label_gray,
                });
            }
        }
    }
    Some(MonthPlan {
        cols,
        header_top: y0,
        rows,
        shape,
        week_start,
        dated: month.is_some(),
        square,
        labels,
        left_out,
    })
}

/// The plan of mini calendar `m` in `frame` (left, top, right, bottom, whole pixels) on `target`; its numbers and
/// title are in `gray`, its weekday initials in the Label role's gray. Rows share the height: the title (when it
/// has one), the initials and each week the month spans; columns share the width.
pub fn mini_plan(
    m: &MiniCalendarShape,
    frame: (f64, f64, f64, f64),
    gray: u8,
    target: &Profile,
) -> MiniPlan {
    let (x0, y0, x1, y1) = frame;
    let mut labels = Vec::new();
    let mut left_out = Vec::new();
    if !(x1 - x0 >= 7.0 && y1 - y0 >= 3.0) {
        return MiniPlan { labels, left_out };
    }
    let shape = m.shape();
    let weeks = shape.weeks(m.week_start) as usize;
    let title_rows = usize::from(m.title);
    let rows = edges(y0, y1, title_rows + 1 + weeks);
    let cols = edges(x0, x1, 7);
    let row_h = (0..rows.len() - 1)
        .map(|r| rows[r + 1] - rows[r])
        .fold(f64::INFINITY, f64::min);
    let cell_w = (0..7)
        .map(|c| cols[c + 1] - cols[c])
        .fold(f64::INFINITY, f64::min);
    let sizes = Sizes::of(target);
    let label_gray = target.role(LineRole::Label).gray;

    // One size for the initials and the numbers: the row's height, unless the widest is too wide.
    let initials: Vec<(usize, String)> = (0..7)
        .map(|c| {
            let day = Weekday::from_index(m.week_start.index() + c as u32);
            (c, weekday_name(day, 2))
        })
        .collect();
    let mut texts: Vec<String> = initials.iter().map(|(_, t)| t.clone()).collect();
    texts.extend((1..=shape.days).map(|d| d.to_string()));
    let pt = sizes
        .pt_for_cap(MINI_CAP_SHARE * row_h)
        .min(sizes.pt_for_width(widest_em(&texts), WIDTH_SHARE * cell_w));
    let small = pt + 1e-9 < sizes.min_pt;

    if m.title {
        let title = title_of(m.month);
        let tpt = pt.min(sizes.pt_for_width(
            widest_em(std::slice::from_ref(&title)),
            TITLE_WIDTH_SHARE * (x1 - x0),
        ));
        if tpt + 1e-9 < sizes.min_pt {
            left_out.push(LeftOut {
                kind: LabelKind::Title,
                pt: tpt,
            });
        } else {
            labels.push(Label {
                slot: TITLE_SLOT,
                kind: LabelKind::Title,
                text: title,
                size_pt: tpt,
                x: x0,
                w: x1 - x0,
                baseline: sizes.centred_baseline(rows[0], rows[1], tpt),
                align: TextAlign::Center,
                gray,
            });
        }
    }
    if small {
        left_out.push(LeftOut {
            kind: LabelKind::Weekday,
            pt,
        });
        left_out.push(LeftOut {
            kind: LabelKind::Day,
            pt,
        });
        return MiniPlan { labels, left_out };
    }
    let head = title_rows;
    let baseline = sizes.centred_baseline(rows[head], rows[head + 1], pt);
    for (c, text) in initials {
        labels.push(Label {
            slot: c as u32,
            kind: LabelKind::Weekday,
            text,
            size_pt: pt,
            x: cols[c],
            w: cols[c + 1] - cols[c],
            baseline,
            align: TextAlign::Center,
            gray: label_gray,
        });
    }
    for cell in 0..(weeks as u32 * 7) {
        let Some(d) = shape.day_at(cell, m.week_start) else {
            continue;
        };
        let (r, c) = (head + 1 + (cell / 7) as usize, (cell % 7) as usize);
        labels.push(Label {
            slot: DAY_SLOT + u32::from(d),
            kind: LabelKind::Day,
            text: d.to_string(),
            size_pt: pt,
            x: cols[c],
            w: cols[c + 1] - cols[c],
            baseline: sizes.centred_baseline(rows[r], rows[r + 1], pt),
            align: TextAlign::Center,
            gray,
        });
    }
    MiniPlan { labels, left_out }
}

/// The labels of calendar `object` drawn from `outlines` (`px_per_mm`: the device's scale): each label's
/// letters as a filled path, its baseline where the plan puts it. A label the outlines do not hold is left out.
pub fn label_items(
    labels: &[Label],
    object: Uuid,
    outlines: &ProfileOutlines,
    px_per_mm: f64,
) -> Vec<Item> {
    let mut out = Vec::new();
    for l in labels {
        let Some(text) = outlines.text(label_id(object, l.slot)) else {
            continue;
        };
        let Some(first) = text.glyphs.first() else {
            continue;
        };
        // The text was laid out from its box's top; move it so its baseline lands on the plan's.
        let origin = Point::new(l.x, l.baseline - f64::from(first.y_mm()) * px_per_mm);
        if let Ok(path) = text_path(outlines, text, origin, px_per_mm) {
            out.push(Item::Fill { path, gray: l.gray });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_ids_differ_by_slot_and_object() {
        // Neighbouring ids, every slot a calendar uses: all different, and none an object's own.
        let mut seen = std::collections::HashSet::new();
        for o in 0..64u128 {
            for slot in (0..7).chain(101..132).chain([200]) {
                assert!(seen.insert(label_id(Uuid::from_u128(o), slot)));
            }
        }
        assert!((0..64u128).all(|o| !seen.contains(&Uuid::from_u128(o))));
    }

    #[test]
    fn weekday_names_come_in_three_lengths() {
        assert_eq!(weekday_name(Weekday::Wednesday, 0), "Wednesday");
        assert_eq!(weekday_name(Weekday::Wednesday, 1), "Wed");
        assert_eq!(weekday_name(Weekday::Wednesday, 2), "W");
        assert_eq!(title_of(YearMonth::new(2026, 11)), "November 2026");
    }

    #[test]
    fn edges_divide_evenly_on_whole_pixels() {
        assert_eq!(edges(0.0, 10.0, 3), vec![0.0, 3.0, 7.0, 10.0]);
    }
}
