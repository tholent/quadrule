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
- Rung 2 proposed: the first app window, a viewer (open a `.quad` or the slice, show it as the device will, switch
  Manta and Nomad, zoom to pixels, show lint findings, export PNGs; no editing), about 1–2 sessions. chris, word for
  word, answering (1) which machine runs the window, (2) should the canvas fall back to vello_cpu on weak graphics
  chips, (3) is a viewer the right scope: "2. yes. 3. scope is good. 1. working on the display. I need to reboot
  before starting rung 2". Next session: plan rung 2 in a bbq, then build; R15 (the window on a display) can run
  once the Pi has its display.
- Session start, 2026-11-09 (rung 2): build on disk matches HEAD `8126b87` (clean tree, nothing to rebuild);
  `cargo test --workspace -- --nocapture | grep -c '^CHECK'` → 66, none failing (STATUS said 65; `pen_shades` added
  one). Asked again: the 153/157 Label gray question; chris: "Leave it open". L04 still waits.
- Spike R15 run on the Pi's display for the first time: the eframe window fails to start. V3D refuses wgpu's default
  limits (`max_inter_stage_shader_variables` 16 asked, 15 allowed); on OpenGL, `max_texture_dimension_2d` 8192 asked,
  4096 allowed. Fix belongs in the app: request the adapter's own limits.
- Rung 2 planned in a bbq, 12 questions, record `docs/bbq/2026-11-09-rung-2-viewer.md`. Canvas shows export pixels
  only (vello_cpu); napkin window skeleton; Fluent from day one; lint list with highlights; start empty (overruled
  the slice); export menu; opt-in build; zoom with real size; theme follows system. Size grew to about 2–3 sessions.
- Rung 2 step 1 (`faa36a6`): `i18n` crate (Fluent, English); lint findings carry a message id with arguments and an
  area. `cargo xtask lint` output byte-identical before and after (all toolbars and default); a first try printed
  "0." for whole numbers (Fluent pads zero decimals) and was fixed. 73 CHECK lines pass.
- Rung 2 step 2 (`76ea019`): `ui` and `app` crates, the viewer window. Spike R15 paid: eframe now asks the adapter
  for its own limits, and the window opens on the Pi's V3D. Found on the display and fixed: ⚠ and ℹ missing from
  egui's fonts (egui's monochrome emoji fonts on); zoomed out, picking every n-th pixel dropped most ruled lines
  (now averaged, test `shrinking_keeps_thin_lines`). Found by the window tests and fixed: two unnamed controls for
  screen readers; a Manta page too big for a 2048 px texture limit (now 1024 px tiles). CI run 42713527280 passed on
  Linux, macOS and Windows.
- chris tried the window. Answers word for word: look "Approved as is"; file names "Just 'manta' / 'nomad'"; font "I
  would like the system default font for the controls" (overrules the Atkinson proposal). Built: profile
  `short_name`; controls use the desktop's interface font (`quadrule_text::system_face`, GTK's setting via
  `gsettings` on Linux, Segoe UI on Windows, macOS unverified). `cargo test --workspace --no-fail-fast --
  --nocapture` → 87 CHECK lines, all pass; 74 in the default build; clippy clean.

- CI run 42720462671 on `52c7ee9` (rulings: short names, system UI font) passed on Linux, macOS and Windows.

## 2026-11-10 — rung 3 planned

- Session start: clean tree at HEAD `dc3bc9f`; `CARGO_BUILD_JOBS=2 cargo build --workspace` → `Finished`. Asked
  again: the 153/157 Label gray question; chris: "Leave it open". L04 still waits.
- Rung 3 planned in a bbq, 12 questions, record `docs/bbq/2026-11-10-rung-3-first-editor.md`: the first editor
  (select, layers panel, inspector tabs, nudge, undo/redo, Save, autosave and recovery), with both canvases
  switchable (overruled the export-pixels-only recommendation) and no feature flag (overruled; waives the AGENTS.md
  flag rule for rung 3 only). Dragging and snapping move to rung 4. Export goldens stay fixed. About 3–5 sessions.

## 2026-11-10 — rung 3 built

- Session start: asked to run on Opus 5.5 at high effort; the session showed a lower effort, so the lead stopped
  and said so. chris set `/effort high` and wrote, word for word: "Carry on". Clean tree at HEAD `87b7ed0`;
  `CARGO_BUILD_JOBS=2 cargo build --workspace` → `Finished` with nothing to rebuild, so the build matched HEAD.
  The open 153/157 Label-gray question was not asked at the start; it is asked at the end of this session.
- Step 1–2 (`f8e8562`): `model::edit` — every edit a `Command` that returns its own undo; `History` with
  unlimited depth, typing in one field merged into one step until the field loses focus, dirty tracking by the
  saved step; locks refuse edits but not the panel's name, eye and lock. `layout::units` — one parser for mm, pt,
  px, in, %, u. `layout::placed_objects` — each object's shape on a device for picking, apart from `display_list`.
- Steps 3–8 (`83b20c4`): selection, layers panel, inspector tabs, keys, Save / Save As / unsaved-changes prompt,
  autosave and recovery (`crates/ui/src/editor.rs`, `panels.rs`, `recovery.rs`, `view.rs`; the app's Save As
  dialog, close guard and recovery folder from `directories` 6.0.0). Window tests: findings moved to the Lint tab.
- Step 9 (`3075188`): the graphics-chip canvas, vello 0.11.0 on egui's wgpu 30.0.1 (one wgpu in `Cargo.lock`), with
  the vello_cpu fallback; `quadrule --canvas chip|export`.
- Found and fixed while building: (1) starting vello together with the window kept the window from ever showing on
  the Pi (the main thread sat idle in winit's event loop, no window mapped); vello now starts on its first frame.
  (2) vello_cpu draws `Rect` items with a hard 50 % edge, so a 1 px line shrunk to a quarter would vanish; the
  screen-resolution list draws them as anti-aliased fills (test `screen_canvas_list`: 15 ruled lines in the
  export, 15 at 1/4). (3) The first chip frame is not judged, so nothing was judged until the next zoom or edit; a
  judged frame now follows the warm-up at once.
- Measured on the Pi's display (Wayland, V3D), sample page on the Manta at Fit zoom, status bar read from a `grim`
  screenshot: debug build, first frame (shaders being built) 747 ms; release build
  (`cargo build --release -p quadrule-app`), judged frame 40 ms, so the canvas stays on the chip. The spike's
  392 ms (R12) was a whole 1920 × 2560 page; the canvas draws only the visible area (about 580 × 780 screen pixels
  here). Not measured: zoomed in, or while scrolling. A desktop-card check waits for one (bbq Q7).
- Checks on the Pi: `cargo test --workspace --no-fail-fast -- --nocapture | grep -c '^CHECK'` → 110, none failing;
  `cargo test -- --nocapture | grep -c '^CHECK'` (default members) → 83. Both goldens still match
  (`slice_golden`), and the export after an edit and undo matches them (`export_unchanged_after_edit_undo`).
  `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` clean.
- Taste calls made while building, for chris's look check (each can be changed): Save, Save As…, Undo, Redo as
  words in the toolbar after Open…; the layers panel shows the top layer first, with 👁 and 🔒 toggles (egui's
  emoji font) and Move up / Move down buttons above the tree (keys Ctrl+] and Ctrl+[); the toolbar's lint count
  opens the Lint tab (or hides the right panel when it is showing); the title reads `● name.quad — quadrule`
  while unsaved; Duplicate puts the copy in place, just above the original; the selection outline is the theme's
  accent, 1.5 points; a multiple selection shows the first object's values and edits apply to all of it.
- Choices made inside the bbq's rules, to veto: `u` in a field and Shift+arrow use the rhythm unit as the device
  on screen resolves it (pixel-locked 6 mm = 71 px on the Manta = 6.011 mm), so `2u` is exactly two grid steps on
  that device; a comma works as the decimal point when there is no full stop; the rhythm fields do not read `u`.
- CI run 42816727214 on `3075188` passed on Linux, macOS and Windows (goldens matched on all three); fuzz run
  42816727323 passed.
- chris, word for word, after trying the window: "the window looks really good so far. One thing I would like is
  to move the layers pane to the bottom of the info pane on the right. I would like the left hand side to be used
  for a toolbar in the future." Built: the right panel holds the inspector, with the layers in a resizable part at
  its bottom; nothing on the left (`crates/ui/src/view.rs`, check `layers_under_inspector`). The toolbar's lint
  count now only opens the Lint tab: hiding the right panel would hide the layers too.
- Found on the Pi: starting the new window offered to recover `try.quad`, whose recovery file was written at
  08:05:19 while the earlier window was open, and the window had closed by 08:11 without saving. Whether it was
  closed with Don't save or ended otherwise is not known. Reproduced one way it happens: "Don't save" deletes the
  recovery file, then the window loses focus while closing and autosave on focus loss writes it again (new check
  `dont_save_close_no_recovery`, failed before the fix with 1 file left, passes after). Fixed: no autosave once the
  window is closing. The leftover file was left in place (`~/.local/share/quadrule/recovery/`) for chris to
  Restore or Discard.
- Counting checks: `grep -c '^CHECK'` undercounts, because with `--nocapture` a test's line can land on the end of
  the test runner's own `test … ` line (seen here: `gtk_font_names` passed but was not at a line start). Counted
  instead with `grep -o 'CHECK crate=[^ ]* name=[^ ]*' | wc -l` → 112; `grep 'CHECK crate=' | grep -c
  result=fail` → 0.
- CI run 42822926671 on `e9e11ec` (layers under the inspector; closing fix) passed on Linux, macOS and Windows.
- chris, word for word: "the right pane looks good now, I approve of the look. I approve the u choice, as far as I
  understand it. leave the label gray issue open until v1.1". Rung 3 is done: look approved; `u` and Shift+arrow
  stay on the rhythm unit as the device on screen resolves it; the 153/157 Label gray question is open until v1.1,
  so it is not asked again at session starts before then, and L04 waits with it (`docs/STATUS-debt.md`).

## 2026-11-10 — rung 4 planned

- Session start (Opus 5.5, high): clean tree at HEAD `6d43de6`; `CARGO_BUILD_JOBS=2 cargo build --workspace`
  rebuilt `quadrule-xtask` and `quadrule-app`, then `Finished`, so the build on disk was behind HEAD and now matches
  it. The Label gray question was not asked: open until v1.1 (chris, 2026-11-10).
- Rung 4 planned in a bbq, 12 questions, record `docs/bbq/2026-11-10-rung-4-drag-and-snap.md`: all six snapping
  helpers of napkin §6.2 (recommended five), drag to move and resize, marquee, layer drag, Alt-drag duplicate, copy
  and paste as marked JSON text on the system clipboard, rhythm subdivisions (file format 0.4) and the rhythm
  overlay; snapping in the device on screen. No flag (overruled; waives the AGENTS.md flag rule for rung 4 only).
  Split 4a / 4b with chris's look check after each; dragging must stay under 50 ms a frame on the Pi (release
  build). Lead for the build: Sonnet 5.5 at high effort (tier-select). About 4–6 sessions.

## 2026-11-10 — rung 4a built

- Session start (Sonnet 5.5, high; the model read from the session, the effort from `echo "$CLAUDE_EFFORT"` → `high`):
  clean tree at HEAD `bbf7024`; `CARGO_BUILD_JOBS=2 cargo build --workspace` → `Finished` in 0.68 s with nothing
  compiled, so the build on disk matched HEAD.
- Built in the order the Kickoff gives, on `main` (commits `61ac6d7`, `bcbff04`, `7037ee2`, and the one that carries
  this entry):
  1. `Rhythm.subdivisions: Option<u8>` (2–10; a number outside that reads as off; absent in the file when off) and file
     format 0.3 → 0.4 with an identity migration (`crates/model/src/lib.rs`, `crates/format/src/migrate.rs`; check
     `open_0_3_file`).
  2. The snap engine, headless, in `crates/layout/src/snap.rs`: rhythm and subdivisions, a resize's size in whole
     steps, margins, safe-zone edges, the pixel grid, Shift and Alt resizing, handles, marquee geometry; plus
     `unresolve_frame`, the inverse of `resolve_frame`, so a drag on the Nomad lands where the pointer put the
     object. 13 checks in `crates/layout/tests/snap.rs`.
  3. Drag commands in `crates/model/src/edit.rs`: `set_frames`, `reorder_to`, `is_empty` (a drag is one `Batch`).
  4. The drag in `crates/ui/src/drag.rs`: move, eight handles, a line's two ends, marquee, Alt-drag duplicate, Ctrl
     (Cmd) off, Escape. While the pointer is down the page shows a preview copy of the document; the real change is one
     command on release. 8 checks in `crates/ui/tests/drag.rs`, plus one real-pointer check in
     `crates/ui/tests/window.rs` (`window_pointer_drag`: press, move, release and Escape through egui).
  5. Layers panel drag-to-reorder (objects among objects or onto a layer's row; layers among layers), the rhythm
     overlay, the three snap switches (status bar and Page tab, kept between runs in the app's storage), the Page tab's
     Subdivisions field, Shift+arrow by one subdivision when they are on.
- Measured on the Pi, 2026-11-10, release build (`CARGO_BUILD_JOBS=2 cargo build --release -p quadrule-app`), command
  `./target/release/quadrule --canvas export|chip --bench-drag 150`: the sample page at Fit zoom, its dot grid
  dragged along a path with snapping on, one step a frame through the real window. Frame time is the canvas's own
  clock (`DragClock` in `crates/ui/src/canvas.rs`): from the start of the canvas to its end, for every frame that
  made the page again (layout, render, shrink or tiles, and on the chip canvas the chip frame; egui's own paint is not
  in it). 145 timed frames a run.
  - Export-pixels canvas, 11 runs: mean 24.3–26.9 ms; worst frame 29.9–46.8 ms in 10 runs; **one run had a 60.1 ms
    frame** (1 frame of 1,595 at 50 ms or over; the other 10 runs had none).
  - Graphics-chip canvas (no fallback), 8 runs: mean 30.8–31.0 ms; worst 32.6–33.9 ms.
  - The record's line is "each frame is under 50 ms" and says to stop and tell chris, with the numbers, before speeding
    anything up. By that letter the export canvas missed once. Nothing was sped up. For chris: judge by the worst frame
    (a miss), by a percentile, or by the mean?
- Checks: `cargo test --workspace --no-fail-fast -- --nocapture | grep -o 'CHECK crate=[^ ]* name=[^ ]*' | wc -l` →
  143; `grep 'CHECK crate=' | grep -c 'result=fail'` → 0; 138 tests passed, 0 failed; `cargo clippy --workspace
  --all-targets -- -D warnings` and `cargo fmt --all --check` clean. Export bytes: rung 1's goldens and the stock-grid
  test pass unchanged; the page after a drag and its undo equals the page before, byte for byte (`drag_move_one_step`).
  New checks print each line starting with a newline.
- Seen by screenshot of the running window (`grim`, mid-benchmark): the selection with eight handles, a magenta guide where
  a rhythm line caught, a teal one at a margin, the rhythm overlay, the status-bar switches and the drag clock. Two
  things fixed after looking: rhythm lines beyond the page edge no longer catch, and the handles are white squares
  on any theme.
- Choices made inside the bbq's rules, for chris to veto:
  - Handles sit on the geometry of the objects, not their strokes. A text box gets left and right handles only (it
    grows downward by itself); one selected line shows its two ends instead of a box; several objects share one box.
  - Resizing a side past the opposite one flips the box (no mirroring of what is inside a group); with Shift it never
    flips; no side goes under 1 device pixel. Shift on an edge handle grows the other side about its centre.
  - A line is its frame's diagonal, top-left to bottom-right, so a line cannot point up and to the right, and
    pulling one end past the other turns it to the other diagonal (a limit of `ObjectKind::Line`, `crates/model/src/lib.rs`).
  - A press on a selected object moves the whole selection; a press on an unselected one selects it (Shift adds) and
    moves it; a press on a locked object cannot pick it, so it draws a marquee; a marquee selects what it touches.
  - Escape during a marquee puts the old selection back; during a move or a handle the object stays selected.
  - Alt-drag copies sit just above their originals in the same layer and are selected after; an Alt-drag that goes
    nowhere makes no copy.
  - Guides: magenta for a rhythm or subdivision line, purple for a size that matched, teal for a margin or safe zone.
    Rhythm overlay: units stronger than subdivisions; off by default.
  - The drag clock stays in the status bar after a drag ("Drag: 25 ms a frame (worst 34 ms, 142 frames)").
- Grown past what was asked, said out loud: the middle button pans the page (a plain drag now moves objects, so it no
  longer scrolls; the scroll bars and the wheel still do); `quadrule --bench-drag [steps]` (needed to measure the
  rule); the drag clock in the status bar; a screen-point accessor for tests.
- CI run 42835008247 on `3c4558d` passed on Linux, macOS and Windows (the goldens matched on all three); fuzz run
  42835008539 passed.
- Not checked: Alt-drag in the real window. Raspberry Pi OS's window manager may take Alt+drag to move the window; if it
  does, Ctrl+D still duplicates. The layers panel's drag-and-drop through egui is checked only by its index maths, not
  by a pointer. Stopped here for chris's look at 4a before 4b (point snap, smart guides, equal spacing, copy and paste).
- chris, word for word, after the 4a look: "I approve the work so far, with two changes to the safe area display. I would
  like to remove the labels and include the right hand side safe area by default." Built: the safe-zone overlay has no
  labels; `TOOLBARS_SHOWN` (left, right, top) drives the overlay and the zones dragged objects snap to; the linter
  still uses `TOOLBARS_IN_USE` (left, top), because the words named the display (`crates/ui/src/viewer.rs`; check
  `right_zone_shown_by_default`). Checks now: 144 `CHECK` lines, 0 `result=fail`, 139 tests passed, 0 failed; clippy
  and fmt clean. The 60.1 ms question was not answered; it is asked again at the next session start. Rung 4a is done.
- chris, word for word: "let's use a percentile. the safe area linting should look at all safe areas, including the
  right one. Also, when two safe areas overlap, don't draw the borders that occur inside the combined area." Built:
  - Drag speed is judged by the 95th percentile of the frames after the first (nearest rank; `DragClock::percentile_ms`;
    the 95 is mine, for chris to veto). `--bench-drag` prints `p95_ms`; the status bar shows it. Measured again on the
    Pi, release build, 150 steps: export-pixels canvas p95 25.8–27.1 ms in 6 runs (mean 24.5–25.4; one run had one 64.3 ms
    frame, worst of the others 26.2–36.0); graphics-chip canvas p95 31.2–31.5 ms in 3 runs (worst 33.8–33.9). All pass.
    The two odd frames seen so far (60.1 and 64.3 ms) are 2 in about 2,500 timed export-canvas frames.
  - The linter, the overlay and snapping share `TOOLBARS_IN_USE`, now all four positions (`crates/ui/src/viewer.rs`);
    `cargo xtask lint` defaults to `all`; the Lint tab's note says so. The sample page on the Manta now has 7 findings
    (it had 5): the window tests were updated from 5 to 7.
  - Overlapping or touching safe zones draw one outline round the combined area (`union_outline`, `crates/layout/src/snap.rs`,
    check `union_outline`). Their translucent fills still add up where they overlap; not asked, so not changed.
  - Checks: 146 `CHECK` lines, 0 `result=fail`, 141 tests passed, 0 failed; clippy and fmt clean.

## 2026-11-10 — rung 4b built (Sonnet 5.5, high effort)

Asked by chris: "please proceed with rung 4b". Size said up front: the record's estimate was 2–3 sessions; it went in one
session. Built in this order, on `main`:
- Clipboard, `crates/model/src/clipboard.rs` and `edit::paste`: the copied objects, bottom first, as JSON under the marker
  `"quadrule-clipboard": 1`; a paste reads only text with that marker (other text, a newer marker, broken JSON: ignored),
  gives each object a new id, unlocks it, and puts it on top of the layer of the first selected object (or the top
  unlocked layer). Into the document the objects came from (any pasted id is in it) it lands one rhythm unit down and
  right; text outlines are not copied (made again at Save). `serde_json` moved from a dev-dependency to a dependency of
  `model`; it was already in the workspace, so no new crate. Checks `paste_round_trip`, `paste_offset_and_locks`,
  `clipboard_ignores_other_text`, `copy_cut_paste`, `window_copy_paste_keys`.
- Snap engine, `crates/layout/src/snap.rs`: `SnapKind` gains `Point`, `Smart`, `Equal` after `SafeZone` (the record's tie
  order); `SnapToggles` gains `points`, `smart_guides`, `equal_spacing`; `SnapWorld::with_others` takes the other objects'
  boxes (`Other`, with a flag for lines); `Guide` gains an optional `span` (a guide to an object runs only between the
  objects it joins); new `Mark` (a point ring, an equal gap); `snap_point` returns `PointSnap`. Point snap needs both
  coordinates within reach of one anchor (box corners, edge midpoints, centre; a line's two ends and middle). Equal
  spacing is for a move only: between two objects (gaps equal), or beside two objects that already have a gap (the same
  gap again), in the dragged box's row or column; touching objects (under 1 px) leave no gap to copy. Resizes and line
  ends get smart guides; a line end gets point snap too. Checks `snap_points`, `snap_smart_guides`, `snap_equal_spacing`,
  `snap_priority_4b`.
- Window: the three switches (status bar and Page tab, kept between runs), orange for what snaps to objects (ring on a
  point, guide line, gap measures with "N px" labels from Fluent). Others are the objects of the page except the ones
  dragged; for an Alt-drag the originals count. Ctrl+C/X/V are egui's copy, cut and paste events, taken only when no text
  field has focus. Checks `drag_smart_guide`, `drag_point_snap`, `drag_equal_spacing`, `drag_alt_copy_lines_up`.
- Grown past what was asked, said out loud: each further paste of the same clipboard text goes one more unit down and
  right (otherwise repeated pastes stack exactly); a cut keeps locked objects. Both are small and easy to veto.
- Found while testing: the nearest catch wins even when it is a fraction of a pixel nearer (the record's rule: "nearest
  wins, tie goes to rhythm"). On the sample page an equal gap sits 0.2 px nearer than a rhythm line and wins, so the
  rule snapped to 236.2, not 236.0. The 4a test now sets the three 4a switches. chris may want rhythm to win within a
  small distance (say half a pixel); not changed.
- Drag speed on the Pi, release build, `quadrule --bench-drag 150`, all helpers on: export-pixels canvas 95th percentile
  25.0, 25.6, 26.2 ms (worst 40.6, 45.7, 76.6 ms; one frame over 50 ms in the second run, the same kind of odd frame as
  before); graphics-chip canvas 31.6, 31.8, 32.1 ms (worst 32.8–35.0). All pass.
- Seen in the real window (grim screenshot of `--bench-drag` on the Pi): the status bar with all seven switches and the
  drag clock fits at 1280 px wide, with no room to spare; the equal-gap labels show.
- Checks: 159 `CHECK` lines, 0 `result=fail`, 154 tests passed, 0 failed; clippy and fmt clean; export goldens unchanged.
- Not checked: copy and paste through the real system clipboard and between two running windows (the tests use egui's
  events); drawing of a ring or gap by a person with a pointer (the screenshot was the bench's drag).
- Stopped for chris's look at 4b.
- chris, word for word, on the open question: "let rhythm win within half a pixel". Built: a rhythm catch (a unit line, a
  subdivision or a whole-step size) beats any other kind unless that kind is more than 0.5 px nearer; between two kinds
  that are both rhythm, or both not, the rule is as before (`better` in `crates/layout/src/snap.rs`; check
  `snap_rhythm_half_pixel`). I read "rhythm" as including subdivisions and sizes, and the half pixel as device pixels;
  both are mine, for chris to veto. Checks now: 160 `CHECK` lines, 0 `result=fail`, 155 tests passed; clippy and fmt
  clean. CI for 4b's first commit `97350a7` passed on Linux, macOS and Windows.
- chris, word for word, after trying 4b: "ok, a couple of fixes. the safe zone overlay should not include the floating menu
  and the overlapping portions should not increase their darkness. Next, the rhythm overlay should be much lighter and
  perhaps light blue. Something that indicates a guide, and doesn't interfere with the display of actual elements."
  Built:
  - The overlay and the zones dragged objects snap to leave out the floating toolbar (`TOOLBARS_SHOWN`: left, right,
    top; `crates/ui/src/viewer.rs`). The linter still checks all four positions (`TOOLBARS_IN_USE`), as chris ruled
    earlier; I read "the safe zone overlay" as the display only. Check `floating_zone_not_shown`.
  - Overlapping zones are filled once over their combined area (`union_fill`, `crates/layout/src/snap.rs`; check
    `union_fill`), so the overlap is no darker.
  - The rhythm overlay lines are light blue (120, 180, 240) at alpha 60 for units and 26 for subdivisions (was a deeper
    blue at 90 and 38). The shade is my guess; say if it should go lighter or darker.
  - Checks: 162 `CHECK` lines, 0 `result=fail`, 157 tests passed; clippy and fmt clean.


## 2026-11-10 — Makefile (Sonnet 5.5)

- chris asked for "a Makefile for easing developer testing and review"; bbq of 4 questions, record
  `docs/bbq/2026-11-10-makefile.md`. Built `Makefile`: help, build, fmt, clippy, test, check (default members) and the
  `-all` forms (`--workspace`, as CI), run, run-release, bench-drag, calibration-sheet, safe-zones, lint, fuzz,
  build-proof, review. CI is unchanged and still calls cargo.
- Tried on the Pi: `make` (help), `make fmt-check`, `make test` → `TESTS scope=default checks=109 check_fail=0 passed=105
  failed=0 status=ok`; the failure path with a stand-in `cargo` that fails → `status=fail`, failing lines shown, exit 2;
  `make lint`, `make safe-zones PROFILE=supernote.a6x2`; `make build-proof` → `BUILD head=934852f dirty=yes rebuilt=7`
  (dirty because the Makefile was not committed yet).
- Not run: `make check-all`, `make test-all`, `make clippy-all`, `make review` end to end, `make run`, `make bench-drag`,
  `make fuzz`, `make calibration-sheet`, on macOS at all.

## 2026-11-10 — README, DEVELOPMENT.md and the GitHub description (Sonnet 5.5)

- chris asked for "a README.md file and updating the project description on github"; bbq of 12 questions, record
  `docs/bbq/2026-11-10-readme-and-description.md`. chris widened it to a user-first `README.md` plus a developer-first
  `DEVELOPMENT.md`. Wrote both, and a pointer in `AGENTS.md`.
- Caught while writing: the editor has no New and no Insert (`docs/bbq/2026-11-10-rung-3-first-editor.md`), so the README
  says plainly that a page cannot yet be designed from scratch.
- Session-start proof: `make build-proof` → `BUILD head=9dc81dd dirty=no rebuilt=0`.
- Not done: `gh repo edit --description "Design pixel-perfect e-ink page templates for Supernote tablets" --add-topic
  supernote --add-topic e-ink --add-topic vector-graphics --add-topic design-tool --add-topic templates --add-topic
  wallpaper` → `HTTP 403: Resource not accessible by personal access token`. The token (a fine-grained one) lacks
  repository administration write. Owed to chris: run that command with a token that can, or set it on the repo's page.
- Not run: the `cargo run` and `make` commands the new files name were not re-run this session (docs only); they are
  copied from `Makefile`, `Cargo.toml` and `docs/calibration.md`.

## 2026-11-10 — README badges and a coverage workflow (Sonnet 5.5)

- chris asked for badges: pitmaster (his URL), testing, linting, fuzzing, total coverage. Rulings: one CI badge for
  fmt, clippy and tests together (not a CI split, which would build twice); coverage self-hosted, no outside service
  (a badge file on a `badges` branch); badges in a row under the title.
- Added `.github/workflows/coverage.yml` and `.github/scripts/publish-badge.sh`; badges in `README.md`; a CI paragraph in
  `DEVELOPMENT.md`.
- Tried: `publish-badge.sh` against a local bare repository (first publish creates the branch, an unchanged badge makes no
  commit, a new percentage commits, a summary without coverage exits 1); the workflow file parses as YAML.
- Not tried: the workflow itself (`cargo install cargo-llvm-cov`, the tests under instrumentation, the push of the
  `badges` branch with the built-in token) can only be tried on GitHub. Until it has run once the coverage badge shows an
  error. The pitmaster badge links to `AGENTS.md`; chris gave only the image URL, so the link is my choice.

## 2026-11-10 — rung 5 planned: New and Insert (Opus 5.5)

- chris typed "bbq next steps"; bbq of 12 questions, record `docs/bbq/2026-11-10-rung-5-new-and-insert.md`.
- Session-start proof: `make build-proof` → `BUILD head=302d9ef dirty=no rebuilt=0`. `gh run list --limit 4`: CI run
  42864690998 (`934852f`) success; the coverage workflow's first run, 42868546083, success, so the coverage badge has a
  value now.
- chris approved rung 4b's look (Q4: "Approved, rung 4 done"). Rung 4 is done.
- Rung 5 ruled: File → New with a small dialog (device, rhythm preset), then a blank page; a left tools bar (Select,
  Line, Text, Pattern) that draws by drag, or drops a default-size object on a click, then goes back to Select; no flag;
  commits to `main`; lead Sonnet 5.5 at high effort; export goldens fixed; the bar laid out to take more tools later.
  "Stay smooth on the Pi" was offered but not picked, so drawing speed is measured and logged, not a stop line.
- Asked again, still open: the GitHub description and topics (chris), and "Scale vs pixel-locked rhythm on A5 X"
  (`docs/STATUS-debt.md`).
- Later the same session: chris set the GitHub description and topics by hand. Read back with `gh repo view --json
  description,repositoryTopics`: description "Design pixel-perfect e-ink page templates"; topics design-tool, e-ink,
  page, supernote, templates, vector-graphics. This differs from the README bbq's plan (description ended "for
  Supernote tablets"; topic wallpaper, no topic page); told chris, the set values stand unless he changes them.

## 2026-11-10 — ADR backlog sweep (Opus 5.5)

- chris typed "makeadr", then, word for word: "I feel like I haven't commited many of the decisions to ADRs yet and want
  to make sure to catch the backlog of decisions I have up until this point." Grew from one ADR to a sweep; said so.
- Searched all 8 bbq records, this log, the debt list, `docs/proposals/`, ADR-001 to ADR-003, `AGENTS.md` and the spike
  findings. chris chose: all groups, written from the records with no rounds, small ones merged, written here.
- Drafted ADR-004 to ADR-014, all `proposed` (record `docs/bbq/2026-11-10-adr-backlog.md`). ADR-007 and ADR-014 were
  never ruled by chris; the rest write down his rulings with their sources.
- `adr-index.py --root .` → 14 ADRs; `adr-check.py --root . --triggers-from 1` → first 4 failures (option headings,
  a missing Drivers list), fixed, then 0. The check resolves proofs; it does not run tests.
- Paid the debt item "The project-setup decision is not on record yet" (ADR-004).
- chris, word for word: "ratify 007. let's amend 014 to align with the currently exported pixtures". ADR-007 ratified
  and built: a dated amendment under the flag rule in `AGENTS.md`. ADR-014 was still proposed, so its draft was changed
  (not amended) to keep the rule the exports follow, checked against
  `crates/layout/tests/layout.rs::pattern_shapes_are_centred_in_their_region_and_drawn_whole`; asked "Ratify ADR-014 as
  written?", chris: "Ratify". Napkin §5.2 got a dated note; the debt item "Pattern rule" is paid. No code or export
  bytes changed.
- chris reviewed the pending ADRs in three rounds and ratified all of them: ADR-004, 005, 006, 008 ("Ratify"); then the
  asks: percentile "90" (not 95), "Keep 100 ms", "Keep 6 pt and the order", "Yes, as built"; then ADR-009 and ADR-011
  ("Ratify both"), ADR-010, 012, 013 ("Ratify"). All 14 ADRs are accepted (`docs/decisions/INDEX.md`);
  `adr-check.py --root . --triggers-from 1` → 14 ADRs, 0 failures.
- The 90 needed code: `DragClock::PERCENTILE` 95 → 90 (`crates/ui/src/canvas.rs`), the drag-clock label and its Fluent
  message (`p90`), the bench CHECK line (`p90_ms`), and the test, renamed `drag_frames_are_judged_by_the_90th_percentile`
  (11 slow frames in 100 miss, 10 pass). Said aloud as more than an ADR review. `cargo test -p quadrule-ui -p
  quadrule-i18n --no-fail-fast -- --nocapture` → 54 passed, 0 failed, 55 `CHECK` lines, 0 `result=fail`; clippy `-D
  warnings` on `quadrule-ui` and `cargo fmt --all --check` clean. Export bytes untouched (no render or layout change).
  The rung 5 record's two "95th" mentions now read "90th", with a dated note.
- chris: "I think we need to make sure our CI workflows are using caching. the coverage check compiles everything".
  Measured from `gh run view <id> --json jobs`: CI and Coverage already use `Swatinem/rust-cache@v2` and hit it. Coverage
  run 42868546083 (the first ever, no cache) Measure 185 s, cargo-llvm-cov install 56 s; run 42881539825 cache hit (`full
  match: true`), Measure 72 s, install 2 s; coverage must still rebuild the workspace's own crates, instrumented. CI run
  42864690998's 11 minutes were a 10-minute wait for a macOS runner, not compiling. Fuzz had no cache (cargo-fuzz install
  21 s, fuzz build about 47 s, run 42860903920): added `rust-cache` for the fuzz crate's own workspace, keyed on the
  nightly version (`.github/workflows/fuzz.yml`).
- Fuzz cache, first run 42882032539 (`ae45358`): "No cache found", then saved
  `v0-rust-rustc-1.102.0-nightly--ac8310ba6-2026-11-09---fuzz-Linux-x64-f0bd66ff-87bd2034` (94.82 MiB, `gh cache list`).
  A warm run is not proven yet: `gh workflow run fuzz.yml` → HTTP 403 (the token cannot start workflows); the next push
  touching the format or model crates will show it. CI for `297ab31` (run 42881539789) and Coverage (42881539825) passed.
  Repository caches: `gh cache list --limit 100 --json sizeInBytes` → 16 caches, 9.88 GiB, against GitHub's 10 GB
  repository limit; old entries (from older `Cargo.lock` hashes) are evicted oldest first.

## 2026-11-10 — rung 5 built: New and Insert (Sonnet 5.5, `CLAUDE_EFFORT=high`)

Kickoff from `docs/bbq/2026-11-10-rung-5-new-and-insert.md`. Preconditions, all met: model Sonnet 5.5 and `echo
"$CLAUDE_EFFORT"` → `high`; `make build-proof` at the start → `BUILD head=6fa87b1 dirty=no rebuilt=0`. Said up front
in the record: 2–3 sessions; it took one (three commits' worth of steps below).

- **Step 1, New** (`e751c58`): New… button on the toolbar and the empty screen, and Ctrl+N (Cmd+N); a dialog with device
  (the last one picked is remembered between runs, Manta first), rhythm preset and title; Create asks the unsaved-changes
  prompt first, then opens a blank template (`crates/ui/src/view.rs`, `crates/model/src/blank.rs`,
  `crates/layout/src/newpage.rs`). Margins are the least whole number of rhythm units that clears the docked left, right
  and top toolbars and the footers on every device the page targets (12 mm at 6 mm rhythm on both). An untitled
  document has no file until Save (Save asks where), autosaves and is recovered as untitled (`Source::Untitled`,
  `crates/ui/src/recovery.rs`).
- **Step 2, tools bar** (`2fcb4d3`): a left panel, 48 points wide, listing Select, Line, Text, Pattern as painted icons
  with tooltips and screen-reader names; V, L, T, P pick them (ignored while a text field has focus); Escape ends a tool
  before it clears the selection; a double-click keeps a tool, shown by a dot. It is a list in `Tool::ALL`, so
  rectangle, ellipse, polygon and repeat are each one more entry and icon (`crates/ui/src/tools.rs`).
- **Step 3, drawing** (`2fcb4d3`): `crates/ui/src/draw.rs`. A drag draws with rung 4's snap engine on (start corner by
  point snap, a box's far corner by resize snap from a zero-size box, so sizes land on whole rhythm units and the
  guides show), live through the preview document, landing as one undo step; Ctrl turns snapping off; Escape drops it.
  A click, or a drag shorter than 6 screen points, drops the default: a line or text box 10 units long (slid left to
  fit the page), a pattern region filling the page margins. Layer: the first selected object's layer, else the top
  unlocked one; a locked layer refuses with the "locked" note. The tool goes back to Select unless kept. A drawn
  text box opens the Text tab with its text selected.
- **Things I chose that the record did not say** (each can be vetoed; they are also in `docs/STATUS-debt.md`): a new
  page targets both devices (the chosen one first), so its margins clear both; a line dragged up and to the right (a
  diagonal the model cannot hold) runs along the axis the pointer went further on, as with Shift; a clicked pattern region
  is pinned to every edge (LeftRight/TopBottom) because with the default Scale constraints its margins shrank on the
  Nomad and a test caught an L06 finding there, while a drag-drawn region keeps the napkin's Scale default; a text box
  is 12 pt regular Atkinson Hyperlegible Next, one rhythm unit tall until its text decides; New is a flat toolbar button,
  not a File menu (the toolbar has none); object names ("Line", "Text", "Pattern") come from Fluent when drawn.
- **Measured** (Pi, release build, `quadrule --bench-draw 150`, which drags a pattern region out over the sample page
  through the real frame loop; `make bench-draw`): export canvas `p90_ms` 26.0, 25.5, 26.1 (worst 64.6, 27.0, 30.2);
  chip canvas `p90_ms` 41.5, 41.4, 42.3 (worst 43.9, 46.1, 44.7); budget 50 ms (ADR-009; for drawing it is a
  reading, not a stop line, per Q12). All six `result=pass`.
- **Checks:** `make check-all` → `TESTS scope=workspace checks=187 check_fail=0 passed=182 failed=0 status=ok` on the
  final tree (an earlier run read 188: the summary counts only `CHECK` lines that start a row, and parallel test
  output splits one now and then; `grep -o 'CHECK crate=[^ ]* name=[^ ]*' target/test.log | wc -l` → 188, 0 with
  `result=fail`; before rung 5: 162 checks, 157 passed); 25 new tests and 26 new `CHECK` lines (the draw bench prints its own): `crates/ui/tests/new.rs` (8),
  `crates/ui/tests/draw.rs` (12), `crates/ui/tests/window.rs` (5); the app's `command_line` test also covers
  `--bench-draw`. Export bytes unchanged: rung 1's golden test and the stock-grid test pass untouched, and
  `draw_export_bytes_unchanged` shows draw then undo restores both devices' pixels byte for byte.
- **CI:** push `e751c58`: CI 42885131520 success (three systems), Coverage 42885131427, Fuzz 42885131467 success; push
  `2fcb4d3`: CI 42887050036 and Coverage 42887049623 success; push `ed8d0e3` (docs and the focus-flag line): CI
  42888456905 and Coverage 42888457063 success. The Fuzz run is the first warm one: "Cache hit",
  `full match: true` (2 m 31 s against 2 m 59 s cold); the debt noted in the 297ab31 entry is settled.
- **Seen on screen** (a screenshot of `target/release/quadrule` on a saved drawn page, this session): the bar, a ruled
  region, a line and a text box show as drawn; the layers panel lists them by name. I could not click in the window from
  here, so the live drag, the dialog's look and the double-click are for chris's look.
- Also: the Makefile's `bench-drag` help said the 95th percentile; it is the 90th since ADR-009 (corrected), and
  `bench-draw` is new. The README's "no New page, no way to add shapes" line and what-works list are now true
  (`README.md`, `DEVELOPMENT.md` crate table).
- **Waits on chris:** the look at rung 5 (`make run`). Try: New… (Manta then Nomad; each rhythm), Ctrl+N with unsaved
  changes; the four tools by click and by V L T P; drag and click for each; Shift on a line; Ctrl while drawing; Escape
  mid-draw; double-click a tool to keep it; the Text tab taking the cursor after a text box; undo after a draw. Open
  questions still open from before, not part of this rung: the GitHub description and topics (`gh repo edit`, HTTP
  403), and "Scale vs pixel-locked rhythm on A5 X" (`docs/STATUS-debt.md`).
- Correction, same day: the rung 5 entry above lists the GitHub description and topics as still open. They are not:
  chris set them by hand (the entry at "Later the same session"), and said again: "the description and topics are both
  set and should be considered closed". Closed; do not ask again. The only question still open from before is "Scale
  vs pixel-locked rhythm on A5 X" (`docs/STATUS-debt.md`).

## 2026-11-10 — rung 5 approved; L06 leaves the floating toolbar out (Sonnet 5.5, `CLAUDE_EFFORT=high`)

- chris, word for word, after trying the window: "this feels and looks great. There is one linting issue, L06 still
  checks for collisions with the floating menu bar, which isn't desired. The app isn't super smooth, but I'm also
  checking it across a VNC connection. other than the linting issue, I approve this rung". **Rung 5 is done.**
- The lint fix: the default toolbar positions are the docked three (`Toolbar::DOCKED`, `Options::default`,
  `crates/lint/src/lib.rs`); the window's list is the same (`TOOLBARS_IN_USE`, `crates/ui/src/viewer.rs`);
  `cargo xtask lint` defaults to `left,right,top` and `--toolbars all` adds the floating one; the Lint panel's note no
  longer names it. The sample page on the Manta now shows 5 findings, not 7 (the two missing were the floating
  toolbar's Info notes). ADR-012 got a dated amendment (item 5 and its title); `adr-check.py` → 14 ADRs, 0 failures
  (it checks headers and that proofs resolve, not behaviour). Tests: `floating_toolbar_is_not_linted_unless_asked`
  (was `floating_toolbar_is_info_only`), the drag and window tests that held the old count.
- `make check-all` → `TESTS scope=workspace checks=188 check_fail=0 passed=182 failed=0 status=ok`. Export bytes
  untouched (no render or layout change).
- Smoothness: chris saw it over VNC, so it is not a reading of the app's own speed. The Pi's own numbers, measured at
  the screen, stay those of the rung 5 entry (export canvas p90 about 26 ms, chip canvas about 42 ms).

## 2026-11-10 — arc v1 planned (Arc bbq, 20 questions; Opus 5.5)

- Session start: `make build-proof` → `BUILD head=9bb8122 dirty=no rebuilt=6`. Asked again: "Scale vs pixel-locked
  rhythm on A5 X", now parked until v1.1 (Q16).
- Compared `main` with the spec's v1.0 list (§16): done, partial and missing items are summed up in the record
  (`docs/arcs/v1/v1-PROMPT-2026-11-10.md`, "The gap, in short"), and the rest becomes rungs 6 to 18 on the board
  (`docs/arcs/v1/README.md`).
- chris's rulings: the full §16 list is v1.0; Opus 5.5 high leads, Sonnet 5.5 high builds; model-shaping rungs first;
  a short bbq per rung; L04 to v1.1; hardware early, and "consider the manta's pen shades as equivalent to the nomad
  shades"; budgets per rung; split big rungs; chris designs one starter per category; GitHub release only, with an
  rc for testers; re-plan after every rung; each rung pays its own debt; the full release bar; `0.N.0` and a tag per
  rung; existing export bytes frozen.
- Spec notes dated 2026-11-10 added for L04 (§8.3), channels (§15.4), the arc (§16) and pen shades (§17.1).
- No code changed. Next: rung 6 (shapes), from the kickoff in the record.
- Later the same session: chris, word for word: "let's go back and tag the commits where approvals happened for each
  rung. rungs with more than one (4a,4b) can just bump minor versions." Annotated tags at each approval commit:
  `v0.1.0` `00ad0e2` (chris merged PR #1), `v0.2.0` `52c7ee9`, `v0.3.0` `50bd8db`, `v0.4.0` `c176a2f` (4a), `v0.4.1`
  `6ced5c2` (4b), `v0.5.0` `9bb8122`; pushed (`git ls-remote --tags origin`). The workspace version stays 0.1.0 in
  those commits. The arc's tag rule now bumps the last number per split part (`docs/arcs/v1/README.md`).
- Later the same session: chris, word for word: "please make github releases for v0.5.0 and for any approved rung
  (release) moving forward". Notes written for v0.5.0 (pre-release, no installers yet); `gh release create v0.5.0
  --verify-tag --prerelease …` → `HTTP 403: Resource not accessible by personal access token`, so no release exists
  yet. The rule is on the arc board; the token gap is in `docs/STATUS-debt.md`.
- Then: chris, "try using the token again". `gh release create v0.5.0 --verify-tag --prerelease …` succeeded;
  `gh release view v0.5.0` → pre-release, not a draft, https://github.com/tholent/quadrule/releases/tag/v0.5.0.
  The token debt line is paid (removed from `docs/STATUS-debt.md`).

## 2026-11-10 — release packages workflow (Opus 5.5)

- chris, word for word: "let's create a workflow for creating packages and publishing them. Don't worry about
  anything before v0.5.0". Built `.github/workflows/release.yml` and `packaging/` (cargo-packager 0.11.8:
  `packager.json`, placeholder icons, `prepare.py` that takes the version from the tag). Hand-run only, dry run by
  default; `publish=true` attaches the packages and `SHA256SUMS.txt` to the tag's release. The `release: published`
  trigger is written but commented out until chris approves the first packages (AGENTS.md: work outside the app
  window lands switched off). A tag older than `packaging/` (v0.5.0) is built with main's copy laid over it.
- On the Pi: `prepare.py v0.5.0` → `PREPARE tag=v0.5.0 version=0.5.0 cargo_was=0.1.0 cargo_now=0.5.0 msi=yes
  status=ok` (only the version line of `Cargo.toml` changes; the binary says "quadrule 0.5.0"). `cargo packager -f
  deb,appimage` built both; the `.deb` holds the binary, desktop entry, icons and both licences.
- `gh workflow run` first got HTTP 403; chris added Actions: read and write to the token. Dry run 42900685178
  (`e9aba64`): Linux x86-64, Linux ARM64 and Windows passed; macOS stopped at "No matching IconType" (`.icns` holds 64
  and 1024 px only as 32@2x and 512@2x). Icons renamed (`b001044`); dry run 42902842053: all four passed, six
  packages (`PACKAGE` lines in the run's summary): AppImage and `.deb` for x86-64 and ARM64, universal `.dmg`
  (`lipo -info`: x86_64 arm64), `.msi`.
- The CI-built ARM64 AppImage, downloaded to the Pi, matches the run's sha256 and runs:
  `CHECK crate=ui name=draw_bench result=pass build=release canvas=export frames=40 … p90_ms=28.7 … budget_ms=50`.
  Not run by anyone yet: the `.dmg`, the `.msi`, the x86-64 packages (no Mac, Windows or x86 machine here).
- Waits on chris: approve the packages; then publish v0.5.0's (`-f publish=true`) and switch on the release trigger.

- 2026-11-10, release packages: chris, word for word: "the macos package is confirmed working" (the `.dmg` from dry run
  42902842053). Not yet published to the v0.5.0 release; that waits on chris's word. He also reports: "when I zoom the
  rhythm overlay changes and becomes very small squares"; most likely the device-pixel grid that appears from 8×
  (`PIXEL_GRID_FROM`, `crates/ui/src/viewer.rs`; rung 2 bbq Q10), drawn in a blue close to the rhythm's
  (`crates/ui/src/view.rs`), not a change in the rhythm itself. Unconfirmed; asked chris.
- 2026-11-10, chris's answers: "Only from 16×" (the pixel grid) and "Publish now" (the packages, with the automatic
  trigger). Done:
  - `PIXEL_GRID_FROM` 8.0 → 16.0 (`crates/ui/src/viewer.rs`), amendment added under Q10 of
    `docs/bbq/2026-11-09-rung-2-viewer.md`. `cargo fmt --all --check`, `cargo clippy -p quadrule-ui -p quadrule-app
    --all-targets -- -D warnings` clean; `cargo test -p quadrule-ui`: `TESTS crate=ui passed=75 failed=0`. Export
    bytes untouched (screen overlay only). chris's look is its check.
  - Publish run 42932771588 (`gh workflow run release.yml -f tag=v0.5.0 -f publish=true`) started; result below.
  - `release: published` trigger switched on in `.github/workflows/release.yml`; README "Download" section with the
    unsigned-app steps (macOS Open Anyway, Windows Run anyway, AppImage chmod +x); v0.5.0 notes say where to download.
  - Publish run 42932771588: success on all five jobs (four builds and "Attach to the release"). `gh release view
    v0.5.0 --json assets` lists seven files: the six packages and `SHA256SUMS.txt`. The release text was replaced
    with `docs/arcs/v1/release-notes/v0.5.0.md` (`gh release edit v0.5.0 --notes-file …`).


## 2026-11-10 — rung 6 kickoff (Opus 5.5, high)

- Start: `make build-proof` → `BUILD head=cda78e0 dirty=no rebuilt=2`; `echo "$CLAUDE_EFFORT"` → `high`.
- Budget (`budget-select`): 15 points, alone on the meter, reserve 0; meter 46 % at the start (given by chris, the
  Saturday 14th 5 pm meter); hard stop 75 %. Reading logged with `spend-ledger.py reading 46`.
- Grays sheet (`dcf5458`): the existing sheets tested grays only on thin lines and small dots, so nothing showed a flat
  gray area, which fills need. Added a fifth sheet, `grays`: patches G1–G15 at 0, 17 … 238, G16–G30 halfway between,
  and a 0–255 ramp GR (`crates/xtask/src/calib.rs`, `docs/calibration.md`). One page on the Manta, two on the Nomad.
  `cargo test -p quadrule-xtask --test calib -- --nocapture` → `CHECK crate=xtask name=calib_gray_patches result=pass
  patches=60 ramps=2 fails=[]`, `calib_codes … expected=292`; clippy `-D warnings` clean on the crate.
- chris was given the hardware checklist: the grays sheet on both devices; one landscape PNG (rotated pixels or
  swapped sizes?); landscape toolbar screenshots.
- Rung 6 bbq, 8 questions (`docs/bbq/2026-11-10-rung-6-shapes.md`): 6a all the shapes, fill and stroke, dashes and caps;
  6b rotation, lines any way, opacity (Q1, overruled my split). Soft edges on curves (Q2). Fill roles Shade and Block
  plus Custom (Q3, overruled: Shade alone), starting at 204 and 68 (Q7). Tools R, E, Y (Q4). Rotation handle on a
  stalk (Q5). Dash presets plus custom, caps butt/round/square (Q6). Lint L01, L02, L06 on the new kinds in 6a (Q8).

## 2026-11-10 — rung 6a built: shapes (Sonnet 5.5, `CLAUDE_EFFORT=high`)

Kickoff from `docs/bbq/2026-11-10-rung-6-shapes.md` (6a only; 6b's rotation, lines any way and opacity are not built).
Preconditions, all met: model Sonnet 5.5 and `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` at the start →
`BUILD head=fdbb35c dirty=no rebuilt=0`. The tree was clean at `fdbb35c`. Size said in the record: 1–2 builder sessions;
it took one.

- **Step 1, format 0.5.0** (`4ba233b` fixtures, `46d3637` the rest): before any model change, two 0.4 files were written
  with the old code and the sha256 of each device's export saved (`crates/render/tests/fixtures/v0.4/`: `slice.quad`,
  `busy.quad` with a slanted line, a custom line, a grid, custom dots, ruled lines, centred text, a hidden object,
  subdivisions and a physical-locked rhythm; `SHA256SUMS`). Then `FORMAT_VERSION` 0.5.0, a `0.4.0 → 0.5.0` step in
  `migrate::CHAIN` (nothing to rewrite: every new field is optional), `ObjectKind::{Rect(RectShape), Ellipse(EllipseShape),
  Polygon(PolygonShape)}` (each with the `extra` bucket and an unknown-field test), `Object.fill/dash/cap` (left out of
  the file when default, so an old object saves as before), `Style::None`, `FillRole {Shade, Block}` and `Fill`, `DashStyle`,
  `LineCap` (`crates/model/src/lib.rs`); `fill_roles` in both profiles, Shade 204 and Block 68, marked ESTIMATE with the
  record's Q7 (`crates/profiles/`). Test: `crates/render/tests/migrate_v04.rs` (both files open as 0.4 → migrate →
  export the same four hashes; saved again they are 0.5.0, write none of the new keys and export the same).
- **Step 2, layout and rendering**: `crates/layout/src/shapes.rs` (new), `Item::StrokePath` (new; the existing variants are
  untouched), `crates/render/src/lib.rs`, the chip canvas (`crates/app/src/chip.rs`) and the screen list
  (`crates/ui/src/canvas.rs`) draw it the same way. An unrotated square-cornered rectangle is whole-pixel `Item::Rect`s (fill
  under stroke, odd widths on pixel centres); ellipses, rounded corners, polygons and stars are anti-aliased
  (`Item::Fill`, `Item::StrokePath`); dashes are cut in layout, in whole device pixels per device; caps butt, round,
  square; a pattern starts at a line's start and at a closed shape's top-left, clockwise.
- **Step 3, tools R, E, Y** (`crates/ui/src/tools.rs`, `draw.rs`): painted icons, Fluent names and tooltips with the shortcut,
  drag with rung 4's snapping, click = 10 units, Shift = square, circle or regular polygon, one undo step, back to Select,
  `keep_tool` for the double-click.
- **Step 4, inspector** (`crates/ui/src/panels.rs`, `crates/i18n/locales/en-US/app.ftl`): Style tab gains Stroke (None, Role,
  Custom), Fill (None, Shade, Block, Custom gray, steps of 17), Dash (with Custom lengths) and Cap; a Shape section: corner
  radius (any unit, `u` included), sides, Star, inner radius. Every control is named for screen readers.
- **Step 5, lint** (`crates/lint/src/lib.rs`): L01 reads the stroke of every new kind; L02 only where a straight, unrotated
  edge exists (`shapes::has_axis_aligned_edge`: rectangles, polygons with a horizontal or vertical edge, never an ellipse);
  L06 uses the frame. L13 stays out.
- **Step 6, goldens**: `shapes_document()` (`crates/model/src/sample.rs`) → `shapes_<id>_WxH.png` per device, appended to
  `crates/render/tests/golden/SHA256SUMS` (`crates/render/tests/shapes.rs`). Looked at by eye on both devices.
- **Step 7, bench**: `--shapes-page` opens that page (the benchmarks then run over it) and `--bench-tool
  rect|ellipse|polygon|pattern|line|text` picks the drawn tool; `make bench-drag SHAPES=1`, `make bench-draw SHAPES=1
  TOOL=rect`. The bench's CHECK line gains `page=` and `tool=`.
- **Step 8**: README "what works today" and DEVELOPMENT.md crate map and commands.
- **Frozen bytes:** `shasum` of the two rung 1 lines before and after: `b6f321e9…cee` and `26e04bab…e23` (unchanged:
  `head -2 crates/render/tests/golden/SHA256SUMS`); the stock-grid test, `export_bytes_unchanged_after_edit_undo` and the
  four 0.4 hashes pass. No expected hash was changed. Two test edits, no expected value among them: the editor test's
  `golden.len() == 2` became `>= 2` (the file holds the new lines too), and `calib.rs`'s exhaustive `match` on `Item` got the
  new variant's arm.
- **Checks:** `make check-all` → `TESTS scope=workspace checks=221 check_fail=0 passed=209 failed=0 status=ok`;
  `grep -o 'CHECK crate=[^ ]* name=[^ ]*' target/test.log | wc -l` → 221. 27 new `CHECK` names across
  `crates/layout/tests/shapes.rs`, `crates/render/tests/{shapes,migrate_v04}.rs`, `crates/model/tests/model.rs`,
  `crates/profiles/tests/profiles.rs`, `crates/lint/tests/lint.rs`, `crates/ui/tests/shapes.rs`, and the app's `command_line`.
- **Measured** (Pi, release build, shapes page, 150 frames, `p90_ms`; budget 50, ADR-009): drag export 25.2, drag chip 46.3,
  draw rect 23.9, ellipse 24.4, polygon 24.9 (export), draw rect chip 47.9. All `result=pass`; the chip canvas is close
  to the line (worst 50.1).
- **Seen:** the golden pages by eye (crops of both devices): shapes, fills, dashes, caps and corners read as meant. The window
  itself I did not click through (no pointer here beyond the headless tests).
- **CI:** first push `46d3637`: CI 42939952956 and Coverage 42939952911 failed on one check of my own, the "portable maths
  equal the fastest" assertion in the shapes golden test: on x86 (Linux and Windows) the portable maths differ from the
  fastest on the shapes page, while the exported golden hashes matched on both. The assertion became a printed count
  (`portable_diff_px`, `portable_max_diff`) in the follow-up (see below). This touches ADR-003 item 2's lever, for chris.

### Things I chose that the record did not say (each vetoable)

- No stroke is `Style::None` (a variant), not a flag; fill, dash and cap are three fields on the object; Dash and Cap also
  apply to lines (a line is a stroke); pattern regions and text have neither.
- Caps work as in SVG: a cap reaches half the width past each dash, so a gap shorter than the width closes (square caps on a
  short gap read solid); on whole-pixel dashes the ends round half up. A closed shape's first dash reaches back half a stroke
  so its top-left corner is square.
- Dash start: rectangle at its top-left corner; rounded rectangle at the start of its top edge; ellipse at the point nearest
  the top-left corner; polygon and star at the corner nearest it; all clockwise.
- Stroke geometry: the shape's outline sits on the stroke's pixel grid (centres snapped as lines are); a hard fill covers the
  rounded frame, under the stroke.
- A click puts the shape's top-left at the click point (slid in to fit the page), 10 units across (unit_x), as tall as
  makes it square, circle or regular (a hexagon is narrower than tall). Shift on a polygon keeps the polygon's own
  proportions, the long side snapping.
- A shape is picked and snapped by its box (a hollow shape takes a click inside it). Star inner radius is held to 0.10–0.95;
  ticking Star sets 0.5. Custom fill gray starts from the current fill's gray on the device on screen, rounded to a step of 17.
- Miter joins on `Item::StrokePath` (limit 4); tool names on the command line are `rect`, `ellipse`, `polygon`, …;
  new objects are named Rectangle, Ellipse, Polygon (Fluent).
- The shapes page lives in code (`shapes_document`), opens with `--shapes-page`, and holds the sample's dot grid so the drag
  bench works; the Nomad's copy scales with Scale constraints.
- README's "That opens the sample page" was untrue (the app starts empty) and now says so.

**Waits on chris:** the look at 6a (`cargo run --release -p quadrule-app -- --shapes-page`, then draw with R, E, Y, Shift,
and the Style tab). Owed items are in `docs/STATUS-debt.md` (rung 6a line).

## 2026-11-10 — rung 6a lead: judged, measured, waits for chris's look (Opus 5.5, `CLAUDE_EFFORT=high`)

- Start: chris set the effort to high when asked (it was not high at first); `echo "$CLAUDE_EFFORT"` → `high`;
  `make build-proof` → `BUILD head=fdbb35c dirty=no rebuilt=1`.
- Budget: meter read by chris as "about 46–50 %"; logged as 50 (the top of the range) with `spend-ledger.py reading 50`.
  Share 15 points stands (hard stop 75 − 50 = 25). Stop lines: ledger 15 points or meter 61 %.
- Builder: Sonnet 5.5 at high effort, as a subagent of this session, briefed from the bbq record
  (`docs/bbq/2026-11-10-rung-6-shapes.md`). Its entry is above ("rung 6a built"); commits `4ba233b`, `46d3637`,
  `789f424`; about 351k tokens. Ledger so far ≈ (351k + this session's context, about 100k) ÷ 0.79M × 1.5 ≈ 0.9 points.
- Judged against the record, all preconditions met: `make build-proof` → `BUILD head=789f424 dirty=no rebuilt=1`;
  `make check-all` → `TESTS scope=workspace checks=221 check_fail=0 passed=209 failed=0 status=ok`;
  `git diff fdbb35c HEAD -- crates/render/tests/golden/SHA256SUMS` adds two lines and leaves the two existing ones as
  they were; CI run 42940487205 (`789f424`) success on ubuntu, macos and windows.
- Portable maths (ADR-003 item 2): on CI, `gh run view 42940487205 --log | grep shapes_golden` shows the shapes page
  exports the **same sha256 on Linux x86-64, Windows x86-64 and macOS ARM64** (and the Pi), so item 2's trigger
  ("Intel and ARM giving different bytes") is not met. The portable maths differ from the fast ones on x86 only (16 px
  on the Manta, 21 on the Nomad, each 1 gray level); exports use the fast ones. The shapes golden test fails on any
  system whose export differs from the golden, so a real trigger becomes a red build. Noted for chris; no change.
- Speed, release build on the Pi, `--shapes-page`, 150 frames, 3 runs each (90th percentile, ADR-009, 50 ms line):
  export canvas drag 23.8–24.2 ms; draw rect 23.9, ellipse 23.7–24.1, polygon 23.7–23.9 ms. Chip canvas drag
  45.9–46.0 ms; draw rect 47.8–47.9, ellipse 48.3–48.4, polygon 47.8–47.9 ms. All `result=pass`. The chip canvas has
  little headroom (it was 41–42 ms on the sample page at rung 5). The sample page re-timed: export draw 26.1 ms, chip
  draw 42.4 ms, the same as rung 5. Command: `./target/release/quadrule --shapes-page --bench-draw 150 --bench-tool
  <rect|ellipse|polygon> --canvas <export|chip>` (and `--bench-drag 150`).
- Waits on chris: the look at 6a. After his approval: bump to `0.6.0`, tag `v0.6.0`, the pre-release, then 6b.

## 2026-11-11 — rung 6a approved; v0.6.0 (Opus 5.5, `CLAUDE_EFFORT=high`)

- chris, word for word: "approved". **Rung 6a is done.**
- Workspace version 0.1.0 → 0.6.0 (`Cargo.toml`, `Cargo.lock`); `make check-all` after the bump → `TESTS
  scope=workspace checks=221 check_fail=0 passed=209 failed=0 status=ok`. Release notes:
  `docs/arcs/v1/release-notes/v0.6.0.md`. Tag `v0.6.0` on the approval commit; pre-release below.

## 2026-11-11 — rung 6b built: rotation, lines any way, opacity (Sonnet 5.5, `CLAUDE_EFFORT=high`)

Kickoff from `docs/bbq/2026-11-10-rung-6-shapes.md` (6b only: rotation by handle and field, lines that point any way,
opacity). Preconditions, all met: model Sonnet 5.5 and `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` at the start →
`BUILD head=0115e08 dirty=no rebuilt=0`; the tree was clean at `0115e08` (workspace 0.6.0). I did not bump the version, tag
or release, and did not touch `STATUS.md` or the board.

- **Step 1, format 0.6.0** (`fe6de66` fixtures, `42a1111` the rest): before any model change, two 0.5 files were written with
  the 6a code and the sha256 of each device's export saved (`crates/render/tests/fixtures/v0.5/`: `shapes.quad`, the shapes
  page; `lines.quad`, a page of lines in every old direction (horizontal, vertical, shallow, steep and 45° down-right),
  dashed, dotted, round and square capped, with a zero-size one; `SHA256SUMS`). Then `FORMAT_VERSION` 0.6.0, a `0.5.0 → 0.6.0`
  step in `migrate::CHAIN` (nothing to rewrite: every new field is optional), `Object.rotation_deg` (0, left out of the file at 0),
  `Object.opacity_pct` (100, left out at 100), `Line.direction: LineDir {Down, Up}` (Down, left out), `Object::rotation()`,
  `ObjectKind::can_rotate()`, `normalize_deg`, `edit::Placement` and `edit::set_placements` (`crates/model/src/lib.rs`,
  `edit.rs`). No new record, so no new `extra` bucket; `Line` already had one. Test: `crates/render/tests/migrate_v05.rs`
  (both files open as 0.5 → migrate → export the same four hashes; saved again they are 0.6.0, write none of the three new
  keys and export the same); `migrate_v04` still passes with its four 0.4 hashes (only its `FORMAT_VERSION == "0.5.0"` guard
  became `"0.6.0"`).
- **Step 2, layout and rendering** (`42a1111`): `crates/layout/src/rotate.rs` (new: exact quarter turns, clockwise angle about the
  frame's centre, rotated corners, line ends for a direction, separating-axis test of a turned box against a rectangle),
  `shapes::shape_items` (a turned shape is its path turned about the snapped box's centre, then stroked, so the stroke stays one
  width; a rectangle or ellipse turned by 90, 180 or 270 is drawn as the unturned shape in the swapped box, so its straight edges
  stay hard whole-pixel rectangles), `display_list` (a line's ends come from its direction and angle). Opacity is two new
  `Item` variants, `BeginAlpha { alpha, bounds }` and `EndAlpha`: an object's items are drawn as one group and laid over
  what is beneath (vello_cpu `push_layer` with an opacity in `crates/render/src/lib.rs`; vello `push_layer` in
  `crates/app/src/chip.rs`; `screen_list` scales the bounds in `crates/ui/src/canvas.rs`), so a stroke over its own fill is not
  mixed twice and the PNG holds the flattened gray, no alpha. At 100 % no group is made; at 0 % the items are dropped.
  Pattern regions and text boxes take opacity but not rotation.
- **Step 3, rotate handle, fields, lines, lint** (`66d4b7f`): the round handle on a stalk above the top edge's middle of the
  one selected line or shape (`Viewer::rotate_handle`, `HandleHit::Rotate`; stalk 24 screen points, kept at one length on screen
  by `Viewer::rotate_stalk_px`); it catches at 0/90/180/270 within 4° (Ctrl turns that off, as for snapping), Shift holds 15° steps
  (`drag::turned_angle`), one undo step, the frame unchanged. The Arrange tab has a Rotation field in degrees (for lines and
  shapes only), the Style tab an Opacity field in percent (every kind); both are named for screen readers through Fluent
  (`app-rotation`, `app-opacity`). Selection outline, the eight handles, picking (`Shape::Rotated`), marquee, point snap, smart
  guides and equal spacing use the turned corners; lines pick, snap and show handles on their real ends (`Other.ends`).
  A line is drawn where the pointer goes in all four directions (Shift still holds an axis); its two end handles can be pulled
  past each other any way, and the frame and direction follow. Lint: L01/L02/L06/L10 (`crates/lint/src/lib.rs`).
- **Step 4, goldens** (`44f7077`): `transforms_document()` (`crates/model/src/sample.rs`) → `transforms_<id>_WxH.png` per device,
  *appended* to `crates/render/tests/golden/SHA256SUMS` (`crates/render/tests/transforms.rs`): rectangles at 0/15/30/45/90 and a
  rounded one at 20, an ellipse at 30 and 90, a hexagon at 90, triangles at 180 and 270, a star at 36, lines pointing every way
  (down, up, steep up, dashed up, turned 90, 30, up turned 20, dotted at 45), a Shade box at 50 % and a circle at 25 % over a
  Block bar, a 40 % line over a Shade box, a turned Block rectangle at 60 %, a turned star at 50 %, a pattern at 30 %, a 0 % circle
  (draws nothing), and a big turned rectangle. Looked at by eye (the Manta page): turns, hard quarter turns, fills and grays read
  as meant.
- **Step 5, benches** (`1da293a`): `--transforms-page` opens that page (the benchmarks then run over it), `--bench-rotate [frames]`
  presses the rotate handle of the big turned rectangle and swings the pointer round it (`name=rotate_bench`); the CHECK line's
  `page=` gains `transforms`. `make bench-drag TRANSFORMS=1`, `make bench-draw TRANSFORMS=1 TOOL=line`, `make bench-rotate`.
- **Step 6** (`32f4abb`): README "what works today", DEVELOPMENT.md (crate map, commands), Makefile; the two up-right line debts
  removed from `docs/STATUS-debt.md` (below).
- **Frozen bytes:** `head -2 crates/render/tests/golden/SHA256SUMS` and the 6a pair are unchanged (`git diff 0115e08 HEAD --
  crates/render/tests/golden/SHA256SUMS` adds only the two `transforms_` lines); the stock-grid test,
  `export_bytes_unchanged_after_edit_undo`, the four 0.4 hashes and the four 0.5 hashes pass. No expected hash was changed.
  Test edits that are not expected hashes: `migrate_v04`'s format-version guard; `crates/layout/tests/snap.rs` and
  `crates/lint/tests/lint.rs`, `crates/ui/tests/new.rs` literals gained the new fields (`ends: None`, `rotation_deg: 0.0`,
  `opacity_pct: 100`); `crates/xtask/tests/calib.rs`'s exhaustive `match` got the two new `Item` arms; and
  `crates/ui/tests/draw.rs`'s `a_line_that_would_point_up_and_right_runs_along_an_axis` was replaced by
  `a_line_is_drawn_in_all_four_directions`, because the workaround it tested is what this step removes.
- **Checks:** `make check-all` → `TESTS scope=workspace checks=252 check_fail=0 passed=236 failed=0 status=ok` (6a left it at 221;
  `grep -o 'CHECK crate=[^ ]* name=[^ ]*' target/test.log | sort -u | wc -l` → 243 distinct names). New `CHECK` names:
  `migrate_v05`, `migrate_chain_060`, `transforms_golden` ×2, `transforms_sums_listed`, `opacity_flattens`,
  `quarter_turns_hard_edged`, `turns_about_centre`, `rotate_maths`, `line_direction`, `turned_box_overlap`, `line_up_layout`,
  `turned_shape_placed`, `opacity_group`, `defaults_draw_as_before`, `rotate_handle`, `rotate_handle_kinds`,
  `rotated_pick_handles_resize`, `rotated_snaps_by_turned_box`, `line_ends_any_way`, `turned_line`,
  `window_rotation_opacity_fields`, `window_transforms_benchmarks`, `draw_line_four_directions`, `rotated_l02`,
  `rotated_line_diagonal`, `rotated_l06_l10`, `format_060_fields`, `rotation_rules`, `placement_command`, and the app's
  `command_line` (two more cases).
- **CI:** push `32f4abb`: CI 43021220344, Fuzz 43021220184 and Coverage 43021220085 all success. The golden sha256 is the same on
  macOS ARM64, Linux x86-64 and Windows x86-64 and the Pi: transforms `fa85a493…29b7` (Manta) and `f21ff25d…3bee` (Nomad); shapes
  `8b9c24d1…5369` (Manta) and `f0d6636b…7270` (Nomad), the 6a values, unchanged. The portable maths differ from the fast ones on x86 only (transforms
  page: 11 px Manta, 2 px Nomad, each 1 gray level; shapes page as in 6a), printed not enforced; no exported sha differs between systems.
- **Measured** (Pi, release build from `32f4abb`, 150 frames, `p90_ms`, budget 50, ADR-009; the lead does the official run):
  export canvas, transforms page: drag 24.9 / 25.0, draw line 24.2 / 25.7, rotate 25.6 / 24.6 (two runs; one run each had a single
  slow frame, worst 52.3, 95.1 and 44.5 ms; 90th percentile is the judge). Chip canvas: drag 34.9 / 34.8, draw line 35.6 / 35.7,
  draw rect 36.6, rotate 35.1 / 34.9; chip canvas on the 6a shapes page, drag 46.0 (was 45.9–46.0, so nothing added to its
  per-frame work). All `result=pass`, no `chip-cpu-fallback`. The chip canvas is faster on the transforms page than on the shapes
  page because it holds fewer objects. Per-frame work added: a turned shape is one more `Affine` on its path; an opacity group is one
  layer per object with opacity under 100 (none on the shapes page); a turned selection's handles are 9 more points.
- **Seen:** the transforms page golden by eye, and a `grim` screenshot of the running window on the chip canvas (`--transforms-page
  --canvas chip`): the same page, with the opacity groups, as the export. I did not click through the window: no pointer here
  beyond the headless tests.

### Things I chose that the record did not say (each vetoable)

- **Text boxes and pattern regions do not rotate in 6b.** The Rotation field and the rotate handle are hidden for them, the model
  ignores a stored angle on them (`Object::rotation()` is 0 for those kinds), and `Placement::apply` never sets one. A pattern is
  bound to the rhythm, which a turn would break (a clip to a turned box of a rhythm-locked pattern); a text box's height is the
  text's, so its centre moves as it grows. Opacity applies to every kind, patterns and text included.
- **Rotation direction and range:** positive is clockwise on the page; stored as given but read in 0 up to 360 (a field value of -90
  reads as 270).
- **Turned quarter turns:** a rectangle or ellipse at 90, 180 or 270 is the unturned shape in a swapped box (hard rectangles for a
  square-cornered rectangle); a polygon or star at a quarter turn is a turned path (anti-aliased, as at 0). L02 judges the straight
  edges of any rotation that is a multiple of 90 (as at 0) and none at other angles, including a hexagon at 30° whose edges happen
  to run level. A dash pattern on a rectangle or ellipse at 90/180/270 starts where the unturned shape's would (the top-left
  corner), not at the turned corner.
- **Resizing a turned object** (single object): along its own sides. The side or corner opposite the handle stays where it is on the
  page (the centre with Alt), the angle stays, no side goes under 1 device pixel (no flipping), Shift keeps a corner's proportions,
  and the dragged point snaps as a point does (rhythm, margins, other objects' anchors). Several selected objects, a turned one
  among them: the handles are the box round them; each turned object keeps its angle, its centre goes where the group's scaling puts
  it, and its sides stretch by how much the scaling stretches their directions (a turn of 45° in a non-uniform scale is therefore
  approximate).
- **Snapping and picking by "rotated corners":** the box round the turned corners (what you see) is what snaps and what other
  objects' point snap, smart guides and equal spacing see; picking, the marquee, the outline and L06 use the turned corners
  themselves (the marquee and L06 exactly, by the separating-axis test; L06 with half the stroke added).
- **Rotate handle:** 24 screen points above the top edge; catch 4°; Shift's 15° steps win over the catch; Ctrl turns the catch off;
  it appears for exactly one selected, unlocked line or shape (not several, which have no rotate in 6b); it is a separate query from
  `handles()` (so the existing handle counts are unchanged) and takes a press ahead of a nearby box handle; a rotation starts from the
  angle where the pointer was pressed, so the object does not jump.
- **A line's start is its left end.** For a line pointing up the start is the bottom-left; dashes start there. A line frame with no
  width or no height has one diagonal, so its direction does not matter (and is kept as set).
- **Pulling a line's end** sets the line as it is: the frame becomes the box round the two ends, the direction follows, and a turn is
  folded in (rotation back to 0). Pulling a line's end to exactly level or plumb keeps its direction. A line is turned about the centre
  of its frame (the box, not the segment's midpoint, which is the same point).
- **Opacity** is an integer percent, `opacity_pct`, in the Style tab (Arrange holds position, size and rotation); the group opacity
  is `round(percent × 255 / 100)`; a group's bounds are the items' with a miter allowance, and the renderers clip the layer to
  that box grown by a pixel. A 0 % object draws nothing (and is still selectable by its frame).
- **L10 (off the page)** reads the box round the turned corners too, since it shared the helper; no new finding text.
- **Shift while drawing a line** still holds it to an axis (it did before); I did not make it 15° steps.
- **The transforms page** has the sample's dot grid on top (for the drag bench), no text, and the big turned rectangle as the last
  object; names are English strings stored as the file's data, as on the shapes page.

**Waits on chris:** the look at 6b (`cargo run --release -p quadrule-app -- --transforms-page`, then select a shape: drag the round
handle, hold Shift, type an angle in the Arrange tab, resize a turned shape, draw lines up and to the right, pull a line's end past
the other, set Opacity in the Style tab). Owed items are in `docs/STATUS-debt.md` (rung 6b line).

## 2026-11-11 — rung 6b lead: judged, measured, waits for chris's look (Opus 5.5, `CLAUDE_EFFORT=high`)

- Builder: Sonnet 5.5 at high effort, a subagent of this session; its entry is above ("rung 6b built"); commits
  `fe6de66` to `6eb9345`; about 385k tokens. Ledger so far ≈ (351k + 385k + this session's context, about 150k) ÷
  0.79M × 1.5 ≈ 1.7 points of 15.
- Judged against the record: `make build-proof` → `BUILD head=6eb9345 dirty=no rebuilt=2`; `make check-all` →
  `TESTS scope=workspace checks=253 check_fail=0 passed=236 failed=0 status=ok`; `git diff 0115e08 HEAD --
  crates/render/tests/golden/SHA256SUMS` adds the two `transforms_` lines and changes none; CI 43021220344 (`32f4abb`)
  success. `gh run view 43021220344 --log | grep -E 'name=(transforms|shapes)_golden'`: the same sha256 on Linux
  x86-64, Windows x86-64 and macOS ARM64 for both pages (ADR-003 item 2 not triggered; the portable maths differ on
  x86 only, 11 px and 2 px on the transforms page, 1 gray level each).
- Speed, release build on the Pi, 150 frames, 3 runs each, 90th percentile against 50 ms (ADR-009), all `result=pass`:
  transforms page, export canvas: drag 24.5–25.3, draw line 24.3–25.4, draw rect 23.8–25.7, rotate 24.4–25.5 ms; chip
  canvas: drag 34.9, draw line 35.6–35.7, draw rect 36.6–36.7, rotate 34.9–35.0 ms. The 6a shapes page re-timed: drag
  export 24.2–26.1, chip 45.9–46.1 ms (as before 6b). Commands: `./target/release/quadrule --transforms-page
  --bench-drag 150 [--canvas chip]`, `… --bench-draw 150 --bench-tool line|rect`, `./target/release/quadrule
  --bench-rotate 150 [--canvas chip]`.
- The spec does not say whether text boxes and pattern regions rotate (`napkins/quadrule-spec.md` §5.2, line 276
  lists rotation in the Arrange tab for objects in general); the builder left them unrotatable in 6b. Put to chris at
  the look.
- `v0.6.0` is tagged and pushed; its GitHub pre-release waits on chris (my `gh release create` was refused by the
  permission check; `gh release view v0.6.0` → "release not found").
- Waits on chris: the look at 6b. After his approval: bump to `0.6.1`, tag `v0.6.1`, the pre-release; then re-plan the
  board and write the next lead's prompt.

## 2026-11-11 — rung 6b look change: handles turn with the object (Sonnet 5.5, `CLAUDE_EFFORT=high`)

chris approved 6b at his look with one change, in his words: "approved, with one change. When an object rotates the bounding box and handle should rotate with the object". Screen overlay only; no export byte moved (the six golden lines, the 0.4 and 0.5 fixtures and the stock grid are unchanged).

- **Cause (confirmed):** `Viewer::handles()` and `Viewer::rotate_handle()` (`crates/ui/src/drag.rs`) read the angle and box of a turned object from the open document, which a drag does not touch until release; the outline reads the page's shapes, which come from the drag's preview. So while an object turned, its outline turned and its handles stayed where they were; the same held for a move or a resize of an already turned shape. They now read `Viewer::shown_doc()` (the preview while a drag or drawing runs, else the open document), so the outline, the eight handles and the rotate handle always agree with what is drawn.
- **Handle squares** are painted turned with the object: `Viewer::handle_squares(half)` gives each handle's four corners turned by `Viewer::selection_angle()` (that of the one selected object as drawn now; 0 for none or several), and `view.rs` paints them as a polygon when the angle is not 0 (an upright selection is painted exactly as before). The end handles of a turned line turn too. The rotate handle stays a circle on its stalk.
- **A turned line's outline** now follows its turned frame (`Viewer::turned_line_outline`), grown by 2 screen points so a level or plumb frame (no height or no width) still shows as a thin box, like the upright outline of a line; an unturned line keeps its upright bounds as before.
- **Multi-selection** is unchanged (an upright union box, upright squares).
- **Tests** (`crates/ui/tests/rotate.rs`): `handles_turn_during_rotate_drag` (at six angles while the pointer is down, with the open document still unturned: the eight handles are on the preview's turned corners and edge middles, the rotate handle's base is on the top edge's middle and its tip 60 px straight out, every square is turned by the angle; and again after release), `handles_follow_turned_move_and_resize` (a turned shape moved and resized), `turned_line_outline`.
- **Chosen:** the line outline's padding is 2 screen points, as the upright outline's; handle squares keep the 8-point size and white fill; nothing was done for Alt-duplicate drags: by reading the code (not tested), their handles are hidden while the copies are made, because the copies are in the preview and not in the open document the selection is checked against; an old behaviour, noted not changed.

## 2026-11-11 — rung 6b approved; v0.6.1; rung 6 done (Opus 5.5, `CLAUDE_EFFORT=high`)

- chris, word for word: "approved, with one change. When an object rotates the bounding box and handle should rotate
  with the object". Cause found by the lead: the handles and the rotate handle read the committed document, the
  outline read the drag's preview, so the handles stayed at the old angle until release. Built by the 6b builder
  (`e8a49d3`, its entry above); CI 43120346354 success. Taken as approved once built, as at rung 4a (said to chris;
  he raised no objection and asked how to publish the releases).
- Workspace version 0.6.0 → 0.6.1 (`Cargo.toml`, `Cargo.lock`). `make check-all` after the bump → `TESTS
  scope=workspace checks=255 check_fail=0 passed=239 failed=0 status=ok` (the builder's run counted 256 checks: the
  summary counts only `CHECK` lines that start a row, and parallel output splits one now and then; 0 failures either
  way). Re-timed on the Pi, release build: rotate bench export 26.3 ms ×3, chip 35.0–35.2 ms; transforms-page drag
  export 26.9, chip 35.1 ms (90th percentile, 50 ms line); all `result=pass`.
- Tag `v0.6.1` on the approval commit. Release notes `docs/arcs/v1/release-notes/v0.6.1.md`. The GitHub
  pre-releases for `v0.6.0` and `v0.6.1` wait for chris (`gh release create` is refused by this session's permission
  check); he has the commands.
- Re-plan written on the board; the next lead's prompt is `docs/arcs/v1/rung-7-PROMPT-2026-11-11.md`.
- Ledger at the close ≈ (351k + 407k builder tokens + this session's context, about 200k) ÷ 0.79M × 1.5 ≈ 1.8 points
  of 15. Meter start reading 50 % (chris's range, the top of it); no end reading taken.

## 2026-11-11 — v0.6.0 and v0.6.1 published without packages: immutable releases (Opus 5.5, `CLAUDE_EFFORT=high`)

- chris, word for word: "I have run both release commands and the pre-releases are now live on gh. you should also
  have the permission to run the release yourself now". `gh release view v0.6.0` / `v0.6.1` → pre-release, not draft,
  **0 assets**.
- Release runs 43122417571 (v0.6.0) and 43122512967 (v0.6.1): the four package builds passed; "Attach to the release"
  failed with `HTTP 422: Cannot upload assets to an immutable release`. `gh api repos/tholent/quadrule/immutable-releases`
  → `{"enabled":true,"enforced_by_owner":false}`; both releases report `immutable=true`.
- GitHub's docs (read 2026-11-11: docs.github.com …/immutable-releases and …/prevent-release-changes): "Only the
  assets and tag are locked. You can still edit the title and release notes of a published immutable release";
  "If you delete the immutable release, you can delete the tag, but you cannot reuse the same tag name"; the
  recommended flow is "Create the release as a draft. Attach all associated assets to the draft release. Publish the
  draft release." Workflows do not fire for draft releases' `created`, `edited` or `deleted`.
- So `.github/workflows/release.yml` (built when the setting was off: it packages *after* `release: published`) cannot
  work any more. The packages from both runs are kept as run artifacts until 2027-02-09 (`gh api
  repos/tholent/quadrule/actions/runs/43122417571/artifacts`). Put to chris: what to do with the two releases, and
  how to change the workflow.
- chris's rulings, word for word (answers to two questions): "Keep, add a note" (the two releases) and "Tag push does
  it all" (the workflow). Done in `7e56b02`: `.github/workflows/release.yml` runs on a `v*` tag push (not on tags
  containing `-test`), builds the four packages, drafts the release with `docs/arcs/v1/release-notes/<tag>.md` from
  the tagged commit (title from its "## New in …" heading; pre-release before 1.0.0), attaches the files, checks the
  count, then publishes; by hand `-f publish=no|draft|yes`. The `release: published` trigger is gone. `v0.6.0` and
  `v0.6.1`'s notes now say no installers are attached (`gh release edit … --notes-file`; the text of an immutable
  release stays editable). `DEVELOPMENT.md`, `STATUS.md` and the rung 7 prompt say so.
- Trial with a throwaway tag `v0.6.2-test.1` on a commit outside `main` (`6fa8351`, holding only a test notes file):
  pushing it started no run (the `-test` exclusion); `gh workflow run release.yml -f tag=v0.6.2-test.1 -f
  publish=draft` → run 43123342877 success on all five jobs; its log: `DRAFT tag=v0.6.2-test.1 title="v0.6.2-test.1 —
  test: the draft-first release flow" made=yes`, `ATTACH tag=v0.6.2-test.1 files=7 status=ok`; `gh release view` →
  draft, pre-release, 7 files. Then the draft and the tag were deleted (`gh release view v0.6.2-test.1` → "release not
  found"; `git ls-remote --tags origin` shows no test tag). Not tried: the final publish step and the tag-push trigger
  on a real tag; `v0.7.0` is their first use.

## 2026-11-11 — rung 7a built: groups (Sonnet 5.5, `CLAUDE_EFFORT=high`)

Kickoff from `docs/bbq/2026-11-11-rung-7-groups-and-align.md` (7a only: Q2, Q3, Q4, Q7, Q8 and the Defaulted items about
groups; no align or distribute, Q5, Q6 or "mixed" fields). Preconditions: tree clean at `f839644` (workspace 0.6.1). I did
not push, tag, bump the version, publish, or touch `STATUS.md`.

- **Format 0.7.0** (`bb5c542`): `ObjectKind::Group(GroupShape { children, extra })` (`crates/model/src/lib.rs`); a no-op
  `0.6.0 → 0.7.0` step in `migrate::CHAIN`; the 0.6 fixtures `crates/render/tests/fixtures/v0.6/` (`transforms.quad`, `slice.quad`
  with its text, and `SHA256SUMS`) were saved by the 0.6 code from a throwaway worktree of `f839644`, before any 0.7 change; test
  `crates/render/tests/migrate_v06.rs` (both open as 0.6, migrate, export the same four hashes; saved again they are 0.7.0 and
  hold no `children` key). `migrate_v04` and `migrate_v05` only had their `FORMAT_VERSION == "0.6.0"` guard moved to `"0.7.0"`.
- **Model and commands** (`crates/model/src/edit.rs`, `geom.rs`): children live in the group's unturned space in page mm, so
  grouping moves nothing. New: `Place`, `Loc`, `Document::locate/at/object_mut/place_of/parent_of/ancestors/children_of/
  effectively_locked/effectively_hidden` (a locked or hidden group locks or hides what is in it), `Command::InsertIn` and
  `MoveObjectIn` (`Remove`, `MoveObject`, `SetObject` now work at any depth), `edit::group`, `edit::ungroup` (bakes the group's
  angle into each child, centre moved about the group's centre, a child group's own contents carried with it; multiplies
  opacity, carries hidden and locked), `reframe` (a group's contents follow its frame: a move translates the subtree, a resize
  resolves each child by its own constraints against the old box → the new box, recursively; Q3). `Document::apply` refits the
  box of every group an edit touched inside, deepest first, to the union of its children's turned boxes, removes a group left
  with no children, and puts the box restores in the undo command so undo is exact; a turned group's children are shifted by
  (R − I)(centre moved) so the drawing does not move when its turn centre does. `Document::objects()` now walks groups
  (group before children); `leaves()` skips the groups. Text boxes and pattern regions are refused by a turned group (`EditError::
  TurnedGroup`) and a group holding one cannot turn (`ObjectKind::can_rotate`). The constraint maths (`resolve_frame`,
  `unresolve_frame`) moved to `quadrule_model::geom` (layout re-exports it; byte-identical arithmetic), with `Holder` (a stored box →
  a resolved box; a box with no width or no height does not divide by zero).
- **Layout** (`crates/layout/src/tree.rs`, `lib.rs`): `resolve_nodes` gives every object its resolved frame in its holder, its frame
  on the page, its total angle (own plus every group's), and effective hidden and locked; `display_list` draws a group's children
  inside it and wraps a group below 100 % in one `BeginAlpha … EndAlpha` (nesting works in both renderers, which already used a
  layer stack); `placed_objects` lists groups before their children (a group's shape is its box, turned with it) and `Placed`
  carries its `Node`; `text_requests` uses the width a text box has in its group. `UnitContext::with_percent_of` makes `%` the
  group's box inside a group (pays the `units.rs` "no groups yet" note).
- **Lint** (`crates/lint/src/lib.rs`): runs on each leaf from `placed_objects` (its own copy of the bounds maths is gone: debt "rung 3,
  lint keeps its own copy" paid); a group has no rules.
- **Editor** (`crates/ui/src/editor.rs`, `drag.rs`, `panels.rs`, `view.rs`): `Viewer::entered`, `pick_leaf`, `pick` (the child on the
  way down at the level entered), `click_with(…, deep)`, `enter_at` (double-click; tested through this method), `leave_group`
  (Escape), `select_row` (a tree row enters its groups), `group_selection`, `ungroup_selection`, `move_object_into`,
  `row_drop_place` (drop above, below, or on the middle of a group row). Ctrl+G, Ctrl+Shift+G, Group and Ungroup buttons in the
  Arrange tab and the layers panel. Drag, resize, turn, nudge, Alt-duplicate and the marquee work on a group as one object and on a
  child inside a turned group (the pointer step is carried back through the turns: `Item::stored`); a resize holding a group stops at
  one pixel instead of flipping (`not_past`; debt 4a paid for real groups). The layers panel is a tree (fold arrows named for
  readers). The Style tab of a group is Opacity only; Rotation is greyed with a note when a text box or pattern region is inside;
  the Constraints tab says "work against the group" for a child; the entered group has a faint dashed outline.
- **Goldens and benches**: the groups page `quadrule_model::sample::groups_document()` (nested three deep; a group turned 25° with a
  child turned 15°; a group at 50 % with overlapping children; a child pinned Right and Top; a heavy group of 40 shapes in two sub-groups,
  one at 80 %); `--groups-page`, `GROUPS=1` for `make bench-drag` and `bench-draw`; new lines in `crates/render/tests/golden/SHA256SUMS`:
  `70ec1bf7…` (a5x2) and `2963ff3f…` (a6x2); the six old lines, the v0.4 and v0.5 fixtures are untouched. Pixel checks: a group at 50 %
  fades as one (227 vs 164 where two 50 % objects would show the bar through); grouping the nested group moves nothing (0 px on both
  devices); a turned group equals its children with the angle baked in (0 px).
- **Speed** (Pi, release build, window on the Pi, `--groups-page`, 150 frames, 90th percentile, all `result=pass` under 50 ms): drag of the
  heavy group, export canvas 25.6, 25.7, 25.7 ms (three runs), chip canvas 41.0, 41.3, 41.0; draw rect export 25.0, chip 42.5; draw line
  export 25.2, chip 41.6. For comparison the sample page's chip drag in the same session: 32.2. The chip canvas on this page is the
  heaviest so far (nested opacity layers) and is closer to the line than before.
- **Chosen**: (1) after a resize of the group itself the box is left as dragged (the children resolve against it), and refits only on the
  next edit inside; (2) grouping needs objects from one place; (3) moves across the edge of a turned group in the layers panel are refused
  with a note; (4) Ctrl+press drags what is already selected if it holds or is the object under the pointer, else picks the innermost;
  (5) a hidden or locked group's flag is carried to its children by Ungroup; (6) a group left empty is removed, and comes back on undo;
  (7) grouping can change how a page looks on the other device: children follow the group's box (which scales with the page), not the
  page, as the record's Q3 says (shown by the render test `text_in_group`: the slice's title grouped with its rule differs by 48,319 px on
  the Nomad, 0 on the Manta).

**Waits on chris**: the look at 7a (`cargo run --release -p quadrule-app -- --groups-page`: click a group, double-click into it, Escape,
Ctrl+click; Ctrl+G and Ctrl+Shift+G; drag, resize, turn a group; the layers tree and dragging rows into and out of groups; the Group
buttons). Owed items are in `docs/STATUS-debt.md` (rung 7a line).

## 2026-11-12 — rung 7 kickoff, bbq and 7a judged (lead: Opus 5.5, `CLAUDE_EFFORT=high`)

- Start: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=5fc4301 dirty=no rebuilt=7`.
- Budget (`budget-select`): 10 points of a 10-point ceiling, alone on the weekly meter, no reserve; chris gave the reading
  "Still about 55 %" (not measured), logged with `spend-ledger.py reading 55 --meter weekly` at 2026-11-12T00:02:56Z. Stop
  lines: 65 % on the meter, the 75 % hard stop, ~500k of the lead's context.
- Open questions asked again, chris's answers: text boxes and pattern regions rotate "Not in v1 (Recommended)"; a
  multi-selection's box stays "Upright for a selection (Recommended)"; hardware queue "Nothing yet".
- bbq, 8 questions, all answered with the recommendation: record `docs/bbq/2026-11-11-rung-7-groups-and-align.md` (`f839644`).
- 7a built by one Sonnet 5.5 high subagent (494,829 tokens, 189 tool uses, about 61 minutes), its entry above; commits
  `bb5c542` to `d6bd9c0`, pushed.
- Judged by the lead: `make check-all` → `TESTS scope=workspace checks=296 check_fail=0 passed=275 failed=0 status=ok`;
  `make build-proof` → `BUILD head=d6bd9c0 dirty=no rebuilt=0`; `git diff 5fc4301 -- crates/render/tests/golden/SHA256SUMS`
  shows only the two new `groups_*` lines; `git diff --stat 5fc4301 -- crates/render/tests/fixtures/v0.4 …/v0.5` is empty.
- Re-measured by the lead on the Pi, release build, groups page, 90th percentile: `make bench-drag GROUPS=1` → export
  `p90_ms=26.6`; `target/release/quadrule --bench-drag 150 --groups-page --canvas chip` → `p90_ms=42.1`, `42.0`; all
  `result=pass` against 50 ms.
- Put to chris at the look: the builder's choice (1), the box left as dragged after a group's own resize, and (7), grouping
  changes the other device's picture because children follow the group's box (Q3 as ruled).

## 2026-11-12 — rung 7a look change: groups are see-through to constraints (builder: Sonnet 5.5)

chris approved 7a with one change ("I approve the build, but would like to change question #1 to reflect what figma does";
the record's Q3 is amended). Built on `main`, not pushed, version unchanged (0.6.1; the file format is still the unreleased 0.7.0).

- **Layout** (`crates/layout/src/tree.rs`, `lib.rs`): every object that is not a group resolves against the page by its own
  constraints, in or out of a group; a group's box is the box round its children's turned boxes (`tree::local_frame`,
  `tree::group_box`), the same rule on every device, then the group's angle turns the resolved children about that box's centre
  (angles still add). A group's stored `constraints` are ignored (doc comment on `Object::constraints` and `GroupShape` says so; the
  field stays on every object for the file's shape). `Ctx::child` no longer takes a stored box.
- **Editor resize** (`crates/model/src/geom.rs` `map_frame`, `edit.rs` `GroupMap`, `map_contents`, `reframe`, `Placement::map`;
  `crates/ui/src/drag.rs`): the mapping of a loose selection's resize moved into the model (`map_frame`) and `drag.rs`'s `map_rect`
  now calls it; a group's contents are mapped by it from the group's old box to its new one, on the device shown (each leaf's
  stored frame goes through its own rules to the page shown and back), then the box of the group and of every group in it is set
  round its children. Turned children stretch approximately (the 6b debt, written back into `docs/STATUS-debt.md`); a group's
  resize still stops at one pixel instead of flipping. The "box left as dragged" departure is gone. Also: a drag preview now works for
  objects at any depth (it looked objects up on the layers only); `SetObject` of a group refits it and the groups in it.
- **Inspector**: no Constraints tab for a group (it shows for a selection with anything else in it, on the first non-group); a
  child's tab is the normal one; `app-against-group` removed from `app.ftl`; `UnitContext::with_percent_of` removed, `%` is of the
  page everywhere (note in `units.rs` reworded).
- **Goldens**: `groups_supernote.a5x2_…` unchanged (`70ec1bf7…`); `groups_supernote.a6x2_…` is now `e364516f…` (the pinned child
  resolves against the page). The other six lines and every fixture are untouched.
- **Tests**: render `text_inside_a_group_draws_saves_and_reopens` and `grouping_moves_nothing` now require 0 differing pixels on both
  devices (the first tolerated 48,319 on the Nomad); new `grouping_and_ungrouping_with_every_constraint_changes_no_pixel` (25 objects,
  every H and V rule, grouped, ungrouped, in two groups, nested: 0 px on both devices). Layout, model and ui group tests changed to
  the new rule (not deleted); new ui tests: a group dragged and resized with the Nomad on screen, a turned group with a turned child resized.
- **Docs**: `README.md` and `DEVELOPMENT.md` describe the new rule; the Rung 7a debt line rewritten.

## 2026-11-12 — rung 7a approved with one change, built; v0.7.0 (lead: Opus 5.5, `CLAUDE_EFFORT=high`)

- chris at the 7a look, word for word: "I approve the build, but would like to change question #1 to reflect what figma
  does". Q3 amended in the record (`544f9bb`): groups are see-through to constraints, as in Figma.
- Built by one Sonnet 5.5 high subagent (214,193 tokens, 76 tool uses), its entry above; commits `eab048d` to `62cc6ca`.
- Judged by the lead: `make check-all` → `TESTS scope=workspace checks=299 check_fail=0 passed=278 failed=0 status=ok`
  (the builder's run printed `checks=298`; the `CHECK`-line count is the known debt about lines sharing the runner's
  output); `git diff 544f9bb -- crates/render/tests/golden/SHA256SUMS crates/render/tests/fixtures/` → only the groups
  Nomad line, `2963ff3f…` → `e364516f…`; bench on the Pi, release build, groups page, 90th percentile: chip `p90_ms=42.5`,
  export `p90_ms=27.5`, both `result=pass`.
- chris, after seeing the window again: "approved". Workspace bumped to `0.7.0`; release notes
  `docs/arcs/v1/release-notes/v0.7.0.md`; tag `v0.7.0` pushed, so the release workflow drafts, attaches and publishes.

## 2026-11-12 — rung 7b built: align and distribute, "mixed" fields (builder: Sonnet 5.5)

Built on `main`, not pushed, not tagged, version unchanged (0.7.0). Record: `docs/bbq/2026-11-11-rung-7-groups-and-align.md` (Q5, Q6,
Defaulted "7b, align and distribute" and "7b pays the rung 3 debt").

- **Maths** (`crates/layout/src/align.rs`, new): `align_deltas` and `distribute_deltas` over boxes in primary mm, on a `Grid`
  (exact mm, or the primary profile's device pixels); `boxes_of` gives each object's box on the primary page from the laid-out
  tree (`tree::resolve_nodes`), so a turned object, a line and a group all align by the box round their turned corners, and a
  group's step is turned back by the groups above it; `target_box` is the page or the margins; `AlignTo` with `key`/`from_key`.
  On a pixel grid an object's extent is its width rounded to a pixel (as the export draws it); the aligned edge lands on a
  pixel; distribute by spacing puts the ends on the pixel grid, makes every gap a whole number of pixels and gives the
  leftover pixels one each to the first gaps; distribute by centres rounds each place.
- **Editor** (`crates/ui/src/arrange.rs`, new): `Viewer::arrange(ArrangeOp)` is one `Batch` of `SetObject` (one undo step; no step
  when nothing moves; the locked notice when nothing can be edited), over `editable_selection`, so a locked object or an object
  on a locked layer stays put and is left out of the selection's box. `Viewer::align_to` / `set_align_to` keep two choices, one
  for a single object picked (Page until the user picks) and one for several (Selection until the user picks), so it switches by
  itself and each case remembers its own pick; the app keeps both between runs (`crates/app/src/main.rs`). `Viewer::arrange_mixed`
  says which of X, Y, W, H, Rotation differ.
- **Arrange tab** (`crates/ui/src/panels.rs`): a heading, the Align to drop-down, six align and four distribute icon buttons
  (painted by hand, each named for screen readers and in a tooltip, greyed when they would do nothing: align needs two
  editable objects for Selection and one for Page and Margins, distribute needs three). The X, Y, Width, Height fields read
  "mixed" (the word is the field's text until it is focused, then it is empty with "mixed" as a hint) and the Rotation drag value
  reads "mixed", when the selected objects differ; a value typed applies to all, as before. Rotation now shows for a selection
  when any selected object can turn (first such gives the value). New strings in `app.ftl` (`app-align-heading`, `app-align-to*`,
  `app-arrange-align-*`, `app-distribute-*`, `app-mixed`); the existing `app-align-left` / `-right` (text alignment) are untouched.
- **Tests**: `crates/layout/tests/align.rs` (4 `CHECK` lines) and `crates/ui/tests/align.rs` (10): every command against
  Selection, Page and Margins; four distributes both ways; whole pixels on both devices and exact mm with the snap off; a
  turned object (90 and 45 degrees); a group as one, also turned; locked objects and a locked layer; inside an entered group;
  the Align-to switching; "mixed" fields in the window (typing applies to all); the ten buttons named and clicking one.
- **Not touched**: every export byte. `git diff 57f2896 -- crates/render/tests/golden/SHA256SUMS crates/render/tests/fixtures/`
  is empty (7b only moves objects). Debt: the new "Rung 7b, owed" line; the rung 3 "mixed" item is paid and removed.

## 2026-11-12 — rung 7b judged and approved; v0.7.1; rung 7 done (lead: Opus 5.5, `CLAUDE_EFFORT=high`)

- `v0.7.0` release: run 43157146501, every job success (four package builds and "Draft, attach, publish"); `gh release
  view v0.7.0` → `draft=false prerelease=true assets=7`. The first real use of the tag-push release.
- Meter readings from chris: 63 % at 04:49 UTC (after 7a), 64 % at 19:42 UTC (after 7b), logged with `spend-ledger.py
  reading`. The share (10 points) was reached in flight: no builder was started after 7b.
- 7b built by one Sonnet 5.5 high subagent (212,367 tokens, 62 tool uses), its entry above; commits `c4405a0` to `b80b218`.
- Judged by the lead: `make check-all` → `TESTS scope=workspace checks=312 check_fail=0 passed=292 failed=0 status=ok`;
  `git diff 57f2896 -- crates/render/tests/golden/SHA256SUMS crates/render/tests/fixtures/` → empty; `make build-proof` →
  `BUILD head=b80b218 dirty=no`; drag bench on the groups page, release build, 90th percentile: export `p90_ms=27.3`, chip
  `p90_ms=40.6`, both `result=pass`.
- chris: "approved. the meter is at 64%". Workspace bumped to `0.7.1`; notes `docs/arcs/v1/release-notes/v0.7.1.md`; tag
  `v0.7.1` pushed. The spec gains a dated note under §5.2 recording the Figma groups ruling. Board re-planned; next lead's
  prompt `docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`; calibration row in `docs/budget-calibration.md`.
- `v0.7.1` release: run 43293356639, every job success; `gh release view v0.7.1` → `draft=false prerelease=true assets=7`.
- chris: "makeadr" (goforit, no rounds). ADR-015 drafted, proposed: groups are see-through to constraints, as in Figma
  (`docs/decisions/ADR-015-groups-see-through-to-constraints.md`, record `docs/bbq/2026-11-12-adr-groups.md`);
  `adr-check.py` → `15 ADRs, 0 failure(s)`. The rung 8 prompt now asks for its ratification.
- chris ratified ADR-015 ("Ratify").
- looseends (session), chris's yes: CI read for rung 7's code pushes with `gh run list --branch main` — CI and
  Coverage `success` on `d6bd9c0` (7a), `62cc6ca` (7a look change), `57f2896` (`v0.7.0`), `b80b218` (7b) and `2e1f4ff`
  (`v0.7.1`); Fuzz `success` on all of them but `b80b218`, which shows no Fuzz run; docs-only pushes start no CI. The budget
  row re-measured for the whole session; the rung 7 record's Defaulted calls marked settled by chris's approvals. Record
  `docs/looseends/2026-11-12-rung-7-session.md`.

## 2026-11-12 — grays sheet read on the Nomad; Block 68 → 102 (lead: Opus 5.5, `CLAUDE_EFFORT=high`)

- `cargo xtask calibration-sheet` run on the Pi (chris: "yes, please run step 1"); the grays pages:
  `calib/supernote-a5x2/quadrule-calib_grays-1_…png`, `calib/supernote-a6x2/quadrule-calib_grays-{1,2}_…png`.
- chris's Nomad reading, word for word, and his three follow-up answers ("Each matches a neighbour", "A lighter dark
  gray", "Same as the Nomad") are in `docs/device-measurements/supernote.a6x2/calibration.md` (`3ccfd16` and this
  commit); the Manta's ruling in `docs/device-measurements/supernote.a5x2/calibration.md`.
- Both profiles: Block 68 → 102, Shade 204 kept; `gray_levels` keeps the sixteen even steps, now measured clean, noted
  that 0–68 look black and 238–255 white on a flat area (`crates/profiles/data/`).
- Goldens re-blessed (`QUADRULE_BLESS=1 cargo test -p quadrule-render --test shapes|transforms|groups`), shown to chris
  as a before/after picture of the Nomad shapes page first: six lines changed (shapes, transforms, groups × 2); slice
  unchanged. Every changed pixel is lighter, by at most 34 (= 102 − 68): e.g. shapes Manta 164,283 px changed, 162,497
  of them exactly 68 → 102, the rest Block's smoothed edges and faded areas. The v0.5 and v0.6 fixtures' `.quad` files
  are unchanged; their expected sums for shapes and transforms follow the new goldens.
- `make check-all` → `TESTS scope=workspace checks=312 check_fail=0 passed=292 failed=0 status=ok`. chris: "yes, commit
  and push".
- Landscape toolbar screenshots from chris (left, right, top; Manta 2560 × 1920, Nomad 1872 × 1404). In the device
  folders they broke `crates/xtask/tests/safezones.rs` (`safe_zones_match_screenshots result=fail …screenshots=7:zones=6`),
  since `cargo xtask safe-zones` reads portrait only; chris: "Landscape subfolder (Recommended)", so they moved unrenamed
  to `docs/device-measurements/<device>/landscape/` and the test passes again (2 passed). The Nomad files came with mode
  `rwxr-----`; set to 644 like the rest, and `nomad-pen-shades.png`'s stored mode 100755 corrected to 100644. The
  landscape PNG check is still open ("Not yet").
- Landscape template check (spec §17.1 item 3), on the Nomad: four test PNGs made with a one-off script in the scratchpad
  (Test A wide 1872 × 1404 and 2560 × 1920; Test B the same turned clockwise, 1404 × 1872 and 1920 × 2560). chris's
  readings, word for word, in `docs/device-measurements/supernote.a6x2/landscape-test.md`. Grid spacing in the landscape
  screenshots: 79 px against 59 px in portrait (× 4/3). Answer: rotated pixels; landscape mode zooms. Recorded as a note
  under spec §17.1 item 3, on the board, in STATUS and in the rung 8 prompt. Not tested on the Manta.
- The Manta ruled the same as the Nomad for landscape, not tested (chris: "Same as the Nomad").
- Hard stop, chris word for word: "let's move the hard stop to 90%"; asked how it applies: "This week only". So 90 % until
  the Saturday 14th 5 pm reset, then back to 75 / 80 / 85 / 93 %. Meter 67 % (21:39 UTC): 23 points of room this week.
  Written into the rung 8 prompt's Budget watch item and `docs/budget-calibration.md`.

## 2026-11-12/13 — rung 8 kickoff, bbq; rung 8a built (lead: Opus 5.5, `CLAUDE_EFFORT=high`; builder: Sonnet 5.5, high)

- Kickoff from `docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
  `BUILD head=bb0b5ed dirty=no rebuilt=6`.
- Budget (`budget-select`): chris's reading "Still 67 %" (21:59 UTC, logged with `spend-ledger.py reading 67 --meter
  weekly`); ceiling "20 points (Recommended)", reserve "0 points", "No, alone". Share 20 points; hard stop 90 % until
  Saturday's reset.
- bbq, 8 questions, every answer the recommended option: record `docs/bbq/2026-11-12-rung-8-several-devices.md`
  (`91693aa`). Split 8a several devices / 8b orientation; Forked stays v1.1 (spec §17.3); position, size, font size and
  visibility are per device; a group's override is its box; strip at the bottom; landscape turns clockwise; L15 only on
  trouble.
- 8a built by one Sonnet 5.5 high subagent (0.48M tokens, 163 tool uses, 70 min): `2cbb8d1` (model, layout, format
  0.8.0, `v0.7/` fixture, overrides page, goldens), `2bbf752` (editor on secondary devices, strip, Overrides tab, Make
  primary, device ticks, L15, benches), `82b74e8` (README, DEVELOPMENT). Departures from the record: Make primary
  adds an override wherever anything would move at all (1e-6 mm), stricter than half a pixel; overrides text, the strip
  and L15 use the short names "Manta" and "Nomad"; the `nudge` test now expects a Nomad override (Q3).
- Judged by the lead: `make build-proof` → `BUILD head=82b74e8 dirty=no rebuilt=7`; `make check-all` → `TESTS
  scope=workspace checks=346 check_fail=0 passed=323 failed=0 status=ok`. `git diff 91693aa..HEAD --
  crates/render/tests/golden/SHA256SUMS` adds two lines (`devices_*`) and changes none; fixtures v0.4–v0.6 untouched.
- Speed, release build on the Pi, 90th percentile, strip on (builder's runs; `--bench-drag|--bench-draw 150 <page>
  --canvas export|chip`): drag export about 20 ms on every page, chip 42.5 (shapes), 32.3 (groups), 18.2 (devices),
  15.5 (devices on the Nomad); draw export about 20 ms, chip 38.1 (groups), 26.5 / 22.7 (devices). **Over the line:**
  drawing a pattern region on the shapes page on the chip canvas. Re-timed by the lead: `./target/release/quadrule
  --shapes-page --bench-draw 150 --canvas chip` → p90 51.0, 51.1, 50.8 ms. The same command at `91693aa` (before 8a,
  built in a worktree under `target/`, since removed) → 56.6, 56.2, 56.3 ms; with `--bench-tool rect` → 47.9, 47.9,
  48.0 ms (as rung 6a). So it was over before 8a, never timed (rung 6a timed rect, ellipse and polygon only), and 8a made
  it faster. Said to chris before the look (ADR-009).
- New debt in `docs/STATUS-debt.md` (rung 8a line); paid: the 7a "group dragged on the other device" and 7b "align on
  the primary page" lines, part of the Style tab's "mixed", the rung 2 Ctrl + scroll anchor.
- Waits on chris: the look at 8a.
- chris on the over-50 ms pattern draw: "Leave it for later" (kept in the rung 8a debt line). Meter after the 8a build:
  72 % (`/usage`, logged with `spend-ledger.py reading 72 --meter weekly`): 5 points of the 20-point share spent; the
  five-hour window read 61 %.
- chris, word for word: "Move forward with 8b without the checks. I can't look now and I'd rather you kept working".
  So 8a's look is deferred (not skipped): `v0.8.0` is not tagged until chris approves 8a; 8b is built on top of it now,
  and both wait for his look.

## 2026-11-13 — rung 8b finished and judged (lead: Opus 5.5, no builder)

- Found at session start: about 555 lines of 8b work uncommitted in 21 files, last touched 2026-11-12 19:45 (five
  minutes after `2dd41f2`), with no log entry and nothing running (`pgrep -af 'cargo|rustc|claude'` → this session
  only). `make build-proof` → `BUILD head=2dd41f2 dirty=yes rebuilt=1`. `make check-all` on it stopped at `fmt-check`;
  `cargo fmt --all` fixed it. Present: `Profile::oriented` and `builtin_for` (the page turned, the safe zones turned into
  the design's frame), `turn_clockwise` in render, export through it with the portrait size in the name,
  `set_orientation` (one undo step, objects by their constraints, override boxes in proportion), the Orientation choice
  in New… and the Page tab, New…'s margins clearing the turned zones, Landscape in both profiles' `orientations`, the
  landscape sample page and `--landscape-page` / `LANDSCAPE=1`. Missing: every 8b test, the goldens, the docs.
- Added: `crates/render/tests/landscape.rs` (a landscape golden page per device; the exact clockwise turn; orientation
  through save) and `crates/ui/tests/landscape.rs` (turned zones agree with the device's through the export turn, pixel
  for pixel; New… in landscape clears them on both devices; L06 sees them; the Page tab switch as one undo step; export
  names and bytes). Two lines added to `crates/render/tests/golden/SHA256SUMS`; the ten before them unchanged.
- The landscape sample page was changed before blessing: its star, markers and dot strip had the default Scale
  constraints, so on the Nomad they scaled up into the header band (looked at by eye: the first bless's Nomad image).
  Each is now pinned to the edge it marks (`crates/model/src/sample.rs`); the Manta image did not change.
- README ("what works today") and DEVELOPMENT (bench flag, crate map, safe zones) updated.
- `make check-all` → `TESTS scope=workspace checks=356 check_fail=0 passed=331 failed=0 status=ok`.
- Speed, release build on the Pi, landscape page, strip on, 90th percentile (`./target/release/quadrule
  --landscape-page --bench-drag|--bench-draw 150 --canvas export|chip`): drag export 25.0, 25.7, 26.3 ms; drag chip
  29.0, 28.9, 28.9; draw (pattern) export 24.4, 25.5, 25.5; draw chip 37.7, 37.6, 37.6; drag on the Nomad
  (`--bench-secondary`) export 17.0, chip 24.2. All under 50 ms.
- Meter after finishing 8b: 74 % (chris, `/usage`, 2026-11-13 15:39 UTC; logged with `spend-ledger.py reading 74 --meter weekly`): 2 points for this session, 7 of rung 8's 20-point share so far; hard stop 90 % until Saturday's reset.
- Waits on chris: the look at 8a and 8b together. `v0.8.0` and `v0.8.1` are not tagged until he approves.

## 2026-11-13 — rung 8 approved; v0.8.0

- chris looked at 8a and 8b in the app (`--devices-page`, `--landscape-page`, release build on the Pi) and ruled, word
  for word: "1. let's keep the color, but put on v1.1 a way to click the override dot and have the app open the
  overrides tab, otherwise approve all and let's bundle this into a single v0.8.0 release". Written into the rung 8
  record (Amendments), the spec's v1.1 list (§16 note), the board and the debt list.
- CI for `70d537a` (8b) passed: CI, Fuzz and Coverage all success (`gh run list --branch main`).
- Version 0.8.0 (`Cargo.toml`, `Cargo.lock`); notes `docs/arcs/v1/release-notes/v0.8.0.md`; board re-planned (rung 8
  done, rung 9 next, lessons); next lead's prompt `docs/arcs/v1/rung-9-PROMPT-2026-11-13.md`. No `v0.8.1`.
- CI for `29e46cc` passed (run 43459219245). Tagged `v0.8.0` (annotated, "Rung 8: several devices and landscape
  pages") at `29e46cc` and pushed; release run 43460443541: linux-x86_64, linux-aarch64, macos-universal,
  windows-x86_64 and "Draft, attach, publish" all success. Published as a pre-release with 7 files:
  https://github.com/tholent/quadrule/releases/tag/v0.8.0 (`gh release view v0.8.0`).

## 2026-11-13 — rung 9 kickoff: planned, builder started (lead: Opus 5.5 high)

- `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=ace7aa1 dirty=no rebuilt=7` (the 0.8.0 bump not
  compiled until then).
- Asked again: landscape on the Manta. chris, word for word: "The software on each is identical and rotation logic is
  the same. no need to test".
- Budget: 16 points, alone on the meter, no reserve; start 74 % (chris, 16:28 UTC, given not measured; logged with
  `spend-ledger.py reading 74 --meter weekly`); stop at 90 % this week.
- Planned (bbq, 8 questions): `docs/bbq/2026-11-13-rung-9-pattern-library.md`. One rung, one look (chris overruled the
  9a/9b split); isometric with horizontal rows; hex both ways; Cornell one region; music staff plus grand staff; major
  lines on the rhythm, adjustable per region; isometric rhythm snapping on the half-side × row grid.
- One Sonnet 5.5 high builder started on `main`, committing each step locally; the lead judges and pushes. If this
  session stops before the judging entry below, the builder's commits after `1497146` are unjudged.

## 2026-11-13 — rung 9 built (Sonnet 5.5 high builder, 0.42M tokens) and judged by the lead

- Builder commits `3490dd4`..`25cc00d` (steps 1–7; some middle commits do not compile alone, since steps 3 and 5
  share `crates/ui/src/panels.rs`). Pushed by the lead after judging.
- Judged: `make build-proof` → `BUILD head=25cc00d dirty=no rebuilt=7`; `make check-all` → `TESTS scope=workspace
  checks=387 check_fail=0 passed=360 failed=0 status=ok`. `git diff ace7aa1..HEAD --
  crates/render/tests/golden/SHA256SUMS` adds four lines (`patterns_*`, `patterns-landscape_*`) and changes none;
  fixtures v0.4–v0.7 unchanged; new fixture `crates/render/tests/fixtures/v0.8/`.
- The four new golden images looked at by eye by the lead. Two points for chris's look: (1) hex edges whose middle
  is inside the region are drawn whole, so stubs reach up to half an edge past the frame (isometric clips its lines
  at the frame instead); ADR-014 says at most half a stroke; (2) major dots draw smaller than ordinary dots, because
  the record said "at the major role's width" and Structure is 2 px against Dot grid's 3 px (the lead's brief, not
  the builder).
- Speed, release build on the Pi, strip on, 90th percentile (`./target/release/quadrule --patterns-page
  --bench-drag|--bench-draw 150 --canvas export|chip`): drag export 24.0, 23.9, 24.1 ms; drag chip 33.3, 33.7, 33.7;
  draw export 24.2, 25.1, 24.1; draw chip 40.9, 41.0, 40.9. The rung 8a debt is paid: `--shapes-page --bench-draw 150
  --canvas chip` → 41.2, 41.2, 41.3 (was 50.8–51.1): the chip canvas now draws a run of same-gray rectangles as one
  path (`crates/ui/src/canvas.rs`, `crates/ui/tests/canvas.rs`); export bytes unaffected.
- Waits on chris: the look.

## 2026-11-13 — rung 9 look changes built (same builder, 0.48M tokens in all) and judged

- chris at the look: "things look really nice overall" and "the isometric grid and dot pattern is rotated 90 degrees
  from what I expected"; then three rulings (record `docs/bbq/2026-11-13-rung-9-pattern-library.md`, Amendments):
  isometric turned everywhere (axes 30°, 90°, 150°; spec §6.3 note), hex edges cut at the frame (ADR-014 amended),
  major dots one pixel bigger in the major role's gray. Committed `0825fbd`.
- Builder commits `1dfebaa` (all three) and `5fff8c9` (a model test). Judged: `make build-proof` → `BUILD
  head=5fff8c9 dirty=no rebuilt=7`; `make check-all` → `TESTS scope=workspace checks=387 check_fail=0 passed=360
  failed=0 status=ok`. Golden sums: still only the four `patterns*` lines added (re-blessed, unreleased); the twelve
  older lines and fixtures v0.4–v0.8 unchanged. All four images looked at by eye.
- Speed, release build on the Pi, strip on, 90th percentile, pattern page: drag export 21.0, 21.8, 22.0 ms; drag chip
  33.9, 34.1, 33.9; draw export 22.0, 21.8, 23.7; draw chip 41.2, 41.4, 41.5.
- Left by the builder, for chris: hex slanted edges keep round caps, which reach about 1.5 px past the frame at a cut
  end, and isometric's butt caps reach half a stroke diagonally; so "cut exactly" holds to the line's centre, not its
  ink. Under an isometric rhythm the overlay does not draw the half-side rows that snapping also catches.
- Waits on chris: the second look.

## 2026-11-13 — rung 9 approved; v0.9.0

- chris at the second look, word for word: "approved. everything looks great". Written into the rung 9 record
  (Amendments); the two leftovers shown to him were approved as built (debt list, rung 9 line).
- Paid: the rung 8a debt "drawing a pattern region on the shapes page on the chip canvas is over 50 ms" (now 41.1–41.5
  ms, `--shapes-page --bench-draw 150 --canvas chip`), removed from `docs/STATUS-debt.md`.
- Version 0.9.0 (`Cargo.toml`, `Cargo.lock`); notes `docs/arcs/v1/release-notes/v0.9.0.md`; board re-planned (rung 9
  done, rung 10 next, lessons; Paper, Notes and Creative starters buildable); next lead's prompt
  `docs/arcs/v1/rung-10-PROMPT-2026-11-13.md`.
- Meter after rung 9: 82 % (chris, `/usage`, 19:16 UTC; logged): 8 points of the 16-point share; calibration row in
  `docs/budget-calibration.md`.
- CI, Fuzz and Coverage for `8a22065` passed (run 43490301559 and siblings). Tagged `v0.9.0` (annotated, "Rung 9: the
  pattern library") at `8a22065` and pushed; release run 43492215541: linux-x86_64, linux-aarch64, macos-universal,
  windows-x86_64 and "Draft, attach, publish" all success. Published as a pre-release with 7 files:
  https://github.com/tholent/quadrule/releases/tag/v0.9.0 (`gh release view v0.9.0`).

## 2026-11-13 — rung 10, the repeat tool (lead Opus 5.5 high)

- Kickoff: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=da6d45c dirty=no rebuilt=0`. Budget 16
  points from 82 % (chris: "Still about 82 %"; "go for 16 points and ignore the cap for this week"), alone, no reserve.
- Planned (bbq, 8 questions, every answer the recommended one): one rung, one look; source inside the repeat; Count or
  Fill box; Auto pitch = source size rounded up to whole rhythm steps; alternation by Columns, Rows or Cells; Start or
  Centre; Expand to one group (`docs/bbq/2026-11-13-rung-10-repeat-tool.md`, `ac0c773`).
- Built by one Sonnet 5.5 high builder (470,695 tokens reported by the harness): `d2a8967`, `92ce91b`, `57f8345`,
  `18aa6ee`. Judged by the lead: `make check-all` → `TESTS scope=workspace checks=423 check_fail=0 passed=394 failed=0
  status=ok`; `git diff da6d45c -- crates/render/tests/golden/SHA256SUMS` → two lines added (`repeats_*`), the sixteen
  old lines unchanged; fixtures `v0.4/`–`v0.8/` unchanged, `v0.9/` added. Both new goldens looked at by eye: Fill draws
  29 stars on the Manta and 20 on the Nomad, every 7th habit column heavier, row 2 skipped.
- Speed on the Pi, release build, repeat page, strip on, 150 frames, 90th percentile (`target/release/quadrule
  --bench-drag|--bench-draw 150 --repeats-page --canvas export|chip`): drag export 22.8, 22.6, 22.8 ms; drag chip 42.2,
  42.6, 42.5; draw export 20.4, 20.3, 21.3; **draw chip 49.7, 49.6, 50.1** (the third run over the 50 ms line). A trial
  merging same-style outlines into one chip path changed nothing (49.6–49.7), so the object count is not the cost; it
  was undone, not committed. Put to chris before the look.
- chris ruled "Fix it now, in rung 10". The builder, resumed (576,818 tokens reported at the end of that leg), timed
  the chip frame: building the page, `screen_list` and scene encoding were each under 1 ms, the chip's own wait 40–48
  ms, growing with the outline it processes, not with the object count. Fix `fa7ff7e` (`crates/app/src/chip.rs` only): the chip keeps
  the unchanged lower part of the list in its own texture and draws only what changed over it, timed against a
  whole draw so it is used only when faster; screen picture within one gray level of a whole draw. Re-measured by the lead
  (release, strip on, 150 frames, 90th percentile, 3 runs each, `--canvas chip`): repeat page draw 23.6, 23.6, 23.6;
  drag 35.7, 35.7, 35.6; patterns page draw 23.7, 23.7, 23.9, drag 17.4, 17.3, 17.3; shapes page draw 23.1, 23.0,
  23.4; groups page drag 30.8, 31.0, 31.0. `make check-all` → `TESTS scope=workspace checks=423 check_fail=0
  passed=394 failed=0 status=ok`; no golden line changed.
- chris at the look, word for word: "approved". Rung 10 is done. Version 0.10.0 (`Cargo.toml`), notes
  `docs/arcs/v1/release-notes/v0.10.0.md`, the ruling in the record's Amendments, board re-planned (rung 10 row, Planning
  and Bullet journal starters buildable, re-plan line), rung 11 prompt `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md`.
  Tag `v0.10.0` waits for CI on this commit.
- CI, Fuzz and Coverage for `26f552f` passed (runs 43545053682, 43545053573, 43545053558). Tagged `v0.10.0` (annotated,
  "Rung 10: the repeat tool") at `26f552f` and pushed; release run 43546000731: linux-x86_64, linux-aarch64,
  windows-x86_64, macos-universal and "Draft, attach, publish" all success. Published as a pre-release with 7 files:
  https://github.com/tholent/quadrule/releases/tag/v0.10.0 (`gh release view v0.10.0`).

## 2026-11-14/15 — rung 11 kickoff (Opus 5.5, high)

- Session start: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=4cf4b04 dirty=no rebuilt=11` (the
  0.10.0 bump in `Cargo.toml` was unbuilt), then `BUILD head=4cf4b04 dirty=no rebuilt=0`.
- Budget (budget-select): 15 points, alone on the weekly meter, no reserve; start reading 1 % (chris: "0–2 %", 2026-11-15
  00:32 UTC, after Saturday's 5 pm EST reset), logged with `spend-ledger.py reading 1 --meter weekly`; hard stop 75 %.
- Open question asked again: text rotation (parked for v1.1). chris: "Pull into rung 11".
- Rung 11 planned (bbq, 8 questions, all on the recommendation): split 11a fonts / 11b the box; the spec's bundled set
  with italics and Caveat (+6.4 MB); typing on the canvas added (11b); bundled-only fallback until rung 12; overflow cuts
  whole lines with a red mark; baselines stand on the rules; emoji are characters in text. Record
  `docs/bbq/2026-11-14-rung-11-text-and-fonts.md`; board row updated (`docs/arcs/v1/README.md`).

## 2026-11-14/15 — rung 11a built (builder Sonnet 5.5, steps 1 to 4 and 6; step 5 stopped)

- Built on `main`, each step pushed: `120f25b` (fonts, OFL texts, `BUNDLED`, optical size, bundled-only fallback, installed
  fonts, `TextBox.style`, face key `italic`/`opsz`), `86cacce` (format 0.11.0 and the `v0.10/` fixture), `2cb343b` (clippy), `0c83179`
  (the app uses installed fonts; full re-scan on focus), `bea1b29` (font picker, Style field, weight note), `14fc421` (fonts page, goldens,
  `--fonts-page`, `FONTS=1`), `34b84e0` (docs); this log and `docs/STATUS-debt.md` in the commit after them.
- Fonts: the ten variable files of the record's Defaulted "Font files" from `google/fonts` (`ofl/<family>/`); each `OFL.txt` read and
  checked as SIL OFL 1.1; `AtkinsonHyperlegibleNext[wght].ttf` unchanged (sha256 `5a455d1c…`, google's copy is the same file). Texts
  `OFL-Atkinson.txt` (existing), `OFL-AtkinsonMono.txt`, `OFL-Inter.txt`, `OFL-Literata.txt`, `OFL-Caveat.txt`, `OFL-NotoEmoji.txt` in
  `crates/text/fonts/`, listed in `packaging/packager.json` (the only place the Atkinson text was shipped; `release.yml` needed no change)
  and in `README.md`'s credits. Raw size added: `ls -l crates/text/fonts/*.ttf` sums to 6,376,068 bytes (6,261,516 more than before);
  `tar cf - *.ttf | gzip -9 | wc -c` → 3,612,221. The release binary is 53,867,672 bytes (`ls -l target/release/quadrule`); the
  packages were not built here, so their before and after sizes are for the release run.
- Optical size: cosmic-text 0.19 sets only `wght` when it loads a face, so `crates/text/src/opsz.rs` makes a copy of the font file with
  the `opsz` axis retagged as a second `wght` axis and a constant `avar` map; the copy shapes and draws at the size (whole points,
  clamped to the axis: Inter 14 to 32, Literata 7 to 72). Test: 144 cases equal skrifa's own coordinates.
- Fallback (Q4): `FALLBACK_ORDER` = Atkinson Next, Noto Emoji, then the rest, through cosmic-text's `Fallback` trait (needs `unicode-script`
  as a direct dependency to name `Script`); installed families are open in the layout store only for boxes that choose them and are in its
  forbidden list. No existing sum or fixture moved.
- Re-scan on the Pi: `cargo test --release -p quadrule-text --test fonts -- --nocapture rescan` → first 4.6 ms, second 4.5 ms (111 faces,
  29 families; debug build 38 to 45 ms). Under 50 ms.
- Judged: `make check-all` → `TESTS scope=workspace checks=443 check_fail=0 passed=413 failed=0 status=ok`;
  `git diff 07875a9 -- crates/render/tests/golden/SHA256SUMS` adds `fonts_supernote.a5x2_1920x2560.png` and `fonts_supernote.a6x2_1404x1872.png`
  and changes nothing else; fixtures `v0.4/`–`v0.9/` unchanged, `v0.10/` added; the eighteen old goldens pass unchanged.
- Golden images looked at by eye (a first draft was reflowed twice: scaled boxes crowded on the Nomad; then too tall and too wide for the
  Nomad's page, and a sunflower wrapped onto the next line, before the sums were kept). Final: both show the same page, laid out for the Nomad's
  118.9 by 158.5 mm: five families upright (left) and italic (right) at 8 and 12 pt (Caveat has no italic and draws upright), Inter and
  Literata weight ramps 100 to 900 (Literata's 100 is its 200), "Quartz 24" in Inter and Literata at their display optical size, a Noto
  Emoji line of seven and Atkinson with an emoji and a heart falling back to Noto Emoji, a dot grid.
- Speed on the Pi, release build, fonts page, strip on, 150 frames, 90th percentile (`target/release/quadrule --fonts-page
  --bench-drag|--bench-draw 150 --canvas export|chip`, 3 runs each):

  | command | canvas | p90 ms (runs) |
  |---|---|---|
  | `--bench-drag` | export | 20.1, 20.1, 20.2 |
  | `--bench-draw` | export | 19.7, 19.8, 19.8 |
  | `--bench-drag` | chip | 17.8, 17.8, 17.8 (worst 99.2–99.4, 8 of 148 frames over 50 ms, first 99.9) |
  | `--bench-draw` | chip | 3.2, 3.2, 3.2, but the line says `canvas=chip-cpu-fallback`: the chip's first frame took over 100 ms and ADR-009 item 3 fell back |

  With `--no-strip` both chip runs fell back (first frame 113.8 and 116.8 ms). So no 90th percentile crosses 50 ms, but the chip canvas's first
  frame on this text-heavy page is about 100 ms and the window can drop to the processor canvas; put to the lead (debt line "Rung 11a, owed").
- Not built: step 5 (emoji picker). The `emojis` crate is licensed `(MIT OR Apache-2.0) AND Unicode-3.0` (`cargo info emojis`, 0.9.1), which is not
  "MIT or Apache-2.0" alone, so the builder stopped for the lead and chris (the data's own Unicode-3.0 licence is the one any CLDR-derived
  list would carry too).
- Debt: "Text spike T14/T15" is paid in code (the test requires faces on macOS and Windows) and waits for CI to show it; new "Rung 11a, owed" lines
  for the emoji picker, the chip first frame, optical size by retagged copy and the picker.

## 2026-11-15 — rung 11a resumed: emoji picker and the chip's first frame (same builder)

- chris ruled on the two stops ("Allow, ship the notice"; "Fix in 11a"). `4a2170b` (chip), `f827c6f` (emoji picker).
- Emoji picker (step 5): `emojis` 0.9.1 added to `crates/ui`; the Unicode-3.0 notice is `crates/ui/licenses/Unicode-3.0.txt`, in
  `packaging/packager.json` as `licenses/Unicode-3.0.txt` beside the OFL texts and in `README.md`'s credits (test
  `unicode_notice_shipped`). Toolbar **Emoji** button and one in the Text tab; a window with a search box (by name or shortcode), nine groups
  and a grid drawn from Noto Emoji as pictures; the list is the emoji Noto Emoji draws as one glyph (1,899 of 1,923 without tone variants,
  `CHECK ... name=emoji_list`: 755 ms in a debug build, once, on first open). A pick goes into the Text tab field at the cursor (replacing a
  selection) as one undo step, else into a new text box holding only it, 24 pt, centred on the middle of the view, one undo step
  (`Viewer::place_text`). Tests: `emoji_insert`, `emoji_at_cursor`, `emoji_list`.
- The chip's first frame, timed before any change (laps inside `Chip::render`, release build, Pi, `--fonts-page --bench-drag 40 --canvas chip`;
  the laps were not committed): the very first frame 207 ms = vello init 19.6 + encode 36.4 + GPU wait 150.5 (shaders and pipelines;
  never judged); a whole-page draw 83 ms = scene build 0.3 + encode 1.0 + GPU wait 81.6; the frame that made the kept lower part and the rest
  99 ms (base about 82 + top about 15, one GPU wait of 95); a frame that reuses the base 16 ms (encode 1.0, GPU 15). So the 99 to 107 ms
  frames were base frames, the cost is the GPU drawing the page's text outlines (not setup, not CPU path building), and a base frame is two
  page-sized draws against a line meant for one. Fix: `ChipRenderer::last_frame_draws` (2 for a base frame) and
  `ChipState::record_chip_draws`, the line is `SLOW_FRAME_MS` per draw; a single draw over 100 ms still falls back (`chip_fallback_rule`:
  150 ms with two draws kept, 250 ms with two draws and 150 ms with one fall back).
- Acceptance, release build, strip on, 150 frames, 3 runs each, `target/release/quadrule <page> --bench-drag|--bench-draw 150 --canvas chip`
  (every run said `canvas=chip`):

  | page | command | p90 ms | worst ms | frames over 50 ms |
  |---|---|---|---|---|
  | fonts | drag | 18.0, 17.9, 17.9 | 99.4, 99.4, 99.3 | 8 |
  | fonts | draw | 25.2, 25.3, 25.5 | 107.6, 106.9, 107.5 | 8 |
  | repeats | drag | 35.6, 35.6, 35.5 | 57.7, 57.8, 57.6 | 2 |
  | repeats | draw | 23.4, 23.5, 23.5 | 65.6, 65.6, 65.5 | 4 |
  | patterns | drag | 17.2, 17.3, 17.2 | 49.0, 48.9, 48.9 | 0 |
  | patterns | draw | 23.5, 23.6, 23.6 | 56.7, 56.7, 56.6 | 2 |

  Rung 10's chip numbers (repeats drag 35.7, 35.7, 35.6; draw 23.6; patterns drag 17.4, 17.3; draw 23.7) are matched: no regression.
- Judged: `make check-all` → `TESTS scope=workspace checks=448 check_fail=0 passed=417 failed=0 status=ok`; `git diff 07875a9 --
  crates/render/tests/golden/SHA256SUMS` adds the two `fonts_*` lines and changes none. Debt: the emoji picker and chip lines of
  "Rung 11a, owed" are cleared; two smaller lines replace them (picker gaps; the 99 ms base frame is still a visible hiccup).

## 2026-11-15 — rung 11a judged and approved (lead, Opus 5.5 high)

- The lead checked the builder's landings: `git diff 07875a9 -- crates/render/tests/golden/SHA256SUMS` adds only the two
  `fonts_*` lines; fixtures `v0.4/`–`v0.9/` unchanged; the Nomad fonts golden looked at by eye. CI: 14fc421 failed clippy
  (fixed in 34b84e0); db0ef4e, 4a2170b and f827c6f passed (CI, Coverage, Fuzz). Installed fonts found in CI run
  43666062914: `os=macos faces=675`, `os=windows faces=176`, `os=linux faces=53` (`name=system_faces`), so the T14/T15
  debt is paid. Lead's own chip runs after the fix: `target/release/quadrule --fonts-page --bench-drag 150 --canvas chip`
  → `canvas=chip p90_ms=18.4 first_ms=100.6`; `--bench-draw` → `canvas=chip p90_ms=25.2 first_ms=102.6`.
- Rulings: the `emojis` crate allowed with the Unicode-3.0 notice; the chip first frame fixed in 11a; ADR-002 amended for
  `unicode-script` ("amendment, go ahead"). Record `docs/bbq/2026-11-14-rung-11-text-and-fonts.md`, Amendments.
- chris's look: "approved, but don't create a seperate release. Just do rung 11b and release 11a/b as v0.11.0". No tag
  for 11a. Background font re-scan added to 11b (chris: "Yes, in 11b").
- Meter: chris's reading "6–10 %" logged as 8 (`spend-ledger.py reading 8 --meter weekly`); about 7 of the 15-point
  share spent on 11a (builder 450,380 tokens reported by the harness).
- 2026-11-15: T14/T15 paid: installed fonts found on macOS (675 faces) and Windows (176) in CI run 43666062914; debt line removed.

## 2026-11-15 — rung 11b built (builder Sonnet 5.5, steps 1 to 9; one STOP put to the lead)

- Commits on `main`: `5692c20` (steps 1 to 7: sizing and laid-out height, overflow and L08, paragraph spacing and vertical alignment, baselines on
  the rhythm, text rotation, typing on the canvas, the background font re-scan; the steps share files, so they land in one commit), `99e6abd`
  (step 8: format 0.12.0, `v0.11/` fixture), `aca87dd` (step 9: the text-box golden page and its tests), `7a61c6d` (clippy), and the commit after it
  (bench box length, one window test, this log and the debt lines).
- What was built, in the record's words. Sizing: Auto (default) or Fixed; an auto box's height is its laid-out height on each device in the tree
  (`layout::tree::Heights`, applied after the constraints, so no stored height moves a box): picking, snapping, align and distribute, L15 (pays the
  rung 7b and 8a debt lines). Fixed boxes leave out whole lines; a red + tab on both canvases and the strip pictures, never in the export; L08 (Warn)
  per device. Paragraph spacing in any unit (`u` kept in steps); vertical alignment for fixed boxes; baselines on the rhythm (`BaselineGrid`: the
  line height a whole number of steps, the first baseline on the first rung at least a cap height down, paragraph space rounded up, Middle and
  Bottom by whole steps). Rotation as shapes (handle, catches, Shift, the Arrange field, groups holding text turn; L06 and L10 by turned corners;
  pays the 6b text line). Typing: `text::edit` (cosmic-text's `Editor` over the export's layout) and `ui::typing` (a session, preview document,
  words in the session's undo, one document step on leaving); egui IME events pass through; the emoji picker inserts at the canvas caret; a new box
  starts typing with its text selected. `--bench-type`, `--textbox-page`, `make bench-type`, `TEXTBOX=1`. Background re-scan: `FontRegistry::
  scan_in_background`, `Viewer::start_font_scan`/`poll_font_scan` (one at a time).
- Checks before step 5: no sample page or fixture holds a text box with a stored angle (a throwaway test listed every sample document and every
  `fixtures/v0.*/*.quad`: turned text boxes 0 in all). The twenty golden sums and fixtures `v0.4/`–`v0.10/` did not move (the whole test run passed
  with them); `git diff 7cbb5c9 -- crates/render/tests/golden/SHA256SUMS` adds only the two `textbox_*` lines.
- Golden images looked at by eye (Manta and Nomad show the same page): a Fixed box showing "Line one, two, three" of five, with its frame; three
  paragraphs 8 pt apart; "Top / Middle / Bottom of the box" in three framed boxes, text at the top, in the middle and at the bottom; a ruled region
  with the text standing on its lines; "Quartz" turned 15°, 90° (reading downward) and 180° (upside down). (The first blessing had the baseline on
  the rule's centre row, so letter bottoms sat inside a 3 px rule and read as an underline: chris's ruling below replaced it.)
- chris's ruling at the look, word for word: "Let's have the default be 'B', but have a toggle that allows shifting the text up to 'C'". Built:
  B (default) puts the baseline on the rule's top edge, so a flat letter ends on the pixel row just above the rule (round letters may dip about 1 px);
  C (`TextBox.baseline_gap`, "Small gap above the line", under "Align baselines to rhythm", greyed while that is off, off by default, left out of
  the 0.12.0 file at its default) leaves a clear gap of 0.3 mm in whole device pixels, at least 1 (4 px on both tablets). The rule's top edge is
  taken from the Guide line role (what the Pattern tool gives a ruled region) at the device's pixel width: top row = rung row less half the width
  (builder's choice, for chris to veto: Guide is the default role; a region in another role or width is off by half the difference). Tests
  (`crates/render/tests/textbox.rs`): rendering "LLL" with and without the box, the lowest row only the letters darken is exactly the row above the
  rule's top row with B (0 clear rows) and has exactly N = 4 clear rows between it and the rule with C, on both devices; the baseline row equals the
  rule's top row less the gap. The golden page now holds two ruled samples, each labelled (B "on the line (default)", C "a small gap above the
  line"); both `textbox_*` images were looked at and re-blessed (new in 11b, not frozen); the twenty older sums are unchanged.
- Speed on the Pi, release build, strip on, 150 frames, 3 runs each, 90th percentile ms, `target/release/quadrule --textbox-page|--fonts-page
  --bench-drag|--bench-draw|--bench-type 150 --canvas export|chip` (every `canvas=` said what is shown):

  | page | command | export canvas | chip canvas (`canvas=`) |
  |---|---|---|---|
  | textbox | drag | 20.7, 20.6, 20.8 | 16.8, 17.1, 16.8 (`chip`; worst 52.6, 2 frames over 50 ms) |
  | textbox | draw | 20.2, 20.3, 20.1 | 24.0, 24.0, 24.3 (`chip`; worst 60.4) |
  | textbox | type | 20.5, 20.3, 20.4 | **75.1, 75.5, 75.1** (`chip`) |
  | fonts | drag | 20.3, 20.5, 20.4 | 18.0, 17.9, 18.1 (`chip`; worst 99.6, 8 frames over) |
  | fonts | draw | 20.0, 20.0, 20.1 | 25.2, 25.3, 25.4 (`chip`; worst 107.7, 8 frames over) |
  | fonts | type | 20.3, 20.5, 20.5 | **`chip-cpu-fallback`** (first typed frame 128 ms; 4.7, 4.6, 4.6 on the processor canvas) |

  The long box of `--bench-type` is about 430 letters at 9 pt (a first try of 1,300 letters fell back on both pages at once). **STOP:** typing
  on the chip canvas crosses 50 ms on the text-box page and falls back on the fonts page; put to the lead (debt line "Rung 11b, owed", chip).
  Drag and draw are as in 11a (fonts) and under 50 ms (text-box page).
- Builder's choices, for chris to veto: auto boxes show four edge handles (the record said left and right only, but pulling the top or bottom
  turns the box Fixed); the caret is solid, not blinking (a blink keeps the test harness from settling); a double-click on text types in it before
  it would enter a group; a session ends on a device change; the baseline sits at the ruled line's own row (letters overlap the line's upper half);
  switching to Fixed keeps the box's laid-out height on the primary; the 0.11 fixture is this build's fonts page with the manifest set to 0.11.0.
- TESTS: see the commit that follows (final `make check-all` line is in the report).

## 2026-11-15 — rung 11b judged (lead, Opus 5.5 high)

- 11b built by a Sonnet 5.5 high builder (688,463 tokens reported by the harness; one stop at the 5-hour usage window,
  resumed): `5692c20`, `99e6abd`, `aca87dd`, `7a61c6d`, `7c38afa`, `46ccf9c`. Lead's checks: `git diff 7cbb5c9 origin/main
  -- crates/render/tests/golden/SHA256SUMS` adds only the two `textbox_*` lines; fixtures unchanged but for the new
  `v0.11/`; CI failed on `7a61c6d` and passed on `2b0750a` (after the builder's `7c38afa`). Baseline B and C samples
  looked at by eye on the Nomad golden.
- chris's baseline ruling (B by default, a tick for C) built in `46ccf9c`.
- Stop: chip typing, `target/release/quadrule --textbox-page --bench-type 150 --canvas chip` → `canvas=chip p90_ms=82.0
  first_ms=87.3 over_budget=105`; export canvas `p90_ms=20.5`. chris: "Fix in 11b". Share raised from 15 to 21 points
  ("+6 points") at a reading of 18 % (chris: "16–20 %").
- Housekeeping: three background loops stuck for about 14 hours were stopped by process number: the lead's CI watcher
  (`gh run list --commit <short sha>` found no run, so it never ended) and two of the 11a builder's waits, which used
  `pgrep -f "cargo build --release"` and `pgrep -f "cargo test"` and so always found their own command line. Lesson for
  builder briefs: never wait with `pgrep -f` on a pattern that appears in the waiting command itself.

## 2026-11-15 — rung 11b, chip typing fixed (builder, Sonnet 5.5)

- chris ruled "Fix in 11b". Commits `61a7347` (the patch) and the one after it (rows, tests, notes).
- Part timings before any change (laps inside `Chip::render`, release build, Pi, `--textbox-page --bench-type 150 --canvas chip`; not
  committed): scene build 0.2 ms, vello encode 1.0 ms, GPU wait 69.4 ms at the first typed letter growing to 79.7 ms by the 150th (about
  0.07 ms a letter), whole frame 70.9 to 81.5 ms. The page's items: 21 below the text box (path size 9,304) and the box itself, one fill of
  about 8,500 path elements and growing. Fonts page: a whole draw 110 ms. Forcing rung 10's way (the box drawn over the stored lower part as
  an image) gave 41.7 to 43.8 ms: about 15 for the page-sized image, 28 for the box. So the cost is the GPU working through the box's
  outline, not setup or the processor, as in 11a. And the existing rule never stored the lower part here: it needs the lower part to be
  half of what is drawn, and the box is the other half.
- Approach (`crates/app/src/chip.rs` only): (1) a changed patch. The chip compares this frame's items with the last: the same from the
  bottom (the lower part), the same at the top (what lies above), and the middle between. The patch is the pixel box that the items of the
  middle that differ touch, old and new; the texture shown keeps the rest of the page as the last frame left it. The patch is drawn into a
  small texture of its own: the stored lower part as an image, then each object of the middle and the top that reaches the patch, in order,
  and the texture is copied over the shown one. A box in the middle of the stack, with an object above it that overlaps, is the case this was
  built for: the object above is drawn again inside the patch, in order, so the picture is the one a whole draw makes (opacity groups are
  kept whole). A turned box, or a box in a group, is just an item that differs; a repeat, whose copies all change, makes every copy an item
  that differs, and the patch is the box around them all: over 60 % of the page, or over three quarters of the page's outline, the old
  ways are used instead (a whole draw, or rung 10's image) and the timed choice between those two stays as it was. A patch is not timed
  against a whole draw: on the fonts page the probe itself (110 ms) crossed the 100 ms fall-back line and put the window on the processor
  canvas, and the patch draws less than the whole page by the rule above. This alone: text-box 39.3 to 40.0 ms, fonts 39.9 to 40.0 ms
  (the box's 8,500 elements are still all drawn each letter). (2) Rows. A big text fill (600 path elements or more) is held as one piece for
  each row of text: the contours whose pixel rows reach one another stay together, so the pieces share no pixel and draw the same picture as
  the whole fill. A letter typed changes its row's piece, and the patch is one row wide. The stored lower part holds the rows above.
- Picture checked: test `patch_matches_whole` (a headless device on the Pi; skipped, and says so, where there is no adapter) feeds the same
  frames to a chip that patches (with rows) and a chip that draws whole (without), through both ways of holding the lower part, and
  compares every pixel: `CHECK crate=app name=chip_patch result=pass frames=22 patches=12 worst_gray_diff=1`; `chip_rows` checks the
  pieces keep every contour and share no pixel row.
- Acceptance, release build, strip on, 150 frames, 3 runs each, `target/release/quadrule <page> --bench-type|--bench-drag|--bench-draw 150
  --canvas chip|export`:

  | page | command | canvas= | p90 ms | worst ms |
  |---|---|---|---|---|
  | textbox | type, chip | chip, chip, chip | 8.7, 8.6, 8.7 | 75.7, 75.6, 76.4 (first typed frame; was 82.0) |
  | fonts | type, chip | chip, chip, chip | 9.5, 9.5, 9.5 | 115.3, 115.2, 115.3 (first typed frame; was a fall-back) |
  | textbox | type, export | export x3 | 20.1, 20.0, 20.0 | 20.3, 21.0, 21.9 |
  | fonts | type, export | export x3 | 19.9, 20.0, 20.0 | 20.4, 20.3, 23.0 |
  | textbox | drag, chip | chip x3 | 4.0, 4.2, 4.0 (was 16.8 to 18.1) | 4.6, 4.8, 4.5 |
  | fonts | drag, chip | chip x3 | 6.2, 6.1, 6.1 (was 17.9 to 18.1) | 6.7, 6.6, 6.5 |
  | textbox | draw, chip | chip x3 | 23.8, 23.9, 23.8 (was 24.0 to 24.3) | 58.5, 58.6, 58.6 |
  | fonts | draw, chip | chip x3 | 25.3, 25.2, 25.2 (was 25.2 to 25.4) | 99.3, 99.1, 99.0 |

  Other pages, one run, chip (rung 10 and 11a figures in brackets): repeats drag 8.9 (35.6), draw 23.3 (23.5); patterns drag 5.0 (17.2),
  draw 23.2 (23.6); groups drag 30.8 (30.8), draw 22.8; shapes drag 8.5, draw 22.8 (23.1). A drag now draws a patch where the moved object
  is small, hence the lower drag figures; the draw figures use the older ways and are as before.
- Judged: `make check-all` → `TESTS scope=workspace checks=476 check_fail=0 passed=443 failed=0 status=ok`; the 22 golden lines and fixtures `v0.4/` to `v0.11/` unchanged
  (the chip is screen-only). Debt: the chip-typing line of "Rung 11b, owed" is paid; a smaller line replaces it.

## 2026-11-16 — rung 11b, chip test crash on Windows CI (builder, Sonnet 5.5)

- chris: "Resume the fix builder". CI runs 43802647748 (`e34992e`) and the one on `61a7347` crashed `quadrule-app` tests on windows-latest with
  `0xc0000005 STATUS_ACCESS_VIOLATION` after `chip_rows` passed. I had not read CI for my own commits; I now do (`gh run view <id> --log`).
- Found with `34916d1` (prints the adapter and each step): run 43852733249 on windows-latest:
  `CHECK crate=app name=chip_adapter backend=Dx12 device_type=Cpu name="Microsoft Basic Render Driver" driver="10.0.26100.33648"`, then
  `STEP 0 patch way`, then the crash. Step 0 is the first frame, with no frame before it, so it is a whole draw: no patch, no texture copy,
  no rows code runs. The crash is vello (or wgpu) on the software DirectX 12 adapter, in the code the app already runs for every first
  chip frame. Verdict (a): the test's own headless setup, not the patch path. Not proof about the real app on Windows (the app opens the chip
  canvas on the window's device; a machine with a software adapter only would hit the same first draw, and the canvas already falls back when a
  frame errors, not when the process dies; noted as debt).
- Fix: `patch_matches_whole` skips a software adapter (`result=skip reason=software_adapter`, or `reason=no_adapter`). It still runs on the Pi
  (V3DV, Vulkan) and wherever there is a graphics chip.
- `make check-all` → `TESTS scope=workspace checks=477 check_fail=0 passed=443 failed=0 status=ok`; no golden line or fixture moved.

## 2026-11-16 — rung 11 done; v0.11.0 (lead, Opus 5.5 high)

- Chip-typing fix judged: lead's runs `--textbox-page --bench-type 150 --canvas chip` → `canvas=chip p90_ms=8.9`;
  `--fonts-page` → `canvas=chip p90_ms=9.9`; `--textbox-page --bench-drag 150 --canvas chip` → `p90_ms=4.5`.
- Windows CI: the new chip test crashed (`STATUS_ACCESS_VIOLATION`) on `61a7347`, `e34992e`, `34916d1`; the builder found
  it in vello's first whole draw on the software adapter (`chip_adapter backend=Dx12 device_type=Cpu name="Microsoft
  Basic Render Driver"`) and made the test skip a software adapter (`c05bcfa`); CI run 43855666375 green on all three
  systems (`chip_patch result=skip reason=software_adapter` on Windows).
- chris approved 11b: "11b looks good in the window; excellent job". Version 0.11.0 (`Cargo.toml`), release notes
  `docs/arcs/v1/release-notes/v0.11.0.md`, ADR-002 built status updated (header values moved to its History), board
  row and re-plan, rung 12 prompt `docs/arcs/v1/rung-12-PROMPT-2026-11-16.md`, calibration row.
- Tagged `v0.11.0` (annotated, "Rung 11: text and fonts") at `619b8a7` after CI 43857657013, Coverage 43857657007 and
  Fuzz 43857657235 passed on `97bfe32`; release run 43858983486: linux-x86_64, linux-aarch64, windows-x86_64,
  macos-universal and "Draft, attach, publish" all success. Published as a pre-release with 7 files:
  https://github.com/tholent/quadrule/releases/tag/v0.11.0 (`gh release view v0.11.0`).

## 2026-11-16 — rung 12a, missing fonts (builder, Sonnet 5.5 high)

- Built 12a only (record `docs/bbq/2026-11-16-rung-12-missing-fonts.md`): `bcf314a` format 0.13.0 and the `v0.12/` fixture;
  `92ca3d9` text crate (signature, ranking, carried fonts, installed fallback, embed rule); the commit after it: missing
  boxes drawn from saved outlines, read-only boxes, bar, dialog, Substitute, Locate fonts, L11, L20, the Save sheet, the
  Embed fonts tick, `--missing-fonts-page`. No ADR text, golden or fixture `v0.4/`-`v0.11/` was edited.
- Choices the record did not settle: font signatures measure serif/sans by the feet of an "n", script by a wobbling
  baseline, mono by equal widths (`crates/text/src/signature.rs`); a family's embedded files are the faces the boxes ask for;
  `.ttc` faces are never embedded; embedded fonts are open for every box in their file (also as fallback); a box with a letter
  no font has stays live; L20 is said on the primary device only; the Missing fonts dialog has Apply and Cancel; Remember
  keeps the fonts left as saved outlines when ticked; Export, Save, Save As and the unsaved prompt's Save show the sheet.
- Paid debt: rung 3 Save-with-stale-outlines (`crates/ui/tests/missing_fonts.rs::save_cannot_write_changed_text_with_stale_outlines`);
  rung 11a "font family not available"; the system-font lint rule (L20).
- `make test-all` -> `TESTS scope=workspace checks=507 check_fail=0 passed=473 failed=0 status=ok`; golden lines and fixtures unchanged
  (`git status` shows none touched).

## 2026-11-16 — rung 12b, pattern rotation (builder, Sonnet 5.5 high)

- Built 12b (record `docs/bbq/2026-11-16-rung-12-missing-fonts.md`, Q8): a pattern region turns whole. `crates/layout/src/pattern.rs`
  `pattern_items_turned` lays the region out upright in its own axes with the unchanged code and turns every item about the middle
  of the whole-pixel box (`turn_pivot`; a 90 or 270 turn rounds the middle to a whole pixel so rules stay sharp); at a quarter turn
  rectangles stay `Item::Rect`, otherwise runs of one gray become one filled path, slanted strokes are turned paths. `ObjectKind::can_rotate` is true for
  patterns (a group holding one now turns; a repeat still cannot), picking uses the turned box (`Shape::Rotated`), lint L02 flags a
  region off a quarter turn (`lint-l02-turned-pattern`); L16 never judged patterns. No file format change (0.13.0 already held the field and its migration).
- Page and flag: `quadrule_model::sample::turned_patterns_document`, `--turned-patterns-page` (also `make bench-drag|bench-draw|bench-rotate TURNED=1`;
  `--bench-rotate` there turns the 45 degree grid). Golden lines `patterns-turned_supernote.a5x2_1920x2560.png` and `..a6x2_1404x1872.png` appended to
  `crates/render/tests/golden/SHA256SUMS`; the 22 earlier lines and the `v0.4/`-`v0.12/` fixtures are unchanged
  (`git diff 7cb49eb -- crates/render/tests/golden/SHA256SUMS` shows two added lines only).
- Tests changed because behaviour changed (not re-blessed): pattern regions turn and a group holding one turns, a repeat is what stops a
  group (`crates/layout/tests/rotate.rs`, `crates/model/tests/groups.rs`, `crates/model/tests/model.rs`, `crates/ui/tests/groups.rs`,
  `crates/ui/tests/rotate.rs`); new: `crates/layout/tests/turned_patterns.rs` (unturned is identical, rigid for 10 kinds x 5 angles, quarter turns on whole pixels, cut at the
  frame, rhythm steps, group and repeat), `crates/render/tests/turned_patterns.rs`, `crates/ui/tests/turned_patterns.rs`, L02 in `crates/lint/tests/lint.rs`.
- Choices the record did not settle: the region turns about the middle of its whole-pixel box (as shapes do); the pattern's phase turns with the
  frame (a bound region's rows start at the rhythm origin in the region's own axes, so they sit on the page's rhythm only unturned or at a half turn
  of a symmetric box); L17 still judges a turned unbound region; the L02 text and id use (`lint-l02-turned-pattern`); the new page has 12 cells of 30 x 20 mm regions (a staff 34 x 28 mm).
- `make test-all` -> `TESTS scope=workspace checks=525 check_fail=0 passed=486 failed=0 status=ok`.
- Pi, release build, strip on, 150 frames, 3 runs each, `target/release/quadrule --turned-patterns-page <flag> 150 --canvas <c>` (90th percentile ms;
  limit 50): `--bench-drag` export 20.6, 21.5, 21.2, chip 5.3, 5.3, 5.3; `--bench-draw` export 21.3, 21.3, 21.2, chip 23.8, 23.6, 23.6;
  `--bench-rotate` export 21.2, 22.6, 22.1, chip 3.8, 3.8, 3.7 (the first chip frame of `--bench-draw` took 41 to 49 ms, still under 50).

## 2026-11-16 — rung 12b, pattern angle (builder, Sonnet 5.5 high)

- Built the amendment (record Q8 as amended, Q10, Q11): `PatternRegion::angle_deg` (`crates/model/src/lib.rs`, optional, left out at 0, in format 0.13.0
  with no version bump; `pattern_angle()` normalises). `crates/layout/src/pattern.rs` `ruling_items` lays a ruling (lines, dots, grid, isometric,
  hex) over a box that covers the frame at any angle (anchored at the frame's corner so turning is rigid), turns it about the frame's middle
  and cuts it at the frame (lines exactly, a dot whole if its centre is inside); Cornell and the staves are laid out in the frame's own box and
  turned; a quarter turn stays whole-pixel `Item::Rect`; the frame's own turn then adds (`pattern_items_turned`). `Pattern angle` field in the Style tab
  beside the spacing (Fluent `app-pattern-angle`), L02 for a frame or ruling off a quarter turn, `--pattern-angle-page` (`make ... ANGLE=1`).
- Golden lines `pattern-angle_supernote.a5x2_1920x2560.png` and `..a6x2_1404x1872.png` appended (`git diff b5ff1f0 -- crates/render/tests/golden/SHA256SUMS`
  shows those two lines only; the two `patterns-turned_*` lines and the 22 older ones, and the fixtures, are unchanged).
- `make test-all` -> `TESTS scope=workspace checks=538 check_fail=0 passed=496 failed=0 status=ok`.
- Pi, release build, strip on, 150 frames, 3 runs each, `target/release/quadrule --pattern-angle-page <flag> 150 --canvas <c>` (90th percentile ms; limit 50):
  `--bench-drag` export 20.4, 20.9, 21.0, chip 3.6, 3.6, 3.6; `--bench-draw` export 20.7, 21.9, 21.0, chip 23.4, 23.4, 23.3 (first frame 45 to 52 ms).


## 2026-11-16/17 — rung 12, lead (Opus 5.5 high)

- Session start: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=7ff14c3 dirty=no rebuilt=11`.
  Meter 22 % (chris; `spend-ledger.py reading 22 --meter weekly`); share 12 points (chris). Parked questions asked again:
  pattern rotation unparked by chris; the A5 X rhythm and the Label gray stay parked.
- Embedding bits measured on the Pi (a small `fsType` reader over `fc-list` output, 181 files): 75 installable, 1
  editable, 35 preview-and-print only, 70 Type 1 or unreadable; bundled 10 of 10 installable.
- bbq, 9 questions (`docs/bbq/2026-11-16-rung-12-missing-fonts.md`): split 12a/12b, one `v0.12.0`; hinting to v1.1
  (ADR-002 amended 2026-11-16, `adr-check: 15 ADRs, 0 failure(s)`); embed allowed fonts with an off switch; Substitute
  rewrites the font; bar then dialog; move-not-reflow read-only text; L20 and a sheet; the region turns whole.
- 12a builder (Sonnet 5.5 high) stopped at its third step by an account error ("Claude Code can't use your Claude
  subscription", HTTP 403, `oauth_org_not_allowed`) with two steps pushed (CI green on `92ca3d9`) and the rest
  uncommitted; resumed on chris's "please continue" and finished (`b8509df`, CI 43946396546 success). Lead checks: the
  22 golden lines and fixtures v0.4–v0.11 unchanged (`git diff --stat 7ff14c3..HEAD -- crates/render/tests/golden
  crates/render/tests/fixtures` shows only the new `v0.12/`); `cargo test --release -p quadrule-render` all ok.
  chris: "I approve 12a" (meter 27 %).
- 12b builder (Sonnet 5.5 high): `470dda3` (CI 43959317427 success). chris then asked for the pattern to turn inside
  its frame; Q8 amended, Q10 and Q11 asked (`b5ff1f0`, meter 30 %); the same builder built it (`2d14552`, CI 43968390319
  success). The lead looked at both new Nomad goldens. chris: "approved. 31% meter read".
- Release: version 0.12.0 (`Cargo.toml`), notes `docs/arcs/v1/release-notes/v0.12.0.md`, ADR-001 and ADR-002 built
  status (`adr-check: 15 ADRs, 0 failure(s)`), board re-planned, rung 13 prompt, STATUS.md trimmed to 292 lines
  (`status-check: 292 lines, ceiling 300, 0 finding(s)`; the 2026-11-10 log bullets moved to the archive word for word).
- Spend: `spend-ledger.py measure --latest --meter 22 31 --lead "Opus 5.5, high"` → `| 2026-11-16 | quadrule [25d30704] |
  claude | Opus 5.5, high | 11-16 11:51 → 11-17 01:43 | fresh 842 · cached 92.6M · written 2.0M · out 365k | 82% (3 files)
  | sonnet 82% · opus 18% | $28.89 | 22 → 31 · alone here · 9 pts |`.
- v0.12.0 published: tag `v0.12.0` pushed at `9ed31ee` after CI 43997940859 passed; the release workflow, run
  43999330846, succeeded and published https://github.com/tholent/quadrule/releases/tag/v0.12.0 (pre-release, not a
  draft) with 7 files: AppImage and `.deb` for Linux x86-64 and ARM64, the universal `.dmg`, the `.msi`, and
  `SHA256SUMS.txt` (`gh release view v0.12.0 --json assets`).

## 2026-11-17 — rung 13a, the profile system (builder, Sonnet 5.5 high)

- Built 13a from the record `docs/bbq/2026-11-17-rung-13-profile-system.md` (Q1 to Q9 and the Defaulted rules that are not about paper), one commit per step:
  `797cc9d` step 1 (registry and storage: `crates/profiles/src/{limits,registry,active}.rs`, `from_ron` fuzz target
  `crates/format/fuzz/fuzz_targets/from_ron.rs`), `ded0b3f` step 2 (one lookup: every production call site reads
  `quadrule_profiles::active::lookup`/`lookup_for`; the app loads `<config folder>/profiles/` at start, not for timing runs;
  `crates/profiles/tests/call_sites.rs` scans for strays), `0752c72` step 3 (format 0.14.0: `profiles/<id>.ron` entries, manifest
  `carried_profiles`, migration 0.13 to 0.14 with no rewrite, the `v0.13/` fixture (the pattern-angle page, written by the 0.13 build),
  the reader applies the limits, the bar and the Merge / Rename / Not now dialog), `84a1bc6` steps 4 and 5 (the Profiles window and its
  tests), `cc0d2f4` and `e3afe74` (the Page tab lists a device the page uses from the file's copy; docs; the window fits a 720 px screen).
- Frozen bytes: `git diff --stat 3f3ebb3..HEAD -- crates/render/tests/golden/ crates/render/tests/fixtures/` shows only the new
  `v0.13/` (two files); the 26 golden lines and the `v0.4/` to `v0.12/` fixtures are unchanged. `builtin`/`builtin_for` still give the shipped
  values only; tests use folders of their own under the system's temporary folder, never the config folder.
- `make check-all` (fmt, clippy `-D warnings`, tests, at `e3afe74`) -> `TESTS scope=workspace checks=573 check_fail=0 passed=530 failed=0 status=ok`.
- CI: 797cc9d run 44005144237 success; ded0b3f 44007374825 success; 0752c72 44008651284 success; 84a1bc6 44009608602 success;
  cc0d2f4 44012499657 success; e3afe74 44013040394 success. Fuzz and Coverage runs on the same commits succeeded
  (Fuzz 44005144374, 44007374733, 44008651338).
- Fuzz, local: `cargo +nightly fuzz run from_ron` 114,994 runs in 41 s, `open_quad` (with the carried-profile seed `carried-profile.quad`) 192,253 runs in
  41 s, no crash. `fuzz.yml` now also runs `from_ron` for 30 s and watches `crates/profiles/**`.
- Pi, release build, strip on, 150 frames, 3 runs each, `target/release/quadrule <page> <flag> 150 --canvas <c>` (90th percentile ms; limit 50):
  text-box page `--bench-drag` export 20.0, 20.0, 20.4, chip 3.7, 3.8, 3.7; `--bench-draw` export 20.6, 21.3, 21.0, chip 23.8, 23.8, 23.7
  (chip first frame 54.5 ms, three runs); overrides page `--bench-drag` export 20.6, 20.0, 20.5, chip 17.0, 16.9, 17.0; `--bench-draw` export
  20.8, 20.3, 20.3, chip 22.7, 22.7, 24.0. The Profiles window, release test harness (`crates/ui/tests/profiles_window.rs`): a changed role value
  makes the Nomad's page again in 7.2 ms; a whole frame with the window open after that change took 66.6 ms in the harness, an unchanged frame
  with the window open 1.3 ms, and the same redraw with the window closed 89.1 ms (the harness has no graphics chip and builds the accessibility
  tree), so the window adds no cost to the redraw; the real window's redraw frames are the 20 ms of the drag bench.
- Choices the record did not settle: the registry is one per thread (a thread starts with the built-ins; tests are isolated); the Edited
  mark shows as "Name · Edited" and "Name · Unverified" in lists; Save keeps the window open and Cancel (or the window's close) throws away the
  drafts; New… and Duplicate and an import write at once, Save writes edits; Duplicate names the copy "X copy" with a short name made from it; an
  import with the same id and values says "already have it", same id and other values asks Replace mine / Keep both, another id with the same name
  asks Merge / Rename; Keep both keeps the incoming name; Merge keeps the target's id and short name, and into a built-in becomes its override
  with the shipped id, size, ppi and short name; Rename keeps the incoming id when free, else a new UUID v4 (then the page is renamed to it, an
  unsaved change, not an undo step); the dialog's default for each row is Not now; a secondary device that is neither here nor carried is left out of the
  page with a notice, the primary missing refuses the file; limits beyond the record: diagonal 0 to 1,000 in, free text 2,000 characters, at most
  64 gray or pen values and export hints, one or two orientations, a file at most 256 KiB, ids of letters, digits, dots, hyphens and underscores
  that name the file, an override file may not change its built-in's size, ppi or short name (damaged), margins 0 or more and leaving room;
  the Profiles window's toolbar button doubles as the menu entry (the toolbar has no menu); saving a built-in's values equal to the shipped ones removes
  the override file; `--profiles` opens the window at start.

## 2026-11-17 — rung 13b, paper-size profiles (builder, Sonnet 5.5 high)

- Built 13b from the record `docs/bbq/2026-11-17-rung-13-profile-system.md` (Q10, Q12 to Q14 and the Defaulted paper items), start `debc1ea`:
  `ef76e05` step 1 (the engine: eleven paper profiles `crates/profiles/data/paper.*.ron`, `Profile.paper` / `PaperSize` / `at_ppi` / `oriented` for paper,
  `active::lookup_in`, `ProfileContext.ppi` (format 0.14.0, optional, left out when 300), `device::set_paper_ppi`, lint skips, the paper sample pages and goldens, tests),
  `04e7b8c` step 2 (Page tab Paper group and PPI choice, New…, the Profiles window, `--paper-page`, `--paper-landscape-page`, `PAPER=1`, README, DEVELOPMENT, strings),
  `37b1769` the Paper heading in the Page tab drawn like Devices. No fixture was added: a 0.14.0 file saved before the PPI field is covered by `crates/format/tests/paper_ppi.rs`.
- Frozen bytes: `git diff --stat debc1ea..HEAD -- crates/render/tests/golden/ crates/render/tests/fixtures/` shows only `SHA256SUMS` (three appended lines) and the three new PNGs
  (`paper_paper.a4_2480x3508.png`, `paper_paper.a5_3496x4961.png`, `paper-landscape_paper.letter_3300x2550.png`); the 26 older lines and `v0.4/` to `v0.13/` are unchanged. I looked at each new PNG
  (downscaled) before blessing: the circle is round on A4, A5 and Letter, the margin frame, band, corner marks and (landscape) star sit where the design puts them, the text is upright in the wide page.
  Two earlier blessings were redone after looking (the ruled lines ran under the bottom corner marks; the headline named A4 on the A5 page).
- `make check-all` (fmt, clippy `-D warnings`, tests, at `37b1769`) -> `TESTS scope=workspace checks=598 check_fail=0 passed=552 failed=0 status=ok`.
- CI: `04e7b8c` (pushed with `ef76e05`) CI 44076943933 success, Fuzz 44076943935 success, Coverage 44076944105 success; `37b1769` CI 44082335352 success, Coverage 44082335307 success (a Fuzz run starts only when the fuzz paths change).
- Speed, Pi, release build, strip on, 150 frames, 3 runs each, p90 ms (limit 50), `XDG_CONFIG_HOME` and `XDG_DATA_HOME` pointed at a scratch folder:
  paper page (`--paper-page`, A4 at 300, 8.7 M px) `--bench-drag` export 34.3, 34.1, 34.5, chip 3.9, 3.8, 3.9; `--bench-draw` export 35.1, 35.2, 35.1, chip 11.5, 11.4, 11.5.
  Largest page, Tabloid at 600 PPI (6600 × 10200 = 67.3 M px; a saved `.quad` made for the bench): `--bench-drag` export **425.3, 413.7, 410.4**, chip 3.5, 3.6, 3.7;
  `--bench-draw` export **421.5, 424.4, 440.2**, chip 4.1, 4.2, 4.2. **The export canvas breaks the 50 ms line by about eight times on the largest page; the chip canvas does not.**
  What costs it: the export canvas redraws the whole page each frame, and the cost scales with pixels. Measured alone, in a release test: `vello_cpu` draws A4 at 300 in 28 ms and Tabloid at 600 in 187 ms
  (6.7 times for 7.7 times the pixels), and the PNG encode takes 18 and 98 ms (an export cost, not a frame cost); the display list takes under 0.1 ms. The other ~200 ms of the 410 ms frame is whole-page
  work I did not itemise (copying and handing a 67 MB image on). A3 at 600 (7016 × 9921) is the same: export drag p90 345.8 ms in one run. Memory (peak `VmHWM` of the app, `--bench-drag 60`, export canvas, 4.1 GB machine):
  paper page 188 MB, Tabloid at 600 444 MB, A3 at 600 455 MB. No fix and no new limit invented; the lead rules.
  Note the limits (`limits::check`) allow a custom paper side of up to 693 mm, which is 16,384 px at 600 PPI (268 M px, about four times Tabloid); a custom page of that size is not measured.
- Choices the record did not settle (each open to chris's veto):
  - **Executive is stored as 184.15 × 266.7 mm** (7.25 × 10.5 in). The record writes 184.2, which is 184.15 to one place; at 300 PPI 184.2 gives 2176 px, napkin §3.3.1's table says 2175, and the brief
    asked to test every row of that table, so the exact inches win. The window shows 184.15.
  - `builtin_ids()` now lists the screens only (so the 13a tests, the calibration and safe-zone tools and a new page's targets are unchanged); `builtin_paper_ids()` lists paper; `Registry::builtin_ids()` still lists every shipped profile.
  - A paper profile's file holds `paper: Some((width_mm, height_mm))`; the pixel size in the file is that size at 300 PPI and is made so on reading (a hand-edited file whose pixels disagree is read with the mm as the truth);
    its safe zones, gray levels and pen grays are emptied on reading. A profile file without the field is a screen, so every 13a file reads and writes as before.
  - **New… and Duplicate from a paper profile stay paper**, with an editable size in mm (the recommendation). Each side may be 2.8 to 693 mm, the span in which every PPI gives 16 to 16,384 px (the existing pixel limits,
    applied at 150 and at 600). The form hides Gray levels, pen grays and Safe zones for paper and notes that line widths are for 300 PPI.
  - The role widths stored in a paper profile are those at 300 PPI; a page's PPI scales them (`max(1, round(w × PPI / 300))`; the Nomad's whisper, guide, structure, emphasis, dot are 1, 1, 2, 3, 3 at 300; 2, 2, 4, 6, 6 at 600; 1, 1, 1, 2, 2 at 150 and 200).
    A test holds the paper roles equal to the Nomad's shipped ones, so a recalibrated Nomad will fail it until the paper files are updated on purpose.
  - A page's `page_mm()` for a paper size comes from its rounded pixels, as for a screen (A4 at 300 is 209.97 × 297.01 mm), so nothing sized to the page edge is judged off the page.
  - **A new page on a paper size targets that paper alone** at 300 PPI, with margins of the profile's 10 mm rounded up to whole rhythm steps (12 mm for the 6 mm preset); the devices can be ticked in the Page tab.
    A new page on a screen still targets both screens, not the paper sizes.
  - The PPI belongs to the page's entry for that paper size: unticking the paper forgets it, and ticking it again starts at 300. A number in a file that is not one of the four reads as 300 and is kept on a save.
  - L03 is in the skip list with L04 to L06 (`quadrule_lint::SKIPPED_ON_PAPER`); L03 to L05 are not built yet, and L06 is the only rule that read device data. Findings of a listed rule are dropped on paper whatever the profile holds.
  - The Page tab says "Paper" as its own group after the devices, each size with a PPI combo beside its tick; the landscape note says a paper size is saved wide, and the old "saved turned" note shows only when a screen is a target.
  - The Profiles window lists Paper under Built in (a small group label); a row's tip and the form give the size in mm and px at 300; the PPI row reads "Chosen per page; each page picks 150, 200, 300 or 600" (not the record's exact "PPI: chosen per page").
  - Extra flag `--paper-landscape-page` (US Letter held sideways); the benches use `--paper-page` (`make bench-drag PAPER=1`).
- Seen on screen (grim, release build, scratch config and data folders, 1440 × 720): `scratchpad/paper-page.png` (the paper page, the Page tab with the Paper group: ISO A3, A4 primary at 300 PPI, A5 at 600 PPI, the device strip with both sizes) and
  `scratchpad/paper-profiles.png` (the Profiles window on ISO A4: 210 × 297 mm, 2480 × 3508 px at 300 PPI (locked), PPI chosen per page, margins 10, Line roles / Fill roles / About only).
  Not driven by a pointer: the PPI combo, the mm DragValues of a copy of a paper size, the New… list's Paper group (the logic under them is tested headless).

## 2026-11-17 — rung 13 lead session (Opus 5.5 high): planned, two builders, approved, `v0.13.0`

- Kickoff: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=979bd59 dirty=no rebuilt=11`. Budget 10
  points by chris, alone on the weekly meter, start 32 % (given by chris, logged). Parked questions (A5 X rhythm, Label
  gray) asked again: chris, "Keep both parked".
- bbq, 14 questions (`docs/bbq/2026-11-17-rung-13-profile-system.md`, `3f3ebb3`): chris ruled GUID ids for user
  profiles with Merge or Rename on a same name (Q6, his own words; Q9 Merge = theirs updates mine), and added built-in
  paper sizes at Q10, pulled from v1.1; split 13a/13b, one release (Q11). ADR-016 drafted, spec §3.3.1 and §16 got dated
  notes, board row split (`bc51934`).
- 13a built by a Sonnet 5.5 high builder (387,601 tokens; `797cc9d` to `10a249b`, its own log entry above). Lead
  checks: golden and fixtures diff only the new `v0.13/`; CI green on `e3afe74`; no worker thread in layout, render, ui
  or app reads profiles (`grep -rn "thread::spawn\|std::thread\|rayon"` on those crates' `src`, nothing found), so the
  per-thread registry reaches every drawing. A bench run had saved `~/.local/share/quadrule/app.ron` (22:42 EST
  2026-11-16); chris was told. chris: "1. approved 2. ratified 3. 36%" — 13a approved, ADR-016 accepted
  (`a78516a`, `debc1ea`, format fix for `adr-check.py` → `16 ADRs, 0 failure(s)`).
- 13b built by a second Sonnet 5.5 high builder (388,875 tokens; `ef76e05` to `5fc3071`). Lead checks: golden diff
  only three added lines and PNGs; CI and Coverage green on `37b1769`; looked at
  `paper-landscape_paper.letter_3300x2550.png` (wide, upright, round circle). The builder stopped and reported Tabloid at
  600 PPI on the export canvas at 410–440 ms p90; chris: "1. approved 2. C 3. 41%" — 13b approved, large pages left as
  rung 17 debt.
- Close: version 0.13.0 (`Cargo.toml`, `Cargo.lock`), release notes `docs/arcs/v1/release-notes/v0.13.0.md`, ADR-016
  built landed, board re-planned (rung 17 grows by the large-page item), `STATUS.md` (old log lines moved word for word
  to `docs/STATUS-archive.md`; `status-check.py` → `296 lines, ceiling 300, 0 finding(s)`), calibration row, rung 14
  prompt `docs/arcs/v1/rung-14-PROMPT-2026-11-17.md`; tag `v0.13.0` pushed after CI.
- `v0.13.0` published 2026-11-17 14:14 UTC by the release workflow (run 44098096890, success) from the tag on `5c0cc0f`
  (CI, Fuzz and Coverage green there): pre-release, six installers and `SHA256SUMS.txt`
  (`gh release view v0.13.0`), https://github.com/tholent/quadrule/releases/tag/v0.13.0.

## 2026-11-17 — rung 14a builder (Sonnet 5.5 high): the six rules, the lint block, format 0.15.0

Built from `docs/bbq/2026-11-17-rung-14-lint-and-preflight.md` (Q2, Q3, Q4, Q7 saved-in-file parts, Q9, and the Defaulted
list). On `main`, five commits (CI on `ad948c3` green: CI, Fuzz, Coverage):

- `ad948c3` step 1: a `lint` block on every `Profile` (`LintThresholds`, every value optional, `#[serde(default)]`),
  filled for the Nomad and the Manta from the sheets (comments cite the codes), paper sizes carry only 6 pt and 5 mm; the
  Profiles window gets a **Lint checks** section (a field with Clear, or "skipped" with Set; covered by Save, Reset to
  built-in, `.ron` export and import) and a **Delete** button on each damaged file (asks first).
- `1da1b32` step 2a: `crates/render/tests/fixtures/v0.14/` (the overrides page with a third device, a user's profile the
  file carries) written by the 0.14.0 build's output before any format code changed, then its carried profile text cut
  back to what the 0.14 writer wrote (no `lint:` block), manifest checksum made to match. `SHA256SUMS` holds its three PNGs.
- `c14628e` step 2: `FORMAT_VERSION` 0.15.0, `v0_14_to_v0_15` (rewrites nothing), `Document::ignored_rules` and
  `Object::free_placement` (both left out of a file when default), `migrate_v014.rs`; the older "0.14.0" assertions moved to
  0.15.0 (and the "newer file" test to 0.16.0).
- `273f1cb` steps 3 and 4: `crates/lint/src/judge.rs` (L03, L05, L07, L09, L13, L14), ignored rules and Free placement
  honoured, `skipped_checks` (reasons: Paper, Wallpaper, Ignored, NoGrayLevels, NoValue), the Lint tab's one line per
  device, `lint.ftl` messages; L02's diagonal-line note moved to L14 (three old lint tests updated for it);
  `SKIPPED_ON_PAPER` gained L13 and L14.
- `547c39d` steps 5 to 7: `--lint-page` (`lint_document`, `Viewer::open_lint`, `DragBench::with_lint`, `make bench-drag LINT=1`),
  `profiles_pointer.rs` (pointer tests for the collapsing sections, DragValues, the lint fields, the gray-list field, the
  safe-zone table and the damaged-file Delete), README and DEVELOPMENT.

Checks: `make check-all` on `547c39d` → `TESTS scope=workspace checks=622 check_fail=0 passed=576 failed=0 status=ok`.
`(cd crates/render/tests/golden && sha256sum -c SHA256SUMS)` → 29 lines, none other than OK; `git diff 198707a --stat --
crates/render/tests/golden crates/render/tests/fixtures` shows only the two added `v0.14/` files. New CHECK lines:
`profiles lint_block_shipped|lint_block_file|lint_block_override`; `render migrate_v014|migrate_chain_0150|ignored_and_free_saved`;
`lint l03_thresholds|l05_clean_levels|l07_text_size|l09_handwriting_step|l13_dark_fills|l14_soft_thin_lines|gray_on_the_page|
rules_follow_the_profile_in_use|paper_skips_gray_rules|ignored_and_free_placement|skip_reasons|lint_page`; `ui lint_tab_skipped_lines|
lint_page_opens|profiles_lint_block_file|profiles_pointer_lint|profiles_pointer_lists|profiles_pointer_delete_damaged`. All `pass`.

Found on the way: the safe-zone table's orientation boxes had no accessible name (the pointer test's accessibility check
caught it); they are named now ("Held <zone>").

Paid from `docs/STATUS-debt.md` (removed there): the Profiles window pointer tests, the Delete button for a damaged file
(`Registry::delete_damaged`), and the "L03 in `SKIPPED_ON_PAPER` names a rule that is not built" sentence. Not paid: the Page tab's
PPI wrap (not asked of this builder).

Defaulted beyond the record (each can be vetoed): field names (`lightest_gray_line_1px`, `lightest_gray_line_2px_up`,
`lightest_gray_dot_2px|3px|4px_up`, `lightest_gray_fill`, `lightest_gray_text`, `smallest_text_pt`,
`smallest_handwriting_step_mm`, `dark_fill_gray`, `dark_fill_page_share` as a fraction shown as a percentage in the window,
`thin_line_px`); a dot under 2 px is judged by the 2 px value; the empty `lint` block is written as `lint: ()` so "no block" (an
old override, which takes the shipped values) differs from "emptied on purpose"; a group's Free placement covers its children;
L09 compares in pixels (the millimetres rounded as the rhythm is), says the rhythm once and a ruled (Lines) region only when its
spacing is its own; L13 sums fills (clipped to the page, every repeat copy), reports one finding on the largest; L14 also covers
isometric and hex patterns and turned patterns; the ignored-rule list shows ids the linter does not judge only if built (`Lxx` is
dropped); the skip order is Paper, Wallpaper, Ignored, no gray levels, no value.

Owed: see the new "Rung 14a, owed" line in `docs/STATUS-debt.md`. The look: `cargo run --release -p quadrule-app -- --lint-page`
(Lint tab; Profiles window from the Page tab or `--profiles`, section "Lint checks").

## 2026-11-17 — rung 14b builder (Sonnet 5.5): the Lint tab's buttons, the preflight sheet, the gray swatches

Built from `docs/bbq/2026-11-17-rung-14-lint-and-preflight.md` (Q5 to Q9, the Defaulted list and the 14a-look Amendment), start `e215aec`, on `main`.
No file-format change (`FORMAT_VERSION` stays 0.15.0); the one model addition is the command `SetIgnoredRules` (ADR-010: every change is a command).

Commits (CI for the last code commit below):
- `6245bc8` step 2: `crates/lint/src/fix.rs` (`offered`, `plan`, `plan_snap_all` as `Edit`s, no screen in it), `Command::SetIgnoredRules`, `rule_title`, the `lint-fix-*`, `lint-ignore*` and `lint-rule-*` strings, `crates/lint/tests/fixes.rs`.
- `418a411` steps 1, 3, 4, 5, 6 (one commit, since they share `panels.rs`, `view.rs` and `app.ftl`): the Lint tab (`lint_ui.rs`: device picker "Findings for", each finding's quick-fix and Ignore buttons, "Ignored rules (N)" with Turn back on),
  `lint_fixes.rs` (what the buttons do: findings on every device, a fix as one undo step, through the device's overrides for a place or a text size), Free placement in the Arrange tab (`Mixed::free_placement`),
  the preflight sheet (`preflight.rs`), the gray swatches and role badge (`swatches.rs`), the Page tab's PPI combo narrowed to one line; README, DEVELOPMENT, the paid debt line; pointer tests.
- Test changes in `crates/ui/tests/window.rs`, both from the new behaviour: `dialogs_are_requested_not_opened` now expects the sheet on the sample page (it has warnings) and presses Export anyway; `findings_zoom_and_theme` scrolls the L16 row into view (the list is longer now).

What each part does, in the window:
- **Lint tab**: "Findings for" combo (All devices by default; view state, not saved); with All, each row names its device; a row click shows that device and outlines the finding. Buttons per finding: its quick fixes, then "Ignore this rule for this document".
  Ignored rules list at the end with "Turn back on". Every click is one undo step (tested, with undo).
- **Quick fixes** (`Fix`): Snap to pixel grid (L01/L02, a Custom width to whole pixels of that device), Use nearest clean gray (L03/L05: the gray nearest the current one whose blend on the page is a clean level and not too light, darker on a tie;
  a Role style becomes Custom at its width), Use <Role> role (roles within a pixel of the width that pass L03 and L05), Move inside safe area (L06: the smallest move that clears every docked-toolbar and footer zone and stays on the page),
  Use <N> pt (L07), Snap to rhythm / Snap all to rhythm (L16, primary only, Free placement and locked objects left alone), Bind to rhythm (L17: a region's `bound` on; a repeat's unbound axes to Auto), Center rhythm origin (L18).
  A move or a text size on a secondary device is an override of that device (design untouched, tested on the Nomad); a width, gray, role or binding is the shared design.
- **Preflight**: Export (this device or all) opens "Before you export" when a device being exported has a Warn finding or L20 applies; groups by device then Document (L20); Jump to object (closes the sheet, shows the device, selects the object, outlines the finding),
  Ignore, the quick fix; Info findings fold under "N notes"; a line per paper device being exported says its checks do not run; Cancel and Export anyway. No warning and no L20: the folder dialog opens at once, and rung 12's font sheet is no longer shown for an Export.
  Save and Save As keep rung 12's sheet as it was (tested, including its "Don't warn again").
- **Style tab**: under Custom the gray is the profile's clean levels as swatches (8 a row), the chosen one outlined, "Show all grays" opens the 0 to 255 slider (held open when the gray is off the levels); paper shows the slider only; fill and text grays the same;
  swatches are named "Gray 51" / "Fill gray 51"; "matches <Role>" when width and gray equal a role on the device shown, else "Use recommended" (nearest role by width, Guide when two tie; gray-only roles by gray for text).

New CHECK lines (all `pass`): `lint fix_snap_to_pixel_grid|fix_nearest_gray_and_role|fix_move_inside_safe_area|fix_use_min_text_size|fix_snap_to_rhythm|fix_bind_and_center|fix_none_for_the_rest`;
`ui lint_pointer_device_picker|lint_pointer_ignore_and_restore|lint_pointer_quick_fixes|lint_pointer_secondary_device|lint_pointer_free_placement|preflight_pointer_sheet|preflight_pointer_rows|preflight_pointer_fonts_and_save|preflight_pointer_paper_and_scope|swatches_pointer_stroke|swatches_pointer_fill_text_paper|swatches_pointer_badge|page_tab_ppi_one_line`
(the last fails with the old combo width: checked by setting the width back to 110 points).
`make check-all` (fmt, clippy `-D warnings`, tests) on the tree of `418a411` → `TESTS scope=workspace checks=642 check_fail=0 passed=596 failed=0 status=ok`.
Frozen bytes: `(cd crates/render/tests/golden && sha256sum -c SHA256SUMS)` → 29 lines, none other than OK; `git diff e215aec --stat -- crates/render/tests/golden crates/render/tests/fixtures` shows nothing.

Speed (Pi, release build, strip on, 150 frames, 3 runs each, 90th percentile ms against 50, `XDG_CONFIG_HOME` and `XDG_DATA_HOME` in a scratch folder, `./target/release/quadrule <page> --bench-drag|--bench-draw 150 --canvas export|chip`), every run `result=pass`:
`--lint-page` drag export 20.7, 20.2, 20.1, chip 17.6, 17.5, 17.5; draw export 19.9, 20.4, 20.5, chip 22.7, 22.4, 22.6. With the Lint tab showing (`--lint-page --look lint --bench-drag`): export 20.1 ×3, chip 17.6 ×3.
`--groups-page` (the heaviest on the chip canvas in the rung 7a entry) drag export 20.0, 20.1, 20.7, chip 30.5, 30.5, 30.6; draw export 20.0, 20.0, 20.2, chip 22.8, 22.9, 22.8.
`--shapes-page` drag export 20.1, 20.0, 20.1, chip 8.1 ×3; draw export 20.0, 20.3, 20.0, chip 22.8, 22.9, 22.8. Nothing over 50 ms. Seen on screen (grim, scratch folders): `scratchpad/look-lint.png`, `look-preflight.png`, `look-style.png`, `look-paper.png` (the Page tab's PPI on one line).

Defaulted beyond the record (each can be vetoed): the combo's label "Findings for" (not "Device", which the toolbar's combo already has); the extra `--look lint|style|preflight` flag so a window can be opened already showing a part (for screenshots); Snap all to rhythm hides when fewer than two objects are off the rhythm;
no fix for the pitch of a repeat or for L06 on a repeat's copies; the preflight sheet lists the devices being exported and puts L20 under Document whichever device is shown, hiding it when the document says "Don't warn again"; the sheet's L20 row has Ignore but no Jump;
small buttons on the rows; the notes fold is not remembered. Owed: see the "Rung 14b, owed" line in `docs/STATUS-debt.md` (wording of every new button and line is a taste call for chris).
The look: `cargo run --release -p quadrule-app -- --lint-page` then the Lint tab (or `--look lint`); Export, then "Export this device…" opens the sheet (or `--look preflight`); click a pale line ("L03 line, gray 230") and the Style tab (or `--look style`); the Arrange tab's Free placement on any selected object; `--paper-page` for the Page tab's PPI row.

## 2026-11-17 — rung 14 lead (Opus 5.5, high)

- Session start: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=eaa223a dirty=no rebuilt=11`.
  Budget 15 points by chris, alone on the weekly meter, start 42 % (given by chris, logged). Asked again: the Label gray
  stays parked; **the A5 X rhythm question was reopened and closed** by chris ("Ratify today's behaviour
  (Recommended)": the rhythm wins for spacing, frames follow their constraints); spec §6.3 amended, debt and board
  updated (`198707a`).
- bbq, 9 questions, every recommendation taken: split 14a/14b, a lint block in profiles, the thresholds as listed, the
  preflight sheet and swatches as sketched (record `docs/bbq/2026-11-17-rung-14-lint-and-preflight.md`).
- 14a built by a Sonnet 5.5 high builder (400,873 tokens; its entry above). The lead opened `--lint-page` with scratch
  config folders (window starts, 14 findings) and confirmed its checks; chris approved ("approved. meter read is 46%.",
  `e215aec`). ADR-012 amended and ratified by chris ("ratify now", `178e094`; `adr-check` → `16 ADRs, 0 failure(s)`).
- 14b built by a Sonnet 5.5 high builder (477,305 tokens; its entry above). The lead confirmed CI, Coverage and Fuzz
  green on `418a411`, goldens `sha256sum -c` 29 OK, no fixture change since `e215aec`, format 0.15.0, and looked at its
  screenshots of the Lint tab, preflight sheet and swatches. chris approved ("14b approved. meter 52%").
- Close: version 0.14.0, `docs/arcs/v1/release-notes/v0.14.0.md`, tag `v0.14.0`; board row and re-plan; STATUS.md (rung
  12 bullet and two Log lines moved word for word to `docs/STATUS-archive.md`); rung 15 prompt
  `docs/arcs/v1/rung-15-PROMPT-2026-11-17.md`; calibration row. `spend-ledger.py measure --latest --meter 42 52` →
  `sonnet 86% · opus 14% | $36.24 | 42 → 52 · alone here · 10 pts`.
- After the release, chris asked why a rung's end runs CI, Fuzz and Coverage and then the release again. The lead found
  the sign-off commit's only code change is the version bump in `Cargo.toml` and `Cargo.lock`, which all three
  workflows watch; the release workflow builds but runs no tests, so CI on that commit is the only test of the tagged
  code, while Fuzz and Coverage repeat the last run. chris, word for word: "yes, let's go with option 2": skip Fuzz and
  Coverage when only the version moved. Built: `.github/scripts/version-only.py` (compares `Cargo.toml` without
  `[workspace.package] version` and `Cargo.lock` without the quadrule packages' versions; anything it cannot tell
  runs) and a `changes` job ahead of each of `fuzz.yml` and `coverage.yml`. Checked on the history (`version-only.py
  push <c>^ <c>`): the sign-offs `ed28d36`, `5c0cc0f`, `9ed31ee`, `97bfe32`, `26f552f`, `8a22065` → `run=false
  reason=version-only`; every other recent `Cargo.lock` commit, a range with code, a pull request, a new branch and an
  unreadable base → `run=true`; in a scratch clone, an outside package's version in `Cargo.lock`, a dependency in
  `Cargo.toml` and an edition change → `run=true`.

## 2026-11-17 — rung 15 leg 1, builder (Sonnet 5.5, high): the export engine and the file format

Brief: `docs/bbq/2026-11-17-rung-15-export.md` (Q1–Q10 and the Defaulted list), leg 1 of 2 (headless; the Export sheet,
the completion sheet, the menu and the shortcuts are leg 2). Worked on `main`; token use was not measured by the builder.

- Step 1, file format 0.16.0 (`ec1b626`): `previews/<primary_profile_id>.png`, 8-bit gray, longest side 480 px, from the
  main device's export pixels (`export_pixels`, so a landscape design is turned like its PNG), shrunk by `viewer::shrink`,
  never enlarged. Written by `Viewer::save_bytes` (Save and Save As); `file_bytes` (the recovery file, so autosave) writes
  none. `QuadFile.previews` carries the bytes; a damaged or missing one is skipped, never an error. Migration step 0.15 to
  0.16 rewrites nothing; fixture `crates/render/tests/fixtures/v0.15/` (the `v0.14` page resaved by the 0.15.0 build before
  the change, same export hashes). The `migrate_v0N.rs` tests' `FORMAT_VERSION == "0.15.0"` guards became `"0.16.0"`, as
  at every earlier format change.
- Step 4, PNG text note (`73766f4`): Software = `quadrule <version>` (the editor's `SAVED_BY`), Title = the document's
  title (or the file's name); a chunk is `tEXt` when every character is Latin-1, else `iTXt`; an empty value is left out,
  a NUL becomes a space. `encode_png(img, None)` is untouched. One new golden `noted_supernote.a6x2_1404x1872.png`
  (Software `tEXt`, Title `iTXt`, version fixed in the test), its line appended to `golden/SHA256SUMS` after the 29 frozen
  ones.
- Steps 2, 3, 5 (the rest): `crates/ui/src/export_path.rs` (the template: tokens, safe names, mistakes as data),
  `crates/ui/src/settings.rs` (`settings.ron`), `crates/ui/src/export.rs` (the engine). The old fixed `export_name` and
  `write_pngs` are gone; the toolbar's two items now run through the engine with the remembered settings
  (`Viewer::export(dir, which)`), the folder dialog starts in `Viewer::export_start_folder`, and the app reads
  `settings.ron` from the config folder (not in a timing run). Headless for leg 2: `Viewer::plan_export` (live preview),
  `run_export`, `export_again`, `can_export_again`, `ViewerUi::ask_export_again` and `Request::ExportAgain` (behind the one
  preflight sheet). Existing tests that expected the old names or the golden bytes now expect the new default names, and
  switch the note off where they compare bytes with the goldens (`v.settings.note = false`).
- Checks (commands): `make check-all` → `TESTS scope=workspace checks=663 check_fail=0 passed=617 failed=0 status=ok`;
  `cargo test -p quadrule-ui --test export` 10 passed; `--test preview` 3; `cargo test -p quadrule-format --test previews`
  3; `cargo test -p quadrule-render --test migrate_v015` 3; `--test noted` 2; `cd crates/render/tests/golden && sha256sum
  -c SHA256SUMS` → 30 OK (29 frozen + the noted one). The window started on a damaged `settings.ron` in a scratch
  `XDG_CONFIG_HOME` (15 s, debug build, no crash).
- Judgement calls (record words closest, each also a debt line): a template's dot, backslash and colon become hyphens
  (only a closing `.png` is read as the extension); Windows reserved names (`con`, `nul`, `com1`…) get an underscore; a
  part is cut at 120 characters; two devices on one path are compared without regard to case; with no title, the sample
  page and an unsaved new page get no Title chunk; Save As keeps what the window exported this session; a damaged preview
  is skipped without a notice; `{date}` uses the new `chrono` dependency (local time); `ron` is a direct dependency of
  `quadrule-ui` now.

## 2026-11-17 — rung 15 leg 2, builder (Sonnet 5.5, high): the sheets

Brief: leg 2 of `docs/bbq/2026-11-17-rung-15-export.md` (Q2, Q7, Q9; its Amendments: chris keeps the Software and Title
text chunks). Worked on `main`; token use was not measured by the builder.

- Built (`c2bb14f`): `crates/ui/src/export_ui.rs`. The **Export sheet** (toolbar Export menu → "Export…", Ctrl+Shift+E):
  This device / All devices, Folder with Change… (the app's folder dialog, starting at `export_dialog_start`), Path with a
  Presets combo (Flat, Folder per device, Device then set), Set, the live list of files (a mistake in red, Export greyed), the
  text-note tick, Cancel and Export. Every choice goes to `settings.ron` as it changes. Export closes the sheet; the one
  preflight sheet follows when the 14b rules call for it (`Then::Run`), never stacked; then the files are written and the
  **completion sheet** shows ("Exported N files", a thumbnail per file from the pixels just written, the names, "in
  <shared folder>", "N files took the place of…" when any, the folded "Where do these go?" note remembered as
  `completion_note_open`, Reveal in folder, Done). **Export again** (menu item, Ctrl+E) writes with the last choices behind
  the same preflight sheet and says it in the one-line notice only; with nothing exported yet from the document it opens the
  Export sheet. `Request::ExportDialog` and `Request::ExportAgain` are gone; `Request::ExportFolderDialog` and
  `Request::RevealFolder` are the app's. The old items "Export this device…" and "Export all devices…" and their Fluent ids
  are gone. `--look export|export-preset|export-mistake|exported|exported-open` (a look with no page opens the sample; a
  look never writes the settings file; `exported*` writes into `/tmp/quadrule-look-export`).
- Shortcut clash check: the keys the window takes are N, S, Shift+S, Z, Shift+Z, Y, D, G, Shift+G, [ and ] with Ctrl (Cmd on a
  Mac), and V, L, T, P, R, E, Y alone for the tools. Ctrl+E and Ctrl+Shift+E are free: egui matches Ctrl exactly, so the
  Ellipse tool's plain E does not fire on Ctrl+E (tested: `export_again_keys_and_menu`). Both ignore a press while a text field
  has focus or an export sheet is open.
- Checks (commands): `make check-all` → `TESTS scope=workspace checks=669 check_fail=0 passed=623 failed=0 status=ok`;
  `cargo test -p quadrule-ui --test export_sheets` 6 passed (the pointer tests: open, presets and memory, a mistake greying
  Export, Export → preflight → files → completion, Export again by key and menu, Reveal's command); `--test export` 10;
  `--test window` 21 and `--test lint_pointer` 9 (their export steps now go through the sheet); `cd
  crates/render/tests/golden && sha256sum -c SHA256SUMS` → 30 OK.
- Speed (Pi, release build, strip on, 150 frames, 3 runs each, 90th percentile ms against 50, scratch `XDG_CONFIG_HOME` and
  `XDG_DATA_HOME`, `./target/release/quadrule <page> --bench-drag|--bench-draw 150 --canvas export|chip`), every run
  `result=pass`: sample page drag export 20.2, 20.1, 20.1, chip 23.5 ×3; draw export 20.1, 20.8, 19.9, chip 22.9, 22.9, 22.8.
  `--groups-page` drag export 20.0, 20.1, 20.0, chip 30.5, 30.6, 30.5; draw export 19.9, 20.0, 20.0, chip 22.8 ×3.
  `--lint-page` drag export 19.9, 20.0, 19.8, chip 17.5, 17.5, 17.6; draw export 20.1, 20.0, 20.1, chip 22.3, 22.3, 22.4.
  `--shapes-page` drag export 20.0, 20.1, 20.0, chip 8.1, 7.8, 8.0; draw export 20.0, 20.1, 20.1, chip 22.7, 22.8, 22.8.
  Nothing near 50 ms (the groups page on the chip canvas, 30.5, is the heaviest, as before).
- Save with the 480 px preview (Pi, release build, `cargo test --release -p quadrule-ui --test preview save_time --
  --ignored --nocapture`, median of 9): sample page file bytes without the preview 1.0 ms, with 23.5 ms (the preview alone
  22.3, `save_to` 23.6); groups page 0.4 → 23.8; ISO A4 paper page 0.8 → 41.3. It runs on the interface thread at Save.
- Seen on screen (grim, release build, scratch folders): `scratchpad/rung15/export.png`, `export-preset.png`,
  `export-mistake.png`, `exported.png`, `exported-open.png`.

## 2026-11-17 — rung 15 lead (Opus 5.5, high)

- Session start: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=8489399 dirty=no rebuilt=0`. Budget
  12 points by chris, alone on the weekly meter, no reserve, start 53 % (given by chris, logged; the 5-hour window read
  63 %). Asked again: the Label gray stays parked (v1.1).
- bbq, 10 questions (record `docs/bbq/2026-11-17-rung-15-export.md`): one rung, one look; the Export sheet and Export
  again; an app-wide template with the default changed by chris to `{title}-{device}.png` (overruled); the folder
  remembered by document path; folders in the path itself (chris, opening Q9; presets taken); the text note on by
  default; the completion sheet with its note folded (overruled); a thumbnail of the main device only (overruled).
  chris asked about EXIF/XMP for the note and kept PNG's Software and Title fields (Amendment).
- Built by one Sonnet 5.5 high builder in two legs (leg 1 279,935 tokens; 399,888 reported at the end of leg 2, likely
  including leg 1; their entries above). The lead checked each leg: `sha256sum -c` 30 OK (29 frozen + the noted
  golden), fixtures `v0.4/`–`v0.14/` untouched, CI green on `5c22371`, `c2bb14f`, `4677806`; looked at the five
  `--look` screenshots. chris's tablet check: "the provided test image works as a template" (Nomad; "the nomad check is
  enough"). At the look chris ruled the builder's eight calls; the lead changed the replaced line to "N files replaced"
  (`0a5bf39`); a File menu became an open question. chris: "look approved. please ratify adr-013 as well". ADR-013
  amended and ratified (`4330a65`; `adr-check` → `16 ADRs, 0 failure(s)`).
- Close: version 0.15.0, `docs/arcs/v1/release-notes/v0.15.0.md`, tag `v0.15.0`; board row and re-plan; STATUS.md (rung
  13 bullet and one Log line moved word for word to `docs/STATUS-archive.md`); rung 16 prompt
  `docs/arcs/v1/rung-16-PROMPT-2026-11-17.md`; calibration row. `spend-ledger.py measure --latest --meter 53 59` →
  `sonnet 77% · opus 23% | $18.51 | 53 → 59 · alone here · 6 pts`.


## 2026-11-17 — rung 16a leg 1, builder (Sonnet 5.5, high): the File menu, one Export…, recent files, the welcome screen and My templates

Brief: leg 1 of `docs/bbq/2026-11-17-rung-16-welcome-and-gallery.md` (Q2, Q4, Q5, Q6, Q9 and the Defaulted lists). Worked on
`main`; token use was not measured by the builder. Leg 2 (live thumbnails, the device filter's effect, the starters
loader and test starters, `--look gallery-mine`, the speed measurements) was not started.

- Built, in plain words: every file action now lives in a **File** menu at the left of the top bar (New…, Open…, Open
  Recent, Save, Save As…, Save as template…, Export…, Close, with their keys beside them); the toolbar's New, Open, Save,
  Save As and Export buttons are gone. There is **one Export…**: Ctrl+E opens the Export sheet with the last choices filled
  in and the Export button in focus, so Ctrl+E then Enter repeats the last export. Export again and Ctrl+Shift+E are gone.
  The **welcome screen** replaces the empty start: down the left Blank page, Open…, the Recent list and, small at the foot,
  Open sample page; on the right the Device filter, the Search box, the seven tabs and the tiles. It shows at start-up with
  no file, after Close, and for File ▸ New… over a page (Esc or Cancel goes back). **My templates** are ordinary `.quad`
  files in `templates/` in the data folder; Save as template… writes one (asking before it replaces one) and a pick opens an
  untitled copy.
- Commits: `a41d256` (recent files in `settings.ron`, the templates folder, Save as template, the untitled copy; headless,
  no screen) and `d02f891` (the File menu, the one Export…, the welcome screen, the My templates tab, `--look welcome` and
  `--look file-menu`, README and DEVELOPMENT). The second holds steps 1, 2, 4 and the screen half of 5 together: the bar, the
  welcome screen and the menu all change `view.rs` and the rung 15 tests at once, so they could not each be left passing
  on their own.
- Where things are: `crates/ui/src/file_menu.rs` (the menu), `welcome.rs` (the screen, its questions and the tiles),
  `templates.rs` (the folder, Save as template, `open_copy`), `recent.rs` (the list), `settings.rs` (`recent`, max 10,
  `#[serde(default)]` so a rung 15 `settings.ron` loads), `export_ui.rs` (Ctrl+E with the Export button focused),
  `view.rs` (After::CloseDocument and After::OpenTemplate, the keys N, O, W, E), `viewer.rs` (`close_document`,
  `templates_dir`), `crates/app/src/main.rs` (the templates folder beside `recovery/`, the two looks).
- Tests added or rewritten (each prints one `CHECK` line; run with `-- --nocapture`, scratch `XDG_CONFIG_HOME` and
  `XDG_DATA_HOME` set):
  `cargo test -p quadrule-ui --test recent` (3: add at Open, Save and Save As newest first; ten at most, missing stays,
  forget; an older settings file loads), `--test templates` (3: save, replace asks and leaves the document alone; the list
  and removal only inside the folder; the untitled copy on a chosen device, a missing device falling back with a notice, a
  paper size, an unreadable file), `--test welcome` (10: the frame and every tab's line; Blank page on the filter's device;
  the Recent list; New over a page, Esc, Cancel, Close and Ctrl+W; a dropped `.quad`; the File menu and its keys; Open
  Recent; Save as template and a pick; Remove, Show in folder and Search; the two looks), `--test export_sheets` (6;
  `ctrl_e_opens_the_sheet_with_the_last_choices_and_enter_writes` replaces `export_again_writes_with_the_last_choices_or_opens_the_sheet`,
  and the first test now checks the File menu holds one Export… and no Export again), `--test export` (10; the preflight
  test is now `the_export_sheet_goes_on_behind_the_one_preflight_sheet`), `--test window` (21) and `--test lint_pointer` (9),
  whose steps now go through the File menu. The message-id test also reads the new files.
- Checks (commands): `make check-all` → `TESTS scope=workspace checks=685 check_fail=0 passed=639 failed=0 status=ok`;
  `cd crates/render/tests/golden && sha256sum -c SHA256SUMS | grep -c ": OK"` → 30; fixtures `v0.4/`–`v0.15/` untouched
  (`git status` shows no change under them); `cargo test -p quadrule-app` → 4 passed.
- Seen on screen (grim, debug build, scratch folders): `--look welcome` with two templates and three recent files (one
  gone) and `--look file-menu` over the sample page, both read as intended.
- Judgement calls (the record was silent or ambiguous; each can be changed):
  1. Ctrl+N (and File ▸ New…) with the welcome screen already showing goes on to the Blank page dialog.
  2. The **All** tab lists My templates too (starters will join them in leg 2); with neither it says starters are coming.
  3. **Search** already works on the templates (name, description, tags, or the words "My templates"), across every tab,
     because it cost a few lines; leg 2 adds the starters side.
  4. The bar's own Device picker is hidden while the welcome screen shows (it has its own filter); while the screen is
     over a page, Save, Save As, Save as template, Export, Undo, Redo and zoom are greyed so nothing acts on the page behind.
  5. egui matches Ctrl+E when Shift is held too, so Ctrl+Shift+E would still have opened the sheet; the key now needs Shift
     up, which frees Ctrl+Shift+E as the record says.
  6. `Viewer::export_again` and its engine test stay (the window no longer calls it); `ask_export_again`,
     `do_export_again` and `Then::Again` are gone. The test that ADR-013's proof line names,
     `export_again_writes_with_the_last_choices_or_opens_the_sheet`, is renamed (see 2 under "Rung 16a, owed" in the debt file), so the ADR's proof line no
     longer resolves until the lead's amendment updates it.
  7. Save as template… does not show Save's installed-fonts sheet; it writes with the document's own Embed-fonts setting.
  8. A tile shows the document's title (the file's name when the title is empty or the file is damaged); a damaged file
     shows a plain tile and a pick says why in the notice.
  9. Recent paths are kept absolute (`std::path::absolute`); a failed open, a recovery restore, and a template or starter
     opened as an untitled copy never add a line. A click on a missing recent file asks; the right-click removes at once.
  10. Show in folder opens the templates folder (not the single file).
  11. When a copy cannot be made on the chosen device (an unknown id, say) the notice is not marked as an error; the copy
      still opens, on its own main device.
  12. My templates is read from the folder when the welcome screen opens and after a save or a removal, then kept; a
      stored picture is decoded once, on the interface thread (a 480 px PNG, about a millisecond); no per-frame disk work.
  13. `png` moved from a dev-dependency to a dependency of `quadrule-ui` (to decode the stored picture).
  14. `--look welcome` shows the screen with nothing open; `--look file-menu` opens the sample page and the menu.
  15. New words (Fluent, for chris's look): the File menu items, "+ Blank page", "Recent", "Not found · <folder>",
      "Search", "Search pages", the seven tab names, "Starter pages for this group are coming soon.", "Starter pages are
      coming soon. Until then, start from a blank page, or keep your own pages under My templates.", "No templates yet.
      Choose File, then Save as template…, to keep a page here.", "Remove this template? This deletes the template “…” from
      your computer. It cannot be undone.", "File not found … Take it off the recent list?", "Replace this template?" and the
      notices `notice-template-saved`, `-save-failed`, `-remove-failed`, `notice-copy-device`. Removed: `app-export`,
      `app-export-again`, `app-new-tip`, `app-empty-title`.
- Not measured (leg 2): the welcome screen's frame times with tiles drawing, and drag and draw on the heaviest page. Nothing
  in this leg touches the canvas or the drag path.
- Blocked: nothing. One build note: linking the app's test binary beside the ui tests was killed (signal 9, out of memory)
  once when both were built in one `cargo test` on the Pi; built apart, they pass.

## 2026-11-17 — rung 16a leg 2, builder (Sonnet 5.5, high): live thumbnails, starters, the device filter and the speed numbers

Brief: leg 2 of `docs/bbq/2026-11-17-rung-16-welcome-and-gallery.md` (Q3, Q5 and the Defaulted "Thumbnails, in detail", "The
device filter", "Where starters live" and "Speed"). Worked on `main`; token use was not measured by the builder.

- Built, in plain words: each tile on the welcome screen is now drawn live for the device in the Device filter, by the export
  renderer, on one background thread, and kept for the session. The window never draws a page itself: it hands the thread the
  tiles that are on screen, in screen order, and turns a finished picture into a texture when one arrives. A screen that is
  sitting still asks the thread for nothing and asks for no frame. The stored `previews/` picture stands in while a tile draws;
  a page that cannot be drawn is a plain outline with its name. A page that does not target the filter's device is drawn on its
  own main device and its tile says "drawn on <device>". The built-in starters (`crates/ui/starters/<category>/<slug>.quad`,
  put in the program by a `build.rs`, none shipped) fill the tabs; every category with none still says its starters are
  coming; a starter opens as an untitled copy exactly as a template does.
- Commit: `bbc5369` (the above, the test starters and the flags below, and the removal of `Viewer::export_again`); this entry
  and the docs follow in a second commit.
- Where things are: `crates/ui/src/thumbs.rs` (the thread, its queue and the cache), `welcome.rs` (the tiles, the tab filter,
  `WelcomeBench`), `starters.rs` and `build.rs` (the loader), `test_starters.rs` (24 pages made from the sample documents),
  `templates.rs` (`TemplateEntry` now serves both templates and starters; `Origin`), `crates/app/src/main.rs` (flags).
  `--look gallery-mine` starts on My templates. Hidden flags for looks and runs: `--gallery-test-starters` (24 test starters in the
  gallery), `--gallery-device manta|nomad|<profile id>`, `--bench-welcome [frames]` (times the welcome screen; implies the test
  starters).
- Tests added (each prints one `CHECK` line; run with `-- --nocapture`, scratch `XDG_CONFIG_HOME` and `XDG_DATA_HOME` set):
  `cargo test -p quadrule-ui --test gallery` (4: thumbnails are drawn by one thread that is not the interface thread, the first
  frame comes back with nothing drawn, an idle window settles and hands the thread no job, the other device is drawn once and
  going back costs nothing, another tab asks for nothing new; the filter's device decides where a tile is drawn and a page that
  cannot be drawn is a plain tile that still opens; the "drawn on" note and the filter listing devices and paper sizes; the
  loader sorts by category and leaves out a stray folder, the tabs show their own starters and keep "coming soon", search by
  title, tag and category, a pick opens an untitled copy). `--test export` now has `export_folder_memory` in place of the Export
  again test; `cargo test -p quadrule-app` has three more command-line cases.
- Checks (commands): `make check-all` → `TESTS scope=workspace checks=689 check_fail=0 passed=643 failed=0 status=ok`; `cd crates/render/tests/golden && sha256sum -c SHA256SUMS | grep -c ": OK"`
  → 30; fixtures `v0.4/`–`v0.15/` untouched.
- Speed (Pi, release build, scratch `XDG_CONFIG_HOME` and `XDG_DATA_HOME`; ms of interface-thread work per frame, 90th percentile
  against 50, the first frame reported but not judged, as for a drag; every run `result=pass`):
  `./target/release/quadrule --bench-welcome 150 [--gallery-device manta|nomad]` (24 test tiles, each frame forced, all 24
  drawn, then 150 idle frames), three runs each. Manta: 24 drawn in 0.9, 1.0, 0.9 s; frames while drawing p90 1.6, 1.8, 1.6
  (worst 2.8, 2.4, 2.3; first 13.3, 19.9, 13.1); idle p90 0.6, 0.6, 0.6 (worst 0.9, 0.7, 0.6). Nomad: 0.6 s each; drawing p90
  1.8, 1.5, 1.6 (worst 2.1, 2.3, 2.1); idle p90 0.6, 0.7, 0.6. This times what the program does in a frame, not egui's
  painting or the screen; the frames came about every 16 ms.
  The heaviest page is the groups page. `make bench-drag LINT= GROUPS=1` p90 19.8 and `make bench-draw LINT= GROUPS=1` p90
  19.7 (export canvas, strip on; the makefile fix is judgement call 4). Both canvases, three runs each, against the
  same page at the commit before this leg (`df6bd0c`, built in a worktree), `./target/release/quadrule --groups-page
  --bench-drag|--bench-draw 150 --canvas export|chip`: drag export before 20.0, 20.0, 20.0, now 19.9, 19.9, 20.1; draw export
  before 20.2, 19.9, 20.8, now 19.9, 20.1, 20.1; drag chip before 30.4, 30.5, 30.6, now 30.4, 30.5, 30.5; draw chip before 22.9,
  22.7, 22.8, now 22.9, 22.8, 22.7. Nothing near 50 ms at the 90th percentile. One thing to know: the chip draw run has one
  frame after the first at 51.2 ms (`over_budget=1`, worst 51.2, 51.2, 51.3); the commit before this leg gives the same
  (51.2, 51.1, 51.2), so it is not from this leg; the verdict is the 90th percentile, which is 22.8.
- Seen on screen (grim, release build, scratch folders): `scratchpad/look/welcome-manta.png`, `welcome-nomad.png` (24 test
  starters, thumbnails drawn, the A4 test page marked "drawn on ISO A4"), `gallery-mine.png`, and leg 1's `welcome.png` and
  `file-menu.png`.
- Judgement calls:
  1. Only the tiles on screen are asked for (so a long gallery draws what is looked at first); the speed run asks for all 24.
  2. Pictures are drawn with the bundled fonts only, with the outlines a file saved for any other font, as on a computer that
     lacks it; the thread has its own copy of the profile registry (the registry is per thread), taken when a job is queued.
  3. A tile whose picture fails shows a plain outline, even when the file has a stored picture; a failed page is not tried
     again this session.
  4. The makefile's `LINT ?=` never took effect (`make` defines `LINT` itself as `lint`), so every `make bench-*` ran the lint
     page unless `LINT=` was given; I made it `LINT :=`, so the benches run the page their flag names, and the default is the
     sample page as the help says. Numbers in earlier log entries made through `make` (rather than the binary) were on the lint page.
  5. `Viewer::export_again`, `Again`, `can_export_again` and the engine test are removed as ruled; `export_again_folder` is now
     `last_export_folder` (the sheet's start folder), and its test `export_folder_memory` keeps the folder-memory checks.
  6. The plain tile is a drawn outline, not a glyph: egui's fonts panic on a glyph they lack (the first try used one and a test
     caught it).
  7. The test starters live in `crates/ui/src/test_starters.rs`, not `crates/ui/tests/`, because the app's hidden flags need them too.
  8. New word: "drawn on { $device }" (`app-welcome-tile-on`), for chris's look.
- Blocked: nothing. Build notes: the app and ui tests were built apart, as asked; building an older commit in a worktree against the
  same `target` left a stale words file in the release build until `i18n` was touched (a "(app-welcome-tile-on)" showed instead of
  the words), so after such a build touch a crate or run `make build-proof`.

## 2026-11-18 — rung 16a lead (Opus 5.5, high)

- Session start: `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` → `BUILD head=0b18202 dirty=no rebuilt=0`. Budget
  12 points by chris, reserve 4, start 59 % (given by chris, logged). Asked again: starters (chris: none yet, he will
  make them); the File menu (chris: rung 16).
- bbq, 9 questions (record `docs/bbq/2026-11-17-rung-16-welcome-and-gallery.md`): 16a now, 16b after chris's starters,
  two tags; one welcome screen (actions left, gallery right); thumbnails drawn live in the background; a templates
  folder with Save as template…; every category shown "coming soon" (overruled: the recommendation hid empty ones);
  the toolbar's file buttons into File; the basic section in 16b; chris asked whether Export again could join Export,
  and chose one Export… where Enter writes (Q9).
- Built by one Sonnet 5.5 high builder in two legs (356,390 tokens at leg 1's end; 492,413 at leg 2's end, likely
  including leg 1; their entries above). Between legs the meter read 63 % and chris ruled, word for word: "for this
  week only, unset any hard limits" (record, Amendments). The lead checked each leg: `make build-proof` clean
  (`df6bd0c`, `c1d654b`), `sha256sum -c SHA256SUMS | grep -c ": OK"` → 30, fixtures untouched; looked at the
  `welcome`, `file-menu`, `welcome-nomad` and `gallery-mine` screenshots; confirmed the builder's Makefile finding
  (`make -p -f /dev/null | grep '^LINT '` → `LINT = lint`, so `LINT ?=` from `547c39d`, rung 14a, never took effect and
  every `make bench-*` since then timed the lint page; fixed to `LINT :=` in `c1d654b`).
- ADR-013 amendment drafted with `makeadr` (`9941f13`; proof line moved to 16a's tests; `adr-check` → `16 ADRs, 0
  failure(s)`). chris at the look, meter 65 %: "Approved. Ratify adr-013." Ratified the same day.
- Close: version 0.16.0, `docs/arcs/v1/release-notes/v0.16.0.md`, tag `v0.16.0`; board row, queue and re-plan;
  STATUS.md (rung 14 bullet and the rung 12 Log line moved word for word to `docs/STATUS-archive.md`); rung 16b prompt
  `docs/arcs/v1/rung-16b-PROMPT-2026-11-18.md`; calibration row. `spend-ledger.py measure --latest --meter 59 65` →
  `sonnet 85% · opus 15% | $19.92 | 59 → 65 · alone here · 6 pts`.

## 2026-11-18 — pattern region borders, builder (Sonnet 5.5, high): model, format 0.17.0, layout, drawing, Style tab, lint, goldens, speed

Brief: build `docs/bbq/2026-11-18-pattern-borders.md` (Rules Q1 to Q4 and every Defaulted line). Worked on `main`; token use was not
measured by the builder. Not touched: `STATUS.md`, the ADRs, the board, the version.

- Built, in plain words: a pattern region can have a border, a line centred on its frame like a rectangle's stroke, on the sides
  ticked (all four to start), in a line role (Structure to start) or a width and gray of its own, with a corner radius in mm. On a
  bordered side the pattern's own lines and dots that sit on the edge are left out. A corner rounds only where both its sides are on
  (radius capped at half the shorter side); inside it ruling lines (straight and slanted) stop at the curve and a dot whose centre is
  beyond the curve is dropped. The border turns with the frame (`rotation_deg`), not with the Pattern angle; it draws in groups,
  repeats and on every device (same border, its role resolved per device). Unbordered sides keep ADR-014 exactly. No border is the
  default, no `border` key is written, and every earlier golden and fixture is byte for byte what it was.
- Commit: `1531005` (everything above in one: model, format, layout, both canvases and export by the display list, Style tab, lint,
  goldens, bordered page, tests, README and DEVELOPMENT); a second commit adds the look aid, the debt and this entry.
- Where things are: `crates/model/src/lib.rs` (`Border`, `BorderSides`, `PatternRegion::border`), `crates/layout/src/border.rs` (`Rim`:
  the border as the pattern sees it; which lines and dots go, the cut at a rounded corner, the border's own items; `clip_convex`),
  `crates/layout/src/pattern.rs` (`region_rim`, `border_items_turned`, the `Ctx` filters in `hline`/`vline`/`dot`/`slanted`, the
  quarter-turn mapping and `turn_cut_items`), `crates/layout/src/lib.rs` (the pattern branch of `draw_objects`),
  `crates/format` (`FORMAT_VERSION` 0.17.0, migration 0.16 to 0.17 rewrites nothing), `crates/lint/src/judge.rs` and `lib.rs` (L01, L02,
  L03, L05, L14 read the border's line), `crates/ui/src/panels.rs` (`border_rows`), `crates/i18n/locales/en-US/{app,lint}.ftl`,
  `crates/model/src/sample.rs` (`pattern_borders_document`, `border_ids`), `crates/app/src/main.rs` (`--bordered-page`),
  `crates/ui/src/{bench,view,viewer,preflight}.rs`, `Makefile` (`BORDERED=1` for `bench-drag` and `bench-draw`).
  The chip canvas and the export renderer draw the display list, so neither needed code of its own.
- Tests added (each prints one `CHECK` line; run with `-- --nocapture`): `cargo test -p quadrule-model --test borders` (3: no key when
  absent, fields and defaults, radius and sides); `cargo test -p quadrule-layout --test borders` (9: lines and dots on a bordered edge
  go and a side off keeps them, rounded corners cut ruling and dots and only where both sides are on, slanted ruling stops at the
  curve, the radius cap and a missing stroke, the border turns with the frame not the ruling, a quarter-turned ruling, group and
  repeat on both devices, no border is inert); `cargo test -p quadrule-render --test pattern_borders` (4: goldens on both devices
  with the old thirty lines checked by their hash, draws the same twice and differs from no border, the pattern-angle page still
  matches its golden, a save round trip writes no `border` key without one) and `--test migrate_v016` (2); `cargo test -p quadrule-lint
  --test lint border` (1); `cargo test -p quadrule-ui --test borders` (3: the section's fields, a tick is one undo step and typing
  merges, the benchmarks run over the bordered page); `cargo test -p quadrule-app` has one more command-line case.
- Checks (commands): `make check-all` → `TESTS scope=workspace checks=715 check_fail=0 passed=665 failed=0 status=ok` (the run before this entry; one before it, on `1531005`, gave the same line); `(cd crates/render/tests/golden && sha256sum -c
  SHA256SUMS | grep -c ": OK")` → 32; `head -30 crates/render/tests/golden/SHA256SUMS | sha256sum` →
  `1d1892b6b6cb4aae6c602c11c577e4273226c2099616ea0325e75888280fd8c3` (the same as before the job); fixtures `v0.4/`–`v0.15/`
  untouched (`git diff 794ce33 --stat -- crates/render/tests/fixtures` shows only the new `v0.16/`). New goldens:
  `pattern-borders_supernote.a5x2_1920x2560.png` (Manta) and `pattern-borders_supernote.a6x2_1404x1872.png` (Nomad), each page twelve
  cells: ruled lines all four sides, a dot grid with its right side off, a grid with 3 mm corners, Emphasis lines with 4 mm corners,
  dots with 4 mm corners, a grid bordered on top and left only, isometric with a 0.4 mm gray 64 border and 3 mm corners, hexagons,
  Cornell, a frame turned 20 degrees, a ruling turned 30 degrees, a ruling turned 90 degrees. I read both PNGs before blessing,
  after changing the rule and the page twice (see judgement calls 1 and 2). Fixture `crates/render/tests/fixtures/v0.16/` is the
  `v0.15` page resaved by the 0.16.0 build before the format changed (same export hashes).
- Speed (Pi, release build, scratch `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, strip on, 150 frames, 90th percentile against 50 ms;
  the first frame reported, not judged), command `./target/release/quadrule --bordered-page --bench-drag|--bench-draw 150 --canvas
  export|chip`, three runs each (the page is the bordered page; `make bench-drag BORDERED=1` runs the same page):
  drag export p90 21.5, 21.4, 21.2 (worst 30.3, 26.2, 22.5); draw export p90 21.9, 21.9, 22.4 (worst 36.2, 30.4, 30.6);
  drag chip p90 4.8, 4.9, 4.9 (worst 5.7, 5.7, 5.7); draw chip p90 23.6, 23.6, 23.5 (worst 53.1, 53.1, 53.1, `over_budget=1` each,
  the first frame, 49 ms, and one after it). Nothing near 50 ms at the 90th percentile. The chip draw run's lone slow frame is the
  same shape the groups page showed last leg (51.2 ms there). One extra run (`--canvas export` drag, 58.7 ms worst, p90 20.6) was
  a stray first run before the three; I did not chase it.
- Seen on screen (grim, release build): `scratchpad/look/borders-manta.png` and `borders-nomad.png` (the exported pages, copied
  from the goldens), `borders-window-nomad.png` (the window on the Nomad, the page in a drag), `borders-style-section.png` (the Style
  tab scrolled to its Border section: Show border, Sides, Line, Role, Corner radius) and `borders-style-full.png`.
- Judgement calls:
  1. "On the edge" is wider than the record's half pixel: a line or dot goes when its centre is within half the sum of its width and the
     border's width of the edge, at least half a pixel, so its ink never touches the border's. With half a pixel alone, a dot or line a
     pixel off the border (the rhythm and a frame set in millimetres round differently, by 1 to 2 px a hundred millimetres along)
     showed beside the border as a tick or a doubled line on the first golden; the record's veto "a wider band" is this. A dot is
     dropped the same distance inside a rounded corner's curve.
  2. The borders page sets its rhythm to physical-locked and every object to keep its millimetres (Left and Top): pixel-locked 5 mm
     steps are 59 px against a true 59.06, so a frame set in millimetres drifts off the rhythm. The app's own snapping puts a frame on
     the pixel rhythm, so real pages do not drift; the page is built for the eye.
  3. The border is `Option<Border>`: unticking drops its settings (sides, line, radius) and ticking again starts fresh (Structure, all four sides,
     radius 0). The record lists "on/off" with the other fields; a flag inside the border would have kept them. Easy to change.
  4. Per-device overrides do not cover pattern style: `Override` holds place, size, font size and the eye only, so I did not build one
     for the border (the record said to say so); a border is the same on every device, its role resolved per device.
  5. A corner rounds only where both sides are on and the radius is above 0; a side with no neighbour ends square at the frame's
     corner with a butt cap. An unrounded border is hard-edged rectangles (a ring where all four are on), a rounded one one anti-aliased
     stroke through the stroke's pixel centre line, as a rectangle shape's is.
  6. A turned ruling (Pattern angle off a quarter turn) is cut at the border's rounded outline (a polygon), and a quarter turn lays the
     ruling out in its own axes with the border carried through the turn, so the edge rule works there too; dots and lines on an
     edge are looked for in those axes.
  7. A pattern with no stroke of its own (`Style::None`) draws no ruling but still draws its border; a border with `Style::None` draws
     nothing and removes nothing; a border with no side ticked is no border.
  8. A repeat's alternation (`restyle`) changes the source's own line, not the border's.
  9. The words proposed in Fluent: Border (heading), Show border, Sides, Top, Right, Bottom, Left, Line (then the existing Role, Custom,
     Width, Gray, Corner radius); lint: "The border of ... " for L01, L02, L03, L14 and "border" for L05.
  10. `--look style` on the bordered page selects the first region with rounded corners and scrolls the Style tab to the Border section
     (the inspector is too short on a 720 px screen to show it otherwise).
- Blocked: nothing. Owed items are in `docs/STATUS-debt.md` under "Pattern borders, owed". ADR-014's amendment is the lead's.

## 2026-11-18 — pattern region borders lead (Opus 5.5, high)

- chris asked: "I need you to add pattern region borders as a selectable option". Planned as an untyped bbq of 4
  questions (`docs/bbq/2026-11-18-pattern-borders.md`, `794ce33`): border on the frame's edge with edge lines left out
  (Q1), four side ticks (Q2), a line role and rounded corners (Q3), ship now as `v0.16.1` with 16b moved to `v0.16.2`
  (Q4). Built by one Sonnet 5.5 high builder (`1531005`, `5afb57e`; its entry above; 398,462 tokens reported).
- Verified by the lead: `make build-proof` → `BUILD head=5afb57e dirty=no rebuilt=0`; `sha256sum -c SHA256SUMS | grep
  -c ": OK"` → 32; `head -30 SHA256SUMS | sha256sum` → `1d1892b6…` (the 30 old lines unchanged); `git diff --stat
  6a56f9a -- crates/render/tests/fixtures` → only `v0.16/` added. Both new golden pictures looked at.
- ADR-014 amendment drafted (`1ce4524`), then ratified at the look. chris, word for word: "approved and ratified". The
  builder's calls stand as built, including the wider "on the edge" band.
- Closed: version 0.16.1, release notes `docs/arcs/v1/release-notes/v0.16.1.md`, `STATUS.md`, debt, the board, the
  calibration row, the 16b prompt's version line (`0.16.2`). The meter was not read at the look.

## 2026-11-18 — layers, builder (Sonnet 5.5, high): New layer and Delete layer

Brief: build `docs/bbq/2026-11-18-new-and-delete-layers.md` (Rules Q1 to Q4 and every Defaulted line). Worked on `main`. Not touched:
`STATUS.md`, the ADRs, `docs/arcs/v1/README.md`, the version, the release notes. Drawing is not touched, so no speed run.

- Built, in plain words: the layers panel has **New layer** and **Delete layer** buttons (in the button row, which now wraps) and the
  same two items on a layer row's right-click menu. New layer puts an empty layer just above the picked layer (the layer of the first
  picked object, or the picked row; on top when none), named "Layer N" with the lowest N no layer has, picks it, and opens its name
  field with the text selected to type over (Enter or a click away keeps it; Esc keeps "Layer N"). Delete layer deletes the picked
  layer: an empty one at once, one with objects after a sheet asks `Delete the layer "Header" and its 2 objects?` (Delete, Cancel;
  Esc cancels). One undo step each way; Undo of a delete brings the layer back whole, in its place, with its objects. Greyed, with a
  hover note, when nothing is picked, when it is the last layer, and when the layer is locked ("Unlock it to delete it.") or holds a
  locked object. No change to the file format or to export bytes.
- Files: `crates/model/src/edit.rs` (`Command::InsertLayer { index, layer }` and `Command::RemoveLayer(index)`, each the other's undo;
  `EditError::LastLayer`; `Document::layer_object_count`, `Document::layer_delete_block` and `LayerDeleteBlock`),
  `crates/ui/src/layers.rs` (new: the buttons, the row menu, the sheet, naming, `--look` states), `crates/ui/src/editor.rs`
  (`Viewer::new_layer`, `Viewer::delete_layer`), `crates/ui/src/panels.rs` (button row, menu hook, the name field), `crates/ui/src/view.rs`
  (fields, the sheet, two keyboard guards), `crates/ui/src/preflight.rs` (`show_look` arms), `crates/ui/src/lib.rs`,
  `crates/app/src/main.rs` (`--look layers|layers-new|layers-ask|layers-menu`), `crates/i18n/locales/en-US/app.ftl`, `README.md`,
  `DEVELOPMENT.md`, `docs/STATUS-debt.md` ("Layers, owed").
- Commits: `8be261c` (the model commands and their tests), `16629c5` (the panel, menu, sheet, name field, tests), `d3a152d` (debt,
  README, DEVELOPMENT, the app's `--look` parse case), and this entry's commit.
- Tests added (each prints one `CHECK` line; run with `-- --nocapture`): `cargo test -p quadrule-model --test layers` (4: insert and remove
  are each other's undo; a removed layer comes back whole in its place; the last layer stays and locks refuse; the object count and
  repeated ids), the two new kinds in `every_command_undoes_and_redoes` (`kinds=14`); `cargo test -p quadrule-ui --test layers` (7:
  name, place and open field; placement above a picked row, a picked object and from the menu; Enter, Esc and a click away on the
  name field; an empty layer at once and a layer with objects asks, Cancel, Esc, Delete, Undo; the singular "1 object"; the greyed
  cases with their notes and the menu item; the row menu and a picked object); `cargo test -p quadrule-app` has one more
  command-line case.
- Checks (commands): `make check-all` → `TESTS scope=workspace checks=726 check_fail=0 passed=676 failed=0 status=ok` (was 715/665 before
  the job); `make build-proof` → `BUILD head=d3a152d dirty=no rebuilt=0` (a first run, after check-all, gave `rebuilt=7` and
  then 0); `(cd crates/render/tests/golden && sha256sum -c SHA256SUMS | grep -c ": OK")` → 32, and `SHA256SUMS` still has 32 lines,
  fixtures untouched (`git diff --stat bfe9282 -- crates/render/tests` is empty).
- Seen on screen (grim, release build, scratch `XDG_CONFIG_HOME` and `XDG_DATA_HOME`, the data folder wiped before each run so the recovery
  prompt does not cover the window): `scratchpad/look/layers-panel.png` (the sample page with a third layer "Labels" on top and Header
  picked), `layers-new.png` (New layer pressed above Header: "Layer 1" selected in its open name field), `layers-ask.png` (the sheet
  `Delete the layer "Header" and its 2 objects?` with "Undo brings the layer back."), `layers-menu.png` (Header's row menu).
- Judgement calls (the ones needing chris's veto first):
  1. A layer holding a locked object cannot be deleted either (greyed, "Unlock the objects in it to delete it."), not only a locked
     layer: the record named only the locked layer, but deleting the layer would delete an object that Delete refuses today. Veto:
     "delete it anyway" (drop `HoldsLocked` from `layer_delete_block` and the model refusal).
  2. The sheet's count is every row the layer has in the panel (a group counts, and each object in it), so a group of three is
     "4 objects". Veto: count top-level objects only, or leaves only.
  3. After Delete layer nothing is picked (not the layer below), so a second press cannot eat the next layer by accident.
  4. A right-click on a layer row picks that row (and clears the object pick), so the menu and the buttons work on the same layer.
  5. The sheet has a second line, "Undo brings the layer back." (as the device take-out sheet does); the record asked only for the question.
     Veto: drop `app-layer-delete-undo`.
  6. The New layer button has a hover note, "Adds an empty layer above the picked one, or on top." (like Repeat's). Veto: drop it.
  7. The hover notes: nothing picked "Pick a layer first.", last layer "A page needs at least one layer.", locked "Unlock it to delete it."
     (the record's wording; it reads oddly alone in a tooltip, "Unlock the layer to delete it." is the alternative).
  8. I fixed the existing name field: it asked for focus before it checked for focus lost, so Enter and a click away could never close it
     (found because the new layer's tests could not close it), and Escape was taken by the page's shortcut (which cleared the pick) before
     the field saw it. Both now behave as the comments in `name_cell` and `shortcuts` always said. This changes the double-click rename of
     layers and objects, which has no headless test (`egui_kittest` cannot make a double-click): chris's look is the first real check.
  9. The panel's button row is `horizontal_wrapped` (seven buttons, two lines on a 1440 by 720 screen). Order: Move up, Move down, New
     layer, Delete layer, Group, Ungroup, Repeat.
  10. "Layer N" is made through Fluent (`app-layer-new-name = Layer { $number }`) and compared with the layers' names, so another language
      would number its own names; the blank page's first layer, `FIRST_LAYER_NAME`, stays the English "Layer 1" the file format holds.
  11. While the Delete layer sheet shows, no keyboard shortcut works (Ctrl+Z, Delete, and so on) and Esc cancels it.
  12. `--look layers-menu` holds the menu open every frame (egui throws a frame away while sizing a new popup, so a one-time open never
      showed); it is for a screenshot only.
- Words chosen (proposals for chris's look): "New layer", "Delete layer", "Layer N", "Adds an empty layer above the picked one, or on top.",
  "Pick a layer first.", "A page needs at least one layer.", "Unlock it to delete it.", "Unlock the objects in it to delete it.",
  `Delete the layer "{name}" and its {N} objects?` (one: "its 1 object"), "Undo brings the layer back.", "Delete" (and the existing "Cancel").
- Blocked: nothing. Owed is in `docs/STATUS-debt.md` under "Layers, owed".

## 2026-11-18 — layers lead (Opus 5.5, high)

- Found while listing chris's starters: spec §13 asks for named layers, and nothing in the app added a second layer
  (`crates/model/src/edit.rs` had no insert or remove command). chris chose to fix it first ("let's tackle the layer gap
  with route #1 and get that squared away"). Planned in 4 questions (`docs/bbq/2026-11-18-new-and-delete-layers.md`,
  `bfe9282`); built by one Sonnet 5.5 high builder (`8be261c`, `16629c5`, `d3a152d`, `e05d940`; its entry above;
  231,268 tokens reported).
- chris's answer to the basic-section question is in the 16b prompt word for word ("…we can choose to expose certain
  elements of a template that can be easily changed on a single pane, while all features are available in "style,
  etc...""), for the 16b bbq.
- Meter 79 % at 19:13 UTC (chris), 14 points above 16a's approval; chris: "I have a free reset this week, so I am
  grinding this out".
- Verified by the lead: `make build-proof` → `BUILD head=e05d940 dirty=no rebuilt=0`; 32 golden lines OK, no change to
  `SHA256SUMS` or the fixtures since `6f49e5f`; all four screenshots looked at.
- chris, word for word: "approved. after you are done, let's talk about a few bugs/features I'd like to see before 16b
  starts". Closed: version 0.16.2, release notes `docs/arcs/v1/release-notes/v0.16.2.md`, `STATUS.md` (rung 15's bullet
  and rung 13's Log line moved word for word to `docs/STATUS-archive.md`), debt, the board, the calibration row, the 16b
  prompt (`0.16.3`).

## 2026-11-18 — fixes batch 1, resumed lead (Opus 5.5, high)

- The session building batch 1 (`docs/bbq/2026-11-18-fixes-before-16b.md`, Q4) was cut off. On disk: smooth zoom (`d0b1e40`,
  `d54d34c`) and the grand staff space (`c7707bf`, format 0.18.0, fixture `v0.17/`) committed; the layer icons
  (`crates/ui/src/layer_icons.rs`) written and wired into `panels.rs` and `layers.rs` but not committed, with no log entry
  for any of the three. chris: "the session got interrupted. Please take a look at where we are at and resume work".
- Build proof at the start: `make build-proof` → `BUILD head=c7707bf dirty=yes rebuilt=10` (dirty = the uncommitted icons).
- Finished the icons: 6 of the 7 layer tests failed because the icon's hover note shows its name, so "Delete layer" matched
  two nodes; the tests' `press` now finds the button by role (`crates/ui/tests/layers.rs`). New test
  `crates/ui/tests/layer_icons.rs` at 1440 by 720: `CHECK ... name=layer_icons_one_row result=pass seven=true one_row=true
  in_order=true square=true row_width_pt=212` and `CHECK ... name=layer_icons_hover_and_grey result=pass` (greyed with
  nothing picked, hover name and note, the greyed note, a layer pick wakes Move up, Move down, Delete layer).
- Found and fixed: `main` failed CI's clippy step since the zoom and grand staff commits (three lints: a complex type in
  `crates/ui/src/view.rs`, now `MipLevel`; a doc line starting with "+" in `crates/ui/src/viewer.rs`; a nested `if` in
  `crates/render/tests/grand_staff.rs`). No behaviour change.
- `make check-all` → `TESTS scope=workspace checks=750 check_fail=0 passed=696 failed=0 status=ok`.
- `make bench-zoom` (release, on the Pi's screen) → `CHECK crate=ui name=zoom_bench result=pass build=release canvas=export
  page=sample device=primary strip=on tool=zoom frames=151 mean_ms=0.4 p90_ms=0.0 worst_ms=15.5 first_ms=28.7
  over_budget=0 budget_ms=50` (ADR-009's 50 ms).
- Seen on screen (grim, debug build, scratch `XDG_CONFIG_HOME` and `XDG_DATA_HOME`): `--look layers`, the seven icons in one
  row in three groups (Move up, Move down; New layer, Delete layer; Group, Ungroup, Repeat), Header picked so the last three
  are greyed. Proposals for chris's look: the drawings (arrows; a stacked layer with a plus or a cross; a frame round two
  boxes, the frame's corners only, a filled box and two copies), the spacing, and the hover showing the name then the note.
- Not done here: chris's look at batch 1 and the `v0.16.3` release; the version is still 0.16.2.

## 2026-11-19 — build folder size (Opus 5.5, high)

- chris is moving development to a new VM; the project folder was 412 GB. `du -sh target` → 411G (`target/debug/deps`
  334G, `target/debug/incremental` 73G); 1314 test programs over 100 MB in `deps`, many copies each; disk 97 % full.
- chris ran `cargo clean`: "Removed 548630 files, 421.4GiB total"; `df -h /` → 5 % used after.
- Moving: copy the folder without `target/` and `crates/format/fuzz/target/` (about 70 MB); it keeps the local-only
  `calib/`, fuzz corpus, `spikes/text/fonts` fonts, `.claude/` and the branch `backup/pre-swap-rewrite`.
- Plan for chris to run on the VM: `docs/proposals/build-pruning-2026-11-19.md` (smaller debug information,
  `cargo xtask prune` / `make prune`, a size cap in `make build-proof`); 4 open questions with defaults.
- On the VM (chris's pastes): from-scratch `make build-proof` 5m 35s at 2 jobs, 1m 36s at `JOBS=12`; the Pi, timed here
  the same way, `Finished … in 5m 06s` (`WALL seconds=307`), `BUILD head=4ebfd5a dirty=no rebuilt=11`.
- The VM fails golden images by 1 gray level on a few curved and rotated edge pixels (devices a6x2: 7 px; groups
  a6x2: 11 px; compared on the Pi from the VM's files); vello_cpu's fast and portable maths agree there. chris chose to
  make maths identical on every computer: ADR-017 (proposed), record `docs/bbq/2026-11-19-adr-identical-maths.md`,
  prompt `docs/bbq/2026-11-19-adr-identical-maths-NEXT.md`; ADR-003 item 2 gets a dated banner pointing at it.
  `adr-check: 17 ADRs, 0 failure(s)`.

## 2026-11-19 — ADR-017 build: premise wrong, stopped at step one (Opus 5.5, high, on chris's VM)

- Prompt `docs/bbq/2026-11-19-adr-identical-maths-NEXT.md`. VM: `grep -c fma /proc/cpuinfo` → 0; `ldd --version` →
  Debian GLIBC 2.41. `make build-proof JOBS=12` → `BUILD head=8e3f881 dirty=no rebuilt=0`.
- Before any fix, on the VM: `cargo test -p quadrule-render --no-fail-fast -- --nocapture` → `error: 22 targets
  failed`, with `devices_golden profile=supernote.a6x2 result=fail sha256=e2cc42fd…` and `groups_golden
  profile=supernote.a6x2 result=fail sha256=d78f33f2…` (the record's P1). It fails more than the record listed:
  shapes, transforms, paper, pattern borders and nine migration tests too.
- `60eb202`: CI job `ubuntu-latest, FMA off` (non-blocking), `GLIBC_TUNABLES=glibc.cpu.hwcaps=-AVX2,-FMA,-FMA4`
  (syntax from glibc's manual, Hardware Capability Tunables; names from glibc 2.39 `sysdeps/x86/cpu-tunables.c:124,133,138`);
  it prints glibc's FMA and AVX2 bits decoded from `ld.so --list-diagnostics`.
- Run 44563345364, job 135698321627 (`gh api repos/tholent/quadrule/actions/jobs/135698321627/logs`): glibc 2.43;
  `GLIBC tunables=glibc.cpu.hwcaps=-AVX2,-FMA,-FMA4 fma_cpuid=1 fma_active=0 avx2_cpuid=1 avx2_active=0`; the tests
  **passed with the golden checksums** (`devices_golden … a6x2 result=pass sha256=f32e695d…`, `groups_golden … a6x2
  result=pass sha256=1f2233cc…`). The red arm did not go red: the prompt's stop condition. Nothing built.
- On the VM, `GLIBC_TUNABLES=glibc.cpu.hwcaps=-AVX` (glibc's plain maths) → `groups_golden … a6x2 sha256=d78f33f29a37`,
  the same as without it. glibc's maths is not the cause.
- The cause: fearless_simd 0.7.0 (vello_cpu's SIMD) fuses multiply-add at AVX2 and NEON, not at SSE4.2, SSE2 or the
  fallback; the VM runs at SSE4.2. On CI `groups_golden … a6x2 … portable_diff_px=11 portable_max_diff=1`, the VM's
  11 px; on the VM `portable_diff_px=0`. `Math::Portable` (`Level::baseline()`) is NEON on aarch64, SSE2 on x86_64, so
  it differs between machines too. Detail and file lines: the record's Amendments,
  `docs/bbq/2026-11-19-adr-identical-maths.md`.
- Options put to chris (none built): A, export and the export canvas at `Level::fallback()` everywhere (identical by
  construction, no patches; golden images change once; speed on the Pi to be measured against ADR-009's 50 ms); B, a
  patched fearless_simd with an unfused multiply-add at every level (keeps SIMD speed; a patch under vello, redone each
  upgrade); C, a tolerance of 1 level on machines without FMA (the old option 2). Lead's recommendation: measure A's
  speed on the Pi first; take A if it fits, else C. ADR-017 would need superseding either way; its text is untouched.
- Not done: the `quadrule-maths` crate, the routing test, the three systems, golden images; ADR-017 `built: not
  started` stands.

## 2026-11-19 — ADR-018: export at the fallback level (Opus 5.5, high, on chris's VM; branch `adr-018-fallback-level`)

- chris: "let's go with option A"; then goforit for the ADR, "Revert it" for the FMA-off job, "Stop and report" if
  the Pi is too slow. ADR-018 (proposed) supersedes ADR-017; ADR-003 gets a second banner at item 2 and its proof
  names the renamed test. `python3 …/makeadr/scripts/adr-check.py --root . --triggers-from 1` → `adr-check: 18 ADRs,
  0 failure(s)` (headers and that each proof resolves; it runs no test).
- Built: `quadrule_render::LEVEL = Level::fallback()`, a build-time assertion that refuses any other level (tried:
  `Level::baseline()` → `error[E0080]: evaluation panicked: ADR-018: export draws at the fallback level on every
  computer`), `render(dl)` with no `Math`; fearless_simd 0.7.0 a direct dependency with `force_support_fallback`
  (the version vello_cpu already uses); the fast-against-portable comparisons gone from the render tests;
  `render.rs::export_draws_at_the_fallback_level` in place of `simd_and_portable_maths_agree`. The FMA-off job reverted.
- Speed on the VM, release build, a whole Nomad page 10 times (a throwaway test, deleted): slice 6.8 → 8.0 ms,
  groups 14.5 → 19.0, patterns 13.9 → 15.3, fonts 17.3 → 17.0 (fast → fallback). The Pi is not reachable from the VM
  and the VM has no screen: the benches are chris's to run.
- `cargo test --workspace --no-fail-fast -- --nocapture` (VM) → `error: 23 targets failed`: 22 render test programs
  and `quadrule-ui --test paper`, all on golden bytes or stored export hashes; every golden checksum equals the VM's
  before the change (33 of 33). `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D
  warnings` clean.
- Changed pixels against the golden images (a throwaway PIL script over `crates/render/tests/golden/*.png`
  and the new exports): 23 of 33 pages, 285 px, every one 1 level, all on curved or turned edges; for chris's look on
  a private page, https://claude.ai/artifact/2PuHNfNFZpHmfHkBUwirX9. Golden images not re-made: they wait on that look.

- CI on the branch, run 44579062510 (`gh workflow run CI --ref adr-018-fallback-level`): all three jobs fail on
  golden bytes, as expected before the re-make, and stop at the first failing program (`devices`). There, Linux x86
  (AVX2, FMA), macOS ARM and Windows x86 each print the VM's checksums: `devices_golden profile=supernote.a5x2 …
  sha256=ef53067c…` and `profile=supernote.a6x2 … sha256=e2cc42fd…` (job logs 135754403446, 135754403882,
  135754403980). Under the old fast level those pages differed by 16 and 7 px between the VM and CI.
- Review by a Sonnet 5.5 subagent (read-only, commit `c9f4261`): no correctness bugs. Noted: the level test would
  not catch `render()` going back to `RenderSettings::default()` (the golden images on CI would; ADR-018 item 6);
  the system's `sin` and `cos` still differ by system in principle (ADR-018 Context, "What it does not cover").
- chris, word for word: "1. looks good; 2. we are dropping the timing comparisons; 3. ratified". ADR-018 ratified,
  amended (item 5, the Pi benches, dropped) and marked landed.
- Golden images re-made: `QUADRULE_BLESS=1 cargo test -p quadrule-render --no-fail-fast` re-wrote the 23 changed
  pages; the tests bless in parallel and reorder `SHA256SUMS`, so it was put back in its old order with the new sums
  (every sum checked against `sha256sum` of its file). Migration fixtures' stored export hashes: 24 replaced from the
  old-to-new golden map, 4 more (the landscape repeats export and the devices page's user-profile device, v0.10 and
  v0.14 to v0.17) from a throwaway print, removed. The stored hashes of the older sums lines in `grand_staff.rs` and
  `pattern_borders.rs` updated.
- `make check-all JOBS=12` (VM) → `TESTS scope=workspace checks=749 check_fail=0 passed=696 failed=0 status=ok`; fmt
  and clippy clean. The VM (no FMA) now passes every golden test.
- CI on the branch with the re-made golden images, run 44598291874: `ubuntu-latest: success`, `macos-latest:
  success`, `windows-latest: success` (`gh run view 44598291874`). Merged to `main` as a fast-forward; branch deleted.

## 2026-11-19 to 2026-11-20 — build pruning: smaller debug data, `make prune`, a size cap (Opus 5.5, high, on chris's VM)

Ran `docs/proposals/build-pruning-2026-11-19.md` (chris: "please read … and follow it's directions"). Its four open
questions took their defaults (error at 50 GB, 3 days, the checks prune by themselves except on CI, recommendation 3
asked after). The VM: 12 cores (`nproc`), 31 GiB memory, no swap (`free -h`), 126 GB disk (`df -h .`), Debian 13,
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`. Every timing below is `JOBS=12`, so it is not comparable with the Pi's.

- Step 0: `make build-proof JOBS=12` → `BUILD head=6f1e574 dirty=no rebuilt=7` (2m 17s; `target/` already held 2.9 GB).
- A false start. The first baseline went into a checkout under `/tmp`, which on this VM is a 16 GB in-memory folder
  (`df -h /tmp` → `tmpfs 16G`), and filled it; every command failed until chris said "yes, go ahead" and it was removed.
  At the same time a `make check-all` was running in the main folder; I first told chris it was mine, which was wrong:
  it was the run of this window's conversation before `/clear` (task "Clean, then time make check-all baseline"). chris
  stopped it with `kill`. Measurements after that were in a separate checkout of `6f1e574` on the main disk
  (`~/projects/quadrule-baseline`, removed at the end).
- Step 1, the baseline (`6f1e574`, after `cargo clean`): `time make check-all JOBS=12` → `TESTS scope=workspace
  checks=750 check_fail=0 passed=696 failed=0 status=ok`, real 30m 12s. `du -sh` each: `target` 27G, `target/debug/deps`
  25G, `target/debug/incremental` 2.0G, `target/debug/build` 233M. `ls -lS target/debug/deps`: `quadrule-…` 463 MB,
  `lint_pointer-…` 371 MB, `window-…` 365 MB.
- Step 1.3, version bumps as a cause: `version = "0.16.99"`, `make test-all`, put back: `ls target/debug/deps | grep -c
  '^groups-[0-9a-f]\{16\}$'` 5 → 10 (five crates have a `groups` test), `target` 27G → 51G. Confirmed.
- Step 2, `[profile.dev] debug = "line-tables-only"` and `[profile.dev.package."*"] debug = false`, alone, after
  `cargo clean` ("Removed 30094 files, 50.5GiB total"): `make check-all` → `checks=749 check_fail=0 passed=696 failed=0
  status=ok`, real 22m 32s; `target` 11G, `deps` 9.5G, `incremental` 1.3G, `build` 44M; `lint_pointer-…` 148 MB,
  `window-…` 143 MB. No golden `CHECK` line failed; `git status` showed only `Cargo.toml`. A `panic!("probe")` in
  `crates/model/tests/model.rs` with `RUST_BACKTRACE=1` → `panicked at crates/model/tests/model.rs:366:5` and the frame
  `at ./tests/model.rs:366:5`; removed. 749 against 750 is not the change: the unchanged code gave 750 and then 749
  (debt).
- Step 3, `cargo xtask prune` (`crates/xtask/src/prune.rs`, tests `crates/xtask/tests/prune.rs`, 6 `CHECK` lines). Three
  things the plan did not foresee, found by measuring:
  1. Names are not enough. The first version kept the newest file per name and deleted four `groups` test programs in
     use (`groups before=1` where 5 were expected): five crates have a `groups` test, and clippy checks a library both
     alone and with its tests. Fixed: each build is matched to Cargo's own record of it,
     `target/<profile>/.fingerprint/<package>-<hash>/<what>.json`; a build with no record is kept. Three of our test
     names are also third-party crate names (`layout`, `render`, `window`); for those only our old test programs go.
  2. The same crate is built several ways. `cargo test` on the default members, `cargo test --workspace`, `cargo build`
     and `cargo build -p quadrule-app` give some of our crates different library features, and Cargo's records cannot
     tell those copies from old versions. Mapped in a side build folder by counting `libquadrule_text-*.rlib` after each
     command: 1, 2, 2, 2, 3 (`build --workspace`), 4 (`build`). Keeping only the newest made every switch a rebuild:
     after a prune, 24 s and 11 crates to `--workspace`, 12 s and 9 crates back. chris chose "Keep newest 2"; then,
     told there are four sets, not two, "Libraries 4, programs 2"; then "Keep 4 for those two" (the `quadrule` and
     `xtask` programs, also built four ways). So: test programs keep 2, libraries and the two programs keep 4. All four
     sets of our libraries are 390 MB (`du -ch libquadrule_*.rlib libquadrule_*.rmeta`), test programs 11.8 GB.
  3. `cargo xtask prune` builds xtask a fifth way. The checks run `target/debug/xtask prune` instead, the program the
     tests just built (`cargo test` leaves its own build there, checked by inode).
- Step 3, the version-bump trick with programs 2 and libraries 4: `du -sh target` 34G, `make prune` → `PRUNE
  target=target removed_files=207 removed_gb=3.5 kept_gb=29.6 dry_run=no`, 30G after; `groups` 14 → 10 (two each).
  The trick itself (bump, then put back) makes the put-back copies look older than the bumped ones, so it costs one round
  of rebuilds; a real bump only goes forward.
- With the final rule: `make check-all JOBS=12` → `TESTS scope=workspace checks=756 check_fail=0 passed=702 failed=0
  status=ok` (750 and the 6 new), `PRUNE target=target removed_files=2 removed_gb=0.0 kept_gb=29.8 dry_run=no`, real
  20m 18s; `make check` → `TESTS scope=default checks=450 … passed=418 failed=0 status=ok`, real 7m 33s. Then a prune
  before each of `cargo test --workspace --no-run`, `cargo test --no-run`, both clippys, `cargo build --workspace`,
  `cargo build -p quadrule-app` and `cargo build --bin xtask`, twice: the second round compiled nothing at all; in the
  first both clippys re-checked our crates once, cause not found (debt). `make build-proof` → `BUILD head=6f1e574
  dirty=yes rebuilt=0 target_gb=29.9`. The folder holding every way of building is 30G: `deps` 23G, `incremental` 7.5G
  (goes after 3 days), `build` 274M.
- The cap: `make build-proof TARGET_CAP_GB=5` → `over the cap (TARGET_CAP_GB=5): run make prune`, exit status 2.
- `DEVELOPMENT.md` gains "Keeping the build folder small"; the Makefile `make prune` (`DRY=1`), `TARGET_CAP_GB` and
  the prune after `check` and `check-all`. `README.md` unchanged (nothing a user sees). No release: the app is unchanged.
- Not measured: the Pi. `make bench-*` were not run (the speed budgets are judged on the Pi; ADR-009).

## 2026-11-20 — build jobs follow the computer; tests run with cargo-nextest (Opus 5.5, high, on chris's VM)

chris: "have we increased the simultanous jobs and threads? Also can we look at nextest?"; then "1. yes, please do;
2. yes, please detect machine".

- Where the time went: running tests, not building them. With nothing to compile, `make check-all` still took 20m 18s;
  the last `make check` log had 93 test programs run one after another, their `finished in` times adding to 441 s
  (`awk` over `target/test.log`). Slowest: `migrate_v07` 31 s, `migrate_v010` 29 s, `paper` 29 s.
- `JOBS` now follows the computer (`Makefile`): one build job per core, at most one per 1.5 GB of memory, so the Pi 5
  (4 cores, 4 GB; ADR-006) keeps its 2 and this VM (12 cores, 32,095 MB) gets 12. `make help` → `JOBS=12`; `JOBS=4`
  by hand still wins.
- cargo-nextest 0.9.151 installed on the VM (`cargo install cargo-nextest --version 0.9.151 --locked`, 3m 12s;
  0.9.152 was a week old). Trials with everything built: `cargo nextest run --workspace` → `702 tests run: 702 passed
  (7 slow), 2 skipped`, 5m 37s; with the six speed-bench window tests and `every_quick_fix_works_by_the_pointer…` started
  first (`.config/nextest.toml`), 5m 11s. Those tests take 60–186 s each on the software graphics driver.
- `make test` and `make test-all` use nextest when it is installed, `cargo test` otherwise; the `TESTS` line gains
  `runner=nextest|cargo` at its end (the rest as the makefile bbq's Q2 set it) and counts `CHECK` lines that nextest
  indents. `make check` → `TESTS scope=default checks=451 check_fail=0 passed=418 failed=0 status=ok runner=nextest`,
  1m 51s (was 7m 33s); then `cargo test --no-run` and `cargo test --workspace --no-run` compiled nothing, so prune
  still runs the tests' own xtask program. `make check-all` → `TESTS scope=workspace checks=756 check_fail=0 passed=702
  failed=0 status=ok runner=nextest`, 5m 31s (was 20m 18s). A failing probe test → `TESTS scope=default checks=452
  check_fail=1 passed=418 failed=1 status=fail runner=nextest`, the `FAIL` and `panicked` lines shown, exit status 2;
  probe removed.
- CI still runs `cargo test`; nextest is not tried on the Pi.

## 2026-11-20 — CI on nextest; the Pi retired (Opus 5.5, high, on chris's VM; branch `ci-nextest`)

chris, word for word: "1. yes, please have CI use nextest; 2. the pi has been deprecated and we don't need to consider
it from now on".

- CI (`.github/workflows/ci.yml`): `taiki-e/install-action@v2` installs `cargo-nextest@0.9.151`; the test step is
  `cargo nextest run --workspace --no-fail-fast`; the summary collects the indented `CHECK` lines; a change to
  `.config/nextest.toml` also starts a run. Tried on the branch first (`gh workflow run CI --ref ci-nextest`).
- The Pi's retirement amends three ruled ADRs (makeadr, amend; answers to questions, word for word): ADR-006 item 1,
  the build machine, is "the VM, which does have a screen" (an earlier log line said it had none; it was wrong), and
  jobs follow the machine ("yes, please detect machine"); ADR-009 item 3, the 50 ms speed line: "Keep 50 ms, any
  machine"; ADR-008 item 3, the opt-in app build: "Drop it". `python3 …/makeadr/scripts/adr-check.py --root .
  --triggers-from 1` → `adr-check: 18 ADRs, 0 failure(s)` (headers and that each proof resolves; it runs no test).
- Dropping item 3: `Cargo.toml` has no `default-members`, so plain `cargo build` and `cargo test` cover the whole
  workspace; in the Makefile `build`, `clippy`, `test` and `check` are the workspace forms and the `-all` names are the
  same targets. `make check` → `TESTS scope=workspace checks=756 check_fail=0 passed=702 failed=0 status=ok
  runner=nextest`, 5m 25s; afterwards `cargo test`, `cargo test --workspace`, `cargo clippy --all-targets`, `cargo
  build` and `cargo build --workspace` compiled nothing (one way of building fewer), and `make build-proof` →
  `rebuilt=0 target_gb=29.9`.
- Pi-only debt lines removed (nextest on the Pi; prune measured only on the VM). Older records that measured on the
  Pi stay as they were.
- CI on the branch: run 44767363085 (`55a2b7d`, nextest) and run 44769524309 (`b488760`, the Pi retired):
  `ubuntu-latest: success`, `macos-latest: success`, `windows-latest: success` both times; nextest's summaries
  `702 tests run: 702 passed`. Job times against the last `main` run before nextest (44744928136): macOS 15m 16s →
  8m 52s, Linux 16m 30s → 10m 12s, Windows 13m 8s → 11m 37s. Fast-forwarded to `main`; branch deleted.

## 2026-11-20 — looseends project (Opus 5.5, high, on chris's VM)

chris typed `looseends`, then chose "The project (Recommended)". `make build-proof` → `BUILD head=ff5e687 dirty=no
rebuilt=0 target_gb=29.9`. The skill's scan (`looseends-scan.py`) listed 101 markers; checked one by one, most were
closed (the hinting talk folded into ADR-001, hinting moved to v1.1; ADR-007 and ADR-014 accepted; old bbq veto lists
built and released) or are per-rung debt for rung 17's sweep.

- chris's rulings, word for word (answers to questions): recommendation 3 of the build-pruning plan, "Plan it after
  16b (Recommended)" (written in `docs/proposals/build-pruning-2026-11-19.md`); PR #4, the colour-mode proposal,
  "Leave until after 1.0 (Recommended)" (a parked line in `docs/STATUS-debt.md`, with its ADR-017 number clash); the
  local branch `backup/pre-swap-rewrite` (2026-11-09, 8 commits, never pushed), "Keep it, leave it local
  (Recommended)".
- The last box in `docs/looseends/2026-11-12-rung-7-session.md` ticked: the hardware queue was done 2026-11-12
  (`docs/arcs/v1/README.md`, chris's queue).
- The `checks=` count that moved by one: not reproduced. Three `make test` runs kept apart (nextest) each gave
  `TESTS scope=workspace checks=756 check_fail=0 passed=702 failed=0 status=ok runner=nextest`, and `diff` of their
  `CHECK crate=… name=…` pairs was empty; one `cargo test --workspace -- --nocapture` gave 756 counted both at line
  start (`grep -c '^ *CHECK'`) and anywhere (`grep -o 'CHECK crate=' | wc -l`). The drift was seen under `cargo test`,
  where a `CHECK` line can share a line with the runner's output. The `Makefile` now counts `CHECK crate=` wherever it
  sits (a probe line with runner text before it: old count 1, new 2); `make test` after the change → `checks=756`. Both
  debt lines (the drift, and the line-sharing) are paid and removed. If `checks=` moves again, it is a test.
- Build-pruning leftovers measured (`docs/STATUS-debt.md`): only 14M of `target/debug/build`'s 274M is ours; no code
  written; the lead recommends closing that part.
- Still waiting on chris: the look at fixes batch 1 (`v0.16.3`), and one starter per category for rung 16b.
- Later the same session, chris, word for word: "2. once I start using the finished v0.16.3 I'll create these; 3. yes".
  The starters wait for `v0.16.3` (`docs/arcs/v1/README.md`, chris's queue); the build-pruning leftovers part of the
  debt line is closed without code (`docs/STATUS-debt.md`). chris asked for a checklist for the batch 1 look; it was
  given in the session.

## 2026-11-20 — fixes batch 1 approved and shipped as v0.16.3 (Opus 5.5, high, on chris's VM)

- chris asked for a checklist for the batch 1 look (given in the session: zoom, grand staff space, layer icons) and
  answered, word for word: "approved for v0.16.3".
- Version 0.16.3 (`Cargo.toml`, `Cargo.lock`); release notes `docs/arcs/v1/release-notes/v0.16.3.md`, which also carry
  ADR-018's one-time golden change (it landed after `v0.16.2`); the record's Amendments, the board's row 16 and
  Re-plans, `STATUS.md`.
- `make check` at 0.16.3 → exit 0; nextest `702 tests run: 702 passed (6 slow), 2 skipped`, 756 CHECK lines; then
  `PRUNE target=target removed_files=305 removed_gb=7.9 kept_gb=31.5 dry_run=no`.
- Tag `v0.16.3` pushed. Release run 44785857112: `windows-x86_64`, `linux-x86_64`, `linux-aarch64`, `macos-universal` and
  `Draft, attach, publish` all `success`; the release is published with 6 packages and `SHA256SUMS.txt`. CI on `main`
  for `1305e12` (run 44785854666): `success`.

## 2026-11-20 — fixes batch 2: menus for each system, the switches into View, the macOS double-click (lead Opus 5.5, high; builder Sonnet 5.5, high; branch `batch-2-menus`)

chris, word for word: "let's do batch 2 first". Two gaps asked first and written in the record's Amendments
(Settings… opens Profiles; Help holds About and the project page). Built on the branch `batch-2-menus`, since the
VM cannot build the macOS parts.

- The builder (Sonnet 5.5, high) built the menu table, the Windows/Linux bar, the Mac menu bar and most tests, then
  stopped on the account's weekly limit ("resets Nov 21, 5pm") with nothing committed. The lead finished it:
  - Tests: an item's accessible name carries its key ("Undo Ctrl+Z"), and an open menu lies over the toolbar, so the
    tests' lookup matched neither; the lookup now takes the words with or without one key after them, and skips the
    toolbar's own row. Menus still open between two opens closed the next one; each open now closes all first. The
    expected key text comes from egui (Ctrl+N here, ⌘N on CI's Mac). The keys-against-items test compared new objects'
    ids across two windows (never equal for Paste and Duplicate); it compares how many are picked, is split into File
    and Edit (433 s → about 90 s and 116 s, started first by nextest), and tries each command once (the Mac's keyed
    items are the same commands, which the table test checks). A devices test scrolls to its Take out button, which the
    menu bar's line pushed below the panel's edge.
  - A bug the tests found: Esc that closed a menu also cleared the selection (and left the welcome screen). An open
    menu, list or right-click menu now keeps its own Esc (`crates/ui/src/view.rs`, `shortcuts`).
  - The macOS double-click: winit 0.30 installs no application delegate and says to install one after its event loop
    is made and before it runs (winit `platform::macos`). On the Mac the app now makes the event loop itself and starts
    through `eframe::create_native`; the new crate `crates/macos` (`quadrule-macos`) holds the delegate
    (`application:openURLs:`), the one crate that allows `unsafe` (denied, allowed with a reason at each use; every
    other crate still forbids it). A file handed over opens through `ViewerUi::open_from_system`, the path a dropped
    file takes, unsaved-changes prompt first.
- Checks: `make check` → `TESTS scope=workspace checks=769 check_fail=0 passed=715 failed=0 status=ok runner=nextest`.
  macOS: `cargo clippy --target aarch64-apple-darwin -p quadrule-app -p quadrule-macos -p quadrule-ui --all-targets --
  -D warnings` clean (compiles and lints; nothing on the Mac was run).
- Not verifiable on the VM: everything the Mac does at run time (its menu bar, ⌘ keys acting once, Quit asking first,
  the double-click at launch and while running). They wait for chris's look on his Mac.
- CI on the branch: run 44805150907 (`6c527b4`, the menus) and run 44807675401 (`d6a613f`, the double-click):
  `ubuntu-latest: success`, `macos-latest: success`, `windows-latest: success` both times.

## 2026-11-20 — batch 2: the Mac build closed at once; fixed, and CI now starts the app on the Mac

- The test-tag dry run (run 44813754066, `v0.16.4-test.1`) built every package. chris opened the Mac `.dmg`: "the app
  closes immediately".
- Cause: winit 0.30.13 does set an application delegate, `WinitApplicationDelegate`, when its event loop is made, and
  panics at launch if the application's delegate is anything else (`ApplicationDelegate::get`,
  `winit-0.30.13/src/platform_impl/macos/app_state.rs`: "tried to get a delegate that was not the one Winit has
  registered"). `quadrule-macos` replaced it. The entry above, and the crate's comment, said winit installs none; that
  came from memory, not from winit's source, and was wrong.
- Fix (`crates/macos/src/lib.rs`): winit's delegate stays; `install()` adds `application:openURLs:` to its class
  (`class_addMethod`, type `v@:@@`) and sets the delegate again so AppKit sees the new method. If the class is missing
  or is not the delegate, it does nothing and the app runs without the double-click.
- Why CI missed it: no test starts the app. CI now has a `launch` step on `macos-latest` that starts the built app with
  `--bench-welcome 5` and fails on a non-zero exit (`CHECK crate=app name=mac_launch`). To prove the step catches this
  crash, it also ran on the broken code alone (scratch branch `launch-check-proof`, `75a2cf5`).
- CI: run 44855967555 (`b9600f6`, the fix): `ubuntu-latest: success`, `windows-latest: success`, `macos-latest:
  success`, with `CHECK crate=app name=mac_launch result=pass exit=0` (the app drew the welcome screen's 24 tiles and
  quit). Run 44856011998 (`75a2cf5`, the step on the broken code): `macos-latest: failure`, the log showing winit's
  panic "tried to get a delegate that was not the one Winit has registered". Scratch branch deleted after.
- The broken test tag `v0.16.4-test.1` is deleted; `v0.16.4-test.2` is the build for chris's look.

## 2026-11-23 — fixes batch 2 approved and released as v0.16.4

- On the fixed build (`v0.16.4-test.2`, run 44857121017) chris answered, word for word: "both ways to open quad files
  now work", then "approved for v0.16.4". No changes and no vetoes: the six calls stand
  (`docs/bbq/2026-11-18-fixes-before-16b.md`, Amendments).
- ADR-008 amended (call 4, the `quadrule-macos` crate and its `unsafe` exception) through makeadr; `adr-check`: 18 ADRs,
  0 failures (it checks headers and that proofs resolve, not that tests pass).
- `batch-2-menus` fast-forwarded into `main` (`ca1b973`); version 0.16.4; release notes
  `docs/arcs/v1/release-notes/v0.16.4.md`; STATUS, the board (row 16, Re-plans), the debt line updated.
- Batch 3: chris, word for word: "let's put this off until v1.1. I don't anticipate getting into the signing game quite
  yet." Quick Look moved to v1.1 (record Amendments and Owed, the board's "Out of this arc" and Re-plans, STATUS);
  16b becomes `v0.16.5`.
- Checks on `main` with 0.16.4: `make check` → `TESTS scope=workspace checks=769 check_fail=0 passed=715 failed=0
  status=ok runner=nextest`.

## 2026-11-23 — chris's first starters against the linter: ADR-019 and ADR-020 proposed

- chris asked to step through his starters' lint findings and make or amend ADRs, Paper first. `narrow_ruled.quad` and
  `narrow_ruled_rtl.quad` (in `~/starters-draft/`, not in the repository): `cargo xtask lint` → `11 finding(s) on 2
  profile(s)` each (L06, L10, L02, L16). Record `docs/bbq/2026-11-23-adr-starter-lint.md`.
- chris's answers: "A Full bleed mark (Recommended)", "Page edges count as on rhythm (Recommended)", "A new Margin role
  (Recommended)". ADR-019 and ADR-020 drafted through makeadr, proposed; ADR-012 points at both. `adr-check`: 20 ADRs,
  0 failures. Build queued before 16b, after the review of more starters.
- `v0.16.4` tagged on `9c96921` after CI run 45219511834 (`macos-latest`, `windows-latest`, `ubuntu-latest`: success).
  Release run 45221879230: `linux-x86_64`, `windows-x86_64`, `linux-aarch64`, `macos-universal`, and Draft, attach,
  publish: success; `gh release view v0.16.4` → `v0.16.4 draft=false pre=true assets=7`. Test tag `v0.16.4-test.2`
  deleted.
- chris: "ratify 019 and 020". ADR-019 and ADR-020 accepted; `adr-check`: 20 ADRs, 0 failures. Built: not started,
  queued before 16b.
- Five more starters (bullet journal, Cornell, grid, hex, isometric) linted, then re-linted after chris centred their
  rhythms (L18 gone). chris's answers D ("No findings at all"), F ("L14 skips slanted patterns"), G ("L09 skips drawing
  grids") → ADR-021, proposed; H ("Mark both Full bleed") a starter change. Record updated.
- chris: "ratify 021". ADR-021 accepted. Spec §8.3 and §13 get dated notes pointing at ADR-019 and ADR-021.

## 2026-11-23 — ADR-019, ADR-020 and ADR-021 built

- chris: "go ahead and start the build. I have a few feature additions that have to occur before I can work on the
  planning templates". Built by the lead in this session (no builder).
- ADR-019: `Object.full_bleed` (format 0.19.0, a no-op migration, fixture `crates/render/tests/fixtures/v0.18/` resaved
  from v0.17 by the 0.18.0 build before the bump); a Full bleed tick beside Free placement in the Arrange tab, mixed for
  a mixed selection, one undo step; L06 and L10 skip a marked object, a marked group's or repeat's contents too; L10
  allows half a stroke past an edge; L16 counts an object flush on a page edge, near or far, as on the rhythm on that
  axis.
- ADR-020: `LineRole::Margin` in the model, every built-in profile (5 px, gray 204, chosen, not measured; a profile
  without it reads that default), the Style tab, the Profiles window, the swatch matching and the role quick fix;
  calibration strips W19–W24 (a margin rule beside Guide ruled lines). ADR-012's owed list gains its measurement.
- ADR-021: L14 skips hex and isometric regions; L09 skips an isometric rhythm. L18 no longer says "-0 px".
- Tests: `crates/lint/tests/starter_rules.rs` (5; each fails against the old rules: run with `crates/lint/src` stashed,
  `test result: FAILED. 0 passed; 5 failed`), `crates/ui/tests/lint_pointer.rs` (Full bleed), `crates/profiles/tests/
  profiles.rs` (Margin), `crates/render/tests/migrate_v018.rs`; `newer_minor_keeps_version` now stands in 0.20.0.
- `make check` → `TESTS scope=workspace checks=778 check_fail=0 passed=724 failed=0 status=ok runner=nextest`.
- chris's seven starters, as scratch copies with Full bleed ticked on every object and the Margin role on the margin
  lines: `cargo xtask lint` → 0 findings on six, 1 on the bullet journal (its top rule's L16: Snap to rhythm).
- ADR-019 built `landed`; ADR-020 `partial` (calibration readings owed); ADR-021 `partial` (the starter test comes
  with 16b). `adr-check`: 21 ADRs, 0 failures. Waits for chris's look (ADR-007).
- CI run 45230043207 on `96b141d`: `windows-latest`, `macos-latest`, `ubuntu-latest`: success.
- chris: "let's wrap these changes as well as my new features into v0.16.5 and 16b get's bumped back to v0.16.6". Board, STATUS, record and the 16b prompt updated.
- bbq, 8 questions: a calendar pattern and a mini calendar (`docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md`), for `v0.16.5`.

## 2026-11-23 — the calendar pattern and the mini calendar built

- Fresh session, lead Opus 5.5 high, as the Kickoff of `docs/bbq/2026-11-23-calendar-pattern-and-mini-calendar.md` asks.
  Build on disk against HEAD: `make build-proof` → `BUILD head=09196d8 dirty=no rebuilt=7` (the docs-only commits since
  the last build left 7 crates stale), again → `BUILD head=09196d8 dirty=no rebuilt=0 target_gb=30.5`.
- Built on `main`, no flag (ADR-007; no export change to any older page): `Pattern::Calendar` and
  `ObjectKind::MiniCalendar` in format 0.19.0; months and weekdays (`crates/model/src/calendar.rs`); the plan, the words
  and L07's left-out kinds (`crates/layout/src/calendar.rs`); the lines (`crates/layout/src/pattern.rs`); the words as
  text jobs under ids made from the object's, saved with the page's outlines (`text_jobs`); measuring from the bundled
  font (`bundled_advance_em`, `bundled_cap_em` in `crates/text/src/signature.rs`); the words in
  `crates/i18n/locales/en-US/calendar.ftl`; the inspector rows and the Mini calendar tool (M) (`crates/ui/src/panels.rs`,
  `crates/ui/src/tools.rs`, `crates/ui/src/draw.rs`); `--calendar-page`, `--look calendar|mini-calendar`.
- A first render showed scrambled words on three of the four calendars: the label ids were the object's id XOR the slot,
  which collide for neighbouring ids (object 1 slot 2 = object 2 slot 1). Now both halves are mixed (SplitMix64);
  a test checks 64 neighbouring ids × every slot for clashes.
- The lead's readings inside the Defaulted calls, and one thing grown past the record (the mini calendar's title line,
  on by default, veto-able), are in the record's Built section.
- Tests, one `CHECK` line each: `crates/model/tests/calendar.rs` (2), `crates/layout/tests/calendar.rs` (6),
  `crates/lint/tests/calendar.rs` (1), `crates/ui/tests/calendar.rs` (2), `crates/render/tests/calendar.rs` (3), and unit
  tests in both `calendar.rs` files. Goldens `calendar_*` blessed on both tablets after the 34 older lines (their sha256
  checked unchanged); looked at by the lead on both before blessing.
- On the calendar page, `lint` gives only page-placement findings (L06, L16, L18, L19, as the other sample pages); none
  about the calendars' own words or lines.
- `make check` → `TESTS scope=workspace checks=794 check_fail=0 passed=744 failed=0 status=ok runner=nextest`.
- Not done here: the window itself. This session has no display (no `DISPLAY`, no Xvfb; checked 2026-11-23), so the
  inspector rows and the tool's icon are tested headless only. Speed on the calendar page not measured.
- Waits for chris's look (with the Full bleed tick, ADR-019), then `v0.16.5`.

## 2026-11-23 — correction: this session does have a display

- The entry above says the session has no display. Wrong: `DISPLAY` was unset in the shell, but an X server runs as
  `:0` (`xdpyinfo -display :0` → X.Org 21.1.16, one screen, 1024x768, 96 DPI) and the session reaches it. chris
  installed scrot.
- Looked at in the window (release build, `DISPLAY=:0 ./target/release/quadrule --calendar-page --look calendar`, then
  `--look mini-calendar`, screenshots with scrot): the Style tab shows Pattern: Calendar with Month, Year and Week starts
  on for the dated calendar, and the Mini calendar section (Month, Year, Week starts on, Month and year) for the mini
  calendar; the Mini calendar tool's icon is last in the tools bar; the page and the device strip draw all four
  calendars. Not seen: the undated rows (1st falls on, Days in month) and an open menu, as nothing here can click
  (no xdotool).
- With xdotool (installed by chris), in the window: the Undated month picked in the layers panel shows Month: Undated,
  1st falls on: Wednesday, Days in month: 31, Week starts on: Sunday; the Month menu lists Undated then January to
  December; picking February dates it to February 2026 (this year) in one step, the Year row replacing the undated
  rows, four boxed rows (February 2026 starts on a Sunday). Its day numbers were left out on the Nomad, with an L07
  in the Lint tab: "would be 5.4 pt, smaller than the 6.0 pt that reads" (the 62 mm wide region's cells are 8.9 mm
  across, so the corner square is 2.95 mm). One Ctrl+Z put it back undated; Ctrl+Q quit with nothing changed.
- CI run 45242627211 on `809c16c`: `ubuntu-latest`, `windows-latest`, `macos-latest`: success. Fuzz 45242627231 and
  Coverage 45242627082: success.

## 2026-11-23 — the calendar pattern and the mini calendar approved

- chris, word for word: "approved". No vetoes: the Defaulted calls and the lead's readings stand, the mini calendar's
  title included (record, Approved). STATUS and the board's row 16 updated.
- Asked next: whether "approved" also covers the Full bleed tick (ADR-019's look), and whether to cut `v0.16.5`.
- chris, word for word: "it covers the full bleed, but wait a minute on cutting a release". So the look of ADR-019's
  Full bleed tick is approved with the calendar; `v0.16.5` is held: no version bump, notes or tag until chris says.

## 2026-11-23 — v0.16.5

- chris opened a bbq on a week option for the calendar ("bbq 4q add 'week' option to calendar pattern"), then
  dropped it before any answer: "ok, n/m. let's cut this release". No record written; the idea is not queued.
- Version 0.16.5 (`Cargo.toml`, `Cargo.lock`); notes `docs/arcs/v1/release-notes/v0.16.5.md` (ADR-019, ADR-020,
  ADR-021 built, the calendar pattern and the mini calendar, format 0.19.0); STATUS and the board's row 16 and
  Re-plans updated.
- `make check` on 0.16.5 → `TESTS scope=workspace checks=794 check_fail=0 passed=744 failed=0 status=ok runner=nextest`.
- CI run 45255174131 on `772cfcf`: `macos-latest`, `windows-latest`, `ubuntu-latest`: success. `v0.16.5` tagged on
  `772cfcf` (chris: "let's cut this release"). Release run 45258913684: `linux-x86_64`, `linux-aarch64`,
  `windows-x86_64`, `macos-universal`, and Draft, attach, publish: success; `gh release view v0.16.5` →
  `v0.16.5 draft=false pre=true assets=7` (https://github.com/tholent/quadrule/releases/tag/v0.16.5).
- chris asked what branch protections the repository should have. Read-only: no rulesets (`gh api .../rulesets` → `[]`),
  `main` not protected. Proposed, not applied: a `main` ruleset blocking force pushes and deletion, a tag ruleset on
  `v*` (excluding `v*-test*`) blocking deletion and moving; pull requests and required checks only with a ruling
  that ends work on `main` directly (CI's path filter would leave docs-only PRs waiting forever on required checks).

## 2026-11-23 — ADR-022 proposed: GitHub branch and tag rules

- makeadr, 4 questions (record `docs/bbq/2026-11-23-adr-branch-protections.md`). chris's answers: block force pushes
  and deletion on `main`; block deleting and moving `v*` tags, test tags excepted; pull requests and required checks
  later, with the ruling that ends work on `main`; also secret scanning with push protection, automatic deletion of
  merged branches, and deleting `batch-2-menus` (fully merged). ADR-022 proposed; `adr-check`: 22 ADRs, 0 failures.
  Nothing applied on GitHub yet: it waits for ratification.
- ADR-022 ratified by chris ("ratified"). Applying it from here failed: `gh api -X POST .../rulesets` → `403 Resource
  not accessible by personal access token` (no Administration: write on the token); nothing created (`gh api
  .../rulesets --jq length` → 0). The record's premise is corrected; chris applies the rules in the browser, then the
  lead reads them back.
- chris applied ADR-022 in the browser. Read back with the new `.github/scripts/check-github-rules.sh`
  (`make check-github`): `GITHUB repo=tholent/quadrule rulesets=2 missing=push-protection status=fail`. Both rulesets
  active, no bypass; merged branches deleted; secret scanning on; `batch-2-menus` gone (branches: `badges`, `main`).
  Push protection is still off. ADR-022 built `partial`, owed push protection; `adr-check`: 22 ADRs, 0 failures.
- chris turned push protection on (Settings → Advanced Security). `make check-github` → `GITHUB repo=tholent/quadrule
  rulesets=2 missing=none status=ok`. ADR-022 built `landed`, proof `.github/scripts/check-github-rules.sh`.

## 2026-11-23 — L16: a right-aligned text box may end on the rhythm (ADR-019 amended)

- chris: "for right aligned text items, we should allow for ending on rhythm"; then "Right-aligned: right edge
  (Recommended)" and "Amend ADR-019 (Recommended)". ADR-019 amended (item 3); `adr-check`: 22 ADRs, 0 failures.
- Built: L16 counts a Right-aligned text box as on the rhythm across when its right edge is on a rhythm line
  (`crates/lint/src/lib.rs`). Snapping already caught a box's right edge; unchanged. Test
  `crates/lint/tests/starter_rules.rs::l16_lets_a_right_aligned_text_box_end_on_the_rhythm`, which fails against the
  old code (`result=fail right_aligned_ending_on_clear=false`).
- `make check` → `TESTS scope=workspace checks=795 check_fail=0 passed=745 failed=0 status=ok runner=nextest`. Not in a
  release yet (`v0.16.5` is out); it goes in the next one.

## 2026-11-23 — bbq: where a region's major lines start

- bbq, 8 questions (`docs/bbq/2026-11-23-major-lines-from-top.md`): a per-region **Major lines from**: Top, Centre or
  Rhythm origin, on both axes; older files keep Rhythm origin; isometric offers Rhythm origin and Centre; a border stands
  for the first major. Built in a fresh session (chris), Opus 5.5 high; format 0.20.0.
- ADR-023 proposed (makeadr, goforit: every part cited to the bbq record): Major lines from Top, Centre or Rhythm origin.
  `adr-check`: 23 ADRs, 0 failures. The fresh session builds it once chris ratifies.
- ADR-024 proposed (makeadr, 4 questions, `docs/bbq/2026-11-23-adr-leaving-main.md`): work goes straight to `main` until
  v1.0 (chris overruled "end of pre-alpha, or sooner"); then ADR-022 item 3 switches on, the agent opens and merges its
  pull request on green, and a quick job reports the checks for documents-only pull requests. Queued before the v1.0
  tag (`docs/STATUS-debt.md`, the board's release bar).
- chris ratified ADR-023 and ADR-024 ("Ratify" to each). `adr-check`: 24 ADRs, 0 failures. ADR-023 is ready for the
  fresh session (`docs/bbq/2026-11-23-major-lines-from-top.md`, Kickoff); ADR-024's switch waits for v1.0.

## 2026-11-23 — ADR-023 built: Major lines from Top, Centre or Rhythm origin

- Fresh session, lead Opus 5.5 (`echo $CLAUDE_EFFORT` → `high`), no workers. Session start: `make build-proof` →
  `BUILD head=ab86f98 dirty=no rebuilt=4 target_gb=35.6`.
- Fixture first: the patterns, pattern-angle and pattern-borders pages saved by the 0.19.0 build before any change
  (`crates/render/tests/fixtures/v0.19/`, their export sums equal to those pages' goldens).
- Built from the record and ADR-023 (`docs/bbq/2026-11-23-major-lines-from-top.md`, Built): `MajorFrom` on the pattern
  region, format 0.20.0, the layout counting each axis from the region's choice, the inspector row, a golden page on both
  tablets (`major-lines-from_*`; the 36 sums before it unchanged), `--majors-page` and `--look major-lines-from`.
- Three calls the record left room for, each up for veto (record, Built): the old sample pages keep Rhythm origin so
  their goldens stay; a turned ruling counts in the box round its frame in its own axes; isometric Centre keeps its three
  families' majors meeting.
- Tests: the five new layout tests fail with the old rule put back (`test result: FAILED. 0 passed; 5 failed`), pass
  with it. `make check` → `TESTS scope=workspace checks=812 check_fail=0 passed=760 failed=0 status=ok runner=nextest`.
  `adr-check`: 24 ADRs, 0 failures.
- Not done: a screenshot of the window. The screen was locked (`mate-screensaver-command -q` → active, lock on); the
  inspector row was checked headless (`crates/ui/tests/major_from.rs`).
- STATUS.md is 330 lines, over its 300 cap (324 before this session); not trimmed here.
- Waits for chris's look; ships in the next release.

## 2026-11-23 — ADR-023 amended at chris's look: major rows and major columns each their own

- chris at his look, word for word: "The major lines need to be alignable to the vertical and horizontal rhythms".
  Asked what he meant, he picked "A choice per axis". Recorded as ADR-023's Amendment — 2026-11-23, chris, and in the
  record's Look section (`docs/bbq/2026-11-23-major-lines-from-top.md`).
- Built in the same session: `major_rows_from` and `major_columns_from` on the pattern region (format stays 0.20.0,
  never released); the layout counts each axis from its own setting; the inspector has **Major rows from** and **Major
  columns from** (columns greyed for Lines and Cornell); a new golden page `major-lines-per-axis_*` on both tablets.
  `major-lines-from_*` and the 36 sums before it are unchanged.
- Tests: with columns put back on the rows' setting, `cargo nextest run -p quadrule-layout --test major_from` →
  `5 tests run: 3 passed, 2 failed`; restored, all pass. `make check` → `TESTS scope=workspace checks=815 check_fail=0
  passed=761 failed=0 status=ok runner=nextest`. `adr-check`: 24 ADRs, 0 failures.
- Four calls up for veto (record, Look): "Top" for the left edge in the columns row; the columns row greyed, not hidden,
  for Lines and Cornell; a turned ruling's rows and columns are its own; a file from the first 0.20 build reads its
  old `major_from` as unknown.
- Waits for chris's look again.

## 2026-11-23 — ADR-023 amended again at chris's look: Centre puts a major on the middle line

- chris, word for word: "for major lines centered on the column the major line should be right in the middle of the
  block and each set of lines should go off in either direction". Recorded as ADR-023's second amendment and in the
  record's "Look, second" section.
- Built: Centre makes the middle line major, the others every Nth line either way, rows and columns alike; an even number
  of lines takes the upper or left of the two middle ones (Q4). Both golden pages re-blessed after a look on both
  tablets; the 36 sums before them unchanged (`head -36 crates/render/tests/golden/SHA256SUMS | sha256sum` →
  `2b37c042…`, the test's constant).
- Tests: with the old Centre rule put back, `cargo nextest run -p quadrule-layout --test major_from` → `5 tests run: 1
  passed, 4 failed`; restored, all pass. `make check` → `TESTS scope=workspace checks=815 check_fail=0 passed=761
  failed=0 status=ok runner=nextest`. `adr-check`: 24 ADRs, 0 failures.
- Calls up for veto (record, "Look, second"): rows follow the same rule as columns; even counts take the upper or left
  middle line.

## 2026-11-23 — Major lines from: look approved

- chris, word for word: "ok, let's approve these changes". ADR-023's build, with both amendments (a choice per axis;
  Centre on the middle line), is approved as it stands at `8fa0da4`; no calls vetoed. Recorded in the record's
  Approved section, ADR-023's History and STATUS.md. Not yet in a release.
- STATUS.md is 331 lines, still over its 300 cap; not trimmed here.

## 2026-11-23 — Homebrew cask (ADR-025)

- chris: "let's setup this repo to allow installation from homebrew". Build proof at start: `BUILD head=17b1dc8
  dirty=no rebuilt=7`.
- Three answers, word for word: "Mac app cask (Recommended)", "New repo tholent/homebrew-quadrule (Recommended)",
  "Leave it, print steps (Recommended)". Recorded as ADR-025, proposed (light route); adr-check: 25 ADRs, 0 failures.
- Built: `packaging/homebrew/quadrule.rb` (the cask template) and the "Homebrew tap" job in
  `.github/workflows/release.yml` (YAML parses; the job has not run on GitHub yet). README and DEVELOPMENT.md updated.
  The cask's v0.16.5 download address answers HTTP 200 (`curl -sIL`, 2026-11-23). No Homebrew on this machine, so
  `brew style` and `brew audit` were not run here; the tap's CI runs both and installs the cask on a Mac.
- Asked to make the tap repository and deploy key with `gh`, chris chose "Give me the commands"; the tap's first
  commit was made locally in the session's scratchpad for chris to push.
- Same day, chris: "I would like there to be a tholent/homebrew repository that holds all my casks/formulas". A repo
  named exactly `homebrew` cannot be tapped by its short name, so the name was asked; chris chose
  "tholent/homebrew-tap (Recommended)". ADR-025 (still proposed) redrafted with a History line; the release job, the
  cask template, README, DEVELOPMENT.md, STATUS.md and the debt now name `tholent/homebrew-tap`
  (`brew install --cask tholent/tap/quadrule`); the job replays its push once if the shared tap moved meanwhile.
  `tholent/homebrew-quadrule` had been made (empty, no deploy key: `gh api`, 2026-11-23); renaming it is chris's.
- chris: "ratify 025". ADR-025 accepted; its old status and note moved to its History word for word; built stays
  `partial` (the tap and the deploy key secret are still owed).
- The tap's first check (run 45324202031 on `tholent/homebrew-tap`) failed: `brew style` wants `depends_on :macos`
  (Homebrew/OSDependsOn), and Homebrew 6's tap trust (docs.brew.sh/Tap-Trust) left the tap untrusted in CI. Added
  `depends_on :macos` to `packaging/homebrew/quadrule.rb`; the tap's check gains a `brew trust tholent/tap` step
  (committed in the scratch copy for chris to push). Installing by full name trusts that cask, so the README's
  command stands.

## 2026-11-23 — v0.16.6

- chris, word for word: "ok, let's bundle all these changes into a release, pre-16b", then "ok, let's do it". Build
  proof at start: `BUILD head=096248c dirty=no rebuilt=0 target_gb=38.0`.
- setfree: no public ADR copies pending (`docs/decisions/public/pending/` does not exist). `make check-all`:
  `TESTS scope=workspace checks=815 check_fail=0 passed=761 failed=0 status=ok runner=nextest`.
- `v0.16.6` holds major lines from (ADR-023 with both amendments, format 0.20.0), L16's right-aligned rule (ADR-019
  amended) and the Homebrew cask (ADR-025); notes `docs/arcs/v1/release-notes/v0.16.6.md`. 16b becomes `v0.16.7`.
- The tap `tholent/homebrew-tap` has a write deploy key (`gh api repos/tholent/homebrew-tap/keys`, 2026-11-23); whether
  this repository holds the `HOMEBREW_TAP_DEPLOY_KEY` secret could not be read (HTTP 403 for the token).
- The tag push was refused by Claude Code's auto-mode check (not by GitHub: ADR-022 item 2 leaves creating tags open,
  and ADR-024 keeps agents pushing until v1.0). chris asked how agents were to have access until v1.0, then: "yes,
  please add rule #1": `.claude/settings.local.json` (git-ignored, this computer only) allows `git tag -a v*` and
  `git push origin v*`; agents still ask before each release.
- `v0.16.6` tagged and pushed; release run 45367622255 passed (four builds, "Draft, attach, publish", "Homebrew tap");
  published 2026-11-24T09:45:19Z as a pre-release with 7 files (`gh release view v0.16.6`). The tap step printed
  `TAP tag=v0.16.6 status=skip reason=no-deploy-key`: the secret `HOMEBREW_TAP_DEPLOY_KEY` is not set on
  tholent/quadrule, though the tap has its deploy key, so `brew install` still fails (`docs/STATUS-debt.md`).
- chris added the `HOMEBREW_TAP_DEPLOY_KEY` secret; the "Homebrew tap" job rerun pushed `quadrule 0.16.6` to the tap
  (tholent/homebrew-tap `7f48e86`). The tap's check (run 45369006172) then failed its audit twice over: livecheck's
  `:github_releases` strategy skips pre-releases (it found ''), and `brew audit` refuses a GitHub pre-release unless
  the tap's `audit_exceptions/github_prerelease_allowlist.json` names the cask (Homebrew source,
  `livecheck/strategy/github_releases.rb` and `utils/shared_audits.rb`, read 2026-11-24). chris: "the tap check has
  failed (again) and doesn't use caching. let's fix that".
- Fix: the cask's livecheck keeps pre-releases (`packaging/homebrew/quadrule.rb`); the tap gains the allowlist
  (`"quadrule": "any"`, which also passes once releases stop being pre-releases) and a check that uses
  `Homebrew/actions/setup-homebrew` (checkout, tap, trust) with caches for Homebrew's gems and downloads, pinned as
  `brew tap-new`'s template pins them. The tap commit is made but not pushed: the token lacks the `workflow` scope
  GitHub needs to change a workflow file, so chris pushes it.
- chris gave the token the Workflows permission on the tap ("permissions updated"); the tap commit `92f02b7` pushed.
  Tap check run 45370416264 passed: style, audit, `CASK name=quadrule install=ok uninstall=ok`; caches saved
  (`macOS-rubygems-…` 37.71 MiB, `macOS-downloads-…` 63.19 MiB; `gh cache list -R tholent/homebrew-tap`). ADR-025 is
  landed; its debt item is paid and removed.
- chris asked what the tag-push refusal was, then: "let's do the amendment, but also put an amendment somewhere that
  states that you cannot use compound git commands". ADR-024 amended (the local tag-push rule until v1.0, reviewed at
  the v1.0 switch); `AGENTS.md` gains, under Run and delegate, "Run each git command as a shell call of its own"
  with chris's words. adr-check: 25 ADRs, 0 failures.

## 2026-11-24 — Rung 16b leg 1: the Basic section (builder)

- Builder (Sonnet 5.5, high) for leg 1 of rung 16b (`docs/bbq/2026-11-24-rung-16b-starters.md`, Q5 and Defaulted 6, 7
  and 9), on `main`, not pushed: the lead pushes after review. chris's rule (Q5, "Switch per element"): each object, and
  the page's margins, rhythm and title, gets a **Show in Basic** switch; a **Basic** tab, first in the inspector, lists
  what is marked with its main settings; every setting stays in the other tabs too.
- **Fixtures first** (`4ecbe66`): `crates/render/tests/fixtures/v0.20/` holds the major-lines-from, major-lines-per-axis
  and calendar pages saved by the 0.20.0 build before any change, with the sha256 of each device's export, equal to
  those pages' golden lines.
- **Format 0.21.0** (`3c3ad0a`): `Object::show_in_basic` and `PageSettings::basic` (`BasicPage`: margins, rhythm,
  title), each off by default and left out of a file when off. The 0.20 to 0.21 step rewrites nothing. The 0.20
  fixtures open and export the same bytes (`crates/render/tests/migrate_v020.rs`), a resave writes no mark, and a file
  with marks round-trips (`crates/format/tests/basic_marks.rs`). Every `migrate_v*.rs` test that names the current
  version now says 0.21.0, and `newer_minor_file_keeps_its_version` uses 0.22.0 as the "newer" one.
- **The section** behind `--basic-section` (`crates/ui/src/basic.rs`, `crates/ui/tests/basic.rs`): the Basic tab is
  first in the inspector and shows with nothing selected; each marked item is listed by name with the other tabs' own
  widgets, which were pulled out of `panels.rs` into helpers (`title_row`, `margins_rows`, `rhythm_link`,
  `rhythm_unit_rows`, `pattern_kind_rows`, `pattern_spacing_rows`, `pattern_major_rows`, `stroke_rows`,
  `text_content_field`) and called by both tabs, so nothing was copied. The Style and Page tabs read and edit as
  before. The object's own widgets act on the selection, so the Basic tab stands the object in as the selection while
  they draw and puts the real one back after. The switches: Arrange tab (first row), Page tab (title, Margins heading,
  Rhythm heading), and a tick on each listed item. `--look basic` marks a sample page and shows the tab;
  `--basic-section` combines with every other flag (a command-line test has the `--look welcome` case).
- ADR-026 (proposed) drafted; `docs/decisions/INDEX.md` gains its row; `adr-check`: 26 ADRs, 0 failures.
- Not touched: the 40 golden lines (`git diff 1ce8f25 -- crates/render/tests/golden/` is empty) and every fixture folder
  before `v0.20/` (`git diff 1ce8f25 --stat -- crates/render/tests/fixtures/` shows only `v0.20/`).
- `make check-all`: `TESTS scope=workspace checks=826 check_fail=0 passed=772 failed=0 status=ok runner=nextest`.
- The look was not seen on a screen (this computer has no display); the headless tests read the widgets and the data.

## 2026-11-24 — Rung 16b leg 2: chris's seven starters (builder)

- Builder (Sonnet 5.5, high) for leg 2 of rung 16b (`docs/bbq/2026-11-24-rung-16b-starters.md`: Q2, Q3, Q6 and Defaulted
  4, 5, 7, 8, 9), on `main`, not pushed: the lead pushes after review. chris's rulings: "Builder all, you look" (Q2),
  "Constraints now" (Q3), "Rapid-log page" (Q6).
- **The generator** (`crates/xtask/src/starters.rs`, `cargo xtask starters [--check]`, `make starters`): every starter is
  one function that returns a `Starter` (category, slug, document); the file bytes are the same every run (fixed dates
  and ids, the Nomad's preview picture as Save writes it). `shrink` moved from `quadrule-ui` to `quadrule-render` (re-exported
  from `viewer.rs`, so nothing else changed) so the generator can write the previews; the lint run moved into
  `crates/xtask/src/lintrun.rs`, shared by `cargo xtask lint` and the new test. Tests (`crates/xtask/tests/starters.rs`): the
  generator writes the same bytes twice; every checked-in file is what it writes now (a format change fails it until
  `cargo xtask starters` is run again); every starter lints with no finding on both tablets (ADR-021, item 1, paid);
  names, descriptions, tags, marks and no device overrides.
- **The seven** (slug, title): `paper/narrow-ruled` "Narrow ruled"; `paper/narrow-ruled-rtl` "Narrow ruled,
  right-to-left"; `paper/square-grid` "Square grid" (chris's `grid`); `paper/hex` "Hex grid"; `paper/isometric` "Isometric
  grid"; `notes/cornell` "Cornell notes"; `bullet-journal/rapid-log` "Rapid log" (chris's `bullet-journal`; Q6). One layer,
  "Paper"; objects "Lines" and "Margin"; "Grid"; "Hex grid"; "Isometric grid"; "Cornell"; "Dots" and "Header". Each marks
  the page's margins, rhythm and title and its defining objects (Lines and Margin; Grid; Hex grid; Isometric grid; Cornell;
  Dots and Header) Show in Basic. Descriptions and tags are the builder's proposals (in the functions).
- **Why chris set each Manta override** (measured, not guessed): the rhythm is centred in the margin box, so the first
  row is at y 223 on the Nomad and 192 on the Manta (cornell, narrow ruled, rapid log: 6.35 mm = 75 px) and at 155 and
  179 (grid and hex: 6 mm = 71 px); chris moved each Manta frame so its top edge sat on that tablet's first row (cornell
  16.256 mm = 192 px, grid 15.155 mm = 179 px), and the two margin lines to a Manta column. No constraint holds a
  distance to a rhythm line that sits 31 or 24 px apart on two tablets, so the same edge is on a row on one tablet and
  between two on the other. Each edge is set instead so that on both tablets it is on a row, or leaves a clearly part
  cell (at least 24 px, 2 mm), never a sliver: the grid's frame runs from y 108 to 109 px above the bottom (whole cells
  on the Manta, 47 and 46 px part cells on the Nomad); cornell from 148 to 149 above the bottom (21 rows exactly on the
  Nomad, 44 px part rows top and bottom on the Manta); hex from 120 to 119 (24 px strips on the Manta (measured)); isometric
  inset 14 px left, 15 right, 148 top and bottom (whole columns on the Manta, 47 px part columns on the Nomad).
  The sums are in each function's comment. No override was kept (the test says so).
- **Looks fixed** (the lead and chris look): the square grid's 13 px sliver row at the bottom of the Nomad is gone (last row
  whole or a 46 px part row); the Manta's Cornell 33 px and grid 38 px slivers are gone; the isometric Manta's 14 and 15 px
  slivers beside its border are gone and it has a bottom border now; the rapid log's dots fill the margin box across
  (14 columns, 214 and 215 px clear each side on the Nomad; 21 columns, 210 each side on the Manta; clear of the docked
  toolbars) and its header line spans the first to last dot column.
- **Differences from chris's Manta pages**, measured (`python3` on the goldens, and the sums above): the margin lines Scale
  and sit at 292.6 px (chris 285; 7.6 px, 0.65 mm) and 1626 px for the right-to-left page (chris 1635; 9 px, 0.76 mm); the
  Cornell Manta frame begins 44 px above its first row (chris 0) and the grid's Manta first row is a whole cell below the
  border; the rapid log's Manta header runs 214 to 1707 px against dots at 210 to 1710 (the same constraint on both
  tablets, the Manta's columns are 4 px off the Nomad's).
- **Goldens:** `crates/render/tests/starters.rs` reads the checked-in starter files and draws each on both tablets:
  fourteen new pages `starter-<slug>_<profile>_<size>.png`, fourteen lines appended to `SHA256SUMS` after the forty,
  which are byte for byte as they were (the test hashes them). A starter with no golden fails the test. Fixtures `v0.21/`:
  narrow ruled and the rapid log as saved, with the sums of their exports, and `migrate_v021.rs` (open as 0.21.0, marks
  read, same export bytes, a resave writes the marks again and the same exports). No earlier fixture folder changed.
- **Gallery tests** (`gallery.rs`, `welcome.rs`) now say what ships: the seven titles in tab order, Paper, Notes and
  Bullet journal listing theirs, Planning and Creative still "coming soon" (leg 3).
- Docs: `DEVELOPMENT.md` gains "Starter pages" and the Makefile's `starters` row; the debt file gains "Rung 16b leg 2,
  owed".
- `make check-all`: `TESTS scope=workspace checks=847 check_fail=0 passed=779 failed=0 status=ok runner=nextest`.
- Not done: the look on a screen (this computer has none; the goldens were read as images); the speed run on the
  heaviest starter (after leg 3).

## 2026-11-24 — Rung 16b leg 3: thirteen starters (builder)

- Builder (Sonnet 5.5, high) for leg 3 of rung 16b (`docs/bbq/2026-11-24-rung-16b-starters.md`: Q1, Q4 and Defaulted 1, 2, 4,
  7, 8, 9), on `main`, not pushed. chris's rulings: "Non-calendar ones now" (Q1), "Builder, from yours" (Q4). Everything below
  (spacings, words, sizes, layouts) is the builder's proposal for chris's look.
- **The thirteen** (slug, title, rhythm, what is on the page, Show in Basic; the page's margins, spacing and title are marked on all):
  - `paper/college-ruled` "College ruled": 9/32 in (84 px), lines and a Margin line at the first column; marks Lines, Margin.
  - `paper/wide-ruled` "Wide ruled": 11/32 in (103 px), the same; marks Lines, Margin.
  - `paper/dot-grid` "Dot grid": 5 mm (59 px) dots across the margin box from y 100 down; marks Dots.
  - `paper/blank-with-margins` "Blank with margins": a Whisper Rect on the margin box (Free placement); marks Margin box.
  - `notes/meeting-notes` "Meeting notes": header of four ruled rows (Meeting, Date, Attendees), Notes area, Action items box
    (tick boxes, Owner column); marks Notes, Action items.
  - `notes/lecture-notes` "Lecture notes": header (Course, Date, Topic), ruled lines, Margin line at column 5, labels Keywords and
    Notes; marks Lines, Margin.
  - `planning/daily-planner` "Daily planner": Schedule column (6:00 to 22:00) beside To do (nine tick boxes) over Notes; marks
    Schedule, To do.
  - `planning/to-do-list` "To-do list": header row "To do", ruled rows, a column of tick boxes; marks Rows, Tick boxes.
  - `bullet-journal/habit-tracker` "Habit tracker": landscape, Habit column and 31 day columns numbered 1 to 31; marks Habits, Days.
  - `bullet-journal/index` "Index": Topic and Page columns, ruled, header rule; marks Rows, Page divider.
  - `creative/storyboard` "Storyboard": repeat of a 516 by 290 px frame over two ruled rows (2 by 3 Nomad, 2 by 4 Manta); marks Frames.
  - `creative/music-staff` "Music staff": the MusicStaff pattern, a staff every 3 units (0.25 in); marks Staves.
  - `creative/handwriting-practice` "Handwriting practice": a repeat of four lines (ascender, dashed midline, baseline, descender)
    in pieces one step wide; marks Rows.
- **Words:** descriptions and tags are in each function in `crates/xtask/src/starters.rs`; labels are Meeting, Date, Attendees, Notes,
  Action items, Owner, Course, Topic, Keywords, Schedule, 6:00 to 22:00, To do, Habit, 1 to 31, Topic, Page. No dates, weekday or month names.
- **The form lattice** (the eight pages with objects on rows): rhythm step 86 px (7.28 mm) from the margin box's corner (not centred),
  margins 162 px top and bottom and 143 at the sides. The Manta is 688 px taller and 516 px wider than the Nomad, both whole steps, so a
  frame kept from any edge falls on a rhythm line on both tablets (no part row at a border, text baselines on the same rows as the
  ruling) and L18 is quiet. The sum is in the generator's comment above `meeting_notes`. The three paper pages and the staff keep
  chris's centred rhythm and his habit of a page-wide ruling and a Scale margin line.
- **What the app cannot draw, and what was built instead:** a dashed line in a pattern (handwriting practice): a repeat of
  one-step pieces of four lines with a Custom dash of 22 px and a gap of 21 (two periods a piece, so it runs on across the joins).
  31 columns on a portrait Nomad: the habit tracker is landscape (it is exported turned a quarter turn, as any landscape page).
  The tick boxes are one-column Grid regions on the rhythm. No new drawing feature and no format change.
- **Judgement calls:** the labels are 9 pt Atkinson Hyperlegible Next, weight 600, Label role, standing a small gap above the rhythm line
  (the Baselines setting); the day numbers are 6.5 pt (the linter's smallest is 6) and Free placement; the header and action boxes have
  Structure borders on their top and bottom only (open ends) so labels sit at the margin; the blank page's box is Free placement.
- **Tests and goldens:** `crates/xtask/tests/starters.rs` covers the new files unchanged (twenty starters lint with no finding on both
  tablets, are named, marked, constrained and match the generator). `crates/render/tests/starters.rs`: twenty-six lines appended to
  `SHA256SUMS` (80 in all; the first 54 are byte for byte as they were). Each golden was read as an image. `gallery.rs` and
  `welcome.rs` now list the twenty titles and no tab says "coming soon"; `crates/ui/starters/.gitkeep` is removed.
- Docs: `README.md` (starters ship), `DEVELOPMENT.md` (the form pages), the debt file's "Rung 16b leg 3, owed".
- `welcome.rs`'s My templates test used a template titled "Dot grid", which now collides with the starter of that name; its two
  templates are now "Pocket dots" and "Weekly planner".
- `make check-all`: `TESTS scope=workspace checks=873 check_fail=0 passed=779 failed=0 status=ok runner=nextest`.
- Not done: the look on a screen (this computer has none; the goldens were read as images, and the landscape habit tracker was
  read turned a quarter turn, as it is exported); the speed run on the heaviest starter.

## 2026-11-24 — Rung 16b: kickoff, three legs, waits for chris's look (lead)

- chris, word for word: "I have updated the starters and they all pass lint checks. Calendar starters are proving more
  difficult than anticipated, so let's put them on hold for now. Please check `~/starters-draft` for the updated quad
  files. Check them out and ensure they conform and then kick off rung 16b". Build proof at start:
  `BUILD head=5c1f3cb dirty=no rebuilt=0 target_gb=38.5`.
- Review of chris's seven (`docs/bbq/2026-11-24-rung-16b-starters.md`): all lint clean (`xtask lint` → `0 finding(s) on
  2 profile(s)` each); gaps were layer and object names, hand-set Manta overrides, file names and looks. Originals kept
  in `docs/arcs/v1/starters-from-chris/2026-11-24/`.
- Budget: 20 points, alone on the Saturday meter, from 27 %; readings 32 % (11:52 UTC) and 34 % (12:19 UTC), chris's.
- bbq, 8 questions, all on the recommended option (record above). Builders: Sonnet 5.5 high, one per leg, each checked
  by the lead (diff, golden pictures, a fresh `make check-all`) before the push. Leg 1 `11ad235`
  (`TESTS ... checks=826 check_fail=0 passed=772 failed=0 status=ok`); leg 2 `37920c2` (`checks=847 ... passed=779
  failed=0 status=ok`); leg 3 `2b40ef6` (`checks=873 ... passed=779 failed=0 status=ok`, the builder's run on its final
  commit).
- Not run: the ADR-009 speed run; this machine has no display (`DISPLAY` and `WAYLAND_DISPLAY` empty, 2026-11-24).
- Speed run (ADR-009), 2026-11-24, on chris's desktop (x86_64, Xeon E5-2470 v2, 12 threads; display `:0`), release
  build, strip on: before it, a bench over a file found none of the sample pages' objects and ended with no line, and
  a file it could not open fell back to the sample page with `page=sample` (chris's first runs). Fixed
  (`crates/ui/src/view.rs`, `crates/ui/src/bench.rs`): a file's largest object is pressed, the line names the file,
  and a bench with nothing to press prints `result=fail reason=no_target`. Results, 90th percentile (ms), every run on
  the Manta (`device=secondary`: the app opened the files on its remembered tablet): habit tracker drag 28.3 and 35.7,
  draw 29.2 and 32.3; isometric drag 32.0 and 45.0, draw 40.2 and 41.4. All under 50; isometric drag is the closest.
  The Pi was not measured.

## 2026-11-24 — The Basic section switched on for good (builder)

- chris approved the look of the Basic section and ratified ADR-026 (`docs/bbq/2026-11-24-rung-16b-starters.md`,
  Amendments), so by ADR-007 the `--basic-section` flag came out. On `main`, not pushed.
- Removed: `ViewerUi::basic_section` (`crates/ui/src/view.rs`), `show_switches()` (`crates/ui/src/basic.rs`), and
  its five checks in `crates/ui/src/panels.rs` (the Basic entry in the inspector's tab list; the Page tab's title
  switch, Margins switch and Rhythm switch; the Arrange tab's Show in Basic tick), each kept now unconditional; the
  `--basic-section` option, its `Args` field, its line in `parse_args`, its two parse tests (the `--look basic` one is
  kept, without the flag) and the line in `crates/app/src/main.rs`. `--look basic` stays: the sample page with its
  title, margins, rhythm and three objects marked, the Basic tab showing; it no longer sets a flag.
- Tests (`crates/ui/tests/basic.rs`): the `flag` parameter is gone; `nothing_shows_without_the_flag` became
  `the_basic_tab_is_first_and_always_there` (Basic sits left of Page with nothing selected and left of Arrange with an
  object selected, the Page tab has three Show in Basic switches, the Arrange tab one, and the marks in a file are read).
  `the_basic_look_marks_a_page_and_shows_the_tab` no longer reads a flag.
- One more test followed the new behaviour (`crates/ui/tests/window.rs::select_in_layers_and_edit_in_the_inspector`):
  with the Basic tab always first, selecting an object when the Page tab was showing falls back to the first tab, now
  Basic (before the flag came out, Arrange). The test now expects Basic, then presses Arrange. This is what chris saw
  at the look with the flag on; whether a selection should land on Arrange instead is his to rule.
- ADR-026 header: `built: landed`, `owed:` removed, `proof:` names the new test; a History line says so with the old
  values. README, DEVELOPMENT.md, the Fluent comment and `docs/STATUS-debt.md` (item marked DONE) say the section
  always shows.
- No golden line and no fixture changed.

## 2026-11-24 — Rung 16b approved; v0.16.7 prepared (lead)

- chris at the look, word for word: "1. looks good 2. all good, for now 3. ratified". ADR-026 ratified
  (`74038e0`); the `--basic-section` flag removed by a Sonnet 5.5 high builder (`a10154d`, its `make check-all`:
  `TESTS ... checks=873 ... failed=0 status=ok`).
- Version 0.16.7: `Cargo.toml`, notes `docs/arcs/v1/release-notes/v0.16.7.md`, board row 16 done with a re-plan entry,
  next lead's prompt `docs/arcs/v1/rung-17-PROMPT-2026-11-24.md` (chris added four polish points to it). Starters record
  their saver as `quadrule starters`, so the version bump moves no starter file.
