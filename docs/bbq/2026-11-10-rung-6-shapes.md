# bbq record — quadrule rung 6: shapes

2026-11-10 · untyped bbq (the next rung of arc v1, "let's kick off rung 6") · 8 questions · lead: Opus 5.5 at high
effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `STATUS.md`, `docs/arcs/v1/README.md` (rung 6 row), `docs/arcs/v1/v1-PROMPT-2026-11-10.md` (Q2, Q6, Q10,
Q17, Q19, Q20), `docs/decisions/INDEX.md`, `docs/bbq/2026-11-10-rung-5-new-and-insert.md`, `docs/STATUS-debt.md`
(rung 4a and rung 5 line-direction debt), `napkins/quadrule-spec.md` §5.2, §6.1, §6.5, §8.2, §8.3, §10.2,
`crates/model/src/lib.rs` (`Object`, `Frame`, `Style`, `LineRole`, `ObjectKind`: Line, Text, Pattern; no rotation,
opacity or fill), `crates/layout/src/lib.rs` (`Item`: Rect, Stroke, Fill), `crates/format/src/lib.rs`
(`FORMAT_VERSION` 0.4.0), `docs/device-measurements/supernote.a6x2/calibration.md` (dash readings).

Measured at the start of this session (2026-11-10, on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=cda78e0 dirty=no rebuilt=2`.

Done before the bbq (kickoff step): the existing sheets tested grays only on thin lines and small dots, so a **grays
sheet** was added for the clean gray levels: flat patches G1–G15 (0, 17 … 238), G16–G30 (8, 25 … 246) and a ramp GR
(`crates/xtask/src/calib.rs`, `docs/calibration.md`, check `calib_gray_patches`; commit `dcf5458`). chris has the
hardware checklist (grays sheet on both devices; landscape check; landscape toolbar screenshots).

Asked again at session start: "Scale vs pixel-locked rhythm on A5 X" stays parked until v1.1 (arc Q16); the Label
gray stays open until v1.1 (`docs/STATUS-debt.md`).

## Budget
Share: 15 points (1 point = 1 % of the weekly limit) of a 15-point ceiling, alone on the meter; reserve 0 kept for
chris.
Room: 15 points (the lower of the ceiling and the hard stop less the reading, 75 − 46 = 29, less the reserve).
Written when the Saturday-14th-5pm meter read 46 % (2026-11-10, given by chris, not measured); that reading is not
yours. On pickup: take the reading (or ask chris for it), name your meter, note your start reading, and cap the share
at your hard stop minus your reading. If other sessions share the meter, your share line is your ledger, never a
meter percentage. Stop starting new work at the first of: your ledger reaching 15 points ((finished workers' tokens +
your context) ÷ K × 1.5; K = 0.79M tokens/point, starting K); the meter reaching 61 %; the hard stop (75 % beyond 48 h,
80 % at 24–48 h, 85 % at 12–24 h, 93 % under 12 h). In-flight work finishes. Hand off at ~500k of your context. Before
handoff, write your row in the calibration log (`docs/budget-calibration.md`, made on first use).

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6): 8.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Set with `budget-select` above.

## Rules

- **Q1 Split.** Two parts, a look each (arc Q10). **6a, all the shapes:** rectangle with corner radius, ellipse,
  polygon and star; fill and stroke; dashes and caps. **6b, the transforms:** rotation (handle and field), lines that
  point any way, opacity. (Overruled: the recommendation was 6a shapes and fill with lines-any-way and opacity, 6b
  rotation with polygon, star, dashes and caps.) So the up-right line debt is paid in 6b, not first.
- **Q2 Edges.** Curves (ellipses, rounded corners) and slanted edges (polygons, stars, later rotated shapes) are
  anti-aliased, like text and slanted lines now. Straight, unrotated edges stay pixel-exact.
- **Q3 Fill gray.** Two new fill roles that each device resolves, **Shade** (a light tint for boxes and highlights)
  and **Block** (a dark solid, e.g. header bars), plus a Custom gray. (Overruled: the recommendation was Shade alone.)
- **Q4 Tools bar.** Three new entries: **Rectangle R, Ellipse E, Polygon Y**. A star is a polygon with "Star" ticked
  and an inner radius (one kind, spec §5.2). Shift draws a square, circle or regular polygon.
- **Q5 Rotation (6b).** A small round handle on a short stalk above the middle of the top edge; it catches at 0, 90,
  180 and 270° when near. Rotation is about the centre; the Arrange tab gets a Rotation field in degrees; Shift holds
  the angle to 15° steps.
- **Q6 Dashes and caps.** Named presets resolved to whole device pixels (Dotted, Short dash, Dash, Long dash) plus
  Custom dash and gap in mm; caps butt (the default), round, square.
- **Q7 Starting grays.** Shade 204, Block 68 in both built-in profiles, marked estimates, until chris's grays-sheet
  readings replace them.
- **Q8 Anything else to protect or avoid?** Lint the new shapes in 6a: L01 (stroke thinner than a pixel), L02 (stroke
  not pixel-aligned) and L06 (safe zones) cover them, not waiting for rung 14. The arc's standing rules hold too:
  existing export bytes frozen (stop and ask chris if any would change), drawing and dragging under 50 ms at the 90th
  percentile on the Pi (ADR-009).

## Owed

(empty)

## Defaulted

- **Size, said out loud:** 6a about 1–2 builder sessions, 6b about 1–2; the whole rung about 3 sessions plus two looks.
  veto: none expected.
- **File format 0.5.0 (6a)**, through the migration chain, so every 0.4 file opens unchanged and keeps its export
  bytes. New `ObjectKind`s: `Rect { corner_radius_mm }`, `Ellipse`, `Polygon { sides, star }` where `star` holds the
  inner radius as a fraction of the outer one. Objects gain an optional fill (`None` for every existing object) and
  a way to have no stroke; strokes gain dash and cap (defaults: solid, butt, so old lines are unchanged). A separate
  `FillRole { Shade, Block }` with gray-only values in each profile, apart from `LineRole`, since a fill has no width.
  6b adds rotation, line direction and opacity in format 0.6.0. veto: one format bump for the whole rung.
- **Stroke placement:** centred on the frame edge, as lines are, so a box drawn on the rhythm lines up with ruled lines;
  odd widths centre on pixel centres (spec §10.2). An unrotated rectangle with radius 0 draws as hard-edged
  whole-pixel rectangles, like lines today. veto: strokes inside the frame.
- **Polygon geometry:** the polygon fills its frame (stretched to the box, so constraints apply unchanged); a new one
  has 6 sides, point up; a star has 5 points and an inner radius of 0.5. Sides 3 to 32. veto: other defaults.
- **New shape style:** stroke role Structure ("box borders", spec §8.2), no fill; corner radius 0. veto: Guide stroke.
- **Drawing:** as rung 5: drag with rung 4's snapping (sizes land on whole rhythm units); a click drops a 10-unit
  square (or circle, or polygon) at the click point; one undo step; back to Select; double-click keeps the tool.
  veto: none expected.
- **Dash presets** in millimetres at 300 PPI matching the calibration sheet's rows, rounded to whole device pixels per
  device: Dotted = dash and gap equal to the stroke width; Short dash 0.5/0.5 mm (6/6 px); Dash 1/0.5 mm (12/6 px);
  Long dash 2/1 mm (24/12 px). chris's Nomad readings: 2/2 and 2/4 blur into a solid line lighter than gray 51–102,
  6/6 and longer stay dashed to 187. A dash pattern starts at a line's start and, on a closed shape, at its top-left
  corner. veto: other preset sizes.
- **Inspector:** the Style tab gains Stroke (None, a role, Custom), Fill (None, Shade, Block, Custom gray), Dash and
  Cap; a Shape section (corner radius, sides, Star, inner radius) appears for the new kinds. Corner radius takes any
  unit, rhythm units (u) included. Custom grays step by 17 until the clean levels are measured (spec §8.2 swatches are
  rung 14). veto: none expected.
- **Snapping:** the new kinds snap by their frames, with every rung 4 helper, as existing objects do. veto: none
  expected.
- **Lint (Q8):** L01 and L02 read each stroke of the new kinds; L02 checks only straight, unrotated edges (curves and
  slants are soft by Q2); L06 uses the shape's frame. L13 (large dark fills) stays in rung 14, though Block makes it
  likelier. veto: none expected.
- **Goldens:** one new golden page per device holding every new kind, both fill roles, a custom fill, each dash
  preset and each cap, added beside rung 1's (`crates/render/tests/golden/SHA256SUMS`), whose existing sums stay
  unchanged. veto: one golden per kind.
- **Rendering:** new display-list items or new fields may be added; the existing `Item` variants keep drawing exactly
  as now, so existing pages' bytes do not move. veto: none expected.
- **If chris reports the grays sheet before 6a's look,** Shade and Block move to the nearest clean level he reports,
  recorded in the profiles' calibration notes. veto: keep 204 and 68 until rung 14.
- **6b, settled now for its builder:** rotation stored in degrees on the object, about the frame's centre; the frame
  stays the unrotated box, constraints act on it and the rotation follows; selection, hit tests, snapping and lint
  use the rotated corners. A line gains a direction (its diagonal runs top-left to bottom-right or bottom-left to
  top-right), so it can point any way while its frame stays a box; this pays the rung 4a and rung 5 debt. Opacity
  0–100 %, exported as gray flattened over what lies beneath (spec §5.2). veto: a line stored as two end points.
- **Speed:** draw and drag measured on the Pi in a release build on both canvases for each part, 90th percentile
  under 50 ms (ADR-009), with a page holding the new kinds. veto: none expected.
- **Versions and releases:** 6a's approval commit bumps the workspace to `0.6.0` and is tagged `v0.6.0`; 6b's is
  `0.6.1`, `v0.6.1`; each gets a GitHub pre-release with notes in `docs/arcs/v1/release-notes/` (arc board). veto:
  none expected.
- **Words and access:** every new string through Fluent; every new control named for screen readers. veto: none.
- **Tests:** headless, one `CHECK` line each, printed starting with a newline: the 0.4 → 0.5 migration keeps old
  files and export bytes; each kind draws by drag and by click; Shift; fill roles resolve per device; dash presets
  land on whole pixels on both devices; caps; lint on the new kinds; existing goldens unchanged; new goldens.
  veto: none expected.
- **Landing:** commits straight to `main` (`AGENTS.md`, amendment of 2026-11-09); CI checked on each push. README
  ("what works today") and DEVELOPMENT.md (crate map) kept true. veto: none expected.

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(read the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board `docs/arcs/v1/README.md` and this record
(`docs/bbq/2026-11-10-rung-6-shapes.md` on `main`). Prove the build matches HEAD (`make build-proof`). Take the meter
reading and apply the Budget block. Have **Sonnet 5.5 at high effort** build **6a** on `main`: file format 0.5.0 with
the migration; rectangle (corner radius), ellipse, polygon and star; fill and stroke with the Shade and Block roles
(204 and 68, estimates); dashes and caps; the R, E, Y tools; the inspector fields; lint L01, L02, L06 on the new kinds;
a new golden page per device. Keep existing export bytes frozen (stop and ask chris if any would change). Judge the
builder's work against this record, run `make check-all`, measure drawing and dragging on the Pi (90th percentile,
ADR-009), show chris the window and stop for his look. After his approval: bump to `0.6.0`, tag `v0.6.0`, publish
the pre-release, then 6b (rotation, lines any way, opacity) the same way, tagged `v0.6.1`. Re-plan the board after
the rung, and write the next lead's prompt in `docs/arcs/v1/`. Update `STATUS.md`, `docs/STATUS-log.md` and
`docs/STATUS-debt.md` at the end of each session.
