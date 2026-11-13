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

//! Lint and repeats (rung 10, docs/bbq/2026-11-13-rung-10-repeat-tool.md, Defaulted "L16 and L17
//! learn repeats", "Other lint rules on copies"): L06 and L10 judge every drawn copy and give one
//! finding per repeat and device with the copy count; L16 and L17 judge the pitch; the other rules
//! judge the source once. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_i18n::{Arg, Message};
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

fn findings(doc: &Document, id: &str) -> Vec<Finding> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let target = builtin(id).unwrap();
    lint(doc, &primary, &target, &empty(id))
}

fn num_arg(m: &Message, name: &str) -> Option<f64> {
    m.args.iter().find_map(|(n, a)| match a {
        Arg::Num(v, _) if *n == name => Some(*v),
        _ => None,
    })
}

/// A page holding one repeat of a 10 x 6 mm box, made by `tweak`, with the rhythm at 5 mm.
fn page(at: (f64, f64), tweak: impl FnOnce(&mut RepeatShape)) -> Document {
    let mut doc = repeats_document();
    doc.layers[0].objects.clear();
    let mut src = doc_object();
    src.frame = Frame::new(at.0, at.1, 10.0, 5.0);
    let mut shape = RepeatShape::new(src);
    tweak(&mut shape);
    let mut rep = doc_object();
    rep.id = Uuid::from_u128(0x7200);
    rep.name = "Row".into();
    rep.style = Style::None;
    rep.kind = ObjectKind::Repeat(shape);
    rep.frame = rep.repeat_fit((5.0, 5.0)).unwrap();
    doc.layers[0].objects.push(rep);
    doc
}

fn doc_object() -> Object {
    let mut o = repeats_document().layers[0].objects[2].children()[0].clone();
    o.id = Uuid::from_u128(0x7201);
    o.name = "Box".into();
    o.constraints = Constraints {
        h: HConstraint::Left,
        v: VConstraint::Top,
    };
    o.kind = ObjectKind::Rect(RectShape::default());
    o.fill = None;
    o.style = Style::Role(LineRole::Structure);
    o
}

