// Copyright 2026 Chris Wells
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The viewer's state, with no UI in it: what is open, which device shows, the rendered page, the
//! lint findings, zoom maths and export. The egui widgets in [`crate::view`] only read and change
//! this, so everything here is tested headless. Editing (rung 3) adds to it in [`crate::editor`].

use quadrule_format::{Manifest, PREVIEW_SIDE, QuadFile, ReadOnly, preview_path};
use quadrule_i18n::Message;
use quadrule_layout::snap::SnapToggles;
use quadrule_layout::{
    DisplayList, Placed, SavedText, display_list, lay_out_text_saved, placed_objects,
};
use quadrule_lint::{
    EmbedStatus, Finding, FontFacts, InstalledFont, MissingBox, Options, Toolbar, lint_against,
};
use quadrule_model::edit::History;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{Document, Uuid};
use quadrule_profiles::Profile;
use quadrule_profiles::active::{self, lookup, lookup_in};
use quadrule_render::{Gray, Math, encode_png, render, turn_clockwise};
use quadrule_text::embed::NoEmbed;
use quadrule_text::{BundledFont, FontRegistry};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The family the missing-fonts page names, which no computer has.
pub const MISSING_PAGE_FONT: &str = "Quadrule Missing Test Font";

/// Toolbar positions the viewer lints for, shows safe zones for, and snaps to: the docked ones,
/// left, right and top (chris, 2026-11-10: "the safe area linting should look at all safe areas,
/// including the right one"; and, after rung 5, that L06 checking the floating menu bar "isn't
/// desired"; earlier, 2026-11-09, only "Docked left, Docked top"). It is the one list, so what is
/// drawn is what is checked.
pub const TOOLBARS_IN_USE: [Toolbar; 3] = Toolbar::DOCKED;

/// Toolbar positions whose safe zones are drawn and snapped to: left, right and top, not the
/// floating toolbar (chris, 2026-11-10: "the safe zone overlay should not include the floating
/// menu"). Since rung 5 the linter leaves it out too ([`TOOLBARS_IN_USE`]).
pub const TOOLBARS_SHOWN: [Toolbar; 3] = [Toolbar::Left, Toolbar::Right, Toolbar::Top];

/// Where the open document came from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// The rung 1 sample page, built in memory.
    Sample,
    /// A new document from File → New, not saved yet (rung 5).
    Untitled,
    /// A `.quad` file.
    File(PathBuf),
}

/// One device's view of the document: the exact export pixels and what the linter found.
#[derive(Clone, Debug)]
pub struct Page {
    /// The profile.
    pub profile: Profile,
    /// The page as vello_cpu renders it for export (ADR-003): what the PNG holds.
    pub image: Gray,
    /// Lint findings on this device.
    pub findings: Vec<Finding>,
    /// True when some text box was drawn from the outlines saved in the file because its font is
    /// missing here (ADR-001 item 2; rung 12a).
    pub saved_outlines: bool,
    /// What the linter was told about fonts for this page (rung 12a): the boxes drawn from saved
    /// outlines and the installed fonts used. The Save sheet lists the latter.
    pub font_facts: FontFacts,
    /// Each object's shape on this device, bottom first, for picking (rung 3).
    pub placed: Vec<Placed>,
    /// What was drawn, for the graphics-chip canvas (rung 3 bbq Q4).
    pub display_list: DisplayList,
    /// How many repeats drew fewer copies than their counts give, because of the cap (rung 10).
    pub repeats_capped: usize,
    /// The Fixed text boxes that left lines out on this device, for the red + tab at their
    /// bottom-right corners (rung 11b). On the canvases and the strip only, never exported.
    pub overflows: Vec<Overflow>,
}

/// A Fixed text box that leaves lines out on one device (rung 11b).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Overflow {
    /// The text box.
    pub id: Uuid,
    /// How many whole lines are left out.
    pub hidden: u32,
    /// The box's bottom-right corner on the device, in pixels (turned with the box).
    pub corner: (f64, f64),
}

/// The open document.
#[derive(Debug)]
pub struct Open {
    /// Where it came from.
    pub source: Source,
    /// The document.
    pub doc: Document,
    /// Text outlines saved in the file, by profile id (empty for the sample).
    pub(crate) saved: BTreeMap<String, ProfileOutlines>,
    /// Pages already rendered, by profile id. Emptied by every edit.
    pub(crate) pages: BTreeMap<String, Page>,
    /// Undo and redo, and whether there are unsaved changes (rung 3).
    pub history: History,
    /// The file's manifest as read, so a save keeps its extras (ADR-001 item 4).
    pub(crate) manifest: Option<Manifest>,
    /// Set when the file needs features this build lacks; it cannot be saved (napkin §12.2).
    pub read_only: Option<ReadOnly>,
    /// While an object is being dragged: the document as the drag would leave it. The page shows
    /// this instead of `doc` until the mouse is let go (rung 4 bbq, Defaulted "One drag, one undo
    /// step"); `doc` and its history are untouched.
    pub(crate) preview: Option<Document>,
    /// The lint findings of the last page made from `doc` itself, by profile id. A page made
    /// from the preview shows these rather than running the linter on every frame of a drag.
    pub(crate) findings: BTreeMap<String, Vec<Finding>>,
    /// The fonts the document names that are not on this computer, and the boxes that use them
    /// (rung 12a). Made again after every change and every re-scan.
    pub(crate) missing: Vec<MissingFont>,
    /// The bar above the canvas was closed with its ×; it stays closed until the document is opened
    /// again.
    pub(crate) bar_closed: bool,
    /// What the profiles the page uses need: carried copies this computer lacks, clashes to settle
    /// (rung 13a).
    pub(crate) profile_use: crate::profile_use::ProfileUse,
    /// The folder this document last exported to, this session (Export again; rung 15 bbq).
    pub(crate) exported_to: Option<PathBuf>,
}

/// A font a document names that is not on this computer (napkin §7.3; rung 12a).
#[derive(Clone, Debug, PartialEq)]
pub struct MissingFont {
    /// The family.
    pub family: String,
    /// The text boxes that use it, in document order.
    pub boxes: Vec<Uuid>,
    /// chris chose to keep it as saved outlines for this document ("Remember for this document"):
    /// the bar does not name it.
    pub kept: bool,
}

