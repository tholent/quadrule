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

//! `cargo xtask prune` checks, on a made-up build folder under `target/tmp` (never the real one).
//! Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_xtask::prune::{Names, Options, Report, prune, split};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const DAY: Duration = Duration::from_secs(24 * 60 * 60);
const H4: &str = "4444444444444444";
const H5: &str = "5555555555555555";
const H6: &str = "6666666666666666";
/// Five builds of one library, oldest first (prune keeps four).
const L: [&str; 5] = [
    "a000000000000000",
    "a100000000000000",
    "a200000000000000",
    "a300000000000000",
    "a400000000000000",
];
/// Four checks of the same library (rmeta only), all newer than its check with its tests.
const C: [&str; 4] = [
    "c000000000000000",
    "c100000000000000",
    "c200000000000000",
    "c300000000000000",
];
/// The third-party `fonts` library's two builds: never the hash of one of ours.
const T: [&str; 2] = ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb"];

/// The `i`th build's hash for one crate (`tag`): every build Cargo makes has its own hash, and
/// the records are found by it.
fn h(tag: char, i: usize) -> String {
    format!("{tag}{i:015x}")
}

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// An empty made-up build folder for one test.
fn target(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("prune-{name}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("debug/deps")).unwrap();
    std::fs::create_dir_all(d.join("debug/incremental")).unwrap();
    d
}

/// Writes `bytes` bytes to `path`, last changed `age` ago.
fn file(path: &Path, bytes: usize, age: Duration) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, vec![0u8; bytes]).unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(SystemTime::now() - age)
        .unwrap();
}

/// `groups` and `quadrule_model` are ours; `syn` and `fonts` are third-party, and `fonts` is
/// also one of our test files.
fn names() -> Names {
    let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect();
    Names {
        ours: set(&["groups", "quadrule_model", "fonts", "xtask"]),
        theirs: set(&["syn", "fonts"]),
    }
}

/// Cargo's record of one build: `.fingerprint/<package>-<hash>/<what>.json`.
fn record(t: &Path, package: &str, hash: &str, what: &str) {
    let d = t.join(format!("debug/.fingerprint/{package}-{hash}"));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join(format!("{what}.json")), "{}").unwrap();
    std::fs::write(d.join(format!("dep-{what}")), "").unwrap();
    std::fs::write(d.join("invoked.timestamp"), "").unwrap();
}

fn exists(t: &Path, rel: &str) -> bool {
    t.join(rel).exists()
}

/// The made-up folder every test but the incremental one uses: three builds each of two test
/// programs and a third-party library, five of a built library (prune keeps two programs and
/// four libraries), and builds that only look like old copies (another crate's `groups` test, a
/// library checked with its tests, a build Cargo has no record of).
fn deps_folder(name: &str) -> PathBuf {
    let t = target(name);
    let d = t.join("debug/deps");
    for (i, age) in [3 * DAY, 2 * DAY, DAY].into_iter().enumerate() {
        let (g, s, f) = (h('1', i), h('5', i), h('f', i));
        file(&d.join(format!("groups-{g}")), 1000, age);
        file(&d.join(format!("groups-{g}.d")), 10, age);
        record(&t, "quadrule-layout", &g, "test-integration-test-groups");
        file(&d.join(format!("libsyn-{s}.rlib")), 100, age);
        file(&d.join(format!("libsyn-{s}.rmeta")), 100, age);
        file(&d.join(format!("syn-{s}.d")), 10, age);
        record(&t, "syn", &s, "lib-syn");
        file(&d.join(format!("fonts-{f}")), 100, age);
        record(&t, "quadrule-format", &f, "test-integration-test-fonts");
    }
    for (hash, age) in [(T[0], 2 * DAY), (T[1], DAY)] {
        file(&d.join(format!("libfonts-{hash}.rlib")), 100, age);
        record(&t, "fonts", hash, "lib-fonts");
    }
    // A built library (rlib and rmeta) five times, and checked (rmeta only, from clippy) four
    // times: the checks are not old copies of the built one.
    for (i, hash) in L.iter().enumerate() {
        let age = (8 - i as u32) * DAY;
        file(&d.join(format!("libquadrule_model-{hash}.rlib")), 500, age);
        file(&d.join(format!("libquadrule_model-{hash}.rmeta")), 50, age);
        file(&d.join(format!("quadrule_model-{hash}.d")), 10, age);
        record(&t, "quadrule-model", hash, "lib-quadrule_model");
        file(&d.join(format!("xtask-{}", h('e', i))), 200, age);
        record(&t, "quadrule-xtask", &h('e', i), "bin-xtask");
    }
    for (i, hash) in C.iter().enumerate() {
        let age = (4 - i as u32) * DAY;
        file(&d.join(format!("libquadrule_model-{hash}.rmeta")), 50, age);
        file(&d.join(format!("quadrule_model-{hash}.d")), 10, age);
        record(&t, "quadrule-model", hash, "lib-quadrule_model");
    }
    // Older than everything above, and none is a copy of anything: render's own `groups` test,
    // the library checked with its tests (clippy --all-targets), a `groups` with no record.
    file(&d.join(format!("groups-{H4}")), 1000, 5 * DAY);
    record(&t, "quadrule-render", H4, "test-integration-test-groups");
    file(
        &d.join(format!("libquadrule_model-{H5}.rmeta")),
        50,
        5 * DAY,
    );
    record(&t, "quadrule-model", H5, "test-lib-quadrule_model");
    file(&d.join(format!("groups-{H6}")), 1000, 6 * DAY);
    t
}

