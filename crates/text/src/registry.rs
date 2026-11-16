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
//! re-scanned when the window regains focus, ADR-002 item 8), the fonts a document carries inside
//! its file (rung 12a), and the rule for a character the chosen font lacks: it comes from the bundled
//! fonts, then, on a registry made by [`FontRegistry::with_system`], from an installed font
//! (ADR-002 item 6; rung 12a), which the layout reports ([`crate::Laid::fallback`]) so the linter can
//! warn. A registry made by [`FontRegistry::bundled_only`] never looks further than the bundled fonts.
//!
//! Two stores are kept apart. The **catalog** lists the installed faces for the app's pickers and
//! is shared by copies of a registry. The **layout store** is the cosmic-text `FontSystem` that
//! text is actually shaped with: it holds the bundled faces, a copy of each bundled face at the
//! optical sizes asked for ([`crate::opsz`]), and only those installed families some text box
//! uses. The families of the last kind are *forbidden as fallback* in its [`Fallback`] list, so
//! they serve only the boxes that chose them, except the few opened as fallback for a character no
//! bundled font has.

use crate::opsz;
use crate::signature;
use crate::{BUNDLED, BundledFont};
use cosmic_text::skrifa::{FontRef, MetadataProvider, Tag};
use cosmic_text::{Fallback, FontSystem, fontdb};
use quadrule_model::{FontSignature, FontStyle};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};
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
    /// True for a font that comes from the open file only (embedded in it, rung 12a), and is not
    /// installed on this computer.
    pub embedded: bool,
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

/// How many background scans have been started in this process (for a test that a second focus
/// during a scan starts none).
static SCANS_STARTED: AtomicUsize = AtomicUsize::new(0);

/// How many background scans of the installed fonts this process has started.
pub fn scans_started() -> usize {
    SCANS_STARTED.load(Ordering::SeqCst)
}

/// A scan of the installed fonts running on a thread of its own (rung 11b).
pub struct PendingScan {
    rx: mpsc::Receiver<Finished>,
}

impl PendingScan {
    /// The result, once the scan is done; `None` while it still runs. Never waits.
    pub fn finished(&self) -> Option<Finished> {
        self.rx.try_recv().ok()
    }
}

/// A finished scan of the installed fonts, not yet taken in ([`FontRegistry::adopt`]).
pub struct Finished {
    catalog: Catalog,
    /// How long the scan took, on its own thread.
    elapsed: Duration,
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

