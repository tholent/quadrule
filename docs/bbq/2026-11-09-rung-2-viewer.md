# bbq record — quadrule rung 2: the viewer window

2026-11-09 · untyped bbq · 12 questions · lead: Opus 5.5, high effort.

Grounding: `STATUS.md` (Next: rung 2, scope agreed by chris 2026-11-09), `docs/STATUS-log.md` (last entry),
`docs/STATUS-debt.md`, `napkins/quadrule-spec.md` §6.1, §10, §11, §14, §15, `docs/decisions/ADR-003-export-renderer.md`,
`spikes/render/FINDINGS.md`, `docs/bbq/2026-11-09-rung-1-headless-slice.md`, `AGENTS.md`.

Scope, agreed before this bbq (chris, 2026-11-09, `docs/STATUS-log.md`): a viewer. Open a `.quad` or the slice,
show it as the device will, switch Manta and Nomad, zoom to pixels, show lint findings, export PNGs; no editing.

Measured at the start of this session (2026-11-09, on the Pi, Wayland desktop on `wayland-0`):
- Build on disk matches HEAD `8126b87`: clean tree, `cargo build --workspace` had nothing to rebuild;
  `cargo test --workspace -- --nocapture | grep -c '^CHECK'` → 66, none failing.
- Spike R15 run for the first time (`~/.cargo/quadrule-spike-target/release/examples/canvas`): the window does not
  open. eframe asks the graphics chip for wgpu's default limits; V3D (Vulkan) refuses with
  `LimitsExceeded max_inter_stage_shader_variables requested 16 allowed 15`; with `WGPU_BACKEND=gl`,
  `max_texture_dimension_2d requested 8192 allowed 4096`. The fix is in code: ask for the adapter's own limits.

## Rules

- **Q1 Count.** 12.
- **Q2 Lead.** Opus 5.5 at high effort.
- **Q3 Canvas.** Export pixels only. The viewer shows the vello_cpu export render (the PNG's own pixels) as a
  texture. The graphics-chip vello canvas and its vello_cpu fallback (ruled yes, 2026-11-09) come with the editing
  rung, measured on a desktop graphics card.
- **Q4 Layout.** The napkin's skeleton (§6.1): toolbar (Open, Profile ▾, zoom, lint count, Export), canvas on a
  neutral backdrop, a right panel holding the lint findings, a status bar (cursor in px and mm, zoom, lint count).
  No left sidebar yet. The editor fills these places in later without rearranging.
- **Q5 Words on screen.** Through Project Fluent from day one, in a new `i18n` crate; English only. Lint messages
  included.
- **Q6 Lint.** A list of findings; clicking one outlines its area on the page; a toggle overlays the safe zones.
- **Q7 Opening — overruled.** Recommended was "open on the slice"; chris chose: start on an empty backdrop with an
  Open button. Open by the system file dialog, by dragging a file onto the window, or as `quadrule file.quad`.
- **Q8 Export.** One Export button with a menu: "Export this device" and "Export all devices".
- **Q9 Flag.** A separate, opt-in build: plain `cargo build` / `cargo test` at the workspace root skip the app
  (headless work stays fast on the Pi); `cargo run -p quadrule-app` builds and runs it; CI builds and tests it on
  Linux, macOS and Windows. Until chris approves the look, nothing else ships it.
- **Q10 Zoom.** Fit page; 1:1 (one device pixel per screen pixel); zoom up to 16× with square (nearest-neighbour)
  pixels and a pixel grid from 8×; and "real size", the page at its true size on the tablet.
- **Q11 Theme.** Follow the system, with Light / Dark / System under View. The page is never tinted; only the
  backdrop and controls follow the theme.
- **Q12 Anything else to protect or avoid?** Nothing more.

## Owed

(empty: all three moved to Defaulted)

## Defaulted

- **Size — grown, said out loud:** the agreed viewer was "about 1–2 sessions". Q5 (Fluent, including lint
  messages), Q6 (finding areas), Q7 (file dialog, drag and drop) and Q10 (real size needs a ruler check) add to it:
  now about 2–3 sessions on the Pi. veto: cut real size or the file dialog to a later rung.
- **Owed from Q7 — nothing to open yet:** no `.quad` of the slice exists on disk, so the empty screen offers
  "Open…" and "Open sample page" (the slice, built in memory as rung 1 does). veto: a `cargo xtask` that writes
  `sample.quad` instead, or nothing.
- **Owed from Q10 — real size:** no system reports a screen's size reliably (winit gives a scale factor, not
  millimetres), so "real size" asks once for a ruler check: drag until an on-screen bar matches a bank card's
  width (85.6 mm, ISO/IEC 7810 ID-1) or 100 mm on a ruler. The result is remembered in the app's preferences file.
  Until it is done, "real size" opens the check. veto: drop real size for now.
- **Owed from Q6 — where a finding is:** `quadrule_lint::Finding` has no area (`crates/lint/src/lib.rs:43`). It
  gains `area: Option<PxRect>` in device pixels for the profile it was found on (the object's laid-out bounds, or
  the safe zone it hits). A finding with no area is listed but not outlined. veto: highlight only by object.
