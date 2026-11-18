---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify, asked: Ratify ADR-014 as written?)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: design
built: landed
proof: "`crates/layout/tests/layout.rs::pattern_shapes_are_centred_in_their_region_and_drawn_whole`, `crates/layout/tests/borders.rs::a_bordered_side_loses_the_lines_on_its_edge_and_a_side_without_keeps_them`, `crates/layout/tests/borders.rs::a_bordered_dot_grid_leaves_out_the_dots_on_the_edge_and_a_side_off_keeps_its_own`, `crates/layout/tests/borders.rs::a_rounded_corner_cuts_the_ruling_and_drops_dots_beyond_the_curve_and_only_where_both_sides_are_on`"
supersedes: nothing
amends: "napkin §5.2 (\"They clip to their bounds\")"
consulted: the rung 1 build agent (Opus 5.5), who chose it
---
# ADR-014 — A pattern region draws every line and dot whose centre lies inside it, whole, instead of clipping shapes at its edge

## Decision

A pattern region (ruled lines, dots, grid) draws each line or dot whose centre is inside the region, and draws it
whole, so it reaches at most half its stroke past the region's edge. A dot on the region's edge straddles the edge
rather than being cut in half. This is the rule today's exported pictures follow (rung 1's goldens,
`crates/render/tests/golden/SHA256SUMS`), and chris chose to keep it (2026-11-10: "let's amend 014 to align with
the currently exported pixtures").

## Context

- Napkin §5.2 says pattern regions "clip to their bounds" (`napkins/quadrule-spec.md`).
- During rung 1, clipping cut edge dots into half-dots, which look like mistakes on e-ink; trying "only shapes wholly
  inside" removed the sample page's first row and column of dots (`docs/STATUS-debt.md`, "Pattern rule").
- The rule lives in `crates/layout/src/lib.rs`; the goldens (`crates/render/tests/golden/SHA256SUMS`) depend on it.

Drivers:
- No cut shapes on the tablet.
- Keep the first row and column the design asks for.

## Alternatives

### Centre inside, drawn whole (chosen: no half-dots and no lost rows, and it is what the exports already do)
- Good: every shape on the page is whole; the rows the design places are all there.
- Bad: shapes may poke up to half their size past the region's edge.

### Clip at the bounds (napkin §5.2)
- Good: the region's edge is exact.
- Bad: half-dots and half-width lines at the edges.

### Only shapes wholly inside
- Good: exact edge and whole shapes.
- Bad: lost the sample's first dot row and column.

## Consequences

- No export bytes change; the goldens stand.
- A designer who needs an exact edge sets the region half a stroke inside it.
- On ratification, napkin §5.2 gets a dated amendment note.

## Asks left open

1. Ratify as written.

## History

- 2026-11-09: chosen by the rung 1 build agent for chris to veto (`docs/STATUS-debt.md`).
- 2026-11-10: written up as an ADR in chris's backlog sweep.
- 2026-11-10: chris, word for word: "let's amend 014 to align with the currently exported pixtures". Still proposed,
  so the draft was changed rather than amended: the Decision now states chris's choice and the half-stroke reach the
  test checks; the old ask ("Keep centre-inside-drawn-whole, or choose clipping or wholly-inside?") is answered.
  Prior header values: status-note `chosen 2026-11-09 by a build agent for chris to veto; never ruled; written up
  2026-11-10 in the backlog sweep`.
- 2026-11-10: ratified by chris (his answer "Ratify" to "Ratify ADR-014 as written?"). Prior header values: status
  `proposed`, status-note `chris chose 2026-11-10 to keep the rule the exports already follow; draft updated to say so;
  awaiting ratification`. Napkin §5.2 given its dated note, as Consequences says.
- 2026-11-13: amended by chris at the rung 9 look: slanted pattern lines are cut at the region's frame (see Amendment
  below). Header values unchanged.
- 2026-11-18: amendment drafted by the lead from chris's pattern-borders bbq rulings, see "Amendment — 2026-11-18,
  chris (proposed)" below; **awaiting chris's ratification at the borders look**. The proof header gained the border
  tests. Prior header value: proof `` `crates/layout/tests/layout.rs::pattern_shapes_are_centred_in_their_region_and_drawn_whole` ``.

## Amendment — 2026-11-13, chris

**What changed.** Slanted pattern lines (the isometric pattern's lines and the hex pattern's edges, in both
orientations) are **cut exactly at the region's frame**, so nothing reaches past it. Lines parallel to the frame
(ruled lines, grid lines, the isometric pattern's vertical lines, the hex pattern's flat edges) and dots keep the
Decision above: drawn whole when their centre is inside, reaching at most half a stroke past the edge.

**Why.** Rung 9's first build drew a hex edge whole when its middle was inside, so stubs reached up to half an edge
past the frame, far beyond the half stroke the Decision allows. The isometric lines were already cut at the frame (the
builder's choice). chris, asked "Hex edges at the region's border: today an edge whose middle is inside is drawn whole,
so stubs stick out up to half an edge (long on pointy top). Which do you want?", answered "Cut at the frame
(Recommended)" (the option read "Like the isometric lines: the region's edge is exact and nothing sticks out").

**Touches.** Decision. Record: `docs/bbq/2026-11-13-rung-9-pattern-library.md` (Amendments); built in rung 9
(`crates/layout/src/pattern.rs`).

## Amendment — 2026-11-18, chris (proposed)

**Status: proposed, awaiting chris's ratification at the pattern-borders look.** Touches the Decision and the
2026-11-13 amendment, for pattern regions that have a border only. A region with no border is unchanged, and so is
every side without a border. The rulings are chris's, from `docs/bbq/2026-11-18-pattern-borders.md`:

- **A bordered side leaves out what lies on its edge** (Q1, chris: "On the frame edge, skip edge lines
  (Recommended)"). The border is centred on the frame's edge. A line or dot whose centre is on a bordered edge is not
  drawn, so nothing doubles up with the border. "On the edge" means the centre is within half the sum of the pattern
  line's width and the border's width of that edge, and never less than half a device pixel, so the two inks never
  touch. The record's Defaulted list said half a pixel; the builder widened it after the first golden showed a line a
  pixel beside the border where the rhythm and the frame rounded differently (the record's "a wider band" veto).
  **This band is for chris to confirm or veto at the look.**
- **Rounded corners cut the pattern** (Q3, chris: "Rounded corners"; the curve rules are the record's Defaulted list).
  A corner rounds only when both of its sides have a border, and the radius is capped at half the shorter side.
  Inside a rounded corner, ruling lines are cut at the curve, as slanted lines are cut at the frame (2026-11-13), and a
  dot is dropped when its centre lies outside the curve or within the same band of it.
- The border turns with the frame (`rotation_deg`), not with the pattern's own angle (`angle_deg`).

**Why.** chris asked for pattern region borders as a selectable option (2026-11-18: "I need you to add pattern region
borders as a selectable option"). Without this change, a bordered ruled region would draw its edge lines twice, once as
pattern and once as border, and the pattern would stick out past a rounded corner.

**Touches.** Decision, and the 2026-11-13 amendment. Record: `docs/bbq/2026-11-18-pattern-borders.md`. Built by
`crates/layout/src/border.rs` and `crates/layout/src/pattern.rs`; tested in `crates/layout/tests/borders.rs` and
`crates/render/tests/pattern_borders.rs` (goldens on both devices). No existing golden moved (30 frozen lines intact).
