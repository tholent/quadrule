# Starters against the linter: where chris's starters and the rules disagree (2026-11-23)

chris, word for word: "I've been working on a few starters, but they violate some linting rules. I think we need to
step through a few of these files and create/amend ADRs to address the discontinuity. I believe we should start with a
few "papers" templates, as the bulk of the issues will crop up in looking at these."

Each starter is linted with `cargo xtask lint <file>` (every toolbar position, both devices); its findings are grouped
by rule, and each disagreement is ruled as: the starter changes, the rule changes (an ADR), or the starters' lint-clean
rule changes (spec §13, arc board row 16 and Q18).

## Premises

| Premise | Checked |
|---|---|
| Starters must pass lint with no rhythm warnings | `napkins/quadrule-spec.md` §13 (line 644); `docs/arcs/v1/README.md` row 16 and the release bar; `docs/arcs/v1/rung-16b-PROMPT-2026-11-18.md` "Each starter" |
| Starters use line roles | same three places |
| Supernote's stock templates run under the toolbar and footer | `docs/device-measurements/README.md:59` ("Full bleed") |
| L06 checks every toolbar position and the footers | ADR-012 item 5 |
| L10 judges the object's box with half its stroke | `crates/lint/src/lib.rs` (`LaidOut` box; L10 block) |
| L16 judges position on the primary device only; Free placement opts out | `crates/lint/src/lib.rs` (L16 block, `free_placement`); spec §8.3 |
| Role values come from calibration; seven roles | ADR-012 items 1 and 2; `crates/profiles/data/supernote.a6x2.ron` `line_roles` |
| Free placement is a field on each object | `crates/model/src/lib.rs` (`free_placement`) |

## Starter 1 and 2: `narrow_ruled.quad`, `narrow_ruled_rtl.quad` (Paper)

Nomad primary, both tablets; rhythm 6.35 mm (¼″), centred; page margins 12.7 mm. A Lines pattern region (Guide role)
edge to edge across, from 12.53 mm to the bottom edge; a full-height margin line (custom: 0.45 mm, gray 204) at
18.12 mm on the Nomad and 24.13 mm on the Manta (a per-device position), mirrored to the right in the RTL copy.

`cargo xtask lint` → `11 finding(s) on 2 profile(s)` for each file:

| Rule | Severity | Object | Devices |
|---|---|---|---|
| L06 under a safe zone | Warn | Lines (footers, left and right toolbars); Margin (a footer, top toolbar) | both |
| L10 off the page | Info | Lines; Margin | both |
| L02 stroke not whole pixels (5.31 px drawn 5) | Info | Margin | both |
| L16 off the rhythm (down) | Warn | Margin | Nomad (the primary) |

## Questions and rulings

**A. Full bleed.** "How should a deliberately full-bleed object (lines under the toolbar and footers, like
Supernote's stock templates) meet L06 and L10?" chris: "A Full bleed mark (Recommended)". The option as offered: a
tick per object in the Arrange tab, like Free placement, that L06 and L10 skip; and L10 stops counting a line's
half-thickness at an edge it only reaches. Not taken: Ignore rules per document; keep inside safe zones; exempt
starters from L06/L10. → ADR-019.

**B. Page edges and the rhythm.** "The margin line starts at the page's top edge, which is off the rhythm (L16). What
should count?" chris: "Page edges count as on rhythm (Recommended)", for every page, not just starters. Not taken: the
Full bleed mark covers L16; Free placement on the line. → ADR-019.

**C. The margin line.** "The margin line is a wide, light line (0.45 mm, gray 204), which no line role covers. What
should a starter's margin line be?" chris: "A new Margin role (Recommended)". Not taken: custom values in whole
pixels; an existing role. → ADR-020.

## Owed

- ~~ADR-019 and ADR-020 wait for chris's ratification~~ ratified 2026-11-23, chris: "ratify 019 and 020". Their build is
  queued before rung 16b.
- The Margin role's calibration on both tablets (ADR-020).
- More Paper starters, then the other categories, stepped through the same way.

## Defaulted (the lead's calls; each can be vetoed)

| Call | Why | veto: |
|---|---|---|
| A and B in one ADR (ADR-019), C in its own (ADR-020) | A and B change the same rules for the same pages; C is a profile and calibration change | veto: one ADR each, or all three in one |
| The Full bleed mark is inherited by what is inside a marked group or repeat, as Free placement is | the same as the mark it copies | veto: per object only |
| Page edges count on the near and the far edge of each axis (an object flush right or flush bottom passes too) | a page edge is an edge, whichever side | veto: the top and left edges only |
| Until calibrated, the Margin role takes chris's drawn values: 5 px (0.45 mm at 300 PPI, whole pixels) and gray 204, marked chosen, not measured | ADR-012's owed list already carries "chosen, not measured" values | veto: calibrate before it ships |
