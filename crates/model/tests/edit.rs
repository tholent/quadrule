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

//! Commands and undo (rung 3 bbq, Defaulted "Commands and undo"). Each test prints one
//! structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::edit::*;
use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn nudge(h: &mut History, doc: &mut Document, id: Uuid, dx: f64, key: Option<&str>) {
    let o = moved(doc, id, dx);
    h.apply(doc, Command::SetObject(o), key).unwrap();
}

fn moved(doc: &Document, id: Uuid, dx: f64) -> Object {
    let mut o = doc.object(id).unwrap().clone();
    o.frame.x_mm += dx;
    o
}

/// One command of every kind, each valid on the slice document.
fn every_kind(doc: &Document) -> Vec<(&'static str, Command)> {
    let mut page = doc.page.clone();
    page.background_gray = 240;
    let mut rhythm = doc.rhythm.clone();
    rhythm.v_unit_mm = 7.0;
    let mut meta = doc.metadata.clone();
    meta.title = "Edited".into();
    let mut copy = doc.object(ids::RULE).unwrap().clone();
    copy.id = Uuid::from_u128(99);
    vec![
        ("SetObject", Command::SetObject(moved(doc, ids::TITLE, 3.0))),
        (
            "SetLayer",
            Command::SetLayer {
                index: 0,
                props: LayerProps {
                    name: "Back".into(),
                    visible: false,
                    locked: true,
                },
            },
        ),
        ("SetPage", Command::SetPage(page)),
        ("SetRhythm", Command::SetRhythm(rhythm)),
        ("SetMetadata", Command::SetMetadata(meta)),
        ("SetUnits", Command::SetUnits(Unit::Pt)),
        (
            "Insert",
            Command::Insert {
                layer: 1,
                index: 0,
                object: copy,
            },
        ),
        ("Remove", Command::Remove(ids::DOTS)),
        (
            "MoveObject",
            Command::MoveObject {
                id: ids::RULE,
                layer: 0,
                index: 1,
            },
        ),
        ("MoveLayer", Command::MoveLayer { from: 0, to: 1 }),
        (
            "Batch",
            Command::Batch(vec![
                Command::SetObject(moved(doc, ids::RULE, -1.0)),
                Command::Remove(ids::LINES),
            ]),
        ),
    ]
}

#[test]
fn every_command_undoes_and_redoes() {
    let base = slice_document();
    let mut failed = Vec::new();
    for (name, cmd) in every_kind(&base) {
        let mut doc = base.clone();
        let mut h = History::new();
        let applied = h.apply(&mut doc, cmd, None).is_ok();
        let after = doc.clone();
        let changed = after != base;
        let undone = h.undo(&mut doc) && doc == base;
        let redone = h.redo(&mut doc) && doc == after;
        if !(applied && changed && undone && redone) {
            failed.push(name);
        }
    }
    let ok = failed.is_empty();
    println!(
        "CHECK crate=model name=every_command_undoes_and_redoes result={} kinds={} failed={failed:?}",
        pass(ok),
        every_kind(&base).len()
    );
    assert!(ok);
}

