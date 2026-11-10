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

//! Copy and paste (rung 4b; bbq Q8, Defaulted "Copy and paste"). Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::clipboard::{from_text, to_text};
use quadrule_model::edit::{History, paste};
use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn next_ids() -> impl FnMut() -> Uuid {
    let mut n = 1000_u128;
    move || {
        n += 1;
        Uuid::from_u128(n)
    }
}

#[test]
fn paste_round_trip() {
    let doc = slice_document();
    let ids = [ids::TITLE, ids::RULE];
    let text = to_text(&doc, &ids).expect("copies");
    let objects = from_text(&text).expect("reads back");
    let same_objects = objects.len() == 2
        && objects.iter().all(|o| {
            let original = doc
                .object(o.id)
                .expect("the copy kept the old id until pasted");
            original == o
        });

    let mut pasted_doc = doc.clone();
    let (cmd, new) = paste(&pasted_doc, &objects, 0, (0.0, 0.0), next_ids());
    let mut h = History::new();
    h.apply(&mut pasted_doc, cmd, None).unwrap();
    let fresh = new.len() == 2
        && new.iter().all(|id| doc.object(*id).is_none())
        && new.iter().all(|id| pasted_doc.object(*id).is_some());
    let same_content = new.iter().zip(&objects).all(|(id, o)| {
        let mut p = pasted_doc.object(*id).unwrap().clone();
        p.id = o.id;
        p == *o
    });
    // Both land on top of layer 0, in the copied order, and one undo takes them away.
    let layer = &pasted_doc.layers[0].objects;
    let on_top = layer[layer.len() - 2].id == new[0] && layer[layer.len() - 1].id == new[1];
    h.undo(&mut pasted_doc);
    let undone = pasted_doc == doc;
    println!(
        "\nCHECK crate=model name=paste_round_trip result={} same_objects={same_objects} fresh_ids={fresh} same_content={same_content} on_top={on_top} undone={undone}",
        pass(same_objects && fresh && same_content && on_top && undone)
    );
    assert!(same_objects && fresh && same_content && on_top && undone);
}

#[test]
fn paste_offset_and_locks() {
    let mut doc = slice_document();
    let (l, o) = doc.find(ids::RULE).unwrap();
    doc.layers[l].objects[o].locked = true;
    let text = to_text(&doc, &[ids::RULE]).unwrap();
    let objects = from_text(&text).unwrap();
    let (cmd, new) = paste(&doc, &objects, l, (6.0, 7.0), next_ids());
    let mut after = doc.clone();
    after.apply(cmd).unwrap();
    let p = after.object(new[0]).unwrap();
    let moved = (p.frame.x_mm - objects[0].frame.x_mm - 6.0).abs() < 1e-9
        && (p.frame.y_mm - objects[0].frame.y_mm - 7.0).abs() < 1e-9;
    let unlocked = objects[0].locked && !p.locked;
    let (none, ids_none) = paste(&doc, &objects, 99, (0.0, 0.0), next_ids());
    let no_layer = quadrule_model::edit::is_empty(&none) && ids_none.is_empty();
    println!(
        "\nCHECK crate=model name=paste_offset_and_locks result={} moved={moved} unlocked={unlocked} no_layer={no_layer}",
        pass(moved && unlocked && no_layer)
    );
    assert!(moved && unlocked && no_layer);
}

#[test]
fn clipboard_ignores_other_text() {
    let doc = slice_document();
    let good = to_text(&doc, &[ids::TITLE]).unwrap();
    let cases: [(&str, bool); 8] = [
        ("hello world", false),
        ("", false),
        ("{}", false),
        ("{\"objects\": []}", false),
        ("{\"quadrule-clipboard\": 1, \"objects\": []}", false),
        ("{\"quadrule-clipboard\": 2, \"objects\": []}", false),
        ("{\"quadrule-clipboard\": 1, \"objects\": [1, 2]}", false),
        (good.as_str(), true),
    ];
    let all = cases.iter().all(|(t, ok)| from_text(t).is_some() == *ok);
    // Nothing to copy: no text.
    let nothing = to_text(&doc, &[Uuid::from_u128(424242)]).is_none();
    // A newer file's unknown fields on an object survive the round trip.
    let mut with_extra = doc.clone();
    let mut o = with_extra.object(ids::TITLE).unwrap().clone();
    o.extra
        .insert("future".into(), ciborium::Value::Integer(7.into()));
    with_extra.apply(edit::Command::SetObject(o)).unwrap();
    let text = to_text(&with_extra, &[ids::TITLE]).unwrap();
    let kept = from_text(&text).unwrap()[0].extra.contains_key("future");
    println!(
        "\nCHECK crate=model name=clipboard_ignores_other_text result={} cases={all} nothing={nothing} unknown_fields_kept={kept}",
        pass(all && nothing && kept)
    );
    assert!(all && nothing && kept);
}
