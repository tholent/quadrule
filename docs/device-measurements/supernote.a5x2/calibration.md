# Calibration — Supernote Manta (A5 X2), `supernote.a5x2`

Results from the calibration sheets (`cargo xtask calibration-sheet`, how-to in `docs/calibration.md`), judged by eye
on the device by chris.

## All four sheets — 2026-11-09

chris, word for word, after the Nomad's results
(`docs/device-measurements/supernote.a6x2/calibration.md`): "I also can report that the same preferences and
findings on the nomad hold true for the manta. I think there displays are very similar."

So the Manta takes the Nomad's values and findings, sheet by sheet: the lines, dots, text and writing (set 2) sheets
read the same on both. Both are 300 PPI, so the same pixel sizes are the same physical sizes.

| role | width | gray | code |
|---|---|---|---|
| Whisper | 1 px | 119 | LA8 |
| Guide | 1 px | 51 | LA4, W1 |
| Structure | 2 px | 51 | LB4 |
| Emphasis | 3 px | 0 | LC1 |
| Dot grid | 3 px square | 51 | W15 |
| Text | — | 51 | TD2, TL2 (9 pt) |
| Label | — | 153 | TD4, TL4 (9 pt) |

The Nomad's findings that hold here too: square dots, not round; 1 px dots invisible and 2 px dots too small to use;
square dots speckle from gray 170; lines do not speckle; every text sample legible.

Not recorded separately for the Manta: per-code results (last visible gray per row, and so on). If the two screens
turn out to differ, run the sheets again and record them here.

## Grays — ruled equal to the Nomad, 2026-11-12

Not measured on the Manta. chris, asked whether to read the Manta's grays sheet or treat it as the Nomad: "Same as
the Nomad". So the Manta's profile (`crates/profiles/data/supernote.a5x2.ron`) takes the Nomad's reading
(`docs/device-measurements/supernote.a6x2/calibration.md`, "Grays sheet — 2026-11-12"): all sixteen even steps clean,
0–68 black and 238–255 white on a flat area, nine distinct middle grays 85–221; Shade 204, Block 102. If the screens
turn out to differ, run the grays sheet here and record it.
