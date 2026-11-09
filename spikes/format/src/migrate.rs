//! The migration chain (§12.2): each step rewrites the untyped CBOR tree of the scene from one
//! version to the next, before it is read into the typed model.

use crate::container::QuadError;
use ciborium::Value;
use semver::Version;

type Step = fn(&mut Value) -> Result<(), QuadError>;

/// (from, to, step). Ordered. Add one line per format change.
const CHAIN: &[(&str, &str, Step)] = &[("0.1.0", "0.2.0", v0_1_to_v0_2)];

pub fn upgrade(scene: &mut Value, from: &Version, to: &Version) -> Result<(), QuadError> {
    let mut at = from.clone();
    while &at < to {
        let (_, next, step) = CHAIN
            .iter()
            .find(|(f, _, _)| Version::parse(f).unwrap() == at)
            .ok_or_else(|| QuadError::Version(format!("no migration from {at}")))?;
        step(scene)?;
        at = Version::parse(next).unwrap();
    }
    Ok(())
}

pub fn map_get_mut<'a>(v: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    match v {
        Value::Map(m) => m.iter_mut().find(|(k, _)| k.as_text() == Some(key)).map(|(_, v)| v),
        _ => None,
    }
}

fn map_take(v: &mut Value, key: &str) -> Option<Value> {
    match v {
        Value::Map(m) => {
            let i = m.iter().position(|(k, _)| k.as_text() == Some(key))?;
            Some(m.remove(i).1)
        }
        _ => None,
    }
}

fn map_put(v: &mut Value, key: &str, val: Value) {
    if let Value::Map(m) = v {
        m.push((Value::Text(key.into()), val));
    }
}

/// 0.1 had `rhythm.unit_mm` (one unit for both axes). 0.2 splits it into
/// `h_unit_mm` and `v_unit_mm` and adds `pixel_locked` (default true, napkin §6.3).
fn v0_1_to_v0_2(scene: &mut Value) -> Result<(), QuadError> {
    let rhythm = map_get_mut(scene, "rhythm").ok_or(QuadError::Missing("rhythm".into()))?;
    let unit = map_take(rhythm, "unit_mm").ok_or(QuadError::Missing("rhythm.unit_mm".into()))?;
    map_put(rhythm, "h_unit_mm", unit.clone());
    map_put(rhythm, "v_unit_mm", unit);
    if map_get_mut(rhythm, "pixel_locked").is_none() {
        map_put(rhythm, "pixel_locked", Value::Bool(true));
    }
    Ok(())
}
