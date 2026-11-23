---
status: proposed
status-note: drafted 2026-11-23 from chris's answers in docs/bbq/2026-11-23-major-lines-from-top.md (goforit); awaiting ratification
owner: chris
kind: design
built: not started
proof: "queued for a fresh session: docs/bbq/2026-11-23-major-lines-from-top.md, Kickoff"
supersedes: nothing
amends: "docs/bbq/2026-11-13-rung-9-pattern-library.md, Defaulted \"Which patterns have major lines\" (counting starts at the rhythm origin)"
---
# ADR-023 — A pattern region's major lines count from its top, its centre or the rhythm origin, Top for new regions

## Decision

1. **A setting per region.** Each pattern region with major lines has **Major lines from**, with three choices: **Top**,
   **Centre** and **Rhythm origin** (Q1, Q5). It sits under Major lines in the pattern settings.
   - **Top:** the line on the region's top edge is major, then every Nth line down; across, the line on its left edge,
     then every Nth line right (Q1, Q2).
   - **Centre:** along each axis the majors are placed so the ordinary lines left above the first major and below the
     last differ by at most one; the extra one goes below (or right), as the Centred rhythm origin puts its odd pixel
     (Q1, Q2, Q4).
   - **Rhythm origin:** every Nth line counted from the page's rhythm origin, as every region has done since rung 9 (Q3).
2. **Older pages keep their look.** A file saved before this reads as Rhythm origin and exports the same bytes; every
   region made from now on starts at Top. Rhythm origin stays a full choice for any region (Q3).
3. **Isometric** offers Rhythm origin and Centre, not Top: its slanted lines have no top of their own (Q6).
4. **A bordered edge** counts as the edge: with Top, the border stands where the first major line would be, and the
   next major is N lines in, so adding or removing a border never moves the majors (Q7).

## Context

Rung 9 ruled that major lines count from the rhythm origin, "so the origin row is major" (rung 9 record, Defaulted
"Which patterns have major lines"; `crates/layout/src/pattern.rs`, `is_major` on the index from the grid's origin). A
region whose top is not a multiple of N rows from the origin therefore starts its majors part-way in, which chris met
designing his starters: "major division lines start at top of object", "major division lines centered?" (the bbq's
opening words). The bbq settled the choice in 8 questions (`docs/bbq/2026-11-23-major-lines-from-top.md`, Q1–Q8). Lines,
Grid, Dots, Isometric and Cornell have major lines; Hex, Music staff and Calendar do not (`Pattern::has_major`). Format
0.19.0 is out in `v0.16.5`, so the field needs 0.20.0.

Drivers:
- A region's ruling should look the same wherever the region is placed on the page.
- No page already made may change its export (ADR-007's rule for export bytes).
- A page-wide grid that lines up across regions must stay possible (Rhythm origin).

## Alternatives

### A choice of Top, Centre and Rhythm origin, Top for new regions (chosen: covers both of chris's notes and keeps old pages)

- Good: each region looks the same wherever it sits; Centre gives symmetric ruling; old pages export the same bytes.
- Bad: one more setting and one more file field (format 0.20.0); three ways to test on five patterns.

### Top only, or Centre only

- Good: no new setting.
- Bad: drops one of chris's two notes; old pages must either change or keep a hidden mode anyway. Not chosen (Q1).

### Change every region, old ones too

- Good: no third choice.
- Bad: older pages export differently and their golden images are re-made. Not chosen (Q3, which also kept Rhythm
  origin as a full choice).

## Consequences

- The model gains a field on `PatternRegion` (`major_from`; Top, Centre, RhythmOrigin), absent in older files and read
  as RhythmOrigin; format 0.20.0, with a 0.19 fixture opening and exporting the same bytes (record, Defaulted).
- The pattern layout counts majors per region and axis by the setting, for Lines, Grid, Dots (a dot is major where its
  row and column both are), Cornell (from the region's top, not the notes area's), and Isometric (columns and each
  slanted family centred over the lines crossing the region); a ruling turned inside its frame counts in its own axes
  from the frame's corner or middle (record, Defaulted).
- Unchanged: the page-wide rhythm overlay, L16 and L17, and every page saved before (record, Defaulted).
- Tests, each one `CHECK` line: each choice on each pattern, the odd line below, a border not moving the majors, the
  0.19 fixture; one new golden page on both tablets with the three choices side by side; no flag (record, Defaulted).

## History

- 2026-11-23: drafted with goforit (no rounds) from chris's answers Q1–Q8 and the lead's Defaulted calls in
  `docs/bbq/2026-11-23-major-lines-from-top.md`.