#[test]
fn locks_refuse_edits_but_not_the_panel() {
    let base = slice_document();
    let mut doc = base.clone();
    // Lock the title, then try to move it.
    let mut locked = doc.object(ids::TITLE).unwrap().clone();
    locked.locked = true;
    doc.apply(Command::SetObject(locked)).unwrap();
    let before = doc.clone();
    let mut m = moved(&doc, ids::TITLE, 2.0);
    m.locked = true;
    let move_refused = doc.apply(Command::SetObject(m)) == Err(EditError::Locked(ids::TITLE));
    let delete_refused = doc.apply(Command::Remove(ids::TITLE)).is_err();
    let unchanged = doc == before;
    // The panel's eye, name and lock still work.
    let mut panel = doc.object(ids::TITLE).unwrap().clone();
    panel.visible = false;
    panel.name = "Heading".into();
    panel.locked = false;
    let panel_ok = doc.apply(Command::SetObject(panel)).is_ok();

    // A locked layer refuses its objects' edits, inserts and moves into it.
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
    let mut copy = doc.object(ids::RULE).unwrap().clone();
    copy.id = Uuid::from_u128(7);
    let layer_refuses = doc
        .apply(Command::SetObject(moved(&doc, ids::DOTS, 1.0)))
        .is_err()
        && doc.apply(Command::Insert {
            layer: 0,
            index: 0,
            object: copy,
        }) == Err(EditError::LayerLocked(0))
        && doc.apply(Command::MoveObject {
            id: ids::RULE,
            layer: 0,
            index: 0,
        }) == Err(EditError::LayerLocked(0));
    let editable = !doc.editable(ids::DOTS) && doc.editable(ids::RULE);

    let ok = move_refused && delete_refused && unchanged && panel_ok && layer_refuses && editable;
    println!(
        "CHECK crate=model name=locks_refuse_edits result={} move_refused={move_refused} delete_refused={delete_refused} unchanged={unchanged} panel_fields_allowed={panel_ok} locked_layer_refuses={layer_refuses} editable={editable}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_failing_batch_changes_nothing() {
    let base = slice_document();
    let mut doc = base.clone();
    let cmd = Command::Batch(vec![
        Command::SetObject(moved(&doc, ids::TITLE, 5.0)),
        Command::Remove(ids::DOTS),
        Command::Remove(Uuid::from_u128(12345)),
    ]);
    let refused = doc.apply(cmd) == Err(EditError::NoSuchObject(Uuid::from_u128(12345)));
    let unchanged = doc == base;
    let ok = refused && unchanged;
    println!(
        "CHECK crate=model name=failing_batch_changes_nothing result={} refused={refused} unchanged={unchanged}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn typing_merges_until_sealed_and_dirty_follows_saves() {
    let base = slice_document();
    let mut doc = base.clone();
    let mut h = History::new();
    let clean_at_start = !h.is_dirty();
    // Three keystrokes in the x field: one undo step.
    for dx in [1.0, 2.0, 3.0] {
        let mut o = base.object(ids::TITLE).unwrap().clone();
        o.frame.x_mm += dx;
        h.apply(&mut doc, Command::SetObject(o), Some("x")).unwrap();
    }
    let one_step = h.undo_len() == 1;
    h.seal();
    // After the field loses focus, the next edit is its own step.
    nudge(&mut h, &mut doc, ids::TITLE, 1.0, Some("x"));
    let sealed = h.undo_len() == 2;
    // A different field never joins.
    nudge(&mut h, &mut doc, ids::RULE, 1.0, Some("y"));
    let other_field = h.undo_len() == 3;
    // Remove never merges, even with a key.
    let mut d2 = base.clone();
    let mut h2 = History::new();
    h2.apply(&mut d2, Command::Remove(ids::DOTS), Some("k"))
        .unwrap();
    h2.apply(&mut d2, Command::Remove(ids::LINES), Some("k"))
        .unwrap();
    let no_merge_remove = h2.undo_len() == 2;

    // Dirty: saved → clean; edit → dirty; undo back to the save → clean; redo → dirty.
    h.mark_saved();
    let clean_after_save = !h.is_dirty();
    nudge(&mut h, &mut doc, ids::RULE, 1.0, Some("z"));
    let dirty_after_edit = h.is_dirty();
    h.undo(&mut doc);
    let clean_after_undo = !h.is_dirty();
    h.redo(&mut doc);
    let dirty_after_redo = h.is_dirty();
    // Typing on after a save, merging into the saved step, is a change too.
    nudge(&mut h, &mut doc, ids::RULE, 1.0, Some("w"));
    h.mark_saved();
    let steps = h.undo_len();
    nudge(&mut h, &mut doc, ids::RULE, 1.0, Some("w"));
    let dirty_after_merge = h.is_dirty() && h.undo_len() == steps;
    // Undo everything: back to the start, with the first merged step restoring the original x.
    while h.undo(&mut doc) {}
    let back_to_start = doc == base;
    let ok = clean_at_start
        && one_step
        && sealed
        && other_field
        && no_merge_remove
        && clean_after_save
        && dirty_after_edit
        && clean_after_undo
        && dirty_after_redo
        && dirty_after_merge
        && back_to_start;
    println!(
        "CHECK crate=model name=merge_and_dirty result={} merged_to_one={one_step} sealed={sealed} other_field={other_field} remove_never_merges={no_merge_remove} clean_after_save={clean_after_save} dirty_after_edit={dirty_after_edit} clean_after_undo={clean_after_undo} dirty_after_redo={dirty_after_redo} dirty_after_merge={dirty_after_merge} back_to_start={back_to_start}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn undo_depth_is_unlimited() {
    let base = slice_document();
    let mut doc = base.clone();
    let mut h = History::new();
    let steps = 10_000;
    for _ in 0..steps {
        nudge(&mut h, &mut doc, ids::RULE, 0.1, None);
    }
    let depth = h.undo_len();
    let mut undone = 0;
    while h.undo(&mut doc) {
        undone += 1;
    }
    let ok = depth == steps && undone == steps && doc == base && !h.can_undo() && h.can_redo();
    println!(
        "CHECK crate=model name=undo_depth_unlimited result={} steps={steps} depth={depth} undone={undone} back_to_start={}",
        pass(ok),
        doc == base
    );
    assert!(ok);
}

#[test]
fn duplicate_and_delete() {
    let base = slice_document();
    let mut doc = base.clone();
    let mut h = History::new();
    // Dots and lines share the bottom layer: each copy lands just above its original.
    let mut n = 1000;
    let (cmd, copies) = duplicate(&doc, &[ids::DOTS, ids::LINES], || {
        n += 1;
        Uuid::from_u128(n)
    });
    h.apply(&mut doc, cmd, None).unwrap();
    let order: Vec<Uuid> = doc.layers[0].objects.iter().map(|o| o.id).collect();
    let placed = copies.len() == 2
        && order == vec![ids::DOTS, copies[0], ids::LINES, copies[1]]
        && doc.object(copies[0]).map(|o| &o.frame) == base.object(ids::DOTS).map(|o| &o.frame);
    h.apply(&mut doc, remove(&[ids::TITLE, copies[1]]), None)
        .unwrap();
    let removed = doc.object(ids::TITLE).is_none() && doc.object(copies[1]).is_none();
    h.undo(&mut doc);
    h.undo(&mut doc);
    let restored = doc == base;
    let ok = placed && removed && restored;
    println!(
        "CHECK crate=model name=duplicate_and_delete result={} copies_above_originals={placed} removed={removed} undone={restored}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_drag_is_one_step_and_locks_refuse_it() {
    let mut doc = slice_document();
    let before = doc.clone();
    let mut h = History::new();
    let (title, rule) = (ids::TITLE, ids::RULE);
    let f = |doc: &Document, id: Uuid, dx: f64| {
        let mut fr = doc.object(id).unwrap().frame;
        fr.x_mm += dx;
        (id, fr)
    };
    // Two objects moved by one drag: one command, one undo step.
    let frames = [f(&doc, title, 5.0), f(&doc, rule, 5.0)];
    let cmd = set_frames(&doc, &frames);
    h.apply(&mut doc, cmd, None).unwrap();
    let moved = doc.object(title).unwrap().frame.x_mm
        == before.object(title).unwrap().frame.x_mm + 5.0
        && doc.object(rule).unwrap().frame.x_mm == before.object(rule).unwrap().frame.x_mm + 5.0;
    let one_step = h.undo_len() == 1;
    h.undo(&mut doc);
    let undone = doc == before;
    h.redo(&mut doc);
    let redone =
        doc.object(title).unwrap().frame.x_mm == before.object(title).unwrap().frame.x_mm + 5.0;
    h.undo(&mut doc);

    // A frame that did not change is left out, so a drag that went nowhere makes no step.
    let same = [(title, doc.object(title).unwrap().frame)];
    let nothing = is_empty(&set_frames(&doc, &same));

    // A locked object, or an object in a locked layer, refuses the drag, and the other object in
    // the batch is not moved either.
    doc.apply(Command::SetLayer {
        index: 1,
        props: LayerProps {
            name: "Header".into(),
            visible: true,
            locked: true,
        },
    })
    .unwrap();
    let snapshot = doc.clone();
    let cmd = set_frames(&doc, &[f(&doc, title, 5.0)]);
    let refused = h.apply(&mut doc, cmd, None).is_err();
    let untouched = doc == snapshot;
    let ok = moved && one_step && undone && redone && nothing && refused && untouched;
    println!(
        "\nCHECK crate=model name=drag_one_step result={} moved={moved} one_step={one_step} undone={undone} redone={redone} unchanged_is_empty={nothing} locked_refused={refused} untouched={untouched}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn alt_drag_duplicates_in_one_step_and_reorder_is_one_move() {
    let mut doc = slice_document();
    let before = doc.clone();
    let mut h = History::new();
    let mut fr = doc.object(ids::RULE).unwrap().frame;
    fr.y_mm += 30.0;
    // Alt-drag: the copies are made, then given the frames the pointer left them at, as one step.
    let mut n = 1000u128;
    let (dup, copies) = duplicate(&doc, &[ids::RULE], || {
        n += 1;
        Uuid::from_u128(n)
    });
    let mut with_copies = doc.clone();
    with_copies.apply(dup.clone()).unwrap();
    let cmd = Command::Batch(vec![dup, set_frames(&with_copies, &[(copies[0], fr)])]);
    h.apply(&mut doc, cmd, None).unwrap();
    let copy = doc.object(copies[0]).unwrap();
    let placed = copy.frame == fr
        && !copy.locked
        && doc.object(ids::RULE).unwrap().frame == before.object(ids::RULE).unwrap().frame;
    // The copy sits just above its original, in the same layer.
    let (lo, oo) = doc.find(ids::RULE).unwrap();
    let (lc, oc) = doc.find(copies[0]).unwrap();
    let above = lo == lc && oc == oo + 1;
    let one_step = h.undo_len() == 1;
    h.undo(&mut doc);
    let undone = doc == before;

    // Reordering by dragging in the layers panel: into another layer at a place; a move to the
    // place it is in is empty.
    let (l, o) = doc.find(ids::TITLE).unwrap();
    let stay = is_empty(&reorder_to(&doc, ids::TITLE, l, o));
    let cmd = reorder_to(&doc, ids::TITLE, 0, 0);
    h.apply(&mut doc, cmd, None).unwrap();
    let moved = doc.find(ids::TITLE) == Some((0, 0));
    h.undo(&mut doc);
    let back = doc == before;
    let ok = placed && above && one_step && undone && stay && moved && back;
    println!(
        "\nCHECK crate=model name=alt_drag_and_reorder result={} placed={placed} above_original={above} one_step={one_step} undone={undone} same_place_is_empty={stay} reordered={moved} reorder_undone={back}",
        pass(ok)
    );
    assert!(ok);
}
