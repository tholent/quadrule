# Talk: file version label and text hinting

**Status: open: nothing here is ruled unless marked agreed.** Agreed in talk is not ratified; the format ADR
(proposed by the format spike, `spikes/format/ADR-PROPOSAL.md`) is where these get ruled.

2026-11-09 · text · chris and Claude (Opus 5.5) · Produced by talk-through.

## Context

The format spike parked two questions for chris (`spikes/format/FINDINGS.md`, "Parked for chris"; plan:
`docs/bbq/2026-11-09-quadrule-spikes.md`):
1. When an older Quadrule saves a newer file, which version label does the file carry?
2. Must the saved text outlines match each device's pixels (hinting), or are plain shapes enough?

## Key points

- Resaving a newer file in an older build only happens for "optional extras": the spike carries unknown optional
  fields through a save (check F05), and files needing an unknown required feature open read-only (F06).
- Stamping the older build's version would make the newer app re-run its upgrade step on return, which could
  overwrite extras that were carried through safely. Keeping the file's own label hides that an older app touched
  it. Recording both avoids both problems.
- The napkin draws text as plain outlines for screen and export (`napkins/quadrule-spec.md:349`). If live text is
  never hinted, saved plain shapes with per-device positions reproduce the export exactly.
- Hinting depends on pixels per letter height (font size × device PPI), so hinted shapes differ per device and per
  size. Hinting needs the font's own instructions, so it cannot be applied to saved shapes later: if a device is
  hinted, its hinted shapes must be saved.
- Light (vertical-only) hinting snaps heights and horizontal strokes to pixel rows and leaves widths alone, so line
  breaks stay identical across devices (text spike check T03). Full hinting changes widths and can change line
  breaks per device, which fights "design once in mm".
- skrifa has a hinting engine (TrueType instructions and an autohinter). **Unverified:** not tested in this spike.
- The napkin's primary/secondary profile list (`napkins/quadrule-spec.md:138`) already decides which devices a
  document saves outlines for (`:584`). A separate single/multi-profile file mode adds nothing.
- Edge case: with a font missing, adding a new profile has no saved outlines for it. Plain shapes from another
  profile can be reused when the box is the same size (T03); hinted shapes, or boxes resized by constraints, cannot.

## Decisions

| decision | status | where it came up |
|---|---|---|
| **D1** A resaved file keeps its own `format_version`; the manifest also records which app version last saved it, so a newer app knows an older one touched it and can re-check derived data. | agreed | chris: "I like your recommendation on this. it feels like an elegant degredation" |
| **D2** No single-profile / multi-profile file mode. Outlines are saved for exactly the document's profiles (primary + secondaries). | agreed | chris raised the mode idea ("I'm not sure this get's us much with how much work it might entail"); agreed: "that sits right" |
| **D3** Saved text outlines are plain (unhinted) shapes with per-profile positions, unless that profile has hinting on. | agreed | talk on question 2; "that sits right" |
| **D4** Text hinting is a per-profile setting in the document: off / light. Light means vertical-only. Full hinting is not offered. | agreed | chris: "let's talk about how hinting could be enabled"; "that sits right" |
| **D5** Hinting defaults to off and lands behind a feature flag (AGENTS.md: "Land a new feature switched off behind a flag"). The calibration sheets (napkin §8.4) show hinted and plain text side by side, and the default is chosen from real hardware. | agreed | "that sits right" |
| **D6** A test renders live text and text from saved outlines and requires byte-identical PNGs, per profile and hinting mode, so the two can never drift apart unnoticed. | agreed | "that sits right" |
| **D7** Adding a profile while a font is missing: reuse plain shapes from another profile where the box is the same size; otherwise show the text badged "needs the font" until it is found. | agreed | "that sits right" |

## Open questions

1. **Ratify into the format ADR.** These decisions are agreed in talk, not ratified. Recommendation: fold D1–D7 into
   the format spike's ADR proposal before chris ratifies it.
2. **Is skrifa's light hinting good enough on e-ink?** Recommendation: test it on the calibration sheets once the
   Manta and Nomad are calibrated (napkin §8.4), not before.

## Action items

- **Fold D1–D7 into `spikes/format/ADR-PROPOSAL.md`.** Done when the proposal's text names each and cites this record.
- **Add a hinting check to a future spike or the real `text` crate:** skrifa light hinting at 226 and 300 PPI.
  Done when a CHECK line shows hinted outlines extracted and line breaks unchanged against plain text.
- **Add hinted vs plain text rows to the calibration sheet spec** (napkin §8.4). Done when the sheet generator's
  task lists them.

## Corrections

- None. The talk confirmed that the profile list already does what a single/multi-profile mode would.

## Possible next steps

- `makeadr`: ratify the format ADR with D1–D7 folded in, once chris says "ratify".
- `looseends`: pick up open question 2 after calibration.