    /// A catalog of the given font files, each filed under the family name given (whatever the
    /// file calls itself): the fonts a document carries, and the stand-ins tests use.
    fn from_files(files: &[(String, Arc<Vec<u8>>)]) -> Self {
        let mut db = fontdb::Database::new();
        let mut by_name: BTreeMap<String, Vec<fontdb::ID>> = BTreeMap::new();
        for (name, bytes) in files {
            for id in db.load_font_source(fontdb::Source::Binary(bytes.clone())) {
                let Some(mut info) = db.face(id).cloned() else {
                    continue;
                };
                db.remove_face(id);
                info.families = vec![(name.clone(), fontdb::Language::English_UnitedStates)];
                let new_id = db.push_face_info(info);
                by_name.entry(name.clone()).or_default().push(new_id);
            }
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
    /// The fonts the open file carries inside it (rung 12a); empty for most documents. They serve
    /// the boxes that choose them, and only in this document.
    embedded: Arc<Catalog>,
    /// Installed families open in `fs`.
    used: BTreeSet<String>,
    /// Installed families opened because a character no bundled font has is in some text: they may
    /// serve as fallback for any box (ADR-002 item 6).
    fallback: BTreeSet<String>,
    /// For each character looked for in the installed fonts, the family chosen to draw it.
    fallback_for: BTreeMap<char, Option<String>>,
    /// Measured signatures by family (ADR-001 item 6), made on first use.
    signatures: BTreeMap<String, Option<FontSignature>>,
    /// (bundled family, optical size) pairs that have a copy in `fs`.
    instances: BTreeSet<(String, u16)>,
    weights: BTreeMap<String, Option<Weights>>,
    /// True for a registry made by [`FontRegistry::with_system`]: it scans this computer's fonts.
    scans_system: bool,
}

fn binary(data: &'static [u8]) -> fontdb::Source {
    fontdb::Source::Binary(Arc::new(data))
}

/// The layout store: the bundled faces, the installed or embedded families in `used` (which serve
/// only the boxes that chose them) and the installed families in `fallback` (which may serve any).
fn build_store(
    catalog: &Catalog,
    embedded: &Catalog,
    used: &BTreeSet<String>,
    fallback: &BTreeSet<String>,
) -> FontSystem {
    let mut db = fontdb::Database::new();
    for f in BUNDLED {
        db.load_font_source(binary(f.data));
    }
    let mut forbidden = Vec::new();
    let mut pushed: BTreeSet<&str> = BTreeSet::new();
    for name in used.union(fallback) {
        let Some((cat, ids)) = lookup(catalog, embedded, name) else {
            continue;
        };
        for id in ids {
            if let Some(face) = cat.db.face(*id) {
                db.push_face_info(face.clone());
            }
        }
        pushed.insert(name);
        // Installed fonts serve only the boxes that chose them; the file's own fonts are part of the
        // document and may serve any box in it.
        if !fallback.contains(name) && catalog.find(name).is_some() {
            forbidden.push(intern(name));
        }
    }
    // The file's own fonts are always open, for their boxes and as fallback.
    for (name, ids) in &embedded.families {
        if pushed.contains(name.as_str()) || catalog.find(name).is_some() {
            continue;
        }
        for id in ids {
            if let Some(face) = embedded.db.face(*id) {
                db.push_face_info(face.clone());
            }
        }
    }
    FontSystem::new_with_locale_and_db_and_fallback(
        "en-US".into(),
        db,
        BundledFallback { forbidden },
    )
}

/// The faces of a family on this computer: the installed ones, else the ones the file carries.
fn lookup<'a>(
    catalog: &'a Catalog,
    embedded: &'a Catalog,
    family: &str,
) -> Option<(&'a Catalog, &'a Vec<fontdb::ID>)> {
    catalog
        .find(family)
        .map(|ids| (catalog, ids))
        .or_else(|| embedded.find(family).map(|ids| (embedded, ids)))
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

    /// A registry whose "installed" fonts are the given files, each under the family name given
    /// (whatever the file calls itself), for tests that must not depend on this computer's fonts.
    /// It looks in them for fallback as a registry made by [`FontRegistry::with_system`] does; a
    /// re-scan would replace them with the real ones.
    #[doc(hidden)]
    pub fn with_installed_for_test(files: &[(String, Arc<Vec<u8>>)]) -> Self {
        Self::with_catalog(Arc::new(Catalog::from_files(files)), true)
    }

    fn with_catalog(catalog: Arc<Catalog>, scans_system: bool) -> Self {
        let used = BTreeSet::new();
        let embedded = Arc::new(Catalog::empty());
        let fallback = BTreeSet::new();
        FontRegistry {
            fs: build_store(&catalog, &embedded, &used, &fallback),
            catalog,
            embedded,
            fallback,
            fallback_for: BTreeMap::new(),
            signatures: BTreeMap::new(),
            used,
            instances: BTreeSet::new(),
            weights: BTreeMap::new(),
            scans_system,
        }
    }

    /// Another registry over the same list of installed fonts, with a layout store of its own: for
    /// the pickers, whose previews would otherwise fill the page's store with every family shown.
    pub fn sibling(&self) -> Self {
        let mut s = Self::with_catalog(self.catalog.clone(), self.scans_system);
        s.embedded = self.embedded.clone();
        s.fs = build_store(&s.catalog, &s.embedded, &s.used, &s.fallback);
        s
    }

    /// Takes in the fonts a file carries (rung 12a): `(family, file bytes)` for each, replacing the
    /// ones of the document before. They are this document's only: never installed on the
    /// computer, and gone when the next document sets its own. A family this computer has
    /// installed is used from here as before; the embedded copy serves when it is not. Returns true
    /// when the set of families changed, so pages need laying out again.
    pub fn set_embedded(&mut self, files: &[(String, Arc<Vec<u8>>)]) -> bool {
        let fresh = Catalog::from_files(files);
        let changed = fresh.names() != self.embedded.names();
        self.embedded = Arc::new(fresh);
        self.weights.clear();
        self.signatures.clear();
        self.used
            .retain(|n| lookup(&self.catalog, &self.embedded, n).is_some());
        self.rebuild();
        changed
    }

    /// Makes the layout store again from what is in use.
    fn rebuild(&mut self) {
        self.fs = build_store(&self.catalog, &self.embedded, &self.used, &self.fallback);
        self.instances.clear();
    }

    /// True when `family` comes from the open file's embedded fonts only: not bundled, not installed.
    pub fn is_embedded_only(&self, family: &str) -> bool {
        BundledFont::for_family(family).is_none()
            && self.catalog.find(family).is_none()
            && self.embedded.find(family).is_some()
    }

    /// The font files an installed or embedded family needs for faces at the given (weight, italic)
    /// pairs, each once, whole: what Save writes into the file. Errors with the reason when a file
    /// may not be embedded ([`crate::embed`]). An empty list for a bundled family or one not here.
    pub fn font_files(
        &self,
        family: &str,
        wanted: &[(u16, bool)],
    ) -> Result<Vec<FontFileData>, crate::embed::NoEmbed> {
        if BundledFont::for_family(family).is_some() {
            return Ok(Vec::new());
        }
        let Some((cat, ids)) = lookup(&self.catalog, &self.embedded, family) else {
            return Ok(Vec::new());
        };
        let mut out: Vec<FontFileData> = Vec::new();
        for (weight, italic) in wanted {
            let q = fontdb::Query {
                families: &[fontdb::Family::Name(family)],
                weight: fontdb::Weight(*weight),
                style: if *italic {
                    fontdb::Style::Italic
                } else {
                    fontdb::Style::Normal
                },
                ..Default::default()
            };
            let Some(id) = cat.db.query(&q).filter(|id| ids.contains(id)) else {
                continue;
            };
            let Some((bytes, face_weight, face_italic)) = cat
                .db
                .with_face_data(id, |data, _| data.to_vec())
                .zip(
                    cat.db
                        .face(id)
                        .map(|f| (f.weight.0, f.style != fontdb::Style::Normal)),
                )
                .map(|(b, (w, i))| (b, w, i))
            else {
                continue;
            };
            crate::embed::verdict(&bytes)?;
            if !out
                .iter()
                .any(|f| f.bytes == bytes && f.italic == face_italic)
            {
                out.push(FontFileData {
                    bytes,
                    weight: face_weight,
                    italic: face_italic,
                });
            }
        }
        Ok(out)
    }

    /// Whether the family may be saved inside a file, from the licence bits of the first face of it
    /// ([`crate::embed`]); the family's other faces carry the same licence. `Ok` for a family that is
    /// not here (nothing to embed).
    pub fn embed_verdict(&self, family: &str) -> Result<(), crate::embed::NoEmbed> {
        if BundledFont::for_family(family).is_some() {
            return Ok(());
        }
        let Some((cat, ids)) = lookup(&self.catalog, &self.embedded, family) else {
            return Ok(());
        };
        let mut verdict = Ok(());
        for id in ids {
            if let Some(Err(e)) = cat
                .db
                .with_face_data(*id, |data, _| crate::embed::verdict(data))
            {
                verdict = Err(e);
                break;
            }
        }
        verdict
    }

    /// True when `family` is installed on this computer (and not shadowed by a bundled family).
    pub fn is_installed(&self, family: &str) -> bool {
        BundledFont::for_family(family).is_none() && self.catalog.find(family).is_some()
    }

    /// Scans this computer's fonts again, in full (ADR-002 item 8: no change detection), and waits
    /// for it. Does nothing for a bundled-only registry. The window uses
    /// [`FontRegistry::scan_in_background`] instead, so focus never waits.
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
        let fresh = Finished {
            catalog: Catalog::scan(),
            elapsed: start.elapsed(),
        };
        self.adopt(fresh)
    }

