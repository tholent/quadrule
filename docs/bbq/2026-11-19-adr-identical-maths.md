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

(written as each is answered)

## Defaulted

| call | who does it that way | veto |
|---|---|---|

## Owed

## Amendments
