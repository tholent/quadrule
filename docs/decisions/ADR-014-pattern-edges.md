---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify, asked: Ratify ADR-014 as written?)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: design
built: landed
proof: "`crates/layout/tests/layout.rs::pattern_shapes_are_centred_in_their_region_and_drawn_whole`"
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
