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

//! The window's widgets (napkin §6.1, rung 2 bbq Q4): toolbar, canvas, side panels and status
//! bar. Rung 3 adds selecting on the canvas, the layers panel and inspector ([`crate::panels`]),
//! keys, Save and the prompts. Everything a person reads comes from `quadrule_i18n` (Q5).
//!
//! The widgets never open a system dialog themselves: they return a [`Request`], and the app
//! answers it (the file dialog lives in `quadrule-app`), so this crate runs headless in tests.

use crate::canvas::{
    CanvasMode, ChipRenderer, ChipState, DragClock, Fallback, ScreenView, screen_list,
};
use crate::drag::{
    DragKind, DragMods, HANDLE_SLACK_POINTS, HandleHit, ROTATE_STALK_POINTS, SNAP_REACH_POINTS,
};
use crate::editor::{PICK_SLACK_POINTS, Toward};
use crate::panels::{Renaming, Tab};
use crate::recovery::{Autosave, Recovered, Recovery};
use crate::tools::{BAR_WIDTH, Tool};
use crate::viewer::{
    CARD_WIDTH_MM, PIXEL_GRID_FROM, Source, TOOLBARS_SHOWN, Tile, Viewer, ZOOM_GESTURE_IDLE, Zoom,
    halve, ruler_px_per_mm, shrink, tiles,
};
use egui::{
    Color32, ColorImage, CornerRadius, CursorIcon, Key, KeyboardShortcut, Modifiers, PointerButton,
    Rect, ScrollArea, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, ThemePreference,
    pos2, vec2,
};
use quadrule_i18n::{Message, Strings};
use quadrule_layout::newpage::NewPage;
use quadrule_layout::snap::{Handle, Mark, PxRect, SnapKind, union_fill, union_outline};
use quadrule_layout::units::Axis;
use quadrule_lint::Toolbar;
use quadrule_model::Orientation;
use quadrule_model::blank::RhythmPreset;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Something the window asks the app to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Show the system dialog to pick a `.quad` file.
    OpenDialog,
    /// Show the system dialog to pick the Export sheet's folder (Change…), starting at
    /// [`ViewerUi::export_dialog_start`], then call [`ViewerUi::export_set_folder`] with the pick.
    ExportFolderDialog,
    /// Show this folder in the system's file manager (Reveal in folder on the completion sheet; see
    /// [`crate::export_ui::reveal_command`]); on failure call [`ViewerUi::reveal_failed`].
    RevealFolder(PathBuf),
    /// Show the system dialog to pick where to save, then call [`ViewerUi::save_as`] with the
    /// path and this follow-up.
    SaveAsDialog(Option<After>),
    /// Show the system dialog to pick a profile \`.ron\` file, then import it
    /// ([\`ViewerUi::profiles_import_path\`]).
    ProfileImportDialog,
    /// Show the system dialog to pick where to write the selected profile, with this suggested file
    /// name, then call [\`ViewerUi::profiles_export_to\`].
    ProfileExportDialog(String),
    /// Close the window: the user chose Save or Don't save at the unsaved-changes prompt.
    Close,
}

/// What to do once unsaved changes are dealt with (rung 3 bbq, Defaulted "Save").
#[derive(Clone, Debug, PartialEq)]
pub enum After {
    /// Show the Open dialog.
    OpenDialog,
    /// Open the sample page.
    OpenSample,
    /// Open this file (dropped on the window).
    OpenPath(PathBuf),
    /// Make a new blank document from the New dialog's choices (rung 5).
    NewPage(NewPage),
    /// Open a built-in starter (its `.quad` bytes) as an untitled copy on this device (rung 16).
    OpenStarter {
        /// The starter's file.
        bytes: std::sync::Arc<Vec<u8>>,
        /// The device that becomes the copy's main one.
        device_id: String,
    },
    /// Close the document and show the welcome screen (File > Close, rung 16).
    CloseDocument,
    /// Open a template as an untitled copy on this device (rung 16).
    OpenTemplate {
        /// The template's file.
        path: PathBuf,
        /// The device that becomes the copy's main one.
        device_id: String,
    },
    /// Close the window.
    Close,
}

/// Texture tiles are at most this many pixels a side.
const TILE: u32 = 1024;

/// The colour of what snaps to other objects (points, smart guides, equal gaps): an orange the
/// rhythm's magenta and the margins' teal do not share. Overlays are never exported.
const OBJECT_SNAP: Color32 = Color32::from_rgb(235, 100, 20);

/// What the page texture shows: which document, which device, at what size.
#[derive(Clone, Debug, PartialEq)]
struct TextureKey {
    generation: u64,
    revision: u64,
    profile: String,
    size: (u32, u32),
}

/// Which page the half-size copies are of: the document, its revision and the device.
type MipKey = (u64, u64, String);

/// How many copies the ladder has: full size, a half, a quarter and an eighth (the zoom's floor).
const MIP_LEVELS: usize = 4;

/// Uploads `image` as textures in tiles, because a Manta page (1920 × 2560) is larger than some graphics
/// chips take in one texture.
fn upload_tiles(
    ctx: &egui::Context,
    image: &quadrule_render::Gray,
    name: &str,
    options: TextureOptions,
) -> Vec<(Tile, TextureHandle)> {
    let (tw, th) = (image.width, image.height);
    tiles(tw, th, TILE)
        .into_iter()
        .map(|t| {
            let (x, y, tw_, th_) = t;
            let mut data = Vec::with_capacity((tw_ * th_) as usize);
            for row in y..y + th_ {
                let start = (row * tw + x) as usize;
                data.extend_from_slice(&image.data[start..start + tw_ as usize]);
            }
            let img = ColorImage::from_gray([tw_ as usize, th_ as usize], &data);
            (t, ctx.load_texture(format!("{name}-{x}-{y}"), img, options))
        })
        .collect()
}

/// The graphics-chip canvas's last image.
enum VectorTexture {
    /// Drawn on the chip into a texture the app owns.
    Chip(egui::TextureId),
    /// Drawn by vello_cpu.
    Cpu(TextureHandle),
}

/// The ruler check's references.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Reference {
    Card,
    Ruler100,
}

impl Reference {
    fn mm(self) -> f64 {
        match self {
            Reference::Card => CARD_WIDTH_MM,
            Reference::Ruler100 => 100.0,
        }
    }
}

/// The ruler check while it is open.
#[derive(Clone, Copy, Debug)]
struct Ruler {
    reference: Reference,
    /// The bar's length in points.
    bar: f32,
}

/// The viewer window: state plus what only the screen needs (the page texture, the dialogs).
pub struct ViewerUi {
    /// The viewer's state.
    pub viewer: Viewer,
    /// The words on screen.
    pub strings: Strings,
    /// Light, dark or follow the system (rung 2 bbq, Q11).
    pub theme: ThemePreference,
    /// Whether the right panel (inspector and lint) shows.
    pub show_lint: bool,
    /// Scroll the Style tab to its Border section once (`--look style` on the pattern-borders page, for a screenshot).
    pub look_border: bool,
    /// The inspector's tab.
    pub tab: Tab,
    /// Where unsaved changes are written while the window is open (the app sets it; `None` in
    /// tests that do not need it).
    pub recovery: Option<Recovery>,
    /// Which canvas shows the page (rung 3 bbq Q4, Q7): export pixels by default.
    pub canvas_mode: CanvasMode,
    /// The graphics-chip renderer, or why there is none (the app sets it).
    pub chip: Result<Box<dyn ChipRenderer>, String>,
    /// What the chip canvas has done this session, and whether it has fallen back.
    pub chip_state: ChipState,
    /// How long the frames of the last drag took (rung 4 bbq, Defaulted "Pi-smooth, the number").
    pub drag_clock: DragClock,
    /// A drag is being timed.
    clock_live: bool,
    /// Set when this frame made the page again (a new texture, or a new chip frame).
    rebuilt: bool,
    /// Where the page's top-left corner was drawn in screen points, and how many screen points a
    /// device pixel took, on the last frame (for tests that press the canvas).
    geom: Option<(egui::Pos2, f32)>,
    /// The drag in progress was begun by the pointer on the canvas (not by a test or the
    /// benchmark), so a release the canvas missed must still end it.
    pointer_drag: bool,
    /// A drag replayed through the real frame loop, to time it (`quadrule --bench-drag`).
    pub bench: Option<crate::bench::DragBench>,
    vector: Option<(String, VectorTexture)>,
    texture: Option<(TextureKey, Vec<(Tile, TextureHandle)>)>,
    /// The page and its half-size copies, drawn smoothed while a zoom gesture is going (fixes batch 1).
    mips: Option<(MipKey, Vec<((u32, u32), Vec<(Tile, TextureHandle)>)>)>,
    ruler: Option<Ruler>,
    pub(crate) scroll_to_selected: bool,
    cursor_px: Option<(f32, f32)>,
    canvas_size: Option<egui::Vec2>,
    pub(crate) selected_layer: Option<usize>,
    /// A device whose removal from the page waits on the user's answer (it has changes that would
    /// be lost; rung 8a).
    pub(crate) take_out: Option<String>,
    /// Ctrl + scroll just zoomed: the screen point of the pointer and the device pixel under it, so
    /// the next frame can scroll that pixel back under the pointer (rung 2 debt, paid in 8a).
    zoom_anchor: Option<(egui::Pos2, f32, f32)>,
    /// The anchor is to be applied on the next frame: the zoom just changed (fixes batch 1, Q1).
    zoom_anchor_due: bool,
    /// A zoom the speed bench feeds the canvas in place of the input's: the factor and the pointer.
    pub(crate) bench_zoom_step: Option<(f32, egui::Pos2)>,
    /// The previews strip's pictures, by device (rung 8a).
    pub(crate) strip_pics: HashMap<String, crate::strip::StripPic>,
    /// Where each strip picture's device leaves lines of a Fixed text box out: the box corners and
    /// the page size they are in (rung 11b).
    #[allow(clippy::type_complexity)]
    pub(crate) strip_overflows: HashMap<String, (Vec<(f64, f64)>, (u32, u32))>,
    /// Put the cursor in the Text tab's text field, with its text selected, on the next frame
    /// that shows it (a text box was just drawn).
    pub(crate) focus_text_content: bool,
    /// The canvas takes the keyboard focus on the next frame (a typing session just began).
    pub(crate) take_canvas_focus: bool,
    /// A pointer drag on the canvas is selecting text.
    type_selecting: bool,
    /// The canvas's widget id, to tell when something else takes the focus.
    canvas_id: Option<egui::Id>,
    /// The font picker window (rung 11a).
    pub(crate) font_picker: crate::fonts::FontPicker,
    /// The emoji picker window (rung 11a).
    pub(crate) emoji_picker: crate::emoji::EmojiPicker,
    /// The Missing fonts dialog, while it shows (rung 12a).
    pub(crate) missing_dialog: Option<crate::missing_ui::MissingDialog>,
    /// The Profiles window, while it shows (rung 13a).
    pub(crate) profiles_win: Option<crate::profiles_win::ProfilesWindow>,
    /// The dialog that asks Merge, Rename or Not now for a carried profile that clashes (rung 13a).
    pub(crate) clash_dialog: Option<crate::profiles_ui::ClashDialog>,
    /// The sheet that waits for an answer before Save, Save As or Export (rung 12a).
    pub(crate) font_sheet: Option<crate::missing_ui::Sheet>,
    /// The sheet's "Don't warn again for this document" tick.
    pub(crate) sheet_dont_warn: bool,
    /// The gray fields whose 0 to 255 slider the person opened with Show all grays (rung 14b; view state).
    pub(crate) show_all_grays: std::collections::HashSet<&'static str>,
    /// The preflight sheet before Export, while it shows (rung 14b).
    pub(crate) preflight: Option<crate::preflight::Preflight>,
    /// The Export sheet, while it shows (rung 15).
    pub(crate) export_sheet: Option<crate::export_ui::ExportSheet>,
    /// The completion sheet after an export from the Export sheet, while it shows (rung 15).
    pub(crate) completion: Option<crate::export_ui::Completion>,
    /// The device the Lint tab lists; `None` is All devices (rung 14b; view state, not saved).
    pub(crate) lint_device: Option<String>,
    /// The missing font the font picker is choosing a substitute for (Other font… in the dialog).
    pub(crate) font_picker_for: Option<String>,
    /// A pick the font picker made for a row of the dialog: (missing family, chosen family).
    pub(crate) picked_for_missing: Option<(String, String)>,
    /// The id of the Text tab's text field on the last frame, whose cursor an emoji goes to.
    pub(crate) text_field_id: Option<egui::Id>,
    /// The middle of what the canvas shows, in device pixels of the page on screen.
    pub(crate) view_centre_px: Option<(f64, f64)>,
    pub(crate) renaming: Option<Renaming>,
    /// The layer name field that just opened (New layer) starts with its text selected, to type over.
    pub(crate) rename_select_all: bool,
    /// The layer (index, name) a Delete layer sheet is asking about.
    pub(crate) delete_layer_ask: Option<(usize, String)>,
    /// Open the picked layer row's right-click menu on the next frame (`--look layers-menu`).
    pub(crate) look_layer_menu: bool,
    /// Groups folded shut in the layers tree (rung 7a).
    pub(crate) folded: std::collections::HashSet<quadrule_model::Uuid>,
    /// What is typed in a length field while it has focus, by field key.
    pub(crate) field_text: HashMap<String, String>,
    pub(crate) link_margins: bool,
    pub(crate) link_rhythm: bool,
    /// The welcome screen's state (rung 16): which tab, the Device filter, the Search box.
    pub(crate) welcome: crate::welcome::Welcome,
    /// The welcome screen's speed run, when `--bench-welcome` asked for one.
    pub welcome_bench: Option<crate::welcome::WelcomeBench>,
    /// Open the File menu on the next frame (`--look file-menu`).
    pub(crate) open_file_menu: bool,
    /// Waiting on the unsaved-changes prompt.
    pending: Option<After>,
    /// The New dialog, while it shows (rung 5).
    new_dialog: Option<NewPage>,
    /// The device the New dialog starts on: the last one picked (the app keeps it between runs).
    new_device: String,
    /// Recovery files found on start, offered one at a time.
    offers: Vec<Recovered>,
    autosave: Autosave,
    was_focused: bool,
    /// Set once the window has been told to close: no more autosaves, or the focus lost while
    /// closing would write back the recovery file "Don't save" just deleted.
    closing: bool,
}

