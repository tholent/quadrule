# bbq: where a region's major lines start (2026-11-23)

Plain bbq, 8 questions (chris: "bbq 8q major division line changes: "- major division lines start at top of object -
major division lines centered?""). Lead: this session (Opus 5.5, high), which builds it; no workers.

## Grounding

| Premise | Source |
|---|---|
| A region's major lines are every Nth line counted from the page's rhythm origin ("the origin row is major") | `docs/bbq/2026-11-13-rung-9-pattern-library.md`, Defaulted "Which patterns have major lines"; `crates/layout/src/pattern.rs`, `is_major` on the index from `Grid.ox`/`Grid.oy` |
| An unbound region counts from its own top-left corner | `crates/layout/src/pattern.rs`, `region_grid` |
| Lines, Grid, Dots, Isometric and Cornell have major lines; Hex, Music staff and Calendar none | `crates/model/src/lib.rs`, `Pattern::has_major` |
| Format 0.19.0 went out in `v0.16.5`, so a new field means a new format version | `crates/format/src/lib.rs`; `docs/STATUS-log.md`, v0.16.5 |

## Rules

**Q1 Where they start.** chris: "A choice: Top or Centred (Recommended)". A per-region setting: Top (the region's first
line is major, then every Nth) or Centred (the ordinary lines left over above the first major and below the last are
equal).

**Q2 Axes.** chris: "Both axes (Recommended)". Rows from the top (or centred down), columns from the left (or centred
across).

**Q3 Older pages.** chris, word for word: "#1, but allow for "from rhhthm origin" as a legit third option". Files saved
before keep counting from the rhythm origin and export the same bytes; every new region starts at Top; and "from the
rhythm origin" is a full third choice any region can pick, not only a leftover for older files.

**Q4 An odd line over.** chris: "At the bottom (and right) (Recommended)". When Centred cannot split the leftover lines
evenly, the extra one goes below the last major (and right of the last major column), as the Centred rhythm origin puts
its odd pixel.

**Q5 Words.** chris: "'Major lines from': Top, Centre, Rhythm origin (Recommended)". A row **Major lines from** under
Major lines in a pattern region's settings, with the choices **Top**, **Centre** and **Rhythm origin**.

**Q6 Isometric.** chris, word for word: "isometric can do rhythm origin, or centered". An isometric region offers
Rhythm origin and Centre, not Top.

**Q7 A bordered edge.** chris: "The edge line, under the border (Recommended)". Counting starts at the region's edge
whether or not that edge has a border: the border stands where the first major would be, and the next major is N lines
in. Adding or removing a border never moves the majors.

**Q8 Anything else.** chris: "Nothing more (Recommended)".

## Owed

None.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| A field on the pattern region, `major_from`: `Top`, `Centre` or `RhythmOrigin`; a file without it reads as `RhythmOrigin` (Q3), and every region made from now on (the Pattern tool, New…, the calendar's and the samples' regions) gets `Top` | Q1, Q3 | veto: another field name |
| Format 0.20.0 (0.19.0 is out in `v0.16.5`); a 0.19 fixture opens and exports the same bytes | one format step per release | none |
| No flag, as the calendar and borders landed (ADR-007 precedent); older pages' goldens unchanged, one new golden page per tablet showing Top, Centre and Rhythm origin side by side | Q3 keeps every old export | veto: flag it |
| "Top" means the region's top edge down and its left edge across, counted from where the edge's line is (the first line at or inside the edge is index 0, major) | Q1, Q2, Q7 | veto: count from the first whole line inside |
| Centre: of the lines in the region along an axis, the majors are placed so the ordinary lines left above the first and below the last differ by at most one, the extra below (or right) | Q1, Q4 | none |
| Isometric: its vertical columns and each slanted family are centred the same way, each over the lines that cross the region; an isometric region set to Top draws as Rhythm origin and shows Top greyed; switching a region to Isometric while it is Top sets Rhythm origin | Q6 | veto: switching sets Centre |
| Dots: a dot is major where its row and its column both are, each counted by the setting | today's rule, Q2 | none |
| Cornell counts its ruled lines the same way, from the region's top (or centred), not from the notes area's top | the region is the object | veto: count from the notes area |
| A ruling turned inside its frame counts in its own axes from the frame's top-left corner (Top) or middle (Centre) | the anchor the turned ruling already uses | none |
| An unbound region already counts from its own corner, so Top and Rhythm origin look the same there; the choice is still shown | `region_grid` | none |
| The page-wide rhythm overlay is unchanged (still from the rhythm origin) | Q8 "Nothing more" | none |
| L16 and L17 unchanged: majors do not move the lines, only which ones are drawn stronger | | none |
| Ships in the next release | | veto: a release of its own |

## Kickoff

Route: a fresh session (chris: "A fresh session (Recommended)"; this one had built the calendar, cut `v0.16.5`, made
ADR-022 and amended ADR-019). Lead: Opus 5.5, high. Build `major_from` (Q1–Q7) from this record and ADR-023 (`docs/decisions/ADR-023-major-lines-from.md`, ratified
by chris 2026-11-23) on `main`, with tests that
print one `CHECK` line each, a 0.19 fixture, and one new golden page on both tablets. Then chris's look.
