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

//! The welcome screen (rung 16 bbq, Q2, Q4, Q5 and Defaulted "When the welcome screen shows"): one screen, actions on
//! the left, the gallery on the right. It replaces the empty start. It shows at start-up with no file, after
//! File > Close, and for File > New… over an open document, where Esc or Cancel goes back to the document.
//!
//! Down the left: Blank page (rung 5's New dialog with the Device filter's device chosen), Open…, the Recent list
//! and, small at the foot, Open sample page. On the right: the Device filter, the Search box, the category tabs and
//! the tiles. A category with nothing in it says its starters are coming (Q5); My templates lists the folder Save
//! as template… writes to (Q4).
//!
//! The words here never change a document themselves: every pick becomes an [`After`] and goes through
//! [`ViewerUi::guard`], so unsaved changes are asked about first.

use crate::templates::{Origin, TemplateEntry};
use crate::thumbs::{self, Job, Thumb, Thumbs};
use crate::view::{After, Request, ViewerUi};
use egui::{
    Align, Color32, ColorImage, FontId, Layout, Rect, RichText, Role, ScrollArea, Sense, Stroke,
    StrokeKind, TextureHandle, TextureOptions, WidgetInfo, pos2, vec2,
};
use quadrule_i18n::Message;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// The gallery's tabs (Q2, Q5), in the order they show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Category {
    /// Every starter, and the person's own templates.
    #[default]
    All,
    /// Ruled, dotted, squared and other paper.
    Paper,
    /// Note-taking pages.
    Notes,
    /// Planners.
    Planning,
    /// Bullet journal pages.
    BulletJournal,
    /// Creative pages.
    Creative,
    /// The templates the person kept (Q4).
    MyTemplates,
}

impl Category {
    /// Every tab, left to right.
    pub const ALL: [Category; 7] = [
        Category::All,
        Category::Paper,
        Category::Notes,
        Category::Planning,
        Category::BulletJournal,
        Category::Creative,
        Category::MyTemplates,
    ];

    /// The Fluent id of the tab's name.
    pub fn message_id(self) -> &'static str {
        match self {
            Category::All => "app-cat-all",
            Category::Paper => "app-cat-paper",
            Category::Notes => "app-cat-notes",
            Category::Planning => "app-cat-planning",
            Category::BulletJournal => "app-cat-bujo",
            Category::Creative => "app-cat-creative",
            Category::MyTemplates => "app-cat-mine",
        }
    }

    /// The category whose starters are kept in `folder`.
    pub fn from_folder(folder: &str) -> Option<Category> {
        Category::ALL
            .into_iter()
            .find(|c| c.folder() == Some(folder))
    }

    /// The folder the category's starters are kept in, under `crates/ui/starters/` (Defaulted "Where starters
    /// live"); `None` for the two tabs that are not one category of starters.
    pub fn folder(self) -> Option<&'static str> {
        match self {
            Category::All | Category::MyTemplates => None,
            Category::Paper => Some("paper"),
            Category::Notes => Some("notes"),
            Category::Planning => Some("planning"),
            Category::BulletJournal => Some("bullet-journal"),
            Category::Creative => Some("creative"),
        }
    }
}

/// What the welcome screen remembers while it shows (view state, never saved).
#[derive(Default)]
pub struct Welcome {
    /// It shows over an open document (File > New…), so Esc and Cancel can go back to it.
    pub over_document: bool,
    /// The tab on.
    pub category: Category,
    /// The Device filter, or `None` for the device New last used.
    pub device: Option<String>,
    /// The Search box.
    pub search: String,
    /// My templates as last read from the folder; `None` makes the next frame read it again.
    templates: Option<Vec<Arc<TemplateEntry>>>,
    /// The stored pictures, made into textures once each: by file and its time of last change.
    pictures: HashMap<(PathBuf, Option<SystemTime>), Option<TextureHandle>>,
    /// The built-in starters; `None` makes the next frame load them (the app's, or a test's own, see
    /// [`ViewerUi::set_starters`]).
    starters: Option<Vec<Arc<TemplateEntry>>>,
    /// Ask for every tile's picture, not just the ones on screen (the speed run, which times all of them).
    pub(crate) draw_all: bool,
    /// The live thumbnails and the thread that draws them (rung 16 bbq, Q3).
    pub(crate) thumbs: Thumbs,
    /// A template whose Remove waits on its confirm.
    pub(crate) confirm_remove: Option<PathBuf>,
    /// A recent file that was not found and was clicked: offered for removal from the list.
    pub(crate) gone: Option<PathBuf>,
    /// Save as template… found a template of that name and waits on Replace or Cancel.
    pub(crate) replace_template: bool,
}

