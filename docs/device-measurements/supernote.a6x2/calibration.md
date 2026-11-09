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
