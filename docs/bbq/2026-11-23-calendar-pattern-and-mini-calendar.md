# bbq: a calendar pattern and a mini calendar (2026-11-23)

Plain bbq, 8 questions (chris: "bbq 8q calendar pattern and mini calendar"). Lead: this session (Opus 5.5), which builds
it as it built ADR-019 to ADR-021 (Defaulted). Ships in `v0.16.5` with the starter lint fixes (chris: "let's wrap these
changes as well as my new features into v0.16.5 and 16b get's bumped back to v0.16.6"); chris needs both before he makes
the Planning starters.

## Grounding

| Premise | Source |
|---|---|
| The Planning starters include a "monthly calendar (static grid)" | `napkins/quadrule-spec.md` §13 |
| A calendar generator and text tokens (`{n}`, `{date}`) are planned for v1.2 or later | spec §16 "v1.2 and later"; §6.4 |
| Patterns draw lines and dots only; none draws text today | `crates/model/src/lib.rs` `Pattern`; `crates/layout/src/pattern.rs` |
| Repeats can lay out a grid of boxes but cannot number them | spec §6.4; `crates/model/src/lib.rs` (repeat) |
| Text is drawn from the bundled fonts as outlines (also used outside text boxes, by the calibration sheets) | `crates/layout` `text_path`; `crates/xtask/src/calib.rs` |
| Format 0.19.0 is not yet released (it goes out in `v0.16.5`) | `crates/format/src/lib.rs`; `docs/STATUS-log.md` 2026-11-23 |

## Rules

**Q1 Dates.** chris, word for word: "let's have months be optional, but allow starting day of the week and days in the
month be options for switching borders. additionally, having small squares in teh top of boxes so that a day can be
added". So:
- The calendar pattern has an optional **month and year**. Set, the days are numbered in their right cells, and changing
  the month is one field.
- Undated, two settings shape the month instead: **the weekday the 1st falls on** and **the days in the month**
  (28–31). They decide which cells are the month's days, and so which cells have borders (Q5). (The lead's reading of
  "starting day of the week", told to chris before Q5 and not corrected; *veto:* if it meant the week's first day, Q3
  already covers that.)
- Each day's cell has a **small square at its top** to write the day's number in (where, and when dated: Q6).
- This brings forward part of v1.2's calendar generator (dates in a pattern); tokens and the generator itself stay
  v1.2.

**Q2 The mini calendar.** chris: "Its own small object (Recommended)". A compact month block, weekday initials and day
numbers, no cells to write in, placed anywhere (a corner of a daily page). Always dated: a blank mini calendar has
nothing to show.

**Q3 Week start.** chris: "A setting, Monday first (Recommended)". Each calendar and mini calendar has **Week starts on**;
new ones start on Monday.

**Q4 Words.** chris, word for word: "number 1, but allow for empty squares when a month/year isn't added (see "date"
question)". The pattern draws its own weekday header and day numbers in the bundled font, grays from the Label or Text
role, sized to the cells. With no month and year, the day squares are empty.

**Q5 Days outside the month.** chris: "No borders (Recommended)". Only the month's own days are boxed; the grid's outline
steps round them.

**Q6 The day square.** chris, word for word: "top right; number alone when dated". Undated: an empty square in each day's
**top-right** corner. Dated: the printed number alone, in that corner, with no square.

**Q7 Weekday names.** chris: "By the room there is (Recommended)". "Monday" when the column is wide enough, else "Mon",
else "M", the same for every column. The mini calendar always uses one letter.

**Q8 Anything else.** chris: "Nothing more".

## Owed

None.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| This session leads and builds; no worker | it built ADR-019 to ADR-021 the same way; the job is one coherent change | veto: a Sonnet 5.5 high builder |
| The calendar is a new pattern kind, `Calendar`, in the pattern region (a frame, constraints, borders, roles as the others have) | the shape every pattern already has; Cornell is the nearest | veto: its own object kind |
| The mini calendar is a new object kind, `MiniCalendar`, with a frame | Q2: its own object, placed anywhere | veto: a pattern kind |
| Rows: as many weeks as the month spans (4 to 6), dated or undated | Q1's two settings, or the month, fix it | veto: always 6 rows |
| Cells fill the region: 7 equal columns, the header row and the week rows sharing the height, bound to the rhythm where the region is (header one rhythm unit tall) | as Cornell divides its region | veto: cells in whole rhythm units, the region sized to fit |
| The day square is one rhythm unit across, at most a third of the cell | small, and on the rhythm | veto: a fixed size in mm |
| Roles: grid lines in the region's role (Guide by default); header and numbers in Label gray; the mini calendar's numbers in Text gray | the roles that exist; Label is the quiet text role | veto: other roles |
| Month and weekday names go through the translation files (`crates/i18n/locales/en-US/`), English only for now | ADR-008 item 5 | none |
| Text drawn by the pattern is judged by L07 (smallest text): the numbers and names shrink to fit their cells but never below the profile's smallest text size; below it, the label is left out and L07 says so | ADR-021: starters show no findings, so the sizes must stay legible | veto: always draw, and let L07 fire |
| The format stays 0.19.0 (unreleased): the new kind and object are added to it, with the 0.18 fixture still opening unchanged | one format step per release | veto: 0.20.0 |
| New golden pages on both tablets (a dated month, an undated one, a mini calendar), frozen at the look | as the grand staff and borders were (no flag; ADR-007 precedent) | veto: flag it |
| Size: medium to large (a new pattern kind with text, a new object kind, dates, the inspector, goldens); one lead session | text in a pattern and date arithmetic are new | none |

## Kickoff

Build the calendar pattern (Q1, Q3–Q7) and the mini calendar (Q2, Q3, Q7) from this record, on `main`, with tests that
print one `CHECK` line each and goldens on both tablets. Then chris's look, together with the Full bleed tick (ADR-019),
and `v0.16.5`.
