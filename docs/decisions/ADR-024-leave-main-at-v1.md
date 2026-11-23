---
status: proposed
status-note: drafted 2026-11-23 from chris's answers (docs/bbq/2026-11-23-adr-leaving-main.md); awaiting ratification
owner: chris
kind: process
built: not started
proof: "queued for v1.0: docs/STATUS-debt.md (ADR-024) and docs/arcs/v1/README.md (release bar)"
supersedes: nothing
amends: nothing
---
# ADR-024 — Work goes straight to `main` until v1.0; from then on it lands by pull request, merged by the agent on green

## Decision

1. **Until v1.0, nothing changes.** Agents and chris keep committing and pushing to `main` directly, as AGENTS.md's
   amendment of 2026-11-09 allows, whoever has write access in the meantime (Q1).
2. **At v1.0, ADR-022 item 3 is switched on**, before the v1.0 tag: `main` takes a pull request before merging (0
   approvals), the three CI jobs `ubuntu-latest`, `macos-latest` and `windows-latest` are required, and chris may
   bypass. AGENTS.md's work-on-`main` amendment is closed by a dated amendment pointing here.
3. **An agent lands its own work** from then on: it opens the pull request and merges it when CI is green. It never
   uses the bypass. chris's looks and ratifications stay the gate for taste, as now (Q2).
4. **Documents do not wait on CI.** A pull request that changes only documents gets the three required check names
   reported by a quick job in seconds; a change to code still gets the full three-system run (Q3).

## Context

ADR-022 (ratified 2026-11-23) put rulesets on `main` and the release tags now and laid out what `main` gets once work
leaves it (item 3), leaving *when* to a later ruling. AGENTS.md lets work go straight to `main` "until specifically
necessary … This is pre-alpha software at this point" (amendment of 2026-11-09). CI runs only when code files change
(`.github/workflows/ci.yml`, paths filter) and takes 12 to 22 minutes (`docs/STATUS-log.md`), so a required check on a
documents-only pull request would never arrive without a change. Premises with sources:
`docs/bbq/2026-11-23-adr-leaving-main.md`.

Drivers:
- Keep today's pace while quadrule is pre-alpha.
- From v1.0, nothing reaches `main` without CI passing on three systems.
- No pull request may wait forever, and documents should not cost a full CI run.

## Alternatives

### Switch at v1.0 (chosen: chris's ruling)

- Good: no change to how work lands until the release that matters most; one switch, done with the v1.0 work.
- Bad: if someone else gets write access before v1.0, their pushes reach `main` unchecked too.

### Switch at the end of pre-alpha, or sooner if someone else gets write access (overruled)

- Good: guards `main` as soon as anyone besides chris and his agents can push.
- Bad: the switch could come in the middle of an arc. The lead's recommendation; chris chose v1.0 (Q1).

### Switch now

- Good: every change checked before it lands.
- Bad: every change waits 12 to 22 minutes; ends the pre-alpha ruling early.

### The agent opens the pull request and chris merges, or chris opens it

- Good: chris sees each change before it lands.
- Bad: work waits on chris; the taste gate is already his looks and ratifications. Not chosen (Q2).

### Full CI on every pull request, documents too

- Good: the simplest CI file.
- Bad: every documents change waits 12 to 22 minutes and uses runner time. Not chosen (Q3).

## Consequences

- Until v1.0: no change on GitHub, in AGENTS.md or in how sessions land work.
- The switch is one change made before the v1.0 tag: the documents-only CI job, the `main` ruleset gaining the pull
  request and required-checks rules with chris on its bypass list, the AGENTS.md amendment, and
  `.github/scripts/check-github-rules.sh` checking the new rules (record, Defaulted).
- After it, an agent's session ends a piece of work with a merged pull request, not a push.

**Confirmed by:** `make check-github` once the switch is made, extended to read the new rules.

**Revisit when:** v1.0 is near (the switch is due), or someone besides chris gets write access before it (the case
Q1 accepted).

## History

- 2026-11-23: drafted from chris's answers to four questions (`docs/bbq/2026-11-23-adr-leaving-main.md`), deciding when
  ADR-022 item 3 applies.
