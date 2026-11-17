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

//! How the open document and the profiles meet (rung 13a bbq, Q4, Q6, Q9 and Defaulted "When
//! Merge/Rename is asked"): the copies a `.quad` carries, what to say when this computer lacks one or
//! has another of the same name or id, and what Merge, Rename and Not now do. The window's widgets are in
//! [`crate::profiles_ui`].

use crate::viewer::{Notice, Viewer};
use quadrule_format::QuadFile;
use quadrule_i18n::Message;
use quadrule_profiles::{Incoming, Profile, active};
use std::collections::BTreeSet;

/// A profile the open file carries that this computer has another version of.
#[derive(Clone, Debug, PartialEq)]
pub struct Clash {
    /// The file's copy.
    pub incoming: Profile,
    /// Same id with other values, or another id with the same name (and which one).
    pub kind: Incoming,
    /// The name of the profile here it clashes with.
    pub mine: String,
}

/// What the user chose for one clash.
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileChoice {
    /// The file's values update the profile here.
    Merge,
    /// The file's copy joins the user's own under this name.
    Rename(String),
    /// Decide later: the page shows the file's copy for this session.
    NotNow,
}

/// What the open document's profiles need.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileUse {
    /// Profiles the file carries and this computer lacks, by id: the bar's [Add to my profiles].
    pub missing: Vec<String>,
    /// Carried profiles that clash with one here, waiting for an answer.
    pub clashes: Vec<Clash>,
    /// Clashes the user put off (Not now): the page shows the file's copy.
    pub deferred: Vec<Clash>,
    /// Secondary devices left out of the page: neither here nor carried (or carried and broken).
    pub left_out: Vec<String>,
    /// Carried profile entries that could not be read: (path, why).
    pub damaged: Vec<(String, String)>,
    /// Every profile the file carried that the page uses.
    pub carried: Vec<Profile>,
    /// The bar was closed with its ×; it stays closed until the document is opened again.
    pub bar_closed: bool,
}

impl Viewer {
    /// Settles which profiles the file's page uses: those the file carries are shown as carried
    /// copies; a secondary device that is neither here nor carried is left out of the page; the
    /// primary missing refuses the file. The copies go to the registry when the document is set.
    pub(crate) fn meet_profiles(&mut self, file: &mut QuadFile) -> Result<ProfileUse, String> {
        let mut u = ProfileUse {
            damaged: file.damaged_profiles.clone(),
            ..ProfileUse::default()
        };
        let ids: Vec<String> = file
            .doc
            .profile_ids()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut first = BTreeSet::new();
        active::with(|reg| {
            for id in &ids {
                match file.profiles.get(id) {
                    Some(c) => {
                        u.carried.push(c.clone());
                        match reg.classify(c) {
                            Incoming::Same => {}
                            Incoming::New => u.missing.push(id.clone()),
                            kind @ Incoming::IdClash => {
                                first.insert(id.clone());
                                let mine = reg
                                    .stored(id)
                                    .map(|p| p.display_name.clone())
                                    .unwrap_or_default();
                                u.clashes.push(Clash {
                                    incoming: c.clone(),
                                    kind,
                                    mine,
                                });
                            }
                            Incoming::NameClash(with) => {
                                let mine = reg
                                    .stored(&with)
                                    .map(|p| p.display_name.clone())
                                    .unwrap_or_default();
                                u.clashes.push(Clash {
                                    incoming: c.clone(),
                                    kind: Incoming::NameClash(with),
                                    mine,
                                });
                            }
                        }
                    }
                    None if reg.get_saved(id).is_none() => u.left_out.push(id.clone()),
                    None => {}
                }
            }
        });
        if let Some(primary) = u
            .left_out
            .iter()
            .find(|id| **id == file.doc.primary_profile_id)
        {
            return Err(format!(
                "this page is designed on the profile `{primary}`, which is not on this computer and not in the file"
            ));
        }
        if !u.left_out.is_empty() {
            file.doc
                .profiles
                .retain(|p| !u.left_out.contains(&p.profile_id));
            file.outlines.retain(|id, _| !u.left_out.contains(id));
        }
        self.pending_carried = Some((u.carried.clone(), first));
        Ok(u)
    }

    /// What the open document's profiles need, if a document is open.
    pub fn profile_use(&self) -> Option<&ProfileUse> {
        self.open.as_ref().map(|o| &o.profile_use)
    }

    /// The profiles the bar names, while it shows: those the file carries that this computer lacks,
    /// and those the user put off deciding about.
    pub fn profile_bar(&self) -> Option<(Vec<String>, Vec<String>)> {
        let o = self.open.as_ref()?;
        let u = &o.profile_use;
        if u.bar_closed || (u.missing.is_empty() && u.deferred.is_empty()) {
            return None;
        }
        let name = |id: &str| active::display_name(id).unwrap_or_else(|| id.to_string());
        Some((
            u.missing.iter().map(|i| name(i)).collect(),
            u.deferred.iter().map(|c| name(&c.incoming.id)).collect(),
        ))
    }

