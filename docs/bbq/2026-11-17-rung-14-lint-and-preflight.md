# bbq record — quadrule rung 14: lint rules (14a) and the lint window and preflight (14b)

2026-11-17 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-14-PROMPT-2026-11-17.md`) · 9 questions (the
arc's "about 8") · lead: Opus 5.5 at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 14 row), `docs/arcs/v1/rung-14-PROMPT-2026-11-17.md`,
`docs/decisions/INDEX.md`, `napkins/quadrule-spec.md` §8.2 and §8.3, `crates/lint/src/lib.rs` (L01/L02 at lines
411–462, `SKIPPED_ON_PAPER` at 954), `crates/profiles/src/lib.rs` (the `Profile` fields: no lint thresholds today),
`crates/profiles/data/supernote.a6x2.ron` and `supernote.a5x2.ron`, `docs/device-measurements/supernote.a6x2/calibration.md`
(the sheet codes behind Q3), `crates/ui/src/panels.rs` (`lint_tab` at 2467: a plain list, no filter or fixes; the
Style tab's Custom gray is a 0–255 number field at 1505), `crates/ui/src/missing_ui.rs` (rung 12's sheet before Save,
Save As and Export), `crates/format/src/lib.rs` (`FORMAT_VERSION` 0.14.0), `docs/STATUS-debt.md`,
`docs/budget-calibration.md`.

Measured at the start of this session (2026-11-17 on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=eaa223a dirty=no rebuilt=11` (the 0.13.0 bump had left eleven crates stale; rebuilt, the build matches HEAD).

## Asked again at session start (AGENTS.md)

- The Label gray: kept parked (v1.1), chris did not reopen it.
- **The A5 X rhythm question (arc Q16, parked since `spikes/render/FINDINGS.md:55`, 2026-11-09): reopened and ruled.**
  chris, word for word: "let's look at the scale vs pixel locked rhythm question". The lead showed that since rung 9
  the code already answers it: a pattern bound to the rhythm takes its spacing from the rhythm resolved on each device
  in millimetres, never scaled (`crates/layout/src/pattern.rs:102`, `crates/layout/src/lib.rs:291`), so a 6 mm rhythm is
  53 px on the A5 X (226 PPI); the pattern's frame and every other object follow their constraints (Scale by default),
  so an object placed on the rhythm on the primary can fall between the rhythm's lines on another device, and L16
  checks only the primary. chris, word for word: **"Ratify today's behaviour (Recommended)"**: the rhythm wins for
  spacing (patterns, repeat pitch in `u`); frames follow their constraints. No code change; goldens untouched. The
  question is closed; the A5 X profile (v1.1) applies this rule.

## Budget
Share: 15 points (1 point = 1 % of the weekly limit), chris: "15 points (Recommended)"; alone on the meter, reserve 0
(chris: "Alone, no reserve (Recommended)").
Room: 33 points (the hard stop, 75 %, less the reading; reset about 103 h away).
Written when the weekly meter (reset Saturday 2026-11-21, 5 pm EST) read 42 % (2026-11-17 14:32 UTC, given by chris,
not measured; logged with `spend-ledger.py reading 42 --meter weekly`); that reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. Read the meter before each batch of dispatches; log a reading only at batch ends and when a stop line is within
5 points. Stop starting new work at the first of: your ledger reaching 15 points (alone on the meter: the reading minus
your start reading); the meter reaching 57 %; the hard stop (75 % beyond 48 h, 80 % at 24–48 h, 85 % at 12–24 h, 93 %
under 12 h). K = 12.2M tokens/point fitted over two pairs in `docs/budget-calibration.md`, not yet trusted, so trust
the meter's movement. In-flight work finishes. Hand off at ~500k of your context. Before handoff, write your row in the
calibration log.

## Rules

- **Q1 Split.** chris: "14a rules + 14b window (Recommended)". **14a**: the six new rules (L03, L05, L07, L09, L13,
  L14) and their thresholds in the profiles. **14b**: quick fixes, Ignore for this document, Free placement, the device
  filter in the Lint tab, the preflight sheet, the Style tab gray swatches. One `v0.14.0` when both are approved, a look
  each.
