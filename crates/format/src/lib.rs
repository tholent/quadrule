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

//! The `.quad` project file (napkin §12) as ADR-001 decides it: a ZIP of Deflate-compressed
//! CBOR entries with field names kept, a semver `format_version`, a migration chain, newer
//! files opened read-only when they need an unknown feature, and unknown fields kept on save.
//!
//! ```text
//! project.quad
//! ├── manifest.cbor            format version, last saver, title, profiles, fonts, checksums
//! ├── scene.cbor               the Document
//! ├── outlines/<profile>.cbor  text outlines, one file per document profile (ADR-001 item 2)
//! ├── assets/fonts/<sha256>.<ttf|otf>  embedded installed fonts, whole files (rung 12a)
//! └── profiles/<id>.ron        the profiles the page uses that are not an unedited built-in (rung 13a)
//! ```
//!
//! The same document always gives the same bytes, on every machine: entries are written in a
//! fixed order with fixed timestamps, permissions and system byte.
//!
//! Not built yet: `previews/` and image assets (napkin §12.1). Font signatures in the manifest
//! (ADR-001 item 6) and embedded fonts came with format 0.13; a damaged embedded font is skipped
//! and never stops a file opening. Format 0.14 carries profiles (the user's own, and an edited Manta
//! or Nomad) as the same RON text Export… writes; a carried profile outside the limits is skipped
//! the same way, and the page opens with that profile missing.

#![forbid(unsafe_code)]

pub mod migrate;

use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{Document, Extra, FontSignature};
use quadrule_profiles::{Profile, parse_checked, to_ron};
use semver::Version;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipArchive, ZipWriter};

/// The format version this build writes. Pre-1.0 while the format settles.
pub const FORMAT_VERSION: &str = "0.14.0";

/// Required features this build understands. A file that requires any other opens read-only.
pub const KNOWN_FEATURES: &[&str] = &["text-outlines"];

/// No entry may decompress to more than this; a guard against damaged or hostile files.
pub const MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;

const MANIFEST: &str = "manifest.cbor";
const SCENE: &str = "scene.cbor";

/// The file's table of contents (napkin §12.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Manifest {
    /// Semver of the format (napkin §12.2). A resaved newer file keeps its own (ADR-001 item 5).
    pub format_version: String,
    /// The app version that last saved the file (ADR-001 item 5).
    pub saved_by: String,
    /// The document's title, readable without decoding the scene.
    pub title: String,
    /// The document's profiles, primary first.
    pub profiles: Vec<String>,
    /// Fonts the document uses.
    pub fonts: Vec<FontRef>,
    /// The profiles saved inside the file (format 0.14), each also an entry under `profiles/`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub carried_profiles: Vec<ProfileRef>,
    /// Features a reader must understand to edit the file.
    pub required_features: Vec<String>,
    /// Every other entry's path, with its checksum and size.
    pub entries: BTreeMap<String, EntryInfo>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// A font the document uses (napkin §7.3: bundled fonts by id and version).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FontRef {
    /// Family name.
    pub family: String,
    /// The bundled font's id, when the font ships with quadrule.
    pub bundled_id: Option<String>,
    /// The font's version string.
    pub version: Option<String>,
    /// The font's measured signature (ADR-001 item 6; format 0.13), so a computer without the font
    /// can pick a close substitute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<FontSignature>,
    /// The font files saved inside the file (format 0.13), when the font was embedded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<FontFile>,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

impl FontRef {
    /// A reference with no signature and no embedded files.
    pub fn new(family: &str, bundled_id: Option<&str>, version: Option<String>) -> Self {
        FontRef {
            family: family.into(),
            bundled_id: bundled_id.map(String::from),
            version,
            signature: None,
            files: Vec::new(),
            extra: Extra::new(),
        }
    }
}

/// A profile saved inside the file (format 0.14), so a computer without it can still show the page.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProfileRef {
    /// The profile's id.
    pub id: String,
    /// Its name, readable without parsing the profile.
    pub name: String,
    /// Where it is in the archive: `profiles/<id>.ron`.
    pub path: String,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The folder carried profiles are kept in.
