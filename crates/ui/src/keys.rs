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

//! Every keyboard shortcut, in one table (rung 17 bbq, Q4 and Defaulted 5). The window reads its keys here
//! ([`consume`]), the menus show them from here ([`shortcut`]), and the Keyboard shortcuts sheet (Help, Ctrl+/) lists
//! them from here, so the three cannot drift: a key the window answers is a row of [`table`], and a test fails on
//! any other place that reads a key by itself (`crates/ui/tests/keys.rs`).
//!
//! Ctrl is Cmd on the Mac and Alt is Option, as egui maps them. Keys match exactly: Alt+V is not the V tool, and
//! Shift+1 is found by the key's place on the keyboard as well as by its letter (Shift+1 makes `!` on some layouts, and
//! Option+A makes `å` on the Mac).

use crate::arrange::ArrangeOp;
use crate::tools::Tool;
use egui::{Event, Key, KeyboardShortcut, Modifiers};
use quadrule_layout::align::{Edge, Spread};
use quadrule_layout::units::Axis;

/// What a key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// File > New…
    New,
    /// File > Open…
    Open,
    /// File > Save.
    Save,
    /// File > Save As….
    SaveAs,
    /// File > Export….
    Export,
    /// File > Close.
    Close,
    /// File > Preferences… (the Mac: Quadrule > Preferences…).
    Preferences,
    /// Quadrule > Quit (the Mac only; the system takes the key).
    Quit,
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
    /// Delete what is picked.
    Delete,
    /// Group what is picked.
    Group,
    /// Ungroup it.
    Ungroup,
    /// Move the picked objects up (toward the front).
    Front,
    /// Move the picked objects down (toward the back).
    Back,
    /// View > Zoom in.
    ZoomIn,
    /// View > Zoom out.
    ZoomOut,
    /// View > Fit.
    ZoomFit,
    /// View > Actual size.
    ZoomActual,
    /// An align or distribute button of the Arrange tab.
    Arrange(ArrangeOp),
    /// Pick a tool.
    Tool(Tool),
    /// The arrow keys nudge the picked objects one device pixel.
    Nudge,
    /// Shift and the arrow keys nudge them one rhythm unit.
    NudgeRhythm,
    /// Arrow keys and Enter in the font and emoji lists.
    Picker,
    /// Escape: drop a drag, end a tool, step out of a group, then clear the selection.
    Escape,
    /// Help > Keyboard shortcuts.
    ShortcutsSheet,
}

/// The part of the sheet a row is listed under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// New, Open, Save, Export, Close, Preferences.
    File,
    /// Undo, copy and paste, select all, delete, group, order.
    Edit,
    /// Zoom.
    View,
    /// Align and distribute, nudging.
    Arrange,
    /// The tools bar's letters.
    Tools,
    /// Escape and the sheet itself.
    Other,
}

impl Group {
    /// The groups in the order the sheet lists them.
    pub const ALL: [Group; 6] = [
        Group::File,
        Group::Edit,
        Group::View,
        Group::Arrange,
        Group::Tools,
        Group::Other,
    ];

    /// The Fluent id of the group's heading.
    pub fn message_id(self) -> &'static str {
        match self {
            Group::File => "keys-group-file",
            Group::Edit => "keys-group-edit",
            Group::View => "keys-group-view",
            Group::Arrange => "keys-group-arrange",
            Group::Tools => "keys-group-tools",
            Group::Other => "keys-group-other",
        }
    }
}

/// One row: an action, its keys and its words.
#[derive(Clone, Debug)]
pub struct Row {
    /// What it does.
    pub act: Act,
    /// The keys that do it. The first `shown` are listed in the sheet and menus; the rest are other ways to the same.
    pub keys: Vec<(Modifiers, Key)>,
    /// How many of `keys` the sheet lists.
    pub shown: usize,
    /// The Fluent id of its words.
    pub words: &'static str,
    /// Where the sheet lists it.
    pub group: Group,
    /// Only on the Mac (the sheet leaves it out elsewhere).
    pub mac_only: bool,
}

