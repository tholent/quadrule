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

use quadrule_format::{Manifest, QuadFile, ReadOnly};
use quadrule_i18n::Message;
use quadrule_layout::snap::SnapToggles;
use quadrule_layout::{DisplayList, Placed, display_list, lay_out_text, placed_objects};
use quadrule_lint::{Finding, Options, Toolbar, lint_with};
use quadrule_model::edit::History;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{Document, Uuid};
use quadrule_profiles::{Profile, builtin};
use quadrule_render::{Gray, Math, encode_png, render};
use quadrule_text::FontRegistry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    /// True when the page used the outlines saved in the file because a font is missing here
    /// (ADR-001 item 2).
    pub saved_outlines: bool,
    /// Each object's shape on this device, bottom first, for picking (rung 3).
    pub placed: Vec<Placed>,
    /// What was drawn, for the graphics-chip canvas (rung 3 bbq Q4).
    pub display_list: DisplayList,
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
}

/// The two choices under Export (rung 2 bbq, Q8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWhich {
    /// The device on screen.
    ThisDevice,
    /// Every device the document has.
    AllDevices,
}

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

/// The zoom steps, in screen pixels per device pixel: down to 1/8, up to 16× (rung 2 bbq, Q10).
pub const ZOOM_STEPS: [f32; 8] = [0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];

/// From this zoom up, the canvas draws a grid between device pixels (rung 2 bbq, Q10; moved from 8× to
/// 16× by chris, 2026-11-10, because at 8× its squares read as the rhythm overlay shrinking).
pub const PIXEL_GRID_FROM: f32 = 16.0;

/// A bank card's width (ISO/IEC 7810 ID-1), one of the ruler check's references.
pub const CARD_WIDTH_MM: f64 = 85.6;

/// The viewer.
pub struct Viewer {
    /// The open document; `None` on the empty start screen (rung 2 bbq, Q7).
    pub open: Option<Open>,
    /// Counts the documents opened, so the window knows when to redraw its page texture.
    pub generation: u64,
    /// Counts changes to the open document (edits, undo, redo), for the same purpose.
    pub revision: u64,
    /// The selected objects, in the order they were picked (rung 3 bbq Q9).
    pub selection: Vec<Uuid>,
    /// The device on screen.
    pub profile_id: String,
    /// How big the page shows.
    pub zoom: Zoom,
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
    /// Which snap helpers are on (rung 4 bbq, Defaulted "Toggles"): app settings, not saved in
    /// the `.quad` file.
    pub snap: SnapToggles,
    /// The clipboard text last pasted and how many times in a row, so each paste goes one more
    /// step down and right of the last (rung 4b).
    pub(crate) paste_run: Option<(String, u32)>,
    /// Whether the rhythm is drawn faintly over the page (rung 4 bbq Q9); never exported.
    pub show_rhythm: bool,
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
    pub(crate) fonts: FontRegistry,
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
            generation: 0,
            revision: 0,
            selection: Vec::new(),
            profile_id: quadrule_model::sample::MANTA_ID.to_string(),
            zoom: Zoom::Fit,
            selected: None,
            show_safe_zones: false,
            screen_px_per_mm: None,
            notice: None,
            export_dir: None,
            snap: SnapToggles::default(),
            paste_run: None,
            show_rhythm: false,
            drag: None,
            tool: crate::tools::Tool::Select,
            tool_kept: false,
            draw: None,
            names: crate::draw::NewNames::default(),
            focus_new_text: false,
            fonts: FontRegistry::bundled_only(),
        }
    }
}

