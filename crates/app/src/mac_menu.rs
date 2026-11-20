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

//! The Mac's menu bar (fixes batch 2, docs/bbq/2026-11-18-fixes-before-16b.md, Q7): the menus go to the top of the
//! screen, with ⌘ keys, and the window draws no menu bar of its own. The menus are the table in `quadrule_ui::menus`
//! (the one the Windows and Linux bar is drawn from), turned into the system's menus with `muda`.
//!
//! Two things to know when reading this:
//!
//! - **Keys.** A system menu item with a key takes that key before the window sees it, so each key does its action
//!   once: through the item. [`ViewerUi::run_cmd`](quadrule_ui::ViewerUi::run_cmd) hands the Edit keys that text wants
//!   (a field, typing on the page) back to egui as the events the keys would have made. An item that is greyed does not
//!   take its key, so the key goes on to the window as it always did (a text field's own undo, say).
//! - **Quit.** Quit Quadrule is an item of our own that goes through the unsaved-changes prompt, as closing the
//!   window does; the system's own Quit would end the program with the work unsaved.

use eframe::egui;
use muda::accelerator::{Key, KeyAccelerator, Modifiers};
use muda::{CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use quadrule_ui::menus::{Entry, Item, Kind, Menu as Table, Native, menus};
use quadrule_ui::{Cmd, Platform, ViewerUi};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How often the Open Recent list is read again (it looks at the disk for files that are gone).
const RECENT_EVERY: Duration = Duration::from_secs(2);

/// One item the app keeps up to date.
enum Handle {
    Plain(MenuItem),
    Check(CheckMenuItem),
}

/// The Mac's menu bar, installed.
pub struct MacMenu {
    /// Kept alive: the system's menu bar is this.
    _menu: Option<Menu>,
    items: Vec<(Cmd, Handle)>,
    recent: Option<Submenu>,
    recent_shown: Vec<(Option<Cmd>, String, bool)>,
    recent_read: Instant,
    /// The items the person picked since the last frame (the system calls us on the main thread).
    picked: Arc<Mutex<Vec<Cmd>>>,
}

/// The key of a table item as the system's menu wants it: ⌘ for the table's Command.
fn accelerator(item: &Item) -> Option<KeyAccelerator> {
    let sc = item.shortcut?;
    let mut mods = Modifiers::empty();
    if sc.modifiers.command || sc.modifiers.mac_cmd {
        mods |= Modifiers::META;
    }
    if sc.modifiers.shift {
        mods |= Modifiers::SHIFT;
    }
    if sc.modifiers.alt {
        mods |= Modifiers::ALT;
    }
    if sc.modifiers.ctrl {
        mods |= Modifiers::CONTROL;
    }
    let key = match sc.logical_key {
        egui::Key::Comma => ",".to_string(),
        k => {
            let name = k.name();
            if name.chars().count() == 1 {
                name.to_lowercase()
            } else {
                return None;
            }
        }
    };
    Some(KeyAccelerator::new(mods, Key::Character(key)))
}

impl MacMenu {
    /// Makes the menus and puts them in the system's menu bar, replacing the one winit made. Call it once the app is
    /// running (the first frame), when the system's application object exists.
    pub fn install(ctx: &egui::Context, ui: &ViewerUi) -> Self {
        let picked: Arc<Mutex<Vec<Cmd>>> = Arc::default();
        let sink = Arc::clone(&picked);
        let wake = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(cmd) = Cmd::from_id(&event.id.0)
                && let Ok(mut v) = sink.lock()
            {
                v.push(cmd);
                wake.request_repaint();
            }
        }));
        let menu = Menu::new();
        let mut me = MacMenu {
            _menu: None,
            items: Vec::new(),
            recent: None,
            recent_shown: Vec::new(),
            recent_read: Instant::now(),
            picked,
        };
        for table in menus(Platform::Mac) {
            let sub = me.submenu(ui, &table);
            let _ = menu.append(&sub);
            match table.label {
                "menu-window" => sub.set_as_windows_menu_for_nsapp(),
                "menu-help" => sub.set_as_help_menu_for_nsapp(),
                _ => {}
            }
        }
        menu.init_for_nsapp();
        me._menu = Some(menu);
        me.sync(ui);
        me
    }

    /// A menu of the table as a system menu.
    fn submenu(&mut self, ui: &ViewerUi, table: &Table) -> Submenu {
        let sub = Submenu::with_id(table.label, ui.text(table.label), true);
        for entry in &table.entries {
            match entry {
                Entry::Item(item) => {
                    let state = ui.cmd_state(item.cmd);
                    let id = item.cmd.id();
                    let words = ui.text(item.label);
                    let handle = match item.kind {
                        Kind::Action => {
                            let it = MenuItem::with_id(id, words, state.enabled, None);
                            let _ = it.set_key_accelerator(accelerator(item));
                            let _ = sub.append(&it);
                            Handle::Plain(it)
                        }
                        Kind::Check | Kind::Radio => {
                            let it = CheckMenuItem::with_id(
                                id,
                                words,
                                state.enabled,
                                state.checked.unwrap_or(false),
                                None,
                            );
                            let _ = it.set_key_accelerator(accelerator(item));
                            let _ = sub.append(&it);
                            Handle::Check(it)
                        }
                    };
                    self.items.push((item.cmd, handle));
                }
                Entry::Separator => {
                    let _ = sub.append(&PredefinedMenuItem::separator());
                }
                Entry::Sub(inner) => {
                    let s = self.submenu(ui, inner);
                    let _ = sub.append(&s);
                }
                Entry::Recent => {
                    let recent =
                        Submenu::with_id("open-recent", ui.text("app-file-open-recent"), true);
                    let _ = sub.append(&recent);
                    self.recent = Some(recent);
                }
                Entry::Native(which, label) => {
                    let words = Some(ui.text(label));
                    let words = words.as_deref();
                    let item = match which {
                        Native::Hide => PredefinedMenuItem::hide(words),
                        Native::HideOthers => PredefinedMenuItem::hide_others(words),
                        Native::ShowAll => PredefinedMenuItem::show_all(words),
                        Native::Minimize => PredefinedMenuItem::minimize(words),
                        Native::ZoomWindow => PredefinedMenuItem::maximize(words),
                    };
                    let _ = sub.append(&item);
                }
            }
        }
        sub
    }

    /// The commands picked since the last call.
    pub fn take_picked(&self) -> Vec<Cmd> {
        self.picked
            .lock()
            .map(|mut v| std::mem::take(&mut *v))
            .unwrap_or_default()
    }

    /// Brings each item's greyed and ticked state in step with the window, and the Open Recent list. Compared with the
    /// items' own state (not a copy of it), since the system ticks a switch itself when it is picked.
    pub fn sync(&mut self, ui: &ViewerUi) {
        for (cmd, handle) in &self.items {
            let state = ui.cmd_state(*cmd);
            match handle {
                Handle::Plain(it) => {
                    if it.is_enabled() != state.enabled {
                        it.set_enabled(state.enabled);
                    }
                }
                Handle::Check(it) => {
                    if it.is_enabled() != state.enabled {
                        it.set_enabled(state.enabled);
                    }
                    let on = state.checked.unwrap_or(false);
                    if it.is_checked() != on {
                        it.set_checked(on);
                    }
                }
            }
        }
        if self.recent_shown.is_empty() || self.recent_read.elapsed() >= RECENT_EVERY {
            self.recent_read = Instant::now();
            let lines = ui.recent_lines();
            if lines != self.recent_shown
                && let Some(recent) = &self.recent
            {
                while recent.remove_at(0).is_some() {}
                for (cmd, words, enabled) in &lines {
                    let id = cmd.map(|c| c.id()).unwrap_or_else(|| "no-recent".into());
                    let it = MenuItem::with_id(id, words, *enabled, None);
                    let _ = recent.append(&it as &dyn IsMenuItem);
                }
                self.recent_shown = lines;
            }
        }
    }
}