/// Every row, in the order the sheet lists them.
pub fn table() -> Vec<Row> {
    let c = Modifiers::COMMAND;
    let cs = Modifiers::COMMAND | Modifiers::SHIFT;
    let alt = Modifiers::ALT;
    let alt_shift = Modifiers::ALT | Modifiers::SHIFT;
    let n = Modifiers::NONE;
    let mut rows = Vec::new();
    let mut add = |act, keys: Vec<(Modifiers, Key)>, shown, words, group| {
        rows.push(Row {
            act,
            keys,
            shown,
            words,
            group,
            mac_only: false,
        });
    };
    add(Act::New, vec![(c, Key::N)], 1, "app-new", Group::File);
    add(Act::Open, vec![(c, Key::O)], 1, "app-open", Group::File);
    add(Act::Save, vec![(c, Key::S)], 1, "app-save", Group::File);
    add(
        Act::SaveAs,
        vec![(cs, Key::S)],
        1,
        "app-save-as",
        Group::File,
    );
    add(
        Act::Export,
        vec![(c, Key::E)],
        1,
        "app-export-sheet",
        Group::File,
    );
    add(
        Act::Close,
        vec![(c, Key::W)],
        1,
        "app-file-close",
        Group::File,
    );
    add(
        Act::Preferences,
        vec![(c, Key::Comma)],
        1,
        "menu-preferences",
        Group::File,
    );
    add(Act::Undo, vec![(c, Key::Z)], 1, "app-undo", Group::Edit);
    // The menu shows the Redo key the system's own programs show (`menus`); both work everywhere.
    add(
        Act::Redo,
        vec![(cs, Key::Z), (c, Key::Y)],
        2,
        "app-redo",
        Group::Edit,
    );
    add(Act::Cut, vec![(c, Key::X)], 1, "menu-cut", Group::Edit);
    add(Act::Copy, vec![(c, Key::C)], 1, "menu-copy", Group::Edit);
    add(Act::Paste, vec![(c, Key::V)], 1, "menu-paste", Group::Edit);
    add(
        Act::Duplicate,
        vec![(c, Key::D)],
        1,
        "menu-duplicate",
        Group::Edit,
    );
    add(
        Act::SelectAll,
        vec![(c, Key::A)],
        1,
        "menu-select-all",
        Group::Edit,
    );
    add(
        Act::Delete,
        vec![(n, Key::Delete), (n, Key::Backspace)],
        2,
        "keys-delete",
        Group::Edit,
    );
    add(Act::Group, vec![(c, Key::G)], 1, "app-group", Group::Edit);
    add(
        Act::Ungroup,
        vec![(cs, Key::G)],
        1,
        "app-ungroup",
        Group::Edit,
    );
    add(
        Act::Front,
        vec![(c, Key::CloseBracket)],
        1,
        "keys-front",
        Group::Edit,
    );
    add(
        Act::Back,
        vec![(c, Key::OpenBracket)],
        1,
        "keys-back",
        Group::Edit,
    );
    // Ctrl+= is Ctrl and the plus sign's key; a keyboard that makes + with Shift is read too.
    add(
        Act::ZoomIn,
        vec![(c, Key::Equals), (cs, Key::Plus), (c, Key::Plus)],
        1,
        "app-zoom-in",
        Group::View,
    );
    add(
        Act::ZoomOut,
        vec![(c, Key::Minus)],
        1,
        "app-zoom-out",
        Group::View,
    );
    add(
        Act::ZoomFit,
        vec![(Modifiers::SHIFT, Key::Num1)],
        1,
        "app-zoom-fit",
        Group::View,
    );
    add(
        Act::ZoomActual,
        vec![(Modifiers::SHIFT, Key::Num0)],
        1,
        "menu-zoom-actual",
        Group::View,
    );
    // Align: A and D the left and right edges, W and S the top and bottom, H and V the horizontal and vertical centres.
    for (axis, edge, key) in [
        (Axis::X, Edge::Start, Key::A),
        (Axis::X, Edge::End, Key::D),
        (Axis::Y, Edge::Start, Key::W),
        (Axis::Y, Edge::End, Key::S),
        (Axis::X, Edge::Centre, Key::H),
        (Axis::Y, Edge::Centre, Key::V),
    ] {
        let op = ArrangeOp::Align(axis, edge);
        add(
            Act::Arrange(op),
            vec![(alt, key)],
            1,
            op.message_id(),
            Group::Arrange,
        );
    }
    add(
        Act::Arrange(ArrangeOp::Distribute(Axis::X, Spread::Spacing)),
        vec![(alt_shift, Key::H)],
        1,
        "app-distribute-h-spacing",
        Group::Arrange,
    );
    add(
        Act::Arrange(ArrangeOp::Distribute(Axis::Y, Spread::Spacing)),
        vec![(alt_shift, Key::V)],
        1,
        "app-distribute-v-spacing",
        Group::Arrange,
    );
    let arrows = |m| {
        vec![
            (m, Key::ArrowLeft),
            (m, Key::ArrowRight),
            (m, Key::ArrowUp),
            (m, Key::ArrowDown),
        ]
    };
    add(Act::Nudge, arrows(n), 4, "keys-nudge", Group::Arrange);
    add(
        Act::NudgeRhythm,
        arrows(Modifiers::SHIFT),
        4,
        "keys-nudge-rhythm",
        Group::Arrange,
    );
    for tool in Tool::ALL {
        add(
            Act::Tool(tool),
            vec![(n, tool.key())],
            1,
            tool.message_id(),
            Group::Tools,
        );
    }
    add(
        Act::Escape,
        vec![(n, Key::Escape)],
        1,
        "keys-escape",
        Group::Other,
    );
    add(
        Act::Picker,
        vec![
            (n, Key::ArrowDown),
            (n, Key::ArrowUp),
            (n, Key::ArrowLeft),
            (n, Key::ArrowRight),
            (n, Key::Enter),
        ],
        5,
        "keys-picker",
        Group::Other,
    );
    add(
        Act::ShortcutsSheet,
        vec![(c, Key::Slash)],
        1,
        "menu-shortcuts",
        Group::Other,
    );
    rows.push(Row {
        act: Act::Quit,
        keys: vec![(c, Key::Q)],
        shown: 1,
        words: "menu-quit",
        group: Group::File,
        mac_only: true,
    });
    rows
}

