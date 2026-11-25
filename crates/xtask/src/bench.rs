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

//! The speed reference (rung 17, Q2; ADR-009's amendment of rung 17): `cargo xtask bench-check` runs a fixed set of
//! the app's own benches in a release build, takes each one's 90th percentile frame time, and fails when any is over
//! 50 ms or more than 20 % slower than the number recorded for this computer in `docs/bench/<machine>.txt`.
//! `--record` writes that file. Everything here but [`run`] is plain functions on text, tested without a window.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The line a bench's 90th percentile must stay under, in milliseconds (ADR-009).
pub const LIMIT_MS: f64 = 50.0;
/// How much slower than its recorded number a bench may be before the check fails (Q2: 20 %).
pub const SLOWER_FRACTION: f64 = 0.20;
/// A bench is slower only when it is also this many milliseconds over its recorded number, so that a 1.7 ms bench
/// cannot fail on 0.4 ms of noise (the builder's addition to Q2).
pub const NOISE_MS: f64 = 1.0;
/// Frames each bench replays.
pub const FRAMES: u32 = 150;
/// How long one run of the app may take before the check gives up on it.
pub const RUN_LIMIT: Duration = Duration::from_secs(180);

/// One bench of the fixed set: a name for the record and the app's arguments.
#[derive(Clone, Debug, PartialEq)]
pub struct Bench {
    /// The name on its line and in the recorded file.
    pub name: String,
    /// The arguments to the app, after `--bench-...`'s own.
    pub args: Vec<String>,
    /// The profile it must run on, when the bench names one (the line fails if the page lacks it).
    pub profile: Option<&'static str>,
    /// The canvas it must run on: `export` or `chip`.
    pub canvas: &'static str,
}

/// The tablets the starters are timed on: short name for `--device`, and the profile id it must give.
pub const TABLETS: [(&str, &str); 2] = [("nomad", "supernote.a6x2"), ("manta", "supernote.a5x2")];

/// The two canvases, as `--canvas` names them.
pub const CANVASES: [&str; 2] = ["export", "chip"];

/// The fixed set. `starter` is the heaviest starter (a path under the repository); `basic_starter` the one the Basic
/// tab is timed over (it needs objects marked Show in Basic).
pub fn benches(starter: &str, basic_starter: &str) -> Vec<Bench> {
    let mut out = Vec::new();
    let n = FRAMES.to_string();
    let mk = |name: String, kind: &str, extra: &[&str], profile, canvas: &'static str| {
        let mut args = vec![format!("--bench-{kind}"), n.clone()];
        args.extend(extra.iter().map(|s| (*s).to_string()));
        args.extend(["--canvas".to_string(), canvas.to_string()]);
        Bench {
            name,
            args,
            profile,
            canvas,
        }
    };
    // Drag and draw on the heaviest starter, on both tablets and both canvases.
    for (short, id) in TABLETS {
        for canvas in CANVASES {
            for kind in ["drag", "draw"] {
                out.push(mk(
                    format!("{kind}-starter-{short}-{canvas}"),
                    kind,
                    &["--device", short, starter],
                    Some(id),
                    canvas,
                ));
            }
        }
    }
    // The paper page: A4 at 300 PPI first, on both canvases.
    for canvas in CANVASES {
        for kind in ["drag", "draw"] {
            out.push(mk(
                format!("{kind}-paper-a4-300-{canvas}"),
                kind,
                &["--paper-page"],
                Some("paper.a4"),
                canvas,
            ));
        }
    }
    // The biggest pages the paper list makes: A3 and Tabloid at 600 PPI, on both canvases.
    for canvas in CANVASES {
        out.push(mk(
            format!("drag-paper-a3-600-{canvas}"),
            "drag",
            &["--large-paper-page"],
            Some("paper.a3"),
            canvas,
        ));
        out.push(mk(
            format!("draw-paper-a3-600-{canvas}"),
            "draw",
            &["--large-paper-page"],
            Some("paper.a3"),
            canvas,
        ));
        out.push(mk(
            format!("drag-paper-tabloid-600-{canvas}"),
            "drag",
            &["--large-paper-page", "--device", "paper.tabloid"],
            Some("paper.tabloid"),
            canvas,
        ));
    }
    // Typing into a long text box, and zooming like a pinch, on both canvases.
    for canvas in CANVASES {
        out.push(mk(
            format!("type-sample-{canvas}"),
            "type",
            &[],
            None,
            canvas,
        ));
        out.push(mk(
            format!("zoom-sample-{canvas}"),
            "zoom",
            &[],
            None,
            canvas,
        ));
        out.push(mk(
            format!("zoom-groups-{canvas}"),
            "zoom",
            &["--groups-page"],
            None,
            canvas,
        ));
    }
    // The Basic tab: a drag with the Basic tab open, timing the whole window frame (the panels as well as the canvas).
    for (short, id) in TABLETS {
        out.push(mk(
            format!("basic-tab-starter-{short}-export"),
            "drag",
            &["--basic-tab", "--device", short, basic_starter],
            Some(id),
            "export",
        ));
    }
    out
}

