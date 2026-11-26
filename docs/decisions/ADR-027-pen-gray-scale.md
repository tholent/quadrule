---
status: accepted
status-note: "ratified by chris 2026-11-26 (\"1. ratify\")"
ratified-by: chris
ratified-on: "2026-11-26"
owner: chris
kind: design
built: partial
proof: "`crates/profiles/tests/pen_grays.rs::the_tablets_steps_end_on_255_and_hold_every_pen_but_white`, `crates/profiles/tests/pen_grays.rs::roles_and_lint_grays_move_to_the_new_steps`, `crates/profiles/tests/pen_grays.rs::paper_roles_move_and_the_registry_ships_the_new_values`, `crates/format/tests/pen_grays.rs::on_step_grays_move_and_others_stay`, `crates/ui/tests/pen_gray_names.rs::pen_swatches_are_named_after_their_pen`, `crates/xtask/tests/pen_grays_sheet.rs::the_grays_sheet_shows_the_new_steps_and_the_points_between`, `crates/xtask/tests/pen_grays_starters.rs::every_starter_lints_clean_on_the_pen_scale`"
owed: "chris's readings of the new grays sheet (N1–N30) on both tablets and his look, all behind `--pen-grays`; then the flag removed, the format at 0.22.0, the profiles' data files written with the new values and the golden lines made again once (Steps to build, items 4 and 6)"
supersedes: nothing
amends: "ADR-012 §Decision item 2 (role grays: moved to the new steps by chris's ruling, then checked on a new grays sheet) and its open Label gray item"
---
# ADR-027 — The tablets' 16 grays are built on the Supernote pen grays, labels take the dark grey pen's gray, and L04 is retired

## Decision

1. **A template gray may match a pen's gray.** Writing in a pen that looks the same as a printed line or label is
   fine (D1). This settles the 153/157 question parked since 2026-11-09.
2. **The 16 steps are rebuilt on the pen grays.** The black pen (1), the dark grey pen (157) and the light grey pen
   (201) become steps in place of the steps nearest them; the page white stays 255; the other twelve are spaced
   evenly between those four (D4, D5, D6):

   | old step | 0 | 17 | 34 | 51 | 68 | 85 | 102 | 119 | 136 | 153 | 170 | 187 | 204 | 221 | 238 | 255 |
   |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
   | **new step** | **1** | 18 | 36 | 53 | 70 | 88 | 105 | 122 | 140 | **157** | 172 | 186 | **201** | 219 | 237 | **255** |

   The white pen's 254 is not a step. The Manta takes the same steps: its pens are the Nomad's (D12).
3. **Each role moves to the new step nearest its old gray** (D8), on both tablets and on every paper profile (D13):
   Emphasis 0 → 1; Guide, Structure, Dot grid and Text 51 → 53; Whisper 119 → 122; Block 102 → 105; **Label 153 → 157**
   (the dark grey pen); Margin and Shade 204 → 201 (the light grey pen). Widths do not change.
4. **The gray picker names the pen steps**, for example "157 · dark grey pen", from the profile's `ink_values`; a
   profile with no pens (the paper sizes) shows no names (D11).
5. **Older files move onto the new steps.** A file format step moves each gray picked by hand in a `.quad` that sits
   on one of the old 16 steps to the matching new step when the file is opened (D10); a gray off the old steps stays
   as it is (chris: "move only on-step grays").
6. **L04 is retired** (D9): "Gray or weight close to pen ink" is dropped from the spec's lint list by a dated
   amendment under the §8.3 table, and is not built.
7. **Built as its own job after `v0.17.0`, behind a flag until chris's look, with new grays sheets for both tablets**
   (D7). The flag, `--pen-grays` (chris: "keep"), covers export, lint, the profiles and the format step, since all four
   change export bytes or the file format (ADR-007, amendment of 2026-11-10).

## Context

- Rung 17's bbq left the Label gray for a talk (Q9, `docs/bbq/2026-11-24-rung-17-polish.md`). The talk is
  `docs/proposals/talk-2026-11-26-label-gray-and-pen-grays.md`; the D-numbers above are its decisions. Every one is
  agreed (in the talk and its two amendments of 2026-11-26).
- Labels are 9 pt at gray 153 (`label_gray`, `crates/profiles/data/supernote.a6x2.ron`), chosen on the text sheet
  (`docs/device-measurements/supernote.a6x2/calibration.md`). The pens write 1, 157, 201 and 254 (`ink_values`), and
  each sits within 4 of an old step: 0, 153, 204, 255.
- On the grays sheet of 2026-11-12 every patch was clean, the in-between ones included, and each in-between patch
  matched a neighbour (chris: "Each matches a neighbour"). Every new step is within 4 of the old step it replaces, so
  the tablet very likely shows each new step exactly as it showed the old one. That is inferred; the new grays sheets
  check it.
- Supernote's own stock grid draws at gray 1 on a page of 255 (`docs/device-measurements/README.md`): the black pen's
  gray, and a white that is not the white pen's.
