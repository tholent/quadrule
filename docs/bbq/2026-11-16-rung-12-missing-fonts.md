# bbq record — quadrule rung 12: missing fonts (12a) and pattern rotation (12b)

2026-11-16 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-12-PROMPT-2026-11-16.md`) · 9 questions (the
arc's "about 8", plus one for pattern rotation, unparked by chris at session start) · lead: Opus 5.5 at high effort;
builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 12 row), `docs/arcs/v1/rung-12-PROMPT-2026-11-16.md`,
`docs/decisions/INDEX.md`, ADR-001 and ADR-002 in full, `docs/bbq/2026-11-14-rung-11-text-and-fonts.md` (Q4: bundled-only
fallback until rung 12), `napkins/quadrule-spec.md` §7.2, §7.3, §8.3 (L11), §12.1 (`assets/fonts/<id>.ttf`, "only fonts
whose license allows embedding"), `docs/STATUS-debt.md` (rung 3, 6b, 7a, 11a, 11b lines), `spikes/text/FINDINGS.md`
(T13, substitution by measurement), `crates/text/src/registry.rs` (`FontRegistry::bundled_only`, `with_system`),
`crates/text/src/lib.rs:282` (`TextError::MissingFamily`), `crates/ui/src/editor.rs:1149` (Save keeps a missing font's
saved outlines), `crates/ui/src/editor.rs:1221` (`font_refs`), `crates/format/src/lib.rs:48` (`FORMAT_VERSION` 0.12.0),
`crates/model/src/lib.rs:396` (`rotation_deg`, ignored on pattern regions by `Object::rotation`), `crates/lint/src/lib.rs`
(rules up to L19; L20 is free).

Measured at the start of this session (2026-11-16 on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=7ff14c3 dirty=no rebuilt=11`. Embedding bits (the OS/2 table's `fsType`), read by a small script over
`fc-list --format '%{file}\n' | sort -u` (181 files) and `crates/text/fonts/*.ttf`: bundled 10 of 10 installable;
installed 75 installable, 1 editable (DroidSansFallbackFull), 35 preview-and-print only (the URW C059 family and kin,
`.otf`), 35 Type 1 `.pfb` files with no OS/2 table, 35 `.t1` files unreadable as OpenType. macOS and Windows fonts were
not measured (no machine here).

Asked again at session start (AGENTS.md): the three parked v1.1 questions. chris: the A5 X rhythm and the Label gray
stay parked; **pattern rotation is unparked** ("pattern rotation"). It is not in the board's rung 12 row, so this
grows the rung (said before Q1 was asked); Q1 placed it.

## Budget
Share: 12 points (1 point = 1 % of the weekly limit), absolute allocation by chris ("12 points (Recommended)"), alone on
the meter, reserve 0.
Room: 12 points (the hard stop, 75 %, is 53 points away).
Written when the weekly meter (reset Saturday 2026-11-21, 5 pm EST) read 22 % (2026-11-16 16:19 UTC, given by chris,
not measured; logged with `spend-ledger.py reading 22 --meter weekly`); that reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. Read the meter before each batch of dispatches; log a reading only at batch ends and when a stop line is within
5 points. Stop starting new work at the first of: your ledger reaching 12 points (alone on the meter: the reading minus
your start reading); the meter reaching 34 %; the hard stop (75 % beyond 48 h, 80 % at 24–48 h, 85 % at 12–24 h, 93 %
under 12 h). K = 12.2M tokens/point fitted over two pairs in `docs/budget-calibration.md`, not yet trusted, so trust
the meter's movement. In-flight work finishes. Hand off at ~500k of your context. Before handoff, write your row in the
calibration log.

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6; the rung 12 prompt); 9 asked, the ninth for pattern rotation.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Above.
- **Frozen bytes.** The 22 golden lines in `crates/render/tests/golden/SHA256SUMS` and the fixtures `v0.4/` to `v0.11/`
  stay unchanged; any change that would move them is a stop to ask chris (arc Q20).
