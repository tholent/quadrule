# bbq record — quadrule rung 11: text and fonts

2026-11-14 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md`) · 8 questions · lead:
Opus 5.5 at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 11 and 12 rows), `docs/arcs/v1/rung-11-PROMPT-2026-11-13.md`,
`docs/decisions/INDEX.md` and ADR-002 in full (owed: hinting flag, fallback with its warning, substitution, re-scan on
focus, paragraph spacing, vertical alignment, overflow), `docs/bbq/2026-11-13-rung-10-repeat-tool.md`,
`napkins/quadrule-spec.md` §6.3 line 330 (baselines to rhythm), §7 (lines 640–690), §8.3 (L08 line 451, L11 line 454),
§16 (v1 list: "text boxes (single style), monochrome emoji"; no text rotation), `docs/STATUS-debt.md` (rung 4a, 6b, 7a,
7b, 8a lines), `crates/text/src/lib.rs` (one bundled font, `FontRegistry::bundled_only`, `lay_out` returns
`height_mm`, `Wrap::WordOrGlyph`), `crates/model/src/lib.rs:1021` (`TextBox`: content, family, weight, size_pt,
line_height, letter_spacing_em, align; no style field), `crates/format/src/lib.rs:48` (`FORMAT_VERSION` 0.10.0),
`Cargo.toml` (cosmic-text 0.19.0 with `fontconfig`; egui 0.37.0).

Measured at the start of this session (2026-11-14 on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=4cf4b04 dirty=no rebuilt=11` (the 0.10.0 version bump in `Cargo.toml`, `26f552f`, had not been built), then
again → `BUILD head=4cf4b04 dirty=no rebuilt=0`. Network: `curl -sI https://github.com/google/fonts/...` → HTTP/2 302
(reachable). Font sizes from `gh api repos/google/fonts/contents/ofl/<family>`: Atkinson Hyperlegible Next 114,552 and
italic 123,916; Atkinson Hyperlegible Mono 53,960 and 56,364; Inter 876,576 and 906,596; Literata 955,132 and 902,728;
Caveat 403,648; Patrick Hand 214,772; Noto Emoji 1,982,596 bytes.

Asked again at session start (AGENTS.md): text rotation, parked for v1.1. chris, choosing the option: **"Pull into
rung 11"**. Still parked, not re-asked: "Scale vs pixel-locked rhythm on A5 X" (v1.1, arc Q16); the Label gray (v1.1).
Pattern rotation stays v1.1 (Defaulted below).

## Budget
Share: 15 points (1 point = 1 % of the weekly limit), absolute allocation by chris ("15 points (Recommended)"), alone on
the meter, reserve 0 ("Alone, no reserve").
Room: 15 points (the hard stop, 75 %, is 74 points away).
Written when the weekly meter (reset Saturday 2026-11-14, 5 pm EST; next reset 2026-11-21) read 1 % (2026-11-15 00:32
UTC, given by chris as "0–2 %", not measured; logged with `spend-ledger.py reading 1 --meter weekly`); that reading is
not yours. On pickup: take the reading (or ask chris for it), name your meter, note your start reading, and cap the
share at your hard stop minus your reading. Read the meter before each batch of dispatches; log a reading only at batch
ends and when a stop line is within 5 points. Stop starting new work at the first of: your ledger reaching 15 points
(alone on the meter: the reading minus your start reading); the meter reaching 16 %; the hard stop (75 % beyond 48 h,
80 % at 24–48 h, 85 % at 12–24 h, 93 % under 12 h). K = 12.2M tokens/point fitted over two pairs in
`docs/budget-calibration.md`, not yet trusted, so trust the meter's movement. In-flight work finishes. Hand off at
~500k of your context. Before handoff, write your row in the calibration log.

## Settled before the rounds (cited, not asked)

- **Count.** 8 (arc Q6; the rung 11 prompt).
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Above.
- **Text rotation.** In rung 11 (chris at session start, above). It is not in spec §16's v1 list, so this adds to the
  arc; the board notes it.
- **Frozen bytes.** The eighteen golden lines in `crates/render/tests/golden/SHA256SUMS` and the fixtures `v0.4/` to
  `v0.9/` stay unchanged; any change that would move them is a stop to ask chris (arc Q20).
- **Profiles.** Every profile lookup that draws or lays out a page goes through `builtin_for(id,
  doc.page.orientation)` (rung 11 prompt).
- **What rung 12 holds.** The missing-font flow (§7.3), L11, the system-font lint and its save and export warning
  (ADR-002 item 6), substitution by signature (item 7), embedded fonts in `assets/fonts/` (board, rung 12 row).

