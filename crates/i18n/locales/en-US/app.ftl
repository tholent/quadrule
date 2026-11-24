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
app-file = File
app-new = New…
app-open = Open…
app-save = Save
app-save-as = Save As…
app-file-open-recent = Open Recent
app-file-recent-none = No recent files
app-file-recent-gone = { $name } (Not found)
app-file-save-template = Save as template…
app-file-close = Close
app-undo = Undo
app-redo = Redo
app-device = Device
new-group-paper = Paper
app-zoom-fit = Fit
app-zoom-actual = 1:1
app-zoom-in = Zoom in
app-zoom-out = Zoom out
app-zoom-real = Real size
app-lint-count = ⚠ { $count }
app-export-sheet = Export…
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

# The welcome screen (rung 16 bbq, Q2, Q4 and Q5), which replaces the empty start of rung 2.
app-open-sample = Open sample page
app-empty-drop = or drag a .quad file onto this window
app-welcome-blank = + Blank page
app-welcome-recent = Recent
app-welcome-recent-none = Files you open or save are listed here.
app-welcome-not-found-in = Not found · { $folder }
app-welcome-search = Search
app-welcome-search-hint = Search pages
app-welcome-remove = Remove
app-welcome-remove-list = Remove from list
app-welcome-show-folder = Show in folder
app-welcome-gone-title = File not found
app-welcome-gone-body = { $name } is no longer where it was. Take it off the recent list?
app-welcome-remove-title = Remove this template?
app-welcome-remove-body = This deletes the template “{ $name }” from your computer. It cannot be undone.
app-welcome-tile-on = drawn on { $device }
app-welcome-nomatch = Nothing matches “{ $text }”.
app-welcome-soon = Starter pages for this group are coming soon.
app-welcome-soon-all = Starter pages are coming soon. Until then, start from a blank page, or keep your own pages under My templates.
app-welcome-mine-none = No templates yet. Choose File, then Save as template…, to keep a page here.
app-cat-all = All
app-cat-paper = Paper
app-cat-notes = Notes
app-cat-planning = Planning
app-cat-bujo = Bullet journal
app-cat-creative = Creative
app-cat-mine = My templates
app-template-replace-title = Replace this template?
app-template-replace-body = A template called “{ $name }” is already kept. Replace it with this page?
app-template-replace = Replace

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
export-error-nothing-open = There is nothing open to export.
export-error-unknown-token = The export path has a word in braces that Quadrule does not know: { $token }
export-error-unclosed = The export path has a { "{" } with no { "}" } after it.
export-error-no-file-name = The export path leaves no file name.
export-error-same-path = { $first } and { $second } would both write { $path }. Put { "{device}" } in the path to give each device its own file.
# The three path presets under the export path field (rung 15).
export-preset-flat = Flat
export-preset-folder-per-device = Folder per device
export-preset-device-then-set = Device, then set
notice-settings-damaged = Quadrule could not read its settings file, so it is using the defaults: { $error }
notice-settings-save-failed = Could not save the settings: { $error }
notice-template-saved = Kept “{ $name }” under My templates.
notice-template-save-failed = Could not keep the template: { $error }
notice-template-remove-failed = Could not remove { $path }: { $error }
notice-copy-device = This page could not be set up for { $device } ({ $error }), so it opened on the device it was made for.
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

# New layer and Delete layer (docs/bbq/2026-11-18-new-and-delete-layers.md; words proposed for chris's look).
app-layer-new = New layer
app-layer-delete = Delete layer
app-layer-new-name = Layer { $number }
app-layer-new-hint = Adds an empty layer above the picked one, or on top.
app-layer-delete-none = Pick a layer first.
app-layer-delete-last = A page needs at least one layer.
app-layer-delete-locked = Unlock it to delete it.
app-layer-delete-holds-locked = Unlock the objects in it to delete it.
app-layer-delete-ask = { $count ->
    [one] Delete the layer "{ $name }" and its { $count } object?
   *[other] Delete the layer "{ $name }" and its { $count } objects?
}
app-layer-delete-undo = Undo brings the layer back.
app-layer-delete-yes = Delete

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
app-orientation-landscape-paper-tip = A landscape paper size is saved wide, as you see it. It is not turned; only a tablet's page is.
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
app-pattern-angle = Pattern angle
app-opacity = Opacity
app-rotate-handle = Rotate handle