/// The two choices under Export (rung 2 bbq, Q8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWhich {
    /// The device on screen.
    ThisDevice,
    /// Every device the document has.
    AllDevices,
}

/// A pinch or Ctrl + scroll in progress (fixes batch 1, Q1): the scale the gesture itself has
/// reached, which the zoom shown is settled from, and when it last moved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoomGesture {
    /// The gesture's scale, in screen pixels per device pixel, before settling.
    pub raw: f32,
    /// The zoom this gesture last set; if the zoom is anything else the gesture was interrupted.
    pub shown: Zoom,
    /// The time of the gesture's last step, in seconds.
    pub last: f64,
}

/// A gesture that has not moved for this long is over, and the next one starts from the zoom shown.
pub const ZOOM_GESTURE_IDLE: f64 = 0.3;

/// How big the page shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zoom {
    /// The whole page fits the canvas.
    Fit,
    /// This many screen pixels per device pixel (1.0 is 1:1).
    Scale(f32),
    /// The page at its true size on the tablet, using the ruler check.
    RealSize,
}

/// The zoom steps, in screen pixels per device pixel: down to 1/8, up to 16× (rung 2 bbq, Q10). The
/// + and − buttons move by these; a pinch or Ctrl + scroll moves smoothly between them
/// (fixes batch 1, Q1).
pub const ZOOM_STEPS: [f32; 8] = [0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];

/// The smallest and the largest zoom, in screen pixels per device pixel (the ends of [`ZOOM_STEPS`]).
pub const ZOOM_MIN: f32 = 0.125;
/// See [`ZOOM_MIN`].
pub const ZOOM_MAX: f32 = 16.0;

/// What one notch of a mouse wheel with Ctrl held multiplies the scale by (fixes batch 1, Q1: a fixed
/// fraction, not a whole step). egui's own is 1.22.
pub const ZOOM_WHEEL_NOTCH: f32 = 1.15;

/// How close to 1:1 or to Fit a smooth zoom must come to settle on it, as a share of the scale
/// (fixes batch 1, Q1: "about 6 %").
pub const ZOOM_SETTLE: f32 = 0.06;

/// What a smooth zoom shows once the gesture's own scale `raw` (screen pixels per device pixel) is
/// clamped to [`ZOOM_MIN`] to [`ZOOM_MAX`] and settled: within [`ZOOM_SETTLE`] of 1:1 it is exactly 1:1,
/// within it of the Fit scale `fit` it is [`Zoom::Fit`] (the nearer wins when both are close);
/// anywhere else it is `Scale(raw)`. The caller keeps `raw` between frames, so the settling holds the
/// view still while the gesture crosses the zone and lets go of it on the far side.
pub fn settle_zoom(raw: f32, fit: f32) -> Zoom {
    let raw = raw.clamp(ZOOM_MIN, ZOOM_MAX);
    let off = |target: f32| (raw / target - 1.0).abs();
    let one = off(1.0);
    let fit_off = if fit.is_finite() && fit > 0.0 {
        off(fit)
    } else {
        f32::INFINITY
    };
    if fit_off <= ZOOM_SETTLE && fit_off < one {
        Zoom::Fit
    } else if one <= ZOOM_SETTLE {
        Zoom::Scale(1.0)
    } else {
        Zoom::Scale(raw)
    }
}

/// From this zoom up, the canvas draws a grid between device pixels (rung 2 bbq, Q10; moved from 8× to
/// 16× by chris, 2026-11-10, because at 8× its squares read as the rhythm overlay shrinking).
pub const PIXEL_GRID_FROM: f32 = 16.0;

/// A bank card's width (ISO/IEC 7810 ID-1), one of the ruler check's references.
pub const CARD_WIDTH_MM: f64 = 85.6;

