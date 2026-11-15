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

//! The fonts text can be laid out with (rung 11a, docs/bbq/2026-11-14-rung-11-text-and-fonts.md):
//! the bundled families, the families installed on this computer (found by fontdb's system scan,
//! re-scanned when the window regains focus, ADR-002 item 8), and the rule that a character the
//! chosen font lacks comes from the bundled fonts only (rule Q4; rung 12 opens it to any installed
//! font together with its save and export warning, ADR-002 item 6).
//!
//! Two stores are kept apart. The **catalog** lists the installed faces for the app's pickers and
//! is shared by copies of a registry. The **layout store** is the cosmic-text `FontSystem` that
//! text is actually shaped with: it holds the bundled faces, a copy of each bundled face at the
//! optical sizes asked for ([`crate::opsz`]), and only those installed families some text box
//! uses. The families of the last kind are *forbidden as fallback* in its [`Fallback`] list, so
//! they serve only the boxes that chose them.

use crate::opsz;
use crate::{BUNDLED, BundledFont};
use cosmic_text::skrifa::{FontRef, MetadataProvider, Tag};
use cosmic_text::{Fallback, FontSystem, fontdb};
use quadrule_model::FontStyle;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use unicode_script::Script;

/// Fallback families in the order rule Q4 gives: Atkinson Hyperlegible Next first, then Noto Emoji,
/// then the rest of the bundled families. A test requires this list to name every bundled family.
pub const FALLBACK_ORDER: &[&str] = &[
    "Atkinson Hyperlegible Next",
    "Noto Emoji",
    "Atkinson Hyperlegible Mono",
    "Inter",
    "Literata",
    "Caveat",
];

/// Installed families open in the layout store at once; past this the store is rebuilt with only
/// the families asked for (a picker scrolling through hundreds of names must not hold them all).
const MAX_SYSTEM_FAMILIES_OPEN: usize = 48;

/// The marker in a family name that tells a copy at one optical size from its source font.
const OPSZ_MARK: &str = " [opsz ";

/// cosmic-text's fallback list with only bundled fonts in it.
struct BundledFallback {
    /// Installed families that must never serve as fallback for another font.
    forbidden: Vec<&'static str>,
}

impl Fallback for BundledFallback {
    fn common_fallback(&self) -> &[&'static str] {
        FALLBACK_ORDER
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        &self.forbidden
    }

    fn script_fallback(&self, _script: Script, _locale: &str) -> &[&'static str] {
        &[]
    }
}

/// A family name that lives for the whole run (cosmic-text's fallback lists want `&'static str`).
/// Each distinct name is kept once.
fn intern(name: &str) -> &'static str {
    static NAMES: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());
    let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(n) = names.get(name) {
        return n;
    }
    let leaked: &'static str = Box::leak(name.to_owned().into_boxed_str());
    names.insert(leaked);
    leaked
}

/// One family found on this computer.
#[derive(Clone, Debug, PartialEq)]
pub struct FamilyInfo {
    /// The family name, as a text box stores it.
    pub name: String,
    /// True for a font that ships inside quadrule.
    pub bundled: bool,
    /// True when the family has an italic (or oblique) face of its own.
    pub italic: bool,
}

/// The weights a family offers.
#[derive(Clone, Debug, PartialEq)]
pub enum Weights {
    /// A variable font's own `wght` range (both ends included).
    Range(u16, u16),
    /// The weights of the installed faces of a font that is not variable.
    Set(Vec<u16>),
}

impl Weights {
    /// The nearest weight the family offers.
    pub fn clamp(&self, w: u16) -> u16 {
        match self {
            Weights::Range(lo, hi) => w.clamp(*lo, *hi),
            Weights::Set(set) => set
                .iter()
                .copied()
                .min_by_key(|s| s.abs_diff(w))
                .unwrap_or(w),
        }
    }

    /// The lightest weight offered.
    pub fn min(&self) -> u16 {
        match self {
            Weights::Range(lo, _) => *lo,
            Weights::Set(s) => s.first().copied().unwrap_or(400),
        }
    }

    /// The heaviest weight offered.
    pub fn max(&self) -> u16 {
        match self {
            Weights::Range(_, hi) => *hi,
            Weights::Set(s) => s.last().copied().unwrap_or(400),
        }
    }
}

/// What a re-scan found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scan {
    /// Installed faces found (those that are not shadowed by a bundled family).
    pub faces: usize,
    /// Installed families found.
    pub families: usize,
    /// How long the scan took.
    pub elapsed: Duration,
    /// True when the list of families differs from before.
    pub changed: bool,
}

