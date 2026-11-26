# ADR question record: the gray scale built on the pen grays (ADR-027)

2026-11-26 · makeadr, full route, goforit (chris chose "goforit" for the question count) · chris and Claude (Opus 5.5)

No question rounds: every answer the ADR needs is chris's, in the talk record
`docs/proposals/talk-2026-11-26-label-gray-and-pen-grays.md` (D1–D13 and its two amendments). Each is cited in the ADR.
Last question: "Make the ADR now, and also a prompt for the next agent to get things started?" chris: "Just make the
ADR (Recommended)".

## Premises

Each checked with one search or file read this session.

| premise | checked in |
|---|---|
| Both tablets offer the 16 even steps 0, 17 … 255 | `crates/profiles/data/supernote.a6x2.ron`, `supernote.a5x2.ron` (`gray_levels`) |
| The pens write 1, 157, 201, 254 on both tablets (the Manta's assumed) | the same files (`ink_values`); `docs/STATUS-log.md`, pen shades |
| Every gray on the grays sheet showed clean; each in-between patch matched a neighbour | `docs/device-measurements/supernote.a6x2/calibration.md`, "Grays sheet — 2026-11-12" |
| Supernote's stock grid draws at gray 1 on a page of 255 | `docs/device-measurements/README.md` |
| Role grays must be clean levels, by test | `crates/profiles/tests/profiles.rs::role_grays_are_clean_levels` |
| Role values come from calibration, never code | `docs/decisions/ADR-012-roles-profiles-safe-zones.md`, Decision item 2 |
| The Label gray question is an open item of ADR-012 and the debt list | ADR-012 Consequences; `docs/STATUS-debt.md`, "Open until v1.1" |
| L04 is not built; the linter names it as waiting for v1.1 | `crates/lint/src/lib.rs` (module comment, `SKIPPED_ON_PAPER`) |
| A page carries copies only of profiles that are not unedited built-ins | `docs/decisions/ADR-016-user-profiles.md`, Decision item 4 |
| A change to export bytes or the file format lands behind a flag | `docs/decisions/ADR-007-look-check-gates-editor-rungs.md`, amendment of 2026-11-10 |
| The newest file format is 0.21.0 | `crates/format/src/migrate.rs` (`"0.20.0", "0.21.0"`) |
| Rung 17 keeps the golden lines and fixtures frozen | `docs/bbq/2026-11-24-rung-17-polish.md`, Q11 |
| ADR-027 is free | `git log --all -S'ADR-027' --oneline` (no output) |

## Defaulted (the lead's calls; each can be vetoed)

| call | who does it this way | veto |
|---|---|---|
| The flag is an app switch, `--pen-grays`, with the same switch on the `cargo xtask` commands that export or lint, as ADR-026's `--basic-section` was. The name is a proposal (ADR-027, ask 1). | ADR-026 | `veto: a different name or a build feature` |
| The format step is 0.21.0 → 0.22.0 and lands with the flag; with the flag off, files are saved at 0.21.0 as today. | ADR-001's migration chain; ADR-023 and ADR-026 added steps the same way | `veto: none expected` |
| User-made profiles and edited Manta or Nomad overrides keep their own grays; Reset to built-in brings the new scale. They are the user's data (ADR-016). | ADR-016 | `veto: migrate user profiles too` |
| The golden lines change once, when the flag comes out at chris's look, as ADR-018's one-time golden change did; until then the flag-on pages are checked by their own tests. | ADR-018 | `veto: none expected` |
| The grays sheet gains the new steps and the points halfway between them, as the 2026-11-12 sheet did with the old steps; the lines and text sheets are not re-made. | `crates/xtask/src/calib.rs`; the 2026-11-12 sheet | `veto: re-make every sheet` |
| The pen names follow the order of `ink_values`: black, dark grey, light grey, white. The white pen gets no name, since 254 is not a step (D6). | the talk ("157 · dark grey pen", D11) | `veto: other words` |