/// What one frame of the screen asks for, done after the drawing so the screen's own state is not borrowed.
enum Act {
    Blank,
    Open,
    Sample,
    Cancel,
    Recent(PathBuf),
    Gone(PathBuf),
    ForgetRecent(PathBuf),
    Pick(Arc<TemplateEntry>),
    AskRemove(PathBuf),
    Reveal,
}

/// The welcome screen's speed run (`quadrule --bench-welcome`, rung 16 bbq, Defaulted "Speed"): the frames of the
/// screen are timed while its tiles are drawn on the thread, then while it sits with every tile drawn. Each frame is
/// forced (a repaint is asked for every time), which is harder than a real gallery, where an idle screen draws no
/// frames at all. One `CHECK` line, the same budget as a drag: the 90th percentile under 50 ms (ADR-009).
#[derive(Default)]
pub struct WelcomeBench {
    started: Option<std::time::Instant>,
    /// Frames timed with tiles still being drawn.
    pub drawing: crate::canvas::DragClock,
    /// Frames timed with every tile drawn.
    pub idle: crate::canvas::DragClock,
    /// How many idle frames to time.
    pub idle_frames: u32,
    /// Seconds from the first tile asked for to the last one drawn.
    pub all_drawn_s: Option<f64>,
    done: bool,
}

impl WelcomeBench {
    /// A run that times `idle_frames` idle frames after the tiles are drawn.
    pub fn new(idle_frames: u32) -> Self {
        WelcomeBench {
            idle_frames,
            ..Default::default()
        }
    }
}

/// A tile's picture box, in screen points, and the room for the name under it.
const TILE_W: f32 = 156.0;
const TILE_PICTURE_H: f32 = 208.0;
const TILE_NAME_H: f32 = 56.0;
/// The left column's width.
const LEFT_W: f32 = 270.0;

impl ViewerUi {
    /// Whether the welcome screen is what the window shows now: nothing is open, or File > New… put it over a
    /// document.
    pub fn welcome_showing(&self) -> bool {
        self.viewer.open.is_none() || self.welcome.over_document
    }

    /// Whether the welcome screen is over an open document (Esc or Cancel goes back to it).
    pub fn welcome_over_document(&self) -> bool {
        self.welcome.over_document && self.viewer.open.is_some()
    }

    /// Shows the welcome screen (File > New…, Ctrl+N): over the open document, if there is one. The device filter
    /// goes back to the device New last used, the search box empties, and My templates is read again.
    pub fn show_welcome(&mut self) {
        self.viewer.end_typing();
        self.welcome.over_document = self.viewer.open.is_some();
        self.welcome.device = None;
        self.welcome.search.clear();
        self.welcome.templates = None;
    }

    /// Goes back to the document (Esc, Cancel). Nothing happens when no document is open.
    pub fn close_welcome(&mut self) {
        self.welcome.over_document = false;
    }

    /// File > New… and Ctrl+N: the welcome screen, or, when it is already showing, the Blank page dialog.
    pub fn ask_new(&mut self) {
        if self.welcome_showing() {
            self.open_blank_page();
        } else {
            self.show_welcome();
        }
    }

    /// Blank page: rung 5's New dialog, with the Device filter's device chosen.
    pub fn open_blank_page(&mut self) {
        let device = self.welcome_device();
        self.set_new_device(&device);
        self.open_new_dialog();
    }

    /// File > Close (Ctrl+W): back to the welcome screen, after asking about unsaved changes.
    pub fn ask_close_document(&mut self) -> Vec<Request> {
        if self.viewer.open.is_none() {
            return Vec::new();
        }
        self.guard(After::CloseDocument)
    }

