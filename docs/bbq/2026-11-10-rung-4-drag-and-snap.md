# bbq record — quadrule rung 4: dragging and snapping

2026-11-10 · untyped bbq · 12 questions · lead: Sonnet 5.5, high effort (tier-select, row "spec'd building").

Grounding: `STATUS.md` (Next: "Rung 4, dragging and snapping, is chris's to plan"), `docs/STATUS-log.md` (last
entry, "rung 3 built"), `docs/STATUS-debt.md`, `docs/bbq/2026-11-10-rung-3-first-editor.md`,
`napkins/quadrule-spec.md` §5.2, §6.2, §6.3, §6.5, `crates/model/src/lib.rs` (`Frame`, `Rhythm`, `ObjectKind`),
`crates/format/Cargo.toml`, tier-select's `rosters/claude.md` (last checked 2026-10-16) and `leads.md`.

Measured at the start of this session (2026-11-10, on the Pi): clean tree at HEAD `6d43de6`;
`CARGO_BUILD_JOBS=2 cargo build --workspace` rebuilt `quadrule-xtask` and `quadrule-app`, then `Finished`, so the
build on disk now matches HEAD (it was behind before).

The Label gray question is not asked again: chris left it open until v1.1 (rung 3 record, "After the build").

## Rules

- **Q1 Count.** 12.
- **Q2 Lead.** Chosen by tier-select (Q4 settles it).
- **Q3 Shape.** A plain bbq, not a stew.
- **Q4 Lead.** Sonnet 5.5 at high effort.
- **Q5 Snapping.** All six helpers of napkin §6.2: pixel grid, rhythm, point snap, smart guides, equal spacing,
  margins and safe zones. (Recommended was five, with equal spacing later. chris chose more, so this is not marked
  overruled.)
- **Q6 Transform.** Drag to move; resize handles (Shift keeps proportions, Alt/Option from the centre); handles on
  a line's two ends; marquee select; drag-to-reorder in the layers panel; Alt-drag duplicate; copy and paste,
  between documents too. No rotation: the file has no rotation field.
- **Q7 Flag — overruled.** Recommended was a Cargo feature `drag`, off by default. chris chose **no flag, as
  rung 3**: dragging and snapping are on in the app build. This waives AGENTS.md's "land a new feature switched
  off behind a flag" for rung 4 only. chris still sees the look before it is called done.
- **Q8 Clipboard.** The system clipboard, as text: the copied objects as JSON with a quadrule marker, through
  egui's clipboard. No new crate.
- **Q9 Rhythm.** Both: a rhythm overlay (faint lines over the canvas, major lines stronger, never exported,
  switched on and off from the status bar) and rhythm subdivisions (2–10 per unit, napkin §6.3), added to the file
  as one optional field.
- **Q10 Snap space.** Snapping uses the device on screen: its pixel grid and its resolved rhythm. This matches
  rung 3's nudge and the `u` rule chris approved. The stored design stays in mm (rung 3 Defaulted, "Edits change
  the shared design").
- **Q11 Split.** Two parts, with chris's look check after each:
  - **4a:** drag to move, resize and line-end handles, marquee, drag-to-reorder layers, Alt-drag duplicate; pixel
    grid, rhythm (with subdivisions) and margins/safe-zone snapping; the rhythm overlay; the snap toggles.
  - **4b:** point snap, smart guides, equal spacing, copy and paste.
- **Q12 Anything else to protect or avoid?** (1) Keep the export goldens fixed: rung 1's goldens
  (`crates/render/tests/golden/SHA256SUMS`) and the stock-grid test pass unchanged; if a change would alter
  export bytes, stop and ask chris. (2) Dragging stays smooth on the Pi, measured in a release build; if it does
  not, the build stops and says so (the number is under Defaulted).

## Owed

(empty: Q8 to Q11 answered what Q5 and Q6 opened; the rest moved to Defaulted)

## Defaulted

- **Size, grown and said out loud:** Q5 (all six) and Q6 (plus paste) make rung 4 about 4–6 sessions on the Pi:
  4a about 2–3, 4b about 2–3. veto: move equal spacing or paste to a later rung.
