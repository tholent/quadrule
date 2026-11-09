//! GPU backend: vello on wgpu, rendered offscreen (no window) and read back.

use crate::layout::{DisplayList, Item};
use crate::Gray;
use vello::kurbo::{Affine, Cap, Ellipse, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{Color, Fill};
use vello::wgpu;
use vello::{AaConfig, RenderParams, Renderer, RendererOptions, Scene};

pub fn instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::VULKAN, ..wgpu::InstanceDescriptor::new_without_display_handle() })
}

/// Names of the Vulkan adapters wgpu can see.
pub fn adapter_names() -> Vec<String> {
    let inst = instance();
    pollster::block_on(inst.enumerate_adapters(wgpu::Backends::VULKAN)).iter().map(|a| a.get_info().name).collect()
}

fn scene(dl: &DisplayList) -> Scene {
    let mut s = Scene::new();
    let g = |v: u8| Color::from_rgba8(v, v, v, 255);
    let id = Affine::IDENTITY;
    s.fill(Fill::NonZero, id, g(dl.background), None, &Rect::new(0.0, 0.0, dl.width as f64, dl.height as f64));
    let st = |w: f64| Stroke::new(w).with_caps(Cap::Butt);
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

/// Render on the adapter whose name contains `want`. Returns the image and a timing in ms
/// for a second (warm) render.
pub fn render(dl: &DisplayList, want: &str) -> Result<(Gray, f64), String> {
    let inst = instance();
    let adapters = pollster::block_on(inst.enumerate_adapters(wgpu::Backends::VULKAN));
    let adapter = adapters.into_iter().find(|a| a.get_info().name.contains(want)).ok_or(format!("no adapter matching {want}"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .map_err(|e| format!("request_device: {e}"))?;
    let mut renderer = Renderer::new(&device, RendererOptions { antialiasing_support: vello::AaSupport::area_only(), ..Default::default() }).map_err(|e| format!("Renderer::new: {e}"))?;

    let (w, h) = (dl.width, dl.height);
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sc = scene(dl);
    let params = RenderParams { base_color: Color::from_rgba8(255, 255, 255, 255), width: w, height: h, antialiasing_method: AaConfig::Area };

    renderer.render_to_texture(&device, &queue, &sc, &view, &params).map_err(|e| format!("render: {e}"))?;
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("poll: {e}"))?;
    let t = std::time::Instant::now();
    renderer.render_to_texture(&device, &queue, &sc, &view, &params).map_err(|e| format!("render: {e}"))?;
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("poll: {e}"))?;
    let ms = t.elapsed().as_secs_f64() * 1000.0;

    // Read back. Rows must be padded to 256 bytes.
    let row = (w * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| format!("poll: {e}"))?;
    let bytes = slice.get_mapped_range().map_err(|e| format!("map range: {e:?}"))?;
    let mut data = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        let start = (y * row) as usize;
        data.extend(bytes[start..start + (w * 4) as usize].as_chunks::<4>().0.iter().map(|p| p[0]));
    }
    Ok((Gray { width: w, height: h, data }, ms))
}