/// What a bench printed: its `CHECK` line's `key=value` words.
pub type Fields = BTreeMap<String, String>;

/// The `key=value` words of a bench's `CHECK` line, or `None` when `line` is not one that carries `p90_ms`.
pub fn parse_check(line: &str) -> Option<Fields> {
    let line = line.trim();
    if !line.starts_with("CHECK crate=") {
        return None;
    }
    let fields: Fields = line
        .split_whitespace()
        .filter_map(|w| w.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    fields.contains_key("p90_ms").then_some(fields)
}

/// How one bench came out.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    /// Under the line, and not slower than recorded (or nothing recorded for this computer).
    Ok,
    /// Over [`LIMIT_MS`].
    Over,
    /// More than [`SLOWER_FRACTION`] over the recorded number.
    Slower,
    /// This computer has recorded numbers, but not for this bench.
    Unrecorded,
    /// It did not run to a number: why.
    Failed(String),
}

impl Status {
    /// The word on the line.
    pub fn word(&self) -> String {
        match self {
            Status::Ok => "ok".into(),
            Status::Over => "over_50ms".into(),
            Status::Slower => "slower".into(),
            Status::Unrecorded => "unrecorded".into(),
            Status::Failed(why) => format!("failed({why})"),
        }
    }

    /// Whether the check passes with this status.
    pub fn passes(&self) -> bool {
        *self == Status::Ok
    }
}

/// Judges one 90th percentile, in ms: over the line, slower than `recorded` (when this computer has a record),
/// unrecorded (it has a record, without this bench), or fine.
pub fn judge(p90: f64, recorded: Option<f64>, has_record: bool) -> Status {
    if p90 > LIMIT_MS {
        return Status::Over;
    }
    match recorded {
        Some(r) if p90 > r * (1.0 + SLOWER_FRACTION) && p90 - r > NOISE_MS => Status::Slower,
        Some(_) => Status::Ok,
        None if has_record => Status::Unrecorded,
        None => Status::Ok,
    }
}

/// Reads a recorded file: `name number` per line, `#` lines and blank lines ignored.
pub fn parse_recorded(text: &str) -> BTreeMap<String, f64> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let (name, v) = l.split_once(char::is_whitespace)?;
            Some((name.to_string(), v.trim().parse().ok()?))
        })
        .collect()
}

/// The recorded file's text for `machine`: a header that says what the numbers are, then one line per bench.
pub fn write_recorded(machine: &str, date: &str, numbers: &BTreeMap<String, f64>) -> String {
    let mut out = format!(
        "# Recorded bench numbers for `{machine}`: the 90th percentile of frame time in milliseconds (best of the runs), release\n\
         # build, from `make bench-record` on {date}. `make bench-check` fails when a bench is over 50 ms or more than 20 %\n\
         # (and 1 ms) slower than its number here. Record again only after a change that is meant to move them.\n"
    );
    for (name, v) in numbers {
        out.push_str(&format!("{name} {v:.1}\n"));
    }
    out
}

/// The percent change from `recorded` to `p90`, as `+3` or `-12`.
pub fn change_pct(p90: f64, recorded: f64) -> String {
    if recorded <= 0.0 {
        return "n/a".into();
    }
    format!("{:+.0}", (p90 / recorded - 1.0) * 100.0)
}

/// What a bench came to after all its runs: the best (lowest) 90th percentile, and the fields of that run.
#[derive(Clone, Debug)]
pub struct Measured {
    /// The lowest 90th percentile of the runs, in ms.
    pub p90: f64,
    /// The `CHECK` line's fields of that run.
    pub fields: Fields,
}