#[test]
fn off_the_page_is_one_finding_per_repeat_with_the_copy_count() {
    // Ten boxes from x = 100 mm on a 162.56 mm page: the sixth on is off or partly off.
    let doc = page((100.0, 40.0), |r| r.count = 10);
    let f = findings(&doc, MANTA_ID);
    let l10: Vec<&Finding> = f.iter().filter(|x| x.rule == "L10").collect();
    let one = l10.len() == 1;
    let m = &l10[0].message;
    let count = num_arg(m, "count").unwrap_or(0.0);
    let total = num_arg(m, "total").unwrap_or(0.0);
    // Boxes start at 100, 110, 120, 130, 140, 150 (to 160, in), 160 (to 170, partly), 170, 180, 190.
    let says = total == 10.0 && count == 4.0 && m.id.starts_with("lint-l10-repeat");
    let names = l10[0].object == Some(Uuid::from_u128(0x7200));
    // A page that fits has none.
    let fits = findings(&page((20.0, 40.0), |r| r.count = 4), MANTA_ID)
        .iter()
        .all(|x| x.rule != "L10");
    let ok = one && says && names && fits;
    println!(
        "\nCHECK crate=lint name=lint_repeat_l10 result={} one_finding={one} copies_off={count} of={total} fits_gives_none={fits}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn a_copy_in_a_safe_zone_is_one_l06_with_the_count() {
    // Three boxes down from y = 196 mm: the third reaches into the left footer's zone (from
    // 206.9 mm down, 11 to 42 mm across).
    let doc = page((20.0, 196.0), |r| {
        r.mode = RepeatMode::Down;
        r.count = 3;
    });
    let f = findings(&doc, MANTA_ID);
    let l06: Vec<&Finding> = f.iter().filter(|x| x.rule == "L06").collect();
    let one = l06.len() == 1 && l06[0].severity == Severity::Warn;
    let m = &l06[0].message;
    let says = m.id == "lint-l06-repeat-fixed"
        && num_arg(m, "count") == Some(1.0)
        && num_arg(m, "total") == Some(3.0);
    let ok = one && says;
    println!(
        "\nCHECK crate=lint name=lint_repeat_l06 result={} one_finding={one} says_count={says}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_pitch_is_judged_by_l16_when_bound_and_by_l17_when_in_mm() {
    // The first box sits two rhythm units in from the margin corner (59 px each at 300 PPI), so
    // its place is on the rhythm and only the pitch is in question.
    let at = (212.0 / (300.0 / 25.4), 448.0 / (300.0 / 25.4));
    let l = |tweak: &dyn Fn(&mut RepeatShape), dev: &str, rule: &str| {
        let doc = page(at, |r| {
            r.count = 3;
            tweak(r);
        });
        findings(&doc, dev)
            .iter()
            .filter(|x| x.rule == rule)
            .count()
    };
    // Bound: Auto and a whole number of units are on the rhythm; 2.5u is not.
    let auto = l(&|_| {}, MANTA_ID, "L16");
    let whole = l(&|r| r.h_spacing = Spacing::Units(3.0), MANTA_ID, "L16");
    let half = l(&|r| r.h_spacing = Spacing::Units(2.5), MANTA_ID, "L16");
    // With subdivisions of 2, a half unit is a step.
    let with_subs = {
        let mut doc = page(at, |r| {
            r.count = 3;
            r.h_spacing = Spacing::Units(2.5);
        });
        doc.rhythm.subdivisions = Some(2);
        findings(&doc, MANTA_ID)
            .iter()
            .filter(|x| x.rule == "L16")
            .count()
    };
    // The pitch is not judged on a secondary device by L16 (checked where the design is made).
    let not_on_nomad = l(&|r| r.h_spacing = Spacing::Units(2.5), NOMAD_ID, "L16");
    let l16_ok = auto == 0 && whole == 0 && half == 1 && with_subs == 0 && not_on_nomad == 0;
    // In mm: 10 mm is two steps (118 px against 59); 12 mm is 142 px, off the multiples.
    let ten = l(&|r| r.h_spacing = Spacing::Mm(10.0), MANTA_ID, "L17");
    let twelve_m = l(&|r| r.h_spacing = Spacing::Mm(12.0), MANTA_ID, "L17");
    let twelve_n = l(&|r| r.h_spacing = Spacing::Mm(12.0), NOMAD_ID, "L17");
    // An unbound pitch is judged by L17 only.
    let no_l16 = l(&|r| r.h_spacing = Spacing::Mm(12.0), MANTA_ID, "L16");
    let l17_ok = ten == 0 && twelve_m == 1 && twelve_n == 1 && no_l16 == 0;
    let ok = l16_ok && l17_ok;
    println!(
        "\nCHECK crate=lint name=lint_repeat_pitch result={} l16_bound_pitch={l16_ok} (auto={auto} whole={whole} half={half} subs={with_subs} nomad={not_on_nomad}) l17_unbound_pitch={l17_ok} (ten={ten} twelve_manta={twelve_m} twelve_nomad={twelve_n} no_l16={no_l16})",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_source_is_judged_once_not_once_per_copy() {
    // A thin custom stroke breaks L01 on its source; ten copies still give one finding.
    let doc = page((20.0, 40.0), |r| {
        r.count = 10;
        r.source.style = Style::Custom {
            width_mm: 0.01,
            gray: 0,
        };
    });
    let f = findings(&doc, MANTA_ID);
    let l01 = f.iter().filter(|x| x.rule == "L01").count();
    // The repeat's box is judged for the rhythm (L16): at 20 mm it is off the 5 mm rhythm origin
    // of the 8 mm margin, once, by the repeat and not by its source.
    let l16 = f.iter().filter(|x| x.rule == "L16").count();
    let ok = l01 == 1 && l16 == 1;
    println!(
        "\nCHECK crate=lint name=lint_repeat_source_once result={} l01={l01} l16={l16}",
        pass(ok)
    );
    assert!(ok);
}
