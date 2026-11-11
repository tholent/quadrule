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
    CARD_WIDTH_MM, ExportWhich, PIXEL_GRID_FROM, Source, TOOLBARS_SHOWN, Tile, Viewer, Zoom,
    ruler_px_per_mm, shrink, tiles,
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
use quadrule_model::blank::RhythmPreset;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Something the window asks the app to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Show the system dialog to pick a `.quad` file.
    OpenDialog,
    /// Show the system dialog to pick a folder, then export.
    ExportDialog(ExportWhich),
    /// Show the system dialog to pick where to save, then call [`ViewerUi::save_as`] with the
    /// path and this follow-up.
    SaveAsDialog(Option<After>),
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
    ruler: Option<Ruler>,
    pub(crate) scroll_to_selected: bool,
    cursor_px: Option<(f32, f32)>,
    canvas_size: Option<egui::Vec2>,
    pub(crate) selected_layer: Option<usize>,
    /// Put the cursor in the Text tab's text field, with its text selected, on the next frame
    /// that shows it (a text box was just drawn).
    pub(crate) focus_text_content: bool,
    pub(crate) renaming: Option<Renaming>,
    /// What is typed in a length field while it has focus, by field key.
    pub(crate) field_text: HashMap<String, String>,
    pub(crate) link_margins: bool,
    pub(crate) link_rhythm: bool,
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
        };
        ViewerUi {
            viewer,
            strings,
            theme: ThemePreference::System,
            show_lint: true,
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
            ruler: None,
            scroll_to_selected: false,
            cursor_px: None,
            canvas_size: None,
            selected_layer: None,
            focus_text_content: false,
            renaming: None,
            field_text: HashMap::new(),
            link_margins: true,
            link_rhythm: true,
            pending: None,
            new_dialog: None,
            new_device: quadrule_model::sample::MANTA_ID.to_string(),
            offers: Vec::new(),
            autosave: Autosave::default(),
            was_focused: true,
            closing: false,
        }
    }

    pub(crate) fn t(&self, id: &str) -> String {
        self.strings.get(id)
    }

    /// Draws the whole window into `ui` and returns what the app must do.
    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Request> {
        let mut requests = Vec::new();
        let ctx = ui.ctx().clone();
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
            self.tab = Tab::Text;
            self.focus_text_content = true;
        }
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui, &mut requests));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        // The tools bar on the left (chris, 2026-11-10: the left side was kept for it; rung 5).
        if self.viewer.open.is_some() {
            egui::Panel::left("tools")
                .resizable(false)
                .exact_size(BAR_WIDTH)
                .show(ui, |ui| self.tools_bar(ui));
        }
        // One panel on the right: the inspector, with the layers below it.
        if self.show_lint && self.viewer.open.is_some() {
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
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(backdrop(ui.visuals())))
            .show(ui, |ui| {
                if self.viewer.open.is_some() {
                    self.canvas(ui);
                } else {
                    self.empty(ui, &mut requests);
                }
            });
        self.ruler_window(&ctx);
        // The Text tab, if it showed, has taken the cursor by now; a request it could not use
        // (no text box selected) must not wait for a later selection.
        self.focus_text_content = false;
        self.new_dialog(&ctx, &mut requests);
        self.unsaved_prompt(&ctx, &mut requests);
        self.recovery_prompt(&ctx);
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
        if self.pending.is_some() || self.new_dialog.is_some() {
            return;
        }
        let cmd = Modifiers::COMMAND;
        let shift_cmd = Modifiers::COMMAND | Modifiers::SHIFT;
        let key = |m: Modifiers, k: Key| {
            ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)))
        };
        // New works on the empty start screen too (rung 5 bbq, Defaulted "New: where and what it
        // replaces"), but not in the middle of a drag.
        if !self.viewer.dragging() && key(cmd, Key::N) {
            self.open_new_dialog();
            return;
        }
        if self.viewer.open.is_none() {
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
            requests.push(Request::SaveAsDialog(None));
        } else if key(cmd, Key::S) {
            requests.extend(self.save());
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        if key(shift_cmd, Key::Z) || key(cmd, Key::Y) {
            self.viewer.redo();
        } else if key(cmd, Key::Z) {
            self.viewer.undo();
        }
        if key(cmd, Key::D) {
            let _ = self.viewer.duplicate_selection();
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
        if key(Modifiers::NONE, Key::Escape) && !self.viewer.end_tool() {
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

    /// Save: to the document's file, or, when it has none (the sample page), by asking where.
    pub fn save(&mut self) -> Vec<Request> {
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
        });
    }

    /// The device the New dialog starts on.
    pub fn new_device(&self) -> &str {
        &self.new_device
    }

    /// Sets the device the New dialog starts on (the app's remembered choice). A device this
    /// build does not know is ignored.
    pub fn set_new_device(&mut self, id: &str) {
        if quadrule_profiles::builtin(id).is_ok() {
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
                egui::ComboBox::from_id_salt("new-device")
                    .selected_text(device_name(&spec.device_id))
                    .show_ui(ui, |ui| {
                        for id in quadrule_profiles::builtin_ids() {
                            ui.selectable_value(
                                &mut spec.device_id,
                                id.to_string(),
                                device_name(id),
                            );
                        }
                    })
                    .response
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
        self.was_focused = focused;
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
            if ui
                .button(self.t("app-new"))
                .on_hover_text(self.t("app-new-tip"))
                .clicked()
            {
                self.open_new_dialog();
            }
            if ui.button(self.t("app-open")).clicked() {
                requests.extend(self.guard(After::OpenDialog));
            }
            let open = self.viewer.open.is_some();
            ui.add_enabled_ui(open, |ui| {
                if ui.button(self.t("app-save")).clicked() {
                    requests.extend(self.save());
                }
                if ui.button(self.t("app-save-as")).clicked() {
                    requests.push(Request::SaveAsDialog(None));
                }
            });
            ui.separator();
            let can_undo = self
                .viewer
                .open
                .as_ref()
                .is_some_and(|o| o.history.can_undo());
            let can_redo = self
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

            // Device.
            let ids = self.viewer.profile_ids();
            let name = |id: &str| {
                quadrule_profiles::builtin(id)
                    .map(|p| p.display_name)
                    .unwrap_or_else(|_| id.to_string())
            };
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
            ui.separator();

            // Zoom (rung 2 bbq, Q10).
            let open = self.viewer.open.is_some();
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

            // Export (rung 2 bbq, Q8).
            ui.add_enabled_ui(open, |ui| {
                ui.menu_button(self.t("app-export"), |ui| {
                    if ui.button(self.t("app-export-this")).clicked() {
                        requests.push(Request::ExportDialog(ExportWhich::ThisDevice));
                    }
                    if ui.button(self.t("app-export-all")).clicked() {
                        requests.push(Request::ExportDialog(ExportWhich::AllDevices));
                    }
                });
            });

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
            if let Some((x, y)) = self.cursor_px
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
            if self.viewer.open.is_some() {
                let pct = self.shown_scale(ui) * 100.0;
                let msg = Message::new("app-status-zoom").num("percent", pct as f64, 0);
                ui.label(self.strings.format(&msg));
                ui.separator();
                let count = self.lint_count();
                let msg = Message::new("app-lint-count").num("count", count as f64, 0);
                ui.label(self.strings.format(&msg));
            }
            if self.viewer.open.is_some() {
                // The snap helpers and the rhythm overlay, each on or off (napkin §6.2).
                ui.separator();
                self.snap_toggles(ui);
            }
            if let Some(line) = self.canvas_status() {
                ui.separator();
                ui.label(line);
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

    fn empty(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.heading(self.t("app-empty-title"));
            ui.add_space(12.0);
            if ui.button(self.t("app-new")).clicked() {
                self.open_new_dialog();
            }
            if ui.button(self.t("app-open")).clicked() {
                requests.extend(self.guard(After::OpenDialog));
            }
            if ui.button(self.t("app-open-sample")).clicked() {
                requests.extend(self.guard(After::OpenSample));
            }
            ui.add_space(8.0);
            ui.weak(self.t("app-empty-drop"));
        });
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
                    if b.transforms {
                        self.viewer.open_transforms();
                    } else if b.shapes {
                        self.viewer.open_shapes();
                    } else {
                        self.viewer.open_sample();
                    }
                }
                b.phase = if n >= crate::bench::WARM_FRAMES {
                    Phase::Begin
                } else {
                    Phase::Warm(n + 1)
                };
            }
            Phase::Begin => {
                // The dot grid, or for the rotate run the big turned rectangle.
                let target = if b.rotate {
                    quadrule_model::sample::transform_ids::TURNED
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
                        let began = if b.rotate {
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
                self.viewer.gesture_end();
                self.viewer.undo();
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
                    "\nCHECK crate=ui name={} result={verdict} build={} canvas={canvas} page={} tool={} frames={} mean_ms={:.1} p90_ms={:.1} worst_ms={:.1} first_ms={:.1} over_budget={} budget_ms={}",
                    if b.rotate {
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
                    if b.transforms {
                        "transforms"
                    } else if b.shapes {
                        "shapes"
                    } else {
                        "sample"
                    },
                    if b.rotate {
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

    /// The screen point of device pixel (x, y) of the page on the last frame, for tests that press
    /// the canvas with a pointer.
    pub fn screen_point(&self, x: f64, y: f64) -> Option<egui::Pos2> {
        let (min, scale) = self.geom?;
        Some(pos2(min.x + x as f32 * scale, min.y + y as f32 * scale))
    }

    /// Whether the benchmark has run to the end.
    pub fn bench_finished(&self) -> bool {
        self.bench
            .as_ref()
            .is_some_and(|b| b.phase == crate::bench::Phase::Done)
    }

    /// The canvas, timed: while an object is dragged, each frame that made the page again is
    /// added to [`ViewerUi::drag_clock`].
    fn canvas(&mut self, ui: &mut egui::Ui) {
        let start = std::time::Instant::now();
        self.rebuilt = false;
        self.canvas_frame(ui);
        if self.viewer.dragging() {
            if !self.clock_live {
                self.clock_live = true;
                self.drag_clock = DragClock::default();
            }
            if self.rebuilt {
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
        let Some(page) = self.viewer.page() else {
            return;
        };
        let (w, h) = (page.profile.width_px, page.profile.height_px);
        // Zoomed in or at 1:1, the export pixels unchanged, drawn as squares (Q3, Q10). Zoomed
        // out, the page shrunk by averaging to exactly the screen pixels it covers (`shrink`).
        let phys = scale * ppp;
        let (tw, th) = if phys < 1.0 {
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
        if export_canvas && self.texture.as_ref().is_none_or(|(k, _)| *k != key) {
            let image = if (tw, th) == (w, h) {
                page.image.clone()
            } else {
                shrink(&page.image, tw, th)
            };
            // In tiles, because a Manta page (1920 × 2560) is larger than some graphics chips
            // take in one texture (egui's test harness: 2048).
            let tiles = tiles(tw, th, TILE)
                .into_iter()
                .map(|t| {
                    let (x, y, tw_, th_) = t;
                    let mut data = Vec::with_capacity((tw_ * th_) as usize);
                    for row in y..y + th_ {
                        let start = (row * tw + x) as usize;
                        data.extend_from_slice(&image.data[start..start + tw_ as usize]);
                    }
                    let img = ColorImage::from_gray([tw_ as usize, th_ as usize], &data);
                    let tex =
                        ctx.load_texture(format!("page-{x}-{y}"), img, TextureOptions::NEAREST);
                    (t, tex)
                })
                .collect();
            self.texture = Some((key, tiles));
            self.rebuilt = true;
        }
        let tiles: Vec<_> = match &self.texture {
            Some((_, tiles)) if export_canvas => {
                tiles.iter().map(|(t, tex)| (*t, tex.id())).collect()
            }
            _ => Vec::new(),
        };
        let zones: Vec<_> = page.profile.safe_zones.clone();
        let area = selected
            .and_then(|i| page.findings.get(i))
            .and_then(|f| f.area);
        // The selection's outlines: an overlay, never exported (rung 3 bbq, Defaulted
        // "Selecting").
        let outlines: Vec<Outline> = page
            .placed
            .iter()
            .filter(|p| selection.contains(&p.id))
            .map(|p| (p.shape.bounds(), p.shape.corners()))
            .collect();

        // What the drag draws, and the handles of the selection (all overlays, never exported).
        let handles = if self.viewer.tool == Tool::Select {
            self.viewer.handles()
        } else {
            Vec::new()
        };
        let rotate_handle = if self.viewer.tool == Tool::Select {
            self.viewer.rotate_handle()
        } else {
            None
        };
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

        let size = if phys < 1.0 {
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
            // The page's corner sits on a whole screen pixel, so 1:1 and the zoom steps stay
            // sharp.
            let min = rect.center() - size / 2.0;
            let min = pos2((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
            let page_rect = Rect::from_min_size(min, size);
            self.geom = Some((min, scale));
            let painter = ui.painter_at(rect);
            let to_screen =
                |x: f64, y: f64| pos2(min.x + x as f32 * scale, min.y + y as f32 * scale);

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
                        px as f32 / tw as f32 * size.x,
                        py as f32 / th as f32 * size.y,
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

            // The rhythm, as a light blue guide that stays out of the way of the objects: units a little
            // stronger than subdivisions (rung 4 bbq Q9; chris, 2026-11-10: "much lighter and perhaps
            // light blue").
            if let Some((xs, ys)) = &rhythm {
                let clip = painter.clip_rect().intersect(page_rect);
                let line = |major: bool| {
                    Stroke::new(
                        1.0 / ppp,
                        Color32::from_rgba_unmultiplied(120, 180, 240, if major { 60 } else { 26 }),
                    )
                };
                for &(x, major) in xs {
                    let sx = min.x + x as f32 * scale;
                    if sx >= clip.min.x && sx <= clip.max.x {
                        painter.line_segment(
                            [pos2(sx, clip.min.y), pos2(sx, clip.max.y)],
                            line(major),
                        );
                    }
                }
                for &(y, major) in ys {
                    let sy = min.y + y as f32 * scale;
                    if sy >= clip.min.y && sy <= clip.max.y {
                        painter.line_segment(
                            [pos2(clip.min.x, sy), pos2(clip.max.x, sy)],
                            line(major),
                        );
                    }
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

            // The selected objects.
            for ((x0, y0, x1, y1), corners) in &outlines {
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
                for (hit, hx, hy) in &handles {
                    let c = to_screen(*hx, *hy);
                    let r = Rect::from_center_size(c, vec2(8.0, 8.0));
                    // White on any theme: the handles sit on the page, which is always light.
                    painter.rect_filled(r, CornerRadius::ZERO, Color32::WHITE);
                    painter.rect_stroke(
                        r,
                        CornerRadius::ZERO,
                        Stroke::new(1.5, accent),
                        if matches!(hit, HandleHit::Box(_)) {
                            StrokeKind::Inside
                        } else {
                            StrokeKind::Middle
                        },
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
            if response.clicked()
                && let Some(p) = response.interact_pointer_pos()
            {
                let (x, y) = to_dev(p);
                if drawing_tool {
                    let _ = self.viewer.place_click(x, y, mods, reach);
                } else {
                    self.viewer.click(x, y, slack, mods.shift);
                }
                self.selected_layer = None;
            }

            // Where the pointer is, in device pixels, for the status bar.
            self.cursor_px = response.hover_pos().and_then(|p| {
                let x = (p.x - min.x) / scale;
                let y = (p.y - min.y) / scale;
                (x >= 0.0 && y >= 0.0 && x < w as f32 && y < h as f32).then_some((x, y))
            });

            // Ctrl + scroll (or a pinch) zooms by one step.
            if response.hovered() {
                let z = ui.input(|i| i.zoom_delta());
                if z > 1.0 {
                    self.viewer.zoom_step(scale * ppp, 1);
                } else if z < 1.0 {
                    self.viewer.zoom_step(scale * ppp, -1);
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
                            self.chip_state.record_chip(ms);
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

/// The backdrop round the page (napkin §14: it follows the theme; the page never does).
/// The mouse cursor over a handle.
/// A built-in device's name for the pickers; its id when it is not one.
fn device_name(id: &str) -> String {
    quadrule_profiles::builtin(id)
        .map(|p| p.display_name)
        .unwrap_or_else(|_| id.to_string())
}

/// A selected object's outline: its bounds, and its turned corners when it is a turned shape.
type Outline = ((f64, f64, f64, f64), Option<[(f64, f64); 4]>);

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
