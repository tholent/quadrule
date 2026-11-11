# STATUS — quadrule

_Last updated: 2026-11-10_

The project's live state. Update it when work lands. Every claim cites at least one file that exists.

## Now

- Rung 1, the headless end-to-end slice, is on `main` (PR #1, `00ad0e2`): a test builds a document, saves it as
  `.quad`, reopens it and exports Manta and Nomad PNGs matching golden images byte for byte
  (`crates/render/tests/slice.rs`, `crates/render/tests/golden/SHA256SUMS`).
- Calibration sheets (napkin §8.4): `cargo xtask calibration-sheet` writes lines, dots, text, writing and grays pages per
  profile, each sample coded, with a `codes.txt` key (`crates/xtask/src/calib.rs`, how-to in `docs/calibration.md`).
- Both devices calibrated by chris (2026-11-09): role values, the new Dot grid and Label roles, in their profiles
  (`crates/profiles/data/`, records in `docs/device-measurements/*/calibration.md`); file format 0.3 then.
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
- Rung 3, the first editor, is on `main`, look approved by chris 2026-11-10 ("I approve of the look") (record `docs/bbq/2026-11-10-rung-3-first-editor.md`): click and Shift-click to select, a layers panel
  under the inspector on the right (eye, lock, rename, Move up/down; the left side is kept for a future tools bar),
  the inspector's tabs (Page; Arrange, Style, Text, Constraints; Lint) with one length
  field reading mm, pt, px, in, %, u; arrows nudge 1 device pixel, Shift+arrows one rhythm unit; Delete, Ctrl+D;
  unlimited undo/redo; Save, Save As and the unsaved-changes prompt; autosave every 30 s and on focus loss, with
  Restore offered after a crash (`crates/model/src/edit.rs`, `crates/layout/src/units.rs`, `crates/ui/src/editor.rs`,
  `crates/ui/src/panels.rs`, `crates/ui/src/recovery.rs`). View → Canvas switches to the graphics-chip canvas
  (vello on wgpu, `crates/app/src/chip.rs`), falling back to vello_cpu when the chip cannot start or a frame
  takes over 100 ms (`crates/ui/src/canvas.rs`); on the Pi it measured 40 ms a frame (`docs/STATUS-log.md`).
  Export bytes are unchanged (`crates/ui/tests/editor.rs`).
- Rung 4a, dragging and the first snapping, is built; look approved by chris 2026-11-10 with two changes to the safe-zone display, built (record
  `docs/bbq/2026-11-10-rung-4-drag-and-snap.md`): drag an object to move it; eight resize handles (Shift keeps
  proportions, Alt resizes from the centre; text boxes get left and right only); a line shows its two ends; drag
  on empty page for a marquee (Shift adds); Alt-drag duplicates; Ctrl turns snapping off for a drag; Escape drops
  it; one drag is one undo step; drag rows in the layers panel to reorder. Snapping catches within 6 screen points:
  the rhythm and its subdivisions (2–10 a unit, file format 0.4, `Rhythm.subdivisions`), a resize's size in whole
  steps, the margins and safe-zone edges, and the pixel grid (`crates/layout/src/snap.rs`,
  `crates/model/src/edit.rs`, `crates/ui/src/drag.rs`, `crates/ui/src/view.rs`, `crates/ui/src/panels.rs`). The
  rhythm overlay (light blue, faint) and the snap switches are in the status bar and the Page tab and are kept between runs
  (`crates/app/src/main.rs`). No flag, as chris ruled; export bytes unchanged (`crates/ui/tests/editor.rs`,
  `crates/render/tests/golden/SHA256SUMS`).
- Drag speed is judged by the 90th percentile of the frames, under 50 ms (ADR-009, ratified by chris 2026-11-10; it was
  the 95th until then, and these numbers were measured at the 95th, so the 90th is no higher): measured on the Pi in a release build with `quadrule --bench-drag` (the sample page's dot grid, Fit zoom):
  export-pixels canvas 95th percentile 25.8–27.1 ms in 6 runs, graphics-chip canvas 31.2–31.5 ms in 3 runs, against the
  50 ms line (`crates/ui/src/canvas.rs`, `docs/STATUS-log.md`, 2026-11-10 rung 4a).
- The safe-zone overlay has no labels, shows the left, right and top toolbar zones and the footers (not the floating
  toolbar), fills overlapping zones once so they are no darker, draws borders only round the combined area, and the
  linter checks the same three docked positions and the footers, not the floating toolbar (chris, after rung 5; ADR-012
  amendment); dragged objects snap to the same zones (`crates/ui/src/view.rs`, `crates/ui/src/viewer.rs`,
  `crates/layout/src/snap.rs`, `crates/lint/src/lib.rs`).
- Rung 4b, the rest of snapping and copy and paste, is built; look approved by chris 2026-11-10, so rung 4 is done
  (record `docs/bbq/2026-11-10-rung-5-new-and-insert.md`, Q4; rung 4 record
  `docs/bbq/2026-11-10-rung-4-drag-and-snap.md`): a dragged box's corners, midpoints and centre land on another
  object's (a line's ends and middle), with a ring where they meet (point snap); its edges and centre line up with
  other objects', with an orange guide joining them (smart guides); and it goes where the gaps to its neighbours in
  a row or column are equal, or equal to the gap between two others, with each gap drawn and its size (equal
  spacing). A resize and a line end use the same guides and point snap. Each has its own switch in the status bar
  and the Page tab, kept between runs, and Ctrl turns them off with the rest (`crates/layout/src/snap.rs`,
  `crates/ui/src/drag.rs`, `crates/ui/src/view.rs`). Ctrl+C, Ctrl+X and Ctrl+V copy, cut and paste objects as text
  on the system clipboard, between documents too; a paste into the same document lands one rhythm unit down and
  right, each further paste one unit more (`crates/model/src/clipboard.rs`, `crates/ui/src/editor.rs`). Drag speed
  re-measured on the Pi with all helpers on, release build: export canvas 95th percentile 25.0–26.2 ms (3 runs),
  chip canvas 31.6–32.1 ms (3 runs), against 50 ms (`docs/STATUS-log.md`). A rhythm catch beats any other kind that is
  less than half a pixel nearer (chris's ruling, `crates/layout/src/snap.rs`). Export bytes unchanged
  (`crates/render/tests/golden/SHA256SUMS`).
- Rung 5, New and Insert, is done, look approved by chris 2026-11-10 (record `docs/bbq/2026-11-10-rung-5-new-and-insert.md`).
  **New…** (button, Ctrl+N; on the empty screen too) asks for a device, a rhythm preset (handwriting 6 mm, narrow,
  college and wide ruled, 5 mm grid, 1/4 in grid) and a title, then opens a blank template that targets both devices
  with margins rounded up to whole rhythm units that clear the docked toolbars and footers, so a region filling them
  passes lint L06 on both (`crates/layout/src/newpage.rs`, `crates/model/src/blank.rs`, `crates/ui/src/view.rs`). A
  **tools bar** on the left (Select V, Line L, Text T, Pattern P; Escape ends a tool; a double-click keeps one on) is a
  list that takes more tools (`crates/ui/src/tools.rs`). **Drawing**: drag to draw with rung 4's snapping (a drawn size
  lands on whole rhythm units), or click for a default (a line or text box 10 units long, a pattern region filling the
  margins); one undo step each; the tool goes back to Select; a new text box opens the Text tab with its text selected
  (`crates/ui/src/draw.rs`). Drawing speed on the Pi, release build, `quadrule --bench-draw` over the sample page:
  export canvas 90th percentile 25.5–26.1 ms (3 runs), chip canvas 41.4–42.3 ms (3 runs), against 50 ms
  (`docs/STATUS-log.md`). No flag, as chris ruled; export bytes unchanged (`crates/ui/tests/draw.rs`,
  `crates/render/tests/golden/SHA256SUMS`).
- Rung 6a, shapes, is done, approved by chris 2026-11-11 and released as `v0.6.0` (record `docs/bbq/2026-11-10-rung-6-shapes.md`): tools
  Rectangle R, Ellipse E, Polygon Y (Star is a tick on a polygon), drawn as in rung 5 with Shift for square, circle or
  regular polygon; fill roles Shade (204) and Block (68), estimates until the grays sheet is read, plus a custom gray;
  dash presets and custom dashes on whole device pixels; butt, round and square caps; corner radius; the Style tab's
  Stroke, Fill, Dash and Cap and a Shape section; lint L01, L02, L06 on the new kinds; file format 0.5.0, every 0.4 file
  opening with the same export bytes (`crates/model/src/lib.rs`, `crates/layout/src/shapes.rs`, `crates/ui/src/draw.rs`,
  `crates/ui/src/panels.rs`, `crates/render/tests/fixtures/v0.4/`). A new golden page per device; the existing sums are
  unchanged (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, release build, shapes page, 90th percentile:
  export canvas about 24 ms, chip canvas 45.9–48.4 ms, against 50 ms (`docs/STATUS-log.md`). Try it:
  `cargo run --release -p quadrule-app -- --shapes-page`.
- Every string on screen goes through Fluent (`crates/i18n/`, English in `crates/i18n/locales/en-US/`), lint
  messages included; findings about an object carry its area (`crates/lint/src/lib.rs`).
- A linter with the seven rules that need no calibration (`crates/lint/src/lib.rs`); `cargo xtask lint`.
- The `.quad` reader is fuzzed with cargo-fuzz: 3.3 million runs on the Pi, 60 s on every CI push, no crash
  (`crates/format/fuzz/fuzz_targets/open_quad.rs`).
- Licensed Apache-2.0 (`LICENSE`); the repository is public. every Rust file carries the header, enforced by a test
  (`crates/xtask/tests/license_headers.rs`).
- Checks, 2026-11-10 on the Pi: `make check-all` → `TESTS scope=workspace checks=221 check_fail=0 passed=209 failed=0
  status=ok` (after rung 6a; `grep -o 'CHECK crate=[^ ]* name=[^ ]*' target/test.log | wc -l` → 221); fmt and clippy `-D warnings` clean
  (`Makefile`). CI for rung 5: see `docs/STATUS-log.md`
  (`.github/workflows/ci.yml`).
- Release packages: `.github/workflows/release.yml` builds an AppImage and a `.deb` (Linux x86-64 and ARM64), a
  universal unsigned `.dmg` and an `.msi` from a release tag with cargo-packager (`packaging/`), and attaches them
  when a release is published (approved by chris 2026-11-10; hand runs stay possible). Tried: the `.dmg` (chris, a
  Mac) and the ARM64 AppImage (the Pi). README has install steps (`docs/STATUS-log.md`, `DEVELOPMENT.md`, `README.md`).
- A `Makefile` wraps the usual commands: `make` lists them; `make test` prints one `TESTS` line, `make check-all` is what CI
  runs, `make review` adds the build-matches-HEAD proof and the diff against `origin/main` (`Makefile`,
  `docs/bbq/2026-11-10-makefile.md`).
- Work goes straight to `main` until a job needs a branch (chris's ruling, `AGENTS.md`). Rust 1.99.1 pinned
  (`rust-toolchain.toml`).

- A `README.md` for users (pre-alpha banner, what works today, e-ink features, how to run it) and a `DEVELOPMENT.md`
  for developers (setup, commands, crate map, contributing, calibration) are on `main` (`README.md`, `DEVELOPMENT.md`,
  record `docs/bbq/2026-11-10-readme-and-description.md`). The GitHub description and topics were set by hand by chris
  on 2026-11-10 (the `gh` token got HTTP 403 on `gh repo edit`); read back with `gh repo view` (`docs/STATUS-log.md`).

- README badges: AI framework pitmaster, CI (fmt, clippy, tests), Fuzz, and coverage. Coverage is new: `.github/workflows/coverage.yml`
  runs `cargo llvm-cov` on the workspace and `.github/scripts/publish-badge.sh` writes the percentage to a
  `badges` branch for shields.io to read. Its first run on GitHub, 42868546083 on 2026-11-10, passed (`docs/STATUS-log.md`).

## Next

- **Arc v1** plans the rest of the way to v1.0 (the full §16 list) as rungs 6 to 18: the board is
  `docs/arcs/v1/README.md`, and the next lead's prompt is `docs/arcs/v1/v1-PROMPT-2026-11-10.md` (Opus 5.5 high
  leads, Sonnet 5.5 high builds). Rung 6, shapes, is planned (bbq, 8 questions, `docs/bbq/2026-11-10-rung-6-shapes.md`):
  6a all the shapes with fill (Shade and Block roles), dashes and caps; 6b rotation, lines any way, opacity. 6a is done
  (`v0.6.0`); next: 6b's build.
- Waits on chris (early, alongside rungs 6–7): the clean gray levels on both devices (the new grays sheet,
  `docs/calibration.md`), and a landscape check on a
  device plus landscape toolbar screenshots (`docs/arcs/v1/README.md`, chris's queue). L04 moved to v1.1
  (`napkins/quadrule-spec.md` §8.3 note).

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- Clean gray levels are not measured on either device; the Manta's pen shades are ruled equal to the Nomad's
  (`docs/STATUS-debt.md`).
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
- 2026-11-10: rung 3 built (steps 1–9), chip canvas measured on the Pi; layers moved right; look approved
  (`docs/STATUS-log.md`).
- 2026-11-10: rung 4 planned (bbq, 12 questions): dragging and snapping, split 4a/4b (`docs/STATUS-log.md`).
- 2026-11-10: rung 4a built (Sonnet 5.5, high): drag, handles, marquee, layer drag, snapping, overlay; measured on the Pi;
  look approved with two safe-zone changes, built (`docs/STATUS-log.md`).
- 2026-11-10: rung 4b built (Sonnet 5.5, high): point snap, smart guides, equal spacing, copy and paste; measured on the
  Pi; waits for chris's look (`docs/STATUS-log.md`).
- 2026-11-10: Makefile for testing and review (bbq, 4 questions), built (`docs/STATUS-log.md`).
- 2026-11-10: README.md and DEVELOPMENT.md written (bbq, 12 questions); GitHub description blocked by a 403 (`docs/STATUS-log.md`).
- 2026-11-10: rung 4b approved, rung 4 done; rung 5 planned (bbq, 12 questions): New and Insert (`docs/STATUS-log.md`).
- 2026-11-10: ADR backlog sweep: ADR-004 to ADR-014 drafted and all ratified by chris; drag speed now the 90th
  percentile (`docs/STATUS-log.md`, `docs/decisions/INDEX.md`).
- 2026-11-10: rung 5 built (Sonnet 5.5, high): New, the tools bar, drawing; measured on the Pi; waits for chris's look
  (`docs/STATUS-log.md`).
- 2026-11-10: rung 5 approved by chris; L06 stops checking the floating toolbar (ADR-012 amended) (`docs/STATUS-log.md`).
- 2026-11-10: arc v1 planned (Arc bbq, 20 questions): rungs 6–18 to v1.0 (`docs/arcs/v1/README.md`, `docs/STATUS-log.md`).
- 2026-11-10: rungs 1–5 tagged at their approvals, `v0.1.0` to `v0.5.0` (4b is `v0.4.1`) (`docs/STATUS-log.md`).
- 2026-11-10: v0.5.0 released on GitHub; release packages workflow built, dry run green on four systems (`docs/STATUS-log.md`).
- 2026-11-10: v0.5.0 packages published; pixel grid now from 16× zoom, not 8× (chris) (`docs/STATUS-log.md`).
- 2026-11-10: rung 6 kickoff: budget 15 points; grays calibration sheet added; rung 6 planned (bbq, 8 questions), split
  6a/6b (`docs/STATUS-log.md`, `docs/bbq/2026-11-10-rung-6-shapes.md`).
- 2026-11-10: rung 6a built (Sonnet 5.5, high): shapes, fill roles, dashes, caps, format 0.5.0; measured on the Pi;
  waits for chris's look (`docs/STATUS-log.md`).
- 2026-11-11: rung 6a approved by chris; `v0.6.0` tagged and released (`docs/STATUS-log.md`).

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
