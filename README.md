# Quadrule

[![AI framework: pitmaster](https://img.shields.io/badge/AI_framework-pitmaster-yellow)](https://github.com/MapleSyrupOS/pitmaster)
[![CI: fmt, clippy, tests](https://github.com/tholent/quadrule/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/tholent/quadrule/actions/workflows/ci.yml)
[![Fuzz](https://github.com/tholent/quadrule/actions/workflows/fuzz.yml/badge.svg?branch=main)](https://github.com/tholent/quadrule/actions/workflows/fuzz.yml)
[![Coverage](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Ftholent%2Fquadrule%2Fbadges%2Fcoverage.json)](https://github.com/tholent/quadrule/actions/workflows/coverage.yml)

Design pixel-perfect e-ink page templates for Supernote tablets.

Quadrule is a free, open-source vector design tool. You draw lines, grids, boxes and text on a page, and it exports
a PNG at your device's exact screen size, ready to copy to the tablet and use as a note template.

> **Pre-alpha.** Quadrule runs, but it is not finished, and you cannot design a page from scratch in it yet: today it
> opens and edits an existing page and exports it. There is no New page, no way to add shapes or text, no template
> gallery, no wallpaper mode, and no prebuilt download (you build it from source, see below). Files and menus may
> change without warning.

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

- Open a `.quad` file (Quadrule's own format) or the built-in sample page, and save changes.
- Select, move, resize, duplicate, copy and paste the objects on a page; rename, hide, lock and reorder layers;
  unlimited undo and redo; autosave with recovery after a crash.
- Export a PNG for the Manta or the Nomad, named like `{title}_{manta|nomad}_{w}x{h}.png`.
- Check a page with the linter (seven rules so far).

Not yet: creating a new page, adding objects, the starter gallery and wallpapers. The plan is in the design spec,
`napkins/quadrule-spec.md` §16. Day-to-day progress is in [STATUS.md](STATUS.md).

## Try it

You need [Rust](https://rustup.rs) (the repository pins the exact version in `rust-toolchain.toml`, and `rustup`
installs it for you). Then, from a clone of this repository:

```
cargo run --release -p quadrule-app
```

That opens the sample page. To open a file: `cargo run --release -p quadrule-app -- path/to/page.quad`. Quadrule is
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