/// The computer's name: `BENCH_MACHINE`, else `hostname`.
pub fn machine_name() -> String {
    if let Ok(m) = std::env::var("BENCH_MACHINE")
        && !m.trim().is_empty()
    {
        return m.trim().to_string();
    }
    Command::new("hostname")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

/// Runs the app once with `args` and returns its bench line. The app gets a scratch home, so nothing a person keeps
/// is read or written, and is stopped if it takes over [`RUN_LIMIT`].
fn run_once(root: &Path, app: &Path, home: &Path, args: &[String]) -> Result<Fields, String> {
    let mut cmd = Command::new(app);
    cmd.args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for (k, sub) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_STATE_HOME", "state"),
    ] {
        cmd.env(k, home.join(sub));
    }
    cmd.env("HOME", home)
        .env("APPDATA", home)
        .env("LOCALAPPDATA", home);
    let mut child = cmd.spawn().map_err(|e| format!("{}: {e}", app.display()))?;
    let mut stdout = child.stdout.take().ok_or("no output")?;
    let reader = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = std::io::Read::read_to_string(&mut stdout, &mut text);
        text
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > RUN_LIMIT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("timed_out".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let text = reader.join().map_err(|_| "no output".to_string())?;
    let fields = text.lines().find_map(parse_check).ok_or_else(|| {
        text.lines()
            .find(|l| l.contains("result=fail"))
            .map_or("no_bench_line".to_string(), |l| {
                l.split_whitespace()
                    .find_map(|w| w.strip_prefix("reason="))
                    .map_or("no_bench_line".into(), |r| r.to_string())
            })
    })?;
    Ok(fields)
}

/// Runs one bench `runs` times and keeps the best 90th percentile. Checks the line says it ran where the bench asked.
pub fn measure(
    root: &Path,
    app: &Path,
    home: &Path,
    bench: &Bench,
    runs: u32,
) -> Result<Measured, String> {
    let mut best: Option<Measured> = None;
    for _ in 0..runs.max(1) {
        let fields = run_once(root, app, home, &bench.args)?;
        if fields.get("build").map(String::as_str) != Some("release") {
            return Err("not_a_release_build".into());
        }
        if let Some(want) = bench.profile
            && fields.get("profile").map(String::as_str) != Some(want)
        {
            return Err(format!(
                "ran_on_{}",
                fields.get("profile").map_or("?", String::as_str)
            ));
        }
        let canvas = fields.get("canvas").map_or("?", String::as_str);
        if canvas != bench.canvas {
            return Err(format!("canvas_was_{canvas}"));
        }
        let p90: f64 = fields
            .get("p90_ms")
            .and_then(|v| v.parse().ok())
            .ok_or("no_p90")?;
        if best.as_ref().is_none_or(|b| p90 < b.p90) {
            best = Some(Measured { p90, fields });
        }
    }
    best.ok_or_else(|| "no_runs".into())
}

/// What `cargo xtask bench-check` was asked.
#[derive(Clone, Debug)]
pub struct Options {
    /// Write the recorded file instead of comparing with it.
    pub record: bool,
    /// How many times each bench runs; the best counts.
    pub runs: u32,
    /// The heaviest starter, relative to the repository.
    pub starter: String,
    /// The starter the Basic tab is timed over.
    pub basic_starter: String,
    /// Only the benches whose name contains this (for trying one; never with `--record`).
    pub only: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            record: false,
            runs: 2,
            starter: HEAVIEST_STARTER.into(),
            basic_starter: BASIC_STARTER.into(),
            only: None,
        }
    }
}

/// The heaviest starter on both tablets and both canvases (measured on `gibson`, rung 17 leg 2; the log says by how
/// much).
pub const HEAVIEST_STARTER: &str = "crates/ui/starters/paper/hex.quad";
/// The starter the Basic tab is timed over: one with objects marked Show in Basic.
pub const BASIC_STARTER: &str = "crates/ui/starters/paper/hex.quad";

/// The recorded file for `machine`.
pub fn recorded_path(root: &Path, machine: &str) -> PathBuf {
    root.join("docs")
        .join("bench")
        .join(format!("{machine}.txt"))
}

/// The result of a whole check: one line per bench and the summary line, and whether it passed.
pub struct Report {
    /// The `BENCH` lines, then the `BENCH-CHECK` line.
    pub lines: Vec<String>,
    /// Whether every bench passed.
    pub passed: bool,
}

