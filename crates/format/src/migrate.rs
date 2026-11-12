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

//! The migration chain (napkin §12.2): each step rewrites the untyped CBOR tree of the scene
//! from one format version to the next, before it is read into the typed model. Add one line per
//! format change, with a test.

use crate::FormatError;
use ciborium::Value;
use semver::Version;

/// One step: rewrites the scene in place.
pub type Step = fn(&mut Value) -> Result<(), FormatError>;

/// (from, to, step), in order.
pub const CHAIN: &[(&str, &str, Step)] = &[
    ("0.1.0", "0.2.0", v0_1_to_v0_2),
    ("0.2.0", "0.3.0", v0_2_to_v0_3),
    ("0.3.0", "0.4.0", v0_3_to_v0_4),
    ("0.4.0", "0.5.0", v0_4_to_v0_5),
    ("0.5.0", "0.6.0", v0_5_to_v0_6),
    ("0.6.0", "0.7.0", v0_6_to_v0_7),
    ("0.7.0", "0.8.0", v0_7_to_v0_8),
];

/// 0.2 adds new choices only (the `Centered` rhythm origin, the `Grid` pattern), so every 0.1
/// scene is already a valid 0.2 scene: nothing to rewrite. A 0.1 build cannot read a 0.2 file
/// that uses them; it reports an error rather than guessing.
fn v0_1_to_v0_2(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.3 adds two roles (`DotGrid`, `Label`), new choices only: every 0.2 scene is already a
/// valid 0.3 scene. Dot patterns in older files keep the role they were saved with.
fn v0_2_to_v0_3(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.4 adds `rhythm.subdivisions`, an optional field (rung 4 bbq, Defaulted "Subdivisions in the
/// file"): a 0.3 scene without it reads as subdivisions off, so there is nothing to rewrite.
fn v0_3_to_v0_4(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.5 adds the rectangle, ellipse and polygon kinds, the `Style::None` choice, and three optional
/// fields on an object (`fill`, `dash`, `cap`; rung 6 bbq, Defaulted "File format 0.5.0"). A 0.4
/// object has none of them, which reads as no fill, a solid stroke and butt caps, exactly what it
/// drew before, so there is nothing to rewrite. A 0.4 build cannot read a 0.5 file that uses the
/// new kinds; it reports an error rather than guessing.
fn v0_4_to_v0_5(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.6 adds three optional fields (rung 6 bbq, Defaulted "File format 0.5.0 (6a) … 6b adds rotation,
/// line direction and opacity in format 0.6.0"): `rotation_deg` and `opacity_pct` on an object and
/// `direction` on a line. A 0.5 object has none of them, which reads as no rotation, fully
/// opaque and a line from the top-left to the bottom-right corner, exactly what it drew before,
/// so there is nothing to rewrite. A 0.5 build cannot read a 0.6 file that uses them correctly (it
/// would keep them as unknown fields and draw the object unrotated); it keeps them on a save.
fn v0_5_to_v0_6(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.7 adds the group kind (`Group`, with its `children`; rung 7a bbq, Defaulted "File format 0.7.0
/// (7a)"). It is a new choice only: every 0.6 scene is already a valid 0.7 scene, nothing is
/// rewritten (as 0.1 to 0.2). A 0.6 build cannot read a 0.7 file that holds a group; it reports an
/// error rather than guessing.
fn v0_6_to_v0_7(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.8 adds two optional fields (rung 8 bbq, Defaulted "File format 0.8.0"): `overrides` on an
/// object (what differs on one secondary device: place, size, font size, eye) and `orientation` on
/// the page settings. A 0.7 file has neither, which reads as no overrides and a portrait page,
/// exactly what it drew before, so there is nothing to rewrite. A 0.7 build keeps the new fields
/// as unknown ones on a save, but would draw the object where its constraints put it.
fn v0_7_to_v0_8(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// Upgrades `scene` from `from` to `to` by walking `chain` one step at a time.
pub fn upgrade(
    scene: &mut Value,
    from: &Version,
    to: &Version,
    chain: &[(&str, &str, Step)],
) -> Result<(), FormatError> {
    let mut at = from.clone();
    while &at < to {
        let (_, next, step) = chain
            .iter()
            .find(|(f, _, _)| Version::parse(f).is_ok_and(|v| v == at))
            .ok_or_else(|| FormatError::Version(format!("no migration from {at}")))?;
        step(scene)?;
        at = Version::parse(next).map_err(|e| FormatError::Version(e.to_string()))?;
    }
    Ok(())
}

/// The value under `key` in a CBOR map, for migration steps.
pub fn map_get_mut<'a>(v: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    match v {
        Value::Map(m) => m
            .iter_mut()
            .find(|(k, _)| k.as_text() == Some(key))
            .map(|(_, v)| v),
        _ => None,
    }
}
