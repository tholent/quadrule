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

//! The menus (fixes batch 2, docs/bbq/2026-11-18-fixes-before-16b.md, Q3, Q6 and Q7): one table that says what each
//! menu holds, read by the menu bar along the top of the window on Windows and Linux ([`crate::menu_bar`]) and by the
//! Mac's system menu bar (the app crate), so the two can never drift. Every item is a [`Cmd`]; [`ViewerUi::run_cmd`]
//! does it and [`ViewerUi::cmd_state`] says whether it is greyed or ticked. The keys themselves are read in
//! `ViewerUi::shortcuts`; an item with a key does what that key does.
//!
//! The Edit commands go through the keyboard path where a text field or typing on the page has the keys: on the Mac the
//! system menu takes ⌘C before egui sees it, so the item hands egui the event the key would have made, and the field
//! still copies, pastes and undoes its own text, exactly once.

use crate::CanvasMode;
use crate::view::{After, Request, ViewerUi};
use egui::{Event, Key, KeyboardShortcut, Modifiers};

/// Where the project lives; the Help menu opens it and the About sheet shows it.
pub const PROJECT_URL: &str = "https://github.com/tholent/quadrule";

/// The seven snap and overlay switches (the Page tab holds them too; they are app settings, kept between runs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snap {
    /// Pixel grid.
    Pixel,
    /// Rhythm.
    Rhythm,
    /// Margins and safe zones.
    Margins,
    /// Point snap.
    Points,
    /// Smart guides.
    Smart,
    /// Equal spacing.
    Equal,
    /// Rhythm overlay.
    Overlay,
}

impl Snap {
    /// All seven, in the order the menu lists them.
    pub const ALL: [Snap; 7] = [
        Snap::Pixel,
        Snap::Rhythm,
        Snap::Margins,
        Snap::Points,
        Snap::Smart,
        Snap::Equal,
        Snap::Overlay,
    ];

    /// The word on the switch.
    pub fn label(self) -> &'static str {
        match self {
            Snap::Pixel => "app-snap-pixel",
            Snap::Rhythm => "app-snap-rhythm",
            Snap::Margins => "app-snap-margins",
            Snap::Points => "app-snap-points",
            Snap::Smart => "app-snap-smart",
            Snap::Equal => "app-snap-equal",
            Snap::Overlay => "app-rhythm-overlay",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Snap::Pixel => "pixel",
            Snap::Rhythm => "rhythm",
            Snap::Margins => "margins",
            Snap::Points => "points",
            Snap::Smart => "smart",
            Snap::Equal => "equal",
            Snap::Overlay => "overlay",
        }
    }
}

/// What a menu item does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    /// File > New…
    New,
    /// File > Open…
    Open,
    /// File > Open Recent > the nth recent file.
    OpenRecent(usize),
    /// File > Save.
    Save,
    /// File > Save As…
    SaveAs,
    /// File > Save as template…
    SaveTemplate,
    /// File > Export…
    Export,
    /// File > Close.
    Close,
    /// Edit > Undo.
    Undo,
    /// Edit > Redo.
    Redo,
    /// Edit > Cut.
    Cut,
    /// Edit > Copy.
    Copy,
    /// Edit > Paste.
    Paste,
    /// Edit > Duplicate.
    Duplicate,
    /// Edit > Select all.
    SelectAll,
    /// View > Zoom in.
    ZoomIn,
    /// View > Zoom out.
    ZoomOut,
    /// View > Fit.
    ZoomFit,
    /// View > Actual size (1:1).
    ZoomActual,
    /// View > Real size check…
    RealSizeCheck,
    /// View > Device previews.
    Strip,
    /// View > Snapping > one switch.
    Snap(Snap),
    /// View > Show safe zones.
    SafeZones,
    /// View > Canvas > one choice.
    Canvas(CanvasMode),
    /// Help > About quadrule (on the Mac, in the Quadrule menu).
    About,
    /// Help > Keyboard shortcuts.
    ShortcutsSheet,
    /// Help > Developer > Export as readable JSON…
    ExportJson,
    /// Help > the project page.
    ProjectPage,
    /// File > Preferences… (the Mac: Quadrule > Preferences…): the Preferences window.
    Preferences,
    /// Quadrule > Quit (the Mac only).
    Quit,
}

