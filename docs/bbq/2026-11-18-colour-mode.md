# bbq record — quadrule: colour mode (an outside proposal)

2026-11-18 · ADR bbq · 15 questions (12 asked, 3 owed then asked) · grilled: the proposer, an outside contributor; owner of
any ruling: chris · lead for the build: Opus 5.5 at high effort (Defaulted).

This is **not** chris's work and rules nothing. It is a proposal the proposer is scaffolding on their own fork so that chris
can fold it in or not. Every rule below is the proposer's answer; it becomes binding only if chris ratifies the ADR it feeds.

Grounding: napkin §1.2 (`napkins/quadrule-spec.md:36`, "Color e-ink devices" a non-goal, "The profile system should
not prevent adding them later"); napkin §11.1 (8-bit grayscale PNG export); ADR-001 (format: unknown fields kept,
newer files read-only); ADR-004 (agents propose, chris ratifies); ADR-007 (export bytes and format changes land
behind a flag); ADR-012 (role values come from calibration and live in profile data); ADR-016 (user profiles);
`crates/model/src/lib.rs` (`Fill`, `FillRole`, `LineRole`); `crates/format/src/lib.rs` (`FORMAT_VERSION` 0.17.0);
`crates/render/tests/golden/SHA256SUMS`. Screen facts: Kaleido 3 is a colour filter over a 300 PPI Carta panel,
black-and-white detail 300 PPI and colour 150 PPI at once (the-ebook-reader.com 2023-06-08,
trustedreviews.com Kaleido explainer). The proposer's device: BOOX Note Air5 C, 1860 × 2480 at 300 PPI, colour layer
930 × 1240 (user profile `boox.noteair5c`, figures from BOOX's shop pages via search; re-check before shipping).

## Rules

- **Q1 Count.** 12 questions; three more were owed and asked (Q13–Q15).
- **Q2 Two modes.** Colour is a separate page mode, not a tint sprinkled on gray pages: colour e-ink devices are a
  different set of devices, with a colour layer at a different resolution from their black-and-white one.
- **Q3 Palette.** The colours a device offers are measured on that device and kept in its profile, as gray levels
  are today (ADR-012's rule extended to colour).
- **Q4 Gray twin.** When a colour page is shown or exported in gray, each coloured role keeps the role's own gray,
  exactly as today, so gray exports never change by a byte.
- **Q5 Hand-off.** Work on a branch on the proposer's own fork and keep it there for now; open an issue and a pull request on
  chris's repository only once there is a workable implementation.
- **Q6 Targets.** A colour-mode page targets colour devices only. Gray pages stay exactly as they are (Manta and
  Nomad).
- **Q7 Resolution.** A colour page exports at the panel's full black-and-white resolution (1860 × 2480 on the Air5 C),
  so black lines and text stay sharp; coloured areas and lines snap to the colour layer's 2 × 2 pixel cells so they
  do not fringe. The proposer: "as long as we ensure compatibility" (see Q12 and Q13).
- **Q8 Switching.** A page can switch between gray and colour mode both ways. Colour to gray uses Q4's gray twins; gray
  to colour starts with no tints.
- **Q9 Flag.** A Cargo feature named `color` (the code's own spelling), off by default. Without it the app builds
  byte-for-byte as today; chris turns it on with one build option.
- **Q10 First rung.** Colour mode on a page, tints from the device palette, RGB PNG export for colour profiles, **and**
  a colour calibration sheet. The proposer tests on the device (chris has no colour tablet); chris's look check covers the
  editor only.
- **Q11 Palette seed.** The starting palette is the pen colours of BOOX's own drawing app (6 to 10), read from a
  screenshot of one stroke per pen colour, the way the Nomad's `ink_values` were read; marked ESTIMATE until the
  colour calibration sheet replaces it.
- **Q12 Protect.** Gray mode is untouched: without the flag, and in gray mode with it, every export, file and screen
  is byte-for-byte today's.
- **Q13 File format.** Each object gets an optional `tint` naming a palette colour; format 0.18.0. A gray-mode file
  saves with no tint field at all, so its bytes do not change, and an older quadrule keeps the tint when it resaves
  (ADR-001's unknown-field rule).
- **Q14 Lint.** Two new rules with the first rung: a coloured line under 2 px (one colour cell) and coloured text
  below a size the profile sets. Both recommend and explain, never block.
- **Q15 Non-goal.** The ADR proposes lifting napkin §1.2's "Color e-ink devices" non-goal for the flagged build only,
  and only if chris ratifies; without the flag the non-goal stands. The proposer never edits the spec text.

## Owed

None (Q13–Q15 were asked).

## Defaulted

- **Words on screen.** The page-mode switch says "Gray" and "Colour"; every word goes through Fluent (ADR-008).
  veto: naming is chris's; he may want "Color" to match the spec.
- **Palette size limit.** A profile lists at most 16 palette colours, each an sRGB value with a name.
  veto: the limit and the colour model.
- **Colour profile fields.** A colour profile adds `color: Some((cell_px: 2, palette: [...], lint: (...)))`; gray
  profiles have `color: None` and their RON files do not change. veto: the field layout.
- **Export format.** Colour profiles export 8-bit RGB PNG (no alpha); gray profiles keep 8-bit grayscale.
  veto: whether BOOX needs anything else (a specific folder, file naming).
- **Golden images.** Colour export gets its own golden images; `SHA256SUMS` for gray pages must not change, and a test
  says so with one `CHECK` line (ADR-006). veto: none expected.
- **Lead.** The build session is led by Opus 5.5 at high effort. veto: model and effort.

## Kickoff

After the ADR is written (makeadr) on branch `colour-mode`: a fresh session reads this record and the proposed ADR,
builds rung C1 (Q10) behind the `color` feature on that branch, runs `make check-all` with and without
`--features color`, and stops for The proposer's device test. It pushes only to the proposer's fork, never to `tholent/quadrule`.
Lead: Opus 5.5 at high effort; a session on another model or effort stops and says so first.
