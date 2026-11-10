# bbq record — quadrule rung 3: the first editor

2026-11-10 · untyped bbq · 12 questions · lead: Opus 5.5, high effort.

Grounding: `STATUS.md` (Next: "Rung 3 is chris's to choose"), `docs/STATUS-log.md` (last entry, CI run 42720462671),
`docs/STATUS-debt.md`, `napkins/quadrule-spec.md` §5, §6, §15, §16, `docs/bbq/2026-11-09-rung-2-viewer.md`,
`spikes/render/FINDINGS.md`, `crates/model/src/lib.rs`, `AGENTS.md`.

Measured at the start of this session (2026-11-10, on the Pi): clean tree at HEAD `dc3bc9f`;
`CARGO_BUILD_JOBS=2 cargo build --workspace` → `Finished`, so the build on disk matches HEAD.

Asked again at session start (not counted): the Label gray 153 vs dark grey pen 157 (L04). chris: "Leave it open".

## Rules

- **Q1 Count.** 12.
- **Q2 Lead.** Opus 5.5 at high effort.
- **Q3 Scope.** The first editor: select objects, an inspector, undo/redo, Save. (Q5 moves dragging to rung 4.)
- **Q4 Canvas — overruled.** Recommended was "keep export pixels, chip canvas only if too slow"; chris chose:
  **both canvases, switchable** — the export-pixels canvas (vello_cpu, as rung 2) and the graphics-chip canvas
  (vello on wgpu) with its ruled vello_cpu fallback on weak chips.
- **Q5 Split.** Rung 3: select, inspector, undo/redo, Save (and Q4's canvases). Rung 4: drag to move and resize,
  with snapping. chris approves each look before the next starts.
- **Q6 Saving.** Unlimited undo/redo, Save and Save As, and autosave to a recovery file with a restore offer after
  a crash (napkin §5.6). No history panel.
- **Q7 Canvas default.** Export pixels is the default; the graphics-chip canvas is a View toggle, measured on the
  Pi now (CI builds it); a desktop-card check later, when chris has one.
- **Q8 Inspector.** The napkin's tabs (§6.1), basic fields only: the Page menu when nothing is selected (margins,
  rhythm, background); Arrange, Style, Text, Constraints for an object. Advanced disclosures later. Lint becomes
  its own tab.
- **Q9 Selecting.** Click an object, Shift-click to add; a Layers panel (tree, visibility, lock); arrow keys nudge
  1 device pixel, Shift+arrow one rhythm unit. No marquee (rung 4).
- **Q10 Units.** One shared number field that reads every unit of napkin §5.3: `mm`, `pt`, `px`, `in`, `%`, `u`.
- **Q11 Flag — overruled.** Recommended was a Cargo feature `editor`; chris chose **no flag**: editing is on in the
  app build. This waives AGENTS.md's "land a new feature switched off behind a flag" for rung 3 only. The look is
  still shown to chris before it is called done.
- **Q12 Anything else to protect or avoid?** Keep the export goldens fixed: no change to export bytes; rung 1's
  goldens (`crates/render/tests/golden/SHA256SUMS`) and the stock-grid test must still pass unchanged.

## Owed

(empty: all moved to Defaulted)

## Defaulted

- **Size — grown, said out loud:** rung 3 as scoped is about 3–5 sessions on the Pi; Q4 (both canvases) adds
  1–2 of those. veto: move the chip canvas to rung 4.
- **The chip canvas on the Pi will be slow:** spike R12 measured vello on the V3D chip at about 392 ms a Manta page,
  about 30× slower than vello_cpu (`spikes/render/FINDINGS.md`). So on the Pi the toggle will mostly show the
  fallback. The fallback rule: if the chip canvas cannot start (no adapter, limits refused) or a measured frame
  takes over 100 ms, the canvas draws with vello_cpu at screen resolution and the status bar says so. vello 0.11
  shares wgpu 30.0.1 with eframe 0.37 (R14), so no second wgpu. The chip canvas never makes an export (ADR-003).
  veto: the 100 ms line.
- **Commands and undo:** in `model` (napkin §15.1), headless: every edit is a command that can be done and undone;
  a `History` with unlimited depth; typing in one field merges into one step until the field loses focus. Locked
  objects and locked layers refuse edits. Ctrl+Z / Ctrl+Shift+Z (Cmd on macOS). veto: none expected.
- **Delete and Duplicate:** included (Delete key, Ctrl+D), since rung 3 has no Insert tools and an editor that can
  only change what exists needs at least these. veto: leave them to the drawing-tools rung.
