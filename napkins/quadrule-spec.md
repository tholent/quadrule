# Quadrule — Design Specification

**Status:** Draft v0.1
**Name:** Quadrule (crate and binary: `quadrule`)
**License:** Apache-2.0
**Language / stack:** Rust, egui (UI), wgpu (GPU)
**Platforms:** macOS, Windows, Linux (desktop only)

---

## 1. Overview

Quadrule is a free, open-source vector design tool for creating page templates and wallpapers for Supernote e-ink tablets. It combines an Illustrator-style canvas with a Pages-style inspector panel, and it adds e-ink-aware guidance: grayscale and line-weight recommendations, a lint/preflight system, and per-device previews.

It must serve two kinds of user equally well:

- **Casual users** pick a starter template, change a few things, and export. They never need to understand pixels, PPI or gray values.
- **Advanced users** control every coordinate, constraint, line role, gray level, and export setting.

The complexity is always present but never required. Moving or adding a line must never need the advanced controls.

### 1.1 Goals

1. Produce pixel-perfect, single-page PNG templates at each device's native resolution.
2. Make "good on e-ink" the default through recommendations, linting and calibration, without blocking any user choice.
3. Let one design target several devices through a device profile system with live, editable previews.
4. Use an open, efficient, shareable project format that survives missing fonts.
5. Stay pure Rust and cross-platform, with GPU acceleration and a CPU fallback.

### 1.2 Non-goals (for now)

- Mobile, tablet or web builds.
- Multi-page or hyperlinked PDF planners.
- SVG **export**. SVG import is planned for later.
- Direct device transfer over USB or cloud. The app exports to a folder; the user copies files to the device.
- Color e-ink devices. The profile system should not prevent adding them later.

---

## 2. Users and Core Workflows

### 2.1 Personas

| Persona | Wants | Touches |
|---|---|---|
| **Quick start** | "A nice dot grid for my Nomad, now." | Gallery, basic inspector fields, Export |
| **Tinkerer** | Customize a planner: move blocks, change fonts, adjust spacing. | Canvas, snapping, inspector Style/Text tabs |
| **Power designer** | Build complex layouts from scratch for several devices. | Constraints, overrides, repeat tool, line roles, lint config, calibration |

### 2.2 Primary workflows

**W1: Template from the gallery (casual)**
1. Launch → Welcome screen shows the starter gallery, filtered to the user's default device.
2. Pick a template → it opens fully editable.
3. Tweak with the inspector (e.g. grid spacing, title text).
4. Export → PNG saved to the export folder, with preflight shown only if issues exist.

**W2: Design from scratch (advanced)**
1. New document → choose a primary device profile, orientation, and document type (Template or Wallpaper).
2. Draw with shapes, lines, text and pattern regions; use snapping, guides and the repeat tool.
3. Add secondary profiles and review their previews; adjust constraints or per-object overrides, or fork a profile.
4. Run preflight → batch export for all profiles.

Device calibration is a **maintainer task**, not a user workflow. Recommended values are measured on real hardware by the project maintainer and shipped as presets in the built-in profiles (§8.4). Users get good defaults with no setup.

---

## 3. Device Profile System

### 3.1 Concept

A **device profile** describes one target screen. Profiles are data, not code. They ship built in, but users can edit, duplicate, import and share them as small files.

### 3.2 Profile fields

| Field | Type | Notes |
|---|---|---|
| `id` | string | Stable identifier, e.g. `supernote.a5x2` |
| `display_name` | string | "Supernote Manta (A5 X2)" |
| `width_px`, `height_px` | u32 | Native portrait resolution |
| `ppi` | f32 | Used for mm/pt ↔ px conversion |
| `diagonal_in` | f32 | Informational |
| `panel` | string | e.g. "Carta 1300 (flexible)" |
| `gray_levels` | list of u8 | Gray values the panel renders cleanly (**to verify**, see §17) |
| `ink_values` | list of u8 | Gray values used by pen colors, for "competes with ink" checks |
| `safe_zones` | list of named rects (px) | Areas covered by system UI (toolbar, etc.), measured from device screenshots (§3.5) |
| `default_margins` | mm × 4 | Suggested page margins |
| `orientations` | portrait / landscape | Supported template orientations |
| `line_roles` | map | Recommended width and gray per role (§8.2), from maintainer calibration |
| `calibration_meta` | record | Provenance of the presets: date, firmware version, method notes, `verified: bool` |
| `export_hints` | strings | Device folder names, shown as instructions only |

Storage: RON files, embedded in the binary for built-ins and stored in the user config directory for custom or edited profiles. A user-edited built-in becomes a local override that can be reset to default.

### 3.3 Built-in profiles

| Profile | Screen | Resolution (portrait) | PPI | Release |
|---|---|---|---|---|
| **A5 X2 Manta** | 10.7" | 1920 × 2560 | 300 | v1.0 |
| **A6 X2 Nomad** | 7.8" | 1404 × 1872 | 300 | v1.0 |
| A5 X | 10.3" | 1404 × 1872 | 226 | v1.1 |
| A6 X | 7.8" | 1404 × 1872 | 300 | v1.1 |
| A5 (original) | 10.3" | 1404 × 1872 | 226 | v1.1 |
| A6 (original) | 7.8" | 1404 × 1872 | 300 | v1.1 |

