//! Measures outline storage size and read/write speed on a text-heavy planner.
//! Run: cargo run --release --bin measure

use flate2::{Compression, write::DeflateEncoder};
use spike_format::container::*;
use spike_format::model::Document;
use spike_format::outlines::*;
use std::io::Write;
use std::time::Instant;

fn deflated(b: &[u8]) -> usize {
    let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
    e.write_all(b).unwrap();
    e.finish().unwrap().len()
}

fn median_ms<F: FnMut()>(runs: usize, mut f: F) -> f64 {
    let mut v: Vec<f64> = (0..runs)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[runs / 2]
}

fn main() {
    let data = std::fs::read(FONT_PATH).expect("font missing; see FINDINGS.md");
    let font = Font::new(&data);
    let doc = planner_document();
    let glyphs = glyph_count(&doc);
    let texts = text_objects(&doc).len();
    println!("CHECK id=F11 name=planner_size result=info value=texts={texts},glyphs={glyphs} cmd=\"cargo run --release --bin measure\"");

    let m = build_forms(&font, &doc, &MANTA);
    let a = build_forms(&font, &doc, &A5X);
    println!(
        "CHECK id=F12 name=glyph_reuse result=info value=instances_per_profile={},distinct_glyphs={}",
        m.instances, m.distinct_glyphs
    );

    let row = |id: &str, name: &str, parts: Vec<Vec<u8>>| {
        let raw: usize = parts.iter().map(|p| p.len()).sum();
        let z: usize = parts.iter().map(|p| deflated(p)).sum();
        println!("CHECK id={id} name={name} result=info value=raw={raw}B,deflated={z}B,profiles=2");
        (raw, z)
    };
    let naive = row("F13", "outlines_A_naive_per_instance", vec![to_cbor(&m.naive), to_cbor(&a.naive)]);
    let packed = row("F14", "outlines_B_packed_per_instance", vec![to_cbor(&m.packed), to_cbor(&a.packed)]);
    let inline = row("F15", "outlines_C1_table_per_profile", vec![to_cbor(&m.table_inline), to_cbor(&a.table_inline)]);
    let shared = row(
        "F16",
        "outlines_C2_one_shared_table",
        vec![to_cbor(&m.table_shared), to_cbor(&a.table_shared), to_cbor(&m.shared_table)],
    );
    let _ = (naive, packed, inline, shared);

    let scene = to_cbor(&doc);
    let json = to_debug_json(&doc);
    println!(
        "CHECK id=F17 name=scene_size result=info value=cbor={}B,cbor_deflated={}B,json={}B",
        scene.len(),
        deflated(&scene),
        json.len()
    );

    // A whole file: scene, outlines in form C1 (one file per profile, as §12.1 lays out),
    // and two 240x320 preview thumbnails.
    let mut p = Parts::from_document(&doc);
    p.outlines.insert(MANTA.id.into(), to_cbor(&m.table_inline));
    p.outlines.insert(A5X.id.into(), to_cbor(&a.table_inline));
    p.previews.insert(MANTA.id.into(), tiny_png(240, 320, 0xF0));
    p.previews.insert(A5X.id.into(), tiny_png(240, 320, 0xF0));
    let file = write_quad(&p);
    println!("CHECK id=F18 name=quad_file_size result=info value={}B,form=C1", file.len());

    let write_ms = median_ms(21, || {
        let mut p = Parts::from_document(&doc);
        p.outlines.insert(MANTA.id.into(), to_cbor(&m.table_inline));
        p.outlines.insert(A5X.id.into(), to_cbor(&a.table_inline));
        let _ = write_quad(&p);
    });
    let read_ms = median_ms(21, || {
        let o = read_quad(&file).unwrap();
        for b in o.outlines.values() {
            let _: TableFile = from_cbor(b).unwrap();
        }
    });
    let scene_ms = median_ms(21, || {
        let _: Document = from_cbor(&scene).unwrap();
    });
    println!(
        "CHECK id=F19 name=speed_pi5_release result=info value=write={write_ms:.2}ms,read={read_ms:.2}ms,scene_decode={scene_ms:.2}ms,runs=21,median"
    );
}
