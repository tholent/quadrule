# bbq record — quadrule rung 7: groups; align and distribute

2026-11-11 (UTC 2026-11-12) · untyped bbq (the next rung of arc v1, "kick off rung 7") · 8 questions · lead: Opus 5.5
at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `STATUS.md`, `docs/arcs/v1/README.md` (rung 7 row), `docs/arcs/v1/rung-7-PROMPT-2026-11-11.md`,
`docs/decisions/INDEX.md`, `docs/bbq/2026-11-10-rung-6-shapes.md`, `docs/STATUS-debt.md` (rung 3, 4a, 6b debt),
`docs/STATUS-log.md` (newest entry), `napkins/quadrule-spec.md` §5.1–§5.5, §6.1 (line 276), §6.3 (align and distribute),
`crates/model/src/lib.rs` (`Object`, `ObjectKind`: no group; `Constraints`), `crates/layout/src/lib.rs`
(`resolve_frame` against the page), `crates/layout/src/units.rs:19` ("there are no groups yet"),
`crates/format/src/migrate.rs` (chain ends at 0.6.0), `crates/ui/src/editor.rs` (selection is a list of object ids).

Measured at the start of this session (2026-11-12 00:00 UTC, on the Pi): `echo "$CLAUDE_EFFORT"` → `high`;
`make build-proof` → `BUILD head=5fc4301 dirty=no rebuilt=7`.

Asked again at session start (AGENTS.md), chris's answers word for word:
- Should text boxes and pattern regions rotate? → "Not in v1 (Recommended)": parked for v1.1.
- Should a multi-selection's box turn? → "Upright for a selection (Recommended)": a loose selection's box stays square
  to the page; only a group, which keeps its own angle, gets a turned box.
- chris's hardware queue (grays sheet, landscape check and screenshots)? → "Nothing yet": both stay on his queue;
  Shade 204 and Block 68 stay estimates.
- Still parked, not re-asked: "Scale vs pixel-locked rhythm on A5 X" (v1.1, arc Q16); the Label gray (v1.1).

