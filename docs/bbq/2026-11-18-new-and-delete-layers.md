# bbq record — quadrule: making and deleting layers

2026-11-18 · untyped bbq (a gap found while listing chris's starters: spec §13 asks for named layers, and nothing in
the app adds a second layer) · 4 questions · lead: Opus 5.5 at high effort (this session); builder Sonnet 5.5 at high
effort (arc Q2).

Grounding: `Layer` in `crates/model/src/lib.rs:407` (name, visible, locked, objects); `Command` in
`crates/model/src/edit.rs` has `SetLayer` (rename, eye, lock), `MoveLayer` and `MoveObject` (to another layer), and no
command that adds or removes a layer; a blank page has one layer (`crates/model/src/blank.rs`, `FIRST_LAYER_NAME`);
the layers panel (`crates/ui/src/panels.rs`, `layers_panel`) has Move up, Move down, Group and Ungroup, renames on
double-click, and already moves an object to another layer by dragging its row (`Plan::Object`, `Place::Layer`);
spec §5.5 ("Starter templates use clearly named layers ("Grid", "Header", "Labels")").

chris chose route 1 ("let's tackle the layer gap with route #1 and get that squared away"): add the controls now,
ship as `v0.16.2`, and the starters (16b) move to `v0.16.3`.

## Budget

No share set. chris lifted this week's hard limits until the reset, Saturday 2026-11-21, 5 pm EST (rung 16 record,
Amendments). Meter last read 65 % (at 16a's approval); not read since. Sized as one builder leg, about 1–3 points.

## Rules

- **Q1 Controls.** chris: "Buttons + right-click (Recommended)". **New layer** and **Delete layer** buttons in the
  layers panel's button row (beside Move up, Move down, Group, Ungroup), and the same two items on a layer row's
  right-click menu.
- **Q2 Delete.** chris: "Ask first (Recommended)". An empty layer is deleted at once. A layer with objects asks
  "Delete the layer and its N objects?" first. Either way, one Undo brings the layer and its objects back.
- **Q3 New layer.** chris: "Above selected, name open (Recommended)". A new layer goes just above the selected layer
  (on top when none is selected), named "Layer 2", "Layer 3"…, with its name field open to type over at once. It
  becomes the selected layer.
- **Q4 Anything else.** chris: "Nothing more (Recommended)". A Move to layer menu on the page was offered and not
  picked; dragging rows in the layers panel stays the way to move objects between layers.

## Owed

Empty.

## Defaulted (the lead's calls; each can be vetoed)

- **The model.** Two new commands in `crates/model/src/edit.rs`: insert a layer at an index, and remove a layer at an
  index; each undoes the other, the removed layer coming back whole with its objects in its place. *veto:* none
  expected.
- **The last layer cannot be deleted.** With one layer left, Delete layer is greyed, with a hover note saying why.
  *veto:* none expected.
- **Locked layers are not deleted.** Delete layer is greyed on a locked layer, with a hover note ("Unlock it to
  delete it"), as edits to a locked layer's objects are refused today. *veto:* delete it anyway.
- **Which layer is "selected".** The layer whose row is picked in the panel; when an object is picked on the page,
  the layer holding it. Delete layer deletes that one. *veto:* none expected.
- **The name.** "Layer N" uses the lowest N not already taken, through Fluent (ADR-008); the name field opens as a
  double-click opens it today, and Enter or clicking away keeps it, Esc keeps "Layer N". *veto:* none expected.
- **One undo step each** (ADR-010): New layer (with its name, if typed before the field closes, as one or two steps,
  whichever the existing rename does), Delete layer.
- **No change to the file format or export bytes.** Layers already save and load; the 32 golden lines and fixtures
  `v0.4/`–`v0.16/` stay frozen, and a change that moves any is a stop.
- **Words** (proposals for chris's look): "New layer", "Delete layer", "Delete the layer "{name}" and its {N}
  objects?", "Delete", "Cancel", the hover notes. *veto:* other words.
- **No flag** (ADR-007: editor features land switched on; this changes neither the format nor export bytes).
- **No ADR change.** ADR-013 places the layers panel; adding two buttons and a menu inside it changes nothing it rules.
  *veto:* none expected.

## Amendments

(Rulings made after the bbq, at the look or later, go here word for word, dated.)

## Kickoff

Builder: **Sonnet 5.5 at high effort**, on `main`, committing as each step lands. Build from this record's Rules and
Defaulted list: the two model commands with their undo, the buttons and the right-click items, the ask-first sheet,
the new layer's open name field, tests that print one structured line, a look state for the screenshots. Then the lead
shows chris the window and stops for his look. After approval: `0.16.2`, release notes, the tag, `STATUS.md`, the
log, the board, and the 16b prompt's version line (`0.16.3`).
