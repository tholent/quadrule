# STATUS-archive — quadrule

Finished entries moved out of `STATUS.md`, word for word, with the date they moved. Newest at the bottom.

## Moved 2026-11-09 (rung 1 session)

From Now:
- Project prepared for agents with project-prep: the rules are in `AGENTS.md`, the split files in `docs/`.
- Design spec is a draft (v0.1) (`napkins/quadrule-spec.md`); plan for the first work in
  `docs/bbq/2026-11-09-quadrule-spikes.md`.
- The three technical spikes are done on branch `spikes/2026-11-09` (local only): format, text and rendering, each a
  throwaway crate with findings and a proposed ADR (`spikes/format/FINDINGS.md`, `spikes/text/FINDINGS.md`,
  `spikes/render/FINDINGS.md`). Tests: format 10, text 12, render 9 passed.
- Rust 1.99.1 installed on the Pi with rustup (`spikes/format/FINDINGS.md` header).

From Next:
- Three ADRs ratified 2026-11-09, none built yet: `.quad` format (ADR-001), text stack (ADR-002), export renderer
  (ADR-003) (`docs/decisions/INDEX.md`). Next: plan v1.0's first rung — the Cargo workspace (napkin §15.1) and the
  real `format`, `text` and `render` crates built from these ADRs.

## Moved 2026-11-10 (rung 3 done)

From Next:
- Rung 3 waits on chris's look check of the window (`docs/bbq/2026-11-10-rung-3-first-editor.md`, Kickoff): try
  `cargo run --release -p quadrule-app -- [file.quad]` (add `--canvas chip` for the chip canvas). Taste calls
  made for it are listed in `docs/STATUS-log.md` (2026-11-10, rung 3 built). Then rung 4: dragging and snapping.

## Moved 2026-11-16 from STATUS.md, Log (word for word)

- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).
- 2026-11-09: rung 1 built and merged (PR #1), CI green on three OSes (`docs/STATUS-log.md`).
- 2026-11-09: LICENSE and headers; calibration sheets; repo public; safe zones measured; stock grid matched;
  linter started; fuzzing (`docs/STATUS-log.md`).
- 2026-11-09: rung 2 planned (bbq, 12 questions) and built: the viewer window; spike R15 paid (`docs/STATUS-log.md`).

## Moved 2026-11-17 from STATUS.md, Log (word for word)

- 2026-11-10: rung 3 planned (bbq, 12 questions): the first editor (`docs/STATUS-log.md`).
- 2026-11-10: rung 3 built (steps 1–9), chip canvas measured on the Pi; layers moved right; look approved
  (`docs/STATUS-log.md`).
- 2026-11-10: rung 4 planned (bbq, 12 questions): dragging and snapping, split 4a/4b (`docs/STATUS-log.md`).
- 2026-11-10: rung 4a built (Sonnet 5.5, high): drag, handles, marquee, layer drag, snapping, overlay; measured on the Pi;
  look approved with two safe-zone changes, built (`docs/STATUS-log.md`).
- 2026-11-10: rung 4b built (Sonnet 5.5, high): point snap, smart guides, equal spacing, copy and paste; measured on the
  Pi; waits for chris's look (`docs/STATUS-log.md`).
- 2026-11-10: Makefile for testing and review (bbq, 4 questions), built (`docs/STATUS-log.md`).
- 2026-11-10: README.md and DEVELOPMENT.md written (bbq, 12 questions); GitHub description blocked by a 403 (`docs/STATUS-log.md`).
- 2026-11-10: rung 4b approved, rung 4 done; rung 5 planned (bbq, 12 questions): New and Insert (`docs/STATUS-log.md`).
- 2026-11-10: ADR backlog sweep: ADR-004 to ADR-014 drafted and all ratified by chris; drag speed now the 90th
  percentile (`docs/STATUS-log.md`, `docs/decisions/INDEX.md`).
- 2026-11-10: rung 5 built (Sonnet 5.5, high): New, the tools bar, drawing; measured on the Pi; waits for chris's look
  (`docs/STATUS-log.md`).
- 2026-11-10: rung 5 approved by chris; L06 stops checking the floating toolbar (ADR-012 amended) (`docs/STATUS-log.md`).
- 2026-11-10: arc v1 planned (Arc bbq, 20 questions): rungs 6–18 to v1.0 (`docs/arcs/v1/README.md`, `docs/STATUS-log.md`).
- 2026-11-10: rungs 1–5 tagged at their approvals, `v0.1.0` to `v0.5.0` (4b is `v0.4.1`) (`docs/STATUS-log.md`).
- 2026-11-10: v0.5.0 released on GitHub; release packages workflow built, dry run green on four systems (`docs/STATUS-log.md`).
- 2026-11-10: v0.5.0 packages published; pixel grid now from 16× zoom, not 8× (chris) (`docs/STATUS-log.md`).
- 2026-11-10: rung 6 kickoff: budget 15 points; grays calibration sheet added; rung 6 planned (bbq, 8 questions), split
  6a/6b (`docs/STATUS-log.md`, `docs/bbq/2026-11-10-rung-6-shapes.md`).
- 2026-11-10: rung 6a built (Sonnet 5.5, high): shapes, fill roles, dashes, caps, format 0.5.0; measured on the Pi;
  waits for chris's look (`docs/STATUS-log.md`).
