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
app-every-device = On { $device }, position, size, font size and the eye change this device only. Everything else changes the design for every device.
app-selected-count = { $count ->
    [one] 1 object selected
   *[other] { $count } objects selected
}
app-selection-locked = Locked. Unlock it in the layers panel to edit it.
app-field-invalid = Not a length. Use mm, pt, px, in, % or u, such as 5mm or 2u.

app-page-title = Title
app-page-units = Units
app-page-background = Background gray
# Orientation (rung 8b).
app-page-orientation = Orientation
app-orientation-portrait = Portrait
app-orientation-landscape = Landscape
app-orientation-landscape-tip = A landscape page is saved turned a quarter turn clockwise, as the tablet's portrait page. Read it with the tablet's right edge up.
app-page-margins = Margins
app-margin-top = Top
app-margin-right = Right
app-margin-bottom = Bottom
app-margin-left = Left
app-link-margins = Same on all sides
app-rhythm = Rhythm
app-rhythm-type = Type
app-rhythm-type-rectangular = Rectangular
app-rhythm-type-isometric = Isometric
app-rhythm-side = Side length
app-rhythm-major = Major every
app-major-off = Off
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
app-rotation = Rotation
app-opacity = Opacity
app-rotate-handle = Rotate handle

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
app-pattern-isometric = Isometric
app-pattern-hex = Hex
app-pattern-cornell = Cornell
app-pattern-staff = Music staff
app-pattern-settings = Pattern settings
app-iso-style = Style
app-iso-lines = Lines
app-iso-dots = Dots
app-hex-orientation = Orientation
app-hex-flat = Flat top
app-hex-pointy = Pointy top
app-hex-height = Height
app-cornell-cue = Cue column
app-cornell-summary = Summary band
app-staff-gap = Line gap
app-staff-pitch = Staff every
app-staff-grand = Grand staff
app-pattern-bound = Bound to rhythm
app-pattern-across = Spacing across
app-pattern-down = Spacing down
app-pattern-major = Major lines
app-major-follow = Follow the rhythm
app-major-own = Every N
app-pattern-major-n = Major every
app-pattern-major-role = Major role
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
new-orientation = Orientation
new-create = Create
new-rhythm-handwriting = Handwriting, 6 mm
new-rhythm-narrow = Narrow ruled, 1/4 in (6.35 mm)
new-rhythm-college = College ruled, 9/32 in (7.14 mm)
new-rhythm-wide = Wide ruled, 11/32 in (8.73 mm)
new-rhythm-grid-5mm = 5 mm grid
new-rhythm-grid-quarter-inch = 1/4 in grid (6.35 mm)
new-rhythm-isometric-5mm = Isometric, 5 mm side

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

# Groups (rung 7a).
new-name-group = Group
notice-turned-group = The group "{ $name }" is turned, and text boxes and pattern regions do not turn, so they cannot go into it.
notice-group-mixed = Those objects are in different groups. Pick objects from one place to group them.
notice-group-few = Pick two or more objects to group them.
notice-ungroup-none = Pick a group to ungroup it.
notice-turned-move = A turned group has a space of its own. Use Group and Ungroup to move objects across its edge.
app-group = Group
app-ungroup = Ungroup
app-fold-item = Show what is inside { $name }
app-rotation-note = A group with a text box or a pattern region in it cannot turn.

# Align and distribute, and mixed values (rung 7b).
app-align-heading = Align and distribute
app-align-to = Align to
app-align-to-selection = Selection
app-align-to-page = Page
app-align-to-margins = Margins
app-arrange-align-left = Align left edges
app-arrange-align-centre = Align horizontal centres
app-arrange-align-right = Align right edges
app-arrange-align-top = Align top edges
app-arrange-align-middle = Align vertical middles
app-arrange-align-bottom = Align bottom edges
app-distribute-h-spacing = Distribute horizontally by spacing
app-distribute-h-centres = Distribute horizontally by centres
app-distribute-v-spacing = Distribute vertically by spacing
app-distribute-v-centres = Distribute vertically by centres
app-mixed = mixed