app-style-kind = Look
app-style-role = Role
app-style-custom = Custom
app-role-whisper = Whisper
app-role-guide = Guide
app-role-structure = Structure
app-role-emphasis = Emphasis
app-role-margin = Margin
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
app-pattern-calendar = Calendar
app-pattern-settings = Pattern settings
app-border-section = Border
app-border = Show border
app-border-sides = Sides
app-border-top = Top
app-border-right = Right
app-border-bottom = Bottom
app-border-left = Left
app-border-line = Line
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
app-staff-grand-space = Space between grand staves
app-calendar-month = Month
app-calendar-undated = Undated
app-calendar-year = Year
app-calendar-first-day = 1st falls on
app-calendar-days = Days in month
app-calendar-week-start = Week starts on
app-mini-calendar = Mini calendar
app-mini-title = Month and year
app-pattern-bound = Bound to rhythm
app-pattern-across = Spacing across
app-pattern-down = Spacing down
app-pattern-major = Major lines
app-major-follow = Follow the rhythm
app-major-own = Every N
app-pattern-major-n = Major every
app-pattern-major-role = Major role
app-pattern-major-rows-from = Major rows from
app-pattern-major-columns-from = Major columns from
app-major-from-top = Top
app-major-from-centre = Centre
app-major-from-origin = Rhythm origin
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
app-font-pick-tip = Choose a font…
app-font-missing = This font is not available on this computer.
app-font-picker-title = Font
app-font-search = Search fonts
app-font-bundled = Bundled
app-font-system = System
app-font-none-found = No font matches.
app-font-preview = Preview at { $size } pt, as it exports on { $device }
app-emoji = Emoji
app-emoji-tip = Insert an emoji
app-emoji-picker-title = Emoji
app-emoji-search = Search emoji by name
app-emoji-none-found = No emoji matches.
app-emoji-group-smileys = Smileys
app-emoji-group-people = People
app-emoji-group-nature = Nature
app-emoji-group-food = Food
app-emoji-group-travel = Travel
app-emoji-group-activities = Activities
app-emoji-group-objects = Objects
app-emoji-group-symbols = Symbols
app-emoji-group-flags = Flags
app-weight = Weight
app-weight-clamped = This font offers weights { $min } to { $max }; the text is drawn at { $used }.
app-style = Style
app-style-normal = Normal
app-style-italic = Italic
app-style-no-italic = This font has no italic, so the text stays upright.
app-size = Size
app-line-height = Line height
app-letter-spacing = Letter spacing (em)
app-align = Align
app-align-left = Left
app-align-center = Center
app-align-right = Right
app-align-justify = Justify
app-sizing = Sizing
app-sizing-auto = Auto height
app-sizing-fixed = Fixed
app-paragraph-spacing = Paragraph spacing
app-valign = Vertical align
app-valign-top = Top
app-valign-middle = Middle
app-valign-bottom = Bottom
app-baselines = Align baselines to rhythm
app-baselines-gap = Small gap above the line
app-line-height-steps = { $n } rhythm { $n ->
        [one] step
       *[other] steps
    } a line (set by the rhythm)
app-valign-auto-note = A box as tall as its text has no room to move it. Set Sizing to Fixed.

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
tool-mini-calendar = Mini calendar tool
tool-mini-calendar-tip = Mini calendar (M): this month, placed anywhere.
tools-keep-hint = Double-click to keep it on for several objects.

# What a drawn object is called, and what a new text box says.
new-name-line = Line
new-name-text = Text
new-name-mini-calendar = Mini calendar
new-name-pattern = Pattern
new-name-rect = Rectangle
new-name-ellipse = Ellipse
new-name-polygon = Polygon
new-text-content = Text

# Groups (rung 7a).
new-name-group = Group
notice-text-read-only = "{ $name }" is drawn from saved outlines because its font is not on this computer, so its text cannot change. It can be moved, turned, faded, copied, grouped and deleted. Choose a font to edit it.
notice-locate-failed = The fonts folder could not be opened: { $error }
notice-turned-group = The group "{ $name }" is turned, and repeats do not turn, so they cannot go into it.
notice-group-mixed = Those objects are in different groups. Pick objects from one place to group them.
notice-group-few = Pick two or more objects to group them.
notice-ungroup-none = Pick a group to ungroup it.
notice-turned-move = A turned group has a space of its own. Use Group and Ungroup to move objects across its edge.
app-group = Group
app-ungroup = Ungroup
app-fold-item = Show what is inside { $name }
app-rotation-note = A group with a repeat in it cannot turn.

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
app-page-paper = Paper
app-paper-ppi-name = Export PPI for { $device }
app-paper-ppi-option = { $ppi } PPI
app-paper-ppi-tip = The pixels per inch this page is exported at on this paper size. The design stays the same size in millimetres.
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
app-tab-repeat = Repeat settings
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