#[test]
fn split_reads_cargo_file_names() {
    let cases = [
        ("groups-0123456789abcdef", Some(("groups", Some("program")))),
        (
            "groups-0123456789abcdef.exe",
            Some(("groups", Some("program"))),
        ),
        ("groups-0123456789abcdef.d", Some(("groups", None))),
        (
            "libquadrule_model-0123456789abcdef.rlib",
            Some(("quadrule_model", Some("rlib"))),
        ),
        (
            "libquadrule_model-0123456789abcdef.rmeta",
            Some(("quadrule_model", Some("rmeta"))),
        ),
        (
            "libserde_derive-0123456789abcdef.so",
            Some(("serde_derive", Some("dylib"))),
        ),
        (
            "serde_derive-0123456789abcdef.dll",
            Some(("serde_derive", Some("dylib"))),
        ),
        ("groups-0123456789abcdef.pdb", Some(("groups", None))),
        ("groups-0123", None),
        ("groups", None),
    ];
    let fails: Vec<_> = cases
        .iter()
        .filter(|(name, want)| {
            let got = split(name);
            let got = got.as_ref().map(|(n, _, k)| (n.as_str(), *k));
            got != *want
        })
        .map(|(name, _)| *name)
        .collect();
    println!(
        "CHECK crate=xtask name=prune_split result={} cases={} fails={fails:?}",
        pass(fails.is_empty()),
        cases.len()
    );
    assert!(fails.is_empty());
}

