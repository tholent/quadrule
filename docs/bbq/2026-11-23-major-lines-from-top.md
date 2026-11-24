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

## Built (2026-11-23)

Built on `main` by the lead in a fresh session (Opus 5.5, high; `echo $CLAUDE_EFFORT` → `high`), as the Kickoff says.
Waits for chris's look.

- **Model:** `MajorFrom` (`Top`, `Centre`, `RhythmOrigin`) and `PatternRegion::major_from`; a file without it reads as
  `RhythmOrigin`, which is never written; `PatternRegion::new` starts at Top (isometric at Rhythm origin);
  `set_pattern_kind` to isometric while at Top sets Rhythm origin; `major_origin()` is what draws (an isometric region a
  file sets to Top draws from the rhythm origin) (`crates/model/src/lib.rs`).
- **Format 0.20.0**, a step that rewrites nothing (`crates/format/src/migrate.rs`). Fixture
  `crates/render/tests/fixtures/v0.19/`: the patterns, pattern-angle and pattern-borders pages saved by the 0.19.0 build
  before the bump, with the sha256 of each export (the same as their goldens).
- **Layout:** each axis finds the index of a major line from the lines in the frame (`major_base`,
  `crates/layout/src/pattern.rs`): the first for Top, the first plus half the leftover for Centre, the origin's for Rhythm
  origin. The border only hides lines, so it never moves a major.
- **Inspector:** a **Major lines from** row under Major lines (and Major every), before Major role: Top, Centre, Rhythm
  origin; Top greyed on isometric; one undo step (`crates/ui/src/panels.rs`, words in `crates/i18n/locales/en-US/app.ftl`).
- **Golden page** `major-lines-from_*` on both tablets (`majors_document`, `crates/model/src/sample.rs`): three columns
  (Top, Centre, Rhythm origin) of lines, grid, dots, Cornell, isometric and a bordered grid, on a 3 mm rhythm with a
  major every 7, each object keeping its millimetres on the Nomad. The 36 goldens before it are unchanged. In the app:
  `cargo run --release -p quadrule-app -- --majors-page --look major-lines-from`.
- **Tests** (one `CHECK` line each): `crates/layout/tests/major_from.rs` (5; each fails with the old rule put back, `test
  result: FAILED. 0 passed; 5 failed`), `crates/model/tests/major_from.rs` (2), `crates/ui/tests/major_from.rs` (2),
  `crates/ui/tests/patterns.rs` (the Pattern tool's region starts at Top; Major lines from is one undo step),
  `crates/render/tests/major_lines_from.rs` (3), `crates/render/tests/migrate_v019.rs` (2).

### Calls the build made where the record left room (each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| Every sample page that already had a golden keeps Rhythm origin (`region_0_19` in `crates/model/src/sample.rs`), although Defaulted listed "the samples' regions" among those that start at Top | those pages are the old goldens, and Q3, ADR-023 item 2 and Defaulted's "older pages' goldens unchanged" keep their bytes; with Top, thirteen goldens moved (patterns, its landscape, turned, angle and borders on both tablets, and the three paper pages). The only place the app makes a region, the Pattern tool, starts at Top; New… makes none | veto: re-make those goldens at Top |
| A ruling turned inside its frame counts in the box round the frame in the ruling's own axes: Top's first line is the first one that meets the frame, Centre centres over the lines that meet it | at a quarter turn this is the frame's own edge; counting from the old anchor (the frame's corner before turning) would put Top's first major off the edge of a frame that is not square | veto: count from the old anchor |
| Isometric, Centre: the vertical majors are centred; the slanted majors are placed so that all three families still meet at one crossing on a major column, centred down it; each slanted family is then centred to within a line, not on its own | centred one by one, the three families' majors would cross in three different places and draw small stray triangles instead of a big triangle grid | veto: centre each family on its own |
| Isometric dots, Centre: a major dot is on a centred major column and a centred major place down it (the rule they had, counted from the frame) | the dots' own rule | none |