- **Fallback.** A character missing from the chosen font may come from any installed font, with a warning at save and
  at export (ADR-002 item 6, chris's ruling; rung 11 Q4 held it back until this rung).
- **Substitution by signature** (ADR-002 item 7) and **signatures in the manifest** (ADR-001 item 6): ruled; built here.
- **The three choices** for a missing font, applying to all boxes and remembering per document (spec §7.3): ruled.
- **L11** "Missing font (using saved outlines)", Warn (spec §8.3).
- **Profiles.** Every profile lookup that draws or lays out a page goes through `builtin_for(id,
  doc.page.orientation)` (rung 11 prompt).

## Rules

- **Q1 Split.** **12a missing fonts, 12b pattern rotation**, built in that order, each with its own look; **no release
  between them**: one `v0.12.0` when both are approved. chris: "let's split it into 12a / 12b, but don't cut a release
  in the middle. Just do the work in two parts and cut the v0.12.0 release when both are approved."
- **Q2 Hinting.** **Out of v1, to v1.1.** ADR-002 item 4 (the off/light hinting setting) and ADR-001 item 3's hinted
  outline tables wait for v1.1; plain outlines stay the only kind. Recorded as a dated amendment to ADR-002 (its ruled
  text unchanged).
- **Q3 Embedding.** **Embed the allowed ones, with a switch to turn it off.** On Save, every installed (non-bundled)
  font face the page uses whose `fsType` says installable or editable is written whole into `assets/fonts/`; an
  **Embed fonts** tick in the Page tab, on by default, turns it off for the document. Bundled fonts are never
  embedded (spec §7.3). Preview-and-print, restricted, bitmap-only and Type 1 faces are not embedded; their text
  travels as saved outlines.
- **Q4 Substitute.** **Rewrite the font.** Choosing Substitute changes every affected text box's family (and face) to
  the substitute, as one undo step, saved like any edit. The missing font is not restored by itself if it returns.
- **Q5 The prompt.** **Bar, then dialog.** A page with missing fonts opens at once, drawn from its saved outlines. A
  yellow bar above the canvas names the missing fonts ("2 fonts missing: Futura, Gill Sans. Showing saved outlines.")
  with **Choose…** and a close **×**. Choose… opens the Missing fonts dialog: one row per missing font with how many
  boxes use it and a choice (Use saved outlines / Substitute: <best match> / other matches / Other font…), a **Locate
  fonts…** button, and a **Remember for this document** tick.
- **Q6 Read-only text.** **Move, not reflow.** A box drawn from saved outlines can be moved, turned, faded, copied,
  grouped, reordered and deleted. Its text, font, weight, style, size, width, height, sizing, spacing, alignment and
  baseline settings are greyed (per-device font-size overrides too), typing on the canvas does not start, and the Text
  tab opens with a badge: "⚠ Saved outlines. <Font> is not on this computer. [Choose a font…]". Its row in the layers
  panel carries a small ⚠.
- **Q7 The installed-font warning.** **A lint rule and a sheet.** A new rule **L20** "Uses fonts installed on this
  computer" (Warn; id proposed by the lead, chris to veto) lists each installed font the page uses, chosen or as
  fallback, and whether it is embedded. On **Save**, **Save As** and **Export** a short sheet says the same, with
  Continue and Cancel and a **Don't warn again for this document** tick. Autosave never asks.
- **Q8 Pattern rotation (12b).** **The region turns whole**, as every other object has since rung 6b: the rotate
  handle and the Rotation field turn the frame and its ruling together, the ruling staying parallel to the frame's
  edges. Turned lines no longer sit on whole pixels and come out softer; L02 says so.
- **Q9 Anything else.** No: the standing rules hold (frozen goldens and fixtures; tests and goldens on bundled fonts
  only; file format 0.13.0 through the migration chain with a `v0.12/` fixture; 50 ms at the 90th percentile on the Pi;
  builders commit as each step lands and read CI for their own commits).

## Owed

None.

## Defaulted

Settled by the lead without asking; each stands unless chris strikes it.

- **What "missing" means.** A text box's font is missing when its family has no face on this computer: not bundled, not
  installed, not embedded in this file. A family that is present but lacks the exact weight or italic is not missing
  (the nearest face is used, as now). Checked on open, after each font re-scan, and when a box's family changes.
  veto: count a missing weight or italic as missing too.
- **Saved outlines on a device the file has none for.** When a secondary device is added while a font is missing, its
  outlines are reused from another device where the box is the same size in mm (ADR-001 item 7); otherwise the box
  draws nothing on that device, and L11 says "needs the font" for it. veto: draw the other device's outlines scaled.
- **Save while text is read-only.** Saving keeps the file's saved outlines for read-only boxes, as now
  (`crates/ui/src/editor.rs:1149`). Because Q6 makes their text read-only, the rung 3 debt line ("Save keeps the
  file's saved outlines for that device even if the text changed") can no longer happen; a test proves it and the debt
  line is paid. veto: none needed; this is a fix.
- **Remember for this document.** Stores, in the document, the fonts chris chose to keep as saved outlines, so the next
  open shows no bar for them (L11 still lists them). Substitute needs no memory (Q4 rewrites the font). The tick is on
  by default. veto: start it off.
- **The best match.** By the measured signature (ADR-002 item 7, spike T13): an exact family name on this computer wins;
  otherwise class (serif, sans, mono, script) by measurement, then the font's own labels, then words in its name; then
  weight, width, italic, average letter width and x-height. Candidates are all fonts here, bundled and installed. The
  dialog's list shows the best match and the next four; "Other font…" opens rung 11's font picker. The signature of
  every face a document uses (bundled included) is measured and saved in the manifest's font list on Save (ADR-001
  item 6), so a later computer can match without the font. A file saved before 0.13 has no signatures; the match then
  uses the family name and the saved outlines' own letter widths where it can, else the name alone. veto: measure
  only installed fonts, not bundled ones.
- **Locate fonts….** Opens this computer's user font folder in the file manager (Linux `~/.local/share/fonts`, made if
  absent; macOS `~/Library/Fonts`; Windows the Fonts settings page), and the re-scan on focus (ADR-002 item 8) picks up
  what was added. A font that turns up is used live at once; the bar and L11 update. veto: also show a short how-to
  per system in the dialog.
