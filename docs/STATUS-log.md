# STATUS-log — quadrule

A dated, append-only record of what each session did. Newest at the bottom; split by month when it grows.

## 2026-11-09
- Prepared with project-prep: agent rules, STATUS and its split files, and the folders.
- napkin + bbq (9 questions) planned the first work: the three spikes
  (`docs/bbq/2026-11-09-quadrule-spikes.md`). goforit, run as three parallel Opus 5.5 workers in worktrees at chris's
  request; rustup installed Rust 1.99.1.
- Spikes merged into `spikes/2026-11-09` (merge `67860b6`): format, text, rendering, each with FINDINGS.md and
  ADR-PROPOSAL.md. The workers could not write report files; the lead saved each FINDINGS.md from their reports.
  Re-run on the merge: format 10, text 12, render 9 tests passed.
- chris ruled on the text spike's parked items (font fallback from any installed font, warned at save and export;
  font signature stored per font), recorded word for word in the bbq record.
- Talk-through on file version label and text hinting: 7 decisions agreed in talk
  (`docs/proposals/talk-2026-11-09-format-versions-and-hinting.md`). Nothing pushed.
- chris: "ratify all three and commit the napkin". Napkin committed (`c8f5717`); paid the "napkins/ untracked" debt.
  ADR format chosen: ADR+. ADR-001 (format, with talk D1–D3, D7 and the font signature folded in), ADR-002 (text,
  with the fallback ruling and talk D4–D6), ADR-003 (export renderer) written and ratified; adr-check: 3 ADRs,
  0 failures.
- Rung 1 (headless slice) run from `docs/bbq/2026-11-09-rung-1-headless-slice.md` on `rung-1/headless-slice`, lead
  Opus 5.5 at high effort, no subagents. Session start: HEAD `3fdd1e7` = `origin/rung-1/headless-slice`; no workspace
  build existed yet. Repo setup steps of the kickoff were already done by chris (remote, deploy key, `gh` signed in).
- Built the workspace and six crates in the record's order: model + profiles (`bc15346`), text + layout (`1a35004`),
  format (`85ef689`), render + slice test + golden images (`412da04`), CI (`b7364dd`). `spikes/` untouched.
