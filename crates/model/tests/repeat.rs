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

//! The repeat in the model (rung 10, docs/bbq/2026-11-13-rung-10-repeat-tool.md): its record, making
//! one in a single undo step, the box that refits to the source, and the rules that keep the source
//! inside. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::edit::*;
use quadrule_model::sample::*;
use quadrule_model::*;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn leaf(n: u128, frame: Frame) -> Object {
    let mut o = groups_document().layers[1].objects[0].children()[0].clone();
    o.id = Uuid::from_u128(0x7100 + n);
    o.name = format!("o{n}");
    o.frame = frame;
    o.constraints = Constraints::default();
    o.rotation_deg = 0.0;
    o.opacity_pct = 100;
    o.visible = true;
    o.locked = false;
    o.kind = ObjectKind::Rect(RectShape::default());
    o
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x7100 + n)
}

fn page() -> Document {
    let mut doc = groups_document();
    doc.rhythm.h_unit_mm = 5.0;
    doc.rhythm.v_unit_mm = 5.0;
    doc.layers = vec![Layer {
        name: "A".into(),
        visible: true,
        locked: false,
        objects: vec![
            leaf(1, Frame::new(10.0, 10.0, 10.0, 5.0)),
            leaf(2, Frame::new(40.0, 20.0, 5.0, 5.0)),
            leaf(3, Frame::new(60.0, 60.0, 5.0, 5.0)),
        ],
        extra: Extra::new(),
    }];
    doc
}

fn rid() -> Uuid {
    Uuid::from_u128(0x7100_0000)
}

fn gid() -> Uuid {
    Uuid::from_u128(0x7100_0001)
}

#[test]
fn a_new_repeat_has_the_defaults_and_a_box_round_its_copies() {
    let doc = page();
    let cmd = make_repeat(&doc, &[id(1)], rid(), gid(), "Repeat", "Group").unwrap();
    let mut work = doc.clone();
    let mut hist = History::new();
    hist.apply(&mut work, cmd, None).unwrap();
    let r = work.object(rid()).unwrap();
    let s = r.repeat().unwrap();
    // Linear across, count 4, Auto, from the start; the source is the object, inside.
    let defaults = s.mode == RepeatMode::Across
        && !s.fill
        && s.count == 4
        && s.h_spacing == Spacing::Auto
        && s.v_spacing == Spacing::Auto
        && s.basis == SpacingBasis::Pitch
        && s.leftover == Leftover::Start
        && s.rules.is_empty();
    // A 10 mm source on a 5 mm rhythm: pitch 10; four copies are 40 mm wide, 5 mm high.
    let boxed = r.frame.near(&Frame::new(10.0, 10.0, 40.0, 5.0));
    let inside = work.parent_of(id(1)) == Some(rid()) && work.object(id(1)).is_some();
    let one_step = hist.undo_len() == 1;
    hist.undo(&mut work);
    let undone = work == doc;
    println!(
        "\nCHECK crate=model name=repeat_make result={} defaults={defaults} box={boxed} source_inside={inside} one_step={one_step} undone={undone}",
        pass(defaults && boxed && inside && one_step && undone)
    );
    assert!(defaults && boxed && inside && one_step && undone);
}

#[test]
fn several_objects_are_grouped_first_in_one_step() {
    let doc = page();
    let cmd = make_repeat(&doc, &[id(1), id(2)], rid(), gid(), "Repeat", "Group").unwrap();
    let mut work = doc.clone();
    let mut hist = History::new();
    hist.apply(&mut work, cmd, None).unwrap();
    let r = work.object(rid()).unwrap();
    let src = &r.repeat().unwrap().source;
    let grouped = src.id == gid() && src.is_group() && src.children().len() == 2;
    // The group is 10..45 across and 10..25 down: 35 mm wide is 7 steps, so a pitch of 35.
    let boxed = r
        .frame
        .near(&Frame::new(10.0, 10.0, 3.0 * 35.0 + 35.0, 15.0));
    let one_step = hist.undo_len() == 1;
    hist.undo(&mut work);
    let undone = work == doc;
    println!(
        "\nCHECK crate=model name=repeat_make_groups_first result={} grouped={grouped} box={boxed} one_step={one_step} undone={undone}",
        pass(grouped && boxed && one_step && undone)
    );
    assert!(grouped && boxed && one_step && undone);
    assert!(make_repeat(&doc, &[], rid(), gid(), "R", "G").is_err());
}

