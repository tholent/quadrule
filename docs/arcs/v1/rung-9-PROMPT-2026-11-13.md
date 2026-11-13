# Next lead's prompt — quadrule rung 9: pattern library

2026-11-13 · written by the lead that finished rung 8 (Opus 5.5) at chris's approval, as the board asks
(`docs/arcs/v1/README.md`, "Every lead … updates it at the end of its rung"). The arc's rules (record
`docs/arcs/v1/v1-PROMPT-2026-11-10.md`, Q1–Q20) hold unchanged; this file only hands over.

## Where rung 8 left things

- Rung 8 is done: 8a (several devices: overrides, the previews strip, the Overrides tab, Make primary, device ticks,
  L15; format 0.8.0) and 8b (portrait and landscape; a landscape design exports turned a quarter turn clockwise into
  the portrait PNG; the safe zones turn with it) were approved together on 2026-11-13 and **shipped as one `v0.8.0`**.
  chris, word for word: "1. let's keep the color, but put on v1.1 a way to click the override dot and have the app
  open the overrides tab, otherwise approve all and let's bundle this into a single v0.8.0 release". Record
  `docs/bbq/2026-11-12-rung-8-several-devices.md` (with its Amendments); log `docs/STATUS-log.md` (2026-11-12/13 and
  2026-11-13 entries).
- How it ran: two lead sessions and one Sonnet 5.5 high subagent (8a, 0.48M tokens); 8b's code was left uncommitted
  and unlogged when a session stopped, and the next session finished it. The weekly meter moved **7 points** (67 → 74
  %, chris's readings) for a 20-point share.
- **Frozen export bytes are now twelve golden lines** in `crates/render/tests/golden/SHA256SUMS` (`slice_*`,
  `shapes_*`, `transforms_*`, `groups_*`, `devices_*`, `landscape_*`), plus the fixtures
  `crates/render/tests/fixtures/v0.4/` to `v0.7/`. A rung 9 change that moves any of them is a stop: ask chris (arc
  Q20). Look at every new golden image before blessing it.
- **Every profile lookup that draws or lays out a page goes through `builtin_for(id, doc.page.orientation)`**
  (`crates/profiles/src/lib.rs`); plain `builtin` is for names, pen widths and grays only. New pattern code must keep
  to this, or landscape pages break.
- **Releases:** for each, commit `docs/arcs/v1/release-notes/<tag>.md` before tagging, then push the tag; the release
  workflow drafts, attaches and publishes. Never `gh release create` by hand.

## Watch items

- **Budget:** rung 9 is "large" on the board. Rung 8 cost 7 points with one builder; rung 7 cost 9 with three. Until
  Saturday 14th, 5 pm the hard stop is 90 % (chris, this week only); after the reset, back to 75 / 80 / 85 / 93 %.
- **Speed:** isometric and hex patterns draw more lines than any page so far. The chip canvas already reaches 51 ms
  drawing a pattern region on the shapes page (left for later by chris, 2026-11-13; `docs/STATUS-debt.md`, rung 8a
  line). Build a pattern page holding every new pattern, time dragging and drawing on it on the Pi (release build,
  both canvases, strip on, 90th percentile, ADR-009), and say before anything crosses 50 ms. The pattern draw may be
  the place to pay that debt.
- **Rhythm:** the spec's rhythm has a Rectangular and an **Isometric** type (§6.3, line 304) and "Major every N"
  (line 309); pattern regions bind to it by default (§5.2, line 218). Isometric rhythm touches snapping
  (`crates/layout/src/snap.rs`) and the overlay, not only drawing.
- **Debt touched by rung 9:** text boxes and pattern regions do not rotate (v1.1, chris 2026-11-12: "Not in v1"); the
  rung 8a and 8b lines in `docs/STATUS-debt.md`.

## Open questions to ask again at the start (AGENTS.md: "ask open questions again at the next session start")

- Still parked: "Scale vs pixel-locked rhythm on A5 X" (v1.1, arc Q16); the Label gray (v1.1); text and pattern
  rotation (v1.1).
- Landscape on the Manta: ruled the same as the Nomad, never tried on the device. Ask whether chris wants to try a
  `v0.8.0` landscape export on it.

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(read the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board `docs/arcs/v1/README.md`, this file,
`docs/decisions/INDEX.md`, the rung 8 record `docs/bbq/2026-11-12-rung-8-several-devices.md` and
`docs/STATUS-debt.md`. Prove the build matches HEAD (`make build-proof`). Take the meter reading (or ask chris) and set
this rung's budget with `budget-select`. Ask the open questions above. Then run a short bbq (about 8 questions) for
**rung 9: pattern library** — isometric, hex, Cornell and music staff patterns (spec §5.2 lines 209 and 218);
"every Nth line is major"; unbinding a region from the rhythm (its own spacing); the isometric rhythm type (§6.3);
lint L17 (§8.3, line 449); the file format bump this needs (0.9.0 through the migration chain, a `v0.8/` fixture); and
whether to split 9a/9b. Have Sonnet 5.5 at high effort build it on `main`; commit as each step lands, so a stopped
session leaves no uncommitted work; keep the twelve golden lines and the fixtures frozen (stop and ask chris if any
would change); add a golden page per device for the new patterns, portrait and one landscape; measure as above; pay
the debt in code the rung touches. Show chris the window and stop for his look. After his approval: bump to `0.9.0`
(or as chris rules for a split), commit the notes in `docs/arcs/v1/release-notes/`, tag (the tag push publishes),
re-plan the board, and write the next lead's prompt in `docs/arcs/v1/`. Update `STATUS.md`, `docs/STATUS-log.md` and
`docs/STATUS-debt.md` at the end of each session, and write the calibration row in `docs/budget-calibration.md`.
