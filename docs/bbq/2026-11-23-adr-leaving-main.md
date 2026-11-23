# makeadr: when work leaves `main` (2026-11-23)

New ADR (ADR-024), 4 questions (chris: "4 (Recommended)"). Written by this session (Opus 5.5, high).

## Premises

| Premise | Source (checked 2026-11-23) |
|---|---|
| Agents commit and push to `main` directly "until specifically necessary" (pre-alpha) | `AGENTS.md`, amendment of 2026-11-09 |
| The rule's shape when work leaves `main` is already ruled: a pull request with 0 approvals, the three CI jobs required, chris may bypass, CI first made to run on every pull request | `docs/decisions/ADR-022-github-branch-and-tag-rules.md`, Decision item 3 (ratified) |
| CI runs only when code files change (paths filter); its three jobs take 12 to 22 minutes | `.github/workflows/ci.yml`; `docs/STATUS-log.md` (runs 45242627211, 45255174131) |
| `make check-github` reads the rules back | `.github/scripts/check-github-rules.sh` |

## Rules

(Filled in as each round returns.)

## Owed

None yet.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| (Filled in at the end.) | | |