# Missing fonts (rung 12a). The bar above the canvas, the Missing fonts dialog, the Text tab's badge,
# the layers row's warning, the Page tab's tick and the Save sheet.
app-fonts-bar = { $count ->
        [one] One font missing: { $fonts }. Showing saved outlines.
       *[other] { $count } fonts missing: { $fonts }. Showing saved outlines.
    }
app-fonts-bar-name = { $name }
app-fonts-bar-sep = {", "}
app-fonts-bar-choose = Choose…
app-fonts-bar-close = Close
app-fonts-bar-close-tip = Hide this bar until the document is opened again
app-missing-title = Missing fonts
app-missing-intro = These fonts are not on this computer. Their text is drawn from the outlines saved in the file, which looks as designed but cannot be edited.
app-missing-row = { $count ->
        [one] { $font } (1 text box)
       *[other] { $font } ({ $count } text boxes)
    }
app-missing-use-saved = Use saved outlines
app-missing-substitute = Substitute: { $font }
app-missing-other = Other font…
app-missing-locate = Locate fonts…
app-missing-locate-tip = Open the folder where fonts are added on this computer; fonts added there are used at once
app-missing-remember = Remember for this document
app-missing-remember-tip = Keep the fonts left as saved outlines from being named in the bar again
app-missing-apply = Apply
app-missing-cancel = Cancel
app-missing-none = No fonts are missing.
app-text-saved-outlines = ⚠ Saved outlines. { $font } is not on this computer.
app-choose-a-font = Choose a font…
app-layer-saved-outlines = Drawn from saved outlines: its font is not on this computer
app-embed-fonts = Embed fonts
app-embed-fonts-tip = Save the installed fonts this page uses inside the file, where their licences allow it
app-sheet-title = Fonts installed on this computer
app-sheet-intro = This page uses fonts installed on this computer. Someone without them sees other fonts unless the fonts are embedded.
app-sheet-item = { $font }: { $status }
app-sheet-embedded = embedded in the file
app-sheet-off = not embedded: Embed fonts is off
app-sheet-cannot = cannot be embedded: { $reason }
app-sheet-fallback = (for letters the chosen font lacks)
app-sheet-continue = Continue
app-sheet-cancel = Cancel
app-sheet-dont-warn = Don't warn again for this document

# The Lint tab's device picker (rung 14b).
app-lint-device = Findings for
app-lint-all-devices = All devices

# The preflight sheet before Export (rung 14b; napkin §8.3).
app-pre-title = Before you export
app-pre-document = Document
app-pre-notes = { $count ->
        [one] 1 note
       *[other] { $count } notes
    }
app-pre-jump = Jump to object
app-pre-cancel = Cancel
app-pre-export = Export anyway
app-pre-clear = Nothing is left to warn about.
app-pre-rule = { $rule }
app-pre-paper = { $device } is paper, which has no screen to judge against, so these checks do not run on it: { $rules }.

# The Export sheet and the completion sheet (rung 15; napkin §11).
app-exp-title = Export
app-exp-devices = Devices
app-exp-this = This device: { $name }
app-exp-all = All devices: { $names }
app-exp-name = { $name }
app-exp-folder = Folder
app-exp-change = Change…
app-exp-no-folder = No folder chosen yet
app-exp-path = Path
app-exp-presets = Presets
app-exp-path-tip = Folders and names for the files. { "{title}" }, { "{device}" }, { "{set}" }, { "{date}" } and the other words in braces are filled in; a slash makes a folder.
app-exp-set = Set
app-exp-set-tip = A name for a group of templates. It fills { "{set}" } in the path; leave it empty to leave that folder out.
app-exp-files = Files that will be written
app-exp-no-files = No file to write.
app-exp-note = Add a text note (app, version, title)
app-exp-cancel = Cancel
app-exp-go = Export
app-done-title = { $count ->
        [one] Exported 1 file
       *[other] Exported { $count } files
    }
app-done-in = in { $folder }
app-done-replaced = { $count ->
        [one] 1 file replaced
       *[other] { $count } files replaced
    }
app-done-where = Where do these go?
app-done-where-body = On your Supernote, templates go in the MyStyle folder. Files in a folder inside MyStyle appear under the Customization tab in notes.
app-done-reveal = Reveal in folder
app-done-done = Done
notice-reveal-failed = Could not open the folder: { $error }

# Free placement (rung 14b).
app-free-placement = Free placement
app-free-placement-tip = Placed off the rhythm on purpose. The linter's off-rhythm check (L16) leaves it alone; nothing else changes.
app-full-bleed = Full bleed
app-full-bleed-tip = Meant to run under the tablet's toolbars and footers and to the page edge, as paper does. The linter's safe-zone check (L06) and off-the-page check (L10) leave it alone; nothing else changes.

