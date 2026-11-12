---
status: proposed
status-note: drafted 2026-11-12 from chris's ruling at the rung 7a look and the rung 7 record (goforit); awaiting ratification
owner: chris
kind: design
built: landed
proof: "`crates/render/tests/groups.rs::grouping_and_ungrouping_with_every_constraint_changes_no_pixel`"
supersedes: nothing
amends: "napkin §5.2 (\"Group | Transform and constraint container\") and §5.3 (\"`%` is relative to the parent (page or group)\")"
consulted: the rung 7 lead (Opus 5.5) and the rung 7a builders (Sonnet 5.5), who built both rules
---
# ADR-015 — A group is see-through to constraints, as in Figma: every object inside keeps its own constraints against the page, and a group only moves, turns, fades and scales what it holds

## Decision

A group holds objects (and other groups, to any depth) so they can be picked, moved, turned, faded and resized as
one, but it is **not a constraint container**. Each object inside a group keeps its own horizontal and vertical
constraints (spec §5.4) and resolves against the **page** on every device, exactly as if it were not in a group. A
group has no constraints of its own: its stored `constraints` are ignored and it has no Constraints tab. A group's box
on any device is the outer box of its resolved children. The `%` unit in a field is of the page, inside a group too.
Resizing a group by its handles scales everything inside it, as a loose selection does. So **grouping or ungrouping
never changes how a page looks on any device.** This is how Figma's groups behave; chris chose it at the rung 7a
look (2026-11-12): "I approve the build, but would like to change question #1 to reflect what figma does".

The rest of the groups model, ruled in the same rung and unchanged by this ADR: a group keeps its own angle and its
children's angles add to it, and a group holding a text box or a pattern region does not turn (rung 7 record Q2); a
click picks the outermost group, a double-click goes one level in, Escape steps out, Ctrl+click picks the innermost
object (Q4); a group below 100 % opacity fades as one (Q7).

## Context

- The spec lists "Group | ✅ | Transform and constraint container" (`napkins/quadrule-spec.md` §5.2) and says "`%` is
  relative to the parent (page or group)" (§5.3). Rung 7's bbq built that reading: Q3, "Each child obeys its own
  constraints against the group's box" (`docs/bbq/2026-11-11-rung-7-groups-and-align.md`).
- At the 7a look, the lead showed chris the cost: a render test grouping the slice page's title with its rule left
  the Manta's picture unchanged but changed 48,319 pixels on the Nomad, because the children followed the group's box,
  which itself scales with the page (`docs/STATUS-log.md`, 2026-11-12, the 7a builder's entry, choice 7).
- chris's ruling is recorded word for word as the Q3 amendment in the rung 7 record and as a dated note under spec
  §5.2. Built in the 7a look change (`eab048d` to `62cc6ca`) and released in `v0.7.0`.

Drivers:
- Grouping is a tidying act; it should never change what a device shows.
- quadrule's whole point is one design that adapts to several devices (spec §3.4), so the rule for how an object
  adapts should not depend on whether it happens to sit in a group.
- Familiar behaviour: Figma users expect groups to pass constraints through.

## Alternatives

### See-through groups, as in Figma (chosen: grouping never changes any device's picture)
- Good: grouping and ungrouping are free of side effects on every device (proved for all 25 pairs of constraints,
  nested too); one rule for how objects adapt; matches Figma.
- Bad: there is no way, yet, to make a box whose contents pin to *it* rather than to the page (Figma's frames); a
  header block that should stretch as a unit needs each object pinned on its own.

### Groups as constraint containers (overruled at the look: Q3 as first ruled, the lead's recommendation)
- Good: matches the spec's wording; a block's parts can pin to the block's edges.
- Bad: grouping silently changes the other device's picture (48,319 pixels in the test); two rules for adapting.

### Groups scale their contents like a picture between devices
- Good: simple to explain.
- Bad: ignores every child's constraints, so a header pinned to the top would stretch with the group. Kept only for
  resizing a group by hand in the editor, where it is what users expect.

## Consequences

- Easier: rung 8's per-device work (secondary profiles, overrides, previews) has one rule for every object, grouped or
  not; a group's box on each device is derived, never stored per device.
- Harder: pinning several objects to a shared box needs a future container kind (a frame or component, spec §5.2
  "Component instance", v1.1). If one is added, it becomes the constraint container, not the group.
- Debt (`docs/STATUS-debt.md`, rung 7a line): resizing a group by hand stretches turned children approximately, as a
  loose selection does.

Revisit when: a frame or component kind is planned (v1.1 or later), since that is where a constraint container would
live.

## History

- 2026-11-12: drafted by the rung 7 lead with makeadr (goforit: every part cited, no rounds), after chris's ruling at
  the rung 7a look. Record `docs/bbq/2026-11-12-adr-groups.md`.
