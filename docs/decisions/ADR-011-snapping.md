---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify both, for ADR-009 and ADR-011, after answering their asks)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: design
built: partial
proof: "`crates/layout/tests/snap.rs::rhythm_catches_within_reach`, `crates/layout/tests/snap.rs::nearest_wins_and_ties_go_to_rhythm`, `crates/layout/tests/snap.rs::pixel_grid_rounds_when_nothing_caught_and_ctrl_turns_all_off`, `crates/ui/tests/drag.rs::a_move_lines_up_with_another_object`, `crates/ui/tests/drag.rs::a_move_snaps_to_equal_gaps`, `crates/ui/tests/window.rs::snap_toggles_and_subdivisions_are_named_controls`"
owed: "equal spacing for a resize; point snap for a resize's corner"
supersedes: nothing
amends: nothing
consulted: Claude (Opus 5.5) proposed the reach and the tie order in the rung 4 bbq; build agents (Sonnet 5.5) built them
---
# ADR-011 — Dragging snaps to six helpers within 6 screen points, rhythm beats any other catch within half a device pixel, and Ctrl turns snapping off

## Decision

1. **Six helpers** (napkin §6.2): the pixel grid, the rhythm (with 2–10 subdivisions a unit, file format 0.4),
   point snap (corners, midpoints, centres, a line's ends and middle), smart guides (edges and centre lines, drawn
   orange), equal spacing (equal gaps in a row or column, drawn with their size), and margins and safe-zone edges.
2. **Reach:** a snap catches within 6 screen points, whatever the zoom.
3. **Who wins:** a rhythm catch (a unit line, a subdivision or a whole-step size) beats any other kind unless that
   kind is more than half a device pixel nearer. Otherwise the nearest wins; on a tie: rhythm, then margins and safe
   zones, then point snap, then smart guides, then equal spacing. The pixel grid then puts the result on whole device
   pixels (odd-width strokes on pixel centres).
4. **What snaps:** on a move, the selection's edges and centre, as one box; on a resize, the dragged edge and the
   size in whole steps; on a line end, the end point.
5. **Switches:** each helper has its own switch in the status bar and the Page tab, all on at first, kept between
   runs as app settings (not in the `.quad` file). Holding Ctrl (Cmd on macOS) turns them all off for one drag.
6. **Rhythm overlay:** faint light-blue lines (fainter for subdivisions), off by default, never exported.

## Context

- Rung 4 Q5 (all six; recommended was five), Q9, Q10, Q11, and Defaulted "Snap distance", "What snaps", "Toggles"
  (`docs/bbq/2026-11-10-rung-4-drag-and-snap.md`).
- chris, word for word: "let rhythm win within half a pixel" (rung 4 record, "After 4b"). Reading "rhythm" as
  including subdivisions and sizes, and the half pixel as device pixels, was the agent's (`docs/STATUS-log.md`).
- chris on the overlay, word for word: "the rhythm overlay should be much lighter and perhaps light blue. Something
  that indicates a guide, and doesn't interfere with the display of actual elements."

## Consequences

- Snapping works in device pixels of the device on screen (ADR-010), so a design snapped on the Manta may sit a
  pixel off a rhythm line on the Nomad.
- Owed (`docs/STATUS-debt.md`): point snap and smart guides count a page-sized pattern region's edges; the status bar
  is full at 1280 px with seven switches.

## Asks left open

1. Answered 2026-11-10: the 6-point reach and the tie order stay (chris: "Keep 6 pt and the order").
2. Answered 2026-11-10: "rhythm" includes subdivisions and whole-step sizes, and the half pixel is a device pixel
   (chris: "Yes, as built").

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
- 2026-11-10, before ratification, in chris's review of the pending ADRs: asked to keep the 6-point reach and the tie
  order, chris answered "Keep 6 pt and the order"; asked whether "rhythm" and "half a pixel" meant what was built, chris
  answered "Yes, as built". The Asks left open now record the answers; nothing else changed.
- 2026-11-10: ratified by chris (answer "Ratify both" to "Ratify ADR-009 … and ADR-011 … with your answers above?").
  Prior header values: status `proposed`, status-note `drafted 2026-11-10 from the project's records (backlog sweep, no
  rounds, chris's choice); awaiting ratification`.