    /// The device the filter is on.
    pub fn welcome_device(&self) -> String {
        self.welcome
            .device
            .clone()
            .unwrap_or_else(|| self.new_device().to_string())
    }

    /// Sets the Device filter.
    pub fn set_welcome_device(&mut self, id: &str) {
        self.welcome.device = Some(id.to_string());
    }

    /// Empties the Search box.
    pub fn welcome_search_clear(&mut self) {
        self.welcome.search.clear();
    }

    /// Makes the next frame read My templates again (after a template was saved or removed).
    pub fn refresh_templates(&mut self) {
        self.welcome.templates = None;
    }

    /// Save as template…: writes the open document to My templates. A template of that name already there asks
    /// before it is replaced ([`ViewerUi::answer_replace_template`]).
    pub fn save_as_template(&mut self) {
        match self.viewer.save_as_template(false) {
            Err(crate::templates::TemplateError::Exists(_)) => self.welcome.replace_template = true,
            Ok(_) => self.refresh_templates(),
            Err(_) => {}
        }
    }

    /// The answer to "Replace template?".
    pub fn answer_replace_template(&mut self, replace: bool) {
        self.welcome.replace_template = false;
        if replace && self.viewer.save_as_template(true).is_ok() {
            self.refresh_templates();
        }
    }

    /// Whether the Replace template question is showing.
    pub fn replace_template_open(&self) -> bool {
        self.welcome.replace_template
    }

    /// Puts these starters in the gallery in place of the built-in ones (a test's own, or the hidden
    /// `--gallery-test-starters` of the app).
    pub fn set_starters(&mut self, list: Vec<crate::templates::TemplateEntry>) {
        self.welcome.starters = Some(list.into_iter().map(Arc::new).collect());
    }

    /// The tiles the tab and the Search box leave in, in the order they show: starters (by category, then name), then
    /// the person's templates. With text in the Search box every tab is looked through, and a tile matches on any part
    /// of its title, description, tags or category name, ignoring case (Defaulted "Search"); without, the tab's own.
    pub fn welcome_items(&mut self) -> Vec<Arc<TemplateEntry>> {
        let starters = self
            .welcome
            .starters
            .get_or_insert_with(|| {
                crate::starters::built_in()
                    .into_iter()
                    .map(Arc::new)
                    .collect()
            })
            .clone();
        let templates = self
            .welcome
            .templates
            .get_or_insert_with(|| self.viewer.templates().into_iter().map(Arc::new).collect())
            .clone();
        let words = self.welcome.search.trim().to_lowercase();
        let category = self.welcome.category;
        let label_of = |this: &Self, e: &TemplateEntry| -> String {
            match &e.origin {
                Origin::Template => this.t(Category::MyTemplates.message_id()),
                Origin::Starter { folder } => Category::from_folder(folder)
                    .map(|c| this.t(c.message_id()))
                    .unwrap_or_default(),
            }
            .to_lowercase()
        };
        let all = starters.into_iter().chain(templates);
        if !words.is_empty() {
            return all
                .filter(|t| {
                    t.name.to_lowercase().contains(&words)
                        || t.description.to_lowercase().contains(&words)
                        || t.tags.iter().any(|g| g.to_lowercase().contains(&words))
                        || label_of(self, t).contains(&words)
                })
                .collect();
        }
        all.filter(|t| match (&t.origin, category) {
            (_, Category::All) => true,
            (Origin::Template, Category::MyTemplates) => true,
            (Origin::Starter { folder }, c) => c.folder() == Some(folder.as_str()),
            _ => false,
        })
        .collect()
    }

    /// How many thumbnails are asked for and not finished.
    pub fn thumbs_pending(&self) -> usize {
        self.welcome.thumbs.pending()
    }

    /// How many thumbnails are drawn and ready.
    pub fn thumbs_ready(&self) -> usize {
        self.welcome.thumbs.ready()
    }

    /// How many thumbnails failed to draw.
    pub fn thumbs_failed(&self) -> usize {
        self.welcome.thumbs.failed()
    }

    /// How many thumbnail jobs were ever handed to the drawing thread.
    pub fn thumbs_jobs_sent(&self) -> u64 {
        self.welcome.thumbs.jobs_sent()
    }

