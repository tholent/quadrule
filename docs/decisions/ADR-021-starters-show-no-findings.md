---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"ratify 021\")"
ratified-by: chris
ratified-on: "2026-11-23"
owner: chris
kind: design
built: partial
proof: "`crates/lint/tests/starter_rules.rs::l14_and_l09_leave_drawing_grids_alone`"
owed: "item 1's test, which lints every starter on both tablets and finds nothing, arrives with the starters in rung 16b"
supersedes: nothing
amends: "spec §13 (\"All starters must pass lint with no rhythm warnings\") and §8.3 L09 and L14; arc v1 release bar (Q18)"
---
# ADR-021 — A starter is lint-clean only when it shows no findings at all, and L14 and L09 leave drawing grids alone

## Decision

1. **Lint-clean means no findings.** A starter passes only when the linter finds nothing on it, Info included, on
   every device it targets, at every toolbar position lint checks. A fresh starter opens showing ⚠ 0. This is what
   spec §13's "must pass lint with no rhythm warnings" and the arc's release bar ("every starter lint-clean") mean.
   When a rule fires on a starter that is designed as it should be, the rule is fixed, not the starter bent to it.
2. **L14 skips slanted patterns.** A pattern region whose pattern is made of slanted lines by nature (hex, isometric)
   is not judged by L14 (thin slanted lines look soft): choosing the pattern is choosing slanted lines. L14 still
   judges slanted and curved shapes, turned objects, and patterns that are turned (rung 12b).
3. **L09 skips drawing grids.** L09 (a step too small to write between) does not judge an isometric rhythm, nor hex or
   isometric pattern regions: they are for drawing, not for writing between lines. It still judges rectangular rhythms
   and ruled, dot, grid, Cornell and staff regions.

## Context

- chris's next five starters (bullet journal, Cornell, grid, hex, isometric; record
  `docs/bbq/2026-11-23-adr-starter-lint.md`) showed three Info rules on top of the safe-zone findings ADR-019 settles:
  L18 on all five, L14 on hex and isometric, L09 on isometric (its rhythm steps 2.5 mm, then 3.0 mm, under the 5 mm
  handwriting limit).
- The ⚠ count in the window counts every finding, Info included (`crates/ui/src/view.rs`, `lint_count`); preflight stops
  only for Warn findings (`crates/ui/src/preflight.rs`). So an Info finding left on a starter shows to every user who
  opens it.
- L18 needed no rule change: chris centred the five starters' rhythms (as the narrow-ruled pages and Supernote's stock
  grid are), and it went (record, E).
- Hex and isometric lines are slanted whatever their width; only a darker 2 px role would quiet L14.

Drivers:
- A starter is the first page a new user sees; it should open clean.
- A rule that fires on a well-made page teaches users to ignore the linter.
- The rest of each rule keeps judging what it was made for.

## Alternatives

For 1, lint-clean:

### No findings at all (chosen: a starter opens showing ⚠ 0)
- Good: the gallery is clean; every misfiring rule is found now, while starters are made.
- Bad: more rule refinement before 16b.

### No Warn findings and no rhythm findings
- Good: less rule work.
- Bad: starters open showing a few Info findings.

### No Warn findings
- Good: matches what preflight stops for.
- Bad: Info findings stay on starters and in the ⚠ count.

For 2, L14:

### L14 skips slanted patterns (chosen: the pattern is the choice)
- Good: hex and isometric keep the light lines chris chose; everything else is still judged.
- Bad: a user's own hex page is never told its lines look soft.

### Thicker lines in the starters
- Good: no rule change.
- Bad: a 2 px role (Structure, gray 51) is darker than the Whisper and Guide chris chose.

### A per-object "Soft is fine" mark
- Good: works for any object.
- Bad: one more tick in the Arrange tab, for a case the pattern already says.

For 3, L09:

### L09 skips drawing grids (chosen: isometric and hex are for drawing)
- Good: no setting; the rule keeps judging writing pages.
- Bad: someone who writes on an isometric page is not warned.

### A per-page "For drawing" mark
- Good: covers a drawing page on a square grid too.
- Bad: one more page setting.

### A bigger isometric step
- Good: no rule change.
- Bad: the triangles grow until the step reaches 5 mm, much coarser than drawing paper.

## Consequences

- Each starter in 16b is checked by a test that lints it on both tablets and finds nothing (with the golden test).
- L14 and L09 fire less: tests in `crates/lint/tests/` show a hex and an isometric region excused, and a slanted line
  and a turned grid still caught by L14, a 3 mm ruled region still caught by L09.
- Spec §13 and the arc's release bar read as above; their text gets a dated note pointing here.
- Later starters may find more rules that misfire; each is ruled the same way, through this record.

## Asks left open

1. Ratify ADR-021 as written?

## History

- 2026-11-23: drafted from chris's answers D, F and G in the starter lint record, through makeadr.
- 2026-11-23: ratified by chris ("ratify 021"). Prior header values, word for word: status `proposed`, status-note
  `drafted 2026-11-23 from chris's answers (starter lint record, D, F and G); awaiting ratification`.
- 2026-11-23: built (lead session, after the ratification). Prior header values, word for word: built `not started`, proof
  `"queued before rung 16b with ADR-019 and ADR-020 (`docs/bbq/2026-11-23-adr-starter-lint.md`, Owed; `docs/arcs/v1/README.md` row 16)"`.
