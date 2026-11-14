# Arc v1 — the board

The arc from rung 5 (done) to quadrule **v1.0** as `napkins/quadrule-spec.md` §16 defines it, in full (Q3). Planned
2026-11-10 in a 20-question Arc bbq; its record and the first lead's prompt are
`docs/arcs/v1/v1-PROMPT-2026-11-10.md`. Rulings are cited by question number (Q1…Q20) from that record.

Every lead reads this board first and updates it at the end of its rung: the rung's row, chris's queue, and the
re-plan below (Q15: the board is re-planned after **every** rung).

## The rungs

Model-shaping rungs come first, so features built later need no rework (Q5). Each rung gets a short bbq of about 8
questions before building (Q6), its own budget from `budget-select` at kickoff (Q9), and chris's look before it is
done (ADR-007). A rung bigger than one session splits into 6a, 6b… with a look each (Q10). When chris approves a rung,
it becomes version `0.N.0` with a pushed git tag `v0.N.0` (Q19); each later part of a split rung bumps the last
number (6a `v0.6.0`, 6b `v0.6.1`), as rungs 1 to 5 were tagged (chris, 2026-11-10). Rung 8's two parts shipped as
one `v0.8.0` (chris, 2026-11-13: "let's bundle this into a single v0.8.0 release"). Each approved rung or part also gets a
GitHub release on its tag, marked pre-release until v1.0, with notes on what the rung added (chris, 2026-11-10:
"please make github releases for v0.5.0 and for any approved rung (release) moving forward").

