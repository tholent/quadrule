# Calibration — Supernote Nomad (A6 X2), `supernote.a6x2`

Results from the calibration sheets (`cargo xtask calibration-sheet`, how-to in `docs/calibration.md`), judged by eye
on the device by chris. Each sheet's result is recorded word for word, then read into a table. Nothing here is in
the profile (`crates/profiles/data/supernote.a6x2.ron`) until the whole set is in and chris has agreed the values.

## Dots sheet — 2026-11-09

chris, word for word: "Nomad Dots: PD3 is my favorite. nothing on row A is visible. last visible column: PB3, PC5,
PD6, PE1, PF4, PG6, PH7. on any visible squre row the grays speckle >=5. rounded dots speckle until row H."

| row | dot | last visible (gray) | speckles |
|---|---|---|---|
| A | square 1 px | none, not even black | — |
| B | square 2 px | PB3 (102) | from column 5 (gray 170) |
| C | square 3 px | PC5 (170) | from column 5 (gray 170) |
| D | square 4 px | PD6 (187) | from column 5 (gray 170) |
| E | round 2 px | PE1 (0) | yes |
| F | round 3 px | PF4 (136) | yes |
| G | round 4 px | PG6 (187) | up to about PG3 |
| H | round 6 px | PH7 (204) | no |

