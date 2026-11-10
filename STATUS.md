# STATUS — quadrule

_Last updated: 2026-11-10_

The project's live state. Update it when work lands. Every claim cites at least one file that exists.

## Now

- Rung 1, the headless end-to-end slice, is on `main` (PR #1, `00ad0e2`): a test builds a document, saves it as
  `.quad`, reopens it and exports Manta and Nomad PNGs matching golden images byte for byte
  (`crates/render/tests/slice.rs`, `crates/render/tests/golden/SHA256SUMS`).
- Calibration sheets (napkin §8.4): `cargo xtask calibration-sheet` writes lines, dots, text and writing pages per
  profile, each sample coded, with a `codes.txt` key (`crates/xtask/src/calib.rs`, how-to in `docs/calibration.md`).
- Both devices calibrated by chris (2026-11-09): role values, the new Dot grid and Label roles, in their profiles
  (`crates/profiles/data/`, records in `docs/device-measurements/*/calibration.md`); file format 0.3.
- Safe zones measured for both devices from screenshots (portrait: toolbar left, right, top, floating; plus the
  footer), by `cargo xtask safe-zones`, and written into the profiles (`crates/profiles/data/`,
  `docs/device-measurements/README.md`). A test keeps the profiles and screenshots in step.
- quadrule reproduces Supernote's stock 5 mm grid: pixel-identical to the Manta's screenshots, and to the Nomad's
  but for a 1 px tie-break (`crates/xtask/tests/stock_grid.rs`, `docs/device-measurements/README.md`).
- Rung 2, the viewer window, is on `main`, look approved by chris 2026-11-09: `cargo run -p quadrule-app --
  [file.quad]` opens a `.quad` (or the sample page), shows the exact export pixels for Manta or Nomad, zooms from
  Fit to 16× with a pixel grid or to real size (after a ruler check), lists lint findings and outlines them, overlays
  safe zones, and exports PNGs named `{title}_{manta|nomad}_{w}x{h}.png` (`crates/ui/`, `crates/app/`; record
  `docs/bbq/2026-11-09-rung-2-viewer.md`). It is an opt-in build: plain `cargo build`/`cargo test` leave it out
  (`Cargo.toml` `default-members`); CI builds and tests it on three OSes.
- Rung 3, the first editor, is built on `main`, look not yet approved (record
  `docs/bbq/2026-11-10-rung-3-first-editor.md`): click and Shift-click to select, a layers panel (eye, lock,
  rename, Move up/down), the inspector's tabs (Page; Arrange, Style, Text, Constraints; Lint) with one length
  field reading mm, pt, px, in, %, u; arrows nudge 1 device pixel, Shift+arrows one rhythm unit; Delete, Ctrl+D;
  unlimited undo/redo; Save, Save As and the unsaved-changes prompt; autosave every 30 s and on focus loss, with
  Restore offered after a crash (`crates/model/src/edit.rs`, `crates/layout/src/units.rs`, `crates/ui/src/editor.rs`,
  `crates/ui/src/panels.rs`, `crates/ui/src/recovery.rs`). View → Canvas switches to the graphics-chip canvas
  (vello on wgpu, `crates/app/src/chip.rs`), falling back to vello_cpu when the chip cannot start or a frame
  takes over 100 ms (`crates/ui/src/canvas.rs`); on the Pi it measured 40 ms a frame (`docs/STATUS-log.md`).
  Export bytes are unchanged (`crates/ui/tests/editor.rs`).
- Every string on screen goes through Fluent (`crates/i18n/`, English in `crates/i18n/locales/en-US/`), lint
  messages included; findings about an object carry its area (`crates/lint/src/lib.rs`).
- A linter with the seven rules that need no calibration (`crates/lint/src/lib.rs`); `cargo xtask lint`.
- The `.quad` reader is fuzzed with cargo-fuzz: 3.3 million runs on the Pi, 60 s on every CI push, no crash
  (`crates/format/fuzz/fuzz_targets/open_quad.rs`).
- Licensed Apache-2.0 (`LICENSE`); the repository is public. every Rust file carries the header, enforced by a test
  (`crates/xtask/tests/license_headers.rs`).
- Checks, 2026-11-10 on the Pi: `cargo test --workspace --no-fail-fast -- --nocapture` → 110 `CHECK` lines, all
  pass (83 with the default members); clippy `-D warnings` and fmt clean. CI on Linux, macOS and Windows
  (`.github/workflows/ci.yml`).
- Work goes straight to `main` until a job needs a branch (chris's ruling, `AGENTS.md`). Rust 1.99.1 pinned
  (`rust-toolchain.toml`).

## Next

- Rung 3 waits on chris's look check of the window (`docs/bbq/2026-11-10-rung-3-first-editor.md`, Kickoff): try
  `cargo run --release -p quadrule-app -- [file.quad]` (add `--canvas chip` for the chip canvas). Taste calls
  made for it are listed in `docs/STATUS-log.md` (2026-11-10, rung 3 built). Then rung 4: dragging and snapping.
- Also owed: the lint rules that need calibrated values (L03, L05, L07, L09, L13, L14; L04 waits on an open question;
  `crates/lint/src/lib.rs`, `docs/STATUS-debt.md`).

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- Clean gray levels are not measured on either device; the Manta's pen shades are assumed (`docs/STATUS-debt.md`).
- ADR parts not built yet: font signature, fuzzing, hinting flag, system-font fallback and substitution, canvas
  (`docs/STATUS-debt.md`).

## Log

Session history: `docs/STATUS-log.md`. Finished work: `docs/STATUS-archive.md`.
- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).
- 2026-11-09: rung 1 built and merged (PR #1), CI green on three OSes (`docs/STATUS-log.md`).
- 2026-11-09: LICENSE and headers; calibration sheets; repo public; safe zones measured; stock grid matched;
  linter started; fuzzing (`docs/STATUS-log.md`).
- 2026-11-09: rung 2 planned (bbq, 12 questions) and built: the viewer window; spike R15 paid (`docs/STATUS-log.md`).
- 2026-11-10: rung 3 planned (bbq, 12 questions): the first editor (`docs/STATUS-log.md`).
- 2026-11-10: rung 3 built (steps 1–9), chip canvas measured on the Pi; look waits on chris (`docs/STATUS-log.md`).

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
