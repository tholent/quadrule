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

//! The Profiles window's state and what its buttons do (rung 13a bbq, Q2, Q3, Q5, Q7, Q8, Q9 and the
//! Defaulted rules for editing, limits and deleting). No widgets here, so all of it is tested headless;
//! the widgets are in [`crate::profiles_win_ui`].
//!
//! Editing a profile does not touch its file: a draft is shown in place of the saved values (the open
//! page redraws as a value changes) until Save writes it or Cancel throws it away. Reset to built-in and
//! Delete (each asked first) and New…, Duplicate and an import write at once.

use crate::view::ViewerUi;
use crate::viewer::Notice;
use quadrule_i18n::Message;
use quadrule_profiles::{
    Damaged, Incoming, Profile, Source, active, limits, parse_checked, to_ron,
};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// What a button asked about and waits for a yes on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confirm {
    /// Reset this built-in to the shipped values (deletes its override file).
    Reset(String),
    /// Delete this profile of the user's.
    Delete(String),
    /// Delete this damaged file (its name) from the profiles folder (rung 14a, paying the 13a debt).
    DeleteDamaged(String),
}

/// The New… dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewForm {
    /// The new profile's name.
    pub name: String,
    /// The profile whose values it starts from.
    pub from: String,
}

/// What an import with a clash will do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportPick {
    /// The import's values take the place of mine (same id).
    Replace,
    /// Both stay: the import gets a new GUID id (same id).
    KeepBoth,
    /// The import's values update mine (same name).
    Merge,
    /// The import joins mine under this name (same name).
    Rename(String),
}

/// An import waiting for an answer.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportAsk {
    /// The profile in the file.
    pub incoming: Profile,
    /// How it clashes.
    pub kind: Incoming,
    /// The name of the profile here it clashes with.
    pub mine: String,
    /// The name Rename starts from.
    pub new_name: String,
}

/// One row of the list on the left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListEntry {
    /// The profile's id.
    pub id: String,
    /// Its name, as drafted if it is being edited.
    pub name: String,
    /// A built-in (the Built in group) or one of the user's (Mine).
    pub built_in: bool,
    /// A built-in with a local override, or one being edited now: the **Edited** mark.
    pub edited: bool,
    /// Not verified on a device.
    pub unverified: bool,
    /// Changed in this window and not saved yet.
    pub unsaved: bool,
    /// A paper size, not a screen (rung 13b).
    pub paper: bool,
    /// For a paper size, its size in mm and px at 300 PPI, as the row's tip says it.
    pub detail: Option<String>,
}

/// The Profiles window.
#[derive(Clone, Debug, Default)]
pub struct ProfilesWindow {
    /// The profile shown in the form.
    pub selected: String,
    /// Edited copies, by id, not saved yet.
    pub(crate) drafts: BTreeMap<String, Profile>,
    /// A question waiting for a yes.
    pub confirm: Option<Confirm>,
    /// The New… dialog, while it shows.
    pub new_form: Option<NewForm>,
    /// An import waiting for an answer.
    pub import: Option<ImportAsk>,
    /// The last thing a button did, or why it could not.
    pub message: Option<Message>,
    /// Text being typed in a field that parses to a list (gray levels and pen grays).
    pub(crate) texts: HashMap<String, String>,
}

impl ViewerUi {
    /// Opens the Profiles window on the device on screen (or the first built-in).
    pub fn open_profiles_window(&mut self) {
        let shown = self.viewer.profile_id.clone();
        let known = active::with(|r| r.stored(&shown).is_some());
        let selected = if known {
            shown
        } else {
            active::with(|r| r.builtin_ids().first().cloned()).unwrap_or_default()
        };
        self.profiles_win = Some(ProfilesWindow {
            selected,
            ..ProfilesWindow::default()
        });
    }

    /// Whether the Profiles window is open.
    pub fn profiles_window_open(&self) -> bool {
        self.profiles_win.is_some()
    }

