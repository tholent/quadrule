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
number (6a `v0.6.0`, 6b `v0.6.1`), as rungs 1 to 5 were tagged (chris, 2026-11-10). Each approved rung or part also gets a
GitHub release on its tag, marked pre-release until v1.0, with notes on what the rung added (chris, 2026-11-10:
"please make github releases for v0.5.0 and for any approved rung (release) moving forward").

| Rung | What it holds | Size | State |
|---|---|---|---|
| 6 | Shapes: rectangle (corner radius), ellipse, polygon and star; fill and stroke; line dashes and caps; lines that point any way; rotation (handle and field); opacity. Split (bbq Q1, `docs/bbq/2026-11-10-rung-6-shapes.md`): **6a** all the shapes, fill (Shade and Block roles), dashes and caps; **6b** rotation, lines any way, opacity | large: 6a + 6b | 6a done, approved by chris 2026-11-11, `v0.6.0`; 6b next |
| 7 | Groups (nested, double-click to enter, group constraints); align and distribute (§6.3) | medium | — |
| 8 | Several devices: secondary profiles, the device previews strip, per-object overrides with badges and an Overrides tab, lint L15; portrait and landscape orientation (needs chris's landscape check, below) | large | — |
| 9 | Pattern library: isometric, hex, Cornell, music staff; "every Nth line is major"; unbinding a region from the rhythm; isometric rhythm; major every N; lint L17 | large | — |
| 10 | Repeat tool: linear and grid, pitch or gap, fill to bounds, alternation, Expand (§6.4) | large | — |
| 11 | Text and fonts: the bundled set with its OFL texts, system fonts on all three systems, the font picker, fixed-size boxes and L08, paragraph spacing, vertical alignment, baselines on the rhythm; monochrome emoji and its picker | large | — |
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
  - Clean gray levels on both devices (needed by L03, L05, L14, the gray swatches and rung 6's Shade and Block fill
    roles). The grays sheet was added 2026-11-10 (`dcf5458`): run `cargo xtask calibration-sheet`, put each device's
    `quadrule-calib_grays-*.png` on it, and report which of G1–G30 are clean and how many bands GR shows
    (`docs/calibration.md`).
  - Landscape: put one landscape PNG on a device and say whether it expects rotated pixels or swapped sizes
    (spec §17.1 item 3); landscape toolbar screenshots for the safe zones (needed by rung 8).
  - Not needed: the Manta's pen shades. chris ruled them equal to the Nomad's (Q8).
- **Starters (Q11, Q13):** chris designs one starter per category in the app as the features land; agents follow
  his example for the rest of that category. The lead says when each becomes buildable:
  - Paper (ruled, dot, grid, isometric, hex, blank): buildable after rung 9.
  - Notes (Cornell, meeting, lecture): after rungs 7 and 9.
  - Planning (daily, weekly, monthly, to-do): after rung 10.
  - Bullet journal (rapid log, habit tracker, future log, index): after rung 10.
  - Creative (storyboard, music staff, handwriting practice): after rung 9.
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

L04 (v1.1, Q7). The A5 X rhythm question (parked until the A5 X profile, v1.1, Q16). Winget and Flathub listings
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