Notes:
- Nomad and A6 X share a resolution but may differ in panel behavior, so they stay separate profiles.
- A5 X and Manta differ in **both** resolution and PPI. That makes this pair the main test case for the constraint system.
- The original A5/A6 ran a different OS; template support on those devices must be verified before they are listed as supported.
- The built-in list should be re-checked at implementation time in case new models have launched.
- Older models ship as **"Unverified"** until calibrated on hardware (§8.4).

### 3.3.1 Paper size profiles (v1.1)

Alongside device profiles, Quadrule ships **paper profiles** for standard sizes. These are useful for other e-ink devices, for printing, or for designing a size-agnostic master layout. A paper profile is defined by its physical size plus an **export PPI** the user chooses (150, 200, **300** default, or 600).

| Paper | Size (mm) | At 300 PPI (px) |
|---|---|---|
| ISO A4 | 210 × 297 | 2480 × 3508 |
| ISO A5 | 148 × 210 | 1748 × 2480 |
| ISO A6 | 105 × 148 | 1240 × 1748 |
| ISO B5 | 176 × 250 | 2079 × 2953 |
| JIS B5 | 182 × 257 | 2150 × 3035 |
| US Letter | 215.9 × 279.4 | 2550 × 3300 |
| US Legal | 215.9 × 355.6 | 2550 × 4200 |
| Half Letter (Statement) | 139.7 × 215.9 | 1650 × 2550 |
| Executive | 184.2 × 266.7 | 2175 × 3150 |
| Custom | user-defined | — |

How paper profiles differ from device profiles:
- **No device-specific data:** no safe zones, no device gray levels, no ink values. Lint rules that need that data (L04, L05, L06) are skipped; the others use generic thresholds.
- **Line roles** resolve from physical sizes in mm, so a "Guide" line looks the same weight on paper as on a 300 PPI device.
- **Aspect ratios differ:** ISO sizes are about 1:1.414 and Supernote screens are 3:4. Constraints (§5.4) handle this, and the preview strip makes the difference visible.
- They work as primary or secondary profiles, like any other.

### 3.4 Primary and secondary profiles

Each document has **one primary profile**, where the user designs. It can have any number of **secondary profiles**, each shown as a live preview.

Each secondary profile is in one of two modes:

- **Linked (default):** Layout derives from the primary using each object's constraints (§5.4). Individual objects can carry **per-profile overrides**, such as a different position, size, font size or visibility.
- **Forked:** The profile gets its own independent copy of the scene and no longer follows the primary. Forking is explicit and reversible only by "Re-link (discard fork changes)", with a confirmation.

UI: a profile switcher in the toolbar, plus a "Device previews" strip showing thumbnails of all profiles. Clicking a preview makes it the active editing context. Edits there become overrides, clearly badged as such in the inspector.

### 3.5 Measuring safe zones (maintainer procedure)

Safe zones are measured from on-device screenshots, which Supernote saves at native screen resolution, so pixel coordinates map 1:1 to the template.

1. On each device, open a note with a blank template and capture screenshots with the toolbar in every position and state that can cover the page (e.g. docked left, docked right, floating, collapsed), in both portrait and landscape.
2. Measure each covered area as a pixel rectangle `(x, y, width, height)` in an image editor.
3. Record them in the profile RON as named zones, for example:
   ```ron
   safe_zones: [
       (name: "toolbar_left", orientation: Portrait, rect: (0, 0, W, H)),
       (name: "toolbar_floating_default", orientation: Portrait, rect: (X, Y, W, H)),
   ],
   ```
4. Store the source screenshots under `docs/device-measurements/<profile_id>/` in the repository so values can be re-checked after firmware updates.

In the app, the Page menu lets users choose which toolbar configuration matches how they use the device, and only those zones are shown and linted (L06). A floating toolbar can move, so its zone is marked advisory: shown in the overlay, but linted as Info rather than Warn.

A future maintainer tool (`cargo xtask measure`) could load a screenshot and let the maintainer drag rectangles to emit RON directly. This is optional; hand measurement is fine.

---

## 4. Document Types

| Type | Purpose | Defaults and lint behavior |
|---|---|---|
| **Template** | Background under handwriting | Lines default to guide roles; lints flag lines that compete with ink, content in safe zones, overly dark fills |
| **Wallpaper** (v1.1) | Lock / sleep screen art | Full gray range expected; ink-competition lints off; **gamut warnings** on (§9.4); raster images encouraged |

Both export to PNG at the profile's native resolution. Type can be changed after creation; lint rules update immediately.

---

## 5. Document and Object Model

### 5.1 Hierarchy

```
Document
├── metadata (title, author, license, description, tags, created/modified, app version)
├── document_type (Template | Wallpaper)
├── primary_profile_id
├── profiles: [ProfileContext]          // primary + secondary, each Linked or Forked
├── units (display preference: px | mm | pt | in)
├── page settings (background gray, margins, orientation)
├── rhythm (horizontal + vertical units, origin, subdivisions; §6.3)
├── line_roles (document-level overrides of profile defaults)
├── layers: [Layer]
│   └── objects: [Object]              // tree via Group
├── components: [ComponentDef]         // v1.1
└── assets (images, embedded fonts, text outlines)
```

### 5.2 Object types

