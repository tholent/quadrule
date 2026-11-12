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

//! The rung 7a groups page (docs/bbq/2026-11-11-rung-7-groups-and-align.md, Defaulted "Goldens (7a)"):
//! groups nested three deep, a turned group with a turned child, a group at 50 percent whose
//! children overlap, a group whose child is pinned to the top and right, and a heavy group,
//! exported on both devices and compared with golden images in `tests/golden/`, whose sums sit
//! after rung 1's two, 6a's two and 6b's two in `SHA256SUMS`. Then pixel checks on what groups
//! promise: grouping moves nothing, a group fades as one, a turned group is its children turned.
//! Each step prints one `CHECK` line (run with `-- --nocapture`).
//!
//! To accept new output as golden (after a deliberate change, reviewed by eye):
//! `QUADRULE_BLESS=1 cargo test -p quadrule-render --test groups`, then commit `tests/golden/`.
//! Blessing only replaces this test's own lines of `SHA256SUMS`.

use quadrule_format::sha256_hex;
use quadrule_layout::display_list;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_render::*;
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
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("groups");
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
fn groups_page_goldens() {
    let bless = std::env::var("QUADRULE_BLESS").is_ok_and(|v| v == "1");
    let doc = groups_document();
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
        let name = format!("groups_{id}_{}x{}.png", target.width_px, target.height_px);
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
            "\nCHECK crate=render name=groups_golden profile={id} result={} sha256={sha} golden_sha256={} portable_diff_px={d_px} portable_max_diff={d_max} items={} os={} arch={}",
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
                    .is_some_and(|n| n.starts_with("groups_"))
            })
            .map(|l| format!("{l}\n"))
            .collect();
        merged.push_str(&sums);
        std::fs::write(&path, merged).unwrap();
    }
    // The sums file holds them too, after the six that were there before, which are unchanged.
    let file = std::fs::read_to_string(golden_dir().join("SHA256SUMS")).unwrap();
    let lines: Vec<&str> = file.lines().collect();
    let listed = sums.lines().all(|l| lines.contains(&l));
    let old_six = lines.len() >= 6
        && lines[..2].iter().all(|l| l.contains("slice_"))
        && lines[2..4].iter().all(|l| l.contains("shapes_"))
        && lines[4..6].iter().all(|l| l.contains("transforms_"));
    println!(
        "\nCHECK crate=render name=groups_sums_listed result={} listed={listed} old_six_first={old_six}",
        pass(listed && old_six)
    );
    assert!(all_ok && listed && old_six, "see the CHECK lines");
}

fn px_of(img: &Gray, id: &str, x_mm: f64, y_mm: f64) -> f64 {
    let k = builtin(id).unwrap().mm_to_px(1.0);
    f64::from(img.at((x_mm * k) as u32, (y_mm * k) as u32))
}

fn render_doc(id: &str, doc: &Document) -> Gray {
    render(&list(id, doc), Math::Fastest)
}

/// A page holding only `objects`.
fn page_of(objects: Vec<Object>) -> Document {
    let mut doc = groups_document();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc
}

#[test]
fn grouping_moves_nothing() {
    // The page's three nested groups, flat or grouped, give the same pixels: an unturned group at
    // full opacity adds nothing to the drawing (rung 7a: "grouping and ungrouping moves nothing").
    let grouped = groups_document();
    let mut flat = grouped.clone();
    let leaves: Vec<Object> = grouped.layers[1]
        .objects
        .iter()
        .flat_map(|o| {
            let mut v = Vec::new();
            fn walk(o: &Object, v: &mut Vec<Object>) {
                if let ObjectKind::Group(g) = &o.kind {
                    g.children.iter().for_each(|c| walk(c, v));
                } else {
                    v.push(o.clone());
                }
            }
            // Only the nested group; the others change the picture on purpose (turn, opacity, a
            // pinned child that is pinned to its group, not the page).
            if o.id == group_ids::NESTED {
                walk(o, &mut v);
            } else {
                v.push(o.clone());
            }
            v
        })
        .collect();
    flat.layers[1].objects = leaves;
    // On the design's own device the pixels are the same. On the other the same positions come
    // out of a different order of float arithmetic (a child's offset in its group, scaled, plus
    // the group's), so an edge can land on the next pixel: a handful of pixels, reported.
    let mut same = true;
    let mut parts = Vec::new();
    for (n, id) in grouped.profile_ids().into_iter().enumerate() {
        let (a, b) = (render_doc(id, &grouped), render_doc(id, &flat));
        let diff = a.data.iter().zip(&b.data).filter(|(x, y)| x != y).count();
        let good = if n == 0 {
            diff == 0
        } else {
            diff * 1000 < a.data.len()
        };
        same &= good;
        parts.push(format!("{id}:differing_px={diff}"));
    }
    println!(
        "\nCHECK crate=render name=grouping_moves_nothing result={} {}",
        pass(same),
        parts.join(" ")
    );
    assert!(same);
}

