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
    ("0.8.0", "0.9.0", v0_8_to_v0_9),
    ("0.9.0", "0.10.0", v0_9_to_v0_10),
    ("0.10.0", "0.11.0", v0_10_to_v0_11),
    ("0.11.0", "0.12.0", v0_11_to_v0_12),
    ("0.12.0", "0.13.0", v0_12_to_v0_13),
    ("0.13.0", "0.14.0", v0_13_to_v0_14),
    ("0.14.0", "0.15.0", v0_14_to_v0_15),
    ("0.15.0", "0.16.0", v0_15_to_v0_16),
    ("0.16.0", "0.17.0", v0_16_to_v0_17),
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

/// 0.9 adds new `Pattern` choices (isometric, hex, Cornell, music staff, each carrying its own
/// settings) and optional fields: `rhythm_type`, `side_mm` and `major_every` on the rhythm, and
/// `bound`, `h_spacing_mm`, `v_spacing_mm`, `major` and `major_role` on a pattern region (rung 9 bbq,
/// Defaulted "File format 0.9.0"). A 0.8 file has none of them, which reads as a rectangular
/// rhythm, no major lines and regions bound to the rhythm, exactly what it drew before, so there is
/// nothing to rewrite. A 0.8 build cannot read a 0.9 file that uses the new patterns; it reports an
/// error rather than guessing.
fn v0_8_to_v0_9(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.10 adds the repeat kind (`Repeat`, holding its `source` and its settings; rung 10 bbq, Defaulted
/// "File format 0.10.0"). It is a new choice only: every 0.9 scene is already a valid 0.10 scene,
/// nothing is rewritten (as 0.6 to 0.7 added the group). A 0.9 build cannot read a 0.10 file that
/// holds a repeat; it reports an error rather than guessing.
fn v0_9_to_v0_10(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.11 adds `style` on a text box (Normal or Italic) and `italic` and `opsz` on the face of a saved
/// glyph table (rung 11a bbq, Defaulted "File format 0.11.0"). A 0.10 text box has no style, which
/// reads as Normal, and a 0.10 table has neither, which reads as upright and no optical size, exactly
/// what they drew before, so there is nothing to rewrite. A 0.10 build keeps the new fields as
/// unknown ones on a save but would draw an italic box upright.
fn v0_10_to_v0_11(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.12 adds four optional fields on a text box (`sizing`, `paragraph_spacing`, `v_align` and
/// `baselines`) and `lines_hidden` on a saved text's outlines (rung 11b bbq, Defaulted "File format
/// 0.12.0"), and lets a text box carry a rotation. Each is left out of a file when it holds its
/// default, and a 0.11 text box has none, which reads as an Auto-height box with no paragraph space,
/// at the top, with baselines off, none hidden: exactly what it drew before. A 0.11 text box carrying
/// a stored `rotation_deg` (never set by the app, which ignored it) would now turn; no app-made file
/// holds one. Nothing is rewritten. A 0.11 build keeps the new fields as unknown ones on a save but
/// would lay a Fixed box out as tall as its text.
fn v0_11_to_v0_12(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.13 adds `fonts` on the page settings (Embed fonts, Don't warn again, the fonts kept as saved
/// outlines), and, in the manifest, a measured signature and the embedded files of a font (rung 12
/// bbq, Defaulted "File format 0.13.0"). Each is left out of a file when it holds its default, so a
/// 0.12 scene needs no rewriting for them. One thing is rewritten: a pattern region's stored
/// `rotation_deg` is dropped (set to 0). Pattern regions ignored it until now, and from 0.13 they turn
/// with it, so an old file holding one keeps drawing its region unturned and no old page changes.
fn v0_12_to_v0_13(scene: &mut Value) -> Result<(), FormatError> {
    if let Some(Value::Array(layers)) = map_get_mut(scene, "layers") {
        for layer in layers {
            if let Some(Value::Array(objects)) = map_get_mut(layer, "objects") {
                for o in objects {
                    zero_pattern_rotation(o);
                }
            }
        }
    }
    Ok(())
}

/// 0.14 adds profiles carried inside the file (`profiles/<id>.ron` entries, `carried_profiles` in the
/// manifest; rung 13a bbq, Q4 and Defaulted "Carrying in the file"). Nothing in the scene changes, so a
/// 0.13 scene needs no rewriting: a 0.13 file carries none, and every profile it names is one this
/// build ships. A 0.13 build keeps the new entries as unknown ones.
fn v0_13_to_v0_14(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.15 adds two optional fields (rung 14 bbq, Q2 and Q7): `ignored_rules` on the document (the lint rules
/// this page turns off, on every device) and `free_placement` on an object (L16 leaves it), and the
/// `lint` block in a carried profile (`profiles/<id>.ron`, which has a default of every value empty). Each
/// is left out of a file when it holds its default, so a 0.14 scene needs no rewriting: no rule ignored,
/// no object free, and a carried 0.14 profile reads with its lint values empty (those checks are skipped on
/// it until the person fills them in). A 0.14 build keeps the new fields as unknown ones on a save.
fn v0_14_to_v0_15(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.16 adds `previews/<primary_profile_id>.png` (rung 15 bbq, Q8): a thumbnail of the main device, written at Save
/// and Save As. Nothing in the scene changes, so a 0.15 scene needs no rewriting: a 0.15 file has no preview and
/// stays valid, and it gets one at its next save. A 0.15 build ignores the new entry (it is not the scene or an
/// outline, so it reads past it) and drops it on a save.
fn v0_15_to_v0_16(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// 0.17 adds an optional `border` on a pattern region (pattern borders bbq, docs/bbq/2026-11-18-pattern-borders.md,
/// Defaulted "Where it lives"): which sides, the line (a role or a width and gray) and a corner radius. A region
/// with none has no border and the field is left out of its file, so a 0.16 scene needs no rewriting and draws
/// and exports as it did. A 0.16 build keeps the new field as an unknown one on a save, but would draw the
/// region with no border.
fn v0_16_to_v0_17(_scene: &mut Value) -> Result<(), FormatError> {
    Ok(())
}

/// Drops `rotation_deg` from a pattern region, and from every pattern region inside a group or a
/// repeat's source.
fn zero_pattern_rotation(object: &mut Value) {
    let mut kind_name = None;
    if let Some(Value::Map(kind)) = map_get_mut(object, "kind") {
        kind_name = kind
            .first()
            .and_then(|(k, _)| k.as_text().map(String::from));
    }
    match kind_name.as_deref() {
        Some("Pattern") => {
            if let Value::Map(m) = object {
                m.retain(|(k, _)| k.as_text() != Some("rotation_deg"));
            }
        }
        Some("Group") => {
            if let Some(Value::Map(kind)) = map_get_mut(object, "kind")
                && let Some((_, inner)) = kind.first_mut()
                && let Some(Value::Array(children)) = map_get_mut(inner, "children")
            {
                for c in children {
                    zero_pattern_rotation(c);
                }
            }
        }
        Some("Repeat") => {
            if let Some(Value::Map(kind)) = map_get_mut(object, "kind")
                && let Some((_, inner)) = kind.first_mut()
                && let Some(source) = map_get_mut(inner, "source")
            {
                zero_pattern_rotation(source);
            }
        }
        _ => {}
    }
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