Not seen: the inspector in the real window. The screen was locked (`mate-screensaver-command -q` → active, lock on), so
no screenshot; the row was checked headless instead (`crates/ui/tests/major_from.rs`).

## Look (2026-11-23)

chris, at his look, word for word: "The major lines need to be alignable to the vertical and horizontal rhythms".
Asked which he meant (a choice per axis; one row with mixed choices; or a fault where Rhythm origin does not line up),
he picked **"A choice per axis"**: split the one row into "Major rows from" and "Major columns from", each Top / Centre /
Rhythm origin. This changes Q2 (one setting for both axes); recorded as ADR-023's Amendment — 2026-11-23, chris.

Built in the same session:

- **Model:** `PatternRegion::major_from` became `major_rows_from` and `major_columns_from`, each read as Rhythm origin
  when absent and written only when not that; `major_rows_origin()` and `major_columns_origin()` are what draw;
  `Pattern::has_major_columns` (Grid, Dots, Isometric). Format 0.20.0 has not been released, so it stays 0.20.0.
- **Layout:** columns count from the columns setting, rows from the rows setting. Isometric: the three families' majors
  still meet at one crossing, on the major column the columns setting picks, at the place down it the rows setting picks.
- **Inspector:** two rows, **Major rows from** and **Major columns from**; the columns row is greyed for Lines and Cornell;
  Top greyed in both on isometric.
- **Golden pages:** `major-lines-from_*` unchanged byte for byte (each cell sets both axes alike); a new
  `major-lines-per-axis_*` page on both tablets: a grid, dots and isometric lines with rows and columns mixed three ways.
- **Tests:** the layout choices test runs all nine row/column pairs on lines, grid, dots and Cornell; the isometric test
  the four Centre/Rhythm origin pairs. With the columns put back on the rows' setting, both fail (`5 tests run: 3
  passed, 2 failed`).

### Calls the amendment's build made (each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| Columns say **Top** for the left edge, as chris's pick reads ("each Top / Centre / Rhythm origin") | his words; ADR-023 item 1 already calls the left edge Top for columns | veto: call it **Left** in the columns row |
| The columns row is shown but greyed for Lines and Cornell, which have no major columns | the inspector greys Top on isometric the same way, and the rows stay in one place for every pattern | veto: hide the row instead |
| A turned ruling's rows and columns are its own (a Lines ruling turned a quarter turn still uses Major rows from) | the turned ruling already counts in its own axes (Built, second call) | veto: rows and columns of the page |
| A file saved by this unreleased build's first version (a `major_from` field) reads that field as unknown and both axes as Rhythm origin | 0.20.0 never went out; no reader needs it | veto: read `major_from` into both |

## Look, second (2026-11-23)

chris, word for word: "for major lines centered on the column the major line should be right in the middle of the block
and each set of lines should go off in either direction". Recorded as ADR-023's Amendment — 2026-11-23, chris (second).

Built in the same session: Centre makes the middle line major, the others every Nth line either way from it
(`major_base`, `crates/layout/src/pattern.rs`), rows and columns alike; isometric's three families meet at the middle
crossing of the middle major column. Both golden pages re-blessed after a look on both tablets (their Centre cells
changed; the 36 sums before them unchanged). Tests: `centre_puts_a_major_on_the_middle_line` (9, 13 and 12 lines); with
the old Centre rule put back, `5 tests run: 1 passed, 4 failed`.

| Call | Why | veto: |
|---|---|---|
| Applied to rows as well as columns, though chris said "on the column" | Centre is one word for both settings; a centred row that worked differently from a centred column would be two meanings for one word | veto: rows keep the old Centre |
| An even number of lines puts the major on the upper (or left) of the two middle lines, the extra line below (or right) | Q4's odd line goes below and right | veto: the lower (or right) one |
| The isometric test uses a twelve-line region: on nine or eleven, the middle crossing happens to fall on one of the rhythm origin's, so rows from Centre and from Rhythm origin draw alike there | that is right for those sizes, not a fault; the test needs a size where they differ | none |
