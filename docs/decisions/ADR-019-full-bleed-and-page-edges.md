---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"ratify 019 and 020\")"
ratified-by: chris
ratified-on: "2026-11-23"
owner: chris
kind: design
built: landed
proof: "`crates/lint/tests/starter_rules.rs::full_bleed_is_left_alone_by_l06_and_l10_and_the_rest_is_still_checked`, `crates/lint/tests/starter_rules.rs::l10_judges_the_frame_not_half_a_stroke_on_an_edge`, `crates/lint/tests/starter_rules.rs::l16_counts_a_page_edge_as_on_the_rhythm_on_that_axis`, `crates/ui/tests/lint_pointer.rs::full_bleed_is_a_checkbox_beside_free_placement_with_a_mixed_state`, `crates/render/tests/migrate_v018.rs::v018_files_open_migrate_and_export_the_same_bytes`"
supersedes: nothing
amends: "ADR-012 §Decision item 5 (lint checks every toolbar position and the footers); spec §8.3 L10 and L16"
---
# ADR-019 — An object can be marked Full bleed, which L06 and L10 then skip, and an object flush on a page edge is on the rhythm on that axis

## Decision

1. **A Full bleed mark.** Any object can be marked **Full bleed** in the Arrange tab, beside Free placement. L06
   (under a safe zone) and L10 (off the page) skip a marked object and everything inside it (a marked group or repeat),
   as Free placement is inherited. Every other rule still judges it, and unmarked objects on the same page are still
   checked for safe zones. The mark is saved in the `.quad` file (a field on the object, as `free_placement` is).
2. **L10 counts the frame, not the stroke's overhang.** An object whose frame lies inside the page is not "off the
   page" because half its stroke reaches past an edge it sits on. L10 fires only when the frame itself leaves the page.
3. **Page edges are on the rhythm.** L16 treats an object placed flush on a page edge (its frame on the near or the far
   edge of an axis, to the pixel) as on the rhythm on that axis, on every page, starter or not. The other axis is still
   judged.

## Context

- chris's Paper starters (`narrow_ruled.quad` and its right-to-left copy) run their lines edge to edge, under the
  toolbar and footers, as real paper does and as Supernote's own stock templates do (`docs/device-measurements/README.md`:
  "Full bleed. The grid runs under the toolbar and footer; the stock template ignores safe zones").
- Linted, each gives L06 (Warn) and L10 (Info) on both tablets for both objects, and L16 (Warn) on the Nomad for the
  margin line, which runs from the top edge (record `docs/bbq/2026-11-23-adr-starter-lint.md`).
- Starters must pass lint with no rhythm warnings (spec §13; arc board row 16 and release bar), so as the rules stood,
  no full-bleed paper starter could ship.
- L06 has no way to tell a deliberate full-bleed object from a mistake; L10 counted half the stroke, so anything
  reaching an edge was "partly off the page" (`crates/lint/src/lib.rs`, L10 block); L16 judges the frame's position
  against a centred rhythm, which a page edge rarely meets.

Drivers:
- Paper starters must look like paper, and like the tablet's own templates.
- A copy of a starter is the user's page: what they add to it must still be checked for safe zones.
- Spec §13's "pass lint" should mean the rules pass, not that they were turned off.

## Alternatives

### A Full bleed mark (chosen: the page stays checked; only what is meant to bleed is excused)
- Good: one tick per object, the shape Free placement already has; the rest of the page and anything added later is
  still checked; the starters pass §13 as written.
- Bad: a new field in the file format; one more tick in the Arrange tab.

### Ignore this rule for this document
- Good: exists today (rung 14b); no code.
- Bad: turns L06 and L10 off for the whole page, so a user's own text under the toolbar on a copy is never flagged; §13's
  "pass lint" becomes "pass with rules off".

### Keep paper starters inside the safe zones
- Good: no rule change.
- Bad: every paper starter gets a frame of blank paper, unlike the stock templates.

### Exempt starters from L06 and L10 in spec §13
- Good: no code.
- Bad: the gallery's own lint shows warnings on every paper starter.

For B (page edges), also weighed: the Full bleed mark also covering L16 (ties the rhythm to the mark), and Free
placement on the margin line (no change, but its other axis stops being checked too).

## Consequences

- Full-bleed paper starters can pass lint as §13 asks; the starters' margin lines also need ADR-020.
- The `.quad` format gains the mark (a minor version; older files open unchanged, and a marked page opened in an older
  version loses only the mark). ADR-007's rule for file-format changes applies.
- L10 fires less: lines and regions that sit on an edge no longer show "partly off the page".
- L16 fires less on every page: anything flush on an edge passes on that axis.
- Each part gets a test in `crates/lint/tests/` (a marked object skipped by L06 and L10, an unmarked one beside it
  still caught; an edge-sitting line with no L10; a flush object with no L16 on that axis and one still caught on the
  other).

## Asks left open

1. Ratify ADR-019 as written?

## History

- 2026-11-23: drafted from chris's answers A and B in the starter lint record, through makeadr.
- 2026-11-23: ratified by chris ("ratify 019 and 020"). Prior header values, word for word: status `proposed`,
  status-note `drafted 2026-11-23 from chris's answers (starter lint record, A and B); awaiting ratification`.
- 2026-11-23: built (lead session, after the ratification). Prior header values, word for word: built `not started`, proof
  `"queued before rung 16b (`docs/bbq/2026-11-23-adr-starter-lint.md`, Owed; `docs/arcs/v1/README.md` row 16)"`.
