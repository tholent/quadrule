# bbq record — quadrule rung 1: the headless end-to-end slice

2026-11-09 · untyped bbq · 8 questions · lead: Opus 5.5, high effort.

Grounding: `STATUS.md`, `docs/decisions/INDEX.md` (ADR-001 `.quad` format, ADR-002 text stack, ADR-003 export
renderer, all accepted), `spikes/*/FINDINGS.md`, `napkins/quadrule-spec.md` §15–§16, `AGENTS.md`.

## Rules

- **Q1 Count.** 8.
- **Q2 Rung 1 scope.** A headless end-to-end slice. A test builds a small document (a dot-grid pattern region,
  ruled lines, one text box), saves it as `.quad`, reopens it, and exports Manta and Nomad PNGs that must match
  golden images (saved reference PNGs). It proves model → format → layout → text → render join up. No window.
- **Q3 Lead.** Opus 5.5 at high effort.
- **Q4 CI.** Set up GitHub now, so CI runs on Linux, macOS and Windows and the ADRs' owed cross-machine checks
  (ADR-002 T14/T15, ADR-003 R06/R09 hashes) can run.
- **Q5 Crates.** Only what the slice needs: `model`, `format`, `profiles`, `layout`, `text`, `render`. The other five
  (napkin §15.1: `app`, `ui`, `lint`, `imaging`, `i18n`) arrive with the rung that uses them.
- **Q6 Profiles.** Manta and Nomad use the napkin's resolution and PPI (`napkins/quadrule-spec.md:99-100`); line
  roles and gray levels are clearly labelled estimates with `calibration_meta.verified: false`, replaced by
  calibration later without code changes.
- **Q7 Visibility.** The GitHub repository is private for now. Reversible: chris can make it public any time.
- **Q8 Anything else to protect or avoid?** Keep `spikes/` untouched: the real crates may copy ideas from it but
  never edit or delete it.

## Owed

(empty)

## Defaulted

- **Size:** about 2–4 sessions on the Pi. veto: trim the slice.
- **gh without sudo:** install the `gh` binary from GitHub's release tarball into `~/.local/bin` (checksum
  verified); undo by deleting it. chris signs in with `! gh auth login`. veto: install it some other way.
- **Repository:** private repo named `quadrule` under chris's GitHub account; `main` pushed as the default branch.
  Creating it and pushing are outward-facing: the lead asks before each. veto: another name or owner.
- **Landing:** rung 1 works on branch `rung-1/headless-slice` from `main`, opens one PR, and chris merges. Never
  push straight to `main` after the first push that creates it. veto: merge on green.
- **Workspace:** a root `Cargo.toml` workspace with the six crates under `crates/`; `spikes/` stays outside the
  workspace. Rust edition and minimum Rust version pinned to what the Pi has (1.99.1). veto: another layout.
- **Order:** `model` and `profiles` first (everything depends on them), then `layout`, `format`, `text`, `render`,
  then the slice test. The lead may hand an independent crate to a subagent once `model` has landed, reviewing its
  diff. veto: lead alone.
- **Fonts:** Atkinson Hyperlegible Next is committed as a bundled font with its OFL licence text (napkin §7.2
  bundles fonts in the app); sha256 recorded. Noto Emoji waits for the rung that adds emoji. veto: fetch fonts at
  build time instead.
- **Golden images:** rendered by vello_cpu (ADR-003), committed under the `render` crate's test folder, with a
  sha256 per image; CI compares hashes on all three OSes. If they differ, ADR-003 item 2 (portable scalar mode)
  applies, and the lead says so. veto: none expected.
- **Text in the slice:** plain (unhinted) outlines only; hinting stays unbuilt (ADR-002 puts it behind a flag,
  for a later rung). veto: none expected.
- **Flags:** AGENTS.md's "new feature behind a flag" applies to user-facing features; rung 1 has none (no UI).
  veto: say it applies.
- **Emulator before hardware (AGENTS.md):** rung 1's "emulator" is viewing the exported PNGs on a computer; the
  hardware step is chris copying them to a Manta and a Nomad (MyStyle folder, napkin §11.3) and saying how they
  look. That step is owed to chris and does not block the PR. veto: make it block.
- **Checks:** `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
  with test results as one structured line each (AGENTS.md). Build with `CARGO_BUILD_JOBS=2` on the Pi.
- **Parked render questions** (Scale vs pixel-locked rhythm on A5 X; canvas fallback on weak GPUs) do not block
  rung 1: A5 X is not in the slice and there is no canvas. veto: settle them first.

## Kickoff

Run on **Opus 5.5 at high effort**; on any other model or effort, stop and say so before doing anything. Read
`AGENTS.md`, `docs/decisions/` ADR-001 to ADR-003 and this record. Install `gh` into `~/.local/bin`, ask chris to run
`! gh auth login`, then ask before creating the private `quadrule` repository and pushing `main`. On branch
`rung-1/headless-slice`, build the workspace and the six crates in the Defaulted order until the slice test passes
locally and in CI on three OSes; open one PR for chris to merge. Leave `spikes/` untouched. Update `STATUS.md` and
`docs/STATUS-log.md` at the end.
