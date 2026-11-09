# Text spike: findings

2026-11-09 · napkin §7 and §16 spike 2 (`napkins/quadrule-spec.md:342-389`, `:708`) · plan:
`docs/bbq/2026-11-09-quadrule-spikes.md` · machine: Raspberry Pi 5, 4 GB, Debian aarch64, rustc 1.99.1.
Saved by the lead from the worker's report, word for word (the worker could not write report files).

Throwaway code. It answers questions; it is not the start of the real `text` crate.

## How to reproduce

```
sh spikes/text/fetch-fonts.sh        # downloads the two OFL fonts (not kept in git)
cargo run --release --manifest-path spikes/text/Cargo.toml    # prints the CHECK lines below
cargo test --manifest-path spikes/text/Cargo.toml             # 12 pass/fail tests
```

Fonts used (both SIL Open Font License 1.1, licence files checked):

| file | from | sha256 |
|---|---|---|
| `AtkinsonHyperlegibleNext[wght].ttf` | `https://raw.githubusercontent.com/google/fonts/main/ofl/atkinsonhyperlegiblenext/` | `5a455d1cfa099b601ab70751bb9673e8fe1854dc4500c80e1a220d0d75e31745` |
| `NotoEmoji[wght].ttf` | `https://raw.githubusercontent.com/google/fonts/main/ofl/notoemoji/` | `de6c18832938afc99caf132b39d6a30a19bac7f2e812e28db2535b4608d27551` |

Crate versions: cosmic-text 0.19.0, which brings fontdb 0.23.0, skrifa 0.40.0 and harfrust 0.5.2 (the shaper);
kurbo 0.13.1.

## Checks

One line per check, exactly as `cargo run --release` printed them.

```
CHECK id=T01 name=fontdb_enum result=pass value=4.5ms faces=111 families=29 cmd="fontdb::Database::load_system_fonts"
CHECK id=T02 name=fontdb_rescan result=pass value=4.2ms stat_walk=0.51ms entries=317 fontsystem_new=4.1ms cmd="median of 5 full re-scans"
CHECK id=T03 name=wrap_scale_invariant result=pass lines300=5 lines226=5 lines_mm=5 box=80mm size=10pt
CHECK id=T04 name=justify result=pass worst_gap_mm=0.0000 lines=5
CHECK id=T05 name=letter_spacing result=pass added_mm=0.1764 expected_mm=0.1764 lines0=5 lines_ls=5
CHECK id=T06 name=line_height result=pass value_mm=4.9389 expected_mm=4.9389
CHECK id=T07 name=variable_weight result=pass first10_w400_mm=17.477 first10_w700_mm=18.634
CHECK id=T08 name=outline_extract ppi=300 result=pass glyphs=221 inked=190 path_elements=4551 pen_chain_err_px=0.000023 value=0.53ms
CHECK id=T09 name=kerning_in_layout ppi=300 result=pass kerned_glyphs=62 of=221
CHECK id=T10 name=advance_agree_plain ppi=300 result=pass worst_px=0.00891 worst_font_units=0.214 font_unit_px=0.04167
CHECK id=T08 name=outline_extract ppi=226 result=pass glyphs=221 inked=190 path_elements=4551 pen_chain_err_px=0.000031 value=0.47ms
CHECK id=T09 name=kerning_in_layout ppi=226 result=pass kerned_glyphs=62 of=221
CHECK id=T10 name=advance_agree_plain ppi=226 result=pass worst_px=0.01194 worst_font_units=0.380 font_unit_px=0.03139
CHECK id=T11 name=atkinson_used result=pass families="Atkinson Hyperlegible Next"
CHECK id=T12a name=emoji_auto_fallback result=fail picks="U+1F4DD=Noto_Emoji,U+2705=FreeSans,U+1F319=Noto_Emoji"
CHECK id=T12b name=emoji_explicit_noto result=pass glyphs=3 all_noto_inked=true
CHECK id=T13 name=substitute hidden="Liberation Serif" class=Serif result=pass picked="Nimbus Roman" picked_class=Serif weight=400 score=17 value=2.5ms
CHECK id=T13 name=substitute hidden="Liberation Sans" class=Sans result=pass picked="Nimbus Sans" picked_class=Sans weight=400 score=24 value=1.2ms
CHECK id=T13 name=substitute hidden="Liberation Mono" class=Mono result=pass picked="DejaVu Sans Mono" picked_class=Mono weight=400 score=7 value=1.1ms
CHECK id=T13 name=substitute hidden="DejaVu Sans" class=Sans result=pass picked="Liberation Sans" picked_class=Sans weight=700 score=390 value=1.1ms
CHECK id=T13 name=substitute hidden="Atkinson Hyperlegible Next" class=Sans result=pass picked="Nimbus Sans" picked_class=Sans weight=400 score=49 value=1.2ms
CHECK id=T14 name=text_on_macos result=owed reason="no macOS machine in this session"
CHECK id=T15 name=text_on_windows result=owed reason="no Windows machine in this session"
```

