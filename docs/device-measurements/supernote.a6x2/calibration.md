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
| G | round 4 px | PG6 (187) | yes |
| H | round 6 px | PH7 (204) | no |

Favourite: **PD3**, a square 4 px dot at gray 102.

Read from this:

- **Square dots read better than round** at the same size: round ones speckle at every size up to 6 px, and the
  2 px round dot shows only in black.
- **A single-pixel dot disappears**, even in black.
- **Grays 170 and lighter speckle** on square dots; 0, 51, 102 and 136 stay clean. (The lines sheet tests more grays.)
- **The slice page's dot grid is invisible on the Nomad**: it uses the estimated Whisper role, 2 px at gray 187, and
  2 px squares fade out after gray 102 (PB3).