impl ViewerUi {
    /// A window on the empty start screen.
    pub fn new(mut viewer: Viewer) -> Self {
        let strings = Strings::english();
        viewer.names = crate::draw::NewNames {
            line: strings.get("new-name-line"),
            text: strings.get("new-name-text"),
            pattern: strings.get("new-name-pattern"),
            rect: strings.get("new-name-rect"),
            ellipse: strings.get("new-name-ellipse"),
            polygon: strings.get("new-name-polygon"),
            text_content: strings.get("new-text-content"),
            group: strings.get("new-name-group"),
            repeat: strings.get("new-name-repeat"),
        };
        ViewerUi {
            viewer,
            strings,
            theme: ThemePreference::System,
            show_lint: true,
            look_border: false,
            tab: Tab::Page,
            recovery: None,
            canvas_mode: CanvasMode::ExportPixels,
            chip: Err("no graphics-chip renderer".into()),
            chip_state: ChipState::default(),
            drag_clock: DragClock::default(),
            clock_live: false,
            rebuilt: false,
            pointer_drag: false,
            geom: None,
            bench: None,
            vector: None,
            texture: None,
            mips: None,
            ruler: None,
            scroll_to_selected: false,
            cursor_px: None,
            canvas_size: None,
            selected_layer: None,
            take_out: None,
            zoom_anchor: None,
            zoom_anchor_due: false,
            bench_zoom_step: None,
            strip_pics: HashMap::new(),
            strip_overflows: HashMap::new(),
            focus_text_content: false,
            take_canvas_focus: false,
            type_selecting: false,
            canvas_id: None,
            renaming: None,
            rename_select_all: false,
            delete_layer_ask: None,
            look_layer_menu: false,
            folded: std::collections::HashSet::new(),
            field_text: HashMap::new(),
            link_margins: true,
            link_rhythm: true,
            welcome: Default::default(),
            welcome_bench: None,
            open_file_menu: false,
            pending: None,
            new_dialog: None,
            new_device: quadrule_model::sample::MANTA_ID.to_string(),
            offers: Vec::new(),
            autosave: Autosave::default(),
            was_focused: true,
            font_picker: Default::default(),
            emoji_picker: Default::default(),
            missing_dialog: None,
            clash_dialog: None,
            profiles_win: None,
            font_sheet: None,
            sheet_dont_warn: false,
            preflight: None,
            export_sheet: None,
            completion: None,
            show_all_grays: Default::default(),
            lint_device: None,
            font_picker_for: None,
            picked_for_missing: None,
            text_field_id: None,
            view_centre_px: None,
            closing: false,
        }
    }

    pub(crate) fn t(&self, id: &str) -> String {
        self.strings.get(id)
    }

