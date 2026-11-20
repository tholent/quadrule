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
//! ```

use quadrule_render::{PngMeta, encode_png, render, sha256_hex};
use quadrule_text::FontRegistry;
use quadrule_xtask::calib;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask calibration-sheet [--profile <id>|all] [--out <dir>]\n       cargo xtask safe-zones [--profile <id>|all]\n       cargo xtask lint [--toolbars <list>] [<file.quad>]\n       cargo xtask prune [--dry-run]";

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

/// Toolbar positions `cargo xtask lint` checks unless told otherwise: the docked ones (chris,
/// 2026-11-10: "the safe area linting should look at all safe areas, including the right one", and
/// after rung 5 that checking the floating menu bar "isn't desired"; on 2026-11-09 it was "Docked
/// left, Docked top"; docs/device-measurements/supernote.a6x2/calibration.md). `--toolbars all` adds
/// the floating one.
const DEFAULT_TOOLBARS: &str = "left,right,top";

/// Lints a `.quad` file (or the rung 1 slice document) on each of its profiles and prints one
/// line per finding: profile, rule, severity, message. `--toolbars` takes a comma-separated
/// list of `left`, `right`, `top`, `floating`, or `all`.
fn lint(args: &[String]) -> Result<(), String> {
    let mut toolbars = DEFAULT_TOOLBARS.to_string();
    let mut path = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--toolbars" => toolbars = it.next().ok_or(USAGE)?.clone(),
            _ if path.is_none() && !a.starts_with("--") => path = Some(a.clone()),
            _ => return Err(format!("unknown argument {a:?}\n{USAGE}")),
        }
    }
    let opts = quadrule_lint::Options {
        toolbars: if toolbars == "all" {
            quadrule_lint::Toolbar::ALL.to_vec()
        } else {
            toolbars
                .split(',')
                .map(|s| {
                    quadrule_lint::Toolbar::parse(s.trim())
                        .ok_or_else(|| format!("unknown toolbar position {s:?}\n{USAGE}"))
                })
                .collect::<Result<_, _>>()?
        },
        ..quadrule_lint::Options::default()
    };
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
    let primary = quadrule_profiles::active::lookup_in(&doc, &doc.primary_profile_id)
        .map_err(|e| e.to_string())?;
    let mut reg = FontRegistry::bundled_only();
    let strings = quadrule_i18n::Strings::english();
    let mut count = 0;
    for id in doc.profile_ids() {
        let target = quadrule_profiles::active::lookup_in(&doc, id).map_err(|e| e.to_string())?;
        let (laid, how) =
            quadrule_layout::lay_out_text_saved(&doc, &primary, &target, &mut reg, &saved)
                .map_err(|e| e.to_string())?;
        let mut opts = opts.clone();
        for id in &laid.unavailable {
            if let Some(quadrule_model::ObjectKind::Text(t)) = doc.object(*id).map(|o| &o.kind) {
                opts.fonts.missing.push(quadrule_lint::MissingBox {
                    id: *id,
                    family: t.family.clone(),
                    needs_font: how.needs_font.contains(id),
                });
            }
        }
        for f in quadrule_lint::lint_with(&doc, &primary, &target, &laid.outlines, &opts) {
            count += 1;
            let message = strings.format(&f.message);
            println!("{}  {}  {:?}  {message}", f.profile, f.rule, f.severity);
        }
    }
    println!("{count} finding(s) on {} profile(s)", doc.profiles.len());
    Ok(())
}
