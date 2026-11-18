# Next lead's prompt — quadrule rung 16b: the starters

2026-11-18 · written by the lead that finished rung 16a (Opus 5.5) at chris's approval, as the board asks
(`docs/arcs/v1/README.md`). The arc's rules (`docs/arcs/v1/v1-PROMPT-2026-11-10.md`, Q1–Q20) and rung 16's bbq record
(`docs/bbq/2026-11-17-rung-16-welcome-and-gallery.md`, Q1–Q9 and its Amendments) hold unchanged; this file only hands
over. **Start 16b only when chris says his starters are ready** (Q1, arc Q13).

## Where rung 16a left things

- Rung 16a is done and shipped as **`v0.16.0`** (approved by chris 2026-11-18: "Approved. Ratify adr-013."). ADR-013's
  rung 16 amendment is ratified (welcome screen, File menu, one Export…).
- **The gallery is ready for starters.** A `.quad` placed in `crates/ui/starters/<category>/<slug>.quad` is built into
  the app by `crates/ui/build.rs`; the folder names the category (`paper`, `notes`, `planning`, `bullet-journal`,
  `creative`, as `crates/ui/src/starters.rs` names them). No starter ships yet; empty categories say
  "coming soon" (Q5). Picking a starter opens an untitled copy on the filter's device.
- **How chris's starters reach the repository:** chris designs one per category in the app and keeps it with File ▸
  Save as template…, which writes `~/.local/share/quadrule/templates/<slug>.quad` on Linux. Ask chris where his five
  are, copy them into `crates/ui/starters/<category>/`, and read each (layers, roles, constraints, rhythm) before
  agents follow it for the rest of that category (arc Q13).
- **Frozen export bytes are still 30 golden lines** in `crates/render/tests/golden/SHA256SUMS`, and fixtures `v0.4/` to
  `v0.15/`; 16a moved none. *(2026-11-18, after this prompt: pattern region borders shipped as `v0.16.1`, adding two
  golden lines (32 now, all frozen), file format 0.17.0 and fixture `v0.16/`; ADR-014 amended. So 16b ships as
  `v0.16.2`; record `docs/bbq/2026-11-18-pattern-borders.md`.)* Each starter adds its own goldens on **both devices** (arc Q11, Q13); look at every new
  golden picture before blessing it (rung 8 lesson).
- **Test starters** (24 pages made from the sample documents, `crates/ui/src/test_starters.rs`) stay for the speed run
  and the tests; `--gallery-test-starters` shows them.
- **Speed:** `--bench-welcome 150` times the welcome screen while every tile draws (16a: p90 1.5–1.8 ms). The
  Makefile's `LINT` bug is fixed (`LINT :=`): before `c1d654b`, every `make bench-*` since rung 14a (`547c39d`) timed
  the lint page whatever page it named, so rung 14's and 15's `make` numbers are for the lint page.

## What 16b holds

- **The starters** of spec §13 (about 24): Paper (ruled narrow, college, wide; dot grid; square grid; isometric; hex;
  blank with margins), Notes (Cornell, meeting, lecture), Planning (daily, weekly vertical and horizontal, monthly
  calendar, to-do), Bullet journal (rapid log, habit tracker, future log, index), Creative (storyboard, music staff,
  handwriting practice). chris makes one per category; agents build the rest following his.
- **Each starter**: named layers, line roles, constraints and its own rhythm; lint-clean with no rhythm warnings on
  both devices (spec §13); golden-tested on both devices (arc Q11, Q13).
- **The basic section** (Q7): each starter's top-level settings (spacing, margins, title) at the top of the inspector,
  so quick-start users rarely need more. Ask chris how a starter names which settings show.
- **Speed** (arc Q18, the release bar): drawing and dragging on the heaviest starter, Pi, release build, both canvases,
  strip on, 90th percentile under 50 ms (ADR-009). Say before any crosses it.

## Watch items

- **Budget:** chris lifted this week's hard limits (rung 16 record, Amendments: "for this week only, unset any hard
  limits"); that ends at the weekly reset, Saturday 2026-11-21, 5 pm EST. The meter read 65 % at 16a's approval. After
  the reset the usual stops return; run `budget-select` at kickoff.
- **Debt in code 16b touches** (`docs/STATUS-debt.md`): "Rung 16a, owed" and "Rung 16a leg 2, owed" (the thumbnail
  cache, failed tiles not retried, installed fonts in thumbnails).

## Open questions to ask again at the start (AGENTS.md)

- Which starters has chris made, and where are they?
- The basic section: how does a starter mark which of its settings show?
- Still parked: the Label gray (v1.1).

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(`echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board, this file, the rung 16 record with its Amendments, spec §13,
`crates/ui/src/starters.rs`, `crates/ui/src/welcome.rs`, and `docs/STATUS-debt.md`. Prove the build matches HEAD
(`make build-proof`). Take the meter reading (or ask chris) and set the budget with `budget-select`. Ask the open
questions above. Run a short bbq (about 8 questions) for 16b: the starters' list and names, who builds which, the basic
section, each starter's rhythm and margins, goldens, and anything chris's examples leave open. Have Sonnet 5.5 at high
effort build on `main`, committing as each step lands; keep the 30 golden lines and the fixtures frozen. Show chris
the gallery with the starters (`--look welcome`) and stop for his look. After approval: `0.16.2`, release notes in
`docs/arcs/v1/release-notes/`, the tag, the board's re-plan, the next lead's prompt (rung 17), `STATUS.md`, the log,
the debt and the calibration row.