#[test]
fn the_source_stays_inside_and_the_box_follows_edits_to_it() {
    let doc = page();
    let mut work = doc.clone();
    let mut hist = History::new();
    hist.apply(
        &mut work,
        make_repeat(&doc, &[id(1)], rid(), gid(), "Repeat", "Group").unwrap(),
        None,
    )
    .unwrap();
    // The source cannot be deleted or moved out; the repeat can be deleted whole.
    let before = work.clone();
    let del = hist.apply(&mut work, remove(&[id(1)]), None);
    let out = hist.apply(&mut work, reorder_to(&before, id(1), 0, 0), None);
    let stays = del.is_err() && out.is_err() && work == before;
    // Making the source wider (a 2.3-unit source needs 3 units) refits the box and moves nothing
    // else: the box is 4 pitches of 15 mm.
    let mut src = work.object(id(1)).unwrap().clone();
    src.frame = Frame::new(10.0, 10.0, 11.5, 5.0);
    hist.apply(&mut work, Command::SetObject(src), None)
        .unwrap();
    let refit =
        work.object(rid())
            .unwrap()
            .frame
            .near(&Frame::new(10.0, 10.0, 3.0 * 15.0 + 11.5, 5.0));
    // Moving the source moves the box's corner with it.
    let mut src = work.object(id(1)).unwrap().clone();
    src.frame.x_mm = 30.0;
    src.frame.y_mm = 40.0;
    hist.apply(&mut work, Command::SetObject(src), None)
        .unwrap();
    let f = work.object(rid()).unwrap().frame;
    let follows = (f.x_mm, f.y_mm) == (30.0, 40.0);
    // A change of rhythm refits an Auto pitch (a 4 mm rhythm: 11.5 mm is 3 steps of 4 mm).
    let mut rhythm = work.rhythm.clone();
    rhythm.h_unit_mm = 4.0;
    hist.apply(&mut work, Command::SetRhythm(rhythm), None)
        .unwrap();
    let f = work.object(rid()).unwrap().frame;
    let rhythm_ok = (f.w_mm - (3.0 * 12.0 + 11.5)).abs() < 1e-9;
    println!(
        "\nCHECK crate=model name=repeat_source_and_box result={} stays={stays} refit={refit} follows={follows} rhythm={rhythm_ok}",
        pass(stays && refit && follows && rhythm_ok)
    );
    assert!(stays && refit && follows && rhythm_ok);
}

#[test]
fn moving_a_repeat_moves_its_source_and_fill_keeps_its_size() {
    let doc = page();
    let mut work = doc.clone();
    let mut hist = History::new();
    hist.apply(
        &mut work,
        make_repeat(&doc, &[id(1)], rid(), gid(), "Repeat", "Group").unwrap(),
        None,
    )
    .unwrap();
    let mut r = work.object(rid()).unwrap().clone();
    reframe(&mut r, Frame::new(20.0, 50.0, 40.0, 5.0));
    hist.apply(&mut work, Command::SetObject(r), None).unwrap();
    let moved = work
        .object(id(1))
        .unwrap()
        .frame
        .near(&Frame::new(20.0, 50.0, 10.0, 5.0));
    // Fill mode: the box is the room; a new size is kept and the source goes with the corner.
    let mut r = work.object(rid()).unwrap().clone();
    r.repeat_mut().unwrap().fill = true;
    reframe(&mut r, Frame::new(25.0, 50.0, 120.0, 5.0));
    hist.apply(&mut work, Command::SetObject(r), None).unwrap();
    let r = work.object(rid()).unwrap();
    let fill = r.frame.near(&Frame::new(25.0, 50.0, 120.0, 5.0))
        && work.object(id(1)).unwrap().frame.x_mm == 25.0;
    println!(
        "\nCHECK crate=model name=repeat_move_and_fill result={} moved={moved} fill={fill}",
        pass(moved && fill)
    );
    assert!(moved && fill);
}

