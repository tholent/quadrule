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

//! Where a region's major lines count from (ADR-023 as amended 2026-11-23; docs/bbq/2026-11-23-major-lines-from-top.md):
//! Top, Centre and Rhythm origin on lines, the grid, dots and Cornell, the rows and the columns each set on their own;
//! Centre's odd line after the last major; a border that does not move the majors; isometric, which has no Top, its
//! three families' majors meeting on a major column however its rows and columns are set; a ruling turned a quarter
//! turn inside its frame, counted in its own axes. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).
//!
//! The region is columns 2 to 10 and rows 2 to 10 of a 5 mm rhythm starting at the page's corner (59 px on the
//! Manta), with a major every 5: nine lines each way. Top makes lines 0 and 5 major; Centre lines 1 and 6 (1 line
//! before, 2 after); Rhythm origin lines 3 and 8 (rows 5 and 10 from the origin).

use quadrule_layout::pattern::{Env, Pen, pattern_items_turned};
use quadrule_layout::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use std::collections::BTreeSet;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

const PEN: Pen = Pen { w: 1, gray: 128 };
const UNIT: f64 = 59.0;
const FIRST: f64 = 2.0 * UNIT;
const LINES: i64 = 9;

fn doc() -> Document {
    let mut d = slice_document();
    d.page.margins_mm = Margins::all(0.0);
    d.rhythm.resolution = RhythmResolution::PixelLocked;
    d.rhythm.h_unit_mm = 5.0;
    d.rhythm.v_unit_mm = 5.0;
    d.rhythm.major_every = Some(5);
    d
}

fn region() -> (f64, f64, f64, f64) {
    (FIRST, FIRST, FIRST + 8.0 * UNIT, FIRST + 8.0 * UNIT)
}

/// The Emphasis role's gray on the Manta: what the test's majors are drawn in.
fn major_gray() -> u8 {
    stroke(Style::Role(LineRole::Emphasis), &builtin(MANTA_ID).unwrap()).1
}

/// A region of `p` with its rows counted from `rows` and its columns from `cols`.
fn region_axes(p: Pattern, rows: MajorFrom, cols: MajorFrom) -> PatternRegion {
    PatternRegion {
        major_rows_from: rows,
        major_columns_from: cols,
        major_role: LineRole::Emphasis,
        ..PatternRegion::new(p)
    }
}

/// A region of `p` with its rows and columns both counted from `from`.
fn region_of(p: Pattern, from: MajorFrom) -> PatternRegion {
    region_axes(p, from, from)
}

/// The items of `pr` on the region, its frame turned by `deg`.
fn draw(pr: &PatternRegion, deg: f64) -> Vec<Item> {
    let doc = doc();
    let manta = builtin(MANTA_ID).unwrap();
    let env = Env {
        target: &manta,
        rhythm: &doc.rhythm,
        rhythm_px: RhythmPx::new(&doc, &manta),
        labels: None,
    };
    assert_eq!((env.rhythm_px.origin_x, env.rhythm_px.unit_x), (0.0, UNIT));
    let mut items = Vec::new();
    pattern_items_turned(pr, PEN, region(), deg, &env, &mut items);
    items
}

/// Which line from the region's first a position is.
fn index(pos: f64) -> i64 {
    ((pos - FIRST) / UNIT).round() as i64
}

/// The major rows and columns among the hard-edged rectangles (a dot counts for its row and its column).
fn majors(items: &[Item]) -> (BTreeSet<i64>, BTreeSet<i64>) {
    let g = major_gray();
    let (mut rows, mut cols) = (BTreeSet::new(), BTreeSet::new());
    for i in items {
        if let Item::Rect {
            x0,
            y0,
            x1,
            y1,
            gray,
        } = *i
            && gray == g
        {
            let (w, h) = (x1 - x0, y1 - y0);
            if w > 2.0 * h {
                rows.insert(index((y0 + y1) / 2.0));
            } else if h > 2.0 * w {
                cols.insert(index((x0 + x1) / 2.0));
            } else {
                rows.insert(index((y0 + y1) / 2.0));
                cols.insert(index((x0 + x1) / 2.0));
            }
        }
    }
    (rows, cols)
}

