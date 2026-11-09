# PROPOSED: project file format (`.quad`) as the napkin describes, with outlines as a shape table

Status: **PROPOSED**, not ratified. For chris to ratify, change or reject. 2026-11-09. Evidence:
`spikes/format/FINDINGS.md` (checks F01–F19).

## Context

The napkin settles the container (`napkins/quadrule-spec.md:572-597`): a ZIP called `.quad` holding CBOR entries
encoded with `ciborium` and `serde`, field names kept, Deflate compression, a semver `format_version`, a migration
chain, unknown optional fields kept on save, and text outlines per profile. It leaves open how the outlines are
stored and how "keep unknown fields" is done. The format spike tested both on a text-heavy weekly planner.

## Options weighed (lifetime cost)

**Outline storage**

| Option | Size (2 devices, compressed) | Lifetime cost |
|---|---|---|
| A. Every letter's full outline, as tagged commands | 284 KB (F13) | Simplest code, but files 10× bigger forever; every planner pays it |
| B. Same, packed into byte strings | 237 KB (F14) | Still ~10× bigger; packing adds code without fixing the cause |
| **C1. Each letter shape once per device file, plus positions** | **24 KB (F15)** | Small; matches the napkin's `outlines/<profile_id>.cbor` layout; one more step (place the shape) when drawing from outlines |
| C2. C1 with one table shared by all devices | 20 KB (F16) | Saves 4 KB more; adds a file the napkin doesn't list and ties the device files together |

**Keeping unknown fields**

| Option | Lifetime cost |
|---|---|
| **An `extra` bucket (`#[serde(flatten)]`) on every record that may grow** | Works today with plain serde (F05). Each record must remember its bucket: a test per record type guards that. Can't use strict "reject unknown" mode. Speed cost unmeasurable at this size (F19). |
| Read everything as raw CBOR and merge edits back | No per-record buckets, but every edit has to be merged into a raw tree: much more code, easy to get wrong |
| Don't keep unknown fields | Goes against the napkin (`:597`); older apps would quietly destroy newer files' data |

## Recommendation

1. Adopt the container exactly as the napkin describes (F01–F08: round-trip, byte-identical output, JSON debug
   export, migration chain, newer files read-only, checksums).
2. Store text outlines as **C1**: per device file, a table of each distinct letter shape (in half font units, exact)
   plus each text box's letter positions in mm.
3. Keep unknown fields with an `extra` bucket on every record that may grow, plus a test per record type that an
   unknown field survives a save.
4. Set up `cargo-fuzz` (needs the nightly compiler) for the reader in the real `format` crate; the stable-Rust
   mutation test (F09) stays as a quick check.

Two calls are parked for chris in `FINDINGS.md`: the version label on a resaved newer file, and whether saved
outlines must match each device's pixels (decide with the text spike).