# Style tab grays (rung 14b): swatches of the device's clean levels, and which role a stroke is.
app-style-show-grays = Show all grays
app-style-matches = matches { $role }
app-style-use-recommended = Use recommended
app-style-use-recommended-tip = Sets the { $role } role, the nearest to this width.

# Menus (fixes batch 2, Q3, Q6, Q7): the menu bar along the top of the window on Windows and Linux, the system menu bar
# on the Mac. The File menu's words are the ones above.
menu-edit = Edit
menu-cut = Cut
menu-copy = Copy
menu-paste = Paste
menu-duplicate = Duplicate
menu-select-all = Select all
menu-zoom-actual = Actual size (1:1)
menu-strip = Device previews
menu-snapping = Snapping
menu-help = Help
menu-about = About quadrule
menu-project-page = quadrule on GitHub
# The Mac's own menus: the app menu, Window, and the words the system gives its standard items.
menu-app = Quadrule
menu-about-mac = About Quadrule
menu-hide = Hide Quadrule
menu-hide-others = Hide Others
menu-show-all = Show All
menu-quit = Quit Quadrule
menu-window = Window
menu-minimize = Minimize
menu-zoom-window = Zoom

# About quadrule (Help, or the Quadrule menu on the Mac).
about-title = About quadrule
about-version = Version { $version }
about-licence = Licensed under the Apache License, Version 2.0 (Apache-2.0).
about-tagline = A free, open-source vector design tool for page templates and wallpapers for Supernote tablets.
about-close = Close

# The Basic section (rung 16b, Q5; ADR-026).
app-tab-basic = Basic
app-show-in-basic = Show in Basic
app-show-in-basic-tip = Lists this in the Basic tab, the first tab, with its main settings. Every setting stays in the other tabs too, and nothing on the page changes.
app-basic-empty = Nothing is in Basic yet. Tick Show in Basic on an object's Arrange tab, or on the Page tab for the page's title, margins and rhythm, and it is listed here.
app-basic-page = Page
app-basic-select = Select
app-basic-select-tip = Picks this on the page, to change the rest of its settings in the other tabs.
app-basic-other-tabs = The rest of its settings are in the other tabs.

# Preferences (rung 17 bbq, Q3): the window, Interface size and the theme (moved here from the View menu).
menu-preferences = Preferences…
prefs-title = Preferences
prefs-size = Interface size
prefs-size-follow = Follow system
prefs-size-percent = { $percent } %
prefs-size-note = Applied on top of your system's own scale. Kept for this computer, not saved in any page.
prefs-theme = Theme
prefs-profiles = Device profiles…

# Keyboard shortcuts (rung 17 bbq, Q4): the sheet under Help, one table of keys (`crates/ui/src/keys.rs`).
menu-shortcuts = Keyboard shortcuts
keys-title = Keyboard shortcuts
keys-group-file = File
keys-group-edit = Edit
keys-group-view = Zoom
keys-group-arrange = Align and move
keys-group-tools = Tools
keys-group-other = Other
keys-or = { " or " }
keys-delete = Delete
keys-front = Move up (toward the front)
keys-back = Move down (toward the back)
keys-nudge = Nudge one pixel
keys-nudge-rhythm = Nudge one rhythm unit
keys-arrows = Arrow keys
keys-arrows-shift = Shift + arrow keys
keys-picker = In the font and emoji lists: move and pick
keys-picker-keys = Arrow keys, Enter
keys-escape = Drop a drag, end a tool, step out of a group, then clear the selection
keys-note = Ctrl is Cmd on the Mac and Alt is Option. No key works while a text field has the keyboard.

# Help > Developer (rung 17 bbq, Q6): the readable JSON form of a page (napkin §12.1).
menu-developer = Developer
menu-export-json = Export as readable JSON…
notice-json-exported = Wrote the page as readable JSON to { $path }.
notice-json-failed = Could not write the readable JSON: { $error }

# Names for screen readers where the shown words would be the same as another control's in the window (rung 17 bbq, Q5).
app-show-in-basic-for = Show in Basic: { $what }
app-group-picked = Group the picked objects
app-ungroup-picked = Ungroup the picked group
app-repeat-picked = Repeat the picked objects
app-emoji-text-tab = Emoji, add to this text
app-finding-tag = { $rule }, { $object }, { $device }
app-finding-tag-page = { $rule }, { $device }
app-ruler-shorter = − Make the bar one pixel shorter
app-ruler-longer = + Make the bar one pixel longer
