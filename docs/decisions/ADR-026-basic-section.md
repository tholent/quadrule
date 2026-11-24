---
status: accepted
status-note: "ratified by chris 2026-11-24 after the rung 16b look: \"3. ratified\""
ratified-by: chris
ratified-on: "2026-11-24"
owner: chris
kind: design
built: landed
proof: "`crates/format/tests/basic_marks.rs::marks_round_trip_and_unmarked_files_write_none`, `crates/format/tests/basic_marks.rs::taking_a_mark_off_writes_nothing_for_it`, `crates/render/tests/migrate_v020.rs::v020_files_open_migrate_and_export_the_same_bytes`, `crates/render/tests/migrate_v020.rs::the_chain_has_the_0_20_to_0_21_step`, `crates/ui/tests/basic.rs::the_basic_tab_is_first_and_always_there`, `crates/ui/tests/basic.rs::empty_basic_tab_says_how_to_mark_something`, `crates/ui/tests/basic.rs::the_switches_mark_and_unmark_in_one_undo_step_each`, `crates/ui/tests/basic.rs::the_basic_tab_lists_what_is_marked_with_the_other_tabs_widgets`, `crates/ui/tests/basic.rs::an_edit_in_the_basic_tab_is_the_same_edit_and_leaves_the_selection`"
supersedes: nothing
amends: nothing
---
# ADR-026 — A Basic tab lists what is marked "Show in Basic", so a template's few main settings sit on one pane

## Decision

1. **A switch per element.** Each object, and the page's **margins**, **rhythm** and **title**, has a **Show in Basic**
   switch, off by default (Q5, chris: "Switch per element").