Favourite: **PD2**, a square 4 px dot at gray 51. (First given as PD3; chris's correction below.)

Follow-up, chris, word for word, answering three questions: "1. I guess the speckling stops at PG3. 2. yes, square
dots are the ruling. 3. PD2 is actually my favorite overall". Asked: (1) do round rows E–G speckle at every gray and is
row H clean; (2) record square dots as the ruling; (3) is PD3 the favourite to write over, or the best-looking dot.

- Round dots: speckling stops at PG3 (chris's own estimate, "I guess"); row H does not speckle.
- **Ruling: square dots** (the rung 1 taste question on dot shape).

Read from this:

- **Square dots read better than round** at the same size: round ones speckle up to about PG3, and the 2 px round
  dot shows only in black. chris ruled square dots.
- **A single-pixel dot disappears**, even in black.
- **Grays 170 and lighter speckle** on square dots; 0, 51, 102 and 136 stay clean. (The lines sheet tests more grays.)
- **The slice page's dot grid is invisible on the Nomad**: it uses the estimated Whisper role, 2 px at gray 187, and
  2 px squares fade out after gray 102 (PB3).

## Lines sheet — 2026-11-09

chris, word for word: "nomad lines: favorite overall is LA4. last visible of each row: LA12, LB13, LC13, LD13, LE13,
LF13, DA4, DB4, DC4, DD4, DE4. last column where a dashed line was distinguishable: DA1, DB2, DC4, DD4, DE4. I don't
notice speckling across any of the samples."

| row | line | last visible (gray) |
|---|---|---|
| LA | 1 px | LA12 (187) |
| LB | 2 px | LB13 (204) |
| LC | 3 px | LC13 (204) |
| LD | 4 px | LD13 (204) |
| LE | 5 px | LE13 (204) |
| LF | 6 px | LF13 (204) |

| row | dashes, 2 px wide (dash/gap px) | last visible (gray) | last still reading as dashes (gray) |
|---|---|---|---|
| DA | 2/2 | DA4 (187) | DA1 (51) |
| DB | 2/4 | DB4 (187) | DB2 (102) |
| DC | 6/6 | DC4 (187) | DC4 (187) |
| DD | 12/6 | DD4 (187) | DD4 (187) |
| DE | 24/12 | DE4 (187) | DE4 (187) |

Favourite overall: **LA4**, a 1 px line at gray 51. No speckling on any line or dash sample.

Read from this:

- **Lines do not speckle at any gray**, while dots of gray 170 and lighter do: the speckle comes with very small
  shapes, not with the grays themselves.
- **A 1 px line shows** (to gray 187) though a 1 px dot does not; lines 2 px and wider show to gray 204. Grays 221
  and 238 vanish at every width.
- **The favourite line (1 px) and the favourite dot (4 px, PD2) differ in size**, both at gray 51, so a dot grid
  needs its own size rather than borrowing a line role's width.
- **Fine dash patterns blur into a solid line** as they get lighter: 2/2 reads as dashes only at gray 51 and 2/4 to
  102; 6/6 and longer stay dashed to 187.

## Text sheet — 2026-11-09

chris, word for word: "everything is legible at all sizes and grey levels. Most comfortable regular is TD2. Most
comfortable bold is TL2. all samples in column three are a bit harder to read. full black seems less legible than 51
for some reason"

| | code | size | gray |
|---|---|---|---|
| most comfortable, regular | TD2 | 9 pt | 51 |
| most comfortable, bold | TL2 | 9 pt | 51 |
| smallest legible | TA3, TI3 (every sample legible) | 6 pt | 102 |
| best for a label, regular | TE3 | 10 pt | 102 |
| best for a label, bold | TM3 | 10 pt | 102 |

Follow-up, chris, word for word: "if I had to choose a label, TE3 and TM3 would be the best". So comfortable
reading (9 pt, gray 51) and a label (10 pt, gray 102: one size up, one step lighter) are different choices.

Second follow-up, chris, word for word: "for labels, we can bump them up to row 2. They aren't bad, and I'd like to
keep the two the same". Read as column 2 (gray 51; the sheet's rows are sizes, its columns grays): **labels are TE2
and TM2, 10 pt at gray 51**, the same gray as reading text. This replaces the TE3/TM3 choice above.
*Misread; reversed by the third follow-up.*

Third follow-up, chris, word for word: "reverse that. the labels should be matched for size, but differ in shade of
gray". Asked which labels, chris chose **TD3 and TL3**: labels are the same size as reading text (9 pt), one shade
lighter (gray 102). This is the ruling; it replaces both earlier label choices.

| | regular | bold | size | gray |
|---|---|---|---|---|
| reading text | TD2 | TL2 | 9 pt | 51 |
| labels | TD3 | TL3 | 9 pt | 102 |

*The labels row is replaced by the fourth follow-up.*

Fourth follow-up. chris, word for word: "I'd like to consider a lighter shade of gray for labels, maybe 153?" The text
sheet gained column 4 (gray 153). chris, word for word, after judging it: "td4 and tl4 work for lables". **Labels are
TD4 and TL4**: the same size as reading text (9 pt), at gray 153. This is the current ruling.

| | regular | bold | size | gray |
|---|---|---|---|---|
| reading text | TD2 | TL2 | 9 pt | 51 |
| labels | TD4 | TL4 | 9 pt | 153 |

Read from this:

- **Every sample is legible**, down to 6 pt at gray 102, so the sheet did not find a floor; the smallest size tested
  (6 pt) is the lowest known to work.
- **Gray 51 reads best.** Gray 102 (column 3) is "a bit harder to read" at every size, and black (column 1) reads
  less well than 51. Why black is worse is not known; that is chris's observation, not a measurement.
- **9 pt is the comfortable size** for both weights.
- **Gray 51 is chris's favourite on every sheet so far**: line LA4, dot PD2, text TD2 and TL2. For labels, though,
  chris picks the lighter gray 102 at 10 pt (TE3, TM3): a label should stay out of the way, and the slightly larger
  size makes up for the lighter gray. *Replaced by the second follow-up: labels are gray 51 too (TE2, TM2), to keep
  text and labels the same gray.* *Reversed by the third follow-up: labels are 9 pt at gray 102 (TD3, TL3), the same
  size as reading text and one shade lighter.* *Then lightened by the fourth follow-up: labels are 9 pt at gray 153
  (TD4, TL4).*
- **Text at gray 153 works on the Nomad**, though square dots speckle from gray 170: only one step separates them.

## Writing sheet, set 2 — 2026-11-09

chris, word for word: "for writing, all lines are visible. for dots, w13 and w14 are not visible. W1 is my favorite
line. W15 is my favorite dot"

| strips | what | result |
|---|---|---|
| W1–W12 | ruled lines 1, 2, 3 px × grays 51, 102, 153, 187 | all visible |
| W13, W14 | square dots 2 px, grays 51 and 102 | visible, but not usable (see the follow-up) |
| W15–W18 | square dots 3 and 4 px, grays 51 and 102 | visible |

Follow-up, chris, word for word: "I do see them, they are just not usable at that size". So W13 and W14 show; 2 px
dots are too small to work with, not invisible. This agrees with the dots sheet (PB2, PB3 visible).

Favourite line: **W1** (1 px, gray 51), the same as LA4 on the lines sheet. Favourite dot: **W15** (square 3 px,
gray 51), one size smaller than PD2 (4 px) on the dots sheet.

Read from this:

- ~~**2 px dots are not visible on a page of dots 5 mm apart**, although PB2 and PB3 (the same size and grays) showed
  on the dots sheet. The difference is in the looking: on the dots sheet each sample is a small 3 × 3 cluster, 4 mm
  apart, right under its code, so the eye knows where to look.~~ *Withdrawn after chris's follow-up: the 2 px dots
  are visible, just too small to use; there is no difference between the sheets to explain.* Checked: the writing
  page draws W13 and W14 as 60 dots each, every one exactly 2 × 2 px at gray 51 and 102.
- **For a working dot grid, 3 px is the smallest usable size**: 2 px dots show but are "not usable at that size".
- **For writing, chris prefers a 3 px dot** (W15) over the 4 px dot he liked best on the dots sheet (PD2).
- **The favourite line holds when writing**: W1 matches LA4 (1 px, gray 51).
- **Gray 51 is the favourite on every sheet**: lines, dots, reading text and writing.

## Rulings from the Nomad results — 2026-11-09

Asked as three questions; chris's answers, word for word (the option chosen):

1. A dot-grid setting of its own in each profile, apart from the four line roles: "Yes, own setting (Recommended)".
   On the Nomad: **W15**, square 3 px, gray 51.
2. The Nomad line roles: "Yes, use these (Recommended)".

   | role | code | width | gray |
   |---|---|---|---|
   | Whisper | LA8 | 1 px | 119 |
   | Guide | LA4 | 1 px | 51 |
   | Structure | LB4 | 2 px | 51 |
   | Emphasis | LC1 | 3 px | 0 |

3. Toolbar positions chris uses: "Docked left, Docked top". The safe-zone lint (L06) should warn only about these.

Text, from the text sheet: reading text 9 pt at gray 51 (TD2, TL2); labels 9 pt at gray 153 (TD4, TL4).

None of this is in `crates/profiles/data/supernote.a6x2.ron` yet.

## Grays sheet — 2026-11-12

Sheet: `cargo xtask calibration-sheet` run 2026-11-12 on the Pi (`calib/supernote-a6x2/quadrule-calib_grays-1_…png`,
sha256 `13e67e50…`, and `…grays-2_…png`, sha256 `fed559df…`). Patches G1–G15 are the even steps 0, 17 … 238; G16–G30
are halfway between them, 8, 25 … 246; GR is a ramp from 0 to 255 (`calib/supernote-a6x2/codes.txt`).

chris, word for word: "nomad: all patches clean, though G1-G5,G16-G19 are indistinguishable from each other (all appear
black). G15,G29-G30 just appear white. the ramp appears to have about 16 steps which includes two large bands on each
end where the shades are indistinguishably white or black."

What it says, in grays:

| reading | patches | grays |
|---|---|---|
| clean (no speckle, dots, stripes or blotches) | all, G1–G30 | 0 to 246 |
| look black, the same as each other | G1–G5, G16–G19 | 0, 8, 17, 25, 34, 42, 51, 59, 68 |
| look white | G15, G29, G30 | 229, 238, 246 |
| between (neither black nor white) | G6–G14, G20–G28 | 76 to 221 |
| ramp | GR | about 16 steps, a wide black band at the dark end and a wide white band at the light end |

So on a flat area every gray from 0 to 68 shows as black, and every gray from 229 up as white. Not said yet: whether the
between-step patches in the middle (G20–G28) can be told from the even steps beside them, which decides how many
distinct middle levels there are.

Asked next (2026-11-12), chris's answers word for word:
1. Can each between-step patch be told from the even steps beside it? → "Each matches a neighbour". So the middle has
   the nine even steps 85, 102 … 221 as its distinct grays; G20 (76) is not black, so it matches 85.
2. Block, 68, shows as plain black: what should it be? → "A lighter dark gray" (about 102, G7). Block is now 102 on
   both devices; Shade stays 204 (G13: clean, visible, not white).
3. The Manta: read its own sheet, or the same as the Nomad? → "Same as the Nomad".

In the profile (`crates/profiles/data/supernote.a6x2.ron`): `gray_levels` keeps all sixteen even steps, now measured
(each is clean: flat, no speckle), with a note that 0–68 look black and 238–255 white on a flat area; `fill_roles`
Shade 204, Block 102. The ramp's "about 16 steps" is more than the eleven distinct grays the patches show (black, the
nine middle steps, white); a ramp is hard to count by eye, so the patches are taken as the reading.