- **Fallback order.** The chosen font, then the bundled fonts (Noto Emoji for emoji), then installed fonts. Goldens,
  fixtures and tests keep `FontRegistry::bundled_only()`, so no golden can depend on this machine. veto: installed
  fonts before bundled ones.
- **L11.** One finding per box per device: "Missing font <name>: drawn from saved outlines" (or "needs the font" where
  none were saved). Warn. Through Fluent, like every lint message.
- **L20 in detail.** Counts a face that comes from this computer's installed fonts, or from this file's embedded fonts,
  i.e. anything not bundled. Each entry says "embedded", "not embedded: Embed fonts is off", or "cannot be embedded:
  <licence reason>". Fallback use is found by the layout (which face drew each letter). Per document, not per device.
  The spec §8.3 table gains an L20 row by dated amendment note (as L04's was). veto: name it differently or fold it
  into L11.
- **The sheet.** Shown once per Save, Save As or Export action when L20 has findings and the document's "Don't warn
  again" is off; Continue goes on, Cancel stops (no file written). The flag is saved in the document. veto: show it on
  Export only.
- **Embedding in detail.** Whole font files, never subset (no subsetter in the stack; ADR-002 item 1). Allowed: `fsType`
  0 (installable) or with bit 0x0008 (editable), and without bit 0x0200 (bitmap only). A face inside a collection
  (`.ttc`/`.otc`) is embedded only if it can be written as one font file without a new dependency; otherwise it is
  treated as not embeddable. Stored as `assets/fonts/<sha256>.<ttf|otf>` (the spec's `<id>` is the content hash), named
  in the manifest's font list with its family, weight, italic and signature. On open, embedded faces are loaded into
  this document's fonts only (never installed on the computer) and edit like any font. A damaged or unreadable embedded
  font is skipped (its text falls back to saved outlines) and never stops the file opening; the fuzz target covers the
  reader. veto: a size cap (say 10 MB a file).
- **File format 0.13.0.** Through the migration chain: the manifest's font entries gain the signature and the embedded
  asset; the document gains "Embed fonts", "Don't warn again" and the remembered saved-outline fonts; a pattern region
  turns (12b). A `v0.12/` fixture is added and opens with the same export bytes. Any older file holding a non-zero
  `rotation_deg` on a pattern region (which drew it unturned) has it set to 0 on migration, so no old page changes.
  veto: none; this keeps the frozen bytes.
- **12a draws nothing new**, so it adds no golden page; it adds tests: a page whose font is missing exports from saved
  outlines byte-identical to the live export (made by laying out with a bundled font registered under a test-only
  name), signature classes for the bundled set (Literata serif, Inter and Atkinson sans, Atkinson Mono mono, Caveat
  script), substitution picks, the embed rule on the measured `fsType` values, an embedded font round-trip, read-only
  enforcement, L11 and L20.
- **12b in detail.** The rotate handle, Rotation field, catches at 0/90/180/270° and Shift for 15° steps as for shapes;
  every pattern kind turns (lines, dots, grid, isometric, hex, Cornell, staffs), cut at the turned frame as rung 9 cuts
  at the upright one (ADR-014 amended rule, applied in the region's own axes); a bound region keeps the rhythm's
  spacing along its own axes. L16 does not judge a turned region's ruling (it cannot sit on the page's rhythm), L02
  flags it. A group holding a pattern region may now turn (the rung 7a limit lifts for patterns). Turned regions in a
  repeat turn per copy. Both canvases draw it; a golden page per device (`patterns-turned_*`); speed measured on it.
  veto: L16 judges the ruling along the region's own axes.
- **Speed.** Measured as before on the Pi (release build, both canvases, strip on, 90th percentile, ADR-009): opening
  and dragging on a text page with a missing font (12a); drawing and dragging on the turned-patterns page (12b).
- **Debt paid on the way.** The rung 3 Save line (above); the rung 11a picker line "a document that names a system font
  this computer lacks still stops at 'font family not available'"; the rung 6b line "pattern regions do not rotate, and
  a stored angle on them is ignored" (12b).

## Kickoff

This session leads (Opus 5.5 high; context well under a third used). Build **12a** with one Sonnet 5.5 high builder on
`main`, committing as each step lands; show chris the window and stop for his look. Then **12b** the same way, and its
look. After both approvals: bump to `0.12.0`, commit `docs/arcs/v1/release-notes/v0.12.0.md`, push the tag (the
workflow publishes), re-plan the board, write the rung 13 prompt, update `STATUS.md`, `docs/STATUS-log.md`,
`docs/STATUS-debt.md`, and write the calibration row. A fresh lead picking this up must run on Opus 5.5 at high effort;
on any other model or effort, stop and say so before doing anything.

## Amendments

- 2026-11-16, chris, at his look at 12a (`--missing-fonts-page`), word for word: "I approve 12a". The builder's
  unsettled choices stand as built (Substitute keeps weight and italic; a font present but lacking a letter is laid
  out live; L20 on the primary device only; the sheet also before the unsaved-changes prompt's Save; `.ttc` faces never
  embedded; L20 keeps its id). No release yet (Q1): `v0.12.0` waits for 12b's approval. Meter 27 % after 12a (chris,
  `/usage`), 5 points of the 12-point share.
- 2026-11-16, chris, after trying 12b (before approving it), word for word: "What if the pattern itself could rotate
  independent of it's frame? So the pattern still get's trimmed by its frame, but the pattern itself could rotate
  independently." **Q8 amended:** the region still turns whole (as built), and a region also gets its own **pattern
  angle**, turning the ruling inside the frame about the frame's centre, cut at the frame; the two angles add. This is
  Q8's option "Both", which the bbq offered and chris did not pick then. Asked as two questions:
  **Q10 Where.** "Add to 12b now (Recommended)": one more builder leg before the 12b look; `v0.12.0` ships with it; the
  field joins format 0.13.0 (not yet released), read as 0 from files without it.
  **Q11 Control.** "Field in the Pattern tab (Recommended)": a Pattern angle field beside the pattern's spacing; no
  second canvas handle.
  Meter 29–30 % (chris, `/usage`; logged as 30), about 3 points for 12b's first leg.
- 2026-11-17 (01:42 UTC), chris, at his look at 12b (turned regions and the pattern angle), word for word: "approved.
  31% meter read". The builder's unsettled choices stand as built (the Pattern angle field sits in the Style tab, which
  holds the pattern's settings, since there is no Pattern tab; a bound ruling turns about the frame's centre and an
  unbound one is anchored at its top-left corner; Cornell and the staves turn as a block; the 90°/270° pivot rounded to a
  whole pixel). Both parts approved, so rung 12 is done and ships as one `v0.12.0` (Q1). Meter 31 % (logged): about 9
  points of the 12-point share.