| Object | v1 | Description |
|---|---|---|
| Line | ✅ | Straight segment; stroke width, gray, dash pattern, caps |
| Rectangle | ✅ | Optional corner radius; fill and/or stroke |
| Ellipse | ✅ | Fill and/or stroke |
| Polygon / star | ✅ | N sides; inner radius for stars |
| Text box | ✅ | Paragraph box (§7) |
| Emoji | ✅ (mono) | Single glyph object with an emoji source choice (§7.4) |
| Pattern region | ✅ | Parametric fill: lines, dots, square grid, **isometric**, hex, Cornell, music staff |
| Group | ✅ | Transform and constraint container |
| Repeat (array) | ✅ | Live array of a source object or group (§6.4) |
| Image | v1.1 | Raster with a non-destructive filter stack (§9) |
| Component instance | v1.1 | Linked instance of a ComponentDef |
| Path (Bézier) | later | Pen tool output |

All objects share: `id` (UUID), `name`, `transform`, `constraints`, `visible`, `locked`, `opacity` (exports as flattened gray), `line_role` or explicit style, and `overrides: Map<ProfileId, PropertyPatch>`.

**Pattern regions** are the backbone of grid-style templates. An isometric sheet is one object with a spacing value, not hundreds of lines. They clip to their bounds and support "every Nth line uses a different role" (e.g. a stronger line every 5th grid line). By default a pattern region's spacing is **bound to the document rhythm** (§6.3), so changing the rhythm updates every grid at once. Unbinding lets a region use its own spacing.

### 5.3 Units and coordinates

- Internal geometry is stored in **physical units (mm, f64)**. This keeps designs meaningful across PPI differences such as A5 X (226) vs Manta (300).
- Every numeric field accepts any unit: typing `5mm`, `12pt`, `40px`, `0.25in` or `50%` converts to the display unit. `%` is relative to the parent (page or group).
- Pixel snapping (§6.2) happens in each profile's device-pixel space at layout time, so lines are crisp on every device without changing the stored design.

### 5.4 Constraints (relative dimensioning made easy)

Each object has horizontal and vertical constraints, edited with a simple visual widget like Figma's:

- **Horizontal:** Left · Right · Left & Right (stretch) · Center · Scale
- **Vertical:** Top · Bottom · Top & Bottom (stretch) · Center · Scale

Defaults: objects use **Scale** for position and size, and stroke widths come from line roles. This means most designs adapt sensibly with no user action. Advanced users can pin headers to the top, footers to the bottom, and so on.

Text has an extra option: **font size** is "Fixed physical size" (default, stays legible) or "Scale with page."

### 5.5 Layers and groups

- A layers panel with a tree view: drag to reorder, eye toggles visibility, lock toggles editing, double-click renames.
- Groups can nest. Selecting inside a group uses double-click to enter it, as in Illustrator.
- Starter templates use clearly named layers ("Grid", "Header", "Labels") so casual users can hide or lock whole parts.

### 5.6 Undo, autosave, recovery

- Command-based undo/redo with unlimited depth per session; history panel later.
- Autosave to a recovery file every N seconds and on focus loss; restore offered after a crash.

---

## 6. Editor and Interaction Design

### 6.1 Window layout (Pages-inspired)

```
┌─────────────────────────────────────────────────────────────┐
│ Toolbar: Insert ▾ │ Line Rect Ellipse Text Pattern Repeat │ │
│          Zoom │ Profile ▾ │ Preview ◐ │ Lint ⚠3 │ Export   │
├──────────┬───────────────────────────────────┬──────────────┤
│ Sidebar  │                                   │ Inspector    │
│ ▸ Layers │           Canvas                  │ (tabs)       │
│ ▸ Profile│     (page on neutral backdrop)    │ Arrange      │
│   previews│                                  │ Style        │
│ ▸ Assets │                                   │ Text         │
│          │                                   │ Constraints  │
│          │                                   │ ── or ──     │
│          │                                   │ Page menu    │
├──────────┴───────────────────────────────────┴──────────────┤
│ Status: cursor mm/px · zoom · snapping toggles · lint count │
└─────────────────────────────────────────────────────────────┘
```

**Inspector behavior:**
- **Nothing selected → Page menu:** profile, orientation, document type, background, margins, safe-zone overlay, grids and snapping settings, line roles, lint settings.
- **Object selected → Format tabs:** Arrange (position, size, rotation, align, distribute), Style (line role or custom stroke/fill), Text, Constraints, and Overrides (when a secondary profile is active).
- Each tab shows **basic fields first**, with an "Advanced" disclosure section for precise values. This is progressive disclosure, not a mode switch, so nothing is ever hidden permanently.

### 6.2 Snapping and alignment helpers

All helpers can be toggled individually from the status bar and Page menu. Holding **Ctrl/Cmd while dragging** temporarily disables snapping.

| Helper | Default | Behavior |
|---|---|---|
| **Pixel grid snap** | **On** | Snaps edges and stroke centers so lines land on whole device pixels; odd-width strokes center on pixel centers |
| **Rhythm snap** | **On** | Snaps positions and sizes to the document rhythm (§6.3); takes priority over smart guides |
| Point snap | On | Snaps to object anchors: corners, midpoints, centers, line endpoints |
| Smart guides | On | Alignment lines to other objects' edges and centers while dragging |
| Equal spacing | On | Shows and snaps to equal gaps between three or more objects |
| Margins and safe zones | On | Snaps to margins; safe-zone overlay shows areas covered by system UI |

