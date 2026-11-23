---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"ratified\")"
ratified-by: chris
ratified-on: "2026-11-23"
owner: chris
kind: process
built: partial
proof: "`.github/scripts/check-github-rules.sh`"
owed: "push protection: off when read back on 2026-11-23 (`make check-github` → missing=push-protection)"
supersedes: nothing
amends: nothing
---
# ADR-022 — GitHub guards `main` and the release tags now, and adds pull requests and required checks only when work leaves `main`

## Decision

1. **`main` cannot be rewritten or deleted.** A GitHub ruleset on the default branch blocks force pushes and deletion
   (Q1). Nothing else about `main` changes: agents and chris keep committing and pushing to it directly, as AGENTS.md's
   amendment of 2026-11-09 allows.
2. **A release tag stays put.** A tag ruleset on `v*`, excluding `v*-test*`, blocks deleting and moving a tag (Q2).
   Creating one stays open, so a release is still made by pushing its tag, and a test tag can still be deleted after a
   dry run (`DEVELOPMENT.md`, "Releases and packages").
3. **Pull requests and required checks wait** (Q3). They are turned on with the ruling that ends work on `main`
   directly, not before. When they are, `main` also gets: a pull request before merging, with 0 approvals (GitHub does
   not let an owner approve their own); the three CI jobs `ubuntu-latest`, `macos-latest` and `windows-latest`
   required; chris allowed to bypass. Before that, CI is changed to run on every pull request, because its path
   filter would leave a pull request that changes only documents waiting forever on checks that never run.
4. **Secrets and old branches** (Q4). Secret scanning with push protection is on, so a password or token is refused at
   the push; branches are deleted automatically once merged; the merged `batch-2-menus` branch is deleted.

The two rulesets have no bypass list, and the `badges` branch has no rule (Defaulted).

## Context

The repository is public and its only shared memory between sessions, agents and accounts (AGENTS.md). On 2026-11-23
it had no rulesets and no branch protection (`gh api repos/tholent/quadrule/rulesets` → `[]`; `main` → "Branch not
protected"), and secret scanning, push protection and automatic branch deletion were off. Agents push to `main` with
chris's credentials, which carry admin rights. Releases are immutable once published, but a tag could still be moved or
deleted before its release publishes. CI runs only when code files change (`.github/workflows/ci.yml`, `paths`), and
the coverage workflow writes the badge number to the `badges` branch on every run (`.github/scripts/publish-badge.sh`).
Premises with their sources: `docs/bbq/2026-11-23-adr-branch-protections.md`.

Drivers:
- Keep the way of working chris ruled (work on `main` directly) while it lasts.
- Block the few acts no one should ever take: rewriting `main`'s history, deleting it, moving or deleting a release tag.
- Leave nothing that waits forever (a required check that never runs) or breaks quietly (the badge).

## Alternatives

### Guard now, require pull requests later (chosen: protects history without changing how work lands)

- Good: one ruleset each for `main` and the tags; nothing in the daily flow changes; undone by switching a ruleset off.
- Bad: a broken commit can still reach `main`, as today; CI reports it after the push, not before.

### Pull requests and required checks now

- Good: nothing reaches `main` without CI passing on three systems.
- Bad: ends chris's 2026-11-09 ruling; every change, documents too, waits 12 to 22 minutes for CI; CI's path filter
  must be changed first.

### Linear history on `main` too

- Good: `main` stays one straight line, as branches have been landed so far (fast-forward).
- Bad: GitHub's merge button would refuse a merge commit; not asked for. Not chosen (Q1).

### Only chris may create release tags

- Good: guards a collaborator from starting a release.
- Bad: agents push as chris, so it would not stop them; no other collaborator yet. Not chosen (Q2).

## Consequences

- A force push to `main`, deleting `main`, and deleting or moving a `v*` tag (not a test tag) are refused by GitHub.
- A push holding something that looks like a secret is refused; the pusher is told which line.
- Merged branches disappear on their own; an unmerged one stays.
- Switching to pull requests later is a ruling plus a CI change, then one more rule in the same ruleset.

**Confirmed by:** `gh api repos/tholent/quadrule/rulesets` listing the two rulesets with their rules, and `gh api
repos/tholent/quadrule` showing secret scanning, push protection and `delete_branch_on_merge` on; there is no test, as
the CI token cannot read repository settings.

**Revisit when:** chris ends work on `main` directly (item 3 then applies), or a second person gets write access.

## History

- 2026-11-23: drafted by the lead from chris's answers to four questions (record
  `docs/bbq/2026-11-23-adr-branch-protections.md`), after chris asked what protections the repository should have.
- 2026-11-23: ratified by chris, word for word: "ratified". Earlier values, word for word: `status: proposed`;
  `status-note: drafted 2026-11-23 from chris's answers (docs/bbq/2026-11-23-adr-branch-protections.md); awaiting
  ratification`.
- 2026-11-23: applied by chris in the browser (the lead's token could not: 403), read back by the lead with
  `.github/scripts/check-github-rules.sh` (`make check-github`): both rulesets active with no bypass (the tag ruleset
  also blocks force pushes, which repeats Restrict updates), merged branches deleted, secret scanning on,
  `batch-2-menus` deleted; push protection off. Earlier values, word for word: `built: not started`; `proof: "applied
  by the lead with gh after ratification (docs/bbq/2026-11-23-adr-branch-protections.md, Defaulted)"`.