## Budget
Share: 10 points (1 point = 1 % of the weekly limit) of a 10-point ceiling, alone on the meter; reserve 0 kept for
chris.
Room: 10 points (the lower of the ceiling and the hard stop less the reading, 75 − 55 = 20, less the reserve).
Written when the weekly meter (resets Saturday 14th, 5 pm) read 55 % (2026-11-12 00:02 UTC, given by chris, not
measured; logged with `spend-ledger.py reading 55 --meter weekly`); that reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. If other sessions share the meter, your share line is your ledger, never a meter percentage. Read the meter
before each batch of dispatches; log a reading only at batch ends and when a stop line is within 5 points. Stop
starting new work at the first of: your ledger reaching 10 points (alone on the meter: the reading minus your start
reading; the token ledger is (finished workers' tokens + your context) ÷ K, K = 15.1M tokens/point fitted from one
pair in `docs/budget-calibration.md`, not yet trusted, so trust the meter's movement); the meter reaching your start
+ 10; the hard stop (75 % beyond 48 h, 80 % at 24–48 h, 85 % at 12–24 h, 93 % under 12 h). In-flight work finishes.
Hand off at ~500k of your context. Before handoff, write your row in the calibration log.

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6; the rung 7 prompt): 8.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Budget.** Set with `budget-select` above.
- **Text and pattern rotation, the selection box:** chris's session-start answers above.

## Rules

- **Q1 Split.** Two parts, a look each (arc Q10). **7a, groups:** make and ungroup, nesting, double-click to enter,
  the layers tree, group constraints, group turning, file format 0.7.0. **7b, align and distribute** in the Arrange
  tab, working on groups as single objects.
- **Q2 Group turning.** A group stores its own angle; its children keep theirs and the two add up. The group's box and
  handles turn with it, as one object's do today. A group holding a text box or a pattern region (at any depth) cannot
  turn: its rotate handle is hidden and the Rotation field greyed with a note, following the session-start ruling that
  text and patterns do not rotate in v1. Ungroup bakes the group's angle into each child.
- **Q3 Resizing a group.** Each child obeys its own constraints against the group's box, the same rule another device
  uses (spec §5.4, "Transform and constraint container", §5.2). Default children (Scale) scale; a child pinned Top
  keeps its height. What the user sees while resizing is what a different-sized device gets.
- **Q4 Selecting inside groups.** A click picks the outermost group; a double-click goes one level in and picks the
  child under the pointer; Escape, or a click outside the entered group, steps back out one level; Ctrl+click picks
  the innermost object at once (Ctrl+drag still drags with snapping off). The layers panel becomes a tree with fold
  arrows, and picking a row selects at any depth (entering its groups). The entered group gets a faint dashed outline.
- **Q5 Align to.** An "Align to" choice: Selection, Page, Margins (spec §6.3). It is Selection when two or more objects
  are picked and Page when one is, switching by itself, and it remembers what the user picks. Turned objects align by
  their turned outer box. No key object.
- **Q6 Pixels.** When the pixel-grid snap is on, align and distribute results land on whole device pixels of the
  primary profile; distribute makes the gaps equal to the pixel and spreads any leftover pixel from the first gap on.
  The rhythm is not forced (centring on the page may sit between rhythm lines). With the pixel snap off, results are
  exact millimetres.
- **Q7 Group opacity.** A group fades as one: drawn, then faded as a whole, so overlapping children do not show
  through each other (as Figma and SVG do), using the opacity group rung 6b built (`Item::BeginAlpha`).
- **Q8 Anything else to protect or avoid?** Nothing more: the standing rules hold (the six golden lines in
  `crates/render/tests/golden/SHA256SUMS` and the fixtures `crates/render/tests/fixtures/v0.4/`, `v0.5/` frozen, stop
  and ask chris if any would move; drawing and dragging under 50 ms at the 90th percentile on the Pi, ADR-009; commits
  straight to `main`).

## Owed

(empty: the three owed after round 1 became Q5, Q6, Q7)

## Defaulted

- **Size, said out loud:** 7a about one builder run (about 0.4–0.6M tokens), 7b about half that; the rung about 3–6
  points of the 10, plus two looks. veto: none expected.
- **File format 0.7.0 (7a)**, through the migration chain: a new `ObjectKind::Group(GroupShape { children:
  Vec<Object>, extra })`. Nothing old changes, so the 0.6 → 0.7 step is a no-op (as 0.1 → 0.2 was) and every older
  file opens with the same export bytes; an older build reading a group reports an error. Add a `v0.6/` fixture beside
  `v0.4/` and `v0.5/`. A group uses `frame`, `constraints`, `visible`, `locked`, `rotation_deg`, `opacity_pct`, `name`;
  its `style`, `fill`, `dash` and `cap` are ignored. veto: a separate tree beside the objects.
- **Where children live:** children's frames are stored in page millimetres in the group's unturned space, so grouping
  and ungrouping an unturned group moves nothing. veto: frames relative to the group.
- **The group's box:** a group is not a frame: its stored box is refitted to its children's outer box (each child's
  turned box) after any edit inside it, and only a resize of the group itself (Q3) resolves the children against a
  new box. veto: a stored box that does not refit.
- **Layout (7a):** a group's frame resolves against the page by its own constraints; its children resolve against
  the group's stored box → the group's resolved box, recursively, then the group's angle turns them all about the
  group's centre. A turned child in a stretched group keeps its own angle while its unturned frame follows its
  constraints (no skew). The `%` unit in fields is of the parent group when inside one (pays the
  `crates/layout/src/units.rs:19` note). veto: none expected.
- **Making and breaking:** Ctrl+G groups two or more selected objects; Ctrl+Shift+G ungroups; Group and Ungroup
  buttons also in the Arrange tab and the layers panel. The group goes on the layer and at the stack place of the
  topmost selected object, and objects from other layers move into it, keeping their order. Its name is the Fluent
  word "Group" stored as data (as rung 5 names drawn objects). Ungroup puts the children where the group was, with
  the group's angle baked in (angle added, centre moved) and its opacity multiplied into each child (so overlap
  shows through after ungrouping; the user can undo); the children keep their own constraints, now against the
  parent. One undo step each. veto: none expected.
- **Nesting:** no depth limit; tests reach at least three levels. veto: a limit.
- **A turned group (Q2):** text boxes and pattern regions cannot be grouped into, pasted into or dragged in the layers
  panel into a turned group (refused with a status-bar note). veto: none expected.
- **Visible and locked** cascade: a hidden group hides its children, a locked group locks them, as layers do.
  veto: none expected.
- **Picking, dragging, snapping:** a group is one object for picking (hit where any of its children is hit, not its
  whole box), dragging, nudging, Alt-drag duplicating and snapping (point snap, smart guides and equal spacing use its
  box, a turned group its turned corners). Inside an entered group, both its other children and the objects outside
  it count for snapping. veto: picked by the group's box.
- **Clipboard and duplicate:** copying a group copies its whole tree; paste and duplicate give every descendant a new
  id. Copied text from rung 6 builds pastes unchanged. veto: none expected.
- **Layers tree:** fold arrows and indent per depth; a group row shows its children; dragging rows reorders anywhere in
  the tree, into and out of groups (a drop on the middle of a group row puts the row inside it, on top). Rename, eye
  and lock work on group rows. veto: moving into and out of groups only by Group/Ungroup.
- **Inspector for a group:** Arrange (position, size, rotation), Style with Opacity only (no stroke, fill, dash, cap),
  Constraints; the children's Constraints tab reads "against the group" when inside one. veto: none expected.