Nudging: arrow keys move 1 device pixel; Shift+arrow moves one rhythm unit (or one subdivision, if subdivisions are on).

### 6.3 Rhythm

Every document has its own **rhythm**: a horizontal unit and a vertical unit that define the underlying structure of the page. Grids, ruled lines, boxes and text all align to it by default. Rhythm is what makes a template feel consistent and intentional, and it replaces a separate "document grid."

**Rhythm settings** (Page menu → Rhythm):

| Setting | Notes |
|---|---|
| Type | **Rectangular** (independent H and V units) or **Isometric** (one side length; axes at 0°, 60°, 120°) |
| Horizontal unit | e.g. 5 mm; linked to vertical by default (chain icon to unlink) |
| Vertical unit | e.g. 5 mm; for ruled templates this is the line spacing |
| Origin | Top-left margin corner (default), page corner, or centered (rhythm balanced so leftover space splits evenly on both sides) |
| Subdivisions | Optional 2–10 per unit, for finer snapping (e.g. half-steps) |
| Major every N | Optional; marks every Nth unit as major, used by pattern regions and the overlay |
| Resolution mode | **Pixel-locked** (default) or **Physical-locked** (see below) |

**Per-profile resolution:** Each profile resolves the rhythm to device pixels. In **pixel-locked** mode, the unit rounds to a whole number of device pixels (5 mm at 300 PPI = 59 px, i.e. 4.995 mm). Lines then stay perfectly crisp and evenly spaced, with no drift across the page. In **physical-locked** mode, exact millimetres are kept and pixel snapping places each line on the nearest pixel, which can make spacing vary by 1 px. The inspector shows the resolved value per profile, e.g. "5 mm → 59 px (Manta), 44 px (A5 X)." Secondary profiles can override the rhythm, like any other per-profile property.

**What rhythm drives by default:**
- **Snapping:** Object edges, line endpoints and sizes snap to rhythm units or subdivisions.
- **Pattern regions:** Spacing bound to the rhythm (or a multiple of it).
- **Repeat tool:** Pitch defaults to one rhythm unit; values can be set as multiples ("3u").
- **New shapes:** Drawn sizes snap to whole units.
- **Text:** Text box positions snap to the rhythm. An "Align baselines to rhythm" setting sets line height to a rhythm multiple so text sits on the ruling. It is **off** for blank documents and **on** in ruled starter templates.
- **Units in fields:** `u` is accepted as a unit anywhere (`2u`, `0.5u`), alongside mm, px and %.

**Overlay:** A toggleable rhythm overlay (status bar) draws the rhythm faintly over the canvas, with major lines stronger. The overlay never exports.

**Easy defaults:** New blank documents start with a sensible handwriting rhythm (e.g. 6 mm, roughly college-ruled). Each starter template defines its own rhythm, so changing "line spacing" on a ruled starter is one field. A "Suggest rhythm" button offers presets: narrow, college and wide ruled, 5 mm grid, and 1/4 in grid.

Align and distribute commands: align left/center/right/top/middle/bottom to selection, page or margins; distribute horizontally or vertically by spacing or centers.

### 6.4 Repeat (array) tool

A **live** object that repeats a source object or group:

- **Modes:** Linear (count + direction) and Grid (rows × columns).
- **Spacing:** by pitch (center-to-center) or gap (edge-to-edge), in any unit; defaults to one rhythm unit.
- **Fill to bounds:** repeat as many as fit inside a region or the page margins.
- **Alternation:** "every Nth instance uses role X" or "skip instance N."
- **Edit source → all instances update.** "Expand" converts to plain objects.

Calendars, habit trackers, and weekly columns build on this. Index-aware text tokens (`{n}`, `{weekday}`, `{date}`) arrive in v1.2 with the calendar generator.

### 6.5 Selection and transform

Standard behavior: click and marquee select, Shift to add, bounding box with resize handles (Shift = keep proportions, Alt/Option = from center), rotation handle, duplicate with Alt-drag, and copy/paste between documents.

---

## 7. Text and Fonts

### 7.1 Text boxes

- **Paragraph boxes** with two sizing modes: *auto height* (fixed width, grows down) and *fixed size* (shows an overflow indicator and a lint warning when text is clipped).
- Properties: font family, weight, style, size, line height, letter spacing, paragraph spacing, alignment (left, center, right, justify), and vertical alignment in fixed boxes.
- v1: one style per text box. Rich text runs (mixed bold/italic within a box) come in v1.1.
- Text renders to outlines for display and export, so the output is identical everywhere.

### 7.2 Font sources

1. **System fonts:** Every font installed on the computer is available automatically, with no import step. They are discovered at startup with `fontdb` (all platforms), and the list refreshes when the app regains focus, so a newly installed font appears without a restart.
2. **Core bundled fonts** ship inside the app, kept lean to limit download size. All are under the SIL Open Font License (verify each license and include OFL texts):

| Family | Why |
|---|---|
| **Atkinson Hyperlegible Next** + **Mono** | Legibility-first; ideal for labels on e-ink; the default UI and text font |
| Inter | Neutral UI-style sans |
| Literata | Designed for screen reading; the default serif |
| Caveat or Patrick Hand | Handwritten-style headings |
| **Noto Emoji** | Monochrome emoji (§7.4) |