/// The viewer.
pub struct Viewer {
    /// The open document; `None` on the empty start screen (rung 2 bbq, Q7).
    pub open: Option<Open>,
    /// The profiles a file being opened carries, handed to the registry when its document is set
    /// (rung 13a): the copies and the ids whose copy shows first.
    pub(crate) pending_carried: Option<(Vec<Profile>, std::collections::BTreeSet<String>)>,
    /// Counts the documents opened, so the window knows when to redraw its page texture.
    pub generation: u64,
    /// Counts changes to the open document (edits, undo, redo), for the same purpose.
    pub revision: u64,
    /// The selected objects, in the order they were picked (rung 3 bbq Q9).
    pub selection: Vec<Uuid>,
    /// The groups entered with a double-click, outermost first (rung 7a bbq, Q4): a click picks
    /// among the children of the innermost one. Empty: a click picks the outermost group.
    pub entered: Vec<Uuid>,
    /// The device on screen.
    pub profile_id: String,
    /// How big the page shows.
    pub zoom: Zoom,
    /// A pinch or Ctrl + scroll in progress: the gesture's own scale, kept between frames so the
    /// settling on 1:1 and Fit does not trap it (fixes batch 1, Q1).
    pub zoom_gesture: Option<ZoomGesture>,
    /// The finding outlined on the page, as an index into the current page's findings.
    pub selected: Option<usize>,
    /// Whether the safe zones are laid over the page (rung 2 bbq, Q6).
    pub show_safe_zones: bool,
    /// Physical screen pixels per millimetre, from the ruler check; `None` until it is done.
    pub screen_px_per_mm: Option<f64>,
    /// The last thing to tell the user (an error opening a file, where an export went).
    pub notice: Option<Notice>,
    /// The folder the last export went to, this session (rung 2 bbq, Defaulted "Export").
    pub export_dir: Option<PathBuf>,
    /// What Export remembers between runs (rung 15 bbq): the path template, the Set, the note tick, the devices
    /// choice, the folder of each document. Kept in `settings.ron` when the app gives it a file.
    pub settings: crate::settings::Settings,
    /// The settings file, when the app gave one; changes are written there.
    pub settings_path: Option<PathBuf>,
    /// The folder My templates lives in (`templates/` in the app's data folder); `None` in a test that has none.
    pub templates_dir: Option<PathBuf>,
    /// A fixed day for the `{date}` token (a test sets it); `None` is today.
    pub export_date: Option<String>,
    /// Which snap helpers are on (rung 4 bbq, Defaulted "Toggles"): app settings, not saved in
    /// the `.quad` file.
    pub snap: SnapToggles,
    /// The clipboard text last pasted and how many times in a row, so each paste goes one more
    /// step down and right of the last (rung 4b).
    pub(crate) paste_run: Option<(String, u32)>,
    /// Whether the rhythm is drawn faintly over the page (rung 4 bbq Q9); never exported.
    pub show_rhythm: bool,
    /// Whether the device previews strip is open (rung 8a, Q5); folded, only its header shows. An
    /// app setting, kept between runs, not saved in the `.quad` file.
    pub show_strip: bool,
    /// The drag in progress, if any (rung 4).
    pub drag: Option<crate::drag::Drag>,
    /// The tool on (rung 5): Select, or one that draws.
    pub tool: crate::tools::Tool,
    /// The tool stays on after each object (a double-click on its button).
    pub tool_kept: bool,
    /// The drawing in progress, if any (rung 5).
    pub draw: Option<crate::draw::Draw>,
    /// The names a drawn object gets (the window fills them from Fluent).
    pub names: crate::draw::NewNames,
    /// A text box was just drawn: the window opens the Text tab with its text selected.
    pub focus_new_text: bool,
    /// How far the rotate handle stands from the top edge, in device pixels: the window sets it
    /// from the zoom so the stalk keeps one length on screen (rung 6b).
    pub rotate_stalk_px: f64,
    /// What Align to is for a single object picked, as the user last chose it (rung 7b, Q5):
    /// the page, until they pick otherwise. An app setting, not saved in the `.quad` file.
    pub align_single: quadrule_layout::align::AlignTo,
    /// What Align to is for two or more objects picked: the selection, until they pick otherwise.
    pub align_multi: quadrule_layout::align::AlignTo,
    pub(crate) fonts: FontRegistry,
    /// Whether each installed or embedded family may be saved inside a file, found once per family
    /// (rung 12a); made again after a re-scan.
    pub(crate) embed_cache: BTreeMap<String, Result<(), NoEmbed>>,
    /// The typing session on the canvas, if any (rung 11b).
    pub(crate) typing: Option<crate::typing::Typing>,
    /// The re-scan of the installed fonts running now, if any (rung 11b).
    pub(crate) font_scan: Option<quadrule_text::PendingScan>,
    /// What the last re-scan of the installed fonts found and how long it took (rung 11a, ADR-002
    /// item 8: a full re-scan whenever the window regains focus).
    pub last_font_scan: Option<quadrule_text::Scan>,
}

/// A message for the user, and whether it is an error.
#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    /// What to say.
    pub message: Message,
    /// True for an error.
    pub error: bool,
}

impl std::fmt::Debug for Viewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Viewer")
            .field("open", &self.open)
            .field("profile_id", &self.profile_id)
            .field("zoom", &self.zoom)
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl Default for Viewer {
    fn default() -> Self {
        Viewer {
            open: None,
            pending_carried: None,
            generation: 0,
            revision: 0,
            selection: Vec::new(),
            entered: Vec::new(),
            profile_id: quadrule_model::sample::MANTA_ID.to_string(),
            zoom: Zoom::Fit,
            zoom_gesture: None,
            selected: None,
            show_safe_zones: false,
            screen_px_per_mm: None,
            notice: None,
            export_dir: None,
            settings: crate::settings::Settings::default(),
            settings_path: None,
            templates_dir: None,
            export_date: None,
            snap: SnapToggles::default(),
            paste_run: None,
            show_rhythm: false,
            show_strip: true,
            drag: None,
            tool: crate::tools::Tool::Select,
            tool_kept: false,
            draw: None,
            names: crate::draw::NewNames::default(),
            focus_new_text: false,
            rotate_stalk_px: 24.0 / 0.3,
            align_single: quadrule_layout::align::AlignTo::Page,
            align_multi: quadrule_layout::align::AlignTo::Selection,
            fonts: FontRegistry::bundled_only(),
            embed_cache: BTreeMap::new(),
            typing: None,
            font_scan: None,
            last_font_scan: None,
        }
    }
}

impl Viewer {
    /// Lets text use the fonts installed on this computer as well as the bundled ones (the app
    /// does this; tests keep the bundled fonts alone so they are the same on every machine).
    pub fn use_system_fonts(&mut self) {
        self.fonts = FontRegistry::with_system();
        self.invalidate_pages();
    }

    /// Scans the installed fonts again in full (ADR-002 item 8: no change detection), and waits for
    /// it. When the list of families differs, every page is laid out again. The window does not use
    /// this: see [`Viewer::start_font_scan`].
    pub fn rescan_fonts(&mut self) -> quadrule_text::Scan {
        let scan = self.fonts.rescan();
        self.last_font_scan = Some(scan);
        if scan.changed {
            self.invalidate_pages();
        }
        scan
    }

    /// Starts the re-scan of the installed fonts on a thread of its own and returns at once (rung
    /// 11b; the record's Amendments, "background re-scan"): focus never waits. One scan at a time: a
    /// focus while one runs starts no second. Returns whether a scan was started.
    pub fn start_font_scan(&mut self) -> bool {
        if self.font_scan.is_some() {
            return false;
        }
        self.font_scan = self.fonts.scan_in_background();
        self.font_scan.is_some()
    }

    /// Whether a scan of the installed fonts is running.
    pub fn font_scan_running(&self) -> bool {
        self.font_scan.is_some()
    }

    /// Takes in a finished scan: the font list swaps to the new one and, when the families differ,
    /// every page is laid out again. Returns what the scan found, once, when it has finished; never
    /// waits.
    pub fn poll_font_scan(&mut self) -> Option<quadrule_text::Scan> {
        let done = self.font_scan.as_ref()?.finished()?;
        self.font_scan = None;
        let scan = self.fonts.adopt(done);
        self.last_font_scan = Some(scan);
        if scan.changed {
            self.invalidate_pages();
        }
        Some(scan)
    }

