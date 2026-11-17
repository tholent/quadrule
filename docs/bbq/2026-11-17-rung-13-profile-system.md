# bbq record — quadrule rung 13: the profile system (13a) and paper sizes (13b)

2026-11-17 · untyped bbq (the next rung of arc v1, `docs/arcs/v1/rung-13-PROMPT-2026-11-17.md`) · 14 questions (the
arc's "about 8"; Q9 asked to read chris's Q6 ruling, Q11–Q14 because chris added paper sizes at Q10) · lead: Opus 5.5
at high effort; builders Sonnet 5.5 at high effort (arc Q2).

Grounding: `AGENTS.md`, `STATUS.md`, `docs/arcs/v1/README.md` (rung 13 row), `docs/arcs/v1/rung-13-PROMPT-2026-11-17.md`,
`docs/decisions/INDEX.md`, ADR-012 in full, `docs/bbq/2026-11-16-rung-12-missing-fonts.md`, `napkins/quadrule-spec.md`
§3 (profiles; §3.3.1 paper sizes), §8.2 (role values "editable in the profile editor … local override with Reset to
built-in"), §12.3 (custom profiles "export as standalone `.ron` files"), §16 (paper sizes listed under v1.1),
`crates/profiles/src/lib.rs` (`builtin`, `builtin_for`, `builtin_ids`, `from_ron`), `docs/STATUS-debt.md`,
`docs/budget-calibration.md`.

Measured at the start of this session (2026-11-17 on the Pi): `echo "$CLAUDE_EFFORT"` → `high`; `make build-proof` →
`BUILD head=979bd59 dirty=no rebuilt=11`. Production code reads profiles in about 30 places, all through
`quadrule_profiles::builtin`, `builtin_for` or `builtin_ids` (`grep -rn "builtin_for\|builtin(\|builtin_ids"
crates/*/src`: `crates/layout/src/{lib,newpage,repeat,device}.rs`, `crates/ui/src/{editor,viewer,fonts,missing,view,
panels}.rs`, `crates/xtask/src/main.rs`). A `.quad` naming a profile this build lacks refuses to open today
(`crates/ui/src/viewer.rs:539`). The config folder comes from `directories::ProjectDirs::from("org", "quadrule",
"quadrule")` (`crates/app/src/main.rs:126`, used for recovery files); `uuid` with `v4` and `rfd` are already workspace
dependencies (`Cargo.toml`).

Asked again at session start (AGENTS.md): the two parked v1.1 questions. chris: "Keep both parked (Recommended)": the
A5 X rhythm (arc Q16) and the Label gray stay v1.1.

## Budget
Share: 10 points (1 point = 1 % of the weekly limit), absolute allocation by chris ("10 points, alone (Recommended)"),
alone on the meter, reserve 0.
Room: 10 points (the hard stop, 75 %, is 43 points away).
Written when the weekly meter (reset Saturday 2026-11-21, 5 pm EST) read 32 % (2026-11-17 02:20 UTC, given by chris,
not measured; logged with `spend-ledger.py reading 32 --meter weekly`); that reading is not yours. On pickup: take the
reading (or ask chris for it), name your meter, note your start reading, and cap the share at your hard stop minus your
reading. Read the meter before each batch of dispatches; log a reading only at batch ends and when a stop line is within
5 points. Stop starting new work at the first of: your ledger reaching 10 points (alone on the meter: the reading minus
your start reading); the meter reaching 42 %; the hard stop (75 % beyond 48 h, 80 % at 24–48 h, 85 % at 12–24 h, 93 %
under 12 h). K = 12.2M tokens/point fitted over two pairs in `docs/budget-calibration.md`, not yet trusted, so trust
the meter's movement. In-flight work finishes. Hand off at ~500k of your context. Before handoff, write your row in the
calibration log. chris was told at Q11 that 10 points is tight for 13a and 13b together; the lead asks for more before
starting 13b if the share runs out.

## Settled before the rounds (cited, not asked)

- **Count.** About 8 (arc Q6; the rung 13 prompt); 14 asked, for the reasons in the header.
- **Lead.** Opus 5.5 at high effort leads, Sonnet 5.5 at high effort builds (arc Q2).
- **Frozen bytes.** The 26 golden lines in `crates/render/tests/golden/SHA256SUMS` and the fixtures `v0.4/` to `v0.12/`
  stay unchanged; any change that would move them is a stop to ask chris (arc Q20).
- **Profile data.** Role values come from calibration and live in profile data (ADR-012 item 2); built-ins ship
  embedded, user profiles live in the config folder, an edited built-in is a local override with Reset (spec §3.2,
  §8.2); custom profiles export as `.ron` (spec §12.3).
- **Paper profiles' shape** (spec §3.3.1, once paper sizes were added at Q10): physical size plus an export PPI of 150,
  200, 300 (default) or 600; no safe zones, no device gray levels, no ink values; lint rules that need them (L04, L05,
  L06) are skipped and the others use generic thresholds; line roles resolve from physical sizes; they work as primary
  or secondary profiles.

## Rules

- **Q1 Split.** One rung, one look. chris: "One rung, one look (Recommended)". **Amended by Q11** once paper sizes
  joined the rung.
- **Q2 What a user may edit.** Everything, with the screen locked on built-ins. chris: "Everything, screen locked on
  built-ins (Recommended)". A user's own profile: every field. An edited Manta or Nomad: every field except its id,
  pixel size, ppi and short name, so it stays the same device. Calibrated values (line roles, fill roles, gray levels,
  pen grays, safe zones) sit under an Advanced section.
- **Q3 Editing a built-in.** A local override in the config folder that applies to every page on this computer using
  that device, existing files included, when drawn or exported, with an **Edited** mark wherever the device's name
  shows (the Page tab, the previews strip, New…, the profile editor), and **Reset to built-in**, which deletes the
  override file. chris: "Yes, with an Edited mark (Recommended)".
- **Q4 A page naming a profile this computer lacks.** The `.quad` carries its profiles: saving puts a copy of every
  profile the page uses that is not an unedited built-in (the user's own, or an edited Manta or Nomad) inside the file.
  Opening it where that profile is missing uses the carried copy, with a bar: "Uses a profile you don't have: X. [Add
  to my profiles]". File format **0.14.0**; older files open unchanged. chris: "Carry them, offer to add
  (Recommended)". The option's "Use the file's / Use mine" bar for differing copies is replaced by Q6 and Q9.
- **Q5 The profile editor.** Its own **Profiles window**, opened by a button beside the device ticks in the Page tab
  and from the menu; the list on the left (Built in, Mine; Paper after 13b), the form on the right: a Screen section
  (Name, Size, PPI, Margins; Size and PPI locked on built-ins), collapsible Line roles, Fill roles, Gray levels and pen
  grays, Safe zones (a table of pixel boxes), About (verified, date, notes); New…, Duplicate, Import…, Export…, Reset to
  built-in, Cancel, Save. The open page redraws as values change. chris chose "A: Profiles window (Recommended)" with
  this sketch:

  ```
  ┌ Profiles ───────────────────────────────────────────┐
  │ BUILT IN          │ Supernote Nomad (A6 X2) · Edited │
  │   Manta           │ ── Screen ────────────────────── │
  │ ▶ Nomad · Edited  │ Name    [Supernote Nomad (A6 X2)]│
  │ MINE              │ Size    1404 × 1872 px   🔒       │
  │   Kindle Scribe   │ PPI     300              🔒       │
  │                   │ Margins [6] [6] [6] [6] mm       │
  │                   │ ▸ Line roles      (calibrated)   │
  │                   │ ▸ Fill roles                     │
  │                   │ ▸ Gray levels and pen grays      │
  │                   │ ▸ Safe zones  (6 boxes, px)      │
  │                   │ ▸ About: verified, date, notes   │
  │ [New…] [Duplicate]│                                  │
  │ [Import…][Export…]│ [Reset to built-in]  [Cancel][Save]│
  └───────────────────────────────────────────────────┘
  ```
- **Q6 Ids and clashes.** chris, word for word, in place of the options offered ("Mine until chosen, remembered" was
  recommended, overruled): "let's make the IDs a guid so that collisions don't occur, but you are given the choice to
  merge or rename the incoming profile that has a same name". So a profile a user makes (New…, Duplicate, Rename on
  import) gets a GUID id; the built-ins keep their fixed ids (`supernote.a5x2`, `supernote.a6x2`, which every saved file
  and the frozen fixtures name). An incoming profile (carried in an opened `.quad`, or an imported `.ron`) with the same
  name as one of the user's offers **Merge** or **Rename**. Q9 says what Merge does.
