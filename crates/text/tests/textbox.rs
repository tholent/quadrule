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

//! The text box's own layout (rung 11b, docs/bbq/2026-11-14-rung-11-text-and-fonts.md): a Fixed box
//! leaves out whole lines that do not fit. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_model::{TextBox, Uuid, pt_to_mm};
use quadrule_text::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn five_lines() -> TextBox {
    TextBox {
        content: "One\nTwo\nThree\nFour\nFive".into(),
        ..TextBox::default()
    }
}

fn lay(tb: &TextBox, height_mm: Option<f64>) -> Laid {
    let mut reg = FontRegistry::bundled_only();
    let req = TextRequest {
        height_mm,
        ..TextRequest::new(Uuid::from_u128(1), tb, 60.0)
    };
    lay_out(&mut reg, "test", &[req]).expect("lay out")
}

#[test]
fn a_fixed_box_leaves_out_whole_lines_that_do_not_fit() {
    let tb = five_lines();
    let line = pt_to_mm(12.0) * 1.25;
    let auto = lay(&tb, None);
    let a = &auto.outlines.texts[0];
    // Room for two lines and most of a third: the third is left out whole.
    let fixed = lay(&tb, Some(line * 2.8));
    let f = &fixed.outlines.texts[0];
    let lowest = f.glyphs.iter().map(|g| g.y_mm()).fold(f32::MIN, f32::max);
    let cut = f.lines_hidden == 3
        && f.glyphs.len() < a.glyphs.len()
        && f64::from(lowest) <= line * 2.0
        && a.lines_hidden == 0;
    // A box with room for all five leaves nothing out, and draws what the auto box draws.
    let roomy = lay(&tb, Some(line * 5.0 + 0.5));
    let all =
        roomy.outlines.texts[0].lines_hidden == 0 && roomy.outlines.texts[0].glyphs == a.glyphs;
    // A box too short for one line leaves every line out.
    let none = lay(&tb, Some(line * 0.5)).outlines.texts[0].lines_hidden == 5;
    let ok = cut && all && none;
    println!(
        "\nCHECK crate=text name=fixed_box_cuts_whole_lines result={} hidden={} lowest_baseline_mm={lowest:.3} room_for_all_hidden={} too_short_hidden=5:{none}",
        pass(ok),
        f.lines_hidden,
        roomy.outlines.texts[0].lines_hidden
    );
    assert!(ok);
}

#[test]
fn the_text_height_is_the_text_not_the_box() {
    let tb = five_lines();
    let line = (pt_to_mm(12.0) * 1.25) as f32;
    let fixed = lay(&tb, Some(10.0));
    let h = fixed.outlines.texts[0].height_mm;
    let ok = (h - line * 5.0).abs() < 1e-3;
    println!(
        "\nCHECK crate=text name=height_is_the_texts result={} height_mm={h:.3} five_lines_mm={:.3}",
        pass(ok),
        line * 5.0
    );
    assert!(ok);
}

fn baselines(l: &Laid) -> Vec<f32> {
    let mut ys: Vec<f32> = l.outlines.texts[0]
        .glyphs
        .iter()
        .map(|g| g.y_mm())
        .collect();
    ys.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    ys
}

fn lay_with(tb: &TextBox, f: impl Fn(&mut TextRequest)) -> Laid {
    let mut reg = FontRegistry::bundled_only();
    let mut req = TextRequest::new(Uuid::from_u128(1), tb, 60.0);
    f(&mut req);
    lay_out(&mut reg, "test", &[req]).expect("lay out")
}

