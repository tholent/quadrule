//! Minimal eframe window that shows the test page rendered by vello on egui's own wgpu device.
//! Compile check here; running needs a display.
//! `cargo run --release --example canvas --features canvas`
use eframe::egui;
use eframe::egui_wgpu;
use spike_render::layout::{layout, DisplayList, Item};
use spike_render::scene::test_scene;
use spike_render::MANTA;
use vello::kurbo::{Affine, Cap, Ellipse, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{Color, Fill};
use vello::wgpu;

struct App {
    tex_id: egui::TextureId,
    size: egui::Vec2,
}

fn vello_scene(dl: &DisplayList) -> vello::Scene {
    let mut s = vello::Scene::new();
    let g = |v: u8| Color::from_rgba8(v, v, v, 255);
    let id = Affine::IDENTITY;
    let st = |w: f64| Stroke::new(w).with_caps(Cap::Butt);
    s.fill(Fill::NonZero, id, g(dl.background), None, &Rect::new(0.0, 0.0, dl.width as f64, dl.height as f64));
    for it in &dl.items {
        match *it {
            Item::Rect { x0, y0, x1, y1, gray, .. } => s.fill(Fill::NonZero, id, g(gray), None, &Rect::new(x0, y0, x1, y1)),
            Item::Line { x0, y0, x1, y1, width, gray } => s.stroke(&st(width), id, g(gray), None, &Line::new((x0, y0), (x1, y1))),
            Item::Dot { cx, cy, r, gray } => s.fill(Fill::NonZero, id, g(gray), None, &Ellipse::new((cx, cy), (r, r), 0.0)),
            Item::RoundRect { x, y, w, h, radius, width, gray } => s.stroke(&st(width), id, g(gray), None, &RoundedRect::new(x, y, x + w, y + h, radius)),
            Item::Ellipse { cx, cy, rx, ry, width, gray } => s.stroke(&st(width), id, g(gray), None, &Ellipse::new((cx, cy), (rx, ry), 0.0)),
        }
    }
    s
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu backend");
        let dl = layout(&test_scene(), &MANTA, true);
        let (w, h) = (dl.width, dl.height);
        let tex = rs.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("page"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mut r = vello::Renderer::new(&rs.device, vello::RendererOptions { antialiasing_support: vello::AaSupport::area_only(), ..Default::default() }).expect("vello");
        let params = vello::RenderParams { base_color: Color::from_rgba8(255, 255, 255, 255), width: w, height: h, antialiasing_method: vello::AaConfig::Area };
        r.render_to_texture(&rs.device, &rs.queue, &vello_scene(&dl), &view, &params).expect("render");
        let tex_id = rs.renderer.write().register_native_texture(&rs.device, &view, wgpu::FilterMode::Nearest);
        App { tex_id, size: egui::vec2(w as f32, h as f32) }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::ScrollArea::both().show(ui, |ui| {
            ui.image((self.tex_id, self.size * 0.4));
        });
    }
}

fn main() -> eframe::Result {
    let opts = eframe::NativeOptions { renderer: eframe::Renderer::Wgpu, ..Default::default() };
    let _ = egui_wgpu::WgpuConfiguration::default();
    eframe::run_native("quadrule canvas spike", opts, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
