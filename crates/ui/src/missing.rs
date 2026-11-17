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

//! Missing fonts in the viewer, with no screen in it (rung 12a, docs/bbq/2026-11-16-rung-12-missing-fonts.md):
//! which fonts the open document names that this computer lacks, the text boxes that are read-only
//! because of it (Q6), the choices the Missing fonts dialog offers (Q4, Q5), the Save sheet's facts
//! (Q7), what Save writes for fonts (Q3), and where to look for fonts (Locate fonts…). The widgets are
//! in [`crate::missing_ui`]; everything here is tested headless.

use crate::viewer::{MissingFont, Notice, Viewer};
use quadrule_format::{FontFile, FontRef, QuadFile, font_path, sha256_hex};
use quadrule_i18n::Message;
use quadrule_lint::InstalledFont;
use quadrule_model::edit::{Command, EditError};
use quadrule_model::{ObjectKind, Uuid};
use quadrule_profiles::active::lookup_for;
use quadrule_text::{BundledFont, Match, Want};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

impl Viewer {
    /// Works out again which fonts the document names that are not here, and makes their text boxes
    /// read-only in the editing model. Called after every change and every re-scan, and when a
    /// document opens.
    pub(crate) fn refresh_font_state(&mut self) {
        let Viewer { open, fonts, .. } = self;
        let Some(o) = open.as_mut() else {
            return;
        };
        let mut by_family: BTreeMap<String, Vec<Uuid>> = BTreeMap::new();
        let mut known: BTreeMap<String, bool> = BTreeMap::new();
        for ob in o.doc.objects() {
            if let ObjectKind::Text(t) = &ob.kind {
                let here = *known
                    .entry(t.family.clone())
                    .or_insert_with(|| fonts.has_family(&t.family));
                if !here {
                    by_family.entry(t.family.clone()).or_default().push(ob.id);
                }
            }
        }
        let keep = &o.doc.page.fonts;
        o.missing = by_family
            .into_iter()
            .map(|(family, boxes)| MissingFont {
                kept: keep.keeps(&family),
                family,
                boxes,
            })
            .collect();
        let ro: BTreeSet<Uuid> = o
            .missing
            .iter()
            .flat_map(|m| m.boxes.iter().copied())
            .collect();
        o.history.set_read_only_text(ro);
    }

    /// The fonts the document names that this computer lacks, with the boxes that use each.
    pub fn missing_fonts(&self) -> &[MissingFont] {
        self.open.as_ref().map_or(&[], |o| &o.missing)
    }

    /// Whether the text box `id` is drawn from saved outlines because its font is missing: its text
    /// cannot be changed (Q6).
    pub fn is_read_only_text(&self, id: Uuid) -> bool {
        self.open
            .as_ref()
            .is_some_and(|o| o.history.is_read_only_text(id))
    }

    /// The fonts the bar names: those missing and not kept as saved outlines, unless the bar was
    /// closed.
    pub fn bar_fonts(&self) -> Vec<&MissingFont> {
        match &self.open {
            Some(o) if !o.bar_closed => o.missing.iter().filter(|m| !m.kept).collect(),
            _ => Vec::new(),
        }
    }

    /// Closes the bar with its ×; it stays closed until the document is opened again.
    pub fn close_font_bar(&mut self) {
        if let Some(o) = &mut self.open {
            o.bar_closed = true;
        }
    }

    /// Adds a character to the text of box `id`, as a test of whether it can be edited.
    #[doc(hidden)]
    pub fn edit_selected_for_test(&mut self, id: Uuid) -> Result<(), EditError> {
        let Some(mut o) = self.doc().and_then(|d| d.object(id)).cloned() else {
            return Ok(());
        };
        if let ObjectKind::Text(t) = &mut o.kind {
            t.content.push('x');
        }
        self.edit(Command::SetObject(o), None)
    }

    /// Whether some family of this name can be laid out here, for tests.
    #[doc(hidden)]
    pub fn fonts_has_family_for_test(&self, family: &str) -> bool {
        self.fonts.has_family(family)
    }

    /// Makes the given files this computer's "installed" fonts, for tests.
    #[doc(hidden)]
    pub fn use_installed_for_test(&mut self, files: &[(String, std::sync::Arc<Vec<u8>>)]) {
        self.fonts = quadrule_text::FontRegistry::with_installed_for_test(files);
        self.invalidate_pages();
    }

