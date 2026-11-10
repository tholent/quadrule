---
status: proposed
status-note: "drafted 2026-11-10 from the project's records (backlog sweep, no rounds, chris's choice); awaiting ratification"
owner: chris
kind: process
built: n/a
proof: "`AGENTS.md`, `docs/decisions/README.md`, `STATUS.md`"
supersedes: nothing
amends: nothing
---
# ADR-004 — quadrule is run the pitmaster way: the repository is the shared memory, work climbs numbered rungs that each wait for chris's look, and commits go straight to main for now

## Decision

1. **The repository is the only shared memory** between sessions, agents and accounts. Live state is `STATUS.md`;
   each session appends to `docs/STATUS-log.md`; owed work is `docs/STATUS-debt.md`; finished entries move word
   for word to `docs/STATUS-archive.md`; plans are bbq records in `docs/bbq/`; decisions are ADRs here.
2. **Decisions are ADR+ records** in `docs/decisions/`, numbered `ADR-NNN-slug.md` from 001, owner chris. Agents
   propose; only chris ratifies. Ruled text is never edited: it is amended (a dated section) or superseded.
3. **Work climbs numbered rungs** (rung 1, 2, 3 … with parts such as 4a and 4b). Each rung is planned by a bbq whose
   record is the prompt, and is called done only when chris approves its look in the running app.
4. **Taste, naming and direction are chris's.** An agent's choice on any of them is a proposal, marked for veto.
5. **Commits go straight to `main`** until chris says otherwise or a job needs a branch (chris, 2026-11-09:
   "until specifically necessary, let's just work on the main branch directly. This is pre-alpha software at this
   point."). This amends `AGENTS.md`'s branch rule there, as a dated amendment.

## Context

- The project was prepared with pitmaster's project-prep on 2026-11-09 (`docs/STATUS-log.md`, first entry), which
  wrote `AGENTS.md` (with `CLAUDE.md` pointing to it) and `STATUS.md`. chris chose ADR+ as the template on 2026-11-09
  (`docs/decisions/README.md`).
- Rungs so far: rung 1 headless slice (`docs/bbq/2026-11-09-rung-1-headless-slice.md`), rung 2 viewer
  (`docs/bbq/2026-11-09-rung-2-viewer.md`), rung 3 first editor (`docs/bbq/2026-11-10-rung-3-first-editor.md`),
  rung 4 drag and snap (`docs/bbq/2026-11-10-rung-4-drag-and-snap.md`), rung 5 planned
  (`docs/bbq/2026-11-10-rung-5-new-and-insert.md`). Each was done on chris's look (STATUS.md "Now").
- The work-on-main ruling followed PR #1 (`docs/STATUS-log.md`, 2026-11-09).
- `docs/STATUS-debt.md` listed "The project-setup decision is not on record yet"; this ADR pays it.

## Alternatives

### Branch and PR for every change (the original `AGENTS.md` rule)
- Good: every change reviewed before `main`.
- Bad: chris judged the overhead not worth it for pre-alpha software; it is kept as the rule to return to.

## Consequences

- Any agent can pick up from `STATUS.md` and the newest record without the chat that made it.
- With no PR, CI on `main` is the only automatic gate; a red run must be fixed on `main` at once.
- Doctrine (the project's rule for choosing) is still unset; chris sets it, agents do not invent it (`AGENTS.md`).

## History

- 2026-11-10: drafted from `AGENTS.md`, `docs/decisions/README.md` and the records cited above, in a backlog sweep
  chris asked for ("catch the backlog of decisions I have up until this point").