# Several devices (rung 8a, docs/bbq/2026-11-12-rung-8-several-devices.md).
notice-shared-edit = That change applies to every device, not only { $device }.
app-style-shared = Style changes apply to every device.
app-tab-overrides = Overrides
app-override-badge = Changed on { $device }
app-overrides-intro = What differs on { $device } from the design.
app-overrides-none = Nothing differs on { $device }.
app-overrides-many = With several objects picked, Reset all applies to every one of them.
app-override-position = Position
app-override-size = Size
app-override-font = Font size
app-override-eye = Shown
app-override-here = Here: { $value }
app-override-linked = In the design: { $value }
app-override-yes = shown
app-override-no = hidden
app-override-reset = Reset
app-override-reset-name = Reset { $what } on { $device }
app-override-reset-all = Reset all on this device
app-override-reset-all-name = Reset all changes on { $device }

# The Page tab's devices.
app-page-devices = Devices
app-device-target = Target { $device }
app-device-primary = primary
app-device-make-primary = Make primary
app-device-make-primary-name = Make { $device } the primary device
app-device-primary-tip = Designs are made on the primary device; the others follow it.
app-device-take-out = Take { $device } out of this page?
app-device-take-out-lose = { $count ->
    [one] One object has a change on it that will be lost. Undo brings it back.
   *[other] { $count } objects have changes on it that will be lost. Undo brings them back.
}
app-device-take-out-yes = Take out
app-device-take-out-no = Keep it

# The previews strip (Q5).
app-strip = Devices
app-strip-fold = Hide the device previews
app-strip-unfold = Show the device previews
app-strip-device = Show { $device }
app-strip-count = { $count ->
    [one] 1 change
   *[other] { $count } changes
}
app-strip-on-screen = on screen

# The repeat tool (rung 10).
new-name-repeat = Repeat
app-repeat = Repeat
app-repeat-hint = Draw the picked objects again in a row, a column or a grid. Edit the first one and every copy follows.
notice-repeat-none = Pick something to repeat.
notice-repeat-source = The first object inside "{ $name }" is the one that is repeated, so it stays inside. Use Expand to turn the copies into plain objects, or delete the repeat.
notice-expand-none = Pick a repeat to expand it.
notice-expanded = Expanded into { $count ->
    [one] 1 object
   *[other] { $count } objects
}.
notice-expanded-differs = Expanded into { $count } objects, as the { $primary } drew them. The { $device } drew { $drawn } before; the copies keep the { $primary } count.
app-status-repeat-capped = A repeat draws at most { $max } copies; the rest are left out.
app-tab-repeat = Repeat
app-repeat-layout = Layout
app-repeat-across = Across
app-repeat-down = Down
app-repeat-grid = Grid
app-repeat-how-many = Number of copies
app-repeat-count-fixed = A set count
app-repeat-count-fill = Fill the box
app-repeat-copies = Copies
app-repeat-columns = Columns
app-repeat-rows = Rows
app-repeat-fill-note = As many as fit in the box. Drag the box's handles to add or take away copies.
app-repeat-spacing-by = Space by
app-repeat-pitch = Pitch, centre to centre
app-repeat-gap = Gap, edge to edge
app-repeat-step-across = Step across
app-repeat-step-down = Step down
app-repeat-step-hint = Auto, or a length such as 2u or 12 mm
app-repeat-auto = Auto
app-repeat-leftover = Spare room
app-repeat-start = From the start
app-repeat-centre = Split to both sides
app-repeat-rules = Alternation
app-repeat-rules-none = No rules. Every copy is drawn like the first.
app-repeat-rule-add = Add a rule
app-repeat-rule-remove = Remove this rule
app-repeat-rule-up = Move this rule up
app-repeat-rule-down = Move this rule down
app-repeat-rule-counts = Counts
app-repeat-rule-copies = Copies
app-repeat-rule-columns = Columns
app-repeat-rule-rows = Rows
app-repeat-rule-cells = Cells, left to right then down
app-repeat-rule-when = Hits
app-repeat-rule-every = Every Nth
app-repeat-rule-list = These numbers
app-repeat-rule-n = N
app-repeat-rule-from = From
app-repeat-rule-numbers = Numbers
app-repeat-rule-numbers-hint = For example 3, 7
app-repeat-rule-does = Does
app-repeat-rule-skip = Skip the copy
app-repeat-rule-line = Set the line role
app-repeat-rule-fill = Set the fill role
app-repeat-rule-role = Role
app-repeat-rules-later = Rules apply in order; a later one wins.
app-repeat-expand = Expand
app-repeat-expand-hint = Turn the copies into plain objects in one group. Nothing moves on the primary device.
app-repeat-drawn = { $drawn } of { $total } copies are drawn on this device.
app-repeat-capped-note = The cap of { $max } copies cut it short.
app-repeat-locked = This repeat is locked.
