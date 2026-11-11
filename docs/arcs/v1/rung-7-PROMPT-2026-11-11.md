# Next lead's prompt — quadrule rung 7: groups; align and distribute

2026-11-11 · written by the rung 6 lead (Opus 5.5, high effort) at the end of rung 6, as the board asks
(`docs/arcs/v1/README.md`, "Every lead … updates it at the end of its rung"). The arc's rules (record
`docs/arcs/v1/v1-PROMPT-2026-11-10.md`, Q1–Q20) hold unchanged; this file only hands over.

## Where rung 6 left things

- Rung 6 is done: 6a (shapes, fill roles, dashes, caps; format 0.5.0) approved 2026-11-11, tagged `v0.6.0`; 6b
  (rotation, lines any way, opacity; format 0.6.0) approved 2026-11-11 with one change (the box and handles turn with
  the object, built in `e8a49d3`), tagged `v0.6.1`. Record `docs/bbq/2026-11-10-rung-6-shapes.md`; log
  `docs/STATUS-log.md` (2026-11-10 and 2026-11-11 entries).
- How it ran: one lead session; each part built by one Sonnet 5.5 high subagent of the lead's session (6a about 351k
  tokens; 6b about 385k, and about 407k as reported at the end of the look change), on `main`. Far under the record's "about 3
  sessions"; the bbq record carried enough detail that the builders needed no questions.
- **Frozen export bytes are now six golden lines** in `crates/render/tests/golden/SHA256SUMS`: `slice_*` (rung 1),
  `shapes_*` (6a), `transforms_*` (6b), plus the format fixtures `crates/render/tests/fixtures/v0.4/` and `v0.5/`.
  A rung 7 change that moves any of them is a stop: ask chris (arc Q20).
- **GitHub releases:** chris published `v0.6.0` and `v0.6.1` by hand on 2026-11-11, and they have **no packages**:
  the repository has immutable releases on, so the old workflow could not attach files after publishing. chris ruled
  "Keep, add a note" and "Tag push does it all": `.github/workflows/release.yml` now runs on a `v*` tag push, drafts
  the release with `docs/arcs/v1/release-notes/<tag>.md`, attaches the packages and publishes (`DEVELOPMENT.md`). So
  for `v0.7.0`: commit the notes file *before* tagging, then push the tag; **never `gh release create` by hand**, or
  the release goes out empty and its tag name is burnt. chris said the lead now has permission to release (not tried
  yet: the next tag push is its first real use). Watch the run (`gh run list --workflow release.yml`) and check the
  release's files after.

## Watch items rung 6 left for later rungs

- **The graphics-chip canvas is near its line on the shapes page:** 90th percentile 45.9–48.4 ms against 50 ms
  (ADR-009) on the Pi, release build, against about 25 ms on the export canvas. Groups add little per frame, but rungs
  9, 10 and 16 (patterns, repeat, starters) make heavier pages. Measure each rung on the heaviest page it can make,
  and say so before it crosses 50 ms.
- **Portable maths (ADR-003 item 2):** on x86 the portable maths differ from the fast ones by one gray level on a few
  pixels of the shapes and transforms pages; the exported bytes are identical on Linux, Windows, macOS and the Pi, so
  item 2's trigger is not met. The golden tests fail on any system whose export differs, which is the real trigger.
- **Rotation in groups (6b debt, `docs/STATUS-debt.md`):** only one object turns at a time; a group has no rotate
  handle; resizing a selection with turned objects stretches them approximately; text boxes and pattern regions do
  not rotate. Rung 7 must decide how a group turns, and whether its children's angles compose.
- **Picking by box:** hollow and turned shapes are picked by their box, not their outline (6a, 6b debt).

## Open questions to ask again at the start (AGENTS.md: "ask open questions again at the next session start")

- Should text boxes and pattern regions rotate, and in which rung (6b left them out; the spec is silent, §5.2)?
- Should a multi-selection's box turn (6b left it upright; chris's look change covered one object)?
- chris's hardware queue (board): the grays sheet on both devices (Shade 204 and Block 68 are estimates until then);
  a landscape PNG check and landscape toolbar screenshots (rung 8 needs them).
- Still parked: "Scale vs pixel-locked rhythm on A5 X" (v1.1, arc Q16); the Label gray (v1.1).

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(read the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board `docs/arcs/v1/README.md`, this file,
`docs/decisions/INDEX.md`, the rung 6 record `docs/bbq/2026-11-10-rung-6-shapes.md` and `docs/STATUS-debt.md`. Prove
the build matches HEAD (`make build-proof`). Take the meter reading (or ask chris) and set this rung's budget with
`budget-select`. Ask the open questions above. Then run a short bbq (about 8 questions) for **rung 7: groups (nested,
double-click to enter, group constraints) and align and distribute** (napkin §5.5, §6.3; spec line 276 puts align and
distribute in the Arrange tab), including how groups turn (6b debt); the file format bump this needs (0.7.0 through
the migration chain); and whether to split 7a/7b. Have Sonnet 5.5 at high effort build it on `main`; keep the six
golden lines and the fixtures frozen (stop and ask chris if any would change); add a golden page per device for what
changes export; measure drawing and dragging on the Pi (90th percentile, ADR-009) on the heaviest page the rung can
make, the chip canvas included; pay the debt in code the rung touches. Show chris the window and stop for his look.
After his approval: bump to `0.7.0`, tag `v0.7.0`, publish the pre-release with notes in
`docs/arcs/v1/release-notes/`, re-plan the board, and write the next lead's prompt in `docs/arcs/v1/`. Update
`STATUS.md`, `docs/STATUS-log.md` and `docs/STATUS-debt.md` at the end of each session.
