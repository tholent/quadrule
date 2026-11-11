# Copyright 2026 Chris Wells. Licensed under the Apache License, Version 2.0.
#
# The window: the viewer (rung 2, docs/bbq/2026-11-09-rung-2-viewer.md) and the first editor
# (rung 3, docs/bbq/2026-11-10-rung-3-first-editor.md).

app-title = quadrule
app-window-title = { $name } — quadrule
app-window-title-dirty = ● { $name } — quadrule
app-sample-name = Sample page
app-untitled-name = Untitled

# Toolbar.
app-new = New…
app-new-tip = New page (Ctrl+N)
app-open = Open…
app-save = Save
app-save-as = Save As…
app-undo = Undo
app-redo = Redo
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
app-canvas = Canvas
app-canvas-export = Export pixels
app-canvas-chip = Graphics chip
app-chip-ms = Graphics chip: { $ms } ms a frame
app-chip-slow = Graphics chip too slow ({ $ms } ms a frame): drawing with vello_cpu ({ $cpu } ms)
app-chip-none = Graphics chip canvas could not start ({ $error }): drawing with vello_cpu ({ $cpu } ms)

# Empty start screen (rung 2 bbq, Q7).
app-empty-title = Nothing open
app-open-sample = Open sample page
app-empty-drop = or drag a .quad file onto this window

# Lint panel.
app-lint-heading = Lint ({ $count })
app-lint-toolbars = Checked for the toolbar docked left, right or top, and the footers.
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
notice-saved = Saved { $path }.
notice-save-failed = Could not save: { $error }
notice-locked = The selection is locked.
notice-object-locked = "{ $name }" is locked.
notice-layer-locked = The layer "{ $name }" is locked.
notice-edit-failed = Could not change that: { $error }
notice-recovery-failed = Could not write the recovery file: { $error }

# Layers panel (rung 3 bbq Q9).
app-layers = Layers
app-move-up = Move up
app-move-down = Move down
app-show-item = Show { $name }
app-lock-item = Lock { $name }
app-rename = Name

# Inspector (rung 3 bbq Q8).
app-tab-page = Page
app-tab-arrange = Arrange
app-tab-style = Style
app-tab-text = Text
app-tab-constraints = Constraints
app-every-device = Edits change the design for every device; { $device } shows how it adapts.
app-selected-count = { $count ->
    [one] 1 object selected
   *[other] { $count } objects selected
}
app-selection-locked = Locked. Unlock it in the layers panel to edit it.
app-field-invalid = Not a length. Use mm, pt, px, in, % or u, such as 5mm or 2u.

app-page-title = Title
app-page-units = Units
app-page-background = Background gray
app-page-margins = Margins
app-margin-top = Top
app-margin-right = Right
app-margin-bottom = Bottom
app-margin-left = Left
app-link-margins = Same on all sides
app-rhythm = Rhythm
app-rhythm-across = Across
app-rhythm-down = Down
app-link-rhythm = Same across and down
app-rhythm-origin = Origin
app-origin-margin = Margin corner
app-origin-page = Page corner
app-origin-centered = Centered
app-rhythm-resolution = Resolution
app-res-pixel = Pixel-locked
app-res-physical = Physical-locked
app-rhythm-on-device = { $px } px on { $device }
app-rhythm-subdivisions = Subdivisions
app-subdivisions-off = Off
app-subdivisions-n = { $n } a unit

app-snap = Snap
app-snap-pixel = Pixel grid
app-snap-rhythm = Rhythm
app-snap-margins = Margins and safe zones
app-snap-points = Point snap
app-snap-smart = Smart guides
app-snap-equal = Equal spacing
app-gap-size = { $px } px
app-rhythm-overlay = Rhythm overlay
app-snapping = Snapping and guides
app-drag-clock = Drag: { $mean } ms a frame (90th percentile { $p90 } ms, worst { $worst } ms, { $frames } frames)
app-drop-here = Drop here

app-x = X
app-y = Y
app-width = Width
app-height = Height