impl Cmd {
    /// The row of [`crate::keys::table`] whose key does this, if a key does.
    pub fn act(self) -> Option<crate::keys::Act> {
        use crate::keys::Act;
        Some(match self {
            Cmd::New => Act::New,
            Cmd::Open => Act::Open,
            Cmd::Save => Act::Save,
            Cmd::SaveAs => Act::SaveAs,
            Cmd::Export => Act::Export,
            Cmd::Close => Act::Close,
            Cmd::Preferences => Act::Preferences,
            Cmd::Quit => Act::Quit,
            Cmd::Undo => Act::Undo,
            Cmd::Redo => Act::Redo,
            Cmd::Cut => Act::Cut,
            Cmd::Copy => Act::Copy,
            Cmd::Paste => Act::Paste,
            Cmd::Duplicate => Act::Duplicate,
            Cmd::SelectAll => Act::SelectAll,
            Cmd::ZoomIn => Act::ZoomIn,
            Cmd::ZoomOut => Act::ZoomOut,
            Cmd::ZoomFit => Act::ZoomFit,
            Cmd::ZoomActual => Act::ZoomActual,
            Cmd::ShortcutsSheet => Act::ShortcutsSheet,
            _ => return None,
        })
    }

    /// A name for the command that survives a trip through the Mac's menu: [`Cmd::from_id`] reads it back.
    pub fn id(self) -> String {
        let s = |t: &str| t.to_string();
        match self {
            Cmd::New => s("new"),
            Cmd::Open => s("open"),
            Cmd::OpenRecent(i) => format!("recent-{i}"),
            Cmd::Save => s("save"),
            Cmd::SaveAs => s("save-as"),
            Cmd::SaveTemplate => s("save-template"),
            Cmd::Export => s("export"),
            Cmd::Close => s("close"),
            Cmd::Undo => s("undo"),
            Cmd::Redo => s("redo"),
            Cmd::Cut => s("cut"),
            Cmd::Copy => s("copy"),
            Cmd::Paste => s("paste"),
            Cmd::Duplicate => s("duplicate"),
            Cmd::SelectAll => s("select-all"),
            Cmd::ZoomIn => s("zoom-in"),
            Cmd::ZoomOut => s("zoom-out"),
            Cmd::ZoomFit => s("zoom-fit"),
            Cmd::ZoomActual => s("zoom-actual"),
            Cmd::RealSizeCheck => s("real-size"),
            Cmd::Strip => s("strip"),
            Cmd::Snap(k) => format!("snap-{}", k.key()),
            Cmd::SafeZones => s("safe-zones"),
            Cmd::Canvas(CanvasMode::ExportPixels) => s("canvas-export"),
            Cmd::Canvas(CanvasMode::Chip) => s("canvas-chip"),
            Cmd::About => s("about"),
            Cmd::ShortcutsSheet => s("shortcuts"),
            Cmd::ExportJson => s("export-json"),
            Cmd::ProjectPage => s("project-page"),
            Cmd::Preferences => s("preferences"),
            Cmd::Quit => s("quit"),
        }
    }

    /// The command [`Cmd::id`] named, if it names one.
    pub fn from_id(id: &str) -> Option<Cmd> {
        if let Some(n) = id.strip_prefix("recent-") {
            return n.parse().ok().map(Cmd::OpenRecent);
        }
        if let Some(k) = id.strip_prefix("snap-") {
            return Snap::ALL.into_iter().find(|s| s.key() == k).map(Cmd::Snap);
        }
        Some(match id {
            "new" => Cmd::New,
            "open" => Cmd::Open,
            "save" => Cmd::Save,
            "save-as" => Cmd::SaveAs,
            "save-template" => Cmd::SaveTemplate,
            "export" => Cmd::Export,
            "close" => Cmd::Close,
            "undo" => Cmd::Undo,
            "redo" => Cmd::Redo,
            "cut" => Cmd::Cut,
            "copy" => Cmd::Copy,
            "paste" => Cmd::Paste,
            "duplicate" => Cmd::Duplicate,
            "select-all" => Cmd::SelectAll,
            "zoom-in" => Cmd::ZoomIn,
            "zoom-out" => Cmd::ZoomOut,
            "zoom-fit" => Cmd::ZoomFit,
            "zoom-actual" => Cmd::ZoomActual,
            "real-size" => Cmd::RealSizeCheck,
            "strip" => Cmd::Strip,
            "safe-zones" => Cmd::SafeZones,
            "canvas-export" => Cmd::Canvas(CanvasMode::ExportPixels),
            "canvas-chip" => Cmd::Canvas(CanvasMode::Chip),
            "about" => Cmd::About,
            "shortcuts" => Cmd::ShortcutsSheet,
            "export-json" => Cmd::ExportJson,
            "project-page" => Cmd::ProjectPage,
            "preferences" => Cmd::Preferences,
            "quit" => Cmd::Quit,
            _ => return None,
        })
    }
}

