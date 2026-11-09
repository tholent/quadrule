---
status: accepted
status-note: ratified by chris 2026-11-09 ("ratify all three")
ratified-by: chris
ratified-on: "2026-11-09"
owner: chris
kind: architecture
built: not started
proof: queued in STATUS.md (Next) for the real format crate; evidence in spikes/format/FINDINGS.md
supersedes: nothing
amends: nothing
consulted: format spike worker (Opus 5.5 subagent); talk-through with Claude (docs/proposals/talk-2026-11-09-format-versions-and-hinting.md)
---
# ADR-001 — The `.quad` file is a ZIP of CBOR as the napkin describes, with text outlines stored as a shape table per profile

## Decision

Quadrule's project file is built exactly as the napkin describes (`napkins/quadrule-spec.md:572-597`): a ZIP
named `.quad` holding CBOR entries (via serde and ciborium) with field names kept, Deflate for the CBOR entries, a
semver `format_version`, a migration chain, newer files read-only when they need an unknown feature, and unknown
optional fields kept on save. On top of that:

1. **Text outlines (C1).** Each profile's `outlines/<profile_id>.cbor` holds a table of each distinct letter shape
   once (in half font units, exact) plus each text box's letter positions in mm.
2. **Outlines are saved for exactly the document's profiles** (primary + secondaries). There is no separate
   single-profile / multi-profile file mode. (talk D2)
3. **Saved outlines are plain (unhinted) shapes**, unless that profile has text hinting switched on; then the
   table key is letter + pixel size and the hinted shapes are saved. (talk D3; hinting itself is ADR-002)
4. **Keeping unknown fields:** every record that may grow carries an `extra` bucket (`#[serde(flatten)]`), and
   each record type has a test that an unknown field survives a save.
5. **Resaved newer files** keep their own `format_version`; the manifest also records which app version last
   saved the file, so a newer app knows an older one touched it and can re-check derived data. (talk D1)
6. **Font signature:** for each font a document uses, the file stores a small measured record (class, weight,
   width, italic, average letter width, x-height, monospace test) so a good substitute can be found when the font
   is missing. (chris's ruling on the text spike's parked item 2; used by ADR-002)
7. **Missing font, new profile:** when a profile is added while a font is missing, plain shapes are reused from
   another profile where the box is the same size; otherwise the text is badged "needs the font". (talk D7)
8. **Fuzzing:** the real `format` crate gets `cargo-fuzz` on its reader (needs nightly Rust); the stable-Rust
   mutation test (spike check F09) stays as a quick check.

## Context

The napkin settles the container and leaves open how outlines are stored and how unknown fields are kept. The
format spike (`spikes/format/FINDINGS.md`) built it on a text-heavy weekly planner (1,581 letters, 43 distinct
shapes, F11–F12): round-trip, byte-identical output, JSON debug export, migration, read-only newer files and
checksums all pass (F01–F08); a whole planner file is 30 KB and opens in under 1 ms on a Raspberry Pi 5 (F18–F19).
The two calls the spike parked were settled in a talk with chris
(`docs/proposals/talk-2026-11-09-format-versions-and-hinting.md`, D1–D7).

Drivers:
- File size on every planner, forever.
- Keep the napkin's layout (`outlines/<profile_id>.cbor`).
- Older apps must never quietly destroy newer files' data (`napkins/quadrule-spec.md:597`).

## Alternatives

### C1: each shape once per profile, plus positions (chosen: 10× smaller and keeps the napkin's layout)
- Good: 24 KB compressed for two profiles (F15); matches `outlines/<profile_id>.cbor`.
- Bad: one more step (place the shape) when drawing from outlines.

### A: every letter's full outline, as tagged commands
- Good: simplest code.
- Bad: 284 KB (F13), about 10× bigger, paid by every planner.

### B: A packed into byte strings
- Good: a little smaller than A.
- Bad: 237 KB (F14); packing adds code without fixing the cause.

### C2: one shape table shared by all profiles
- Good: 4 KB smaller than C1 (F16).
- Bad: a file the napkin doesn't list; ties the profile files together; can't hold per-profile hinted shapes.

### Unknown fields: read raw CBOR and merge edits back
- Good: no per-record buckets.
- Bad: much more code; every edit must be merged into a raw tree.

### Resaved file stamped with the older app's version
- Good: simple.
- Bad: the newer app would re-run its upgrade step on return and could overwrite extras carried through safely.

## Consequences

- Each new record type must add its `extra` bucket and its survival test; forgetting one silently drops fields,
  so the test per type is the guard.
- Serde's strict "reject unknown fields" mode can't be used.
- The manifest grows a "last saved by" field and a font-signature list.
- The real `format` crate needs nightly Rust in CI for fuzzing.

## History

- 2026-11-09: drafted by the format spike worker as `spikes/format/ADR-PROPOSAL.md` (Status: PROPOSED, not
  ratified). Before ratifying, the lead told chris it would fold in talk decisions D1–D7 and the font-signature
  ruling; items 2, 3, 5, 6 and 7 of the Decision are those folds. Prior header values: status `proposed`,
  status-note `drafted 2026-11-09 from spikes/format/ADR-PROPOSAL.md and the talk record; awaiting ratification`.
- 2026-11-09: ratified by chris ("ratify all three").
