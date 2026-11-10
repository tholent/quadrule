---
status: proposed
status-note: "proposed 2026-11-10 by an agent from a pattern in chris's rulings; never ruled; awaiting chris's word"
owner: chris
kind: process
built: not started
proof: "on ratification: a dated amendment under the flag rule in `AGENTS.md`"
supersedes: nothing
amends: "`AGENTS.md` §Engineering craft (\"Land a new feature switched off behind a flag\")"
consulted: Claude (Opus 5.5), who drafted it from the rung records
---
# ADR-007 — Editor features land switched on, and chris's look check, not a flag, is what holds a rung open

## Decision

**Proposed, not ruled.** For work inside the app's editor, a new feature lands switched on in the app build, with
no feature flag. The rung is not done until chris has tried it in the running app and approved its look; until then
`STATUS.md` says "waits for chris's look". The flag rule in `AGENTS.md` still holds for anything that changes export
bytes, the file format, or behaviour outside the app window, and for any job chris asks to be flagged.

## Context

- `AGENTS.md`: "Land a new feature switched off behind a flag, tested, until the owner approves how it looks."
- chris waived it three times running, each time against the agent's recommendation:
  - rung 3 Q11, "no flag: editing is on in the app build" (`docs/bbq/2026-11-10-rung-3-first-editor.md`);
  - rung 4 Q7, "no flag, as rung 3" (`docs/bbq/2026-11-10-rung-4-drag-and-snap.md`);
  - rung 5 Q6, "No flag, as rungs 3–4" (`docs/bbq/2026-11-10-rung-5-new-and-insert.md`).
- Rung 2 used a different gate: the whole app is an opt-in build (rung 2 Q9; ADR-008). That gate already keeps
  unfinished editor work out of plain `cargo build`.
- Each rung was still held open until chris's look (STATUS.md "Now").

Drivers:
- Stop re-asking a question chris answers the same way every rung.
- Keep a real gate before a rung is called done.
- Keep the flag where it protects users' files and exports.

## Alternatives

### Look check gates editor rungs (chosen: matches how every editor rung has actually run; proposed, not ruled)
- Good: no flag code to add and later remove; one fewer bbq question each rung.
- Bad: a half-liked feature is live in the app build until fixed (acceptable while pre-alpha, ADR-004).

### Keep the rule and waive it each rung
- Good: no rule change.
- Bad: the waiver is a ritual; the written rule says one thing and practice another.

### Flag everything, as written
- Good: the strictest.
- Bad: chris has overruled it three times.

## Consequences

- On ratification, `AGENTS.md` gets a dated amendment under the flag rule pointing here, and bbqs stop asking the
  flag question for editor rungs.
- Revisit when quadrule has users beyond chris (the first release), when a live half-finished feature costs more.

## Asks left open

1. Ratify, change the scope (for example "the whole app"), or turn it down.

## History

- 2026-11-10: proposed by an agent in chris's backlog sweep, from the three rung records cited above.
