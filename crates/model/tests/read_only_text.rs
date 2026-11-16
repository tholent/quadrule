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

//! Text boxes drawn from saved outlines are read-only (rung 12a bbq, Q6): a command that would lay
//! their text out again is refused by the editing model itself, while moving, turning, fading,
//! copying, grouping, reordering and deleting still work. Each test prints one structured `CHECK`
//! line (run with `-- --nocapture`).

use quadrule_model::edit::*;
use quadrule_model::sample::*;
use quadrule_model::*;
use std::collections::BTreeSet;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// The text-box page and the id of one of its text boxes, made read-only.
fn setup() -> (Document, History, Uuid) {
    let doc = textbox_document();
    let id = doc
        .objects()
        .find(|o| matches!(o.kind, ObjectKind::Text(_)) && o.keeps_height())
        .map(|o| o.id)
        .unwrap();
    let mut h = History::new();
    h.set_read_only_text(BTreeSet::from([id]));
    (doc, h, id)
}

fn change(doc: &Document, id: Uuid, f: impl Fn(&mut Object)) -> Command {
    let mut o = doc.object(id).unwrap().clone();
    f(&mut o);
    Command::SetObject(o)
}

#[test]
fn a_read_only_box_moves_turns_fades_and_goes_but_does_not_reflow() {
    let (mut doc, mut h, id) = setup();
    let allowed: Vec<(&str, Command)> = vec![
        ("move", change(&doc, id, |o| o.translate(3.0, 4.0))),
        ("turn", change(&doc, id, |o| o.rotation_deg = 15.0)),
        ("fade", change(&doc, id, |o| o.opacity_pct = 50)),
        ("rename", change(&doc, id, |o| o.name = "renamed".into())),
        ("lock", change(&doc, id, |o| o.locked = true)),
        ("hide", change(&doc, id, |o| o.visible = false)),
        (
            "move on a device",
            change(&doc, id, |o| {
                o.overrides.insert(
                    NOMAD_ID.into(),
                    Override {
                        position: Some(Position {
                            x_mm: 5.0,
                            y_mm: 5.0,
                        }),
                        ..Default::default()
                    },
                );
            }),
        ),
    ];
    let mut wrongly_refused = Vec::new();
    for (name, cmd) in allowed {
        if h.apply(&mut doc, cmd, None).is_err() {
            wrongly_refused.push(name);
        }
    }
    let refused: Vec<(&str, Command)> = vec![
        (
            "content",
            change(&doc, id, |o| {
                if let ObjectKind::Text(t) = &mut o.kind {
                    t.content.push('x');
                }
            }),
        ),
        (
            "family",
            change(&doc, id, |o| {
                if let ObjectKind::Text(t) = &mut o.kind {
                    t.family = "Inter".into();
                }
            }),
        ),
        (
            "weight",
            change(&doc, id, |o| {
                if let ObjectKind::Text(t) = &mut o.kind {
                    t.weight = if t.weight == 700 { 400 } else { 700 };
                }
            }),
        ),
        (
            "size",
            change(&doc, id, |o| {
                if let ObjectKind::Text(t) = &mut o.kind {
                    t.size_pt += 1.0;
                }
            }),
        ),
        (
            "sizing",
            change(&doc, id, |o| {
                if let ObjectKind::Text(t) = &mut o.kind {
                    t.sizing = TextSizing::Fixed;
                }
            }),
        ),
        ("width", change(&doc, id, |o| o.frame.w_mm += 5.0)),
        (
            "constraints",
            change(&doc, id, |o| {
                o.constraints.h = if o.constraints.h == HConstraint::Left {
                    HConstraint::Right
                } else {
                    HConstraint::Left
                }
            }),
        ),
        (
            "device font size",
            change(&doc, id, |o| {
                o.overrides.insert(
                    NOMAD_ID.into(),
                    Override {
                        font_size_pt: Some(9.0),
                        ..Default::default()
                    },
                );
            }),
        ),
        (
            "device size",
            change(&doc, id, |o| {
                o.overrides.insert(
                    NOMAD_ID.into(),
                    Override {
                        size: Some(Size {
                            w_mm: 30.0,
                            h_mm: 10.0,
                        }),
                        ..Default::default()
                    },
                );
            }),
        ),
    ];
    let before = doc.clone();
    let mut wrongly_allowed = Vec::new();
    for (name, cmd) in refused {
        if !matches!(h.apply(&mut doc, cmd, None), Err(EditError::TextReadOnly(i)) if i == id) {
            wrongly_allowed.push(name);
        }
    }
    let unchanged = doc == before;
    // A batch with one refused step changes nothing at all.
    let batch = Command::Batch(vec![
        change(&doc, id, |o| o.translate(1.0, 1.0)),
        change(&doc, id, |o| o.frame.w_mm += 1.0),
    ]);
    let batch_refused = h.apply(&mut doc, batch, None).is_err() && doc == before;
    // Delete, and put back by undo.
    let deleted = h.apply(&mut doc, Command::Remove(id), None).is_ok() && doc.object(id).is_none();
    let undone = h.undo(&mut doc) && doc.object(id).is_some();
    let ok = wrongly_refused.is_empty()
        && wrongly_allowed.is_empty()
        && unchanged
        && batch_refused
        && deleted
        && undone;
    println!(
        "\nCHECK crate=model name=read_only_text result={} wrongly_refused={wrongly_refused:?} wrongly_allowed={wrongly_allowed:?} unchanged_after_refusals={unchanged} batch_refused={batch_refused} deleted={deleted} undone={undone}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_group_holding_a_read_only_box_moves_but_does_not_scale_it() {
    let (mut doc, mut h, id) = setup();
    // Group the box with a neighbour, as the viewer does.
    let other = doc
        .objects()
        .find(|o| matches!(o.kind, ObjectKind::Text(_)) && o.id != id)
        .map(|o| o.id)
        .unwrap();
    let grouped = quadrule_model::edit::group(&doc, &[id, other], Uuid::from_u128(0xabc), "G")
        .ok()
        .and_then(|cmd| h.apply(&mut doc, cmd, None).ok());
    let g = Uuid::from_u128(0xabc);
    let made = grouped.is_some() && doc.object(g).is_some();
    // Moving the group moves the box with it; stretching it would stretch the text box.
    let moved = {
        let mut o = doc.object(g).unwrap().clone();
        o.translate(2.0, 2.0);
        h.apply(&mut doc, Command::SetObject(o), None).is_ok()
    };
    let stretched = {
        let mut o = doc.object(g).unwrap().clone();
        for c in o.children_mut() {
            c.frame.w_mm += 4.0;
        }
        h.apply(&mut doc, Command::SetObject(o), None)
    };
    let ok = made && moved && matches!(stretched, Err(EditError::TextReadOnly(i)) if i == id);
    println!(
        "\nCHECK crate=model name=read_only_text_in_group result={} grouped={made} moved={moved} stretch_refused={}",
        pass(ok),
        stretched.is_err()
    );
    assert!(ok);
}

#[test]
fn substituting_lifts_the_rule_in_one_undo_step() {
    let (mut doc, mut h, id) = setup();
    let cmd = change(&doc, id, |o| {
        if let ObjectKind::Text(t) = &mut o.kind {
            t.family = "Inter".into();
        }
    });
    let plain = h.apply(&mut doc, cmd.clone(), None).is_err();
    let lifted = h.apply_lifting_read_only(&mut doc, cmd, None).is_ok();
    let family = |d: &Document| match &d.object(id).unwrap().kind {
        ObjectKind::Text(t) => t.family.clone(),
        _ => String::new(),
    };
    let now = family(&doc);
    let undone = h.undo(&mut doc) && family(&doc) != "Inter";
    // The rule still holds after the lifted step.
    let widen = change(&doc, id, |o| o.frame.w_mm += 1.0);
    let still = h.apply(&mut doc, widen, None).is_err();
    let ok = plain && lifted && now == "Inter" && undone && still;
    println!(
        "\nCHECK crate=model name=read_only_text_substitute result={} refused_plain={plain} lifted={lifted} one_undo={undone} rule_back={still}",
        pass(ok)
    );
    assert!(ok);
}
