# STATUS — quadrule

_Last updated: 2026-11-17_

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
- Rung 6a, shapes, is done, approved by chris 2026-11-11 and tagged `v0.6.0` (record `docs/bbq/2026-11-10-rung-6-shapes.md`): tools
  Rectangle R, Ellipse E, Polygon Y (Star is a tick on a polygon), drawn as in rung 5 with Shift for square, circle or
  regular polygon; fill roles Shade (204) and Block (68 then; 102 since the grays sheet, 2026-11-12), plus a custom gray;
  dash presets and custom dashes on whole device pixels; butt, round and square caps; corner radius; the Style tab's
  Stroke, Fill, Dash and Cap and a Shape section; lint L01, L02, L06 on the new kinds; file format 0.5.0, every 0.4 file
  opening with the same export bytes (`crates/model/src/lib.rs`, `crates/layout/src/shapes.rs`, `crates/ui/src/draw.rs`,
  `crates/ui/src/panels.rs`, `crates/render/tests/fixtures/v0.4/`). A new golden page per device; the existing sums are
  unchanged (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, release build, shapes page, 90th percentile:
  export canvas about 24 ms, chip canvas 45.9–48.4 ms, against 50 ms (`docs/STATUS-log.md`). Try it:
  `cargo run --release -p quadrule-app -- --shapes-page`.
- Rung 6b, rotation, lines any way and opacity, is done, approved by chris 2026-11-11 with one change (the box and
  its handles turn with the object, built) and tagged `v0.6.1` (record
  `docs/bbq/2026-11-10-rung-6-shapes.md`): a round rotate handle on a stalk above a selected line or shape, catching at
  0, 90, 180 and 270°, Shift for 15° steps, and a Rotation field in the Arrange tab; lines drawn and pulled in every
  direction (the up-right debt is paid); Opacity 0–100 % in the Style tab, flattened to gray on export; file format
  0.6.0, 0.4 and 0.5 files opening with the same export bytes (`crates/render/tests/fixtures/v0.5/`). Text boxes and
  pattern regions do not rotate yet. A new golden page per device; the existing four sums are unchanged
  (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, 90th percentile: export canvas about 25 ms, chip canvas
  34.9–36.7 ms on the transforms page, against 50 ms (`docs/STATUS-log.md`). Try it: `cargo run --release -p
  quadrule-app -- --transforms-page`.
- Rung 7a, groups, is done, approved by chris 2026-11-12 with one change (groups see-through to constraints, as in
  Figma; Q3 amended, built) and tagged `v0.7.0` (record `docs/bbq/2026-11-11-rung-7-groups-and-align.md`): Ctrl+G
  and Ctrl+Shift+G, nesting, click picks the outermost group, double-click enters, Escape leaves, Ctrl+click picks the
  innermost; children keep their own constraints against the page, so grouping changes no pixel on either device, and
  resizing a group scales what is inside; a group keeps its own angle (not with text or a
  pattern inside); a group fades as one; the layers panel is a tree; file format 0.7.0, 0.6 files opening with the same
  export bytes (`crates/render/tests/fixtures/v0.6/`, `crates/layout/src/tree.rs`, `crates/ui/src/editor.rs`). A new
  golden page per device; the six old sums unchanged (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, groups
  page, 90th percentile: export about 27 ms, chip about 42 ms, against 50 ms (`docs/STATUS-log.md`). Try it:
  `cargo run --release -p quadrule-app -- --groups-page`.
- Rung 7b, align and distribute, is done, approved by chris 2026-11-12 and tagged `v0.7.1`, so rung 7 is done (record
  `docs/bbq/2026-11-11-rung-7-groups-and-align.md`): in the Arrange tab, six align and four distribute buttons (by gaps
  and by centres) and an Align to choice (Selection, Page, Margins; Page with one object picked, Selection with several,
  each remembering the user's pick); results land on whole device pixels when the pixel snap is on; turned objects and
  groups align by their turned box; locked objects stay put; one undo step each. With several objects selected, X, Y,
  Width, Height and Rotation read "mixed" where they differ (`crates/layout/src/align.rs`, `crates/ui/src/arrange.rs`,
  `crates/ui/src/panels.rs`). No export change: all eight golden sums unchanged
  (`crates/render/tests/golden/SHA256SUMS`).
- Rung 8, several devices, is done: 8a and 8b approved together by chris 2026-11-13 and shipped as one `v0.8.0`
  ("let's bundle this into a single v0.8.0 release"; record `docs/bbq/2026-11-12-rung-8-several-devices.md`,
  Amendments). The overrides dot keeps the accent colour; clicking it to open the Overrides tab is v1.1
  (`napkins/quadrule-spec.md` §16).
  **8a**: per-device overrides of position, size, font size and the eye on a secondary device, the previews strip, the
  Overrides tab, Make primary, device ticks in the Page tab, lint L15, file format 0.8.0
  (`crates/layout/src/device.rs`, `crates/ui/src/strip.rs`, `crates/lint/src/lib.rs`). **8b**: Portrait or Landscape in
  New… and the Page tab (one undo step); a landscape page is designed upright and exported turned a quarter turn
  clockwise into the device's portrait PNG, an exact pixel turn; the safe zones are turned with it for the overlay,
  snapping, L06 and New…'s margins (`crates/profiles/src/lib.rs`, `crates/render/src/lib.rs`,
  `crates/render/tests/landscape.rs`, `crates/ui/tests/landscape.rs`). Two golden pages each (`devices_*`,
  `landscape_*`); the eight older sums unchanged (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, 90th
  percentile, strip on: landscape page drag 25–26 ms export, 29 ms chip; draw 24–26 ms export, 38 ms chip
  (`docs/STATUS-log.md`). Try it: `cargo run --release -p quadrule-app -- --devices-page` or `-- --landscape-page`.
- Rung 9, the pattern library, is done: approved by chris 2026-11-13 ("approved. everything looks great") after three
  changes at his first look, and shipped as `v0.9.0` (record `docs/bbq/2026-11-13-rung-9-pattern-library.md`, with its
  Amendments). New patterns: isometric lines and dots (axes 30°, 90°, 150°; spec §6.3 note), hex with a flat or pointy
  top, Cornell, music staff and grand staff; lines crossing a region's border are cut there (ADR-014 amended
  2026-11-13). Major lines: "Major every N" on the rhythm, adjusted per region, major dots one pixel bigger. A region
  can be unbound from the rhythm with its own spacing; lint L17 notes a mixed rhythm. The rhythm has a Type,
  Rectangular or Isometric (snapping a column across and half a side down); New… has "Isometric 5 mm"; file format 0.9.0
  (`crates/layout/src/pattern.rs`, `crates/model/src/lib.rs`, `crates/lint/src/lib.rs`,
  `crates/render/tests/fixtures/v0.8/`). Four golden lines added (`patterns_*`, `patterns-landscape_*`); the twelve older
  ones unchanged (`crates/render/tests/golden/SHA256SUMS`). The chip canvas now merges same-gray rectangles into one
  path, which paid the rung 8a pattern-draw debt (51 → 41 ms) (`crates/ui/src/canvas.rs`). Speed on the Pi, pattern
  page, 90th percentile: drag 21–22 ms export, 34 ms chip; draw 22–24 ms export, 41 ms chip (`docs/STATUS-log.md`).
  Try it: `cargo run --release -p quadrule-app -- --patterns-page`.
- Rung 10, the repeat tool, is done: approved by chris 2026-11-13 ("approved") and shipped as `v0.10.0` (record
  `docs/bbq/2026-11-13-rung-10-repeat-tool.md`, with its Amendments). A live repeat holds its source inside, like a
  group: Linear (across or down) or Grid; Count or Fill its box (a box pinned to the margins gives each device its own
  count); Auto pitch = the source's size rounded up to whole rhythm steps, or a pitch or gap in any unit; Start or
  Centre; alternation rules by columns, rows or cells (skip, line role, fill role); double-click any copy to edit the
  source; Expand to one group; a Repeat tab and a Repeat button beside Group; copies pixel-identical; L06 and L10 judge
  every copy, L16 and L17 the pitch; file format 0.10.0 (`crates/model/src/repeat.rs`, `crates/layout/src/repeat.rs`,
  `crates/ui/src/repeat_tab.rs`, `crates/lint/src/lib.rs`, `crates/render/tests/fixtures/v0.9/`). Two golden lines
  added (`repeats_*`); the sixteen older ones unchanged (`crates/render/tests/golden/SHA256SUMS`). The chip canvas now
  keeps the unchanged part of the page in its own texture and draws only what changed (chris: "Fix it now, in rung
  10"; `crates/app/src/chip.rs`). Speed on the Pi, repeat page, 90th percentile: drag 22.6–22.8 ms export, 35.6–35.7
  chip; draw 20.3–21.3 export, 23.6 chip (`docs/STATUS-log.md`). Try it:
  `cargo run --release -p quadrule-app -- --repeats-page`.
- Rung 11, text and fonts, is done: 11a approved by chris 2026-11-15, 11b 2026-11-16 ("11b looks good in the window;
  excellent job"), shipped together as `v0.11.0` ("release 11a/b as v0.11.0"; record
  `docs/bbq/2026-11-14-rung-11-text-and-fonts.md`, with its Amendments). **11a**: bundled Atkinson Hyperlegible Next and
  Mono, Inter, Literata (each with italic), Caveat and Noto Emoji with their OFL texts and the Unicode-3.0 notice; a
  Style field (italic); weights per family; optical size; installed fonts on all three systems, re-scanned in the
  background on focus; fallback from bundled fonts only until rung 12 (ADR-002 amended for `unicode-script`); the font
  picker; the emoji picker (`crates/text/src/registry.rs`, `crates/ui/src/fonts.rs`, `crates/ui/src/emoji.rs`). **11b**:
  Auto height or Fixed; overflow cut at whole lines with a red mark and L08; paragraph spacing; vertical alignment;
  baselines on the rhythm, on the rule's top edge with a "Small gap above the line" tick (chris); text rotation; typing
  on the canvas (`crates/text/src/lib.rs`, `crates/ui/src/typing.rs`, `crates/lint/src/lib.rs`). File format 0.12.0
  (`crates/render/tests/fixtures/v0.10/`, `v0.11/`). Four golden lines added (`fonts_*`, `textbox_*`); the eighteen
  older ones unchanged (`crates/render/tests/golden/SHA256SUMS`). The chip canvas draws only the changed patch, with text
  held in rows (`crates/app/src/chip.rs`). Speed on the Pi, 90th percentile, chip canvas: typing about 9 ms (was 82),
  dragging 4–6 ms, drawing 23–25 ms; export canvas about 20 ms throughout (`docs/STATUS-log.md`). Try it:
  `cargo run --release -p quadrule-app -- --textbox-page` or `-- --fonts-page`.
- Rung 12, missing fonts and turning patterns, is done: 12a approved by chris 2026-11-16 ("I approve 12a"), 12b
  2026-11-17 ("approved"), shipped together as `v0.12.0` (record `docs/bbq/2026-11-16-rung-12-missing-fonts.md`, with
  its Amendments). **12a**: a page whose fonts are missing opens from its saved outlines with a yellow bar and a Missing
  fonts dialog (Use saved outlines, read-only with a badge; Substitute by measured font signature, rewriting the font;
  Locate fonts…; Remember for this document); installed fonts whose `fsType` allows it are embedded in
  `assets/fonts/` (Embed fonts tick, on); missing characters may come from any installed font; lint L11 and L20 with a
  sheet on Save, Save As and Export (`crates/ui/src/missing.rs`, `crates/ui/src/missing_ui.rs`,
  `crates/text/src/registry.rs`, `crates/lint/src/lib.rs`). **12b**: pattern regions turn whole (handle, Rotation
  field, groups), and a Pattern angle (Style tab) turns the ruling inside its frame, cut at the frame (chris's addition,
  Q8 amended) (`crates/layout/src/pattern.rs`). Hinting moved to v1.1 (ADR-002 amendment 2026-11-16). File format
  0.13.0 (`crates/render/tests/fixtures/v0.12/`). Four golden lines added (`patterns-turned_*`, `pattern-angle_*`);
  the 22 older ones unchanged (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, 90th percentile: missing-font
  page drag 19.8 ms export, 5.2 chip; pattern-angle page drag about 21 export, 3.6 chip, draw about 21 export, 23.4 chip
  (`docs/STATUS-log.md`). Try it: `cargo run --release -p quadrule-app -- --missing-fonts-page`,
  `-- --turned-patterns-page` or `-- --pattern-angle-page`.
- Rung 13, the profile system and paper sizes, is done: 13a and 13b approved by chris 2026-11-17 ("approved" each),
  shipped together as `v0.13.0` (record `docs/bbq/2026-11-17-rung-13-profile-system.md`, with its Amendments; ADR-016
  ratified). **13a**: a Profiles window (toolbar and Page tab) to edit every profile; an edited Manta or Nomad is a local
  override marked Edited, with Reset to built-in; New…, Duplicate, Delete, `.ron` Import and Export; user profiles get
  GUID ids and live in the config folder's `profiles/`; a `.quad` carries the profiles it uses that are not plain
  built-ins, and a same-named incoming one offers Merge (its values update yours), Rename or Not now; a damaged profile
  file is skipped and listed (`crates/profiles/src/registry.rs`, `crates/profiles/src/active.rs`,
  `crates/ui/src/profiles_win.rs`). **13b**: eleven built-in paper sizes (A3 to Tabloid), PPI per page (150/200/300/600),
  the Nomad's line weights by physical size, wide landscape exports, safe-zone lint skipped on paper
  (`crates/profiles/src/lib.rs`, `crates/layout/src/device.rs`). File format 0.14.0
  (`crates/render/tests/fixtures/v0.13/`). Three golden lines added (`paper_*`, `paper-landscape_*`); the 26 older ones
  unchanged (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, 90th percentile: A4 at 300 PPI 34–35 ms export,
  4–12 ms chip; Tabloid at 600 PPI 410–440 ms export (over the 50 ms line; ruled debt for rung 17 by chris, "C"), 4 ms
  chip (`docs/STATUS-log.md`, `docs/STATUS-debt.md`). Try it: `cargo run --release -p quadrule-app -- --profiles` or
  `-- --paper-page`.
- The grays sheet is read (chris, 2026-11-12, on the Nomad; the Manta ruled the same): every gray is clean, but on a
  flat area 0–68 all look black and 238–255 white, leaving nine distinct middle grays 85–221. Block moved from 68
  (black) to 102, "A lighter dark gray"; Shade stays 204. The shapes, transforms and groups golden pages changed
  only where Block is (every changed pixel lighter, by at most 34) (`crates/profiles/data/`,
  `docs/device-measurements/supernote.a6x2/calibration.md`, `crates/render/tests/golden/SHA256SUMS`).
- Every string on screen goes through Fluent (`crates/i18n/`, English in `crates/i18n/locales/en-US/`), lint
  messages included; findings about an object carry its area (`crates/lint/src/lib.rs`).
- A linter with the seven rules that need no calibration (`crates/lint/src/lib.rs`); `cargo xtask lint`.
- The `.quad` reader is fuzzed with cargo-fuzz: 3.3 million runs on the Pi, 60 s on every CI push, no crash
  (`crates/format/fuzz/fuzz_targets/open_quad.rs`).
- Licensed Apache-2.0 (`LICENSE`); the repository is public. every Rust file carries the header, enforced by a test
  (`crates/xtask/tests/license_headers.rs`).
- Checks, 2026-11-12 on the Pi: `make check-all` → `TESTS scope=workspace checks=312 check_fail=0 passed=292 failed=0
  status=ok` (after rung 7b); fmt and clippy `-D warnings` clean
  (`Makefile`). CI for rung 5: see `docs/STATUS-log.md`
  (`.github/workflows/ci.yml`).
- Release packages: `.github/workflows/release.yml` builds an AppImage and a `.deb` (Linux x86-64 and ARM64), a
  universal unsigned `.dmg` and an `.msi` from a release tag with cargo-packager (`packaging/`). Since 2026-11-11 a
  tag push does the whole release: draft, attach, publish (the repository's releases are immutable; chris: "Tag push
  does it all"). `v0.6.0` and `v0.6.1` went out without packages and say so. Tried: the `.dmg` (chris, a
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
  6a all the shapes with fill (Shade and Block roles), dashes and caps; 6b rotation, lines any way, opacity. **Rung 6 is
  done** (6a `v0.6.0`, 6b `v0.6.1`); both are published on GitHub, without packages. Rung 7, groups; align and distribute, is
  planned (bbq, 8 questions, `docs/bbq/2026-11-11-rung-7-groups-and-align.md`): **rung 7 is done** (7a groups `v0.7.0`, 7b
  align and distribute `v0.7.1`). **Rung 8 is done** (`v0.8.0`, 8a and 8b in one release). **Rung 9 is
  done** (`v0.9.0`). **Rung 10 is done** (`v0.10.0`). **Rung 11 is done** (`v0.11.0`). **Rung 12 is done** (`v0.12.0`). **Rung 13 is done** (`v0.13.0`). Next: rung 14, lint and preflight; the
  next lead's prompt is `docs/arcs/v1/rung-14-PROMPT-2026-11-17.md`. The Paper, Notes, Creative, Planning and Bullet journal starters are
  buildable now; chris designs one per category (`docs/arcs/v1/README.md`).
- Landscape answered on the Nomad (chris, 2026-11-12): the device wants rotated pixels. A landscape template must be a
  design turned a quarter turn and saved as a portrait PNG; a wide PNG is shrunk, and landscape mode zooms the page by
  4/3 (`docs/device-measurements/supernote.a6x2/landscape-test.md`). Rung 8's orientation work builds on this (`docs/arcs/v1/README.md`, chris's queue). L04 moved to v1.1
  (`napkins/quadrule-spec.md` §8.3 note).

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- The Manta's gray levels and pen shades are ruled equal to the Nomad's, not measured (`docs/STATUS-debt.md`).
- ADR parts not built yet: the hinting flag and hinted outline tables (v1.1), parts of the canvas, snapping and
  roles ADRs (`docs/decisions/INDEX.md`, `docs/STATUS-debt.md`).

## Log

Session history: `docs/STATUS-log.md`. Finished work: `docs/STATUS-archive.md`.

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
- 2026-11-13: rung 9 planned (bbq, 8 questions; one rung, one look), built (Sonnet 5.5, high), changed at chris's look
  (isometric turned 90°, hex cut at the frame, bigger major dots; ADR-014 amended), approved; `v0.9.0`; board
  re-planned; rung 10 prompt written (`docs/STATUS-log.md`, `docs/arcs/v1/rung-10-PROMPT-2026-11-13.md`).
- 2026-11-13: rung 10 planned (bbq, 8 questions; one rung, one look), built (Sonnet 5.5, high), chip-canvas speed fix
  at chris's word (draw 49.7 → 23.6 ms), approved; `v0.10.0`; board re-planned; rung 11 prompt written
  (`docs/STATUS-log.md`, `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md`).
- 2026-11-14/16: rung 11 planned (bbq, 8 questions; text rotation and typing on the canvas added by chris), split
  11a/11b, built (four Sonnet 5.5 high legs), baseline gap and chip-typing fix at chris's word, approved; one release
  `v0.11.0`; ADR-002 amended; board re-planned; rung 12 prompt written (`docs/STATUS-log.md`,
  `docs/arcs/v1/rung-12-PROMPT-2026-11-16.md`).
- 2026-11-16/17: rung 12 planned (bbq, 9 questions; pattern rotation unparked by chris, hinting to v1.1), split 12a/12b,
  built (Sonnet 5.5 high: 12a, stopped once by an account error and resumed; 12b with chris's pattern-angle addition),
  approved; one release `v0.12.0`; ADR-001 and ADR-002 built status; board re-planned; rung 13 prompt written
  (`docs/STATUS-log.md`, `docs/arcs/v1/rung-13-PROMPT-2026-11-17.md`).
- 2026-11-17: rung 13 planned (bbq, 14 questions; GUID ids by chris; paper sizes pulled in from v1.1 by chris), split
  13a/13b, built (two Sonnet 5.5 high builders), approved; ADR-016 ratified; large paper pages on the export canvas
  ruled debt for rung 17; one release `v0.13.0`; board re-planned; rung 14 prompt written (`docs/STATUS-log.md`,
  `docs/arcs/v1/rung-14-PROMPT-2026-11-17.md`).
