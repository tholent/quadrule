---
status: accepted
status-note: ratified by chris 2026-11-09 ("ratify all three")
ratified-by: chris
ratified-on: "2026-11-09"
owner: chris
kind: architecture
built: not started
proof: queued in STATUS.md (Next) for the real render crate; evidence in spikes/render/FINDINGS.md
supersedes: nothing
amends: nothing
consulted: rendering spike worker (Opus 5.5 subagent)
revisit-when: the cross-machine hash check (spike R06/R09 on x86 Linux, macOS, Windows) shows differences, or vello_cpu's API breaks badly at an upgrade
---
# ADR-003 — PNG export renders with vello_cpu; the GPU never makes export files; tiny-skia stays as a test check

## Decision

1. The export renderer (the part that makes the PNG file) is **vello_cpu**.
2. If the cross-machine check shows Intel and ARM giving different bytes, export switches to vello_cpu's portable
   scalar mode (`Level::baseline()`).
3. **The GPU never makes export files.**
4. **tiny-skia stays in the test suite only**, as an independent second check of the snapped-line pixel rules.
5. Golden-image tests are tied to vello_cpu.
6. Owed before this is fully proven: the spike's R06 and R09 checks on x86 Linux, macOS and Windows (GitHub Actions,
   napkin §15.3), comparing PNG hashes with the Pi's.

The canvas renderer is not decided here; it stays the napkin's vello-on-wgpu choice until measured on a desktop GPU.

## Context

The napkin leaves the export renderer to this spike (`napkins/quadrule-spec.md:530`). The rendering spike
(`spikes/render/FINDINGS.md`) on a Raspberry Pi 5 found: snapped lines exact in tiny-skia, vello_cpu and vello GPU
(R01, R02, R07, R12b); byte-identical repeat renders (R06, R09, R12); vello_cpu output identical with SIMD on or off
(R09); a full Manta page in 9 ms (tiny-skia), 13 ms (vello_cpu), 392 ms (vello on the Pi's GPU) (R10, R12); two
GPU drivers giving different pixels (R12 vs R13); the CPU renderers differing on about 1% of pixels, all on curve
edges (R08).

Drivers:
- Identical bytes on every computer.
- One display list for canvas and export, with no translation layer.
- A lever to pull if machines disagree.

## Alternatives

### vello_cpu (chosen: same family as the canvas, and a switch for portable maths)
- Good: shares kurbo/peniko types with vello; can draw glyph runs; SIMD on/off gives the same output.
- Bad: young (0.3); its API will change and need upkeep at each upgrade.

### tiny-skia
- Good: mature, slow-moving, slightly faster here.
- Bad: its own shape types (a second translation); no switch to force portable maths.

### GPU export
- Good: fast on a strong GPU.
- Bad: two drivers gave different pixels (R12, R13); breaks golden-image tests.

## Consequences

- vello_cpu upgrades need care; pin it in `Cargo.lock` and re-run golden images at each upgrade.
- CI must compare export hashes across three operating systems before this is fully proven.
- Two questions the spike parked stay open for chris (in `docs/STATUS-debt.md`): Scale vs pixel-locked rhythm on
  A5 X, and a vello_cpu fallback for the canvas on weak GPUs.

## History

- 2026-11-09: drafted by the rendering spike worker as `spikes/render/ADR-PROPOSAL.md` (Status: PROPOSED, for chris
  to ratify). Carried over without change of substance. Prior header values: status `proposed`, status-note
  `drafted 2026-11-09 from spikes/render/ADR-PROPOSAL.md; awaiting ratification`.
- 2026-11-09: ratified by chris ("ratify all three").
