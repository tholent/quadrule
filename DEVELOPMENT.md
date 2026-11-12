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

### The window is an opt-in build

The graphics stack is heavy to compile, so plain `cargo build` and `cargo test` at the root leave out the `ui` and
`app` crates (`default-members` in `Cargo.toml`). Name them with `-p quadrule-app` or `-p quadrule-ui`, or use
`--workspace` to build everything, as CI does.

## Everyday commands

If you have `make` (Linux and macOS), `make` lists the shortcuts. Each one is a thin wrapper that prints the cargo
command it runs, so you can always run that directly.

| Command | What it does |
|---|---|
| `make test` | Tests on the default members; prints one `TESTS` summary line, full log in `target/test.log` |
| `make check` | rustfmt, clippy and tests on the default members (quick) |
| `make check-all` | The same on the whole workspace; this is what CI runs, so run it before you push |
| `make run` | Open the app (`FILE=page.quad` to open a file) |
| `make bench-drag`, `make bench-draw`, `make bench-rotate` | Time dragging, drawing and turning in a release build (`SHAPES=1`, `TRANSFORMS=1`, `GROUPS=1` or `DEVICES=1` over the shapes, transforms, groups or overrides page, `NOMAD=1` to run on the Nomad so a drag writes overrides, `NOSTRIP=1` to fold the device previews strip shut (it is open in a bench otherwise), `TOOL=rect` to draw with another tool) |
| `make lint` | Lint a document from the command line |
| `make fuzz` | Fuzz the `.quad` reader (needs nightly Rust and `cargo-fuzz`) |

Without `make`, CI's three steps are:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI is `.github/workflows/ci.yml` (rustfmt, clippy and tests, on all three), `fuzz.yml` (a 60-second fuzz of the
`.quad` reader) and `coverage.yml` (line coverage of the whole workspace with `cargo-llvm-cov`, published as the
README's coverage badge through a `badges` branch by `.github/scripts/publish-badge.sh`). The badges show the latest
run on `main` of each; the workflows run only when files they test change.

Tests print one `CHECK` line per check, so a result reads as a line, not prose. CI runs on Linux, macOS and
Windows, and a change passes only when all three agree.

## How the code is laid out

One crate per concern, under `crates/`. Each crate's `Cargo.toml` has a one-line description, and the section
numbers refer to the design spec.

| Crate | What it holds |
|---|---|
| `model` | The document model: objects (lines, text, patterns, rectangles, ellipses, polygons and stars, each with an angle and an opacity; a line's diagonal; groups, which hold other objects, to any depth) and geometry in millimetres, with the constraint maths and the editing commands, no graphics or UI (§5) |
| `profiles` | Device profiles (the Manta and the Nomad) as RON data: line roles, fill roles, safe zones (§3) |
| `layout` | Constraints, the rhythm, pixel snapping, the snapping rules, shapes with their dashes and caps, rotation and line direction (`rotate`), the tree of groups laid out (`tree`: every object resolves against the page by its own constraints, a group's box is round its children), align and distribute (`align`: the maths on the objects' turned boxes, on the pixel grid or in millimetres), the display list with its opacity groups (§10), per-device overrides (`tree` reads them, a group's override scales what is in it into its box), and `device` (Make primary, which devices a page targets, what Ungroup bakes in) |
| `format` | The `.quad` file: a ZIP of CBOR entries (§12, ADR-001) |
| `text` | Fonts and text layout in millimetres, drawn as plain outlines (§7, ADR-002) |
| `render` | The export renderer: `vello_cpu` to an 8-bit grayscale PNG (ADR-003) |
| `lint` | The linter: rules that recommend and explain, never block (§8.3) |
| `i18n` | Every string on screen, through Project Fluent; English first (§14) |
| `ui` | The egui widgets and the editor's state: canvas, panels, dragging, the tools bar and drawing, the shapes' inspector fields, the Arrange tab's align and distribute commands (`arrange`) (§15.1) |
| `app` | The program itself: window setup and preferences |
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

Each rung chris approves gets a version (`0.N.0`, or `0.N.1` for a second part such as 4b), an annotated git tag
and a GitHub pre-release, with notes in `docs/arcs/v1/release-notes/` (`docs/arcs/v1/README.md`).

The installers are built by `.github/workflows/release.yml` on GitHub's machines: an AppImage and a `.deb` for Linux
(x86-64 and ARM64), an unsigned universal `.dmg` for macOS and an `.msi` for Windows, plus `SHA256SUMS.txt`. The
config is in `packaging/`: `packager.json` (read by [cargo-packager](https://crates.io/crates/cargo-packager) 0.11.8),
the icons, and `prepare.py`, which takes the version from the tag.

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

## Calibration and device measurements

Quadrule's line widths and grays are measured on real Supernote devices, not guessed. If you own one, you can help.

- **Calibration sheets.** `cargo xtask calibration-sheet` writes test pages at your device's exact size, each sample
  coded. Copy them to the tablet, look, and note which codes read best. The whole procedure is in
  [docs/calibration.md](docs/calibration.md).
- **Safe zones.** The areas the tablet's own toolbar and footer cover were measured from screenshots.
  [docs/device-measurements/README.md](docs/device-measurements/README.md) says how to take the screenshots and what
  `cargo xtask safe-zones` does with them. Landscape and other toolbar positions are still to be measured.

Send findings in an issue. A different model or a firmware update can change the numbers, which is why the raw
screenshots are kept in `docs/device-measurements/`.
