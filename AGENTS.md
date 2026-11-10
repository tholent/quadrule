# AGENTS.md — quadrule

Read this first, every session. Every agent reads this file; `CLAUDE.md` points here.

## What this project is
A free, open-source vector design tool (Rust, egui, wgpu) for making page templates and wallpapers for Supernote e-ink tablets. The design spec is `napkins/quadrule-spec.md`.

## Doctrine
None yet. The owner sets one; agents do not invent it.

## Where things live
- `STATUS.md` — where we are right now. Short. Update it at the end of each session.
- `docs/STATUS-log.md` — a dated, append-only record of what each session did.
- `docs/STATUS-debt.md` — known debt and owed work.
- `docs/STATUS-archive.md` — finished entries moved out of `STATUS.md`, word for word.
- `docs/decisions/` — decision records (ADRs). Its `README.md` names the format once one is chosen.
- `docs/rules/task-homes.md` — each scheduled task and its one home account.
- `Makefile` — shortcuts for building, testing and review; `make` lists them, `make check-all` is what CI runs.
- `napkins/` — rough design specs (the spec is `napkins/quadrule-spec.md`).

## Ground rules
- The repository is the only shared memory between sessions, agents and accounts.
- Taste, naming and direction are chris's to rule. Agents propose; chris ratifies.
- Decisions: never edit a ruled decision's text. Add a dated amendment, or supersede it with a new one. Follow the
  active decision; superseded ones are history only.
- Work on a branch. Never push straight to the main branch without chris's approval.
  - Amended 2026-11-09 by chris, word for word: "until specifically necessary, let's just work on the main branch
    directly. This is pre-alpha software at this point." So agents commit and push to `main` directly until chris
    says otherwise, or a job specifically needs a branch (say so when one does).
- Keep `docs/` tidy: no loose dated files at its top level.

<!-- pitmaster rules: plan, run, check, plain, craft · catalogue 2026-10-23 · project-prep rewrites everything down to the end line -->
## Rules

### Plan and decide
- Say how big a job is before starting it; if it is weeks of sessions, say so.
- Write each ruling down word for word when it is made, and ask open questions again at the next session start.
- Never delete advice you withdrew; keep it, mark it withdrawn, and say why.
- Weigh each option by its lifetime cost (the gaps it leaves and the rework it causes), not today's; when the bigger option closes more gaps, recommend it.
- Say up front which options combine and which conflict; when the owner picks "1 + 3", build the merge and say what was kept from each.
- Price each option against the project's doctrine (its rule for choosing), if it has one.

### Run and delegate
- For a delegated landing, show every precondition met.
- Before an action, say whether it can be undone and how; if it cannot, say it is one-way.
- Say so out loud the moment the work grows past what was asked.
- When a worker refuses or fails a brief, check the brief before blaming the worker.
- Start long jobs detached, and judge them by their log, never by what the launcher returned.
- Give every scheduled task one home account, written in `docs/rules/task-homes.md`; check that list before making or moving one.

### Check and review
- Report a number as the command that produces it; for "not found", name every file searched.
- Test what this session can actually do (build, run, push, reach the network) instead of assuming; date the result.

### Plain words
- When asked about a term, explain it the way this project uses it first; give a general meaning only if the project does not use it, and say so.
- Write for a reader who is not a programmer: plain words, a short bracket for any hard word, never "simply" or "obviously".
- Offer once to file a new term in the glossary, and write it only on a yes.
- While explaining a term, never run a command that changes anything; show it instead, or ask first and run only a look-only one.

### Engineering craft
- At the start of every session, prove the build on disk matches HEAD.
- Make tests read one structured line, not prose.
- When only comments change, check the built files are byte-identical.
- Turn a future danger into a build error wherever you can.
- Land a new feature switched off behind a flag, tested, until the owner approves how it looks.
- Name milestones as numbered rungs; reach each in the emulator before real hardware.
- Take hardware values and protocol rules from the datasheet or spec section, never from memory.
<!-- pitmaster rules: end -->
