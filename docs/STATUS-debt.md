# STATUS-debt — quadrule

Known debt and owed work. One line each. When an item is paid, remove it and note it in `STATUS-log.md`.

- The project-setup decision is not on record yet (its decision format is now chosen: ADR+).
- A lint/preflight rule for system-font use, at save and at export, is owed to the napkin's lint list (ADR-002).
- Text spike T14/T15, what is left: system-font discovery on macOS and Windows. Layout and outlines with the bundled
  font now match byte for byte there (CI run 42660403429); system fonts are not loaded yet (`crates/text/src/lib.rs`).
- Render spike R15: run the eframe canvas window on a display (`spikes/render/FINDINGS.md`).
- Format: set up cargo-fuzz (needs nightly Rust) on the `.quad` reader, ADR-001 item 8; a stable-Rust random-damage
  test stands in (`crates/format/tests/format.rs`).
- Parked for chris: rendering — Scale vs pixel-locked rhythm on A5 X; canvas fallback to vello_cpu on weak GPUs;
  the build-folder instruction the render worker reports (`spikes/render/FINDINGS.md`).
- Device test, owed to chris: put the slice PNGs (`crates/render/tests/golden/`) and the calibration sheets
  (`cargo xtask calibration-sheet`) on a Manta and a Nomad; report codes (`docs/calibration.md`). Settles the role
  values and square vs round dots.
- The `gh` token expires 90 days after it was made (chris, 2026-11-09); if made 2026-11-09, that is about 2027-02-07.
  Renew it before then, or `gh` stops opening PRs and reading CI (rung 1 record, Defaulted "Repository").
- The `gh` fine-grained token cannot read check summaries (`gh pr checks` fails: GitHub offers no `checks:read` for
  fine-grained tokens); read CI with `gh run view <id> --log` instead.
- ADR-001 not built yet: font signature in the manifest (item 6), missing-font reuse for a new profile (item 7),
  hinted outline tables (item 3, with hinting), `assets/` and `previews/` entries (napkin §12.1).
- ADR-002 not built yet: hinting flag (item 4), system-font fallback and its save/export warning (item 6),
  substitution by signature (item 7), re-scan on focus (item 8), paragraph spacing, vertical alignment and the
  overflow signal (item 3).
- ADR-003 not built yet: nothing in its Decision is unbuilt for export; the canvas renderer is outside it. The
  ADRs' `built:` headers still say "not started" — update them through makeadr when chris wants.
- Layout limits in rung 1: pattern strokes may reach half a stroke past their region (no clipping); a dot grid stops
  short of the right margin when the width is not a whole number of rhythm units (lint L18's job)
  (`crates/layout/src/lib.rs`).
- Safe zones still to capture (screenshots, chris): landscape, a collapsed toolbar, other floating positions
  (`docs/device-measurements/README.md`).
- The slice page's 8 mm margin (94 px) is narrower than a docked toolbar (Manta 116 px, Nomad 99 px), so a toolbar
  docked left or right covers the edge of its title and rules. Lint L06 (content in a safe zone) will flag this once
  the linter exists; the slice itself is left as is (`crates/model/src/sample.rs`).
