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
  or wide ruled, a 5 mm or 1/4 in grid) and a title. The margins clear the tablet's docked toolbars and footers.
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
- Check a page with the linter (seven rules so far).
- Turn a line or a shape with the round handle above its top edge (it catches at 0, 90, 180 and 270 degrees; Shift
  holds 15-degree steps) or type an angle in the Arrange tab; a turned shape resizes along its own sides. Lines point any
  way (draw one up and to the right, or drag its ends past each other). Opacity, 0 to 100 percent, is in the Style tab;
  the export is flattened to gray, with no transparency. A turned edge is smooth; a quarter turn keeps sharp edges.
  Text boxes and pattern regions do not turn yet.
- Group objects (Ctrl+G, or the Group button; Ctrl+Shift+G or Ungroup breaks it up). A group moves, turns, resizes and
  fades as one: its children keep their own constraints against the group's box, so a child pinned to the top and right
  keeps its size when the group grows. Groups nest as deep as you like. A click picks the whole group; double-click goes
  one level in (Escape or a click outside steps back out), and Ctrl+click picks the object inside at once. The Layers
  panel is a tree with fold arrows, and rows can be dragged into and out of groups. A group with a text box or a pattern
  region in it cannot turn.

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
`-- --shapes-page` (or `-- --transforms-page` for turned shapes, lines every way and opacity). To open a file: `cargo run --release -p quadrule-app -- path/to/page.quad`. Quadrule is
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
