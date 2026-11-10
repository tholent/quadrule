---
status: proposed
status-note: "drafted 2026-11-10 from the project's records (backlog sweep, no rounds, chris's choice); awaiting ratification"
owner: chris
kind: design
built: landed
proof: "`crates/ui/tests/window.rs::empty_start_then_sample`, `crates/ui/tests/window.rs::layers_sit_under_the_inspector_on_the_right`, `crates/ui/tests/window.rs::findings_zoom_and_theme`, `crates/ui/tests/viewer.rs::file_names_are_safe`, `crates/app/src/main.rs::gtk_font_names`"
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
