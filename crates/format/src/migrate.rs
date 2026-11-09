//! The migration chain (napkin §12.2): each step rewrites the untyped CBOR tree of the scene
//! from one format version to the next, before it is read into the typed model. The chain is
//! empty at 0.1.0, the first version; add one line per format change, with a test.

use crate::FormatError;
use ciborium::Value;
use semver::Version;

/// One step: rewrites the scene in place.
pub type Step = fn(&mut Value) -> Result<(), FormatError>;

/// (from, to, step), in order.
pub const CHAIN: &[(&str, &str, Step)] = &[];

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