#[test]
fn a_repeat_is_found_with_its_source_and_saves_and_reads_back() {
    let doc = page();
    let mut work = doc.clone();
    let mut hist = History::new();
    hist.apply(
        &mut work,
        make_repeat(&doc, &[id(1)], rid(), gid(), "Repeat", "Group").unwrap(),
        None,
    )
    .unwrap();
    // Walks reach the source; leaves leave the repeat out.
    let n_objects = work.objects().count();
    let n_leaves = work.leaves().count();
    let walks = n_objects == 4 && n_leaves == 3 && work.ancestors(id(1)) == vec![rid()];
    // The record survives a trip through bytes and keeps a field from a newer version.
    let mut extra_set = work.clone();
    if let Some(s) = extra_set.object_mut(rid()).and_then(|o| o.repeat_mut()) {
        s.extra
            .insert("x_future".into(), ciborium::Value::Integer(7.into()));
        s.rules.push(AlternationRule {
            counts: RuleCounts::Columns,
            when: RuleWhen::Every {
                n: 7,
                start: Some(3),
            },
            action: RuleAction::LineRole(LineRole::Structure),
        });
        s.h_spacing = Spacing::Units(2.5);
        s.v_spacing = Spacing::Mm(12.0);
    }
    let bytes = {
        let mut b = Vec::new();
        ciborium::into_writer(&extra_set, &mut b).unwrap();
        b
    };
    let back: Document = ciborium::from_reader(bytes.as_slice()).unwrap();
    let round = back == extra_set;
    // A repeat written without the optional fields reads with the defaults.
    let plain = {
        let mut m =
            ciborium::Value::serialized(work.object(rid()).unwrap().repeat().unwrap()).unwrap();
        if let ciborium::Value::Map(entries) = &mut m {
            entries.retain(|(k, _)| k.as_text() == Some("source"));
        }
        let s: RepeatShape = m.deserialized().unwrap();
        s.count == 4 && s.mode == RepeatMode::Across && s.rules.is_empty() && !s.fill
    };
    println!(
        "\nCHECK crate=model name=repeat_record result={} walks={walks} round_trip={round} defaults_on_read={plain}",
        pass(walks && round && plain)
    );
    assert!(walks && round && plain);
}

#[test]
fn the_alternation_rules_count_from_one_and_the_cap_counts_nested_copies() {
    let src = leaf(1, Frame::new(0.0, 0.0, 5.0, 5.0));
    let mut inner = RepeatShape::new(src.clone());
    inner.mode = RepeatMode::Grid;
    inner.columns = 10;
    inner.rows = 10;
    let mut inner_obj = leaf(2, Frame::new(0.0, 0.0, 5.0, 5.0));
    inner_obj.kind = ObjectKind::Repeat(inner);
    let w = inner_obj.copy_weight((5.0, 5.0));
    let mut outer = RepeatShape::new(inner_obj);
    outer.count = 7;
    let mut outer_obj = leaf(3, Frame::new(0.0, 0.0, 5.0, 5.0));
    outer_obj.kind = ObjectKind::Repeat(outer);
    let total = outer_obj.copy_weight((5.0, 5.0));
    let weighs = w == 100 && total == 700;
    // "Every 2nd, from 1" hits 1, 3, 5 ...; a bare "every 3rd" starts at 3.
    let from_one = AlternationRule {
        counts: RuleCounts::Cells,
        when: RuleWhen::Every {
            n: 2,
            start: Some(1),
        },
        action: RuleAction::Skip,
    };
    let bare = AlternationRule {
        counts: RuleCounts::Cells,
        when: RuleWhen::Every { n: 3, start: None },
        action: RuleAction::Skip,
    };
    let rules = (1..=6).filter(|k| from_one.hits(*k)).collect::<Vec<_>>() == [1, 3, 5]
        && (1..=9).filter(|k| bare.hits(*k)).collect::<Vec<_>>() == [3, 6, 9]
        && !from_one.hits(0);
    // The last rule wins on one thing: a later line role beats an earlier one; Skip stays.
    let mut s = RepeatShape::new(src);
    s.rules = vec![
        AlternationRule {
            counts: RuleCounts::Cells,
            when: RuleWhen::List(vec![1]),
            action: RuleAction::LineRole(LineRole::Whisper),
        },
        AlternationRule {
            counts: RuleCounts::Cells,
            when: RuleWhen::List(vec![1]),
            action: RuleAction::LineRole(LineRole::Emphasis),
        },
    ];
    let later = s.effect(1, 1, 4).line == Some(LineRole::Emphasis);
    println!(
        "\nCHECK crate=model name=repeat_rules_and_weight result={} weights={weighs} counting={rules} later_wins={later}",
        pass(weighs && rules && later)
    );
    assert!(weighs && rules && later);
}