    /// The threads that drew finished thumbnails.
    pub fn thumbs_drawn_by(&self) -> Vec<std::thread::ThreadId> {
        self.welcome.thumbs.drawn_by().to_vec()
    }

    /// Times a frame of the welcome screen for the speed run, and prints its line when the run is over.
    pub(crate) fn welcome_bench_frame(&mut self, ctx: &egui::Context, took: std::time::Duration) {
        let Some(mut b) = self.welcome_bench.take() else {
            return;
        };
        self.welcome.draw_all = true;
        ctx.request_repaint();
        let ms = took.as_secs_f64() * 1000.0;
        let jobs = self.welcome.thumbs.jobs_sent();
        if !b.done && self.welcome_showing() {
            if jobs > 0 && b.started.is_none() {
                b.started = Some(std::time::Instant::now());
            }
            if b.all_drawn_s.is_none() {
                if b.started.is_some() {
                    b.drawing.record(ms);
                }
                if b.started.is_some() && self.welcome.thumbs.pending() == 0 {
                    b.all_drawn_s = b.started.map(|t| t.elapsed().as_secs_f64());
                }
            } else {
                b.idle.record(ms);
                if b.idle.frames >= b.idle_frames {
                    b.done = true;
                    let verdict = |c: &crate::canvas::DragClock| {
                        if cfg!(debug_assertions) {
                            "n/a"
                        } else if c.within_budget() {
                            "pass"
                        } else {
                            "fail"
                        }
                    };
                    let (d, i) = (&b.drawing, &b.idle);
                    let result = if verdict(d) == "fail" || verdict(i) == "fail" {
                        "fail"
                    } else {
                        verdict(d)
                    };
                    println!(
                        "\nCHECK crate=ui name=welcome_bench result={result} build={} device={} tiles={} ready={} failed={} drawn_in_s={:.1} drawing_frames={} drawing_mean_ms={:.1} drawing_p90_ms={:.1} drawing_worst_ms={:.1} drawing_first_ms={:.1} idle_frames={} idle_mean_ms={:.1} idle_p90_ms={:.1} idle_worst_ms={:.1} budget_ms=50",
                        if cfg!(debug_assertions) {
                            "debug"
                        } else {
                            "release"
                        },
                        self.welcome_device(),
                        self.welcome.thumbs.ready() + self.welcome.thumbs.failed(),
                        self.welcome.thumbs.ready(),
                        self.welcome.thumbs.failed(),
                        b.all_drawn_s.unwrap_or(0.0),
                        d.frames,
                        d.mean_ms(),
                        d.percentile_ms(),
                        d.worst_ms,
                        d.first_ms,
                        i.frames,
                        i.mean_ms(),
                        i.percentile_ms(),
                        i.worst_ms,
                    );
                }
            }
        }
        self.welcome_bench = Some(b);
    }

    /// Whether the welcome speed run has printed its line.
    pub fn welcome_bench_finished(&self) -> bool {
        self.welcome_bench.as_ref().is_some_and(|b| b.done)
    }

    /// Chooses a tab (`--look gallery-mine` starts on My templates).
    pub fn set_welcome_category(&mut self, category: Category) {
        self.welcome.category = category;
    }

