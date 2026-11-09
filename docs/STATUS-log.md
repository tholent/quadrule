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
- Stock-grid match: model gains `RhythmOrigin::Centered` (floor(L/u) positions, leftover split evenly, odd pixel to
  the right/bottom) and `Pattern::Grid`; format 0.2.0 with the first real migration step (0.1.0 → 0.2.0, nothing to
  rewrite; tested by opening a 0.1.0 file). No single centring rule matches both stock templates (best 3 of 4
  axes): the Manta and Nomad templates break the odd-pixel tie in opposite directions. Result
  (`crates/xtask/tests/stock_grid.rs`): quadrule's export of the stock grid matches the Manta screenshots exactly
  (0 px outside the toolbar and footer) and the Nomad's exactly once shifted 1 px up-left.
- Linter started: new crate `quadrule-lint` (napkin §8.3) with the rules that need no calibration values — L01
  stroke under 1 px, L02 stroke not pixel-aligned, L06 content in a safe zone (fixed zones Warn, floating toolbar
  Info, one finding per object), L10 off the page, L16 position off the rhythm (positions only; sizes owed), L18
  rhythm fits the margins unevenly, L19 uneven physical-locked spacing. `cargo xtask lint [<file.quad>]` prints the
  findings; the slice page gives 15 (its 8 mm margin sits under docked toolbars; its title rule is off the rhythm).
  Waiting on calibration: L03, L04, L05, L07, L09, L13, L14. Waiting on features: L08, L11, L12, L15, L17.
- Checks: `cargo test --workspace -- --nocapture` → 58 passed, 0 failed, 62 CHECK lines pass; clippy and fmt clean.
- Fuzzing (ADR-001 item 8): nightly Rust 1.102.0 (2026-11-08) and cargo-fuzz 0.13.2 installed on the Pi (undo:
  `rustup toolchain uninstall nightly`, `cargo uninstall cargo-fuzz`). Target `crates/format/fuzz/fuzz_targets/
  open_quad.rs`: any bytes as a `.quad` must not panic, and an openable, savable file must save, reopen and save to
  the same bytes. Seed: the slice `.quad`. 10-minute run: 3,312,293 runs, 5,511 a second, no crash, peak 565 MB.
  CI gains a 60-second fuzz job. Paid the debt "set up cargo-fuzz". ADR-001 header: proof adds the fuzz target,
  owed drops item 8 (old values in its History). License-header test now skips `target/` folders.
