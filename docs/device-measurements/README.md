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
them, so their edges show exactly where the lines stop.

**Measured so far (2026-11-09):** portrait only, toolbar docked left, right and top, and floating at its default
place, for `supernote.a5x2` and `supernote.a6x2`.

**Still to capture:** landscape, a collapsed toolbar, and any other floating positions.