/// The row for `act`.
pub fn row(act: Act) -> Row {
    table()
        .into_iter()
        .find(|r| r.act == act)
        .unwrap_or_else(|| panic!("{act:?} has no row in keys::table"))
}

/// The key a menu item for `act` shows (the first of its row), or `None` for a row with no key.
pub fn shortcut(act: Act) -> Option<KeyboardShortcut> {
    row(act)
        .keys
        .first()
        .map(|(m, k)| KeyboardShortcut::new(*m, *k))
}

/// Whether the modifiers on an event are the ones a row wants, exactly (Ctrl on Windows and Linux is Cmd; on the Mac
/// the Control key alone is not).
fn same_modifiers(have: &Modifiers, want: &Modifiers) -> bool {
    have.alt == want.alt
        && have.shift == want.shift
        && have.command == want.command
        && (want.command || have.ctrl == want.ctrl)
}

/// Whether `key` was pressed with exactly `mods` this frame, and how many times (key repeats count); the presses are
/// taken, so no other reader sees them. A key pressed with Shift or Alt is also found by its place on the keyboard.
pub fn count_and_consume(ctx: &egui::Context, mods: Modifiers, key: Key) -> usize {
    ctx.input_mut(|i| {
        let mut n = 0;
        i.events.retain(|e| match e {
            Event::Key {
                key: k,
                physical_key,
                pressed: true,
                modifiers,
                ..
            } if same_modifiers(modifiers, &mods)
                && (*k == key
                    || ((modifiers.shift || modifiers.alt) && *physical_key == Some(key))) =>
            {
                n += 1;
                false
            }
            _ => true,
        });
        n
    })
}

/// Whether a key of `act`'s row was pressed this frame (and takes it). Ask for the most specific act first.
pub fn consume(ctx: &egui::Context, act: Act) -> bool {
    row(act)
        .keys
        .iter()
        .any(|(m, k)| count_and_consume(ctx, *m, *k) > 0)
}