app-style-kind = Look
app-style-role = Role
app-style-custom = Custom
app-role-whisper = Whisper
app-role-guide = Guide
app-role-structure = Structure
app-role-emphasis = Emphasis
app-role-dotgrid = Dot grid
app-role-text = Text
app-role-label = Label
app-stroke-width = Width
app-stroke-gray = Gray
app-pattern = Pattern
app-pattern-lines = Lines
app-pattern-dots = Dots
app-pattern-grid = Grid
app-style-stroke = Stroke
app-stroke-none = None
app-fill = Fill
app-fill-none = None
app-fill-shade = Shade
app-fill-block = Block
app-fill-custom = Custom gray
app-fill-gray = Fill gray
app-dash = Dash
app-dash-solid = Solid
app-dash-dotted = Dotted
app-dash-short = Short dash
app-dash-dash = Dash
app-dash-long = Long dash
app-dash-custom = Custom
app-dash-length = Dash length
app-gap-length = Gap length
app-cap = Cap
app-cap-butt = Butt
app-cap-round = Round
app-cap-square = Square
app-shape = Shape
app-corner-radius = Corner radius
app-sides = Sides
app-star = Star
app-star-inner = Inner radius

app-text-content = Text
app-font = Font
app-weight = Weight
app-size = Size
app-line-height = Line height
app-letter-spacing = Letter spacing (em)
app-align = Align
app-align-left = Left
app-align-center = Center
app-align-right = Right
app-align-justify = Justify

app-constraint-h = Horizontal
app-constraint-v = Vertical
app-c-left = Left
app-c-right = Right
app-c-left-right = Left and right
app-c-top = Top
app-c-bottom = Bottom
app-c-top-bottom = Top and bottom
app-c-center = Center
app-c-scale = Scale

# Unsaved changes (rung 3 bbq, Defaulted "Save").
app-unsaved-title = Save changes?
app-unsaved-body = “{ $name }” has changes that are not saved.
app-unsaved-save = Save
app-unsaved-discard = Don’t save
app-cancel = Cancel

# Recovery (napkin §5.6; rung 3 bbq, Defaulted "Autosave").
app-recover-title = Recover unsaved work?
app-recover-body = quadrule closed before “{ $name }” was saved. Restore the changes it had?
app-recover-restore = Restore
app-recover-discard = Discard

# New dialog (rung 5 bbq, Defaulted "New dialog fields").
new-heading = New page
new-title = Title
new-default-title = Untitled
new-rhythm = Rhythm
new-create = Create
new-rhythm-handwriting = Handwriting, 6 mm
new-rhythm-narrow = Narrow ruled, 1/4 in (6.35 mm)
new-rhythm-college = College ruled, 9/32 in (7.14 mm)
new-rhythm-wide = Wide ruled, 11/32 in (8.73 mm)
new-rhythm-grid-5mm = 5 mm grid
new-rhythm-grid-quarter-inch = 1/4 in grid (6.35 mm)

# Tools bar (rung 5 bbq, Defaulted "The tools bar").
tool-select = Select tool
tool-select-tip = Select (V)
tool-line = Line tool
tool-line-tip = Line (L)
tool-text = Text box tool
tool-text-tip = Text box (T)
tool-pattern = Pattern region tool
tool-pattern-tip = Pattern region (P)
tool-rect = Rectangle tool
tool-rect-tip = Rectangle (R). Shift draws a square.
tool-ellipse = Ellipse tool
tool-ellipse-tip = Ellipse (E). Shift draws a circle.
tool-polygon = Polygon tool
tool-polygon-tip = Polygon or star (Y). Shift draws a regular polygon.
tools-keep-hint = Double-click to keep it on for several objects.

# What a drawn object is called, and what a new text box says.
new-name-line = Line
new-name-text = Text
new-name-pattern = Pattern
new-name-rect = Rectangle
new-name-ellipse = Ellipse
new-name-polygon = Polygon
new-text-content = Text