- **Q7 New and import.** New… asks for a name and a profile to start from and copies all its values; size and ppi are
  editable; the new profile is marked Unverified. An import whose id the computer already has asks: **Replace mine**
  or **Keep both** (the import gets a new GUID). chris: "New copies one; clash asks (Recommended)".
- **Q8 A damaged profile file.** The app never stops because of one. The broken file is left untouched and not loaded;
  the Profiles window lists it with its error and line; a note appears once at start. A broken override of the Manta
  or Nomad falls back to the built-in values, marked. chris: "Skip it, say so (Recommended)".
- **Q9 Merge.** **Merge = the incoming values update mine**: the user's same-named profile takes the incoming values
  (for an edited or plain Manta or Nomad, they become the local override; Reset to built-in still brings the shipped
  values back). **Rename**: the incoming profile joins the user's profiles under a new name. The same choice is offered
  when the same GUID arrives with different values. chris: "Merge = theirs updates mine" (the recommendation, "Merge =
  use mine; asked on open", overruled).
- **Q10 Anything else.** chris, word for word: "I would like to add new built-in profiles for "print" templates. These
  built-ins should include common print paper sizes (letter, legal, tabloid, A4, A5, etc...)". This pulls the spec's
  v1.1 paper profiles (§3.3.1, §16) into rung 13; the lead said so before Q11, and Q11–Q14 placed them.
- **Q11 Split.** **13a the profile system** (Q1–Q9), **13b paper profiles** (Q10, Q12–Q14), a look each, **one
  `v0.13.0`** when both are approved, as rung 12 did. chris: "13a profiles, 13b paper; one v0.13.0 (Recommended)".
- **Q12 Paper sizes and PPI.** The spec's list plus Tabloid and A3, eleven sizes: ISO A3 (297 × 420 mm), ISO A4
  (210 × 297), ISO A5 (148 × 210), ISO A6 (105 × 148), ISO B5 (176 × 250), JIS B5 (182 × 257), US Letter (215.9 ×
  279.4), US Legal (215.9 × 355.6), Tabloid (279.4 × 431.8, 11 × 17 in), Half Letter (139.7 × 215.9), Executive (184.2
  × 266.7). **PPI per page**: each page picks 150, 200, 300 (default) or 600 in the Page tab beside the paper's tick.
  Other sizes come from New… (Q7). chris: "Spec list + Tabloid + A3; PPI per page (Recommended)".
- **Q13 Paper line roles.** The Nomad's values by physical size: each role keeps the Nomad's thickness in millimetres
  and its gray (fill roles too), rounded to whole pixels at the page's PPI, at least 1; marked Unverified; editable in
  the profile editor. chris: "The Nomad's, by physical size (Recommended)".
- **Q14 Landscape paper.** A landscape paper page exports a wide PNG, not turned (A4 landscape at 300 PPI is 3508 ×
  2480); the quarter turn stays a device-only rule. chris: "A wide PNG, not turned (Recommended)".

## Owed

Empty. Every question an answer opened was asked (Q9 from Q6; Q11–Q14 from Q10).

## Defaulted (the lead's, each open to chris's veto)

- **GUID form.** A user profile's id is a UUID v4 string (the `uuid` crate, already a dependency). It is never typed by
  hand; the About section shows it small for bug reports. veto: hide it everywhere, or a readable prefix such as
  `user.` first.
- **Short name.** Every profile has a short name (the export file name's `{profile}` part, `manta`, `a4`), lowercase
  letters, digits and hyphens. A user's short names are unique on this computer: New…, Duplicate and Rename suggest one
  (`nomad-2`) and refuse a taken one. veto: allow repeats and number the exports instead.
- **"Same name".** Two profiles have the same name when their display names match ignoring case and outer spaces.
  veto: match on short name instead.
- **When Merge/Rename is asked.** In a dialog as the file opens or the `.ron` is imported, one row per clashing
  profile, with a third choice **Not now** (the page shows the file's copy for this session and asks again next time).
  veto: a bar like rung 12's instead of a dialog.
- **Where they live.** `<config folder>/profiles/<id>.ron` (on Linux `~/.config/quadrule/profiles/`); an override of
  the Nomad is `supernote.a6x2.ron` there. Files are written whole to a temporary name and renamed, so a crash never
  leaves half a file. veto: another folder.
- **One registry.** One lookup for the app: built-ins, then overrides, then the user's own, then the open document's
  carried copies, loaded at start and refreshed when the Profiles window saves. `builtin`/`builtin_for` keep returning
  the shipped values, so every test and golden stays independent of the config folder; tests that need user profiles
  use a temporary folder. The lead or builder checks every production call site listed in the header moves to the new
  lookup. veto: none expected; a design note.
- **Editing and undo.** Profile edits are not document edits: they are not on the page's undo list. Cancel throws away
  everything since the window opened or last saved; Save writes the file. Reset to built-in and Delete ask first; both
  are one-way (no copy is kept). veto: keep a `.bak` copy beside every replaced or deleted file.
- **Verified.** Editing a calibrated value does not change the verified mark; the About section has the tick, which
  the user sets. New… and Duplicate start Unverified (Q7). veto: any calibrated edit clears it.
- **Limits.** The form, the import and the carried-copy reader accept only: width and height 16–16,384 px, ppi
  50–1,200, grays 0–255, role widths 1–32 px, safe zones inside the screen, at most 64 zones, names up to 80
  characters. A profile outside them counts as damaged (Q8), so a hostile file cannot ask for a huge page. veto: other
  limits.
- **Delete.** Only the user's own profiles; a page that uses a deleted one still opens from its carried copy (Q4).
  veto: refuse to delete a profile open pages use.
- **Carrying in the file.** `profiles/<id>.ron` entries in the `.quad` (the same text Export… writes), listed in the
  manifest. Format 0.14.0 through the migration chain with a `v0.13/` fixture; 13b's paper PPI field goes into the same
  0.14.0 (no release between the parts), as an optional field with a default so files saved by a 13a build still open.
  veto: CBOR entries instead of RON text.
- **The `.quad` reader** treats a carried profile like a damaged one when it breaks the limits: the page opens with that
  profile missing and a message, never a crash. The fuzz target covers files with carried profiles, and a second target
  fuzzes `from_ron`. veto: none expected.
- **ADR.** The lead drafts **ADR-016, user profiles** (GUID ids, overrides, carrying, Merge/Rename, limits), status
  proposed, for chris to ratify at the 13a look; the spec gets a dated note that paper sizes moved into v1 (Q10), and
  the board's rung 13 row says so. veto: no ADR, the record is enough.
- **13a goldens.** 13a draws nothing new, so no new golden lines; tests prove an override changes the export and Reset
  restores the shipped bytes exactly.
- **Paper ids and names.** `paper.a3`, `paper.a4`, … `paper.letter`, `paper.legal`, `paper.tabloid`,
  `paper.half-letter`, `paper.executive`, `paper.b5-iso`, `paper.b5-jis`; short names `a4`, `letter`, …; display names
  as Q12 lists them ("ISO A4", "US Letter", "Tabloid"). veto: other names.
- **Paper margins.** 10 mm on every side for every paper size. veto: 0.5 in for the US sizes.
- **Paper in the UI.** New… and the Page tab list Paper as its own group after the devices; a paper size can be primary
  or secondary. The Page tab's PPI choice (150/200/300/600) shows beside a ticked paper. Paper pixel size is the size in
  mm × PPI ÷ 25.4, rounded to whole pixels (A4 at 300 → 2480 × 3508, as §3.3.1's table says). veto: other placement.
- **Paper lint.** L04, L05 and L06 are skipped on paper (spec §3.3.1), and any other rule that reads gray levels or ink
  values; the rest run as on devices. veto: none expected.
- **13b goldens.** New golden lines for ISO A4 portrait at 300 PPI, US Letter landscape at 300 (wide), and ISO A5 at
  600; the 26 older lines unchanged. veto: other sizes.
- **Export.** PNG only, as now; PDF for print is not in the spec and not in this rung. veto: ask for PDF (a new rung's
  worth of work).

## Kickoff

This session leads (step 6: the bbq used little of the lead's context; run here). Opus 5.5 at high effort leads;
Sonnet 5.5 at high effort builds 13a on `main`, committing and pushing each finished step, reading CI for its own
commits, never waiting with `pgrep -f` on a pattern in its own command. Keep the 26 golden lines and fixtures v0.4–v0.12
frozen (stop and ask chris if any would change). Measure as before (Pi, release build, both canvases, strip on, 90th
percentile, ADR-009). Pay the debt in code the rung touches. Show chris the window and stop for his look after 13a;
then 13b, its look, and one `v0.13.0`.

## Amendments

- **2026-11-17, the 13a look.** chris, word for word, answering "Approve 13a, or what to change", "Ratify ADR-016, or
  not yet" and "Your meter reading now": "1. approved 2. ratified 3. 36%". 13a is approved as built (`797cc9d` to
  `10a249b`), with the builder's own choices listed in `docs/STATUS-log.md` (2026-11-17, rung 13a) standing, among
  them: each row of the clash dialog starts on Not now; Keep both keeps the incoming name; Save keeps the window open;
  saving a built-in's shipped values deletes its override. ADR-016 is accepted. Meter 36 % (logged), so 13a spent
  4 points of the 10; 13b starts under the same share.
