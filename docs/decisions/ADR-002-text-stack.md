---
status: accepted
status-note: ratified by chris 2026-11-09 ("ratify all three")
ratified-by: chris
ratified-on: "2026-11-09"
owner: chris
kind: architecture
built: partial
proof: "`crates/text/tests/text.rs::title_lays_out_with_no_missing_glyphs`, `crates/text/tests/text.rs::bold_draws_bolder_shapes_and_wider_text`, `crates/render/tests/slice.rs::slice_end_to_end`"
owed: "hinting flag (item 4); fallback to any installed font with its save and export warning (item 6); substitution by font signature (item 7); re-scan on focus (item 8); paragraph spacing, vertical alignment and the overflow signal (item 3)"
supersedes: nothing
amends: nothing
consulted: text spike worker (Opus 5.5 subagent); talk-through with Claude (docs/proposals/talk-2026-11-09-format-versions-and-hinting.md)
revisit-when: the owed macOS and Windows text checks (spike T14, T15) run, or a machine with thousands of fonts makes the full re-scan slow
---
# ADR-002 — Text uses cosmic-text alone, laid out in millimetres, drawn as plain outlines, with fallback to any installed font

## Decision

1. **One dependency.** The `text` crate depends on `cosmic-text` only and reaches fontdb and skrifa through
   `cosmic_text::fontdb` and `cosmic_text::skrifa`, with the default `swash` feature switched off.
2. **Layout in millimetres.** Text is laid out once in mm and positions are scaled to each profile's pixels
   afterwards.
3. **Outlines.** skrifa draws each glyph at the variable-font weight, placed at cosmic-text's positions, so kerning
   carries into the export. Paragraph spacing, vertical alignment in fixed boxes and the overflow signal are
   quadrule's own small additions.
4. **No hinting by default.** Live text and saved outlines are plain (unhinted) shapes. Text hinting is a
   per-profile setting in the document, **off / light** (light = vertical-only; full hinting is not offered), off
   by default and behind a feature flag. Its default is chosen from hinted-vs-plain rows on the calibration sheets
   (napkin §8.4) on real hardware. (talk D4, D5)
5. **Identity test.** A test renders live text and text from saved outlines and requires byte-identical PNGs, per
   profile and hinting mode. (talk D6)
6. **Fallback.** A character missing from the chosen font may come from **any installed font**. When system fonts
   are used, a check warns **both when the `.quad` is saved and in export preflight**. (chris's ruling)
7. **Substitution** for a missing font uses a **measured font signature** (stored per ADR-001): an exact family
   name wins first; otherwise class (serif, sans, mono, script) from measurement, then the font's labels, then its
   name; plus weight, width, italic, average letter width and x-height.
8. **Re-scan** system fonts fully when the app regains focus; no change detection.

## Context

The napkin names fontdb, cosmic-text and skrifa (`napkins/quadrule-spec.md:645`, `:655`) and wants text identical
everywhere (`:349`). The text spike (`spikes/text/FINDINGS.md`) found on Linux: font scan about 5 ms (T01–T02);
the same line breaks in mm and at 300 and 226 PPI (T03); justify, letter spacing, line height and variable weight
work (T04–T07); outlines placed at layout positions agree to under 0.4 font units (T08–T10); substitution by
measurement picks what a typographer would (T13). Listing the three crates separately pulled in two fontdb and
three skrifa versions. Automatic emoji fallback picked a system font (T12a), which is why the fallback rule was
put to chris.

Drivers:
- One version line to follow, not three drifting ones.
- "Design once in mm" across devices.
- Live and saved text must never quietly differ.

## Alternatives

### A: cosmic-text with its re-exports, swash off (chosen: one version line, already proven)
- Good: done in the spike; upgrading moves all three together.
- Bad: three small layout pieces to write ourselves.

### B: the three crates as separate dependencies (napkin §15.2 as written)
- Good: matches the napkin's list.
- Bad: duplicate versions today, type mismatches whenever they drift.

### C: Parley
- Good: may fit vello more closely.
- Bad: not tested; would need its own spike.

### D: our own line breaking on the harfrust shaper
- Good: full control.
- Bad: weeks of work; every line-break and bidirectional-text bug is ours.

### Fallback: bundled fonts only
- Good: identical on every machine.
- Bad: fewer characters covered. Not chosen by chris.

### Full hinting
- Good: crisper small text.
- Bad: changes letter widths, so lines can wrap differently per device.

## Consequences

- Upgrading cosmic-text is the one place text dependencies move.
- System-font fallback means output can differ between machines; the save and export warnings, and the saved
  outlines (ADR-001), are the guard.
- A new lint/preflight check for system-font use, at save as well as export, is owed to the napkin's lint list.
- macOS and Windows behaviour is unproven until T14 and T15 run.

## History

- 2026-11-09: drafted by the text spike worker as `spikes/text/ADR-PROPOSAL.md` (Status: PROPOSED, waiting for
  chris to ratify). Before ratifying, chris ruled on fallback ("any installed fonts. pre-flight for saving should
  warn when system fonts are being used"; then "both, saving and export") and the font signature ("this sounds
  likea good idea"), and agreed talk decisions D4–D6; items 4, 5 and 6 of the Decision are those folds. Prior
  header values: status `proposed`, status-note `drafted 2026-11-09 from spikes/text/ADR-PROPOSAL.md, chris's
  rulings and the talk record; awaiting ratification`.
- 2026-11-09: ratified by chris ("ratify all three").
- 2026-11-09: built status updated after the real `text` crate; the live-vs-saved identity test (item 5) is in the slice test (rung 1). Prior header values: built `not started`, proof
  `queued in STATUS.md (Next) for the real text crate; evidence in spikes/text/FINDINGS.md`.
