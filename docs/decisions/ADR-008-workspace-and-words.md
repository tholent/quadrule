---
status: proposed
status-note: "drafted 2026-11-10 from the project's records (backlog sweep, no rounds, chris's choice); awaiting ratification"
owner: chris
kind: architecture
built: landed
proof: "`Cargo.toml`, `crates/i18n/tests/i18n.rs::english_loads_and_formats_every_message`, `crates/ui/tests/window.rs::every_message_id_exists`, `crates/lint/tests/lint.rs::every_message_id_exists`"
supersedes: nothing
amends: nothing
---
# ADR-008 — The code is a Cargo workspace whose crates arrive with the rung that needs them, the app is an opt-in build, and every word on screen goes through Fluent

## Decision

1. **Stack:** Rust, egui for the window, wgpu and vello for the graphics-chip canvas, a CPU renderer for export
   (napkin §15, kept as written by the spikes record Q3; the export renderer is ADR-003).
2. **Crates arrive with the rung that uses them.** Rung 1 made `model`, `format`, `profiles`, `layout`, `text`,
   `render`; `ui`, `app`, `lint`, `i18n` came later; `imaging` waits for v1.1 (napkin §15.1).
3. **The app is an opt-in build.** `Cargo.toml`'s `default-members` leave `quadrule-app` out, so plain `cargo build`
   and `cargo test` stay quick on the Pi; `cargo run -p quadrule-app` builds it; CI builds and tests it everywhere.
4. **Where code goes:** headless maths in the lower crates (snapping and units in `layout`, edits as commands in
   `model::edit`, the clipboard format in `model::clipboard`); pointer handling, panels and overlays in `ui`; the
   window and settings in `app`.
5. **Words on screen:** every string, lint messages included, goes through Project Fluent in the `i18n` crate,
   English only for now (`crates/i18n/locales/en-US/`). Every control is named for screen readers.
6. **`spikes/` is never edited.** Real crates may copy ideas from it.

## Context

- Spikes Q3 and Q6 (`docs/bbq/2026-11-09-quadrule-spikes.md`); rung 1 Q5 and Q8
  (`docs/bbq/2026-11-09-rung-1-headless-slice.md`); rung 2 Q5 and Q9 (`docs/bbq/2026-11-09-rung-2-viewer.md`);
  rung 4 Defaulted "Where the code goes" (`docs/bbq/2026-11-10-rung-4-drag-and-snap.md`).
- The napkin asks for translation infrastructure in v1.0 (§14, §16).

## Consequences

- Each headless crate is testable on the Pi without a window.
- Owed (`docs/STATUS-debt.md`): Fluent's Unicode direction marks are switched off, which is right for English and
  wrong for right-to-left languages; switch them on with the first such translation.

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