fn set(v: &[i64]) -> BTreeSet<i64> {
    v.iter().copied().collect()
}

/// The majors each choice should give on nine lines.
fn expected(from: MajorFrom) -> BTreeSet<i64> {
    match from {
        MajorFrom::Top => set(&[0, 5]),
        MajorFrom::Centre => set(&[1, 6]),
        MajorFrom::RhythmOrigin => set(&[3, 8]),
    }
}

const FROMS: [MajorFrom; 3] = [MajorFrom::Top, MajorFrom::Centre, MajorFrom::RhythmOrigin];

#[test]
fn each_choice_places_the_majors_on_lines_grid_dots_and_cornell() {
    let mut ok = true;
    let mut parts = Vec::new();
    let cornell = Pattern::Cornell {
        cue_units: None,
        summary_units: None,
    };
    for (name, p) in [
        ("lines", Pattern::Lines),
        ("grid", Pattern::Grid),
        ("dots", Pattern::Dots),
        ("cornell", cornell),
    ] {
        // Each of the nine ways to set the rows and the columns.
        let mut good = true;
        for row_from in FROMS {
            for col_from in FROMS {
                let (rows, cols) = majors(&draw(&region_axes(p, row_from, col_from), 0.0));
                let (want_rows, want_cols) = (expected(row_from), expected(col_from));
                good &= match name {
                    "lines" => rows == want_rows && cols.is_empty(),
                    "grid" | "dots" => rows == want_rows && cols == want_cols,
                    // Cornell's summary divider takes row 6 (two rows above the last), drawn as a divider.
                    _ => rows == &want_rows - &set(&[6]) && cols.is_empty(),
                };
            }
        }
        ok &= good;
        parts.push(format!("{name}_nine_ways={good}"));
    }
    println!(
        "\nCHECK crate=layout name=major_from_choices result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn centre_leaves_the_odd_line_after_the_last_major() {
    // Nine lines, a major every 5: 8 = 5 + 3 lines over, split 1 before and 2 after, down and across.
    let (rows, cols) = majors(&draw(&region_of(Pattern::Grid, MajorFrom::Centre), 0.0));
    let before = |s: &BTreeSet<i64>| s.first().copied().unwrap_or(-1);
    let after = |s: &BTreeSet<i64>| LINES - 1 - s.last().copied().unwrap_or(LINES);
    let ok = (before(&rows), after(&rows), before(&cols), after(&cols)) == (1, 2, 1, 2);
    println!(
        "\nCHECK crate=layout name=major_from_centre_odd_line result={} rows_before={} rows_after={} cols_before={} cols_after={}",
        pass(ok),
        before(&rows),
        after(&rows),
        before(&cols),
        after(&cols)
    );
    assert!(ok);
}

#[test]
fn a_border_does_not_move_the_majors() {
    let mut ok = true;
    let mut parts = Vec::new();
    let edges = set(&[0, LINES - 1]);
    for from in FROMS {
        let plain = region_of(Pattern::Grid, from);
        let bordered = PatternRegion {
            border: Some(Border::new()),
            ..plain.clone()
        };
        let (rows, cols) = majors(&draw(&plain, 0.0));
        let (brows, bcols) = majors(&draw(&bordered, 0.0));
        // The border hides the edge lines and nothing else moves.
        let same = brows == &rows - &edges && bcols == &cols - &edges;
        ok &= same;
        parts.push(format!("{from:?}={same}"));
    }
    // With Top, the border stands where the first major was, and the next is five lines in.
    let (rows, _) = majors(&draw(
        &PatternRegion {
            border: Some(Border::new()),
            ..region_of(Pattern::Grid, MajorFrom::Top)
        },
        0.0,
    ));
    let next_in = rows == set(&[5]);
    ok &= next_in;
    println!(
        "\nCHECK crate=layout name=major_from_border result={} {} top_next_major_5_in={next_in}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

/// The major slanted segments of an isometric drawing, as lines `y = a + s * x`.
fn major_slants(items: &[Item]) -> Vec<(f64, f64)> {
    let g = major_gray();
    let mut out = Vec::new();
    for i in items {
        if let Item::StrokePath { path, gray, .. } = i
            && *gray == g
        {
            let mut at = (0.0, 0.0);
            for el in path.elements() {
                match *el {
                    kurbo::PathEl::MoveTo(p) => at = (p.x, p.y),
                    kurbo::PathEl::LineTo(p) => {
                        let s = (p.y - at.1) / (p.x - at.0);
                        out.push((at.1 - s * at.0, s));
                        at = (p.x, p.y);
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

#[test]
fn isometric_has_no_top_and_its_centred_majors_still_meet() {
    let iso = Pattern::Isometric {
        style: IsoStyle::Lines,
    };
    let top = draw(&region_of(iso, MajorFrom::Top), 0.0);
    let origin = draw(&region_of(iso, MajorFrom::RhythmOrigin), 0.0);
    // Set to Top it draws as Rhythm origin.
    let top_is_origin = top == origin;
    // Each of the four ways to set rows and columns from Centre and Rhythm origin: the vertical majors are the
    // columns' choice (1 and 6 of nine centred, 3 and 8 from the origin), every crossing of a "/" major with a "\"
    // major is on a major column (the three families' majors meet), and the four drawings all differ.
    let two = [MajorFrom::Centre, MajorFrom::RhythmOrigin];
    let (mut cols_right, mut crossings, mut on_major) = (true, 0, true);
    let mut drawings = Vec::new();
    for row_from in two {
        for col_from in two {
            let items = draw(&region_axes(iso, row_from, col_from), 0.0);
            let (_, cols) = majors(&items);
            let want = expected(col_from);
            cols_right &= cols == want;
            let first = *want.first().unwrap();
            let slants = major_slants(&items);
            let (down, up): (Vec<_>, Vec<_>) = slants.iter().partition(|(_, s)| *s > 0.0);
            for &(a1, s1) in &down {
                for &(a2, s2) in &up {
                    let x = (a2 - a1) / (s1 - s2);
                    let k = (x - FIRST) / UNIT;
                    crossings += 1;
                    on_major &= (k - k.round()).abs() < 0.02
                        && (k.round() as i64 - first).rem_euclid(5) == 0;
                }
            }
            drawings.push(items);
        }
    }
    let meet = crossings > 0 && on_major;
    let all_differ =
        (0..drawings.len()).all(|i| (i + 1..drawings.len()).all(|j| drawings[i] != drawings[j]));
    // Isometric dots, centred: a major dot on a major column only.
    let dots = draw(
        &region_of(
            Pattern::Isometric {
                style: IsoStyle::Dots,
            },
            MajorFrom::Centre,
        ),
        0.0,
    );
    let (_, dot_cols) = majors(&dots);
    let dots_ok = !dot_cols.is_empty() && dot_cols.is_subset(&set(&[1, 6]));
    let ok = top_is_origin && cols_right && meet && all_differ && dots_ok;
    println!(
        "\nCHECK crate=layout name=major_from_isometric result={} top_draws_as_origin={top_is_origin} cols_per_setting={cols_right} major_crossings={crossings} all_on_major_columns={on_major} four_ways_differ={all_differ} dots_on_major_columns={dots_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_ruling_turned_a_quarter_turn_counts_in_its_own_axes() {
    // Turned a quarter turn clockwise inside its frame, the ruling's own top is the frame's right side: with Top its
    // rows 0 and 5 are the screen's columns 8 and 3; Centre's 1 and 6 are 7 and 2; the rhythm origin's 3 and 8 are 5
    // and 0.
    let mut ok = true;
    let mut parts = Vec::new();
    for (from, want) in [
        (MajorFrom::Top, set(&[3, 8])),
        (MajorFrom::Centre, set(&[2, 7])),
        (MajorFrom::RhythmOrigin, set(&[0, 5])),
    ] {
        let pr = PatternRegion {
            angle_deg: 90.0,
            ..region_of(Pattern::Lines, from)
        };
        let (rows, cols) = majors(&draw(&pr, 0.0));
        let good = rows.is_empty() && cols == want;
        ok &= good;
        parts.push(format!(
            "{from:?}={good}:cols={}",
            cols.iter()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join("+")
        ));
    }
    println!(
        "\nCHECK crate=layout name=major_from_turned_ruling result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
