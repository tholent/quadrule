# bbq — rung 16b: the starters

2026-11-24 · Arc · 8 questions · lead Opus 5.5 high (this session, as `docs/arcs/v1/rung-16b-PROMPT-2026-11-18.md` names);
builders Sonnet 5.5 high. The arc's rules (`docs/arcs/v1/v1-PROMPT-2026-11-10.md`) and rung 16's record
(`docs/bbq/2026-11-17-rung-16-welcome-and-gallery.md`) hold unless a rule here says otherwise.

## Asked again at session start (AGENTS.md)

- Which starters has chris made, and where? chris, word for word: "I have updated the starters and they all pass lint
  checks. Calendar starters are proving more difficult than anticipated, so let's put them on hold for now. Please check
  `~/starters-draft` for the updated quad files. Check them out and ensure they conform and then kick off rung 16b".
  Seven files, copied byte for byte to `docs/arcs/v1/starters-from-chris/2026-11-24/`: `bullet-journal`, `cornell`,
  `grid`, `hex`, `isometric`, `narrow_ruled`, `narrow_ruled_rtl`.
- The basic section: answered by Q5.

## The review of chris's seven (2026-11-24, before the questions)

- Build proof: `make build-proof` → `BUILD head=5c1f3cb dirty=no rebuilt=0 target_gb=38.5`.
- Lint: `target/debug/xtask lint <file>` → `0 finding(s) on 2 profile(s)` for each of the seven (ADR-021's bar).
- All seven: format 0.20.0, saved by quadrule 0.16.6, type Template, profiles `supernote.a6x2` (primary) and
  `supernote.a5x2`, each with its own rhythm, line roles only (Guide, Margin, Dot grid, Whisper).
- Gaps against spec §13: every file has one layer named "Layer 1" (§13 asks for named layers); five name their objects
  "Pattern" or "Line"; bullet-journal, cornell, grid, hex and both ruled margin lines carry hand-set `supernote.a5x2`
  position or size overrides instead of constraints; `narrow_ruled` and `narrow_ruled_rtl` use underscores; titles mix
  "Bullet Journal" with "Narrow ruled"; descriptions and tags are empty.
- Looks (drawn on both tablets with the golden tests' render path): grid on the Nomad has a sliver row of about 1 mm
  at the bottom beside the border, reading as a double line; the bullet journal's dots leave about 12 mm on the left
  and 5 mm on the right, and its header line starts left of the first dot column; cornell, grid, hex and isometric run
  their border along the page's left and right edges (lint allows it, ADR-019).
- Two UI tests assume no starter ships (`crates/ui/tests/gallery.rs:304`, `crates/ui/tests/welcome.rs:185`): copying
  the files into `crates/ui/starters/` fails them (`cargo nextest run -p quadrule-ui --no-fail-fast` → 2 failed), so
  the originals wait in `docs/` and the builder updates those tests as the starters land.

## Budget

Share: 20 points (1 point = 1 % of the weekly limit) of a 20-point ceiling, alone on the meter; reserve 0.
Room: 20 points (the lower of the ceiling and the hard stop less the reading, less the reserve).
Written when the Saturday meter read 27 % (2026-11-24, 10:34 UTC, chris's reading; logged with `spend-ledger reading`).
Reset Saturday 2026-11-28, 5 pm EST (the board's, not given today), more than 48 h away. Stop starting new work at the
first of: the meter reaching 47 % (alone on the meter: the reading less 27); the hard stop (75 % beyond 48 h, 80 % at
24–48 h, 85 % at 12–24 h, 93 % under 12 h); about 500k of the lead's context. In-flight work finishes. Before
handoff, write the row in `docs/budget-calibration.md`.

## Rules

- **Q1 Planning.** chris: "Non-calendar ones now". Builders make the daily planner and the to-do list in 16b; the weekly
  spreads (vertical and horizontal) and the monthly calendar wait with the calendars.
- **Q2 Fixes to chris's seven.** chris: "Builder all, you look". The builder makes every fix from the review, the names,
  files, titles, descriptions and tags and the looks (the grid's sliver row, the bullet journal's spacing and header
  line), and chris judges them at the look.
- **Q3 Per-device overrides.** chris: "Constraints now". The hand-set `supernote.a5x2` positions and sizes become
  constraints, so each starter adapts to any tablet (v1.1 adds more); any visible change shows at the look.
- **Q4 The rest.** chris: "Builder, from yours". A Sonnet 5.5 high builder makes the rest of each category following
  chris's examples; chris judges them all at one look.
- **Q5 The basic section.** chris: "Switch per element". Each object, and the page's margins, rhythm and title, gets a
  "Show in Basic" switch; a **Basic** tab, first in the inspector, lists what is marked with its main settings; every
  setting stays in Style and the other tabs. It changes the file format, so it lands behind a flag until chris approves
  its look (ADR-007, amendment of 2026-11-10).
- **Q6 The bullet-journal draft.** chris: "Rapid-log page". It is the Bullet journal category's Rapid-log page; builders
  make the habit tracker and the index from it.
- **Q7 Order.** chris: "One release". The basic section is built first, then every starter is saved with its marks; one
  look, one release, `v0.16.7`.
- **Q8 Anything else.** chris: "Nothing more". The standing rules hold: the frozen golden lines and fixtures stay as they
  are; each new starter adds goldens on both tablets; drawing and dragging on the heaviest starter stays under 50 ms at
  the 90th percentile (ADR-009). *Correction after the answer: the option said "the 32 frozen golden lines and fixtures
  v0.4 to v0.16"; the repository holds 40 golden lines (`wc -l crates/render/tests/golden/SHA256SUMS` → 40) and
  fixtures `v0.4` to `v0.19` (`ls crates/render/tests/fixtures`). All 40 and every fixture folder stay frozen.*

## Owed

None.

## Defaulted (the lead's calls; each can be vetoed)

1. **The list, 20 starters.** chris's seven, plus thirteen by builders. Paper: college ruled, wide ruled, dot grid,
   blank with margins. Notes: meeting, lecture. Planning: daily planner, to-do list. Bullet journal: habit tracker,
   index. Creative: storyboard, music staff, handwriting practice (no example from chris; they follow Paper's
   conventions). `veto: name any to add, drop or hold.`
2. **The future log waits with the calendars**: it is laid out by month, as the calendars are. `veto: build it now.`
3. **The right-to-left ruled page ships in Paper** as "Narrow ruled, right-to-left" (not in §13's list; chris made it).
   `veto: drop it or rename it.`
4. **Names.** File names take hyphens (`narrow-ruled`, `narrow-ruled-rtl`, `rapid-log`); titles take sentence case
   ("Rapid log", "Square grid"); layers and objects are named for what they are ("Lines", "Margin", "Dots",
   "Header"). Words are the builder's proposals for chris's look. `veto: any name.`
5. **How starters are made.** One generator, `cargo xtask starters`, writes every starter from code (chris's seven
   reproduced from his files' values, with the fixes), and a test fails when a checked-in `.quad` differs from what the
   generator writes, so a format change re-makes them all and none drifts. `veto: hand-edited files instead.`
6. **The basic section's format.** File format `0.21.0`: an optional "show in Basic" mark on objects and on the page's
   margins, rhythm and title; the reader always understands it, and the flag hides only the Basic tab and the
   switches, so a starter saved with marks opens the same with the flag off. New fixtures `v0.20/` and `v0.21/`. The
   builder drafts it as an ADR (next free number) for chris to ratify at the look. `veto: no ADR, or a different shape.`
7. **What each starter exposes**: its spacing (the rhythm), its page margins and its title, plus the one or two
   settings that make it that page (the Cornell cue and summary sizes, the grid's major-line interval). The builder
   proposes; chris judges at the look. `veto: a fixed list.`
8. **Goldens and the lint test.** Each starter adds two golden lines (Nomad and Manta), looked at by the lead before
   blessing (rung 8 lesson); a test lints every starter on both tablets and finds nothing (ADR-021 item 1, owed until
   now). `veto: none expected.`
9. **Legs.** Leg 1: the basic section (flag, format 0.21.0, fixtures, ADR draft). Leg 2: chris's seven, fixed, with
   constraints and marks, through the generator, plus goldens, the lint test and the two gallery tests updated.
   Leg 3: the thirteen new starters. One builder per leg, one after another; the lead checks each leg's diff and
   pictures before the next. Then the speed run on the heaviest starter, then chris's look (`--look welcome`).
   `veto: another order.`

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything.
This session leads 16b itself (the 16b prompt names it). Read this record, `docs/arcs/v1/rung-16b-PROMPT-2026-11-18.md`
and spec §13. Prove the build matches HEAD. Take the meter reading. Dispatch the legs (Defaulted 9) to Sonnet 5.5 high
builders, committing to `main` as each step lands, and stop for chris's look. After approval: `0.16.7`, release notes in
`docs/arcs/v1/release-notes/`, the tag (ask first), the board's re-plan, the next lead's prompt (rung 17), `STATUS.md`,
the log, the debt and the calibration row.

## Amendments

(Rulings made after the bbq, at the look or later, go here word for word, dated.)

- **2026-11-24, the 16b look** (meter 34 % at chris's last reading): asked (1) the look, (2) the seven points (Manta
  overrides replaced by constraints, the square grid's edge strips, the form pages' 7.28 mm step and margins, the
  landscape habit tracker, the light Label gray, keeping `--basic-section` until the look, the Basic tab showing on
  deselect), (3) ratifying ADR-026. chris, word for word: "1. looks good 2. all good, for now 3. ratified". Rung 16b
  is approved as built; the seven stand as built ("for now": each stays open in `docs/STATUS-debt.md`); ADR-026 is
  ratified; the `--basic-section` flag comes out (ADR-007; ADR-026's owed line). Ships as `v0.16.7`.