- **No New, no Insert:** rung 3 edits an opened `.quad` or the sample page. A blank New document and the Insert ▾
  tools come with a later rung. veto: add New (a blank page with the napkin's 6 mm rhythm).
- **Save:** Save (Ctrl+S) and Save As, by the system file dialog (`rfd`, as rung 2). Saving writes text outlines
  for every profile, as ADR-002 needs, through the existing `format` code. The window title shows the file name
  and a dot when there are unsaved changes; closing or opening another file with unsaved changes asks
  Save / Don't save / Cancel. The file format version changes only if a field is added; unknown fields still round
  trip (`Extra`). The sample page saves as a new file (Save As). veto: none expected.
- **Autosave:** every 30 seconds while there are unsaved changes, and when the window loses focus, to one recovery
  file per open document in the user's data folder (the `directories` crate); deleted after a clean Save or close.
  On start, if a recovery file is there, offer Restore / Discard. The napkin leaves N open; 30 s is chosen here.
  veto: the 30 s value.
- **Edits change the shared design:** the model stores each object's frame for the primary profile, in mm. Rung 3
  edits that base design; per-profile overrides (the Overrides tab) are not in rung 3. When the profile shown is not
  the primary, a line in the inspector says edits change every profile. Nudge by 1 device pixel uses the shown
  profile's pixel size (25.4 / ppi mm). veto: lock editing to the primary profile.
- **Selecting, in detail:** a click picks the topmost visible, unlocked object whose laid-out shape on the shown
  profile is under the pointer, with a few screen pixels of slack for thin lines; Escape clears. The selection is
  drawn as an outline over the canvas (resize handles come with rung 4). The outline is an overlay; it never
  exports. veto: none expected.
- **Layers panel:** the left sidebar of napkin §6.1 arrives now (rung 2 Q4 left its place free): a tree of layers
  and their objects, an eye and a lock on each row, double-click to rename, selection kept in step with the
  canvas. Reorder by Move up / Move down (buttons and keys); drag-to-reorder in the panel waits for rung 4.
  veto: drag-to-reorder now.
- **Inspector fields:** Page menu: title, background gray, margins (four, linked by default), rhythm (horizontal
  and vertical unit, linked by default; origin; resolution), safe-zone overlay. Arrange: x, y, width, height.
  Style: line role, or Custom width and gray; for a pattern region, its pattern (lines, dots, grid). Text: content,
  family, weight, size, line height, letter spacing, align. Constraints: horizontal and vertical pickers (§5.4).
  The per-profile resolved value is shown beside the rhythm unit (§6.3, "5 mm → 59 px (Manta)"). veto: any field.
- **Units, in detail (Q10):** the parser lives in `layout` (§15.1 puts unit conversion there), headless and
  tested. `px` means device pixels of the shown profile; `%` is of the page (no groups yet); `u` is the rhythm
  unit along the field's axis. Fields show values in the document's display unit. veto: none expected.
- **Live update:** each command re-lays-out and re-renders the shown profile and re-runs lint. veto: none expected.
- **Goldens (Q12):** the export code path is not changed; if a change would alter export bytes, stop and ask chris.
- **Words and access:** every new string through Fluent (`crates/i18n/`); every new control named for screen
  readers (rung 2's window tests check it). veto: none expected.
- **Tests:** headless, one `CHECK` line each: commands and undo, unit parsing, hit testing, save round trip
  (open → edit → save → reopen → same document), autosave and recovery, export bytes unchanged after an
  edit-and-undo. Widgets with `egui_kittest` where it needs no graphics chip. veto: none expected.
- **Landing:** commits straight to `main` (chris's amendment of 2026-11-09 in `AGENTS.md`); the bbq skill's default
  of a branch and PR is overruled by the project rule. CI checked on each push. veto: one branch and PR for rung 3.
- **spikes/:** untouched (rung 1 Q8 stands); the chip canvas may copy ideas from `spikes/render/examples/canvas`.
- **Checks:** `cargo test --workspace`, `cargo test -p quadrule-app -p quadrule-ui`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`; build with `CARGO_BUILD_JOBS=2`.

## Kickoff

Run on **Opus 5.5 at high effort**; on any other model or effort, stop and say so before doing anything. Read
`AGENTS.md`, `docs/decisions/` ADR-001 to ADR-003, `docs/bbq/2026-11-09-rung-2-viewer.md` and this record. Prove the
build matches HEAD. Build rung 3 on `main` in this order: commands and undo in `model`; the unit parser in
`layout`; hit testing and selection in `ui`; the layers panel; the inspector tabs (Page, Arrange, Style, Text,
Constraints, Lint); nudge, Delete, Duplicate; Save, Save As and the unsaved-changes prompt; autosave and recovery;
then the graphics-chip canvas toggle with its fallback, measured on the Pi's display. Keep the export goldens
unchanged. Show chris the window before calling the look done. Update `STATUS.md`, `docs/STATUS-log.md` and
`docs/STATUS-debt.md` at the end of each session.

## After the build (2026-11-10)

Added after the build; the rules above are unchanged. chris's answers, word for word:
- The look, first try: "the window looks really good so far. One thing I would like is to move the layers pane to
  the bottom of the info pane on the right. I would like the left hand side to be used for a toolbar in the
  future." Built: the layers sit in a resizable part at the bottom of the right panel; the left side is empty.
  Replaces Defaulted "Layers panel" ("the left sidebar of napkin §6.1 arrives now") on where the panel goes.
- The look, after the move: "the right pane looks good now, I approve of the look."
- `u` in fields and Shift+arrow use the rhythm unit as the device on screen resolves it (a choice made inside
  Q10): "I approve the u choice, as far as I understand it."
- The Label gray 153 vs dark grey pen 157 (asked again at the end of the build session): "leave the label gray issue open until
  v1.1".