    /// The window's state, while it is open.
    pub fn profiles_window(&self) -> Option<&ProfilesWindow> {
        self.profiles_win.as_ref()
    }

    fn win(&mut self) -> Option<&mut ProfilesWindow> {
        self.profiles_win.as_mut()
    }

    fn profiles_said(&mut self, m: Message) {
        if let Some(w) = self.win() {
            w.message = Some(m);
        }
    }

    fn profiles_failed(&mut self, e: String) {
        self.profiles_said(Message::new("profiles-error").text("error", e));
    }

    /// The list on the left: the built-ins, then the user's own by name.
    pub fn profiles_list(&self) -> Vec<ListEntry> {
        let drafts = self.profiles_win.as_ref().map(|w| &w.drafts);
        active::with(|r| {
            let mut out = Vec::new();
            let mut push = |p: &Profile, built_in: bool| {
                let draft = drafts.and_then(|d| d.get(&p.id));
                let shown = draft.unwrap_or(p);
                out.push(ListEntry {
                    id: p.id.clone(),
                    name: shown.display_name.clone(),
                    built_in,
                    edited: built_in && r.is_edited(&p.id),
                    unverified: !shown.calibration_meta.verified,
                    unsaved: draft.is_some(),
                    paper: shown.is_paper(),
                    detail: shown.paper.map(|s| {
                        let (w, h) = s.px_at(quadrule_model::DEFAULT_PAPER_PPI);
                        self.strings.format(
                            &Message::new("profiles-paper-list-tip")
                                .text("w", quadrule_layout::units::trim_number(s.width_mm, 2))
                                .text("h", quadrule_layout::units::trim_number(s.height_mm, 2))
                                .num("pw", f64::from(w), 0)
                                .num("ph", f64::from(h), 0),
                        )
                    }),
                });
            };
            for id in r.builtin_ids() {
                if let Some(p) = r.get_saved(&id) {
                    push(p, true);
                }
            }
            for p in r.mine() {
                push(p, false);
            }
            out
        })
    }

    /// The profile in the form: the draft if there is one, else the saved values.
    pub fn profiles_current(&self) -> Option<Profile> {
        let w = self.profiles_win.as_ref()?;
        w.drafts
            .get(&w.selected)
            .cloned()
            .or_else(|| active::with(|r| r.get_saved(&w.selected).cloned()))
    }

    /// Whether the selected profile is a built-in (its id, size, ppi and short name are locked).
    pub fn profiles_selected_is_builtin(&self) -> bool {
        self.profiles_win
            .as_ref()
            .is_some_and(|w| active::with(|r| r.is_builtin(&w.selected)))
    }

    /// Whether the selected built-in carries a saved local override (Reset to built-in has something
    /// to remove; a damaged override file counts).
    pub fn profiles_can_reset(&self) -> bool {
        self.profiles_win.as_ref().is_some_and(|w| {
            active::with(|r| {
                r.is_builtin(&w.selected)
                    && (r.get_saved(&w.selected) != r.shipped(&w.selected)
                        || r.has_broken_override(&w.selected))
            })
        })
    }

    /// Selects a profile in the list.
    pub fn profiles_select(&mut self, id: &str) {
        let known = active::with(|r| r.stored(id).is_some());
        if let Some(w) = self.win()
            && known
        {
            w.selected = id.to_string();
            w.texts.clear();
        }
    }

    /// Changes the selected profile in the form. The change is a draft: the open page redraws with
    /// it, nothing is written. A built-in's id, size, ppi and short name cannot change (Q2).
    pub fn profiles_edit(&mut self, f: impl FnOnce(&mut Profile)) {
        let Some(mut p) = self.profiles_current() else {
            return;
        };
        let before = p.clone();
        f(&mut p);
        // Which profile this is never changes.
        p.id = before.id.clone();
        active::with(|r| {
            if let Some(s) = r.shipped(&p.id) {
                p.width_px = s.width_px;
                p.height_px = s.height_px;
                p.ppi = s.ppi;
                p.short_name = s.short_name.clone();
            }
        });
        p.turned = false;
        if p == before {
            return;
        }
        self.show_draft(p);
    }

