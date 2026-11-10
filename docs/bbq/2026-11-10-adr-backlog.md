# makeadr record — quadrule ADR backlog sweep

2026-11-10 · makeadr, mode new, many at once · no rounds (cited from records) · written by this session (Opus 5.5).

chris, word for word: "I feel like I haven't commited many of the decisions to ADRs yet and want to make sure to catch
the backlog of decisions I have up until this point."

Searched: `docs/bbq/` (all 8 records), `docs/STATUS-log.md`, `docs/STATUS-debt.md`, `docs/proposals/`,
`docs/decisions/` (ADR-001 to ADR-003), `AGENTS.md`, `spikes/*/FINDINGS.md`. Already covered by ADRs: the file format
with the hinting talk's D1–D7 (ADR-001, which cites `docs/proposals/talk-2026-11-09-format-versions-and-hinting.md`),
the text stack (ADR-002), the export renderer (ADR-003). Left out as not ADR-sized: calibration readings (kept in
`docs/device-measurements/*/calibration.md`), README wording, one-off acts (the history rewrite).

## Rules

- **S1 Scope.** All four groups of the backlog list: process, architecture, design, and the never-ruled choice.
- **S2 Method.** From the records, no rounds: each claim cited to its record; all land as `proposed` for chris to
  ratify one by one.
- **S3 Grouping.** Merge the small ones: clipboard into the editing model, the speed line into the canvases, Fluent
  into the code layout, safe zones into roles and profiles. 11 ADRs.
- **S4 Who writes.** This session, now.

## Made

| ADR | about | asks left open |
|---|---|---|
| ADR-004 | project setup and way of working (pays the debt item) | — |
| ADR-005 | public, Apache-2.0, headers, `quadrule-*` | — |
| ADR-006 | the Pi, CI on three systems with path filters, coverage, `CHECK` lines, Makefile | — |
| ADR-007 | look check, not a flag, gates editor rungs | **never ruled**: ratify, rescope or turn down |
| ADR-008 | stack, crates by rung, opt-in app build, Fluent, `spikes/` untouched | — |
| ADR-009 | two canvases, 100 ms fallback, 95th percentile under 50 ms | the 95 and the 100 ms were agents' |
| ADR-010 | editing model, autosave, clipboard format | — |
| ADR-011 | snapping | the 6-point reach and tie order; what "rhythm" and "half a pixel" mean |
| ADR-012 | seven roles, calibrated profiles, safe zones | — |
| ADR-013 | window layout, theme, font, empty start, export names | — |
| ADR-014 | pattern edges: centre inside, drawn whole | **never ruled**: keep, or clip, or wholly-inside |

Checks: `adr-index.py` → 14 ADRs; `adr-check.py --triggers-from 1` → 0 failures (it checks headers and that each proof's
file and test name exist; it does not run the tests).

## Defaulted

- **Landing:** committed straight to `main` (ADR-004 item 5); makeadr's default of a branch and PR is overruled by the
  project rule. veto: none expected.
- **No next-agent prompt:** these record past work; the next build already has its prompt
  (`docs/bbq/2026-11-10-rung-5-new-and-insert.md`). veto: ask for one.
- **ADR-007 amends `AGENTS.md` only on ratification;** nothing in `AGENTS.md` changed now. veto: none expected.
