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

//! Strings checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_i18n::{Arg, Message, Strings};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

#[test]
fn english_loads_and_formats_every_message() {
    // Every message formats with no errors once its arguments are given; ids are found from the
    // files themselves, so a new message is checked the day it is added.
    let s = Strings::english();
    let ids = s.ids();
    let mut arg_names: Vec<&'static str> = quadrule_i18n::english_sources()
        .iter()
        .flat_map(|(_, src)| src.split('$').skip(1))
        .map(|rest| {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                .unwrap_or(rest.len());
            &rest[..end]
        })
        .filter(|n| !n.is_empty())
        .collect();
    arg_names.sort();
    arg_names.dedup();
    let failed: Vec<_> = ids
        .iter()
        .filter(|id| {
            // Every argument any message uses ($name in the files), all given; Fluent ignores the
            // extra ones.
            let mut m = Message::new(Box::leak(id.to_string().into_boxed_str()));
            for name in &arg_names {
                m = m.num(name, 1.0, 0);
            }
            s.try_format(&m).is_err()
        })
        .collect();
    let ok = failed.is_empty() && ids.len() > 20;
    println!(
        "CHECK crate=i18n name=english_formats result={} messages={} arguments={} failed={failed:?}",
        pass(ok),
        ids.len(),
        arg_names.len()
    );
    assert!(ok);
}

#[test]
fn numbers_round_like_rust() {
    // Arg::Num must print the digits `format!("{:.N}")` prints, halves and negatives included.
    let s = Strings::load("en-US", &[("t.ftl", "n = { $n }")]).unwrap();
    let cases = [
        (0.5, 0),
        (1.5, 0),
        (2.5, 0),
        (-0.4, 0),
        (71.0, 0),
        (2.675, 2),
        (70.866141732, 2),
        (0.0846666, 3),
        (1.0, 2),
        (-2.71, 1),
    ];
    let wrong: Vec<_> = cases
        .iter()
        .filter_map(|&(n, d)| {
            let got = s.format(&Message::new("n").num("n", n, d));
            let want = format!("{n:.*}", d as usize);
            (got != want).then(|| format!("{n}:{d} got={got} want={want}"))
        })
        .collect();
    let ok = wrong.is_empty();
    println!(
        "CHECK crate=i18n name=numbers_round_like_rust result={} cases={} wrong={wrong:?}",
        pass(ok),
        cases.len()
    );
    assert!(ok);
}

#[test]
fn lists_nested_and_missing() {
    let s = Strings::load(
        "en-US",
        &[(
            "t.ftl",
            "a = apple\nb = pear\nsep = {\", \"}\nbasket = { $items } and { $one }",
        )],
    )
    .unwrap();
    let m = Message::new("basket")
        .arg(
            "items",
            Arg::List {
                items: vec![Message::new("a"), Message::new("b")],
                sep: "sep",
            },
        )
        .arg("one", Arg::Msg(Box::new(Message::new("a"))));
    let list = s.format(&m);
    let missing = s.format(&Message::new("nope"));
    let missing_get = s.get("nope");
    let missing_arg = s.try_format(&Message::new("basket")).is_err();
    let ok = list == "apple, pear and apple"
        && missing == "⟨nope⟩"
        && missing_get == "⟨nope⟩"
        && missing_arg;
    println!(
        "CHECK crate=i18n name=lists_and_missing result={} list={list:?} missing={missing:?} missing_arg_is_error={missing_arg}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn bad_files_are_refused() {
    // A syntax error or a message defined twice names the file instead of loading half a language.
    let syntax = Strings::load("en-US", &[("bad.ftl", "= nothing")]);
    let twice = Strings::load("en-US", &[("a.ftl", "x = 1"), ("b.ftl", "x = 2")]);
    let ok = syntax.as_ref().is_err_and(|e| e.starts_with("bad.ftl"))
        && twice.as_ref().is_err_and(|e| e.starts_with("b.ftl"));
    println!(
        "CHECK crate=i18n name=bad_files_refused result={} syntax={} twice={}",
        pass(ok),
        syntax.is_err(),
        twice.is_err()
    );
    assert!(ok);
}