pub const PROFILE_DIR: &str = "profiles/";

/// The path a carried profile is stored at: `profiles/<id>.ron`.
pub fn profile_path(id: &str) -> String {
    format!("{PROFILE_DIR}{id}.ron")
}

/// One embedded font file (format 0.13): a whole font file under `assets/fonts/`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FontFile {
    /// Where it is in the archive: `assets/fonts/<sha256>.<ttf|otf>`.
    pub path: String,
    /// sha256 of the file, lowercase hex (also its name).
    pub sha256: String,
    /// The weight of the face the file holds, 100 to 900 (for a variable font, its default).
    pub weight: u16,
    /// True when the face is italic or oblique.
    pub italic: bool,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// The folder embedded fonts are kept in.
pub const FONT_DIR: &str = "assets/fonts/";

/// The path an embedded font file is stored at: `assets/fonts/<sha256>.<ttf|otf>`.
pub fn font_path(bytes: &[u8]) -> String {
    let ext = if bytes.starts_with(b"OTTO") {
        "otf"
    } else {
        "ttf"
    };
    format!("{FONT_DIR}{}.{ext}", sha256_hex(bytes))
}

/// One entry's fingerprint.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EntryInfo {
    /// sha256 of the uncompressed bytes, lowercase hex.
    pub sha256: String,
    /// Uncompressed size in bytes.
    pub size: u64,
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
}

/// Why a file opened read-only (napkin §12.2).
#[derive(Debug, Clone, PartialEq)]
pub enum ReadOnly {
    /// Written by a newer major format version.
    NewerMajor(String),
    /// Requires features this build does not know.
    UnknownFeatures(Vec<String>),
}

/// Why a file could not be read or written.
#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    /// The ZIP container is damaged.
    Zip(String),
    /// An entry is not valid CBOR for its type.
    Cbor(String),
    /// A required entry is missing.
    Missing(String),
    /// An entry's bytes do not match the manifest's checksum or size.
    Checksum(String),
    /// An entry is larger than [`MAX_ENTRY_BYTES`].
    TooLarge(String),
    /// The version is unreadable, or no migration leads from it.
    Version(String),
    /// The file is read-only and cannot be saved.
    ReadOnly(ReadOnly),
    /// Outlines must be saved for exactly the document's profiles (ADR-001 item 2).
    OutlinesMismatch {
        /// The document's profiles.
        profiles: Vec<String>,
        /// The profiles outlines were given for.
        outlines: Vec<String>,
    },
    /// The readable JSON debug form did not parse.
    Json(String),
    /// A profile to be carried is not acceptable (outside the limits, or filed under another id).
    Profile(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FormatError {}

/// An opened (or new) `.quad` file.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadFile {
    /// The manifest as read, or a fresh one for a new file.
    pub manifest: Manifest,
    /// The document.
    pub doc: Document,
    /// Text outlines, keyed by profile id.
    pub outlines: BTreeMap<String, ProfileOutlines>,
    /// The embedded font files, by their path in the archive (format 0.13).
    pub font_files: BTreeMap<String, Vec<u8>>,
    /// The profiles the file carries, by id (format 0.14). Each was read through the profile limits.
    pub profiles: BTreeMap<String, Profile>,
    /// Carried profiles that were damaged or outside the limits and were skipped on opening, each
    /// as (its path in the archive, why); the file still opened.
    pub damaged_profiles: Vec<(String, String)>,
    /// Embedded font files that were damaged or missing and were skipped on opening; the file
    /// still opened (rung 12a bbq, Defaulted "Embedding in detail").
    pub damaged_fonts: Vec<String>,
    /// Set when the file must not be saved by this build.
    pub read_only: Option<ReadOnly>,
    /// The version the scene was upgraded from, if it was.
    pub migrated_from: Option<Version>,
}

