# Budget calibration log

What each lead session spent against its meter, so later budgets (`budget-select`) can price work from real numbers.
Made on first use, as the rung 6 record's Budget block asks (`docs/bbq/2026-11-10-rung-6-shapes.md`). One row per
session, newest at the bottom; the numbers come from the `spend-ledger` skill's script, named with each row.

## Sessions

| date | session | lead | window (UTC) | tokens | workers | meter | landed | note |
|---|---|---|---|---|---|---|---|---|
| 2026-11-11 | quadrule [57701332] | Opus 5.5, high | 01:06–23:48 | fresh 704 · cached 74.1M · written 1.5M · out 354k | 81 % (2 files; Sonnet 81 %, Opus 19 %) | weekly 50 → 55 %, alone on it, 5 points | 12 commits to `main`, tags `v0.6.0`, `v0.6.1` | rung 6a and 6b, two Sonnet 5.5 high builder subagents; list price $23.48 |
| 2026-11-12 | quadrule [3e22d7d3] | Opus 5.5, high | 00:01–05:18 (then waits for chris) | fresh 788 · cached 92.1M · written 1.3M · out 416k | 83 % (3 files; Sonnet 83 %, Opus 17 %) | weekly 55 → 64 %, alone on it, 9 points of a 10-point share | rung 7 (7a, its look change, 7b), tags `v0.7.0`, `v0.7.1` | three Sonnet 5.5 high builder subagents (0.49M, 0.21M, 0.21M); the share line fired in flight (63 % before 7b ended, 64 % after); overrun 0; no other sessions; list price $27.28 |

Command: `spend-ledger.py measure --latest --meter 50 55 --lead "Opus 5.5, high"`. The start reading came from chris as
"about 46–50 %" and was logged as 50, so the real spend may be up to 9 points; 55 % is from chris's `/usage` on
2026-11-11.

## Fitted K

`spend-ledger.py calibrate --since 2026-11-10` → one kept pair, 2026-11-11T01:07:35Z to 23:48:25Z, weekly meter, 5
points, 75.3M tokens (cached reads included), **K = 15.1M tokens per point**, $4.61 per point. Not yet trusted:
`budget-select` wants three pairs.

The rung 6 Budget block's starting K (0.79M tokens per point) counts a different thing (session context, not cached
reads), so its ledger estimate of about 1.8 points was far below the meter's 5. Until three pairs are fitted, trust the
meter's movement over that estimate.

2026-11-12, rung 7: `spend-ledger.py measure --latest --meter 55 64 --lead "Opus 5.5, high"` gave the row above (measured
through 05:18 UTC; the closing work after chris's approval is not in it). `spend-ledger.py calibrate --since 2026-11-10` →
two kept pairs (15.1M and 10.5M tokens per point) and one small move flagged; **K = 12.2M tokens per point over 2 pairs**,
still not trusted (three wanted). The start reading, 55 %, was chris's "Still about 55 %".
