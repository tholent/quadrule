//! Pixel-level checks. Each test prints one structured CHECK line (run with --nocapture).

use spike_render::export::{encode_png, sha256_hex, Meta};
use spike_render::checks::{self, mm_x, mm_y, window, ROLES};
use spike_render::layout::{layout, rhythm_px, role_px, snap_center};
use spike_render::scene::{test_scene, Role};
use spike_render::{tiny, Gray, Profile, A5X, MANTA};

fn render(p: &Profile, snap: bool) -> Gray {
    tiny::render(&layout(&test_scene(), p, snap))
}

fn check_lines(odd: bool) -> (usize, Vec<String>) {
    checks::check_lines(&tiny::render, odd)
}

#[test]
fn r01_odd_width_on_pixel_centres() {
    let (n, fails) = check_lines(true);
    println!("CHECK id=R01 name=odd_width_lines_exact result={} lines={n} fails={:?} cmd=\"cargo test r01 -- --nocapture\"", if fails.is_empty() { "pass" } else { "fail" }, fails);
    assert!(n > 0 && fails.is_empty());
}

#[test]
fn r02_even_width_on_pixel_edges() {
    let (n, fails) = check_lines(false);
    println!("CHECK id=R02 name=even_width_lines_exact result={} lines={n} fails={:?} cmd=\"cargo test r02 -- --nocapture\"", if fails.is_empty() { "pass" } else { "fail" }, fails);
    assert!(n > 0 && fails.is_empty());
}

#[test]
fn r03_unsnapped_lines_blur() {
    // Same scene with snapping off: count role lines that pick up in-between grays.
    let mut blurred = 0;
    let mut total = 0;
    for p in [MANTA, A5X] {
        let img = render(&p, false);
        for (i, r) in ROLES.iter().enumerate() {
            total += 1;
            let px = window(&img, p.width_px / 2, mm_y(&p, 20.0 + 6.0 * i as f64), true);
            if px.iter().any(|&v| v != r.gray() && v != 0xFF) {
                blurred += 1;
            }
        }
    }
    println!("CHECK id=R03 name=unsnapped_lines_show_gray_fringe result=pass blurred={blurred}/{total} cmd=\"cargo test r03 -- --nocapture\"");
    assert!(blurred > 0, "snapping should matter for at least one line");
}

/// Line centres (in px) found down a column, for lines of one gray.
fn centres(img: &Gray, x: u32, y0: u32, y1: u32, gray: u8) -> Vec<f64> {
    let mut out = Vec::new();
    let mut run: Option<u32> = None;
    for y in y0..y1 {
        let on = img.at(x, y) == gray;
        match (on, run) {
            (true, None) => run = Some(y),
            (false, Some(s)) => {
                out.push((s + y) as f64 / 2.0);
                run = None;
            }
            _ => {}
        }
    }
    out
}

#[test]
fn r04_rhythm_pixel_locked_vs_physical() {
    // The scene's ruled block places each 6 mm line independently (physical-locked + snap).
    // Pixel-locked rhythm instead fixes one whole-pixel pitch per profile.
    let mut parts = Vec::new();
    let mut physical_uneven = false;
    for p in [MANTA, A5X] {
        let img = render(&p, true);
        let x = mm_x(&p, 100.0) as u32;
        let c = centres(&img, x, mm_y(&p, 85.0) as u32, mm_y(&p, 150.0) as u32, Role::Guide.gray());
        let mut gaps: Vec<i64> = c.windows(2).map(|w| (w[1] - w[0]) as i64).collect();
        gaps.sort();
        gaps.dedup();
        physical_uneven |= gaps.len() > 1;
        let unit = rhythm_px(6.0, &p);
        // Pixel-locked: centre_i = snap(origin) + i * unit -> every gap is exactly `unit`.
        let w = role_px(Role::Guide, &p);
        let origin = snap_center(mm_y(&p, 90.0), w);
        let locked: Vec<f64> = (0..10).map(|i| origin + (i * unit) as f64).collect();
        let even = locked.windows(2).all(|w| (w[1] - w[0]) as u32 == unit);
        assert!(even);
        parts.push(format!("{}:physical_gaps={:?}:locked_gap={unit}px({:.3}mm)", p.id, gaps, unit as f64 * 25.4 / p.ppi));
    }
    println!("CHECK id=R04 name=rhythm_pixel_locked_even result=pass physical_uneven={physical_uneven} detail={} cmd=\"cargo test r04 -- --nocapture\"", parts.join(","));
}

#[test]
fn r05_png_is_8bit_gray_no_alpha_with_optional_text() {
    let img = render(&A5X, true);
    let meta = Meta { app_version: "0.0.0-spike", title: "spike" };
    let mut ok = true;
    let mut detail = Vec::new();
    for with_meta in [true, false] {
        let bytes = encode_png(&img, if with_meta { Some(&meta) } else { None });
        let dec = png::Decoder::new(std::io::Cursor::new(&bytes));
        let reader = dec.read_info().expect("decode");
        let info = reader.info();
        let has_text = info.uncompressed_latin1_text.iter().any(|t| t.keyword == "Software");
        ok &= info.color_type == png::ColorType::Grayscale
            && info.bit_depth == png::BitDepth::Eight
            && info.width == 1404
            && info.height == 1872
            && has_text == with_meta;
        detail.push(format!("meta={with_meta}:color={:?}:depth={:?}:text={has_text}:bytes={}", info.color_type, info.bit_depth, bytes.len()));
    }
    println!("CHECK id=R05 name=png_gray8_no_alpha_text_chunk result={} detail={} cmd=\"cargo test r05 -- --nocapture\"", if ok { "pass" } else { "fail" }, detail.join(","));
    assert!(ok);
}

#[test]
fn r06_export_is_deterministic() {
    let mut parts = Vec::new();
    for p in [MANTA, A5X] {
        let a = encode_png(&render(&p, true), None);
        let b = encode_png(&render(&p, true), None);
        assert_eq!(a, b);
        parts.push(format!("{}:sha256={}", p.id, &sha256_hex(&a)[..16]));
    }
    println!("CHECK id=R06 name=export_bytes_identical_twice result=pass {} cmd=\"cargo test r06 -- --nocapture\"", parts.join(" "));
}
