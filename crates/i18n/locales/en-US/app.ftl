# Copyright 2026 Chris Wells. Licensed under the Apache License, Version 2.0.
#
# The viewer window (rung 2, docs/bbq/2026-11-09-rung-2-viewer.md).

app-title = quadrule

# Toolbar.
app-open = Open…
app-device = Device
app-zoom-fit = Fit
app-zoom-actual = 1:1
app-zoom-in = Zoom in
app-zoom-out = Zoom out
app-zoom-real = Real size
app-lint-count = ⚠ { $count }
app-export = Export
app-export-this = Export this device…
app-export-all = Export all devices…
app-view = View
app-theme = Theme
app-theme-system = Follow the system
app-theme-light = Light
app-theme-dark = Dark
app-safe-zones = Show safe zones
app-ruler-open = Real size check…

# Empty start screen (rung 2 bbq, Q7).
app-empty-title = Nothing open
app-open-sample = Open sample page
app-empty-drop = or drag a .quad file onto this window

# Lint panel.
app-lint-heading = Lint ({ $count })
app-lint-toolbars = Checked for the toolbar docked left and docked top, and the footers.
app-lint-none = Nothing to report on this device.
app-saved-outlines = A font is missing here, so text shows as saved in the file.

# Status bar.
app-status-cursor = x { $x } px / { $xmm } mm · y { $y } px / { $ymm } mm
app-status-zoom = zoom { $percent }%

# Real size check (rung 2 bbq, Defaulted "real size").
app-ruler-title = Real size check
app-ruler-help = Hold a bank card or a ruler against the screen, and drag until the bar is exactly as long.
app-ruler-card = Bank card (85.6 mm)
app-ruler-100 = Ruler, 100 mm
app-ruler-save = Save
app-ruler-bar = Bar length

# Notices.
notice-open-failed = Could not open { $path }: { $error }
notice-exported = { $count ->
    [one] Exported 1 file
   *[other] Exported { $count } files
} to { $folder }. Copy them to the device's MyStyle folder.
notice-export-failed = Could not export: { $error }
