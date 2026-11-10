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
