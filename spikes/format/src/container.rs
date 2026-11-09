//! The `.quad` container (§12.1): a ZIP of CBOR entries, with a manifest that carries the
//! format version (§12.2), the required features and a SHA-256 checksum per entry.

use crate::migrate;
use crate::model::{Document, Extra};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const CURRENT_VERSION: &str = "0.2.0";
/// Features this build understands. A file listing any other required feature opens read-only.
pub const KNOWN_FEATURES: &[&str] = &["text-outlines", "overrides"];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Manifest {
    pub format_version: String,
    pub app_version: String,
    pub title: String,
    pub profiles: Vec<String>,
    pub font_refs: Vec<FontRef>,
    pub required_features: Vec<String>,
    /// path -> checksum and size of every other entry
    pub entries: BTreeMap<String, EntryInfo>,
    #[serde(flatten)]
    pub extra: Extra,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FontRef {
    pub id: String,
    pub version: String,
    pub embedded: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EntryInfo {
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug)]
pub enum QuadError {
    Zip(String),
    Cbor(String),
    Version(String),
    Checksum(String),
    Missing(String),
}

impl std::fmt::Display for QuadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for QuadError {}

pub fn to_cbor<T: Serialize>(v: &T) -> Vec<u8> {
    let mut out = Vec::new();
    ciborium::into_writer(v, &mut out).expect("cbor encode");
    out
}

pub fn from_cbor<T: for<'de> Deserialize<'de>>(b: &[u8]) -> Result<T, QuadError> {
    ciborium::from_reader(b).map_err(|e| QuadError::Cbor(e.to_string()))
}

fn sha_hex(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// Everything that goes into a file, already encoded.
pub struct Parts {
    pub format_version: String,
    pub required_features: Vec<String>,
    pub title: String,
    pub profiles: Vec<String>,
    pub font_refs: Vec<FontRef>,
    pub manifest_extra: Extra,
    pub scene: Vec<u8>,
    /// profile id -> encoded outline file
    pub outlines: BTreeMap<String, Vec<u8>>,
    /// profile id -> PNG bytes
    pub previews: BTreeMap<String, Vec<u8>>,
}

impl Parts {
    pub fn from_document(doc: &Document) -> Self {
        Parts {
            format_version: CURRENT_VERSION.into(),
            required_features: vec!["text-outlines".into()],
            title: doc.metadata.title.clone(),
            profiles: doc.profiles.iter().map(|p| p.profile_id.clone()).collect(),
            font_refs: vec![],
            manifest_extra: Extra::new(),
            scene: to_cbor(doc),
            outlines: BTreeMap::new(),
            previews: BTreeMap::new(),
        }
    }
}

pub fn write_quad(p: &Parts) -> Vec<u8> {
    // Fixed timestamp so the same document always gives the same bytes.
    let deflate = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default());

    let mut files: Vec<(String, &[u8], bool)> = vec![("scene.cbor".into(), &p.scene, true)];
    for (id, b) in &p.outlines {
        files.push((format!("outlines/{id}.cbor"), b, true));
    }
    for (id, b) in &p.previews {
        files.push((format!("previews/{id}.png"), b, false)); // PNG is already compressed
    }
    let entries = files
        .iter()
        .map(|(n, b, _)| (n.clone(), EntryInfo { sha256: sha_hex(b), size: b.len() as u64 }))
        .collect();
    let manifest = Manifest {
        format_version: p.format_version.clone(),
        app_version: "0.0.0-spike".into(),
        title: p.title.clone(),
        profiles: p.profiles.clone(),
        font_refs: p.font_refs.clone(),
        required_features: p.required_features.clone(),
        entries,
        extra: p.manifest_extra.clone(),
    };

    let mut z = ZipWriter::new(Cursor::new(Vec::new()));
    z.start_file("manifest.cbor", deflate).unwrap();
    z.write_all(&to_cbor(&manifest)).unwrap();
    for (name, b, compress) in files {
        z.start_file(name, if compress { deflate } else { stored }).unwrap();
        z.write_all(b).unwrap();
    }
    z.finish().unwrap().into_inner()
}

/// Why a file opened read-only, if it did.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadOnly {
    NewerMajor(String),
    UnknownRequiredFeatures(Vec<String>),
}

