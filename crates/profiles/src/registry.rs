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

//! The one lookup for profiles (rung 13a bbq, Defaulted "One registry"): the shipped built-ins, then
//! the user's local overrides of them (an edited Manta or Nomad), then the user's own profiles, then
//! the copies the open document carries. Loaded from `<config folder>/profiles/*.ron` at start;
//! a damaged file is left where it is, listed and skipped, and never stops the app (Q8). Nothing here
//! panics on a bad file, and nothing reads a file except [`Registry::load`] and the save, delete and
//! reset calls: a lookup is a map read, so it is safe to ask every frame.

use crate::{BUILTIN_RON, Profile, from_ron, limits, parse_checked, to_ron};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Where a profile comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A built-in, with the values the app ships.
    Shipped,
    /// A built-in with a local override (the **Edited** mark).
    Edited,
    /// One of the user's own profiles.
    Mine,
    /// Only the open document's carried copy.
    Carried,
}

/// A profile file that could not be loaded (rung 13a bbq, Q8).
#[derive(Debug, Clone, PartialEq)]
pub struct Damaged {
    /// The file's name in the profiles folder.
    pub file: String,
    /// What is wrong.
    pub error: String,
    /// The line it stopped at, when RON gave one.
    pub line: Option<usize>,
}

/// What an incoming profile (an imported file, or one a `.quad` carries) is to this computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoming {
    /// The computer already has this id with these very values.
    Same,
    /// The computer has the id, with other values.
    IdClash,
    /// The computer has another profile of the same name (the id of that one).
    NameClash(String),
    /// Nothing like it here.
    New,
}

/// Every profile the app knows.
#[derive(Debug, Clone)]
pub struct Registry {
    dir: Option<PathBuf>,
    shipped: Vec<Profile>,
    overrides: BTreeMap<String, Profile>,
    broken: BTreeSet<String>,
    mine: BTreeMap<String, Profile>,
    carried: BTreeMap<String, Profile>,
    carried_first: BTreeSet<String>,
    previews: BTreeMap<String, Profile>,
    damaged: Vec<Damaged>,
}