impl QuadFile {
    /// A new file for a document, its outlines (one per profile) and the fonts it uses.
    pub fn new(doc: Document, outlines: Vec<ProfileOutlines>, fonts: Vec<FontRef>) -> Self {
        let manifest = Manifest {
            format_version: FORMAT_VERSION.into(),
            saved_by: String::new(),
            title: doc.metadata.title.clone(),
            profiles: vec![],
            fonts,
            carried_profiles: Vec::new(),
            required_features: vec!["text-outlines".into()],
            entries: BTreeMap::new(),
            extra: Extra::new(),
        };
        let outlines = outlines
            .into_iter()
            .map(|o| (o.profile_id.clone(), o))
            .collect();
        QuadFile {
            manifest,
            doc,
            outlines,
            font_files: BTreeMap::new(),
            profiles: BTreeMap::new(),
            damaged_profiles: Vec::new(),
            damaged_fonts: Vec::new(),
            read_only: None,
            migrated_from: None,
        }
    }

    /// Writes the file. `app_version` is recorded as the last saver. Refuses read-only files and
    /// outlines that do not match the document's profiles exactly.
    pub fn to_bytes(&self, app_version: &str) -> Result<Vec<u8>, FormatError> {
        if let Some(r) = &self.read_only {
            return Err(FormatError::ReadOnly(r.clone()));
        }
        let profiles: Vec<String> = self
            .doc
            .profile_ids()
            .into_iter()
            .map(String::from)
            .collect();
        let want: BTreeSet<&str> = profiles.iter().map(String::as_str).collect();
        let have: BTreeSet<&str> = self.outlines.keys().map(String::as_str).collect();
        let ids_match = self.outlines.iter().all(|(k, o)| *k == o.profile_id);
        if want != have || want.len() != profiles.len() || !ids_match {
            return Err(FormatError::OutlinesMismatch {
                profiles,
                outlines: self.outlines.keys().cloned().collect(),
            });
        }

        let mut files: Vec<(String, Vec<u8>)> = vec![(SCENE.into(), to_cbor(&self.doc)?)];
        for (id, o) in &self.outlines {
            files.push((format!("outlines/{id}.cbor"), to_cbor(o)?));
        }
        for (path, bytes) in &self.font_files {
            files.push((path.clone(), bytes.clone()));
        }
        for (id, p) in &self.profiles {
            if *id != p.id {
                return Err(FormatError::Profile(format!("{id} holds profile {}", p.id)));
            }
            quadrule_profiles::limits::check(p)
                .map_err(|e| FormatError::Profile(format!("{id}: {e}")))?;
            files.push((profile_path(id), to_ron(p).into_bytes()));
        }
        let mut m = self.manifest.clone();
        m.carried_profiles = self
            .profiles
            .values()
            .map(|p| ProfileRef {
                id: p.id.clone(),
                name: p.display_name.clone(),
                path: profile_path(&p.id),
                extra: Extra::new(),
            })
            .collect();
        // A newer file keeps its own label, so its unknown fields stay labelled (ADR-001 item 5).
        let ours = Version::parse(FORMAT_VERSION).expect("FORMAT_VERSION is semver");
        let theirs =
            Version::parse(&m.format_version).map_err(|e| FormatError::Version(e.to_string()))?;
        m.format_version = theirs.max(ours).to_string();
        m.saved_by = app_version.into();
        m.title = self.doc.metadata.title.clone();
        m.profiles = profiles;
        m.entries = files
            .iter()
            .map(|(n, b)| {
                (
                    n.clone(),
                    EntryInfo {
                        sha256: sha256_hex(b),
                        size: b.len() as u64,
                        extra: Extra::new(),
                    },
                )
            })
            .collect();

        // Fixed time, permissions and system byte: the same bytes on every machine.
        let opts = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(DateTime::default())
            .unix_permissions(0o644)
            .system(System::Unix);
        let mut z = ZipWriter::new(Cursor::new(Vec::new()));
        let manifest = to_cbor(&m)?;
        for (name, bytes) in std::iter::once((MANIFEST.to_string(), manifest)).chain(files) {
            z.start_file(name, opts)
                .map_err(|e| FormatError::Zip(e.to_string()))?;
            z.write_all(&bytes)
                .map_err(|e| FormatError::Zip(e.to_string()))?;
        }
        Ok(z.finish()
            .map_err(|e| FormatError::Zip(e.to_string()))?
            .into_inner())
    }

