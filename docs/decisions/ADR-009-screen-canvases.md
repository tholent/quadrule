---
status: proposed
status-note: "drafted 2026-11-10 from the project's records (backlog sweep, no rounds, chris's choice); awaiting ratification"
owner: chris
kind: architecture
built: partial
proof: "`crates/ui/tests/canvas.rs::slow_chip_frames_fall_back`, `crates/ui/tests/canvas.rs::drag_frames_are_judged_by_the_95th_percentile`, `crates/ui/tests/window.rs::chip_canvas_switch_and_fallback`, `crates/ui/tests/viewer.rs::shown_pixels_are_the_export`"
owed: "the graphics-chip canvas measured zoomed in, while scrolling, and on a desktop graphics card; drag speed on a text-heavy page"
supersedes: nothing
amends: nothing
consulted: Claude (Opus 5.5) proposed the 100 ms fallback line in the rung 3 bbq (Defaulted, "veto - the 100 ms line"); the rung 4a build agent (Sonnet 5.5) chose the 95 in the percentile
---
# ADR-009 — The window has two canvases, export pixels by default and the graphics chip as a switch that falls back to the processor, judged by the 95th percentile of frames against 50 ms on the Pi

## Decision

1. **Export-pixels canvas (the default):** the window shows the vello_cpu export render, the PNG's own pixels, so
   what you see is what the tablet gets.
2. **Graphics-chip canvas (View → Canvas):** vello on wgpu. It falls back to vello_cpu when the chip cannot start or
   a frame takes over 100 ms.
3. **Speed line:** while dragging on the Pi in a release build, the 95th percentile of frames (after the first) is
   under 50 ms (20 frames a second), on both canvases. A miss stops the work and goes to chris with the numbers.
4. Zoom: Fit, 1:1, up to 16× with square pixels and a pixel grid from 8×, and real size after a ruler check.

## Context

- Rung 2 Q3 chose export pixels only, with the chip canvas for the editing rung (`docs/bbq/2026-11-09-rung-2-viewer.md`).
- Rung 3 Q4, **overruled**: the recommendation was "keep export pixels, chip canvas only if too slow"; chris chose
  both, switchable. Q7: export pixels is the default (`docs/bbq/2026-11-10-rung-3-first-editor.md`).
- The vello_cpu fallback on weak chips: ruled yes, 2026-11-09 (`docs/STATUS-debt.md`).
- Speed: rung 4 Q12 set "smooth on the Pi"; chris, word for word: "let's use a percentile" (rung 4 record, "After
  4a"). The 95 was chosen for chris to veto. Measured: export canvas 25.0–26.2 ms, chip canvas 31.6–32.1 ms
  (`STATUS.md`).
- Rung 5 Q12: chris did not pick the speed line for drawing, so there it is measured, not a stop line.

## Alternatives

### Export pixels only (overruled, rung 3 Q4)
- Good: one renderer; nothing to drift.
- Bad: chris wanted the chip canvas built and compared now.

## Consequences

- Two paths to keep in step; a test checks the screen list matches the export at 1:1.
- The chip canvas redraws the visible area on every scroll step (`docs/STATUS-debt.md`).

## Asks left open

1. The 95 in "95th percentile" was an agent's choice: keep it?
2. The 100 ms fallback line was an agent's choice: keep it?

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
