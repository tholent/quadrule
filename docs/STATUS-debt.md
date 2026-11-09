# STATUS-debt — quadrule

Known debt and owed work. One line each. When an item is paid, remove it and note it in `STATUS-log.md`.

- The project-setup decision is not on record yet (its decision format is now chosen: ADR+).
- A lint/preflight rule for system-font use, at save and at export, is owed to the napkin's lint list (ADR-002).
- Text spike T14/T15, what is left: system-font discovery on macOS and Windows. Layout and outlines with the bundled
  font now match byte for byte there (CI run 42660403429); system fonts are not loaded yet (`crates/text/src/lib.rs`).
- Render spike R15: run the eframe canvas window on a display (`spikes/render/FINDINGS.md`).
- Parked for chris: rendering — Scale vs pixel-locked rhythm on A5 X; canvas fallback to vello_cpu on weak GPUs;
  the build-folder instruction the render worker reports (`spikes/render/FINDINGS.md`).
- Calibration, in progress (chris): the Nomad is done and its values are in its profile
  (`crates/profiles/data/supernote.a6x2.ron`, `verified: true`); the four Manta sheets are next. The Manta's Dot
  grid and Label values are the Nomad's until then. Clean gray levels and ink values are not measured on either.
- The `gh` token expires 90 days after it was made (chris, 2026-11-09); if made 2026-11-09, that is about 2027-02-07.
  Renew it before then, or `gh` stops opening PRs and reading CI (rung 1 record, Defaulted "Repository").
- The `gh` fine-grained token cannot read check summaries (`gh pr checks` fails: GitHub offers no `checks:read` for
  fine-grained tokens); read CI with `gh run view <id> --log` instead.
- ADR build status is in each ADR's `built` and `owed` header fields (`docs/decisions/INDEX.md`): ADR-001 and
  ADR-002 partial, ADR-003 landed.
- Pattern rule, chosen 2026-11-09 for chris to veto: a pattern draws every line and dot whose centre is in its
  region, whole, so a dot on the margin straddles the region edge rather than being cut (`crates/layout/src/lib.rs`;
  napkin §5.2 says patterns "clip to their bounds"). Trying whole-shapes-inside removed the slice's first dot row
  and column.
- Safe zones still to capture (screenshots, chris): landscape, a collapsed toolbar, other floating positions
  (`docs/device-measurements/README.md`).
- The slice page's 8 mm margin (94 px) is narrower than a docked toolbar (Manta 116 px, Nomad 99 px), so a toolbar
  docked left or right covers the edge of its title and rules; lint L06 flags it (`cargo xtask lint`). The slice
  itself is left as is (`crates/model/src/sample.rs`).