    fn show_draft(&mut self, p: Profile) {
        // A draft equal to what is saved is no draft.
        let saved = active::with(|r| r.get_saved(&p.id).cloned());
        let same = saved.as_ref() == Some(&p);
        let id = p.id.clone();
        if let Some(w) = self.win() {
            if same {
                w.drafts.remove(&id);
            } else {
                w.drafts.insert(id.clone(), p.clone());
            }
        }
        active::with_mut(|r| {
            if same {
                r.clear_previews();
            } else {
                r.set_preview(p);
            }
        });
        self.restore_other_previews();
        self.viewer.invalidate_pages();
    }

    /// Previews are the drafts: put back the ones that `clear_previews` dropped.
    fn restore_other_previews(&mut self) {
        let drafts: Vec<Profile> = self
            .profiles_win
            .as_ref()
            .map(|w| w.drafts.values().cloned().collect())
            .unwrap_or_default();
        active::with_mut(|r| {
            r.clear_previews();
            for d in drafts {
                r.set_preview(d);
            }
        });
    }

    /// Whether anything is changed and not saved.
    pub fn profiles_dirty(&self) -> bool {
        self.profiles_win
            .as_ref()
            .is_some_and(|w| !w.drafts.is_empty())
    }

    /// Save: writes every draft (a built-in's as its local override; equal to the shipped values, no
    /// override is kept). A draft that cannot be saved stays, with the reason in the message.
    pub fn profiles_save(&mut self) {
        let drafts: Vec<Profile> = match self.profiles_win.as_ref() {
            Some(w) => w.drafts.values().cloned().collect(),
            None => return,
        };
        let mut failed: Option<String> = None;
        let mut saved = 0;
        for p in drafts {
            let id = p.id.clone();
            match active::with_mut(|r| {
                r.clear_previews();
                r.save(p)
            }) {
                Ok(()) => {
                    saved += 1;
                    if let Some(w) = self.win() {
                        w.drafts.remove(&id);
                    }
                }
                Err(e) => failed = Some(e),
            }
        }
        self.restore_other_previews();
        self.viewer.invalidate_pages();
        match failed {
            Some(e) => self.profiles_failed(e),
            None if saved > 0 => self.profiles_said(Message::new("profiles-saved")),
            None => {}
        }
    }

    /// Cancel: throws away everything changed since the window opened or last saved, and closes it.
    pub fn profiles_cancel(&mut self) {
        active::with_mut(|r| r.clear_previews());
        self.profiles_win = None;
        self.viewer.invalidate_pages();
    }

    /// New…: a new profile of the user's, copied from `from`, under `name`, Unverified.
    pub fn profiles_new(&mut self, name: &str, from: &str) {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > limits::MAX_NAME_CHARS {
            self.profiles_failed(format!(
                "the name must be 1 to {} characters",
                limits::MAX_NAME_CHARS
            ));
            return;
        }
        match active::with_mut(|r| r.create(from, name)) {
            Ok(id) => {
                self.profiles_select(&id);
                if let Some(w) = self.win() {
                    w.new_form = None;
                }
                self.profiles_said(Message::new("profiles-created").text("name", name.to_string()));
                self.viewer.invalidate_pages();
            }
            Err(e) => self.profiles_failed(e),
        }
    }