- CI path filters (chris: "let's ensure that or CI workflows are only firing on changes that should be tested. a
  change to docs probably doesn't need a fuzzer run, etc..."): `ci.yml` runs on code, workspace, toolchain and cargo
  config changes, the device screenshots, and itself; fuzzing moved to `fuzz.yml`, which runs on `crates/format`,
  `crates/model`, the workspace manifest and itself. Both keep "run by hand". Checked by pushing this docs-only
  entry and seeing no run start.
- Calibration, Nomad dots sheet: chris's results and follow-up recorded word for word in
  `docs/device-measurements/supernote.a6x2/calibration.md`. Rulings: **square dots** ("yes, square dots are the
  ruling"); favourite dot **PD2** (square 4 px, gray 51). Found: 1 px dots invisible even in black; square dots
  speckle from gray 170; the slice's estimated dot grid (2 px, gray 187) is invisible on the Nomad. No profile
  values changed yet.
- Calibration, Nomad lines sheet recorded word for word (`docs/device-measurements/supernote.a6x2/calibration.md`):
  favourite LA4 (1 px, gray 51); lines show to gray 187 (1 px) and 204 (2–6 px); no speckling on any line; fine
  dashes read as dashes only when dark.
- chris, word for word: "let's redo the calibration sheets to include 51". Every sheet but writing already had gray
  51; writing strips are now set 2: ruled lines 1–3 px × grays 51, 102, 153, 187 (W1–W12) and square dots 2–4 px ×
  grays 51, 102 (W13–W18). Gray 221 dropped (invisible on the Nomad). Sheets regenerated in `calib/`; the old writing
  pages on the devices should be replaced.
- Calibration, Nomad text sheet recorded word for word (`docs/device-measurements/supernote.a6x2/calibration.md`):
  every sample legible (6–18 pt, grays 0–102); most comfortable TD2 (9 pt regular, gray 51) and TL2 (9 pt bold,
  gray 51); gray 102 a bit harder to read; black less legible than 51. Gray 51 is the favourite on every sheet so far.
- chris, word for word: "if I had to choose a label, TE3 and TM3 would be the best" (10 pt regular and bold, gray
  102). Recorded with the text sheet results.
- chris, word for word: "for labels, we can bump them up to row 2. They aren't bad, and I'd like to keep the two the
  same". Read as column 2: labels are TE2 and TM2 (10 pt, gray 51), the same gray as reading text; replaces TE3/TM3.
  Withdrawn: the proposal to split the profile's text gray into a reading gray (51) and a label gray (102), because
  chris wants the two the same.
- chris, word for word: "reverse that. the labels should be matched for size, but differ in shade of gray". Asked
  which labels, chris chose TD3 and TL3. Ruling: reading text TD2/TL2 (9 pt, gray 51); labels TD3/TL3 (9 pt, gray
  102). The previous entry's reading (labels at gray 51) was a misread and is reversed; the withdrawn proposal to give
  labels their own gray stands again.
- chris, word for word: "I'd like to consider a lighter shade of gray for labels, maybe 153?". The text sheet gains
  column 4, gray 153 (`TEXT_GRAYS` in `crates/xtask/src/calib.rs`); columns 1–3 keep their codes. With four columns,
  14 and 18 pt samples wrap onto two lines. `cargo test -p quadrule-xtask` → calib_codes expected=261 pass, all
  xtask CHECK lines pass; clippy clean. Sheets regenerated in `calib/`.
- chris, word for word: "td4 and tl4 work for lables". Ruling: labels are TD4/TL4 (9 pt, gray 153); reading text
  stays TD2/TL2 (9 pt, gray 51). Replaces TD3/TL3. Recorded in `docs/device-measurements/supernote.a6x2/calibration.md`.
- Calibration, Nomad writing sheet (set 2), chris, word for word: "for writing, all lines are visible. for dots, w13
  and w14 are not visible. W1 is my favorite line. W15 is my favorite dot". Favourite line W1 (1 px, gray 51, same as
  LA4); favourite dot W15 (square 3 px, gray 51). 2 px dots (W13, W14) invisible; checked the page: 60 dots each,
  exactly 2 × 2 px at the stated gray, so the dots sheet's PB2/PB3 showing is down to the samples being small,
  labelled clusters. The four Nomad sheets are done; Manta next.
- chris, word for word: "I do see them, they are just not usable at that size" (W13, W14). Correction to the previous
  entry: 2 px dots are visible but too small to use, not invisible; the explanation offered for the difference from
  the dots sheet is withdrawn (kept, struck through, in the calibration record). 3 px stays the smallest usable dot.
- Rulings, chris's answers word for word (options chosen): dot grid gets its own profile setting, "Yes, own setting
  (Recommended)" (Nomad: W15, 3 px square, gray 51); Nomad line roles, "Yes, use these (Recommended)" (Whisper LA8
  1 px/119, Guide LA4 1 px/51, Structure LB4 2 px/51, Emphasis LC1 3 px/0); toolbar positions used, "Docked left,
  Docked top" (L06 to warn only for these). Recorded in `docs/device-measurements/supernote.a6x2/calibration.md`.