    /// Draws the whole window into `ui` and returns what the app must do.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Request> {
        let frame_start = std::time::Instant::now();
        let mut requests = Vec::new();
        let ctx = ui.ctx().clone();
        // One notch of Ctrl + wheel zooms by a fixed fraction (fixes batch 1, Q1).
        ctx.options_mut(|o| {
            let io = &mut o.input_options;
            let speed = crate::viewer::ZOOM_WHEEL_NOTCH.ln() / io.line_scroll_speed;
            if io.scroll_zoom_speed != speed {
                io.scroll_zoom_speed = speed;
            }
        });
        // A step stays open to merging only while its field has focus or is being dragged.
        let idle = ctx.memory(|m| m.focused().is_none()) && !ctx.input(|i| i.pointer.any_down());
        if idle {
            self.viewer.seal();
        }
        self.take_dropped_files(&ctx, &mut requests);
        self.drive_bench(&ctx);
        self.shortcuts(&ctx, &mut requests);
        self.tick_autosave(&ctx);
        if std::mem::take(&mut self.viewer.focus_new_text) {
            // A text box just drawn is typed in at once, with its text selected (rung 11b, Q3); the
            // Text tab shows its style.
            self.tab = Tab::Text;
            if let Some(id) = self.viewer.selection.first().copied() {
                self.viewer.begin_typing(id, crate::typing::Start::All);
                self.take_canvas_focus = self.viewer.is_typing();
            }
        }
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui, &mut requests));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        // The welcome screen takes the window's middle; the document's panels wait behind it (rung 16, Q2).
        let in_document = self.viewer.open.is_some() && !self.welcome.over_document;
        // The tools bar on the left (chris, 2026-11-10: the left side was kept for it; rung 5).
        if in_document {
            egui::Panel::left("tools")
                .resizable(false)
                .exact_size(BAR_WIDTH)
                .show(ui, |ui| self.tools_bar(ui));
        }
        // One panel on the right: the inspector, with the layers below it.
        if self.show_lint && in_document {
            egui::Panel::right("inspector")
                .default_size(320.0)
                .show(ui, |ui| {
                    egui::Panel::bottom("layers")
                        .resizable(true)
                        .default_size(260.0)
                        .show(ui, |ui| self.layers_panel(ui));
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(ui, |ui| self.inspector(ui));
                });
        }
        // The device previews strip, along the bottom of the canvas (rung 8a, Q5).
        if in_document {
            self.refresh_strip(&ctx);
            egui::Panel::bottom("strip")
                .resizable(false)
                .show(ui, |ui| self.strip(ui));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(backdrop(ui.visuals())))
            .show(ui, |ui| {
                if in_document {
                    // The bar for missing fonts stands above the canvas (rung 12a, Q5).
                    if !self.viewer.bar_fonts().is_empty() {
                        egui::Panel::top("fonts-bar")
                            .frame(egui::Frame::NONE)
                            .show_separator_line(false)
                            .show(ui, |ui| self.font_bar(ui));
                    }
                    // The bar for profiles the file carries (rung 13a, Q4).
                    if self.viewer.profile_bar().is_some() {
                        egui::Panel::top("profiles-bar")
                            .frame(egui::Frame::NONE)
                            .show_separator_line(false)
                            .show(ui, |ui| self.profile_bar(ui));
                    }
                    self.canvas(ui);
                } else {
                    self.welcome_screen(ui, &mut requests);
                }
            });
        // Something else taking the keyboard (a field in the inspector) ends the typing.
        if self.viewer.is_typing()
            && let Some(f) = ctx.memory(|m| m.focused())
            && Some(f) != self.canvas_id
        {
            self.viewer.end_typing();
        }
        self.ruler_window(&ctx);
        self.font_picker_window(&ctx);
        self.missing_dialog_window(&ctx);
        self.clash_dialog_window(&ctx);
        self.profiles_window_ui(&ctx, &mut requests);
        self.emoji_picker_window(&ctx);
        // The Text tab, if it showed, has taken the cursor by now; a request it could not use
        // (no text box selected) must not wait for a later selection.
        self.focus_text_content = false;
        self.new_dialog(&ctx, &mut requests);
        self.welcome_dialogs(&ctx);
        self.unsaved_prompt(&ctx, &mut requests);
        self.delete_layer_sheet(&ctx);
        self.font_sheet_window(&ctx, &mut requests);
        self.preflight_window(&ctx);
        self.export_sheet_window(&ctx, &mut requests);
        self.completion_window(&ctx, &mut requests);
        self.recovery_prompt(&ctx);
        self.welcome_bench_frame(&ctx, frame_start.elapsed());
        requests
    }

    fn take_dropped_files(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if let Some(path) = dropped.first() {
            requests.extend(self.guard(After::OpenPath(path.clone())));
        }
    }

    /// Keys (rung 3 bbq Q9, Defaulted "Commands and undo", "Delete and Duplicate", "Save"). Ctrl
    /// is Cmd on macOS. While a text field has focus, only Save and Save As are taken; the field
    /// keeps its own undo, arrows and Delete.
    fn shortcuts(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        if self.pending.is_some() || self.new_dialog.is_some() || self.welcome_question_open() {
            return;
        }
        let exporting = self.export_sheets_open();
        let cmd = Modifiers::COMMAND;
        let shift_cmd = Modifiers::COMMAND | Modifiers::SHIFT;
        let key = |m: Modifiers, k: Key| {
            ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)))
        };
        // New, Open and Close (rung 16, Q6). New shows the welcome screen, which has its own Blank page; with it
        // showing already, Ctrl+N goes on to the Blank page dialog. None of them in the middle of a drag.
        if !self.viewer.dragging() {
            if key(cmd, Key::N) {
                self.ask_new();
                return;
            }
            if key(cmd, Key::O) {
                requests.extend(self.guard(After::OpenDialog));
                return;
            }
        }
        if self.welcome_showing() {
            // Esc goes back to the document the welcome screen is over.
            if self.welcome_over_document() && key(Modifiers::NONE, Key::Escape) {
                self.close_welcome();
            }
            return;
        }
        if self.viewer.open.is_none() {
            return;
        }
        // The Delete layer sheet has the keyboard: Esc cancels it (and goes no further), and no shortcut works round it.
        if self.delete_layer_ask.is_some() {
            return;
        }
        if !self.viewer.dragging() && !self.viewer.is_typing() && key(cmd, Key::W) {
            requests.extend(self.ask_close_document());
            return;
        }
        // Typing on the canvas takes the keyboard (rung 11b).
        if self.viewer.is_typing() {
            self.type_keys(ctx, requests);
            return;
        }
        // While an object is being dragged, only Escape counts: it drops the drag (rung 4 bbq,
        // Defaulted "One drag, one undo step").
        if self.viewer.dragging() {
            if key(Modifiers::NONE, Key::Escape) {
                self.viewer.drag_cancel();
            }
            return;
        }
        // Most specific first: Ctrl+Shift+S before Ctrl+S.
        if key(shift_cmd, Key::S) {
            self.ask(Request::SaveAsDialog(None), requests);
        } else if key(cmd, Key::S) {
            requests.extend(self.save());
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        // Export… (rung 16 bbq, Q9): one item; the sheet opens with the last choices and Export in focus.
        // Ctrl+Shift+E is free again: egui matches a Ctrl shortcut with Shift held too, so Shift must be up.
        if !exporting && !ctx.input(|i| i.modifiers.shift) && key(cmd, Key::E) {
            self.open_export_sheet();
        }
        if key(shift_cmd, Key::Z) || key(cmd, Key::Y) {
            self.viewer.redo();
        } else if key(cmd, Key::Z) {
            self.viewer.undo();
        }
        if key(cmd, Key::D) {
            let _ = self.viewer.duplicate_selection();
        }
        // Group and ungroup (rung 7a bbq, "Making and breaking"); the more specific first.
        if key(shift_cmd, Key::G) {
            let _ = self.viewer.ungroup_selection();
        } else if key(cmd, Key::G) {
            let _ = self.viewer.group_selection();
        }
        // Copy, cut and paste arrive as events (egui turns Ctrl+C, Ctrl+X and Ctrl+V, Cmd on
        // macOS, into them), and paste brings the clipboard's text (rung 4b).
        let (copy, cut, pasted) = ctx.input_mut(|i| {
            let (mut copy, mut cut, mut pasted) = (false, false, None);
            i.events.retain(|e| match e {
                egui::Event::Copy => {
                    copy = true;
                    false
                }
                egui::Event::Cut => {
                    cut = true;
                    false
                }
                egui::Event::Paste(text) => {
                    pasted = Some(text.clone());
                    false
                }
                _ => true,
            });
            (copy, cut, pasted)
        });
        if copy && let Some(text) = self.viewer.copy_text() {
            ctx.copy_text(text);
        }
        if cut && let Ok(Some(text)) = self.viewer.cut_text() {
            ctx.copy_text(text);
        }
        if let Some(text) = pasted {
            let _ = self.viewer.paste_text(&text);
        }
        if key(Modifiers::NONE, Key::Delete) || key(Modifiers::NONE, Key::Backspace) {
            let _ = self.viewer.delete_selection();
        }
        if key(cmd, Key::CloseBracket) {
            self.move_selected(Toward::Front);
        } else if key(cmd, Key::OpenBracket) {
            self.move_selected(Toward::Back);
        }
        // Escape ends a tool that is on first; with Select on, it clears the selection.
        // Inside a group, Escape first steps out of it (rung 7a bbq, Q4).
        // A name field open in the layers panel keeps its Escape: egui lets go of the field's focus on that key before
        // this runs, so taking it here would leave the field to read it as a click away and keep what was typed.
        if self.renaming.is_none()
            && key(Modifiers::NONE, Key::Escape)
            && !self.viewer.end_tool()
            && !self.viewer.leave_group()
        {
            self.viewer.selection.clear();
            self.selected_layer = None;
        }
        // V, L, T and P pick a tool (a text field with focus never gets here).
        for tool in Tool::ALL {
            if key(Modifiers::NONE, tool.key()) {
                self.viewer.set_tool(tool);
            }
        }
        if !self.viewer.selection.is_empty() {
            for (k, dx, dy) in [
                (Key::ArrowLeft, -1.0, 0.0),
                (Key::ArrowRight, 1.0, 0.0),
                (Key::ArrowUp, 0.0, -1.0),
                (Key::ArrowDown, 0.0, 1.0),
            ] {
                // Shift first: the plain key would also match Shift+key.
                let n_shift = ctx.input_mut(|i| i.count_and_consume_key(Modifiers::SHIFT, k));
                let n = ctx.input_mut(|i| i.count_and_consume_key(Modifiers::NONE, k));
                for _ in 0..n_shift {
                    let _ = self.viewer.nudge(dx, dy, true);
                }
                for _ in 0..n {
                    let _ = self.viewer.nudge(dx, dy, false);
                }
            }
        }
    }

    /// The keyboard while typing on the canvas (rung 11b, Q3): letters, arrows (Shift selects, Ctrl
    /// goes by words), Home and End, Backspace and Delete, Enter for a new paragraph, Ctrl+A, Ctrl+C,
    /// Ctrl+X and Ctrl+V as plain text, Ctrl+Z back through typed words, Escape to end. Input method
    /// events pass through. Only Save and Save As act on the document; both end the typing first.
    fn type_keys(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        use quadrule_text::edit::{Go, Op};
        let cmd = Modifiers::COMMAND;
        let shift_cmd = Modifiers::COMMAND | Modifiers::SHIFT;
        if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(shift_cmd, Key::S))) {
            self.viewer.end_typing();
            self.ask(Request::SaveAsDialog(None), requests);
            return;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::S))) {
            self.viewer.end_typing();
            requests.extend(self.save());
            return;
        }
        // Take the typing events; the rest (the pointer) stay for the canvas.
        let events: Vec<egui::Event> = ctx.input_mut(|i| {
            let mut taken = Vec::new();
            i.events.retain(|e| {
                let ours = matches!(
                    e,
                    egui::Event::Text(_)
                        | egui::Event::Key { .. }
                        | egui::Event::Copy
                        | egui::Event::Cut
                        | egui::Event::Paste(_)
                        | egui::Event::Ime(_)
                );
                if ours {
                    taken.push(e.clone());
                }
                !ours
            });
            taken
        });
        let had_events = !events.is_empty();
        for e in events {
            if !self.viewer.is_typing() {
                break;
            }
            match e {
                egui::Event::Text(t) => {
                    for c in t.chars().filter(|c| !c.is_control()) {
                        self.viewer.type_op(Op::Insert(c));
                    }
                }
                egui::Event::Copy => {
                    if let Some(t) = self.viewer.type_copy() {
                        ctx.copy_text(t);
                    }
                }
                egui::Event::Cut => {
                    if let Some(t) = self.viewer.type_cut() {
                        ctx.copy_text(t);
                    }
                }
                egui::Event::Paste(t) => self.viewer.type_str(&t),
                egui::Event::Ime(ime) => match ime {
                    egui::ImeEvent::Preedit { text, .. } => self.viewer.type_preedit(Some(text)),
                    egui::ImeEvent::Commit(t) => self.viewer.type_str(&t),
                    _ => self.viewer.type_preedit(None),
                },
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers: m,
                    ..
                } => {
                    let by = m.command || m.alt;
                    let go = |to| Op::Go {
                        to,
                        select: m.shift,
                    };
                    match key {
                        Key::Escape => self.viewer.end_typing(),
                        Key::ArrowLeft => {
                            self.viewer
                                .type_op(go(if by { Go::WordLeft } else { Go::Left }))
                        }
                        Key::ArrowRight => {
                            self.viewer
                                .type_op(go(if by { Go::WordRight } else { Go::Right }))
                        }
                        Key::ArrowUp => self.viewer.type_op(go(Go::Up)),
                        Key::ArrowDown => self.viewer.type_op(go(Go::Down)),
                        Key::Home => {
                            self.viewer
                                .type_op(go(if m.command { Go::Start } else { Go::Home }))
                        }
                        Key::End => {
                            self.viewer
                                .type_op(go(if m.command { Go::Finish } else { Go::End }))
                        }
                        Key::Backspace => self.viewer.type_op(Op::Backspace),
                        Key::Delete => self.viewer.type_op(Op::Delete),
                        Key::Enter => self.viewer.type_op(Op::Enter),
                        Key::A if m.command => self.viewer.type_op(Op::SelectAll),
                        Key::Z if m.command && m.shift => self.viewer.type_redo(),
                        Key::Z if m.command => self.viewer.type_undo(),
                        Key::Y if m.command => self.viewer.type_redo(),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if had_events {
            ctx.request_repaint();
        }
    }

    /// Save: to the document's file, or, when it has none (the sample page), by asking where. A page
    /// that uses installed fonts shows the sheet first (rung 12a, Q7); this is Save, never autosave.
    pub fn save(&mut self) -> Vec<Request> {
        if !self.viewer.font_sheet_fonts().is_empty() {
            self.font_sheet = Some(crate::missing_ui::Sheet::Save);
            return Vec::new();
        }
        self.save_now()
    }

    /// [`ViewerUi::save`] with the sheet answered.
    pub(crate) fn save_now(&mut self) -> Vec<Request> {
        match self.viewer.file_path().map(Path::to_path_buf) {
            Some(path) => {
                self.save_as(&path, None);
                Vec::new()
            }
            None => vec![Request::SaveAsDialog(None)],
        }
    }

    /// Saves to `path` (the app picked it in the Save As dialog), deletes the recovery file the
    /// document had, and then does `then`.
    pub fn save_as(&mut self, path: &Path, then: Option<After>) -> Vec<Request> {
        let before = self.viewer.open.as_ref().map(|o| o.source.clone());
        if self.viewer.save_to(path).is_err() {
            return Vec::new();
        }
        if let Some(rec) = &self.recovery {
            if let Some(b) = &before {
                rec.discard(b);
            }
            rec.discard(&Source::File(path.to_path_buf()));
        }
        then.map(|a| self.proceed(a)).unwrap_or_default()
    }

    /// Does `after` now, or, with unsaved changes, asks Save / Don't save / Cancel first.
    pub fn guard(&mut self, after: After) -> Vec<Request> {
        if self.viewer.is_dirty() {
            self.pending = Some(after);
            Vec::new()
        } else {
            self.proceed(after)
        }
    }

    /// The window was asked to close. True when it may close now; otherwise the
    /// unsaved-changes prompt shows and its answer comes back as [`Request::Close`].
    pub fn may_close(&mut self) -> bool {
        if self.viewer.is_dirty() {
            if self.pending.is_none() {
                self.pending = Some(After::Close);
            }
            false
        } else {
            true
        }
    }

    /// Whether the unsaved-changes prompt is showing.
    pub fn prompt_open(&self) -> bool {
        self.pending.is_some()
    }

    fn proceed(&mut self, after: After) -> Vec<Request> {
        match after {
            After::OpenDialog => vec![Request::OpenDialog],
            After::OpenSample => {
                self.viewer.open_sample();
                self.after_open();
                Vec::new()
            }
            After::NewPage(spec) => {
                if self.viewer.open_new(&spec).is_ok() {
                    self.after_open();
                }
                Vec::new()
            }
            After::OpenPath(p) => {
                if self.viewer.open_path(&p).is_ok() {
                    self.after_open();
                }
                Vec::new()
            }
            After::OpenTemplate { path, device_id } => {
                if self.viewer.open_template(&path, &device_id).is_ok() {
                    self.after_open();
                }
                Vec::new()
            }
            After::OpenStarter { bytes, device_id } => {
                if self.viewer.open_copy(&bytes, &device_id).is_ok() {
                    self.after_open();
                }
                Vec::new()
            }
            After::CloseDocument => {
                self.viewer.close_document();
                self.after_open();
                self.show_welcome();
                Vec::new()
            }
            After::Close => {
                self.closing = true;
                vec![Request::Close]
            }
        }
    }

    /// Opens a file picked in the Open dialog.
    pub fn open_file(&mut self, path: &Path) {
        if self.viewer.open_path(path).is_ok() {
            self.after_open();
        }
    }

    fn after_open(&mut self) {
        self.welcome.over_document = false;
        self.selected_layer = None;
        self.renaming = None;
        self.field_text.clear();
        self.tab = Tab::Page;
    }

    fn unsaved_prompt(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        let Some(after) = self.pending.clone() else {
            return;
        };
        let name = self.document_name();
        let mut choice = None;
        egui::Modal::new(egui::Id::unique("unsaved")).show(ctx, |ui| {
            ui.heading(self.t("app-unsaved-title"));
            ui.label(
                self.strings
                    .format(&Message::new("app-unsaved-body").text("name", name)),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-unsaved-save")).clicked() {
                    choice = Some(0);
                }
                if ui.button(self.t("app-unsaved-discard")).clicked() {
                    choice = Some(1);
                }
                if ui.button(self.t("app-cancel")).clicked() {
                    choice = Some(2);
                }
            });
        });
        match choice {
            Some(0) if !self.viewer.font_sheet_fonts().is_empty() => {
                self.pending = None;
                self.font_sheet = Some(crate::missing_ui::Sheet::SaveThen(after));
            }
            Some(0) => {
                self.pending = None;
                match self.viewer.file_path().map(Path::to_path_buf) {
                    Some(p) => requests.extend(self.save_as(&p, Some(after))),
                    None => requests.push(Request::SaveAsDialog(Some(after))),
                }
            }
            Some(1) => {
                self.pending = None;
                if let (Some(rec), Some(o)) = (&self.recovery, &self.viewer.open) {
                    rec.discard(&o.source);
                }
                requests.extend(self.proceed(after));
            }
            Some(_) => self.pending = None,
            None => {}
        }
    }

    /// Shows the New dialog, on the device picked last time and the handwriting rhythm.
    pub fn open_new_dialog(&mut self) {
        self.new_dialog = Some(NewPage {
            title: self.t("new-default-title"),
            device_id: self.new_device.clone(),
            rhythm: RhythmPreset::Handwriting,
            orientation: quadrule_model::Orientation::Portrait,
        });
    }

    /// The device the New dialog starts on.
    pub fn new_device(&self) -> &str {
        &self.new_device
    }

    /// Sets the device the New dialog starts on (the app's remembered choice). A device this
    /// build does not know is ignored.
    pub fn set_new_device(&mut self, id: &str) {
        if quadrule_profiles::active::device_ids()
            .iter()
            .chain(quadrule_profiles::active::paper_ids().iter())
            .any(|d| d == id)
        {
            self.new_device = id.to_string();
        }
    }

    /// Whether the New dialog is showing.
    pub fn new_dialog_open(&self) -> bool {
        self.new_dialog.is_some()
    }

    /// The New dialog (rung 5 bbq, Defaulted "New dialog fields"): device, rhythm and title.
    /// Create asks to leave unsaved changes first, then opens the blank page.
    fn new_dialog(&mut self, ctx: &egui::Context, requests: &mut Vec<Request>) {
        let Some(mut spec) = self.new_dialog.take() else {
            return;
        };
        if self.pending.is_some() {
            self.new_dialog = Some(spec);
            return;
        }
        let mut create = false;
        let modal = egui::Modal::new(egui::Id::unique("new-page")).show(ctx, |ui| {
            ui.set_width(320.0);
            ui.heading(self.t("new-heading"));
            ui.add_space(8.0);
            egui::Grid::new("new-fields").num_columns(2).show(ui, |ui| {
                // Each control is named for screen readers by the label beside it.
                let label = ui.label(self.t("app-device"));
                self.device_picker(ui, "new-device", &mut spec.device_id)
                    .labelled_by(label.id);
                ui.end_row();
                let label = ui.label(self.t("new-rhythm"));
                egui::ComboBox::from_id_salt("new-rhythm")
                    .selected_text(self.t(spec.rhythm.message_id()))
                    .show_ui(ui, |ui| {
                        for preset in RhythmPreset::ALL {
                            ui.selectable_value(
                                &mut spec.rhythm,
                                preset,
                                self.t(preset.message_id()),
                            );
                        }
                    })
                    .response
                    .labelled_by(label.id);
                ui.end_row();
                let label = ui.label(self.t("new-orientation"));
                egui::ComboBox::from_id_salt("new-orientation")
                    .selected_text(self.t(orientation_message(spec.orientation)))
                    .show_ui(ui, |ui| {
                        for o in [Orientation::Portrait, Orientation::Landscape] {
                            ui.selectable_value(
                                &mut spec.orientation,
                                o,
                                self.t(orientation_message(o)),
                            );
                        }
                    })
                    .response
                    .labelled_by(label.id);
                ui.end_row();
                let label = ui.label(self.t("new-title"));
                ui.add(egui::TextEdit::singleline(&mut spec.title))
                    .labelled_by(label.id);
                ui.end_row();
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("new-create")).clicked() {
                    create = true;
                }
                if ui.button(self.t("app-cancel")).clicked() {
                    ui.close();
                }
            });
        });
        if create {
            if spec.title.trim().is_empty() {
                spec.title = self.t("new-default-title");
            }
            self.new_device = spec.device_id.clone();
            requests.extend(self.guard(After::NewPage(spec)));
        } else if !modal.should_close() {
            self.new_dialog = Some(spec);
        }
    }

    /// Looks for recovery files left by a crash and offers each back (napkin §5.6).
    pub fn offer_recovery(&mut self) {
        if let Some(rec) = &self.recovery {
            self.offers = rec.list();
        }
    }

    /// Whether a recovery offer is showing.
    pub fn recovery_open(&self) -> bool {
        !self.offers.is_empty()
    }

    fn recovery_prompt(&mut self, ctx: &egui::Context) {
        if self.pending.is_some() {
            return;
        }
        let Some(r) = self.offers.first().cloned() else {
            return;
        };
        let name = match &r.source {
            Source::File(p) => p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            Source::Sample => self.t("app-sample-name"),
            Source::Untitled => self.t("app-untitled-name"),
        };
        let mut choice = None;
        egui::Modal::new(egui::Id::unique("recover")).show(ctx, |ui| {
            ui.heading(self.t("app-recover-title"));
            ui.label(
                self.strings
                    .format(&Message::new("app-recover-body").text("name", name)),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(self.t("app-recover-restore")).clicked() {
                    choice = Some(true);
                }
                if ui.button(self.t("app-recover-discard")).clicked() {
                    choice = Some(false);
                }
            });
        });
        let Some(restore) = choice else {
            return;
        };
        self.offers.remove(0);
        let Some(rec) = self.recovery.clone() else {
            return;
        };
        if restore {
            if let Err(e) = rec.restore(&r, &mut self.viewer) {
                self.viewer.notice = Some(crate::viewer::Notice {
                    message: Message::new("notice-open-failed")
                        .text("path", r.quad.display().to_string())
                        .text("error", e),
                    error: true,
                });
            } else {
                self.after_open();
                // Only one document is open at a time: further offers wait for the next start.
                self.offers.clear();
            }
        } else {
            rec.forget(&r);
        }
    }

    /// Writes the recovery file every 30 seconds while there are unsaved changes, and when the
    /// window loses focus (rung 3 bbq, Defaulted "Autosave").
    fn tick_autosave(&mut self, ctx: &egui::Context) {
        let (now, focused) = ctx.input(|i| (i.time, i.viewport().focused.unwrap_or(true)));
        let lost_focus = self.was_focused && !focused;
        let gained_focus = !self.was_focused && focused;
        self.was_focused = focused;
        if gained_focus {
            // The installed fonts may have changed while the window was in the background. The scan
            // runs off this thread; the list swaps in when it is done (rung 11b).
            self.viewer.start_font_scan();
        }
        if self.viewer.font_scan_running() {
            if self.viewer.poll_font_scan().is_none() {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            } else {
                ctx.request_repaint();
            }
        }
        if self.closing {
            return;
        }
        let Some(rec) = self.recovery.clone() else {
            return;
        };
        if self.autosave.due(&self.viewer, now, lost_focus)
            && let Err(e) = rec.write(&mut self.viewer)
        {
            self.viewer.notice = Some(crate::viewer::Notice {
                message: Message::new("notice-recovery-failed").text("error", e),
                error: true,
            });
        }
        if self.viewer.is_dirty() {
            // Wake up for the next autosave even if nothing else happens.
            ctx.request_repaint_after(std::time::Duration::from_secs(5));
        }
    }

    /// The open document's name for the title bar and prompts: its file name, or "Sample page".
    pub fn document_name(&self) -> String {
        match self.viewer.open.as_ref().map(|o| &o.source) {
            Some(Source::File(p)) => p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            Some(Source::Sample) => self.t("app-sample-name"),
            Some(Source::Untitled) => self.t("app-untitled-name"),
            None => String::new(),
        }
    }

    /// The window title: the file name, with a dot when there are unsaved changes.
    pub fn window_title(&self) -> String {
        if self.viewer.open.is_none() {
            return self.t("app-title");
        }
        let id = if self.viewer.is_dirty() {
            "app-window-title-dirty"
        } else {
            "app-window-title"
        };
        self.strings
            .format(&Message::new(id).text("name", self.document_name()))
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.horizontal_wrapped(|ui| {
            self.file_menu(ui, requests);
            // What the document in view lets the rest of the bar do; the welcome screen over a document hides it.
            let open = self.in_document();
            ui.separator();
            let can_undo = open
                && self
                    .viewer
                    .open
                    .as_ref()
                    .is_some_and(|o| o.history.can_undo());
            let can_redo = open
                && self
                    .viewer
                    .open
                    .as_ref()
                    .is_some_and(|o| o.history.can_redo());
            if ui
                .add_enabled(can_undo, egui::Button::new(self.t("app-undo")))
                .clicked()
            {
                self.viewer.undo();
            }
            if ui
                .add_enabled(can_redo, egui::Button::new(self.t("app-redo")))
                .clicked()
            {
                self.viewer.redo();
            }
            ui.separator();
            self.emoji_button(ui);
            ui.separator();

            // Device (the welcome screen has its own filter).
            if !self.welcome_showing() {
                let ids = self.viewer.profile_ids();
                let name = |id: &str| self.full_name(id);
                let current = self.viewer.profile_id.clone();
                let mut chosen = current.clone();
                // Named for screen readers (napkin §14, AccessKit) by the label beside it.
                let label = ui.label(self.t("app-device"));
                egui::ComboBox::from_id_salt("profile")
                    .selected_text(name(&current))
                    .show_ui(ui, |ui| {
                        for id in &ids {
                            ui.selectable_value(&mut chosen, id.clone(), name(id));
                        }
                    })
                    .response
                    .labelled_by(label.id);
                if chosen != current {
                    self.viewer.set_profile(&chosen);
                }
            }
            // The menu entry for the Profiles window (rung 13a, Q5).
            if ui
                .button(self.t("profiles-open"))
                .on_hover_text(self.t("profiles-open-tip"))
                .clicked()
            {
                self.open_profiles_window();
            }
            ui.separator();

            // Zoom (rung 2 bbq, Q10).
            ui.add_enabled_ui(open, |ui| {
                if ui.button(self.t("app-zoom-fit")).clicked() {
                    self.viewer.zoom = Zoom::Fit;
                }
                if ui.button(self.t("app-zoom-actual")).clicked() {
                    self.viewer.zoom = Zoom::Scale(1.0);
                }
                let now = self.shown_scale(ui);
                if ui
                    .button("−")
                    .on_hover_text(self.t("app-zoom-out"))
                    .clicked()
                {
                    self.viewer.zoom_step(now, -1);
                }
                if ui
                    .button("+")
                    .on_hover_text(self.t("app-zoom-in"))
                    .clicked()
                {
                    self.viewer.zoom_step(now, 1);
                }
                if ui.button(self.t("app-zoom-real")).clicked() {
                    if self.viewer.screen_px_per_mm.is_some() {
                        self.viewer.zoom = Zoom::RealSize;
                    } else {
                        self.open_ruler(ui.ctx());
                    }
                }
            });
            ui.separator();

            // Lint count, which shows or hides the panel.
            let count = self.lint_count();
            let label =
                self.strings
                    .format(&Message::new("app-lint-count").num("count", count as f64, 0));
            let on = self.show_lint && self.tab == Tab::Lint;
            if ui
                .add_enabled(open, egui::Button::selectable(on, label))
                .clicked()
            {
                // Shows the Lint tab. It never hides the right panel, which holds the layers too.
                self.show_lint = true;
                self.tab = Tab::Lint;
            }
            ui.separator();

            // View.
            ui.menu_button(self.t("app-view"), |ui| {
                ui.label(self.t("app-theme"));
                for (pref, id) in [
                    (ThemePreference::System, "app-theme-system"),
                    (ThemePreference::Light, "app-theme-light"),
                    (ThemePreference::Dark, "app-theme-dark"),
                ] {
                    if ui.radio(self.theme == pref, self.t(id)).clicked() {
                        self.theme = pref;
                        ui.ctx().set_theme(pref);
                    }
                }
                ui.separator();
                let label = self.t("app-safe-zones");
                ui.checkbox(&mut self.viewer.show_safe_zones, label);
                ui.separator();
                ui.label(self.t("app-canvas"));
                for (mode, id) in [
                    (CanvasMode::ExportPixels, "app-canvas-export"),
                    (CanvasMode::Chip, "app-canvas-chip"),
                ] {
                    if ui.radio(self.canvas_mode == mode, self.t(id)).clicked() {
                        self.canvas_mode = mode;
                        self.vector = None;
                    }
                }
                if ui.button(self.t("app-ruler-open")).clicked() {
                    self.open_ruler(ui.ctx());
                }
            });
        });
    }

    fn open_ruler(&mut self, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point();
        // Start from what the system claims (96 points per inch), so the bar is near right.
        let bar = match self.viewer.screen_px_per_mm {
            Some(r) => (r * CARD_WIDTH_MM) as f32 / ppp,
            None => (CARD_WIDTH_MM / quadrule_model::MM_PER_IN * 96.0) as f32,
        };
        self.ruler = Some(Ruler {
            reference: Reference::Card,
            bar,
        });
    }

    /// Physical screen pixels per device pixel now showing.
    fn shown_scale(&mut self, ui: &egui::Ui) -> f32 {
        // The canvas's own size is known once it has been drawn (the toolbar comes first in a
        // frame); until then, the window's.
        let ppp = ui.ctx().pixels_per_point();
        let canvas = self
            .canvas_size
            .unwrap_or_else(|| ui.ctx().content_rect().size());
        self.viewer
            .scale((canvas.x, canvas.y), ppp)
            .map_or(1.0, |s| s * ppp)
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.in_document()
                && let Some((x, y)) = self.cursor_px
                && let Some(page) = self.viewer.page()
            {
                let mm = |px: f32| px as f64 * quadrule_model::MM_PER_IN / page.profile.ppi;
                let msg = Message::new("app-status-cursor")
                    .num("x", x.floor() as f64, 0)
                    .num("y", y.floor() as f64, 0)
                    .num("xmm", mm(x), 1)
                    .num("ymm", mm(y), 1);
                ui.label(self.strings.format(&msg));
                ui.separator();
            }
            if self.in_document() {
                let pct = self.shown_scale(ui) * 100.0;
                let msg = Message::new("app-status-zoom").num("percent", pct as f64, 0);
                ui.label(self.strings.format(&msg));
                ui.separator();
                let count = self.lint_count();
                let msg = Message::new("app-lint-count").num("count", count as f64, 0);
                ui.label(self.strings.format(&msg));
            }
            if self.in_document() {
                // The snap helpers and the rhythm overlay, each on or off (napkin §6.2).
                ui.separator();
                self.snap_toggles(ui);
            }
            if let Some(line) = self.canvas_status() {
                ui.separator();
                ui.label(line);
            }
            if self.in_document() && self.viewer.page().is_some_and(|p| p.repeats_capped > 0) {
                ui.separator();
                let msg = Message::new("app-status-repeat-capped").num(
                    "max",
                    quadrule_model::REPEAT_COPIES_MAX as f64,
                    0,
                );
                ui.label(self.strings.format(&msg));
            }
            if self.drag_clock.frames > 0 {
                ui.separator();
                let c = &self.drag_clock;
                let msg = Message::new("app-drag-clock")
                    .num("mean", c.mean_ms(), 0)
                    .num("p90", c.percentile_ms(), 0)
                    .num("worst", c.worst_ms, 0)
                    .num("frames", f64::from(c.frames), 0);
                ui.label(self.strings.format(&msg));
            }
            if let Some(n) = &self.viewer.notice {
                ui.separator();
                let text = self.strings.format(&n.message);
                if n.error {
                    ui.colored_label(ui.visuals().error_fg_color, text);
                } else {
                    ui.label(text);
                }
            }
        });
    }

    /// The snap helpers' switches and the rhythm overlay's, for the status bar and the Page tab.
    /// They are app settings, kept between runs, not saved in the document.
    pub(crate) fn snap_toggles(&mut self, ui: &mut egui::Ui) {
        let rows = [
            ("app-snap-pixel", 0),
            ("app-snap-rhythm", 1),
            ("app-snap-margins", 2),
            ("app-snap-points", 3),
            ("app-snap-smart", 4),
            ("app-snap-equal", 5),
            ("app-rhythm-overlay", 6),
        ];
        for (id, which) in rows {
            let label = self.t(id);
            let flag = match which {
                0 => &mut self.viewer.snap.pixel_grid,
                1 => &mut self.viewer.snap.rhythm,
                2 => &mut self.viewer.snap.margins_safe,
                3 => &mut self.viewer.snap.points,
                4 => &mut self.viewer.snap.smart_guides,
                5 => &mut self.viewer.snap.equal_spacing,
                _ => &mut self.viewer.show_rhythm,
            };
            ui.checkbox(flag, label);
        }
    }

    pub(crate) fn lint_count(&mut self) -> usize {
        self.viewer.page().map_or(0, |p| p.findings.len())
    }

    /// Replays a drag of the sample page's dot grid through the real frame loop and prints its
    /// timing as a `CHECK` line (`quadrule --bench-drag`; rung 4 bbq, Defaulted "Pi-smooth, the
    /// number"). One step per frame, so the canvas does the work it does for a person dragging.
    fn drive_bench(&mut self, ctx: &egui::Context) {
        use crate::bench::Phase;
        let Some(mut b) = self.bench.take() else {
            return;
        };
        ctx.request_repaint();
        match b.phase {
            Phase::Warm(n) => {
                if self.viewer.open.is_none() {
                    if b.missing_fonts {
                        self.viewer.open_missing_fonts();
                    } else if b.textbox {
                        self.viewer.open_textbox();
                    } else if b.fonts {
                        self.viewer.open_fonts();
                    } else if b.lint {
                        self.viewer.open_lint();
                    } else if b.paper {
                        self.viewer.open_paper();
                    } else if b.bordered {
                        self.viewer.open_bordered();
                    } else if b.pattern_angle {
                        self.viewer.open_pattern_angle();
                    } else if b.turned_patterns {
                        self.viewer.open_turned_patterns();
                    } else if b.repeats {
                        self.viewer.open_repeats();
                    } else if b.patterns {
                        self.viewer.open_patterns();
                    } else if b.landscape {
                        self.viewer.open_landscape();
                    } else if b.devices {
                        self.viewer.open_devices();
                    } else if b.groups {
                        self.viewer.open_groups();
                    } else if b.transforms {
                        self.viewer.open_transforms();
                    } else if b.shapes {
                        self.viewer.open_shapes();
                    } else {
                        self.viewer.open_sample();
                    }
                }
                // The strip is open in a bench unless asked otherwise, whatever the settings say.
                self.viewer.show_strip = !b.no_strip;
                if n == 0 && b.secondary {
                    let ids = self.viewer.profile_ids();
                    if let Some(second) = ids.get(1) {
                        self.viewer.set_profile(second);
                    }
                }
                b.phase = if n >= crate::bench::WARM_FRAMES {
                    Phase::Begin
                } else {
                    Phase::Warm(n + 1)
                };
            }
            Phase::Begin if b.zoom => {
                // The zoom run starts a quarter of the way down, with no gesture going.
                self.viewer.zoom = Zoom::Scale(crate::bench::zoom_target(0, b.frames));
                self.viewer.zoom_gesture = None;
                b.phase = Phase::Run(0);
            }
            Phase::Run(i) if b.zoom && i < b.frames => {
                // A pinch a frame, by the factor that takes the scale to where the schedule has it
                // next, at a pointer that wanders a little round the middle of the window.
                let now = crate::bench::zoom_target(i, b.frames);
                let next = crate::bench::zoom_target(i + 1, b.frames);
                let c = ctx.content_rect().center();
                let t = f64::from(i);
                let pointer = c + vec2(
                    120.0 * (0.21 * t).sin() as f32,
                    80.0 * (0.13 * t).cos() as f32,
                );
                self.bench_zoom_step = Some((next / now, pointer));
                b.phase = Phase::Run(i + 1);
            }
            Phase::Begin => {
                // The dot grid, or for the rotate run the big turned rectangle.
                let target = if b.rotate && b.turned_patterns {
                    quadrule_model::sample::turned_ids::TURNED
                } else if b.rotate {
                    quadrule_model::sample::transform_ids::TURNED
                } else if b.groups {
                    // A cell of the heavy group: the press picks the group it is in.
                    quadrule_model::sample::group_ids::nth(120)
                } else if b.repeats {
                    // The habit tracker: the press picks the repeat, and the drag moves every copy.
                    quadrule_model::sample::repeat_ids::HABIT
                } else {
                    quadrule_model::sample::ids::DOTS
                };
                let at = self.viewer.page().and_then(|p| {
                    p.placed
                        .iter()
                        .find(|s| s.id == target)
                        .map(|s| s.shape.bounds())
                });
                b.phase = match at {
                    Some((x0, y0, x1, y1)) => {
                        b.centre = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                        let none = DragMods::default();
                        let began = if b.typing {
                            // A long box, and the caret at its end; the steps type into it.
                            match self.viewer.place_long_text() {
                                Some(id) => self.viewer.begin_typing(id, crate::typing::Start::End),
                                None => false,
                            }
                        } else if b.rotate {
                            // Press on the rotate handle of the selected rectangle.
                            self.viewer.selection = vec![target];
                            self.viewer.rotate_stalk_px = ROTATE_STALK_POINTS / 0.3;
                            match self.viewer.rotate_handle() {
                                Some(h) => self.viewer.drag_begin(h.tip.0, h.tip.1, none, 4.0, 6.0),
                                None => false,
                            }
                        } else if b.draw {
                            // From the page's top-left corner, out into the page.
                            b.centre = (150.0, 150.0);
                            self.viewer.set_tool(b.tool);
                            self.viewer.draw_begin(150.0, 150.0, none, 6.0)
                        } else {
                            self.viewer
                                .drag_begin(b.centre.0, b.centre.1, none, 4.0, 6.0)
                        };
                        if began { Phase::Run(0) } else { Phase::Done }
                    }
                    None => Phase::Done,
                };
            }
            Phase::Run(i) if i < b.frames => {
                let t = f64::from(i);
                let (cx, cy) = b.centre;
                if b.typing {
                    // A letter a step; every twelfth is a space, every sixtieth a new paragraph.
                    let op = if i % 60 == 59 {
                        quadrule_text::edit::Op::Enter
                    } else if i % 12 == 11 {
                        quadrule_text::edit::Op::Insert(' ')
                    } else {
                        quadrule_text::edit::Op::Insert(char::from(b'a' + (i % 26) as u8))
                    };
                    self.viewer.type_op(op);
                    b.phase = Phase::Run(i + 1);
                    self.bench = Some(b);
                    return;
                }
                let (x, y) = if b.rotate {
                    // The pointer swings round the rectangle's centre, a quarter turn each way.
                    let a = (-90.0 + 70.0 * (0.21 * t).sin()).to_radians();
                    (cx + 600.0 * a.cos(), cy + 600.0 * a.sin())
                } else if b.draw {
                    // The far corner sweeps back and forth over most of the page.
                    (
                        150.0 + 900.0 + 800.0 * (0.31 * t).sin(),
                        150.0 + 1200.0 + 1000.0 * (0.17 * t).sin(),
                    )
                } else {
                    (cx + 150.0 * (0.37 * t).sin(), cy + 120.0 * (0.23 * t).cos())
                };
                let scale = self
                    .canvas_size
                    .and_then(|c| self.viewer.scale((c.x, c.y), ctx.pixels_per_point()))
                    .unwrap_or(0.3);
                let reach = SNAP_REACH_POINTS / f64::from(scale);
                if b.draw {
                    self.viewer.draw_update(x, y, DragMods::default(), reach);
                } else {
                    self.viewer.drag_update(x, y, DragMods::default(), reach);
                }
                b.phase = Phase::Run(i + 1);
            }
            Phase::Run(_) => {
                if b.zoom {
                    self.viewer.zoom = Zoom::Fit;
                    self.viewer.zoom_gesture = None;
                } else if b.typing {
                    // The text typed is one step; the box and its text are taken away again.
                    self.viewer.end_typing();
                    self.viewer.undo();
                    self.viewer.undo();
                } else {
                    self.viewer.gesture_end();
                    self.viewer.undo();
                }
                let c = self.drag_clock.clone();
                let canvas = match (self.canvas_mode, &self.chip_state.fallback) {
                    (CanvasMode::ExportPixels, _) => "export",
                    (CanvasMode::Chip, None) => "chip",
                    (CanvasMode::Chip, Some(_)) => "chip-cpu-fallback",
                };
                // The 50 ms line is for a release build on the Pi; a debug build is far slower, so
                // it only reports.
                let verdict = if cfg!(debug_assertions) {
                    "n/a"
                } else if c.within_budget() {
                    "pass"
                } else {
                    "fail"
                };
                println!(
                    "\nCHECK crate=ui name={} result={verdict} build={} canvas={canvas} page={} device={} strip={} tool={} frames={} mean_ms={:.1} p90_ms={:.1} worst_ms={:.1} first_ms={:.1} over_budget={} budget_ms={}",
                    if b.zoom {
                        "zoom_bench"
                    } else if b.typing {
                        "type_bench"
                    } else if b.rotate {
                        "rotate_bench"
                    } else if b.draw {
                        "draw_bench"
                    } else {
                        "drag_bench"
                    },
                    if cfg!(debug_assertions) {
                        "debug"
                    } else {
                        "release"
                    },
                    if b.missing_fonts {
                        "missing_fonts"
                    } else if b.textbox {
                        "textbox"
                    } else if b.fonts {
                        "fonts"
                    } else if b.lint {
                        "lint"
                    } else if b.paper {
                        "paper"
                    } else if b.bordered {
                        "bordered"
                    } else if b.pattern_angle {
                        "pattern_angle"
                    } else if b.turned_patterns {
                        "turned_patterns"
                    } else if b.repeats {
                        "repeats"
                    } else if b.patterns {
                        "patterns"
                    } else if b.landscape {
                        "landscape"
                    } else if b.devices {
                        "devices"
                    } else if b.groups {
                        "groups"
                    } else if b.transforms {
                        "transforms"
                    } else if b.shapes {
                        "shapes"
                    } else {
                        "sample"
                    },
                    if self.viewer.override_device().is_some() {
                        "secondary"
                    } else {
                        "primary"
                    },
                    if self.viewer.show_strip { "on" } else { "off" },
                    if b.zoom {
                        "zoom"
                    } else if b.typing {
                        "type"
                    } else if b.rotate {
                        "rotate"
                    } else if b.draw {
                        b.tool.cli_name()
                    } else {
                        "move"
                    },
                    c.frames,
                    c.mean_ms(),
                    c.percentile_ms(),
                    c.worst_ms,
                    c.first_ms,
                    c.over_budget,
                    DragClock::BUDGET_MS
                );
                b.phase = Phase::Done;
            }
            Phase::Done => {}
        }
        self.bench = Some(b);
    }

    /// Whether the layers tree shows group `id` folded shut (rung 7a).
    pub fn is_folded(&self, id: quadrule_model::Uuid) -> bool {
        self.folded.contains(&id)
    }

    /// The screen point of device pixel (x, y) of the page on the last frame, for tests that press
    /// the canvas with a pointer.
    pub fn screen_point(&self, x: f64, y: f64) -> Option<egui::Pos2> {
        let (min, scale) = self.geom?;
        Some(pos2(min.x + x as f32 * scale, min.y + y as f32 * scale))
    }

    /// Whether the benchmark has run to the end.
    pub fn bench_finished(&self) -> bool {
        self.welcome_bench_finished()
            || self
                .bench
                .as_ref()
                .is_some_and(|b| b.phase == crate::bench::Phase::Done)
    }

    /// The canvas, timed: while an object is dragged, each frame that made the page again is
    /// added to [`ViewerUi::drag_clock`].
    fn canvas(&mut self, ui: &mut egui::Ui) {
        let start = std::time::Instant::now();
        self.rebuilt = false;
        self.canvas_frame(ui);
        let zooming = self
            .bench
            .as_ref()
            .is_some_and(|b| b.zoom && matches!(b.phase, crate::bench::Phase::Run(_)));
        if self.viewer.dragging() || self.viewer.is_typing() || zooming {
            if !self.clock_live {
                self.clock_live = true;
                self.drag_clock = DragClock::default();
            }
            // A zoom run times every frame: a pinch is a new picture each frame, drawn or not.
            if self.rebuilt || zooming {
                self.drag_clock
                    .record(start.elapsed().as_secs_f64() * 1000.0);
            }
        } else {
            self.clock_live = false;
        }
    }

    fn canvas_frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let avail = ui.available_size();
        self.canvas_size = Some(avail);
        let Some(scale) = self.viewer.scale((avail.x, avail.y), ppp) else {
            return;
        };
        // The rotate handle's stalk keeps one length on screen at any zoom (rung 6b).
        self.viewer.rotate_stalk_px = ROTATE_STALK_POINTS / f64::from(scale);
        let generation = self.viewer.generation;
        let revision = self.viewer.revision;
        let selected = self.viewer.selected;
        let selection = self.viewer.selection.clone();
        let entered = self.viewer.entered.clone();
        let time = ctx.input(|i| i.time);
        let gesture_live = self
            .viewer
            .zoom_gesture
            .is_some_and(|g| time - g.last <= ZOOM_GESTURE_IDLE);
        if gesture_live {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(ZOOM_GESTURE_IDLE + 0.05));
        }
        let Some(page) = self.viewer.page() else {
            return;
        };
        let (w, h) = (page.profile.width_px, page.profile.height_px);
        // Zoomed in or at 1:1, the export pixels unchanged, drawn as squares (Q3, Q10). Zoomed
        // out, the page shrunk by averaging to exactly the screen pixels it covers (`shrink`).
        let phys = scale * ppp;
        // While a pinch or Ctrl + scroll is going, zoomed out, the page is drawn from a ladder of
        // half-size copies (made once, drawn smoothed), not shrunk to the exact pixels every frame, which
        // takes 40 to 80 ms on the Pi (fixes batch 1, Q1 and ADR-009). A moment after the gesture rests the
        // exact shrink takes over.
        let mip_level =
            (gesture_live && phys < 1.0 && self.canvas_mode == CanvasMode::ExportPixels)
                .then(|| ((1.0 / phys).log2().floor() as usize).min(MIP_LEVELS - 1));
        let (tw, th) = if phys < 1.0 && mip_level.is_none() {
            (
                ((w as f32 * phys).round() as u32).max(1),
                ((h as f32 * phys).round() as u32).max(1),
            )
        } else {
            (w, h)
        };
        let key = TextureKey {
            generation,
            revision,
            profile: page.profile.id.clone(),
            size: (tw, th),
        };
        let export_canvas = self.canvas_mode == CanvasMode::ExportPixels;
        if mip_level.is_some() {
            let mkey = (generation, revision, page.profile.id.clone());
            if self.mips.as_ref().is_none_or(|(k, _)| *k != mkey) {
                let mut levels = Vec::with_capacity(MIP_LEVELS);
                let mut img = page.image.clone();
                for l in 0..MIP_LEVELS {
                    if l > 0 {
                        img = halve(&img);
                    }
                    levels.push((
                        (img.width, img.height),
                        upload_tiles(&ctx, &img, "mip", TextureOptions::LINEAR),
                    ));
                }
                self.mips = Some((mkey, levels));
                self.rebuilt = true;
            }
        }
        if export_canvas
            && mip_level.is_none()
            && self.texture.as_ref().is_none_or(|(k, _)| *k != key)
        {
            let image = if (tw, th) == (w, h) {
                page.image.clone()
            } else {
                shrink(&page.image, tw, th)
            };
            // In tiles, because a Manta page (1920 × 2560) is larger than some graphics chips
            // take in one texture (egui's test harness: 2048).
            let tiles = upload_tiles(&ctx, &image, "page", TextureOptions::NEAREST);
            self.texture = Some((key, tiles));
            self.rebuilt = true;
        }
        let (tex_dims, tiles): ((u32, u32), Vec<_>) = match (&self.texture, &self.mips, mip_level) {
            (_, Some((_, levels)), Some(l)) if export_canvas => {
                let (dims, tiles) = &levels[l];
                (*dims, tiles.iter().map(|(t, tex)| (*t, tex.id())).collect())
            }
            (Some((_, tiles)), _, None) if export_canvas => (
                (tw, th),
                tiles.iter().map(|(t, tex)| (*t, tex.id())).collect(),
            ),
            _ => ((tw, th), Vec::new()),
        };
        let zones: Vec<_> = page.profile.safe_zones.clone();
        let overflows: Vec<(f64, f64)> = page.overflows.iter().map(|o| o.corner).collect();
        let area = selected
            .and_then(|i| page.findings.get(i))
            .and_then(|f| f.area);
        // The selection's outlines: an overlay, never exported (rung 3 bbq, Defaulted
        // "Selecting").
        let mut outlines: Vec<(quadrule_model::Uuid, Outline)> = page
            .placed
            .iter()
            .filter(|p| selection.contains(&p.id))
            .map(|p| (p.id, (p.shape.bounds(), p.shape.corners())))
            .collect();
        // The groups entered get a faint dashed outline (rung 7a bbq, Q4).
        let entered_outlines: Vec<Outline> = entered
            .iter()
            .filter_map(|g| page.placed.iter().find(|p| p.id == *g))
            .map(|p| (p.shape.bounds(), p.shape.corners()))
            .collect();
        // A turned line is outlined along its turned frame, like a turned shape (the page's own
        // shape for a line is its segment).
        for (id, o) in &mut outlines {
            if o.1.is_none() {
                o.1 = self
                    .viewer
                    .turned_line_outline(*id, OUTLINE_PAD_POINTS / f64::from(scale));
            }
        }

        // What the drag draws, and the handles of the selection (all overlays, never exported).
        let typing = self.viewer.is_typing();
        let handles = if self.viewer.tool == Tool::Select && !typing {
            self.viewer.handles()
        } else {
            Vec::new()
        };
        let rotate_handle = if self.viewer.tool == Tool::Select && !typing {
            self.viewer.rotate_handle()
        } else {
            None
        };
        let typing_view = self.viewer.typing_view();
        let preedit = self.viewer.typing_preedit().map(String::from);
        let guides = self.viewer.drag_guides().to_vec();
        let marquee = self.viewer.drag_marquee();
        // Equal gaps are drawn with their size; the words come from Fluent.
        let marks: Vec<(Mark, String)> = self
            .viewer
            .drag_marks()
            .iter()
            .map(|m| {
                let label = match m {
                    Mark::Gap { from, to, .. } => self
                        .strings
                        .format(&Message::new("app-gap-size").num("px", (to - from).abs(), 0)),
                    Mark::Point { .. } => String::new(),
                };
                (*m, label)
            })
            .collect();
        let rhythm = self
            .viewer
            .show_rhythm
            .then(|| self.viewer.rhythm_overlay())
            .flatten();

        let size = if phys < 1.0 && mip_level.is_none() {
            vec2(tw as f32 / ppp, th as f32 / ppp)
        } else {
            vec2(w as f32 * scale, h as f32 * scale)
        };
        let margin = 16.0;
        ScrollArea::both().auto_shrink(false).show(ui, |ui| {
            let total = vec2(
                (size.x + 2.0 * margin).max(avail.x),
                (size.y + 2.0 * margin).max(avail.y),
            );
            let (rect, response) = ui.allocate_exact_size(total, Sense::click_and_drag());
            self.canvas_id = Some(response.id);
            // A typing session has the keyboard: arrows, Tab and Escape go to it and not to the
            // focus (rung 11b).
            if std::mem::take(&mut self.take_canvas_focus) {
                response.request_focus();
            }
            if self.viewer.is_typing() {
                ctx.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        response.id,
                        egui::EventFilter {
                            tab: true,
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            escape: true,
                        },
                    )
                });
            }
            // The page's corner sits on a whole screen pixel, so 1:1 and the zoom steps stay
            // sharp.
            let min = rect.center() - size / 2.0;
            let min = pos2((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
            let page_rect = Rect::from_min_size(min, size);
            self.geom = Some((min, scale));
            // The middle of what is in view, as a device pixel of the page, kept on the page.
            let c = ui.clip_rect().center();
            self.view_centre_px = Some((
                f64::from((c.x - min.x) / scale).clamp(0.0, f64::from(w)),
                f64::from((c.y - min.y) / scale).clamp(0.0, f64::from(h)),
            ));
            let painter = ui.painter_at(rect);
            let to_screen =
                |x: f64, y: f64| pos2(min.x + x as f32 * scale, min.y + y as f32 * scale);
            // A Ctrl + scroll zoom a frame ago: scroll the page so the pixel that was under the
            // pointer is under it again.
            if std::mem::take(&mut self.zoom_anchor_due)
                && let Some((p, x, y)) = self.zoom_anchor
            {
                let d = p - to_screen(f64::from(x), f64::from(y));
                if d.length() >= 0.5 {
                    ui.scroll_with_delta_animation(d, egui::style::ScrollAnimation::none());
                    ctx.request_repaint();
                }
            }

            painter.rect_stroke(
                page_rect.expand(1.0),
                CornerRadius::ZERO,
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                StrokeKind::Outside,
            );
            if !export_canvas {
                self.paint_screen_canvas(&ctx, &painter, page_rect, (w, h), ppp);
            }
            for ((x, y, tw_, th_), id) in &tiles {
                let at = |px: u32, py: u32| {
                    min + vec2(
                        px as f32 / tex_dims.0 as f32 * size.x,
                        py as f32 / tex_dims.1 as f32 * size.y,
                    )
                };
                painter.image(
                    *id,
                    Rect::from_min_max(at(*x, *y), at(x + tw_, y + th_)),
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }

            // A grid between device pixels from 16× (Q10, amended 2026-11-10), only where it shows.
            if scale * ppp >= PIXEL_GRID_FROM {
                let clip = painter.clip_rect().intersect(page_rect);
                let stroke = Stroke::new(
                    1.0 / ppp,
                    Color32::from_rgba_unmultiplied(128, 128, 255, 90),
                );
                let x0 = ((clip.min.x - min.x) / scale).floor().max(0.0) as u32;
                let x1 = (((clip.max.x - min.x) / scale).ceil() as u32).min(w);
                let y0 = ((clip.min.y - min.y) / scale).floor().max(0.0) as u32;
                let y1 = (((clip.max.y - min.y) / scale).ceil() as u32).min(h);
                for x in x0..=x1 {
                    let sx = min.x + x as f32 * scale;
                    painter.line_segment([pos2(sx, clip.min.y), pos2(sx, clip.max.y)], stroke);
                }
                for y in y0..=y1 {
                    let sy = min.y + y as f32 * scale;
                    painter.line_segment([pos2(clip.min.x, sy), pos2(clip.max.x, sy)], stroke);
                }
            }

            let accent = ui.visuals().selection.stroke.color;

            // A Fixed text box that leaves lines out wears a small red + tab at its bottom-right
            // corner (rung 11b, Q5): an overlay, never exported.
            for &(cx, cy) in &overflows {
                paint_overflow_tab(&painter, to_screen(cx, cy), OVERFLOW_TAB_POINTS);
            }

            // The rhythm, as a light blue guide that stays out of the way of the objects: units a little
            // stronger than subdivisions (rung 4 bbq Q9; chris, 2026-11-10: "much lighter and perhaps
            // light blue").
            if let Some(ov) = &rhythm {
                use crate::drag::OverlayLine;
                let clip = painter.clip_rect().intersect(page_rect);
                // Major lines (rung 9 bbq, Q6) are drawn stronger: darker, and a little thicker.
                let line = |l: OverlayLine| {
                    let (alpha, width) = match l {
                        OverlayLine::Sub => (26, 1.0),
                        OverlayLine::Unit => (60, 1.0),
                        OverlayLine::Major => (150, 2.0),
                    };
                    Stroke::new(
                        width / ppp,
                        Color32::from_rgba_unmultiplied(120, 180, 240, alpha),
                    )
                };
                for &(x, l) in &ov.xs {
                    let sx = min.x + x as f32 * scale;
                    if sx >= clip.min.x && sx <= clip.max.x {
                        painter.line_segment([pos2(sx, clip.min.y), pos2(sx, clip.max.y)], line(l));
                    }
                }
                for &(y, l) in &ov.ys {
                    let sy = min.y + y as f32 * scale;
                    if sy >= clip.min.y && sy <= clip.max.y {
                        painter.line_segment([pos2(clip.min.x, sy), pos2(clip.max.x, sy)], line(l));
                    }
                }
                for &((ax, ay), (bx, by), l) in &ov.slanted {
                    painter
                        .with_clip_rect(clip)
                        .line_segment([to_screen(ax, ay), to_screen(bx, by)], line(l));
                }
            }

            // Safe zones (Q6), with no labels (chris, 2026-11-10): fixed ones red, movable ones amber.
            if self.viewer.show_safe_zones {
                let shown: Vec<_> = zones
                    .iter()
                    .filter(|z| match Toolbar::of_zone(&z.name) {
                        Ok(Some(t)) => TOOLBARS_SHOWN.contains(&t),
                        _ => true,
                    })
                    .collect();
                let rects: Vec<PxRect> = shown
                    .iter()
                    .map(|z| {
                        let (x, y, zw, zh) = z.rect;
                        PxRect {
                            x0: x as f64,
                            y0: y as f64,
                            x1: (x + zw) as f64,
                            y1: (y + zh) as f64,
                        }
                    })
                    .collect();
                let color_of = |i: usize| {
                    if shown[i].advisory {
                        Color32::from_rgb(230, 150, 0)
                    } else {
                        Color32::from_rgb(220, 40, 40)
                    }
                };
                // Each kind of zone is filled once over its combined area, so where zones overlap the
                // shade is not darker (chris, 2026-11-10).
                for advisory in [false, true] {
                    let of_kind: Vec<PxRect> = rects
                        .iter()
                        .zip(&shown)
                        .filter(|(_, z)| z.advisory == advisory)
                        .map(|(r, _)| *r)
                        .collect();
                    let color = if advisory {
                        Color32::from_rgb(230, 150, 0)
                    } else {
                        Color32::from_rgb(220, 40, 40)
                    };
                    for r in union_fill(&of_kind) {
                        let r = Rect::from_min_max(to_screen(r.x0, r.y0), to_screen(r.x1, r.y1));
                        painter.rect_filled(r, CornerRadius::ZERO, color.gamma_multiply(0.18));
                    }
                }
                // Borders only round the combined area: where zones overlap or meet, the edges
                // inside it are not drawn (chris, 2026-11-10).
                for (i, ((ax, ay), (bx, by))) in union_outline(&rects) {
                    painter.line_segment(
                        [to_screen(ax, ay), to_screen(bx, by)],
                        Stroke::new(1.0, color_of(i)),
                    );
                }
            }

            // The selected finding's area (Q6).
            if let Some(a) = area {
                let r =
                    Rect::from_min_max(to_screen(a.x0, a.y0), to_screen(a.x1, a.y1)).expand(3.0);
                let accent = ui.visuals().selection.stroke.color;
                painter.rect_stroke(
                    r,
                    CornerRadius::same(2),
                    Stroke::new(2.0, accent),
                    StrokeKind::Outside,
                );
                if self.scroll_to_selected {
                    ui.scroll_to_rect(r, Some(egui::Align::Center));
                    self.scroll_to_selected = false;
                }
            }

            // The groups entered: a faint dashed box.
            for ((x0, y0, x1, y1), corners) in &entered_outlines {
                let mut pts: Vec<egui::Pos2> = match corners {
                    Some(c) => c.iter().map(|&(x, y)| to_screen(x, y)).collect(),
                    None => vec![
                        to_screen(*x0, *y0),
                        to_screen(*x1, *y0),
                        to_screen(*x1, *y1),
                        to_screen(*x0, *y1),
                    ],
                };
                pts.push(pts[0]);
                painter.extend(egui::Shape::dashed_line(
                    &pts,
                    Stroke::new(1.0, accent.gamma_multiply(0.7)),
                    6.0,
                    4.0,
                ));
            }

            // The selected objects.
            for (_, ((x0, y0, x1, y1), corners)) in &outlines {
                if let Some(c) = corners {
                    // A turned shape is outlined along its turned corners.
                    let pts: Vec<egui::Pos2> = c.iter().map(|&(x, y)| to_screen(x, y)).collect();
                    painter.add(egui::Shape::closed_line(pts, Stroke::new(1.5, accent)));
                    continue;
                }
                let r = Rect::from_min_max(to_screen(*x0, *y0), to_screen(*x1, *y1)).expand(2.0);
                painter.rect_stroke(
                    r,
                    CornerRadius::ZERO,
                    Stroke::new(1.5, accent),
                    StrokeKind::Outside,
                );
            }

            // Handles on the selection, then what the drag shows: the snap that caught, and the
            // marquee box.
            let to_dev = |p: egui::Pos2| {
                (
                    ((p.x - min.x) / scale) as f64,
                    ((p.y - min.y) / scale) as f64,
                )
            };
            if marquee.is_none() {
                // The rotate handle: a round handle on a short stalk above the top edge.
                if let Some(rh) = rotate_handle {
                    let (a, b) = (
                        to_screen(rh.base.0, rh.base.1),
                        to_screen(rh.tip.0, rh.tip.1),
                    );
                    painter.line_segment([a, b], Stroke::new(1.5, accent));
                    painter.circle_filled(b, 5.0, Color32::WHITE);
                    painter.circle_stroke(b, 5.0, Stroke::new(1.5, accent));
                }
                // Each handle is a square turned with the selection, so the box reads as turned.
                let angle = self.viewer.selection_angle();
                let squares = if angle == 0.0 {
                    Vec::new()
                } else {
                    self.viewer.handle_squares(4.0 / f64::from(scale))
                };
                for (hit, hx, hy) in &handles {
                    let stroke_kind = if matches!(hit, HandleHit::Box(_)) {
                        StrokeKind::Inside
                    } else {
                        StrokeKind::Middle
                    };
                    if let Some((_, c)) = squares.iter().find(|(h, _)| h == hit) {
                        let pts: Vec<egui::Pos2> =
                            c.iter().map(|&(x, y)| to_screen(x, y)).collect();
                        painter.add(egui::Shape::convex_polygon(
                            pts,
                            Color32::WHITE,
                            Stroke::new(1.5, accent),
                        ));
                        continue;
                    }
                    let c = to_screen(*hx, *hy);
                    let r = Rect::from_center_size(c, vec2(8.0, 8.0));
                    // White on any theme: the handles sit on the page, which is always light.
                    painter.rect_filled(r, CornerRadius::ZERO, Color32::WHITE);
                    painter.rect_stroke(
                        r,
                        CornerRadius::ZERO,
                        Stroke::new(1.5, accent),
                        stroke_kind,
                    );
                }
            }
            for g in &guides {
                // A guide to another object runs from the lowest to the highest of the objects it
                // joins; the rest run across the page.
                let (lo, hi) = match (g.span, g.axis) {
                    (Some(s), _) => s,
                    (None, Axis::X) => (0.0, h as f64),
                    (None, Axis::Y) => (0.0, w as f64),
                };
                let (a, b) = match g.axis {
                    Axis::X => (to_screen(g.at, lo), to_screen(g.at, hi)),
                    Axis::Y => (to_screen(lo, g.at), to_screen(hi, g.at)),
                };
                let color = match g.kind {
                    SnapKind::Rhythm | SnapKind::Subdivision => Color32::from_rgb(230, 40, 150),
                    SnapKind::RhythmSize => Color32::from_rgb(160, 60, 220),
                    SnapKind::Margin | SnapKind::SafeZone => Color32::from_rgb(0, 160, 170),
                    SnapKind::Point | SnapKind::Smart | SnapKind::Equal => OBJECT_SNAP,
                };
                painter.line_segment([a, b], Stroke::new(1.0, color));
            }
            for (mark, label) in &marks {
                match *mark {
                    Mark::Point { x, y } => {
                        // A ring with a cross through it, on the anchor that caught.
                        let c = to_screen(x, y);
                        let stroke = Stroke::new(1.5, OBJECT_SNAP);
                        painter.circle_stroke(c, 5.0, stroke);
                        painter.line_segment([c - vec2(8.0, 0.0), c + vec2(8.0, 0.0)], stroke);
                        painter.line_segment([c - vec2(0.0, 8.0), c + vec2(0.0, 8.0)], stroke);
                    }
                    Mark::Gap {
                        axis,
                        from,
                        to,
                        cross,
                    } => {
                        // The gap as a measure: a line with a tick at each end, and its size.
                        let (a, b, tick) = match axis {
                            Axis::X => {
                                (to_screen(from, cross), to_screen(to, cross), vec2(0.0, 4.0))
                            }
                            Axis::Y => {
                                (to_screen(cross, from), to_screen(cross, to), vec2(4.0, 0.0))
                            }
                        };
                        let stroke = Stroke::new(1.0, OBJECT_SNAP);
                        painter.line_segment([a, b], stroke);
                        painter.line_segment([a - tick, a + tick], stroke);
                        painter.line_segment([b - tick, b + tick], stroke);
                        let galley = painter.layout_no_wrap(
                            label.clone(),
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                        let at = a.lerp(b, 0.5) - galley.size() / 2.0;
                        painter.rect_filled(
                            Rect::from_min_size(at, galley.size()).expand(2.0),
                            CornerRadius::same(2),
                            OBJECT_SNAP,
                        );
                        painter.galley(at, galley, Color32::WHITE);
                    }
                }
            }
            if let Some(m) = marquee {
                let r = Rect::from_min_max(to_screen(m.x0, m.y0), to_screen(m.x1, m.y1));
                painter.rect_filled(r, CornerRadius::ZERO, accent.gamma_multiply(0.12));
                painter.rect_stroke(
                    r,
                    CornerRadius::ZERO,
                    Stroke::new(1.0, accent),
                    StrokeKind::Inside,
                );
            }

            // The caret and the selection of a typing session (rung 11b): an overlay, drawn from the
            // very layout the letters were, turned with the box.
            if let Some(v) = &typing_view {
                for q in &v.selection {
                    let pts: Vec<egui::Pos2> = q.iter().map(|&(x, y)| to_screen(x, y)).collect();
                    painter.add(egui::Shape::convex_polygon(
                        pts,
                        accent.gamma_multiply(0.30),
                        Stroke::NONE,
                    ));
                }
                let outline: Vec<egui::Pos2> =
                    v.outline.iter().map(|&(x, y)| to_screen(x, y)).collect();
                painter.add(egui::Shape::closed_line(
                    outline,
                    Stroke::new(1.0, accent.gamma_multiply(0.6)),
                ));
                let (a, b) = (
                    to_screen(v.caret.0.0, v.caret.0.1),
                    to_screen(v.caret.1.0, v.caret.1.1),
                );
                // A solid caret: it needs no repaint of its own while the window is idle.
                let w = ((v.caret_width as f32 * scale).max(1.5 / ppp)).max(1.0);
                painter.line_segment([a, b], Stroke::new(w, Color32::BLACK));
                if let Some(text) = &preedit {
                    // What an input method is composing, at the caret, underlined.
                    let font = egui::FontId::proportional(14.0);
                    let galley = painter.layout_no_wrap(text.clone(), font, Color32::BLACK);
                    let at = a + vec2(2.0, 0.0);
                    painter.rect_filled(
                        Rect::from_min_size(at, galley.size()),
                        CornerRadius::ZERO,
                        Color32::from_rgb(255, 250, 200),
                    );
                    let under = at + vec2(0.0, galley.size().y);
                    painter.line_segment(
                        [under, under + vec2(galley.size().x, 0.0)],
                        Stroke::new(1.0, Color32::BLACK),
                    );
                    painter.galley(at, galley, Color32::BLACK);
                }
                // Where an input method puts its window.
                ctx.output_mut(|o| {
                    o.ime = Some(egui::output::IMEOutput {
                        purpose: egui::IMEPurpose::Normal,
                        rect: Rect::from_two_pos(a, b).expand(2.0),
                        cursor_rect: Rect::from_two_pos(a, b),
                        should_interrupt_composition: false,
                    })
                });
            }

            // Typing on the canvas (rung 11b): a press on the box puts the caret there and a drag
            // selects; a press elsewhere ends the typing and is then an ordinary press.
            let press_mods = ui.input(|i| i.modifiers);
            if self.viewer.is_typing()
                && response.drag_started_by(PointerButton::Primary)
                && let Some(p) = ui.input(|i| i.pointer.press_origin())
            {
                let (x, y) = to_dev(p);
                if self.viewer.typing_contains(x, y) {
                    self.viewer.type_pointer(x, y, press_mods.shift, false);
                    self.type_selecting = true;
                    response.request_focus();
                } else {
                    self.viewer.end_typing();
                }
            }
            if self.type_selecting {
                if response.dragged()
                    && let Some(p) = ui.input(|i| i.pointer.latest_pos())
                {
                    let (x, y) = to_dev(p);
                    self.viewer.type_drag_to(x, y);
                    ctx.request_repaint();
                }
                if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                    self.type_selecting = false;
                }
            }

            // Dragging (rung 4a): a press that moves is a drag; one that does not is a click.
            let mods = ui.input(|i| DragMods {
                shift: i.modifiers.shift,
                alt: i.modifiers.alt,
                snapping_off: i.modifiers.command,
            });
            let slack = PICK_SLACK_POINTS / scale as f64;
            let handle_slack = HANDLE_SLACK_POINTS / scale as f64;
            let reach = SNAP_REACH_POINTS / scale as f64;
            // With a drawing tool on, a press draws a new object instead of moving one (rung 5).
            let drawing_tool = self.viewer.tool != Tool::Select;
            if response.drag_started_by(PointerButton::Primary)
                && !self.type_selecting
                && let Some(p) = ui.input(|i| i.pointer.press_origin())
            {
                let (x, y) = to_dev(p);
                let began = if drawing_tool {
                    self.viewer.draw_begin(x, y, mods, reach)
                } else {
                    self.viewer.drag_begin(x, y, mods, slack, handle_slack)
                };
                if began {
                    self.selected_layer = None;
                    self.pointer_drag = true;
                }
                ctx.request_repaint();
            }
            if self.viewer.dragging() && response.dragged() {
                if let Some(p) = ui.input(|i| i.pointer.latest_pos()) {
                    let (x, y) = to_dev(p);
                    if self.viewer.drawing() {
                        self.viewer.draw_update(x, y, mods, reach);
                    } else {
                        self.viewer.drag_update(x, y, mods, reach);
                    }
                }
                ctx.request_repaint();
            }
            if response.drag_stopped() {
                self.viewer.gesture_end();
                ctx.request_repaint();
            }
            // A release the canvas never saw (the pointer let go outside the window) still ends
            // the drag the pointer began.
            if self.pointer_drag
                && self.viewer.dragging()
                && !ui.input(|i| i.pointer.primary_down())
            {
                self.viewer.gesture_end();
                ctx.request_repaint();
            }
            if !self.viewer.dragging() {
                self.pointer_drag = false;
            }
            // The middle button pans: a plain drag now moves objects, so it no longer scrolls.
            if response.dragged_by(PointerButton::Middle) {
                ui.scroll_with_delta_animation(
                    response.drag_delta(),
                    egui::style::ScrollAnimation::none(),
                );
            }
            // The pointer says what a press would do.
            let icon = match self.viewer.drag.as_ref().map(|d| d.kind) {
                _ if drawing_tool => Some(CursorIcon::Crosshair),
                Some(DragKind::Move) => Some(CursorIcon::Move),
                Some(DragKind::Marquee) => Some(CursorIcon::Crosshair),
                Some(DragKind::Handle(h)) => Some(handle_cursor(h)),
                None => response.hover_pos().and_then(|p| {
                    let (x, y) = to_dev(p);
                    self.viewer.handle_at(x, y, handle_slack).map(handle_cursor)
                }),
            };
            if let Some(icon) = icon {
                ctx.set_cursor_icon(icon);
            }

            // A click picks the object under the pointer; Shift adds to the selection. With a
            // drawing tool on, it drops a new object at its default size instead.
            // While typing, a click on the box moves the caret (a double-click selects the word); a
            // click elsewhere ends the typing and then does what it would have.
            let mut typed_click = false;
            if self.viewer.is_typing()
                && response.clicked()
                && let Some(p) = response.interact_pointer_pos()
            {
                let (x, y) = to_dev(p);
                if self.viewer.typing_contains(x, y) {
                    self.viewer
                        .type_pointer(x, y, press_mods.shift, response.double_clicked());
                    typed_click = true;
                    response.request_focus();
                } else {
                    self.viewer.end_typing();
                }
            }
            if response.clicked()
                && !typed_click
                && let Some(p) = response.interact_pointer_pos()
            {
                let (x, y) = to_dev(p);
                if drawing_tool {
                    let _ = self.viewer.place_click(x, y, mods, reach);
                } else if response.double_clicked() {
                    // A double-click on text types in it, at the click (rung 11b), a copy in a repeat
                    // and text inside a group included; on a group it goes one level in (rung 7a
                    // bbq, Q4); on anything else it is the click it was.
                    if self.viewer.begin_typing_at(x, y, slack) {
                        self.take_canvas_focus = true;
                        response.request_focus();
                        ctx.request_repaint();
                    } else if !self.viewer.enter_at(x, y, slack) {
                        self.viewer
                            .click_with(x, y, slack, mods.shift, mods.snapping_off);
                    }
                } else {
                    self.viewer
                        .click_with(x, y, slack, mods.shift, mods.snapping_off);
                }
                self.selected_layer = None;
            }

            // Where the pointer is, in device pixels, for the status bar.
            self.cursor_px = response.hover_pos().and_then(|p| {
                let x = (p.x - min.x) / scale;
                let y = (p.y - min.y) / scale;
                (x >= 0.0 && y >= 0.0 && x < w as f32 && y < h as f32).then_some((x, y))
            });

            // Ctrl + scroll (or a pinch) zooms by the gesture's own factor, anchored at the pointer
            // (fixes batch 1, Q1); a bench run feeds its own factor in place of the input's.
            let bench_zoom = self.bench_zoom_step.take();
            if response.hovered() || bench_zoom.is_some() {
                let (z, pointer) = match bench_zoom {
                    Some((z, p)) => (z, Some(p)),
                    None => (
                        ui.input(|i| i.zoom_delta()),
                        ui.input(|i| i.pointer.hover_pos()),
                    ),
                };
                if z != 1.0 {
                    let fit = self
                        .viewer
                        .scale_for(Zoom::Fit, (avail.x, avail.y), ppp)
                        .map_or(f32::NAN, |s| s * ppp);
                    let time = ui.input(|i| i.time);
                    let carried = self.viewer.zoom_by(z, scale * ppp, fit, time);
                    if let Some(p) = pointer {
                        // The device pixel under the pointer stays under it all through the gesture.
                        let keep = carried
                            && self
                                .zoom_anchor
                                .is_some_and(|(a, _, _)| (a - p).length() < 1.0);
                        if !keep {
                            self.zoom_anchor =
                                Some((p, (p.x - min.x) / scale, (p.y - min.y) / scale));
                        }
                    }
                    self.zoom_anchor_due = true;
                    ctx.request_repaint();
                }
            }
        });
    }

    /// The graphics-chip canvas: the visible part of the page at screen resolution, by vello on
    /// the chip, or by vello_cpu when the chip cannot start or is too slow (rung 3 bbq, Defaulted
    /// "The chip canvas on the Pi will be slow").
    fn paint_screen_canvas(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        page_rect: Rect,
        (w, h): (u32, u32),
        ppp: f32,
    ) {
        let visible = painter.clip_rect().intersect(page_rect);
        if visible.width() < 1.0 || visible.height() < 1.0 {
            return;
        }
        // On whole screen pixels, so the image is drawn 1:1.
        let snap = |v: f32| (v * ppp).round() / ppp;
        let vis = Rect::from_min_max(
            pos2(snap(visible.min.x), snap(visible.min.y)),
            pos2(snap(visible.max.x), snap(visible.max.y)),
        );
        let (sx, sy) = (page_rect.width() / w as f32, page_rect.height() / h as f32);
        let view = ScreenView {
            scale: (sx * ppp) as f64,
            x: ((vis.min.x - page_rect.min.x) / sx) as f64,
            y: ((vis.min.y - page_rect.min.y) / sy) as f64,
            width: (vis.width() * ppp).round().max(1.0) as u32,
            height: (vis.height() * ppp).round().max(1.0) as u32,
        };
        let key = format!(
            "{} {} {} {:?}",
            self.viewer.generation, self.viewer.revision, self.viewer.profile_id, view
        );
        if self.vector.as_ref().is_none_or(|(k, _)| *k != key) {
            let Some(dl) = self.viewer.page().map(|p| p.display_list.clone()) else {
                return;
            };
            let list = screen_list(&dl, &view);
            let mut drawn = None;
            let mut warm_up_done = false;
            if self.chip_state.fallback.is_none() {
                match &mut self.chip {
                    Ok(chip) => match chip.render(&list) {
                        Ok((id, ms)) => {
                            self.chip_state
                                .record_chip_draws(ms, chip.last_frame_draws());
                            drawn = Some(VectorTexture::Chip(id));
                            if self.chip_state.frames == 1 {
                                // The first frame builds shaders and is not judged; draw a
                                // second one now, so the 100 ms rule is decided at once.
                                warm_up_done = true;
                            }
                        }
                        Err(e) => self.chip_state.fallback = Some(Fallback::CannotStart(e)),
                    },
                    Err(e) => self.chip_state.fallback = Some(Fallback::CannotStart(e.clone())),
                }
            }
            let tex = match drawn {
                Some(t) => t,
                None => {
                    let start = std::time::Instant::now();
                    let img = quadrule_render::render(&list, quadrule_render::Math::Fastest);
                    self.chip_state.cpu_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
                    let image =
                        ColorImage::from_gray([img.width as usize, img.height as usize], &img.data);
                    VectorTexture::Cpu(ctx.load_texture(
                        "page-screen",
                        image,
                        TextureOptions::NEAREST,
                    ))
                }
            };
            // After the warm-up frame the key is left stale, so the next frame draws again.
            self.vector = Some((if warm_up_done { String::new() } else { key }, tex));
            self.rebuilt = true;
            if warm_up_done {
                ctx.request_repaint();
            }
        }
        if let Some((_, tex)) = &self.vector {
            let id = match tex {
                VectorTexture::Chip(id) => *id,
                VectorTexture::Cpu(t) => t.id(),
            };
            painter.image(
                id,
                vis,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }

    /// The canvas line for the status bar, in graphics-chip mode.
    fn canvas_status(&self) -> Option<String> {
        if self.canvas_mode != CanvasMode::Chip {
            return None;
        }
        let ms = |v: Option<f64>| v.map_or(0.0, |v| v.round());
        let cpu = ms(self.chip_state.cpu_ms);
        let msg = match &self.chip_state.fallback {
            None => Message::new("app-chip-ms").num("ms", ms(self.chip_state.last_ms), 0),
            Some(Fallback::Slow(v)) => Message::new("app-chip-slow")
                .num("ms", v.round(), 0)
                .num("cpu", cpu, 0),
            Some(Fallback::CannotStart(e)) => Message::new("app-chip-none")
                .text("error", e.clone())
                .num("cpu", cpu, 0),
        };
        Some(self.strings.format(&msg))
    }

    fn ruler_window(&mut self, ctx: &egui::Context) {
        let Some(mut ruler) = self.ruler else {
            return;
        };
        let ppp = ctx.pixels_per_point();
        let mut open = true;
        let mut done = None;
        egui::Window::new(self.t("app-ruler-title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(self.t("app-ruler-help"));
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut ruler.reference,
                        Reference::Card,
                        self.t("app-ruler-card"),
                    );
                    ui.radio_value(
                        &mut ruler.reference,
                        Reference::Ruler100,
                        self.t("app-ruler-100"),
                    );
                });
                ui.add(
                    egui::Slider::new(&mut ruler.bar, 100.0..=900.0)
                        .step_by(0.25)
                        .show_value(false)
                        .text(self.t("app-ruler-bar")),
                );
                ui.horizontal(|ui| {
                    if ui.small_button("−").clicked() {
                        ruler.bar -= 1.0 / ppp;
                    }
                    if ui.small_button("+").clicked() {
                        ruler.bar += 1.0 / ppp;
                    }
                });
                let (r, _) = ui.allocate_exact_size(vec2(ruler.bar, 28.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(r, CornerRadius::ZERO, ui.visuals().strong_text_color());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t("app-ruler-save")).clicked() {
                        done = Some(ruler_px_per_mm(ruler.bar, ppp, ruler.reference.mm()));
                    }
                });
            });
        if let Some(r) = done {
            self.viewer.screen_px_per_mm = Some(r);
            self.viewer.zoom = Zoom::RealSize;
            self.ruler = None;
        } else if open {
            self.ruler = Some(ruler);
        } else {
            self.ruler = None;
        }
    }

    /// Whether the ruler check is open.
    pub fn ruler_open(&self) -> bool {
        self.ruler.is_some()
    }
}

