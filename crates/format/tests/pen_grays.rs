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

//! ADR-027 item 5: the 0.22 step moves each hand-picked gray on one of the old 16 steps to the new step in its place
//! and leaves any other gray, and every role, as it was; a file saved now says 0.22.0. Prints one structured `CHECK`
//! line (run with `-- --nocapture`).

use ciborium::Value;
use quadrule_format::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use semver::Version;

fn custom(gray: u8) -> Style {
    Style::Custom {
        width_mm: 0.3,
        gray,
    }
}

#[test]
fn on_step_grays_move_and_others_stay() {
    // A 0.21 scene: custom strokes at 153, 0 and 160, a hand-picked fill at 204, a role, and a page of 238.
    let mut doc = shapes_document();
    let ids: Vec<Uuid> = doc.objects().take(4).map(|o| o.id).collect();
    doc.object_mut(ids[0]).unwrap().style = custom(153);
    doc.object_mut(ids[1]).unwrap().style = custom(0);
    doc.object_mut(ids[2]).unwrap().style = custom(160);
    doc.object_mut(ids[2]).unwrap().fill = Some(Fill::Gray(204));
    doc.object_mut(ids[3]).unwrap().style = Style::Role(LineRole::Structure);
    doc.page.background_gray = 238;
    let mut scene = Value::serialized(&doc).unwrap();
    let (from, to) = (Version::new(0, 21, 0), Version::new(0, 22, 0));
    migrate::upgrade(&mut scene, &from, &to, migrate::CHAIN).unwrap();
    let back: Document = scene.deserialized().unwrap();

    let style = |i: usize| back.object(ids[i]).unwrap().style;
    let moved = style(0) == custom(157) && style(1) == custom(1);
    let off_step_kept = style(2) == custom(160);
    let fill_moved = back.object(ids[2]).unwrap().fill == Some(Fill::Gray(201));
    let role_kept = style(3) == Style::Role(LineRole::Structure);
    let page_moved = back.page.background_gray == 237;
    let current = FORMAT_VERSION == "0.22.0"
        && migrate::CHAIN.last().map(|(f, t, _)| (*f, *t)) == Some(("0.21.0", "0.22.0"));

    let ok = moved && off_step_kept && fill_moved && role_kept && page_moved && current;
    println!(
        "CHECK crate=format name=pen_grays_migration result={} on_step_moved={moved} off_step_kept={off_step_kept} fill_moved={fill_moved} role_kept={role_kept} page_moved={page_moved} writes_022={current}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
