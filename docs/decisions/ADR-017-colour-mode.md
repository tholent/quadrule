---
status: proposed
status-note: drafted 2026-11-18 by an outside contributor from their bbq answers (docs/bbq/2026-11-18-colour-mode.md); awaiting chris's ruling
owner: chris
kind: design
built: not started
proof: "rung C1 in `docs/bbq/2026-11-18-colour-mode.md` (Kickoff), on the contributor's fork, branch `colour-mode`"
supersedes: nothing
amends: "napkin §1.2 (the \"Color e-ink devices\" non-goal, lifted for the `color` build only) and §11.1 (colour profiles export RGB PNG)"
consulted: an outside contributor who owns a BOOX Note Air5 C (the proposer); Opus 5.5 ran the bbq
---
# ADR-017 — A flagged colour mode: a page is either gray or colour, colour pages target colour devices only, keep sharp black at full resolution with colour snapped to the colour cells, take their colours from a palette measured on each device, and leave gray mode byte-for-byte unchanged

## Decision

**Proposed by an outside contributor; nothing here binds until chris rules on it.** Behind a Cargo feature named
`color`, off by default (Q9), quadrule gains a second page mode for colour e-ink screens such as Kaleido 3:

1. **Two modes.** A page is in gray mode (today's quadrule) or colour mode (Q2). A page can switch both ways (Q8):
   colour to gray keeps every role's own gray (Q4), gray to colour starts with no tints.
2. **Colour pages target colour devices only** (Q6). Gray pages keep targeting the gray devices (Manta, Nomad).
3. **Tints, not free colour.** An object in a colour page may carry an optional `tint` that names a colour from the
   device's palette (Q2, Q3, Q13). Its gray twin is always the role's own gray (Q4).
4. **Palettes are measured on the device** and kept in its profile, like gray levels (Q3, extending ADR-012 item 2).
   A device's first palette is the pen colours of its own drawing app, read from a screenshot of one stroke per pen,
   marked ESTIMATE until a colour calibration sheet replaces it (Q11).
5. **Full resolution, colour on the colour cells.** A colour page exports at the panel's full black-and-white
   resolution so black lines and text stay sharp; coloured fills and lines snap to the colour layer's cells (2 × 2
   pixels on a 300/150 PPI Kaleido 3 panel) so they do not fringe (Q7).
6. **Export.** Colour profiles export 8-bit RGB PNG; gray profiles keep 8-bit grayscale (Defaulted).
7. **File format 0.18.0.** `tint` is an optional field. A gray-mode page saves with no tint field, so its bytes do not
   change; an older quadrule keeps an unknown tint when it resaves (Q13, ADR-001 item 4).
8. **Two lint rules**, recommending and never blocking: a coloured line narrower than one colour cell, and coloured
   text below a size the profile sets (Q14).
9. **Gray mode untouched** (Q12). Without the feature, and in gray mode with it, every export, file and screen is
   byte-for-byte today's: `crates/render/tests/golden/SHA256SUMS` does not change.

## Context

- Napkin §1.2 (`napkins/quadrule-spec.md:36`) lists "Color e-ink devices" as a non-goal "for now", and adds "The
  profile system should not prevent adding them later." This ADR proposes lifting that non-goal **only** for the
  `color` build, and only if chris ratifies (Q15). The spec text is not edited.
- **How the screen works.** Kaleido 3 is a passive colour filter laid over an ordinary Carta black-and-white panel:
  black-and-white detail is 300 PPI and colour detail 150 PPI, on the same page at the same time
  (blog.the-ebook-reader.com, 2023-06-08, "Color E Ink Kaleido screens are black and white Carta screens";
  trustedreviews.com, "What is E Ink's Kaleido colour tech"). So showing colour does not halve the sharpness of
  black lines; only coloured areas are coarse. This is why Q7 keeps full resolution.
- **The proposer's device.** BOOX Note Air5 C: 1860 × 2480 at 300 PPI, colour layer 930 × 1240 at 150 PPI (BOOX shop
  pages read through search, cross-checked with Notebookcheck's Note Air4 C test, which has the same screen; to be
  checked against the device). It is not a built-in profile; the proposer keeps it as a user profile (ADR-016).
