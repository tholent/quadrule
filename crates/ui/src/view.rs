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

use crate::editor::{PICK_SLACK_POINTS, Toward};
use crate::panels::{Renaming, Tab};
use crate::recovery::{Autosave, Recovered, Recovery};
use crate::viewer::{
    CARD_WIDTH_MM, ExportWhich, PIXEL_GRID_FROM, Source, TOOLBARS_IN_USE, Tile, Viewer, Zoom,
    ruler_px_per_mm, shrink, tiles,
};
use egui::{
    Align2, Color32, ColorImage, CornerRadius, FontId, Key, KeyboardShortcut, Modifiers, Rect,
    ScrollArea, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, ThemePreference, pos2,
    vec2,
};
use quadrule_i18n::{Message, Strings};
use quadrule_lint::Toolbar;
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
    /// Close the window.
    Close,
}

/// Texture tiles are at most this many pixels a side.
const TILE: u32 = 1024;

/// What the page texture shows: which document, which device, at what size.
#[derive(Clone, Debug, PartialEq)]
struct TextureKey {
    generation: u64,
    revision: u64,
    profile: String,
    size: (u32, u32),
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
    texture: Option<(TextureKey, Vec<(Tile, TextureHandle)>)>,
    ruler: Option<Ruler>,
    pub(crate) scroll_to_selected: bool,
    cursor_px: Option<(f32, f32)>,
    canvas_size: Option<egui::Vec2>,
    pub(crate) selected_layer: Option<usize>,
    pub(crate) renaming: Option<Renaming>,
    /// What is typed in a length field while it has focus, by field key.
    pub(crate) field_text: HashMap<String, String>,
    pub(crate) link_margins: bool,
    pub(crate) link_rhythm: bool,
    /// Waiting on the unsaved-changes prompt.
    pending: Option<After>,
    /// Recovery files found on start, offered one at a time.
    offers: Vec<Recovered>,
    autosave: Autosave,
    was_focused: bool,
}

