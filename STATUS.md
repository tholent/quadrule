# STATUS — quadrule

_Last updated: 2026-11-09_

The project's live state. Update it when work lands. Every claim cites at least one file that exists.

## Now

- Project prepared for agents with project-prep: the rules are in `AGENTS.md`, the split files in `docs/`.
- Design spec is a draft (v0.1); no code yet (`napkins/quadrule-spec.md`).

## Next

- Put the first decision on record: how this project is set up, and its decision format (`docs/decisions/README.md`).

## Known debt

Full debt list: `docs/STATUS-debt.md`.
- The first decision is not on record yet (`docs/STATUS-debt.md`).

## Log

Session history: `docs/STATUS-log.md`. Finished work: `docs/STATUS-archive.md`.
- 2026-11-09: prepared with project-prep (`docs/STATUS-log.md`).

## How this file is kept

Live state stays here: what works now, what is next, and the top of the debt list. Detail goes in the split files:
debt in `docs/STATUS-debt.md`, each session in `docs/STATUS-log.md` (dated, newest at the bottom; split by
month when it grows), and finished entries, moved word for word, in `docs/STATUS-archive.md`. Every bullet under Now, Next
and Known debt cites at least one file that exists. Keep this file under 300 lines; the project-prep
skill's `scripts/status-check.py` checks both.
