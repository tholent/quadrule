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

//! `cargo xtask`: maintainer tools (napkin §8.4).
//!
//! ```text
//! cargo xtask calibration-sheet [--profile <id>|all] [--out <dir>]
//! cargo xtask safe-zones [--profile <id>|all]
//! cargo xtask lint [--toolbars <list>] [<file.quad>]
//! cargo xtask prune [--dry-run]
//! cargo xtask starters [--check]
//! cargo xtask bench-check [--record] [--runs <n>] [--only <text>] [--starter <file>] [--basic-starter <file>]
//! ```

use quadrule_render::{PngMeta, encode_png, render, sha256_hex};
use quadrule_text::FontRegistry;
use quadrule_xtask::{calib, lintrun};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask calibration-sheet [--profile <id>|all] [--out <dir>]\n       cargo xtask safe-zones [--profile <id>|all]\n       cargo xtask lint [--toolbars <list>] [<file.quad>]\n       cargo xtask prune [--dry-run]\n       cargo xtask starters [--check]\n       cargo xtask bench-check [--record] [--runs <n>] [--only <text>] [--starter <file>] [--basic-starter <file>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("calibration-sheet") => match calibration_sheet(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Some("lint") => match lint(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Some("prune") => match prune(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Some("starters") => match starters(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Some("bench-check") => match bench_check(&args[1..]) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(2)
            }
        },
        Some("safe-zones") => match safe_zones(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// `bench-check`: the fixed set of benches against the recorded numbers (or, with `--record`, writing them).
/// True when it passed.
fn bench_check(args: &[String]) -> Result<bool, String> {
    let mut opts = quadrule_xtask::bench::Options::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--record" => opts.record = true,
            "--runs" => {
                opts.runs = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n > 0)
                    .ok_or(USAGE)?;
            }
            "--only" => opts.only = Some(it.next().ok_or(USAGE)?.clone()),
            "--starter" => opts.starter = it.next().ok_or(USAGE)?.clone(),
            "--basic-starter" => opts.basic_starter = it.next().ok_or(USAGE)?.clone(),
            _ => return Err(format!("unknown argument {a:?}\n{USAGE}")),
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = root.canonicalize().unwrap_or(root);
    let report = quadrule_xtask::bench::run(&root, &opts)?;
    Ok(report.passed)
}

/// Writes every calibration page for the chosen profiles, plus a `codes.txt` key per profile.
fn calibration_sheet(args: &[String]) -> Result<(), String> {
    let mut profile = "all".to_string();
    let mut out = PathBuf::from("calib");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--profile" => profile = it.next().ok_or(USAGE)?.clone(),
            "--out" => out = PathBuf::from(it.next().ok_or(USAGE)?),
            _ => return Err(format!("unknown argument {a:?}\n{USAGE}")),
        }
    }
    let ids: Vec<String> = if profile == "all" {
        quadrule_profiles::builtin_ids().map(String::from).collect()
    } else {
        vec![profile]
    };
    let version = env!("CARGO_PKG_VERSION");
    let mut reg = FontRegistry::bundled_only();
    for id in ids {
        let p = quadrule_profiles::builtin(&id).map_err(|e| e.to_string())?;
        let dir = out.join(calib::slug(&p.id));
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let pages = calib::sheets(&p, &mut reg, version)?;
        let mut key = format!(
            "Calibration codes for {} ({}), quadrule {version}\n\n",
            p.display_name, p.id
        );
        for page in &pages {
            let software = format!("Quadrule {version}");
            let title = format!("Calibration {} {} of {}", page.sheet, page.number, page.of);
            let png = encode_png(
                &render(&page.dl),
                Some(PngMeta {
                    software: &software,
                    title: &title,
                }),
            );
            let path = dir.join(page.file_name(&p));
            std::fs::write(&path, &png).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("wrote {} sha256={}", path.display(), sha256_hex(&png));
            for s in &page.samples {
                key.push_str(&format!(
                    "{}\t{}\t{} page {}\n",
                    s.code, s.what, page.sheet, page.number
                ));
            }
        }
        std::fs::write(dir.join("codes.txt"), key).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Measures safe zones from `docs/device-measurements/<profile>/` and prints them as RON for
/// the profile file, with a note when they differ from what the profile holds.
fn safe_zones(args: &[String]) -> Result<(), String> {
    let profile = match args {
        [] => "all".to_string(),
        [flag, id] if flag == "--profile" => id.clone(),
        _ => return Err(USAGE.into()),
    };
    let ids: Vec<String> = if profile == "all" {
        quadrule_profiles::builtin_ids().map(String::from).collect()
    } else {
        vec![profile]
    };
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/device-measurements");
    for id in ids {
        let p = quadrule_profiles::builtin(&id).map_err(|e| e.to_string())?;
        let shots = quadrule_xtask::safezones::load_dir(&root.join(&id))?;
        let zones = quadrule_xtask::safezones::measure(&shots)?;
        println!("// {id}: measured from {} screenshots", shots.len());
        if !shots.is_empty() {
            let g = quadrule_xtask::safezones::find_grid(&shots)?;
            let (w, h) = (shots[0].width, shots[0].height);
            let last = |(o, p, _): (usize, usize, usize), n: usize| o + (n - 1 - o) / p * p;
            let (lx, ly) = (last(g.x, w), last(g.y, h));
            println!(
                "// template grid: spacing {} x {} px, line {} px; first line x {} y {}; space left/right {}/{}, top/bottom {}/{}",
                g.x.1,
                g.y.1,
                g.x.2,
                g.x.0,
                g.y.0,
                g.x.0,
                w - 1 - lx,
                g.y.0,
                h - 1 - ly
            );
        }
        println!("safe_zones: [");
        for z in &zones {
            println!(
                "    (name: {:?}, orientation: {:?}, rect: ({}, {}, {}, {}), advisory: {}),",
                z.name, z.orientation, z.rect.0, z.rect.1, z.rect.2, z.rect.3, z.advisory
            );
        }
        println!("],");
        if zones != p.safe_zones {
            println!("// differs from crates/profiles/data/{id}.ron");
        }
    }
    Ok(())
}

/// Writes every starter page to `crates/ui/starters/<category>/<slug>.quad` (`quadrule_xtask::starters`), the same
/// bytes every run, and prints one `STARTERS` line. `--check` writes nothing and fails when a file there differs.
fn starters(args: &[String]) -> Result<(), String> {
    let check = match args {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => return Err(USAGE.into()),
    };
    let root = quadrule_xtask::starters::starters_dir();
    if check {
        let files = quadrule_xtask::starters::files()?;
        let mut differ = Vec::new();
        for (rel, bytes) in &files {
            if std::fs::read(root.join(rel)).ok().as_ref() != Some(bytes) {
                differ.push(rel.display().to_string());
            }
        }
        for stale in quadrule_xtask::starters::stale_files(&root, &files) {
            differ.push(stale.display().to_string());
        }
        println!(
            "STARTERS check={} starters={} differ={}",
            if differ.is_empty() { "pass" } else { "fail" },
            files.len(),
            differ.join(",")
        );
        return if differ.is_empty() {
            Ok(())
        } else {
            Err("run `cargo xtask starters` to write them again".into())
        };
    }
    let written = quadrule_xtask::starters::write_all(&root)?;
    for p in &written {
        println!("wrote {}", p.display());
    }
    println!("STARTERS written={}", written.len());
    Ok(())
}

/// Deletes old copies of this workspace's builds from Cargo's build folder and incremental caches
/// unused for three days (`quadrule_xtask::prune`), then prints one `PRUNE` line. `--dry-run`
/// lists what would go and deletes nothing.
fn prune(args: &[String]) -> Result<(), String> {
    let dry_run = match args {
        [] => false,
        [flag] if flag == "--dry-run" => true,
        _ => return Err(USAGE.into()),
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = std::process::Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata: {e}"))?;
    let (target, names) = quadrule_xtask::prune::names_from_metadata(&json)?;
    let opts = quadrule_xtask::prune::Options {
        dry_run,
        ..Default::default()
    };
    let report = quadrule_xtask::prune::prune(&target, &names, &opts)
        .map_err(|e| format!("{}: {e}", target.display()))?;
    for name in &report.skipped_names {
        println!("libraries left alone: {name} (a third-party crate has the same name)");
    }
    if dry_run {
        for path in &report.removed {
            println!("would remove {}", path.display());
        }
    }
    let shown = json["workspace_root"]
        .as_str()
        .and_then(|r| target.strip_prefix(r).ok())
        .unwrap_or(&target);
    println!("{}", report.line(&shown.display().to_string()));
    Ok(())
}

/// Lints a `.quad` file (or the rung 1 slice document) on each of its profiles and prints one
/// line per finding: profile, rule, severity, message. `--toolbars` takes a comma-separated
/// list of `left`, `right`, `top`, `floating`, or `all`.
fn lint(args: &[String]) -> Result<(), String> {
    let mut toolbars = lintrun::DEFAULT_TOOLBARS.to_string();
    let mut path = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--toolbars" => toolbars = it.next().ok_or(USAGE)?.clone(),
            _ if path.is_none() && !a.starts_with("--") => path = Some(a.clone()),
            _ => return Err(format!("unknown argument {a:?}\n{USAGE}")),
        }
    }
    let opts = lintrun::options(&toolbars).map_err(|e| format!("{e}\n{USAGE}"))?;
    // A file brings its saved outlines, which draw the text whose font this computer lacks (L11).
    let (doc, saved) = match path {
        None => (quadrule_model::sample::slice_document(), Default::default()),
        Some(path) => {
            let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            let f = quadrule_format::QuadFile::from_bytes(&bytes)
                .map_err(|e| format!("{path}: {e}"))?;
            // The profiles the file carries stand in for any this computer lacks (rung 13a).
            quadrule_profiles::active::with_mut(|r| {
                r.set_carried(f.profiles.values().cloned().collect(), Default::default());
            });
            (f.doc, f.outlines)
        }
    };
    println!("toolbar positions checked: {toolbars}");
    let lines = lintrun::lint_lines(&doc, &saved, &opts)?;
    for l in &lines {
        println!("{l}");
    }
    println!(
        "{} finding(s) on {} profile(s)",
        lines.len(),
        doc.profiles.len()
    );
    Ok(())
}
