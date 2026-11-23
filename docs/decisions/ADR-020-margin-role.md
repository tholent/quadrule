---
status: accepted
status-note: "ratified by chris 2026-11-23 (\"ratify 019 and 020\")"
ratified-by: chris
ratified-on: "2026-11-23"
owner: chris
kind: design
built: partial
proof: "`crates/profiles/tests/profiles.rs::every_profile_has_the_margin_role_and_older_ones_read_the_default`, `crates/profiles/tests/profiles.rs::role_grays_are_clean_levels`, `crates/xtask/tests/calib.rs::codes_are_unique_and_mean_the_same_everywhere`"
owed: "chris's readings of the margin strips W19–W24 on both tablets, which set the role's values (item 2); until then chris's drawn 5 px, gray 204 (item 3)"
supersedes: nothing
amends: "ADR-012 §Decision item 1 (the named line roles) and item 2 (values from calibration)"
---
# ADR-020 — Line roles gain an eighth, Margin, for wide, light lines such as a notebook's margin

## Decision

1. **A Margin role.** The line roles gain **Margin**, a wide, light line: a notebook's margin rule, and the like in
   Cornell, notes and planning starters. It sits with the others in the Style tab's role list and in every profile's
   `line_roles`, with a width in whole pixels and a gray.
2. **Calibrated like the others.** Its values come from the calibration sheets on both tablets (ADR-012 item 2): the
   sheet gains Margin samples, and chris's readings set the values.
3. **Until then, chris's drawn values.** Both tablets start at 5 px (his 0.45 mm at 300 PPI, to whole pixels) and gray
   204, marked chosen, not measured, as L09's and L13's values are (ADR-012, owed). Paper profiles take the Nomad's
   values by physical size, as they do for every role.

## Context

- chris's Paper starters draw a full-height margin line with custom values (0.45 mm, gray 204). L02 flags it on both
  tablets (5.31 px, drawn 5), and the starters must use line roles (spec §13; `docs/arcs/v1/rung-16b-PROMPT-2026-11-18.md`
  "Each starter"). Record `docs/bbq/2026-11-23-adr-starter-lint.md`, C.
- No role is wide and light: Whisper is 1 px at gray 119, Guide 1 px at 51, Structure 2 px at 51, Emphasis 3 px at 0
  (`crates/profiles/data/supernote.a6x2.ron`).
- Role values come from calibration, never code (ADR-012 item 2), so a new role needs readings.

Drivers:
- Margins recur across categories (Paper, Notes, Planning), so they should look alike and adapt per device.
- Starters use roles, not their own numbers.

## Alternatives

### A new Margin role (chosen: one look for every margin, set per device by calibration)
- Good: every starter's margin matches; each tablet's value is measured; a user changes all margins at once in the
  Profiles window.
- Bad: a new role in the model, the profiles and the file format; a calibration round on both tablets.

### Custom values in whole pixels (5 px, gray 204), allowed in starters
- Good: no new role, no calibration.
- Bad: each starter carries its own numbers, which drift apart and do not adapt per device; the 16b rule changes.

### An existing role (Structure or Whisper)
- Good: no change at all.
- Bad: the margin comes out darker and thinner than chris drew it.

## Consequences

- The role list, each built-in profile, the profile editor and the `.quad` format (a role name) gain Margin; older
  files open unchanged.
- The calibration sheet gains Margin samples (`cargo xtask calibration-sheet`); chris reads them on both tablets.
- ADR-012's owed list gains the Margin role's measured values until they are read.
- Tests: every profile has a Margin role; the starters' margin line uses it and L02 is quiet.

## Asks left open

1. Ratify ADR-020 as written?

## History

- 2026-11-23: drafted from chris's answer C in the starter lint record, through makeadr.
- 2026-11-23: ratified by chris ("ratify 019 and 020"). Prior header values, word for word: status `proposed`,
  status-note `drafted 2026-11-23 from chris's answer (starter lint record, C); awaiting ratification`.
- 2026-11-23: built (lead session, after the ratification). Prior header values, word for word: built `not started`, proof
  `"queued before rung 16b (`docs/bbq/2026-11-23-adr-starter-lint.md`, Owed; `docs/arcs/v1/README.md` row 16)"`.
