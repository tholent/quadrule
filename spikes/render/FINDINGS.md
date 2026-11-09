# Rendering spike: findings

Date: 2026-11-09. Machine: Raspberry Pi 5, 4 GB, aarch64, Rust 1.99.1, Mesa 26.2.6. Crates: tiny-skia 0.12.0,
vello_cpu 0.3.0, vello 0.11.0, wgpu 30.0.1, eframe/egui-wgpu 0.37.0, png 0.18.1. Every `cargo` command below also
had `CARGO_TARGET_DIR=~/.cargo/quadrule-spike-target CARGO_BUILD_JOBS=1 --manifest-path spikes/render/Cargo.toml
--release`. Saved by the lead from the worker's report, word for word (the worker could not write report files).

```
CHECK id=R01 name=odd_width_lines_exact result=pass lines=8 fails=[] cmd="cargo test r01 -- --nocapture"
CHECK id=R02 name=even_width_lines_exact result=pass lines=8 fails=[] cmd="cargo test r02 -- --nocapture"
CHECK id=R03 name=unsnapped_lines_show_gray_fringe result=pass blurred=8/8 cmd="cargo test r03 -- --nocapture"
CHECK id=R04 name=rhythm_pixel_locked_even result=pass physical_uneven=true detail=manta:physical_gaps=[70, 71]:locked_gap=71px(6.011mm),a5x:physical_gaps=[51, 52]:locked_gap=53px(5.957mm) cmd="cargo test r04 -- --nocapture"
CHECK id=R05 name=png_gray8_no_alpha_text_chunk result=pass detail=meta=true:color=Grayscale:depth=Eight:text=true:bytes=56303,meta=false:...:text=false:bytes=56239 cmd="cargo test r05 -- --nocapture"
CHECK id=R06 name=export_bytes_identical_twice result=pass manta:sha256=c699f2055296bd7c a5x:sha256=cdf82e8a164498b8 cmd="cargo test r06 -- --nocapture"
CHECK id=R07 name=vello_cpu_snapped_lines_exact result=pass lines=16 fails=[] cmd="cargo test --features vello-cpu r07 -- --nocapture"
CHECK id=R08 name=vello_cpu_vs_tiny_skia_pixel_diff result=info manta:differing_px=51145(1.041%):max_diff=56 a5x:differing_px=37004(1.408%):max_diff=40 cmd="cargo test --features vello-cpu r08 -- --nocapture"
CHECK id=R09 name=vello_cpu_simd_vs_scalar_and_repeat result=pass manta:simd_vs_scalar_diff_px=0:repeat_identical=true:sha256=f0410c87a5c8d6ac a5x:simd_vs_scalar_diff_px=0:repeat_identical=true:sha256=04a388832ec27073 cmd="cargo test --features vello-cpu r09 -- --nocapture"
CHECK id=R10 name=cpu_render_time_ms result=info profile=manta items=136 tiny_skia_ms=9.0 vello_cpu_ms=13.3 png_encode_ms=32.3 cmd="cargo run --example bench --features vello-cpu"
CHECK id=R10 name=cpu_render_time_ms result=info profile=a5x items=136 tiny_skia_ms=6.3 vello_cpu_ms=6.7 png_encode_ms=22.8 cmd="cargo run --example bench --features vello-cpu"
CHECK id=R11 name=wgpu_vulkan_adapters result=pass adapters=["V3D 7.1.7.0","llvmpipe (LLVM 19.1.7, 128 bits)"] cmd="cargo run --example gpu --features gpu,vello-cpu"
CHECK id=R12 name=vello_gpu_offscreen_V3D result=pass manta:warm_ms=391.9:vs_tiny_skia_diff_px=49192:repeat_identical=true a5x:warm_ms=228.2:vs_tiny_skia_diff_px=35617:repeat_identical=true cmd="cargo run --example gpu --features gpu,vello-cpu"
CHECK id=R12b name=vello_gpu_snapped_lines_exact_V3D result=pass lines=16 fails=[] cmd="cargo run --example gpu --features gpu,vello-cpu"
CHECK id=R13 name=vello_gpu_offscreen_llvmpipe result=pass manta:warm_ms=72.0:vs_tiny_skia_diff_px=49420 a5x:warm_ms=43.9:vs_tiny_skia_diff_px=35773 cmd="cargo run --example gpu --features gpu,vello-cpu"
CHECK id=R13b name=vello_gpu_snapped_lines_exact_llvmpipe result=pass lines=16 fails=[] cmd="cargo run --example gpu --features gpu,vello-cpu"
CHECK id=R14 name=eframe_wgpu_vello_canvas_compiles result=pass missing_packages=none cmd="cargo build --example canvas --features canvas"
CHECK id=R15 name=eframe_canvas_runs_on_display result=owed reason="winit: neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set" cmd="cargo run --example canvas --features canvas"
CHECK id=R16 name=clippy_all_targets result=pass warnings=0 cmd="cargo clippy --all-targets --features canvas,vello-cpu"
```

## In plain words

- **Snapped lines are exact.** With snapping on (lines moved to sit on whole pixels), every line is exactly its
  role's width in pixels, with no soft grey edge, on Manta and A5 X and in all three renderers. With snapping off,
  all 8 test lines get soft edges (R03).
- **The PNG format is right:** 8-bit grey, no transparency, optional text note written when asked for (R05).
- **Export repeats exactly:** rendering twice gives byte-identical files (R06, R09).
- **vello_cpu doesn't depend on its speed-ups:** output is the same with SIMD (doing several pixels at once) on or
  off (R09). A good sign for identical output on every computer; only a test on an Intel machine can prove it.
- **The two CPU renderers differ slightly:** about 1% of pixels, all on the soft edges of curves (R08). Golden-image
  tests (comparing output against saved reference PNGs) must be tied to one renderer.
- **The Pi's graphics chip works, but slowly:** vello on the chip (V3D) draws a full Manta page correctly in about
  390 ms; the CPU stand-in driver (llvmpipe) takes 72 ms; vello_cpu takes 13 ms.
- **The GPU must never make the export file:** the two graphics drivers give slightly different pixels (R12 vs R13).
- **The egui window builds with no extra system packages.** eframe and vello share one wgpu (30.0.1), so the canvas
  can use egui's own graphics connection (R14).

## Still to do

- **R15:** run the window in a desktop session, at the Pi's own screen or on another machine.
- **Other machines:** run R06 and R09 on x86 Linux, macOS and Windows and compare the sha256 values.
- **macOS and Windows builds** not tried; no such machine here.

## Parked for chris

1. **Scale vs rhythm on A5 X.** The napkin's default "Scale" constraint (§5.4) shrinks the 6 mm ruling to 51–52 px
   on A5 X; a pixel-locked rhythm (§6.3, resolved per profile) wants 53 px (R04). Which wins for rhythm-bound
   content?
2. **Canvas on weak graphics chips.** On the Pi, vello on the graphics chip is about 30× slower than vello_cpu for a
   full page. Should the canvas fall back to vello_cpu on weak chips? Measure on a desktop graphics card first.
3. **Build folder.** The worker built in `~/.cargo/quadrule-spike-target` with one job, not the shared
   `~/.cache` folder, and says chris told it to stay inside `~/.cargo`. The lead did not see that instruction in its
   own conversation (it may have come as an answer to a permission prompt) — chris to confirm. Peak memory was about
   1.5 GB of 4 GB.
