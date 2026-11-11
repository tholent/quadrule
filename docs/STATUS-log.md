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