#[test]
fn paragraph_spacing_goes_after_each_paragraph_but_the_last() {
    let tb = TextBox {
        content: "One\nTwo\nThree".into(),
        ..TextBox::default()
    };
    let line = (pt_to_mm(12.0) * 1.25) as f32;
    let plain = baselines(&lay(&tb, None));
    let spaced = lay_with(&tb, |r| r.paragraph_mm = 4.0);
    let b = baselines(&spaced);
    let gaps_ok = plain.len() == 3
        && b.len() == 3
        && ((b[1] - b[0]) - (line + 4.0)).abs() < 1e-3
        && ((b[2] - b[1]) - (line + 4.0)).abs() < 1e-3;
    // The height is the lines and two paragraph spaces: none after the last.
    let h = spaced.outlines.texts[0].height_mm;
    let height_ok = (h - (line * 3.0 + 8.0)).abs() < 1e-3;
    // Lines within one paragraph (a wrap) get no space.
    let long = TextBox {
        content: "word word word word word word word word word word word word".into(),
        ..TextBox::default()
    };
    let wrapped = lay_with(&long, |r| {
        r.paragraph_mm = 4.0;
        r.width_mm = 30.0
    });
    let w = baselines(&wrapped);
    let wrap_ok = w.len() > 1 && w.windows(2).all(|p| ((p[1] - p[0]) - line).abs() < 1e-3);
    let ok = gaps_ok && height_ok && wrap_ok;
    println!(
        "\nCHECK crate=text name=paragraph_spacing result={} gaps_ok={gaps_ok} height_mm={h:.3} height_ok={height_ok} wrap_adds_none={wrap_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_fixed_box_puts_its_text_top_middle_or_bottom_and_an_auto_box_ignores_it() {
    use quadrule_model::VerticalAlign::*;
    let line = (pt_to_mm(12.0) * 1.25) as f32;
    let at = |v, height: Option<f64>| {
        let tb = TextBox {
            content: "One\nTwo".into(),
            v_align: v,
            ..TextBox::default()
        };
        baselines(&lay_with(&tb, |r| r.height_mm = height))
    };
    let top = at(Top, Some(30.0));
    let mid = at(Middle, Some(30.0));
    let bot = at(Bottom, Some(30.0));
    let free = 30.0 - line * 2.0;
    let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
    let ok_pos = near(mid[0] - top[0], free / 2.0) && near(bot[0] - top[0], free);
    let auto = at(Bottom, None);
    let ok_auto = near(auto[0], top[0]);
    // Text taller than the box stays at the top, whatever the alignment.
    let tall_top = at(Top, Some(line as f64 * 1.5));
    let tall_bot = at(Bottom, Some(line as f64 * 1.5));
    let ok_tall = near(tall_top[0], tall_bot[0]);
    let ok = ok_pos && ok_auto && ok_tall;
    println!(
        "\nCHECK crate=text name=vertical_alignment result={} middle_moves_mm={:.3} bottom_moves_mm={:.3} auto_ignores={ok_auto} overflow_stays_top={ok_tall}",
        pass(ok),
        mid[0] - top[0],
        bot[0] - top[0]
    );
    assert!(ok);
}

fn grid(top_px: f64) -> BaselineGrid {
    // The Manta: 300 ppi, a 6 mm rhythm of 71 px starting 94 px down.
    BaselineGrid {
        top_px,
        px_per_mm: 300.0 / 25.4,
        origin_px: 94.0,
        unit_px: 71.0,
        rule_px: 3,
        gap_px: 0.0,
    }
}

#[test]
fn baselines_stand_on_rhythm_lines_with_a_whole_step_line_height() {
    let tb = TextBox {
        content: "One\nTwo and some more words to wrap over a line\n\nThree".into(),
        baselines: true,
        ..TextBox::default()
    };
    let g = grid(100.0);
    let laid = lay_with(&tb, |r| {
        r.grid = Some(g);
        r.paragraph_mm = 2.0;
        r.width_mm = 40.0
    });
    let ys = baselines(&laid);
    let rows: Vec<f64> = ys
        .iter()
        .map(|y| (g.top_px + f64::from(*y) * g.px_per_mm).round())
        .collect();
    // A baseline stands on the top edge of the rule on its rung: the rung's row less half the rule.
    let on_lines = rows.iter().all(|r| {
        let rung = r + f64::from(g.rule_px / 2);
        ((rung - g.origin_px) / g.unit_px - ((rung - g.origin_px) / g.unit_px).round()).abs() < 1e-9
    });
    // Exactly on, not just near: the glyph's own coordinate is the line's, to a thousandth of a pixel.
    let exact = ys.iter().all(|y| {
        let px = g.top_px + f64::from(*y) * g.px_per_mm;
        (px - px.round()).abs() < 1e-3
    });
    let cap = pt_to_mm(12.0) * 0.6;
    let first_ok = f64::from(ys[0]) >= cap && f64::from(ys[0]) < cap + g.step_mm() + 1.0;
    let steps_apart = rows
        .windows(2)
        .all(|w| ((w[1] - w[0]) / g.unit_px).fract() == 0.0);
    // 12 pt at 1.25 is 5.3 mm; the step is 6 mm: one step a line. The paragraph's 2 mm rounds up to one
    // more step.
    let diffs: Vec<f64> = rows.windows(2).map(|w| (w[1] - w[0]) / g.unit_px).collect();
    let ok = on_lines
        && exact
        && first_ok
        && steps_apart
        && diffs.contains(&1.0)
        && diffs.contains(&2.0);
    println!(
        "\nCHECK crate=text name=baselines_on_the_rhythm result={} lines={} on_lines={on_lines} exact={exact} first_below_cap={first_ok} steps={diffs:?}",
        pass(ok),
        rows.len()
    );
    assert!(ok);
}

#[test]
fn middle_and_bottom_move_whole_steps_on_the_rhythm() {
    use quadrule_model::VerticalAlign::*;
    let g = grid(100.0);
    let at = |v, h: f64| {
        let tb = TextBox {
            content: "One\nTwo".into(),
            baselines: true,
            v_align: v,
            ..TextBox::default()
        };
        let l = lay_with(&tb, |r| {
            r.grid = Some(g);
            r.height_mm = Some(h)
        });
        baselines(&l)
    };
    let top = at(Top, 60.0);
    let mid = at(Middle, 60.0);
    let bot = at(Bottom, 60.0);
    let whole = |a: f32, b: f32| {
        let steps = f64::from(a - b) / g.step_mm();
        (steps - steps.round()).abs() < 1e-3
    };
    let ok = whole(mid[0], top[0])
        && whole(bot[0], top[0])
        && bot[0] > mid[0]
        && mid[0] > top[0]
        && f64::from(bot[1]) <= 60.0;
    println!(
        "\nCHECK crate=text name=baselines_vertical_alignment_in_steps result={} middle_steps={:.2} bottom_steps={:.2}",
        pass(ok),
        f64::from(mid[0] - top[0]) / g.step_mm(),
        f64::from(bot[0] - top[0]) / g.step_mm()
    );
    assert!(ok);
}
