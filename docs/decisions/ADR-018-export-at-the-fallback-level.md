---
status: proposed
status-note: "drafted 2026-11-19 from chris's ruling (\"let's go with option A\") and answers; awaiting ratification"
owner: chris
kind: architecture
built: partial
proof: "`crates/render/tests/render.rs::export_draws_at_the_fallback_level`, `crates/render/src/lib.rs`"
owed: the golden images and the migration tests' stored export hashes re-made after chris's look; drag and draw speed measured on the Pi against ADR-009's 50 ms
supersedes: ADR-017
amends: "ADR-003 §Decision item 2 (the lever if machines disagree)"
revisit-when: vello_cpu or fearless_simd change what the fallback level computes, or the Pi's speed measured against ADR-009 no longer fits
---
# ADR-018 — Export draws at fearless_simd's fallback level on every computer, so no computer's processor changes a pixel

## Decision

Supersedes ADR-017: its cause (glibc's choice of FMA maths) was tested and is not the cause (Context).

1. **One level everywhere.** vello_cpu draws every export, and the editor's export canvas, thumbnails and font
   previews that share `quadrule_render::render`, at fearless_simd's **fallback** level (plain scalar arithmetic), on
   every processor and system. No level is detected at run time (chris: "let's go with option A").
2. **No switch.** The `Math` choice (Fastest or Portable) is removed; `render` takes no level. Setting
   `quadrule_render::LEVEL` to anything but the fallback is a build error.
3. **The golden images change once.** They stand only after chris's look, shown how many pixels changed and
   where (ADR-017 Q2, carried over).
4. **No flag.** It lands switched on; chris's look at the golden images is the gate (ADR-017 Q3, carried over).
5. **Speed is measured on the Pi before it lands.** If the drag or draw benches miss ADR-009's 50 ms line, the
   work stops and is reported; nothing lands (chris, 2026-11-19: "Stop and report").
6. **The FMA-off CI job is removed** (chris: "Revert it"). It changed glibc's choice only, which does not decide
   these pixels. The golden tests on CI's three systems are the guard: each now runs a fused-capable processor
   (AVX2 or NEON), so a detected level would show there as changed bytes.
7. **ADR-003 item 2 is replaced.** Its lever (`Level::baseline()`) is NEON, fused, on ARM and SSE2, unfused, on
   x86, so it would not have made computers agree.

## Context

On 2026-11-19 chris's VM (Intel Xeon E5-2470 v2: no FMA, no AVX2) failed golden images by 1 gray level on a few
curved and rotated edge pixels (ADR-017, Context). ADR-017 laid this on glibc picking FMA or plain `sin` and `cos`.
Its first step tested that and it failed (record `docs/bbq/2026-11-19-adr-identical-maths.md`, Amendments):

- CI with glibc's FMA and AVX2 off (`GLIBC tunables=glibc.cpu.hwcaps=-AVX2,-FMA,-FMA4 … fma_active=0 avx2_active=0`,
  run 44563345364) passed with the golden checksums; on the VM, glibc's plain maths (`-AVX`) left the groups page
  unchanged.
- fearless_simd 0.7.0, which vello_cpu uses, fuses multiply-add at the AVX2 level (`_mm_fmadd_ps`) and NEON
  (`vfmaq_f32`), and does not at SSE4.2, SSE2 or the fallback (`a * b + c`). The VM runs at SSE4.2.
- On CI, the fast level against the portable one differs on the groups page by `portable_diff_px=11
  portable_max_diff=1`, the VM's 11 pixels; on the VM the two agree.

Pinning the level, rather than detecting it, makes the arithmetic the same on every processor. The fallback is the
one level every processor has. It needs fearless_simd's `force_support_fallback` feature, which only makes
`Level::fallback()` exist: fearless_simd 0.7.0 becomes a direct dependency of `quadrule-render` at the version
vello_cpu already uses (`Cargo.toml`); no version changes and nothing is patched.

Speed on the VM, release build, a whole Nomad page rendered 10 times (a throwaway test, not kept): slice 6.8 → 8.0
ms, groups 14.5 → 19.0, patterns 13.9 → 15.3, fonts 17.3 → 17.0 (fast level → fallback). The Pi is measured before
landing (item 5).

What it does not cover: the system maths library's `sin`, `cos` and the rest still come from each system. On the
evidence above they move no pixel today on Linux (FMA and plain), the Pi, macOS or Windows; ADR-017's `libm` route
stays on file in its Alternatives should that change.

Drivers:
- Identical export bytes on every computer, by construction (ADR-003's first driver).
- kurbo and vello unchanged and unpatched (chris, ADR-017's record).
- The editor's canvas keeps showing the exact export pixels (rung 2).

## Alternatives

### Fallback level everywhere (chosen: identical by construction, nothing patched)
- Good: one setting; no patches; every processor computes the same; the editor still shows the export pixels.
- Bad: slower than SIMD (1.0–1.3× on the VM; the Pi to be measured); golden images change once.

### A patched fearless_simd with an unfused multiply-add at every level (option B)
- Good: keeps SIMD speed.
- Bad: a patch under vello, redone at every upgrade.

### A tolerance of 1 level on computers without FMA (option C)
- Good: small; no golden image changes.
- Bad: gives up identical bytes on such computers.

### The app's own `sin` and the rest from `libm` (ADR-017, superseded)
- Good: covers the system maths library, which the fallback level does not.
- Bad: aimed at glibc, which the FMA-off run showed does not move these pixels.

## Consequences

- Golden images and the migration tests' stored export hashes are re-made once, after chris's look.
- Export, thumbnails and the export canvas are somewhat slower; ADR-009's budgets on the Pi decide (item 5).
- A computer without FMA, like chris's VM, passes the golden tests.
- `quadrule-render` depends on fearless_simd directly; an upgrade of vello_cpu moves both together.

## Asks left open

1. Ratify ADR-018, after the golden images' look and the Pi's speed numbers.

## History

- 2026-11-19: drafted from chris's ruling, "let's go with option A", and three answers in one round (goforit: no
  more questions, every point cited; the FMA-off job: "Revert it"; too slow on the Pi: "Stop and report"). Record
  `docs/bbq/2026-11-19-adr-identical-maths.md`, Amendments. ADR-017 superseded and ADR-003 item 2 given a banner in
  the same commit.
