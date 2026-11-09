# PROPOSED: text stack for quadrule

Status: **PROPOSED**, waiting for chris to ratify. Not in `docs/decisions/` yet; makeadr files it once ratified
(its format is still unchosen; `docs/decisions/README.md`).
Date: 2026-11-09 · Evidence: `spikes/text/FINDINGS.md` · Napkin: §7 (`napkins/quadrule-spec.md:342-389`),
§15.1–15.2 (`:645`, `:655`).

## Context

The napkin names fontdb (font discovery), cosmic-text (shaping and wrapping) and skrifa (outlines) for the `text`
crate. The spike checked that these three work together on Linux for what §7 needs. Text must be laid out in
millimetres, turned into vector outlines, and kept identical across devices. When a font is missing, the app must
find a sensible substitute.

## Options, weighed by lifetime cost

| option | what it costs now | what it costs later |
|---|---|---|
| **A. cosmic-text 0.19, using its re-exported fontdb and skrifa, swash off** (napkin's choice, tidied) | Done in the spike. Three small layout pieces to write ourselves (paragraph spacing, vertical alignment, overflow), each a few lines. | One version line to follow: upgrading cosmic-text upgrades all three together, so they never split. |
| B. Same three crates as separate, independently versioned dependencies (napkin §15.2 as written) | Same code | Two fontdb and three skrifa copies today: bigger, slower builds, and type mismatches whenever they drift. Ongoing upkeep. |
| C. Parley (Linebender's layout crate, a sibling of vello) | **Not tested in this spike.** It would need its own spike. | Might fit vello more closely. Unknown until measured. |
| D. harfrust shaper plus our own line breaking | Weeks of work | All bidirectional-text, line-break and justification bugs are ours to keep. |

## Recommendation

**Option A.** In detail:

1. Depend on `cosmic-text` only. Reach fontdb and skrifa through `cosmic_text::fontdb` and `cosmic_text::skrifa`.
   Switch off the default `swash` feature.
2. Lay text out in millimetres (layout units = mm). Scale positions to each profile's pixels afterwards. The spike
   showed line breaks do not change with scale (T03).
3. Make outlines with skrifa: unhinted, at the variable-font weight, placed at cosmic-text's glyph positions. That
   way kerning carries into the export (T08–T10).
4. Pick substitutes by a **measured font signature**: class (serif, sans, mono, script) from measurement, then the
   font's labels, then its name; plus weight, width, italic, average letter width and x-height. An exact family
   name wins first. Stored per font in the document (see FINDINGS "Parked" 2).
5. Re-scan system fonts fully when the app regains focus; at about 5 ms here, no change detection is worth
   building. Re-check this on a machine with thousands of fonts.

Not covered by this decision, and still open for chris: where font fallback may look (FINDINGS "Parked" 1), and
macOS/Windows proof (T14, T15, owed).
