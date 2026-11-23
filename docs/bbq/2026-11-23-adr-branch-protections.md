# makeadr: GitHub branch and tag rules (2026-11-23)

New ADR (ADR-022), 4 questions (chris: "4 (Recommended)"). Written by this session (Opus 5.5, high), from the
proposal it made when chris asked what branch protections the repository should have (`docs/STATUS-log.md`,
2026-11-23, v0.16.5).

## Premises

| Premise | Source (checked 2026-11-23) |
|---|---|
| The repository has no rulesets and `main` has no branch protection | `gh api repos/tholent/quadrule/rulesets` → `[]`; `gh api .../branches/main/protection` → "Branch not protected" (404) |
| Agents commit and push to `main` directly until chris says otherwise | `AGENTS.md`, amendment of 2026-11-09 |
| The signed-in `gh` token has admin rights on the repository | `gh api repos/tholent/quadrule --jq .permissions` → `admin: true` |
| CI runs only when code files change (paths filter), on pushes to `main` and on pull requests | `.github/workflows/ci.yml`, `on.push.paths`, `on.pull_request.paths` |
| CI's three jobs are named `ubuntu-latest`, `macos-latest`, `windows-latest` | `.github/workflows/ci.yml`, `jobs.check.name: ${{ matrix.os }}` |
| The coverage workflow pushes the badge number to the `badges` branch | `.github/scripts/publish-badge.sh` line 38 |
| A release is made by pushing its tag; test tags (`-test` in the name) are deleted afterwards | `DEVELOPMENT.md`, "Releases and packages" |
| Releases are immutable: a published release's files cannot change, a deleted release's tag name cannot be reused | `DEVELOPMENT.md` line 140 |
| No earlier ADR or record rules on branch or tag protection | `grep -rln -i "branch protection\|ruleset" docs/decisions docs/bbq` → nothing |

## Rules

**Q1 `main`.** chris: "Block force pushes and deletion (Recommended)". A ruleset on the default branch blocks force pushes
and deletion; nothing else changes, agents keep pushing to `main`.

**Q2 Release tags.** chris: "Block deleting and moving v* tags, test tags excepted (Recommended)". A tag ruleset on `v*`,
excluding `v*-test*`, blocks deletion and moving (update); creating a tag stays open.

**Q3 Pull requests and required checks.** chris: "Later, with the ruling that ends work on main (Recommended)". Not turned
on now. Written as the plan: when chris ends work on `main` directly, `main` also gets a pull request first (0 approvals),
the three CI jobs (`ubuntu-latest`, `macos-latest`, `windows-latest`) required, chris allowed to bypass, and, first, CI
changed to run on every pull request (its path filter would leave a docs-only pull request waiting forever).

**Q4 Anything else.** chris: "Also secret scanning and branch cleanup". Turn on secret scanning with push protection and
automatic deletion of merged branches, and delete the `batch-2-menus` branch (fully merged: `git log
origin/main..origin/batch-2-menus` → 0 commits, checked 2026-11-23).

## Owed

None.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| Rulesets, not classic branch protection | GitHub's newer mechanism: covers tags as well as branches, can be switched off without deleting, several can stack | veto: classic branch protection for `main` |
| The `badges` branch gets no rule | the coverage workflow writes it on every run; a rule would break the badge | veto: protect it and give the workflow a bypass |
| The two rulesets have no bypass list | they block only what nobody should do (rewriting `main`, deleting or moving a release tag); a ruleset can be switched off in Settings if ever needed | veto: chris as a bypass actor |
| The lead applies the rulesets and the Q4 settings with `gh api` once the ADR is ratified, and reads them back | the token has admin rights; one command each, undone by deleting the ruleset or switching the setting off | veto: chris applies them in the browser |
| Deleting `batch-2-menus` is the only one-way step; it is done last, after the read-back | the branch is fully merged into `main`, so nothing is lost | veto: keep the branch |

## Outcome

The last question: chris, "Just make the ADR (Recommended)". ADR-022 written, proposed
(`docs/decisions/ADR-022-github-branch-and-tag-rules.md`); `adr-check`: 22 ADRs, 0 failures. Nothing applied on GitHub
until chris ratifies.

## Correction (2026-11-23, after ratification)

The premise "the signed-in `gh` token has admin rights" was wrong. `.permissions.admin: true` is chris's role on the
repository, not what the token may do: `gh api -X POST repos/tholent/quadrule/rulesets` → `403 Resource not accessible
by personal access token` (the token lacks Administration: write; the same 403 `gh repo edit` met on 2026-11-10). So the
Defaulted call "the lead applies them with gh" falls to its veto: chris applies them in the browser, or gives the
token Administration: write, and the lead reads them back.