/// The installed faces, grouped by family.
struct Catalog {
    db: fontdb::Database,
    /// Family name (first name of its faces) → its faces; sorted by name, case-insensitively.
    families: Vec<(String, Vec<fontdb::ID>)>,
}

impl Catalog {
    fn empty() -> Self {
        Catalog {
            db: fontdb::Database::new(),
            families: Vec::new(),
        }
    }

    /// Scans this computer's fonts. A family whose name a bundled family has is left out: the
    /// bundled copy is what a file refers to (record Defaulted "System fonts").
    fn scan() -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        let bundled: BTreeSet<String> = BUNDLED.iter().map(|b| b.family.to_lowercase()).collect();
        let shadowed: Vec<fontdb::ID> = db
            .faces()
            .filter(|f| {
                f.families
                    .iter()
                    .any(|(n, _)| bundled.contains(&n.to_lowercase()))
            })
            .map(|f| f.id)
            .collect();
        for id in shadowed {
            db.remove_face(id);
        }
        let mut by_name: BTreeMap<String, Vec<fontdb::ID>> = BTreeMap::new();
        for f in db.faces() {
            let Some((name, _)) = f.families.first() else {
                continue;
            };
            // Names that start with a dot are the system's own hidden interface fonts.
            if name.is_empty() || name.starts_with('.') {
                continue;
            }
            by_name.entry(name.clone()).or_default().push(f.id);
        }
        let mut families: Vec<_> = by_name.into_iter().collect();
        families.sort_by_cached_key(|(n, _)| (n.to_lowercase(), n.clone()));
        Catalog { db, families }
    }

    fn faces(&self) -> usize {
        self.families.iter().map(|(_, ids)| ids.len()).sum()
    }

    fn find(&self, family: &str) -> Option<&Vec<fontdb::ID>> {
        self.families
            .iter()
            .find(|(n, _)| n == family)
            .map(|(_, ids)| ids)
    }

    fn names(&self) -> Vec<&str> {
        self.families.iter().map(|(n, _)| n.as_str()).collect()
    }
}

/// The fonts text can be laid out with.
pub struct FontRegistry {
    pub(crate) fs: FontSystem,
    catalog: Arc<Catalog>,
    /// Installed families open in `fs`.
    used: BTreeSet<String>,
    /// (bundled family, optical size) pairs that have a copy in `fs`.
    instances: BTreeSet<(String, u16)>,
    weights: BTreeMap<String, Option<Weights>>,
    /// True for a registry made by [`FontRegistry::with_system`]: it scans this computer's fonts.
    scans_system: bool,
}

fn binary(data: &'static [u8]) -> fontdb::Source {
    fontdb::Source::Binary(Arc::new(data))
}

/// The layout store: the bundled faces and the installed families in `used`.
fn build_store(catalog: &Catalog, used: &BTreeSet<String>) -> FontSystem {
    let mut db = fontdb::Database::new();
    for f in BUNDLED {
        db.load_font_source(binary(f.data));
    }
    let mut forbidden = Vec::new();
    for name in used {
        let Some(ids) = catalog.find(name) else {
            continue;
        };
        for id in ids {
            if let Some(face) = catalog.db.face(*id) {
                db.push_face_info(face.clone());
            }
        }
        forbidden.push(intern(name));
    }
    FontSystem::new_with_locale_and_db_and_fallback(
        "en-US".into(),
        db,
        BundledFallback { forbidden },
    )
}

/// (family, optical size) of a face name made by [`FontRegistry::ensure_instance`], or the name
/// itself with `None`.
pub(crate) fn split_instance(name: &str) -> (&str, Option<u16>) {
    if let Some(at) = name.find(OPSZ_MARK) {
        let n = name[at + OPSZ_MARK.len()..].trim_end_matches(']');
        if let Ok(n) = n.parse() {
            return (&name[..at], Some(n));
        }
    }
    (name, None)
}

/// The `opsz` default of each bundled family that has the axis, in whole points.
fn bundled_opsz_default(family: &str) -> Option<u16> {
    static DEFAULTS: OnceLock<BTreeMap<&'static str, u16>> = OnceLock::new();
    DEFAULTS
        .get_or_init(|| {
            BUNDLED
                .iter()
                .filter_map(|b| {
                    let (_, d, _) = opsz::opsz_range(b.data)?;
                    Some((b.family, d.round() as u16))
                })
                .collect()
        })
        .get(family)
        .copied()
}

/// What a text box's font choice comes to: the family to ask cosmic-text for, the weight after
/// clamping, and the style.
pub(crate) struct Resolved {
    pub family: String,
    pub weight: u16,
    pub style: cosmic_text::Style,
}

