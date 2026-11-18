# bbq record — quadrule rung 16: welcome screen and the starter gallery

2026-11-17 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-16-PROMPT-2026-11-17.md`) · 9 questions (the
arc's "about 8"; Q9 was owed by chris's question in his Q6 answer) · lead: Opus 5.5 at high effort; builder Sonnet 5.5
at high effort (arc Q2; not re-asked).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 16 row, chris's queue), the rung 16 prompt,
`docs/decisions/INDEX.md`, ADR-013 (rule 4 "Start: an empty backdrop with an Open button"; its rung 15 amendment, lines
84–88: Export… Ctrl+Shift+E and Export again Ctrl+E), `napkins/quadrule-spec.md` §13 and §16, `crates/app/src/main.rs`
(start-up, `--look`, the recovery folder in `ProjectDirs::data_dir()` at 158–161), `crates/ui/src/viewer.rs`
(`open_new` at 546, `open_path` at 557), `crates/ui/src/settings.rs` (`settings.ron`), `crates/layout/src/newpage.rs`
(`NewPage`: title, device, rhythm, orientation), `crates/layout/src/device.rs` (`make_primary` at 266),
`crates/model/src/lib.rs` (`Metadata` has title, description and tags), `crates/i18n/locales/en-US/app.ftl` (the
unsaved-changes sheet, rung 3), `docs/STATUS-debt.md` ("Rung 15, owed", "Rung 15, lead's close"),
`docs/budget-calibration.md`.

Measured at the start of this session (2026-11-18 01:44 UTC on the Pi): `echo "$CLAUDE_EFFORT"` → `high`;
`make build-proof` → `BUILD head=0b18202 dirty=no rebuilt=0`.

## Asked again at session start (AGENTS.md)

- **Starters** (arc Q11, Q13; none is in the repository): chris, **"None yet, will make them"**.
- **A File menu** (rung 15 look): chris, **"Rung 16 (Recommended)"**.
- The Label gray: still parked (v1.1); not re-asked.

## Budget
Share: 12 points (1 point = 1 % of the weekly limit), chris: "12 points, keep 4 (Recommended)"; reserve 4 kept for
chris. Alone on the meter (Defaulted below).
Room: 16 points (the hard stop, 75 %, less the reading; reset about 92 h away), less the reserve, gives 12.
Written when the weekly meter (reset Saturday 2026-11-21, 5 pm EST) read 59 % (2026-11-18 01:44 UTC, given by chris,
not measured; logged with `spend-ledger.py reading 59 --meter weekly`). That reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. Read the meter before each batch of dispatches; log a reading only at batch ends and when a stop line is within
5 points. Stop starting new work at the first of: your ledger reaching 12 points (alone on the meter: the reading minus
your start reading); the meter reaching 71 %; the hard stop (75 % beyond 48 h, 80 % at 24–48 h, i.e. from Thursday
2026-11-19 5 pm EST, 85 % at 12–24 h, 93 % under 12 h). K = 12.2M tokens/point fitted over two pairs in
`docs/budget-calibration.md`, not yet trusted, so trust the meter's movement. If the 5-hour window fills, wait for its
reset and note the wait. In-flight work finishes. Hand off at ~500k of your context. Before handoff, write your row in
the calibration log.

## Rules

- **Q1 Split.** chris: "16a now, 16b later; two tags (Recommended)". **16a**: the welcome screen, the gallery, My
  templates, recent files and the File menu, built now inside the 12 points, looked at, then **`v0.16.0`**. **16b**: the
  starters and their goldens on both devices (arc Q11, Q13), after chris's five designs, then **`v0.16.1`**.
- **Q2 Layout.** chris: "One screen: actions left, gallery right (Recommended)". The welcome screen replaces ADR-013
  rule 4's empty start. Down the left: **Blank page**, **Open…**, and the **Recent** list. On the right: the
  category tabs (All, Paper, Notes, Planning, Bullet journal, Creative, My templates), a **Device** filter, a
  **Search** box, and a grid of thumbnails, each with its name under it. File ▸ New… opens this same screen over an open
  document. chris's chosen sketch:

  ```
  +--------------------------------------------------------------+
  | quadrule                                                     |
  +----------------+---------------------------------------------+
  | [+ Blank page] |  Device: [Nomad v]      Search: [_______]   |
  | [ Open...    ] |  All | Paper | Notes | Planning | BuJo |     |
  |                |  Creative | My templates                    |
  | Recent         |  +------+ +------+ +------+ +------+        |
  |  weekly.quad   |  |::::::| |------| |######| |      |        |
  |  dots-5mm.quad |  |::::::| |------| |######| |      |        |
  |  meeting.quad  |  +------+ +------+ +------+ +------+        |
  |  ...           |   Dot grid  Ruled   Square   Blank          |
  |                |                                             |
  +----------------+---------------------------------------------+
  ```

- **Q3 Thumbnails.** chris: "Render live, in the background (Recommended)". Each tile is drawn for the filter's
  device off the interface thread, once, then cached; a `.quad`'s stored `previews/` picture stands in while it draws.
  Works for any device, paper size or user profile; nothing extra goes in the repository.
- **Q4 My templates.** chris: "A templates folder, Save as template… (Recommended)". File ▸ **Save as template…**
  copies the document into the app's data folder (`templates/` beside `recovery/`: `~/.local/share/quadrule/templates`
  on Linux). The gallery's My templates lists that folder; right-click removes one. Copies keep working when the
  original moves or is deleted.
- **Q5 Before the starters.** chris: "all categories, marked "coming soon" I will add the appropriate starters later".
  **Overruled**: the recommendation was to hide empty categories. Every category tab shows from 16a; an empty one says
  its starters are coming. chris adds the starters in 16b.
- **Q6 File menu.** chris: "the move into the file, but can export again be combined with export?" A **File** menu,
  leftmost in the top bar (`crates/ui/src/view.rs` `toolbar`, where View is already a menu): New…, Open…, Open Recent ▸, Save, Save As…, Save as template…, Export…, Close. The toolbar's
  New, Open, Save and Export go into it; the toolbar keeps the tools and view controls. His question became Q9.
- **Q7 The basic section** (spec §13: each starter's spacing, margins and title at the top of the inspector). chris:
  "16b, with the starters (Recommended)".
- **Q8 Anything else.** chris: "Nothing more (Recommended)". The standing rules hold: the 30 golden lines and the
  fixtures `v0.4/`–`v0.15/` frozen (a change that moves any is a stop, arc Q20), 50 ms at the 90th percentile on the Pi
  (ADR-009), the debt paid in code the rung touches, chris's look before it is done (ADR-007).
- **Q9 Export, merged.** Answered first, with evidence (ADR-013, amended 2026-11-17, lines 84–88): Export… opens the
  sheet with the last choices filled in; Export again writes them at once with no sheet. chris: "One Export…, Enter
  writes (Recommended)". **One item, File ▸ Export… (Ctrl+E)**: it opens the sheet with the last choices filled and
  the Export button focused, so Ctrl+E then Enter repeats the last export. Export again goes. This changes ADR-013's
  rung 15 amendment, so ADR-013 gets a new dated amendment (with Q2's start and Q6's File menu), through `makeadr`, for
  chris to ratify at the look.

## Owed

Empty. Q9 (owed by the Q6 answer) was asked.

## Defaulted (the lead's calls; each can be vetoed)

- **Alone on the meter.** The share line is the reading minus the start reading. *veto:* name the other sessions; the
  share line becomes the ledger.
- **When the welcome screen shows.** At start-up when no file is named, after File ▸ Close (Ctrl+W), and for File ▸
  New… (Ctrl+N) over an open document, where Esc or Cancel returns to the document. A file named on the command line or
  dropped on the window opens straight away, and dropping a `.quad` on the welcome screen opens it. `--look welcome`
  (and `--look gallery-mine`, `--look file-menu`) start on it for screenshots. *veto:* say when it should not show.
- **Blank page** opens rung 5's New dialog (title, device, rhythm, orientation) with the filter's device chosen.
  *veto:* a blank page without the dialog.
- **A starter or template becomes a document as an untitled copy.** No file until Save, title from the starter, created
  and modified now, no unsaved changes until edited (as New). The filter's device becomes the main device with the Page
  tab's Make primary, adding it as a target first if the starter lacks it; if that fails, the copy opens on its own main
  device with a notice saying why. *veto:* keep the starter's own main device.
- **Unsaved changes.** Picking Blank page, a starter, a template, a recent file, Open… or Close while the open document
  has unsaved changes asks rung 3's "Save changes?" first. *veto:* none expected.
- **Recent files** live in `settings.ron`, newest first, at most 10, added at Open, Save and Save As. Each line shows the
  file's name and its folder. A missing file shows greyed with "Not found"; clicking it offers to remove it, and
  right-click has Remove from list. File ▸ Open Recent ▸ shows the same list. No thumbnails in the list (Q2's sketch).
  *veto:* a different count, or thumbnails.
- **The device filter** lists the built-in devices, then the user's profiles, then the paper sizes, and starts on the
  device New last used (rung 5). It hides nothing: starters adapt to every device. A template that does not target the
  chosen device is drawn on its own main device, with that device's name on the tile. *veto:* hide such templates.
- **Search** matches, ignoring case, any part of the title, description, tags (all in `Metadata` already) or category
  name, across every category while the box has text; clearing it returns to the chosen tab. *veto:* search one tab
  only.
- **Where starters live.** `crates/ui/starters/<category>/<slug>.quad`, built into the app; the folder names the
  category, so the file format does not change. 16a ships the loader with no starters; its tests use their own test
  starters under `crates/ui/tests/`. *veto:* another home (a new crate, a data folder).
- **Thumbnails, in detail.** Drawn by the export renderer at the tile's size times the screen's pixel ratio, on one
  background thread fed in the order tiles are on screen, cached for the session by (source, device, file time). The
  interface thread never renders a page, and an idle gallery does no work per frame (the prompt's watch item: "Opening
  the gallery must not render 24 pages on every frame"). A page that fails to draw shows a plain tile with its name.
  *veto:* none expected.
- **Save as template…** writes through the same code as Save (preview included) to `templates/<slug>.quad`; if a
  template of that name exists it asks to replace it. Right-click on a template: **Remove** (deletes the copy after a
  confirm; one-way, so the confirm says so) and **Show in folder**. A template opens as an untitled copy, like a
  starter. *veto:* removal into the system's trash instead of deleting.
- **The File menu, in detail.** New… Ctrl+N · Open… Ctrl+O · Open Recent ▸ · — · Save Ctrl+S · Save As… Ctrl+Shift+S ·
  Save as template… · — · Export… Ctrl+E · — · Close Ctrl+W. Ctrl+Shift+E is freed (Q9). The toolbar's New, Open, Save
  and Save As buttons and its Export menu go; the device picker, Profiles, zoom, the switches and View stay. The empty
  start's "Open sample page" becomes a small link at the foot of the welcome screen's left column. *veto:* any order or
  shortcut, or drop the sample link.
- **No flag** (ADR-007, amendment of 2026-11-10): nothing here changes export bytes or the file format, and Save as
  template writes an ordinary `.quad`. *veto:* flag the welcome screen.
- **Words.** Every new word goes through Fluent; the builder proposes them (the "coming soon" line included) for
  chris's look. *veto:* none expected.
- **The ADR-013 amendment** (Q2 start, Q6 File menu, Q9 one Export…) is drafted with `makeadr` before the look and
  ratified by chris at it. *veto:* ratify it now.
- **Debt.** The Save preview's time on the interface thread ("Rung 15, lead's close") stays with rung 17's large-page
  item; 16a reads previews but does not change how Save writes them. The rung 15 judgement calls about the toolbar's
  Export menu are closed by Q6 and Q9. *veto:* move the Save preview off the interface thread now.
- **Speed.** On the Pi, release build: the welcome screen's frames (90th percentile) while 24 test tiles draw, and while
  idle, under 50 ms; drawing and dragging on the heaviest existing page, both canvases, strip on, still under 50 ms
  (ADR-009). Say before anything crosses it. *veto:* none expected.
- **Goldens.** 16a adds none (no export change); the 30 lines and the fixtures stay frozen.
- **Versions.** `0.16.0` at 16a's approval, `0.16.1` at 16b's (Q1). *veto:* other numbers.

## Amendments

(Rulings made after the bbq, at the look or later, go here word for word, dated.)

- **2026-11-18, budget, after 16a leg 1** (meter 63 %, chris's reading): chris, word for word: "for this week only,
  unset any hard limits". Until the weekly reset (Saturday 2026-11-21, 5 pm EST), the 12-point share line, the 71 %
  backstop and the 75 / 80 / 85 / 93 % hard stops do not stop work; the lead still reads the meter at each leg's end and
  reports it. The reserve of 4 is read as lifted with them (the lead's reading; chris may correct it).
- **2026-11-18, the 16a look** (meter 65 %, chris's reading): chris, word for word: "Approved. Ratify adr-013." 16a is
  approved with no changes, so the builder's twenty judgement calls and proposed words stand as built (`docs/STATUS-log.md`,
  the two 16a builder entries). ADR-013's rung 16 amendment is ratified. Shipped as `v0.16.0`; 16b waits for chris's
  starters (`docs/arcs/v1/rung-16b-PROMPT-2026-11-18.md`).

## Kickoff

Lead: **Opus 5.5 at high effort** (this session). Builder: **Sonnet 5.5 at high effort**, on `main`, committing as each
step lands; on any other model or effort, stop and say so before doing anything. Build **16a** from this record's
Rules and Defaulted list: the welcome screen (Q2), the gallery with its device filter, search, live thumbnails (Q3) and
every category tab with "coming soon" (Q5), My templates and Save as template… (Q4), recent files, the File menu (Q6)
and the one Export… (Q9). Point `XDG_CONFIG_HOME` and `XDG_DATA_HOME` at a scratch folder whenever the app runs. Keep
the 30 golden lines and the fixtures frozen; stop and ask chris if any would change. Measure as the Defaulted "Speed"
line says. Tests read one structured line each (ADR-006). Then the lead drafts the ADR-013 amendment with `makeadr`,
shows chris the window (`--look welcome` and friends) and stops for his look. After approval: `0.16.0`, release notes
in `docs/arcs/v1/release-notes/`, the tag, the board's re-plan, `STATUS.md`, the log, the debt and the calibration row.
16b waits for chris's five starters.
