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

//! The export path template (rung 15 bbq, Q3, Q5 and Q9, and Defaulted "Tokens", "Safe names" and "Mistakes block
//! Export"). One template for the whole app, such as `{device}/{set}/{title}-{device}.png`, turns into one
//! relative path for each device chosen. No screen and no disk in here: text in, text out, so it is tested
//! headless.
//!
//! - `{title} {device} {profile} {model} {width} {height} {orientation} {type} {date} {version} {set}` are the
//!   tokens. Each value is made file-name safe by [`slug_or_empty`]. `{profile}` is the spec's name for
//!   `{device}` and gives the same value.
//! - `/` makes a folder. Any other character in the template's own text that is not a letter, digit, `-` or `_`
//!   becomes a hyphen (a run of them, one hyphen); that includes a dot, a backslash and a colon, so a path can
//!   never hold `..`, a drive letter or a Windows separator. A leading `/` is dropped.
//! - A part that comes out empty is dropped (an empty `{set}` drops its folder). The last part is the file's
//!   name; `.png` is added to it if the template did not end in it.
//! - Mistakes are data, not panics: [`PathError`] says an unknown token, a brace left open, no file name, or two
//!   devices landing on one path.

use std::collections::BTreeMap;

/// The default template: one flat folder, the device in the name (Q3).
pub const DEFAULT_TEMPLATE: &str = "{title}-{device}.png";
/// The Flat preset (Q9).
pub const PRESET_FLAT: &str = "{title}-{device}.png";
/// The Folder per device preset (Q9).
pub const PRESET_FOLDER_PER_DEVICE: &str = "{device}/{title}-{device}.png";
/// The Device then set preset (Q9).
pub const PRESET_DEVICE_THEN_SET: &str = "{device}/{set}/{title}-{device}.png";

/// The presets in the order the menu offers them: (the Fluent id of the name, the template).
pub const PRESETS: [(&str, &str); 3] = [
    ("export-preset-flat", PRESET_FLAT),
    ("export-preset-folder-per-device", PRESET_FOLDER_PER_DEVICE),
    ("export-preset-device-then-set", PRESET_DEVICE_THEN_SET),
];

/// Every token a template may hold, without the braces.
pub const TOKENS: [&str; 11] = [
    "title",
    "device",
    "profile",
    "model",
    "width",
    "height",
    "orientation",
    "type",
    "date",
    "version",
    "set",
];

/// The longest a single part of a path may be, so a long title cannot break a file system's limit.
const MAX_PART: usize = 120;

/// What a mistake in the template (or its result) is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    /// `{name}` is not a token.
    UnknownToken(String),
    /// A `{` with no `}` after it.
    Unclosed,
    /// The last part of the path is empty, so there is no file to write.
    NoFileName,
    /// Two devices would write the same file.
    SamePath {
        /// The first device's id.
        first: String,
        /// The second device's id.
        second: String,
        /// The path both land on.
        path: String,
    },
}

/// The values that are the same for every device of one export.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocTokens {
    /// The document's title (or the file's name, `untitled`, `sample`).
    pub title: String,
    /// `portrait` or `landscape`, the design's.
    pub orientation: String,
    /// `template` or `wallpaper`.
    pub kind: String,
    /// The export day, `YYYY-MM-DD`, local time.
    pub date: String,
    /// The app's version with dots as hyphens, `0-15-0`.
    pub version: String,
    /// The Set box.
    pub set: String,
}

/// The values that belong to one device.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceTokens {
    /// The profile's id (not a token; names the device in a [`PathError::SamePath`]).
    pub id: String,
    /// The profile's short name: `{device}` and `{profile}`.
    pub device: String,
    /// The profile's display name: `{model}`.
    pub model: String,
    /// The PNG's width in pixels (the device's portrait size, for a landscape design too).
    pub width: u32,
    /// The PNG's height in pixels.
    pub height: u32,
}

/// ASCII letters, digits, hyphens and underscores only (napkin §11.3): letters lowercased, any other run of
/// characters becomes one hyphen, none at either end. Empty when nothing is left.
pub fn slug_or_empty(s: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c.to_ascii_lowercase());
        } else {
            // A hyphen the text already has is a gap like any other character.
            gap = true;
        }
    }
    out
}