    /// Duplicate: a copy of the selected profile, named "… copy".
    pub fn profiles_duplicate(&mut self) {
        let Some(p) = self.profiles_current() else {
            return;
        };
        let name = self
            .strings
            .format(&Message::new("profiles-copy-name").text("name", p.display_name.clone()));
        let from = p.id.clone();
        // A draft is what the user sees, so it is what is copied.
        let made = active::with_mut(|r| {
            let mut copy = r.start_from(&from, &name)?;
            let id = copy.id.clone();
            let (short, meta) = (copy.short_name.clone(), copy.calibration_meta.clone());
            copy = p.clone();
            copy.id = id.clone();
            copy.display_name = name.clone();
            copy.short_name = short;
            copy.calibration_meta = meta;
            copy.turned = false;
            r.save_mine(copy)?;
            Ok::<String, String>(id)
        });
        match made {
            Ok(id) => {
                self.profiles_select(&id);
                self.profiles_said(Message::new("profiles-created").text("name", name));
            }
            Err(e) => self.profiles_failed(e),
        }
    }

    /// Asks before Reset to built-in, or before Delete.
    pub fn profiles_ask(&mut self, c: Confirm) {
        if let Some(w) = self.win() {
            w.confirm = Some(c);
        }
    }

    /// Answers the question Reset to built-in or Delete asked.
    pub fn profiles_confirm(&mut self, yes: bool) {
        let Some(c) = self.win().and_then(|w| w.confirm.take()) else {
            return;
        };
        if !yes {
            return;
        }
        let id = match &c {
            Confirm::Reset(i) | Confirm::Delete(i) | Confirm::DeleteDamaged(i) => i.clone(),
        };
        if let Some(w) = self.win() {
            w.drafts.remove(&id);
        }
        let result = active::with_mut(|r| {
            r.clear_previews();
            match &c {
                Confirm::Reset(i) => r.reset_builtin(i),
                Confirm::Delete(i) => r.delete_mine(i),
                Confirm::DeleteDamaged(f) => r.delete_damaged(f),
            }
        });
        self.restore_other_previews();
        match result {
            Ok(()) if matches!(c, Confirm::DeleteDamaged(_)) => {
                self.profiles_said(Message::new("profiles-damaged-deleted").text("file", id));
            }
            Ok(()) => {
                if matches!(c, Confirm::Delete(_)) {
                    let first =
                        active::with(|r| r.builtin_ids().first().cloned()).unwrap_or_default();
                    self.profiles_select(&first);
                    self.profiles_said(Message::new("profiles-deleted"));
                } else {
                    self.profiles_said(Message::new("profiles-was-reset"));
                }
            }
            Err(e) => self.profiles_failed(e),
        }
        self.viewer.invalidate_pages();
    }

