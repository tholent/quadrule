---
status: proposed
status-note: "drafted 2026-11-10 from the project's records (backlog sweep, no rounds, chris's choice); awaiting ratification"
owner: chris
kind: architecture
built: landed
proof: "`crates/model/tests/edit.rs::every_command_undoes_and_redoes`, `crates/model/tests/edit.rs::undo_depth_is_unlimited`, `crates/model/tests/edit.rs::a_drag_is_one_step_and_locks_refuse_it`, `crates/ui/tests/editor.rs::autosave_and_recovery`, `crates/ui/tests/editor.rs::nudge_moves_by_a_device_pixel_or_a_rhythm_unit`, `crates/model/tests/clipboard.rs::paste_round_trip`, `crates/model/tests/clipboard.rs::clipboard_ignores_other_text`"
supersedes: nothing
amends: nothing
---
# ADR-010 — Edits are undoable commands on one shared design in millimetres, one gesture is one undo step, work autosaves for crash recovery, and the clipboard carries objects as marked JSON text

## Decision

1. **One shared design.** Edits change the document's design, stored in mm; each device's pixels are worked out
   from it. Nothing is edited per device (per-device overrides are napkin §5.2, later).
2. **Device pixels for the hand.** Nudges, `u` in fields, and snapping use the device on screen: arrows move one
   device pixel, Shift+arrows one rhythm unit (or one subdivision when they are on), as that device resolves it.
3. **Commands.** Every edit is a command in `model::edit` that undoes and redoes. Undo is unlimited. One gesture
   (a drag, a resize, an Alt-drag copy, a layer reorder, a typing run) is one undo step, applied on release and shown
   live while it lasts; Escape drops it. Locked objects and layers refuse edits.
4. **Saving.** Save and Save As, an unsaved-changes prompt, and autosave to a recovery file every 30 s and on focus
   loss, with Restore offered after a crash. No history panel.
5. **Clipboard.** Ctrl+C, Ctrl+X, Ctrl+V put objects on the system clipboard as JSON text with a top-level
   `"quadrule-clipboard": 1` marker; a paste reads only marked text and gives new ids; pastes work between documents.
   A paste into the same document lands one rhythm unit down and right, each further paste one unit more.

## Context

- Rung 3 Q6, Q9, Q10 and Defaulted "Edits change the shared design" (`docs/bbq/2026-11-10-rung-3-first-editor.md`);
  chris on `u`, word for word: "I approve the u choice, as far as I understand it."
- Rung 4 Q8, Q10, Defaulted "One drag, one undo step" and "Copy and paste" (`docs/bbq/2026-11-10-rung-4-drag-and-snap.md`).
- The napkin's undo, autosave and recovery (§5.6) leaves the interval open; 30 s was the agent's choice.

## Consequences

- The `.quad` file holds one design; text outlines are made per device at save (ADR-001).
- Owed (`docs/STATUS-debt.md`): Save does not update `metadata.modified`; a multiple selection shows the first
  object's values, not "mixed"; only one recovery file is restored per start; a paste goes to the first selected
  object's layer.

## History

- 2026-11-10: drafted from the records cited above, in chris's backlog sweep.