/// How an item is drawn when it is ticked or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Does something when picked.
    Action,
    /// A switch: ticked or not.
    Check,
    /// One of a few: the ticked one is the one in force.
    Radio,
}

/// One item of a menu.
#[derive(Clone, Debug)]
pub struct Item {
    /// What it does.
    pub cmd: Cmd,
    /// The Fluent id of its words.
    pub label: &'static str,
    /// Its key, if it has one.
    pub shortcut: Option<KeyboardShortcut>,
    /// Whether it is ticked, one of a few, or neither.
    pub kind: Kind,
}

/// The system's own items on the Mac (the system does these, not quadrule).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Native {
    /// Hide Quadrule (⌘H).
    Hide,
    /// Hide Others.
    HideOthers,
    /// Show All.
    ShowAll,
    /// Minimize (⌘M).
    Minimize,
    /// Zoom (the window's own).
    ZoomWindow,
}

/// A line of a menu.
#[derive(Clone, Debug)]
pub enum Entry {
    /// An item.
    Item(Item),
    /// A line between groups.
    Separator,
    /// A menu inside the menu.
    Sub(Menu),
    /// The recent files, one item each.
    Recent,
    /// One of the system's own items (the Mac only).
    Native(Native, &'static str),
}

/// A menu: its words (a Fluent id) and what it holds.
#[derive(Clone, Debug)]
pub struct Menu {
    /// The Fluent id of its title.
    pub label: &'static str,
    /// Its lines.
    pub entries: Vec<Entry>,
}

/// Which system the menus are for: the Mac has its own bar and its own keys for Redo and Help.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    /// macOS.
    Mac,
    /// Windows and Linux.
    Other,
}

impl Platform {
    /// The system this is built for.
    pub const fn current() -> Platform {
        if cfg!(target_os = "macos") {
            Platform::Mac
        } else {
            Platform::Other
        }
    }
}

/// An action item; its key is the one `keys::table` has for the command (so a menu never shows a key the window does
/// not answer, and never leaves out one it does).
fn item(cmd: Cmd, label: &'static str) -> Entry {
    Entry::Item(Item {
        cmd,
        label,
        shortcut: cmd.act().and_then(crate::keys::shortcut),
        kind: Kind::Action,
    })
}

fn switch(cmd: Cmd, label: &'static str) -> Entry {
    Entry::Item(Item {
        cmd,
        label,
        shortcut: None,
        kind: Kind::Check,
    })
}

fn choice(cmd: Cmd, label: &'static str) -> Entry {
    Entry::Item(Item {
        cmd,
        label,
        shortcut: None,
        kind: Kind::Radio,
    })
}

fn file_menu(p: Platform) -> Menu {
    let mut menu = Menu {
        label: "app-file",
        entries: vec![
            item(Cmd::New, "app-new"),
            item(Cmd::Open, "app-open"),
            Entry::Recent,
            Entry::Separator,
            item(Cmd::Save, "app-save"),
            item(Cmd::SaveAs, "app-save-as"),
            item(Cmd::SaveTemplate, "app-file-save-template"),
            Entry::Separator,
            item(Cmd::Export, "app-export-sheet"),
            Entry::Separator,
            item(Cmd::Close, "app-file-close"),
        ],
    };
    // Preferences… sits above Close; on the Mac it is in the Quadrule menu.
    if p == Platform::Other {
        let at = menu.entries.len() - 1;
        menu.entries.splice(
            at..at,
            [item(Cmd::Preferences, "menu-preferences"), Entry::Separator],
        );
    }
    menu
}

