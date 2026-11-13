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

//! Rung 8b, orientation (docs/bbq/2026-11-12-rung-8-several-devices.md, Q6 and Defaulted "8b,
//! orientation", "Landscape safe zones", "Export names", "Tests"): the turned safe zones agree with
//! the device's own, pixel for pixel, through the export turn; New… in landscape gives margins that
//! clear them; the Page tab's switch is one undo step that carries objects by their constraints and
//! overrides in proportion; export writes the device's portrait PNG under its portrait size; L06
//! sees the turned zones. Headless. Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use quadrule_layout::newpage::{NewPage, zone_thickness_px};
use quadrule_model::blank::RhythmPreset;
use quadrule_model::edit::Command;
use quadrule_model::sample::{MANTA_ID, NOMAD_ID};
use quadrule_model::{
    Constraints, DashStyle, Document, Extra, Frame, HConstraint, LineCap, LineRole, Object,
    ObjectKind, Orientation, Override, Pattern, PatternRegion, Position, RectShape, Size, Style,
    Uuid, VConstraint,
};
use quadrule_profiles::{Profile, builtin, builtin_for};
use quadrule_render::{Gray, encode_png, turn_clockwise};
use quadrule_ui::Viewer;
use quadrule_ui::viewer::{ExportWhich, export_pixels};
use std::path::PathBuf;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn spec(device: &str, orientation: Orientation) -> NewPage {
    NewPage {
        title: "Sideways".into(),
        device_id: device.into(),
        rhythm: RhythmPreset::Handwriting,
        orientation,
    }
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A white page of `p`'s size with every safe zone painted black.
fn zone_mask(p: &Profile) -> Gray {
    let (w, h) = (p.width_px, p.height_px);
    let mut data = vec![255u8; (w * h) as usize];
    for z in &p.safe_zones {
        let (x, y, zw, zh) = z.rect;
        for py in y..(y + zh).min(h) {
            for px in x..(x + zw).min(w) {
                data[(py * w + px) as usize] = 0;
            }
        }
    }
    Gray {
        width: w,
        height: h,
        data,
    }
}

/// An object of `kind` at `frame` with constraints `c`.
fn object(n: u128, frame: Frame, c: Constraints, kind: ObjectKind) -> Object {
    Object {
        id: Uuid::from_u128(n),
        name: format!("Object {n}"),
        frame,
        constraints: c,
        visible: true,
        locked: false,
        style: Style::Role(LineRole::Structure),
        fill: None,
        dash: DashStyle::Solid,
        cap: LineCap::Butt,
        rotation_deg: 0.0,
        opacity_pct: 100,
        overrides: Default::default(),
        kind,
        extra: Extra::new(),
    }
}

/// A pattern region that fills the page inside its margins, on the page as the document holds it.
fn fill_region(doc: &Document) -> Object {
    let p = builtin_for(&doc.primary_profile_id, doc.page.orientation).unwrap();
    let (w, h) = p.page_mm();
    let m = doc.page.margins_mm;
    object(
        1,
        Frame::new(m.left, m.top, w - m.left - m.right, h - m.top - m.bottom),
        Constraints {
            h: HConstraint::LeftRight,
            v: VConstraint::TopBottom,
        },
        ObjectKind::Pattern(PatternRegion::new(Pattern::Lines)),
    )
}

fn l06_count(v: &mut Viewer, id: &str) -> usize {
    v.page_for(id)
        .unwrap()
        .findings
        .iter()
        .filter(|f| f.rule == "L06" && f.severity != quadrule_lint::Severity::Info)
        .count()
}

#[test]
fn turned_zones_agree_with_the_device_through_the_export_turn() {
    let mut line = String::new();
    let mut ok = true;
    for id in [MANTA_ID, NOMAD_ID] {
        let p = builtin(id).unwrap();
        let l = p.oriented(Orientation::Landscape);
        // The design's page is the device's turned; the PNG stays the device's own size.
        let sizes = (l.width_px, l.height_px) == (p.height_px, p.width_px)
            && l.device_size() == (p.width_px, p.height_px)
            && l.turned
            && !p.turned;
        // Painted in the design's frame and turned as export turns, the zones are the device's.
        let agree = turn_clockwise(&zone_mask(&l)) == zone_mask(&p);
        // Turning back gives the device's profile; asking twice changes nothing.
        let back = l.oriented(Orientation::Portrait) == p
            && l.oriented(Orientation::Landscape) == l
            && p.oriented(Orientation::Portrait) == p
            && builtin_for(id, Orientation::Landscape).unwrap() == l;
        // The device's left toolbar lies along the design's bottom: its zone touches the design's
        // bottom edge and spans its width.
        let left_at_bottom = l.safe_zones.iter().any(|z| {
            z.name == "toolbar_left" && z.rect.1 + z.rect.3 == l.height_px && z.rect.2 > z.rect.3
        });
        // New…'s margins turn with them: the design's top clears the device's right toolbar, and
        // so on round the page.
        let (pm, lm) = (zone_thickness_px(&p), zone_thickness_px(&l));
        let sides = lm.top == pm.right
            && lm.right == pm.bottom
            && lm.bottom == pm.left
            && lm.left == pm.top;
        let this = sizes && agree && back && left_at_bottom && sides;
        ok &= this;
        line.push_str(&format!(
            " {}={}x{}:zones={}:agree={agree}:sides={sides}",
            p.short_name,
            l.width_px,
            l.height_px,
            l.safe_zones.len()
        ));
    }
    println!(
        "CHECK crate=ui name=landscape_turned_zones result={}{line}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn new_in_landscape_clears_the_turned_zones_on_both_devices() {
    let mut results = Vec::new();
    let mut ok = true;
    for device in [MANTA_ID, NOMAD_ID] {
        let mut v = Viewer::default();
        v.open_new(&spec(device, Orientation::Landscape)).unwrap();
        let doc = v.doc().unwrap().clone();
        let m = doc.page.margins_mm;
        let landscape = doc.page.orientation == Orientation::Landscape;
        let whole = [m.top, m.right, m.bottom, m.left]
            .iter()
            .all(|x| (x / 6.0 - (x / 6.0).round()).abs() < 1e-9);
        let clears = [MANTA_ID, NOMAD_ID].iter().all(|id| {
            let p = builtin_for(id, Orientation::Landscape).unwrap();
            let need = zone_thickness_px(&p);
            p.mm_to_px(m.left) >= need.left
                && p.mm_to_px(m.right) >= need.right
                && p.mm_to_px(m.top) >= need.top
                && p.mm_to_px(m.bottom) >= need.bottom
        });
        // A region filling the margins draws on the turned page and touches no zone.
        let region = fill_region(&doc);
        v.edit(
            Command::Insert {
                layer: 0,
                index: 0,
                object: region,
            },
            None,
        )
        .unwrap();
        let wide = {
            let page = v.page_for(device).unwrap();
            page.image.width > page.image.height
        };
        let bad = l06_count(&mut v, MANTA_ID) + l06_count(&mut v, NOMAD_ID);
        let this = landscape && whole && clears && wide && bad == 0;
        ok &= this;
        results.push(format!(
            "{}:margins_mm={}/{}/{}/{}:l06={bad}",
            builtin(device).unwrap().short_name,
            m.top,
            m.right,
            m.bottom,
            m.left
        ));
    }
    println!(
        "CHECK crate=ui name=new_landscape_margins result={} {}",
        pass(ok),
        results.join(" ")
    );
    assert!(ok);
}

#[test]
fn l06_sees_the_turned_zones() {
    // A box along the design's bottom edge sits on the device's left toolbar once turned: L06.
    // The landscape sample page itself, with 12 mm margins, has none.
    let mut v = Viewer::default();
    v.open_landscape();
    let clean = l06_count(&mut v, MANTA_ID) + l06_count(&mut v, NOMAD_ID);
    let (w, h) = builtin_for(MANTA_ID, Orientation::Landscape)
        .unwrap()
        .page_mm();
    let low = object(
        900,
        Frame::new(w / 2.0 - 10.0, h - 4.0, 20.0, 3.0),
        Constraints::default(),
        ObjectKind::Rect(RectShape::default()),
    );
    v.edit(
        Command::Insert {
            layer: 1,
            index: 0,
            object: low,
        },
        None,
    )
    .unwrap();
    let flagged = v
        .page_for(MANTA_ID)
        .unwrap()
        .findings
        .iter()
        .any(|f| f.rule == "L06" && f.object == Some(Uuid::from_u128(900)));
    let ok = clean == 0 && flagged;
    println!(
        "CHECK crate=ui name=landscape_l06 result={} sample_l06={clean} bottom_box_flagged={flagged}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_page_tab_switch_is_one_undo_step() {
    let mut v = Viewer::default();
    v.open_new(&spec(MANTA_ID, Orientation::Portrait)).unwrap();
    let m = v.doc().unwrap().page.margins_mm;
    // One region pinned to every margin, one box pinned top-left, one box that scales, and on the
    // Nomad a position and size override on the scaling box.
    let region = fill_region(v.doc().unwrap());
    let pinned = object(
        2,
        Frame::new(m.left, m.top, 20.0, 10.0),
        Constraints {
            h: HConstraint::Left,
            v: VConstraint::Top,
        },
        ObjectKind::Rect(RectShape::default()),
    );
    let mut scaled = object(
        3,
        Frame::new(50.0, 60.0, 30.0, 40.0),
        Constraints {
            h: HConstraint::Scale,
            v: VConstraint::Scale,
        },
        ObjectKind::Rect(RectShape::default()),
    );
    scaled.overrides.insert(
        NOMAD_ID.into(),
        Override {
            position: Some(Position {
                x_mm: 20.0,
                y_mm: 30.0,
            }),
            size: Some(Size {
                w_mm: 10.0,
                h_mm: 20.0,
            }),
            ..Override::default()
        },
    );
    for (i, o) in [region, pinned, scaled].into_iter().enumerate() {
        v.edit(
            Command::Insert {
                layer: 0,
                index: i,
                object: o,
            },
            None,
        )
        .unwrap();
    }
    let before = v.doc().unwrap().clone();
    let (pw, ph) = builtin(MANTA_ID).unwrap().page_mm();
    let (nw, nh) = builtin(NOMAD_ID).unwrap().page_mm();

    v.set_orientation(Orientation::Landscape).unwrap();
    let after = v.doc().unwrap().clone();
    let turned = after.page.orientation == Orientation::Landscape;
    let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
    let find = |d: &Document, n: u128| d.object(Uuid::from_u128(n)).unwrap().clone();
    // The region still fills the margins on the turned page (LeftRight, TopBottom).
    let r = find(&after, 1).frame;
    let region_follows = close(r.x_mm, m.left)
        && close(r.y_mm, m.top)
        && close(r.x_mm + r.w_mm, ph - m.right)
        && close(r.y_mm + r.h_mm, pw - m.bottom);
    // The pinned box stays where it was, top-left.
    let p = find(&after, 2).frame;
    let pinned_stays = close(p.x_mm, m.left) && close(p.y_mm, m.top) && close(p.w_mm, 20.0);
    // The scaling box keeps its share of the page: its left was 50 / pw across.
    let s = find(&after, 3);
    let scales = close(s.frame.x_mm / ph, 50.0 / pw) && close(s.frame.y_mm / pw, 60.0 / ph);
    // Its Nomad override maps onto the Nomad's turned page in proportion.
    let ov = &s.overrides[NOMAD_ID];
    let pos = ov.position.unwrap();
    let size = ov.size.unwrap();
    let override_scales = close(pos.x_mm, 20.0 * nh / nw)
        && close(pos.y_mm, 30.0 * nw / nh)
        && close(size.w_mm, 10.0 * nh / nw)
        && close(size.h_mm, 20.0 * nw / nh);
    // The pages draw wide.
    let wide = [MANTA_ID, NOMAD_ID].iter().all(|id| {
        let page = v.page_for(id).unwrap();
        page.image.width > page.image.height
    });
    // One undo step puts everything back exactly; switching to the same orientation is nothing.
    v.set_orientation(Orientation::Landscape).unwrap();
    let same_is_nothing = v.doc().unwrap() == &after;
    let undone = v.undo() && v.doc().unwrap() == &before;
    let ok = turned
        && region_follows
        && pinned_stays
        && scales
        && override_scales
        && wide
        && same_is_nothing
        && undone;
    println!(
        "CHECK crate=ui name=page_tab_orientation result={} turned={turned} region_follows={region_follows} pinned_stays={pinned_stays} scales={scales} override_scales={override_scales} wide={wide} same_is_nothing={same_is_nothing} one_undo={undone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn export_writes_the_portrait_png_turned() {
    let mut v = Viewer::default();
    v.open_landscape();
    let dir = tmp("ui-export-landscape");
    let all = v.export(&dir, ExportWhich::AllDevices).unwrap();
    let names: Vec<String> = all
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let want = [
        "rung-8b-landscape_manta_1920x2560.png",
        "rung-8b-landscape_nomad_1404x1872.png",
    ];
    let golden = |id: &str, (w, h): (u32, u32)| {
        std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../render/tests/golden")
                .join(format!("landscape_{id}_{w}x{h}.png")),
        )
        .unwrap_or_default()
    };
    let mut bytes_ok = true;
    let mut shown_upright = true;
    for (path, id) in all.iter().zip([MANTA_ID, NOMAD_ID]) {
        let page = v.page_for(id).unwrap();
        let size = page.profile.device_size();
        shown_upright &= page.image.width > page.image.height;
        let expect = encode_png(&export_pixels(page), None);
        let written = std::fs::read(path).unwrap();
        bytes_ok &= written == expect && written == golden(id, size);
    }
    let ok = names == want && bytes_ok && shown_upright;
    println!(
        "CHECK crate=ui name=landscape_export result={} names={names:?} bytes_match_golden={bytes_ok} shown_upright={shown_upright}",
        pass(ok)
    );
    assert!(ok);
}
