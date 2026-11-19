---
status: superseded
status-note: superseded by ADR-018 on 2026-11-19
ratified-by: chris
ratified-on: "2026-11-19"
owner: chris
kind: architecture
built: not started
proof: "queued in `docs/bbq/2026-11-19-adr-identical-maths-NEXT.md`"
supersedes: nothing
amends: "ADR-003 §Decision item 2 (the lever if machines disagree)"
revisit-when: a system's linker refuses the app's own maths functions, the `libm` crate or the Rust toolchain changes how they link, or CI's FMA-off golden job fails
---
# ADR-017 — Every computer computes the same sines: the app links libm's maths functions over the system's, and CI checks the golden images with FMA turned off

## Decision

1. **The app brings its own maths.** Every program quadrule builds (the app, the tests, `xtask`) provides its own
   copies of the C library's maths functions (`sin`, `cos`, `tan`, `sincos`, `asin`, `acos`, `atan`, `atan2`, `exp`,
   `exp2`, `expm1`, `log`, `log1p`, `log2`, `log10`, `pow`, `hypot`, `cbrt`, `sinh`, `cosh`, `tanh`, and the `f32`
   forms), each a thin call into the pure-Rust **`libm`** crate. So every library inside the program, kurbo and
   vello included, unchanged, gets the same answer on every processor and system, instead of the system's maths
   library, whose answer can differ in the last bit by processor (record Q1).
2. **Proven on all three systems.** CI must build and pass on Linux, macOS and Windows with item 1 in place, and a
   test must show that a Rust `f64::sin` call really reaches `libm`. A system whose linker refuses item 1 falls back
   to lightly patched copies of kurbo and of vello's curve code that call `libm` directly; that fallback is reported
   to chris before it is built (Q1).
3. **The golden images may change once.** If `libm`'s last digits move any pixel, new golden images are made and
   stand only after chris's look, shown how many pixels changed and where (Q2).
4. **No flag.** It lands switched on; chris's look at the golden images is the gate. This is chris's ruling for this
   job; ADR-007's amendment of 2026-11-10 stands unchanged for other work (Q3).
5. **CI checks with FMA turned off.** A permanent CI job runs the golden tests on Linux x86_64 with glibc told to act
   as if the processor had no FMA or AVX2 (`GLIBC_TUNABLES`). Before item 1 it must fail with the VM's checksums,
   which confirms the cause; after, it must pass, on every push (Q4).
6. **ADR-003 item 2 is replaced.** Its lever (switch export to vello_cpu's portable maths) is the wrong lever: the
   difference is not in vello_cpu's arithmetic (see Context). ADR-003 otherwise stands.

## Context

On 2026-11-19 chris's new development VM (Intel Xeon E5-2470 v2: no FMA, no AVX2) failed golden images on the
devices, groups, patterns, patterns-turned, repeats and pattern-angle pages, which pass on CI's Linux x86_64
(`gh run list`, `da8f6cb`) and on the Pi. Measured on the Pi from the VM's own output files (record P1 to P3):

- devices, Nomad: 7 pixels of 2,628,288 differ; groups, Nomad: 11; every difference is 1 gray level of 256.
- Every one sits on a curved or rotated edge: ellipse and circle outlines, a rotated rectangle. None is on a straight
  horizontal or vertical line, none in text.
- On the VM, vello_cpu's fast and portable arithmetic give identical pixels (`portable_diff_px=0`,
  `crates/render/tests/groups.rs`), so ADR-003's item 2 lever would change nothing.

The likely cause, an inference until item 5's job confirms it: on x86_64, glibc picks FMA or plain versions of
`sin`, `cos` and others when a program starts, by what the processor offers, and the two can differ in the last bit.
CI's machines have FMA; the Pi's glibc always uses it; the VM has none. Sines and cosines place every curve and every
rotated corner, through quadrule's own layout code (`crates/layout/src/`, `crates/model/src/geom.rs`) and through
kurbo and vello (record P4, P5). macOS and Windows use their own maths libraries and agree today without a guarantee.

ADR-003's first driver is "Identical bytes on every computer". The difference cannot be seen on paper or e-ink; what
it breaks is that promise, and with it the golden tests on any computer like the VM.

Drivers:
- Identical export bytes on every computer, by construction, not by luck (ADR-003).
- Keep kurbo and vello as they are: established, reviewed libraries (chris, before the ruling).
- A check that fails if the difference ever comes back.

## Alternatives

### The app links its own maths, from `libm` (chosen: every library gets the same answers with none of them changed)
- Good: covers quadrule, kurbo, vello and anything added later; one small crate; `libm` is already in the build.
- Bad: depends on each system's linker taking the program's own functions over the system's; must be proven on
  three systems, and a toolchain change could undo it (hence the CI test and Revisit when).

### Patched copies of kurbo and vello's curve code (kept as the fallback)
- Good: certain to work everywhere; kurbo already has a `libm` switch.
- Bad: kurbo's switch is tied to turning its `std` part off, which vello needs on, so it means patching; every
  upgrade of those libraries redoes the patch.

### quadrule's own code only
- Good: the least work.
- Bad: very likely not enough: the ellipse edges that differed are drawn by kurbo.

### A tolerance on machines without FMA (overruled)
The lead recommended this after chris's challenge (exact bytes on CI and the Pi; up to 1 gray level on a handful of
edge pixels elsewhere, said in the CHECK line). chris ruled option 1 with it in front of him: "ok, I understand more
clearly. We can move forward with option 1".
- Good: small work, no build-time machinery.
- Bad: gives up identical bytes on every computer; a looser test on such machines.

### Document the difference only
- Good: no work.
- Bad: golden tests fail for good on any computer without FMA.

## Consequences

- One new crate, `quadrule-maths` in `crates/maths/` (named by chris 2026-11-19), that every program links; the steps are in the next-agent prompt
  (`docs/bbq/2026-11-19-adr-identical-maths-NEXT.md`): the FMA-off job first (it confirms the cause), then the crate
  and its routing test, then the three systems, then any new golden images for chris's look.
- Golden images and the migration tests' stored export hashes may change once, after chris's look.
- Speed: `libm` may be slower than the system's maths; ADR-009's budgets on the Pi apply as before.
- The FMA-off job adds a few minutes to CI on every push.
- Developers on computers without FMA can trust the golden tests again.

## History

- 2026-11-19: drafted from chris's answers, 4 questions (`docs/bbq/2026-11-19-adr-identical-maths.md`); ADR-003 gets
  a dated note and banner at item 2 in the same commit.
- 2026-11-19: chris answered ask 1, the crate's name: "sounds good" (to `quadrule-maths` in `crates/maths/`); the
  Consequences line names it and the Asks left open section, now empty, is removed.
- 2026-11-19: ratified by chris ("ratified"). Prior header values: status `proposed`, status-note `drafted 2026-11-19
  from chris's answers (docs/bbq/2026-11-19-adr-identical-maths.md); awaiting ratification`.
- 2026-11-19: superseded by ADR-018 (`ADR-018-export-at-the-fallback-level.md`): the FMA-off CI run (item 5's red arm)
  passed with the golden checksums, so glibc is not the cause; vello_cpu's SIMD level is. Prior header values: status
  `accepted`, status-note `ratified by chris 2026-11-19 ("ratified")`. Body unchanged.
