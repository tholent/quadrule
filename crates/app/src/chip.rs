// Copyright 2026 Chris Wells
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The graphics-chip canvas renderer (rung 3 bbq Q4): vello 0.11 on egui's own wgpu device (the
//! same wgpu 30.0.1, spike R14), drawing into a texture egui shows. It never makes an export
//! (ADR-003 item 3). Ideas from `spikes/render/examples/canvas.rs`.

use eframe::egui;
use eframe::egui_wgpu;
use eframe::wgpu;
use quadrule_layout::{DisplayList, Item};
use std::time::Instant;
use vello::kurbo::{Affine, Cap, Join, Line, Rect, Stroke};
use vello::peniko::{Color, Fill, Mix};

/// The texture the page is drawn into, kept while its size stays the same.
struct Target {
    size: (u32, u32),
    texture: wgpu::Texture,
    id: egui::TextureId,
}

/// Vello on the window's graphics chip.
pub struct Chip {
    device: wgpu::Device,
    queue: wgpu::Queue,
    egui: std::sync::Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>,
    /// Started on the first frame, so the window opens without waiting for vello's shaders.
    vello: Option<vello::Renderer>,
    target: Option<Target>,
}

impl Chip {
    /// The renderer for egui's device. Vello itself starts on the first frame (building its
    /// shaders takes seconds on the Pi), which the fallback rule does not judge; if it cannot
    /// start, that frame fails and the canvas falls back.
    pub fn new(rs: &egui_wgpu::RenderState) -> Self {
        Chip {
            device: rs.device.clone(),
            queue: rs.queue.clone(),
            egui: rs.renderer.clone(),
            vello: None,
            target: None,
        }
    }

    /// A texture of `size` to draw into, and its id in egui: the last one when the size is the
    /// same, else a new one under the same id.
    fn target(&mut self, size: (u32, u32)) -> Result<(wgpu::TextureView, egui::TextureId), String> {
        if let Some(t) = &self.target
            && t.size == size
        {
            return Ok((t.texture.create_view(&Default::default()), t.id));
        }
        let max = self.device.limits().max_texture_dimension_2d;
        if size.0 > max || size.1 > max {
            return Err(format!("{}×{} is over the chip's {max} px", size.0, size.1));
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("quadrule-chip-canvas"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let mut egui = self.egui.write();
        let id = match self.target.take() {
            Some(old) => {
                egui.update_egui_texture_from_wgpu_texture(
                    &self.device,
                    &view,
                    wgpu::FilterMode::Nearest,
                    old.id,
                );
                old.id
            }
            None => egui.register_native_texture(&self.device, &view, wgpu::FilterMode::Nearest),
        };
        self.target = Some(Target { size, texture, id });
        Ok((view, id))
    }
}

impl quadrule_ui::ChipRenderer for Chip {
    fn render(&mut self, dl: &DisplayList) -> Result<(egui::TextureId, f64), String> {
        let start = Instant::now();
        if self.vello.is_none() {
            let v = vello::Renderer::new(
                &self.device,
                vello::RendererOptions {
                    antialiasing_support: vello::AaSupport::area_only(),
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?;
            self.vello = Some(v);
        }
        let (view, id) = self.target((dl.width, dl.height))?;
        let params = vello::RenderParams {
            base_color: gray(dl.background),
            width: dl.width,
            height: dl.height,
            antialiasing_method: vello::AaConfig::Area,
        };
        self.vello
            .as_mut()
            .expect("started above")
            .render_to_texture(&self.device, &self.queue, &scene(dl), &view, &params)
            .map_err(|e| e.to_string())?;
        // Wait for the chip, so the time measured is the whole frame (the fallback rule).
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        Ok((id, start.elapsed().as_secs_f64() * 1000.0))
    }
}

fn gray(v: u8) -> Color {
    Color::from_rgba8(v, v, v, 255)
}

/// The display list as a vello scene.
fn scene(dl: &DisplayList) -> vello::Scene {
    let mut s = vello::Scene::new();
    let id = Affine::IDENTITY;
    for item in &dl.items {
        match item {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray: g,
            } => s.fill(
                Fill::NonZero,
                id,
                gray(*g),
                None,
                &Rect::new(*x0, *y0, *x1, *y1),
            ),
            Item::Stroke {
                x0,
                y0,
                x1,
                y1,
                width,
                gray: g,
            } => s.stroke(
                &Stroke::new(*width).with_caps(Cap::Butt),
                id,
                gray(*g),
                None,
                &Line::new((*x0, *y0), (*x1, *y1)),
            ),
            Item::Fill { path, gray: g } => s.fill(Fill::NonZero, id, gray(*g), None, path),
            Item::StrokePath {
                path,
                width,
                gray: g,
                cap,
            } => s.stroke(
                &Stroke::new(*width)
                    .with_caps(quadrule_layout::kurbo_cap(*cap))
                    .with_join(Join::Miter),
                id,
                gray(*g),
                None,
                path,
            ),
            Item::BeginAlpha { alpha, bounds } => {
                let (x0, y0, x1, y1) = *bounds;
                s.push_layer(
                    Fill::NonZero,
                    Mix::Normal,
                    f32::from(*alpha) / 255.0,
                    id,
                    &Rect::new(
                        x0.floor() - 1.0,
                        y0.floor() - 1.0,
                        x1.ceil() + 1.0,
                        y1.ceil() + 1.0,
                    ),
                )
            }
            Item::EndAlpha => s.pop_layer(),
        }
    }
    s
}