- **Who can test.** chris has no colour tablet; the proposer tests on the device, and chris's look check (ADR-007)
  covers the editor only (Q10).
- ADR-007's flag rule applies: this changes export bytes and the file format.

Drivers:
- one design must not fork into two documents; a page should move between gray and colour devices (Q8);
- black lines and text are the heart of a template and must stay sharp (Q7);
- quadrule's values come from the device, never from taste in code (ADR-012);
- nothing in chris's gray pipeline may move (Q12).

## Alternatives

### Two page modes, tints from a measured palette, full resolution with colour on the colour cells (chosen: keeps black sharp and gray mode untouched, and follows ADR-012's measured-values rule)

- Good: gray pages, files and golden images stay byte-for-byte the same; colour pages get the panel's full sharpness
  for black; every colour on offer is one the device shows cleanly.
- Bad: a second mode to build and explain; a palette must be measured per device before it can be trusted; snapping
  to colour cells is new layout work.

### Tint on roles in every page, gray and colour devices together (the Q2 recommendation, overruled)

- Good: one kind of page for every device.
- Bad: colour devices have a colour layer at a different resolution, so a page laid out for both serves neither well
  (the proposer's Q2 answer).

### Free colour per object, or a fixed shared palette (Q2 and Q3 options)

- Good: familiar from drawing apps; no measuring needed.
- Bad: Kaleido colour is muted and only some hues show cleanly, so unmeasured colours look wrong on the device.

### Design and export at the colour resolution (Q7 option)

- Good: simplest; no snapping.
- Bad: throws away the sharp 300 PPI black layer for every line and letter.

### A separate colour layer file in the `.quad` ZIP (Q13 option)

- Good: the design record never changes.
- Bad: the two can drift apart; an optional field already keeps old files' bytes unchanged.

## Consequences

- Easier: templates for BOOX colour tablets, and later Kindle Scribe Colorsoft, Kobo Libra Colour and others.
- Harder: every rung that touches export, layout or lint must also pass with `--features color`; CI would need a
  second build of the colour feature if chris folds it in.
- Commits the proposer to: a colour calibration sheet (`cargo xtask calibration-sheet` gains colour patches), a
  colour profile field set (`color: Some(...)`, Defaulted), and device measurements kept in
  `docs/device-measurements/<id>/` like the Supernotes'.
- Build steps for rung C1, each with its test:
  1. `color` feature in the workspace; a test that a default build exports today's golden bytes.
  2. Profile `color` fields (cell size, palette, lint numbers); a parse test, and a test that gray profiles' RON is
     unchanged.
  3. `tint` in the model, format 0.18.0; a round-trip test, and a test that a gray page's saved bytes match 0.17.
  4. Page mode switch, both ways; a test that colour → gray → colour keeps tints and gray twins.
  5. Colour-cell snapping; a test that every coloured edge lands on a cell boundary.
  6. RGB export for colour profiles; colour golden images of their own.
  7. The colour calibration sheet; a test that its patches sit on whole cells.
  8. The two lint rules; one `CHECK` line each (ADR-006).

Confirmed by: the proposer's look on a real Kaleido 3 device (colour fringing and palette), since chris has no
colour tablet.

## Asks left open

1. Will chris lift the napkin §1.2 non-goal for the `color` build at all?
2. The words on screen: "Gray" / "Colour", or "Color" to match the spec (Defaulted in the record; naming is chris's).
3. Should colour mode ever become a default build, or stay a feature for good?

## History

- 2026-11-18: drafted as `proposed` by an outside contributor from the ADR bbq record
  `docs/bbq/2026-11-18-colour-mode.md` (Q1–Q15), on the contributor's branch `colour-mode`; not yet sent to chris.
- 2026-11-18: chris agreed to a pull request before the build (relayed by the proposer), so this proposal goes to
  him from the proposer's fork now rather than once rung C1 works (changes Q5's timing only).