    /// The closest families here to a missing font, best first, at most five (Defaulted "The best
    /// match"): by the signature the file saved for it, or by its name where the file has none.
    pub fn font_matches(&mut self, family: &str) -> Vec<Match> {
        let signature = self
            .open
            .as_ref()
            .and_then(|o| o.manifest.as_ref())
            .and_then(|m| m.fonts.iter().find(|f| f.family == family))
            .and_then(|f| f.signature.clone());
        let want = Want {
            family: family.into(),
            signature,
        };
        self.fonts.best_matches(&want, 5)
    }

    /// Rewrites the family of every text box that uses a font in `subs` (missing family → the font
    /// to use instead), and, for the families in `keep`, remembers for this document that they stay
    /// saved outlines; all as one undo step (Q4, Q5). The substitute keeps each box's weight and
    /// style; the nearest face the new family offers draws them. The one command that lifts the
    /// read-only rule.
    pub fn apply_font_choices(
        &mut self,
        subs: &[(String, String)],
        keep: &[String],
    ) -> Result<(), EditError> {
        self.end_typing();
        let Some(open) = &mut self.open else {
            return Ok(());
        };
        let mut cmds = Vec::new();
        for (from, to) in subs {
            if from == to {
                continue;
            }
            for id in open
                .missing
                .iter()
                .filter(|m| &m.family == from)
                .flat_map(|m| m.boxes.iter())
            {
                if let Some(o) = open.doc.object(*id) {
                    let mut o = o.clone();
                    if let ObjectKind::Text(t) = &mut o.kind {
                        t.family.clone_from(to);
                    }
                    cmds.push(Command::SetObject(o));
                }
            }
        }
        let mut page = open.doc.page.clone();
        for family in keep {
            if !page.fonts.keeps(family) {
                page.fonts.keep_outlines.push(family.clone());
            }
        }
        // A substituted family is no longer kept.
        page.fonts
            .keep_outlines
            .retain(|f| !subs.iter().any(|(from, to)| from == f && from != to));
        if page != open.doc.page {
            cmds.push(Command::SetPage(page));
        }
        if cmds.is_empty() {
            return Ok(());
        }
        match open
            .history
            .apply_lifting_read_only(&mut open.doc, Command::Batch(cmds), None)
        {
            Ok(()) => {
                self.changed_public();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Sets "Embed fonts" for the document (the Page tab's tick; one undo step).
    pub fn set_embed_fonts(&mut self, on: bool) -> Result<(), EditError> {
        self.set_font_settings(|f| f.embed = on)
    }

    /// Sets "Don't warn again for this document" (the Save sheet's tick; one undo step).
    pub fn set_dont_warn(&mut self, on: bool) -> Result<(), EditError> {
        self.set_font_settings(|f| f.dont_warn = on)
    }

    fn set_font_settings(
        &mut self,
        f: impl Fn(&mut quadrule_model::FontSettings),
    ) -> Result<(), EditError> {
        let Some(open) = &self.open else {
            return Ok(());
        };
        let mut page = open.doc.page.clone();
        f(&mut page.fonts);
        if page == open.doc.page {
            return Ok(());
        }
        self.edit(Command::SetPage(page), None)
    }

    /// The installed fonts the page uses, each with whether the file carries it: what the Save sheet
    /// lists (Q7). Empty when the sheet is not wanted: nothing installed is used, or the document
    /// says not to warn again.
    pub fn font_sheet_fonts(&mut self) -> Vec<InstalledFont> {
        let Some(open) = &self.open else {
            return Vec::new();
        };
        if open.doc.page.fonts.dont_warn {
            return Vec::new();
        }
        let primary = open.doc.primary_profile_id.clone();
        match self.page_for(&primary) {
            Ok(p) => p.font_facts.installed.clone(),
            Err(_) => Vec::new(),
        }
    }

    /// The folder the user adds fonts to on this computer, made if it is not there (Linux and
    /// macOS); `None` on Windows, where the Fonts settings page is opened instead.
    pub fn user_font_folder() -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        if cfg!(target_os = "macos") {
            Some(home.join("Library/Fonts"))
        } else if cfg!(windows) {
            None
        } else {
            Some(home.join(".local/share/fonts"))
        }
    }

    /// Opens this computer's user font folder in the file manager (Linux `~/.local/share/fonts`,
    /// made if absent; macOS `~/Library/Fonts`; Windows the Fonts settings page). The re-scan on
    /// focus then picks up what was added.
    pub fn locate_fonts(&mut self) -> Result<(), String> {
        let (program, args): (&str, Vec<String>) = if cfg!(windows) {
            (
                "cmd",
                vec!["/C".into(), "start".into(), "ms-settings:fonts".into()],
            )
        } else {
            let dir = Self::user_font_folder().ok_or("no home folder")?;
            std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            let opener = if cfg!(target_os = "macos") {
                "open"
            } else {
                "xdg-open"
            };
            (opener, vec![dir.display().to_string()])
        };
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .map(|_| ())
            .map_err(|e| {
                self.notice = Some(Notice {
                    message: Message::new("notice-locate-failed").text("error", e.to_string()),
                    error: true,
                });
                e.to_string()
            })
    }

    /// The families the document uses and the faces of each it asks for, for what Save writes about
    /// fonts: every text box, any depth.
    fn faces_used(&self) -> BTreeMap<String, BTreeSet<(u16, bool)>> {
        let mut out: BTreeMap<String, BTreeSet<(u16, bool)>> = BTreeMap::new();
        if let Some(o) = &self.open {
            for ob in o.doc.objects() {
                if let ObjectKind::Text(t) = &ob.kind {
                    out.entry(t.family.clone())
                        .or_default()
                        .insert((t.weight, t.style == quadrule_model::FontStyle::Italic));
                }
            }
        }
        out
    }

    /// The file's bytes as Save would write them (ADR-001 items 2, 6 and 7; rung 12a). Text is laid
    /// out for every profile; a box whose font is missing keeps the outlines the file saved for it
    /// (Q6 makes its text read-only, so they cannot be stale). For every font the document uses, the
    /// manifest holds its measured signature, and, with "Embed fonts" on, the whole file of each
    /// installed face whose licence allows it.
    pub fn file_bytes(&mut self) -> Result<Vec<u8>, String> {
        self.end_typing();
        let faces = self.faces_used();
        let Viewer { open, fonts, .. } = self;
        let open = open.as_mut().ok_or("nothing is open")?;
        if let Some(r) = &open.read_only {
            return Err(format!("{r:?}"));
        }
        let doc = &open.doc;
        let primary =
            lookup_for(&doc.primary_profile_id, doc.page.orientation).map_err(|e| e.to_string())?;
        let mut outlines = Vec::new();
        let mut fallback: BTreeSet<String> = BTreeSet::new();
        for id in doc.profile_ids() {
            let target = lookup_for(id, doc.page.orientation).map_err(|e| e.to_string())?;
            let (laid, _) =
                quadrule_layout::lay_out_text_saved(doc, &primary, &target, fonts, &open.saved)
                    .map_err(|e| e.to_string())?;
            fallback.extend(laid.fallback.iter().cloned());
            outlines.push(laid.outlines);
        }
        // The fonts: one entry per family used, bundled ones included (ADR-001 item 6).
        let mut refs: Vec<FontRef> = Vec::new();
        let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        let mut wanted = faces;
        for f in fallback {
            wanted.entry(f).or_default().insert((400, false));
        }
        for (family, want) in wanted {
            let want: Vec<(u16, bool)> = want.into_iter().collect();
            let mut r = match BundledFont::for_family(&family) {
                Some(b) => FontRef::new(b.family, Some(b.id), b.version()),
                None => FontRef::new(&family, None, None),
            };
            if fonts.has_family(&family) {
                r.signature = fonts.signature_of(&family);
                if doc.page.fonts.embed
                    && let Ok(list) = fonts.font_files(&family, &want)
                {
                    for f in list {
                        let path = font_path(&f.bytes);
                        r.files.push(FontFile {
                            path: path.clone(),
                            sha256: sha256_hex(&f.bytes),
                            weight: f.weight,
                            italic: f.italic,
                            extra: Default::default(),
                        });
                        files.insert(path, f.bytes);
                    }
                }
            } else if let Some(old) = open
                .manifest
                .as_ref()
                .and_then(|m| m.fonts.iter().find(|f| f.family == family))
            {
                // Not here: what the file knew about it stays, so a computer that has it can match.
                r.signature = old.signature.clone();
                r.extra = old.extra.clone();
            }
            refs.push(r);
        }
        let mut file = QuadFile::new(doc.clone(), outlines, refs);
        file.font_files = files;
        if let Some(m) = &open.manifest {
            // Keep what the file's manifest carried (extras, a newer version label); the font
            // list is made fresh from the document.
            let fonts = file.manifest.fonts.clone();
            file.manifest = m.clone();
            file.manifest.fonts = fonts;
        }
        file.to_bytes(crate::editor::SAVED_BY)
            .map_err(|e| e.to_string())
    }
}
