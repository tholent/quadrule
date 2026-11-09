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

**Still to capture:** landscape, a collapsed toolbar, and any other floating positions.

## What Supernote's stock 5 mm grid shows (2026-11-09)

Read from the screenshots above, which use the stock 5 mm grid template (chris); script-free check:
`cargo xtask safe-zones` reports the same grid it rebuilds.

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
  apart from pen ink at 0. Worth checking before `ink_values` is filled in.
- **Full bleed.** The grid runs under the toolbar and footer; the stock template ignores safe zones.
- **Not shown:** how the panel itself renders grays, or how the lines look to the eye. Screenshots are the screen's
  pixel data; calibration still needs the sheets and eyes.