    /// Starts a full scan of this computer's fonts on a thread of its own and returns at once
    /// (rung 11b; the record's Amendments, "background re-scan"). `None` for a bundled-only
    /// registry, which has nothing to scan. Hand the result to [`FontRegistry::adopt`] when
    /// [`PendingScan::finished`] gives it.
    pub fn scan_in_background(&self) -> Option<PendingScan> {
        if !self.scans_system {
            return None;
        }
        SCANS_STARTED.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let start = Instant::now();
            let catalog = Catalog::scan();
            let _ = tx.send(Finished {
                catalog,
                elapsed: start.elapsed(),
            });
        });
        Some(PendingScan { rx })
    }

    /// Takes in a finished scan: the list of installed families swaps to the new one, and the
    /// layout store is made again with the families in use.
    pub fn adopt(&mut self, scanned: Finished) -> Scan {
        let Finished {
            catalog: fresh,
            elapsed,
        } = scanned;
        let changed = fresh.names() != self.catalog.names();
        let scan = Scan {
            faces: fresh.faces(),
            families: fresh.families.len(),
            elapsed,
            changed,
        };
        self.catalog = Arc::new(fresh);
        self.weights.clear();
        self.signatures.clear();
        self.fallback_for.clear();
        // Installed faces may have moved or gone: open the ones still there again.
        let keep: BTreeSet<String> = self
            .used
            .iter()
            .filter(|n| lookup(&self.catalog, &self.embedded, n).is_some())
            .cloned()
            .collect();
        self.used = keep;
        self.fallback.clear();
        self.rebuild();
        scan
    }

    /// True when some family of this name can be laid out: a bundled one or an installed one.
    pub fn has_family(&self, family: &str) -> bool {
        BundledFont::for_family(family).is_some()
            || lookup(&self.catalog, &self.embedded, family).is_some()
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
                    embedded: false,
                    italic: BUNDLED.iter().any(|x| x.family == b.family && x.italic),
                });
            }
        }
        for (name, ids) in &self.catalog.families {
            out.push(FamilyInfo {
                name: name.clone(),
                bundled: false,
                embedded: false,
                italic: ids.iter().any(|id| {
                    self.catalog
                        .db
                        .face(*id)
                        .is_some_and(|f| f.style != fontdb::Style::Normal)
                }),
            });
        }
        // The file's own fonts, after the installed ones, for a family the computer lacks.
        for (name, ids) in &self.embedded.families {
            if self.catalog.find(name).is_some() {
                continue;
            }
            out.push(FamilyInfo {
                name: name.clone(),
                bundled: false,
                embedded: true,
                italic: ids.iter().any(|id| {
                    self.embedded
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
        let (cat, ids) = lookup(&self.catalog, &self.embedded, family)?;
        let mut range: Option<(u16, u16)> = None;
        let mut set = BTreeSet::new();
        for id in ids {
            if let Some(face) = cat.db.face(*id) {
                set.insert(face.weight.0);
            }
            let r = cat.db.with_face_data(*id, |data, index| {
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
        lookup(&self.catalog, &self.embedded, family).is_some_and(|(cat, ids)| {
            ids.iter().any(|id| {
                cat.db
                    .face(*id)
                    .is_some_and(|f| f.style != fontdb::Style::Normal)
            })
        })
    }

    /// Opens the installed families among `names` in the layout store, one rebuild at most, so
    /// that laying out text in them afterwards does not stop to do it name by name. Laying text
    /// out does this itself; a picker showing many families calls it with the ones in view.
    pub fn ensure_open<'a>(&mut self, names: impl IntoIterator<Item = &'a str>) {
        let wanted: BTreeSet<String> = names
            .into_iter()
            .filter(|n| {
                BundledFont::for_family(n).is_none()
                    && lookup(&self.catalog, &self.embedded, n).is_some()
            })
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
        self.rebuild();
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

/// A whole font file read from an installed or embedded family, to be saved in a document.
#[derive(Clone, Debug, PartialEq)]
pub struct FontFileData {
    /// The file's bytes.
    pub bytes: Vec<u8>,
    /// The weight of the face used (its default, for a variable font).
    pub weight: u16,
    /// True when the face is italic or oblique.
    pub italic: bool,
}

/// What a missing font was like, to find a substitute for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Want {
    /// The family that is missing.
    pub family: String,
    /// Its signature, as the file saved it; `None` for a file from before format 0.13, which gives the
    /// family name only (the match then goes by the class its name hints at).
    pub signature: Option<FontSignature>,
}

/// One candidate substitute.
#[derive(Clone, Debug, PartialEq)]
pub struct Match {
    /// The family.
    pub family: String,
    /// True for a font that ships inside quadrule.
    pub bundled: bool,
    /// How far it is from what was wanted (lower is closer); 0 for the very name.
    pub distance: u32,
}

impl FontRegistry {
    /// The measured signature of a family on this computer (ADR-001 item 6): its upright face nearest
    /// regular, at regular weight. `None` for a family not here or one with no letters to measure.
    pub fn signature_of(&mut self, family: &str) -> Option<FontSignature> {
        if let Some(s) = self.signatures.get(family) {
            return s.clone();
        }
        let sig = self.measure_family(family);
        self.signatures.insert(family.into(), sig.clone());
        sig
    }

    fn measure_family(&self, family: &str) -> Option<FontSignature> {
        let sig = if let Some(b) = BundledFont::for_family(family) {
            signature::measure(b.data, 0, Some(400), family)?
        } else {
            let (cat, ids) = lookup(&self.catalog, &self.embedded, family)?;
            // The upright face whose weight is nearest 400.
            let id = ids
                .iter()
                .filter_map(|id| cat.db.face(*id))
                .min_by_key(|f| {
                    (
                        u8::from(f.style != fontdb::Style::Normal),
                        f.weight.0.abs_diff(400),
                    )
                })?
                .id;
            cat.db
                .with_face_data(id, |data, index| {
                    signature::measure(data, index, Some(400), family)
                })
                .flatten()?
        };
        Some(sig)
    }

    /// The closest families here to a missing font, best first (at most `n`): an exact name (any
    /// capitals) wins; otherwise the lowest distance by signature (ADR-002 item 7). Every family on
    /// this computer is a candidate, bundled ones included, except fonts with nothing to measure
    /// (emoji, symbols).
    pub fn best_matches(&mut self, want: &Want, n: usize) -> Vec<Match> {
        let names: Vec<(String, bool)> = self
            .families()
            .into_iter()
            .map(|f| (f.name, f.bundled))
            .collect();
        let target = want.signature.clone().unwrap_or_else(|| FontSignature {
            class: signature::name_class(&want.family).unwrap_or(quadrule_model::FontClass::Sans),
            weight: 400,
            width: 100,
            italic: false,
            avg_width: 0,
            x_height: 0,
            mono: false,
            extra: Default::default(),
        });
        let mut out = Vec::new();
        for (name, bundled) in names {
            if name == want.family {
                continue;
            }
            let exact = name.eq_ignore_ascii_case(&want.family);
            let Some(sig) = self.signature_of(&name) else {
                if exact {
                    out.push(Match {
                        family: name,
                        bundled,
                        distance: 0,
                    });
                }
                continue;
            };
            let distance = if exact {
                0
            } else {
                signature::distance(&target, &sig)
            };
            out.push(Match {
                family: name,
                bundled,
                distance,
            });
        }
        out.sort_by(|a, b| (a.distance, &a.family).cmp(&(b.distance, &b.family)));
        out.truncate(n);
        out
    }

    /// Looks in the installed fonts for the characters `chars`, which the chosen font and the bundled
    /// ones lack (ADR-002 item 6), and opens the families that have them as fallback. Returns true
    /// when the layout store changed, so the text should be laid out again. Does nothing for a
    /// registry made by [`FontRegistry::bundled_only`].
    pub(crate) fn open_fallbacks(&mut self, chars: &BTreeSet<char>) -> bool {
        if !self.scans_system {
            return false;
        }
        let todo: BTreeSet<char> = chars
            .iter()
            .copied()
            .filter(|c| !c.is_whitespace() && !self.fallback_for.contains_key(c))
            .collect();
        if !todo.is_empty() {
            let found = self.find_covering(&todo);
            for c in todo {
                self.fallback_for.insert(c, found.get(&c).cloned());
            }
        }
        let wanted: BTreeSet<String> = chars
            .iter()
            .filter_map(|c| self.fallback_for.get(c).cloned().flatten())
            .collect();
        if wanted.is_subset(&self.fallback) {
            return false;
        }
        self.fallback.extend(wanted);
        self.rebuild();
        true
    }

    /// For each of `chars`, the installed family to draw it with: the family that covers the most of
    /// those still unplaced, over and over, ties by name.
    fn find_covering(&self, chars: &BTreeSet<char>) -> BTreeMap<char, String> {
        let mut covers: Vec<(String, BTreeSet<char>)> = Vec::new();
        for (name, ids) in &self.catalog.families {
            let mut have = BTreeSet::new();
            // One face of the family is enough to say what it covers.
            if let Some(id) = ids.first() {
                self.catalog.db.with_face_data(*id, |data, index| {
                    if let Ok(f) = FontRef::from_index(data, index) {
                        let map = f.charmap();
                        for c in chars {
                            if map.map(*c).is_some_and(|g| g.to_u32() != 0) {
                                have.insert(*c);
                            }
                        }
                    }
                });
            }
            if !have.is_empty() {
                covers.push((name.clone(), have));
            }
        }
        let mut left = chars.clone();
        let mut out = BTreeMap::new();
        while !left.is_empty() {
            let best = covers
                .iter()
                .map(|(n, have)| (n, have.intersection(&left).count()))
                .filter(|(_, k)| *k > 0)
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)));
            let Some((name, _)) = best else { break };
            let have = covers
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, h)| h.clone())
                .unwrap_or_default();
            for c in have.intersection(&left.clone()) {
                out.insert(*c, name.clone());
                left.remove(c);
            }
        }
        out
    }

    /// Opens fallback for the letters of `text` and returns the families now open as fallback, for
    /// tests.
    #[doc(hidden)]
    pub fn open_fallbacks_for_test(&mut self, text: &str) -> Vec<String> {
        let chars: BTreeSet<char> = text.chars().collect();
        self.open_fallbacks(&chars);
        self.fallback.iter().cloned().collect()
    }

    /// True when the registry may draw a letter from an installed font that no bundled font has.
    pub fn falls_back_to_installed(&self) -> bool {
        self.scans_system
    }
}