/// The Fluent message that names an orientation (the New dialog and the Page tab).
fn orientation_message(o: quadrule_model::Orientation) -> &'static str {
    match o {
        quadrule_model::Orientation::Portrait => "app-orientation-portrait",
        quadrule_model::Orientation::Landscape => "app-orientation-landscape",
    }
}

/// A selected object's outline: its bounds, and its turned corners when it is a turned shape.
const OUTLINE_PAD_POINTS: f64 = 2.0;

type Outline = ((f64, f64, f64, f64), Option<[(f64, f64); 4]>);

/// The mouse cursor over a handle.
fn handle_cursor(h: HandleHit) -> CursorIcon {
    match h {
        HandleHit::Box(Handle::NW | Handle::SE) => CursorIcon::ResizeNwSe,
        HandleHit::Box(Handle::NE | Handle::SW) => CursorIcon::ResizeNeSw,
        HandleHit::Box(Handle::N | Handle::S) => CursorIcon::ResizeVertical,
        HandleHit::Box(Handle::E | Handle::W) => CursorIcon::ResizeHorizontal,
        HandleHit::LineStart | HandleHit::LineEnd => CursorIcon::Crosshair,
        HandleHit::Rotate => CursorIcon::Grab,
    }
}

fn backdrop(v: &egui::Visuals) -> Color32 {
    if v.dark_mode {
        Color32::from_gray(40)
    } else {
        Color32::from_gray(200)
    }
}

