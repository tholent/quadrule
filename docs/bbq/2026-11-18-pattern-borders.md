# bbq record — quadrule: pattern region borders

2026-11-18 · untyped bbq (a small job asked by chris after rung 16a: "I need you to add pattern region borders as a
selectable option") · 4 questions · lead: Opus 5.5 at high effort (this session); builder Sonnet 5.5 at high effort
(arc Q2).

Grounding: `PatternRegion` in `crates/model/src/lib.rs:1265` (no border today); `Object` (`style`, `dash`, `cap`);
`LineRole` (spec §8.2: Structure is for "box borders"); ADR-014 (a pattern draws every line and dot whose centre lies
inside its frame, whole); `crates/layout/src/pattern.rs` (turned rulings cut at the frame, `clip_polygon`, dots kept
whole when their centre is inside); `crates/format/src/lib.rs` (`FORMAT_VERSION` 0.16.0); fixtures `v0.4/`–`v0.15/`.

## Budget

No share set. chris lifted this week's hard limits until the reset, Saturday 2026-11-21, 5 pm EST (rung 16 record,
Amendments: "for this week only, unset any hard limits"). The meter read 65 % at 16a's approval. The lead reads it at
the leg's end and reports it. Sized as one builder leg, about 2–4 points.

## Rules

- **Q1 Placement.** chris: "On the frame edge, skip edge lines (Recommended)". The border is centred on the frame's
  edge, like a rectangle's stroke. On a side that has a border, pattern lines or dots whose centre falls on that edge
  are left out, so nothing doubles up. This changes ADR-014's rule for bordered sides: an amendment, for chris to
  ratify at the look.
- **Q2 Sides.** chris: "Pick each side (Recommended)". Four ticks (top, right, bottom, left), all on when the border is
  turned on.
- **Q3 Controls.** chris: "Its line role (Recommended), Rounded corners". The border has its own line role (Whisper,
  Guide, Structure, Emphasis, or a custom width and gray, as any line), starting on Structure, and a corner radius in mm.
  Dashes were offered and not picked.
- **Q4 Release.** chris: "Now, as v0.16.1 (Recommended)". Built now, chris's look, then `v0.16.1`; the starters (16b)
  move to `v0.16.2`.

## Owed

Empty.

## Defaulted (the lead's calls; each can be vetoed)

- **Where it lives.** A new optional `border` on `PatternRegion` (on/off, the four sides, a role, a corner radius),
  skipped when off, so a file without it reads as no border and every existing page saves and exports the same bytes.
  The file format goes to **0.17.0** with a `v0.16/` fixture, as earlier format changes did. *veto:* none expected.
- **The inspector.** A **Border** section in the pattern region's Style tab: an on/off tick; the four side ticks, shown
  when it is on; the role picker the other strokes use; Corner radius in mm (0 is square). One edit is one undo step
  (ADR-010). Every word goes through Fluent; the builder proposes them for chris's look. *veto:* another tab.
- **What "on the edge" means.** A line or dot is on a bordered edge when its centre is within half a device pixel of
  that edge on the device being drawn (the same half-pixel the rhythm uses, ADR-011). Unbordered sides keep ADR-014's
  rule unchanged. *veto:* a wider band.
- **Rounded corners.** A corner rounds only when both of its sides have a border; otherwise it stays square. The radius
  is capped at half the shorter side. Inside a rounded corner, ruling lines are cut at the curve (as turned rulings are
  cut at the frame, `crates/layout/src/pattern.rs`), and a dot is dropped when its centre lies outside the curve.
  *veto:* the pattern ignores the curve.
- **Turning.** The border turns with the frame (`rotation_deg`), not with the pattern's own angle (`angle_deg`, rung
  12b). *veto:* none expected.
- **Several devices.** The border follows the frame on every device, as the frame's constraints place it; its role's
  width and gray come from each device's profile. It can be overridden per device like any other style (rung 8), if the
  override system already covers the pattern's fields; if not, the builder says so rather than building it.
  *veto:* none expected.
- **Snapping and lint.** The border's edges snap as the frame's edges already do. Lint judges the border's role as it
  judges any line's (L03, L05, L13, L14). *veto:* none expected.
- **Pattern regions inside repeats and groups** draw their border in every copy. *veto:* none expected.
- **Frozen.** The 30 golden lines and fixtures `v0.4/`–`v0.15/` stay frozen; a change that moves any is a stop.
  New goldens, on both devices: a bordered ruled region, a bordered dot grid with one side off, and a rounded-corner
  grid. Each new golden picture is looked at before it is blessed (rung 8 lesson).
- **Speed.** Drawing and dragging a bordered pattern region on the Pi, release build, both canvases, strip on, 90th
  percentile under 50 ms (ADR-009). *veto:* none expected.
- **ADR-014** gets a dated amendment (bordered sides drop edge lines; rounded corners cut the ruling), drafted with
  `makeadr` before the look and ratified by chris at it.
- **No flag**, as rungs 12b and 15 changed the format without one (arc rules); the border is off by default, so no
  existing page changes. *veto:* flag it.

## Amendments

(Rulings made after the bbq, at the look or later, go here word for word, dated.)

- **2026-11-18, the borders look** (meter not read at the look): chris, word for word: "approved and ratified". The
  borders are approved with no changes, so the builder's judgement calls stand as built (`docs/STATUS-log.md`, the
  borders builder entry): the wider "on the edge" band (the Defaulted list's "a wider band" veto, taken), the proposed
  words, and unticking Show border forgetting its settings. ADR-014's pattern-borders amendment is ratified. Shipped
  as `v0.16.1`; the starters (16b) move to `v0.16.2` (Q4).

## Kickoff

Builder: **Sonnet 5.5 at high effort**, on `main`, committing as each step lands. Build the border from this record's
Rules and Defaulted list: the model field and format 0.17.0 with its fixture, the layout (border, edge-line skipping,
rounded corners), both canvases and the export renderer, the Style tab's Border section, goldens on both devices,
tests that print one structured line, the speed numbers. Then the lead drafts the ADR-014 amendment, shows chris the
window, and stops for his look. After approval: `0.16.1`, release notes, the tag, `STATUS.md`, the log, the board and
the 16b prompt's version line.
