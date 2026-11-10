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
//! ```

use quadrule_render::{Math, PngMeta, encode_png, render, sha256_hex};
use quadrule_text::FontRegistry;
use quadrule_xtask::calib;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask calibration-sheet [--profile <id>|all] [--out <dir>]\n       cargo xtask safe-zones [--profile <id>|all]\n       cargo xtask lint [--toolbars <list>] [<file.quad>]";

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
                &render(&page.dl, Math::Fastest),
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

/// Toolbar positions `cargo xtask lint` checks unless told otherwise: all of them (chris,
/// 2026-11-10: "the safe area linting should look at all safe areas, including the right one";
/// on 2026-11-09 it was "Docked left, Docked top"; docs/device-measurements/supernote.a6x2/calibration.md).
const DEFAULT_TOOLBARS: &str = "all";

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
    };
    let doc = match path {
        None => quadrule_model::sample::slice_document(),
        Some(path) => {
            let bytes = std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            quadrule_format::QuadFile::from_bytes(&bytes)
                .map_err(|e| format!("{path}: {e}"))?
                .doc
        }
    };
    println!("toolbar positions checked: {toolbars}");
    let primary = quadrule_profiles::builtin(&doc.primary_profile_id).map_err(|e| e.to_string())?;
    let mut reg = FontRegistry::bundled_only();
    let strings = quadrule_i18n::Strings::english();
    let mut count = 0;
    for id in doc.profile_ids() {
        let target = quadrule_profiles::builtin(id).map_err(|e| e.to_string())?;
        let laid = quadrule_layout::lay_out_text(&doc, &primary, &target, &mut reg)
            .map_err(|e| e.to_string())?;
        for f in quadrule_lint::lint_with(&doc, &primary, &target, &laid.outlines, &opts) {
            count += 1;
            let message = strings.format(&f.message);
            println!("{}  {}  {:?}  {message}", f.profile, f.rule, f.severity);
        }
    }
    println!("{count} finding(s) on {} profile(s)", doc.profiles.len());
    Ok(())
}