## Rules

- **Q1 Split.** **11a fonts, 11b the box**, each with its own look and release. **11a** (`v0.11.0`): the bundled set
  and its OFL texts, system fonts on all three systems with the re-scan on focus, the font picker, italic, monochrome
  emoji and its picker. **11b** (`v0.11.1`): fixed-size boxes, the overflow mark and L08, paragraph spacing, vertical
  alignment, baselines on the rhythm, text rotation, typing on the canvas (Q3). The builder commits each step as it
  lands (rung 8 lesson 1).
- **Q2 Bundled fonts.** **The spec set with italics, and Caveat**: Atkinson Hyperlegible Next and Mono, Inter,
  Literata, Caveat and Noto Emoji, each with its italic where one exists (Caveat and Noto Emoji have none), about
  +6.4 MB in every package. Italic is a new **Style** field on a text box (Normal or Italic).
- **Q3 Typing on the canvas.** **Type in the box** (11b): double-click a text box, or draw a new one, to type in place,
  with a cursor, selection, and copy and paste; the Text tab's field stays. This grows the rung past the board's list
  (said before the question was asked).
- **Q4 Missing characters.** **Bundled-only fallback until rung 12.** System fonts are listed and usable as a box's
  chosen family on Linux, macOS and Windows, re-scanned when the window regains focus (ADR-002 item 8); a character the
  chosen font lacks comes from the bundled fonts only (emoji from Noto Emoji). Rung 12 turns on fallback to any
  installed font together with its save and export warning (ADR-002 item 6), so no export quietly depends on one
  machine's fonts before the warning exists.
- **Q5 Overflow.** **Cut whole lines, red mark.** In a fixed-size box, lines that do not fully fit are left out on
  the canvas and in the export (no half-cut letters); a small red **+** tab at the box's bottom-right corner shows on
  the canvas only; L08 (Warn) names the box and the device and how many lines are hidden. Existing boxes stay auto
  height, so no golden page moves.
- **Q6 Baselines on the rhythm.** **Every baseline on a rule.** With "Align baselines to rhythm" ticked, the line
  height becomes a whole number of rhythm steps (at least one), and the first baseline sits on the first rhythm line
  inside the box, so every line of type stands on the ruling as handwriting does on ruled paper. Off for blank
  documents and in New…; on in ruled starters (spec §6.3 line 330; starters are rung 16).
- **Q7 Emoji.** **Characters in text.** An emoji is a character in a text box, at the box's size and style, drawn
  from Noto Emoji. The picker types into the text being edited; with none, it drops a new text box holding just the
  emoji in the middle of the view. No new object kind; the "source" setting comes with colour-to-gray in v1.1.
- **Q8 Anything else.** No: the standing rules hold (frozen goldens and fixtures; 50 ms at the 90th percentile on the
  Pi on a text-heavy page; commit as each step lands; pay the debt in the code the rung touches).

## Owed

None.

## Defaulted

Each settled by the lead without asking; chris can veto any of them at the look of the part named.

**11a**

