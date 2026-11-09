//! Pass/fail versions of the CHECK lines that must hold. Fonts: run fetch-fonts.sh first.
use cosmic_text::{Align, FontSystem};
use spike_text::*;
use std::sync::{Mutex, OnceLock};

/// One shared font system: building it per test is wasteful and FontSystem is !Sync.
fn fs() -> std::sync::MutexGuard<'static, FontSystem> {
    static FS: OnceLock<Mutex<FontSystem>> = OnceLock::new();
    FS.get_or_init(|| Mutex::new(font_system_with_bundled()))
        .lock()
        .unwrap()
}

#[test]
fn t03_line_breaks_do_not_depend_on_resolution() {
    let mut fs = fs();
    let s = sample_spec(Align::Left, 0.0, 400);
    let a = line_breaks(&layout(&mut fs, &s, 300.0 / MM_PER_IN));
    let b = line_breaks(&layout(&mut fs, &s, 226.0 / MM_PER_IN));
    let c = line_breaks(&layout(&mut fs, &s, 1.0));
    assert_eq!(a, b, "300 vs 226 PPI");
    assert_eq!(a, c, "300 PPI vs mm");
    assert!(a.len() > 1, "sample must wrap");
}

#[test]
fn t04_justified_lines_reach_the_right_edge() {
    let mut fs = fs();
    let b = layout(&mut fs, &sample_spec(Align::Justified, 0.0, 400), 1.0);
    let runs: Vec<_> = b.layout_runs().collect();
    for r in &runs[..runs.len() - 1] {
        let right = r
            .glyphs
            .iter()
            .filter(|g| !r.text[g.start..g.end].trim().is_empty())
            .map(|g| g.x + g.w)
            .fold(0.0f32, f32::max);
        assert!(
            (80.0 - right).abs() < 0.05,
            "line {} ends at {right} mm",
            r.line_i
        );
    }
}

#[test]
fn t05_letter_spacing_adds_em_fraction() {
    let mut fs = fs();
    let a = layout(&mut fs, &sample_spec(Align::Left, 0.0, 400), 1.0);
    let b = layout(&mut fs, &sample_spec(Align::Left, 0.05, 400), 1.0);
    let wa = a.layout_runs().next().unwrap().glyphs[1].w;
    let wb = b.layout_runs().next().unwrap().glyphs[1].w;
    assert!((wb - wa - 0.05 * pt_to_mm(10.0) as f32).abs() < 1e-3);
}

#[test]
fn t06_line_height_is_exact() {
    let mut fs = fs();
    let b = layout(&mut fs, &sample_spec(Align::Left, 0.0, 400), 1.0);
    let lh = b.layout_runs().next().unwrap().line_height;
    assert!((lh - (pt_to_mm(10.0) * 1.4) as f32).abs() < 1e-4);
    assert!((total_height(&b) - lh * b.layout_runs().count() as f32).abs() < 1e-3);
}

#[test]
fn t07_weight_drives_variable_axis() {
    let mut fs = fs();
    let sum = |b: &cosmic_text::Buffer| -> f32 {
        b.layout_runs().next().unwrap().glyphs[..10]
            .iter()
            .map(|g| g.w)
            .sum()
    };
    let r = layout(&mut fs, &sample_spec(Align::Left, 0.0, 400), 1.0);
    let b = layout(&mut fs, &sample_spec(Align::Left, 0.0, 700), 1.0);
    assert!(sum(&b) > sum(&r));
}

