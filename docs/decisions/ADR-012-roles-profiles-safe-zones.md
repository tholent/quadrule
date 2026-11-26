---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify, in his review of the pending ADRs)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: design
built: partial
proof: "`crates/profiles/tests/profiles.rs::verified_profiles_have_a_record`, `crates/profiles/tests/profiles.rs::role_grays_are_clean_levels`, `crates/xtask/tests/safezones.rs::profiles_match_their_screenshots`, `crates/lint/tests/lint.rs::every_toolbar_zone_is_known`, `crates/ui/tests/drag.rs::the_right_toolbar_zone_is_shown_and_caught_by_default`, `crates/layout/tests/snap.rs::union_outline_leaves_out_borders_inside_the_combined_area`, `crates/profiles/tests/lint_block.rs::both_supernotes_carry_the_measured_values_and_paper_only_two`, `crates/lint/tests/gray_rules.rs::a_rule_follows_an_edited_overrides_lint_block_and_an_edited_role`"
owed: "the Manta's gray levels and pen shades (ruled equal to the Nomad's, not measured); safe zones for a collapsed toolbar and other floating positions; measured values for L09, L13 and L14 (chosen, not measured); measured values for the Margin role (ADR-020; chris's drawn 5 px, gray 204 until read)"
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
   *2026-11-26: ADR-027 (proposed) moves every role gray to a new 16-step scale built on the pen grays, by chris's
   ruling, checked on new grays sheets.*
3. **Dots are square.**
4. **Safe zones are measured** from device screenshots by `cargo xtask safe-zones`, and a test keeps the profiles and
   screenshots in step.
5. **Lint checks every toolbar position** (left, right, top, floating) and the footers.

> 2026-11-23: items 1, 2 and 5 are amended by ADR-019 (objects marked Full bleed are skipped by L06) and
> ADR-020 (a Margin role), both ratified by chris 2026-11-23. See their files.
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
  *2026-11-26: settled by ADR-027 (proposed): Label moves to 157, and L04 is retired.*

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
- 2026-11-10: ratified by chris (answer "Ratify" to "ADR-012 … Ratify?", in his review of the pending ADRs). Prior header
  values: status `proposed`, status-note `drafted 2026-11-10 from the project's records (backlog sweep, no
  rounds, chris's choice); awaiting ratification`.
- 2026-11-10: amended by chris, after trying rung 5: the linter no longer checks the floating toolbar (see the
  Amendment below). Header values unchanged.
- 2026-11-17: amended by chris, rung 14a: lint thresholds live in profile data (see the Amendment of 2026-11-17).
  Prior header values, word for word: proof `"`crates/profiles/tests/profiles.rs::verified_profiles_have_a_record`,
  `crates/profiles/tests/profiles.rs::role_grays_are_clean_levels`,
  `crates/xtask/tests/safezones.rs::profiles_match_their_screenshots`, `crates/lint/tests/lint.rs::every_toolbar_zone_is_known`,
  `crates/ui/tests/drag.rs::the_right_toolbar_zone_is_shown_and_caught_by_default`,
  `crates/layout/tests/snap.rs::union_outline_leaves_out_borders_inside_the_combined_area`"`; owed `"clean gray levels
  measured on both devices; the Manta's pen shades (assumed equal to the Nomad's); safe zones for landscape, a collapsed
  toolbar and other floating positions; the lint rules that need calibrated values (L03, L05, L07, L09, L13, L14)"`.
  Status unchanged (`accepted`); built unchanged (`partial`).
- 2026-11-23: two new ADRs from chris's starter lint answers, ratified the same day ("ratify 019 and 020"), amend this one: ADR-019 (a Full bleed mark
  that L06 skips; item 5) and ADR-020 (an eighth role, Margin; items 1 and 2). Header values unchanged.
- 2026-11-23: owed gains the Margin role's measured values (ADR-020, built). Prior value, word for word: owed `"the Manta's gray levels and pen shades (ruled equal to the Nomad's, not measured); safe zones for a collapsed toolbar and other floating positions; measured values for L09, L13 and L14 (chosen, not measured)"`.
- 2026-11-26: ADR-027 (proposed, from the Label gray talk) amends item 2 and settles the open Label gray item: the
  role grays move to a 16-step scale built on the pen grays, and L04 is retired. Banners added at both. Header values
  unchanged.

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

## Amendment — 2026-11-17, chris

**Touches:** Decision item 2 ("Values come from calibration, never code"). Its text above stands as ruled; this extends
it from role values to lint thresholds.

chris, word for word, in the rung 14 bbq (`docs/bbq/2026-11-17-rung-14-lint-and-preflight.md`), Q2: "A lint block in
profiles (Recommended)"; Q3 and Q4: "Accept as listed (Recommended)"; at the 14a look: "approved. meter read is 46%.";
asked whether to record it here: "ratify now".

**What changes:** the numbers the lint rules judge against live in profile data too, never in code. Each profile has a
`lint` block (`LintThresholds` in `crates/profiles/src/lib.rs`, file format 0.15.0): the lightest visible gray for
1 px and wider strokes, for square dots by size, for fills and for text (L03); the smallest legible text size (L07);
the smallest comfortable handwriting step (L09); the dark-fill gray and page share (L13); the thin-line width (L14).
L05 reads the profile's `gray_levels`. The Supernote values come from chris's calibration sheets, each cited by its
sheet code in the RON files and `docs/device-measurements/supernote.a6x2/calibration.md`; the L09, L13 and L14 values
were chosen by chris, not measured. A value left empty skips its rule on that profile, and lint says which checks were
skipped and why. Like role values, the block is editable in the Profiles window as a local override with Reset to
built-in (ADR-016), and every rule reads the profile in use, never the shipped one.

**Why:** so each device, a user's own profile and later ones (the A5 X in v1.1) judge against their own measured
numbers, and recalibration changes data, not code, as item 2 already rules for roles.

**Built:** rung 14a (`ad948c3`, `c14628e`, `273f1cb`). Proof:
`crates/profiles/tests/lint_block.rs::both_supernotes_carry_the_measured_values_and_paper_only_two`,
`crates/lint/tests/gray_rules.rs::a_rule_follows_an_edited_overrides_lint_block_and_an_edited_role`.
