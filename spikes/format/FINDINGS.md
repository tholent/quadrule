# Format spike — findings

2026-11-09 · Raspberry Pi 5, Linux, Rust 1.99.1 · throwaway crate `spike-format`.
Written by the lead from the worker's report (the worker could not write report files).

## Checks

F01–F10: `cargo test -- --nocapture`. F11–F19: `cargo run --release --bin measure`.
Both with `CARGO_TARGET_DIR=$HOME/.cache/quadrule-spike-target CARGO_BUILD_JOBS=2`.

```
CHECK id=F01 name=roundtrip result=pass value=1822B
CHECK id=F02 name=deterministic_bytes result=pass value=same-doc-same-bytes
CHECK id=F03 name=json_debug_roundtrip result=pass value=5347B
CHECK id=F04 name=migrate_0_1_to_0_2 result=pass value=unit_mm->h/v_unit_mm+pixel_locked
CHECK id=F05 name=unknown_optional_fields_survive_resave result=pass value=4-levels
CHECK id=F06 name=unknown_required_feature_read_only result=pass value=boolean-ops
CHECK id=F07 name=newer_major_read_only result=pass value=1.0.0
CHECK id=F08 name=corruption_detected result=pass value=flipped-byte
CHECK id=F09 name=mutation_smoke_no_panic result=pass value=runs=3000,panics=0
CHECK id=F10 name=outline_forms_roundtrip_exact result=pass value=instances=1581
CHECK id=F11 name=planner_size result=info value=texts=99,glyphs=1581
CHECK id=F12 name=glyph_reuse result=info value=instances_per_profile=1581,distinct_glyphs=43
CHECK id=F13 name=outlines_A_naive_per_instance result=info value=raw=726161B,deflated=284367B,profiles=2
CHECK id=F14 name=outlines_B_packed_per_instance result=info value=raw=498271B,deflated=236519B,profiles=2
CHECK id=F15 name=outlines_C1_table_per_profile result=info value=raw=59303B,deflated=23977B,profiles=2
CHECK id=F16 name=outlines_C2_one_shared_table result=info value=raw=53339B,deflated=19527B,profiles=2
CHECK id=F17 name=scene_size result=info value=cbor=33424B,cbor_deflated=3968B,json=85046B
CHECK id=F18 name=quad_file_size result=info value=30194B,form=C1
CHECK id=F19 name=speed_pi5_release result=info value=write=1.80ms,read=0.81ms,scene_decode=0.32ms,runs=21,median
```

## In plain words

- **The napkin's layout works as written.** A `.quad` is a ZIP (one file bundling several) of CBOR entries (a
  compact binary cousin of JSON) with field names kept. Save → open gives back the identical document (F01); the same
  document always gives byte-identical files (F02); the readable JSON export round-trips too (F03).
- **Old files upgrade** through a chain, one step per format change (`src/migrate.rs`, F04).
- **Fields from newer versions survive a save** at four levels — manifest, document, rhythm, and an object inside a
  group (F05) — because each record carries an `extra` bucket that catches unknown fields and writes them back.
  Cost: each record type needs the bucket by hand (forgetting one silently drops fields, so the real crate needs a
  test per record type); serde's strict "reject unknown fields" mode can't be used; reading goes through a slower
  in-between buffer, which does not matter at this size (scene decode 0.32 ms, F19).
- **Newer files are handled safely:** an unknown required feature or a newer major version opens read-only and
  refuses to save (F06, F07). Per-entry SHA-256 fingerprints catch a damaged file (F08).
- **How outlines are stored matters a lot.** 1,581 letters, only 43 distinct shapes (F12). Every letter's full
  outline: 236–284 KB compressed for two devices (F13, F14). Each shape once plus where each letter sits (C1): 24 KB,
  about 10× smaller (F15). One table shared by both devices saves only 4 KB more (F16).
- **Shapes are stored exactly**, in half font units, since some TrueType points fall on .5 (F10).
- **A whole planner file is 30 KB**, opens in under 1 ms and saves in under 2 ms on the Pi (F18, F19). The scene
  shrinks from 33 KB to 4 KB compressed (F17).
- **Fuzzing (feeding random damaged files)** is feasible: it needs the nightly (experimental) Rust compiler —
  `rustup toolchain install nightly`, then `cargo install cargo-fuzz` — not installed here. As a stand-in, F09 damaged
  a valid file 3,000 ways on stable Rust; the reader never crashed.
- **Limits:** Linux only; one TrueType font (DejaVu Sans), CFF fonts not tried; naive letter placement (no shaping or
  kerning — enough to size storage, not to render); unhinted outlines; embedded fonts and images (`assets/`) not
  exercised.

## Recommendation

See `ADR-PROPOSAL.md` (PROPOSED): adopt the container as the napkin describes; store outlines as **C1** (per device,
each distinct shape once plus letter positions in mm — keeps `outlines/<profile_id>.cbor`); keep unknown fields with
`extra` buckets plus a test per record type; set up `cargo-fuzz` in the real `format` crate.

## Parked for chris

1. **Version label on a resaved newer file.** The spike keeps the file's own label (a 0.3 file saved by a 0.2 build
   stays "0.3"). The other choice stamps the build's own version. Hard to change later.
2. **Must saved outlines match each device's pixels?** The napkin keeps outlines "as rendered on each profile"
   (`napkins/quadrule-spec.md:370`). C1 is exact for unhinted text but does not capture per-device hinting or pixel
   snapping of text. Decide together with the text spike.