- **Q2 Where the numbers live.** chris: "A lint block in profiles (Recommended)". Each profile gets a new `lint` block
  (lightest visible gray by stroke width and dot size, for fills and text; smallest text size; smallest comfortable
  handwriting step; the dark-fill gray and page share; the thin-line width), filled from chris's measurements, editable
  in the Profiles window, with Reset to built-in. A page carries profile copies, so the file format goes to **0.15.0**
  through the migration chain with a `v0.14/` fixture; export bytes are untouched. A profile with no value for a rule
  (paper, or a user's own left empty) skips that rule, and the Lint tab and the preflight sheet say which checks were
  skipped and why.
- **Q3 L03, L05, L07.** chris: "Accept as listed (Recommended)". From the Nomad sheets (the Manta is ruled the same):
  - **L03 gray too light (Warn):** a 1 px stroke lighter than 187; a 2 px or wider stroke lighter than 204 (LA12,
    LB13; 221 and 238 vanished); square dots by size: 2 px lighter than 102, 3 px lighter than 170, 4 px and larger
    lighter than 187 (PB3, PC5, PD6); a fill lighter than 204 (Shade 204 shows); text lighter than 153 (lightest
    tested, TD4).
  - **L05 gray not in the clean levels (Info):** any gray not in the profile's `gray_levels`.
  - **L07 text too small (Warn):** under 6 pt (smallest tested; every sample legible).
- **Q4 L09, L13, L14.** chris: "Accept as listed (Recommended)". No measurement; each value sits in the lint block and
  can be retuned without code.
  - **L09 handwriting (Info, Template only):** the rhythm's vertical step, or a ruled pattern's spacing, under 5 mm
    (Supernote's own grid is 5 mm).
  - **L13 dark fills (Info, Template only):** a fill at gray 102 (Block) or darker covering more than 10 % of the page.
  - **L14 soft thin lines (Info):** a stroke 2 px or thinner that is slanted or curved (most of its pixels come out as
    in-between grays). L02 then keeps only straight edges; its "diagonal line" note (`lint-l02-diagonal`) moves to L14.
- **Q5 Preflight sheet.** chris: "As sketched (Recommended)". Export opens one sheet when there are Warn findings,
  listed by device (then a Document group), each row with **Jump to object** and **Ignore this rule for this
  document**; Info findings are folded under "N notes"; a line says why a paper page has fewer checks. Rung 12's font
  warning (L20) becomes a row in this same sheet on Export. Buttons: **Cancel** and **Export anyway**. Save and Save
  As keep rung 12's font-only sheet unchanged. The sketch chosen:

  ```
  +------------------------------------------------+
  | Before you export                              |
  | Manta                                          |
  |  ! L03 "Header" gray 230 may not show  [Jump][Ignore] |
  |  ! L07 "Caption" 5 pt is below 6 pt    [Jump][Ignore] |
  | Nomad                                          |
  |  ! L06 "Logo" sits under the toolbar   [Jump][Ignore] |
  | Document                                       |
  |  ! L20 Uses fonts installed here: Inter (embedded)    |
  | > 3 notes                                      |
  | Paper pages skip gray, ink and safe-zone checks.     |
  |                     [Cancel]  [Export anyway]  |
  +------------------------------------------------+
  ```
- **Q6 Quick fixes.** chris: "All of these (Recommended)". Each finding in the Lint tab and the preflight sheet gets
  a button that fixes it in one undo step: L01/L02 **Snap to pixel grid** (whole-pixel width); L03/L05 **Use nearest
  clean gray** (the closest visible clean level) or **Use <Role> role**; L06 **Move inside safe area**; L07 **Use 6 pt**
  (the profile's smallest legible size); L16 **Snap to rhythm** (this object, or all off-rhythm objects); L17 **Bind to
  rhythm**; L18 **Center rhythm origin**. No fix button for L08, L09, L10, L11, L13, L14, L15, L19, L20.
- **Q7 Ignore, Free placement, device filter.** chris: "As proposed (Recommended)". **Ignore this rule for this
  document** turns a whole rule off for this page on every device; saved in the file (part of 0.15.0); the Lint tab
  ends with "Ignored rules (N)" where each can be turned back on. **Free placement** is a checkbox in the Arrange tab,
  saved per object, that silences L16 only (spec §8.3). The Lint tab gets a **device picker** at the top: All devices
  (default) or one device.
- **Q8 Style tab grays.** chris: "As sketched (Recommended)". Under Custom, the gray number field becomes the
  profile's clean gray swatches (16 on the Nomad and Manta, two rows of 8), the chosen one outlined; **Show all
  grays** opens the 0–255 slider. When width and gray match a role exactly, a **matches <Role>** badge shows; when they
  don't, **Use recommended** sets the nearest role by width. A profile with no gray levels (paper) shows the slider
  only. Fill and text grays get the same swatches. The sketch chosen:

  ```
  Stroke  ( ) Role  (o) Custom
  Width   [ 0.085 mm ]
  Gray    [#][#][#][#][#][#][#][#]
          [#][#][#][#][#][#][#][#]
          [ ] Show all grays
          matches Guide        or  [Use recommended]
  ```
- **Q9 Anything else.** chris: "Nothing more (Recommended)". Held: the 29 golden lines and the `v0.4/`–`v0.13/`
  fixtures stay frozen (stop and ask chris if any would change, arc Q20); the Pi speed check (under 50 ms at the 90th
  percentile, ADR-009); builders run the app with `XDG_CONFIG_HOME` and `XDG_DATA_HOME` in a scratch folder; the rung
  13 debt in this code is paid (Profiles window pointer tests, a delete button for a damaged profile file, the Page
  tab's PPI choice wrapping onto a second line).

## Owed

None.

## Defaulted (the lead's calls; each can be vetoed)

- **Profile in use.** Every new rule reads the profile in use through `quadrule_profiles::active::lookup` /
  `lookup_for` (an override or a user's own), never the shipped values; a test edits an override's lint block and shows
  the rule follow it. veto: none expected.
- **The gray a rule judges** is the gray the object puts on the page: its gray blended by its opacity (and its groups')
  over the page background. A 50 % black line is judged as about 128. veto: judge the set gray, ignoring opacity.
- **Roles are judged too.** L03 and L05 judge role-styled objects as well as Custom ones (the built-in roles pass,
  being calibrated; a user's edited role may not). veto: none expected.
- **Paper profiles' lint blocks** carry only L07 (6 pt) and L09 (5 mm); L03, L05, L13 and L14 stay skipped on paper
  (`SKIPPED_ON_PAPER` gains L13 and L14), since paper has no gray levels and prints at a PPI the user picks. veto: run
  L13 and L14 on paper too.
- **Template only** (L09, L13) means the rule is skipped when the document is a Wallpaper (`crates/model/src/lib.rs:63`).
  veto: none expected.
- **L13's area** is each fill's own laid-out area on the device (a repeat counts all its copies together), not the union
  of overlapping fills. veto: the union.
- **L07's size** is the size the text is drawn at on that device (a "Scale with page" text box shrinks with the page).
  veto: none expected.
- **L03 for dots** reads the dot grid's dot size on the device and the nearest listed size at or below it. veto: none
  expected.
- **The lint block's field names** are the builder's proposal, shown at 14a's look (taste: chris may rename).
- **An ADR-012 amendment** recording "lint thresholds live in profile data" is proposed by the lead at 14a's close
  through `makeadr`, for chris to ratify; it is not ruled by this record.
- **The device filter and the notes fold** are view state, not saved in the file.
- **Builders:** one Sonnet 5.5 high subagent per part, on `main`, committing and pushing each finished step and reading
  CI for its own commits; never waiting with `pgrep -f` on a pattern in its own command.
- **Measurement** (ADR-009) after 14b: Pi, release build, both canvases, strip on, 90th percentile, on the heaviest
  sample page; lint must not push drawing or dragging past 50 ms.

## Kickoff

This session leads (Opus 5.5 at high effort; any other model or effort stops and says so). Brief a Sonnet 5.5 high
builder for 14a from this record; judge it; show chris the window for 14a's look; then 14b the same way. After both
looks: bump to `0.14.0`, commit `docs/arcs/v1/release-notes/v0.14.0.md`, push tag `v0.14.0`, re-plan the board, write
the rung 15 prompt, update `STATUS.md`, `docs/STATUS-log.md`, `docs/STATUS-debt.md` and the calibration row.

## Amendments

- **2026-11-17, 14a look.** The lead opened `--lint-page` on the Pi (release build, scratch config folders): the window
  starts and the Lint tab shows 14 findings; the builder's checks were confirmed (`sha256sum -c` on the goldens: 29
  lines, none not OK; only `v0.14/` added under fixtures; CI green on `ad948c3` and `547c39d`). The lead put the
  builder's taste calls to chris (the lint block's field names; an override saved before 0.15 takes the shipped lint
  values; L13 summing all dark fills on the page into one finding, which departs from Defaulted "L13's area"; L14
  covering isometric, hex and turned patterns; L03 and L05 both firing on one pale gray). chris, word for word:
  **"approved. meter read is 46%."** 14a is approved with those calls as built; Defaulted "L13's area" is replaced by
  the sum over the page. The Page tab's PPI wrap (Q9 debt) moves to 14b.
