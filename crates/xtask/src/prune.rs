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

//! `cargo xtask prune`: deletes old copies from Cargo's build folder, which otherwise only grows
//! (on the Pi it reached 411 GB; docs/proposals/build-pruning-2026-11-19.md).
//!
//! It deletes two things and nothing else:
//! 1. In each `<profile>/deps`, the older builds of this workspace's own crates, tests and
//!    programs. A build is the files sharing one `<name>-<hash>`. Cargo's own record of it,
//!    `<profile>/.fingerprint/<package>-<hash>/`, says which package built it and what it built
//!    (`lib-quadrule_model`, `test-lib-quadrule_model`, `test-integration-test-groups`): five
//!    crates have a `groups` test, and clippy checks a library both as itself and with its tests.
//!    For each name, package, record and kind of file (a program, a checked library, a built
//!    library) the newest [`KEEP_TEST_PROGRAMS`] of a test program, or [`KEEP_OTHERS`] of
//!    anything else, are kept. A
//!    build with no record is left alone. Third-party crates are
//!    never touched: two versions of one library are both in use. Where one of our test files
//!    shares a third-party crate's name (`window`), only its programs are pruned.
//! 2. In each `<profile>/incremental`, folders nothing inside has changed for [`MAX_AGE_DAYS`].
//!
//! Deleting a file that was still needed costs a rebuild, never a wrong result: Cargo checks
//! what is there before using it.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How many builds of each of our test programs are kept. `cargo test` on the default members
/// and on the whole workspace build them with different library features, which looks the
/// same as an old version in Cargo's records; keeping one made every switch between `make
/// check` and `make check-all` a rebuild (chris, 2026-11-19: "Keep newest 2"). Test programs
/// are most of the build folder, and only those two commands make them.
pub const KEEP_TEST_PROGRAMS: usize = 2;

/// How many builds of each of our libraries, and of our programs `quadrule` and `xtask`, are
/// kept. `cargo build` (`make build-proof`, `make run`, `cargo xtask`) builds them with other
/// features again, four ways in all; losing a library re-links every test program on it, and
/// all four sets of our libraries are about 0.4 GB, the two programs about 1.1 GB more (chris,
/// 2026-11-19: "Libraries 4, programs 2", then "Keep 4 for those two").
pub const KEEP_OTHERS: usize = 4;

/// How long an incremental cache may sit unused before it goes (the plan's open question 2).
pub const MAX_AGE_DAYS: u64 = 3;

/// Crate names as they appear in file names (`-` written `_`).
#[derive(Debug, Default, Clone)]
pub struct Names {
    /// This workspace's targets: libraries, programs, test files, benches, examples.
    pub ours: BTreeSet<String>,
    /// Every target of every other package in the lock file.
    pub theirs: BTreeSet<String>,
}

/// What to prune and when.
#[derive(Debug, Clone)]
pub struct Options {
    /// Print and count, delete nothing.
    pub dry_run: bool,
    /// An incremental folder older than this goes.
    pub max_age: Duration,
    /// The time ages are measured from.
    pub now: SystemTime,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            dry_run: false,
            max_age: Duration::from_secs(MAX_AGE_DAYS * 24 * 60 * 60),
            now: SystemTime::now(),
        }
    }
}

/// What a prune did (or, with `dry_run`, would do).
#[derive(Debug, Default, Clone)]
pub struct Report {
    /// Files and folders deleted, in the order they went.
    pub removed: Vec<PathBuf>,
    /// Files inside them, counted one by one.
    pub removed_files: u64,
    /// Their size in bytes.
    pub removed_bytes: u64,
    /// The build folder's size afterwards, each hard-linked file counted once.
    pub kept_bytes: u64,
    /// Our names a third-party crate shares: their old programs go, their library files stay.
    pub skipped_names: Vec<String>,
    /// Whether anything was deleted.
    pub dry_run: bool,
}

impl Report {
    /// The one structured line the tool ends with.
    pub fn line(&self, target: &str) -> String {
        format!(
            "PRUNE target={target} removed_files={} removed_gb={:.1} kept_gb={:.1} dry_run={}",
            self.removed_files,
            gib(self.removed_bytes),
            gib(self.kept_bytes),
            if self.dry_run { "yes" } else { "no" }
        )
    }
}

/// Bytes as GiB, the unit `du -h` and `cargo clean` report in.
pub fn gib(bytes: u64) -> f64 {
    bytes as f64 / (1u64 << 30) as f64
}