- **Lint:** every rule runs on each leaf object at its resolved, turned place; a finding names the object and outlines
  it; a group itself has no rules. Pays the rung 3 debt "Lint keeps its own copy of the object-bounds maths" by using
  `quadrule_layout::placed_objects` (`crates/lint/src/lib.rs`). veto: none expected.
- **Render:** a group's children are drawn in order inside it; a group below 100 % is one opacity group (Q7); hidden
  children are not drawn. The existing `Item` variants keep drawing exactly as now. veto: none expected.
- **Goldens (7a):** one new golden page per device holding nested groups (three levels), a turned group with a turned
  child, a group at 50 % opacity with overlapping children, and a group whose child is pinned Top and Right, resolved
  on both devices; added beside the six existing lines, which stay unchanged. 7b changes no export (align and
  distribute only move objects), so it adds no golden. veto: none expected.
- **Speed:** measure dragging a group and drawing on the Pi in a release build, both canvases, 90th percentile (ADR-009),
  on the heaviest page the rung makes: a groups page opened with `--groups-page`, with `--bench-drag` over it, and
  `make bench-*` targets as before. Say before any number crosses 50 ms. veto: none expected.
- **7b, align and distribute:** in the Arrange tab, six align buttons (left, centre, right, top, middle, bottom), four
  distribute buttons (horizontally and vertically, by spacing and by centres; three or more objects), and the Align to
  choice (Q5). Groups are single objects; inside an entered group, Page and Margins still mean the page's. Locked
  objects stay put and are left out of the selection's box. One undo step per command. Keyboard shortcuts for align
  wait for rung 17 (spec §14, "Keyboard"). veto: shortcuts now.
- **7b pays the rung 3 debt** "A multiple selection shows the first object's values, not 'mixed'" in the Arrange tab
  it changes: position, size and rotation show "mixed" when the selected objects differ. veto: none expected.
- **Debt paid in 7a's code:** "A group resized past its opposite side flips as a box but its contents are not
  mirrored" (4a) — for a real group, a resize stops at zero size instead of flipping; "resizing a group with turned
  objects stretches them approximately" (6b) — for a real group, Q3's constraint rule replaces it (a loose
  multi-selection keeps today's behaviour, still debt). veto: none expected.
- **Tests:** headless, one `CHECK` line each, printed starting with a newline: 0.6 files open with the same export
  bytes; group and ungroup round trip (positions, angles, opacity); nesting three deep; enter and leave (double-click
  through a test method, as rung 5's keep-tool, since `egui_kittest` cannot make one); Ctrl+click; group resize under
  each constraint; group turn and the text/pattern refusal; clipboard of a group; lint inside groups; layers-tree
  reorder into and out of groups; the new goldens; existing goldens unchanged; 7b: each align and distribute command
  against selection, page and margins, whole-pixel results, turned objects, groups as one. veto: none expected.
- **Words and access:** every new string through Fluent; every new control named for screen readers. veto: none.
- **Versions and releases:** 7a's approval commit bumps the workspace to `0.7.0` and is tagged `v0.7.0`; 7b's is `0.7.1`,
  `v0.7.1`. Commit `docs/arcs/v1/release-notes/<tag>.md` before tagging, then push the tag; the release workflow drafts,
  attaches and publishes (`.github/workflows/release.yml`); never `gh release create` by hand. Watch the run and check
  the release's files. veto: none expected.
- **Landing:** commits straight to `main` (`AGENTS.md`, amendment of 2026-11-09); CI checked on each push. README
  ("what works today") and DEVELOPMENT.md kept true. veto: none expected.

## Kickoff

Run on **Opus 5.5 at high effort** as the lead; on any other model or effort, stop and say so before doing anything
(read the effort with `echo "$CLAUDE_EFFORT"`). Read `AGENTS.md`, the board `docs/arcs/v1/README.md` and this record
(`docs/bbq/2026-11-11-rung-7-groups-and-align.md` on `main`). Prove the build matches HEAD (`make build-proof`). Take
the meter reading and apply the Budget block. Have **Sonnet 5.5 at high effort** build **7a** on `main`: file format
0.7.0 with a no-op migration step and a `v0.6/` fixture; the Group kind; group constraints and layout; group turning
with the text/pattern refusal; Ctrl+G and Ctrl+Shift+G; double-click to enter, Escape to leave, Ctrl+click; the layers
tree; lint inside groups; a new golden page per device. Keep existing export bytes frozen (stop and ask chris if any
would change). Judge the builder's work against this record, run `make check-all`, measure drawing and dragging on
the Pi (90th percentile, ADR-009), show chris the window and stop for his look. After his approval: bump to `0.7.0`,
commit the notes, tag `v0.7.0` (the tag push publishes), then 7b (align and distribute) the same way, tagged `v0.7.1`.
Re-plan the board after the rung, and write the next lead's prompt in `docs/arcs/v1/`. Update `STATUS.md`,
`docs/STATUS-log.md` and `docs/STATUS-debt.md` at the end of each session.
