---
status: accepted
status-note: "ratified by chris 2026-11-10 (Ratify, in his review of the pending ADRs)"
ratified-by: chris
ratified-on: "2026-11-10"
owner: chris
kind: design
built: landed
proof: "`crates/ui/tests/window.rs::empty_start_then_sample`, `crates/ui/tests/window.rs::layers_sit_under_the_inspector_on_the_right`, `crates/ui/tests/window.rs::findings_zoom_and_theme`, `crates/ui/tests/viewer.rs::file_names_are_safe`, `crates/app/src/main.rs::gtk_font_names`, `crates/ui/tests/export.rs::the_default_template_and_the_three_presets`, `crates/ui/tests/export.rs::folders_are_remembered_by_document_path_and_capped`, `crates/ui/tests/export_sheets.rs::the_export_menu_opens_one_sheet_with_the_remembered_choices`, `crates/ui/tests/export_sheets.rs::ctrl_e_opens_the_sheet_with_the_last_choices_and_enter_writes`, `crates/ui/tests/welcome.rs::the_welcome_screen_has_its_frame_and_every_tab_says_what_is_in_it`, `crates/ui/tests/welcome.rs::the_file_menu_holds_every_item_and_each_key_does_what_its_item_does`"
supersedes: nothing
amends: nothing
---
# ADR-013 — The window keeps the inspector and layers on the right and the left for tools, follows the system theme without tinting the page, uses the system font, starts empty, and names exports by title, device and size

## Decision

1. **Layout** (napkin §6.1 skeleton): a toolbar on top (Open, Profile, zoom, lint count, Export), the canvas on a
   neutral backdrop, a status bar (cursor in px and mm, zoom, lint count, snap switches). **The right side** holds the
   inspector's tabs (the Page tab when nothing is selected; Arrange, Style, Text, Constraints for an object; Lint)
   with the **layers panel in a resizable part at its foot**. **The left side is kept for a tools bar.**
2. **Theme:** follows the system, with Light, Dark and System under View. The page is never tinted; only the backdrop
   and controls follow the theme.
3. **Controls use the system's interface font** (GTK's setting on Linux, Segoe UI on Windows, the system font on
   macOS).
4. **Start:** an empty backdrop with an Open button. Open by the file dialog, by dropping a file, or as
   `quadrule file.quad`.
5. **Export:** one Export button with "Export this device" and "Export all devices"; files are named
   `{title}_{manta|nomad}_{w}x{h}.png`, using each profile's short name.

## Context

- Rung 2 Q4, Q7, Q8, Q11 (`docs/bbq/2026-11-09-rung-2-viewer.md`). Q7 is **overruled**: the recommendation was to open
  on the sample page; chris chose an empty start.
- After rung 2, chris word for word: look "Approved as is"; file names "Just 'manta' / 'nomad'"; font "I would like
  the system default font for the controls" (overruling the Atkinson proposal) (`docs/STATUS-log.md`).
- Rung 3, chris word for word: "move the layers pane to the bottom of the info pane on the right. I would like the
  left hand side to be used for a toolbar in the future." (`docs/bbq/2026-11-10-rung-3-first-editor.md`, "After the
  build").
- Rung 5 builds that tools bar (`docs/bbq/2026-11-10-rung-5-new-and-insert.md`).

Drivers:
- chris's own judgement on first sight of the window.
- Controls that look native on each system.
- Room on the left for the tools to come.

## Alternatives

### Empty start and the system font (chosen: chris's rulings after trying the window)
- Good: the app opens to nothing until asked, and its controls look like the rest of the desktop.
- Bad: a first-time user sees an empty window; the system font differs by system.

### Open on the sample page (overruled, rung 2 Q7)
- Good: something to look at at once.
- Bad: chris preferred an empty start.

### Atkinson Hyperlegible for the controls (overruled, after rung 2)
- Good: the same face as the bundled document font.
- Bad: chris wanted the system's own font.

## Consequences

- Rung 5's tools bar goes on the left, laid out to take later tools (rung 5 Q12).
- Owed (`docs/STATUS-debt.md`): the macOS font name is unverified; the export writes no PNG text note yet (napkin
  §11.1 has it on by default).

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
- 2026-11-10: ratified by chris (answer "Ratify" to "ADR-013 … Ratify?", in his review of the pending ADRs). Prior header
  values: status `proposed`, status-note `drafted 2026-11-10 from the project's records (backlog sweep, no
  rounds, chris's choice); awaiting ratification`.
- 2026-11-17: amended by chris (rung 15, export), see "Amendment — 2026-11-17, chris" below; ratified by chris the same
  day ("look approved. please ratify adr-013 as well"). Item 5 changes; the consequence "the export writes no PNG text
  note yet" is closed. Prior header value: proof `` `crates/ui/tests/window.rs::empty_start_then_sample`,
  `crates/ui/tests/window.rs::layers_sit_under_the_inspector_on_the_right`, `crates/ui/tests/window.rs::findings_zoom_and_theme`,
  `crates/ui/tests/viewer.rs::file_names_are_safe`, `crates/app/src/main.rs::gtk_font_names` ``.

