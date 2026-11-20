# Plan: keep the local build folder small (recommendations 1 and 2)

**Status: proposed, not ruled.** chris asked for this plan on 2026-11-19, word for word: "let's put together a plan for
recommendations 1/2. I'll run that on the new VM after the move, as it is a significantly more powerful machine".
The open questions at the end take their defaults unless chris says otherwise when he starts the session.

**2026-11-20: run and landed on chris's VM, with three changes chris ruled; see "What was built" at the end.** The
text above is the plan as written and is left as it was.

2026-11-19 · written by Claude (Opus 5.5, high) · to be run on chris's new development VM.

## Why

On 2026-11-18 the project folder on the Pi was 412 GB, of which `target/` (Cargo's build output) was 411 GB, and the disk
was 97 % full. Measured on the Pi before `cargo clean`:

- `du -sh target/debug/deps` → 334G; `du -sh target/debug/incremental` → 73G; `du -sh target/release` → 3.9G.
- `find target/debug/deps -maxdepth 1 -type f -executable -size +100M | wc -l` → 1314 big programs, many copies each:
  57 of `quadrule`, 48 of `groups`, 40 of `window`, 51 of `fonts`, 44 of `devices` (grouped by name without the
  16-character hash).
- chris then ran `cargo clean`: "Removed 548630 files, 421.4GiB total"; disk 5 % used after.

Two causes, both normal Cargo behaviour:

1. **Each copy is big.** The dev profile keeps full debug information (the data a debugger uses to map the program back to
   the source code), and every one of the 39 test files in `crates/ui/tests/` links its own copy of egui and wgpu.
2. **Old copies are never deleted.** Cargo names each built file with a hash that includes the package version, so most
   likely every version bump (`v0.6.0` … `v0.16.2`) left a full set behind, plus one set per feature mix (`-p` versus
   `--workspace` runs). The first step below checks this.

## Size of the job

One session, about an hour or two on a fast machine; most of it is waiting for the before-and-after builds.

## Step 0: before anything

- `make build-proof` → its BUILD line (the VM's first build, from scratch).
- Note the VM's cores, memory and disk (`nproc`, `free -h`, `df -h .`) in the log entry, dated.
- Check the VM has the pinned compiler: `rustc --version` must say `1.99.1` (`rust-toolchain.toml`).

## Step 1: measure today's settings (the baseline)

1. `cargo clean`, then time `make check-all`. Record wall time and `TESTS …` line.
2. Record `du -sh target target/debug/deps target/debug/incremental` and the size of the biggest test program
   (`ls -S target/debug/deps | head`).
3. Make the duplicates on purpose, to test the cause above: change `version` in `[workspace.package]` to `0.16.99`
   (do not commit), run `make test-all`, change it back, and count the copies of one program, e.g.
   `ls target/debug/deps | grep -c '^groups-[0-9a-f]\{16\}$'`. If it went from 1 to 2, version bumps are confirmed as a
   cause; say so in the log. If not, find the real cause before Step 3.

## Step 2: recommendation 1, smaller debug information

Add to the root `Cargo.toml`:

```toml
# Line numbers in error reports, but no full debugger data: the build folder stays a fraction of its size
# (docs/proposals/build-pruning-2026-11-19.md). Does not change debug_assertions or optimisation.
[profile.dev]
debug = "line-tables-only"

# Libraries we don't write get no debug information at all.
[profile.dev.package."*"]
debug = false
```

Then `cargo clean`, time `make check-all` again, and record the same numbers as Step 1. Required to pass:

- `make check-all` passes with the same check count as the baseline.
- The golden images are unchanged: `git status` clean after the run, and the golden lines in the test output all OK.
- A panic in a test still shows file and line: temporarily add `panic!("probe")` to any test, run it with
  `RUST_BACKTRACE=1`, check the backtrace has `crates/…:line`, then remove it.
- `cfg!(debug_assertions)` is unaffected (used in `crates/ui/src/view.rs`, `welcome.rs` and two tests); the
  `debug` setting does not touch it, but the check-all pass shows it.

What is lost: stepping through variables in a debugger. Anyone who needs that can build with
`CARGO_PROFILE_DEV_DEBUG=true cargo build` for one session. Say this in `DEVELOPMENT.md`.

Release builds (the speed benches, packaging) are not touched: the release profile has no debug information already.
CI's cache (`Swatinem/rust-cache`) is rebuilt once after the change, then is smaller too.

## Step 3: recommendation 2, `make prune` and a size line

### `cargo xtask prune`

Written in Rust in `crates/xtask` (beside `calibration-sheet` and `safe-zones`), so it works on macOS, Windows and Linux
alike; `make prune` calls it. What it deletes, and only this:

1. **Old copies of this project's own programs and libraries** in `target/<profile>/deps`: for each name that belongs to
   this workspace (the `quadrule_*` libraries, the test files' names from `crates/*/tests/*.rs`, the `quadrule` and
   `xtask` programs), keep the newest copy by modified time and delete the others, with their `.d` file next to them.
   It never touches third-party crates: two versions of one library (two `syn`s, say) are both in use, and deleting
   one would rebuild it every time.
2. **Old incremental caches** in `target/<profile>/incremental`: folders not modified for 3 days.
3. Nothing else: not `target/release` programs in use, not `target/tmp` (golden-image output), not
   `crates/format/fuzz/target` (its own workspace; it is 550 MB and mostly stable).

Deleting a file that was still needed costs a rebuild, never a wrong result: Cargo checks what is there before using it.

Options: `--dry-run` prints what it would delete and the total, and deletes nothing. It ends with one structured line,
like the tests:

```
PRUNE target=target removed_files=N removed_gb=N.N kept_gb=N.N dry_run=no
```

### When it runs

- `make prune` runs it on demand.
- `make check` and `make check-all` run it after the tests pass, **except on CI** (when the `CI` environment variable is
  set): CI machines start clean and are cached by rust-cache.

### The size line, so it cannot creep up unseen

`make build-proof` gets one more field, `target_gb=N`, and when it is over a cap (`TARGET_CAP_GB`, default 50) it prints
`over the cap: run make prune` and exits non-zero (AGENTS.md: "Turn a future danger into a build error wherever you can").

### Tests

In `crates/xtask/tests/`, on a made-up `target` folder in a temporary directory (never the real one): two copies of a
workspace test program, keeps the newer; two versions of a third-party library, keeps both; an incremental folder 4 days
old goes, one a day old stays; `--dry-run` deletes nothing; the PRUNE line's counts match. Each test prints one `CHECK`
line like the others.

### Measure it

Repeat Step 1's version-bump trick to make duplicates, run `make prune`, and record the PRUNE line and
`du -sh target` before and after.

## Step 4: close

- `DEVELOPMENT.md`: the profile change and how to get full debug information back for one session; `make prune`, when it
  runs, the cap. `README.md` does not change (nothing a user sees).
- `Makefile` help text for `prune` and the new `TARGET_CAP_GB` variable.
- `docs/STATUS-log.md` entry with every number from Steps 1 to 3 as the command that produced it, dated, and the VM's
  description; `STATUS.md` one line.
- Commit and push to `main` (AGENTS.md, as amended 2026-11-09). A tagged release is not needed: nothing in the app
  changes.

## Open questions (defaults in brackets)

1. Should a big `target/` stop `make build-proof` with an error, or only warn? [error, cap 50 GB]
2. How old must an incremental cache be before it goes? [3 days]
3. Should `make check` and `make check-all` prune by themselves, or only `make prune`? [by themselves, except on CI]
4. Recommendation 3 (one test program per crate instead of one per file) is not in this plan; it cuts disk and link time
   the most, and fits on top of 1 and 2 at any time. Plan it after this lands? [ask chris then]

## Not in this plan, but worth knowing on the VM

- Speed budgets (ADR-009) are judged on the Pi. A bench run on the VM is not a pass or fail for the app's speed; keep
  running `make bench-*` on the Pi, or say clearly which machine a number came from.
- Screenshots for chris's look were taken with `grim` on the Pi's own screen (`wayland-0`). Check the VM has a
  screen to draw on before a session that needs one.

## What was built (2026-11-20)

Run on chris's VM; every number is in `docs/STATUS-log.md` (2026-11-19 to 2026-11-20). Steps 0 to 2 as written, and
the open questions took their defaults. Step 3 changed in three ways, each found by measuring:

1. **Copies are told apart by Cargo's own records, not by name.** "Keep the newest copy" per name deleted builds in
   use: five crates have a `groups` test, and clippy checks a library both alone and with its tests. Each build is
   matched to `target/<profile>/.fingerprint/<package>-<hash>/<what>.json`; a build with no record is kept. Where one
   of our test files shares a third-party crate's name (`layout`, `render`, `window`), only our old test programs go.
2. **More than one copy is kept.** `cargo test` on the default members, `cargo test --workspace`, `cargo build` and
   `make run` build some of our crates with different library features, and Cargo cannot tell those copies from old
   versions; keeping one made every switch between them a rebuild. chris's rulings, word for word (answers to
   questions, 2026-11-19/20): "Keep newest 2"; then, told there are four ways, not two, "Libraries 4, programs 2";
   then, for the `quadrule` and `xtask` programs, "Keep 4 for those two". So: each test program keeps 2, each library
   and those two programs keep 4.
3. **The checks run the xtask program the tests built** (`target/debug/xtask prune`), not `cargo xtask prune`, which
   would build our crates yet another way first. `make prune` builds it with `cargo build --bin xtask`.

Open question 4 (recommendation 3, one test program per crate) is still open: asked of chris at the end of the run.

