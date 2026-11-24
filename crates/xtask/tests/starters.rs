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

//! The starter pages (rung 16b; `cargo xtask starters`). The generator writes `crates/ui/starters/<category>/<slug>.quad`
//! from code. These tests say the generator is repeatable (two runs give the same bytes), that every checked-in file is
//! exactly what it writes now (so a format change that re-makes the files fails here until they are written again, and
//! none drifts), that every starter lints clean on both tablets (ADR-021, item 1: nothing at any severity, the toolbars
//! the linter checks by default), and that every starter is named, described and marked as the rules ask (hyphenated
//! slug, a sentence-case title, a description and tags, layers and objects with names of their own, the page's margins,
//! spacing and title and at least one object marked Show in Basic, every frame resolved by constraints and not by a
//! device's override). Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_format::{FORMAT_VERSION, QuadFile};
use quadrule_xtask::{lintrun, starters};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

#[test]
fn the_generator_writes_the_same_bytes_twice() {
    let a = starters::files().unwrap();
    let b = starters::files().unwrap();
    let same = a == b && !a.is_empty();
    let slugs: Vec<_> = a.iter().map(|(p, _)| p.display().to_string()).collect();
    let mut sorted = slugs.clone();
    sorted.sort();
    sorted.dedup();
    let unique = sorted.len() == slugs.len();
    println!(
        "\nCHECK crate=xtask name=starters_repeatable result={} starters={} same_bytes_twice={same} unique_paths={unique}",
        pass(same && unique),
        a.len()
    );
    assert!(same && unique);
}

#[test]
fn every_checked_in_starter_is_what_the_generator_writes() {
    let root = starters::starters_dir();
    let made = starters::files().unwrap();
    let mut differ = Vec::new();
    for (rel, bytes) in &made {
        if std::fs::read(root.join(rel)).ok().as_ref() != Some(bytes) {
            differ.push(rel.display().to_string());
        }
    }
    let stale = starters::stale_files(&root, &made);
    let ok = differ.is_empty() && stale.is_empty();
    println!(
        "\nCHECK crate=xtask name=starters_match_generator result={} starters={} differ=[{}] stale=[{}]",
        pass(ok),
        made.len(),
        differ.join(","),
        stale
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(
        ok,
        "run `cargo xtask starters` to write the starters again, then commit them"
    );
}

#[test]
fn every_starter_lints_clean_on_both_tablets() {
    let opts = lintrun::options(lintrun::DEFAULT_TOOLBARS).unwrap();
    let mut findings = Vec::new();
    let mut checked = 0;
    for (rel, bytes) in starters::files().unwrap() {
        let f = QuadFile::from_bytes(&bytes).unwrap();
        let both = f.doc.profile_ids() == ["supernote.a6x2", "supernote.a5x2"];
        let lines = lintrun::lint_lines(&f.doc, &f.outlines, &opts).unwrap();
        checked += 1;
        if !both {
            findings.push(format!("{}: not the two tablets", rel.display()));
        }
        findings.extend(lines.into_iter().map(|l| format!("{}: {l}", rel.display())));
    }
    let ok = findings.is_empty() && checked > 0;
    println!(
        "\nCHECK crate=xtask name=starters_lint_clean result={} starters={checked} tablets=2 findings={} first=[{}]",
        pass(ok),
        findings.len(),
        findings.first().cloned().unwrap_or_default()
    );
    assert!(ok, "{findings:#?}");
}

/// A title in sentence case: the first word is capitalised and no later word starts with a capital (acronyms such as
/// "RTL" and names are not in these titles).
fn sentence_case(title: &str) -> bool {
    let mut words = title.split_whitespace();
    words
        .next()
        .is_some_and(|w| w.chars().next().is_some_and(char::is_uppercase))
        && words.all(|w| !w.chars().next().is_some_and(char::is_uppercase))
}

#[test]
fn every_starter_is_named_described_marked_and_fitted_by_constraints() {
    let mut problems = Vec::new();
    let mut objects = 0;
    for (rel, bytes) in starters::files().unwrap() {
        let name = rel.display().to_string();
        let f = QuadFile::from_bytes(&bytes).unwrap();
        let d = &f.doc;
        let slug = rel.file_stem().unwrap().to_string_lossy().into_owned();
        if f.manifest.format_version != FORMAT_VERSION || f.migrated_from.is_some() {
            problems.push(format!("{name}: not the current format"));
        }
        if slug.contains('_') || slug.chars().any(char::is_uppercase) {
            problems.push(format!("{name}: slug is not hyphenated lower case"));
        }
        if !sentence_case(&d.metadata.title) {
            problems.push(format!(
                "{name}: title {:?} is not sentence case",
                d.metadata.title
            ));
        }
        if d.metadata.description.trim().is_empty() || d.metadata.tags.len() < 3 {
            problems.push(format!(
                "{name}: needs a description and at least three tags"
            ));
        }
        if d.layers.iter().any(|l| l.name.starts_with("Layer ")) {
            problems.push(format!("{name}: a layer has its default name"));
        }
        if !(d.page.basic.margins && d.page.basic.rhythm && d.page.basic.title) {
            problems.push(format!(
                "{name}: margins, rhythm and title are not all marked Show in Basic"
            ));
        }
        if !d.objects().any(|o| o.show_in_basic) {
            problems.push(format!("{name}: no object is marked Show in Basic"));
        }
        if f.previews.is_empty() {
            problems.push(format!("{name}: no preview picture"));
        }
        for o in d.objects() {
            objects += 1;
            if matches!(o.name.as_str(), "Pattern" | "Line" | "Rectangle") || o.name.is_empty() {
                problems.push(format!("{name}: object {:?} has a default name", o.name));
            }
            if !o.overrides.is_empty() {
                problems.push(format!("{name}: object {:?} has a device override", o.name));
            }
        }
    }
    let ok = problems.is_empty();
    println!(
        "\nCHECK crate=xtask name=starters_named_marked_constrained result={} objects={objects} problems={} first=[{}]",
        pass(ok),
        problems.len(),
        problems.first().cloned().unwrap_or_default()
    );
    assert!(ok, "{problems:#?}");
}