- 2026-11-18: amendment drafted by the rung 16 lead from chris's rung 16 bbq rulings, see "Amendment — 2026-11-18,
  chris (proposed)" below; **awaiting chris's ratification at the 16a look**. The proof header changed because 16a
  renamed a cited test (Export again is gone) and added the welcome screen's and File menu's tests. Prior header value:
  proof `` `crates/ui/tests/window.rs::empty_start_then_sample`, `crates/ui/tests/window.rs::layers_sit_under_the_inspector_on_the_right`, `crates/ui/tests/window.rs::findings_zoom_and_theme`, `crates/ui/tests/viewer.rs::file_names_are_safe`, `crates/app/src/main.rs::gtk_font_names`, `crates/ui/tests/export.rs::the_default_template_and_the_three_presets`, `crates/ui/tests/export.rs::folders_are_remembered_by_document_path_and_capped`, `crates/ui/tests/export_sheets.rs::the_export_menu_opens_one_sheet_with_the_remembered_choices`, `crates/ui/tests/export_sheets.rs::export_again_writes_with_the_last_choices_or_opens_the_sheet` ``.
- 2026-11-18: the rung 16 amendment ratified by chris at the 16a look, word for word: "Approved. Ratify adr-013." Its
  heading lost "(proposed)" and its status line was "**Status: proposed, awaiting chris's ratification at the rung 16a
  look.**".

## Amendment — 2026-11-17, chris

**Touches Decision item 5 (Export) and Consequences.** Ratified by chris 2026-11-17, word for word: "look approved.
please ratify adr-013 as well". The rulings are the rung 15 bbq's (`docs/bbq/2026-11-17-rung-15-export.md`), cited by
question:

- **Export opens one Export sheet** (Q2): the toolbar's Export holds **Export…** (Ctrl+Shift+E), which opens the sheet
  (devices: this one or all; folder; path; set; the text-note tick), and **Export again** (Ctrl+E), which writes with
  the last choices and no sheet. The two items "Export this device" and "Export all devices" are gone. Both pass
  through the one preflight sheet (rung 14b). Whether these move to a File menu is an open question (chris at rung 15's
  look: "Let's put an item into open questions for a file menu creation").
- **Files are named by one app-wide path template** (Q3, Q9), kept in the app's settings, not in the document. Its
  **default is `{title}-{device}.png`** (chris: "app-wide, change the default {title}-{device}.png"), replacing
  `{title}_{manta|nomad}_{w}x{h}.png`. `/` in the template makes folders; a Set box fills `{set}`; three presets
  (Flat, Folder per device, Device then set). Tokens and safe-name rules are in the record's Defaulted list; `{device}`
  is still each profile's short name.
- **The export folder is remembered** per document path in the app's settings, with the last folder used as the
  global default (Q4).
- **PNGs carry a text note by default** (Q6): Software and Title, PNG's own text keywords; a tick turns it off. The
  frozen goldens stay the no-note pixel tests.

Why: spec §11.2 asks for a configurable naming scheme, per-profile subfolders, a set folder and a remembered folder,
and §11.1 for the text note; chris chose the shorter default name at the bbq.

## Amendment — 2026-11-18, chris

**Ratified by chris 2026-11-18, word for word: "Approved. Ratify adr-013."** (at the rung 16a look). Touches Decision item 4 (Start), item 5 and
the 2026-11-17 amendment's first bullet (Export). The rulings are chris's, from the rung 16 bbq
(`docs/bbq/2026-11-17-rung-16-welcome-and-gallery.md`), cited by question:

- **The window starts on a welcome screen** (Q2, chris: "One screen: actions left, gallery right (Recommended)"),
  replacing item 4's empty backdrop. Down the left: Blank page, Open… and the Recent list; on the right, the starter
  gallery (category tabs, a device filter, search, thumbnails, My templates). It shows at start-up when no file is named,
  after Close, and for New… over an open document. Opening by the file dialog, by dropping a file, or as `quadrule
  file.quad` stays as item 4 had it.
- **A File menu holds the file actions** (Q6, chris: "the move into the file"): New…, Open…, Open Recent, Save, Save
  As…, Save as template…, Export… and Close, leftmost in the top bar. The toolbar's New, Open, Save and Save As buttons
  and its Export menu go.
- **One Export…** (Q9, chris: "One Export…, Enter writes (Recommended)"), replacing the 2026-11-17 amendment's two
  items: File ▸ Export… (Ctrl+E) opens the Export sheet with the last choices filled in and its Export button focused,
  so Enter writes them. Export again and Ctrl+Shift+E are gone. The rest of that amendment (the path template, the
  remembered folder, the text note) stands.

Why: spec §13 asks for a starter gallery and My templates, and chris placed the File menu (his open question from rung
15's look, "Let's put an item into open questions for a file menu creation") in rung 16; asked whether Export again could
be combined with Export, he chose one item that always shows where the files go.
