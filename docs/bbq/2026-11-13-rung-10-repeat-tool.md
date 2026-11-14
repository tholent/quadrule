# bbq record — quadrule rung 10: the repeat tool

2026-11-13 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-10-PROMPT-2026-11-13.md`) · 8 questions · lead:
Opus 5.5 at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 10 row), `docs/arcs/v1/rung-10-PROMPT-2026-11-13.md`,
`docs/decisions/INDEX.md` (ADR-009, ADR-011, ADR-015), `docs/STATUS-debt.md` (rung 4b, 7a, 7b, 8a, 9 lines),
`docs/bbq/2026-11-13-rung-9-pattern-library.md`, `napkins/quadrule-spec.md` §6.3 (line 322), §6.4 (lines 333–343),
§8.3 (L16, L17, lines 453–454), `crates/model/src/lib.rs` (`Object`, `Override`, `ObjectKind` with seven kinds,
`GroupShape`, `PatternRegion.bound`, `Rhythm::steps_mm`), `crates/layout/src/tree.rs` (groups see-through to
constraints), `crates/layout/src/units.rs` (`parse_length`, `u` = the rhythm unit), `crates/lint/src/lib.rs` (L16 and
L17 built, neither knows repeats), `crates/ui/src/panels.rs:158` (Group and Ungroup buttons), `crates/format/src/lib.rs`
(`FORMAT_VERSION` 0.9.0).

Measured at the start of this session (2026-11-13, on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=da6d45c dirty=no rebuilt=0`.

Asked again at session start (AGENTS.md): nothing open. Still parked, not re-asked: "Scale vs pixel-locked rhythm on
A5 X" (v1.1, arc Q16); the Label gray (v1.1); text and pattern rotation (v1.1). Rung 9's two leftovers were approved
as built and rung 10 does not touch them.

## Budget
Share: 16 points (1 point = 1 % of the weekly limit) of a 16-point ceiling, alone on the meter; reserve 0 kept for
chris.
Room: 16 points. chris, word for word: "go for 16 points and ignore the cap for this week", so this week's 90 % hard
stop does not apply to this rung.
Written when the weekly meter (resets Saturday 14th, 5 pm EST) read 82 % (2026-11-13 19:32 UTC, given by chris, not
measured: "Still about 82 %"; logged with `spend-ledger.py reading 82 --meter weekly`); that reading is not yours. On
pickup: take the reading (or ask chris for it), name your meter, note your start reading, and cap the share at your
hard stop minus your reading. Read the meter before each batch of dispatches; log a reading only at batch ends and
when a stop line is within 5 points. Stop starting new work at the first of: your ledger reaching 16 points (alone on
the meter: the reading minus your start reading; across Saturday's reset, the points spent before it count against
the 16); the meter reaching 98 %; after the reset, the hard stop (75 % beyond 48 h, 80 % at 24–48 h, 85 % at 12–24 h,
93 % under 12 h). K = 12.2M tokens/point fitted over two pairs in `docs/budget-calibration.md`, not yet trusted, so
trust the meter's movement. In-flight work finishes. Hand off at ~500k of your context. Before handoff, write your row
in the calibration log.

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6; the rung 10 prompt): 8.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Set with `budget-select` above.
- **Live object.** A repeat is live: editing the source updates every copy; Expand turns it into plain objects
  (spec §6.4).
- **Frozen bytes.** The sixteen golden lines in `crates/render/tests/golden/SHA256SUMS` and the fixtures `v0.4/` to
  `v0.8/` stay unchanged; any change that would move them is a stop to ask chris (arc Q20).
- **Profiles.** Every profile lookup that draws or lays out a page goes through `builtin_for(id,
  doc.page.orientation)` (rung 10 prompt).

## Rules

- **Q1 Split.** **One rung, one look.** Everything below lands before chris's look and ships as `v0.10.0`. The builder
  commits each step as it lands (rung 8 lesson 1).
- **Q2 Source.** **Inside the repeat, like a group.** The repeat holds its source (one object, or a group) as its
  only child. A click on any copy selects the whole repeat; a double-click on any copy enters it and selects the
  source; every copy follows each edit live; Escape leaves (rung 7a's enter and leave). The layers panel shows the
  repeat as a tree row with the source under it.
- **Q3 Count.** **A choice on the repeat: a fixed Count, or Fill its box.** In Fill, the repeat's box is resolved like
  any object's (its constraints against the page, and its per-device override), and as many copies are drawn as fit
  in it, so a box pinned to the margins gives each device its own number of copies. Dragging the box's handles in Fill
  adds or removes copies. In Count mode the box is the copies' outer box.
- **Q4 Spacing.** **The default pitch is the source's size rounded up to whole rhythm steps** on each axis (a 1-unit
  source gets 1u, a 2.3-unit source 3u): copies never overlap and land on the rhythm. It is called **Auto** and follows
  the source as it is edited. Pitch (centre to centre) or Gap (edge to edge), in any unit, stay editable per axis.
  This amends spec §6.3 line 322 ("Pitch defaults to one rhythm unit"); a dated note goes in the spec.