- **Pi-smooth, the number (Q12):** while dragging an object on the sample page at Fit zoom in a release build
  (`cargo build --release -p quadrule-app`), each frame is under 50 ms (20 frames a second) on the export-pixels
  canvas and on the chip canvas, timed by the canvas's own frame clock
  (`crates/ui/src/canvas.rs`, `last_ms` and `cpu_ms`). Rung 3 measured the chip canvas at 40 ms a
  frame, so 50 ms leaves a little room. If either canvas misses, stop and tell chris with the numbers before
  speeding anything up. veto: the 50 ms line.
- **One drag, one undo step:** a drag (move, resize, marquee-move, Alt-drag duplicate, layer reorder) is one
  command, applied when the mouse is let go; the canvas shows it live while dragging. Escape during a drag cancels
  it. Locked objects and locked layers do not drag (rung 3's rule). veto: none expected.
- **Snap distance:** a snap catches within 6 screen pixels (it does not grow with zoom). When two targets are in
  reach, the nearest wins, and on a tie: rhythm, then margins and safe zones, then point snap, then smart guides,
  then equal spacing (napkin §6.2: rhythm "takes priority over smart guides"). Pixel grid snap then puts the result
  on whole device pixels (odd-width strokes on pixel centres). veto: the 6 px, or the order.
- **What snaps:** on a move, the selection's edges and centre; on a resize, the dragged edge or corner, and the
  size snaps to whole rhythm units or subdivisions; on a line end, the end point. A multiple selection moves and
  snaps as one box. veto: none expected.
- **Toggles:** each helper has an on/off toggle in the status bar and in the Page tab (napkin §6.2). All start on,
  as the napkin's table says. Holding Ctrl (Cmd on macOS) while dragging turns snapping off for that drag. The
  toggles are app settings, not saved in the `.quad` file. veto: save them per document.
- **Guides on screen:** while dragging, the snap that caught is drawn as a thin accent line (smart guides, margin
  and rhythm lines) or a small marker (point snap, equal-spacing gaps with their size). Overlays never export.
  veto: none expected.
- **Subdivisions in the file (Q9):** `Rhythm` gains `subdivisions: Option<u8>` (2–10, none by default), read by
  the Page tab and by Shift+arrow (napkin §6.2: "or one subdivision, if subdivisions are on"). Adding the field
  moves the file format from 0.3 to 0.4 with a migration in `crates/format/src/migrate.rs`; a 0.3 file opens
  unchanged, and unknown fields still round trip. "Major every N" is not added (pattern regions do not use it
  yet). veto: none expected.
- **Rhythm overlay (Q9):** drawn for the device on screen, from the rhythm origin, at the resolved unit (and
  faintly at subdivisions). Off by default, switched from the status bar. veto: on by default.
- **Equal spacing (4b):** shows and snaps to equal gaps between the dragged object and two or more others in the
  same row or column, napkin §6.2. veto: none expected.
- **Copy and paste (Q8, 4b):** Ctrl+C, Ctrl+X, Ctrl+V. The clipboard text is JSON with a top-level marker
  (`"quadrule-clipboard": 1`) and the objects; a paste reads only text with that marker, and gives the pasted
  objects new ids. A paste into the same document puts the copy one rhythm unit down and right of the original;
  into another document, at the same place. Paste goes into the layer of the current selection, or the top
  unlocked layer. Text outlines are made again for each profile, as Save does. veto: the offset.
- **Marquee:** a drag that starts on empty canvas draws a box; objects whose shape it touches are selected; Shift
  adds to the selection. veto: "wholly inside" instead of "touches".
- **Layers drag-to-reorder:** drag an object row to a new place, inside its layer or into another; drag a layer
  row among layers. Move up / Move down stay. veto: none expected.
- **Lines and resize:** a line's frame can have zero width or height (`Frame` docs); a line shows two end handles,
  not a box. Resizing a frame past zero flips it. veto: none expected.
- **Words and access:** every new string through Fluent (`crates/i18n/`); every new control named for screen
  readers. veto: none expected.
- **Tests:** headless, one `CHECK` line each (and print each starting with a newline, paying the debt item about
  `grep '^CHECK'`): the snap engine for each helper (target, distance, priority, Ctrl off), resize with Shift and
  Alt, a drag as one undo step, marquee, reorder, Alt-drag duplicate, paste round trip (copy → paste → same
  objects, new ids), the 0.3 → 0.4 migration, export bytes unchanged after a drag and undo. Widgets with
  `egui_kittest` where it needs no graphics chip. veto: none expected.
- **Where the code goes:** the snap engine in `layout` (headless: it turns mm into device pixels and knows the
  rhythm); drag commands in `model::edit`; the pointer handling and overlays in `ui`. veto: none expected.
- **Landing:** commits straight to `main` (chris's amendment of 2026-11-09 in `AGENTS.md`); the bbq skill's default
  of a branch and PR is overruled by the project rule. CI checked on each push. veto: a branch for 4b.
- **spikes/:** untouched (rung 1 Q8 stands).
- **Checks:** `cargo test --workspace --no-fail-fast -- --nocapture`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo fmt --all --check`; build with `CARGO_BUILD_JOBS=2`; count checks with
  `grep -o 'CHECK crate=[^ ]* name=[^ ]*' | wc -l`.

## Kickoff

Run on **Sonnet 5.5 at high effort**; on any other model or effort, stop and say so before doing anything (read
the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, `docs/decisions/` ADR-001 to ADR-003, the rung 3
record `docs/bbq/2026-11-10-rung-3-first-editor.md` and this record. Prove the build matches HEAD. Build **rung 4a**
on `main` in this order: the `subdivisions` field and the 0.3 → 0.4 migration; the snap engine in `layout` (pixel
grid, rhythm, margins and safe zones) with its tests; drag commands in `model::edit`; drag to move, resize and
line-end handles, Ctrl to turn snapping off, Escape to cancel; marquee; Alt-drag duplicate; drag-to-reorder in the
layers panel; the rhythm overlay and the snap toggles. Keep the export goldens unchanged. Measure dragging on the
Pi in a release build on both canvases (under 50 ms a frame). Show chris the window and stop for the 4a look
before starting 4b (point snap, smart guides, equal spacing, copy and paste). Update `STATUS.md`,
`docs/STATUS-log.md` and `docs/STATUS-debt.md` at the end of each session.

## After 4a (2026-11-10)

Added after the 4a build; the rules above are unchanged. chris's answer, word for word, after trying the window:
- "I approve the work so far, with two changes to the safe area display. I would like to remove the labels and
  include the right hand side safe area by default."
  Built: the safe-zone overlay draws no labels; it shows the left, right and top toolbar zones (and the footers),
  and dragged objects snap to the same zones. The linter still checks only the left and top toolbars (the choice of
  2026-11-09 in rung 2's Defaulted "Lint toolbars"); chris's words named the display, so lint was not changed.
- Not answered, so still open: the one 60.1 ms export-canvas drag frame (judge by the worst frame, a percentile or
  the mean?). Ask it again at the next session start.

chris's next answer, word for word (2026-11-10, on the 60.1 ms frame and the safe zones):
- "let's use a percentile. the safe area linting should look at all safe areas, including the right one. Also, when
  two safe areas overlap, don't draw the borders that occur inside the combined area."
  Built: the 50 ms line in Q12 is judged by the 95th percentile of the drag frames after the first (the 95 was not
  named; chosen for chris to veto). This replaces "each frame is under 50 ms" in Q12 and Defaulted "Pi-smooth, the
  number". The linter, the overlay and the snap now all use every toolbar position (this replaces the left-and-top
  lint choice of 2026-11-09 and the "lint unchanged" note above). Overlapping or touching zones draw one outline round
  their combined area.

## After 4b (2026-11-10)

Added after the 4b build; the rules above are unchanged. chris's go-ahead, word for word: "please proceed with rung 4b".
Built as Defaulted says (6 px reach; tie order rhythm, margins and safe zones, point snap, smart guides, equal spacing;
all switches on; Ctrl turns them off; guides and gaps never exported; clipboard text with the `"quadrule-clipboard": 1`
marker; paste one rhythm unit down and right in the same document, in place in another). Two small additions, either can be
vetoed: each further paste goes one more unit down and right; a cut leaves locked objects where they are. Equal spacing is
for a move only. 

chris's next answer, word for word: "let rhythm win within half a pixel". Built: a rhythm catch beats any other kind unless
that kind is more than half a device pixel nearer. This refines Defaulted "Snap distance" ("the nearest wins, and on a
tie ..."): the tie band for rhythm is half a pixel, not a millionth.

