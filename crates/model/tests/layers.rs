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

//! Making and deleting layers (docs/bbq/2026-11-18-new-and-delete-layers.md): `InsertLayer` and `RemoveLayer`, each the
//! other's undo; a removed layer comes back whole, in its place; one undo step each; the last layer stays; locks refuse.
//! Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::edit::*;
use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn empty(name: &str) -> Layer {
    Layer {
        name: name.into(),
        visible: true,
        locked: false,
        objects: Vec::new(),
        extra: Extra::new(),
    }
}

#[test]
fn insert_and_remove_are_each_others_undo() {
    let base = slice_document();
    let mut doc = base.clone();
    let undo = doc
        .apply(Command::InsertLayer {
            index: 1,
            layer: empty("Layer 3"),
        })
        .unwrap();
    let inserted = doc.layers.len() == 3 && doc.layers[1].name == "Layer 3";
    let undo_is_remove = undo == Command::RemoveLayer(1);
    let redo = doc.apply(undo).unwrap();
    let back = doc == base;
    let redo_is_insert = matches!(&redo, Command::InsertLayer { index: 1, layer } if layer.name == "Layer 3");
    doc.apply(redo).unwrap();
    let again = doc.layers.len() == 3 && doc.layers[1].name == "Layer 3";
    // On top: an index of the layer count.
    let top = doc.apply(Command::InsertLayer {
        index: 3,
        layer: empty("Top"),
    });
    let on_top = top.is_ok() && doc.layers.last().is_some_and(|l| l.name == "Top");
    let past = doc.apply(Command::InsertLayer {
        index: 9,
        layer: empty("Nowhere"),
    }) == Err(EditError::BadIndex(9));
    let ok = inserted && undo_is_remove && back && redo_is_insert && again && on_top && past;
    println!(
        "CHECK crate=model name=insert_layer_and_remove_layer_undo result={} inserted={inserted} undo_is_remove={undo_is_remove} back={back} redo_is_insert={redo_is_insert} again={again} on_top={on_top} index_past_the_end_refused={past}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_removed_layer_comes_back_whole_in_its_place() {
    let base = slice_document();
    let mut doc = base.clone();
    let mut h = History::new();
    h.apply(&mut doc, Command::RemoveLayer(0), None).unwrap();
    let gone = doc.layers.len() == 1
        && doc.layers[0].name == "Header"
        && doc.object(ids::DOTS).is_none()
        && doc.object(ids::LINES).is_none();
    let one_step = h.undo(&mut doc);
    let whole = doc == base
        && doc.layers[0].objects.iter().map(|o| o.id).collect::<Vec<_>>()
            == vec![ids::DOTS, ids::LINES];
    let redone = h.redo(&mut doc) && doc.layers.len() == 1 && doc.object(ids::DOTS).is_none();
    let undone_again = h.undo(&mut doc) && doc == base;
    // The middle of three keeps its place.
    let mut three = base.clone();
    three
        .apply(Command::InsertLayer {
            index: 1,
            layer: empty("Middle"),
        })
        .unwrap();
    let before = three.clone();
    let mut h3 = History::new();
    h3.apply(&mut three, Command::RemoveLayer(1), None).unwrap();
    let names: Vec<&str> = three.layers.iter().map(|l| l.name.as_str()).collect();
    let neighbours = names == ["Grid", "Header"];
    h3.undo(&mut three);
    let in_place = three == before;
    // Nothing left to undo after the one step.
    let no_second = !h.undo(&mut doc) || doc == base;
    let ok = gone && one_step && whole && redone && undone_again && neighbours && in_place && no_second;
    println!(
        "CHECK crate=model name=removed_layer_returns_whole result={} gone={gone} one_undo_step={one_step} whole={whole} redone={redone} undone_again={undone_again} neighbours={neighbours} in_place={in_place}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_last_layer_stays_and_locks_refuse() {
    let base = slice_document();
    // The last layer cannot go.
    let mut one = base.clone();
    one.apply(Command::RemoveLayer(1)).unwrap();
    let last = one.apply(Command::RemoveLayer(0)) == Err(EditError::LastLayer)
        && one.layer_delete_block(0) == Some(LayerDeleteBlock::Last);
    // A locked layer refuses; so does an unlocked one holding a locked object.
    let mut doc = base.clone();
    doc.apply(Command::SetLayer {
        index: 0,
        props: LayerProps {
            name: "Grid".into(),
            visible: true,
            locked: true,
        },
    })
    .unwrap();
    let locked_layer = doc.apply(Command::RemoveLayer(0)) == Err(EditError::LayerLocked(0))
        && doc.layer_delete_block(0) == Some(LayerDeleteBlock::Locked);
    let mut doc = base.clone();
    let mut title = doc.object(ids::TITLE).unwrap().clone();
    title.locked = true;
    doc.apply(Command::SetObject(title)).unwrap();
    let before = doc.clone();
    let locked_object = doc.apply(Command::RemoveLayer(1)) == Err(EditError::Locked(ids::TITLE))
        && doc.layer_delete_block(1) == Some(LayerDeleteBlock::HoldsLocked)
        && doc.layer_delete_block(0).is_none();
    let unchanged = doc == before;
    // Nothing there.
    let missing = doc.apply(Command::RemoveLayer(5)) == Err(EditError::NoSuchLayer(5));
    let ok = last && locked_layer && locked_object && unchanged && missing;
    println!(
        "CHECK crate=model name=last_layer_stays_and_locks_refuse result={} last_layer={last} locked_layer={locked_layer} locked_object_inside={locked_object} unchanged={unchanged} no_such_layer={missing}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn counts_every_row_a_layer_has_and_refuses_a_repeated_id() {
    let doc = groups_document();
    let counts: Vec<usize> = (0..doc.layers.len())
        .map(|i| doc.layer_object_count(i))
        .collect();
    let total: usize = counts.iter().sum();
    let all = doc.objects().count();
    let sums = total == all && all > doc.layers.iter().map(|l| l.objects.len()).sum::<usize>();
    let none = doc.layer_object_count(99) == 0;
    // A layer whose object id is already in the page is refused.
    let mut dup = doc.clone();
    let clone = doc.layers[0].clone();
    let refused = matches!(
        dup.apply(Command::InsertLayer {
            index: 0,
            layer: clone
        }),
        Err(EditError::DuplicateId(_))
    );
    let ok = sums && none && refused;
    println!(
        "CHECK crate=model name=layer_object_count_and_duplicate_ids result={} counts={counts:?} equals_objects={sums} missing_layer_is_zero={none} duplicate_refused={refused}",
        pass(ok)
    );
    assert!(ok);
}
