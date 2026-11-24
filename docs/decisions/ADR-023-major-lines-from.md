---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"Ratify\", for ADR-023)"
ratified-by: chris
ratified-on: "2026-11-23"
owner: chris
kind: design
built: landed
proof: "`crates/layout/tests/major_from.rs::each_choice_places_the_majors_on_lines_grid_dots_and_cornell`, `crates/layout/tests/major_from.rs::centre_puts_a_major_on_the_middle_line`, `crates/layout/tests/major_from.rs::a_border_does_not_move_the_majors`, `crates/layout/tests/major_from.rs::isometric_has_no_top_and_its_centred_majors_still_meet`, `crates/layout/tests/major_from.rs::a_ruling_turned_a_quarter_turn_counts_in_its_own_axes`, `crates/model/tests/major_from.rs::a_file_without_the_field_reads_as_the_rhythm_origin_and_only_others_are_written`, `crates/ui/tests/major_from.rs::isometric_greys_top_and_shows_what_it_draws`, `crates/render/tests/migrate_v019.rs::v019_files_open_migrate_and_export_the_same_bytes`, `crates/render/tests/major_lines_from.rs::major_lines_from_page_goldens`, `crates/render/tests/major_lines_from.rs::major_lines_per_axis_page_goldens`"
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
- 2026-11-23: ratified by chris ("Ratify", asked "Ratify ADR-023 … as written?"). Earlier values, word for word:
  `status: proposed`; `status-note: drafted 2026-11-23 from chris's answers in docs/bbq/2026-11-23-major-lines-from-top.md (goforit); awaiting ratification`.
- 2026-11-23: built on `main` by the lead in a fresh session (Opus 5.5, high), as the record's Kickoff says; waits for
  chris's look. Earlier values, word for word: `built: not started`; `proof: "queued for a fresh session:
  docs/bbq/2026-11-23-major-lines-from-top.md, Kickoff"`. What the build chose where the record left room is in the
  record's Built section.
- 2026-11-23: amended by chris at his look: one choice per axis (see Amendment — 2026-11-23, chris, below).
- 2026-11-23: the amendment built in the same session; `proof` gains the per-axis golden page. Earlier value, word for
  word: `proof: "`crates/layout/tests/major_from.rs::each_choice_places_the_majors_on_lines_grid_dots_and_cornell`,
  `crates/layout/tests/major_from.rs::centre_leaves_the_odd_line_after_the_last_major`,
  `crates/layout/tests/major_from.rs::a_border_does_not_move_the_majors`,
  `crates/layout/tests/major_from.rs::isometric_has_no_top_and_its_centred_majors_still_meet`,
  `crates/layout/tests/major_from.rs::a_ruling_turned_a_quarter_turn_counts_in_its_own_axes`,
  `crates/model/tests/major_from.rs::a_file_without_the_field_reads_as_the_rhythm_origin_and_only_others_are_written`,
  `crates/ui/tests/major_from.rs::isometric_greys_top_and_shows_what_it_draws`,
  `crates/render/tests/migrate_v019.rs::v019_files_open_migrate_and_export_the_same_bytes`,
  `crates/render/tests/major_lines_from.rs::major_lines_from_page_goldens`"`.
- 2026-11-23: amended again by chris at his look: Centre puts a major on the middle line (see Amendment — 2026-11-23,
  chris, second, below). Built in the same session; the proof's Centre test is renamed. Earlier value, word for word:
  `proof: "`crates/layout/tests/major_from.rs::each_choice_places_the_majors_on_lines_grid_dots_and_cornell`, `crates/layout/tests/major_from.rs::centre_leaves_the_odd_line_after_the_last_major`, `crates/layout/tests/major_from.rs::a_border_does_not_move_the_majors`, `crates/layout/tests/major_from.rs::isometric_has_no_top_and_its_centred_majors_still_meet`, `crates/layout/tests/major_from.rs::a_ruling_turned_a_quarter_turn_counts_in_its_own_axes`, `crates/model/tests/major_from.rs::a_file_without_the_field_reads_as_the_rhythm_origin_and_only_others_are_written`, `crates/ui/tests/major_from.rs::isometric_greys_top_and_shows_what_it_draws`, `crates/render/tests/migrate_v019.rs::v019_files_open_migrate_and_export_the_same_bytes`, `crates/render/tests/major_lines_from.rs::major_lines_from_page_goldens`, `crates/render/tests/major_lines_from.rs::major_lines_per_axis_page_goldens`"`.

## Amendment — 2026-11-23, chris

**What changed.** At his look at the build, chris said, word for word: "The major lines need to be alignable to the
vertical and horizontal rhythms". Asked which he meant, he picked "A choice per axis": split the one row into "Major
rows from" and "Major columns from", each Top / Centre / Rhythm origin.

**Sections it touches.** Decision item 1 and Q2: the one **Major lines from** setting, applied to both axes, becomes
two settings, **Major rows from** (the horizontal lines, stepping down by the vertical rhythm) and **Major columns
from** (the vertical lines, stepping across by the horizontal rhythm), each with Top, Centre and Rhythm origin. Top
means the top edge for rows and the left edge for columns, as item 1 already said. So a region can keep its rows on
the page's rhythm while its columns are centred, or the other way round. Items 2 to 4 hold for each setting on its
own: older pages read both as Rhythm origin, new regions start both at Top, isometric offers Rhythm origin and
Centre on both, and a border moves neither. Consequences: the one field `major_from` becomes two; format 0.20.0 has
not gone out in a release (the last is `v0.16.5`, format 0.19.0), so it stays 0.20.0.

**Why.** One setting for both axes could not line one axis up with the rhythm while the other starts at the top or
centre.

**Who agreed.** chris, in the build session (his words and his pick above).

## Amendment — 2026-11-23, chris (second)

**What changed.** At his look at the per-axis build, chris said, word for word: "for major lines centered on the column
the major line should be right in the middle of the block and each set of lines should go off in either direction".

**Sections it touches.** Decision item 1, **Centre**, and Q1's "the ordinary lines left over above the first major and
below the last are equal": Centre now makes the middle line of the region major, and the other majors go off every Nth
line either way from it, for rows and for columns alike (the first amendment's two settings). With an odd number of lines
the leftover lines before the first major and after the last are then equal. With an even number there is no middle
line: the upper (or left) of the two middle lines is major, so the extra line is below (or right), as Q4 rules for the
odd line. On isometric, the three families' majors meet on the middle major column, at the middle crossing down it.

**Why.** The first rule spread the leftover lines evenly but did not put a major in the middle of the block; chris wants
the middle line to be the major one.

**Who agreed.** chris, in the build session (his words above). The even-number case is the lead's reading of Q4, and can
be vetoed.