/// The text of the template that is not a token, made safe: letters, digits, `-`, `_` and `/` stay, a run of
/// anything else becomes one hyphen.
fn safe_literal(s: &str, out: &mut String) {
    let mut gap = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/') {
            if gap && c != '-' {
                out.push('-');
            }
            gap = false;
            out.push(c);
        } else {
            gap = true;
        }
    }
    if gap && !out.ends_with('-') {
        out.push('-');
    }
}

/// `template` without a closing `.png` (any case): the extension is put back after the parts are made.
fn without_png(template: &str) -> &str {
    let t = template.trim();
    match t.len().checked_sub(4) {
        Some(n) if t.is_char_boundary(n) && t[n..].eq_ignore_ascii_case(".png") => &t[..n],
        _ => t,
    }
}

/// Checks only that every `{...}` in `template` names a token and every brace closes.
pub fn check(template: &str) -> Result<(), PathError> {
    walk(without_png(template), |_| Ok(String::new())).map(|_| ())
}

/// Reads `template`, calling `value` for each token, and returns the text with the values put in. Literal text
/// is made safe as it goes; token values come back slugged from `value`.
fn walk(
    template: &str,
    mut value: impl FnMut(&str) -> Result<String, PathError>,
) -> Result<String, PathError> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        safe_literal(&rest[..open], &mut out);
        let after = &rest[open + 1..];
        let close = after.find('}').ok_or(PathError::Unclosed)?;
        let name = after[..close].trim();
        if !TOKENS.contains(&name) {
            return Err(PathError::UnknownToken(name.to_string()));
        }
        out.push_str(&value(name)?);
        rest = &after[close + 1..];
    }
    safe_literal(rest, &mut out);
    Ok(out)
}

/// A name Windows will not make a file of (`con`, `nul`, `com1`, …) gets a trailing underscore.
fn dodge_reserved(part: &mut String) {
    let stem = part.split('.').next().unwrap_or("").to_ascii_lowercase();
    let reserved = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || ((stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.len() == 4
            && stem.ends_with(|c: char| c.is_ascii_digit() && c != '0'));
    if reserved {
        part.push('_');
    }
}

/// The relative path (with `/` between parts) one device's file gets.
pub fn expand(template: &str, doc: &DocTokens, dev: &DeviceTokens) -> Result<String, PathError> {
    let text = walk(without_png(template), |name| {
        let raw = match name {
            "title" => doc.title.clone(),
            "device" | "profile" => dev.device.clone(),
            "model" => dev.model.clone(),
            "width" => dev.width.to_string(),
            "height" => dev.height.to_string(),
            "orientation" => doc.orientation.clone(),
            "type" => doc.kind.clone(),
            "date" => doc.date.clone(),
            "version" => doc.version.clone(),
            "set" => doc.set.clone(),
            other => return Err(PathError::UnknownToken(other.to_string())),
        };
        Ok(slug_or_empty(&raw))
    })?;
    // The last part is the file; the ones before it are folders.
    let mut parts: Vec<&str> = text.split('/').collect();
    let file = parts.pop().unwrap_or("");
    if file.trim_matches('-').is_empty() {
        return Err(PathError::NoFileName);
    }
    let mut out: Vec<String> = Vec::new();
    for p in parts.into_iter().chain(std::iter::once(file)) {
        let p = p.trim_matches('-');
        if p.is_empty() {
            continue;
        }
        let mut p: String = p.chars().take(MAX_PART).collect();
        while p.ends_with('-') {
            p.pop();
        }
        dodge_reserved(&mut p);
        out.push(p);
    }
    if let Some(last) = out.last_mut() {
        last.push_str(".png");
    }
    Ok(out.join("/"))
}

/// The path of every device, in the order given. Fails on the first mistake: an unknown token, no file name, or
/// two devices on one path (compared without regard to case, since a Mac or Windows disk would fold them).
pub fn plan(
    template: &str,
    doc: &DocTokens,
    devices: &[DeviceTokens],
) -> Result<Vec<String>, PathError> {
    check(template)?;
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    let mut out = Vec::new();
    for d in devices {
        let path = expand(template, doc, d)?;
        if let Some(first) = seen.insert(path.to_ascii_lowercase(), &d.id) {
            return Err(PathError::SamePath {
                first: first.to_string(),
                second: d.id.clone(),
                path,
            });
        }
        out.push(path);
    }
    Ok(out)
}
