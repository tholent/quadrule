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

//! Smooth zoom (docs/bbq/2026-11-18-fixes-before-16b.md, Rule Q1 and the Defaulted zoom line): a pinch or Ctrl + scroll
//! zooms by the gesture's own factor, anchored at the pointer, clamped to 1/8 to 16, settling on 1:1 and on Fit within
//! about 6 %; a wheel notch is a fixed fraction; the + and - buttons keep the clean steps. Headless. Each test prints one
//! structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::{Event, Modifiers, MouseWheelUnit, TouchPhase};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quadrule_ui::viewer::{ZOOM_SETTLE, ZOOM_WHEEL_NOTCH, settle_zoom};
use quadrule_ui::{Viewer, ViewerUi, Zoom};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

fn sample() -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    harness(v)
}

fn harness(v: Viewer) -> Harness<'static, Window> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 900.0))
        .with_max_steps(40)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window {
                ui: ViewerUi::new(v),
            },
        );
    h.run();
    h
}

fn zoom(h: &Harness<'static, Window>) -> Zoom {
    h.state().ui.viewer.zoom
}

/// The scale on screen now, screen points per device pixel, from two device pixels of the page.
fn shown(h: &Harness<'static, Window>) -> f32 {
    let u = &h.state().ui;
    let a = u.screen_point(0.0, 0.0).unwrap();
    let b = u.screen_point(1000.0, 0.0).unwrap();
    (b.x - a.x) / 1000.0
}

