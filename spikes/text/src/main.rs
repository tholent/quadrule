//! Prints one structured CHECK line per measurement. Run:
//!   cargo run --release --manifest-path spikes/text/Cargo.toml
use cosmic_text::{Align, FontSystem, fontdb};
use spike_text::*;
use std::time::Instant;

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn pf(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn walk(dir: &std::path::Path, n: &mut usize) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            *n += 1;
            let _ = e.metadata().map(|m| m.modified());
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                walk(&e.path(), n);
            }
        }
    }
}

fn main() {
    // T01: first enumeration in this process.
    let t = Instant::now();
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    let first = ms(t);
    let mut fams: Vec<&str> = db
        .faces()
        .filter_map(|f| f.families.first().map(|(n, _)| n.as_str()))
        .collect();
    fams.sort();
    fams.dedup();
    println!(
        "CHECK id=T01 name=fontdb_enum result=pass value={first:.1}ms faces={} families={} cmd=\"fontdb::Database::load_system_fonts\"",
        db.len(),
        fams.len()
    );

    // T02: re-scan cost (napkin :353 refreshes on focus regain) vs a cheap change check.
    let rescans: Vec<f64> = (0..5)
        .map(|_| {
            let t = Instant::now();
            let mut d = fontdb::Database::new();
            d.load_system_fonts();
            ms(t)
        })
        .collect();
    let t = Instant::now();
    let mut entries = 0;
    let home = std::env::var("HOME").unwrap_or_default();
    for d in [
        "/usr/share/fonts".to_string(),
        "/usr/local/share/fonts".into(),
        format!("{home}/.local/share/fonts"),
        format!("{home}/.fonts"),
    ] {
        walk(std::path::Path::new(&d), &mut entries);
    }
    let stat_ms = ms(t);
    let t = Instant::now();
    let _fs = FontSystem::new();
    let fs_new = ms(t);
    println!(
        "CHECK id=T02 name=fontdb_rescan result=pass value={:.1}ms stat_walk={stat_ms:.2}ms entries={entries} fontsystem_new={fs_new:.1}ms cmd=\"median of 5 full re-scans\"",
        median(rescans)
    );

    let mut fs = font_system_with_bundled();

    // T03: same box at 300 PPI, 226 PPI and directly in mm: identical line breaks?
    let s = sample_spec(Align::Left, 0.0, 400);
    let l300 = line_breaks(&layout(&mut fs, &s, 300.0 / MM_PER_IN));
    let l226 = line_breaks(&layout(&mut fs, &s, 226.0 / MM_PER_IN));
    let lmm = line_breaks(&layout(&mut fs, &s, 1.0));
    println!(
        "CHECK id=T03 name=wrap_scale_invariant result={} lines300={} lines226={} lines_mm={} box=80mm size=10pt",
        pf(l300 == l226 && l300 == lmm),
        l300.len(),
        l226.len(),
        lmm.len()
    );

    // T04: justify: every line but the last ends at the box edge (ignoring the trailing space).
    let bj = layout(&mut fs, &sample_spec(Align::Justified, 0.0, 400), 1.0);
    let runs: Vec<_> = bj.layout_runs().collect();
    let worst = runs[..runs.len() - 1]
        .iter()
        .map(|r| {
            let right = r
                .glyphs
                .iter()
                .filter(|g| !r.text[g.start..g.end].trim().is_empty())
                .map(|g| g.x + g.w)
                .fold(0.0f32, f32::max);
            (80.0 - right).abs()
        })
        .fold(0.0f32, f32::max);
    println!(
        "CHECK id=T04 name=justify result={} worst_gap_mm={worst:.4} lines={}",
        pf(worst < 0.05),
        runs.len()
    );

    // T05: letter spacing 0.05 em adds 0.05 x size to each glyph's advance.
    let b0 = layout(&mut fs, &sample_spec(Align::Left, 0.0, 400), 1.0);
    let bl = layout(&mut fs, &sample_spec(Align::Left, 0.05, 400), 1.0);
    let w0 = b0.layout_runs().next().unwrap().glyphs[1].w;
    let wl = bl.layout_runs().next().unwrap().glyphs[1].w;
    let expect = 0.05 * pt_to_mm(10.0) as f32;
    println!(
        "CHECK id=T05 name=letter_spacing result={} added_mm={:.4} expected_mm={expect:.4} lines0={} lines_ls={}",
        pf((wl - w0 - expect).abs() < 1e-3),
        wl - w0,
        b0.layout_runs().count(),
        bl.layout_runs().count()
    );

    // T06: line height is exactly what was asked.
    let lh = b0.layout_runs().next().unwrap().line_height;
    let want = (pt_to_mm(10.0) * 1.4) as f32;
    println!(
        "CHECK id=T06 name=line_height result={} value_mm={lh:.4} expected_mm={want:.4}",
        pf((lh - want).abs() < 1e-4)
    );

    // T07: weight drives the variable font's wght axis (bold is wider).
    let bb = layout(&mut fs, &sample_spec(Align::Left, 0.0, 700), 1.0);
    let sum10 = |b: &cosmic_text::Buffer| -> f32 {
        b.layout_runs().next().unwrap().glyphs[..10]
            .iter()
            .map(|g| g.w)
            .sum()
    };
    let (a, b) = (sum10(&b0), sum10(&bb));
    println!(
        "CHECK id=T07 name=variable_weight result={} first10_w400_mm={a:.3} first10_w700_mm={b:.3}",
        pf(b > a)
    );

    for (label, ppi) in [("300", 300.0), ("226", 226.0)] {
        // T08: outlines placed by layout: each glyph's pen position = previous + its advance.
        let bk = layout(
            &mut fs,
            &sample_spec(Align::Left, 0.0, 400),
            ppi / MM_PER_IN,
        );
        let t = Instant::now();
        let outs = outlines(&mut fs, &bk);
        let out_ms = ms(t);
        let inked = outs
            .iter()
            .filter(|o| !o.path.elements().is_empty())
            .count();
        let elems: usize = outs.iter().map(|o| o.path.elements().len()).sum();
        let mut worst_pos = 0.0f64;
        for w in outs.windows(2) {
            if (w[1].origin.y - w[0].origin.y).abs() < 1e-6 {
                let d = (w[0].origin.x + w[0].layout_advance as f64 - w[1].origin.x).abs();
                worst_pos = worst_pos.max(d);
            }
        }
        println!(
            "CHECK id=T08 name=outline_extract ppi={label} result={} glyphs={} inked={inked} path_elements={elems} pen_chain_err_px={worst_pos:.6} value={out_ms:.2}ms",
            pf(inked > 0 && worst_pos < 1e-3),
            outs.len()
        );
        // T09: with kerning/ligatures on, layout advances differ from raw skrifa advances
        // exactly where the font kerns; count those glyphs (informational).
        let kerned = outs
            .iter()
            .filter(|o| (o.layout_advance - o.skrifa_advance).abs() > 0.05)
            .count();
        println!(
            "CHECK id=T09 name=kerning_in_layout ppi={label} result=pass kerned_glyphs={kerned} of={}",
            outs.len()
        );

        // T10: with kerning/ligatures/alternates off, advances agree within half a font unit.
        let mut plain = sample_spec(Align::Left, 0.0, 400);
        plain.plain_shaping = true;
        let bp = layout(&mut fs, &plain, ppi / MM_PER_IN);
        let outs = outlines(&mut fs, &bp);
        let g0 = bp.layout_runs().next().unwrap().glyphs[0].clone();
        let unit = font_unit(&mut fs, g0.font_id, g0.font_weight, g0.font_size);
        let worst = outs
            .iter()
            .map(|o| (o.layout_advance - o.skrifa_advance).abs())
            .fold(0.0f32, f32::max);
        println!(
            "CHECK id=T10 name=advance_agree_plain ppi={label} result={} worst_px={worst:.5} worst_font_units={:.3} font_unit_px={unit:.5}",
            pf(worst <= unit * 0.5 + 1e-4),
            worst / unit
        );
    }

    // T11: Atkinson is the face that actually got used.
    let mut ids: Vec<String> = b0
        .layout_runs()
        .flat_map(|r| r.glyphs.iter().map(|g| family_of(&fs, g.font_id)))
        .collect();
    ids.sort();
    ids.dedup();
    println!(
        "CHECK id=T11 name=atkinson_used result={} families=\"{}\"",
        pf(ids == [ATKINSON]),
        ids.join("|")
    );

    // T12a: emoji inside Atkinson text, left to automatic fallback.
    let text = "Notes 📝 done ✅ moon 🌙";
    let emo = TextBoxSpec {
        text,
        ..sample_spec(Align::Left, 0.0, 400)
    };
    let be = layout(&mut fs, &emo, 300.0 / MM_PER_IN);
    let run = be.layout_runs().next().unwrap();
    let picks: Vec<String> = run
        .glyphs
        .iter()
        .filter(|g| !run.text[g.start..g.end].is_ascii())
        .map(|g| {
            format!(
                "{}={}",
                run.text[g.start..g.end]
                    .chars()
                    .next()
                    .map(|c| format!("U+{:04X}", c as u32))
                    .unwrap(),
                family_of(&fs, g.font_id).replace(' ', "_")
            )
        })
        .collect();
    let all_noto = picks.iter().all(|p| p.ends_with("Noto_Emoji"));
    println!(
        "CHECK id=T12a name=emoji_auto_fallback result={} picks=\"{}\"",
        pf(all_noto),
        picks.join(",")
    );

    // T12b: emoji object with its family set to Noto Emoji explicitly (napkin §7.4 emoji object).
    let emo = TextBoxSpec {
        text: "📝✅🌙",
        family: NOTO_EMOJI,
        ..sample_spec(Align::Left, 0.0, 400)
    };
    let be = layout(&mut fs, &emo, 300.0 / MM_PER_IN);
    let outs = outlines(&mut fs, &be);
    let ok = outs.len() == 3
        && outs
            .iter()
            .all(|o| family_of(&fs, o.font_id) == NOTO_EMOJI && !o.path.elements().is_empty());
    println!(
        "CHECK id=T12b name=emoji_explicit_noto result={} glyphs={} all_noto_inked={ok}",
        pf(ok),
        outs.len()
    );

    // T13: leave-one-out substitution: hide an installed family, ask for it, see what comes back.
    let db = fs.db();
    for (fam, w) in [
        ("Liberation Serif", 400),
        ("Liberation Sans", 400),
        ("Liberation Mono", 400),
        ("DejaVu Sans", 700),
        (ATKINSON, 400),
    ] {
        let id = find_face(db, fam, w).expect("installed");
        let want = signature(db, id).expect("signature");
        let t = Instant::now();
        let (pid, score) = substitute(db, &want, Some(fam)).expect("some face");
        let el = ms(t);
        let got = signature(db, pid).unwrap();
        println!(
            "CHECK id=T13 name=substitute hidden=\"{fam}\" class={:?} result={} picked=\"{}\" picked_class={:?} weight={} score={score} value={el:.1}ms",
            want.class,
            pf(got.class == want.class),
            got.family,
            got.class,
            got.weight
        );
    }

    println!(
        "CHECK id=T14 name=text_on_macos result=owed reason=\"no macOS machine in this session\""
    );
    println!(
        "CHECK id=T15 name=text_on_windows result=owed reason=\"no Windows machine in this session\""
    );
}
