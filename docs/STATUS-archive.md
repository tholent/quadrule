# STATUS-archive — quadrule

Finished entries moved out of `STATUS.md`, word for word, with the date they moved. Newest at the bottom.

## Moved 2026-11-09 (rung 1 session)

From Now:
- Project prepared for agents with project-prep: the rules are in `AGENTS.md`, the split files in `docs/`.
- Design spec is a draft (v0.1) (`napkins/quadrule-spec.md`); plan for the first work in
  `docs/bbq/2026-11-09-quadrule-spikes.md`.
- The three technical spikes are done on branch `spikes/2026-11-09` (local only): format, text and rendering, each a
  throwaway crate with findings and a proposed ADR (`spikes/format/FINDINGS.md`, `spikes/text/FINDINGS.md`,
  `spikes/render/FINDINGS.md`). Tests: format 10, text 12, render 9 passed.
- Rust 1.99.1 installed on the Pi with rustup (`spikes/format/FINDINGS.md` header).

From Next:
- Three ADRs ratified 2026-11-09, none built yet: `.quad` format (ADR-001), text stack (ADR-002), export renderer
  (ADR-003) (`docs/decisions/INDEX.md`). Next: plan v1.0's first rung — the Cargo workspace (napkin §15.1) and the
  real `format`, `text` and `render` crates built from these ADRs.

## Moved 2026-11-10 (rung 3 done)

From Next:
- Rung 3 waits on chris's look check of the window (`docs/bbq/2026-11-10-rung-3-first-editor.md`, Kickoff): try
  `cargo run --release -p quadrule-app -- [file.quad]` (add `--canvas chip` for the chip canvas). Taste calls
  made for it are listed in `docs/STATUS-log.md` (2026-11-10, rung 3 built). Then rung 4: dragging and snapping.

## Moved 2026-11-16 from STATUS.md, Log (word for word)

- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).
- 2026-11-09: rung 1 built and merged (PR #1), CI green on three OSes (`docs/STATUS-log.md`).
- 2026-11-09: LICENSE and headers; calibration sheets; repo public; safe zones measured; stock grid matched;
  linter started; fuzzing (`docs/STATUS-log.md`).
- 2026-11-09: rung 2 planned (bbq, 12 questions) and built: the viewer window; spike R15 paid (`docs/STATUS-log.md`).

## Moved 2026-11-17 from STATUS.md, Log (word for word)

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

## Moved from STATUS.md Log on 2026-11-17 (rung 13 close), word for word

- 2026-11-11: rung 6a approved by chris; `v0.6.0` tagged; its GitHub release waits on chris (`docs/STATUS-log.md`).
- 2026-11-11: rung 6b built (Sonnet 5.5, high): rotation, lines any way, opacity, format 0.6.0; measured on the Pi
  (`docs/STATUS-log.md`).
- 2026-11-11: rung 6b approved with one change (handles turn with the object), built; `v0.6.1` tagged; rung 6 done;
  board re-planned; rung 7 prompt written (`docs/STATUS-log.md`, `docs/arcs/v1/rung-7-PROMPT-2026-11-11.md`).
- 2026-11-12: rung 7 kickoff: budget 10 points; planned (bbq, 8 questions), split 7a/7b; 7a built (Sonnet 5.5, high):
  groups, format 0.7.0; measured on the Pi; approved with the Figma constraints change, built; `v0.7.0` tagged
  (`docs/STATUS-log.md`).
- 2026-11-12: rung 7b built (Sonnet 5.5, high): align and distribute, "mixed" fields; approved by chris; `v0.7.1`
  tagged; rung 7 done; board re-planned; rung 8 prompt written (`docs/STATUS-log.md`,
  `docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`).
- 2026-11-12/13: rung 8 planned (bbq, 8 questions), split 8a/8b; 8a built (Sonnet 5.5, high); its look deferred by
  chris; 8b finished, tested and measured by the lead; both wait for chris's look (`docs/STATUS-log.md`).
- 2026-11-13: rung 8 approved by chris (8a and 8b); version 0.8.0, one release `v0.8.0`; board re-planned; rung 9
  prompt written (`docs/STATUS-log.md`, `docs/arcs/v1/rung-9-PROMPT-2026-11-13.md`).

## Moved from STATUS.md Now on 2026-11-17 (rung 14 close), word for word

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

## Moved from STATUS.md Log on 2026-11-17 (rung 14 close), word for word

- 2026-11-13: rung 9 planned (bbq, 8 questions; one rung, one look), built (Sonnet 5.5, high), changed at chris's look
  (isometric turned 90°, hex cut at the frame, bigger major dots; ADR-014 amended), approved; `v0.9.0`; board
  re-planned; rung 10 prompt written (`docs/STATUS-log.md`, `docs/arcs/v1/rung-10-PROMPT-2026-11-13.md`).
- 2026-11-13: rung 10 planned (bbq, 8 questions; one rung, one look), built (Sonnet 5.5, high), chip-canvas speed fix
  at chris's word (draw 49.7 → 23.6 ms), approved; `v0.10.0`; board re-planned; rung 11 prompt written
  (`docs/STATUS-log.md`, `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md`).

## Moved from STATUS.md on 2026-11-17 (rung 15 close), word for word

From Now:

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

From Log:

- 2026-11-14/16: rung 11 planned (bbq, 8 questions; text rotation and typing on the canvas added by chris), split
  11a/11b, built (four Sonnet 5.5 high legs), baseline gap and chip-typing fix at chris's word, approved; one release
  `v0.11.0`; ADR-002 amended; board re-planned; rung 12 prompt written (`docs/STATUS-log.md`,
  `docs/arcs/v1/rung-12-PROMPT-2026-11-16.md`).

## Moved from STATUS.md on 2026-11-18 (rung 16a close), word for word

- Rung 14, lint and preflight, is done: 14a and 14b approved by chris 2026-11-17 ("approved", "14b approved"), shipped
  together as `v0.14.0` (record `docs/bbq/2026-11-17-rung-14-lint-and-preflight.md`, with its Amendments; ADR-012
  amended). **14a**: L03, L05, L07, L09, L13, L14 judge the gray on the page (opacity included) against limits in each
  profile's new `lint` block (`LintThresholds`), filled from the calibration sheets, edited in the Profiles window's
  Lint checks section; lint lists the checks skipped on each device and why; L02's diagonal note moved to L14
  (`crates/lint/src/lib.rs`, `crates/profiles/src/lib.rs`). **14b**: the Lint tab's Findings for picker, quick fixes
  (`crates/lint/src/fix.rs`), Ignore this rule for this document and Ignored rules, Free placement (Arrange tab), the
  Before you export sheet with the installed-fonts row, gray swatches with the matches badge and Use recommended
  (`crates/ui/src/lint_ui.rs`, `crates/ui/src/preflight.rs`, `crates/ui/src/swatches.rs`). File format 0.15.0
  (`crates/render/tests/fixtures/v0.14/`). The 29 golden lines unchanged (`crates/render/tests/golden/SHA256SUMS`).
  Speed on the Pi, 90th percentile: 17–23 ms on the lint page, 30.6 ms worst (groups page drag, chip)
  (`docs/STATUS-log.md`). Try it: `cargo run --release -p quadrule-app -- --lint-page`.
- 2026-11-16/17: rung 12 planned (bbq, 9 questions; pattern rotation unparked by chris, hinting to v1.1), split 12a/12b,
  built (Sonnet 5.5 high: 12a, stopped once by an account error and resumed; 12b with chris's pattern-angle addition),
  approved; one release `v0.12.0`; ADR-001 and ADR-002 built status; board re-planned; rung 13 prompt written
  (`docs/STATUS-log.md`, `docs/arcs/v1/rung-13-PROMPT-2026-11-17.md`).

## Moved from STATUS.md on 2026-11-18 (layers close), word for word

- Rung 15, export, is done: approved by chris 2026-11-17 ("look approved"), shipped as `v0.15.0` (record
  `docs/bbq/2026-11-17-rung-15-export.md`, with its Amendments; ADR-013 amended and ratified). Export opens one Export
  sheet (devices, a remembered folder, an app-wide path template defaulting to `{title}-{device}.png` with presets,
  `/` for folders and a Set box, a text-note tick), and Export again (Ctrl+E) writes with the last choices; both pass
  the one preflight sheet; a completion sheet shows thumbnails, Reveal in folder and a folded MyStyle note
  (`crates/ui/src/export.rs`, `crates/ui/src/export_path.rs`, `crates/ui/src/export_ui.rs`, `crates/ui/src/settings.rs`).
  PNGs carry Software and Title text chunks by default; the Nomad reads them as templates (chris). File format 0.16.0:
  a 480 px thumbnail of the main device in `previews/` (`crates/render/tests/fixtures/v0.15/`). The 29 golden lines
  unchanged, one noted golden added (`crates/render/tests/golden/SHA256SUMS`). Speed on the Pi, 90th percentile: 30.5 ms
  worst (groups page drag, chip); Save gains about 23 ms (41 on A4 paper) (`docs/STATUS-log.md`). Try it: `cargo run
  --release -p quadrule-app -- --look export`.
- 2026-11-17: rung 13 planned (bbq, 14 questions; GUID ids by chris; paper sizes pulled in from v1.1 by chris), split
  13a/13b, built (two Sonnet 5.5 high builders), approved; ADR-016 ratified; large paper pages on the export canvas
  ruled debt for rung 17; one release `v0.13.0`; board re-planned; rung 14 prompt written (`docs/STATUS-log.md`,
  `docs/arcs/v1/rung-14-PROMPT-2026-11-17.md`).
