# Landscape template test — Nomad, 2026-11-12

Answers spec §17.1 item 3: does the device expect rotated pixels or swapped dimensions for a landscape template?

## The test pages

Made by the rung 7 lead with a one-off script (not in the repository), 8-bit gray PNGs, each with a 3 px black border on
its edge pixels, corner labels, a large arrow marked UP, and two patches of 1-pixel stripes (crisp only when the file is
shown pixel for pixel):

- **Test A:** a landscape design saved with wide pixels, 1872 × 1404.
- **Test B:** the same design turned a quarter turn clockwise, saved with tall pixels, 1404 × 1872.

## chris's readings, word for word

Test A, first report: "testA: 1. doesn't fill all screen, though all content is visible (zoomed to fit width) 2. arrow
points up, text is upright 3. stripes are grey with splotches".

Asked which device and orientation: "nomad, test a while device is in portrait mode. when rotated to landscape the
template fills more of the screen, but the scroll bars are present". Asked where the empty space was: "Bands top and
bottom".

Test B: "when in portrait mode test b points to the right, but fills the screen. when the device is rotated the arrow
points right and the page is zoomed in so that not all the image is visible".

## The landscape screenshots agree

The grid in the landscape toolbar screenshots (`landscape/`, and the Manta's in
`../supernote.a5x2/landscape/`) is 79 px apart on both devices, against 59 px in portrait (measured 2026-11-12 by
scanning a middle row of `nomad-top.png`, `manta-landscape-top.png` and `nomad-landscape-top.png` for line starts; the
median gap). 59 × 4/3 = 78.7: in landscape the device shows the portrait page zoomed by 4/3 to fit the screen's new
width, and scrolls it. So those screenshots show a zoomed view, not template pixels.

## What it means

- A template is always a portrait page of the screen's pixel size. There is no landscape page.
- A wide PNG (Test A) is shrunk to fit the page width: soft, never pixel-exact.
- A **landscape design must be turned a quarter turn and saved as a portrait PNG** (Test B). It is pixel-exact in
  portrait mode, read with the tablet held sideways. In landscape mode the device zooms and scrolls it, so it is never
  pixel-exact there either.
- Which way to turn it (Test B was turned clockwise, so the tablet is held with its right edge up) is for rung 8 to
  decide; so is whether the landscape safe-zone screenshots are needed at all, since a sideways template is used in
  portrait mode, where the portrait safe zones apply.

The Manta: ruled the same as the Nomad, not tested (chris, 2026-11-12: "Same as the Nomad"). Its landscape
screenshots show the same 4/3 zoom (grid 79 px against 59 px).
