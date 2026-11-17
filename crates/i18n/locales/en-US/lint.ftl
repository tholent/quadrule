# Copyright 2026 Chris Wells. Licensed under the Apache License, Version 2.0.
#
# Lint findings (napkin §8.3). Each says what is wrong and what to do, in plain words.
# Ids: lint-<rule>-<case>. $name is an object's name.

lint-l01-thin = "{ $name }" has a { $px } px stroke on this device, thinner than one pixel: it is drawn 1 px wide and may look heavier than meant. Use a role, or at least { $min_mm } mm.
lint-l02-fraction = "{ $name }" has a { $px } px stroke on this device; it is drawn { $drawn } px wide. A role, or a width of whole pixels, keeps it exact.
lint-l02-turned-pattern = "{ $name }" is a pattern region with its frame or its ruling turned off a quarter turn, so its lines and dots cannot sit on whole pixels and come out softer.
lint-l03-stroke = "{ $name }" is gray { $gray } on the page. Lines lighter than { $lightest } may not show on this device. Use a darker gray, or a line role.
lint-l03-dots = "{ $name }" has dots { $px } px wide at gray { $gray } on the page. Dots that size lighter than { $lightest } may not show on this device. Use a darker gray, or make the dots larger.
lint-l03-fill = "{ $name }" has a fill of gray { $gray } on the page. Fills lighter than { $lightest } may not show on this device. Use a darker gray, or a fill role.
lint-l03-text = "{ $name }" has text of gray { $gray } on the page. Text lighter than { $lightest } may be hard to read on this device. Use a darker gray, or a text role.
lint-l05 = "{ $name }" has a { $what } of gray { $gray } on the page, which is not one of the clean grays this device draws flat. It may band or dither. Use one of the device's clean grays.
lint-l05-line = line
lint-l05-dots = dot
lint-l05-fill = fill
lint-l05-text = text
lint-l07 = "{ $name }" is set at { $pt } pt, smaller than the { $min } pt that reads on this device. Make the text larger.
lint-l09 = The rhythm steps { $step } mm down the page, less than the { $min } mm that is comfortable to write between. A larger vertical step leaves room for handwriting.
lint-l09-region = "{ $name }" is ruled { $step } mm apart, less than the { $min } mm that is comfortable to write between. Give it a larger spacing.
lint-l13 = { $count ->
        [one] One dark fill covers
       *[other] { $count } dark fills cover
    } about { $percent } % of the page, more than the { $limit } % this device handles well (gray { $gray } or darker). Large dark areas can ghost or refresh slowly. Lighten them, or make them smaller.
lint-l14 = "{ $name }" is a slanted or curved line only { $px } px wide, so most of its pixels come out as in-between grays and it looks soft. Make it thicker, or accept the softer look.
lint-l06-fixed = "{ $name }" is under the { $zones } on this device. Move it inside the safe area, or widen the margins.
lint-l06-movable = "{ $name }" can be covered by the { $zones } where it first appears; it can be moved.
lint-l08 = "{ $name }" is too small for its text on { $device }: { $count ->
        [one] one line is
       *[other] { $count } lines are
    } left out of the page. Make the box taller or wider, make the text smaller, or switch the box to Auto height.
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

# Repeats (rung 10): L06 and L10 judge every drawn copy and say how many; the pitch rules.
lint-l06-repeat-fixed = { $count } of the { $total } copies in "{ $name }" are under the { $zones } on this device. Move the repeat inside the safe area, or widen the margins.
lint-l06-repeat-movable = { $count } of the { $total } copies in "{ $name }" can be covered by the { $zones } where it first appears; it can be moved.
lint-l10-repeat-gone = { $count } of the { $total } copies in "{ $name }" are entirely off the page on this device.
lint-l10-repeat-partly = { $count } of the { $total } copies in "{ $name }" run off the edge of the page on this device.
lint-l16-repeat-pitch = The pitch of "{ $name }" ({ $axes }) is not a whole number of rhythm steps, so its copies drift off the rhythm. Use Auto, or a whole number of units.
lint-l17-repeat = "{ $name }" repeats every { $spacing } px ({ $axes }), which is not a whole multiple or fraction of the rhythm's { $unit } px on this device. Bind the pitch to the rhythm (use u), or choose one that is.

# Fonts (rung 12a). L11: a text box whose font is missing on this computer. $font is the family.
lint-l11-saved = "{ $name }" uses the font { $font }, which is not on this computer. It is drawn from the outlines saved in the file, so it looks as designed, but its text cannot be changed until you choose a font.
lint-l11-needs = "{ $name }" needs the font { $font }, which is not on this computer, and the file saved no outlines for it on this device, so it is not drawn. Install the font or choose another.

# L20: fonts installed on this computer are used. Each is marked embedded or not, and why not.
lint-l20 = This page uses fonts installed on this computer: { $fonts }. Someone without them sees other fonts unless they are embedded.
lint-l20-item = { $font } ({ $status })
lint-l20-item-fallback = { $font } (for letters the chosen font lacks; { $status })
lint-l20-sep = {"; "}
lint-l20-embedded = embedded in the file
lint-l20-off = not embedded: Embed fonts is off
lint-l20-cannot = cannot be embedded: { $reason }
lint-l20-reason-restricted = its licence does not allow it
lint-l20-reason-preview = its licence allows preview and print only
lint-l20-reason-bitmap = it holds bitmaps only
lint-l20-reason-collection = it is part of a font collection
lint-l20-reason-unknown = it carries no licence information

# Checks that did not run on a device (rung 14a): one line per device in the Lint tab.
lint-skipped = On { $device }, these checks did not run: { $items }.
lint-skipped-item = { $rule } ({ $why })
lint-skipped-sep = {", "}
lint-skip-paper = paper has no screen to judge against
lint-skip-wallpaper = a wallpaper is not written on
lint-skip-ignored = this page ignores it
lint-skip-no-grays = this device lists no clean grays
lint-skip-no-value = this device has no value for it
lint-skipped-none = No checks were skipped on { $device }.
