# STATUS — quadrule

_Last updated: 2026-11-09_

The project's live state. Update it when work lands. Every claim cites at least one file that exists.

## Now

- Rung 1, the headless end-to-end slice, is built on branch `rung-1/headless-slice` and waits in PR #1 for chris
  to merge (`docs/bbq/2026-11-09-rung-1-headless-slice.md`). Six crates under `crates/` (`Cargo.toml`); the slice
  test builds a document, saves it as `.quad`, reopens it and exports Manta and Nomad PNGs that match the golden
  images byte for byte (`crates/render/tests/slice.rs`, `crates/render/tests/golden/SHA256SUMS`).
- Checks, 2026-11-09: `cargo test --workspace -- --nocapture` → 40 tests, 44 `CHECK` lines, all pass, on the Pi
  (aarch64 Linux) and in CI on Linux x86_64, macOS aarch64 and Windows x86_64 (`.github/workflows/ci.yml`, run
  42660403429). The golden PNGs and the `.quad` bytes are identical on all four machines.
- Rust 1.99.1, pinned for the Pi and CI (`rust-toolchain.toml`). Remote: `tholent/quadrule`, private.

## Next

- chris: review and merge PR #1; rule on its three taste proposals (package names, square dots, estimated role
  values) (`docs/STATUS-log.md`, 2026-11-09 rung 1 entry).
- chris: the hardware step — copy `crates/render/tests/golden/*.png` to a Manta and a Nomad (MyStyle folder) and
  say how they look (`docs/STATUS-debt.md`).
- Then plan rung 2 with a bbq. What the ADRs still owe is listed in `docs/STATUS-debt.md`.

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- Hardware step for rung 1 (owed to chris); role widths and grays are estimates until calibration
  (`crates/profiles/data/supernote.a5x2.ron`).
- ADR parts not built yet: font signature, fuzzing, hinting flag, system-font fallback and substitution, canvas
  (`docs/STATUS-debt.md`).

## Log

Session history: `docs/STATUS-log.md`. Finished work: `docs/STATUS-archive.md`.
- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).
- 2026-11-09: rung 1 built; PR #1 open, CI green on three OSes (`docs/STATUS-log.md`).

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
