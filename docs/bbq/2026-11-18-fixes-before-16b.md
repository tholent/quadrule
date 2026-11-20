# bbq record — quadrule: chris's bugs and features before 16b

2026-11-18 · untyped bbq (chris's list after the layers look: "let's talk about a few bugs/features I'd like to see
before 16b starts") · 8 questions · lead: Opus 5.5 at high effort (this session); builders Sonnet 5.5 at high effort
(arc Q2).

## chris's list, word for word

- can't open file when double clicked in macOS
- Quick Look enabled for files
- zoom too jumpy
- music staff — grand staff spacing -> staff every… needs to make that relative to entire grand staff
- remove status bar snapping guide
- make the menu fit macOS/windows/linux
- icon based buttons for layers

## Grounding (what the lead found)

- **Double-click on macOS.** `packaging/packager.json` already associates `.quad` with the app (`fileAssociations`).
  The app reads a file only from its command line (`crates/app/src/main.rs`, `parse_args`); macOS hands a
  double-clicked file over by an Apple Event (`application:openURLs:`), which eframe/winit 0.37 does not pass on, so the
  app opens on the welcome screen. Needs a macOS-only hook; it cannot be tried on the Pi.
- **Quick Look.** A `.quad` already carries a 480 px preview of its main device in `previews/` (rung 15). Quick Look
  needs a preview extension (`.appex`) built with Apple's tools and embedded in the bundle, which cargo-packager does
  not do today; unsigned extensions are a risk.
- **Zoom.** `ZOOM_STEPS` (`crates/ui/src/viewer.rs:172`) are powers of two from 1/8 to 16 (rung 2 bbq, Q10); every frame
  with a zoom delta takes a whole step (`crates/ui/src/view.rs`, around line 2484), so a pinch jumps several steps.
- **Grand staff.** `Pattern::MusicStaff { gap_mm, pitch_units, grand }` (`crates/model/src/lib.rs`); `music_staff`
  (`crates/layout/src/pattern.rs`) starts a grand staff every `pitch_units` rows, top to top, its bass staff one rhythm
  unit below the treble, so a small count overlaps them.
- **Status bar.** Seven toggles: Pixel grid, Rhythm, Margins and safe zones, Point snap, Smart guides, Equal spacing,
  Rhythm overlay (`app-snap-*`, `app-rhythm-overlay` in `crates/i18n/locales/en-US/app.ftl`).
- **Menus.** A File menu button in the toolbar (rung 16a, ADR-013 as amended 2026-11-18); nothing native.
- **Layer buttons.** Seven text buttons that wrap to two lines (`v0.16.2`); the tools bar draws its own icons
  (`crates/ui/src/tools.rs`, `paint_icon`).

## Budget

No share set. chris lifted this week's hard limits (rung 16 record, Amendments) and, at 79 %: "I have a free reset this
week, so I am grinding this out" (layers record, Amendments). The lead reports readings.

## Rules

- **Q1 Zoom.** chris: "Smooth, snaps to 1:1 (Recommended)". Pinch and Ctrl+scroll zoom smoothly, following the
  gesture, anchored at the pointer; the zoom settles on 1:1 and on Fit when it passes close to them. The + and −
  buttons (and their keys) keep clean steps.
- **Q2 Grand staff.** chris: "Space between grand staves (Recommended)". With Grand staff on, the count is the space
  from the bottom of one grand staff (its bass staff) to the top of the next, in rhythm units, so grand staves never
  overlap and changing the line gap keeps the spacing. A single staff keeps "Staff every", top to top.
- **Q3 Status bar.** chris: "Move all into a View menu (Recommended)". The status bar keeps zoom and the warnings
  count; the seven toggles move to View ▸ Snapping (they come with the menus, batch 2).
- **Q4 Order.** chris: "Three batches (Recommended)". **Batch 1, `v0.16.3`:** zoom, grand staff, layer icons (all
  testable on the Pi). **Batch 2, `v0.16.4`:** menus for each system, the status bar, the macOS double-click (needs
  chris's Mac at the look). **Batch 3, `v0.16.5`:** Quick Look, its own job. The starters (16b) become `v0.16.6`.
- **Q5 Layer icons.** chris: "Drawn like the tools bar (Recommended)". Simple line icons drawn as the tools bar draws
  its own, one row, each with its name on hover; no icon font.
- **Q6 Windows and Linux menus.** chris: "A menu bar along the top (Recommended)". File, Edit, View, Help across the top
  of the window, above the toolbar. Edit: Undo, Redo, Cut, Copy, Paste, Duplicate, Select all; View: zoom, the device
  strip, Snapping.
- **Q7 macOS menus.** chris: "The real Mac menu bar (Recommended)". The menus go to the top of the screen: a Quadrule
  menu (About, Settings…, Hide, Quit), then File, Edit, View, Window, Help, with ⌘ keys; no in-window menu on the Mac.
- **Q8 Anything else.** chris: "Nothing more (Recommended)".

## Owed

- Quick Look's scope (Mac only, or Windows and Linux thumbnails too) and how it is signed: asked at batch 3's start.

## Defaulted (the lead's calls; each can be vetoed)

- **Zoom range and the pixel grid** stay as ruled (1/8 to 16; the pixel grid from 16×, rung 2 bbq Q10 as amended).
  "Close to" 1:1 or Fit is within about 6 % of it. Ctrl+scroll on a mouse wheel moves a fixed fraction per notch, not
  a whole step. Speed: zooming on the Pi, release build, both canvases, 90th percentile under 50 ms (ADR-009).
  *veto:* none expected.
- **Grand staff and old files.** The new meaning gets its own optional field (the space between grand staves), written
  only when Grand staff is on; a file without it keeps top-to-top, so every existing page exports the same bytes and
  the 32 golden lines and fixtures `v0.4/`–`v0.16/` stay frozen. A grand staff turned on in this version starts with the
  space that matches today's default look. File format **0.18.0** with a `v0.17/` fixture. The field's words are
  proposals for chris's look. *veto:* convert old files to the new meaning.
- **Layer icons.** Drawn in `crates/ui/src/` beside the tools bar's, greyed exactly as the buttons are today, with the
  hover notes the buttons have now; the row fits on one line on the Pi's 1440 × 720 screen. *veto:* none expected.
- **ADR-013** gets a dated amendment for batch 2 (menu bar, the Mac's menu, the status bar), drafted with `makeadr`
  before batch 2's look. *veto:* none expected.
- **No flag** for batch 1's editor changes (ADR-007); the grand staff field changes the format, so it is tested against
  the frozen goldens and a migration test rather than flagged (as rungs 12b, 15 and borders did). *veto:* flag it.

## Amendments

(Rulings made after the bbq, at the look or later, go here word for word, dated.)

## Kickoff

Batch 1: one Sonnet 5.5 high builder on `main`, committing as each step lands: smooth zoom (Q1), the grand staff space
(Q2, format 0.18.0, fixture `v0.17/`), the layer icons (Q5); tests that print one structured line; zoom speed on the
Pi; screenshots. Then chris's look, `v0.16.3`. Batch 2 and batch 3 follow, each with its own look and release.

## Amendments

- **2026-11-20, the batch 1 look:** chris was given a checklist (smooth zoom, the grand staff space, the layer icons)
  and answered, word for word: "approved for v0.16.3". Approved with no changes, so the builder's judgement calls stand
  (the icon drawings, their spacing, the hover order). Shipped as `v0.16.3` (`docs/arcs/v1/release-notes/v0.16.3.md`).
