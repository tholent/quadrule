# bbq record — quadrule rung 5: New and Insert

2026-11-10 · untyped bbq ("bbq next steps") · 12 questions · lead: Sonnet 5.5, high effort (tier-select's row
"spec'd building", as rungs 4a and 4b).

Grounding: `STATUS.md` (Next: "chris's look at rung 4b"), `docs/STATUS-log.md` (last entries: rung 4b, Makefile,
README, badges), `docs/STATUS-debt.md`, `docs/bbq/2026-11-10-rung-3-first-editor.md` (Defaulted "No New, no
Insert"), `docs/bbq/2026-11-10-rung-4-drag-and-snap.md`, `napkins/quadrule-spec.md` §4, §5.2, §6.1, §6.3, §6.4,
§13, §16, `crates/model/src/lib.rs` (`ObjectKind`: Line, Text, Pattern; `Pattern`: Lines, Dots, Grid; `LineRole`),
`crates/model/src/sample.rs`.

Measured at the start of this session (2026-11-10, on the Pi): `make build-proof` → `BUILD head=302d9ef dirty=no
rebuilt=0`. `gh run list --limit 4`: CI run 42864690998 (the last code commit, `934852f`) success; the coverage
workflow's first run, 42868546083, success.

Asked again at session start, still open (not part of this rung): the GitHub description and topics (chris to run
the `gh repo edit` line in `docs/STATUS-log.md` with a token that can, or set them on the repo page); "Scale vs
pixel-locked rhythm on A5 X" (`docs/STATUS-debt.md`).

## Rules

- **Q1 Count.** 12.
- **Q2 Type.** Untyped, the next rung.
- **Q3 Lead.** Chosen by tier-select (Q11 settles it).
- **Q4 Rung 4b look.** Approved by chris (2026-11-10, his answer to "Is rung 4 done?": "Approved, rung 4 done").
  Rung 4 is done.
- **Q5 Scope.** Rung 5 is File → New plus a tools bar on the left that inserts the three kinds that exist: line,
  text box, pattern region. No new object kinds (rectangle, ellipse, polygon, groups and repeat come later).
- **Q6 Flag.** No flag, as rungs 3 and 4: on in the app build. This waives AGENTS.md's "land a new feature
  switched off behind a flag" for rung 5 only. chris still sees the look before it is called done.
- **Q7 Landing.** Commits straight to `main` (chris's amendment of 2026-11-09 in `AGENTS.md`), CI checked on each
  push.
- **Q8 New.** A small dialog: pick the device (Manta or Nomad) and a rhythm preset, then a blank page with
  margins.
- **Q9 Gesture.** Drag on the canvas to draw, with all snapping on (drawn sizes snap to whole rhythm units,
  napkin §6.3); a plain click drops one at a default size.
- **Q10 After drawing.** The tool goes back to Select. Double-clicking a tool keeps it on for several objects.
- **Q11 Lead.** Sonnet 5.5 at high effort.
- **Q12 Anything else to protect or avoid?** (1) Keep the export goldens fixed: rung 1's goldens
  (`crates/render/tests/golden/SHA256SUMS`) and the stock-grid test pass unchanged; if a change would alter export
  bytes, stop and ask chris. (2) Leave the left bar room to grow: lay it out so rectangle, ellipse, polygon and
  repeat can join later without a redesign. ("Stay smooth on the Pi" was offered and recommended but not picked, so
  there is no stop line for drawing speed; see Defaulted.)

## Owed

(empty)

## Defaulted

- **Size, said out loud:** about 2–3 sessions on the Pi. Step 1 (New) about 1; steps 2–3 (tools bar, drawing)
  about 1–2. veto: split into 5a (New) and 5b (Insert) with a look check between.
- **New: where and what it replaces:** File → New and Ctrl+N (Cmd+N on macOS). The app has one document per
  window, so New replaces the open document after the unsaved-changes prompt, as Open does (`crates/ui/src/view.rs`, `open_file` and the `app-unsaved-*` prompt).
  The new document has no file until Save (Save opens Save As). veto: a new window instead.
- **New dialog fields:** Device (Manta, Nomad; the last one picked is remembered between runs, Manta at first);
  Rhythm, a preset list from napkin §6.3: Handwriting 6 mm (the default), Narrow ruled 1/4 in (6.35 mm), College
  ruled 9/32 in (7.14 mm), Wide ruled 11/32 in (8.73 mm), 5 mm grid, 1/4 in grid; a Title field ("Untitled").
  Orientation portrait (the only one with measured safe zones). Document type Template (Wallpaper is v1.1, napkin
  §4). The ruled-paper inch values are mine, from US ruled-paper sizes, for chris to veto. veto: the preset values,
  or add a free "Custom" field.
- **New page margins:** the margins clear the chosen device's docked left, right and top toolbars and the footer
  (from the profile's safe zones, `crates/profiles/data/`), rounded up to whole rhythm units, so a blank page passes
  lint L06. This avoids the sample page's problem in `docs/STATUS-debt.md` (8 mm margin narrower than a docked
  toolbar). Rhythm origin at the top-left margin corner, pixel-locked (napkin §6.3 defaults). veto: one fixed margin
  for all devices.
- **The tools bar:** a narrow vertical bar on the left edge of the window (the space rung 3 kept for it, rung 3 record
  "After the build"), icon buttons stacked top to bottom: Select, Line, Text, Pattern. Each has a tooltip with its
  name and shortcut, a screen-reader name, and a Fluent string. Laid out as a list so later tools are one more
  entry (Q12). Shortcuts: V Select, L Line, T Text, P Pattern; Escape returns to Select (and cancels a drag in
  progress, as now). Shortcuts are ignored while a text field has focus. veto: the letters, or put Insert ▾ in a
  top toolbar as napkin §6.1 draws it.
- **Drawing (Q9):** the drag uses rung 4's snap engine (`crates/layout/src/snap.rs`) and its switches; Ctrl turns
  snapping off for that drag; Escape drops it; the shape is drawn live and lands as one undo step on release. The new
  object goes into the layer of the current selection, else the top unlocked layer (the paste rule, rung 4 Defaulted
  "Copy and paste"), and becomes the selection. A drag shorter than the snap reach counts as a click. Drawing on a
  locked layer is refused with a status-bar note. veto: none expected.
- **Per kind:**
  - **Line:** drag from one end to the other. Shift keeps it horizontal or vertical (45° is left out: a line cannot
    point up and to the right yet, `docs/STATUS-debt.md`, rung 4a). Click: a horizontal line 10 rhythm units long
    from the click point. Role Guide.
  - **Text box:** drag sets its width (text boxes grow downward, `crates/model/src/lib.rs` `TextBox`); click: 10
    units wide. Font Atkinson Hyperlegible Next (`crates/model/src/sample.rs` `ATKINSON`), role Text, content
    "Text", with the Text tab opened and its content field focused and selected so typing replaces it. veto: role
    Label.
  - **Pattern region:** drag sets its frame; click: a region filling the page margins (the usual case for a
    template). Pattern Lines, role Guide, spacing bound to the rhythm (Dots use role Dot grid if switched in the
    inspector, as the sample does). veto: Dots by default.
- **After drawing (Q10):** back to Select; double-click a tool to keep it (a small mark on the button shows it is
  kept); Escape or V ends it. veto: none expected.
- **Drawing speed:** not a stop line (Q12). Measure once on the Pi in a release build while dragging out a pattern
  region at Fit zoom on both canvases, and write the 95th percentile in `docs/STATUS-log.md`. veto: make 50 ms a stop
  line again.
- **File format:** no change expected (New writes the existing fields). If one is needed, stop and say so before
  adding it. veto: none expected.
- **Words and access:** every new string through Fluent (`crates/i18n/`); every new control named for screen
  readers. veto: none expected.
- **Tests:** headless, one `CHECK` line each, each printed starting with a newline: New gives the chosen device,
  rhythm and margins and passes lint with no L06 finding on both devices; each preset's resolved px per device; draw
  each kind by drag and by click (frame, role, layer, selection); a draw is one undo step; Shift on a line; Escape
  drops a draw; a locked layer refuses; the tool returns to Select, and a kept tool stays; shortcuts ignored in a text
  field; export bytes unchanged after New, draw and undo. Widgets with `egui_kittest` where it needs no graphics chip.
  veto: none expected.
- **Where the code goes:** the blank-document builder in `model` (beside `sample.rs`), the margin maths in `layout`
  (it knows the profiles' safe zones), insert commands in `model::edit`, the dialog and tools bar in `ui`
  (`crates/ui/src/panels.rs` or a new `tools.rs`), the drawing gesture beside `crates/ui/src/drag.rs`. veto: none
  expected.
- **README and DEVELOPMENT.md:** update the README's "a page cannot yet be designed from scratch" line and what works
  today, and DEVELOPMENT.md's crate map if a file is added (AGENTS.md: keep both true). veto: none expected.
- **spikes/:** untouched (rung 1 Q8 stands).
- **Checks:** `make check-all` (what CI runs), and count checks with `cargo test --workspace --no-fail-fast --
  --nocapture | grep -o 'CHECK crate=[^ ]* name=[^ ]*' | wc -l`; build with `CARGO_BUILD_JOBS=2`.

## Kickoff

Run on **Sonnet 5.5 at high effort**; on any other model or effort, stop and say so before doing anything (read the
effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, `docs/decisions/` ADR-001 to ADR-003, the rung 4 record
`docs/bbq/2026-11-10-rung-4-drag-and-snap.md` and this record (`docs/bbq/2026-11-10-rung-5-new-and-insert.md` on
`main`). Prove the build matches HEAD (`make build-proof`). Build **rung 5** on `main` in this order: (1) File → New
with its dialog (device, rhythm preset, title), the blank-document builder and margins that clear the docked
toolbars, with tests; (2) the left tools bar (Select, Line, Text, Pattern) with shortcuts, laid out to take more tools
later; (3) drawing each kind by drag and by click with rung 4's snapping, one undo step each, back to Select after
(double-click keeps a tool). Keep the export goldens unchanged; stop and ask chris if any export byte would change.
Measure drawing once on the Pi in a release build on both canvases and log the 95th percentile. Update `README.md`
and `DEVELOPMENT.md`. Show chris the window and stop for his look. Update `STATUS.md`, `docs/STATUS-log.md` and
`docs/STATUS-debt.md` at the end of each session.
