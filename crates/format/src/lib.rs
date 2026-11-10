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
//! └── outlines/<profile>.cbor  text outlines, one file per document profile (ADR-001 item 2)
//! ```
//!
//! The same document always gives the same bytes, on every machine: entries are written in a
//! fixed order with fixed timestamps, permissions and system byte.
//!
//! Not built yet: `assets/` and `previews/` (napkin §12.1), font signatures in the manifest
//! (ADR-001 item 6), and `cargo-fuzz` on the reader (item 8; a stable-Rust damage test stands in).

#![forbid(unsafe_code)]

pub mod migrate;

use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::{Document, Extra};
use semver::Version;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipArchive, ZipWriter};

/// The format version this build writes. Pre-1.0 while the format settles.
pub const FORMAT_VERSION: &str = "0.4.0";

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
    /// Fields from newer versions.
    #[serde(flatten)]
    pub extra: Extra,
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
        let mut m = self.manifest.clone();
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
        for (name, info) in &manifest.entries {
            let b = read_entry(&mut z, name)?;
            if b.len() as u64 != info.size || sha256_hex(&b) != info.sha256 {
                return Err(FormatError::Checksum(name.clone()));
            }
            blobs.insert(name.clone(), b);
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
            read_only,
            migrated_from,
        })
    }
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