impl Viewer {
    /// Opens the rung 1 sample page.
    pub fn open_sample(&mut self) {
        let doc = quadrule_model::sample::slice_document();
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
        let file = QuadFile::from_bytes(bytes).map_err(|e| e.to_string())?;
        // Every profile the document names must be one this build knows, or nothing can show.
        for id in file.doc.profile_ids() {
            builtin(id).map_err(|e| e.to_string())?;
        }
        builtin(&file.doc.primary_profile_id).map_err(|e| e.to_string())?;
        let read_only = file.read_only.clone();
        self.set_open(source, file.doc, file.outlines, Some(file.manifest));
        if let Some(o) = &mut self.open {
            o.read_only = read_only;
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
        });
        self.drag = None;
        self.draw = None;
        self.tool = crate::tools::Tool::Select;
        self.tool_kept = false;
        self.focus_new_text = false;
        self.generation += 1;
        self.revision += 1;
        self.selection.clear();
        self.selected = None;
        self.notice = None;
        self.zoom = Zoom::Fit;
    }

    /// The device ids the open document has, primary first; both built-in devices on the empty
    /// screen.
    pub fn profile_ids(&self) -> Vec<String> {
        match &self.open {
            Some(o) => o.doc.profile_ids().into_iter().map(String::from).collect(),
            None => quadrule_profiles::builtin_ids().map(String::from).collect(),
        }
    }

    /// Shows another device.
    pub fn set_profile(&mut self, id: &str) {
        if self.profile_id != id {
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
        let Viewer { open, fonts, .. } = self;
        let open = open.as_mut().ok_or("nothing is open")?;
        if !open.pages.contains_key(id) {
            let page = match &open.preview {
                // A frame of a drag: the linter is not run; the last findings stand.
                Some(doc) => {
                    let known = open.findings.get(id).cloned();
                    make_page(doc, &open.saved, id, fonts, Some(known.unwrap_or_default()))?
                }
                None => {
                    let page = make_page(&open.doc, &open.saved, id, fonts, None)?;
                    open.findings.insert(id.to_string(), page.findings.clone());
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

    /// The export file name for device `id` (napkin §11.2's default scheme,
    /// `{title}_{profile}_{width}x{height}.png`, slugified per §11.3).
    pub fn export_name(&self, profile: &Profile) -> String {
        let title = slug(
            self.title()
                .unwrap_or(match self.open.as_ref().map(|o| &o.source) {
                    Some(Source::Sample) => "sample",
                    _ => "untitled",
                }),
        );
        format!(
            "{title}_{}_{}x{}.png",
            profile_token(profile),
            profile.width_px,
            profile.height_px
        )
    }

    /// Writes PNGs into `dir`: the device on screen, or every device. Returns the files written.
    /// The bytes are the export renderer's (ADR-003), the same as rung 1's golden images.
    /// On failure the notice says why.
    pub fn export(&mut self, dir: &Path, which: ExportWhich) -> Result<Vec<PathBuf>, String> {
        let result = self.write_pngs(dir, which);
        if let Err(e) = &result {
            self.notice = Some(Notice {
                message: Message::new("notice-export-failed").text("error", e.clone()),
                error: true,
            });
        }
        result
    }

    fn write_pngs(&mut self, dir: &Path, which: ExportWhich) -> Result<Vec<PathBuf>, String> {
        let ids = match which {
            ExportWhich::ThisDevice => vec![self.profile_id.clone()],
            ExportWhich::AllDevices => self.profile_ids(),
        };
        let mut written = Vec::new();
        for id in ids {
            let page = self.page_for(&id)?.clone();
            let path = dir.join(self.export_name(&page.profile));
            std::fs::write(&path, encode_png(&page.image, None))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            written.push(path);
        }
        self.export_dir = Some(dir.to_path_buf());
        self.notice = Some(Notice {
            message: Message::new("notice-exported")
                .num("count", written.len() as f64, 0)
                .text("folder", dir.display().to_string()),
            error: false,
        });
        Ok(written)
    }

    /// Screen points per device pixel for the current zoom. `canvas` is the canvas size in points,
    /// `ppp` egui's physical pixels per point. `None` when nothing is open.
    pub fn scale(&mut self, canvas: (f32, f32), ppp: f32) -> Option<f32> {
        let zoom = self.zoom;
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
        }
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

fn make_page(
    doc: &Document,
    saved: &BTreeMap<String, ProfileOutlines>,
    id: &str,
    fonts: &mut FontRegistry,
    known_findings: Option<Vec<Finding>>,
) -> Result<Page, String> {
    let primary = builtin(&doc.primary_profile_id).map_err(|e| e.to_string())?;
    let target = builtin(id).map_err(|e| e.to_string())?;
    let laid = lay_out_text(doc, &primary, &target, fonts).map_err(|e| e.to_string())?;
    // Live layout is what export would make; when a font is missing here, the outlines the file
    // saved show the text as it was designed (ADR-001 item 2).
    let (outlines, saved_outlines) = match saved.get(id) {
        Some(s) if laid.missing_glyphs > 0 => (s.clone(), true),
        _ => (laid.outlines, false),
    };
    let dl = display_list(doc, &primary, &target, &outlines).map_err(|e| e.to_string())?;
    let image = render(&dl, Math::Fastest);
    let opts = Options {
        toolbars: TOOLBARS_IN_USE.to_vec(),
    };
    let findings = match known_findings {
        Some(f) => f,
        None => lint_with(doc, &primary, &target, &outlines, &opts),
    };
    let placed = placed_objects(doc, &primary, &target, &outlines);
    Ok(Page {
        profile: target,
        image,
        findings,
        saved_outlines,
        placed,
        display_list: dl,
    })
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

/// A device's name in file names: its profile's short name, made safe (`manta`, `nomad`).
pub fn profile_token(p: &Profile) -> String {
    slug(&p.short_name)
}

/// ASCII letters, digits, hyphens and underscores only (napkin §11.3): letters lowercased, any
/// other run of characters becomes one hyphen, none at either end. Never empty.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c.to_ascii_lowercase());
        } else {
            gap = true;
        }
    }
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
