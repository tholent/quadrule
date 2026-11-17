---
status: proposed
status-note: "drafted 2026-11-17 by the rung 13 lead from the rung 13 bbq record (Defaulted \"ADR\"); awaiting ratification at the 13a look"
owner: chris
kind: design
built: none
proof: ""
owed: "all of it: rung 13a builds the registry, overrides, carrying and Merge/Rename; rung 13b the paper built-ins"
supersedes: nothing
amends: "napkin §12.1 (the `.quad` container gains `profiles/<id>.ron`)"
consulted: the rung 13 lead (Opus 5.5)
---
# ADR-016 — Users make and edit device profiles: their own get GUID ids, an edited Manta or Nomad is a local override with Reset, a page carries copies of the profiles it needs, and a same-named incoming profile is merged into yours or renamed

## Decision

1. **Ids.** The built-in profiles keep their fixed ids (`supernote.a5x2`, `supernote.a6x2`; from rung 13b, `paper.a4`
   and the other paper sizes), because every saved file names them. A profile a user makes (New…, Duplicate, or
   Rename on import) gets a **GUID** (a UUID v4), so two people's profiles never share an id (rung 13 record Q6).
2. **Where they live.** User profiles and overrides are RON files in `<config folder>/profiles/<id>.ron`, written
   whole to a temporary name and renamed. The app reads every profile through **one registry**: built-ins, then
   overrides, then the user's own, then the open page's carried copies. `builtin` and `builtin_for` keep returning the
   shipped values only, so tests and golden images never depend on a user's folder.
3. **Editing a built-in** makes a **local override** that applies to every page on this computer, existing files
   included, marked **Edited** wherever the device's name shows; **Reset to built-in** deletes it. On an override the
   id, pixel size, ppi and short name are locked; everything else may change (Q2, Q3).
4. **A page carries its profiles.** Saving puts a copy of every profile the page uses that is not an unedited
   built-in into the `.quad` as `profiles/<id>.ron`, listed in the manifest (file format 0.14.0). A page whose profile
   this computer lacks opens from the carried copy, with a bar offering **Add to my profiles** (Q4).
5. **Clashes.** An incoming profile (carried in an opened page, or an imported `.ron`) with the same name as one of
   the user's, or the same id with different values, offers **Merge**, **Rename** or **Not now**. **Merge: the
   incoming values update the user's profile** (for a built-in id they become the override; Reset still restores the
   shipped values). **Rename:** the incoming profile joins the user's under a new name (Q6, Q9).
6. **Damage never stops the app.** A profile file that does not parse, misses a field or breaks the limits (screen
   16–16,384 px, ppi 50–1,200, role widths 1–32 px, zones inside the screen, at most 64 zones) is skipped and listed
   with its error; a broken override falls back to the built-in (Q8, Defaulted "Limits").

## Context

- The spec says profiles are data that "users can edit, duplicate, import and share as small files", stored "in the
  user config directory for custom or edited profiles", and that "a user-edited built-in becomes a local override that
  can be reset to default" (`napkins/quadrule-spec.md` §3.1, §3.2, §8.2), and that custom profiles "export as
  standalone `.ron` files" (§12.3).
- Until rung 13, a `.quad` naming a profile the build lacked refused to open (`crates/ui/src/viewer.rs`, before rung
  13a). Rung 12 answered the same question for fonts by carrying them in the file (`assets/fonts/`); the rung 13 prompt
  asked whether profiles should follow.
- Every ruling here is chris's, word for word, in `docs/bbq/2026-11-17-rung-13-profile-system.md` (Q2–Q9) or one of
  its Defaulted items open to his veto.

Drivers:
- A page should open and export the same on any computer, as rung 12 made true for fonts.
- Calibration is per device and taste is chris's: users who tune a role on their own tablet need that tuning to reach
  every page, and a way back to the shipped values.
- Names clash between people; ids must not.

## Alternatives

### GUID ids with Merge or Rename on a same-named profile (chosen, chris's Q6 ruling)
- Good: no two people's profiles collide; the user decides what a same-named profile means.
- Bad: an edited Manta or Nomad still shares its id with everyone else's, so it always meets the clash dialog when
  someone else's edited copy arrives.

### Readable ids (`kindle.scribe`) with a "Use the file's / Use mine" bar (the Q4 option, replaced by Q6)
- Good: readable files.
- Bad: two people's `kindle.scribe` collide silently.

### Merge keeps mine (the recommendation for Q9, overruled)
- Good: opening someone's page never changes the user's own profile.
- Bad: chris chose the other way: an incoming profile is a way to receive someone's tuning.

### Don't carry profiles; name the missing one (Q4 option 3)
- Good: no file format change.
- Bad: a shared page does not open without a separate `.ron`.

## Consequences

- Easier: sharing a template made for any screen; tuning a device's roles without a code change (ADR-012 item 2 still
  holds for the shipped values).
- Harder: an override changes existing pages' exports on that computer, so the Edited mark must show wherever the
  device's name does. Merge rewrites a profile the user may rely on; Reset covers built-ins, and a user's own profile
  has no undo.
- The golden images and fixtures stay tied to the shipped values only.

Revisit when: profiles are shared through the community gallery (spec §16, v1.2 or later), or forking (v1.1) needs a
profile per fork.

## History

- 2026-11-17: drafted by the rung 13 lead from the rung 13 bbq record (Defaulted "ADR"), no rounds; awaiting chris's
  ratification at the 13a look.
