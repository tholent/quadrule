# Device measurements

Screenshots and photos taken on real devices, kept so measured values can be checked again after a firmware update
(napkin §3.5, §8.4). One folder per profile id. Everything here is public.

## Safe zones (napkin §3.5)

**Taking the screenshots.** On the device, open a note whose template is a plain grid (the built-in 5 mm grid works)
and take one screenshot per toolbar position. Name each file `<anything>-<position>.png`, where position is `left`,
`right` or `top` (docked) or `float` (the floating toolbar where it first appears). Copy them, unedited and at native
resolution, into `docs/device-measurements/<profile_id>/`. Check they show nothing personal.

**Measuring.** `cargo xtask safe-zones` (or `--profile <id>`) reads them and prints the zones as RON for the
profile file (`crates/profiles/data/<profile_id>.ron`), noting when they differ from what the file holds. A test
(`crates/xtask/tests/safezones.rs`) re-measures the screenshots on every run and fails if the profile files disagree.

**How it works.** The grid lines' positions are read from the middle of the page and the bare template is rebuilt
from them. Every pixel that differs from the bare template is covered. What every screenshot covers is the footer
(page navigation and note name); the rest of each screenshot is its toolbar. Cover boxes hide the grid lines under
them, so their edges show exactly where the lines stop. A docked toolbar's zone runs the full length of its
edge, because its length depends on which icons the user has chosen; only its thickness is measured.

**Measured so far (2026-11-09):** portrait only, toolbar docked left, right and top, and floating at its default
place, for `supernote.a5x2` and `supernote.a6x2`.

**Landscape, captured 2026-11-12 (chris), not measured yet:** toolbar docked left, right and top on both devices, in
`<profile_id>/landscape/` (Manta 2560 × 1920, Nomad 1872 × 1404; no floating shot). They sit in a subfolder because
`cargo xtask safe-zones` reads only portrait shots from the device folder itself; rung 8 teaches it landscape and
measures them. In landscape a scroll bar shows down the right edge, which portrait does not have. **These are zoomed views:** the grid in them
is 79 px apart against 59 px in portrait (× 4/3), because the device shows the portrait page zoomed to fit the width in
landscape (`supernote.a6x2/landscape-test.md`), so their zones are not in template pixels.

**Still to capture:** a collapsed toolbar, other floating positions, and a floating toolbar in landscape.

## What Supernote's stock 5 mm grid shows (2026-11-09)

Read from the screenshots above, which use the stock 5 mm grid template (chris). `cargo xtask safe-zones` prints the
spacing, line width and placement it finds; the line gray and the page gray were read with a one-off check.

| | Manta (1920 × 2560) | Nomad (1404 × 1872) |
|---|---|---|
| spacing | 59 px both ways | 59 px both ways |
| lines | 32 vertical, 43 horizontal | 23 vertical, 31 horizontal |
| first line | x 45, y 41 | x 52, y 50 |
| space left / right | 45 / 45 px | 52 / 53 px |
| space top / bottom | 41 / 40 px | 50 / 51 px |
| line | 1 px, gray 1, hard edges | 1 px, gray 1, hard edges |
| page | gray 255 | gray 255 |

- **Pixel-locked.** "5 mm" is 59 px on both: 5 mm at 300 PPI is 59.06 px, rounded to whole pixels (4.995 mm). That is
  napkin §6.3's pixel-locked rhythm, and it agrees with both devices being 300 PPI (napkin §3.3).
- **Centred.** The leftover space is split evenly on both sides of each axis: napkin §6.3's "centered" rhythm origin,
  which quadrule does not build yet.
- **Shown 1:1.** Every line is exactly 1 px with pure white beside it, so the template reaches the screen with no
  scaling or smoothing, and screenshot pixels map 1:1 to template pixels (napkin §3.5).
- **Gray 1, not 0.** The lines are 1, one step off black. Why is not known; one guess is that it keeps the template
  apart from pen ink at 0. Worth checking before `ink_values` is filled in. *Checked 2026-11-09 (pen shades below): the black pen also writes gray 1, so the guess is wrong; the stock grid
  is drawn in the black pen's own gray.*
- **Full bleed.** The grid runs under the toolbar and footer; the stock template ignores safe zones.
- **Not shown:** how the panel itself renders grays, or how the lines look to the eye. Screenshots are the screen's
  pixel data; calibration still needs the sheets and eyes.

## quadrule draws the stock grid (2026-11-09)

`crates/xtask/tests/stock_grid.rs` builds the stock template as a quadrule document (one full-page square grid,
5 mm, centred, pixel-locked, 1 px at gray 1), exports it, and compares it pixel by pixel with every screenshot
outside the footer and that screenshot's toolbar:

- **Manta:** identical, 0 differing pixels in all four screenshots.
- **Nomad:** identical once shifted 1 px up and left. On each axis the stock grid cannot sit exactly in the middle
  (an odd pixel is left over), and the two stock templates break that tie in opposite directions: no single rule
  matches both. quadrule's centred rhythm gives the odd pixel to the right and bottom, which matches the Manta.

## Pen shades (2026-11-09)

chris wrote one stroke with each pen color on the Nomad and took a screenshot:
`supernote.a6x2/nomad-pen-shades.png`. Each stroke is one flat gray with soft edges:

| pen color | gray |
|---|---|
| black | 1 |
| dark grey | 157 |
| light grey | 201 |
| white | 254 |

These are the profiles' `ink_values` (napkin §3.2), which the "competes with ink" lint rules use. The Manta's are
assumed the same (pen colors come from the firmware) until a Manta screenshot confirms them.
`crates/xtask/tests/pen_shades.rs` re-measures the screenshot on every run and fails if a profile disagrees.

Like the safe zones, this is the screen's pixel data, not how the panel shows it.