| Rung | What it holds | Size | State |
|---|---|---|---|
| 6 | Shapes: rectangle (corner radius), ellipse, polygon and star; fill and stroke; line dashes and caps; lines that point any way; rotation (handle and field); opacity. Split (bbq Q1, `docs/bbq/2026-11-10-rung-6-shapes.md`): **6a** all the shapes, fill (Shade and Block roles), dashes and caps; **6b** rotation, lines any way, opacity | large: 6a + 6b | done: 6a `v0.6.0`, 6b `v0.6.1`, both approved by chris 2026-11-11 |
| 7 | Groups (nested, double-click to enter, group constraints); align and distribute (§6.3); how a group turns (6b debt) | medium | done: 7a `v0.7.0`, 7b `v0.7.1`, both approved by chris 2026-11-12 (7a with the Figma constraints change) |
| 8 | Several devices: secondary profiles, the device previews strip, per-object overrides with badges and an Overrides tab, lint L15; portrait and landscape orientation (a landscape design exports turned into portrait pixels: chris's Nomad check, 2026-11-12, below) | large: 8a + 8b | done: `v0.8.0` (8a and 8b in one release), approved by chris 2026-11-13 |
| 9 | Pattern library: isometric, hex, Cornell, music staff; "every Nth line is major"; unbinding a region from the rhythm; isometric rhythm; major every N; lint L17 | large | done: `v0.9.0`, approved by chris 2026-11-13 (with the isometric turned 90°, hex edges cut at the frame, bigger major dots) |
| 10 | Repeat tool: linear and grid, pitch or gap, fill to bounds, alternation, Expand (§6.4) | large | done: `v0.10.0`, approved by chris 2026-11-13 (with the chip-canvas speed fix he asked for before the look) |
| 11 | Text and fonts: the bundled set with its OFL texts, system fonts on all three systems, the font picker, fixed-size boxes and L08, paragraph spacing, vertical alignment, baselines on the rhythm; monochrome emoji and its picker | large | next: `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md` |
| 12 | The missing-font flow (§7.3: saved outlines, substitute, locate), L11, the system-font lint (ADR-002), embedded fonts in `assets/fonts/` | medium | — |
| 13 | Profile system: user profiles in the config folder, the profile editor, local override and Reset to built-in, `.ron` import and export | medium | — |
| 14 | Lint and preflight: L03, L05, L07, L09, L13, L14 from measured values; quick fixes; Ignore for this document; Free placement; filter by device; the preflight sheet with Export anyway; Style tab gray swatches, "matches Guide", "Use recommended" (§8.2). L04 is v1.1 (Q7) | medium–large | — |
| 15 | Export: the naming template and its tokens, Export all, per-device subfolders, Export as set, a remembered folder, the completion sheet with the MyStyle note, the PNG text note, `.quad` thumbnails in `previews/` | medium | — |
| 16 | Welcome screen and the starter gallery (§13, about 24 starters): device filter, search, thumbnails, My templates; each starter lint-clean and golden-tested on both devices | large | — |
| 17 | Polish and debt sweep: UI scaling 75–200 %, keyboard shortcuts, an AccessKit check, readable JSON export; the debt the rungs left (Q17) | small–medium | — |
| 18 | Release: installers built in CI (unsigned `.dmg`, MSI, AppImage), `1.0.0-rc.1` on GitHub for testers, blind tastetest, then `1.0.0` (Q12, Q14, Q18) | medium | — |

Five rungs landed in two days of sessions; at that pace this is roughly two to four weeks of sessions, plus chris's
looks and measurements. This estimate is a guess; each re-plan corrects it.

## chris's queue

What waits on chris's hands or taste. The lead adds to it and crosses items off with the date.

- **Hardware, early (Q8), alongside rungs 6–7:**
  - ~~Clean gray levels on both devices~~ done 2026-11-12: the Nomad read, the Manta ruled the same; Block 68 → 102
    (`docs/device-measurements/supernote.a6x2/calibration.md`). (Needed by L03, L05, L14, the gray swatches and rung
    6's Shade and Block fill roles.) The grays sheet was added 2026-11-10 (`dcf5458`): run `cargo xtask calibration-sheet`, put each device's
    `quadrule-calib_grays-*.png` on it, and report which of G1–G30 are clean and how many bands GR shows
    (`docs/calibration.md`).
  - ~~Landscape toolbar screenshots~~ captured 2026-11-12, left, right and top on both devices; rung 8 ruled them
    records only, not measured, since a landscape template is read in portrait mode where the portrait zones apply,
    turned (`docs/device-measurements/<device>/landscape/`, rung 8 record, Defaulted "Landscape safe zones").
  - ~~Landscape: put one landscape PNG on a device and say whether it expects rotated pixels or swapped sizes~~
    answered on the Nomad 2026-11-12: **rotated pixels** (a landscape design turned and saved as a portrait PNG; a wide
    PNG is shrunk, and landscape mode zooms by 4/3) (`docs/device-measurements/supernote.a6x2/landscape-test.md`,
    spec §17.1 item 3 note). The Manta ruled the same, not tested (chris, 2026-11-12).
  - Not needed: the Manta's pen shades. chris ruled them equal to the Nomad's (Q8).
- **Starters (Q11, Q13):** chris designs one starter per category in the app as the features land; agents follow
  his example for the rest of that category. The lead says when each becomes buildable:
  - Paper (ruled, dot, grid, isometric, hex, blank): **buildable now** (rung 9 done, 2026-11-13).
  - Notes (Cornell, meeting, lecture): **buildable now** (rungs 7 and 9 done).
  - Planning (daily, weekly, monthly, to-do): **buildable now** (rung 10 done, 2026-11-13).
  - Bullet journal (rapid log, habit tracker, future log, index): **buildable now** (rung 10 done, 2026-11-13).
  - Creative (storyboard, music staff, handwriting practice): **buildable now** (rung 9 done).
- **Looks:** one per rung or part (ADR-007).

## Standing rules for every rung

- Existing pages' export bytes stay frozen: rung 1's goldens and the stock-grid test pass unchanged. New object
  kinds add new goldens; if any change would alter an existing page's exported pixels, stop and ask chris (Q20).
- Each rung pays the debt in the code it already changes; rung 17 sweeps the rest (Q17).
- Drawing and dragging stay under 50 ms at the 90th percentile on the Pi (ADR-009), measured each rung.
- Commit straight to `main` (`AGENTS.md`, amendment of 2026-11-09).

## The release bar (Q18)

v1.0 ships only when all of these hold: every rung look-approved; every starter lint-clean and golden-tested on
both devices; drawing and dragging under 50 ms at the 90th percentile on the Pi on the heaviest starter; a blind
tastetest says READY; the release candidate has had testers on macOS and Windows (Q14).

## Out of this arc

L04 (v1.1, Q7). Clicking the overrides dot to open the Overrides tab (v1.1, chris 2026-11-13). The A5 X rhythm question (parked until the A5 X profile, v1.1, Q16). Winget and Flathub listings
(after v1.0, Q12). Everything the spec puts in v1.1 or later.

## Re-plans

After each rung's look, the lead writes one dated line here: what the rung taught, and any change to the rungs
above (which chris approves before it counts).

- 2026-11-10: arc planned (this board).
- 2026-11-10: rungs 1 to 5 tagged at their approval commits: `v0.1.0` (`00ad0e2`), `v0.2.0` (`52c7ee9`), `v0.3.0`
  (`50bd8db`), `v0.4.0` 4a (`c176a2f`), `v0.4.1` 4b (`6ced5c2`), `v0.5.0` (`9bb8122`). The workspace `Cargo.toml`
  still says 0.1.0 at all of them; from rung 6 the bump goes in the approval commit.
- 2026-11-10: releases ruled for v0.5.0 and every approved rung after it. v0.5.0's release was first blocked (HTTP 403);
  on a retry the token worked (what changed is not recorded) and it was published: https://github.com/tholent/quadrule/releases/tag/v0.5.0. Each
  rung's notes go in `docs/arcs/v1/release-notes/`.
- 2026-11-11, after rung 6: rung 6 took one lead session and two builder subagents (about 0.76M builder tokens,
  under 2 points of a 15-point share), not the "about 3 sessions" its record guessed, because the bbq record was
  detailed enough to brief from directly. Lessons: (1) the graphics-chip canvas reaches 45.9–48.4 ms on the shapes
  page against the 50 ms line, so rungs 9, 10 and 16 (heavier pages) must be measured on their heaviest page and say
  before they cross it; (2) GitHub releases need chris's hand while this session's permission check refuses `gh
  release create`; (3) rotation in groups is open and lands in rung 7's bbq. No change to the rungs is proposed; the
  rung 7 row now names the group-turn question. Next lead's prompt: `docs/arcs/v1/rung-7-PROMPT-2026-11-11.md`.
- 2026-11-12, after rung 7: rung 7 took one lead session and three builder subagents (7a 0.49M tokens, its look change
  0.21M, 7b 0.21M) and moved the weekly meter 9 points (55 → 64 %, chris's readings), almost all of its 10-point share;
  the fitted K so far is 12.2M tokens per point over two pairs (`docs/budget-calibration.md`). Lessons: (1) a ruled
  answer can turn at the look: chris changed Q3 to Figma's groups (see-through to constraints), so the spec's
  "constraint container" (§5.2) no longer holds and rung 8's per-device work builds on the Figma rule; (2) the chip
  canvas sits at 40–42 ms on the groups page against 50 ms, the heaviest page yet after the shapes page; (3) at
  rung 7's rate, rung 8 (large) is likely 15–20 points, more than this week has left before its hard stop, so it
  should start after the Saturday reset or split 8a/8b. No change to the rungs is proposed. Next lead's prompt:
  `docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`.
- 2026-11-13, after rung 8: rung 8 took two lead sessions and one builder subagent (8a, 0.48M tokens) and moved the
  weekly meter 7 points (67 → 74 %, chris's readings) for a 20-point share. 8b was started after 8a and left
  uncommitted, with no log entry, when that session stopped; this session found it, added its tests, goldens and docs,
  and judged it. Lessons: (1) commit work in progress, or at least log it, before a session ends, so the next one can
  tell finished from half-done; (2) look at new golden images before blessing them: the first landscape bless showed
  the Nomad page's star scaled into the header band (the sample, not the code); (3) chris may bundle a split rung into
  one release at the look, as he did here; (4) the chip canvas holds on the landscape page (29 ms drag, 38 ms draw),
  but drawing a pattern region on the shapes page is 51 ms, left for later by chris, and rung 9 adds heavier patterns
  (isometric, hex), so rung 9 must time pattern drawing on its heaviest pattern page, both canvases. No change to the
  rungs is proposed. Next lead's prompt: `docs/arcs/v1/rung-9-PROMPT-2026-11-13.md`.
- 2026-11-13, after rung 9: rung 9 took one lead session and one builder subagent (Sonnet 5.5 high; 0.42M tokens for
  the build, 0.48M reported at the end of its look-change leg) and was planned, built, looked at, changed and approved
  in one day as one rung (chris overruled the 9a/9b split). Lessons: (1) a picture beats a word for geometry: chris
  answered "horizontal rows" for isometric in the bbq and, seeing it, wanted vertical lines; for a shape question, show
  a sketch or a rendered sample in the question; (2) a Defaulted rule that names a value from another table (major dots
  "at the major role's width") needs a check that the value makes sense there (Structure's 2 px was smaller than the
  3 px dot); (3) the chip canvas's per-rectangle cost was the 8a debt's cause, and merging same-gray rectangles into one
  path took it from 51 to 41 ms, which leaves room for rung 10's repeats. The Paper, Notes and Creative starters are now
  buildable (chris designs one per category). No change to the rungs is proposed. Next lead's prompt:
  `docs/arcs/v1/rung-10-PROMPT-2026-11-13.md`.
- 2026-11-13, after rung 10: rung 10 took one lead session and one Sonnet 5.5 high builder, resumed once for a speed fix
  (the harness reported 470,695 builder tokens at the build's end and 576,818 at the fix's end; whether the second
  includes the first is not shown), and was planned, built, fixed and approved in one day as one rung. Lessons: (1) the
  chip canvas's cost on the Pi follows how much outline it draws, not how many objects: merging objects did nothing,
  while keeping the unchanged part of the page in its own texture took drawing on the repeat page from 49.7 to 23.6 ms
  and dragging on the pattern page from 33.9 to 17.3 ms, which leaves room for the heavier starters of rung 16; (2) time
  each frame's parts before choosing a fix (the lead's guess cost a build and a revert); (3) a bbq whose eight answers all
  took the recommendation, with a long Defaulted list, needed no change at the look. The Planning and Bullet journal
  starters are now buildable. No change to the rungs is proposed. Next lead's prompt:
  `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md`.
