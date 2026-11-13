# bbq record — quadrule rung 8: several devices

2026-11-12 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`) · 8 questions · lead:
Opus 5.5 at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 8 row), `docs/arcs/v1/rung-8-PROMPT-2026-11-12.md`,
`docs/decisions/INDEX.md` (ADR-015), `docs/bbq/2026-11-11-rung-7-groups-and-align.md`, `docs/STATUS-debt.md` (rung 2,
7a, 7b lines), `docs/STATUS-log.md` (newest entries), `docs/device-measurements/supernote.a6x2/landscape-test.md`,
`napkins/quadrule-spec.md` §3.2, §3.4, §5.1–§5.2, §6.1 (line 278), §8.3 (L15), §11, §12, §16, §17.1 item 3, §17.3
("Profile forking: v1.1; per-object overrides cover v1"), `crates/model/src/lib.rs` (`Document.profiles`,
`ProfileContext`: "forking arrives in v1.1"), `crates/ui/src/drag.rs:25` (an edit on a secondary device is unresolved
back into the shared design today), `crates/ui/src/arrange.rs:18` (align on the primary page only),
`crates/profiles/data/*.ron` (`orientations: [Portrait]`).

Measured at the start of this session (2026-11-12, on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=bb0b5ed dirty=no rebuilt=6`.

Asked again at session start (AGENTS.md): the landscape questions the rung 8 prompt carried became Q6 (which way a
landscape design turns) and a Defaulted call (the landscape screenshots). Still parked, not re-asked: "Scale vs
pixel-locked rhythm on A5 X" (v1.1, arc Q16); the Label gray (v1.1); text and pattern rotation (v1.1).

## Budget
Share: 20 points (1 point = 1 % of the weekly limit) of a 20-point ceiling, alone on the meter; reserve 0 kept for
chris.
Room: 20 points (the lower of the ceiling and the hard stop less the reading, 90 − 67 = 23, less the reserve).
Written when the weekly meter (resets Saturday 14th, 5 pm) read 67 % (2026-11-12 21:59 UTC, given by chris, not
measured; logged with `spend-ledger.py reading 67 --meter weekly`); that reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. If other sessions share the meter, your share line is your ledger, never a meter percentage. Read the meter
before each batch of dispatches; log a reading only at batch ends and when a stop line is within 5 points. Stop
starting new work at the first of: your ledger reaching 20 points (alone on the meter: the reading minus your start
reading; the token ledger is (finished workers' tokens + your context) ÷ K, K = 12.2M tokens/point fitted over two pairs
in `docs/budget-calibration.md`, not yet trusted, so trust the meter's movement); the meter reaching your start + 20; the
hard stop (**90 % until Saturday 14th, 5 pm**, chris, this week only; after the reset 75 % beyond 48 h, 80 % at 24–48 h,
85 % at 12–24 h, 93 % under 12 h). In-flight work finishes. Hand off at ~500k of your context. Before handoff, write
your row in the calibration log.

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6; the rung 8 prompt): 8.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Set with `budget-select` above.
- **Groups.** See-through to constraints, as in Figma (ADR-015; rung 7 record, Q3 amendment).

## Rules

- **Q1 Split.** Two parts, a look each (arc Q10). **8a, several devices:** secondary devices, the device previews strip,
  per-device overrides with badges and an Overrides tab, lint L15, file format 0.8.0. **8b, orientation:** portrait
  and landscape, a landscape design exported turned into portrait pixels.
- **Q2 Forked.** Not in rung 8: forking stays in v1.1, as spec §17.3 rules ("Profile forking: v1.1; per-object
  overrides cover v1"). Every secondary device is Linked. The file keeps room for it (`ProfileContext.extra`).
- **Q3 Editing on a secondary device.** The spec's list is per device (§3.4): **position, size, font size and
  visibility** become overrides for the device on screen only. Every other edit (style, fill, dash, cap, text, rotation,
  opacity, constraints, name, lock, order) changes the shared design on all devices, and the status bar says so when it
  is made on a secondary.
- **Q4 Overrides and groups.** A group moved or resized on a secondary gets **one override on the group**: its box on
  that device. Its children resolve by their own constraints (and their own overrides, if any) to an outer box, which
  is then scaled into the group's override box, as a hand resize scales a group (ADR-015). One badge on the group;
  Reset puts it back to its children's resolved outer box. A child can still be overridden inside it.
- **Q5 The previews strip.** Along the bottom of the canvas, above the status bar, foldable (the fold kept between
  runs): a small live picture of each device's page side by side, the device on screen outlined. Clicking a picture
  puts that device on screen as the editing context (spec §3.4).
- **Q6 Which way landscape turns (8b).** **Clockwise, as Test B**: the design's top lands on the device's right edge,
  read holding the tablet with its right edge up (`docs/device-measurements/supernote.a6x2/landscape-test.md`).
- **Q7 Lint L15** (Info, spec §8.3). Fires **only when constraints cause trouble** on a secondary device, for an object
  with no position or size override there: it goes (partly) off the page there while it is on the page on the primary;
  it newly overlaps an object it does not touch on the primary; or a side is squeezed below one device pixel. The
  finding names the device and the object, and outlines it.
- **Q8 Anything else to protect or avoid?** Nothing more: the standing rules hold (the eight golden lines in
  `crates/render/tests/golden/SHA256SUMS` and the fixtures `crates/render/tests/fixtures/v0.4/`, `v0.5/`, `v0.6/` frozen,
  stop and ask chris if any would move; drawing and dragging under 50 ms at the 90th percentile on the Pi with the strip
  showing, the chip canvas included, ADR-009; commits straight to `main`).

## Owed

(empty: three owed after round 1 became Q5, Q6, Q7; the fourth, switching the primary device, is Defaulted below)

## Defaulted

- **Size, said out loud:** 8a about one large builder run (about 0.6–0.9M tokens), 8b about half that; the rung about
  10–16 points of the 20, plus two looks. 8a can land before Saturday's reset; 8b may run after it, on the new week's
  reading. veto: none expected.
- **File format 0.8.0 (8a)**, through the migration chain, carrying both parts' fields so 8b needs no bump: each
  `Object` gains `overrides: Map<profile id, Override>` (empty by default, skipped when empty), an `Override` holding
  optional `position` (x, y), `size` (w, h), `font_size_pt` and `visible`, each in that device's page millimetres; and
  `PageSettings` gains `orientation: Portrait | Landscape` (Portrait by default; only Portrait is offered until 8b).
  Nothing old changes, so the 0.7 → 0.8 step is a no-op and every older file opens with the same export bytes; add a
  `v0.7/` fixture beside the others. An override for a device the document does not target is kept, unused. veto: a
  separate bump in 8b; overrides as a table beside the objects.
- **Position and size are separate overrides:** a move or nudge on a secondary sets position only (size still follows
  constraints); a resize sets both; a font size change sets font size; the eye on a secondary sets visible. veto: one
  frame override.
- **Lines** are overridden by their frame as other objects are (their two ends follow it). veto: none expected.
- **Switching the primary device (the fourth owed question):** the Page tab lists the document's devices, the primary
  marked; **Make primary** on a secondary makes its resolved layout (with its overrides) the stored design, clears its
  overrides, and gives the old primary an override wherever its resolved place would otherwise move by more than half a
  device pixel, so **no pixel changes on either device**. One undo step. veto: Make primary only at New…, or a lossy
  switch.
- **Which devices a document targets:** the Page tab ticks devices in and out (the primary cannot be unticked); a
  document may target one device only; unticking a secondary that has overrides asks first, and undo restores them.
  New… keeps targeting both devices. veto: always both.
- **Badges:** an overridden field in the Arrange and Text tabs shows a small filled dot and a tooltip ("Changed on
  Nomad"); the layers row shows the same dot while that device is on screen; each picture in the strip shows how many
  objects are overridden on it. veto: colour instead of a dot (taste: chris's look decides).
- **The Overrides tab** (spec §6.1 line 278): shown when a secondary is on screen and one object is picked; lists each
  overridden property with its value here and its linked value, a Reset beside each, and "Reset all on this device".
  With several objects picked, "Reset all on this device" applies to all. Each Reset is one undo step. veto: none.
- **The inspector on a secondary:** X, Y, W, H, font size and the eye show the resolved values on that device; typing
  one sets an override. Rotation, Style and the rest edit all devices (Q3), with the Style tab saying so in one line.
  veto: none expected.
- **Snapping on a secondary:** drags and nudges snap in that device's pixels, safe zones and rhythm, as they do today
  when the other device is on screen; the override stores where the object landed. veto: none expected.
- **Align and distribute on a secondary** work on that device's resolved boxes and write position overrides there
  (pays the rung 7b debt "align and distribute work … on the primary page … whichever device is on screen"). On the
  primary they change the stored design, as now. veto: align always on the primary.
- **Group drag on a secondary** writes the group's override (Q4) instead of unresolving each child (pays the rung 7a
  debt "A group dragged or resized while the other device is on screen moves each child's resolved box …"). veto: none.
- **Clipboard and duplicate** carry overrides with the objects; Group bakes nothing (children keep theirs); Ungroup
  bakes a group's override into its children (each child gets the position and size the group's fit gave it on that
  device). veto: drop overrides on Ungroup.
- **The strip's pictures** are drawn with export pixels, scaled down, and update when an edit ends (a gesture's end,
  an undo, a field change), not on every frame of a drag, so a drag costs no more than today; they show the last
  picture meanwhile. Measured on the Pi with the strip showing. veto: live during a drag.
- **The device picker** in the toolbar stays; the strip and the picker always agree. veto: none expected.
- **Lint:** every rule runs per device on its resolved, overridden layout, as now; the Lint panel shows the device on
  screen. Filtering by device stays in rung 14. veto: none expected.
- **Export** of each device uses its overrides. The primary's export never changes from an override.
- **Goldens (8a):** one new golden page per device holding a position override, a size override, an object hidden on
  the Nomad, a font-size override, a group with an override holding a child with its own, and one L15 case; the eight
  existing lines unchanged. veto: none expected.
- **8b, orientation:** chosen in New… (Portrait or Landscape) and in the Page tab (one undo step; objects follow their
  own constraints onto the turned page, which is the device's height by its width). The canvas shows a landscape design
  upright; the export draws it at the turned size and turns the pixels a quarter turn clockwise (Q6) into the device's
  portrait PNG, an exact pixel turn. Both profiles' `orientations` gain Landscape. veto: Page-tab switch absent (New…
  only).
- **Landscape safe zones:** a landscape template is read in portrait mode, so the **portrait safe zones apply, turned
  into the design's frame** (the portrait left toolbar shows along the landscape design's bottom, and so on), for the
  overlay, snapping, L06 and New…'s margins. The landscape toolbar screenshots
  (`docs/device-measurements/<device>/landscape/`) show a zoomed view, not template pixels, and are kept as records
  only, not measured. veto: measure them anyway.
- **Export names** keep ADR-013's `{title}_{device}_{w}x{h}`, with the PNG's portrait size; the `{orientation}` token is
  rung 15's. veto: add "landscape" now.
- **Goldens (8b):** one landscape page per device; all earlier lines unchanged. veto: none expected.
- **Speed:** measure dragging and drawing on the Pi in a release build, both canvases, 90th percentile (ADR-009), on the
  heaviest page (the shapes page and the groups page; whichever is slower), with the strip showing; a bench flag shows
  the strip (`make bench-*` as before). Say before any number crosses 50 ms. veto: none expected.
- **Debt paid in code the rung touches:** the 7b align line and the 7a group-on-the-other-device line (above); the
  Style tab shows "mixed" where the picked objects differ (rung 6a/7b line), since 8a changes the inspector; the rung 2
  viewer's Ctrl + scroll keeps the point under the pointer still, since 8a changes the canvas area (`crates/ui/src/view.rs`).
  veto: none expected.
- **Tests:** headless, one `CHECK` line each, printed starting with a newline: 0.7 files open with the same export
  bytes; overrides round trip through save; a move, resize, nudge, font-size change and eye on a secondary set only that
  device's override and leave the primary's export bytes unchanged; a style edit on a secondary changes both; Q4 group
  overrides with a child override inside; Make primary changes no pixel on either device; unticking a device; Reset and
  Reset all; align on a secondary; clipboard and Ungroup carry overrides; each L15 case and its quiet cases; the strip's
  click puts the device on screen; the new goldens; the existing goldens unchanged; 8b: orientation round trip, the
  clockwise pixel turn, turned safe zones, New… landscape margins. veto: none expected.
- **Words and access:** every new string through Fluent; every new control named for screen readers. veto: none.
- **Versions and releases:** 8a's approval commit bumps the workspace to `0.8.0` and is tagged `v0.8.0`; 8b's is
  `0.8.1`, `v0.8.1`. Commit `docs/arcs/v1/release-notes/<tag>.md` before tagging, then push the tag; the release
  workflow drafts, attaches and publishes; never `gh release create` by hand. veto: none expected.
- **Landing:** commits straight to `main` (`AGENTS.md`, amendment of 2026-11-09); CI checked on each push. README
  ("what works today") and DEVELOPMENT.md kept true. veto: none expected.

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(read the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board `docs/arcs/v1/README.md` and this record
(`docs/bbq/2026-11-12-rung-8-several-devices.md` on `main`). Prove the build matches HEAD (`make build-proof`). Take the
meter reading and apply the Budget block. Have **Sonnet 5.5 at high effort** build **8a** on `main`: file format 0.8.0
with a no-op migration step and a `v0.7/` fixture; per-device overrides (Q3, Q4); the previews strip (Q5); badges and the
Overrides tab; Make primary and the device ticks; align, nudge and group drags on a secondary as overrides; L15 (Q7); a
new golden page per device; the debt named above. Keep existing export bytes frozen (stop and ask chris if any would
change). Judge the builder's work against this record, run `make check-all`, measure drawing and dragging on the Pi
with the strip showing (90th percentile, ADR-009), show chris the window and stop for his look. After his approval:
bump to `0.8.0`, commit the notes, tag `v0.8.0` (the tag push publishes), then 8b (orientation, Q6) the same way,
tagged `v0.8.1`. Re-plan the board after the rung, and write the next lead's prompt in `docs/arcs/v1/`. Update
`STATUS.md`, `docs/STATUS-log.md` and `docs/STATUS-debt.md` at the end of each session, and write the calibration row
in `docs/budget-calibration.md`.

## Amendments

- **2026-11-13, chris at the look of 8a and 8b together, word for word:** "1. let's keep the color, but put on v1.1 a
  way to click the override dot and have the app open the overrides tab, otherwise approve all and let's bundle this
  into a single v0.8.0 release". So: 8a and 8b are approved as built; the overrides dot keeps the accent colour
  (Defaulted "Badges"); clicking the dot to open the Overrides tab is v1.1 (spec §16 v1.1 note); the two calls the
  lead named at the look stand as built (turning a page maps override boxes in proportion; margins stay the numbers
  typed). **Versions and releases** changes: 8a and 8b ship as one `v0.8.0`; there is no `v0.8.1`.
