# Next lead's prompt — quadrule rung 8: several devices

2026-11-12 · written by the rung 7 lead (Opus 5.5, high effort) at the end of rung 7, as the board asks
(`docs/arcs/v1/README.md`, "Every lead … updates it at the end of its rung"). The arc's rules (record
`docs/arcs/v1/v1-PROMPT-2026-11-10.md`, Q1–Q20) hold unchanged; this file only hands over.

## Where rung 7 left things

- Rung 7 is done: 7a (groups; format 0.7.0) approved 2026-11-12 with one change, built, tagged `v0.7.0`; 7b (align and
  distribute, "mixed" Arrange fields; no format or export change) approved 2026-11-12, tagged `v0.7.1`. Record
  `docs/bbq/2026-11-11-rung-7-groups-and-align.md`; log `docs/STATUS-log.md` (2026-11-12 entries).
- **chris changed a ruled answer at the look.** Q3 said a group is a constraint container; chris ruled, word for word, "I
  approve the build, but would like to change question #1 to reflect what figma does". So groups are **see-through to
  constraints**, as in Figma: every object keeps its own constraints against the page, `%` inside a group is of the
  page, and resizing a group by hand scales what is inside (record, Q3 amendment; spec §5.2 amendment note; ADR-015, ratified by chris 2026-11-12). Rung 8's
  per-device work builds on this rule: a group's box on each device is its children's resolved outer box.
- How it ran: one lead session; three Sonnet 5.5 high subagents (7a 0.49M tokens, its look change 0.21M, 7b 0.21M), on
  `main`. The weekly meter moved **9 points** (55 → 64 %, chris's readings) for a 10-point share: more per builder token
  than rung 6 suggested. `docs/budget-calibration.md` has the row and a fitted K of 12.2M tokens per point over two
  pairs (not yet trusted: three wanted).
- **Frozen export bytes are now eight golden lines** in `crates/render/tests/golden/SHA256SUMS` (`slice_*`, `shapes_*`,
  `transforms_*`, `groups_*`), plus the fixtures `crates/render/tests/fixtures/v0.4/`, `v0.5/`, `v0.6/`. A rung 8 change
  that moves any of them is a stop: ask chris (arc Q20).
- **Releases work end to end:** the `v0.7.0` tag push drafted, attached seven files and published (run 43157146501, all
  jobs success); `v0.7.1` the same way (see the log). For each release: commit `docs/arcs/v1/release-notes/<tag>.md`
  before tagging, then push the tag; never `gh release create` by hand.

## Watch items

- **Budget:** rung 8 is "large" on the board. At rung 7's rate it is likely 15–20 points. The weekly meter resets
  Saturday 14th, 5 pm. **Until that reset the hard stop is 90 %** (chris, 2026-11-12: "let's move the hard stop to 90%",
  then "This week only"); after it, back to 75 / 80 / 85 / 93 %. At 67 % (2026-11-12 21:39 UTC) that leaves 23 points
  this week, so rung 8 (or 8a) can start before the reset; a share bigger than what is left must split or wait.
- **The graphics-chip canvas** sits at 40–42 ms (90th percentile) on the groups page against the 50 ms line (ADR-009);
  the shapes page was 45.9–48.4 ms in rung 6. The device previews strip draws more pages per frame: measure dragging
  with the strip showing, on the heaviest page, the chip canvas included, and say before anything crosses 50 ms.
- **Landscape is no longer blocked** (2026-11-12, after this prompt was first written): rotated pixels, on the Nomad;
  see "Open questions" below and `docs/device-measurements/supernote.a6x2/landscape-test.md`.
- **Debt touched by rung 8:** the rung 7b line (align works on the primary page only, whichever device is on screen);
  the rung 7a line; the rung 2 viewer line; the Style tab still shows the first object's values, not "mixed"
  (`docs/STATUS-debt.md`).

## Open questions to ask again at the start (AGENTS.md: "ask open questions again at the next session start")

- **Landscape is answered on the Nomad** (2026-11-12, `docs/device-measurements/supernote.a6x2/landscape-test.md`):
  the device wants **rotated pixels**. A template is always a portrait page; a landscape design must be turned a
  quarter turn and saved as a portrait PNG, read in portrait mode with the tablet held sideways. A wide PNG is shrunk
  (soft); landscape mode zooms the page by 4/3 and scrolls, so nothing is pixel-exact there. Rung 8's bbq must decide
  which way a landscape design turns, and whether the landscape toolbar screenshots
  (`docs/device-measurements/<device>/landscape/`, zoomed views, not template pixels) matter at all, since a sideways
  template is used in portrait mode where the portrait safe zones apply. The Manta is ruled the same, not tested. The grays sheet is done (2026-11-12: Block 68 →
  102; Manta ruled equal to the Nomad).
- Still parked: "Scale vs pixel-locked rhythm on A5 X" (v1.1, arc Q16); the Label gray (v1.1); text and pattern
  rotation (v1.1, chris 2026-11-12: "Not in v1").

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(read the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board `docs/arcs/v1/README.md`, this file,
`docs/decisions/INDEX.md`, the rung 7 record `docs/bbq/2026-11-11-rung-7-groups-and-align.md` and
`docs/STATUS-debt.md`. Prove the build matches HEAD (`make build-proof`). Take the meter reading (or ask chris) and set
this rung's budget with `budget-select`. Ask the open questions above. Then run a short bbq (about 8 questions) for
**rung 8: several devices** — secondary profiles, Linked and Forked (spec §3.4), the device previews strip, per-object
overrides with badges and an Overrides tab (spec §6.1 line 278), lint L15 (spec §8.3), and portrait and landscape
orientation (spec §3.2 `orientations`, §17.1 item 3) — including how overrides apply to groups under the Figma rule; the
file format bump this needs (0.8.0 through the migration chain); and whether to split 8a/8b (orientation is now unblocked: a landscape design exports turned into portrait pixels). Have Sonnet 5.5 at high effort build it on `main`; keep the eight golden lines and the
fixtures frozen (stop and ask chris if any would change); add a golden page per device for what changes export; measure
drawing and dragging on the Pi (90th percentile, ADR-009) on the heaviest page with the previews strip showing, the chip
canvas included; pay the debt in code the rung touches. Show chris the window and stop for his look. After his approval:
bump to `0.8.0`, commit the notes in `docs/arcs/v1/release-notes/`, tag `v0.8.0` (the tag push publishes), re-plan the
board, and write the next lead's prompt in `docs/arcs/v1/`. Update `STATUS.md`, `docs/STATUS-log.md` and
`docs/STATUS-debt.md` at the end of each session, and write the calibration row in `docs/budget-calibration.md`.