- Checks on the Pi: `cargo test --workspace -- --nocapture` → 40 passed, 0 failed, 44 CHECK lines all pass;
  `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
- Golden PNGs blessed on the Pi after viewing both by eye: Manta `2f1cb150…`, Nomad `ac930a98…`
  (`crates/render/tests/golden/SHA256SUMS`).
- PR #1 opened as a draft; CI run 42660403429 green on ubuntu-latest (x86_64), macos-latest (aarch64) and
  windows-latest (x86_64): each 40 tests, 44 CHECK lines pass; both PNGs byte-identical to the Pi's goldens and the
  `.quad` sha256 `36911514…` the same everywhere. Paid the debt "compare R06/R09 export hashes on x86 Linux, macOS and
  Windows"; ADR-003 item 2 (portable maths) is not needed. Text T14/T15 partly paid (bundled-font layout and
  outlines match; system fonts not yet loaded).
- Found and fixed on the way: zip 9.5 marks entries DOS on Windows, which would give different `.quad` bytes there;
  the writer now pins the system byte, permissions and time. The slice page first scaled its dot grid on the Nomad,
  putting a dot row between the title and its rule; the dots are now pinned under the header.
- Proposals for chris (in PR #1): package names `quadrule-*` in folders named as napkin §15.1; square dots one role
  width wide; role widths and grays are estimates. The display list lives in `layout`, not `render` as §15.1's table
  words it. The model holds only what layout draws (rotation, landscape, isometric, overrides, groups, fixed text
  boxes wait for the rung that builds them).
- Note on ADR-001 item 1 ("half font units, exact"): shapes off the half-unit grid are rounded to it, and live text
  uses the same rounded shape, so live and saved text stay byte-identical. At Atkinson weights 400 and 700 nothing
  was rounded (`CHECK crate=text name=bold_differs … rounded_700=0`).
- Cost note: the repo is private, so CI uses Actions minutes (macOS ×10, Windows ×2); this run took about 1, 2 and 4
  minutes. CI runs only on pull requests, pushes to main and by hand.
- chris merged PR #1 (merge commit `00ad0e2`, 2026-11-09 17:09 UTC). CI on `main` after the merge (run 42661800637):
  green on all three OSes. Local `main` fast-forwarded to `00ad0e2`.
- chris ruled, word for word: "until specifically necessary, let's just work on the main branch directly. This is
  pre-alpha software at this point." Recorded as a dated amendment under the branch rule in `AGENTS.md`. PR #2's
  commit went to `main` by fast-forward.
- chris: the `gh` token's expiry is 90 days; written into `docs/STATUS-debt.md`.
- chris, word for word: "let's make this repo public. copy the license from here:
  https://www.apache.org/licenses/LICENSE-2.0.txt and include apache required headers in our coding files.
  1. keep quadrule-* 2. and 3. let's get the calibration sheet generation working and then we can do some actual
  testing". Ruled: package names stay `quadrule-*`. Dot shape and role values wait on the device test.
- History checked before going public (26 commits, all branches): no tokens, keys or passwords; only expected files.
  Every commit carries chris's name and email, which become public. `gh repo edit --visibility public` → HTTP 403
  (the token has no Administration permission), so chris flips it in GitHub settings.
- `LICENSE` copied from apache.org (sha256 `cfc7749b…`); the appendix notice heads every `.rs` under `crates/`
  (copyright holder defaulted to "Chris Wells"; `spikes/` untouched). New crate `quadrule-xtask` (napkin §8.4's
  `cargo xtask`) with a test that fails on a missing header (proved by a probe file). Comment-only change, but line
  numbers moved, so binaries are not byte-identical; the slice goldens and `.quad` bytes are unchanged.
- Calibration sheets: `cargo xtask calibration-sheet [--profile <id>|all] [--out <dir>]` writes lines (6 widths ×
  15 grays, dashes), dots (square 1–4 px, round 2–6 px × 8 grays), text (6–18 pt, regular and bold × 3 grays) and
  writing strips (W1–W16), 243 coded samples per profile; Manta 6 pages, Nomad 9. Round dots and dashes exist only
  on the sheets, not in the document model. Tests: codes unique and identical on both profiles; 204 line samples
  print exactly their claimed width and gray; sheets repeatable. Viewed by eye; fixed a wrapping 18 pt sample and
  cramped strips. How-to: `docs/calibration.md`.
- Checks: `cargo test --workspace -- --nocapture` → 44 passed, 0 failed, 48 CHECK lines pass; clippy and fmt clean.
- chris made the repository public (`gh repo view` → PUBLIC) and uploaded portrait screenshots, toolbar left, right,
  top and floating, for the Manta and the Nomad into `docs/device-measurements/<profile_id>/` (no landscape or
  collapsed toolbar yet). Viewed before committing: a grid template, the toolbar, page number and a date-stamp note
  name; nothing personal.
- Safe zones: prototyped in Python (scratchpad), then built as `cargo xtask safe-zones`
  (`crates/xtask/src/safezones.rs`): rebuild the bare grid template (1 px lines every 59 px on both devices), mark
  every differing pixel as covered; covered in all screenshots = footer, the rest = each screenshot's toolbar. Rust and
  Python gave the same 12 boxes; overlays checked by eye. Manta: docked toolbars 116 px, footer boxes 116 px tall.
  Nomad: 99 px, footer 100 px, and its side toolbars stop at y 1763 above the footer. Written into both profile RON
  files; the floating zone is advisory. Tests: a synthetic grid with known boxes is measured exactly; profiles must
  match their screenshots.
- Calibration sheets now keep clear of every docked toolbar and the footer at once (new test: 0 overlaps on both
  profiles); column headings shortened and wrapped within their column (the Nomad's "gray 238" ran into the right
  toolbar); a section's legend stays with its first row. Manta 6 pages, Nomad 10.
- Checks: `cargo test --workspace -- --nocapture` → 47 passed, 0 failed, 51 CHECK lines pass; clippy and fmt clean;
  slice goldens unchanged.
- Slip: `git add -A docs` swept a vim swap file (`docs/.calibration.md.swp`, from an open vim session) into
  `be6fea9`, pushed to the public repo. It holds the local user name, host name and file path. Untracked in
  `a087163` and `*.swp` / `*.swo` / `*~` now ignored; it stays in history unless chris asks for a rewrite. Lesson: stage
  named paths, not whole folders.
- chris, word for word: "for the nomad, the toolbar does normally extend to the bottom when docked on left and right.
  I simply had edited the icons that appeared and it shrunk up. In looking at the screenshots, can anything be
  divined from the grid? That is the stock 5mm template from super note". Docked toolbar zones now span their whole
  edge (thickness measured, length = page); Nomad side zones 1763 → 1872 px tall. Synthetic test updated to draw
  trimmed toolbars.
- The stock 5 mm grid, read from the screenshots: 59 px spacing on both devices (pixel-locked 5 mm at 300 PPI),
  leftover split evenly (centred origin), 1 px lines at gray 1 with hard edges, white page, full bleed
  (`docs/device-measurements/README.md`).
- Checks: 47 passed, 0 failed, 51 CHECK lines pass; clippy and fmt clean.
- `cargo xtask safe-zones` now prints the template grid it finds (`0db5abd`). CI run 42671324951 on `0db5abd`
  concluded "failure" with only two jobs, ubuntu and windows, both passing; the macOS job was never created. Nothing
  in the code failed; the cause is in the run's annotations, which the token cannot read, and the token cannot
  re-run jobs either.
- Both odd runs were GitHub-side: the public run pages show "Internal server error" annotations (correlation ids
  `f1ab9d35-…` for 42671324951, `59f181e4-…` for 42671778956); githubstatus.com said all operational. The first run
  also warned that `actions/checkout@v4` and `actions/upload-artifact@v4` target the deprecated Node 20: moved both
  to `@v7` (latest releases v7.0.1 and v7.0.2, both `node24`). Runs on `main` are no longer cancelled by a newer push;
  the cost note now says public runners are free.
- looseends (project): 16 of 22 scanned markers still open plus 6 from this session; listed by who acts. chris's
  answers (question tool): crank work "Tidy-up, Stock-grid match, Fuzzing, Start the linter"; copyright "Chris Wells
  (current)"; swap file "Rewrite history"; housekeeping "Ignore .claude, delete branches".
- Housekeeping: deleted merged branches (all ancestors of main): local `spikes/2026-11-09`, `prep/2026-11-09`,
  three `worktree-agent-*`, `rung-1/headless-slice`, `status/2026-11-09-rung-1-merged`; remote
  `rung-1/headless-slice`, `status/2026-11-09-rung-1-merged`. `/.claude/` ignored.
- History rewrite (chris's ruling): `git filter-branch --index-filter` removed `docs/.calibration.md.swp` from
  `be6fea9^..main`; final tree identical to the old main; force-pushed with lease (`4a1b26a` → `8c602c1`). Local
  backup branch `backup/pre-swap-rewrite` (never pushed). GitHub still serves the old commits by hash until its own
  clean-up; a full purge needs GitHub Support. Old → new names for hashes cited above: `be6fea9` → `e1c8021`,
  `a087163` → `94ce048`, `ff573f7` → `986234f`, `df2beba` → `e893bec`, `0db5abd` → `37c4754`, `0ec4e81` → `5e7b506`,
  `4a1b26a` → `6082dfa`, `b7278ab` → `8c602c1`.
- Tidy-up: proposals index marks the talk record folded into ADR-001/002. ADR-003 item 4 built: tiny-skia as a
  test-only second renderer (`crates/render/tests/render.rs::tiny_skia_agrees_on_snapped_shapes`: 0 differing
  pixels on both profiles). ADR `built` headers updated (old values in each History): ADR-001 partial, ADR-002
  partial, ADR-003 landed; `adr-index.py` rewrote INDEX.md, `adr-check.py`: 3 ADRs, 0 failures (proofs resolve; it
  does not run them). Pattern clipping: tried drawing only shapes wholly inside a region; it removed the slice's
  first dot row and column (38 dots on the margin), so reverted. The rule is now written down and tested instead
  (`pattern_shapes_centred_in_their_region_and_drawn_whole`); veto-able in the debt list.
