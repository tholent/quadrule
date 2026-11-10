---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify, in his review of the pending ADRs)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: design
built: partial
proof: "`crates/profiles/tests/profiles.rs::verified_profiles_have_a_record`, `crates/profiles/tests/profiles.rs::role_grays_are_clean_levels`, `crates/xtask/tests/safezones.rs::profiles_match_their_screenshots`, `crates/lint/tests/lint.rs::every_toolbar_zone_is_known`, `crates/ui/tests/drag.rs::the_right_toolbar_zone_is_shown_and_caught_by_default`, `crates/layout/tests/snap.rs::union_outline_leaves_out_borders_inside_the_combined_area`"
owed: "clean gray levels measured on both devices; the Manta's pen shades (assumed equal to the Nomad's); safe zones for landscape, a collapsed toolbar and other floating positions; the lint rules that need calibrated values (L03, L05, L07, L09, L13, L14)"
supersedes: nothing
amends: nothing
---
# ADR-012 — Lines and text use seven named roles whose values come from calibration on the real tablet and live in profile data, and safe zones are measured from screenshots, linted at every toolbar position and drawn without the floating one

## Decision

1. **Seven roles:** Whisper, Guide, Structure, Emphasis, Text, and two added to the napkin: **Dot grid** (the dots of a
   dot grid, with their own size and gray) and **Label** (quiet labels, lighter than Text). File format 0.3.
2. **Values come from calibration, never code.** Each role's width and gray per device is chosen by chris from the
   calibration sheets (`cargo xtask calibration-sheet`, napkin §8.4) and stored in `crates/profiles/data/`, with the
   readings in `docs/device-measurements/*/calibration.md`. A verified profile must have a calibration record.
3. **Dots are square.**
4. **Safe zones are measured** from device screenshots by `cargo xtask safe-zones`, and a test keeps the profiles and
   screenshots in step.
5. **Lint checks every toolbar position** (left, right, top, floating) and the footers.
6. **The overlay and the safe-zone snap show the left, right and top toolbar zones and the footers, not the floating
   one;** overlapping zones are filled once and outlined only round their combined area, with no labels.

## Context

- Role and dot rulings, chris's answers word for word: "Yes, own setting (Recommended)" for the dot grid; "Text + new
  Label (Recommended)"; the name "Dot grid"; "yes, square dots are the ruling"; labels "td4 and tl4 work for lables"
  (9 pt, gray 153) (`docs/STATUS-log.md`, 2026-11-09). Napkin §8.2 carries a dated amendment note.
- Profiles: rung 1 Q6 (estimates labelled unverified, replaced by calibration without code changes).
- Safe zones, chris word for word: "I would like to remove the labels and include the right hand side safe area by
  default"; "the safe area linting should look at all safe areas, including the right one. Also, when two safe areas
  overlap, don't draw the borders that occur inside the combined area"; "the safe zone overlay should not include the
  floating menu and the overlapping portions should not increase their darkness" (rung 4 record, "After 4a" and
  "After 4b"). These replace the earlier ruling that lint checks only "Docked left, Docked top" (2026-11-09).

## Consequences

- Changing a role's look is a data change, re-checked on hardware, not a code change.
- Open until v1.1 (chris: "leave the label gray issue open until v1.1"): Label gray 153 sits 4 steps from the dark
  grey pen 157; lint rule L04 waits on it (`docs/STATUS-debt.md`).

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
- 2026-11-10: ratified by chris (answer "Ratify" to "ADR-012 … Ratify?", in his review of the pending ADRs). Prior header
  values: status `proposed`, status-note `drafted 2026-11-10 from the project's records (backlog sweep, no
  rounds, chris's choice); awaiting ratification`.
- 2026-11-10: amended by chris, after trying rung 5: the linter no longer checks the floating toolbar (see the
  Amendment below). Header values unchanged.

## Amendment — 2026-11-10, chris

**Touches:** Decision item 5 and the title's "linted at every toolbar position". Their text above stands as ruled and
is superseded here for the floating toolbar only.

chris, word for word, after trying rung 5: "There is one linting issue, L06 still checks for collisions with the
floating menu bar, which isn't desired."

**What changes:** lint checks the docked toolbars (left, right, top) and the footers by default; it does not check the
floating toolbar. The overlay and the snap already left it out (item 6), so the window now draws, snaps to and lints
the same three toolbar positions. The floating position stays in the code as an opt-in only (`cargo xtask lint
--toolbars all`), where its notes are Info as before. The new-page margins never cleared it
(`crates/layout/src/newpage.rs`).

**Why:** the floating toolbar moves, and chris does not want a page flagged for sitting where it first appears.

**Built:** `Toolbar::DOCKED` and `Options::default` in `crates/lint/src/lib.rs`; `TOOLBARS_IN_USE` in
`crates/ui/src/viewer.rs`; `cargo xtask lint` defaults to `left,right,top`; the Lint panel's note
(`app-lint-toolbars`). Proof: `crates/lint/tests/lint.rs::floating_toolbar_is_not_linted_unless_asked`,
`crates/ui/tests/drag.rs::the_floating_toolbar_zone_is_not_shown_or_caught`. The sample page's Manta findings went from
7 to 5 (`crates/ui/tests/window.rs::empty_start_then_sample`).
