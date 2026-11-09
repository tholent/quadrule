# PROPOSED — Export renderer: vello_cpu, with tiny-skia kept as a test check

Status: **PROPOSED**, for chris to ratify. Not in `docs/decisions/` until ratified (format not chosen yet).
Date: 2026-11-09. Evidence: `spikes/render/FINDINGS.md` (checks R01–R14).

## The question

The napkin (§10.1) leaves the export renderer open: "CPU rasterization (e.g. `tiny-skia` or vello's CPU path)
… finalized during a rendering spike". The export renderer is the part that makes the PNG file, so it must give
the same bytes every time, on every computer.

## What was measured (on this Raspberry Pi 5)

| | tiny-skia 0.12 | vello_cpu 0.3 | vello 0.11 GPU (V3D) |
|---|---|---|---|
| Snapped lines exact (no blurry edge) | yes (R01, R02) | yes (R07) | yes (R12b) |
| Same bytes when run twice | yes (R06) | yes (R09) | yes (R12) |
| Same output with and without CPU speed-ups (SIMD) | can't switch them off | **yes** (R09) | n/a |
| Full Manta page | 9 ms | 13 ms | 392 ms |
| Same output on two GPU drivers | n/a | n/a | **no** (R12 vs R13) |

The two CPU renderers draw curves very slightly differently (about 1 % of pixels differ, only on the edges of
curves and dots; R08). Neither is "wrong"; they smooth edges differently.

## Options, by lifetime cost

1. **vello_cpu for export (recommended).** Same family as the canvas renderer (vello), with the same shape
   types (kurbo) and colours (peniko), so the canvas and the export draw from one display list with no
   translation layer. It can render text glyphs itself (`glyph_run`). It has a switch that forces plain,
   portable maths (`Level::baseline()`), which is the best tool available for "identical on every computer"
   if Intel and ARM machines turn out to differ. Cost: it is young (0.3), so its API will change and need
   upkeep at each upgrade.
2. **tiny-skia for export.** Mature and slow-moving, the napkin's first suggestion. Cost: its own shape types, so a
   second translation from the display list, and no switch to force portable maths. If Intel and ARM disagree,
   there is no lever to pull.
3. **GPU for export.** Ruled out: two drivers on the same machine gave different pixels (R12, R13), which breaks
   the golden-image tests (§15.3).

## Proposal

- Export renders with **vello_cpu**, using its portable scalar mode if the cross-machine check shows differences.
- **tiny-skia stays in the test suite only**, as an independent second opinion on snapped-line pixel rules.
- Owed before this is final: the same checks on an x86 Linux, macOS and Windows machine (GitHub Actions per §15.3),
  comparing PNG hashes with this Pi's.

## Not decided here (canvas)

The canvas renderer is the napkin's vello-on-wgpu choice. It works on this Pi's GPU (R12) but slowly. See
"Parked for chris" in FINDINGS.md: that is a separate decision, best measured on a desktop GPU first.
