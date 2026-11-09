# STATUS — quadrule

_Last updated: 2026-11-09_

The project's live state. Update it when work lands. Every claim cites at least one file that exists.

## Now

- Rung 1, the headless end-to-end slice, is on `main` (PR #1, `00ad0e2`): a test builds a document, saves it as
  `.quad`, reopens it and exports Manta and Nomad PNGs matching golden images byte for byte
  (`crates/render/tests/slice.rs`, `crates/render/tests/golden/SHA256SUMS`).
- Calibration sheets (napkin §8.4): `cargo xtask calibration-sheet` writes lines, dots, text and writing pages per
  profile, each sample coded, with a `codes.txt` key (`crates/xtask/src/calib.rs`, how-to in `docs/calibration.md`).
- Licensed Apache-2.0 (`LICENSE`); every Rust file carries the header, enforced by a test
  (`crates/xtask/tests/license_headers.rs`).
- Checks, 2026-11-09 on the Pi: `cargo test --workspace -- --nocapture` → 44 tests, 48 `CHECK` lines, all pass;
  clippy `-D warnings` and fmt clean. CI on Linux, macOS and Windows (`.github/workflows/ci.yml`).
- Work goes straight to `main` until a job needs a branch (chris's ruling, `AGENTS.md`). Rust 1.99.1 pinned
  (`rust-toolchain.toml`).

## Next

- chris: make `tholent/quadrule` public (Settings → General → Change visibility); the token cannot
  (`docs/STATUS-log.md`).
- chris: put the slice PNGs and the calibration sheets on a Manta and a Nomad, and report codes as in
  `docs/calibration.md`. The answers set the role values and settle square vs round dots.
- Then: enter the calibrated values in `crates/profiles/data/`, re-bless the slice goldens, and plan rung 2.

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- Role widths and grays are estimates until calibration (`crates/profiles/data/supernote.a5x2.ron`).
- ADR parts not built yet: font signature, fuzzing, hinting flag, system-font fallback and substitution, canvas
  (`docs/STATUS-debt.md`).

## Log

Session history: `docs/STATUS-log.md`. Finished work: `docs/STATUS-archive.md`.
- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).
- 2026-11-09: rung 1 built and merged (PR #1), CI green on three OSes (`docs/STATUS-log.md`).
- 2026-11-09: LICENSE and headers; calibration sheets (`docs/STATUS-log.md`).

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
