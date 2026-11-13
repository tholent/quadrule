# Copyright 2026 Chris Wells. Licensed under the Apache License, Version 2.0.
#
# Lint findings (napkin §8.3). Each says what is wrong and what to do, in plain words.
# Ids: lint-<rule>-<case>. $name is an object's name.

lint-l01-thin = "{ $name }" has a { $px } px stroke on this device, thinner than one pixel: it is drawn 1 px wide and may look heavier than meant. Use a role, or at least { $min_mm } mm.
lint-l02-fraction = "{ $name }" has a { $px } px stroke on this device; it is drawn { $drawn } px wide. A role, or a width of whole pixels, keeps it exact.
lint-l02-diagonal = "{ $name }" is a diagonal line, so its edges cannot sit on whole pixels and are soft.
lint-l06-fixed = "{ $name }" is under the { $zones } on this device. Move it inside the safe area, or widen the margins.
lint-l06-movable = "{ $name }" can be covered by the { $zones } where it first appears; it can be moved.
lint-l10-gone = "{ $name }" is entirely off the page on this device.
lint-l10-partly = "{ $name }" runs off the edge of the page on this device.
lint-l15-off = "{ $name }" is on the page on { $primary } but runs off it on { $device }, where its constraints put it. Change how it follows the page, or place it by hand on this device.
lint-l15-overlap = "{ $name }" overlaps "{ $other }" on { $device }, where its constraints put it; the two do not touch on { $primary }. Change how they follow the page, or place one by hand on this device.
lint-l15-narrow = "{ $name }" is squeezed to { $px } px across on { $device }, where its constraints put it. Change how it follows the page, or size it by hand on this device.
lint-l15-short = "{ $name }" is squeezed to { $px } px down on { $device }, where its constraints put it. Change how it follows the page, or size it by hand on this device.
lint-l16 = "{ $name }" starts off the rhythm ({ $axes }). Snap it to the rhythm so it lines up with the grid.
lint-l17 = "{ $name }" has its own spacing ({ $axes }) that is not a whole multiple or fraction of the rhythm on this device: { $spacing } px against a unit of { $unit } px. Bind it to the rhythm, or choose a spacing that is a whole multiple or fraction of it.
lint-l18 = The rhythm leaves { $a } px at the { $near } margin and { $b } px at the { $far }: more than half a unit ({ $unit } px) apart. Centring the rhythm origin splits the space evenly.
lint-l19 = The rhythm keeps exact millimetres, so on this device its lines are { $x } × { $y } px apart and some gaps are 1 px wider than others. Pixel-locked rhythm keeps every gap the same.

# Directions, for L16 and L18.
lint-axes-both = across and down
lint-axes-across = across
lint-axes-down = down
lint-side-left = left
lint-side-right = right
lint-side-top = top
lint-side-bottom = bottom

# Safe zones (napkin §3.5), named in L06's list. lint-zone-other shows a zone this file does not
# name yet, by its profile name.
lint-zone-footer_left = page-number footer
lint-zone-footer_right = note-name footer
lint-zone-toolbar_left = toolbar when docked left
lint-zone-toolbar_right = toolbar when docked right
lint-zone-toolbar_top = toolbar when docked top
lint-zone-toolbar_floating_default = floating toolbar
lint-zone-other = { $zone }
lint-zone-sep = {", the "}
