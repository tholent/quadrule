# bbq record — quadrule: the three technical spikes

2026-11-09 · untyped bbq, seeded by napkin from `napkins/quadrule-spec.md` · 9 questions (8 asked + the closing
check) · lead: Opus 5.5, high effort.

## From the napkins

Settled by the napkin; no question asked. Cited as `napkins/quadrule-spec.md:<line>`.

- Quadrule is a free, open-source vector design tool for Supernote e-ink page templates and wallpapers (`:13`).
- Crate and binary are named `quadrule`; license Apache-2.0 (`:4-5`).
- Stack: Rust, egui (UI), wgpu (GPU); desktop only: macOS, Windows, Linux (`:6-7`).
- Output is single-page 8-bit grayscale PNG, no alpha, at each device's native resolution (`:24`, `:546-547`).
- Internal geometry is stored in millimetres as f64; pixel snapping happens per profile at layout time (`:222-224`).
- Canvas: vello renders a display list to a wgpu texture shown by egui (`:529`).
- Export: a deterministic rasterizer that gives identical PNGs on every platform; recommended default is CPU
  rasterization (`tiny-skia` or vello's CPU path), finalized during the rendering spike (`:530`).
- Pixel rules: strokes resolve to whole device pixels where possible; odd widths centre on pixel centres, even
  widths on pixel edges; anti-aliasing on for curves and text, effectively off for snapped axis-aligned lines
  (`:535-537`).
- Text: fontdb for discovery, cosmic-text for shaping and wrapping, skrifa for outlines; text renders to outlines
  for display and export (`:349`, `:353`, `:645`).
- File format: `.quad` is a ZIP holding CBOR (`ciborium` via serde) with field names kept, Deflate for CBOR
  entries, text outlines per profile, semver `format_version` with a migration chain (`:572-597`).
- `model`, `layout`, `lint` and `format` must be testable headless (`:651`).
- Built-in v1.0 targets: Manta 1920×2560 @ 300 PPI and Nomad 1404×1872 @ 300 PPI; A5 X at 226 PPI is the main
  constraint test case (`:99-100`, `:108`).
- The three spikes, before any v1 feature work (`:706-710`):
  1. **Rendering:** vello in egui via wgpu; deterministic export renderer; pixel-snap correctness at 226 and 300 PPI.
  2. **Text:** cosmic-text + fontdb on all three OSes; outline extraction; system font enumeration speed.
  3. **Format:** CBOR schema, migrations, and outline storage size on a text-heavy planner.

## Rules

- **Q1 Count.** 8 questions, plus the closing check.
- **Q2 First work.** Keep as written: the first comrun is the three spikes (`:706-710`), before any v1 features.
- **Q3 Stack.** Keep as written: Rust, egui, wgpu, vello for the canvas, a CPU renderer for export.
- **Q4 Lead.** Opus 5.5 at high effort.
- **Q5 Build machine.** Install Rust (rustup) on this Raspberry Pi 5 (4 GB, aarch64). Headless parts build and test
  here; the egui window is checked on the Pi's own display. Undo: `rustup self uninstall`.
- **Q6 Spike code.** Throwaway: each spike is a small crate under `spikes/` with a short findings note. The real
  crates are written afterwards, informed by the findings.
- **Q7 Done when.** Each spike ends with a findings note (each check reported as one structured test line) and a
  proposed ADR (for example "export renderer: tiny-skia"). chris ratifies the ADRs.
- **Q8 Landing.** Local commits only, on a spikes branch. Nothing is pushed anywhere; chris decides when.
- **Q9 Anything else to protect or avoid?** Nothing more; `AGENTS.md` and the answers above cover it.

## Owed

(empty)

## Defaulted

- **Spike order:** format → text → rendering. Format and text are headless and quick to build on the Pi; rendering
  is the heaviest build and the GPU question. veto: name another order.
- **Pi GPU risk:** vello needs wgpu compute shaders. Whether the Pi 5's Vulkan driver runs vello is unknown and is
  measured in the rendering spike, not assumed. If it does not, the spike records that, still proves the CPU
  export path here, and leaves the vello-in-egui check for a machine with a desktop GPU. veto: say to stop instead.
- **"All three OSes" (text spike, `:708`):** only Linux can be tested on this machine. The findings note says so;
  macOS and Windows stay owed. veto: name a machine for them.
- **Build parallelism:** cargo's default job count (4); if a build runs out of memory, drop to `-j 2` and note it.
  veto: fix it at 2 from the start.
- **Branch:** `spikes/2026-11-09`, branched from the current `prep/2026-11-09`. veto: name another base.
- **ADR format:** `docs/decisions/README.md` has not chosen one; the first proposed ADR uses makeadr, which offers
  ADR+ first and asks chris. veto: choose now.
- **Spikes are not features:** AGENTS.md's "land a new feature switched off behind a flag" does not apply to
  throwaway spike crates. veto: say it does.
- **Session-start check:** AGENTS.md asks to prove the build on disk matches HEAD; with no build yet, the first
  session records "no build yet" and the check starts once `spikes/` has crates. veto: none expected.

## Kickoff

Run on **Opus 5.5 at high effort**; on any other model or effort, stop and say so before doing anything. Read
`AGENTS.md`, then this record (`docs/bbq/2026-11-09-quadrule-spikes.md`) and the napkin sections it cites. On branch
`spikes/2026-11-09`: install Rust with rustup (say first that `rustup self uninstall` undoes it), then run the three
spikes as throwaway crates under `spikes/` in the Defaulted order. For each: a findings note with one structured line
per check, and a proposed ADR for chris to ratify. Commit locally only; push nothing. Update `STATUS.md` and
`docs/STATUS-log.md` at the end.