pub struct Opened {
    pub manifest: Manifest,
    pub doc: Document,
    pub outlines: BTreeMap<String, Vec<u8>>,
    pub previews: BTreeMap<String, Vec<u8>>,
    pub migrated_from: Option<Version>,
    pub read_only: Option<ReadOnly>,
}

fn read_entry(z: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<Vec<u8>, QuadError> {
    let mut f = z.by_name(name).map_err(|_| QuadError::Missing(name.into()))?;
    let mut out = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut out).map_err(|e| QuadError::Zip(e.to_string()))?;
    Ok(out)
}

pub fn read_quad(bytes: &[u8]) -> Result<Opened, QuadError> {
    let mut z = ZipArchive::new(Cursor::new(bytes)).map_err(|e| QuadError::Zip(e.to_string()))?;
    let manifest: Manifest = from_cbor(&read_entry(&mut z, "manifest.cbor")?)?;
    let file_v = Version::parse(&manifest.format_version).map_err(|e| QuadError::Version(e.to_string()))?;
    let cur_v = Version::parse(CURRENT_VERSION).unwrap();

    let mut read_only = None;
    let unknown: Vec<String> = manifest
        .required_features
        .iter()
        .filter(|f| !KNOWN_FEATURES.contains(&f.as_str()))
        .cloned()
        .collect();
    if file_v.major > cur_v.major {
        read_only = Some(ReadOnly::NewerMajor(file_v.to_string()));
    } else if !unknown.is_empty() {
        read_only = Some(ReadOnly::UnknownRequiredFeatures(unknown));
    }

    let mut blobs = BTreeMap::new();
    for (name, info) in &manifest.entries {
        let b = read_entry(&mut z, name)?;
        if sha_hex(&b) != info.sha256 || b.len() as u64 != info.size {
            return Err(QuadError::Checksum(name.clone()));
        }
        blobs.insert(name.clone(), b);
    }
    let scene = blobs.remove("scene.cbor").ok_or(QuadError::Missing("scene.cbor".into()))?;

    let (doc, migrated_from) = if file_v < cur_v {
        let mut v: ciborium::Value = from_cbor(&scene)?;
        migrate::upgrade(&mut v, &file_v, &cur_v)?;
        let doc = v.deserialized::<Document>().map_err(|e| QuadError::Cbor(e.to_string()))?;
        (doc, Some(file_v))
    } else {
        (from_cbor::<Document>(&scene)?, None)
    };

    let mut outlines = BTreeMap::new();
    let mut previews = BTreeMap::new();
    for (name, b) in blobs {
        if let Some(id) = name.strip_prefix("outlines/").and_then(|s| s.strip_suffix(".cbor")) {
            outlines.insert(id.to_string(), b);
        } else if let Some(id) = name.strip_prefix("previews/").and_then(|s| s.strip_suffix(".png")) {
            previews.insert(id.to_string(), b);
        }
    }
    Ok(Opened { manifest, doc, outlines, previews, migrated_from, read_only })
}

/// Save an opened document again (keeps unknown optional fields; refuses read-only files).
pub fn resave(o: &Opened) -> Result<Vec<u8>, QuadError> {
    if let Some(r) = &o.read_only {
        return Err(QuadError::Version(format!("read-only: {r:?}")));
    }
    let mut p = Parts::from_document(&o.doc);
    // A file from a newer minor version keeps its version number, so its unknown fields are still labelled.
    let file_v = Version::parse(&o.manifest.format_version).unwrap();
    let cur_v = Version::parse(CURRENT_VERSION).unwrap();
    p.format_version = file_v.max(cur_v).to_string();
    p.required_features = o.manifest.required_features.clone();
    p.font_refs = o.manifest.font_refs.clone();
    p.manifest_extra = o.manifest.extra.clone();
    p.outlines = o.outlines.clone();
    p.previews = o.previews.clone();
    Ok(write_quad(&p))
}

/// The developer menu's "Export as readable JSON" (§12.1).
pub fn to_debug_json(doc: &Document) -> String {
    serde_json::to_string_pretty(doc).unwrap()
}

pub fn from_debug_json(s: &str) -> Result<Document, QuadError> {
    serde_json::from_str(s).map_err(|e| QuadError::Cbor(e.to_string()))
}

/// A tiny 8-bit grayscale PNG, as a stand-in preview thumbnail.
pub fn tiny_png(w: u32, h: u32, gray: u8) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().unwrap();
        wr.write_image_data(&vec![gray; (w * h) as usize]).unwrap();
    }
    out
}