3. **Optional font packs** (v1.1): Additional open fonts (e.g. Noto Sans/Serif for wide language coverage, Source Serif 4, IBM Plex, Lexend, JetBrains Mono) are downloadable from the font menu. Packs are hosted as assets on the project's GitHub releases, downloaded only when the user asks, verified by checksum, and installed into the app's data folder. Quadrule makes no other network requests.

The font picker shows one unified list with *Bundled*, *Downloaded* and *System* sections, plus search, and a live preview at the device's actual size. Starter templates use only core bundled fonts, so they always open without a missing-font prompt.

### 7.3 Text outlines layer (missing-font handling)

Every saved document contains, for each text object, its **vector outlines** as rendered on each profile at save time. When a document opens:

1. If all fonts are available → render live text; ignore stored outlines.
2. If a font is missing → show a dialog listing each missing font by name, with three choices per font:
   - **Use saved outlines:** Text displays and exports exactly as designed, but its content is read-only until a font is chosen. A clear badge marks it in the inspector.
   - **Substitute:** The app recommends a match by classification (serif/sans/mono, weight and width metrics) from **all installed fonts, bundled and system alike**, preferring an exact family-name match on the system if one exists. The user can pick another.
   - **Locate font…:** Opens the OS font folder or installation instructions, then re-scans.
3. The choice can be applied to all affected text objects and remembered for this document.

Bundled fonts are referenced by ID and version rather than embedded, since every installation has them. Their outlines are still saved as a safety net.

### 7.4 Emoji

Each emoji object or emoji run has a **source** setting, chosen by the user:

- **Monochrome (default):** Rendered from Noto Emoji as vector outlines, so it is crisp and e-ink friendly. Available in v1.
- **Color → grayscale:** Rendered from a bundled color emoji font (e.g. Twemoji, CC-BY; check attribution requirements) and passed through the image grayscale pipeline (§9) with editable settings. Available in v1.1.

An emoji picker with search is available from the Insert menu.

---

## 8. E-ink Guidance: Recommendations, Linting, Calibration

### 8.1 Philosophy

**Recommend, explain, never block.** The app suggests good values, shows why, and warns before export, but users can always override and export anyway.

### 8.2 Line roles (recommended styles)

Instead of raw values, objects can use a **semantic line role**. Each profile resolves roles to a concrete width and gray:

| Role | Intended use |
|---|---|
| Whisper | Faint guides that fade under writing (dot grids, iso grids) |
| Guide | Standard ruling lines |
| Structure | Section dividers, box borders |
| Emphasis | Headers, strong separators |
| Text | Labels and headings (gray only) |

Why this matters: the same "Guide" line can resolve to different pixel widths on a 226 PPI A5 X and a 300 PPI Manta. That keeps multi-device designs visually consistent without manual tuning.

- The Style tab shows the role picker first. "Custom" reveals raw width and gray fields.
- When custom values match a role exactly, a "matches *Guide*" badge appears. When they don't, a one-click "Use recommended" button appears.
- Gray swatches show only the profile's clean gray levels by default. "Show all grays" opens a full 0–255 slider.

Role values ship as **maintainer-calibrated presets** in each built-in profile (§8.4). Profiles not yet calibrated on hardware use estimated values, and the profile picker marks them "Unverified." Advanced users can still adjust role values by hand in the profile editor; edits are saved as a local override with "Reset to built-in" available.

### 8.3 Linter and preflight

The linter runs continuously, with results in a Lint panel and an issue count in the toolbar. **Preflight** runs at export: if there are warnings, a summary sheet lists them with "Jump to object", "Ignore this rule for this document", and **"Export anyway"**.

| ID | Rule | Default severity |
|---|---|---|
| L01 | Stroke thinner than 1 device pixel (may vanish or shimmer) | Warn |
| L02 | Stroke not pixel-aligned (blurry edge) | Info |
| L03 | Gray too light for this device (may be invisible) | Warn |
| L04 | Gray or weight close to pen ink (will compete with handwriting) | Warn (Template only) |
| L05 | Gray not in the device's clean gray levels (may band or dither) | Info |
| L06 | Content overlaps a system UI safe zone | Warn |
| L07 | Text smaller than legible minimum at device PPI | Warn |
| L08 | Text overflows its box | Warn |
| L09 | Vertical rhythm below comfortable handwriting size | Info (Template only) |
| L10 | Object partly or fully off the page | Info |
| L11 | Missing font (using saved outlines) | Warn |
| L12 | Color image not converted to grayscale | Warn (v1.1) |
| L13 | Large dark fill areas (ghosting or slow refresh) | Info (Template only) |
| L14 | Too many anti-aliased intermediate grays on thin lines | Info |
| L15 | Per-profile: object differs on a secondary profile due to constraints | Info |
| L16 | **Off-rhythm:** object position, size, or pattern/repeat spacing not on the rhythm (or a subdivision) | Warn |
| L17 | **Mixed rhythm:** pattern region or repeat unbound from the rhythm with a spacing that isn't a clean multiple or fraction of it | Info |
| L18 | Rhythm doesn't fit the page evenly (leftover strip larger than half a unit at the page edge) | Info |
| L19 | Physical-locked rhythm causes uneven pixel spacing on a profile | Info |