- L06 checks only the toolbar positions in use (chris: "Docked left, Docked top"). `quadrule_lint::lint_with` takes
  `Options { toolbars }`; `lint` keeps every position. `cargo xtask lint` defaults to `left,top`
  (`--toolbars all|left,right,top,floating`); footers are always checked. A toolbar zone the linter cannot name is
  still checked, and `every_toolbar_zone_is_known` fails on one. Slice: 15 findings with all positions, 10 with
  left and top. `cargo test -p quadrule-lint -- --nocapture` → 7 passed, 7 CHECK lines pass; clippy clean.
- Names, chris's answers word for word (options chosen): the dot role is "Dot grid"; labels are "Text + new Label
  (Recommended)". Built: `LineRole::DotGrid` and `LineRole::Label` (`crates/model/src/lib.rs`); profiles gain
  `dot_grid` and `label_gray`; file format 0.3.0 with an identity step from 0.2.0 (`crates/format/src/migrate.rs`,
  test `a_0_2_file_opens_through_the_real_chain`). Napkin §8.2 gets a dated amendment note.
- Nomad profile calibrated (`crates/profiles/data/supernote.a6x2.ron`, `verified: true`, date 2026-11-09): Whisper
  1 px/119, Guide 1/51, Structure 2/51, Emphasis 3/0, Dot grid 3/51, text 51, label 153. Manta's new fields copy the
  Nomad's as estimates. The slice's dot grid uses the Dot grid role. Calibration sheet headers say "calibrated" for a
  verified profile. Test `estimates_are_marked_unverified` replaced by `verified_profiles_have_a_record` (a verified
  profile needs a date and its calibration record). Layout and render tests now tell shapes apart by form, as roles
  share grays on the Nomad. Slice goldens re-blessed (Manta 9df03397…, Nomad 26e04bab…).
  `cargo test --workspace -- --nocapture` → 65 CHECK lines pass, 0 fail; clippy clean.
- chris, word for word: "the golden sample looks good." PR #3 merged by fast-forwarding `main` to it (`f06f3a2`;
  the `gh` token cannot merge PRs); branch deleted. CI on #3 passed on Linux, macOS and Windows (run 42697273820).
- chris, word for word: "I also can report that the same preferences and findings on the nomad hold true for the
  manta. I think there displays are very similar." Manta profile takes the Nomad's values, `verified: true`, date
  2026-11-09 (`crates/profiles/data/supernote.a5x2.ron`); record in
  `docs/device-measurements/supernote.a5x2/calibration.md`. Manta golden re-blessed (b6f321e9…). Layout test
  `pattern_shapes_are_centred_in_their_region_and_drawn_whole` sorts shapes by form (Manta roles now share gray 51);
  it now counts 366 shapes (350 dots, 16 lines) where it had counted 734 by gray. `cargo test --workspace --
  --nocapture` → 65 CHECK lines pass; clippy clean. Calibration of role values is done on both devices.
- Pen shades: chris's Nomad screenshot (added as `docs/device-measurements/pen-shades.png`, moved to
  `docs/device-measurements/supernote.a6x2/nomad-pen-shades.png`; the safe-zone tool skips it by name). Black 1, dark
  grey 157, light grey 201, white 254. Both profiles' `ink_values` set to these (the Manta's assumed from the Nomad).
  New test `crates/xtask/tests/pen_shades.rs` re-measures the screenshot → measured=[1, 157, 201, 254], pass. The
  stock grid's gray 1 is the black pen's gray (README note). Found: the Label gray (153) is 4 steps from the dark
  grey pen (157); raised with chris.
- chris, word for word, on the Label gray (153) sitting 4 steps from the dark grey pen (157): "I'm not sure that
  it's all that bad to have template lines that match the pen ink". Label stays 153. This bears on L04 ("Gray or
  weight close to pen ink (will compete with handwriting)", Warn, napkin §8.3); its severity asked of chris.
- chris, word for word: "let's leave the 153/157 question as open". Recorded as an open item in
  `docs/STATUS-debt.md`; L04 waits on it.