/// Prunes the build folder `target` (see the module comment).
pub fn prune(target: &Path, names: &Names, opts: &Options) -> io::Result<Report> {
    let mut report = Report {
        dry_run: opts.dry_run,
        skipped_names: names.ours.intersection(&names.theirs).cloned().collect(),
        ..Report::default()
    };
    for profile in profile_dirs(target)? {
        let deps = profile.join("deps");
        if deps.is_dir() {
            for path in old_builds(&deps, &fingerprints(&profile)?, names)? {
                remove(&path, opts.dry_run, &mut report)?;
            }
        }
        let incremental = profile.join("incremental");
        if incremental.is_dir() {
            for path in stale_caches(&incremental, opts)? {
                remove(&path, opts.dry_run, &mut report)?;
            }
        }
    }
    let total = size(target)?;
    report.kept_bytes = if opts.dry_run {
        total.saturating_sub(report.removed_bytes)
    } else {
        total
    };
    Ok(report)
}

/// `target/<profile>` and `target/<triple>/<profile>`: every folder holding `deps` or
/// `incremental`. `target/tmp` (test output) is never one.
fn profile_dirs(target: &Path) -> io::Result<Vec<PathBuf>> {
    let is_profile = |p: &Path| p.join("deps").is_dir() || p.join("incremental").is_dir();
    let mut out = Vec::new();
    for top in sorted_dirs(target)? {
        if top.file_name().is_some_and(|n| n == "tmp") {
            continue;
        }
        if is_profile(&top) {
            out.push(top);
        } else {
            out.extend(sorted_dirs(&top)?.into_iter().filter(|p| is_profile(p)));
        }
    }
    Ok(out)
}

fn sorted_dirs(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect();
    out.sort();
    Ok(out)
}

/// What a file in `deps` is, by its extension. `None` is a file that goes with a build but does
/// not say what kind it is (`.d`, `.pdb`, `.dSYM`, …).
fn kind(ext: &str) -> Option<&'static str> {
    match ext {
        "" | "exe" => Some("program"),
        "rlib" => Some("rlib"),
        "rmeta" => Some("rmeta"),
        "so" | "dylib" | "dll" => Some("dylib"),
        _ => None,
    }
}

/// Splits a file name in `deps` into crate name, hash and kind: `libquadrule_model-<hash>.rlib`
/// is (`quadrule_model`, hash, `rlib`), `groups-<hash>` is (`groups`, hash, `program`).
pub fn split(file_name: &str) -> Option<(String, String, Option<&'static str>)> {
    let (stem, ext) = file_name.split_once('.').unwrap_or((file_name, ""));
    let (name, hash) = stem.rsplit_once('-')?;
    if hash.len() != 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let kind = kind(ext);
    // Libraries carry a `lib` prefix everywhere but a Windows `.dll`.
    let name = match kind {
        Some("rlib" | "rmeta" | "dylib") if ext != "dll" => {
            name.strip_prefix("lib").unwrap_or(name)
        }
        _ => name,
    };
    Some((name.to_string(), hash.to_string(), kind))
}

/// One build of one crate: every file in `deps` named `<name>-<hash>`.
#[derive(Default)]
struct Build {
    files: Vec<PathBuf>,
    kinds: BTreeSet<&'static str>,
    newest: Option<SystemTime>,
}

/// A build with its newest time and its hash, for sorting.
type Dated = (SystemTime, String, Build);

/// What Cargo's records in `<profile>/.fingerprint` say each build is, by hash: its package and
/// the names of its records (`quadrule-ui test-integration-test-groups`).
fn fingerprints(profile: &Path) -> io::Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let dir = profile.join(".fingerprint");
    if !dir.is_dir() {
        return Ok(out);
    }
    for d in sorted_dirs(&dir)? {
        let Some((package, hash)) = d.file_name().and_then(|n| n.to_str()?.rsplit_once('-')) else {
            continue;
        };
        let mut records: Vec<String> = std::fs::read_dir(&d)?
            .flatten()
            .filter_map(|e| Some(e.file_name().to_str()?.strip_suffix(".json")?.to_string()))
            .collect();
        records.sort();
        out.insert(hash.to_string(), format!("{package} {}", records.join(" ")));
    }
    Ok(out)
}

