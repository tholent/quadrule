# Developing Quadrule

This page is for people who build, test or change Quadrule. If you only want to know what it is, read
[README.md](README.md).

Quadrule is pre-alpha, written in Rust with [egui](https://github.com/emilk/egui) for the interface and
[wgpu](https://wgpu.rs) for the graphics. The design is in `napkins/quadrule-spec.md`; where the work stands is in
[STATUS.md](STATUS.md).

## Set up

1. Install [rustup](https://rustup.rs). `rust-toolchain.toml` pins Rust 1.99.1 (with clippy and rustfmt), and
   rustup fetches it the first time you run `cargo` in the repository. Everyone, and CI, builds with the same
   compiler on purpose: the tests compare exported PNGs byte for byte with stored images.
2. Clone the repository and build:

   ```
   cargo build --workspace
   ```

   On Linux, GitHub's stock Ubuntu runner builds it with no extra packages (`.github/workflows/ci.yml`). If your
   machine complains about fontconfig, install its development files (on Debian or Ubuntu, `libfontconfig1-dev`).
3. Run the app: `cargo run --release -p quadrule-app`, or add `-- path/to/page.quad` to open a file.
4. Optional, for faster tests: [cargo-nextest](https://nexte.st), at the version the project uses:

   ```
   cargo install cargo-nextest --version 0.9.151 --locked
   ```

   `make test` and `make test-all` use it when it is there and plain `cargo test` when it is not. nextest runs every
   test as its own process, many at once, where `cargo test` runs one test program after another: on a 12-core VM the
   whole workspace's tests took 5 min 11 s instead of about 20 min (`docs/STATUS-log.md`, 2026-11-20). Its settings
   are in `.config/nextest.toml`. CI still uses `cargo test`.

Plain `cargo build` and `cargo test` at the root build and test everything, the window included, as CI does (ADR-008,
amendment of 2026-11-20; until then the window crates were left out to keep the old Raspberry Pi build machine
quick).

## Everyday commands

If you have `make` (Linux and macOS), `make` lists the shortcuts. Each one is a thin wrapper that prints the cargo
command it runs, so you can always run that directly. Builds use one job per core, but no more than one per 1.5 GB of
memory (ADR-006, amendment of 2026-11-20); `JOBS=4` sets it by hand.

| Command | What it does |
|---|---|
| `make test` | Tests on the whole workspace; prints one `TESTS` summary line (ending `runner=nextest` or `runner=cargo`), full log in `target/test.log` |
| `make check` | rustfmt, clippy and tests on the whole workspace, then `make prune`; this is what CI runs, so run it before you push (`make check-all` is the same) |
| `make run` | Open the app (`FILE=page.quad` to open a file) |
| `make bench-drag`, `make bench-draw`, `make bench-rotate`, `make bench-type`, `make bench-zoom` | Time dragging, drawing, turning, typing (into a long text box) and zooming (a pinch in and out between a quarter and twelve times; `GROUPS=1`, `BORDERED=1`; it times every frame) in a release build (`SHAPES=1`, `TRANSFORMS=1`, `GROUPS=1`, `DEVICES=1`, `LANDSCAPE=1` or `PATTERNS=1` or `ANGLE=1` (the pattern-angle page) or `BORDERED=1` (the pattern-borders page: borders round pattern regions) or `LINT=1` (the lint page: objects that make L03, L05, L07, L09, L13 and L14 fire) or `PAPER=1` (the paper page: ISO A4 at 300 PPI and ISO A5 at 600) or `TURNED=1` (the turned-patterns page; `make bench-rotate TURNED=1` turns its 45 degree grid) or `REPEATS=1` or `FONTS=1`, `TEXTBOX=1` or `MISSING=1` over the shapes, transforms, groups, overrides, landscape, pattern or repeat page, `NOMAD=1` to run on the Nomad so a drag writes overrides, `NOSTRIP=1` to fold the device previews strip shut (it is open in a bench otherwise), `TOOL=rect` to draw with another tool) |
| `make starters` | Write the built-in starter pages to `crates/ui/starters/` from code (`cargo xtask starters`; `STARTERS_CHECK=1` writes nothing and fails when a file differs) |
| `make lint` | Lint a document from the command line |
| `make fuzz` | Fuzz the `.quad` reader and the profile loader (needs nightly Rust and `cargo-fuzz`) |
| `make prune` | Delete old copies of the project's own builds from `target/`, and caches unused for three days (`DRY=1` lists them only) |

Without `make`, CI's three steps are:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI is `.github/workflows/ci.yml` (rustfmt, clippy and tests, on all three), `fuzz.yml` (a 60-second fuzz of the
`.quad` reader and a 30-second one of the profile loader) and `coverage.yml` (line coverage of the whole workspace with `cargo-llvm-cov`, published as the
README's coverage badge through a `badges` branch by `.github/scripts/publish-badge.sh`). The badges show the latest
run on `main` of each; the workflows run only when files they test change. Fuzz and coverage also skip a push that
only moves the version number (a rung's sign-off), decided by `.github/scripts/version-only.py`; CI still tests it.

Tests print one `CHECK` line per check, so a result reads as a line, not prose. CI runs on Linux, macOS and
Windows, and a change passes only when all three agree.

### Keeping the build folder small

Cargo never deletes old builds by itself, so `target/` only grows: on the Pi it reached 411 GB
(`docs/proposals/build-pruning-2026-11-19.md`). Three things keep it small.

- **Less debug information.** The dev profile in `Cargo.toml` keeps line numbers (a panic still names the file and line)
  but not the full data a debugger uses to step through variables, and libraries we don't write get none. To step
  through our own code in a debugger, build with `CARGO_PROFILE_DEV_DEBUG=true cargo build` for that session. Release
  builds are not changed.
- **`make prune`** (the `xtask prune` tool) deletes, in `target/<profile>/deps`, older copies of this project's own
  libraries, test programs and programs: it keeps the newest two of each test program and the newest four of each
  library and of the `quadrule` and `xtask` programs (Cargo's own records in `target/<profile>/.fingerprint` tell it
  which copy is which; a copy with no record is kept). In `target/<profile>/incremental` it deletes the caches unused
  for three days. It never touches third-party crates, `target/tmp` or the fuzz folder (where one of our test files
  shares a third-party crate's name, such as `window`, only our old test programs go). It ends with one line,
  `PRUNE target=target removed_files=N removed_gb=N.N kept_gb=N.N dry_run=no` (GiB, as `du -h` counts). `make check`
  and `make check-all` run it after the tests pass, except on CI (when `CI` is set).

  Why keep more than one: `make check`, `cargo build`, `make run` and `cargo xtask` each build some of our crates with
  different library features, and Cargo cannot tell those copies from old versions. Test programs, most of the folder,
  come only from the tests, so two copies keep the current one and the one before; libraries and the two programs come
  every way and are small (about 1.5 GB for all four), so four keep every one, and switching between these commands
  rebuilds nothing. Deleting something still needed only costs a rebuild.
- **A cap.** `make build-proof` adds `target_gb=N.N` to its `BUILD` line and fails with `over the cap` when `target/` is
  bigger than `TARGET_CAP_GB` (50 by default; `make build-proof TARGET_CAP_GB=80` to change it).

## How the code is laid out

One crate per concern, under `crates/`. Each crate's `Cargo.toml` has a one-line description, and the section
numbers refer to the design spec.

| Crate | What it holds |
|---|---|
| `model` | The document model: objects (lines, text, patterns with their own settings, rectangles, ellipses, polygons and stars, each with an angle and an opacity; a line's diagonal; groups, which hold other objects, to any depth; repeats (`repeat`), which hold one source and draw it again, with their settings and alternation rules; mini calendars and the months and weekdays of calendars (`calendar`)) and geometry in millimetres, with the constraint maths and the editing commands, no graphics or UI (§5) |
| `profiles` | Device profiles (the Manta and the Nomad) and the eleven paper sizes (`paper.a4`, … ; a profile with a `paper` size in mm, whose pixels and line widths `at_ppi` gives for the PPI a page picks, 150, 200, 300 or 600; a landscape paper page is wide, not turned) as RON data: line roles, fill roles, safe zones (§3), and each profile as a landscape design sees it (`oriented`: the page turned, the zones turned into its frame). The **registry** (`registry`) is the one lookup: the shipped built-ins, the user's overrides of them, the user's own profiles (UUID v4 ids, `<config folder>/profiles/<id>.ron`, saved through a temporary file and a rename) and the open document's carried copies; `limits` is what a profile file may hold (anything else is damaged and skipped, listed with its line); `active` is the registry the app looks up in, one per thread, so a test never sees the config folder. `builtin`, `builtin_for` and `builtin_at` (a paper size at a PPI) give the shipped values only, for tests, goldens and the calibration tools; `builtin_ids` lists the screens and `builtin_paper_ids` the paper sizes; production code uses `active::lookup`, `lookup_for` and `lookup_in` (a document's page: the PPI it picked for a paper size, and the way it is held; a test scans for strays) |
| `layout` | Constraints, the rhythm (rectangular or isometric), pixel snapping, the patterns drawn (`pattern`: lines, dots, grid, isometric, hex, Cornell, music staff and calendar, their major lines and unbound spacing, one path for all of a region's slanted lines; a turned region is drawn upright in its own axes and then turned whole about its middle, `pattern_items_turned`; a region's own ruling angle is laid out over a covering box and cut at the frame, `ruling_items`; a region's border is `border`: a `Rim` that leaves out the lines and dots on a bordered edge, cuts the ruling at a rounded corner and draws the border itself), the snapping rules, shapes with their dashes and caps, rotation and line direction (`rotate`), the tree of groups laid out (`tree`: every object resolves against the page by its own constraints, a group's box is round its children), align and distribute (`align`: the maths on the objects' turned boxes, on the pixel grid or in millimetres), the display list with its opacity groups (§10), per-device overrides (`tree` reads them, a group's override scales what is in it into its box), the copies of a repeat on each device in whole pixels (`repeat`: counts, Fill, the step, Centre, alternation, the 5,000-copy cap, Expand), `calendar` (a calendar pattern's cells and a mini calendar's rows, the words each draws sized to fit and laid out as text under ids made from the object's, what is left out for L07), and `device` (Make primary, which devices a page targets, what Ungroup bakes in, turning the page between portrait and landscape) |
| `format` | The `.quad` file (format 0.21): a ZIP of CBOR entries, with embedded fonts under `assets/fonts/`, carried profiles under `profiles/<id>.ron` and the main device's thumbnail under `previews/<id>.png` (the app writes it at Save and Save As, never at autosave; readers may ignore it), read through the profile limits (§12, ADR-001) |
| `text` | Fonts and text layout in millimetres, drawn as plain outlines (§7, ADR-002): the bundled fonts with their OFL texts (`crates/text/fonts/`), fonts installed on the computer (re-scanned when the window regains focus), optical size, the measured font signature and substitute ranking, fonts carried inside a file, the embed rule, and a fallback that goes bundled fonts first, then installed ones |
| `render` | The export renderer: `vello_cpu` to an 8-bit grayscale PNG with an optional text note, `tEXt` for Latin-1 and `iTXt` otherwise (ADR-003), and the exact quarter turn a landscape page is exported with (`turn_clockwise`) |
| `lint` | The linter: rules that recommend and explain, never block (§8.3). `judge` holds the rules that read a profile's `lint` block (L03, L05, L07, L09, L13, L14: a missing number skips the rule); `skipped_checks` says which checks did not run on a device and why; a page's `ignored_rules` and an object's `free_placement` (format 0.15) are honoured; `fix` works out each rule's quick fix as `Edit`s (`offered`, `plan`, `plan_snap_all`) with no screen in it |
| `i18n` | Every string on screen, through Project Fluent; English first (§14); the words a calendar draws on the page are in `calendar.ftl` |
| `ui` | The egui widgets and the editor's state: the Profiles window (`profiles_win` is its state and what its buttons do, tested headless; `profiles_win_ui` the widgets), the bar and the Merge / Rename / Not now dialog for profiles a file carries (`profile_use`, `profiles_ui`), canvas, panels, dragging, the tools bar and drawing, the shapes' and patterns' inspector fields, the rhythm overlay, the Arrange tab's align and distribute commands (`arrange`), the Basic tab and the Show in Basic switches (`basic`: what is marked lists with the other tabs' own widgets, the page's title, margins and rhythm, a line's role, a text box's text, a pattern's defining settings and spacing; ADR-026; shown only with `--basic-section`, and `--look basic` shows it), the Lint tab (`lint_ui` the widgets, `lint_fixes` what its buttons do: findings on every device, quick fixes through a device's overrides, Ignore, Free placement), the preflight sheet before Export (`preflight`: the report and the sheet), the Export sheet and the completion sheet (`export_ui`: what is drawn and what a press does; `--look export|export-preset|export-mistake|exported|exported-open` opens the window on each, for a screenshot), the export engine (`export`: a request of devices, folder, template, Set and note in; files written, folders made, a result with each file, the replaced count and a small picture of each out; `last_export_folder` is where the sheet starts), the path template (`export_path`: tokens, safe names, mistakes as data) and the settings file (`settings`: `settings.ron`, the folder each document last exported to) and the Style tab's gray swatches and role badge (`swatches`), the layers panel's New layer and Delete layer (`layers`: the buttons, the row's right-click menu, the ask-first sheet and the new layer's open name field; the commands are `InsertLayer` and `RemoveLayer` in `crates/model`; `--look layers|layers-new|layers-ask|layers-menu`), the layers panel's seven buttons as line icons (`layer_icons`: drawn from lines as the tools bar's are, one row, each named on hover and for screen readers), the menus (`menus`: one table of File, Edit, View with Snapping, Help, and the Mac's Quadrule and Window menus, with each item's command, key and greyed or ticked state; `menu_bar`: that table drawn along the top of the window on Windows and Linux, and the About quadrule sheet; `file_menu`: Open Recent; `--look file-menu|menu-edit|menu-view|menu-help|about`), the recent files kept in `settings.ron` (`recent`), My templates in the data folder's `templates/` (`templates`: Save as template…, the list, the untitled copy a template or starter opens as) and the welcome screen (`welcome`: tabs, Device filter, Search, tiles; `--look welcome|file-menu|gallery-mine`), its live thumbnails (`thumbs`: the export renderer for the filter's device on one background thread, cached for the session), the built-in starters (`starters`: `crates/ui/starters/<category>/<slug>.quad` built in by `build.rs`, written by `cargo xtask starters`; see Starter pages) and test starters made from the sample documents (`test_starters`; `--gallery-test-starters`, `--gallery-device manta|nomad`, `--bench-welcome [frames]` times the screen while 24 tiles draw and when idle) (§15.1) |
| `app` | The program itself: window setup and preferences; on the Mac, the system's menu bar built from `ui`'s menu table (`mac_menu`, with `muda`) |
| `macos` | The Mac only: teaches winit's application delegate to hear which `.quad` files Finder asks the program to open (a double-click). The one crate where `unsafe` code is allowed, with a reason beside each use; the rest forbid it |
| `xtask` | Maintainer tools, run as `cargo xtask` |

Export never uses the graphics chip: it goes through `render` alone, so a file comes out the same on every machine
(ADR-003). The window's canvas can draw on the graphics chip for speed, but it falls back to the same CPU renderer.

Where things live beyond the code:

- `napkins/quadrule-spec.md`: the design specification.
- `docs/decisions/`: decision records (ADRs). `INDEX.md` lists them.
- `docs/bbq/`: the planning records for each piece of work.
- `STATUS.md`, `docs/STATUS-log.md`, `docs/STATUS-debt.md`: where the project is, what happened each session, and what
  is owed.
- `AGENTS.md`: the working rules the project's maintainer and AI coding agents follow.

## Releases and packages

The repository's own rules on GitHub (ADR-022): `main` cannot be force-pushed or deleted, a release tag `v*` (not a
`v*-test*` one) cannot be deleted or moved, secret scanning and push protection are on, and merged branches are
deleted. `make check-github` reads them back and prints one `GITHUB` line; it needs a signed-in `gh`, and the rules
themselves are changed in the repository's Settings, by its owner.

Each rung chris approves gets a version (`0.N.0`, or `0.N.1` for a second part such as 4b), an annotated git tag
and a GitHub pre-release, with notes in `docs/arcs/v1/release-notes/` (`docs/arcs/v1/README.md`).

The installers are built by `.github/workflows/release.yml` on GitHub's machines: an AppImage and a `.deb` for Linux
(x86-64 and ARM64), an unsigned universal `.dmg` for macOS and an `.msi` for Windows, plus `SHA256SUMS.txt`. The
config is in `packaging/`: `packager.json` (read by [cargo-packager](https://crates.io/crates/cargo-packager) 0.11.8),
the icons, and `prepare.py`, which takes the version from the tag.

Homebrew (ADR-025): Mac users install the `.dmg`'s app with `brew install --cask tholent/tap/quadrule`, from
tholent's shared tap [tholent/homebrew-tap](https://github.com/tholent/homebrew-tap). The cask's source is
`packaging/homebrew/quadrule.rb`; edit it there, never in the tap. Once a release is published, the workflow's
"Homebrew tap" job fills in the version and the `.dmg`'s sha256 and pushes the cask to the tap, using the secret
`HOMEBREW_TAP_DEPLOY_KEY` (the private half of a deploy key with write access on that tap only). Without that secret
the job prints `TAP … status=skip reason=no-deploy-key` and the release still succeeds. The tap's own CI audits the
cask and installs it on a Mac after every push.

The repository has GitHub's immutable releases on: once a release is published, no file can be added to it, and a
deleted release's tag name can never be used again. So a release is made by **pushing its tag**, never by publishing
it by hand first: the workflow builds the packages, makes a draft pre-release with the notes in
`docs/arcs/v1/release-notes/<tag>.md` (which must be in the tagged commit), attaches the files, then publishes it.

```
git tag -a v0.7.0 -m "…" && git push origin v0.7.0           # the whole release
gh workflow run release.yml -f tag=v0.7.0                     # dry run: the packages are the run's artifacts
gh workflow run release.yml -f tag=v0.7.0 -f publish=draft    # packages on a draft release, not published
gh workflow run release.yml -f tag=v0.7.0 -f publish=yes      # what a tag push does
```

A tag with `-test` in it (`v0.6.2-test.1`) never runs on push, so the draft step can be tried by hand on it; delete
the draft and the tag afterwards.

To try packaging on Linux without GitHub: `cargo install cargo-packager --version 0.11.8 --locked`, build the app
in release mode, then `cargo packager -c packaging/packager.json -f deb,appimage --release` (packages land in
`dist/`). `prepare.py` edits `packaging/packager.json` and `Cargo.toml`, so run it only in a throwaway copy.

## Contributing

Quadrule is pre-alpha, and its direction, naming and look are settled by the maintainer, so the process is small.

1. **Open an issue first.** Bug reports, ideas and questions are all welcome. For a change, describe what you want
   to do before writing code, so nobody's time is wasted on something that does not fit the plan.
2. **Keep changes small and focused**, one thing per pull request.
3. **Run `make check-all`** (or CI's three commands above) and make sure it is clean. Warnings are errors.
4. **Follow the house rules the tests enforce.** `unsafe` code is forbidden across the workspace, and every Rust file
   starts with the Apache-2.0 header (`crates/xtask/tests/license_headers.rs` checks it).
5. **Leave export bytes alone unless that is the point.** A change that alters an exported pixel will fail the golden
   image check (`crates/render/tests/golden/SHA256SUMS`); say so in the pull request if it is intended.

Contributions are accepted under the project's Apache-2.0 licence (section 5 of the licence).

## Starter pages

The built-in starters (the pages on the welcome screen's Paper, Notes, Planning, Bullet journal and Creative tabs) are
`.quad` files in `crates/ui/starters/<category>/<slug>.quad`, built into the program by `crates/ui/build.rs`. Nobody
edits them by hand: `cargo xtask starters` (or `make starters`) writes every one from code in
`crates/xtask/src/starters.rs`, one function per starter, with fixed dates and ids so two runs give the same bytes.

- **Adding or changing a starter:** write or edit its function there, run `cargo xtask starters`, look at the two pages it
  draws (the goldens below), and commit the files. Frames are written in Nomad pixels through constraints, so a page
  adapts to any tablet; the comments on each function say where its frame edges fall on the rhythm of both tablets.
- **A format change** re-makes them all: `crates/xtask/tests/starters.rs` fails while any checked-in starter differs from
  what the generator writes now, and `cargo xtask starters --check` says which.
- **The same test file** lints every starter on both tablets and finds nothing (ADR-021, item 1), and checks the names,
  descriptions, tags and Show in Basic marks the rules ask for.
- **Goldens:** `crates/render/tests/starters.rs` draws each starter file on both tablets and compares with
  `tests/golden/starter-<slug>_<profile>_<size>.png`; a starter with no golden fails. After a deliberate change and a
  look, `QUADRULE_BLESS=1 cargo test -p quadrule-render --test starters` replaces the starters' lines of `SHA256SUMS`.

## Calibration and device measurements

Quadrule's line widths and grays are measured on real Supernote devices, not guessed. If you own one, you can help.

- **Calibration sheets.** `cargo xtask calibration-sheet` writes test pages at your device's exact size, each sample
  coded. Copy them to the tablet, look, and note which codes read best. The whole procedure is in
  [docs/calibration.md](docs/calibration.md).
- **Safe zones.** The areas the tablet's own toolbar and footer cover were measured from screenshots.
  [docs/device-measurements/README.md](docs/device-measurements/README.md) says how to take the screenshots and what
  `cargo xtask safe-zones` does with them. A landscape page is read in portrait mode, so the portrait zones apply,
  turned into the design's frame; the landscape toolbar screenshots in `landscape/` show a zoomed view, not template
  pixels, and are kept as records only. Other toolbar positions are still to be measured.

Send findings in an issue. A different model or a firmware update can change the numbers, which is why the raw
screenshots are kept in `docs/device-measurements/`.
