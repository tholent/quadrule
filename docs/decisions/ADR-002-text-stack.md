---
status: accepted
status-note: ratified by chris 2026-11-09 ("ratify all three")
ratified-by: chris
ratified-on: "2026-11-09"
owner: chris
kind: architecture
built: partial
proof: "`crates/text/tests/text.rs::title_lays_out_with_no_missing_glyphs`, `crates/text/tests/text.rs::bold_draws_bolder_shapes_and_wider_text`, `crates/render/tests/slice.rs::slice_end_to_end`, `crates/text/tests/fonts.rs::rescan_reads_the_fonts_again_and_keeps_the_open_ones`, `crates/text/tests/textbox.rs::a_fixed_box_leaves_out_whole_lines_that_do_not_fit`, `crates/text/tests/textbox.rs::paragraph_spacing_goes_after_each_paragraph_but_the_last`, `crates/text/tests/missing.rs::an_installed_font_is_opened_as_fallback_for_a_letter_the_bundled_fonts_lack`, `crates/text/tests/missing.rs::the_bundled_fonts_are_classed_by_measurement`, `crates/text/tests/missing.rs::a_substitute_is_picked_by_signature`, `crates/lint/tests/lint.rs`"
owed: "hinting flag (item 4; moved to v1.1, amendment 2026-11-16)"
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
- 2026-11-15: amended by chris during rung 11a: `unicode-script` is allowed as a second direct dependency of the
  text crate, for one purpose (see Amendment below). Header values unchanged.
- 2026-11-16: built status updated after rung 11 (11a and 11b, approved by chris): installed fonts on all three
  systems with the full re-scan on focus, run in the background (item 8); paragraph spacing, vertical alignment and the
  overflow signal (item 3). Prior header values: proof ``crates/text/tests/text.rs::title_lays_out_with_no_missing_glyphs`, `crates/text/tests/text.rs::bold_draws_bolder_shapes_and_wider_text`, `crates/render/tests/slice.rs::slice_end_to_end``; owed `hinting flag (item 4); fallback to any installed font with its save and export warning (item 6); substitution by font signature (item 7); re-scan on focus (item 8); paragraph spacing, vertical alignment and the overflow signal (item 3)`.
- 2026-11-16: amended by chris in the rung 12 bbq (Q2): the hinting flag (item 4) moves out of v1 to v1.1 (see
  Amendment — 2026-11-16 below). Prior header values: owed `hinting flag (item 4); fallback to any installed font with its save and export warning (item 6; rung 12, until then bundled fonts only, rung 11 bbq Q4); substitution by font signature (item 7; rung 12)`.
- 2026-11-17: built status updated after rung 12 (approved by chris): fallback to any installed font with its save and
  export warning (item 6: lint L20 and the Save/Export sheet) and substitution by measured font signature (item 7).
  Prior header values: proof "`crates/text/tests/text.rs::title_lays_out_with_no_missing_glyphs`, `crates/text/tests/text.rs::bold_draws_bolder_shapes_and_wider_text`, `crates/render/tests/slice.rs::slice_end_to_end`, `crates/text/tests/fonts.rs::rescan_reads_the_fonts_again_and_keeps_the_open_ones`, `crates/text/tests/textbox.rs::a_fixed_box_leaves_out_whole_lines_that_do_not_fit`, `crates/text/tests/textbox.rs::paragraph_spacing_goes_after_each_paragraph_but_the_last`"; owed "hinting flag (item 4; moved to v1.1, amendment 2026-11-16); fallback to any installed font with its save and export warning (item 6; rung 12, until then bundled fonts only, rung 11 bbq Q4); substitution by font signature (item 7; rung 12)".

## Amendment — 2026-11-15, chris

**What changed.** Item 1 ("The `text` crate depends on `cosmic-text` only") now allows one more direct dependency,
**`unicode-script`**, used only to name the `Script` type in cosmic-text's `Fallback` trait, and declared at the
version cosmic-text itself uses (`unicode-script = "0.5.9"` in the workspace `Cargo.toml`, with a comment saying why;
0.5.9 in `Cargo.lock`, one copy shared with cosmic-text). fontdb and skrifa still come only through cosmic-text's
re-exports, and swash stays off.

**Why.** Rung 11's bbq Q4 (`docs/bbq/2026-11-14-rung-11-text-and-fonts.md`) rules that a character missing from the
chosen font comes from the bundled fonts only until rung 12. cosmic-text 0.19 lets quadrule choose fallback fonts
through its `Fallback` trait, whose methods name `unicode_script::Script`. cosmic-text does not re-export that crate,
so the trait cannot be filled without depending on it directly. The risk item 1 guards against, copies drifting apart
silently, cannot happen here: Cargo shares one 0.5.x copy with cosmic-text, and if cosmic-text moves to another
version the trait's type no longer matches and the build fails. The lead put the choice to chris: amend this ADR, or
have the builder look for a way round it (which might not exist). After the lead explained it in plain words, chris
answered: "amendment, go ahead".

**Touches.** Decision, item 1. Built in rung 11a (`crates/text/src/registry.rs`, `Cargo.toml`). Revisit when
cosmic-text re-exports `Script` or changes its `Fallback` trait: then the direct dependency should go.

## Amendment — 2026-11-16, chris

**What changed.** Item 4 (the off/light text hinting setting per profile, behind a feature flag, its default chosen from
hinted and plain rows on the calibration sheets) is no longer part of v1.0: it moves to **v1.1**. Until then plain
(unhinted) outlines are the only kind, live and saved, so ADR-001 item 3's hinted outline tables wait with it. Item 5's
identity test covers the plain mode only until then. The rest of item 4 stands as ruled for when it is built.

**Why.** It was owed and not on the arc v1 board; the rung 12 prompt asked whether it belonged in rung 12. The lead's
reasons for v1.1: every golden page and both tablets' calibration so far use plain text and nothing has looked wrong
on the devices, and choosing the default needs chris's time on hardware. Asked in the rung 12 bbq, Q2
(`docs/bbq/2026-11-16-rung-12-missing-fonts.md`), chris chose "Out of v1, to v1.1 (Recommended)".

**Touches.** Decision, item 4; the `owed` header. Revisit when v1.1 is planned.
