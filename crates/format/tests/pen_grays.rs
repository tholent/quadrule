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

//! ADR-027 item 5: with `--pen-grays` on, a 0.21 file opens through the 0.22 step, which moves each hand-picked gray on
//! one of the old 16 steps to the new step in its place and leaves any other gray, and every role, as it was. Saved
//! again, it says 0.22.0. One test, so the process-wide switch is never flipped under another. Prints one structured
//! `CHECK` line (run with `-- --nocapture`).

use quadrule_format::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_text::FontRegistry;

/// The page's real text outlines, one per profile.
fn outlines(doc: &Document) -> Vec<ProfileOutlines> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    doc.profile_ids()
        .iter()
        .map(|id| {
            quadrule_layout::lay_out_text(doc, &primary, &builtin(id).unwrap(), &mut reg)
                .unwrap()
                .outlines
        })
        .collect()
}

fn custom(gray: u8) -> Style {
    Style::Custom {
        width_mm: 0.3,
        gray,
    }
}

#[test]
fn on_step_grays_move_and_others_stay() {
    // A 0.21 file, saved with the switch off: custom strokes at 153, 0 and 160, a hand-picked fill at 204, a role left
    // alone, and a page of 238.
    pen_grays::set(false);
    let mut doc = shapes_document();
    let ids: Vec<Uuid> = doc.objects().take(4).map(|o| o.id).collect();
    doc.object_mut(ids[0]).unwrap().style = custom(153);
    doc.object_mut(ids[1]).unwrap().style = custom(0);
    doc.object_mut(ids[2]).unwrap().style = custom(160);
    doc.object_mut(ids[2]).unwrap().fill = Some(Fill::Gray(204));
    doc.object_mut(ids[3]).unwrap().style = Style::Role(LineRole::Structure);
    doc.page.background_gray = 238;
    let o = outlines(&doc);
    let bytes = QuadFile::new(doc, o, vec![])
        .to_bytes("pen-grays-test")
        .unwrap();
    let saved_at_021 = QuadFile::from_bytes(&bytes)
        .is_ok_and(|f| f.manifest.format_version == "0.21.0" && f.migrated_from.is_none());

    // Opened with the switch on.
    pen_grays::set(true);
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let style = |i: usize| back.doc.object(ids[i]).unwrap().style;
    let moved = style(0) == custom(157) && style(1) == custom(1);
    let off_step_kept = style(2) == custom(160);
    let fill_moved = back.doc.object(ids[2]).unwrap().fill == Some(Fill::Gray(201));
    let role_kept = style(3) == Style::Role(LineRole::Structure);
    let page_moved = back.doc.page.background_gray == 237;
    let from_021 = back.migrated_from.as_ref().map(|v| v.to_string()) == Some("0.21.0".into());
    let resaved = QuadFile::from_bytes(&back.to_bytes("pen-grays-test").unwrap())
        .is_ok_and(|f| f.manifest.format_version == "0.22.0" && f.migrated_from.is_none());
    pen_grays::set(false);

    let ok = saved_at_021
        && moved
        && off_step_kept
        && fill_moved
        && role_kept
        && page_moved
        && from_021
        && resaved;
    println!(
        "CHECK crate=format name=pen_grays_migration result={} saved_at_021={saved_at_021} on_step_moved={moved} off_step_kept={off_step_kept} fill_moved={fill_moved} role_kept={role_kept} page_moved={page_moved} from_021={from_021} resaved_at_022={resaved}",
        if ok { "pass" } else { "fail" }
    );
    assert!(ok);
}