The very first run after boot measured T01 at 90.5 ms, because the font files were not yet in memory (a "cold
cache"). Every later run was 4–8 ms.

## What the checks mean, in plain words

**1. Finding system fonts is cheap (T01, T02).** This Pi has 111 font faces (29 families). Listing them all takes
about 5 ms, or about 90 ms the first time after a restart. Doing it again from scratch also takes about 4 ms. The
napkin wants the font list refreshed whenever the app regains focus (`:353`). At this size a full re-scan is
cheaper than working out what changed, so no clever change detection is needed. A design machine with thousands
of fonts will be slower; that number is owed from a Mac or Windows machine (T14, T15).

**2. Paragraph layout works in millimetres (T03–T07).** cosmic-text does not care what unit it is given. The same
80 mm wide, 10 pt box was laid out three ways: in millimetres, at 300 PPI (pixels per inch) and at 226 PPI. All
three broke into exactly the same lines (T03). So quadrule can lay text out once in millimetres, as the napkin
stores geometry (`:222`), and scale the result to each device.

Which text-box properties from napkin §7.1 (`:347`) cosmic-text handles:

| property | cosmic-text | proven by |
|---|---|---|
| font family, weight, style | yes; weight also drives a variable font's weight axis | T07, T11 |
| size | yes | T03 |
| line height | yes, exact | T06 |
| letter spacing | yes, measured in em (a fraction of the font size) | T05 |
| alignment left, center, right | yes | — |
| justify | yes; every line but the last reaches the right edge | T04 |
| auto height (grows down) | yes: give no height, then add up the line heights | T06 (`total_height`) |
| paragraph spacing | **no**: quadrule adds the gap between paragraphs itself | — |
| vertical alignment in fixed boxes | **no**: quadrule shifts the lines itself, from the total height | — |
| overflow indicator (`:346`) | **no** built-in signal: quadrule compares total height with the box height | — |
| baselines on the rhythm (`:315`) | line height can be a rhythm multiple; placing the *first* baseline on the rhythm is quadrule's own work | not tested |

The three "no" rows are small: each is a few lines of arithmetic on numbers cosmic-text already reports.

**3. Text becomes vector outlines that sit exactly where layout put them (T08–T10).** For every laid-out glyph (one
drawn letter shape), skrifa draws its outline as a path, which is placed at the position cosmic-text chose. The
sample paragraph gave 221 glyphs, 190 with ink (the rest are spaces), and 4,551 path pieces, in about half a
millisecond. Each glyph starts exactly where the previous one ended (error under 0.0001 px; T08).

The two libraries agree on letter widths:
- With kerning (pair spacing such as "AV" and "To") switched off, they agree to within 0.4 of a font unit. A font
  unit is the font's own grid step, about 1/24 of a pixel at this size (T10). The tiny gap is because the shaper
  (harfrust) rounds positions to whole font units; it is far too small to see.
- With kerning on, 62 of 221 glyphs differ. That is intended: kerning is meant to change spacing (T09). Because the
  outlines are placed at cosmic-text's positions, kerning carries into the export.

**4. The bundled fonts work (T11, T12).** Atkinson Hyperlegible Next and Noto Emoji both load from files and shape
correctly. An emoji object whose font is set to Noto Emoji gives crisp vector outlines for every emoji (T12b), as
napkin §7.4 wants (`:385`).

But **automatic fallback is machine-dependent (T12a, fail)**. When emoji sit inside Atkinson text and the app
lets cosmic-text pick a font for them, ✅ came from the system's FreeSans, not Noto Emoji, because FreeSans
happens to contain that character. On another computer, with other fonts installed, the same document would
pick differently. That breaks the napkin's promise that text output is "identical everywhere" (`:349`). See
"Parked for chris".

**5. Substitution can be done well, but not from font metadata alone (T13).** fontdb gives family name, weight,
width, italic and a "monospaced" flag. It does not say serif or sans. Fonts' own labels for that are often
missing or wrong:
- Atkinson Hyperlegible Next and Nimbus Roman leave the serif/sans classification fields empty.
- FreeMono does not set its monospaced flag.

So the spike *measures* each font instead: average letter width, x-height (the height of a lowercase "x"), and
whether "i" and "M" are equally wide (monospace). It falls back to the font's labels, then to words in the family
name. With one installed family hidden to pretend it is missing, the picks were the same ones a typographer
would make:

- Liberation Serif → Nimbus Roman (both Times-like)
- Liberation Sans → Nimbus Sans (both Helvetica-like)
- Liberation Mono → DejaVu Sans Mono
- Atkinson Hyperlegible Next → Nimbus Sans

Each pick took about 1–2 ms. An exact family-name match always wins first, and fontdb's own lookup does that part
for free.

**6. Not tested here (T14, T15, result=owed).** Font discovery, layout and outlines on macOS and Windows. The
napkin asks for all three systems (`:708`); this machine is Linux only.

## Other things noticed

- **Version skew.** Adding `fontdb`, `skrifa` and `cosmic-text` as three separate dependencies, as napkin §15.2
  lists them (`:655`), pulled in two versions of fontdb and three of skrifa. Using the copies cosmic-text
  re-exports (`cosmic_text::fontdb`, `cosmic_text::skrifa`), and switching off its `swash` rasterizer (a part
  that turns outlines into pixels, which quadrule does not need), leaves one of each. See `Cargo.toml`.
- **No system packages needed.** Everything built with what this Pi already had; no `apt` commands.
- **For the format spike:** the sample paragraph's outlines are 4,551 path pieces for about 190 glyphs, or roughly
  24 pieces per letter. That is a starting point for its "outline storage size" question (`:709`).

## Parked for chris

1. **Font fallback and "identical everywhere".** When a character is missing from the chosen font, where may
   quadrule look for it? (a) Only in bundled fonts: same result on every machine, fewer characters covered.
   (b) Any installed font: more characters covered, but output can change between machines, and the saved
   outlines (`:370`) become the only guarantee. (c) Bundled first, system second, with a lint warning when a system
   font was used. Not decided here.
2. **A "font signature" in the project file.** Good substitution needs the missing font's measured width and
   x-height, so the document must store them when it is saved. That adds a small record per font to the file
   format (§12). Hand-off for the format spike and for chris.