    /// Reads a file. Damaged input gives an error, never a panic.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FormatError> {
        let mut z =
            ZipArchive::new(Cursor::new(bytes)).map_err(|e| FormatError::Zip(e.to_string()))?;
        let manifest: Manifest = from_cbor(&read_entry(&mut z, MANIFEST)?)?;
        let file_v = Version::parse(&manifest.format_version)
            .map_err(|e| FormatError::Version(e.to_string()))?;
        let ours = Version::parse(FORMAT_VERSION).expect("FORMAT_VERSION is semver");

        let unknown: Vec<String> = manifest
            .required_features
            .iter()
            .filter(|f| !KNOWN_FEATURES.contains(&f.as_str()))
            .cloned()
            .collect();
        let read_only = if file_v.major > ours.major {
            Some(ReadOnly::NewerMajor(file_v.to_string()))
        } else if !unknown.is_empty() {
            Some(ReadOnly::UnknownFeatures(unknown))
        } else {
            None
        };

        let mut blobs = BTreeMap::new();
        let mut font_files = BTreeMap::new();
        let mut damaged_fonts = Vec::new();
        let mut profiles = BTreeMap::new();
        let mut damaged_profiles = Vec::new();
        for (name, info) in &manifest.entries {
            let embedded_font = name.starts_with(FONT_DIR);
            let carried_profile = name.starts_with(PROFILE_DIR);
            let read = read_entry(&mut z, name).and_then(|b| {
                if b.len() as u64 != info.size || sha256_hex(&b) != info.sha256 {
                    Err(FormatError::Checksum(name.clone()))
                } else {
                    Ok(b)
                }
            });
            match read {
                Ok(b) if embedded_font => {
                    font_files.insert(name.clone(), b);
                }
                // A carried profile goes through the same limits as an imported one; one that
                // breaks them is skipped, and the page opens with that profile missing.
                Ok(b) if carried_profile => match carried(name, &b) {
                    Ok(p) => {
                        profiles.insert(p.id.clone(), p);
                    }
                    Err(e) => damaged_profiles.push((name.clone(), e)),
                },
                Err(e) if carried_profile => damaged_profiles.push((name.clone(), e.to_string())),
                Ok(b) => {
                    blobs.insert(name.clone(), b);
                }
                // A damaged embedded font is skipped; its text falls back to saved outlines.
                Err(_) if embedded_font => damaged_fonts.push(name.clone()),
                Err(e) => return Err(e),
            }
        }
        let scene = blobs
            .remove(SCENE)
            .ok_or_else(|| FormatError::Missing(SCENE.into()))?;
        let (doc, migrated_from) = if file_v < ours {
            let mut v: ciborium::Value = from_cbor(&scene)?;
            migrate::upgrade(&mut v, &file_v, &ours, migrate::CHAIN)?;
            let doc = v
                .deserialized::<Document>()
                .map_err(|e| FormatError::Cbor(e.to_string()))?;
            (doc, Some(file_v))
        } else {
            (from_cbor::<Document>(&scene)?, None)
        };

        let mut outlines = BTreeMap::new();
        for (name, b) in blobs {
            if let Some(id) = name
                .strip_prefix("outlines/")
                .and_then(|s| s.strip_suffix(".cbor"))
            {
                outlines.insert(id.to_string(), from_cbor::<ProfileOutlines>(&b)?);
            }
        }
        Ok(QuadFile {
            manifest,
            doc,
            outlines,
            font_files,
            profiles,
            damaged_profiles,
            damaged_fonts,
            read_only,
            migrated_from,
        })
    }
}

