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

**Q1 When.** chris: "At v1.0". Work goes straight to `main` until v1.0 ships, whoever has write access before then; then
ADR-022 item 3 is switched on. *Overruled:* the lead's recommendation "At the end of pre-alpha, or sooner if someone else
gets write access".

**Q2 Landing.** chris: "The agent opens the pull request and merges it on green (Recommended)". The agent never uses
the bypass; chris's looks and ratifications stay the taste gate.

**Q3 CI.** chris: "A quick job that passes the checks for docs-only changes (Recommended)". Code changes keep the full
three-system run; a pull request that changes only documents gets the same three check names reported in seconds.

**Q4 Anything else.** chris: "Nothing more (Recommended)".

## Owed

None.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| The switch is one change at v1.0: the CI docs-only job, the `main` ruleset's pull request and required-checks rules (chris on the bypass list), AGENTS.md amended, `.github/scripts/check-github-rules.sh` checking the new rules; done before the v1.0 tag | ADR-022 item 3, Q1–Q3 | veto: switch on after the v1.0 tag |
| Until then nothing changes on GitHub or in AGENTS.md | Q1 | none |

## Outcome

The last question: chris, "Just make the ADR (Recommended)". ADR-024 written, proposed
(`docs/decisions/ADR-024-leave-main-at-v1.md`); queued for v1.0 in `docs/STATUS-debt.md` and the board's release bar;
ADR-022's History points at it.