/// Runs the set and judges it (or records it). The app must be built in release mode already.
pub fn run(root: &Path, opts: &Options) -> Result<Report, String> {
    if cfg!(target_os = "linux")
        && std::env::var_os("DISPLAY").is_none()
        && std::env::var_os("WAYLAND_DISPLAY").is_none()
    {
        return Err("a bench needs a window session: set DISPLAY (for example DISPLAY=:0)".into());
    }
    let exe = if cfg!(windows) {
        "quadrule.exe"
    } else {
        "quadrule"
    };
    let app = root.join("target").join("release").join(exe);
    if !app.exists() {
        return Err(format!(
            "{} is missing: build it with `cargo build --release -p quadrule-app`",
            app.display()
        ));
    }
    let home = root.join("target").join("bench-home");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).map_err(|e| format!("{}: {e}", home.display()))?;
    let machine = machine_name();
    let path = recorded_path(root, &machine);
    let recorded = std::fs::read_to_string(&path)
        .ok()
        .map(|t| parse_recorded(&t));
    let has_record = recorded.is_some() && !opts.record;
    let recorded = if opts.record { None } else { recorded };
    let mut lines = Vec::new();
    let mut numbers = BTreeMap::new();
    let (mut ok, mut over, mut slower, mut unrecorded, mut failed) = (0, 0, 0, 0, 0);
    let set: Vec<Bench> = benches(&opts.starter, &opts.basic_starter)
        .into_iter()
        .filter(|b| {
            opts.only
                .as_ref()
                .is_none_or(|o| b.name.contains(o.as_str()))
        })
        .collect();
    for bench in &set {
        let was = recorded.as_ref().and_then(|r| r.get(&bench.name)).copied();
        let line = match measure(root, &app, &home, bench, opts.runs) {
            Ok(m) => {
                let status = judge(m.p90, was, has_record);
                match status {
                    Status::Ok => ok += 1,
                    Status::Over => over += 1,
                    Status::Slower => slower += 1,
                    Status::Unrecorded => unrecorded += 1,
                    Status::Failed(_) => failed += 1,
                }
                numbers.insert(bench.name.clone(), m.p90);
                format!(
                    "BENCH name={} canvas={} profile={} page={} runs={} p90_ms={:.1} recorded_ms={} change_pct={} limit_ms={LIMIT_MS} status={}",
                    bench.name,
                    m.fields.get("canvas").map_or("?", String::as_str),
                    m.fields.get("profile").map_or("?", String::as_str),
                    m.fields.get("page").map_or("?", String::as_str),
                    opts.runs,
                    m.p90,
                    was.map_or("none".into(), |r| format!("{r:.1}")),
                    was.map_or("n/a".into(), |r| change_pct(m.p90, r)),
                    status.word()
                )
            }
            Err(why) => {
                failed += 1;
                format!(
                    "BENCH name={} canvas={} profile={} page=? runs={} p90_ms=? status={}",
                    bench.name,
                    bench.canvas,
                    bench.profile.unwrap_or("any"),
                    opts.runs,
                    Status::Failed(why).word()
                )
            }
        };
        println!("{line}");
        lines.push(line);
    }
    let everything_under_line = over == 0 && failed == 0;
    let mut recorded_to = "no".to_string();
    if opts.record {
        if everything_under_line && opts.only.is_none() {
            let dir = path.parent().ok_or("no folder")?;
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            let date = date_today();
            std::fs::write(&path, write_recorded(&machine, &date, &numbers))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            recorded_to = path.display().to_string();
        } else {
            recorded_to = "refused(a bench is over 50 ms, failed, or only some ran)".into();
        }
    }
    let passed = if opts.record {
        everything_under_line && recorded_to != "no" && !recorded_to.starts_with("refused")
    } else {
        over == 0 && slower == 0 && unrecorded == 0 && failed == 0
    };
    let summary = format!(
        "BENCH-CHECK machine={machine} mode={} benches={} ok={ok} over_50ms={over} slower={slower} unrecorded={unrecorded} failed={failed} record={} status={}",
        if opts.record { "record" } else { "check" },
        set.len(),
        if opts.record {
            recorded_to
        } else if has_record {
            path.display().to_string()
        } else {
            "none_for_this_computer(only_the_50ms_line_judged)".into()
        },
        if passed { "ok" } else { "fail" }
    );
    println!("{summary}");
    lines.push(summary);
    Ok(Report { lines, passed })
}

/// Today's date, `YYYY-MM-DD`, from the system clock (UTC).
fn date_today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    quadrule_model::blank::rfc3339_utc(secs)
        .split('T')
        .next()
        .unwrap_or("")
        .to_string()
}
