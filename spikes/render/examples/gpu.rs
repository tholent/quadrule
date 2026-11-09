//! Headless GPU check. `cargo run --release --example gpu --features gpu,vello-cpu`
use spike_render::checks::{check_lines, diff};
use spike_render::layout::layout;
use spike_render::scene::test_scene;
use spike_render::{tiny, vgpu, A5X, MANTA};

fn main() {
    let names = vgpu::adapter_names();
    println!("CHECK id=R11 name=wgpu_vulkan_adapters result={} adapters={:?} cmd=\"cargo run --release --example gpu --features gpu\"", if names.iter().any(|n| n.contains("V3D")) { "pass" } else { "fail" }, names);

    for (id, want) in [("R12", "V3D"), ("R13", "llvmpipe")] {
        let mut parts = Vec::new();
        let mut ok = true;
        for p in [MANTA, A5X] {
            let dl = layout(&test_scene(), &p, true);
            match vgpu::render(&dl, want) {
                Ok((img, ms)) => {
                    let (n, m) = diff(&tiny::render(&dl), &img);
                    let again = vgpu::render(&dl, want).map(|(b, _)| b == img).unwrap_or(false);
                    parts.push(format!("{}:warm_ms={ms:.1}:vs_tiny_skia_diff_px={n}:max_diff={m}:repeat_identical={again}", p.id));
                }
                Err(e) => {
                    ok = false;
                    parts.push(format!("{}:error={e:?}", p.id));
                }
            }
        }
        println!("CHECK id={id} name=vello_gpu_offscreen_{want} result={} {} cmd=\"cargo run --release --example gpu --features gpu\"", if ok { "pass" } else { "fail" }, parts.join(" "));
        if ok {
            let rr = |dl: &_| vgpu::render(dl, want).expect("render").0;
            let (n1, f1) = check_lines(&rr, true);
            let (n2, f2) = check_lines(&rr, false);
            let good = f1.is_empty() && f2.is_empty();
            println!("CHECK id={id}b name=vello_gpu_snapped_lines_exact_{want} result={} lines={} fails={:?} cmd=\"cargo run --release --example gpu --features gpu\"", if good { "pass" } else { "fail" }, n1 + n2, [f1, f2].concat());
        }
    }
}
