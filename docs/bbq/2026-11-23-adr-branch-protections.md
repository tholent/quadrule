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

(Filled in as each round returns.)

## Owed

None yet.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| Rulesets, not classic branch protection | GitHub's newer mechanism: covers tags as well as branches, can be switched off without deleting, several can stack | veto: classic branch protection for `main` |
| The `badges` branch gets no rule | the coverage workflow writes it on every run; a rule would break the badge | veto: protect it and give the workflow a bypass |
| The two rulesets have no bypass list | they block only what nobody should do (rewriting `main`, deleting or moving a release tag); a ruleset can be switched off in Settings if ever needed | veto: chris as a bypass actor |
| The lead applies the rulesets with `gh api` once the ADR is ratified, and reads them back | the token has admin rights; one command each, undone by deleting the ruleset | veto: chris applies them in the browser |
