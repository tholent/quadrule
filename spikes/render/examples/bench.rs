//! Render + encode timing, median of 7 runs. `cargo run --release --example bench --features vello-cpu`
use spike_render::export::encode_png;
use spike_render::layout::layout;
use spike_render::scene::test_scene;
use spike_render::{tiny, A5X, MANTA};
use std::time::Instant;

fn median(mut f: impl FnMut()) -> f64 {
    let mut t: Vec<f64> = (0..7)
        .map(|_| {
            let s = Instant::now();
            f();
            s.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    t.sort_by(|a, b| a.partial_cmp(b).unwrap());
    t[3]
}

fn main() {
    for p in [MANTA, A5X] {
        let dl = layout(&test_scene(), &p, true);
        let ts = median(|| drop(tiny::render(&dl)));
        let img = tiny::render(&dl);
        let enc = median(|| drop(encode_png(&img, None)));
        #[cfg(feature = "vello-cpu")]
        let vc = format!("{:.1}", median(|| drop(spike_render::vcpu::render(&dl, true))));
        #[cfg(not(feature = "vello-cpu"))]
        let vc = "n/a".to_string();
        println!(
            "CHECK id=R10 name=cpu_render_time_ms result=info profile={} items={} tiny_skia_ms={ts:.1} vello_cpu_ms={vc} png_encode_ms={enc:.1} cmd=\"cargo run --release --example bench --features vello-cpu\"",
            p.id,
            dl.items.len()
        );
    }
}