#[test]
fn t08_outlines_sit_where_layout_put_them() {
    let mut fs = fs();
    let b = layout(
        &mut fs,
        &sample_spec(Align::Left, 0.0, 400),
        300.0 / MM_PER_IN,
    );
    let outs = outlines(&mut fs, &b);
    assert!(
        outs.iter()
            .filter(|o| !o.path.elements().is_empty())
            .count()
            > 100
    );
    for w in outs.windows(2) {
        if (w[1].origin.y - w[0].origin.y).abs() < 1e-6 {
            let d = w[0].origin.x + w[0].layout_advance as f64 - w[1].origin.x;
            assert!(d.abs() < 1e-3, "pen chain broken by {d}");
        }
    }
    // Ink of the first inked glyph lies near its pen position (y-down page coords).
    let o = outs.iter().find(|o| !o.path.elements().is_empty()).unwrap();
    let bb = kurbo::Shape::bounding_box(&o.path);
    assert!(
        bb.y1 <= o.origin.y + 1.0 && bb.y0 < o.origin.y,
        "glyph should sit on its baseline"
    );
    assert!(bb.x0 >= o.origin.x - 5.0 && bb.x1 <= o.origin.x + o.layout_advance as f64 + 5.0);
}

#[test]
fn t10_plain_advances_agree_within_half_font_unit() {
    let mut fs = fs();
    for ppi in [300.0, 226.0] {
        let mut s = sample_spec(Align::Left, 0.0, 400);
        s.plain_shaping = true;
        let b = layout(&mut fs, &s, ppi / MM_PER_IN);
        let g = b.layout_runs().next().unwrap().glyphs[0].clone();
        let unit = font_unit(&mut fs, g.font_id, g.font_weight, g.font_size);
        for o in outlines(&mut fs, &b) {
            let d = (o.layout_advance - o.skrifa_advance).abs();
            assert!(
                d <= unit * 0.5 + 1e-4,
                "gid {} differs by {d} px at {ppi} PPI",
                o.glyph_id
            );
        }
    }
}

#[test]
fn t11_atkinson_is_used() {
    let mut fs = fs();
    let b = layout(&mut fs, &sample_spec(Align::Left, 0.0, 400), 1.0);
    for r in b.layout_runs() {
        for g in r.glyphs {
            assert_eq!(family_of(&fs, g.font_id), ATKINSON);
        }
    }
}

#[test]
fn t12b_explicit_noto_emoji_has_outlines() {
    let mut fs = fs();
    let s = TextBoxSpec {
        text: "📝✅🌙",
        family: NOTO_EMOJI,
        ..sample_spec(Align::Left, 0.0, 400)
    };
    let b = layout(&mut fs, &s, 300.0 / MM_PER_IN);
    let outs = outlines(&mut fs, &b);
    assert_eq!(outs.len(), 3);
    for o in outs {
        assert_eq!(family_of(&fs, o.font_id), NOTO_EMOJI);
        assert!(!o.path.elements().is_empty());
    }
}

#[test]
fn t13_substitute_keeps_class() {
    let fs = fs();
    let db = fs.db();
    for (fam, class) in [
        ("Liberation Serif", Class::Serif),
        ("Liberation Sans", Class::Sans),
        ("Liberation Mono", Class::Mono),
        (ATKINSON, Class::Sans),
    ] {
        let Some(id) = find_face(db, fam, 400) else {
            eprintln!("skip: {fam} not installed on this machine");
            continue;
        };
        let want = signature(db, id).unwrap();
        assert_eq!(want.class, class, "{fam} classified");
        let (pid, _) = substitute(db, &want, Some(fam)).unwrap();
        let got = signature(db, pid).unwrap();
        assert_ne!(got.family, fam);
        assert_eq!(got.class, class, "{fam} -> {}", got.family);
    }
}

#[test]
fn t13_exact_family_wins() {
    let fs = fs();
    let db = fs.db();
    let id = find_face(db, ATKINSON, 400).unwrap();
    let want = signature(db, id).unwrap();
    let (pid, score) = substitute(db, &want, None).unwrap();
    assert_eq!(score, 0);
    assert_eq!(signature(db, pid).unwrap().family, ATKINSON);
}

#[test]
fn units() {
    assert!((mm_to_px(25.4, 300.0) - 300.0).abs() < 1e-9);
    assert!((pt_to_mm(72.0) - 25.4).abs() < 1e-9);
    // napkin :308: 5 mm at 300 PPI is 59.06 px
    assert!((mm_to_px(5.0, 300.0) - 59.055).abs() < 1e-3);
}