impl FontRegistry {
    /// Bundled fonts only, no installed ones: the same on every machine. Used by tests, by the
    /// golden pages and by exports that must not depend on this computer.
    pub fn bundled_only() -> Self {
        Self::with_catalog(Arc::new(Catalog::empty()), false)
    }

    /// The bundled fonts and the fonts installed on this computer (Linux, macOS and Windows), which
    /// the app uses. Installed fonts serve only the boxes that choose them; a missing character
    /// comes from the bundled fonts alone (rule Q4).
    pub fn with_system() -> Self {
        Self::with_catalog(Arc::new(Catalog::scan()), true)
    }

    fn with_catalog(catalog: Arc<Catalog>, scans_system: bool) -> Self {
        let used = BTreeSet::new();
        FontRegistry {
            fs: build_store(&catalog, &used),
            catalog,
            used,
            instances: BTreeSet::new(),
            weights: BTreeMap::new(),
            scans_system,
        }
    }

    /// Another registry over the same list of installed fonts, with a layout store of its own: for
    /// the pickers, whose previews would otherwise fill the page's store with every family shown.
    pub fn sibling(&self) -> Self {
        Self::with_catalog(self.catalog.clone(), self.scans_system)
    }

    /// Scans this computer's fonts again, in full (ADR-002 item 8: no change detection). Does
    /// nothing for a bundled-only registry.
    pub fn rescan(&mut self) -> Scan {
        let start = Instant::now();
        if !self.scans_system {
            return Scan {
                faces: 0,
                families: 0,
                elapsed: start.elapsed(),
                changed: false,
            };
        }
        let fresh = Catalog::scan();
        let changed = fresh.names() != self.catalog.names();
        let scan = Scan {
            faces: fresh.faces(),
            families: fresh.families.len(),
            elapsed: start.elapsed(),
            changed,
        };
        self.catalog = Arc::new(fresh);
        self.weights.clear();
        // Installed faces may have moved or gone: open the ones still there again.
        let keep: BTreeSet<String> = self
            .used
            .iter()
            .filter(|n| self.catalog.find(n).is_some())
            .cloned()
            .collect();
        self.used = keep;
        self.fs = build_store(&self.catalog, &self.used);
        self.instances.clear();
        scan
    }

    /// True when some family of this name can be laid out: a bundled one or an installed one.
    pub fn has_family(&self, family: &str) -> bool {
        BundledFont::for_family(family).is_some() || self.catalog.find(family).is_some()
    }

    /// The families, bundled ones first (in the order they ship), then the installed ones
    /// sorted by name.
    pub fn families(&self) -> Vec<FamilyInfo> {
        let mut out: Vec<FamilyInfo> = Vec::new();
        for b in BUNDLED {
            if !out.iter().any(|f| f.name == b.family) {
                out.push(FamilyInfo {
                    name: b.family.into(),
                    bundled: true,
                    italic: BUNDLED.iter().any(|x| x.family == b.family && x.italic),
                });
            }
        }
        for (name, ids) in &self.catalog.families {
            out.push(FamilyInfo {
                name: name.clone(),
                bundled: false,
                italic: ids.iter().any(|id| {
                    self.catalog
                        .db
                        .face(*id)
                        .is_some_and(|f| f.style != fontdb::Style::Normal)
                }),
            });
        }
        out
    }

    /// How many installed faces were found (those a bundled family does not shadow).
    pub fn system_face_count(&self) -> usize {
        self.catalog.faces()
    }

    /// The weights a family offers, or `None` for a family that cannot be laid out.
    pub fn weights_of(&mut self, family: &str) -> Option<Weights> {
        if let Some(w) = self.weights.get(family) {
            return w.clone();
        }
        let w = self.read_weights(family);
        self.weights.insert(family.into(), w.clone());
        w
    }

    fn read_weights(&self, family: &str) -> Option<Weights> {
        if BundledFont::for_family(family).is_some() {
            let faces: Vec<&BundledFont> = BUNDLED
                .iter()
                .filter(|b| b.family == family && !b.italic)
                .collect();
            if let Some((lo, _, hi)) = faces.iter().find_map(|b| opsz::wght_range(b.data)) {
                return Some(Weights::Range(lo.round() as u16, hi.round() as u16));
            }
            return Some(Weights::Set(vec![400]));
        }
        let ids = self.catalog.find(family)?;
        let mut range: Option<(u16, u16)> = None;
        let mut set = BTreeSet::new();
        for id in ids {
            if let Some(face) = self.catalog.db.face(*id) {
                set.insert(face.weight.0);
            }
            let r = self.catalog.db.with_face_data(*id, |data, index| {
                let f = FontRef::from_index(data, index).ok()?;
                let a = f.axes().get_by_tag(Tag::new(b"wght"))?;
                Some((a.min_value().round() as u16, a.max_value().round() as u16))
            });
            if let Some(Some((lo, hi))) = r {
                range = Some(match range {
                    Some((a, b)) => (a.min(lo), b.max(hi)),
                    None => (lo, hi),
                });
            }
        }
        Some(match range {
            Some((lo, hi)) => Weights::Range(lo, hi),
            None => Weights::Set(set.into_iter().collect()),
        })
    }

