---
status: proposed
status-note: "drafted 2026-11-10 from the project's records (backlog sweep, no rounds, chris's choice); awaiting ratification"
owner: chris
kind: process
built: landed
proof: "`LICENSE`, `crates/xtask/tests/license_headers.rs::every_rust_file_has_the_license_header`"
supersedes: nothing
amends: nothing
---
# ADR-005 — quadrule is public and Apache-2.0, every Rust file carries the licence header (a test enforces it), and its packages are named quadrule-*

## Decision

1. The GitHub repository is **public**.
2. The licence is **Apache-2.0**, copied word for word from apache.org into `LICENSE`.
3. **Every Rust file carries the Apache appendix header.** A test fails on any `.rs` file without it (`target/`
   folders are skipped).
4. Cargo packages keep the **`quadrule-*`** names.
5. The README says quadrule is independent: not affiliated with or endorsed by Ratta or Supernote.

## Context

- chris, 2026-11-09, word for word: "let's make this repo public. copy the license from here:
  https://www.apache.org/licenses/LICENSE-2.0.txt and include apache required headers in our coding files.
  1. keep quadrule-* …" (`docs/STATUS-log.md`).
- The repository was private at first (rung 1 record Q7: "Reversible: chris can make it public any time").
- Before going public the history was checked for secrets; a stray editor swap file was removed by a history
  rewrite on chris's ruling (`docs/STATUS-log.md`, 2026-11-09).
- The affiliation line: `docs/bbq/2026-11-10-readme-and-description.md` Q11.

## Consequences

- Every commit's author name and email are public (`docs/STATUS-log.md`).
- A new Rust file without the header fails the test, so the rule cannot be forgotten.
- GitHub Actions minutes are free for a public repository, which made three-system CI cheap (see ADR-006).

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