2. **A Basic tab, first in the inspector.** It lists everything marked, by name, with its main settings, drawn by the
   same widgets the other tabs use. It shows whether or not anything is selected, because it is about the document.
   With nothing marked it says so in a sentence and says how to mark something. **Every setting stays in Style and the
   other tabs too**: Basic is a short list on top, never the only place (chris, 2026-11-18: "we can choose to expose
   certain elements of a template that can be easily changed on a single pane, while all features are available in
   'style, etc...'").
3. **What each kind shows** (the builder's proposal, for chris's look):
   - the page: its title; its margins (the link tick and the four lengths); its rhythm's spacing (the side of an
     isometric rhythm, or across and down with their link tick);
   - a pattern region: what defines that pattern (a Cornell pattern's cue column and summary band, a hex's height, a
     music staff's gap, pitch and grand staff, a calendar's month and week start, an isometric region's style), whether
     it follows the rhythm and its own spacing when it does not, and its major lines (follow, off or its own N);
   - a line: its look and role (or its width and gray, when it has its own);
   - a text box: its text;
   - a shape: its stroke and its fill; a mini calendar: its month, year and week start;
   - a group or a repeat: its name and a line saying the rest is in the other tabs (mark what is in it, or use the
     Repeat tab).
4. **Where the switch is.** On the Arrange tab, first, for the selected objects (mixed when some have it and some do
   not); on the Page tab beside the title row and the Margins and Rhythm headings; and, once an item is listed, a tick
   on its row in the Basic tab takes it off again. Each switch is one undo step.
5. **The file.** Format **0.21.0**: an optional `show_in_basic` on an object and an optional `basic` (`margins`,
   `rhythm`, `title`) on the page, each written only when on, so every file made before reads and saves as it did and
   exports the same bytes. The reader always understands the marks, whether or not the flag below is on.
6. **A flag until chris approves the look.** The Basic tab and the switches show only when the app starts with
   `--basic-section` (ADR-007, amendment of 2026-11-10: a change to the file format lands behind a flag); a file with
   marks opens the same with it off. `--look basic` shows the tab on a sample page with things marked, and the flag
   combines with every other, `--look welcome` included. Once chris approves, the flag is removed (one field, the
   `show_switches()` checks in the inspector, the Arrange tab and the Page tab, and one command-line option; the list
   is in `crates/ui/src/basic.rs`).

## Context

chris asked for a simple pane beside the full inspector (2026-11-18, quoted above). The rung 16b bbq
(`docs/bbq/2026-11-24-rung-16b-starters.md`) asked how, and chris chose "Switch per element" (Q5); the lead defaulted
the format (Defaulted 6: format 0.21.0, new fixtures, the reader always understands the marks) and what each starter
exposes (Defaulted 7: its spacing, its margins and its title, plus the one or two settings that make it that page).
The starters are made by builders in the same rung (legs 2 and 3), and each marks what a person is meant to change, so
the section has to exist first. The previous format change (0.20.0, ADR-023) is the model for the file work.

Drivers:
- A template's maker should be able to hand over a page whose few meant-to-change settings sit together.
- Nothing already made may change its export (ADR-007's rule for export bytes and the file format).
- No setting may live only in Basic (chris's own words).

## Alternatives

### A mark on each element, and a tab that lists the marked (chosen: chris's ruling, Q5)

- Good: the maker chooses exactly what is shown; the tab is built from the same widgets, so a setting is the same
  wherever it is changed; unmarked pages are untouched.
- Bad: one more field on every object and on the page (format 0.21.0); the marks are the maker's work to keep up.

### A fixed list of settings per kind

- Good: no marks, no file change.
- Bad: every page would show the same settings whether or not they matter to it. Not chosen (Q5; Defaulted 7 also
  says no fixed list).

### A second, simpler inspector

- Good: one clean pane.
- Bad: two places to keep true, and the full inspector would lose its place as the one home of every setting. Not
  chosen (Q5).

## Consequences

- The model gains `Object::show_in_basic` and `PageSettings::basic` (`BasicPage`); format 0.21.0 with a no-op
  migration step; the fixtures `crates/render/tests/fixtures/v0.20/` (saved by the 0.20.0 format before the change)
  export the same bytes.
- The inspector's page, pattern, stroke and text widgets are now helper functions that the Basic tab calls
  (`title_row`, `margins_rows`, `rhythm_unit_rows`, `pattern_kind_rows`, `pattern_spacing_rows`, `pattern_major_rows`,
  `stroke_rows`, `text_content_field`), so the two tabs cannot drift apart.
- With the flag on, taking the selection away while an object tab shows returns to the first tab, which is Basic.
- Leg 2 and leg 3 of rung 16b save each starter with its marks; the starters' own words and looks are theirs.
- No golden image changes: the section adds nothing to the page's drawing.

## Asks left open

1. chris's look at the Basic tab and the switches: every word is a proposal (`Basic`, `Show in Basic`, `Page`, `Select`,
   the empty sentence), as is the place of each switch.
2. Whether the layers panel should also mark a marked object (a small sign beside its name); not built.

## History

- 2026-11-24: drafted by the rung 16b leg 1 builder (Sonnet 5.5, high) from the bbq's Q5 and Defaulted 6 and 7;
  built behind `--basic-section` on `main`, awaiting chris's look.
- 2026-11-24: ratified by chris after the rung 16b look, word for word: "1. looks good 2. all good, for now 3. ratified".
  Before: `status: proposed`; `status-note: "drafted 2026-11-24 by the leg 1 builder from chris's ruling (rung 16b bbq, Q5, \"Switch per element\") and the lead's Defaulted 6 and 7; awaiting chris's look and ratification"`.
- 2026-11-24: header only, after the flag came out (ADR-007; chris's approval and ratification above). Before:
  `built: partial`; `owed: "chris's look at the Basic tab and the switches (all words and the look are the builder's proposals), then ratification; removing the `--basic-section` flag after he approves; the marks on each starter (legs 2 and 3 of rung 16b)"`.
  Now `built: landed` and no `owed:` line. In `proof:`, `nothing_shows_without_the_flag` is replaced by
  `the_basic_tab_is_first_and_always_there`, the test of the section with no flag.
