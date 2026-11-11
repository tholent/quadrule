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

//! The rung 6b transforms page (docs/bbq/2026-11-10-rung-6-shapes.md, 6b: rotation, lines that point
//! any way, opacity): shapes rotated by several angles (the quarter turns among them), lines in
//! every direction, and opacity over fills and lines, exported on both devices and compared with
//! golden images in `tests/golden/`, whose sums sit after rung 1's two and 6a's two in
//! `SHA256SUMS`. Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test transforms`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::sha256_hex;
use quadrule_layout::{Item, display_list};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_render::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
}

fn out_dir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("transforms");
    std::fs::create_dir_all(&d).expect("create output folder");
    d
}

fn empty_outlines(id: &str) -> quadrule_model::outlines::ProfileOutlines {
    quadrule_model::outlines::ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn list(id: &str, doc: &Document) -> quadrule_layout::DisplayList {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    display_list(doc, &primary, &target, &empty_outlines(&target.id)).unwrap()
}

#[test]
fn transforms_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = transforms_document();
    let mut all_ok = true;
    let mut sums = String::new();
    for id in doc.profile_ids() {
        let target = builtin(id).unwrap();
        let dl = list(id, &doc);
        let png = encode_png(&render(&dl, Math::Fastest), None);
        // ADR-003 item 2's lever, reported and not enforced here (as for the shapes page).
        let (a, b) = (
            render(&dl, Math::Fastest).data,
            render(&dl, Math::Portable).data,
        );
        let d_px = a.iter().zip(&b).filter(|(x, y)| x != y).count();
        let d_max = a
            .iter()
            .zip(&b)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        let name = format!(
            "transforms_{id}_{}x{}.png",
            target.width_px, target.height_px
        );
        std::fs::write(out_dir().join(&name), &png).unwrap();
        let sha = sha256_hex(&png);
        sums.push_str(&format!("{sha}  {name}\n"));
        let golden_path = golden_dir().join(&name);
        if bless {
            std::fs::write(&golden_path, &png).unwrap();
        }
        let golden = std::fs::read(&golden_path).unwrap_or_default();
        let matches = golden == png;
        all_ok &= matches;
        println!(
            "\nCHECK crate=render name=transforms_golden profile={id} result={} sha256={sha} golden_sha256={} portable_diff_px={d_px} portable_max_diff={d_max} items={} os={} arch={}",
            pass(matches),
            if golden.is_empty() {
                "none".to_string()
            } else {
                sha256_hex(&golden)
            },
            dl.items.len(),
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }
    if bless {
        // Keep every line that is not this test's, in its place; add this test's after them.
        let path = golden_dir().join("SHA256SUMS");
        let old = std::fs::read_to_string(&path).unwrap_or_default();
        let mut merged: String = old
            .lines()
            .filter(|l| {
                !l.split_whitespace()
                    .nth(1)
                    .is_some_and(|n| n.starts_with("transforms_"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    // The sums file holds them too, after rung 1's two and 6a's two, which are unchanged.
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    let first_four = lines.len() >= 4
        && lines[..2].iter().all(|l| l.contains("slice_"))
        && lines[2..4].iter().all(|l| l.contains("shapes_"));
    println!(
        "\nCHECK crate=render name=transforms_sums_listed result={} listed={listed} rung1_and_6a_lines_first={first_four}",
        pass(listed && first_four)
    );
    assert!(all_ok && listed && first_four, "see the CHECK lines");
}

/// A small one-object page on the Manta and the Nomad for pixel tests: a white page, no margins to
/// get in the way, the object's frame given in mm on the Manta.
fn tiny(o: Object) -> Document {
    let mut doc = transforms_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects: vec![o],
        extra: Extra::new(),
    }];
    doc
}

fn base_object(kind: ObjectKind, frame: Frame) -> Object {
    let mut o = transforms_document().layers[1].objects[0].clone();
    o.id = Uuid::from_u128(77);
    o.frame = frame;
    // The same physical place on both devices.
    o.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    o.fill = None;
    o.style = Style::None;
    o.rotation_deg = 0.0;
    o.opacity_pct = 100;
    o.dash = DashStyle::Solid;
    o.cap = LineCap::Butt;
    o.kind = kind;
    o
}

fn rect_kind() -> ObjectKind {
    ObjectKind::Rect(RectShape::default())
}

#[test]
fn opacity_flattens_over_what_lies_beneath() {
    // A gray-100 bar with a gray-200 rectangle across it at 50 percent: over the bar the pixel is
    // the mix of 200 and 100, over the white page the mix of 200 and 255. No alpha in the PNG,
    // and group semantics: a stroke over its own fill is not mixed twice.
    let mut parts = Vec::new();
    let mut ok = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let target = builtin(id).unwrap();
        let k = target.mm_to_px(1.0);
        let expect = |below: f64, over: f64, pct: f64| {
            let a = f64::from(quadrule_layout::alpha_of_percent(pct as u8)) / 255.0;
            below * (1.0 - a) + over * a
        };
        let mut bar = base_object(rect_kind(), Frame::new(10.0, 10.0, 40.0, 10.0));
        bar.fill = Some(Fill::Gray(100));
        let mut top = base_object(rect_kind(), Frame::new(30.0, 5.0, 30.0, 20.0));
        top.fill = Some(Fill::Gray(200));
        top.opacity_pct = 50;
        let mut doc = tiny(bar);
        doc.layers[0].objects.push(top);
        let img = render(&list(id, &doc), Math::Fastest);
        let at = |x_mm: f64, y_mm: f64| f64::from(img.at((x_mm * k) as u32, (y_mm * k) as u32));
        let over_bar = at(40.0, 15.0);
        let over_page = at(55.0, 15.0);
        let bar_alone = at(15.0, 15.0);
        let page = at(5.0, 15.0);
        let near = |got: f64, want: f64| (got - want).abs() <= 1.5;
        let good = near(over_bar, expect(100.0, 200.0, 50.0))
            && near(over_page, expect(255.0, 200.0, 50.0))
            && near(bar_alone, 100.0)
            && near(page, 255.0);
        // 100 percent is no group at all; 0 percent draws nothing.
        let mut opaque = doc.clone();
        opaque.layers[0].objects[1].opacity_pct = 100;
        let no_group = !list(id, &opaque)
            .items
            .iter()
            .any(|i| matches!(i, Item::BeginAlpha { .. } | Item::EndAlpha));
        let mut clear = doc.clone();
        clear.layers[0].objects[1].opacity_pct = 0;
        let mut gone = doc.clone();
        gone.layers[0].objects.truncate(1);
        let nothing =
            render(&list(id, &clear), Math::Fastest) == render(&list(id, &gone), Math::Fastest);
        // A stroke over its own fill, as one group: the inner half of the stroke (over the fill)
        // reads as the stroke's gray mixed with the page, not with the fill under it.
        let mut shape = base_object(rect_kind(), Frame::new(10.0, 40.0, 30.0, 20.0));
        shape.fill = Some(Fill::Gray(200));
        shape.style = Style::Custom {
            width_mm: 6.0 / k,
            gray: 0,
        };
        shape.opacity_pct = 50;
        let sdoc = tiny(shape);
        let simg = render(&list(id, &sdoc), Math::Fastest);
        let inner = f64::from(simg.at((10.0 * k) as u32 + 1, (50.0 * k) as u32));
        let outer = f64::from(simg.at((10.0 * k) as u32 - 2, (50.0 * k) as u32));
        let group = near(inner, expect(255.0, 0.0, 50.0)) && near(outer, expect(255.0, 0.0, 50.0));
        ok &= good && no_group && nothing && group;
        parts.push(format!(
            "{id}:over_bar={over_bar}:over_page={over_page}:no_group_at_100={no_group}:zero_draws_nothing={nothing}:stroke_over_fill_once={group}"
        ));
    }
    println!(
        "\nCHECK crate=render name=opacity_flattens result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn quarter_turns_keep_straight_edges() {
    // Q2 and the 6b record: a rectangle turned by 90, 180 or 270 degrees is hard-edged whole-pixel
    // rectangles again; 90 swaps its sides about its centre. Any other angle is anti-aliased.
    let mut parts = Vec::new();
    let mut ok = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let mut hard = true;
        let mut swapped = true;
        for deg in [90.0, 180.0, 270.0] {
            let mut o = base_object(rect_kind(), Frame::new(20.0, 20.0, 40.0, 16.0));
            o.fill = Some(Fill::Gray(120));
            o.style = Style::Role(LineRole::Structure);
            o.rotation_deg = deg;
            let dl = list(id, &tiny(o.clone()));
            hard &= !dl.items.is_empty() && dl.items.iter().all(|i| matches!(i, Item::Rect { .. }));
            // The same rectangle, unturned, in the swapped box about the same centre.
            let mut flat = o.clone();
            flat.rotation_deg = 0.0;
            if deg != 180.0 {
                flat.frame = Frame::new(20.0 + 20.0 - 8.0, 20.0 + 8.0 - 20.0, 16.0, 40.0);
            }
            swapped &= render(&dl, Math::Fastest) == render(&list(id, &tiny(flat)), Math::Fastest);
        }
        let mut o = base_object(rect_kind(), Frame::new(20.0, 20.0, 40.0, 16.0));
        o.fill = Some(Fill::Gray(120));
        o.style = Style::Role(LineRole::Structure);
        o.rotation_deg = 30.0;
        let dl = list(id, &tiny(o));
        let img = render(&dl, Math::Fastest);
        let grays: BTreeSet<u8> = img.data.iter().copied().collect();
        let soft = dl.items.iter().all(|i| !matches!(i, Item::Rect { .. })) && grays.len() > 3;
        ok &= hard && swapped && soft;
        parts.push(format!(
            "{id}:quarter_turns_hard={hard}:ninety_equals_swapped_box={swapped}:thirty_degrees_antialiased={soft}"
        ));
    }
    println!(
        "\nCHECK crate=render name=quarter_turns_hard_edged result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_turned_shape_turns_about_the_frames_centre() {
    // The painted pixels of a turned ellipse are balanced about the frame's centre: the same ink
    // either side of it, to within a pixel or two of rounding, whatever the angle.
    let mut ok = true;
    let mut parts = Vec::new();
    for id in [MANTA_ID, NOMAD_ID] {
        let target = builtin(id).unwrap();
        let k = target.mm_to_px(1.0);
        let (cx, cy) = ((50.0 + 15.0) * k, (50.0 + 6.0) * k);
        for deg in [0.0, 25.0, 90.0, 133.0] {
            let mut o = base_object(
                ObjectKind::Ellipse(EllipseShape::default()),
                Frame::new(50.0, 50.0, 30.0, 12.0),
            );
            o.fill = Some(Fill::Gray(0));
            o.rotation_deg = deg;
            let img = render(&list(id, &tiny(o)), Math::Fastest);
            let (mut sx, mut sy, mut ink) = (0.0, 0.0, 0.0);
            for y in 0..img.height {
                for x in 0..img.width {
                    let w = f64::from(255 - img.at(x, y));
                    sx += w * (f64::from(x) + 0.5);
                    sy += w * (f64::from(y) + 0.5);
                    ink += w;
                }
            }
            let (mx, my) = (sx / ink, sy / ink);
            let centred = (mx - cx).abs() < 1.5 && (my - cy).abs() < 1.5;
            ok &= centred && ink > 0.0;
            parts.push(format!("{id}:{deg}:centred={centred}"));
        }
    }
    println!(
        "\nCHECK crate=render name=turns_about_centre result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}