#[test]
fn keeps_the_newest_copy_of_ours() {
    let t = deps_folder("newest");
    prune(&t, &names(), &Options::default()).unwrap();
    let old_gone = !exists(&t, &format!("debug/deps/groups-{}", h('1', 0)))
        && !exists(&t, &format!("debug/deps/groups-{}.d", h('1', 0)))
        && !exists(&t, &format!("debug/deps/libquadrule_model-{}.rlib", L[0]))
        && !exists(&t, &format!("debug/deps/quadrule_model-{}.d", L[0]));
    let new_kept = (1..3).all(|i| {
        exists(&t, &format!("debug/deps/groups-{}", h('1', i)))
            && exists(&t, &format!("debug/deps/groups-{}.d", h('1', i)))
    }) && L[1..]
        .iter()
        .all(|h| exists(&t, &format!("debug/deps/libquadrule_model-{h}.rlib")));
    let checked_kept = C
        .iter()
        .all(|h| exists(&t, &format!("debug/deps/libquadrule_model-{h}.rmeta")));
    let other_crate_kept = exists(&t, &format!("debug/deps/groups-{H4}"));
    let test_check_kept = exists(&t, &format!("debug/deps/libquadrule_model-{H5}.rmeta"));
    let unrecorded_kept = exists(&t, &format!("debug/deps/groups-{H6}"));
    // The xtask program itself, not a test, keeps four like a library.
    let program_four = !exists(&t, &format!("debug/deps/xtask-{}", h('e', 0)))
        && (1..5).all(|i| exists(&t, &format!("debug/deps/xtask-{}", h('e', i))));
    let ok = old_gone
        && new_kept
        && checked_kept
        && other_crate_kept
        && test_check_kept
        && unrecorded_kept
        && program_four;
    println!(
        "CHECK crate=xtask name=prune_keeps_newest result={} old_gone={old_gone} new_kept={new_kept} checked_kept={checked_kept} other_crate_kept={other_crate_kept} test_check_kept={test_check_kept} unrecorded_kept={unrecorded_kept} program_four={program_four}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn leaves_third_party_crates_alone() {
    let t = deps_folder("theirs");
    let report = prune(&t, &names(), &Options::default()).unwrap();
    let both = |name: &str| (0..3).all(|i| exists(&t, &name.replace("{h}", &h('5', i))));
    let syn = both("debug/deps/libsyn-{h}.rlib") && both("debug/deps/syn-{h}.d");
    // `fonts` is a test file of ours and a third-party crate: the libraries stay, our old test
    // program goes.
    let shared = T
        .iter()
        .all(|h| exists(&t, &format!("debug/deps/libfonts-{h}.rlib")));
    let program = !exists(&t, &format!("debug/deps/fonts-{}", h('f', 0)))
        && (1..3).all(|i| exists(&t, &format!("debug/deps/fonts-{}", h('f', i))));
    let skipped = report.skipped_names == ["fonts"];
    let ok = syn && shared && program && skipped;
    println!(
        "CHECK crate=xtask name=prune_leaves_theirs result={} syn_both={syn} shared_name_libs_both={shared} shared_name_old_program_gone={program} skipped={:?}",
        pass(ok),
        report.skipped_names
    );
    assert!(ok);
}

#[test]
fn removes_only_stale_incremental_caches() {
    let t = target("incremental");
    let inc = t.join("debug/incremental");
    file(&inc.join("groups-old/s-abc/query-cache.bin"), 100, 4 * DAY);
    file(&inc.join("groups-old/s-abc.lock"), 0, 4 * DAY);
    file(&inc.join("groups-new/s-def/query-cache.bin"), 100, DAY);
    // Old work beside a fresh session is still in use.
    file(&inc.join("groups-mixed/s-old/dep-graph.bin"), 100, 10 * DAY);
    file(&inc.join("groups-mixed/s-new/dep-graph.bin"), 100, DAY / 2);
    let report = prune(&t, &names(), &Options::default()).unwrap();
    let old_gone = !inc.join("groups-old").exists();
    let new_kept = inc.join("groups-new").exists() && inc.join("groups-mixed").exists();
    let ok = old_gone && new_kept && report.removed_files == 2 && report.removed_bytes == 100;
    println!(
        "CHECK crate=xtask name=prune_incremental result={} old_gone={old_gone} new_kept={new_kept} removed_files={}",
        pass(ok),
        report.removed_files
    );
    assert!(ok);
}

/// Every file under `dir`, relative to it.
fn listing(dir: &Path) -> Vec<PathBuf> {
    fn go(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                go(base, &p, out);
            } else {
                out.push(p.strip_prefix(base).unwrap().to_path_buf());
            }
        }
    }
    let mut out = Vec::new();
    go(dir, dir, &mut out);
    out.sort();
    out
}

#[test]
fn dry_run_deletes_nothing() {
    let t = deps_folder("dry");
    file(
        &t.join("debug/incremental/groups-old/s-abc/x.bin"),
        100,
        4 * DAY,
    );
    let before = listing(&t);
    let opts = Options {
        dry_run: true,
        ..Options::default()
    };
    let dry = prune(&t, &names(), &opts).unwrap();
    let after = listing(&t);
    let real = prune(&t, &names(), &Options::default()).unwrap();
    let unchanged = before == after;
    let same_plan = dry.removed == real.removed && dry.removed_files == real.removed_files;
    let ok =
        unchanged && same_plan && dry.removed_files > 0 && dry.line("t").ends_with("dry_run=yes");
    println!(
        "CHECK crate=xtask name=prune_dry_run result={} unchanged={unchanged} same_plan_as_real={same_plan} would_remove={}",
        pass(ok),
        dry.removed_files
    );
    assert!(ok);
}

#[test]
fn prune_line_counts_match() {
    let t = deps_folder("counts");
    let before = listing(&t);
    let report: Report = prune(&t, &names(), &Options::default()).unwrap();
    let after = listing(&t);
    let gone = before.len() - after.len();
    let kept_bytes = quadrule_xtask::prune::size(&t).unwrap();
    // Gone: the oldest of three of groups (1000 + 10) and of fonts' test program (100), and the
    // oldest of five of quadrule_model's built library (500 + 50 + 10) and of the xtask
    // program (200).
    let ok = report.removed_files == gone as u64
        && gone == 7
        && report.removed_bytes == 1870
        && report.kept_bytes == kept_bytes;
    let line = report.line("target");
    let line_ok =
        line == "PRUNE target=target removed_files=7 removed_gb=0.0 kept_gb=0.0 dry_run=no";
    println!(
        "CHECK crate=xtask name=prune_counts result={} removed_files={} files_gone={gone} removed_bytes={} line_ok={line_ok}",
        pass(ok && line_ok),
        report.removed_files,
        report.removed_bytes
    );
    assert!(ok && line_ok, "{line}");
}
