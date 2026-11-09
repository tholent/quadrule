//! Format spike for Quadrule (napkin §12, spike 3). Throwaway code; the findings are in FINDINGS.md.

pub mod container;
pub mod migrate;
pub mod model;
pub mod outlines;

#[cfg(test)]
mod tests {
    use crate::container::*;
    use crate::migrate::map_get_mut;
    use crate::model::*;
    use crate::outlines::*;
    use ciborium::Value;

    fn sample_parts() -> Parts {
        parts_for(&sample_document())
    }

    fn parts_for(doc: &Document) -> Parts {
        let mut p = Parts::from_document(doc);
        p.outlines.insert("supernote.a5x2".into(), vec![1, 2, 3]);
        p.previews.insert("supernote.a5x2".into(), tiny_png(48, 64, 0xEE));
        p
    }

    fn line(id: &str, name: &str, ok: bool, value: &str) {
        println!("CHECK id={id} name={name} result={} value={value}", if ok { "pass" } else { "fail" });
    }

    #[test]
    fn roundtrip() {
        let doc = sample_document();
        let p = parts_for(&doc);
        let bytes = write_quad(&p);
        let o = read_quad(&bytes).unwrap();
        let ok = o.doc == doc && o.outlines == p.outlines && o.previews == p.previews && o.read_only.is_none();
        line("F01", "roundtrip", ok, &format!("{}B", bytes.len()));
        assert!(ok);
    }

    #[test]
    fn deterministic_bytes() {
        let p = sample_parts();
        let ok = write_quad(&p) == write_quad(&p);
        line("F02", "deterministic_bytes", ok, "same-doc-same-bytes");
        assert!(ok);
    }

    #[test]
    fn json_debug_roundtrip() {
        let doc = sample_document();
        let js = to_debug_json(&doc);
        let back = from_debug_json(&js).unwrap();
        let ok = back == doc;
        line("F03", "json_debug_roundtrip", ok, &format!("{}B", js.len()));
        assert!(ok);
    }

    #[test]
    fn migrate_0_1_to_0_2() {
        // Build an old-style 0.1 scene: rhythm has one `unit_mm` and no `pixel_locked`.
        let doc = sample_document();
        let mut v = Value::serialized(&doc).unwrap();
        let r = map_get_mut(&mut v, "rhythm").unwrap();
        if let Value::Map(m) = r {
            m.retain(|(k, _)| !matches!(k.as_text(), Some("h_unit_mm" | "v_unit_mm" | "pixel_locked")));
            m.push((Value::Text("unit_mm".into()), Value::Float(6.0)));
        }
        let mut p = sample_parts();
        p.format_version = "0.1.0".into();
        p.scene = to_cbor(&v);
        let o = read_quad(&write_quad(&p)).unwrap();
        let ok = o.migrated_from.as_ref().map(|v| v.to_string()) == Some("0.1.0".into())
            && o.doc.rhythm.h_unit_mm == 6.0
            && o.doc.rhythm.v_unit_mm == 6.0
            && o.doc.rhythm.pixel_locked
            && o.doc.layers == doc.layers;
        line("F04", "migrate_0_1_to_0_2", ok, "unit_mm->h/v_unit_mm+pixel_locked");
        assert!(ok);
    }

    #[test]
    fn unknown_optional_fields_survive_resave() {
        // A file from a newer minor version (0.3) with fields this build has never heard of,
        // at four levels: manifest, document, rhythm, and an object deep in a group.
        let doc = sample_document();
        let mut v = Value::serialized(&doc).unwrap();
        if let Value::Map(m) = &mut v {
            m.push((Value::Text("future_doc_field".into()), Value::Text("keep me".into())));
        }
        if let Value::Map(m) = map_get_mut(&mut v, "rhythm").unwrap() {
            m.push((Value::Text("local_rhythms".into()), Value::Array(vec![Value::Integer(3.into())])));
        }
        // layers[1].objects[0] is the header group; add a field to its first child.
        let layers = map_get_mut(&mut v, "layers").unwrap();
        let Value::Array(ls) = layers else { panic!() };
        let objs = map_get_mut(&mut ls[1], "objects").unwrap();
        let Value::Array(os) = objs else { panic!() };
        let shape = map_get_mut(&mut os[0], "shape").unwrap();
        let group = map_get_mut(shape, "Group").unwrap();
        let Value::Array(children) = map_get_mut(group, "children").unwrap() else { panic!() };
        if let Value::Map(m) = &mut children[0] {
            m.push((Value::Text("future_object_field".into()), Value::Bool(true)));
        }

        let mut p = sample_parts();
        p.format_version = "0.3.0".into();
        p.scene = to_cbor(&v);
        p.manifest_extra.insert("future_manifest_field".into(), Value::Integer(42.into()));
        let o = read_quad(&write_quad(&p)).unwrap();
        assert!(o.read_only.is_none());

        let again = resave(&o).unwrap();
        let o2 = read_quad(&again).unwrap();
        let Shape::Group { children } = &o2.doc.layers[1].objects[0].shape else { panic!() };
        let ok = o2.doc.extra.contains_key("future_doc_field")
            && o2.doc.rhythm.extra.contains_key("local_rhythms")
            && children[0].extra.contains_key("future_object_field")
            && o2.manifest.extra.contains_key("future_manifest_field")
            && o2.manifest.format_version == "0.3.0";
        line("F05", "unknown_optional_fields_survive_resave", ok, "4-levels");
        assert!(ok);
    }