#[test]
fn a_group_fades_as_one() {
    // A Block bar and a Shade disc that overlap, inside a 50 percent group: the overlap shows the
    // disc (gray 204) mixed once with the page, not the bar through the disc. As two separate 50
    // percent objects the bar would show through, and the pixel would differ.
    let mut ok = true;
    let mut parts = Vec::new();
    for id in [MANTA_ID, NOMAD_ID] {
        let mk = |n: u128, frame: Frame, gray: u8| {
            let mut o = groups_document().layers[1].objects[0].clone();
            o.id = Uuid::from_u128(n);
            o.frame = frame;
            o.constraints = Constraints {
                h: HConstraint::Left,
                v: VConstraint::Top,
            };
            o.style = Style::None;
            o.fill = Some(Fill::Gray(gray));
            o.rotation_deg = 0.0;
            o.opacity_pct = 100;
            o.kind = ObjectKind::Rect(RectShape::default());
            o
        };
        let bar = mk(1, Frame::new(10.0, 10.0, 40.0, 10.0), 0);
        let disc = mk(2, Frame::new(30.0, 5.0, 30.0, 20.0), 200);
        let mut group = mk(3, Frame::new(10.0, 5.0, 50.0, 20.0), 255);
        group.fill = None;
        group.opacity_pct = 50;
        group.kind = ObjectKind::Group(GroupShape {
            children: vec![bar.clone(), disc.clone()],
            extra: Extra::new(),
        });
        let faded = render_doc(id, &page_of(vec![group]));
        let (mut a, mut b) = (bar, disc);
        a.opacity_pct = 50;
        b.opacity_pct = 50;
        let separate = render_doc(id, &page_of(vec![a, b]));
        let alpha = f64::from(quadrule_layout::alpha_of_percent(50)) / 255.0;
        let mix = |below: f64, over: f64| below * (1.0 - alpha) + over * alpha;
        let want = mix(255.0, 200.0);
        let got = px_of(&faded, id, 40.0, 15.0);
        let sep = px_of(&separate, id, 40.0, 15.0);
        let good = (got - want).abs() <= 1.5 && (sep - got).abs() > 10.0;
        ok &= good;
        parts.push(format!(
            "{id}:overlap_group={got}:overlap_separate={sep}:expected={want:.1}"
        ));
    }
    println!(
        "\nCHECK crate=render name=group_fades_as_one result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn a_turned_group_is_its_children_turned() {
    // Ungrouping a turned group bakes its angle into each child; the page looks the same, to the
    // pixel on this page's turned group, before and after.
    let doc = groups_document();
    let g = doc.object(group_ids::TURNED).unwrap().clone();
    let cmd = quadrule_model::edit::ungroup(&doc, group_ids::TURNED).unwrap();
    let mut after = doc.clone();
    after.apply(cmd).unwrap();
    let gone = after.object(group_ids::TURNED).is_none();
    let mut parts = Vec::new();
    let mut ok = gone && g.rotation() == 25.0;
    for id in doc.profile_ids() {
        let a = render_doc(id, &doc);
        let b = render_doc(id, &after);
        let diff = a.data.iter().zip(&b.data).filter(|(x, y)| x != y).count();
        let max = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap_or(0);
        // The same shapes through the same maths, turned in two steps instead of one: float noise
        // may move an anti-aliased edge by a gray level, never a pixel's meaning.
        let good = max <= 4;
        ok &= good;
        parts.push(format!("{id}:differing_px={diff}:max_diff={max}"));
    }
    println!(
        "\nCHECK crate=render name=turned_group_equals_baked_children result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn text_inside_a_group_draws_saves_and_reopens() {
    // The slice page's text boxes, grouped with the page's rule: the page looks the same as flat,
    // the file saves the text's outlines, and it opens again with the same tree and pixels.
    use quadrule_layout::lay_out_text;
    use quadrule_model::edit::group;
    let flat = slice_document();
    let text_ids: Vec<Uuid> = flat
        .leaves()
        .filter(|o| matches!(o.kind, ObjectKind::Text(_)))
        .map(|o| o.id)
        .collect();
    let mut picked = text_ids.clone();
    picked.push(ids::RULE);
    let mut grouped = flat.clone();
    let gid = Uuid::from_u128(0x6000);
    let cmd = group(&flat, &picked, gid, "Words").unwrap();
    grouped.apply(cmd).unwrap();
    let primary = builtin(&flat.primary_profile_id).unwrap();
    let mut reg = quadrule_text::FontRegistry::bundled_only();
    let mut parts = Vec::new();
    let mut ok = !text_ids.is_empty()
        && grouped
            .object(gid)
            .is_some_and(|g| g.children().len() == picked.len());
    let mut outlines = Vec::new();
    for id in flat.profile_ids() {
        let target = builtin(id).unwrap();
        let a = lay_out_text(&flat, &primary, &target, &mut reg).unwrap();
        let b = lay_out_text(&grouped, &primary, &target, &mut reg).unwrap();
        let da = display_list(&flat, &primary, &target, &a.outlines).unwrap();
        let db = display_list(&grouped, &primary, &target, &b.outlines).unwrap();
        let (ra, rb) = (render(&da, Math::Fastest), render(&db, Math::Fastest));
        let diff = ra.data.iter().zip(&rb.data).filter(|(x, y)| x != y).count();
        // Exact on the design's own device. On the other the children follow the group's box (it
        // scales with the page) instead of the page, so the picture may differ on purpose: reported.
        let same = !parts.is_empty() || diff == 0;
        ok &= same;
        parts.push(format!("{id}:differing_px={diff}"));
        outlines.push(b.outlines);
    }
    let file = quadrule_format::QuadFile::new(grouped.clone(), outlines, vec![]);
    let back =
        quadrule_format::QuadFile::from_bytes(&file.to_bytes("groups-test").unwrap()).unwrap();
    let round = back.doc == grouped
        && back.read_only.is_none()
        && back
            .outlines
            .values()
            .all(|o| text_ids.iter().all(|t| o.text(*t).is_some()));
    ok &= round;
    println!(
        "\nCHECK crate=render name=text_in_group result={} {} file_round_trip={round} format={}",
        pass(ok),
        parts.join(" "),
        back.manifest.format_version
    );
    assert!(ok);
}