    /// The weights a family offers, or `None` when it cannot be laid out here.
    pub fn fonts_weights(&mut self, family: &str) -> Option<quadrule_text::Weights> {
        self.fonts.weights_of(family)
    }

    /// True when the family has an italic face of its own.
    pub fn fonts_has_italic(&self, family: &str) -> bool {
        self.fonts.has_italic(family)
    }

    pub(crate) fn invalidate_pages(&mut self) {
        self.revision += 1;
        self.embed_cache.clear();
        if let Some(o) = &mut self.open {
            o.pages.clear();
        }
        self.refresh_font_state();
    }

    /// Opens the rung 1 sample page.
    pub fn open_sample(&mut self) {
        let doc = quadrule_model::sample::slice_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 6a shapes page: every new kind, fill, dash and cap, and the sample's dot
    /// grid (for the drag benchmark). Like the sample, it has no file.
    pub fn open_shapes(&mut self) {
        let doc = quadrule_model::sample::shapes_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 6b transforms page: rotated shapes, lines pointing every way and opacity,
    /// and the sample's dot grid (for the drag benchmark). Like the sample, it has no file.
    pub fn open_transforms(&mut self) {
        let doc = quadrule_model::sample::transforms_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 7a groups page: groups nested three deep, a turned group, a group at 50
    /// percent, a pinned child and a heavy group, and the sample's dot grid (for the drag
    /// benchmark). Like the sample, it has no file.
    pub fn open_groups(&mut self) {
        let doc = quadrule_model::sample::groups_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 8a overrides page: a box moved on the Nomad, an ellipse sized there, a star
    /// hidden there, smaller text, a group with an override, and a box that runs off the Nomad's
    /// page, and the sample's dot grid (for the benchmarks). Like the sample, it has no file.
    pub fn open_devices(&mut self) {
        let doc = quadrule_model::sample::devices_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 8b landscape page: a band and headline across the top, a ruled region, a star
    /// near the top-right corner and markers by the edges, and the sample's dot grid (for the
    /// benchmarks). Like the sample, it has no file.
    pub fn open_landscape(&mut self) {
        let doc = quadrule_model::sample::landscape_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 9 pattern page: every new pattern and style, major lines and an unbound region
    /// (a dot region holds the sample's id, for the benchmarks). Like the sample, it has no file.
    pub fn open_patterns(&mut self) {
        let doc = quadrule_model::sample::patterns_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 12b turned-patterns page: every pattern kind turned, one in a group and one in a
    /// repeat (the benchmarks drag its dot region, and turn its 45 degree grid). Like the sample, it has
    /// no file.
    pub fn open_turned_patterns(&mut self) {
        let doc = quadrule_model::sample::turned_patterns_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the pattern-borders page: a border round pattern regions, square and rounded, turned and not (the
    /// benchmarks drag its bordered dot region). Like the sample, it has no file.
    pub fn open_bordered(&mut self) {
        let doc = quadrule_model::sample::pattern_borders_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 12b pattern-angle page: the ruling turned inside its frame and cut at it (the
    /// benchmarks drag its dot region). Like the sample, it has no file.
    pub fn open_pattern_angle(&mut self) {
        let doc = quadrule_model::sample::pattern_angle_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 14a lint page: objects that make L03, L05, L07, L09, L13 and L14 fire beside ones
    /// that pass (the benchmarks drag its dot grid). Like the sample, it has no file.
    pub fn open_lint(&mut self) {
        let doc = quadrule_model::sample::lint_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 13b paper page: ISO A4 at 300 PPI, also targeting ISO A5 at 600 PPI (the
    /// benchmarks drag its dot region). Like the sample, it has no file.
    pub fn open_paper(&mut self) {
        let doc = quadrule_model::sample::paper_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 13b landscape paper page: US Letter held sideways, exported wide. Like the
    /// sample, it has no file.
    pub fn open_paper_landscape(&mut self) {
        let doc = quadrule_model::sample::paper_landscape_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 10 repeat page: rows, columns, a habit tracker, a Fill repeat with Centre, a
    /// group as the source and a hatch (the benchmarks drag the habit tracker). Like the sample, it
    /// has no file.
    pub fn open_repeats(&mut self) {
        let doc = quadrule_model::sample::repeats_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 11a fonts page: every bundled family upright and italic, weight ramps and
    /// emoji (the benchmarks drag its dot grid). Like the sample, it has no file.
    pub fn open_fonts(&mut self) {
        let doc = quadrule_model::sample::fonts_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 11b text-box page: a Fixed box with hidden lines, the three vertical
    /// alignments, a paragraph's space, baselines on a ruled region and turned text (the benchmarks
    /// drag its dot grid). Like the sample, it has no file.
    pub fn open_textbox(&mut self) {
        let doc = quadrule_model::sample::textbox_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens the rung 12a missing-fonts page: the text-box page with every text box in a font this
    /// computer lacks, so it is drawn from saved outlines and the bar names the font. The saved
    /// outlines are made here, with the bundled Caveat filed under a test-only family name. Like the
    /// sample, it has no file.
    pub fn open_missing_fonts(&mut self) {
        let mut doc = quadrule_model::sample::textbox_document();
        let ids: Vec<Uuid> = doc
            .objects()
            .filter(|o| matches!(o.kind, quadrule_model::ObjectKind::Text(_)))
            .map(|o| o.id)
            .collect();
        for id in ids {
            if let Some(o) = doc.object_mut(id)
                && let quadrule_model::ObjectKind::Text(t) = &mut o.kind
            {
                t.family = MISSING_PAGE_FONT.into();
                t.weight = 400;
            }
        }
        let caveat = quadrule_text::BUNDLED.iter().find(|b| b.id == "caveat");
        let mut saved = BTreeMap::new();
        if let (Some(c), Ok(primary)) = (caveat, lookup_in(&doc, &doc.primary_profile_id)) {
            let mut reg = FontRegistry::bundled_only();
            reg.set_embedded(&[(
                MISSING_PAGE_FONT.into(),
                std::sync::Arc::new(c.data.to_vec()),
            )]);
            for id in doc.profile_ids() {
                if let Ok(t) = lookup_in(&doc, id)
                    && let Ok(l) = quadrule_layout::lay_out_text(&doc, &primary, &t, &mut reg)
                {
                    saved.insert(id.to_string(), l.outlines);
                }
            }
        }
        self.set_open(Source::Sample, doc, saved, None);
    }

    /// The pattern page held sideways (rung 9).
    pub fn open_patterns_landscape(&mut self) {
        let doc = quadrule_model::sample::patterns_landscape_document();
        self.set_open(Source::Sample, doc, BTreeMap::new(), None);
    }

    /// Opens a new blank document from the New dialog's choices (rung 5): the chosen device is
    /// the one shown. It has no file until Save, and no unsaved changes until it is edited.
    pub fn open_new(&mut self, spec: &quadrule_layout::newpage::NewPage) -> Result<(), String> {
        let doc =
            quadrule_layout::newpage::new_document(spec, &quadrule_model::blank::now_rfc3339())
                .map_err(|e| e.to_string())?;
        self.set_open(Source::Untitled, doc, BTreeMap::new(), None);
        self.profile_id = spec.device_id.clone();
        Ok(())
    }

    /// Opens a `.quad` file from disk. On failure the open document stays as it was and the
    /// notice says why.
    pub fn open_path(&mut self, path: &Path) -> Result<(), String> {
        let result = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| self.open_bytes(Source::File(path.to_path_buf()), &b));
        if result.is_ok() {
            self.note_recent(path);
        }
        if let Err(e) = &result {
            self.notice = Some(Notice {
                message: Message::new("notice-open-failed")
                    .text("path", path.display().to_string())
                    .text("error", e.clone()),
                error: true,
            });
        }
        result
    }

    /// Opens a `.quad` file's bytes.
    pub fn open_bytes(&mut self, source: Source, bytes: &[u8]) -> Result<(), String> {
        let mut file = QuadFile::from_bytes(bytes).map_err(|e| e.to_string())?;
        // The profiles the page uses: here, or carried in the file (rung 13a). A secondary device that
        // is neither is left out; the primary missing refuses the file.
        let uses = self.meet_profiles(&mut file)?;
        let read_only = file.read_only.clone();
        // The fonts the file carries serve this document only; a damaged one is skipped (rung 12a).
        let carried: Vec<(String, std::sync::Arc<Vec<u8>>)> = file
            .manifest
            .fonts
            .iter()
            .flat_map(|f| {
                f.files.iter().filter_map(|ff| {
                    file.font_files
                        .get(&ff.path)
                        .map(|b| (f.family.clone(), std::sync::Arc::new(b.clone())))
                })
            })
            .collect();
        self.set_open(source, file.doc, file.outlines, Some(file.manifest));
        if self.fonts.set_embedded(&carried) {
            self.invalidate_pages();
        }
        let left_out = uses.left_out.clone();
        let damaged = uses.damaged.clone();
        if let Some(o) = &mut self.open {
            o.read_only = read_only;
            o.profile_use = uses;
            if !left_out.is_empty() {
                o.history.mark_unsaved();
            }
        }
        if !left_out.is_empty() || !damaged.is_empty() {
            let message = if left_out.is_empty() {
                Message::new("notice-profile-carried-damaged").num("count", damaged.len() as f64, 0)
            } else {
                Message::new("notice-profile-left-out").text("profiles", left_out.join(", "))
            };
            self.notice = Some(Notice {
                message,
                error: true,
            });
        }
        Ok(())
    }

    pub(crate) fn set_open(
        &mut self,
        source: Source,
        doc: Document,
        saved: BTreeMap<String, ProfileOutlines>,
        manifest: Option<Manifest>,
    ) {
        if !doc.profile_ids().contains(&self.profile_id.as_str()) {
            self.profile_id = doc.primary_profile_id.clone();
        }
        // A session in the document that was open is over with it; so are its carried fonts and
        // profiles (a file being opened has put its own in \`pending_carried\`).
        self.typing = None;
        self.fonts.set_embedded(&[]);
        let (carried, first) = self.pending_carried.take().unwrap_or_default();
        active::with_mut(|r| r.set_carried(carried, first));
        self.open = Some(Open {
            source,
            doc,
            saved,
            pages: BTreeMap::new(),
            history: History::new(),
            manifest,
            read_only: None,
            preview: None,
            findings: BTreeMap::new(),
            missing: Vec::new(),
            bar_closed: false,
            profile_use: Default::default(),
            exported_to: None,
        });
        self.refresh_font_state();
        self.drag = None;
        self.draw = None;
        self.tool = crate::tools::Tool::Select;
        self.tool_kept = false;
        self.focus_new_text = false;
        self.generation += 1;
        self.revision += 1;
        self.selection.clear();
        self.entered.clear();
        self.selected = None;
        self.notice = None;
        self.zoom = Zoom::Fit;
    }

    /// Closes the open document, which leaves nothing open (File > Close, rung 16): the welcome screen shows. The
    /// caller has asked about unsaved changes already. Its carried fonts and profiles go with it.
    pub fn close_document(&mut self) {
        if self.open.is_none() {
            return;
        }
        self.end_typing();
        self.typing = None;
        self.open = None;
        self.pending_carried = None;
        self.fonts.set_embedded(&[]);
        active::with_mut(|r| r.set_carried(Vec::new(), Default::default()));
        self.refresh_font_state();
        self.drag = None;
        self.draw = None;
        self.tool = crate::tools::Tool::Select;
        self.tool_kept = false;
        self.focus_new_text = false;
        self.generation += 1;
        self.revision += 1;
        self.selection.clear();
        self.entered.clear();
        self.selected = None;
        self.notice = None;
        self.zoom = Zoom::Fit;
        // The device on screen is one the empty window offers.
        if !self.profile_ids().contains(&self.profile_id) {
            self.profile_id = quadrule_model::sample::MANTA_ID.to_string();
        }
    }

    /// The device ids the open document has, primary first; both built-in devices on the empty
    /// screen.
    pub fn profile_ids(&self) -> Vec<String> {
        match &self.open {
            Some(o) => o.doc.profile_ids().into_iter().map(String::from).collect(),
            None => quadrule_profiles::active::device_ids(),
        }
    }

    /// Shows another device.
    pub fn set_profile(&mut self, id: &str) {
        if self.profile_id != id {
            self.end_typing();
            self.drag_cancel();
            self.profile_id = id.to_string();
            self.selected = None;
        }
    }

    /// The page for the device on screen, rendered on first use.
    pub fn page(&mut self) -> Option<&Page> {
        let id = self.profile_id.clone();
        self.page_for(&id).ok()
    }

    /// The page for device `id`, rendered on first use.
    pub fn page_for(&mut self, id: &str) -> Result<&Page, String> {
        let Viewer {
            open,
            fonts,
            embed_cache,
            ..
        } = self;
        let open = open.as_mut().ok_or("nothing is open")?;
        if !open.pages.contains_key(id) {
            let page = match &open.preview {
                // A frame of a drag: the linter is not run; the last findings stand.
                Some(doc) => {
                    let known = open.findings.get(id).cloned();
                    make_page(
                        doc,
                        &open.saved,
                        id,
                        fonts,
                        embed_cache,
                        Some(known.unwrap_or_default()),
                    )?
                    .0
                }
                None => {
                    let (page, outlines) =
                        make_page(&open.doc, &open.saved, id, fonts, embed_cache, None)?;
                    open.findings.insert(id.to_string(), page.findings.clone());
                    // What was just drawn is the newest text there is: should a font go away later,
                    // these are the shapes to fall back on, never ones from before an edit.
                    if let Some(o) = outlines {
                        open.saved.insert(id.to_string(), o);
                    }
                    page
                }
            };
            open.pages.insert(id.to_string(), page);
        }
        Ok(&open.pages[id])
    }

    /// The document's title, or `None` if it has none.
    pub fn title(&self) -> Option<&str> {
        let o = self.open.as_ref()?;
        let t = o.doc.metadata.title.trim();
        if !t.is_empty() {
            return Some(t);
        }
        match &o.source {
            Source::File(p) => p.file_stem().and_then(|s| s.to_str()),
            Source::Sample | Source::Untitled => None,
        }
    }

    /// The main device's thumbnail for the `.quad` file (format 0.16, rung 15 bbq, Q8): the path it is
    /// stored at and its PNG bytes. 8-bit gray, longest side [`PREVIEW_SIDE`] px (never enlarged), from
    /// the main device's export pixels (turned like the PNG for a landscape design), shrunk by averaging.
    /// `None` when nothing is open or the main device's page cannot be made.
    pub fn preview_png(&mut self) -> Option<(String, Vec<u8>)> {
        let id = self.open.as_ref()?.doc.primary_profile_id.clone();
        let page = self.page_for(&id).ok()?;
        let px = export_pixels(page);
        let scale = (PREVIEW_SIDE as f64 / px.width.max(px.height) as f64).min(1.0);
        let w = ((px.width as f64 * scale).round() as u32).max(1);
        let h = ((px.height as f64 * scale).round() as u32).max(1);
        Some((preview_path(&id), encode_png(&shrink(&px, w, h), None)))
    }

    /// Screen points per device pixel for the current zoom. `canvas` is the canvas size in points,
    /// `ppp` egui's physical pixels per point. `None` when nothing is open.
    pub fn scale(&mut self, canvas: (f32, f32), ppp: f32) -> Option<f32> {
        self.scale_for(self.zoom, canvas, ppp)
    }

    /// Screen points per device pixel were the zoom `zoom` (for the scale Fit would give while
    /// something else shows).
    pub fn scale_for(&mut self, zoom: Zoom, canvas: (f32, f32), ppp: f32) -> Option<f32> {
        let rs = self.screen_px_per_mm;
        let page = self.page()?;
        Some(points_per_device_px(
            zoom,
            (page.profile.width_px, page.profile.height_px),
            page.profile.ppi,
            canvas,
            ppp,
            rs,
        ))
    }

    /// Zooms by `factor` (above 1 in, below 1 out), the amount a pinch or Ctrl + scroll gave this
    /// frame (fixes batch 1, Q1). `now` is the scale showing and `fit` the scale Fit would give, both in
    /// screen pixels per device pixel; `time` is the clock in seconds. The scale is clamped to 1/8 to 16 and
    /// settles on 1:1 and on Fit ([`settle_zoom`]); the gesture keeps its own unsettled scale between
    /// frames, so crossing a settling zone holds the view still and then leaves it. Returns whether this
    /// step carries on a gesture already going (so its anchor holds) and not a new one.
    pub fn zoom_by(&mut self, factor: f32, now: f32, fit: f32, time: f64) -> bool {
        let carried = self
            .zoom_gesture
            .filter(|g| g.shown == self.zoom && time - g.last <= ZOOM_GESTURE_IDLE);
        let base = carried.map_or(now, |g| g.raw);
        let raw = (base * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        self.zoom = settle_zoom(raw, fit);
        self.zoom_gesture = Some(ZoomGesture {
            raw,
            shown: self.zoom,
            last: time,
        });
        carried.is_some()
    }

    /// Zooms one step in (`+1`) or out (`-1`) from the scale now showing (`now`, in screen pixels
    /// per device pixel).
    pub fn zoom_step(&mut self, now: f32, dir: i32) {
        let next = if dir > 0 {
            ZOOM_STEPS.iter().copied().find(|&s| s > now * 1.001)
        } else {
            ZOOM_STEPS.iter().rev().copied().find(|&s| s < now * 0.999)
        };
        if let Some(s) = next {
            self.zoom = Zoom::Scale(s);
            self.zoom_gesture = None;
        }
    }
}

/// The pixels a page's PNG holds: the page as it was drawn, and for a landscape design turned a
/// quarter turn clockwise into the device's portrait size, exactly (rung 8b, Q6).
pub fn export_pixels(page: &Page) -> std::borrow::Cow<'_, Gray> {
    if page.profile.turned {
        std::borrow::Cow::Owned(turn_clockwise(&page.image))
    } else {
        std::borrow::Cow::Borrowed(&page.image)
    }
}

/// Screen points per device pixel. `Scale(s)` means `s` physical screen pixels per device pixel;
/// real size uses the profile's PPI and the ruler check (falling back to 1:1 until it is done).
pub fn points_per_device_px(
    zoom: Zoom,
    page_px: (u32, u32),
    ppi: f64,
    canvas: (f32, f32),
    ppp: f32,
    screen_px_per_mm: Option<f64>,
) -> f32 {
    match zoom {
        Zoom::Fit => {
            // A margin of 16 points round the page, so its edge shows against the backdrop.
            let (w, h) = ((canvas.0 - 32.0).max(1.0), (canvas.1 - 32.0).max(1.0));
            (w / page_px.0 as f32).min(h / page_px.1 as f32)
        }
        Zoom::Scale(s) => s / ppp,
        Zoom::RealSize => match screen_px_per_mm {
            Some(r) => (r * quadrule_model::MM_PER_IN / ppi) as f32 / ppp,
            None => 1.0 / ppp,
        },
    }
}

/// Physical screen pixels per millimetre from the ruler check: a bar `bar_points` long matched
/// `reference_mm` on a ruler or card.
pub fn ruler_px_per_mm(bar_points: f32, ppp: f32, reference_mm: f64) -> f64 {
    (bar_points * ppp) as f64 / reference_mm
}

/// Makes the page for device `id`. Also gives the outlines it was drawn from when they hold the
/// newest text of every box (so the caller may remember them as the saved ones).
fn make_page(
    doc: &Document,
    saved: &BTreeMap<String, ProfileOutlines>,
    id: &str,
    fonts: &mut FontRegistry,
    embed_cache: &mut BTreeMap<String, Result<(), NoEmbed>>,
    known_findings: Option<Vec<Finding>>,
) -> Result<(Page, Option<ProfileOutlines>), String> {
    let primary = lookup_in(doc, &doc.primary_profile_id).map_err(|e| e.to_string())?;
    let target = lookup_in(doc, id).map_err(|e| e.to_string())?;
    // A box whose font is not on this computer is drawn from the shapes the file saved for it
    // (napkin §7.3; ADR-001 items 2 and 7); the others are laid out live.
    let (laid, how) =
        lay_out_text_saved(doc, &primary, &target, fonts, saved).map_err(|e| e.to_string())?;
    let saved_outlines = !how.from_saved.is_empty() || !how.reused.is_empty();
    let font_facts = font_facts(doc, &laid, &how, fonts, embed_cache);
    let outlines = laid.outlines;
    let dl = display_list(doc, &primary, &target, &outlines).map_err(|e| e.to_string())?;
    let image = render(&dl, Math::Fastest);
    let opts = Options {
        toolbars: TOOLBARS_IN_USE.to_vec(),
        fonts: font_facts.clone(),
    };
    let findings = match known_findings {
        Some(f) => f,
        None => {
            // L15 compares a text box with its height on the primary page, so the text is laid out
            // there too (rung 11b).
            let primary_laid = (target.id != primary.id && doc.objects().any(|o| o.keeps_height()))
                .then(|| lay_out_text_saved(doc, &primary, &primary, fonts, saved))
                .and_then(Result::ok)
                .map(|(l, _)| l);
            lint_against(
                doc,
                &primary,
                &target,
                &outlines,
                primary_laid.as_ref().map(|l| &l.outlines),
                &opts,
            )
        }
    };
    let placed = placed_objects(doc, &primary, &target, &outlines);
    let repeats_capped = placed
        .iter()
        .filter(|p| p.node.plan.is_some_and(|plan| plan.capped()))
        .count();
    let overflows = placed
        .iter()
        .filter(|p| !p.node.hidden)
        .filter_map(|p| {
            let hidden = outlines.text(p.id)?.lines_hidden;
            if hidden == 0 || doc.object(p.id).is_none_or(|o| o.keeps_height()) {
                return None;
            }
            let corner = match p.shape.corners() {
                Some(c) => c[2],
                None => {
                    let b = p.shape.bounds();
                    (b.2, b.3)
                }
            };
            Some(Overflow {
                id: p.id,
                hidden,
                corner,
            })
        })
        .collect();
    let fresh = how.from_saved.is_empty() && how.reused.is_empty() && how.needs_font.is_empty();
    Ok((
        Page {
            profile: target,
            image,
            findings,
            saved_outlines,
            font_facts,
            placed,
            display_list: dl,
            repeats_capped,
            overflows,
        },
        fresh.then_some(outlines),
    ))
}

/// What the linter is told about fonts on one device (rung 12a): the boxes drawn from saved outlines,
/// and the installed or embedded fonts the page uses, chosen or as fallback, each with whether the
/// file carries it.
fn font_facts(
    doc: &Document,
    laid: &quadrule_text::Laid,
    how: &SavedText,
    fonts: &FontRegistry,
    cache: &mut BTreeMap<String, Result<(), NoEmbed>>,
) -> FontFacts {
    let mut facts = FontFacts::default();
    for id in &laid.unavailable {
        if let Some(quadrule_model::ObjectKind::Text(t)) = doc.object(*id).map(|o| &o.kind) {
            facts.missing.push(MissingBox {
                id: *id,
                family: t.family.clone(),
                needs_font: how.needs_font.contains(id),
            });
        }
    }
    // Chosen families that are here but not bundled, then the fallback ones.
    let mut used: BTreeMap<String, bool> = BTreeMap::new();
    for o in doc.objects() {
        if let quadrule_model::ObjectKind::Text(t) = &o.kind
            && BundledFont::for_family(&t.family).is_none()
            && fonts.has_family(&t.family)
        {
            used.insert(t.family.clone(), false);
        }
    }
    for f in &laid.fallback {
        if BundledFont::for_family(f).is_none() && fonts.has_family(f) {
            used.entry(f.clone()).or_insert(true);
        }
    }
    for (family, fallback) in used {
        let embed = if !doc.page.fonts.embed {
            EmbedStatus::Off
        } else {
            let verdict = cache
                .entry(family.clone())
                .or_insert_with(|| fonts.embed_verdict(&family));
            match verdict {
                Ok(()) => EmbedStatus::Embedded,
                Err(r) => EmbedStatus::Cannot(r.message_id()),
            }
        };
        facts.installed.push(InstalledFont {
            family,
            fallback,
            embed,
        });
    }
    facts
}

/// The safe zones that count for snapping on `p`, as pixel rectangles: those of the toolbars
/// shown, the same ones the overlay draws (rung 2 bbq, Q6).
pub(crate) fn active_zone_rects(p: &Profile) -> Vec<quadrule_layout::snap::PxRect> {
    p.safe_zones
        .iter()
        .filter(|z| match quadrule_lint::Toolbar::of_zone(&z.name) {
            Ok(Some(t)) => TOOLBARS_SHOWN.contains(&t),
            _ => true,
        })
        .map(|z| {
            let (x, y, w, h) = z.rect;
            quadrule_layout::snap::PxRect {
                x0: x as f64,
                y0: y as f64,
                x1: (x + w) as f64,
                y1: (y + h) as f64,
            }
        })
        .collect()
}

/// A device's short name for the words about overrides, the strip and the Page tab's list: its
/// short name with a capital (`Manta`, `Nomad`); its id when the build has no such profile.
pub fn device_label(id: &str) -> String {
    match lookup(id) {
        // The user's own, and the paper sizes (`ISO A4`, `US Letter`), show their full names.
        Ok(p) if p.is_paper() || !quadrule_profiles::active::with(|r| r.is_builtin(id)) => {
            p.display_name
        }
        Ok(p) => {
            let mut c = p.short_name.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_else(|| id.to_string())
        }
        Err(_) => id.to_string(),
    }
}

/// A device's name in file names: its profile's short name, made safe (`manta`, `nomad`).
pub fn profile_token(p: &Profile) -> String {
    slug(&p.short_name)
}

/// ASCII letters, digits, hyphens and underscores only (napkin §11.3): letters lowercased, any
/// other run of characters becomes one hyphen, none at either end. Never empty.
pub fn slug(s: &str) -> String {
    let out = crate::export_path::slug_or_empty(s);
    if out.is_empty() {
        "untitled".into()
    } else {
        out
    }
}

/// Shrinks `img` to `out_w` × `out_h` by averaging: each output pixel is the mean of the device
/// pixels it covers, weighted by how much of each it covers. Zoomed out, a 1 px line or a small dot
/// turns lighter gray but never disappears, as it would if every n-th pixel were picked.
pub fn shrink(img: &Gray, out_w: u32, out_h: u32) -> Gray {
    let (out_w, out_h) = (out_w.clamp(1, img.width), out_h.clamp(1, img.height));
    // Across, then down; each pass is a box filter with fractional edges.
    let across = box_pass(&img.data, img.width, img.height, out_w, true);
    let down = box_pass(&across, out_w, img.height, out_h, false);
    Gray {
        width: out_w,
        height: out_h,
        data: down
            .iter()
            .map(|v| v.round().clamp(0.0, 255.0) as u8)
            .collect(),
    }
}

/// A copy of `img` at half the size (rounded up), each pixel the mean of the 2 × 2 it covers; at an odd
/// edge the last row or column counts twice. Fast, for the ladder of copies a zoom gesture draws from.
pub fn halve(img: &Gray) -> Gray {
    let (w, h) = (img.width as usize, img.height as usize);
    let (ow, oh) = (w.div_ceil(2).max(1), h.div_ceil(2).max(1));
    let mut data = Vec::with_capacity(ow * oh);
    for y in 0..oh {
        let (r0, r1) = (2 * y, (2 * y + 1).min(h - 1));
        for x in 0..ow {
            let (c0, c1) = (2 * x, (2 * x + 1).min(w - 1));
            let sum = u32::from(img.data[r0 * w + c0])
                + u32::from(img.data[r0 * w + c1])
                + u32::from(img.data[r1 * w + c0])
                + u32::from(img.data[r1 * w + c1]);
            data.push(((sum + 2) / 4) as u8);
        }
    }
    Gray {
        width: ow as u32,
        height: oh as u32,
        data,
    }
}

/// One box-filter pass over a `w` × `h` image, shrinking along x (`across`) or y to `n`.
fn box_pass<T: Copy + Into<f64>>(src: &[T], w: u32, h: u32, n: u32, across: bool) -> Vec<f64> {
    let (len, lines) = if across { (w, h) } else { (h, w) };
    let ratio = len as f64 / n as f64;
    let (ow, oh) = if across { (n, h) } else { (w, n) };
    let mut out = vec![0.0; (ow * oh) as usize];
    for i in 0..n {
        let (a, b) = (i as f64 * ratio, (i + 1) as f64 * ratio);
        let (first, last) = (a.floor() as u32, (b.ceil() as u32).min(len));
        for line in 0..lines {
            let mut sum = 0.0;
            for k in first..last {
                let cover = (b.min(k as f64 + 1.0) - a.max(k as f64)).max(0.0);
                let idx = if across { line * w + k } else { k * w + line };
                sum += src[idx as usize].into() * cover;
            }
            let o = if across { line * n + i } else { i * w + line };
            out[o as usize] = sum / ratio;
        }
    }
    out
}

/// A rectangle of an image in pixels: x, y, width, height.
pub type Tile = (u32, u32, u32, u32);

/// Cuts a `w` × `h` image into tiles at most `side` pixels a side, row by row, covering every pixel
/// once.
pub fn tiles(w: u32, h: u32, side: u32) -> Vec<Tile> {
    let side = side.max(1);
    let mut out = Vec::new();
    for y in (0..h).step_by(side as usize) {
        for x in (0..w).step_by(side as usize) {
            out.push((x, y, side.min(w - x), side.min(h - y)));
        }
    }
    out
}
