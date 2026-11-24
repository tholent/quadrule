# The Profiles window and the notes about profiles (rung 13a, docs/bbq/2026-11-17-rung-13-profile-system.md).

profiles-name-edited = { $name } · Edited

notice-profile-left-out = Left out of this page, because this computer does not have them and the file holds no good copy: { $profiles }.
notice-profile-carried-damaged =
    { $count ->
        [one] A profile saved in this file could not be read, so it was skipped.
       *[other] { $count } profiles saved in this file could not be read, so they were skipped.
    }
notice-profile-failed = The profile could not be changed: { $error }

profiles-bar-name = { $name }
profiles-bar-sep = {", "}
profiles-bar-missing = { $count ->
        [one] Uses a profile you don't have: { $names }.
       *[other] Uses profiles you don't have: { $names }.
    }
profiles-bar-add = Add to my profiles
profiles-bar-deferred = { $count ->
        [one] This page keeps its own version of: { $names }.
       *[other] This page keeps its own versions of: { $names }.
    }
profiles-bar-decide = Merge or rename…
profiles-bar-close-tip = Close this bar until the page is opened again

profiles-clash-title = Profiles in this file
profiles-clash-intro = This file carries profiles that clash with ones on this computer. Choose what to do with each.
profiles-clash-row-id = “{ $name }” has other values here than in the file.
profiles-clash-row-name = “{ $name }” has the same name as your “{ $mine }”.
profiles-clash-merge = Merge: the file's values update “{ $mine }”
profiles-clash-rename = Rename the file's copy:
profiles-clash-rename-field = New name for the file's copy
profiles-clash-notnow = Not now: show the file's copy this time
profiles-clash-new-name = { $name } (from file)
profiles-clash-apply = Apply
profiles-clash-later = Not now for all

# The Profiles window (rung 13a, Q5).
profiles-open = Profiles…
profiles-open-tip = Open the Profiles window: see and change the values each device is drawn with
profiles-title = Profiles
profiles-group-builtin = BUILT IN
profiles-group-paper = Paper
profiles-group-mine = MINE
profiles-none-mine = None yet. New… makes one.
profiles-group-damaged = DAMAGED FILES (skipped)
profiles-damaged = { $file }: { $error }
profiles-damaged-line = { $file }, line { $line }: { $error }
profiles-damaged-delete = Delete
profiles-damaged-delete-tip = Delete the file { $file } from your profiles folder. This cannot be undone.
profiles-damaged-deleted = Deleted { $file }.
profiles-damaged-title = Delete damaged file
profiles-damaged-body = Delete the file { $name } from your profiles folder? It is not read now, so no page loses anything by it. This cannot be undone.
profiles-name-unverified = { $name } · Unverified
profiles-screen = Screen
profiles-name = Name
profiles-size = Size
profiles-size-locked = { $w } × { $h } px (locked)
profiles-size-paper-locked = { $w } × { $h } mm, { $pw } × { $ph } px at 300 PPI (locked)
profiles-size-paper-px = { $pw } × { $ph } px at 300 PPI
profiles-size-mm = Size in millimetres
profiles-ppi-per-page = Chosen per page; each page picks 150, 200, 300 or 600
profiles-paper-roles-note = Line widths are for 300 PPI; a page scales them with its PPI, to whole pixels, at least 1.
profiles-paper-list-tip = { $w } × { $h } mm, { $pw } × { $ph } px at 300 PPI
profiles-ppi = PPI
profiles-ppi-locked = { $ppi } (locked)
profiles-short-name = Short name
profiles-short-name-tip = Lowercase letters, digits and hyphens; it names exported files
profiles-margins = Margins
profiles-margin-top = Top margin, millimetres
profiles-margin-right = Right margin, millimetres
profiles-margin-bottom = Bottom margin, millimetres
profiles-margin-left = Left margin, millimetres
profiles-orientations = Held
profiles-line-roles = Line roles
profiles-calibrated = Calibrated on the device. Change them only when you have measured.
profiles-width-px = Width, px
profiles-gray = Gray (0 black, 255 white)
profiles-fill-roles = Fill roles
profiles-lint = Lint checks
profiles-lint-note = What the lint checks judge this device against. Leave a value empty to skip that check on this device.
profiles-lint-skipped = skipped
profiles-lint-set = Set
profiles-lint-clear = Clear
profiles-lint-clear-tip = Leave this empty: the check that reads it is skipped on this device
profiles-lint-set-tip = Give this a value so the check that reads it runs
profiles-lint-line-1px = Lightest gray, 1 px line
profiles-lint-line-2px = Lightest gray, line 2 px or wider
profiles-lint-dot-2px = Lightest gray, dot 2 px
profiles-lint-dot-3px = Lightest gray, dot 3 px
profiles-lint-dot-4px = Lightest gray, dot 4 px or larger
profiles-lint-fill = Lightest gray, fill
profiles-lint-text-gray = Lightest gray, text
profiles-lint-text-pt = Smallest text, pt
profiles-lint-step-mm = Smallest handwriting step, mm
profiles-lint-dark-gray = Dark fill: this gray or darker
profiles-lint-dark-share = Dark fills: page share, %
profiles-lint-thin-px = Thin line: this many px or thinner
profiles-grays = Gray levels and pen grays
profiles-gray-levels-label = Gray levels the screen renders cleanly
profiles-pen-grays-label = Grays the pens write
profiles-grays-bad = Use whole numbers from 0 to 255, separated by commas.
profiles-safe-zones =
    { $count ->
        [one] Safe zones (1 box, px)
       *[other] Safe zones ({ $count } boxes, px)
    }
