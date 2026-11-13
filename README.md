# Quadrule

[![AI framework: pitmaster](https://img.shields.io/badge/AI_framework-pitmaster-yellow)](https://github.com/MapleSyrupOS/pitmaster)
[![CI: fmt, clippy, tests](https://github.com/tholent/quadrule/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/tholent/quadrule/actions/workflows/ci.yml)
[![Fuzz](https://github.com/tholent/quadrule/actions/workflows/fuzz.yml/badge.svg?branch=main)](https://github.com/tholent/quadrule/actions/workflows/fuzz.yml)
[![Coverage](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Ftholent%2Fquadrule%2Fbadges%2Fcoverage.json)](https://github.com/tholent/quadrule/actions/workflows/coverage.yml)

Design pixel-perfect e-ink page templates for Supernote tablets.

Quadrule is a free, open-source vector design tool. You draw lines, grids, boxes and text on a page, and it exports
a PNG at your device's exact screen size, ready to copy to the tablet and use as a note template.

> **Pre-alpha.** Quadrule runs, but it is not finished: you can start a blank page and draw lines, text boxes and
> pattern regions on it, but there are no other shapes yet, no template gallery and no wallpaper mode. Files and menus may change without warning.

## Why it exists

A template that looks fine on a monitor can look muddy, uneven or too busy on an e-ink screen. Lines a pixel too
wide, grays the panel renders badly, or a grid that sits under the toolbar all get in the way of writing. Quadrule
is built around the e-ink screen so that "looks good on the tablet" is the default, not something you tune by trial
and error.

## What makes it e-ink-aware

- **Device profiles.** Each Supernote model is a profile: its exact resolution and pixel density, and the areas the
  tablet's own toolbar and footer cover. The built-in profiles are the Manta (A5 X2, 1920 × 2560) and the Nomad
  (A6 X2, 1404 × 1872).
- **Exact previews.** The editor shows the same pixels the export will write, per device, with a pixel grid and a
  real-size view.
- **Line roles.** Instead of picking a width and a gray for every line, you give it a role (Whisper, Guide,
  Structure, Emphasis, plus roles for dot grids and labels). Each profile turns a role into the width and gray that
  were measured to look right on that device.
- **Recommends, never blocks.** A checker (the linter) lists what could go wrong, such as an object under the
  toolbar, and says why. You can always ignore it and export.
- **A rhythm.** Each page has a spacing unit, and objects snap to it and to its subdivisions, so grids and ruled
  pages come out even.
- **Snapping and guides.** Dragging snaps to the rhythm, the margins, the pixel grid, other objects' corners,
  edges and centres, and to equal gaps; smart guides show what it caught.
- **Calibrated on real devices.** The values come from calibration sheets printed on a Manta and a Nomad. The
  sheets, the method and the screenshots are in this repository (`docs/calibration.md`,
  `docs/device-measurements/`).

## What works today

- Start a blank page (New…, Ctrl+N): pick the device (Manta or Nomad), a rhythm (handwriting 6 mm, narrow, college
  or wide ruled, a 5 mm or 1/4 in grid, or an isometric rhythm with a 5 mm side) and a title. The margins clear the tablet's docked toolbars and footers.
- Open a `.quad` file (Quadrule's own format) or the built-in sample page, and save changes.
- Draw with the tools bar on the left: a line (L), a text box (T), a pattern region (P), a rectangle (R), an ellipse
  (E) or a polygon (Y); V goes back to Select. Drag to draw (sizes snap to whole rhythm units), or click for a default
  size; Shift draws a square, a circle or a regular polygon; double-click a tool to keep it on.
- Shapes have a stroke (a role, custom values, or none), a fill (Shade, Block or a gray of your own), a dash (dotted,
  short dash, dash, long dash or your own lengths) and a cap (butt, round or square), set in the inspector's Style tab.
  A rectangle has a corner radius (in any unit, rhythm units included); a polygon has 3 to 32 sides and can be a star
  with an inner radius. Curves and slanted edges are smooth; straight edges stay sharp on whole pixels.
- Select, move, resize, duplicate, copy and paste the objects on a page; rename, hide, lock and reorder layers;
  unlimited undo and redo; autosave with recovery after a crash.
- Export a PNG for the Manta or the Nomad, named like `{title}_{manta|nomad}_{w}x{h}.png`.
- Check a page with the linter (nine rules so far).
- Turn a line or a shape with the round handle above its top edge (it catches at 0, 90, 180 and 270 degrees; Shift
  holds 15-degree steps) or type an angle in the Arrange tab; a turned shape resizes along its own sides. Lines point any
  way (draw one up and to the right, or drag its ends past each other). Opacity, 0 to 100 percent, is in the Style tab;
  the export is flattened to gray, with no transparency. A turned edge is smooth; a quarter turn keeps sharp edges.
  Text boxes and pattern regions do not turn yet.
- Group objects (Ctrl+G, or the Group button; Ctrl+Shift+G or Ungroup breaks it up). A group moves, turns, resizes and
  fades as one. Grouping never changes how a page looks on either device: as in Figma, a group is see-through to
  constraints, so each child keeps its own constraints against the page (a group has none, and no Constraints tab), and
  the group's box is always the box round its children. Resizing a group stretches everything in it, as resizing a
  loose selection does. Groups nest as deep as you like. A click picks the whole group; double-click goes
  one level in (Escape or a click outside steps back out), and Ctrl+click picks the object inside at once. The Layers
  panel is a tree with fold arrows, and rows can be dragged into and out of groups. A group with a text box or a pattern
  region in it cannot turn.
- Align and distribute, in the Arrange tab: six align buttons (left edges, horizontal centres, right edges, tops,
  vertical middles, bottoms) and four distribute buttons (horizontally and vertically, by spacing or by centres; three
  or more objects). "Align to" is the selection when two or more objects are picked and the page when one is, switching by
  itself; pick Page, Margins or Selection and it remembers your pick for one object and for several. A turned object
  lines up by the box round its turned corners, a group as one object, and a locked object stays where it is and is left out
  of the selection's box. With the pixel grid snap on, results land on whole device pixels (the gaps equal to the pixel,
  any leftover pixel going to the first gaps); with it off they are exact millimetres. Each is one undo step. When
  several objects are selected and their position, size or rotation differ, the Arrange field reads "mixed"; typing a
  value sets it on all of them.
- Several devices on one page. The design is made on the primary device (the first one); the other follows it through
  each object's constraints. With a secondary device on screen, **position, size, font size and the eye** become
  *overrides* for that device only (a drag, the arrow keys, align and distribute, the Arrange and Text fields, the
  layers eye); every other edit (style, fill, text, rotation, constraints) still changes the design on all devices, and
  the status bar says so. A group moved or resized on a secondary device gets one override, its box there; what is in
  it is scaled into that box. A small dot marks a changed value and a changed layer row, and an **Overrides** tab lists
  what differs on the device on screen, with a Reset for each and "Reset all on this device". A **previews strip** along
  the bottom of the canvas shows each device's page (the one on screen outlined, with how many objects are changed on
  it); click a picture to put that device on screen, or fold the strip shut. The Page tab lists the devices: tick them
  in and out (taking one out asks first if it has changes), and **Make primary** makes a device's layout the design
  without changing a pixel on any device. The linter's L15 notes where constraints alone put an object off the page, on
  top of another object or squeezed thin on a secondary device. Files are saved as format 0.9.0; older files open
  unchanged.
- Landscape pages. Pick **Portrait** or **Landscape** in New… or in the Page tab (one undo step: objects follow their
  own constraints onto the turned page). A landscape page is designed upright on screen and exported turned a quarter
  turn clockwise into the tablet's ordinary portrait PNG, the design's top along the tablet's right edge: read it in
  portrait mode with the tablet's right edge up. The tablet's toolbars and footers are shown, snapped to and linted
  where they fall on the turned page (its left toolbar along the design's bottom, and so on).
- A pattern library. A pattern region can be ruled lines, dots, a square grid, **isometric** (the triangle grid, as
  lines or as dots), **hexagons** (flat top or pointy top, a whole number of rhythm units high), a **Cornell** page (a
  cue column and a summary band) or **music staves** (five lines with a gap of their own, one staff every N rhythm
  units, optionally paired as a grand staff with a brace line). Each pattern's own settings are in the inspector's
  Style tab under the pattern choice. The rhythm has a **Type** in the Page tab: Rectangular, or Isometric with one
  side length, which snaps, nudges (Shift+arrow) and sizes drawn shapes on half a side across and one row down, and
  whose overlay draws the triangles. The rhythm also has **Major every N** (off by default): the pattern regions that
  follow the rhythm draw every Nth line, row or dot in a stronger role (Structure unless you change it), a region can turn
  them off or use its own N, and the overlay draws them stronger. A region can be **unbound** from the rhythm to have
  spacing of its own (the linter's L17 says when that spacing is not a whole multiple or fraction of the rhythm). The
  pattern page opens from the command line with `--patterns-page`.

Not yet: the starter gallery and wallpapers. The plan is in the design spec,
`napkins/quadrule-spec.md` §16. Day-to-day progress is in [STATUS.md](STATUS.md).

## Download

Installers for macOS, Windows and Linux are attached to each release on the
[releases page](https://github.com/tholent/quadrule/releases): a `.dmg` for macOS (Apple chip or Intel), an `.msi`
for Windows, and an AppImage or a `.deb` for Linux on x86-64 or ARM. Every release so far is a pre-release.

The macOS and Windows packages are not signed yet, so the system warns the first time you open one:

- **macOS:** open the app once and close the warning, then go to System Settings → Privacy & Security and click
  **Open Anyway** next to the message about Quadrule.
- **Windows:** when SmartScreen says it protected your PC, click **More info**, then **Run anyway**.
- **Linux AppImage:** mark the file as a program first (`chmod +x quadrule-*.AppImage`, or tick "Allow executing"
  in its properties), then open it.

## Build from source

You need [Rust](https://rustup.rs) (the repository pins the exact version in `rust-toolchain.toml`, and `rustup`
installs it for you). Then, from a clone of this repository:

```
cargo run --release -p quadrule-app
```

That starts empty (New… or "Open sample page" begin). To see every shape, fill, dash and cap at once, add
`-- --shapes-page` (or `-- --transforms-page` for turned shapes, lines every way and opacity, or `-- --patterns-page` for every pattern in the pattern library). To open a file: `cargo run --release -p quadrule-app -- path/to/page.quad`. Quadrule is
built and tested on Linux, macOS and Windows. Setup details, tests and the code layout are in [DEVELOPMENT.md](DEVELOPMENT.md).

## Putting a template on your Supernote

Export a PNG, then copy it to the tablet's `MyStyle` folder. Files in a subfolder of `MyStyle` appear under the
Customization tab when you pick a template in a note. Use a plain filename of letters, digits, hyphens and
underscores; Supernote advises against special characters.

## Help and feedback

Bug reports and ideas are welcome as [issues](https://github.com/tholent/quadrule/issues). Please open an issue
before sending a pull request, because the design is still settling. If you own a Supernote, measurements from your
device are especially useful; [DEVELOPMENT.md](DEVELOPMENT.md) says how to take them.

## Licence

Apache-2.0; see [LICENSE](LICENSE).

Quadrule is an independent project and is not affiliated with or endorsed by Ratta or Supernote.