fn same_name(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

impl Registry {
    /// The built-ins only: what every test and golden sees, and what the app has when it has no
    /// config folder.
    pub fn builtin_only() -> Registry {
        Registry {
            dir: None,
            shipped: BUILTIN_RON
                .iter()
                .map(|(_, src)| from_ron(src).expect("a shipped profile parses"))
                .collect(),
            overrides: BTreeMap::new(),
            broken: BTreeSet::new(),
            mine: BTreeMap::new(),
            carried: BTreeMap::new(),
            carried_first: BTreeSet::new(),
            previews: BTreeMap::new(),
            damaged: Vec::new(),
        }
    }

    /// An empty registry that saves into `dir` (made when first needed) and reads nothing.
    pub fn at(dir: &Path) -> Registry {
        Registry {
            dir: Some(dir.to_path_buf()),
            ..Registry::builtin_only()
        }
    }

    /// Reads every `*.ron` in `dir`, in file-name order. A file that is unreadable, too large, not
    /// valid RON, outside the limits, named other than its id, or an override that changes a built-in's
    /// size, ppi or short name is listed as damaged and left untouched. A folder that is missing or
    /// unreadable is an empty one.
    pub fn load(dir: &Path) -> Registry {
        let mut r = Registry::at(dir);
        let Ok(rd) = std::fs::read_dir(dir) else {
            return r;
        };
        let mut files: Vec<PathBuf> = rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "ron") && p.is_file())
            .collect();
        files.sort();
        for path in files {
            let file = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let stem = path
                .file_stem()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let shipped_id = r.shipped.iter().any(|s| s.id == stem);
            let result = r.read_file(&path, &stem);
            match result {
                Ok(p) if shipped_id => {
                    r.overrides.insert(p.id.clone(), p);
                }
                Ok(p) => {
                    r.mine.insert(p.id.clone(), p);
                }
                Err((error, line)) => {
                    if shipped_id {
                        r.broken.insert(stem);
                    }
                    r.damaged.push(Damaged { file, error, line });
                }
            }
        }
        r
    }

    fn read_file(&self, path: &Path, stem: &str) -> Result<Profile, (String, Option<usize>)> {
        use std::io::Read;
        let file = std::fs::File::open(path).map_err(|e| (e.to_string(), None))?;
        let mut bytes = Vec::new();
        file.take(limits::MAX_FILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| (e.to_string(), None))?;
        if bytes.len() > limits::MAX_FILE_BYTES {
            return Err((
                format!("the file is larger than {} bytes", limits::MAX_FILE_BYTES),
                None,
            ));
        }
        let text =
            String::from_utf8(bytes).map_err(|_| ("the file is not text".to_string(), None))?;
        let mut p = parse_checked(&text).map_err(|e| (e.message, e.line))?;
        // An override saved before the lint block existed (build 0.13 and earlier) has none; the
        // person never chose to leave every threshold empty, so the shipped ones apply until they edit
        // the block (rung 14a, Defaulted).
        if let Some(s) = self.shipped.iter().find(|s| s.id == stem)
            && !text.contains("lint:")
        {
            p.lint = s.lint.clone();
        }
        if p.id != stem {
            return Err((
                format!("the id `{}` does not match the file name", p.id),
                None,
            ));
        }
        if let Some(s) = self.shipped.iter().find(|s| s.id == stem)
            && let Some(what) = locked_difference(s, &p)
        {
            return Err((
                format!("an edited built-in profile cannot change its {what}"),
                None,
            ));
        }
        Ok(p)
    }

    /// The folder profiles are saved in, if there is one.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// The files that could not be loaded.
    pub fn damaged(&self) -> &[Damaged] {
        &self.damaged
    }

    /// The ids of the built-ins, in shipped order.
    pub fn builtin_ids(&self) -> Vec<String> {
        self.shipped.iter().map(|p| p.id.clone()).collect()
    }

    /// Whether `id` is a built-in's.
    pub fn is_builtin(&self, id: &str) -> bool {
        self.shipped.iter().any(|p| p.id == id)
    }

    /// The built-in's shipped values.
    pub fn shipped(&self, id: &str) -> Option<&Profile> {
        self.shipped.iter().find(|p| p.id == id)
    }

    /// The ids of the devices (screens) a page can target on this computer: the built-in ones, then
    /// the user's own by name. The paper sizes are [`Registry::paper_ids`].
    pub fn device_ids(&self) -> Vec<String> {
        self.shipped
            .iter()
            .filter(|p| !p.is_paper())
            .chain(self.mine().into_iter().filter(|p| !p.is_paper()))
            .map(|p| p.id.clone())
            .collect()
    }

    /// The ids of the paper sizes a page can target (rung 13b): the built-in ones in shipped order,
    /// then the user's own by name.
    pub fn paper_ids(&self) -> Vec<String> {
        self.shipped
            .iter()
            .filter(|p| p.is_paper())
            .chain(self.mine().into_iter().filter(|p| p.is_paper()))
            .map(|p| p.id.clone())
            .collect()
    }

    /// Whether profile `id`, as this computer shows it, is a paper size.
    pub fn is_paper(&self, id: &str) -> bool {
        self.get(id).is_some_and(Profile::is_paper)
    }

    /// The user's own profiles, by name.
    pub fn mine(&self) -> Vec<&Profile> {
        let mut mine: Vec<&Profile> = self.mine.values().collect();
        mine.sort_by_key(|p| (p.display_name.to_lowercase(), p.id.clone()));
        mine
    }

    /// The profile `id` stands for: for a built-in its override if there is a good one, else the
    /// shipped values; else one of the user's own; else the open document's carried copy. A carried
    /// copy the user put off deciding about (Not now) comes first.
    pub fn get(&self, id: &str) -> Option<&Profile> {
        if let Some(p) = self.previews.get(id) {
            return Some(p);
        }
        self.get_saved(id)
    }

    /// Like [\`Registry::get\`], without the values the Profiles window is trying out: what is saved
    /// on this computer, or in the open document. A file carries these.
    pub fn get_saved(&self, id: &str) -> Option<&Profile> {
        if self.carried_first.contains(id)
            && let Some(c) = self.carried.get(id)
        {
            return Some(c);
        }
        self.stored(id).or_else(|| self.carried.get(id))
    }

    /// Like [`Registry::get`] but never the document's copy: what this computer itself has.
    pub fn stored(&self, id: &str) -> Option<&Profile> {
        if let Some(s) = self.shipped(id) {
            return Some(self.overrides.get(id).unwrap_or(s));
        }
        self.mine.get(id)
    }

    /// Where `id`'s values come from.
    pub fn source(&self, id: &str) -> Option<Source> {
        if self.carried_first.contains(id) && self.carried.contains_key(id) {
            return Some(Source::Carried);
        }
        if self.is_builtin(id) {
            return Some(if self.overrides.contains_key(id) {
                Source::Edited
            } else {
                Source::Shipped
            });
        }
        if self.mine.contains_key(id) {
            return Some(Source::Mine);
        }
        self.carried.contains_key(id).then_some(Source::Carried)
    }

    // ---- values being tried out -----------------------------------------------------------------

    /// Shows `p` instead of what is saved under its id, until [`Registry::clear_previews`]: the
    /// Profiles window redraws the open page as its fields change. Nothing is written.
    pub fn set_preview(&mut self, p: Profile) {
        self.previews.insert(p.id.clone(), p);
    }

    /// Stops showing the values tried out.
    pub fn clear_previews(&mut self) {
        self.previews.clear();
    }

    /// The values being tried out, by id.
    pub fn previews(&self) -> &BTreeMap<String, Profile> {
        &self.previews
    }

    /// Whether built-in `id` carries a local override (the **Edited** mark).
    pub fn is_edited(&self, id: &str) -> bool {
        match self.previews.get(id) {
            Some(p) => self.shipped(id).is_some_and(|s| s != p),
            None => self.overrides.contains_key(id),
        }
    }

    /// Whether built-in `id` has an override file that could not be used, so the shipped values
    /// show instead.
    pub fn has_broken_override(&self, id: &str) -> bool {
        self.broken.contains(id)
    }

    // ---- the open document's copies -------------------------------------------------------------

    /// Puts the open document's carried profiles in, replacing the last document's. Those named in
    /// `first` show instead of what this computer has under the same id (the user chose Not now).
    pub fn set_carried(&mut self, list: Vec<Profile>, first: BTreeSet<String>) {
        self.carried = list.into_iter().map(|p| (p.id.clone(), p)).collect();
        self.carried_first = first;
    }

    /// Forgets the open document's copies.
    pub fn clear_carried(&mut self) {
        self.carried.clear();
        self.carried_first.clear();
    }

    /// The open document's copy of `id`, if it carries one.
    pub fn carried(&self, id: &str) -> Option<&Profile> {
        self.carried.get(id)
    }

    // ---- saving ---------------------------------------------------------------------------------

    fn path_for(&self, id: &str) -> Result<PathBuf, String> {
        limits::check_id(id)?;
        let dir = self
            .dir
            .as_ref()
            .ok_or_else(|| "there is no folder to save profiles in".to_string())?;
        Ok(dir.join(format!("{id}.ron")))
    }

    fn forget_damaged(&mut self, id: &str) {
        let file = format!("{id}.ron");
        self.damaged.retain(|d| d.file != file);
        self.broken.remove(id);
    }

    /// A short name no other profile uses (the shipped ones and the user's own), made from `base`:
    /// `nomad`, then `nomad-2`, `nomad-3`. `ignoring` is the profile being named, which does not
    /// count.
    pub fn suggest_short_name(&self, base: &str, ignoring: Option<&str>) -> String {
        let mut s = String::new();
        for c in base.to_lowercase().chars() {
            if c.is_ascii_lowercase() || c.is_ascii_digit() {
                s.push(c);
            } else if !s.ends_with('-') {
                s.push('-');
            }
        }
        let mut s = s.trim_matches('-').to_string();
        s.truncate(limits::MAX_SHORT_CHARS - 4);
        let s = s.trim_matches('-').to_string();
        let s = if s.is_empty() {
            "profile".to_string()
        } else {
            s
        };
        let taken = |c: &str| {
            self.shipped
                .iter()
                .chain(self.mine.values())
                .any(|p| p.short_name == c && Some(p.id.as_str()) != ignoring)
        };
        if !taken(&s) {
            return s;
        }
        (2..)
            .map(|n| format!("{s}-{n}"))
            .find(|c| !taken(c))
            .expect("some number is free")
    }

    /// Whether another profile already uses `short` (shipped or the user's own).
    pub fn short_name_taken(&self, short: &str, ignoring: Option<&str>) -> bool {
        self.shipped
            .iter()
            .chain(self.mine.values())
            .any(|p| p.short_name == short && Some(p.id.as_str()) != ignoring)
    }

    /// A copy of profile `from` as a new profile of the user's: a fresh UUID v4 id, the name given,
    /// a short name made from it, Unverified. Not saved yet.
    pub fn start_from(&self, from: &str, name: &str) -> Result<Profile, String> {
        let base = self
            .get(from)
            .ok_or_else(|| format!("no profile `{from}`"))?;
        let mut p = base.clone();
        p.id = uuid::Uuid::new_v4().to_string();
        p.display_name = name.trim().to_string();
        p.short_name = self.suggest_short_name(name, None);
        p.calibration_meta.verified = false;
        p.calibration_meta.date = None;
        p.calibration_meta.firmware = None;
        p.turned = false;
        Ok(p)
    }

    /// Makes and saves a new profile from `from` (New… and Duplicate). Returns its id.
    pub fn create(&mut self, from: &str, name: &str) -> Result<String, String> {
        let p = self.start_from(from, name)?;
        let id = p.id.clone();
        self.save_mine(p)?;
        Ok(id)
    }

    /// Saves one of the user's own profiles, whole, to `<folder>/<id>.ron` through a temporary
    /// name, so a crash never leaves half a file. Refuses values outside the limits, a built-in's
    /// id and a short name another profile uses.
    pub fn save_mine(&mut self, p: Profile) -> Result<(), String> {
        if self.is_builtin(&p.id) {
            return Err("that id belongs to a built-in profile".into());
        }
        limits::check(&p)?;
        if p.turned {
            return Err("a turned profile is never saved".into());
        }
        if self.short_name_taken(&p.short_name, Some(&p.id)) {
            return Err(format!("the short name `{}` is already used", p.short_name));
        }
        write_atomic(&self.path_for(&p.id)?, &to_ron(&p))?;
        self.forget_damaged(&p.id);
        self.mine.insert(p.id.clone(), p);
        Ok(())
    }

    /// Saves an edited built-in as a local override. Its id, size, ppi and short name are put back to
    /// the shipped ones (a built-in stays the same device). If the values equal the shipped ones, the
    /// override file is removed instead.
    pub fn save_override(&mut self, mut p: Profile) -> Result<(), String> {
        let Some(shipped) = self.shipped(&p.id).cloned() else {
            return Err("that is not a built-in profile".into());
        };
        p.id = shipped.id.clone();
        p.width_px = shipped.width_px;
        p.height_px = shipped.height_px;
        p.ppi = shipped.ppi;
        p.paper = shipped.paper;
        p.short_name = shipped.short_name.clone();
        p.turned = false;
        limits::check(&p)?;
        if p == shipped {
            return self.reset_builtin(&p.id);
        }
        write_atomic(&self.path_for(&p.id)?, &to_ron(&p))?;
        self.forget_damaged(&p.id);
        self.overrides.insert(p.id.clone(), p);
        Ok(())
    }

    /// Saves `p` where it belongs: a built-in's id as an override, any other as one of the user's.
    pub fn save(&mut self, p: Profile) -> Result<(), String> {
        if self.is_builtin(&p.id) {
            self.save_override(p)
        } else {
            self.save_mine(p)
        }
    }

    /// Reset to built-in: deletes the override file (a damaged one too) and the shipped values show
    /// again. One-way: no copy is kept.
    pub fn reset_builtin(&mut self, id: &str) -> Result<(), String> {
        if !self.is_builtin(id) {
            return Err("that is not a built-in profile".into());
        }
        remove_file(&self.path_for(id)?)?;
        self.overrides.remove(id);
        self.forget_damaged(id);
        Ok(())
    }

    /// Deletes one of the user's own profiles. One-way. A page that uses it still opens from its
    /// carried copy.
    pub fn delete_mine(&mut self, id: &str) -> Result<(), String> {
        if !self.mine.contains_key(id) {
            return Err("that is not one of your profiles".into());
        }
        remove_file(&self.path_for(id)?)?;
        self.mine.remove(id);
        Ok(())
    }

    /// Removes a damaged file from disk and from the list.
    pub fn delete_damaged(&mut self, file: &str) -> Result<(), String> {
        let Some(i) = self.damaged.iter().position(|d| d.file == file) else {
            return Err("that file is not in the damaged list".into());
        };
        let dir = self.dir.clone().ok_or("there is no profiles folder")?;
        if file.contains(['/', '\\']) {
            return Err("not a file name".into());
        }
        remove_file(&dir.join(file))?;
        self.damaged.remove(i);
        if let Some(stem) = file.strip_suffix(".ron") {
            self.broken.remove(stem);
        }
        Ok(())
    }

    // ---- import, Merge and Rename (Q6, Q7, Q9) --------------------------------------------------

    /// What an incoming profile is here. A built-in's id with other values than what this computer
    /// shows is an id clash; so is one of the user's own.
    pub fn classify(&self, incoming: &Profile) -> Incoming {
        if let Some(cur) = self.stored(&incoming.id) {
            return if cur == incoming {
                Incoming::Same
            } else {
                Incoming::IdClash
            };
        }
        let named = self
            .shipped
            .iter()
            .map(|s| self.stored(&s.id).unwrap_or(s))
            .chain(self.mine.values())
            .find(|p| same_name(&p.display_name, &incoming.display_name));
        match named {
            Some(p) => Incoming::NameClash(p.id.clone()),
            None => Incoming::New,
        }
    }

    /// Adds an incoming profile as one of the user's own, under its own id (which must be free). A
    /// short name that is taken is changed to a free one.
    pub fn add(&mut self, mut p: Profile) -> Result<String, String> {
        if self.stored(&p.id).is_some() {
            return Err("this computer already has that id".into());
        }
        p.turned = false;
        p.short_name = self.suggest_short_name(&p.short_name, Some(&p.id));
        let id = p.id.clone();
        self.save_mine(p)?;
        Ok(id)
    }

    /// Keep both: the incoming profile joins the user's own under a new UUID v4 id. Returns the id.
    pub fn keep_both(&mut self, mut p: Profile) -> Result<String, String> {
        p.id = uuid::Uuid::new_v4().to_string();
        self.add(p)
    }

    /// Replace mine: the incoming values take the place of this computer's profile of the same id
    /// (a built-in's becomes its override).
    pub fn replace(&mut self, mut p: Profile) -> Result<(), String> {
        if self.stored(&p.id).is_none() {
            return Err("this computer has no profile with that id".into());
        }
        if let Some(mine) = self.mine.get(&p.id) {
            p.short_name = mine.short_name.clone();
        }
        self.save(p)
    }

    /// Merge: the incoming values update this computer's profile `into` (its id and short name stay;
    /// for a built-in they become its local override, with the size and ppi kept). Returns `into`.
    pub fn merge(&mut self, incoming: &Profile, into: &str) -> Result<String, String> {
        let target = self
            .stored(into)
            .ok_or_else(|| "this computer has no such profile".to_string())?
            .clone();
        let mut p = incoming.clone();
        p.id = target.id.clone();
        p.short_name = target.short_name.clone();
        self.save(p)?;
        Ok(target.id)
    }

    /// Rename: the incoming profile joins the user's own under `new_name`. It keeps its id when this
    /// computer does not have it, and gets a new UUID v4 id when it does. Returns the id it has now.
    pub fn rename_incoming(
        &mut self,
        incoming: &Profile,
        new_name: &str,
    ) -> Result<String, String> {
        let mut p = incoming.clone();
        p.display_name = new_name.trim().to_string();
        if self.stored(&p.id).is_some() || self.is_builtin(&p.id) {
            p.id = uuid::Uuid::new_v4().to_string();
        }
        p.short_name = self.suggest_short_name(&p.display_name, Some(&p.id));
        self.add(p)
    }
}

/// The first of a built-in's locked fields (id, size, ppi, short name) on which `p` differs from
/// the shipped profile, if any.
fn locked_difference(shipped: &Profile, p: &Profile) -> Option<&'static str> {
    if p.id != shipped.id {
        Some("id")
    } else if p.width_px != shipped.width_px
        || p.height_px != shipped.height_px
        || p.paper != shipped.paper
    {
        Some("size")
    } else if p.ppi != shipped.ppi {
        Some("ppi")
    } else if p.short_name != shipped.short_name {
        Some("short name")
    } else {
        None
    }
}

/// Writes `text` to a new file beside `path` and renames it over `path`, so a crash never leaves
/// half a file. Makes the folder when it is missing.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("no folder")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.saving"));
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", path.display())
    })
}

fn remove_file(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}