    /// Import…: reads a `.ron` file (at most [`limits::MAX_FILE_BYTES`] bytes) and imports it.
    pub fn profiles_import_path(&mut self, path: &Path) {
        use std::io::Read;
        let read = std::fs::File::open(path)
            .map_err(|e| e.to_string())
            .and_then(|f| {
                let mut bytes = Vec::new();
                f.take(limits::MAX_FILE_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() > limits::MAX_FILE_BYTES {
                    return Err(format!(
                        "the file is larger than {} bytes",
                        limits::MAX_FILE_BYTES
                    ));
                }
                String::from_utf8(bytes).map_err(|_| "the file is not text".to_string())
            });
        match read {
            Ok(text) => {
                let _ = self.profiles_import_text(&text);
            }
            Err(e) => {
                self.profiles_said(Message::new("profiles-import-bad").text("error", e));
            }
        }
    }

    /// Imports profile text. A new one is added; one this computer has with the same values says so;
    /// a clash waits for the answer ([`ViewerUi::profiles_import_answer`]): the same id asks Replace
    /// mine or Keep both (Q7), the same name asks Merge or Rename (Q6, Q9).
    pub fn profiles_import_text(&mut self, text: &str) -> Result<(), String> {
        let incoming = match parse_checked(text) {
            Ok(p) => p,
            Err(e) => {
                let why = e.to_string();
                self.profiles_said(Message::new("profiles-import-bad").text("error", why.clone()));
                return Err(why);
            }
        };
        let kind = active::with(|r| r.classify(&incoming));
        match kind {
            Incoming::Same => {
                self.profiles_said(Message::new("profiles-import-same"));
                Ok(())
            }
            Incoming::New => {
                let name = incoming.display_name.clone();
                match active::with_mut(|r| r.add(incoming)) {
                    Ok(id) => {
                        self.profiles_select(&id);
                        self.profiles_said(Message::new("profiles-imported").text("name", name));
                        Ok(())
                    }
                    Err(e) => {
                        self.profiles_failed(e.clone());
                        Err(e)
                    }
                }
            }
            Incoming::IdClash | Incoming::NameClash(_) => {
                let with = match &kind {
                    Incoming::NameClash(w) => w.clone(),
                    _ => incoming.id.clone(),
                };
                let mine = active::display_name(&with).unwrap_or_default();
                let new_name = self.strings.format(
                    &Message::new("profiles-clash-new-name")
                        .text("name", incoming.display_name.clone()),
                );
                if let Some(w) = self.win() {
                    w.import = Some(ImportAsk {
                        incoming,
                        kind,
                        mine,
                        new_name,
                    });
                }
                Ok(())
            }
        }
    }

    /// Answers an import that clashes; `None` cancels it.
    pub fn profiles_import_answer(&mut self, pick: Option<ImportPick>) {
        let Some(ask) = self.win().and_then(|w| w.import.take()) else {
            return;
        };
        let Some(pick) = pick else {
            return;
        };
        let name = ask.incoming.display_name.clone();
        let into = match &ask.kind {
            Incoming::NameClash(w) => w.clone(),
            _ => ask.incoming.id.clone(),
        };
        let result = active::with_mut(|r| match pick {
            ImportPick::Replace => r
                .replace(ask.incoming.clone())
                .map(|()| ask.incoming.id.clone()),
            ImportPick::KeepBoth => r.keep_both(ask.incoming.clone()),
            ImportPick::Merge => r.merge(&ask.incoming, &into),
            ImportPick::Rename(n) => r.rename_incoming(&ask.incoming, &n),
        });
        match result {
            Ok(id) => {
                self.profiles_select(&id);
                self.profiles_said(Message::new("profiles-imported").text("name", name));
                self.viewer.invalidate_pages();
            }
            Err(e) => self.profiles_failed(e),
        }
    }

    /// The file name Export… suggests for the selected profile: its short name and `.ron`.
    pub fn profiles_export_name(&self) -> Option<String> {
        self.profiles_current()
            .map(|p| format!("{}.ron", p.short_name))
    }

    /// Export…: writes the selected profile (as drafted, if it is) to `path` as RON text.
    pub fn profiles_export_to(&mut self, path: &Path) {
        let Some(p) = self.profiles_current() else {
            return;
        };
        match quadrule_profiles::registry::write_atomic(path, &to_ron(&p)) {
            Ok(()) => self.profiles_said(
                Message::new("profiles-exported").text("path", path.display().to_string()),
            ),
            Err(e) => self.profiles_failed(e),
        }
    }

    /// The files that could not be loaded, for the list's Damaged files group.
    pub fn profiles_damaged(&self) -> Vec<Damaged> {
        active::with(|r| r.damaged().to_vec())
    }

    /// A note, once at start, when any profile file in the config folder is damaged (Q8).
    pub fn note_damaged_profiles(&mut self) {
        let n = active::with(|r| r.damaged().len());
        if n > 0 {
            self.viewer.notice = Some(Notice {
                message: Message::new("notice-profiles-damaged").num("count", n as f64, 0),
                error: true,
            });
        }
    }

    /// A device's full name with the **Edited** mark, as the pickers show it.
    pub fn full_name_for_test(&self, id: &str) -> String {
        self.full_name(id)
    }

    /// Where a profile in the list comes from, for the mark beside it.
    pub fn profile_source(&self, id: &str) -> Option<Source> {
        active::with(|r| r.source(id))
    }
}