fn edit_menu(p: Platform) -> Menu {
    // Both Redo keys are in the table and work everywhere; the menu shows the one the system's own programs show.
    let redo = &crate::keys::row(crate::keys::Act::Redo).keys;
    let redo = match p {
        Platform::Mac => redo[0],
        Platform::Other => redo[1],
    };
    Menu {
        label: "menu-edit",
        entries: vec![
            item(Cmd::Undo, "app-undo"),
            Entry::Item(Item {
                cmd: Cmd::Redo,
                label: "app-redo",
                shortcut: Some(KeyboardShortcut::new(redo.0, redo.1)),
                kind: Kind::Action,
            }),
            Entry::Separator,
            item(Cmd::Cut, "menu-cut"),
            item(Cmd::Copy, "menu-copy"),
            item(Cmd::Paste, "menu-paste"),
            Entry::Separator,
            item(Cmd::Duplicate, "menu-duplicate"),
            item(Cmd::SelectAll, "menu-select-all"),
        ],
    }
}

fn view_menu(_p: Platform) -> Menu {
    Menu {
        label: "app-view",
        entries: vec![
            item(Cmd::ZoomIn, "app-zoom-in"),
            item(Cmd::ZoomOut, "app-zoom-out"),
            item(Cmd::ZoomFit, "app-zoom-fit"),
            item(Cmd::ZoomActual, "menu-zoom-actual"),
            item(Cmd::RealSizeCheck, "app-ruler-open"),
            Entry::Separator,
            switch(Cmd::Strip, "menu-strip"),
            Entry::Sub(Menu {
                label: "menu-snapping",
                entries: Snap::ALL
                    .into_iter()
                    .map(|s| switch(Cmd::Snap(s), s.label()))
                    .collect(),
            }),
            Entry::Separator,
            switch(Cmd::SafeZones, "app-safe-zones"),
            Entry::Sub(Menu {
                label: "app-canvas",
                entries: vec![
                    choice(Cmd::Canvas(CanvasMode::ExportPixels), "app-canvas-export"),
                    choice(Cmd::Canvas(CanvasMode::Chip), "app-canvas-chip"),
                ],
            }),
        ],
    }
}

fn help_menu(p: Platform) -> Menu {
    let mut entries = Vec::new();
    // On the Mac, About is in the Quadrule menu.
    if p == Platform::Other {
        entries.push(item(Cmd::About, "menu-about"));
    }
    entries.push(item(Cmd::ShortcutsSheet, "menu-shortcuts"));
    entries.push(item(Cmd::ProjectPage, "menu-project-page"));
    entries.push(Entry::Separator);
    entries.push(Entry::Sub(Menu {
        label: "menu-developer",
        entries: vec![item(Cmd::ExportJson, "menu-export-json")],
    }));
    Menu {
        label: "menu-help",
        entries,
    }
}

/// The menus for the system `p`: File, Edit, View and Help; the Mac's bar also has the Quadrule menu first and
/// Window before Help.
pub fn menus(p: Platform) -> Vec<Menu> {
    let mut out = Vec::new();
    if p == Platform::Mac {
        out.push(app_menu());
    }
    out.push(file_menu(p));
    out.push(edit_menu(p));
    out.push(view_menu(p));
    if p == Platform::Mac {
        out.push(window_menu());
    }
    out.push(help_menu(p));
    out
}

/// The menus the bar along the top of the window shows (Windows and Linux): everything but the Mac's own two.
pub fn bar_menus() -> Vec<Menu> {
    menus(Platform::Other)
}