/// A carried profile's text, read through the limits; the file name must say the profile's id.
fn carried(path: &str, bytes: &[u8]) -> Result<Profile, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "the profile is not text".to_string())?;
    let p = parse_checked(text).map_err(|e| e.to_string())?;
    if profile_path(&p.id) != path {
        return Err(format!("the profile `{}` is filed under {path}", p.id));
    }
    Ok(p)
}

fn read_entry(z: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<Vec<u8>, FormatError> {
    let f = z
        .by_name(name)
        .map_err(|_| FormatError::Missing(name.into()))?;
    let mut out = Vec::new();
    f.take(MAX_ENTRY_BYTES + 1)
        .read_to_end(&mut out)
        .map_err(|e| FormatError::Zip(e.to_string()))?;
    if out.len() as u64 > MAX_ENTRY_BYTES {
        return Err(FormatError::TooLarge(name.into()));
    }
    Ok(out)
}

/// Encodes a value as CBOR.
pub fn to_cbor<T: Serialize>(v: &T) -> Result<Vec<u8>, FormatError> {
    let mut out = Vec::new();
    ciborium::into_writer(v, &mut out).map_err(|e| FormatError::Cbor(e.to_string()))?;
    Ok(out)
}

/// Decodes a value from CBOR.
pub fn from_cbor<T: DeserializeOwned>(b: &[u8]) -> Result<T, FormatError> {
    ciborium::from_reader(b).map_err(|e| FormatError::Cbor(e.to_string()))
}

/// Lowercase hex sha256.
pub fn sha256_hex(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}

/// "Export as readable JSON" (napkin §12.1): the same model, as pretty JSON.
pub fn to_debug_json(doc: &Document) -> Result<String, FormatError> {
    serde_json::to_string_pretty(doc).map_err(|e| FormatError::Json(e.to_string()))
}

/// Opens the readable JSON form.
pub fn from_debug_json(s: &str) -> Result<Document, FormatError> {
    serde_json::from_str(s).map_err(|e| FormatError::Json(e.to_string()))
}

/// The document with profile `from` called `to` everywhere it is named: the primary profile, the
/// document's profile list, and each object's per-device overrides, in groups and repeats too. Used
/// when a clash is settled by giving an incoming profile another id (rung 13a bbq, Q9). Text outlines
/// are keyed by profile too: [`rename_profile_outlines`].
pub fn rename_profile(doc: &Document, from: &str, to: &str) -> Result<Document, FormatError> {
    let mut v = ciborium::Value::serialized(doc).map_err(|e| FormatError::Cbor(e.to_string()))?;
    rename_in_value(&mut v, from, to);
    v.deserialized::<Document>()
        .map_err(|e| FormatError::Cbor(e.to_string()))
}

/// The saved outlines with profile `from` called `to`.
pub fn rename_profile_outlines(
    outlines: &BTreeMap<String, ProfileOutlines>,
    from: &str,
    to: &str,
) -> BTreeMap<String, ProfileOutlines> {
    outlines
        .iter()
        .map(|(k, o)| {
            if k == from {
                let mut o = o.clone();
                o.profile_id = to.to_string();
                (to.to_string(), o)
            } else {
                (k.clone(), o.clone())
            }
        })
        .collect()
}

fn rename_in_value(v: &mut ciborium::Value, from: &str, to: &str) {
    use ciborium::Value;
    match v {
        Value::Array(items) => {
            for i in items {
                rename_in_value(i, from, to);
            }
        }
        Value::Map(entries) => {
            for (k, val) in entries.iter_mut() {
                let key = k.as_text().unwrap_or("");
                match (key, &mut *val) {
                    ("primary_profile_id" | "profile_id", Value::Text(t)) if t == from => {
                        *t = to.to_string();
                    }
                    ("overrides", Value::Map(per_device)) => {
                        for (dk, dv) in per_device.iter_mut() {
                            if dk.as_text() == Some(from) {
                                *dk = Value::Text(to.to_string());
                            }
                            rename_in_value(dv, from, to);
                        }
                    }
                    _ => rename_in_value(val, from, to),
                }
            }
        }
        _ => {}
    }
}