    /// The welcome screen, into the central panel.
    pub(crate) fn welcome_screen(&mut self, ui: &mut egui::Ui, requests: &mut Vec<Request>) {
        let mut acts: Vec<Act> = Vec::new();
        let recent = self.viewer.recent_files();
        let over = self.welcome_over_document();
        egui::Panel::left("welcome-left")
            .resizable(false)
            .exact_size(LEFT_W)
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.heading(self.t("app-title"));
                    if over {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button(self.t("app-cancel")).clicked() {
                                acts.push(Act::Cancel);
                            }
                        });
                    }
                });
                ui.add_space(8.0);
                let wide = vec2(ui.available_width(), 30.0);
                if ui
                    .add_sized(wide, egui::Button::new(self.t("app-welcome-blank")))
                    .clicked()
                {
                    acts.push(Act::Blank);
                }
                if ui
                    .add_sized(wide, egui::Button::new(self.t("app-open")))
                    .clicked()
                {
                    acts.push(Act::Open);
                }
                ui.add_space(10.0);
                ui.label(RichText::new(self.t("app-welcome-recent")).strong());
                ui.separator();
                // The small link at the foot, then the list in the room that is left.
                egui::Panel::bottom("welcome-left-foot")
                    .frame(egui::Frame::NONE)
                    .show_separator_line(false)
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        if ui.link(self.t("app-open-sample")).clicked() {
                            acts.push(Act::Sample);
                        }
                        ui.add_space(2.0);
                        ui.weak(self.t("app-empty-drop"));
                        ui.add_space(4.0);
                    });
                ScrollArea::vertical()
                    .id_salt("welcome-recent")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if recent.is_empty() {
                            ui.weak(self.t("app-welcome-recent-none"));
                        }
                        for r in &recent {
                            self.recent_line(ui, r, &mut acts);
                        }
                    });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::same(10)))
            .show(ui, |ui| self.gallery(ui, &mut acts));
        self.welcome_acts(acts, requests);
    }

    /// One line of the Recent list: the name, and the folder under it; a file that is gone is greyed and says so.
    fn recent_line(&self, ui: &mut egui::Ui, r: &crate::recent::RecentFile, acts: &mut Vec<Act>) {
        let name = if r.exists {
            RichText::new(&r.name)
        } else {
            RichText::new(&r.name).weak()
        };
        let resp = ui
            .add(
                egui::Button::new(name)
                    .frame(false)
                    .min_size(vec2(ui.available_width(), 0.0)),
            )
            .on_hover_text(r.path.display().to_string());
        let sub = if r.exists {
            r.folder.clone()
        } else {
            self.strings
                .format(&Message::new("app-welcome-not-found-in").text("folder", r.folder.clone()))
        };
        ui.add(egui::Label::new(RichText::new(sub).small().weak()).truncate());
        ui.add_space(4.0);
        if resp.clicked() {
            acts.push(if r.exists {
                Act::Recent(r.path.clone())
            } else {
                Act::Gone(r.path.clone())
            });
        }
        resp.context_menu(|ui| {
            if ui.button(self.t("app-welcome-remove-list")).clicked() {
                acts.push(Act::ForgetRecent(r.path.clone()));
                ui.close();
            }
        });
    }

    /// The right side: Device, Search, the tabs, and the tiles or the line that says why there are none.
    fn gallery(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Act>) {
        let mut device = self.welcome_device();
        let mut search = std::mem::take(&mut self.welcome.search);
        ui.horizontal_wrapped(|ui| {
            let label = ui.label(self.t("app-device"));
            self.device_picker(ui, "welcome-device", &mut device)
                .labelled_by(label.id);
            ui.add_space(16.0);
            let label = ui.label(self.t("app-welcome-search"));
            ui.add(
                egui::TextEdit::singleline(&mut search)
                    .hint_text(self.t("app-welcome-search-hint"))
                    .desired_width(220.0),
            )
            .labelled_by(label.id);
        });
        self.welcome.search = search;
        self.welcome.device = Some(device);
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for c in Category::ALL {
                let on = self.welcome.category == c && self.welcome.search.trim().is_empty();
                if ui
                    .add(egui::Button::selectable(on, self.t(c.message_id())))
                    .clicked()
                {
                    self.welcome.category = c;
                    // A tab chosen ends a search, which looks at every tab.
                    self.welcome.search.clear();
                }
            }
        });
        ui.separator();
        let tiles = self.welcome_items();
        let device = self.welcome_device();
        let ctx = ui.ctx().clone();
        let ppp = ctx.pixels_per_point();
        self.welcome.thumbs.poll(&ctx);
        let mut wanted: Vec<Job> = Vec::new();
        let searching = !self.welcome.search.trim().is_empty();
        ScrollArea::vertical()
            .id_salt("welcome-gallery")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if tiles.is_empty() {
                    let line = if searching {
                        self.strings.format(
                            &Message::new("app-welcome-nomatch")
                                .text("text", self.welcome.search.trim()),
                        )
                    } else {
                        match self.welcome.category {
                            Category::MyTemplates => self.t("app-welcome-mine-none"),
                            Category::All => self.t("app-welcome-soon-all"),
                            _ => self.t("app-welcome-soon"),
                        }
                    };
                    ui.add_space(12.0);
                    ui.label(line);
                    return;
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
                    for t in &tiles {
                        self.tile(ui, t, &device, ppp, acts, &mut wanted);
                    }
                });
            });
        // What the tiles on screen want, in screen order; nothing else is drawn, and nothing at all is asked of the
        // thread while the same tiles are showing.
        self.welcome.thumbs.want(&ctx, wanted);
    }

    /// The Device filter's drop-down: the devices, then the user's own, then the paper sizes (Defaulted "The
    /// device filter").
    pub(crate) fn device_picker(
        &self,
        ui: &mut egui::Ui,
        salt: &str,
        value: &mut String,
    ) -> egui::Response {
        egui::ComboBox::from_id_salt(salt)
            .selected_text(self.full_name(value))
            .show_ui(ui, |ui| {
                for id in quadrule_profiles::active::device_ids() {
                    ui.selectable_value(value, id.clone(), self.full_name(&id));
                }
                // The paper sizes are their own group after the devices (rung 13b).
                ui.separator();
                ui.label(RichText::new(self.t("new-group-paper")).small().strong());
                for id in quadrule_profiles::active::paper_ids() {
                    ui.selectable_value(value, id.clone(), self.full_name(&id));
                }
            })
            .response
    }

    /// One tile: the live thumbnail for the filter's device once it is drawn (the stored picture stands in until
    /// then; a page that fails to draw is a plain box), and the name under it. A page that does not target the
    /// filter's device is drawn on its own main device, and the tile says so. A click opens an untitled copy; a
    /// template has a right-click menu with Remove and Show in folder.
    fn tile(
        &mut self,
        ui: &mut egui::Ui,
        t: &Arc<TemplateEntry>,
        device: &str,
        ppp: f32,
        acts: &mut Vec<Act>,
        wanted: &mut Vec<Job>,
    ) {
        let (rect, resp) =
            ui.allocate_exact_size(vec2(TILE_W, TILE_PICTURE_H + TILE_NAME_H), Sense::click());
        resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, t.name.clone()));
        let drawn_on = t.drawn_on(device);
        // The picture: asked for only when the tile is on screen.
        let mut state = None;
        if !t.unreadable
            && !drawn_on.is_empty()
            && (ui.is_rect_visible(rect) || self.welcome.draw_all)
        {
            let key = thumbs::Key {
                id: t.path.display().to_string(),
                device: drawn_on.clone(),
                stamp: t
                    .modified
                    .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_nanos()),
                box_px: (
                    ((TILE_W - 12.0) * ppp).round() as u32,
                    ((TILE_PICTURE_H - 12.0) * ppp).round() as u32,
                ),
            };
            wanted.push(Job {
                source: match &t.bytes {
                    Some(b) => thumbs::Source::Bytes(b.clone()),
                    None => thumbs::Source::File(t.path.clone()),
                },
                key: key.clone(),
            });
            state = Some(key);
        }
        let live = state
            .as_ref()
            .and_then(|k| self.welcome.thumbs.get(k))
            .map(|th| match th {
                Thumb::Ready(tex) => Some(Some(tex.clone())),
                Thumb::Failed => Some(None),
                Thumb::Pending => None,
            })
            .unwrap_or(None);
        // Drawn: that picture. Failed: a plain box. Otherwise the stored picture, if there is one.
        let tex = match live {
            Some(live) => live,
            None => self.template_picture(ui.ctx(), t),
        };
        let paper = Rect::from_min_size(rect.min, vec2(TILE_W, TILE_PICTURE_H));
        let painter = ui.painter_at(rect);
        let visuals = ui.visuals();
        painter.rect_filled(paper, 4.0, visuals.extreme_bg_color);
        match &tex {
            Some(tex) => {
                let s = tex.size_vec2();
                let fit = ((paper.width() - 12.0) / s.x).min((paper.height() - 12.0) / s.y);
                let size = s * fit;
                let img = Rect::from_center_size(paper.center(), size);
                painter.image(
                    tex.id(),
                    img,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.rect_stroke(
                    img,
                    0.0,
                    Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color),
                    StrokeKind::Outside,
                );
            }
            None => {
                // A plain page outline (drawn, not a glyph: egui's fonts panic on one they lack).
                let outline = Rect::from_center_size(paper.center(), vec2(54.0, 72.0));
                painter.rect_stroke(
                    outline,
                    3.0,
                    Stroke::new(1.5, visuals.weak_text_color()),
                    StrokeKind::Inside,
                );
            }
        }
        let edge = if resp.hovered() || resp.has_focus() {
            visuals.widgets.hovered.fg_stroke
        } else {
            visuals.widgets.noninteractive.bg_stroke
        };
        painter.rect_stroke(paper, 4.0, edge, StrokeKind::Inside);
        let mut job = egui::text::LayoutJob::single_section(
            t.name.clone(),
            egui::TextFormat::simple(FontId::proportional(14.0), visuals.text_color()),
        );
        job.wrap.max_width = TILE_W;
        job.wrap.max_rows = 2;
        job.halign = Align::Center;
        let galley = painter.layout_job(job);
        let name_h = galley.size().y;
        painter.galley(
            // The galley is centred on this point (its lines are centred), so no offset for its width.
            pos2(rect.center().x, paper.bottom() + 4.0),
            galley,
            visuals.text_color(),
        );
        if !t.unreadable && !drawn_on.is_empty() && drawn_on != device {
            let note = self.strings.format(
                &Message::new("app-welcome-tile-on")
                    .text("device", crate::viewer::device_label(&drawn_on)),
            );
            // A label of its own, so it is read out and found by name; it takes no clicks (they go to the tile).
            let at = Rect::from_min_size(
                pos2(rect.min.x, paper.bottom() + 6.0 + name_h),
                vec2(TILE_W, 14.0),
            );
            // A child of this tile, not a widget of the row: it must take no room in the wrapping layout.
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(at)
                    .layout(Layout::centered_and_justified(egui::Direction::TopDown)),
            );
            child.add(egui::Label::new(RichText::new(note).size(11.0).weak()).truncate());
        }
        if resp.clicked() {
            acts.push(Act::Pick(t.clone()));
        }
        if t.origin == Origin::Template {
            resp.context_menu(|ui| {
                if ui.button(self.t("app-welcome-remove")).clicked() {
                    acts.push(Act::AskRemove(t.path.clone()));
                    ui.close();
                }
                if ui.button(self.t("app-welcome-show-folder")).clicked() {
                    acts.push(Act::Reveal);
                    ui.close();
                }
            });
        }
    }

    /// A template's stored picture as a texture, made once for each state of the file.
    fn template_picture(
        &mut self,
        ctx: &egui::Context,
        t: &TemplateEntry,
    ) -> Option<TextureHandle> {
        let key = (t.path.clone(), t.modified);
        self.welcome
            .pictures
            .entry(key)
            .or_insert_with(|| {
                let bytes = t.preview.as_ref()?;
                let img = decode_gray_png(bytes)?;
                Some(ctx.load_texture(
                    format!("template-{}", t.path.display()),
                    img,
                    TextureOptions::LINEAR,
                ))
            })
            .clone()
    }

    fn welcome_acts(&mut self, acts: Vec<Act>, requests: &mut Vec<Request>) {
        for act in acts {
            match act {
                Act::Blank => self.open_blank_page(),
                Act::Open => requests.extend(self.guard(After::OpenDialog)),
                Act::Sample => requests.extend(self.guard(After::OpenSample)),
                Act::Cancel => self.close_welcome(),
                Act::Recent(p) => requests.extend(self.guard(After::OpenPath(p))),
                // A click on a file that is not there offers to take it off the list.
                Act::Gone(p) => self.welcome.gone = Some(p),
                Act::ForgetRecent(p) => self.viewer.forget_recent(&p),
                Act::Pick(entry) => {
                    let device_id = self.welcome_device();
                    let after = match (&entry.origin, &entry.bytes) {
                        (Origin::Starter { .. }, Some(bytes)) => After::OpenStarter {
                            bytes: bytes.clone(),
                            device_id,
                        },
                        _ => After::OpenTemplate {
                            path: entry.path.clone(),
                            device_id,
                        },
                    };
                    requests.extend(self.guard(after));
                }
                Act::AskRemove(p) => self.welcome.confirm_remove = Some(p),
                Act::Reveal => {
                    if let Some(d) = self.viewer.templates_dir.clone() {
                        requests.push(Request::RevealFolder(d));
                    }
                }
            }
        }
    }

    /// The questions the welcome screen and Save as template… ask: Remove this template? Not found, remove it from
    /// the list? Replace this template? Each is a small modal; one at a time.
    pub(crate) fn welcome_dialogs(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.welcome.confirm_remove.clone() {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let name = self
                .welcome
                .templates
                .as_ref()
                .and_then(|l| l.iter().find(|t| t.path == path))
                .map_or(name, |t| t.name.clone());
            let mut answer = None;
            egui::Modal::new(egui::Id::unique("template-remove")).show(ctx, |ui| {
                ui.set_max_width(380.0);
                ui.heading(self.t("app-welcome-remove-title"));
                ui.add_space(6.0);
                ui.label(
                    self.strings
                        .format(&Message::new("app-welcome-remove-body").text("name", name)),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t("app-welcome-remove")).clicked() {
                        answer = Some(true);
                    }
                    if ui.button(self.t("app-cancel")).clicked() {
                        answer = Some(false);
                    }
                });
            });
            match answer {
                Some(true) => {
                    self.welcome.confirm_remove = None;
                    let _ = self.viewer.remove_template(&path);
                    self.refresh_templates();
                }
                Some(false) => self.welcome.confirm_remove = None,
                None => {}
            }
            return;
        }
        if let Some(path) = self.welcome.gone.clone() {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let mut answer = None;
            egui::Modal::new(egui::Id::unique("recent-gone")).show(ctx, |ui| {
                ui.set_max_width(380.0);
                ui.heading(self.t("app-welcome-gone-title"));
                ui.add_space(6.0);
                ui.label(
                    self.strings
                        .format(&Message::new("app-welcome-gone-body").text("name", name)),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t("app-welcome-remove-list")).clicked() {
                        answer = Some(true);
                    }
                    if ui.button(self.t("app-cancel")).clicked() {
                        answer = Some(false);
                    }
                });
            });
            if let Some(remove) = answer {
                self.welcome.gone = None;
                if remove {
                    self.viewer.forget_recent(&path);
                }
            }
            return;
        }
        if self.welcome.replace_template {
            let name = self.viewer.title().unwrap_or_default().to_string();
            let mut answer = None;
            egui::Modal::new(egui::Id::unique("template-replace")).show(ctx, |ui| {
                ui.set_max_width(380.0);
                ui.heading(self.t("app-template-replace-title"));
                ui.add_space(6.0);
                ui.label(
                    self.strings
                        .format(&Message::new("app-template-replace-body").text("name", name)),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(self.t("app-template-replace")).clicked() {
                        answer = Some(true);
                    }
                    if ui.button(self.t("app-cancel")).clicked() {
                        answer = Some(false);
                    }
                });
            });
            if let Some(replace) = answer {
                self.answer_replace_template(replace);
            }
        }
    }

    /// Whether a question of the welcome screen (Remove template, Not found, Replace template) is showing.
    pub fn welcome_question_open(&self) -> bool {
        self.welcome.confirm_remove.is_some()
            || self.welcome.gone.is_some()
            || self.welcome.replace_template
    }
}

/// An 8-bit gray PNG (a stored preview) as an image; `None` for anything else.
fn decode_gray_png(bytes: &[u8]) -> Option<ColorImage> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let (w, h) = (info.width as usize, info.height as usize);
    let data = &buf[..info.buffer_size()];
    match info.color_type {
        png::ColorType::Grayscale if data.len() == w * h => {
            Some(ColorImage::from_gray([w, h], data))
        }
        png::ColorType::Rgb if data.len() == w * h * 3 => Some(ColorImage::from_rgb([w, h], data)),
        png::ColorType::Rgba if data.len() == w * h * 4 => {
            Some(ColorImage::from_rgba_unmultiplied([w, h], data))
        }
        _ => None,
    }
}