    /// True when the family has an italic face of its own.
    pub fn has_italic(&self, family: &str) -> bool {
        if BundledFont::for_family(family).is_some() {
            return BUNDLED.iter().any(|b| b.family == family && b.italic);
        }
        self.catalog.find(family).is_some_and(|ids| {
            ids.iter().any(|id| {
                self.catalog
                    .db
                    .face(*id)
                    .is_some_and(|f| f.style != fontdb::Style::Normal)
            })
        })
    }

    /// Opens the installed families among `names` in the layout store (one rebuild at most).
    pub(crate) fn ensure_system<'a>(&mut self, names: impl IntoIterator<Item = &'a str>) {
        let wanted: BTreeSet<String> = names
            .into_iter()
            .filter(|n| BundledFont::for_family(n).is_none() && self.catalog.find(n).is_some())
            .map(String::from)
            .collect();
        if wanted.is_subset(&self.used) {
            return;
        }
        if self.used.len() + wanted.len() > MAX_SYSTEM_FAMILIES_OPEN {
            self.used = wanted;
        } else {
            self.used.extend(wanted);
        }
        self.fs = build_store(&self.catalog, &self.used);
        self.instances.clear();
    }

    /// The name of the copy of a bundled family at optical size `opsz` points, made on first use.
    /// `None` when the family has no such axis or the copy cannot be made.
    fn ensure_instance(&mut self, family: &str, opsz: u16) -> Option<String> {
        let name = format!("{family}{OPSZ_MARK}{opsz}]");
        if self.instances.contains(&(family.into(), opsz)) {
            return Some(name);
        }
        let mut made = 0;
        for b in BUNDLED.iter().filter(|b| b.family == family) {
            let Some(bytes) = opsz::pin_opsz(b.data, f32::from(opsz)) else {
                continue;
            };
            let source = fontdb::Source::Binary(Arc::new(bytes));
            let ids = self.fs.db_mut().load_font_source(source);
            for id in ids {
                let Some(mut info) = self.fs.db().face(id).cloned() else {
                    continue;
                };
                self.fs.db_mut().remove_face(id);
                info.families = vec![(name.clone(), fontdb::Language::English_UnitedStates)];
                self.fs.db_mut().push_face_info(info);
                made += 1;
            }
        }
        if made == 0 {
            return None;
        }
        self.instances.insert((family.into(), opsz));
        Some(name)
    }

    /// Settles which cosmic-text family, weight and style lay a text box out.
    pub(crate) fn resolve(
        &mut self,
        family: &str,
        weight: u16,
        style: FontStyle,
        size_pt: f64,
    ) -> Option<Resolved> {
        let weights = self.weights_of(family)?;
        let mut name = family.to_string();
        if let Some(b) = BundledFont::for_family(family)
            && let Some((lo, def, hi)) = opsz::opsz_range(b.data)
        {
            let want = (size_pt as f32).round().clamp(lo, hi) as u16;
            if want != def.round() as u16
                && let Some(n) = self.ensure_instance(family, want)
            {
                name = n;
            }
        }
        Some(Resolved {
            family: name,
            weight: weights.clamp(weight),
            style: match style {
                FontStyle::Normal => cosmic_text::Style::Normal,
                FontStyle::Italic => cosmic_text::Style::Italic,
            },
        })
    }

    /// The family, italic flag and optical size of a loaded face, for the key of its glyph table.
    pub(crate) fn face_identity(&self, id: fontdb::ID) -> (String, bool, u16) {
        let Some(face) = self.fs.db().face(id) else {
            return (String::new(), false, 0);
        };
        let raw = face.families.first().map(|(n, _)| n.as_str()).unwrap_or("");
        let (base, inst) = split_instance(raw);
        let opsz = inst.or_else(|| bundled_opsz_default(base)).unwrap_or(0);
        (base.to_string(), face.style != fontdb::Style::Normal, opsz)
    }
}
