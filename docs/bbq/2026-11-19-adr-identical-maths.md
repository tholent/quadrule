# bbq record — quadrule: ADR-017, the same maths on every computer

2026-11-19 · makeadr, mode new (amends ADR-003) · 4 questions · lead: Opus 5.5 at high effort (this session).

## How it came up

chris moved development to a new VM (Intel Xeon E5-2470 v2, 12 threads, 32 GB; no FMA or AVX2). On it,
`cargo test -j 12 -p quadrule-render --no-fail-fast` failed golden images on the devices, groups, patterns,
patterns-turned, repeats and pattern-angle pages (chris's paste, 2026-11-19). The same tests pass on CI's Linux x86_64
at `da8f6cb` (`gh run list -R tholent/quadrule`) and on the Pi.

Three options were put to chris: 1 do the maths ourselves through one pure-Rust library; 2 a tolerance on machines
without FMA; 3 document the difference only. chris, word for word: "let's go with 1". He chose 4 questions for the rest.

## Premises (each checked this session)

| # | premise | evidence |
|---|---|---|
| P1 | The VM's output differs from the golden images by 1 gray level on a few edge pixels | Pixel compare on the Pi of the VM's files (sha256 `e2cc42…`, `d78f33…`, matching the VM's CHECK lines) against `crates/render/tests/golden/`: devices a6x2 7 px of 2,628,288, groups a6x2 11 px, every difference 1 level |
| P2 | Every differing pixel is on a curved or rotated edge (ellipse and circle outlines, a rotated rectangle) | `scratchpad/diff-devices.png`, `diff-groups.png` (red boxes on the golden image) |
| P3 | vello_cpu's SIMD level is not the cause | VM: `groups_golden … portable_diff_px=0 portable_max_diff=0` (`crates/render/tests/groups.rs:80`, Fastest against Portable) |
| P4 | quadrule's own export code calls sine, cosine and the like | `crates/layout/src/{border,lib,pattern,shapes,snap}.rs`, `crates/model/src/geom.rs` |
| P5 | So do the libraries it draws with | `kurbo-0.13.1/src/{affine,arc,circle,common,cubicbez,fit,offset,quadbez,…}.rs`; `vello_common-0.3.0/src/{flatten_simd,encode}.rs` |
| P6 | kurbo can use `libm` instead of the system's maths, but only without its `std` feature, which vello_cpu turns on | `kurbo-0.13.1/Cargo.toml` `[features]`; `Cargo.toml:88` |
| P7 | The `libm` crate is already in the build | `cargo tree -p quadrule-render` lists `libm-0.2.16` |
| P8 | ADR-007 as amended: a change to export bytes lands behind a flag | `AGENTS.md`, amendment of 2026-11-10 |

Inference, not proven: glibc on x86_64 picks FMA or plain versions of `sin`, `cos` and others at start-up by the
processor; the plain versions can differ in the last bit. The Pi's glibc always uses FMA.

## Rounds

### Before round 1: chris's challenge and the ruling

chris, word for word: "I'm debating whether we want to do all our own maths. I'm assuming kurbo is rather established
and reviewed. We are throwing that all away because of a few pixels on a sheet being a little off. These templates
aren't for building rocket ships and human imperceptible differences aren't that big of a deal".

The lead explained that option 1 keeps kurbo and vello and changes only where their sine, cosine and the like come
from (the system's maths library, or the pure-Rust `libm`), and, given chris's weighting, **withdrew its
recommendation of option 1 and recommended option 2** (exact bytes on CI and the Pi; on a machine without FMA the
golden tests allow up to 1 gray level on a handful of edge pixels, said in the CHECK line). The withdrawn
recommendation is kept here, not deleted: it was withdrawn because chris weighs imperceptible differences as minor
and option 1 needs build-time machinery proven on three systems.

chris, word for word: "ok, I understand more clearly. We can move forward with option 1". Option 2 is overruled.

### Round 1 (4 questions)

- **Q1 Reach.** chris: "When the app is built (Recommended)". The app supplies its own sine, cosine and the rest
  from `libm` as it is built, so every library inside it (kurbo and vello included, unchanged) uses them. It must be
  proven to build on Linux, macOS and Windows in CI; a system that refuses falls back to patched copies of kurbo and
  vello's curve code that call `libm`.
- **Q2 Golden images.** chris: "Allowed, re-approved (Recommended)". The golden images may change once; the session
  shows chris how many pixels changed and a picture of where, and they stand only after his look.
- **Q3 Flag.** chris: "No flag". It lands switched on; chris's look at the golden images is the only gate. This is
  chris's ruling for this job; ADR-007's amendment of 2026-11-10 is not changed by it.
- **Q4 CI guard.** chris: "Permanent job (Recommended)". A CI job runs the golden tests on Linux with FMA turned off
  (`GLIBC_TUNABLES`) on every push. Before the fix it should fail with the VM's checksums (confirming the cause);
  after, it passes and keeps guarding.

## Closing

- "Anything else to protect or avoid?" chris: "Nothing more (Recommended)".
- "Make the ADR now, and also a prompt for the next agent?" chris: "ADR + next-agent prompt (Recommended)".
- Made: `docs/decisions/ADR-017-same-maths-on-every-computer.md` (proposed); ADR-003 gets a dated banner at item 2 and
  a History line; the prompt is `docs/bbq/2026-11-19-adr-identical-maths-NEXT.md`.

## Defaulted

| call | who does it that way | veto |
|---|---|---|
| The functions covered are the C maths library's transcendental ones that `libm` provides (`sin` … `tanh`, and `f32` forms); not `sqrt`, `floor`, `round` or `fma`, which IEEE 754 requires to be exact everywhere | IEEE 754 §5.4 (exactly rounded operations) | veto: cover those too |
| The FMA-off job runs on Linux x86_64 only, and is non-blocking (`continue-on-error`) until the fix lands, then required | glibc is where the choice happens; `main` stays green while the job proves the cause | veto: required from the start |
| The routing test sweeps many inputs and requires `f64::sin(x).to_bits() == libm::sin(x).to_bits()` (and the same for each function); its red arm is the crate left out, which must make it fail | ADR-006: tests print one CHECK line | veto: none expected |
| The exact `GLIBC_TUNABLES` value is taken from glibc's own manual for the runner's glibc, and the job prints the processor features glibc used (`ld.so --list-diagnostics`) to show FMA is off | AGENTS.md: take values from the spec, never memory | veto: none expected |

## Owed

- (2026-11-19 asks: both answered, see Amendments)
- chris: ADR-017's premise failed (Amendments, 2026-11-19, the building session). Which way now: supersede ADR-017,
  keep or revert the FMA-off CI job (`60eb202`), and which of the options in the session's report.

## Amendments

- 2026-11-19, chris, word for word: "1. sounds good 2. ratified". The crate is `quadrule-maths` in `crates/maths/`;
  ADR-017 is ratified.
- 2026-11-19, the building session (Opus 5.5, high), on chris's VM: **the premise is wrong; work stopped at the
  prompt's first step, before building anything.** The FMA-off CI job (`60eb202`, run 44563345364, job
  135698321627) had glibc's FMA and AVX2 off (`GLIBC tunables=glibc.cpu.hwcaps=-AVX2,-FMA,-FMA4 fma_cpuid=1
  fma_active=0 avx2_cpuid=1 avx2_active=0`, glibc 2.43) and **passed with the golden checksums**:
  `devices_golden profile=supernote.a6x2 result=pass sha256=f32e695d…` and `groups_golden profile=supernote.a6x2
  result=pass sha256=1f2233cc…`, not the VM's `e2cc42…` and `d78f33…`. Also on the VM (glibc 2.41), glibc's plain
  maths in place of its AVX ones (`GLIBC_TUNABLES=glibc.cpu.hwcaps=-AVX`) left the groups page unchanged
  (`sha256=d78f33f29a37…` both ways). glibc's choice of maths is not the cause.
- The cause, found in the same run: **vello_cpu's SIMD level, through fearless_simd 0.7.0's multiply-add.** It is
  fused (one rounding) at the AVX2 level (`_mm_fmadd_ps`, `fearless_simd-0.7.0/src/generated/avx2.rs:359`) and the
  NEON level (`vfmaq_f32`, `neon.rs:355`), but unfused (`a * b + c`) at the SSE4.2 and SSE2 levels (`sse4_2.rs:357`,
  `sse2.rs:436`) and in the fallback (`fallback.rs:361`). CI's x86 runners and Windows have AVX2 and FMA, the Pi and
  CI's Mac are NEON: all fused, all match the golden images. The VM (no AVX2, no FMA) runs at SSE4.2: unfused. Proof:
  on CI, fast against portable differs on exactly the VM's pixels' count (`groups_golden profile=supernote.a6x2 …
  portable_diff_px=11 portable_max_diff=1`; P1: 11 px), and on the VM fast equals portable (`portable_diff_px=0`). P3
  read that the wrong way: the VM's fast path is the portable one; elsewhere the fast path is fused.
- And `Math::Portable` (`Level::baseline()`, `crates/render/src/lib.rs:89`) is not the same everywhere either: it is
  NEON (fused) on aarch64 and SSE2 (unfused) on x86_64 (`fearless_simd-0.7.0/src/lib.rs:658`). So ADR-003 item 2's
  lever, as written, would not have made every computer agree.
- ADR-017's item 1 (the app's own `sin` and the rest from `libm`) would not fix this; its ruled text is untouched.
  The FMA-off CI job (item 5) stays on `main`, non-blocking, but cannot catch this cause (`GLIBC_TUNABLES` does not
  change what `is_x86_feature_detected!` sees). The way forward is chris's (Owed).
