---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"Ratify\", for ADR-024)"
ratified-by: chris
ratified-on: "2026-11-23"
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
- 2026-11-23: ratified by chris ("Ratify", asked "Ratify ADR-024 … as written?"). Earlier values, word for word:
  `status: proposed`; `status-note: drafted 2026-11-23 from chris's answers (docs/bbq/2026-11-23-adr-leaving-main.md); awaiting ratification`.
- 2026-11-24: amended by chris (see Amendment — 2026-11-24, chris, below): until v1.0, an agent pushes a release tag
  through a local Claude Code permission rule, and runs each git command on its own.

## Amendment — 2026-11-24, chris

**What changed.** Item 1 ("Until v1.0, nothing changes") also covers how an agent gets a release tag past Claude Code
on the computer it runs on, not only past GitHub. On 2026-11-23 the `v0.16.6` tag push was refused by Claude Code's
auto-mode check, though ADR-022 item 2 leaves creating tags open and item 1 here lets agents push. chris asked how
agents were to have access until v1.0, then chose a local permission rule ("yes, please add rule #1"), and then said,
word for word: "let's do the amendment, but also put an amendment somewhere that states that you cannot use compound
git commands".

1. **Until v1.0, a release tag goes out through a local permission rule.** `.claude/settings.local.json` allows
   `Bash(git tag -a v*)` and `Bash(git push origin v*)`. The file is git-ignored (`.gitignore`, `/.claude/`), so it
   holds only on the computer that has it; a session on another computer or account that is refused the same way
   adds the same two lines there, on chris's word. An agent still asks chris before each release, as the release
   checklist says (setfree); the rule removes only the auto-mode refusal, not the question.
2. **Each git command runs as a shell call of its own**, never joined to another command with `&&`, `;`, `||` or a
   pipe (`AGENTS.md`, Run and delegate). The refused push was one call joining `git tag`, `git push`, `sleep` and
   `gh run list`.
3. **At v1.0** the switch of item 2 also reviews the rule of point 1: kept as it is, narrowed, or removed.

**Sections it touches.** Decision item 1, and Consequences ("Until v1.0: no change on GitHub, in AGENTS.md or in how
sessions land work"): AGENTS.md gains the rule of point 2, and how a release tag is pushed now depends on a file on
each computer.

**Why.** The ADRs covered GitHub's rules but not the check on the computer itself, so a release they allowed was
refused, and the reason was found only in the log.

**Who agreed.** chris, in the release session of 2026-11-23 and 2026-11-24 (his words above). Reading "compound git
commands" as any shell call that joins a git command to another one is the lead's, and can be vetoed.
