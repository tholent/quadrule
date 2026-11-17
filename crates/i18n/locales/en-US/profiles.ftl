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
