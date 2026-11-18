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
  (A6 X2, 1404 × 1872). The **Profiles window** (a button beside the device ticks in the Page tab, and one in the
  toolbar) shows the values each device is drawn with and lets you change them: an edited Manta or Nomad is a local
  override marked **Edited** (Reset to built-in puts the shipped values back), and New…, Duplicate and Import… make
  profiles of your own for other screens. They live in `profiles/` in the config folder (on Linux
  `~/.config/quadrule/profiles/`, one `.ron` file each; Export… writes the same file to share). A `.quad` file carries
  every profile its page uses that is not an unedited built-in, so it opens on a computer that lacks it ("Uses a
  profile you don't have: X. Add to my profiles"), and a profile that clashes with one you have asks Merge, Rename or
  Not now. A damaged profile file is skipped and listed, never a crash.
- **Paper sizes.** For printing, or a screen Quadrule has no profile for, eleven paper sizes are built in (ISO A3 to
  A6, ISO and JIS B5, US Letter, US Legal, Tabloid, Half Letter and Executive), listed as their own group after the
  devices in New… and in the Page tab, and under Paper in the Profiles window. A paper size is a physical size in
  millimetres; each page picks its export resolution (150, 200, 300 or 600 PPI, beside the ticked size in the Page
  tab) and the PNG is that size in pixels (A4 at 300 PPI is 2480 × 3508). Line widths are the Nomad's, by physical
  size. A paper size can be the primary profile or a secondary one. A landscape paper page exports a wide PNG, not
  turned. A paper size has no safe zones or gray levels, so the lint rules that need them (L03 to L06, L13 and L14) do not run on
  it; the text-size and handwriting rules (L07, L09) do. They are marked Unverified: they are not measured on paper.
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

- Start from the **welcome screen**, which shows when nothing is open, after File ▸ Close (Ctrl+W), and for File ▸ New…
  (Ctrl+N) over an open page (Esc or Cancel goes back to the page). Down its left side: **Blank page** (pick the device,
  a rhythm (handwriting 6 mm, narrow, college or wide ruled, a 5 mm or 1/4 in grid, or an isometric rhythm with a 5 mm
  side) and a title; the margins clear the tablet's docked toolbars and footers), **Open…**, the **Recent** files
  (the last ten opened or saved; one that has gone from the disk is greyed "Not found") and a small "Open sample page"
  link. On its right: tabs (All, Paper, Notes, Planning, Bullet journal, Creative, My templates), a Device filter and a
  Search box, over tiles drawn live for the chosen device (a page that does not target that device is drawn on its own
  main device, and its tile says so). The starter pages are coming; until then the tabs say so. Dropping a `.quad` on it opens it.
- **My templates**: File ▸ Save as template… keeps a copy of the page under My templates (a `templates` folder beside
  the autosave recovery folder in the app's data folder). Picking one opens an untitled copy on the Device filter's
  device; right-click a template to Remove it (it asks first, and removing cannot be undone) or Show in folder.
- Open a `.quad` file (Quadrule's own format) or the built-in sample page, and save changes. Everything for files is in
  the **File** menu, leftmost in the top bar: New…, Open…, Open Recent, Save, Save As…, Save as template…, Export… and
  Close.
- Draw with the tools bar on the left: a line (L), a text box (T), a pattern region (P), a rectangle (R), an ellipse
  (E) or a polygon (Y); V goes back to Select. Drag to draw (sizes snap to whole rhythm units), or click for a default
  size; Shift draws a square, a circle or a regular polygon; double-click a tool to keep it on.
- Shapes have a stroke (a role, custom values, or none), a fill (Shade, Block or a gray of your own), a dash (dotted,
  short dash, dash, long dash or your own lengths) and a cap (butt, round or square), set in the inspector's Style tab.
  A rectangle has a corner radius (in any unit, rhythm units included); a polygon has 3 to 32 sides and can be a star
  with an inner radius. Curves and slanted edges are smooth; straight edges stay sharp on whole pixels.
- Select, move, resize, duplicate, copy and paste the objects on a page; rename, hide, lock and reorder layers;
  unlimited undo and redo; autosave with recovery after a crash.
- Export PNGs for the devices a page targets. **File ▸ Export…** (Ctrl+E; Cmd on a Mac) opens the
  **Export sheet** with the last choices filled in and the Export button in focus, so Ctrl+E then Enter repeats the
  last export: pick This device or All devices, the folder (Change…), the **Path** and a **Set**, and read the list of
  files that will be written (a mistake shows in red and holds Export back). Every choice is remembered. After an export a **completion sheet** shows a picture of each file, where they went,
  Reveal in folder, and a folded "Where do these go?" note about the Supernote's MyStyle folder.
  The **Path** is one template for the whole app. `/` makes a folder, and these words in braces are filled in:
  `{title}`, `{device}` (also `{profile}`), `{model}`, `{width}`, `{height}`, `{orientation}`, `{type}`, `{date}`,
  `{version}` and `{set}`; an empty Set leaves its folder out. Names keep to letters, digits, hyphens and
  underscores, and a path can never leave the chosen folder. It starts as `{title}-{device}.png` (for example
  `isometric-5mm-manta.png`); the Presets menu offers Flat, Folder per device (`{device}/{title}-{device}.png`) and
  Device, then set (`{device}/{set}/{title}-{device}.png`). A file that is already there is replaced; the completion
  sheet says how many were. Each PNG carries a text note (Software `quadrule <version>`, Title the document's
  title) unless "Add a text note" is unticked; the pixels are the same either way. The choices, and the folder each
  document last exported to, are kept in `settings.ron` in the config folder (on Linux `~/.config/quadrule/`),
  never in the `.quad` file.
- Text boxes: double-click one (or draw a new one) to type in it, right where the letters are, with a caret, Shift to
  select, Ctrl+A, Ctrl+C, Ctrl+X and Ctrl+V as plain text, Enter for a new paragraph, Ctrl+Z back through the words you
  typed, and Escape or a click outside to finish (the whole session is one undo step). In the Text tab, Sizing is Auto
  height (the box is as tall as its text, on each device) or Fixed (a height of its own: lines that do not fully fit are
  left out of the page, a red + tab at the corner of the box shows it on the canvas and the device previews, never in
  the export, and lint rule L08 says how many). Fixed boxes can put their text at the top, middle or bottom; paragraph
  spacing is added after each paragraph; "Align baselines to rhythm" makes the line height a whole number of rhythm steps
  and stands every line of type on a rung of the rhythm, as handwriting stands on ruled paper.
- Check a page with the linter (eighteen rules so far). Rung 14a added six, which read numbers each profile carries in a
  **Lint checks** section of the Profiles window (leave a number empty to skip its check on that device): L03 a gray too
  light to show, L05 a gray off the device's clean levels, L07 text under the smallest size that reads, L09 a rhythm or ruled
  region under the smallest comfortable handwriting step, L13 large dark fills, L14 a thin line that is slanted or curved.
  The Lint tab has a device picker (All devices, or one) and says, one line per device, which checks did not run and why.
  Each finding has a button that fixes it in one undo step where there is a fix (snap a stroke to whole pixels, use the
  nearest clean gray or a role that fits, move out of a safe area, use the smallest legible text size, snap to the rhythm
  or bind to it, centre the rhythm origin), and **Ignore this rule for this document**; ignored rules are listed at the
  end of the tab, each with a button to turn it back on. Mark an object **Free placement** in the Arrange tab to
  exempt it from the off-rhythm check. Export opens one **Before you export** sheet when a device has a warning (or the
  page uses fonts installed on this computer): each row can jump to its object, ignore its rule or be fixed, and
  **Export anyway** goes on. Under Custom, the Style tab shows the device's clean grays as swatches (**Show all grays**
  opens the 0 to 255 slider), says which role a stroke matches and offers **Use recommended**.
- Turn a line or a shape with the round handle above its top edge (it catches at 0, 90, 180 and 270 degrees; Shift
  holds 15-degree steps) or type an angle in the Arrange tab; a turned shape resizes along its own sides. Lines point any
  way (draw one up and to the right, or drag its ends past each other). Opacity, 0 to 100 percent, is in the Style tab;
  the export is flattened to gray, with no transparency. A turned edge is smooth; a quarter turn keeps sharp edges.
  Text boxes turn the same way, about the middle of the box. So do pattern regions: the frame and its ruling turn
  together, the ruling staying parallel to the frame's edges, and lines are cut at the turned frame; a group holding a
  pattern region can turn too. A region also has a Pattern angle (in the Style tab, beside the spacing) that turns the ruling inside the frame, cut at
  the frame, and adds to the frame's own turn. A turned ruling is softer than an upright one (lint rule L02 says so).
  A pattern region can also have a **Border** (Style tab): a line centred on the frame's edge, on the sides you tick, in a
  line role (Structure to start) or a width and gray of its own, with an optional corner radius. On a bordered side the
  pattern's own lines and dots that sit on the edge are left out, so nothing doubles up; a corner rounds only where both its
  sides are on, and the ruling stops at the curve. The border turns with the frame, not with the Pattern angle. A border is
  the same on every device (per-device overrides do not cover it yet); lint judges its line like any other.
- Group objects (Ctrl+G, or the Group button; Ctrl+Shift+G or Ungroup breaks it up). A group moves, turns, resizes and
  fades as one. Grouping never changes how a page looks on either device: as in Figma, a group is see-through to
  constraints, so each child keeps its own constraints against the page (a group has none, and no Constraints tab), and
  the group's box is always the box round its children. Resizing a group stretches everything in it, as resizing a
  loose selection does. Groups nest as deep as you like. A click picks the whole group; double-click goes
  one level in (Escape or a click outside steps back out), and Ctrl+click picks the object inside at once. The Layers
  panel is a tree with fold arrows, and rows can be dragged into and out of groups. A group with a pattern region (or a repeat)
  in it cannot turn; a group with text in it can.
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
  top of another object or squeezed thin on a secondary device. Files are saved as format 0.17.0 (0.16 added a small picture of the main device's page
  inside the file, written at Save and Save As so a gallery can show the file; 0.17 adds the pattern region's border); older files
  open unchanged and get the picture at their next save.
- Landscape pages. Pick **Portrait** or **Landscape** in New… or in the Page tab (one undo step: objects follow their
  own constraints onto the turned page). A landscape page is designed upright on screen and exported turned a quarter
  turn clockwise into the tablet's ordinary portrait PNG, the design's top along the tablet's right edge: read it in
  portrait mode with the tablet's right edge up. The tablet's toolbars and footers are shown, snapped to and linted
  where they fall on the turned page (its left toolbar along the design's bottom, and so on).
- A pattern library. A pattern region can be ruled lines, dots, a square grid, **isometric** (the triangle grid with vertical lines, as
  lines or as dots), **hexagons** (flat top or pointy top, a whole number of rhythm units high), a **Cornell** page (a
  cue column and a summary band) or **music staves** (five lines with a gap of their own, one staff every N rhythm
  units, optionally paired as a grand staff with a brace line). Each pattern's own settings are in the inspector's
  Style tab under the pattern choice. The rhythm has a **Type** in the Page tab: Rectangular, or Isometric with one
  side length, which snaps, nudges (Shift+arrow) and sizes drawn shapes on one column across and half a side down, and
  whose overlay draws the triangles. The rhythm also has **Major every N** (off by default): the pattern regions that
  follow the rhythm draw every Nth line, row or dot in a stronger role (Structure unless you change it; a major dot is also one pixel bigger than an ordinary one), a region can turn
  them off or use its own N, and the overlay draws them stronger. A region can be **unbound** from the rhythm to have
  spacing of its own (the linter's L17 says when that spacing is not a whole multiple or fraction of the rhythm). The
  pattern page opens from the command line with `--patterns-page`.
- A **repeat** draws one object, or a group, again in a row, a column or a grid (the **Repeat** button beside Group and
  Ungroup). It is live: edit the first one and every copy follows, and **Expand** in the Repeat tab turns the copies into
  plain objects in one group. A repeat counts a set number of copies, or **fills** its box with as many as fit (so a box
  pinned to the margins gives each device its own count; drag its handles to add or take away copies). The step between
  copies is **Auto** by default (the object's size rounded up to whole rhythm steps), or a pitch (centre to centre) or a
  gap (edge to edge) in `u` (follows the rhythm) or any length, and it is worked out in whole device pixels, so every
  copy is the first one moved by whole pixels. Copies sit from the start of the box or are centred. **Alternation rules**
  pick copies by column, row or cell ("every 7th", or a list like "3, 7") and skip them, or give them another line or
  fill role. Click any copy to pick the whole repeat; double-click goes into it to edit the source; Escape comes back out.
  A repeat draws at most 5,000 copies. The linter judges every copy for the safe areas and the page edge (one finding
  per repeat) and the pitch for the rhythm. The repeat page opens from the command line with `--repeats-page`.

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

That starts on the welcome screen (Blank page or "Open sample page" begin). To see every shape, fill, dash and cap at once, add
`-- --shapes-page` (or `-- --transforms-page` for turned shapes, lines every way and opacity, or `-- --patterns-page` for every pattern in the pattern library, or `-- --pattern-angle-page` for the ruling turned inside its frame, or `-- --bordered-page` for borders round pattern regions, or `-- --lint-page` for objects that make each of the gray, text-size, handwriting, dark-fill and thin-line lint rules fire beside ones that pass, or `-- --paper-page` for ISO A4 at 300 PPI and ISO A5 at 600 (`-- --paper-landscape-page` for US Letter held sideways), or `-- --turned-patterns-page` for pattern regions turned whole, one in a group and one in a repeat, or `-- --repeats-page` for rows, grids and a Fill repeat, or `-- --fonts-page` for every bundled font, or `-- --textbox-page` for fixed boxes, vertical alignment, baselines on a ruled region and turned text). To open a file: `cargo run --release -p quadrule-app -- path/to/page.quad`. Quadrule is
built and tested on Linux, macOS and Windows. Setup details, tests and the code layout are in [DEVELOPMENT.md](DEVELOPMENT.md).

## Putting a template on your Supernote

Export a PNG, then copy it to the tablet's `MyStyle` folder. Files in a subfolder of `MyStyle` appear under the
Customization tab when you pick a template in a note. Quadrule names files with letters, digits, hyphens and
underscores only, because Supernote advises against special characters.

## Help and feedback

Bug reports and ideas are welcome as [issues](https://github.com/tholent/quadrule/issues). Please open an issue
before sending a pull request, because the design is still settling. If you own a Supernote, measurements from your
device are especially useful; [DEVELOPMENT.md](DEVELOPMENT.md) says how to take them.

## Fonts

Quadrule ships these fonts, all under the [SIL Open Font License 1.1](https://openfontlicense.org); each licence text is in the
`licenses` folder of every package and in `crates/text/fonts/`:

- [Atkinson Hyperlegible Next](https://github.com/googlefonts/atkinson-hyperlegible-next) and [Atkinson Hyperlegible Mono](https://github.com/googlefonts/atkinson-hyperlegible-next-mono), with italics (The Atkinson Hyperlegible Next Project Authors)
- [Inter](https://github.com/rsms/inter), with italic (The Inter Project Authors)
- [Literata](https://github.com/googlefonts/literata), with italic (The Literata Project Authors)
- [Caveat](https://github.com/googlefonts/caveat) (The Caveat Project Authors)
- [Noto Emoji](https://github.com/googlefonts/noto-emoji), for the monochrome emoji (Google LLC)

The fonts are the variable files from [google/fonts](https://github.com/google/fonts). Fonts installed on your computer
can be picked too; a character the chosen font lacks is always taken from the bundled fonts, so a design looks the same
on every machine.

The emoji picker's names and groups come from the [`emojis`](https://github.com/rossmacarthur/emojis) crate (MIT or
Apache-2.0), which carries Unicode data under the [Unicode License v3](crates/ui/licenses/Unicode-3.0.txt); the notice is
in the `licenses` folder of every package.

## Licence

Apache-2.0; see [LICENSE](LICENSE).

Quadrule is an independent project and is not affiliated with or endorsed by Ratta or Supernote.