profiles-zone-name = Name
profiles-zone-orientation = Held
profiles-zone-w = Width
profiles-zone-h = Height
profiles-zone-movable = Movable
profiles-zone-remove = Remove the zone { $name }
profiles-zone-add = Add a zone
profiles-zone-new = zone
profiles-about = About
profiles-verified = Verified on a device
profiles-date = Date
profiles-notes = Notes
profiles-id = id { $id }
profiles-new = New…
profiles-duplicate = Duplicate
profiles-import = Import…
profiles-export = Export…
profiles-reset = Reset to built-in
profiles-delete = Delete
profiles-cancel = Cancel
profiles-save = Save
profiles-new-title = New profile
profiles-new-default-name = My profile
profiles-start-from = Start from
profiles-create = Create
profiles-copy-name = { $name } copy
profiles-reset-title = Reset to built-in
profiles-reset-body = This puts back the values { $name } shipped with and deletes your edits. It cannot be undone.
profiles-reset-yes = Reset
profiles-delete-title = Delete profile
profiles-delete-body = Delete { $name }? Pages that use it still open, because each file carries its own copy. This cannot be undone.
profiles-delete-yes = Delete
profiles-keep = Keep
profiles-import-title = Import profile
profiles-import-row-id = “{ $name }” has the same id as your “{ $mine }”, with other values.
profiles-import-row-name = “{ $name }” has the same name as your “{ $mine }”.
profiles-import-replace = Replace mine
profiles-import-both = Keep both
profiles-import-rename = Rename and add

profiles-saved = Saved.
profiles-created = Made “{ $name }”.
profiles-deleted = Deleted.
profiles-was-reset = Back to the built-in values.
profiles-imported = Imported “{ $name }”.
profiles-import-same = This computer already has that profile.
profiles-import-bad = That file is not a usable profile: { $error }
profiles-exported = Wrote { $path }.
profiles-error = { $error }

notice-profiles-damaged =
    { $count ->
        [one] A profile file in the config folder is damaged and was skipped. The Profiles window lists it.
       *[other] { $count } profile files in the config folder are damaged and were skipped. The Profiles window lists them.
    }

# The Page tab's button (the toolbar's has the same shown words; rung 17 bbq, Q5).
profiles-open-devices = Profiles… for the devices of this page