/// The files of every build of ours in `deps` that newer builds of the same name, record and
/// kinds replace ([`KEEP_TEST_PROGRAMS`], [`KEEP_OTHERS`]).
fn old_builds(
    deps: &Path,
    records: &BTreeMap<String, String>,
    names: &Names,
) -> io::Result<Vec<PathBuf>> {
    let mut builds: BTreeMap<(String, String), Build> = BTreeMap::new();
    for e in std::fs::read_dir(deps)?.flatten() {
        let file_name = e.file_name();
        let Some((name, hash, kind)) = split(&file_name.to_string_lossy()) else {
            continue;
        };
        if !names.ours.contains(&name) {
            continue;
        }
        let b = builds.entry((name, hash)).or_default();
        b.files.push(e.path());
        if let Some(k) = kind {
            b.kinds.insert(k);
            let modified = e.metadata()?.modified()?;
            b.newest = b.newest.max(Some(modified));
        }
    }
    // Newest first within each name, record and set of kinds; the first few stay.
    let mut groups: BTreeMap<(String, String, Vec<&'static str>), Vec<Dated>> = BTreeMap::new();
    for ((name, hash), b) in builds {
        // A lone `.d` with nothing beside it says nothing of what it belonged to; leave it.
        let Some(newest) = b.newest else { continue };
        // A name a third-party crate shares: only our programs, since a third-party crate leaves
        // only libraries in `deps`.
        if names.theirs.contains(&name) && b.kinds.iter().ne(["program"].iter()) {
            continue;
        }
        // Without Cargo's record there is no telling what it is; leave it.
        let Some(record) = records.get(&hash) else {
            continue;
        };
        let kinds = b.kinds.iter().copied().collect();
        groups
            .entry((name, record.clone(), kinds))
            .or_default()
            .push((newest, hash, b));
    }
    let mut out = Vec::new();
    for ((_, record, kinds), mut group) in groups {
        // A test program's record is `test-…` (`test-integration-test-groups`, `test-lib-…`,
        // `test-bin-xtask`); a program's own is `bin-…`.
        let test = record.split(' ').skip(1).any(|r| r.starts_with("test-"));
        let keep = if test && kinds.contains(&"program") {
            KEEP_TEST_PROGRAMS
        } else {
            KEEP_OTHERS
        };
        group.sort_by(|a, b| (b.0, &b.1).cmp(&(a.0, &a.1)));
        for (_, _, b) in group.into_iter().skip(keep) {
            out.extend(b.files);
        }
    }
    out.sort();
    Ok(out)
}

/// Folders in `incremental` with nothing inside changed for `opts.max_age`. A folder's age is
/// its newest file's; an empty one goes by its own time.
fn stale_caches(incremental: &Path, opts: &Options) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for dir in sorted_dirs(incremental)? {
        let mut newest = None;
        walk(&dir, &mut |_, m| {
            if m.is_file() {
                newest = newest.max(m.modified().ok());
            }
        })?;
        let newest = match newest {
            Some(t) => t,
            None => std::fs::metadata(&dir)?.modified()?,
        };
        let age = opts.now.duration_since(newest).unwrap_or_default();
        if age > opts.max_age {
            out.push(dir);
        }
    }
    Ok(out)
}

/// Counts `path` (a file or a folder) into the report, then deletes it unless `dry_run`.
fn remove(path: &Path, dry_run: bool, report: &mut Report) -> io::Result<()> {
    walk(path, &mut |_, m| {
        if !m.is_dir() {
            report.removed_files += 1;
            report.removed_bytes += m.len();
        }
    })?;
    if !dry_run {
        if std::fs::symlink_metadata(path)?.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    report.removed.push(path.to_path_buf());
    Ok(())
}

/// Calls `f` on `path` and everything under it, never following a link.
fn walk(path: &Path, f: &mut dyn FnMut(&Path, &std::fs::Metadata)) -> io::Result<()> {
    let m = std::fs::symlink_metadata(path)?;
    f(path, &m);
    if m.is_dir() {
        for e in std::fs::read_dir(path)?.flatten() {
            walk(&e.path(), f)?;
        }
    }
    Ok(())
}

/// The size of everything under `path`, each hard-linked file counted once (as `du` does).
pub fn size(path: &Path) -> io::Result<u64> {
    let mut seen = HashSet::new();
    let mut total = 0;
    walk(path, &mut |_, m| {
        if m.is_file() && link_id(m).is_none_or(|id| seen.insert(id)) {
            total += m.len();
        }
    })?;
    Ok(total)
}

/// Which file a hard link points at, when it has more than one name.
#[cfg(unix)]
fn link_id(m: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    (m.nlink() > 1).then(|| (m.dev(), m.ino()))
}

#[cfg(not(unix))]
fn link_id(_: &std::fs::Metadata) -> Option<(u64, u64)> {
    None
}

/// Reads this workspace's names, and its build folder, from `cargo metadata` output.
pub fn names_from_metadata(json: &serde_json::Value) -> Result<(PathBuf, Names), String> {
    let members: BTreeSet<&str> = json["workspace_members"]
        .as_array()
        .ok_or("cargo metadata: no workspace_members")?
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    let mut names = Names::default();
    for p in json["packages"]
        .as_array()
        .ok_or("cargo metadata: no packages")?
    {
        let ours = p["id"].as_str().is_some_and(|id| members.contains(id));
        for t in p["targets"].as_array().into_iter().flatten() {
            // Build scripts live in `<profile>/build`, which this does not touch.
            let build_script = t["kind"]
                .as_array()
                .is_some_and(|k| k.iter().any(|k| k == "custom-build"));
            let Some(name) = t["name"].as_str().filter(|_| !build_script) else {
                continue;
            };
            let name = name.replace('-', "_");
            if ours {
                names.ours.insert(name);
            } else {
                names.theirs.insert(name);
            }
        }
    }
    let target = json["target_directory"]
        .as_str()
        .ok_or("cargo metadata: no target_directory")?;
    Ok((PathBuf::from(target), names))
}
