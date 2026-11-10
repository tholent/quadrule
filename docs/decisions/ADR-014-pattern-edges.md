---
status: proposed
status-note: "chosen 2026-11-09 by a build agent for chris to veto; never ruled; written up 2026-11-10 in the backlog sweep"
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

**Chosen by an agent, not yet ruled.** A pattern region (ruled lines, dots, grid) draws each line or dot whose
centre is inside the region, and draws it whole. A dot on the region's edge straddles the edge rather than being cut
in half.

## Context

- Napkin §5.2 says pattern regions "clip to their bounds" (`napkins/quadrule-spec.md`).
- During rung 1, clipping cut edge dots into half-dots, which look like mistakes on e-ink; trying "only shapes wholly
  inside" removed the sample page's first row and column of dots (`docs/STATUS-debt.md`, "Pattern rule").
- The rule lives in `crates/layout/src/lib.rs`; the goldens (`crates/render/tests/golden/SHA256SUMS`) depend on it.

Drivers:
- No cut shapes on the tablet.
- Keep the first row and column the design asks for.

## Alternatives

### Centre inside, drawn whole (chosen: no half-dots and no lost rows; an agent's choice, for veto)
- Good: every shape on the page is whole; the rows the design places are all there.
- Bad: shapes may poke up to half their size past the region's edge.

### Clip at the bounds (napkin §5.2)
- Good: the region's edge is exact.
- Bad: half-dots and half-width lines at the edges.

### Only shapes wholly inside
- Good: exact edge and whole shapes.
- Bad: lost the sample's first dot row and column.

## Consequences

- Ruling for clipping or wholly-inside would change export bytes and the goldens, so it is its own small job.
- On ratification, napkin §5.2 gets a dated amendment note.

## Asks left open

1. Keep centre-inside-drawn-whole, or choose clipping or wholly-inside?

## History

- 2026-11-09: chosen by the rung 1 build agent for chris to veto (`docs/STATUS-debt.md`).
- 2026-11-10: written up as an ADR in chris's backlog sweep.
