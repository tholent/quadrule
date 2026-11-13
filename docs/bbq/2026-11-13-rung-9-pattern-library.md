# bbq record — quadrule rung 9: pattern library

2026-11-13 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-9-PROMPT-2026-11-13.md`) · 8 questions · lead:
Opus 5.5 at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 9 row), `docs/arcs/v1/rung-9-PROMPT-2026-11-13.md`,
`docs/decisions/INDEX.md` (ADR-009, ADR-011, ADR-014), `docs/STATUS-debt.md` (rung 6b, 8a, 8b lines),
`docs/STATUS-log.md` (newest entries), `docs/bbq/2026-11-12-rung-8-several-devices.md`, `napkins/quadrule-spec.md`
§5.2 (lines 209, 218), §6.3 (lines 300–330), §8.3 (L16–L19, line 449), `crates/model/src/lib.rs` (`Pattern`:
Lines, Dots, Grid; `PatternRegion` holds only the pattern; `Rhythm` has no type and no major), `crates/layout/src/lib.rs`
(`RhythmPx`; each line or dot is one `Item::Rect`), `crates/ui/src/panels.rs:1327` (the pattern choice),
`crates/lint/src/lib.rs` (L01, L02, L06, L10, L15, L16, L18, L19 built), `crates/format/src/lib.rs`
(`FORMAT_VERSION` 0.8.0).

Measured at the start of this session (2026-11-13, on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=ace7aa1 dirty=no rebuilt=7` (the 0.8.0 version bump not yet compiled; now matches HEAD).

Asked again at session start (AGENTS.md): landscape on the Manta. chris, word for word: "The software on each is
identical and rotation logic is the same. no need to test". Still parked, not re-asked: "Scale vs pixel-locked rhythm
on A5 X" (v1.1, arc Q16); the Label gray (v1.1); text and pattern rotation (v1.1).

## Budget
Share: 16 points (1 point = 1 % of the weekly limit) of a 16-point ceiling, alone on the meter; reserve 0 kept for
chris.
Room: 16 points (the lower of the ceiling and the hard stop less the reading, 90 − 74 = 16, less the reserve).
Written when the weekly meter (resets Saturday 14th, 5 pm) read 74 % (2026-11-13 16:28 UTC, given by chris, not
measured; logged with `spend-ledger.py reading 74 --meter weekly`); that reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. If other sessions share the meter, your share line is your ledger, never a meter percentage. Read the meter
before each batch of dispatches; log a reading only at batch ends and when a stop line is within 5 points. Stop
starting new work at the first of: your ledger reaching 16 points (alone on the meter: the reading minus your start
reading; across Saturday's reset, the points spent before it count against the 16); the meter reaching your start +
16; the hard stop (**90 % until Saturday 14th, 5 pm**, chris, this week only; after the reset 75 % beyond 48 h, 80 % at
24–48 h, 85 % at 12–24 h, 93 % under 12 h). K = 12.2M tokens/point fitted over two pairs in
`docs/budget-calibration.md`, not yet trusted, so trust the meter's movement. In-flight work finishes. Hand off at
~500k of your context. Before handoff, write your row in the calibration log.

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6; the rung 9 prompt): 8.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Set with `budget-select` above.
- **Pattern edges.** Every line and dot whose centre lies in the region is drawn whole (ADR-014).
- **Frozen bytes.** The twelve golden lines in `crates/render/tests/golden/SHA256SUMS` and the fixtures `v0.4/` to
  `v0.7/` stay unchanged; any change that would move them is a stop to ask chris (arc Q20).
- **Profiles.** Every profile lookup that draws or lays out a page goes through `builtin_for(id,
  doc.page.orientation)` (rung 9 prompt).

## Rules

- **Q1 Split.** **One rung, one look** (overruled: the lead recommended 9a patterns / 9b isometric rhythm). Everything
  below lands before chris's look, and ships as `v0.9.0`. Because it is one look over a large rung, the builder commits
  each step as it lands (rung 8 lesson 1), so a stopped session leaves no uncommitted work.
- **Q2 Isometric pattern.** **Horizontal rows**, axes at 0°, 60° and 120° (spec §6.3): horizontal lines one row apart,
  pixel-sharp, with the slanted lines crossing on them. Two styles: **lines** (the triangle grid) and **isometric
  dots** (a dot at each crossing).