- **Font files.** From `google/fonts` (`ofl/<family>/`), the variable files: `AtkinsonHyperlegibleNext[wght]` and
  `-Italic[wght]`, `AtkinsonHyperlegibleMono[wght]` and `-Italic[wght]`, `Inter[opsz,wght]` and `-Italic[opsz,wght]`,
  `Literata[opsz,wght]` and `-Italic[opsz,wght]`, `Caveat[wght]`, `NotoEmoji[wght]`. Each in `crates/text/fonts/` with
  its sha256 in `BUNDLED`, checked by the existing test. The existing Atkinson Next file stays byte-identical (its sha256
  is unchanged), so the slice and every golden page keep their bytes. The optical-size axis follows the point size
  (`opsz` = size in pt, clamped to the font's range), as the fonts intend. veto: pin `opsz` to a fixed value.
- **Licences.** Each family's `OFL.txt` is read from its folder and checked to be the SIL Open Font License 1.1 before
  its font is added (a family that is not OFL is a stop to ask chris). The texts sit beside the fonts as
  `OFL-<Family>.txt`, go into every package's licence folder (`packaging/`), and are listed in `README.md`'s credits.
  veto: an in-app licences window as well.
- **Download size.** Every package grows by about 6.4 MB (Q2). The release workflow's package sizes before and after are
  written in the log at the release. veto: none expected.
- **Weights.** The weight field offers each family's own range (read from its `wght` axis); a value outside it is
  clamped on layout and the field says so. A system font that is not variable offers the weights installed. veto: none.
- **Style field.** `TextBox.style`: Normal (the default) or Italic, a serde default, so every older file opens Normal.
  Italic picks the family's italic face; a family with none (Caveat, Noto Emoji, a system font without one) draws
  upright and the field is greyed with a note. No synthetic slanting. veto: slant upright fonts.
- **System fonts.** `fontdb`'s system scan on all three systems (the `fontconfig` feature stays for Linux), so the debt
  line "Text spike T14/T15" is paid when CI shows system faces found on macOS and Windows: a test lists the system faces
  and prints one structured line with the count on each OS; on macOS and Windows it requires at least one. Bundled
  families win over a system copy of the same name (the bundled version is what the file references). Re-scan on focus
  is a full re-scan (ADR-002 item 8), timed on the Pi and written in the log; over 50 ms is said before it lands. veto:
  a system copy wins.
- **Fallback rule (Q4) in code.** Layout uses a fallback list of the bundled faces only, in the order Atkinson Next,
  Noto Emoji, then the rest; system faces are never fallback in 11a. A character in no bundled font draws as the
  chosen font's missing-glyph box, as today. veto: none expected.
- **The font picker.** In the Text tab, replacing the family field: one list in two sections, **Bundled** then
  **System** (no Downloaded section until v1.1's packs), each family name drawn in its own font, a search box that
  filters both, and a preview line showing the box's own text (or the family name when empty) at the box's point size
  drawn as export pixels for the device on screen, at 1:1. Picking a family is one undo step. veto: chris's look.
- **The emoji picker.** An **Emoji** button in the toolbar and in the Text tab opens a picker window: a search box
  (by emoji name), groups (smileys, people, nature, food, travel, activities, objects, symbols, flags), each emoji drawn
  from Noto Emoji. Names and groups come from the `emojis` crate (licence checked: MIT or Apache-2.0, else a stop) or
  CLDR data of the same licence. Insert puts the emoji at the Text tab's cursor (11a) or the canvas cursor (11b); with
  no text being edited, a new text box holding just the emoji, 24 pt, in the middle of the view, one undo step. No
  shortcut (rung 17). veto: a different home for the button.
- **New text boxes.** Still 12 pt Atkinson Hyperlegible Next, Normal (rung 5); the rung 5 debt "no choice in the
  dialog" stays. veto: none expected.
- **File format 0.11.0** (11a). `TextBox.style` with a serde default; a `crates/render/tests/fixtures/v0.10/` fixture
  proves a 0.10 file opens through the migration chain with the same export bytes. veto: none expected.
- **Golden page 11a.** One fonts page per device in portrait (`fonts_*`): every bundled family upright and italic at
  two sizes, a weight ramp in Inter and Literata, Atkinson Mono, and an emoji line, all bundled (no system font is ever
  on a golden page). Each image is looked at by eye before it is blessed (rung 8 lesson 2). Opens with `--fonts-page`.
  veto: none expected.

**11b**

- **Sizing.** A **Sizing** choice in the Text tab: **Auto height** (today's behaviour, the default, so every older file
  is unchanged) or **Fixed**. An auto box shows left and right handles only (rung 4a); a fixed box shows all eight;
  pulling the top or bottom edge of an auto box turns it Fixed, as in Figma. veto: never switch it by a drag.
- **Laid-out height.** An auto box's height is its laid-out height on each device, everywhere the app reads a box
  height: picking, snapping, align and distribute, L15. This pays the rung 7b debt ("aligns by its stored frame") and
  the rung 8a debt ("L15's squeezed check uses a text box's stored height"). veto: none expected.
- **The overflow mark and L08.** The red **+** tab (Q5) is drawn on both canvases, never exported, and shows on
  every device's strip picture where that device overflows. L08 (Warn) is judged per device, since the font size can
  be overridden per device (rung 8a). veto: none expected.
- **Paragraph spacing.** A **Paragraph spacing** field in any unit (stored in pt; `u` kept in rhythm steps, as rung 10's
  spacing), default 0, added after each paragraph but the last. With baselines on the rhythm, it rounds up to whole
  rhythm steps. veto: before each paragraph instead.
- **Vertical alignment.** Top (the default), Middle or Bottom, for fixed boxes only (greyed for auto). With baselines on
  the rhythm, Middle and Bottom move the text by whole rhythm steps so baselines stay on rules. veto: none expected.
- **Baselines on the rhythm, details.** A per-box tick in the Text tab, off by default. The first baseline sits on the
  first rhythm line at least the font's cap height below the box's top, so capitals do not stick out above the box.
  The step is the rhythm's vertical step (`Rhythm::steps_mm()`, half a side under an isometric rhythm). Under a
  pixel-locked rhythm each baseline lands on the same pixel row as the rhythm line a ruled region draws there, tested
  on both devices. While ticked, the line-height field shows the whole-step value it resolves to and cannot be typed.
  veto: the first rule at or below the box top itself.
- **Text rotation.** As shapes (rung 6b): the round rotate handle on a stalk, catching at 0, 90, 180 and 270°, Shift
  for 15° steps, and the Arrange tab's Rotation field; the box turns about its centre and the export draws the turned
  outlines. Pays the rung 6b debt for text ("text boxes … do not rotate, and a stored angle on them is ignored"). A
  stored angle on a text box in an older file was ignored before: if any fixture or golden page holds a text box with
  a non-zero angle, that is a stop to ask chris. A group holding text can now turn (rung 7a's "not with text or a
  pattern inside" keeps only the pattern). L06 and L10 judge a turned text box by its turned corners. veto: none.
- **Pattern rotation.** Stays v1.1. veto: pull it in too.
- **Typing on the canvas, details.** Double-click a text box (or a text box's copy in a repeat: it edits the source)
  starts typing at the click; a newly drawn text box starts typing with its text selected (instead of the Text tab's
  field, rung 5). Our own editor on cosmic-text's `Editor` over the same layout the export uses, so the cursor and
  selection sit exactly on the drawn letters, turned boxes included (no unturning while typing). Arrows, Home and End,
  Shift to select, Ctrl+A, Ctrl+C, Ctrl+X, Ctrl+V as plain text, Enter for a new paragraph, Escape or a click outside
  ends typing. Inside a typing session Ctrl+Z steps back through typed words; on leaving, the session is one step in the
  document's undo (ADR-010). A double-click on text inside a group types into it directly (no need to enter the group
  first). Input methods (IME) for composed scripts: egui's IME events are passed through; tested on Linux only. veto:
  unturn the box while typing.
- **File format 0.12.0** (11b). Sizing, paragraph spacing, vertical alignment and the baselines tick, each with a serde
  default that keeps today's layout; a `crates/render/tests/fixtures/v0.11/` fixture proves a 0.11 file opens with the
  same export bytes. veto: none expected.
- **Golden page 11b.** One text-box page per device in portrait (`textbox_*`): a fixed box with hidden lines, the three
  vertical alignments, paragraph spacing, a box with baselines on a ruled pattern region, and turned text at 15°, 90° and
  180°. Looked at by eye before blessing. Opens with `--textbox-page`. Landscape is covered by a test that a landscape
  page with turned text exports as the exact quarter turn, not a golden. veto: add landscape goldens.

**Both parts**

- **Speed.** A text-heavy page (the fonts page in 11a, the text-box page in 11b): time dragging and drawing on the Pi
  (release build, both canvases, strip on, 90th percentile, ADR-009), and in 11b typing as well (a new `--bench-type`
  that types into a long box), and say before anything crosses 50 ms. This pays the rung 4a debt line "not on a
  text-heavy page". veto: none expected.
- **Not in rung 11.** The hinting flag (ADR-002 item 4) stays owed; substitution and the missing-font flow are rung 12;
  rich text runs and font packs are v1.1.
- **Debt.** Pay the debt in code the rung touches (arc Q17): at least the T14/T15 line (11a), and the rung 7b, rung 8a
  L15 and rung 6b text lines (11b); each paid line is noted in the log, and new debt goes in `docs/STATUS-debt.md` as a
  "Rung 11a, owed" or "Rung 11b, owed" line.

## Kickoff

This session leads (context well under a third used at the end of the bbq: run here). Opus 5.5 at high effort leads;
on any other model or effort, stop and say so before doing anything. Brief one Sonnet 5.5 high builder from this record
for **11a**, building on `main` and committing as each step lands: (1) the fonts, their licences and `BUNDLED`; (2) the
Style field, format 0.11.0 and the `v0.10/` fixture; (3) system fonts on all three systems and the re-scan on focus,
with the bundled-only fallback; (4) the font picker; (5) the emoji picker; (6) the fonts golden page, tests and the
Pi timings. The lead judges each landing against this record and the frozen sums, shows chris the window, and stops for
his look. After 11a's approval: bump to `0.11.0`, commit `docs/arcs/v1/release-notes/v0.11.0.md`, tag, then brief 11b
the same way (steps: sizing and laid-out height; overflow and L08; paragraph spacing and vertical alignment; baselines
on the rhythm; rotation; typing on the canvas; format 0.12.0 and the `v0.11/` fixture; golden page, tests, timings),
look, `0.11.1`, release. Then re-plan the board and write the next lead's prompt in `docs/arcs/v1/`.