- **Q5 Alternation.** **Each rule picks what it counts: Columns, Rows or Cells** (cells in reading order, left to
  right then down; a linear repeat counts copies). A rule is "every Nth" (with a start) or a list of numbers ("3, 7"),
  and does one thing: **Skip**, set the copy's **line role** (every stroke in that copy), or set its **fill role**.
  Rules apply in list order, a later one winning.
- **Q6 Leftover.** **Start or Centre, a choice on the repeat, Start by default.** Start lays the copies from the box's
  top-left on the rhythm; Centre splits the spare space both sides. Both keep the pitch as set; no Spread.
- **Q7 Expand.** **One group, named after the repeat**: each copy a real copy of the source with new ids,
  alternation baked in, skipped copies left out; with a group source, each copy is a group inside it. One undo step.
- **Q8 Anything else.** No: the standing rules hold (frozen goldens and fixtures, 50 ms at the 90th percentile on the
  Pi on a repeat page, commit as each step lands).

## Owed

None.

## Defaulted

Each settled by the lead without asking; chris can veto any of them at the look.

- **Modes.** Linear (Across or Down) and Grid (columns × rows). No other directions; the repeat itself has no rotate
  handle (a turned source is fine: copies of a turned line make a hatch). veto: a linear repeat at any angle.
- **The source's place.** The source sits in the first cell (top-left, or the first after Centre's spare space). Its
  shape and size are laid out as designed on the primary device and then moved rigidly into each cell, on every device,
  so every copy is the same shape on every device; only the box and the count change per device. Moving the source while
  entered moves the whole repeat. The source has no overrides of its own; the repeat's box carries them (rung 8's
  override of a place, size and eye). veto: let the source resolve its own constraints per device.
- **Whole pixels.** Each axis's pitch is resolved per device to whole device pixels, so every copy is pixel-identical to
  the source on e-ink. veto: none expected.
- **Spacing storage and the rhythm.** A spacing typed in `u` is kept in rhythm units and follows rhythm changes
  (bound); a spacing typed in any other unit is kept in mm (unbound). Auto is always bound. Under an isometric rhythm
  the steps are `Rhythm::steps_mm()` (a column across, half a side down). veto: a Bound tick, as on pattern regions.
- **L16 and L17 learn repeats.** L16 (Warn) fires once per repeat when a bound pitch is not a whole multiple of a
  rhythm step or subdivision, or when the repeat's box position is off the rhythm (as for any object). L17 (Info) fires
  once per repeat and device when an unbound pitch is not within half a pixel of a whole multiple (1–10) or a whole
  fraction (½ to 1/10) of the rhythm step, as rung 9's L17 does for regions. An unbound repeat's pitch is judged by L17
  only. veto: a different tolerance.
- **Other lint rules on copies.** L06 (safe zones) and L10 (off the page) judge every drawn copy and give one finding
  per repeat and device, saying how many copies; the other rules (L01, L02, L15 …) judge the source once, and L15 the
  repeat's box. veto: one finding per copy.
- **Groups and nesting.** A repeat may sit in a group (see-through: its box keeps its own constraints against the page,
  ADR-015). A group may be the source. A repeat may be the source of another repeat (a grid of grids falls out of the
  tree). A repeat draws at most 5,000 copies (nested ones multiplied); past that it draws the first 5,000 and the
  status bar says so. veto: forbid nesting.
- **Snapping.** The repeat's box snaps and is snapped to like any object. Each drawn copy's box also counts for point
  snap and smart guides, so other objects line up with copies. In Fill, the box's handles snap as any resize does; the
  count follows the box. veto: only the repeat's box counts.
- **Alternation details.** Numbers count from 1 (the source is copy 1, column 1, row 1). "Every Nth" with start S
  hits S, S+N, S+2N…; the start defaults to N. A line-role rule changes every stroke in that copy (lines, shape
  outlines, pattern regions) to the role; text keeps its own. A fill-role rule changes the fill of shapes that have one
  and adds none. Skipping copy 1 hides the source's cell but the source stays editable from the layers panel. veto:
  numbers from 0, or a fill rule that adds a fill.