/// The Quadrule menu on the Mac (Q7): About, Preferences…, Hide, Hide Others, Show All and Quit.
pub fn app_menu() -> Menu {
    Menu {
        label: "menu-app",
        entries: vec![
            item(Cmd::About, "menu-about-mac"),
            Entry::Separator,
            item(Cmd::Preferences, "menu-preferences"),
            Entry::Separator,
            Entry::Native(Native::Hide, "menu-hide"),
            Entry::Native(Native::HideOthers, "menu-hide-others"),
            Entry::Native(Native::ShowAll, "menu-show-all"),
            Entry::Separator,
            item(Cmd::Quit, "menu-quit"),
        ],
    }
}

/// The Window menu on the Mac: Minimize and Zoom, which the system does.
pub fn window_menu() -> Menu {
    Menu {
        label: "menu-window",
        entries: vec![
            Entry::Native(Native::Minimize, "menu-minimize"),
            Entry::Native(Native::ZoomWindow, "menu-zoom-window"),
        ],
    }
}

/// Every item in `menus`, in order, however deep.
pub fn items_of(menus: &[Menu]) -> Vec<Item> {
    fn walk(entries: &[Entry], out: &mut Vec<Item>) {
        for e in entries {
            match e {
                Entry::Item(i) => out.push(i.clone()),
                Entry::Sub(m) => walk(&m.entries, out),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for m in menus {
        walk(&m.entries, &mut out);
    }
    out
}

/// Whether an item is greyed, and whether it is ticked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CmdState {
    /// The item can be picked.
    pub enabled: bool,
    /// `Some` for a switch or a choice: whether it is on.
    pub checked: Option<bool>,
}

impl ViewerUi {
    /// Whether the command `cmd` can act now, and for a switch or a choice whether it is on.
    pub fn cmd_state(&self, cmd: Cmd) -> CmdState {
        let doc = self.in_document();
        let has_doc = self.viewer.open.is_some();
        let typing = self.viewer.is_typing();
        let history = self.viewer.open.as_ref().map(|o| &o.history);
        let selected = !self.viewer.selection.is_empty();
        let on = |b: bool| CmdState {
            enabled: true,
            checked: Some(b),
        };
        let plain = |enabled: bool| CmdState {
            enabled,
            checked: None,
        };
        match cmd {
            Cmd::New
            | Cmd::Open
            | Cmd::About
            | Cmd::ShortcutsSheet
            | Cmd::ProjectPage
            | Cmd::Preferences
            | Cmd::Quit => plain(true),
            Cmd::OpenRecent(i) => {
                plain(self.viewer.recent_files().get(i).is_some_and(|r| r.exists))
            }
            Cmd::Save | Cmd::SaveAs | Cmd::ExportJson => plain(doc),
            Cmd::SaveTemplate => plain(doc && self.viewer.templates_dir.is_some()),
            Cmd::Export => plain(doc && !typing && !self.export_sheets_open()),
            // Typing on the page takes the keyboard: its keys do not close the page or open the sheet, nor do these.
            Cmd::Close => plain(has_doc && !typing),
            Cmd::Undo => plain(doc && (typing || history.is_some_and(|h| h.can_undo()))),
            Cmd::Redo => plain(doc && (typing || history.is_some_and(|h| h.can_redo()))),
            Cmd::Cut => plain(doc && (typing || !self.viewer.editable_selection().is_empty())),
            Cmd::Copy => plain(doc && (typing || selected)),
            Cmd::Paste => plain(doc),
            Cmd::Duplicate => plain(doc && !typing && self.can_duplicate()),
            Cmd::SelectAll => plain(doc),
            Cmd::ZoomIn | Cmd::ZoomOut | Cmd::ZoomFit | Cmd::ZoomActual | Cmd::RealSizeCheck => {
                plain(doc)
            }
            Cmd::Strip => CmdState {
                enabled: doc,
                checked: Some(self.viewer.show_strip),
            },
            Cmd::Snap(s) => on(match s {
                Snap::Pixel => self.viewer.snap.pixel_grid,
                Snap::Rhythm => self.viewer.snap.rhythm,
                Snap::Margins => self.viewer.snap.margins_safe,
                Snap::Points => self.viewer.snap.points,
                Snap::Smart => self.viewer.snap.smart_guides,
                Snap::Equal => self.viewer.snap.equal_spacing,
                Snap::Overlay => self.viewer.show_rhythm,
            }),
            Cmd::SafeZones => on(self.viewer.show_safe_zones),
            Cmd::Canvas(m) => on(self.canvas_mode == m),
        }
    }

    /// The words for a Fluent id (the Mac's menus are built outside this crate and ask for theirs here).
    pub fn text(&self, id: &str) -> String {
        self.strings.get(id)
    }

    /// What Open Recent lists: each line's command, words and whether it can be picked; with no recent files, one
    /// greyed line that says so.
    pub fn recent_lines(&self) -> Vec<(Option<Cmd>, String, bool)> {
        let recent = self.viewer.recent_files();
        if recent.is_empty() {
            return vec![(None, self.t("app-file-recent-none"), false)];
        }
        recent
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                let words = if r.exists {
                    r.name.clone()
                } else {
                    self.strings.format(
                        &quadrule_i18n::Message::new("app-file-recent-gone")
                            .text("name", r.name.clone()),
                    )
                };
                (Some(Cmd::OpenRecent(i)), words, r.exists)
            })
            .collect()
    }

    /// Whether Duplicate has an object it may copy (one in a layer that is not locked).
    fn can_duplicate(&self) -> bool {
        self.viewer.doc().is_some_and(|d| {
            self.viewer
                .selection
                .iter()
                .any(|id| d.layer_of(*id).is_some_and(|l| !d.layers[l].locked))
        })
    }

    /// Queues the key event `key` with `mods` for the start of the next frame, as the keyboard would make it.
    fn queue_key(&mut self, ctx: &egui::Context, mods: Modifiers, key: Key) {
        for pressed in [true, false] {
            self.queued_events.push(Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: mods,
            });
        }
        ctx.request_repaint();
    }

    /// Queues an event for the start of the next frame.
    fn queue_event(&mut self, ctx: &egui::Context, event: Event) {
        self.queued_events.push(event);
        ctx.request_repaint();
    }

    /// Hands the queued events to egui's input, where the key or the system would have put them.
    pub(crate) fn inject_queued(&mut self, ctx: &egui::Context) {
        if self.queued_events.is_empty() {
            return;
        }
        let events = std::mem::take(&mut self.queued_events);
        ctx.input_mut(|i| i.events.extend(events));
    }

    /// Whether text, in a field or on the page, has the keyboard (so Edit commands go to it, as its keys would).
    fn text_has_keys(&self, ctx: &egui::Context) -> bool {
        self.viewer.is_typing() || ctx.egui_wants_keyboard_input()
    }

    /// Gives the next frame what the key of this Edit command would have made, where a text field or typing on the
    /// page has the keyboard (and always for Paste, which the window answers with the clipboard's text).
    fn edit_by_keys(&mut self, ctx: &egui::Context, cmd: Cmd) {
        let cmd_mod = Modifiers::COMMAND;
        match cmd {
            Cmd::Undo => self.queue_key(ctx, cmd_mod, Key::Z),
            Cmd::Redo => self.queue_key(ctx, cmd_mod | Modifiers::SHIFT, Key::Z),
            Cmd::Cut => self.queue_event(ctx, Event::Cut),
            Cmd::Copy => self.queue_event(ctx, Event::Copy),
            Cmd::SelectAll => self.queue_key(ctx, cmd_mod, Key::A),
            // The window reads the clipboard and gives the next frame the paste event, as Ctrl+V would.
            Cmd::Paste => ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste),
            _ => {}
        }
    }

    /// Does the menu item `cmd`: what its key does, or what its old button did. Returns what the app must do.
    pub fn run_cmd(&mut self, ctx: &egui::Context, cmd: Cmd) -> Vec<Request> {
        let mut requests = Vec::new();
        let help = matches!(cmd, Cmd::About | Cmd::ShortcutsSheet | Cmd::ProjectPage);
        let edit = matches!(
            cmd,
            Cmd::Undo | Cmd::Redo | Cmd::Cut | Cmd::Copy | Cmd::Paste | Cmd::SelectAll
        );
        // A question, a sheet or a drag in the middle holds every command but Help, as it holds the keys. The Edit
        // commands still go to a text field there (the New dialog's name, say): the Mac's menu took their keys.
        let held = self.question_open()
            || self.welcome_question_open()
            || self.delete_layer_ask.is_some()
            || self.viewer.dragging();
        if held && !help {
            if edit && self.cmd_state(cmd).enabled {
                self.edit_by_keys(ctx, cmd);
            }
            return requests;
        }
        if !self.cmd_state(cmd).enabled {
            return requests;
        }
        match cmd {
            Cmd::New => self.ask_new(),
            Cmd::Open => requests.extend(self.guard(After::OpenDialog)),
            Cmd::OpenRecent(i) => {
                if let Some(r) = self.viewer.recent_files().get(i) {
                    requests.extend(self.guard(After::OpenPath(r.path.clone())));
                }
            }
            Cmd::Save => {
                self.viewer.end_typing();
                requests.extend(self.save());
            }
            Cmd::SaveAs => {
                self.viewer.end_typing();
                self.ask(Request::SaveAsDialog(None), &mut requests);
            }
            Cmd::SaveTemplate => self.save_as_template(),
            Cmd::ExportJson => {
                self.viewer.end_typing();
                let name = format!(
                    "{}.json",
                    crate::viewer::slug(self.viewer.title().unwrap_or("untitled"))
                );
                requests.push(Request::ExportJsonDialog(name));
            }
            Cmd::Export => self.open_export_sheet(),
            Cmd::Close => requests.extend(self.ask_close_document()),
            Cmd::Undo | Cmd::Redo | Cmd::Cut | Cmd::Copy | Cmd::Paste | Cmd::SelectAll => {
                if self.text_has_keys(ctx) || cmd == Cmd::Paste {
                    self.edit_by_keys(ctx, cmd);
                } else {
                    match cmd {
                        Cmd::Undo => {
                            self.viewer.undo();
                        }
                        Cmd::Redo => {
                            self.viewer.redo();
                        }
                        Cmd::Cut => {
                            if let Ok(Some(text)) = self.viewer.cut_text() {
                                ctx.copy_text(text);
                            }
                        }
                        Cmd::Copy => {
                            if let Some(text) = self.viewer.copy_text() {
                                ctx.copy_text(text);
                            }
                        }
                        _ => self.viewer.select_all(),
                    }
                }
            }
            Cmd::Duplicate => {
                if !self.text_has_keys(ctx) {
                    let _ = self.viewer.duplicate_selection();
                }
            }
            Cmd::ZoomIn => {
                let now = self.shown_scale_ctx(ctx);
                self.viewer.zoom_step(now, 1);
            }
            Cmd::ZoomOut => {
                let now = self.shown_scale_ctx(ctx);
                self.viewer.zoom_step(now, -1);
            }
            Cmd::ZoomFit => self.viewer.zoom = crate::Zoom::Fit,
            Cmd::ZoomActual => self.viewer.zoom = crate::Zoom::Scale(1.0),
            Cmd::RealSizeCheck => self.open_ruler(ctx),
            Cmd::Strip => self.viewer.show_strip = !self.viewer.show_strip,
            Cmd::Snap(s) => {
                let flag = match s {
                    Snap::Pixel => &mut self.viewer.snap.pixel_grid,
                    Snap::Rhythm => &mut self.viewer.snap.rhythm,
                    Snap::Margins => &mut self.viewer.snap.margins_safe,
                    Snap::Points => &mut self.viewer.snap.points,
                    Snap::Smart => &mut self.viewer.snap.smart_guides,
                    Snap::Equal => &mut self.viewer.snap.equal_spacing,
                    Snap::Overlay => &mut self.viewer.show_rhythm,
                };
                *flag = !*flag;
            }
            Cmd::SafeZones => self.viewer.show_safe_zones = !self.viewer.show_safe_zones,
            Cmd::Canvas(m) => {
                self.canvas_mode = m;
                self.drop_vector();
            }
            Cmd::About => self.about_open = true,
            Cmd::ShortcutsSheet => self.shortcuts_open = !self.shortcuts_open,
            Cmd::ProjectPage => ctx.open_url(egui::OpenUrl::new_tab(PROJECT_URL)),
            Cmd::Preferences => self.open_preferences(),
            Cmd::Quit => requests.extend(self.guard(After::Close)),
        }
        requests
    }
}
