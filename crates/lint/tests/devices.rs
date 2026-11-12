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

//! L15, constraints cause trouble on a secondary device (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md,
//! Q7): Info, only for an object with no place or size override on that device, and only when it
//! goes (partly) off the page there while it is on the primary page, newly overlaps an object it
//! does not touch on the primary, or has a side squeezed under one device pixel. Each test prints
//! one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_lint::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn empty(id: &str) -> ProfileOutlines {
    ProfileOutlines {
        profile_id: id.into(),
        tables: vec![],
        texts: vec![],
        extra: Extra::new(),
    }
}

fn l15(doc: &Document, id: &str) -> Vec<Finding> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    lint(doc, &primary, &target, &empty(id))
        .into_iter()
        .filter(|f| f.rule == "L15")
        .collect()
}

/// A page of boxes with the given constraints, on the Manta, also targeting the Nomad.
fn page(boxes: &[(&str, Frame, Constraints)]) -> Document {
    let mut doc = groups_document();
    let proto = doc.object(group_ids::nth(2)).unwrap().clone();
    let objects = boxes
        .iter()
        .enumerate()
        .map(|(n, (name, frame, c))| {
            let mut o = proto.clone();
            o.id = Uuid::from_u128(0x8100 + n as u128);
            o.name = (*name).into();
            o.frame = *frame;
            o.constraints = *c;
            o
        })
        .collect();
    doc.layers = vec![Layer {
        name: "L".into(),
        visible: true,
        locked: false,
        objects,
        extra: Extra::new(),
    }];
    doc
}

const SCALE: Constraints = Constraints {
    h: HConstraint::Scale,
    v: VConstraint::Scale,
};
const LEFT_TOP: Constraints = Constraints {
    h: HConstraint::Left,
    v: VConstraint::Top,
};

#[test]
fn an_object_pinned_off_the_nomad_page_is_found_and_quiet_cases_are() {
    let wide = Frame::new(90.0, 20.0, 62.0, 14.0);
    let mut doc = page(&[
        ("Pinned wide", wide, LEFT_TOP),
        ("Scaled wide", wide, SCALE),
    ]);
    doc.layers[0].objects[1].frame = Frame::new(90.0, 60.0, 62.0, 14.0);
    let found = l15(&doc, NOMAD_ID);
    let names: Vec<_> = found.iter().filter_map(|f| f.object).collect();
    let pinned = Uuid::from_u128(0x8100);
    let off = names == vec![pinned]
        && found[0].severity == Severity::Info
        && found[0].profile == NOMAD_ID
        && found[0].area.is_some();
    // The primary device: never.
    let primary_quiet = l15(&doc, MANTA_ID).is_empty();
    // Placed by hand on the Nomad (a position override): quiet there.
    let mut hand = doc.clone();
    hand.object_mut(pinned).unwrap().overrides.insert(
        NOMAD_ID.into(),
        Override {
            position: Some(Position {
                x_mm: 10.0,
                y_mm: 20.0,
            }),
            ..Override::default()
        },
    );
    let by_hand_quiet = l15(&hand, NOMAD_ID).is_empty();
    // An object that was off the page on the primary already is not L15's business.
    let mut already = doc.clone();
    already.object_mut(pinned).unwrap().frame = Frame::new(150.0, 20.0, 40.0, 14.0);
    let already_quiet = l15(&already, NOMAD_ID).is_empty();
    // The message names the device and the object.
    let strings = quadrule_i18n::Strings::english();
    let text = strings.format(&found[0].message);
    let says = text.contains("Pinned wide") && text.contains("Nomad");
    let ok = off && primary_quiet && by_hand_quiet && already_quiet && says;
    println!(
        "\nCHECK crate=lint name=l15_off_page result={} found_pinned_only={off} primary_quiet={primary_quiet} override_quiet={by_hand_quiet} off_on_primary_quiet={already_quiet} names_object_and_device={says}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_object_newly_on_top_of_another_is_found_once() {
    // On the Manta the two boxes are apart; the Nomad's page is shorter and the left-pinned,
    // top-pinned one stays while the scaled one comes up to meet it.
    let doc = page(&[
        ("Pinned", Frame::new(10.0, 120.0, 40.0, 30.0), LEFT_TOP),
        ("Scaled", Frame::new(10.0, 160.0, 40.0, 30.0), SCALE),
    ]);
    let found = l15(&doc, NOMAD_ID);
    let once = found.len() == 1 && found[0].message.id == "lint-l15-overlap";
    // Touching on the primary: not new.
    let touching = page(&[
        ("Pinned", Frame::new(10.0, 120.0, 40.0, 30.0), LEFT_TOP),
        ("Scaled", Frame::new(10.0, 150.0, 40.0, 30.0), SCALE),
    ]);
    let touch_quiet = l15(&touching, NOMAD_ID).is_empty();
    // One of them placed by hand: quiet.
    let mut hand = doc.clone();
    hand.layers[0].objects[1].overrides.insert(
        NOMAD_ID.into(),
        Override {
            position: Some(Position {
                x_mm: 70.0,
                y_mm: 20.0,
            }),
            ..Override::default()
        },
    );
    let hand_quiet = l15(&hand, NOMAD_ID).is_empty();
    // A hidden one does not count.
    let mut hidden = doc.clone();
    hidden.layers[0].objects[1].visible = false;
    let hidden_quiet = l15(&hidden, NOMAD_ID).is_empty();
    let ok = once && touch_quiet && hand_quiet && hidden_quiet;
    println!(
        "\nCHECK crate=lint name=l15_overlap result={} found_once={once} touching_on_primary_quiet={touch_quiet} override_quiet={hand_quiet} hidden_quiet={hidden_quiet}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_side_squeezed_under_one_pixel_is_found() {
    // Pinned left and right (stretching) with a width that the narrower page squeezes to nothing:
    // 2 mm wide between two gaps that leave less than that on the Nomad.
    let mut doc = page(&[(
        "Squeezed",
        Frame::new(20.0, 20.0, 120.0, 14.0),
        Constraints {
            h: HConstraint::LeftRight,
            v: VConstraint::Top,
        },
    )]);
    // LeftRight keeps both distances, so on the Nomad (43.69 mm narrower) it is 43.69 mm narrower:
    // 43.7 mm wide on the Manta is 0.01 mm there.
    doc.layers[0].objects[0].frame = Frame::new(50.0, 20.0, 43.7, 14.0);
    let found = l15(&doc, NOMAD_ID);
    let squeezed = found.iter().any(|f| f.message.id == "lint-l15-narrow");
    // A plain scaled object is not squeezed.
    let calm = page(&[("Calm", Frame::new(20.0, 20.0, 40.0, 14.0), SCALE)]);
    let calm_quiet = l15(&calm, NOMAD_ID).is_empty();
    let ok = squeezed && calm_quiet;
    println!(
        "\nCHECK crate=lint name=l15_squeezed result={} found={squeezed} scaled_object_quiet={calm_quiet}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_overrides_page_holds_one_l15_case() {
    let doc = devices_document();
    let found = l15(&doc, NOMAD_ID);
    let one = found.len() == 1 && found[0].object == Some(device_ids::OFF_PAGE);
    let manta = l15(&doc, MANTA_ID).is_empty();
    println!(
        "\nCHECK crate=lint name=l15_overrides_page result={} one_case={one} manta_quiet={manta}",
        pass(one && manta)
    );
    assert!(one && manta);
}