- **Lint messages and Fluent (from Q5):** lint findings carry a message id and arguments; the English text moves to
  `crates/i18n/locales/en-US/*.ftl`, and `cargo xtask lint` prints through `i18n` too, so English has one source.
  The `i18n` crate is headless (no UI dependencies), like `lint`. veto: lint keeps its own English text.
- **Lint toolbars:** the viewer lints with the toolbar positions in use, docked left and docked top (chris's ruling,
  `docs/STATUS-log.md`, the same default as `cargo xtask lint`). veto: all positions.
- **Crates:** `app` (binary `quadrule`: eframe setup, window, menus, preferences), `ui` (egui widgets: canvas,
  lint panel, dialogs, the viewer's state) and `i18n`, as the napkin's §15.1 names them; rung 1 Q5 said they arrive
  with the rung that uses them. Workspace `default-members` leaves out `app` and `ui` (Q9); `i18n` stays in the
  default build because `lint` and `xtask` use it. veto: one `app` crate holding the widgets too.
- **Graphics backend:** eframe on wgpu, as the napkin says (§15.2), even though Q3 needs no GPU drawing: the later
  vello canvas needs wgpu, and the limits fix (above) gets proven now, closing spike R15. Device limits come from the
  adapter, not wgpu's defaults. veto: eframe's OpenGL backend (glow) for rung 2.
- **Profiles switch:** Manta and Nomad, portrait only (safe zones are measured for portrait only,
  `docs/device-measurements/README.md`). Switching re-lays-out and re-renders with vello_cpu (about 13 ms a Manta
  page on the Pi, spike R10). veto: none expected.
- **File dialog:** `rfd` (napkin §15.2). On Linux it talks to the desktop's file-chooser service (xdg-desktop-portal,
  pure Rust, no system packages); if the Pi's desktop has none, the lead says so and falls back to typing a path.
  veto: GTK dialog (needs GTK development packages).
- **Export:** names from the napkin's default scheme, `{title}_{profile}_{width}x{height}.png`, slugified to ASCII
  letters, digits, hyphens and underscores (§11.2, §11.3); the PNG bytes come from the same code as rung 1's export
  (ADR-003), so they match the goldens. After writing, a short note says where the files went and that they go in
  the device's MyStyle folder (§11.3). The last folder is remembered for the session. veto: remember it across runs.
- **UI font — taste, a proposal:** the controls use the bundled Atkinson Hyperlegible Next (already in the repo
  with its licence) instead of egui's default font. veto: egui's default font.
- **Tests:** the viewer's logic (open, profile switch, zoom maths, real-size maths, export names, finding areas)
  lives in plain Rust in `ui`, tested headless with one `CHECK` line each (AGENTS.md); widgets tested headless with
  `egui_kittest` where it needs no GPU. No screenshot goldens of the window (GPU drivers differ, ADR-003). veto: add
  window screenshots.
- **The window on the display (spike R15):** the lead runs the app on the Pi's display and shows chris; R15 is paid
  when it opens. Emulator before hardware (AGENTS.md): the viewer is the emulator here; exported PNGs on a real Manta
  and Nomad stay chris's step, as in rung 1.
- **Landing:** commits straight to `main` (chris's amendment of 2026-11-09 in `AGENTS.md`); the bbq skill's default
  of a branch and PR is overruled by the project rule. CI checked on each push. veto: one branch and PR for rung 2.
- **spikes/:** untouched (rung 1 Q8 stands): the app may copy ideas from `spikes/render` but never edits it.
- **Checks:** `cargo test --workspace` (the default members), `cargo test -p quadrule-app -p quadrule-ui`,
  `cargo clippy --workspace --all-targets -- -D warnings` (all members), `cargo fmt --check`; build with
  `CARGO_BUILD_JOBS=2` on the Pi.

## Kickoff

Run on **Opus 5.5 at high effort**; on any other model or effort, stop and say so before doing anything. Read
`AGENTS.md`, `docs/decisions/` ADR-001 to ADR-003 and this record. Build rung 2 on `main` in this order: `i18n`
(and lint messages through it), finding areas in `lint`, `ui` (viewer state, then widgets), `app`; then the
window on the Pi's display (R15), real size, file dialog and drag-and-drop, export menu. Keep `cargo test` at the
root free of the app; add the app to CI on three OSes. Show chris the window before calling the look done. Update
`STATUS.md`, `docs/STATUS-log.md` and `docs/STATUS-debt.md` at the end.

## After the build (2026-11-09)

Added after the rounds; the rules above are unchanged. chris's answers, word for word:
- The look: "Approved as is".
- Export file names, `{profile}` token: "Just 'manta' / 'nomad'". Built as a `short_name` per profile
  (`crates/profiles/data/*.ron`). Replaces the Defaulted name `supernote-manta`.
- Controls' font, Defaulted "UI font" (Atkinson proposed) — overruled: "I would like the system default font for the
  controls". Built as the desktop's interface font (on this Pi, GTK's `Nunito Sans Light 12`), egui's own font where
  none is found.