/// The font and emoji lists' keys (the `Picker` row): how many times Down, Up and Enter were pressed this frame.
pub fn picker_keys(ctx: &egui::Context) -> (usize, usize, usize) {
    let row = row(Act::Picker);
    let n = |i: usize| count_and_consume(ctx, row.keys[i].0, row.keys[i].1);
    (n(0), n(1), n(4))
}

/// The emoji grid's sideways keys (the `Picker` row): Left and Right presses this frame.
pub fn picker_sideways(ctx: &egui::Context) -> (usize, usize) {
    let row = row(Act::Picker);
    let n = |i: usize| count_and_consume(ctx, row.keys[i].0, row.keys[i].1);
    (n(2), n(3))
}

/// A key as this system writes it, with punctuation as the mark and not its name (`Ctrl+,`, not `Ctrl+Comma`): egui
/// writes the modifiers and names the key; the key's own mark is put in its place.
pub fn key_text(ctx: &egui::Context, mods: Modifiers, key: Key) -> String {
    let full = ctx.format_shortcut(&KeyboardShortcut::new(mods, Key::A));
    let modifiers = full.strip_suffix('A').unwrap_or(&full);
    let name = match key {
        Key::Minus => "-",
        Key::ArrowDown | Key::ArrowUp | Key::ArrowLeft | Key::ArrowRight => key.name(),
        _ => key.symbol_or_name(),
    };
    format!("{modifiers}{name}")
}

/// The Mac's system menu takes its own keys; this says which acts the system, not the window, reads there.
pub fn on_this_system(row: &Row) -> bool {
    !row.mac_only || cfg!(target_os = "macos")
}

impl crate::view::ViewerUi {
    /// Whether the Keyboard shortcuts sheet shows.
    pub fn shortcuts_showing(&self) -> bool {
        self.shortcuts_open
    }

    /// The keys of a row as the sheet words them: each as this system writes it (Ctrl+Shift+Z, or ⇧⌘Z on the Mac).
    pub(crate) fn key_words(&self, ctx: &egui::Context, row: &Row) -> String {
        match row.act {
            Act::Nudge => self.t("keys-arrows"),
            Act::NudgeRhythm => self.t("keys-arrows-shift"),
            Act::Picker => self.t("keys-picker-keys"),
            _ => row
                .keys
                .iter()
                .take(row.shown)
                .map(|(m, k)| key_text(ctx, *m, *k))
                .collect::<Vec<_>>()
                .join(&self.t("keys-or")),
        }
    }

    /// The Keyboard shortcuts sheet (Help, Ctrl+/): every row of [`table`], by group.
    pub(crate) fn shortcuts_window(&mut self, ctx: &egui::Context) {
        if !self.shortcuts_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        let rows: Vec<Row> = table().into_iter().filter(on_this_system).collect();
        egui::Window::new(self.t("keys-title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height((ctx.content_rect().height() - 140.0).clamp(160.0, 520.0))
                    .show(ui, |ui| {
                        // One grid for every group, so the keys line up all the way down.
                        egui::Grid::new("keys-grid")
                            .num_columns(2)
                            .spacing([28.0, 4.0])
                            .show(ui, |ui| {
                                for group in Group::ALL {
                                    ui.label("");
                                    ui.label("");
                                    ui.end_row();
                                    ui.strong(self.t(group.message_id()));
                                    ui.label("");
                                    ui.end_row();
                                    for row in rows.iter().filter(|r| r.group == group) {
                                        ui.label(self.t(row.words));
                                        ui.label(
                                            egui::RichText::new(self.key_words(ctx, row))
                                                .monospace(),
                                        );
                                        ui.end_row();
                                    }
                                }
                            });
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(self.t("keys-note")).small().weak());
                    });
                ui.add_space(6.0);
                if ui.button(self.t("about-close")).clicked() {
                    close = true;
                }
            });
        if !open || close {
            self.shortcuts_open = false;
        }
    }
}