- **Q3 Hex pattern.** **Both orientations, as a choice on the region**: Flat top (top and bottom edges horizontal and
  pixel-sharp; the default) and Pointy top (two edges vertical and sharp). A hexagon's height across its flats is a whole
  number of rhythm units, 1 by default (vertical units for Flat top, horizontal units for Pointy top).
- **Q4 Cornell.** **One parametric region**: ruled lines on the rhythm, a cue column on the left (width in rhythm
  units) and a summary band at the bottom (height in units). Its dividers use the Structure role and its ruling Guide.
- **Q5 Music staff.** **Five lines with their own gap, on a rhythm pitch, plus a grand-staff choice.** The gap is the
  region's own (2 mm by default, an 8 mm staff, rounded to whole device pixels); each staff starts on a rhythm row; a new
  staff comes every N rhythm units (3 by default). A **Grand staff** tick pairs the staves (treble and bass) with a
  closer gap between the two of a pair, and draws a brace line joining each pair at the left. Lines use the Guide role.
- **Q6 Major lines.** **The rhythm sets N, a region adjusts it.** The rhythm gains "Major every N" (off by default;
  spec §6.3 line 309). A bound region follows it; each region can turn major lines off or set its own N. Major lines
  use the **Structure** role by default, changeable per region. The rhythm overlay draws major lines stronger.
- **Q7 Isometric rhythm.** The rhythm gains a **Type**: Rectangular (today's) or **Isometric** (one side length;
  spec §6.3 line 304). Under Isometric, snapping, Shift+arrow and drawn sizes use the **rectangular grid of half a side
  across and one row down**, which holds every isometric crossing, so today's snapping code works on it unchanged
  (`crates/layout/src/snap.rs`). The overlay draws the triangles.
- **Q8 Anything else.** No: the standing rules hold (frozen goldens and fixtures, 50 ms at the 90th percentile on the
  Pi, commit as each step lands).

## Owed

None. (After round 1: how the isometric rhythm snaps, asked as Q7.)

## Defaulted

Each settled by the lead without asking; chris can veto any of them at the look.

- **Unbinding a region (own spacing).** Each region gets a **Bound to rhythm** tick (on by default; spec §5.2
  line 218). Unticked, the region has its own horizontal and vertical spacing, typed in any unit (`u` included), resolved
  per device like the rhythm (pixel-locked when the rhythm is), and counted from the region's own top-left corner.
  veto: count an unbound region from the rhythm's origin instead.
- **L17, mixed rhythm (Info).** Fires on an unbound region when, on a device, its resolved spacing in pixels is not
  within half a pixel of a whole multiple (1–10) or a whole fraction (½ to 1/10) of the rhythm unit. One finding per
  region and device, carrying its area. An unbound region's spacing is judged by L17 only, not also by L16.
  veto: a different tolerance, or L16 also judging it.
- **Isometric sizes.** Bound to a Rectangular rhythm, an isometric region's row height is the vertical unit and its
  side is the row height × 2/√3 (not a round number of mm). Under an Isometric rhythm, the side is the rhythm's side
  length and the row height is side × √3/2, rounded to whole pixels when pixel-locked; the half-side snap step is the
  side ÷ 2. veto: tie the side, not the row, to the vertical unit.
- **Slanted lines.** Isometric and hex slanted edges are anti-aliased strokes in the region's role; each slanted
  line runs edge to edge of the region, ending where it meets the region's frame, and a hex edge shared by two
  hexagons is drawn once. Isometric dots use the Dot grid role. veto: none expected.
- **Cornell defaults.** The cue column is 30 % of the region's width and the summary band 20 % of its height, each
  rounded to whole rhythm units, so it adapts to both devices; both editable in units. veto: other proportions.