impl ViewerUi {
    /// A window on the empty start screen.
    pub fn new(viewer: Viewer) -> Self {
        ViewerUi {
            viewer,
            strings: Strings::english(),
            theme: ThemePreference::System,
            show_lint: true,
            tab: Tab::Page,
            recovery: None,
            texture: None,
            ruler: None,
            scroll_to_selected: false,
            cursor_px: None,
            canvas_size: None,
            selected_layer: None,
            renaming: None,
            field_text: HashMap::new(),
            link_margins: true,
            link_rhythm: true,
            pending: None,
            offers: Vec::new(),
            autosave: Autosave::default(),
            was_focused: true,
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
        self.shortcuts(&ctx, &mut requests);
        self.tick_autosave(&ctx);
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui, &mut requests));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        if self.viewer.open.is_some() {
            egui::Panel::left("layers")
                .default_size(220.0)
                .show(ui, |ui| self.layers_panel(ui));
        }
        if self.show_lint && self.viewer.open.is_some() {
            egui::Panel::right("inspector")
                .default_size(320.0)
                .show(ui, |ui| self.inspector(ui));
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
        if self.viewer.open.is_none() || self.pending.is_some() {
            return;
        }
        let cmd = Modifiers::COMMAND;
        let shift_cmd = Modifiers::COMMAND | Modifiers::SHIFT;
        let key = |m: Modifiers, k: Key| {
            ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k)))
        };
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
        if key(Modifiers::NONE, Key::Delete) || key(Modifiers::NONE, Key::Backspace) {
            let _ = self.viewer.delete_selection();
        }
        if key(cmd, Key::CloseBracket) {
            self.move_selected(Toward::Front);
        } else if key(cmd, Key::OpenBracket) {
            self.move_selected(Toward::Back);
        }
        if key(Modifiers::NONE, Key::Escape) {
            self.viewer.selection.clear();
            self.selected_layer = None;
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
            After::OpenPath(p) => {
                if self.viewer.open_path(&p).is_ok() {
                    self.after_open();
                }
                Vec::new()
            }
            After::Close => vec![Request::Close],
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
                // Shows the Lint tab, or hides the right panel when it is already showing.
                if on {
                    self.show_lint = false;
                } else {
                    self.show_lint = true;
                    self.tab = Tab::Lint;
                }
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

    pub(crate) fn lint_count(&mut self) -> usize {
        self.viewer.page().map_or(0, |p| p.findings.len())
    }

    fn empty(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.heading(self.t("app-empty-title"));
            ui.add_space(12.0);
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

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        let avail = ui.available_size();
        self.canvas_size = Some(avail);
        let Some(scale) = self.viewer.scale((avail.x, avail.y), ppp) else {
            return;
        };
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
        if self.texture.as_ref().is_none_or(|(k, _)| *k != key) {
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
        }
        let Some((_, tiles)) = &self.texture else {
            return;
        };
        let tiles: Vec<_> = tiles.iter().map(|(t, tex)| (*t, tex.id())).collect();
        let zones: Vec<_> = page.profile.safe_zones.clone();
        let area = selected
            .and_then(|i| page.findings.get(i))
            .and_then(|f| f.area);
        // The selection's outlines: an overlay, never exported (rung 3 bbq, Defaulted
        // "Selecting").
        let outlines: Vec<(f64, f64, f64, f64)> = page
            .placed
            .iter()
            .filter(|p| selection.contains(&p.id))
            .map(|p| p.shape.bounds())
            .collect();

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
            let (rect, response) = ui.allocate_exact_size(total, Sense::click());
            // The page's corner sits on a whole screen pixel, so 1:1 and the zoom steps stay
            // sharp.
            let min = rect.center() - size / 2.0;
            let min = pos2((min.x * ppp).round() / ppp, (min.y * ppp).round() / ppp);
            let page_rect = Rect::from_min_size(min, size);
            let painter = ui.painter_at(rect);
            let to_screen =
                |x: f64, y: f64| pos2(min.x + x as f32 * scale, min.y + y as f32 * scale);

            painter.rect_stroke(
                page_rect.expand(1.0),
                CornerRadius::ZERO,
                Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
                StrokeKind::Outside,
            );
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

            // A grid between device pixels from 8× (Q10), only where it shows.
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

            // Safe zones the linter checks (Q6): fixed ones red, movable ones amber.
            if self.viewer.show_safe_zones {
                for z in &zones {
                    if let Ok(Some(t)) = Toolbar::of_zone(&z.name)
                        && !TOOLBARS_IN_USE.contains(&t)
                    {
                        continue;
                    }
                    let (x, y, zw, zh) = z.rect;
                    let r = Rect::from_min_max(
                        to_screen(x as f64, y as f64),
                        to_screen((x + zw) as f64, (y + zh) as f64),
                    );
                    let color = if z.advisory {
                        Color32::from_rgb(230, 150, 0)
                    } else {
                        Color32::from_rgb(220, 40, 40)
                    };
                    painter.rect_filled(r, CornerRadius::ZERO, color.gamma_multiply(0.18));
                    painter.rect_stroke(
                        r,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, color),
                        StrokeKind::Inside,
                    );
                    painter.text(
                        r.left_top() + vec2(4.0, 4.0),
                        Align2::LEFT_TOP,
                        self.strings.get(&format!("lint-zone-{}", z.name)),
                        FontId::proportional(12.0),
                        color,
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
            let accent = ui.visuals().selection.stroke.color;
            for (x0, y0, x1, y1) in &outlines {
                let r = Rect::from_min_max(to_screen(*x0, *y0), to_screen(*x1, *y1)).expand(2.0);
                painter.rect_stroke(
                    r,
                    CornerRadius::ZERO,
                    Stroke::new(1.5, accent),
                    StrokeKind::Outside,
                );
            }

            // A click picks the object under the pointer; Shift adds to the selection.
            if response.clicked()
                && let Some(p) = response.interact_pointer_pos()
            {
                let x = ((p.x - min.x) / scale) as f64;
                let y = ((p.y - min.y) / scale) as f64;
                let add = ui.input(|i| i.modifiers.shift);
                let slack = PICK_SLACK_POINTS / scale as f64;
                self.viewer.click(x, y, slack, add);
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
