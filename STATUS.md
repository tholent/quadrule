# STATUS — quadrule

_Last updated: 2026-11-09_

The project's live state. Update it when work lands. Every claim cites at least one file that exists.

## Now

- Project prepared for agents with project-prep: the rules are in `AGENTS.md`, the split files in `docs/`.
- Design spec is a draft (v0.1) (`napkins/quadrule-spec.md`); plan for the first work in
  `docs/bbq/2026-11-09-quadrule-spikes.md`.
- The three technical spikes are done on branch `spikes/2026-11-09` (local only): format, text and rendering, each a
  throwaway crate with findings and a proposed ADR (`spikes/format/FINDINGS.md`, `spikes/text/FINDINGS.md`,
  `spikes/render/FINDINGS.md`). Tests: format 10, text 12, render 9 passed.
- Rust 1.99.1 installed on the Pi with rustup (`spikes/format/FINDINGS.md` header).

## Next

- chris to ratify the three proposed ADRs (`spikes/*/ADR-PROPOSAL.md`); the first ratification also picks the
  decision format (`docs/decisions/README.md`). Fold talk decisions D1–D7 into the format ADR first
  (`docs/proposals/talk-2026-11-09-format-versions-and-hinting.md`).

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- The first decision is not on record yet (`docs/STATUS-debt.md`).
- Spike checks owed on other machines: macOS, Windows, x86 Linux, and the canvas window on a display
  (`docs/STATUS-debt.md`).

## Log

Session history: `docs/STATUS-log.md`. Finished work: `docs/STATUS-archive.md`.
- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
