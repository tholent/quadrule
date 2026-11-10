---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify, in his review of the pending ADRs)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: process
built: landed
proof: "`.github/workflows/ci.yml`, `.github/workflows/fuzz.yml`, `.github/workflows/coverage.yml`, `rust-toolchain.toml`, `Makefile`"
supersedes: nothing
amends: nothing
---
# ADR-006 — The Raspberry Pi is the build machine, CI checks Linux, macOS and Windows only when code changes, and every test reports one structured line

## Decision

1. **The build machine is chris's Raspberry Pi 5** (4 GB, aarch64). Headless parts build and test there; the window
   is checked on its display. Builds use 2 jobs (`CARGO_BUILD_JOBS=2`, the Makefile's `JOBS`).
2. **Rust is pinned** in `rust-toolchain.toml` (1.99.1 today).
3. **CI runs on Linux, macOS and Windows** (`.github/workflows/ci.yml`): fmt, clippy with `-D warnings`, and the
   whole workspace's tests, the app included.
4. **Workflows fire only on changes they test.** `ci.yml` runs on code, workspace, toolchain and cargo-config
   changes, the device screenshots and itself; the 60-second fuzz run is its own `fuzz.yml`, on the format and model
   crates. Both can be run by hand. A docs-only change runs nothing.
5. **Coverage is self-hosted:** `coverage.yml` runs `cargo llvm-cov` and writes the badge to a `badges` branch for
   shields.io. No outside coverage service.
6. **Every test prints one structured line**, `CHECK crate=… name=… result=…`, and checks are counted with
   `grep -o 'CHECK crate=[^ ]* name=[^ ]*' | wc -l`.
7. **The Makefile is a convenience** for Linux and macOS; CI keeps calling cargo directly. `make check-all` is what
   CI runs; `make build-proof` proves the build on disk matches HEAD at session start.

## Context

- The Pi: spikes record Q5 (`docs/bbq/2026-11-09-quadrule-spikes.md`). Three systems: rung 1 record Q4, so the ADRs'
  cross-machine checks (ADR-002 T14/T15, ADR-003 R06/R09) could run.
- Path filters, chris, word for word: "let's ensure that or CI workflows are only firing on changes that should be
  tested. a change to docs probably doesn't need a fuzzer run, etc..." (`docs/STATUS-log.md`).
- Coverage and badges: chris chose one CI badge for fmt, clippy and tests together and self-hosted coverage
  (`docs/STATUS-log.md`, 2026-11-10 badges entry). First coverage run, 42868546083, passed.
- Structured lines: `AGENTS.md` ("Make tests read one structured line, not prose").
- Makefile: `docs/bbq/2026-11-10-makefile.md` Q1–Q4.

## Consequences

- The Pi is slow, so plain `cargo test` uses the default members and leaves the app out (ADR-008); CI covers it.
- Owed (in `docs/STATUS-debt.md`): `CHECK` lines can share a line with the test runner's own output, so
  `grep '^CHECK'` can miss one; nothing enforces the leading newline yet.
- The `gh` token is fine-grained: it cannot read check summaries or edit repository settings
  (`docs/STATUS-debt.md`).

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
- 2026-11-10: ratified by chris (answer "Ratify" to "ADR-006 … Ratify?", in his review of the pending ADRs). Prior header
  values: status `proposed`, status-note `drafted 2026-11-10 from the project's records (backlog sweep, no
  rounds, chris's choice); awaiting ratification`.