Rhythm rules offer quick fixes: "Snap to rhythm" (single object or all off-rhythm objects), "Bind to rhythm," and "Center rhythm origin." Objects can be marked **"Free placement"** in the Arrange tab to opt out of L16 deliberately, e.g. a decorative element.

Each rule has a tooltip that explains the problem in plain language and offers a quick fix ("Snap to pixel grid", "Use Guide role", "Move inside safe area"). Rules and thresholds read from the active profile's presets. Lint runs per profile, and the panel can filter by profile.

### 8.4 Calibration (maintainer process)

Calibration is done by the project maintainer on real hardware. The results become the built-in presets. There is no user-facing calibration feature.

**Tooling:** A developer command generates the calibration sheets for each profile, using the app's own export renderer so the sheets match real output exactly:

```
cargo xtask calibration-sheet --profile supernote.a5x2 --out calib/
```

Each sheet contains:
- A matrix of line weights (e.g. 1–6 px) × gray levels, each cell labeled with a code like `C4`.
- Dot grid samples at several dot sizes and grays.
- Dashed and dotted line samples.
- Text samples at several sizes in the bundled fonts.
- Writing strips, where the maintainer writes over samples with each pen color to judge ink contrast.

**Procedure:**
1. Generate sheets; copy them to the device as templates.
2. Write over them with each pen type and color; view under typical lighting.
3. For each line role, choose the best sample code. Also record the minimum visible gray and weight and the "competes with ink" threshold.
4. Photograph or screenshot the results and store them, with notes, under `docs/device-measurements/<profile_id>/`.
5. Enter the values in the profile's RON file (`line_roles`, lint thresholds, `gray_levels`, `ink_values`) and fill in `calibration_meta` (date, firmware version, `verified: true`).
6. Golden-image tests are regenerated; the change goes through normal code review.

**Re-calibration:** Repeat after major firmware updates or new hardware revisions. Community members with other models can follow the same documented procedure and submit measurements as pull requests, which is how unverified profiles become verified.

---

## 9. Raster Images and the Grayscale Pipeline (v1.1)

### 9.1 Import

- Formats: PNG, JPEG, WebP, BMP, TIFF, GIF (first frame).
- The original file is stored unmodified in the project bundle. Images are content-addressed by hash and deduplicated.
- Images can be placed on templates and wallpapers, and masked or clipped to a shape.

### 9.2 Non-destructive filter stack

Each image object has an ordered, editable stack of filters. Each filter can be toggled, reordered, and reset. Presets save common stacks ("Photo → soft gray", "Line art", "Woodcut").

| Category | Filters |
|---|---|
| Color → gray | **Channel mixer** (R, G, B, C, M, Y contribution sliders, like a B&W adjustment), luminance presets (Rec.709, Rec.601, average), desaturate |
| Tone | Levels, curves, brightness/contrast, gamma, exposure |
| Stylize | **Threshold**, **woodcut / linocut**, posterize to N levels, **edge detection** (Sobel, Canny), sharpen, blur |
| Dithering | Floyd–Steinberg, **Atkinson**, ordered (Bayer), blue-noise, none |
| Output | Quantize to device gray levels, invert, opacity |

The final output is always quantized to the profile's gray levels at export. The canvas can preview this live.

### 9.3 Performance

- **GPU by default:** Each filter has a WGSL compute-shader implementation run via wgpu.
- **CPU fallback:** Every filter also has a CPU implementation, parallelized with `rayon`. It is selected automatically if the GPU path fails, or manually in Preferences.
- **Preview scale:** Preferences offer Full, 1/2 or 1/4 resolution for live preview. Export always renders at full resolution.
- Filter results are cached per image and stack hash. Only filters downstream of a change are recomputed.

### 9.4 Gamut warnings

A "Gamut" overlay highlights pixels that will change at export: colors that convert unexpectedly, grays that quantize to a different level, and areas where banding or dither patterns will appear. A small histogram shows the distribution across device gray levels.

---

## 10. Rendering Architecture

### 10.1 Pipeline

```
Document model ──► Layout (per profile: constraints, overrides, pixel snapping, text shaping)
                     │
                     ▼
              Display list (paths, fills, strokes, glyph outlines, images)
                     │
         ┌───────────┴────────────┐
         ▼                        ▼
  Canvas renderer (GPU)    Export renderer (deterministic)
  vello on wgpu,            produces the exact PNG;
  drawn into egui           also used for "Device preview" mode
```

