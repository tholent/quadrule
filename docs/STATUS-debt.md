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
- Rung 1 hardware step, owed to chris: copy `crates/render/tests/golden/*.png` to a Manta and a Nomad (MyStyle
  folder) and say how they look. Does not block PR #1.
- Rung 1 taste proposals waiting on chris: package names `quadrule-*`; square dots one role width wide; the
  estimated role widths and grays (dots look faint at 2 px, gray 187) (`crates/profiles/data/`).
- The `gh` token's expiry date, once chris states it (rung 1 record, Defaulted "Repository").
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