- **Making a repeat.** A **Repeat** button beside Group and Ungroup (the toolbar and the Arrange tab,
  `crates/ui/src/panels.rs:158`) turns the selection into a repeat in one undo step; several selected objects are
  grouped first and the group becomes the source. A new repeat is Linear, Across, Count 4, Auto pitch, Start. No
  keyboard shortcut (rung 17). veto: a shortcut now, or different defaults.
- **The Repeat tab.** A repeat's settings live in a **Repeat** tab in the inspector, shown when a repeat is selected
  (as the Text tab is for text): mode, direction, Count or Fill, count fields, pitch or gap per axis, Start or Centre,
  the alternation rules (add, remove, reorder), and **Expand**. veto: chris's look.
- **Expand on other devices.** Expand changes no pixel on the primary device. Each copy keeps the source's constraints,
  moved to its cell, so on a secondary device a Fill repeat's extra or missing copies are not kept; the status bar says
  so when the counts differ. veto: write per-device overrides so nothing moves.
- **Undo and live editing.** Every change to a repeat or its source is one undo step, as every edit is (ADR-010);
  copies are never stored, only drawn.
- **Align and distribute.** A repeat aligns and distributes by its box, as any object (rung 7b); entered, the source
  aligns as the only child. veto: none expected.
- **File format 0.10.0.** A new `ObjectKind::Repeat` holding the source and its settings, each new field with a serde
  default; a `crates/render/tests/fixtures/v0.9/` fixture proves a 0.9 file opens through the migration chain with the
  same export bytes. veto: none expected.
- **Golden pages.** One new repeat page per device in portrait (`repeats_*`): a linear Across repeat, a linear Down
  repeat, a habit-tracker grid with every 7th column in Structure and a skipped row, a Fill repeat pinned to the margins
  with Centre (a different count on each device), a group source, a turned-line hatch and an unbound (mm) pitch. Each
  image is looked at by eye before it is blessed (rung 8 lesson 2). The page opens with `--repeats-page`. Landscape is
  covered by a test that a landscape repeat exports as the exact quarter turn (as rung 8b's `landscape.rs` does), not a
  golden. veto: add landscape goldens.
- **Speed.** Time dragging and drawing on the repeat page on the Pi (release build, both canvases, strip on, 90th
  percentile, ADR-009), and say before anything crosses 50 ms. veto: none expected.
- **Debt.** Pay the debt in code the rung touches (arc Q17), at least: none of the rung 4b, 7b or 9 lines is required
  by the rung, so each paid line is noted in the log, and new debt goes in `docs/STATUS-debt.md` as a "Rung 10, owed"
  line.

## Kickoff

This session leads (context well under a third used at the end of the bbq: run here). Opus 5.5 at high effort leads;
on any other model or effort, stop and say so before doing anything. Brief one Sonnet 5.5 high builder from this
record, building on `main` and committing as each step lands: (1) the model, format 0.10.0 and the `v0.9/` fixture;
(2) the layout of copies in `crates/layout` (count and fill, pitch and gap, Auto, Start and Centre, alternation, whole
pixels, nesting cap) and drawing on both canvases and the export; (3) editing: making a repeat, picking, entering,
live source edits, Expand, the layers tree, overrides, snapping to copies; (4) L16, L17, L06, L10 for repeats; (5) the
Repeat tab and the buttons, all strings through Fluent; (6) the repeat page, its goldens and tests; (7) README,
DEVELOPMENT and the spec §6.3 note. The lead then judges it (`make check-all`, the golden sums diff, the images by
eye), measures on the Pi, and shows chris the window for his look. After his approval: version `0.10.0`, notes in
`docs/arcs/v1/release-notes/v0.10.0.md`, tag `v0.10.0` (the tag push publishes), re-plan the board, write the next
lead's prompt, and update `STATUS.md`, `docs/STATUS-log.md`, `docs/STATUS-debt.md` and `docs/budget-calibration.md`.

## Amendments

2026-11-13, chris before the look, on the lead's report that drawing on the graphics-chip canvas on the repeat page
was 49.6–50.1 ms at the 90th percentile: asked "Fix it now, in rung 10, leave it as debt, or look first?", answered
"Fix it now, in rung 10 (Recommended)". Built in `fa7ff7e` (the chip keeps the unchanged part of the page in its own
texture; draw 23.6 ms), judged by the lead (`docs/STATUS-log.md`).

2026-11-13, chris at the look (release build on the Pi, `--repeats-page`), word for word: "approved". Rung 10 is done
and ships as `v0.10.0`, with every Defaulted line above as built and the builder's deviations listed in the "Rung 10,
owed" line of `docs/STATUS-debt.md`.