    #[test]
    fn unknown_required_feature_opens_read_only() {
        let mut p = sample_parts();
        p.format_version = "0.3.0".into();
        p.required_features.push("boolean-ops".into());
        let o = read_quad(&write_quad(&p)).unwrap();
        let ok = matches!(&o.read_only, Some(ReadOnly::UnknownRequiredFeatures(f)) if f == &["boolean-ops"])
            && resave(&o).is_err();
        line("F06", "unknown_required_feature_read_only", ok, "boolean-ops");
        assert!(ok);
    }

    #[test]
    fn newer_major_opens_read_only() {
        let mut p = sample_parts();
        p.format_version = "1.0.0".into();
        let o = read_quad(&write_quad(&p)).unwrap();
        let ok = matches!(o.read_only, Some(ReadOnly::NewerMajor(_)));
        line("F07", "newer_major_read_only", ok, "1.0.0");
        assert!(ok);
    }

    #[test]
    fn corruption_is_detected() {
        let p = sample_parts();
        let mut bytes = write_quad(&p);
        // The preview PNG is stored uncompressed; flip a byte inside it.
        let png = &p.previews["supernote.a5x2"];
        let at = bytes.windows(png.len()).position(|w| w == png.as_slice()).unwrap() + png.len() / 2;
        bytes[at] ^= 0xFF;
        let ok = read_quad(&bytes).is_err();
        line("F08", "corruption_detected", ok, "flipped-byte");
        assert!(ok);
    }

    /// A stand-in for real fuzzing on stable Rust: mutate a valid file many ways; the reader must
    /// return Ok or Err, never panic or hang.
    #[test]
    fn mutation_smoke_no_panic() {
        let good = write_quad(&sample_parts());
        let mut seed: u64 = 0x9E3779B97F4A7C15;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let runs = 3000;
        let mut panics = 0;
        for _ in 0..runs {
            let mut b = good.clone();
            for _ in 0..(1 + rnd() % 8) {
                let i = (rnd() as usize) % b.len();
                b[i] = rnd() as u8;
            }
            if rnd() % 10 == 0 {
                b.truncate((rnd() as usize) % b.len());
            }
            if std::panic::catch_unwind(|| {
                let _ = read_quad(&b);
            })
            .is_err()
            {
                panics += 1;
            }
        }
        line("F09", "mutation_smoke_no_panic", panics == 0, &format!("runs={runs},panics={panics}"));
        assert_eq!(panics, 0);
    }

    #[test]
    fn outline_forms_roundtrip_and_exact() {
        let Ok(data) = std::fs::read(FONT_PATH) else {
            println!("SKIP outline test: {FONT_PATH} not found");
            return;
        };
        let font = Font::new(&data);
        let doc = planner_document();
        let f = build_forms(&font, &doc, &MANTA);
        let ok = from_cbor::<NaiveFile>(&to_cbor(&f.naive)).unwrap() == f.naive
            && from_cbor::<PackedFile>(&to_cbor(&f.packed)).unwrap() == f.packed
            && from_cbor::<TableFile>(&to_cbor(&f.table_inline)).unwrap() == f.table_inline
            && table_is_exact(&font, &doc);
        line("F10", "outline_forms_roundtrip_exact", ok, &format!("instances={}", f.instances));
        assert!(ok);
    }
}
