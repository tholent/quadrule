//! vello_cpu vs tiny-skia on the same display list. Run with --features vello-cpu --nocapture.
#![cfg(feature = "vello-cpu")]

use spike_render::checks::{check_lines, diff};
use spike_render::export::{encode_png, sha256_hex};
use spike_render::layout::layout;
use spike_render::scene::test_scene;
use spike_render::{tiny, vcpu, A5X, MANTA};

#[test]
fn r07_vello_cpu_snapped_lines_exact() {
    let r = |dl: &_| vcpu::render(dl, true);
    let (n1, f1) = check_lines(&r, true);
    let (n2, f2) = check_lines(&r, false);
    let ok = f1.is_empty() && f2.is_empty();
    println!("CHECK id=R07 name=vello_cpu_snapped_lines_exact result={} lines={} fails={:?} cmd=\"cargo test --features vello-cpu r07 -- --nocapture\"", if ok { "pass" } else { "fail" }, n1 + n2, [f1, f2].concat());
    assert!(ok);
}

#[test]
fn r08_vello_cpu_vs_tiny_skia_diff() {
    let mut parts = Vec::new();
    for p in [MANTA, A5X] {
        let dl = layout(&test_scene(), &p, true);
        let (n, m) = diff(&tiny::render(&dl), &vcpu::render(&dl, true));
        let total = (p.width_px * p.height_px) as f64;
        parts.push(format!("{}:differing_px={n}({:.3}%):max_diff={m}", p.id, 100.0 * n as f64 / total));
    }
    println!("CHECK id=R08 name=vello_cpu_vs_tiny_skia_pixel_diff result=info {} cmd=\"cargo test --features vello-cpu r08 -- --nocapture\"", parts.join(" "));
}

#[test]
fn r09_vello_cpu_simd_vs_scalar_identical() {
    // If the SIMD and scalar paths agree byte-for-byte here, that is weak evidence the
    // output does not depend on the CPU (the real test is x86 vs ARM in CI).
    let mut parts = Vec::new();
    let mut same = true;
    for p in [MANTA, A5X] {
        let dl = layout(&test_scene(), &p, true);
        let a = vcpu::render(&dl, true);
        let b = vcpu::render(&dl, false);
        let (n, m) = diff(&a, &b);
        same &= n == 0;
        let again = encode_png(&vcpu::render(&dl, true), None) == encode_png(&a, None);
        same &= again;
        parts.push(format!("{}:simd_vs_scalar_diff_px={n}:max={m}:repeat_identical={again}:sha256={}", p.id, &sha256_hex(&encode_png(&a, None))[..16]));
    }
    println!("CHECK id=R09 name=vello_cpu_simd_vs_scalar_and_repeat result={} {} cmd=\"cargo test --features vello-cpu r09 -- --nocapture\"", if same { "pass" } else { "fail" }, parts.join(" "));
}