- **Grand staff.** The gap between the two staves of a pair is 1 rhythm unit; each pair repeats on the staff pitch;
  the brace is a straight vertical line in the Structure role (a curly brace is v1.1 or a starter's own object).
  veto: a curly brace now.
- **Which patterns have major lines.** Lines (every Nth row), Grid (rows and columns), Dots (every Nth dot each way,
  drawn at the major role's width and gray), Isometric (every Nth line on each of the three axes), Cornell (its ruling).
  Hex and Music staff have none (the field is greyed). Counting starts at the rhythm origin, so the origin row is major.
  veto: a different start.
- **Where the settings live.** The pattern choice stays where it is in the inspector (`crates/ui/src/panels.rs:1327`),
  with each pattern's own settings below it (orientation, sizes, Grand staff, major N and role, Bound to rhythm). The
  rhythm's Type, side length and Major every N go in the Page tab with the other rhythm settings. veto: chris's look.
- **New… presets.** One more rhythm preset, "Isometric 5 mm" (Isometric type, 5 mm side). Taste, for chris.
  veto: no preset, or another size.
- **File format 0.9.0.** New `Pattern` variants and `PatternRegion` and `Rhythm` fields, each with a serde default, so a
  0.8 file opens with the same export bytes; a `crates/render/tests/fixtures/v0.8/` fixture proves it through the
  migration chain. veto: none expected.
- **Golden pages.** One new pattern page holding every new pattern and style (isometric lines and dots, hex both ways,
  Cornell, music staff and grand staff, major lines, an unbound region), golden per device in portrait, plus one
  landscape version per device. Each image is looked at by eye before it is blessed (rung 8 lesson 2). The page opens
  with `--patterns-page`. veto: none expected.
- **Speed.** Time dragging and drawing on the pattern page on the Pi (release build, both canvases, strip on, 90th
  percentile, ADR-009), and say before anything crosses 50 ms. Rung 9 rewrites pattern drawing, so it **pays the
  rung 8a debt line** (drawing a pattern region on the shapes page on the chip canvas, 50.8–51.1 ms) in the same code:
  for example by building a pattern's lines as one path instead of one item per line. veto: leave that debt for rung 17.

## Kickoff

This session leads (context well under a third used at the end of the bbq: run here). Opus 5.5 at high effort leads;
on any other model or effort, stop and say so before doing anything. Brief one Sonnet 5.5 high builder from this
record, building on `main` and committing as each step lands: (1) the model, format 0.9.0 and the `v0.8/` fixture; (2)
the pattern geometry and drawing in `crates/layout` (isometric lines and dots, hex both ways, Cornell, music staff and
grand staff, major lines, own spacing), paying the chip-canvas debt; (3) the isometric rhythm type, its snapping grid,
overlay and the New… preset; (4) L17; (5) the inspector and Page tab settings; (6) the pattern page, its goldens and
tests; (7) README and DEVELOPMENT. The lead then judges it (`make check-all`, the golden sums diff, the images by eye),
measures on the Pi, and shows chris the window for his look. After his approval: version `0.9.0`, notes in
`docs/arcs/v1/release-notes/v0.9.0.md`, tag `v0.9.0` (the tag push publishes), re-plan the board, write the next lead's
prompt, and update `STATUS.md`, `docs/STATUS-log.md`, `docs/STATUS-debt.md` and `docs/budget-calibration.md`.

## Amendments

2026-11-13, chris at the rung 9 look (release build on the Pi, `--patterns-page`), word for word: "a couple items.
First, things look really nice overall. Second, the isometric grid and dot pattern is rotated 90 degrees from what I
expected." Then three questions, answered:

- **Q2 amended — isometric turned 90°, everywhere.** Asked "Should that replace horizontal rows, or be a choice on the
  region?", answered "Replace, everywhere (Recommended)". The axes are at 30°, 90° and 150°: vertical lines one column
  apart, pixel-sharp, with the slanted lines crossing on them, for isometric lines, isometric dots and the isometric
  rhythm. Bound to a Rectangular rhythm, the column is the horizontal unit; under an Isometric rhythm, the column is
  side × √3/2 (whole pixels when pixel-locked). Spec §6.3 has the dated note.
- **Q7 amended to match.** The isometric rhythm's snap grid is one column across × half a side down.
- **Hex edges (Defaulted "Slanted lines") — cut at the frame.** Asked "Hex edges at the region's border … Which do you
  want?", answered "Cut at the frame (Recommended)". ADR-014 amended the same day.
- **Major dots (Defaulted "Which patterns have major lines") — one pixel bigger.** Asked "What should a major dot be?",
  answered "One px bigger, major gray (Recommended)": a major dot is the Dot grid dot's size in device pixels plus 1,
  in the major role's gray. The first build drew them at Structure's 2 px, smaller than the 3 px ordinary dot, because
  the Defaulted line said "at the major role's width" (the lead's brief).

2026-11-13, chris at the second look (after the three changes above were built, `1dfebaa`, `5fff8c9`), word for word:
"approved. everything looks great". Rung 9 is done and ships as `v0.9.0`. The two leftovers shown to him (a cut
line's stroke reaching up to about 1.5 px past the frame; the isometric overlay not drawing the half-side snap rows)
were approved as built.