    /// Closes the profiles bar.
    pub fn close_profile_bar(&mut self) {
        if let Some(o) = &mut self.open {
            o.profile_use.bar_closed = true;
        }
    }

    /// The bar's [Add to my profiles]: the carried copies this computer lacked become the user's own.
    pub fn add_missing_profiles(&mut self) -> Result<usize, String> {
        let Some(o) = &mut self.open else {
            return Ok(0);
        };
        let want = std::mem::take(&mut o.profile_use.missing);
        let mut added = 0;
        let mut failed = Vec::new();
        let mut error = None;
        for id in want {
            let Some(p) = o.profile_use.carried.iter().find(|p| p.id == id).cloned() else {
                continue;
            };
            match active::with_mut(|r| r.add(p)) {
                Ok(_) => added += 1,
                Err(e) => {
                    error = Some(e);
                    failed.push(id);
                }
            }
        }
        o.profile_use.missing = failed;
        self.invalidate_pages();
        match error {
            Some(e) => {
                self.notice = Some(Notice {
                    message: Message::new("notice-profile-failed").text("error", e.clone()),
                    error: true,
                });
                Err(e)
            }
            None => Ok(added),
        }
    }

    /// Settles the clashes with the user's choices, one per carried profile (by the id the file
    /// gives it); a clash with no choice stays asked. Merge updates the profile here with the file's
    /// values; Rename adds the file's copy under a new name; Not now keeps showing the file's copy.
    /// When the profile ends up under another id than the page uses, the page is changed to that id
    /// (an unsaved change, not an undo step).
    pub fn resolve_profile_clashes(&mut self, choices: &[(String, ProfileChoice)]) {
        let Some(open) = self.open.as_mut() else {
            return;
        };
        let clashes = std::mem::take(&mut open.profile_use.clashes);
        let mut asked_still = Vec::new();
        let mut deferred = std::mem::take(&mut open.profile_use.deferred);
        let mut errors = Vec::new();
        let mut renames: Vec<(String, String)> = Vec::new();
        for c in clashes {
            let choice = choices
                .iter()
                .find(|(id, _)| *id == c.incoming.id)
                .map(|(_, ch)| ch.clone());
            let id = c.incoming.id.clone();
            let result = match choice {
                None => {
                    asked_still.push(c);
                    continue;
                }
                Some(ProfileChoice::NotNow) => {
                    deferred.push(c);
                    continue;
                }
                Some(ProfileChoice::Merge) => {
                    let into = match &c.kind {
                        Incoming::NameClash(with) => with.clone(),
                        _ => id.clone(),
                    };
                    active::with_mut(|r| r.merge(&c.incoming, &into))
                }
                Some(ProfileChoice::Rename(name)) => {
                    active::with_mut(|r| r.rename_incoming(&c.incoming, &name))
                }
            };
            match result {
                Ok(now) => {
                    if now != id {
                        renames.push((id, now));
                    }
                }
                Err(e) => {
                    errors.push(e);
                    asked_still.push(c);
                }
            }
        }
        open.profile_use.clashes = asked_still;
        open.profile_use.deferred = deferred;
        // The registry shows the file's copy first only where the user put the decision off.
        let first: BTreeSet<String> = open
            .profile_use
            .deferred
            .iter()
            .chain(open.profile_use.clashes.iter())
            .filter(|c| c.kind == Incoming::IdClash)
            .map(|c| c.incoming.id.clone())
            .collect();
        let carried = open.profile_use.carried.clone();
        active::with_mut(|r| r.set_carried(carried, first));
        for (from, to) in renames {
            if let Err(e) = self.rename_profile_in_page(&from, &to) {
                errors.push(e);
            }
        }
        self.invalidate_pages();
        if let Some(e) = errors.into_iter().next() {
            self.notice = Some(Notice {
                message: Message::new("notice-profile-failed").text("error", e),
                error: true,
            });
        }
    }

    /// The page names profile `to` where it named `from`.
    fn rename_profile_in_page(&mut self, from: &str, to: &str) -> Result<(), String> {
        let Some(open) = self.open.as_mut() else {
            return Ok(());
        };
        if open.doc.profile_ids().contains(&to) {
            return Err("this page already uses that profile".into());
        }
        open.doc =
            quadrule_format::rename_profile(&open.doc, from, to).map_err(|e| e.to_string())?;
        open.saved = quadrule_format::rename_profile_outlines(&open.saved, from, to);
        open.pages.clear();
        open.findings.clear();
        open.profile_use.carried.retain(|p| p.id != from);
        open.history.mark_unsaved();
        if self.profile_id == from {
            self.profile_id = to.to_string();
        }
        self.revision += 1;
        Ok(())
    }

    /// Opens the clash dialog again for the clashes put off (the bar's Merge or rename…).
    pub fn ask_deferred_clashes(&mut self) {
        if let Some(o) = &mut self.open {
            let mut back = std::mem::take(&mut o.profile_use.deferred);
            back.append(&mut o.profile_use.clashes);
            o.profile_use.clashes = back;
        }
    }
}
