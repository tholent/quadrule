# STATUS-debt — quadrule

Known debt and owed work. One line each. When an item is paid, remove it and note it in `STATUS-log.md`.

- A lint/preflight rule for system-font use, at save and at export, is owed to the napkin's lint list (ADR-002).
- Text spike T14/T15, what is left: system-font discovery on macOS and Windows. Layout and outlines with the bundled
  font now match byte for byte there (CI run 42660403429); system fonts are not loaded yet (`crates/text/src/lib.rs`).
- Parked for chris: rendering — Scale vs pixel-locked rhythm on A5 X; the build-folder instruction the render worker
  reports (`spikes/render/FINDINGS.md`). (Canvas fallback to vello_cpu on weak GPUs: ruled yes, 2026-11-09.)
- Calibration: the Manta's gray levels are ruled equal to the Nomad's, not measured (chris, 2026-11-12: "Same as the
  Nomad"; `docs/device-measurements/supernote.a5x2/calibration.md`); the Manta's pen shades likewise (arc v1 bbq Q8).
- The `gh` token expires 90 days after it was made (chris, 2026-11-09); if made 2026-11-09, that is about 2027-02-07.
  Renew it before then, or `gh` stops opening PRs and reading CI (rung 1 record, Defaulted "Repository").
- The `gh` fine-grained token cannot read check summaries (`gh pr checks` fails: GitHub offers no `checks:read` for
  fine-grained tokens); read CI with `gh run view <id> --log` instead.
- ADR build status is in each ADR's `built` and `owed` header fields (`docs/decisions/INDEX.md`): ADR-001 and
  ADR-002 partial, ADR-003 landed; of the proposed ADR-004 to ADR-014, ADR-009, ADR-011 and ADR-012 are partial.
- Safe zones still to capture (screenshots, chris): a collapsed toolbar, other floating positions
  (`docs/device-measurements/README.md`). Landscape is no longer owed: a landscape template is read in portrait mode,
  so the portrait zones apply, turned (rung 8 record, Defaulted "Landscape safe zones"; built in 8b).
- The slice page's 8 mm margin (94 px) is narrower than a docked toolbar (Manta 116 px, Nomad 99 px), so a toolbar
  docked left or right covers the edge of its title and rules; lint L06 flags it (`cargo xtask lint`). The slice
  itself is left as is (`crates/model/src/sample.rs`).
- Open until v1.1 (chris, 2026-11-09: "let's leave the 153/157 question as open"; 2026-11-10: "leave the label
  gray issue open until v1.1"): the Label gray (153) sits 4 steps from the
  dark grey pen (157), and chris is "not sure that it's all that bad to have template lines that match the pen ink".
  Bears on L04's severity (napkin §8.3); not built until this is settled (`docs/STATUS-log.md`).
- Fluent's Unicode direction marks around arguments are switched off (`crates/i18n/src/lib.rs`, `Strings::load`):
  right for English in a terminal, wrong for right-to-left languages. Switch them on per language with the first
  right-to-left translation.
- Rung 2 viewer, owed (`crates/ui/src/view.rs`, `crates/app/src/main.rs`): the controls' font on macOS is looked up as
  `.AppleSystemUIFont`, unverified (no Mac here); if fontdb cannot find it, egui's own font shows. Ctrl + scroll
  keeps the point under the pointer still since 8a, untested headless (`egui_kittest` cannot make a scroll-zoom). The export writes no PNG text note (napkin
  §11.1 has it on by default), so its bytes match rung 1's goldens; the note belongs with the export settings.
- The graphics-chip canvas (built in rung 3, `crates/app/src/chip.rs`) is measured only on the Pi at Fit zoom (40 ms
  a frame, release build); still to measure zoomed in, while scrolling, and on a desktop graphics card (rung 3 bbq
  Q7). It redraws the visible area on every scroll step.
- Rung 3 editor, owed (`crates/ui/src/editor.rs`, `crates/ui/src/panels.rs`, `crates/ui/src/view.rs`): Save does not
  update `metadata.modified`. When a font is missing here, Save keeps the file's saved outlines for that device even
  if the text changed. "Don't save" followed by
  cancelling the Open dialog leaves the document unsaved with its recovery file deleted (the next autosave writes
  it again). Only one recovery file is restored per start.
- `CHECK` lines can share a line with the test runner's output under `--nocapture`, so `grep '^CHECK'` (used in
  `.github/workflows/ci.yml`'s Structured results step and in past log entries) can miss one. Turn it into
  something a test enforces: print each line starting with a newline, or count with `grep -o` (`docs/STATUS-log.md`,
  2026-11-10). Rung 4's new checks print the newline; the older ones do not, and nothing enforces it yet.
- Rung 4a, owed (`crates/ui/src/drag.rs`, `crates/layout/src/snap.rs`, `crates/ui/src/view.rs`): drag speed is measured
  only on the sample page at Fit zoom (export canvas: one 60.1 ms frame in 1,595; chip canvas: none over 34 ms); not
  zoomed in, not on a text-heavy page, not on a desktop card. No autoscroll when a drag reaches the canvas edge. Several objects selected on
  different layers are dragged together but a locked one among them stays put without saying so. The layers panel's
  drag-and-drop has no pointer test. Alt-drag may be taken by the window manager on the Pi. The status bar's snap
  switches may not fit in a narrow window.
- Rung 4b, owed (`crates/layout/src/snap.rs`, `crates/ui/src/drag.rs`, `crates/ui/src/editor.rs`): equal spacing works for a
  move only (not a resize); a resize's corner has no point snap (its edges still get smart guides); point snap and smart
  guides count every visible object, including a page-sized pattern region's edges and centre; rhythm wins within half a
  pixel and otherwise the nearest catch wins (chris, 2026-11-10); equal gaps may be half a
  pixel (not rounded to the pixel grid); two open windows holding the same file count as "the same document" for the
  paste offset; a paste goes to the first selected object's layer, not the layer picked in the layers panel; copy and
  paste are tested with egui's events, not the real system clipboard; the status bar is full at 1280 px with seven switches
  (a "Snap" menu would fit narrower windows).

- Rung 5, owed (`crates/ui/src/draw.rs`, `crates/ui/src/tools.rs`, `crates/ui/src/view.rs`, `crates/layout/src/newpage.rs`):
  Double-click to keep a tool is tested through `Viewer::keep_tool`,
  not through the harness (`egui_kittest` cannot make a double-click: two clicks give `double_clicked() == false`,
  measured 2026-11-10); chris's look is its first real check. New… is a flat toolbar button, not a File menu (the
  toolbar has no File menu; the napkin's "File → New"). A new page targets both devices, so its margins clear the
  larger of both devices' toolbars and footers (12 mm at 6 mm rhythm); the floating toolbar is neither cleared nor
  linted (chris, 2026-11-10). The ruled-paper inch presets (9/32 in, 11/32 in) are the agent's, for chris to veto. A drawn
  object's name is its Fluent text at the time ("Line", "Text", "Pattern") stored as the file's own data. A drag-drawn
  object has Scale constraints (the napkin default); only a clicked pattern region is pinned to every edge. The text
  box is 12 pt regular Atkinson Hyperlegible Next with no choice in the dialog.
- Packaging, owed (`packaging/`, `.github/workflows/release.yml`): the icon is a placeholder drawn by an agent (a
  ruled square), chris's to replace. Linux has no MIME definition for `application/x-quadrule`, so `.quad` files are
  not yet recognised by the file manager. The `.deb` puts the @2x icons in `64x64@2` and `1024x1024@2` folders (the
  standard names are `32x32@2`, `512x512@2`); harmless, the plain sizes cover it. The `.dmg` and `.msi` are unsigned
  (macOS Gatekeeper and Windows SmartScreen will warn). chris confirmed the dry run's `.dmg` works on a Mac
  (2026-11-10); nobody has opened the `.msi` or the x86-64 Linux packages yet, and the rc testers (arc Q14) are their
  first check. A pre-release version (`1.0.0-rc.1`) builds an NSIS `.exe` instead of an `.msi`, untried.
- Rung 6a shapes, owed (`crates/layout/src/shapes.rs`, `crates/ui/src/panels.rs`): picking a shape is by its box, so a
  hollow rectangle or ellipse takes a click anywhere inside it (as a pattern region does), and a shape drawn over text
  hides it from the pointer; picking by outline when there is no fill, and a shape's real outline for the marquee, are
  not built. A custom fill steps by 17 (the sixteen levels, all measured clean 2026-11-12, though 0–68 all look black and
  238–255 white); Shade 204 and Block 102 come from the grays sheet (rung 14 has the swatches).
  Caps reach half the stroke past each dash, as in SVG, so a dash gap shorter than the stroke width closes (a square
  cap on Long dash at 1 mm reads solid): the inspector does not warn. A polygon is picked and snapped by its frame, not by
  its corners. L13 (large dark fills) stays in rung 14 although Block makes it likelier. The inspector edits the first
  selected object's values and applies to all, as before (the Arrange tab's X, Y, W, H and Rotation read "mixed" since 7b; the
  Style tab's opacity, stroke kind, role, width and gray since 8a; fill, dash, cap and the shape rows do not). The Style tab shows Dash and Cap for lines
  too (a line is a stroke); a pattern region and a text box have neither. The shapes page is built in code
  (`quadrule_model::sample::shapes_document`) and opened with `--shapes-page`, not as a file.
- Rung 6b, owed (`crates/layout/src/rotate.rs`, `crates/layout/src/shapes.rs`, `crates/ui/src/drag.rs`, `crates/ui/src/view.rs`): text
  boxes and pattern regions do not rotate, and a stored angle on them is ignored; only one object turns at a time (a group has no
  rotate handle and no group turn); resizing a group with turned objects stretches them approximately; a dash pattern on a rectangle
  or ellipse turned by a quarter turn starts at the unturned top-left corner; a turned shape's fill covers the rhythm-snapped box
  turned about its centre, which may sit a pixel off the frame's own centre used for picking; a turned shape is picked by its box
  (as unturned ones are), not by its outline; the rotate handle's name for screen readers (`app-rotate-handle`) is in the strings
  but the canvas is not exposed to a screen reader, so only the inspector's Rotation and Opacity fields are named; an opacity group
  on the chip canvas is one vello layer per object (not measured with many such objects); Shift while drawing a line holds an axis,
  not 15° steps; `--bench-rotate` is timed on one turned rectangle only; the transforms page's chip timing was taken only in the
  window on the Pi, and chip against export pixels for opacity was compared by eye, not by a pixel test.
- Rung 7a, owed (`crates/model/src/edit.rs`, `crates/layout/src/tree.rs`, `crates/ui/src/editor.rs`, `crates/ui/src/drag.rs`,
  `crates/ui/src/panels.rs`, `crates/ui/src/view.rs`): a double-click into a group is tested through `Viewer::enter_at`
  (`egui_kittest` cannot make one), so chris's look is the first real check of it, of Ctrl+click and of the dashed outline
  round the entered group. Look change of 2026-11-12 (Q3 amended: groups are see-through to constraints): a group's box
  is always its children's outer box, hidden children included (it does not change when one is hidden), and a group
  with no children keeps its stored box. Resizing a group stretches turned things inside it approximately, as a loose
  selection does (the 6b debt returns for groups: a turned child keeps its angle and its sides grow by how much the
  stretch lengthens their directions, so a turned child is not sheared; a group inside a turned group stretches the
  same way). The drawing tools put a new
  object on a layer, never inside the group entered. Dragging a row of the layers tree across the edge of a turned group
  is refused with a note (the object would move on the page; Group and Ungroup are the way), and objects can be grouped
  only from one place (the layers, or one group).
  Shift-picking a group and something inside it in the layers panel keeps only the last of the two. Group opacity on the
  chip canvas is one vello layer per opacity group, nested ones nested (not measured with many). A loose multiple
  selection still resizes turned objects approximately.
- Rung 7b, owed (`crates/layout/src/align.rs`, `crates/ui/src/arrange.rs`, `crates/ui/src/panels.rs`): A text box aligns by its stored frame, not by the
  height of its laid-out text. Aligning or distributing objects inside a turned group moves them in the group's own space and
  the group's box is refitted after, so its pivot may shift and the others with it by a little; this is the same for a drag or
  a nudge there. On a pixel grid an object's extent counts as its width rounded to a pixel (how the export draws it), the
  aligned edge lands on a pixel, a centre align puts the left or top edge on a pixel (the centre within half a pixel), and
  distribute by centres rounds each place, with no leftover-pixel spreading (spacing has it). Distribute by spacing
  takes the object with the leftmost (topmost) edge and the one with the rightmost (bottommost) edge as the ends, and
  puts them on the pixel grid too (up to half a pixel). A pick of Selection with one object picked does nothing. The align
  buttons' icons are drawn by hand and not yet seen by chris; no keyboard shortcuts (rung 17); the dragging and drawing
  speed is unchanged and was not re-measured for 7b; the Align-to choice is tested through
  `Viewer::align_to` and `Viewer::set_align_to`, not by opening the dropdown in the window.
- Rung 8a, owed (`crates/ui/src/drag.rs`, `crates/ui/src/editor.rs`, `crates/layout/src/device.rs`, `crates/lint/src/lib.rs`,
  `crates/ui/src/strip.rs`): editing a child inside a group that has an override on the device on screen stores the
  child's override but does not adjust the group's box, so if the children's outer box changes, everything re-scales
  into the same box (a jump). Make primary can shift a child of a turned group on a device whose group box differs (the
  group turns about its box's middle); Ungroup's bake is exact for unturned groups only. A rotation or line-direction
  change made by a drag on a secondary changes the shared design without the status-bar note (only field edits say it).
  L15's "squeezed" check uses a text box's stored height, not its laid-out height. Override keys for a device no longer
  targeted stay only when they came from a file. The strip's pictures and the Ctrl + scroll anchor have no pointer test.
  The overrides dot is the accent colour (kept, chris 2026-11-13; clicking it to open the Overrides tab is v1.1). Drawing a pattern region on the shapes page on the chip
  canvas is over 50 ms at the 90th percentile (8a lead, strip on: 50.8–51.1 ms; first timed in 8a, rung 6a timed only
  rectangle, ellipse and polygon there); chris, 2026-11-13: "Leave it for later" (rung 17's sweep, or sooner if a rung touches it) (`docs/STATUS-log.md`, 2026-11-12/13 rung 8a).
- Rung 8b, owed (`crates/layout/src/device.rs`, `crates/ui/src/strip.rs`, `crates/ui/tests/landscape.rs`): landscape is
  tested on the Nomad only; the Manta is ruled the same, not tried on the device. Turning a page maps every override
  box in proportion to the turned page (as a Scale constraint would), and margins stay the numbers typed, so L06 may
  fire after a turn: both approved as built by chris, 2026-11-13. The strip's
  placeholder, shown until a device's first picture is made, is portrait-shaped. The Page tab's and New…'s
  Orientation controls have no pointer test (the tests call the same functions).