#[test]
fn a_pinch_zooms_by_its_own_factor_clamped_and_settling_on_one_to_one_and_fit() {
    let fit = 0.4271_f32;
    let mut v = Viewer::default();
    // The factor is the gesture's own: 0.5 * 1.1 * 1.1.
    v.zoom = Zoom::Scale(0.5);
    v.zoom_by(1.1, 0.5, fit, 0.0);
    let Zoom::Scale(a) = v.zoom else {
        panic!("not a scale")
    };
    v.zoom_by(1.1, a, fit, 0.05);
    let Zoom::Scale(b) = v.zoom else {
        panic!("not a scale")
    };
    let factor = (a - 0.55).abs() < 1e-5 && (b - 0.605).abs() < 1e-5;

    // The ends: 1/8 and 16, whatever the factor.
    v.zoom_by(1e6, b, fit, 1.0);
    let top = v.zoom == Zoom::Scale(16.0);
    v.zoom_by(1e-9, 16.0, fit, 2.0);
    let bottom = v.zoom == Zoom::Scale(0.125);

    let (r_up, r_down) = (1.0_f32 + 0.07, 1.0_f32 - 0.07);
    // Settling, by `settle_zoom` itself: 6 % of 1:1 and of Fit, and not beyond.
    let near_one = settle_zoom(1.0 - 0.059, fit) == Zoom::Scale(1.0)
        && settle_zoom(1.0 + 0.059, fit) == Zoom::Scale(1.0)
        && settle_zoom(r_up, fit) == Zoom::Scale(r_up)
        && settle_zoom(r_down, fit) == Zoom::Scale(r_down);
    let near_fit = settle_zoom(fit * 1.05, fit) == Zoom::Fit
        && settle_zoom(fit * 0.95, fit) == Zoom::Fit
        && settle_zoom(fit * 1.08, fit) == Zoom::Scale(fit * 1.08_f32)
        && ZOOM_SETTLE == 0.06;

    // A pinch crossing 1:1 holds there and then gets through: the gesture keeps its own scale, so the
    // settling is not a trap.
    let mut v = Viewer::default();
    v.zoom = Zoom::Scale(0.8);
    let mut now = 0.8_f32;
    let mut t = 0.0;
    let (mut held, mut crossed, mut jumped) = (0, false, false);
    let mut last = 0.8_f32;
    for _ in 0..40 {
        v.zoom_by(1.02, now, fit, t);
        t += 0.016;
        now = match v.zoom {
            Zoom::Scale(s) => s,
            _ => fit,
        };
        if now == 1.0 {
            held += 1;
        }
        if now > 1.06 {
            crossed = true;
        }
        if now / last > 1.1 {
            jumped = true;
        }
        last = now;
    }
    let through = held >= 2 && crossed && !jumped && now > 1.06;

    // A gesture that rests 0.3 s is over: the next one starts from what shows, so from exactly 1:1 a
    // little pinch (under 6 %) stays at 1:1.
    let mut v = Viewer::default();
    v.zoom = Zoom::Scale(1.0);
    let carried = v.zoom_by(1.02, 1.0, fit, 5.0);
    let carried_again = v.zoom_by(1.02, 1.0, fit, 5.05);
    let still_one = v.zoom == Zoom::Scale(1.0);
    let fresh = !v.zoom_by(1.02, 1.0, fit, 6.0);
    let gesture = !carried && carried_again && still_one && fresh;

    let ok = factor && top && bottom && near_one && near_fit && through && gesture;
    println!(
        "\nCHECK crate=ui name=zoom_gesture result={} factor={factor} clamp_top={top} clamp_bottom={bottom} settle_one={near_one} settle_fit={near_fit} crosses_one={through} held_frames={held} gesture_ends={gesture}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_buttons_keep_the_clean_steps_from_an_in_between_zoom() {
    let mut h = sample();
    h.get_by_label("1:1").click();
    h.run();
    // Somewhere between steps, as a pinch leaves it.
    h.state_mut().ui.viewer.zoom = Zoom::Scale(1.37);
    h.run();
    h.get_by_label("+").click();
    h.run();
    let up = zoom(&h) == Zoom::Scale(2.0);
    h.state_mut().ui.viewer.zoom = Zoom::Scale(1.37);
    h.run();
    h.get_by_label("−").click();
    h.run();
    let down = zoom(&h) == Zoom::Scale(1.0);
    // And a gesture in progress is dropped by a button: the next pinch starts from the step.
    let dropped = h.state().ui.viewer.zoom_gesture.is_none();
    let ok = up && down && dropped;
    println!(
        "\nCHECK crate=ui name=zoom_buttons_step result={} plus_from_1_37={up} minus_from_1_37={down} gesture_dropped={dropped}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_pinch_in_the_window_zooms_by_its_factor_and_keeps_the_pixel_under_the_pointer() {
    let mut h = sample();
    h.get_by_label("1:1").click();
    h.run();
    // A device pixel well inside the page, on screen at 1:1.
    let (px, py) = (500.0, 400.0);
    let p = h.state().ui.screen_point(px, py).unwrap();
    let before = shown(&h);
    h.hover_at(p);
    h.run();
    h.event(Event::Zoom(1.37));
    h.run();
    let z = zoom(&h);
    let by_factor = matches!(z, Zoom::Scale(s) if (s - 1.37).abs() < 1e-4);
    let after = shown(&h);
    let q = h.state().ui.screen_point(px, py).unwrap();
    let anchored = (q - p).length() < 2.0;
    // Not a whole step: well off both 1:1 and 2.
    let in_between = (after / before - 1.37).abs() < 0.01;

    // The same pixel again after a second pinch out, at a different scale and with the pointer moved.
    let p2 = h.state().ui.screen_point(px + 60.0, py + 40.0).unwrap();
    h.hover_at(p2);
    h.run();
    h.event(Event::Zoom(0.8));
    h.run();
    let q2 = h.state().ui.screen_point(px + 60.0, py + 40.0).unwrap();
    let anchored2 = (q2 - p2).length() < 2.0;
    let second = matches!(zoom(&h), Zoom::Scale(s) if (s - 1.096).abs() < 1e-3);

    let ok = by_factor && anchored && in_between && anchored2 && second;
    println!(
        "\nCHECK crate=ui name=zoom_window_pinch result={} factor_1_37={by_factor} on_screen={in_between} anchor_off_px={:.2} second_pinch={second} anchor2_off_px={:.2}",
        pass(ok),
        (q - p).length(),
        (q2 - p2).length()
    );
    assert!(ok);
}

#[test]
fn a_wheel_notch_with_ctrl_is_a_fixed_fraction_not_a_step() {
    let mut h = sample();
    h.get_by_label("1:1").click();
    h.run();
    h.state_mut().ui.viewer.zoom = Zoom::Scale(0.5);
    h.run();
    let p = h.state().ui.screen_point(500.0, 400.0).unwrap();
    h.hover_at(p);
    h.run();
    let ctrl = Modifiers::CTRL;
    let notch = || Event::MouseWheel {
        unit: MouseWheelUnit::Line,
        delta: egui::vec2(0.0, 1.0),
        phase: TouchPhase::Move,
        modifiers: ctrl,
        source: egui::MouseWheelSource::Unknown,
    };
    h.event_modifiers(notch(), ctrl);
    h.run();
    let one = match zoom(&h) {
        Zoom::Scale(s) => s,
        _ => 0.0,
    };
    let expect = 0.5 * ZOOM_WHEEL_NOTCH;
    let fixed = (one - expect).abs() < 0.005 && one != 1.0;
    // The same notch the other way brings it back.
    let out = Event::MouseWheel {
        unit: MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -1.0),
        phase: TouchPhase::Move,
        modifiers: ctrl,
        source: egui::MouseWheelSource::Unknown,
    };
    h.event_modifiers(out, ctrl);
    h.run();
    let back = match zoom(&h) {
        Zoom::Scale(s) => s,
        _ => 0.0,
    };
    let returns = (back - 0.5).abs() < 0.005;
    let ok = fixed && returns;
    println!(
        "\nCHECK crate=ui name=zoom_wheel_notch result={} notch={ZOOM_WHEEL_NOTCH} from=0.5 after_one_notch={one:.4} after_back={back:.4}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_pinch_settles_on_fit_and_on_one_to_one_in_the_window() {
    let mut h = sample();
    // From Fit, a small pinch (3 %) stays at Fit.
    h.get_by_label("Fit").click();
    h.run();
    let fit_scale = shown(&h);
    let p = h.state().ui.screen_point(500.0, 400.0).unwrap();
    h.hover_at(p);
    h.run();
    h.event(Event::Zoom(1.03));
    h.run();
    let stays_fit = zoom(&h) == Zoom::Fit;
    // A pinch from just under 1:1 (by 3 %) lands on 1:1 exactly.
    h.state_mut().ui.viewer.zoom = Zoom::Scale(0.97);
    h.run();
    let p = h.state().ui.screen_point(500.0, 400.0).unwrap();
    h.hover_at(p);
    h.run();
    h.event(Event::Zoom(1.02));
    h.run();
    let lands = zoom(&h) == Zoom::Scale(1.0);
    // And a pinch from Fit by 2x lands on a non-step scale, twice what Fit showed.
    h.get_by_label("Fit").click();
    h.run();
    let p = h.state().ui.screen_point(500.0, 400.0).unwrap();
    h.hover_at(p);
    h.run();
    h.event(Event::Zoom(2.0));
    h.run();
    let twice = matches!(zoom(&h), Zoom::Scale(s) if (s - 2.0 * fit_scale).abs() < 1e-3);
    let ok = stays_fit && lands && twice;
    println!(
        "\nCHECK crate=ui name=zoom_window_settle result={} fit_scale={fit_scale:.4} small_pinch_stays_fit={stays_fit} lands_on_one={lands} twice_fit={twice}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_zoom_benchmark_runs_both_pages_and_leaves_the_page_as_it_was() {
    // `quadrule --bench-zoom` and `--bench-zoom --groups-page`: headless they finish, stay inside 1/8 to 16 and end at Fit.
    let mut parts = Vec::new();
    let mut ok = true;
    let runs: [(&str, quadrule_ui::DragBench); 2] = [
        ("sample", quadrule_ui::DragBench::zoom(24)),
        ("groups", quadrule_ui::DragBench::zoom(24).with_groups()),
    ];
    for (name, bench) in runs {
        let mut h = harness(Viewer::default());
        h.state_mut().ui.bench = Some(bench);
        let (mut steps, mut low, mut high) = (0, f32::MAX, 0.0_f32);
        while !h.state().ui.bench_finished() && steps < 120 {
            h.step();
            steps += 1;
            if let Zoom::Scale(s) = h.state().ui.viewer.zoom {
                low = low.min(s);
                high = high.max(s);
            }
        }
        let ui = &h.state().ui;
        let done = ui.bench_finished();
        let clean = ui.viewer.zoom == Zoom::Fit && !ui.viewer.is_dirty();
        let swung = low < 0.5 && high > 4.0 && low >= 0.125 && high <= 16.0;
        ok &= done && clean && swung;
        parts.push(format!(
            "{name}:finished={done}:ends_at_fit={clean}:low={low:.2}:high={high:.2}"
        ));
    }
    println!(
        "\nCHECK crate=ui name=zoom_benchmarks result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_zoomed_out_pinch_draws_the_page_at_screen_size_without_the_export_pixels() {
    // A pinch from 0.9 down to 0.6 (zoomed out) sets the zoom and draws, and the page's full-size picture is never
    // made for it: the screen shows the page drawn at its own pixels (rung 17, Q7).
    let mut h = sample();
    h.state_mut().ui.viewer.zoom = Zoom::Scale(0.9);
    h.run();
    let p = h.state().ui.screen_point(500.0, 400.0).unwrap();
    h.hover_at(p);
    h.run();
    h.event(Event::Zoom(2.0 / 3.0));
    h.run();
    let q = h.state().ui.screen_point(500.0, 400.0).unwrap();
    let zoomed = matches!(zoom(&h), Zoom::Scale(s) if (s - 0.6).abs() < 1e-3);
    // (The page's top left cannot scroll further, so zooming out near it cannot hold the anchor; only the zoom is checked.)
    let drawn = h.state().ui.screen_point(0.0, 0.0).is_some() && q.x.is_finite();
    let no_full = !h
        .state_mut()
        .ui
        .viewer
        .page()
        .is_some_and(|p| p.image_drawn());
    let ok = zoomed && drawn && no_full;
    println!(
        "\nCHECK crate=ui name=zoom_copies result={} out_to_0_6={zoomed} drawn={drawn} full_size_picture_not_made={no_full}",
        pass(ok)
    );
    assert!(ok);
}

/// One Ctrl + wheel notch (a line), `dir` up (in) or down (out).
fn ctrl_notch(h: &Harness<'static, Window>, dir: f32) {
    let ctrl = Modifiers::CTRL;
    h.event_modifiers(
        Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: egui::vec2(0.0, dir),
            phase: TouchPhase::Move,
            modifiers: ctrl,
            source: egui::MouseWheelSource::Unknown,
        },
        ctrl,
    );
}

/// Drives `notches` Ctrl + scroll notches, one per frame, `dir` in or out, with the pointer on device
/// pixel (`px`, `py`), and returns how far the pixel is from the pointer on screen after each frame, as
/// drawn (rung 17, D9: a frame that showed the new scale before the scroll caught up was the jump).
fn drift_while_scrolling(
    h: &mut Harness<'static, Window>,
    (px, py): (f64, f64),
    dir: f32,
    notches: usize,
) -> Vec<f32> {
    let p = h.state().ui.screen_point(px, py).unwrap();
    h.hover_at(p);
    h.run();
    let mut drift = Vec::new();
    for _ in 0..notches {
        ctrl_notch(h, dir);
        h.step();
        let q = h.state().ui.screen_point(px, py).unwrap();
        drift.push((q - p).length());
    }
    // And the frame after the last notch, with no event.
    h.step();
    let q = h.state().ui.screen_point(px, py).unwrap();
    drift.push((q - p).length());
    drift
}

#[test]
fn scroll_zooming_keeps_the_pixel_under_the_pointer_on_every_frame_drawn() {
    // Where the page is smaller than the window it stays centred, so a pixel can only hold where the page
    // can scroll: the cases take one in the page's middle from Fit, and zoom out only as far as in.
    let mut parts = Vec::new();
    let mut ok = true;
    for (name, start, px, py) in [
        ("in_from_1_to_1", Zoom::Scale(1.0), 500.0, 400.0),
        ("in_from_fit", Zoom::Fit, 0.0, 700.0),
        ("in_then_out", Zoom::Scale(1.0), 400.0, 300.0),
    ] {
        let mut h = sample();
        h.state_mut().ui.viewer.zoom = start;
        h.run();
        let w = f64::from(h.state_mut().ui.viewer.page().unwrap().profile.width_px);
        let px = if px == 0.0 { w / 2.0 } else { px };
        let before = shown(&h);
        let mut drift = drift_while_scrolling(&mut h, (px, py), 1.0, 6);
        let zoomed = (shown(&h) / before - 1.0).abs() > 0.2;
        if name == "in_then_out" {
            // The same pixel, the same pointer, out again (one gesture: no pause between).
            for _ in 0..4 {
                ctrl_notch(&h, -1.0);
                h.step();
                let q = h.state().ui.screen_point(px, py).unwrap();
                let p = h.state().ui.zoom_anchor_point().unwrap();
                drift.push((q - p).length());
            }
        }
        let worst = drift.iter().copied().fold(0.0_f32, f32::max);
        let good = worst < 1.5 && zoomed;
        ok &= good;
        parts.push(format!("{name}:worst_px={worst:.2}:zoomed={zoomed}"));
    }
    println!(
        "\nCHECK crate=ui name=zoom_scroll_anchor_every_frame result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