- **Canvas:** vello renders the display list to a wgpu texture shown by egui. It supports zooming, panning, selection overlays and guides.
- **Export:** A deterministic rasterizer produces identical PNGs on every platform. Recommended default: CPU rasterization (e.g. `tiny-skia` or vello's CPU path), so golden-image tests are stable across GPUs. The choice should be finalized during a rendering spike (§16).
- **Device preview mode** (toolbar ◐): Shows the canvas through the export renderer at 1:1 device pixels with gray quantization. What you see is exactly what exports.

### 10.2 Pixel-perfect rules

- Layout converts mm → device px per profile, then snaps if pixel snapping is on.
- Stroke widths resolve to integer device pixels where possible. Odd widths center on pixel centers; even widths center on pixel edges.
- Anti-aliasing defaults on for curves and text and is effectively off for snapped axis-aligned lines. An export option can force fully aliased output.

---

## 11. Export

### 11.1 PNG output

- Native resolution of each profile, in the chosen orientation.
- **8-bit grayscale PNG** (no alpha), with the background flattened to the page background gray.
- Optional metadata text chunk: app name, version, source document title (on by default, can be disabled).

### 11.2 Batch export

- "Export all profiles" renders every enabled profile in one action.
- **Naming scheme** (configurable template; default shown):
  `{title}_{profile}_{width}x{height}.png`
  e.g. `isometric-5mm_manta_1920x2560.png`, `isometric-5mm_nomad_1404x1872.png`
- Tokens: `{title}`, `{profile}`, `{model}`, `{width}`, `{height}`, `{orientation}`, `{type}`, `{date}`, `{version}`. Titles are slugified for safe filenames.
- Option for per-profile subfolders.
- Remembered export folder per document, with a global default.

### 11.3 After export

A completion sheet shows thumbnails and "Reveal in folder", plus a short note on where to copy the files on the device. Per Supernote's documentation, custom templates go in the device's **MyStyle** folder, and files placed in a MyStyle **subfolder** appear under the Customization tab in notes. MyStyle also holds custom sleep-screen images, so wallpapers go there too.

Export options that follow from this:
- **Set folder:** Optional "Export as set" writes a named subfolder (e.g. `MyStyle/<set name>/`) so a group of templates stays together in the device's picker.
- **Safe filenames:** Supernote advises against special characters in template filenames, so the naming scheme slugifies to ASCII letters, digits, hyphens and underscores, regardless of the document title.

---

## 12. Project File Format

### 12.1 Container

A single file with the extension **`.quad`** that is a **ZIP archive**:

```
project.quad
├── manifest.cbor          // format version, app version, doc title, profiles,
│                          // font references, asset index, checksums
├── scene.cbor             // document model (§5), compact binary
├── assets/
│   ├── images/<sha256>.<ext>      // original raster files (v1.1)
│   └── fonts/<id>.ttf             // only fonts whose license allows embedding
├── outlines/
│   └── <profile_id>.cbor          // text outlines per text object (§7.3)
└── previews/
    └── <profile_id>.png           // small thumbnails for galleries and file browsers
```

- **Encoding:** CBOR via `serde` (`ciborium`) for compactness and speed. Field names are kept in the encoding, so files remain inspectable with standard CBOR tools and can evolve safely.
- **Compression:** Deflate for CBOR entries; images stored without recompression.
- **Debug export:** "Export as readable JSON" (developer menu) writes the same model as pretty JSON for diffing and bug reports. The app can also open that JSON.

### 12.2 Versioning

- `format_version` in the manifest uses semver.
- Older files are upgraded through a chain of migration functions, with tests per version.
- Newer files open read-only with a warning if they contain unknown required features. Unknown optional fields are preserved on save.

### 12.3 Sharing

- One file holds everything needed to open, edit and export the design: images, outlines, and license-permitted fonts.
- Document metadata includes author, license (default CC-BY-4.0 for shared templates, user-selectable) and description, ready for a future community gallery.
- Custom device profiles (e.g. a user-defined size) export as standalone `.ron` files.

---

## 13. Starter Gallery (v1)

Each starter is a normal project file, organized into named layers, using line roles, constraints and its own rhythm so it adapts to every profile. All starters must pass lint with no rhythm warnings.

| Category | Starters |
|---|---|
| Paper | Ruled (narrow, college, wide), dot grid, square grid, **isometric grid**, hex grid, blank with margins |
| Notes | Cornell notes, meeting notes, lecture notes |
| Planning | Daily planner, weekly spread (vertical and horizontal), monthly calendar (static grid), to-do list |
| Bullet journal | Rapid-log page, habit tracker, future log, index |
| Creative | Storyboard, music staff, handwriting practice |

Gallery features: device filter, search, preview thumbnails per profile. User-saved documents can be added to a "My templates" section. Each starter's top-level settings (spacing, margins, title) are exposed in the inspector's basic section so quick-start users rarely need more.

---

## 14. Accessibility, Localization, Theming

- **UI scaling:** Follows OS scale by default; manual override from 75% to 200% in Preferences.
- **Dark mode:** Light, dark, or follow system. The page itself always renders true to output; the backdrop around it follows the theme.
- **Translations:** All UI strings go through Project Fluent (`fluent-bundle`). English ships first; locale files are community-contributed. Number and unit formatting respect locale.
- **Keyboard:** Standard shortcuts for all common commands (tool keys, align, group, nudge, zoom). Full keyboard-only editing is a later goal.
- **Screen readers:** egui's AccessKit integration exposes UI controls. Canvas content accessibility is out of scope for v1.

---

## 15. Technical Architecture

### 15.1 Cargo workspace

| Crate | Responsibility |
|---|---|
| `app` | Binary: eframe setup, windowing, menus, preferences |
| `ui` | egui widgets: canvas, inspector, layers, gallery, dialogs |
| `model` | Document model, commands and undo; no GPU or UI dependencies |
| `format` | ZIP/CBOR read and write, migrations, JSON debug export |
| `profiles` | Profile schema, built-in RON data, user profile storage, calibration |
| `layout` | Constraints, overrides, unit conversion, pixel snapping |
| `text` | Font registry (fontdb), shaping and wrapping (cosmic-text), outline extraction (skrifa), substitution matching |
| `render` | Display list, vello canvas renderer, deterministic export renderer, PNG encoding |
| `lint` | Rule engine and rules, quick fixes |
| `imaging` | Filter stack, WGSL compute shaders, CPU fallbacks (v1.1) |
| `i18n` | Fluent loading and string lookup |

The `model`, `layout`, `lint` and `format` crates must be fully testable headless. This enables a future command-line exporter (`cargo run -p cli -- export doc.quad --all`).

### 15.2 Key dependencies (candidates)

`eframe`/`egui` (wgpu backend), `wgpu`, `vello`, `kurbo`, `peniko`, `cosmic-text`, `fontdb`, `skrifa`, `tiny-skia` (export option), `image`, `png`, `zip`, `ciborium`, `serde`, `uuid`, `rfd` (file dialogs), `directories`, `fluent-bundle`, `rayon`, `tracing`. Versions are pinned in `Cargo.lock`, and vello's maturity is evaluated in the rendering spike.

### 15.3 Testing and CI

- **Golden-image tests:** Every starter template is exported for every profile and compared to reference PNGs pixel by pixel.
- Unit tests for each lint rule, unit conversion and constraint resolution.
- Format round-trip tests and fuzzing of the file reader (`cargo-fuzz`).
- GitHub Actions on Windows, macOS and Linux: build, test, clippy, rustfmt.

### 15.4 Packaging

- macOS: universal `.dmg`, **unsigned** at first. The download page and README include clear steps for opening an unsigned app (right-click → Open, or System Settings → Privacy & Security → Open Anyway). Apple signing and notarization ($99/year developer account) can be added later if adoption grows.
- Windows: MSI installer, plus winget listing.
- Linux: AppImage and Flatpak (Flathub).

---

## 16. Roadmap

### v1.0: Core designer
- Manta and Nomad profiles; profile system and profile editor.
- Primary and secondary profiles with constraints, per-object overrides and live previews.
- Starter gallery (§13).
- Lines, rectangles, ellipses, polygons, pattern regions (including isometric), text boxes (single style), monochrome emoji.
- Layers, groups, repeat/array tool.
- Snapping and alignment helpers, pixel snapping on by default.
- Per-document **rhythm** (rectangular and isometric), rhythm snapping on by default, rhythm-bound patterns and repeats, rhythm lint rules (L16–L19).
- Line roles with **maintainer-calibrated presets** for Manta and Nomad; measured safe zones; linter and preflight.
- `cargo xtask calibration-sheet` maintainer tool and documented calibration procedure.
- PNG export, batch export with naming scheme, device preview mode.
- Project file format with text outlines and missing-font flow.
- Dark mode, UI scaling, translation infrastructure.

### v1.1: Images and wallpapers
- Remaining Supernote profiles (marked "Unverified" until calibrated on hardware).
- Standard paper size profiles with selectable export PPI (§3.3.1).
- Optional downloadable font packs.
- Raster image import with non-destructive GPU filter stack; gamut warnings.
- Wallpaper document type.
- Reusable components.
- Profile forking.
- Color emoji via grayscale pipeline; rich text runs.

### v1.2 and later
- SVG import.
- Calendar generator and text tokens (`{n}`, `{date}`, …).
- Pen/Bézier tool, boolean operations.
- Document variables and expressions (e.g. `margin * 2`).
- In-app community gallery (backed by a curated Git repository).
- Command-line exporter.

### Early technical spikes (before v1 feature work)
1. **Rendering:** vello in egui via wgpu; deterministic export renderer; pixel-snap correctness at 226 and 300 PPI.
2. **Text:** cosmic-text + fontdb on all three OSes; outline extraction; system font enumeration performance.
3. **Format:** CBOR schema, migrations, and outline storage size on a text-heavy planner.

---

## 17. Open Questions and Items to Verify

### 17.1 Open: maintainer measurements (before v1)

1. **Gray levels and ink values:** Measure with calibration sheets (§8.4) on Manta and Nomad.
2. **Safe zones:** Measure from toolbar screenshots (§3.5), per toolbar state and orientation.
3. **Landscape templates:** Export one landscape PNG and confirm whether the device expects rotated pixels or swapped dimensions.

### 17.2 Deferred

4. **Local rhythms:** Whether regions (e.g. a header band) can define their own rhythm. Revisit after v1.

### 17.3 Resolved

| Topic | Decision |
|---|---|
| Name | **Quadrule**; project files use `.quad`. The `quadrule` crate name is free. A Debian/Ubuntu numerical library is also named QUADRULE, so a distro package would need a distinct name (e.g. `quadrule-designer`); Flatpak and AppImage are unaffected. |
| Device folders | Templates and sleep screens go in MyStyle; subfolders appear under Customization (§11.3). |
| Older models (A5, A6, A5 X, A6 X) | Ship in v1.1 as "Unverified" until calibrated; standard paper sizes added as profiles too (§3.3.1). |
| Profile forking | v1.1; per-object overrides cover v1. |
| Constraints | In v1. |
| macOS signing | Start unsigned with install instructions (§15.4). |
| Fonts | Lean core bundle, all system fonts available automatically, optional downloadable packs (§7.2). |
| Rhythm defaults | Pixel-locked mode, 6 mm default, baseline alignment off for blank documents and on for ruled starters (§6.3). |