- L04 was to warn when a template gray sits near a pen's gray (`napkins/quadrule-spec.md` §8.3, moved to v1.1 on
  2026-11-10). Under this scale Label, Margin and Shade sit on pen grays by design, so it would fire on almost every
  starter.

Drivers:
- A template's grays should say what they are: "the dark grey pen's gray" rather than "153, four from the pen".
- Nothing should change on the tablet's screen.
- Old files should come out on the clean scale, with no L05 notes.

## Alternatives

### Rebuild the 16 steps on the pen grays (chosen: the pens become the scale's fixed points and nothing visible changes)
- Good: Label, Margin and Shade land on pen grays exactly; the scale stays 16 clean steps; no extra choices to learn.
- Bad: every role's gray moves, so every golden page changes once; a format step; new grays sheets.

### Keep the old steps and add the pen grays as extra named choices
- Good: no golden page changes; no format step.
- Bad: two grays a few levels apart that look the same, side by side in the picker; L05 must make an exception.
- Agreed first in the talk (D2), then replaced by D4 when chris asked for the pens in the scale.

### Keep everything as it is
- Good: no work.
- Bad: the Label gray stays 4 from a pen with the question open, and L04 stays unbuildable.

### End the scale at the white pen's 254, or carry both 254 and 255 (overruled)
- Good: every pen gray, white included, would be a step.
- Bad: 254 as the top step changes every page's background; carrying both makes 17 entries that look the same.
- Claude first suggested 254 sit beside 255 as the white pen's step. chris kept 255 as the page white and ended the
  scale on it ("keep 255 as page white"; "yes, end on 255").

## Steps to build, each with its test

1. **The profiles**, behind the flag: the new `gray_levels` on both tablets and the moved role grays on every built-in
   profile. Test: `role_grays_are_clean_levels` still passes with the flag on, and a new test checks the tablets' 16
   steps are the list above, end on 255 and hold every pen gray but the white one.
2. **The picker's pen names**, words through Fluent. Test: the Nomad's picker names 1, 157 and 201 and nothing else;
   a paper profile names none.
3. **The format step** 0.21.0 → 0.22.0. Test: a 0.21 fixture with hand-picked grays (153, 204, 0, 160) opens as 157,
   201, 1 and 160, and its role-styled objects are untouched in the file.
4. **The grays sheet**: the new steps and the points halfway between them (`cargo xtask calibration-sheet`,
   `crates/xtask/src/calib.rs`). Test: the sheet's codes stay unique (`crates/xtask/tests/calib.rs`); chris reads it
   on both tablets and the readings go in each `calibration.md`.
5. **The spec and the debt list**: the L04 amendment under napkin §8.3; the "Open until v1.1" item in
   `docs/STATUS-debt.md` closed; ADR-012's owed note updated. Check: the linter's comments and `SKIPPED_ON_PAPER`
   stop naming L04 (`crates/lint/src/lib.rs`, `crates/lint/tests/paper.rs`).
6. **At chris's look**: the flag comes out and the golden lines are made again once
   (`crates/render/tests/golden/SHA256SUMS`, as ADR-018's one-time change). Test: every golden page differs from the old
   only by the gray moves listed in item 3 of the Decision.

## Consequences

- Every exported page changes by a few gray levels; on the tablet it should look the same (the new sheets confirm).
- Opening an older file rewrites its hand-picked grays that sat on the old steps; one chosen off the steps (160) is
  kept, and L05 notes it as before.
- User-made profiles and edited Manta or Nomad overrides keep their own grays until Reset (Defaulted, question record).
- ADR-012's rule that role values come from calibration holds: chris ruled the moves here, and the new grays sheet is
  the calibration check.
- The question record is `docs/bbq/2026-11-26-adr-pen-gray-scale.md` (premises and the Defaulted calls).

## History

- 2026-11-26: drafted from chris's answers in `docs/proposals/talk-2026-11-26-label-gray-and-pen-grays.md`, through
  makeadr (goforit: no rounds, every answer cited).
- 2026-11-26: revised before ratifying, from chris's answers to the draft's asks, word for word: "2. move only
  on-step grays 3. keep". Decision item 5 narrowed to grays on the old steps (a gray off them stays); item 7 names the
  flag `--pen-grays`; build step 3's test and the Consequences follow; Asks left open removed.
- 2026-11-26: ratified by chris ("1. ratify"). Prior header values, word for word: status `proposed`, status-note
  `drafted 2026-11-26 from chris's answers in the Label gray talk; awaiting ratification`.
- 2026-11-26: built behind `--pen-grays` (steps 1, 2, 3 and 5, and the sheet of step 4), for `v0.17.1`. Prior header values,
  word for word: built `not started`, proof `"queued after `v0.17.0`, behind a flag (`docs/proposals/talk-2026-11-26-label-gray-and-pen-grays.md`, action items 3 and 4)"`.