/// The font for the controls: the system's interface font (chris, 2026-11-09: "I would like the
/// system default font for the controls"), found by the app. `weight` sets a variable font's
/// weight axis; a fixed font ignores it.
#[derive(Clone, Debug)]
pub struct UiFont {
    /// The font file's bytes.
    pub data: Vec<u8>,
    /// The face's index in the file.
    pub index: u32,
    /// 100 to 900; 400 is regular.
    pub weight: u16,
}

/// Puts `font` first for the controls, keeping egui's own fonts behind it for symbols such as ⚠
/// and for anything it lacks. With no font, egui's own fonts alone.
pub fn install_fonts(ctx: &egui::Context, font: Option<UiFont>) {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(f) = font {
        let mut data = egui::FontData::from_owned(f.data);
        data.index = f.index;
        data.tweak.coords = egui::epaint::text::VariationCoords::new([("wght", f.weight as f32)]);
        fonts
            .font_data
            .insert("system-ui".into(), std::sync::Arc::new(data));
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "system-ui".into());
        }
    }
    ctx.set_fonts(fonts);
}

/// How big the red + tab of a Fixed text box that leaves lines out is on screen, in points
/// (rung 11b).
pub(crate) const OVERFLOW_TAB_POINTS: f32 = 14.0;

/// Paints the red + tab with its bottom-right corner at `corner`: a red square, a white plus. The
/// canvases and the strip's pictures both use it (rung 11b).
pub(crate) fn paint_overflow_tab(painter: &egui::Painter, corner: egui::Pos2, side: f32) {
    let rect = Rect::from_min_max(corner - vec2(side, side), corner);
    painter.rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(210, 30, 30));
    let c = rect.center();
    let arm = side * 0.28;
    let stroke = Stroke::new((side * 0.14).max(1.0), Color32::WHITE);
    painter.line_segment([c - vec2(arm, 0.0), c + vec2(arm, 0.0)], stroke);
    painter.line_segment([c - vec2(0.0, arm), c + vec2(0.0, arm)], stroke);
}
