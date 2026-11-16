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

/// A texture the page is drawn into, kept while its size stays the same.
struct Target {
    size: (u32, u32),
    texture: wgpu::Texture,
    id: egui::TextureId,
}

/// What the last frame held, to find what has not changed since.
struct Last {
    size: (u32, u32),
    background: u8,
    items: Vec<Item>,
}

/// The unchanged lower part of the page, drawn once into a texture of its own that vello then
/// draws as an image under what has changed.
struct Base {
    /// How many items from the bottom of the stack it holds.
    len: usize,
    size: (u32, u32),
    texture: wgpu::Texture,
    image: vello::peniko::ImageData,
}

/// Fewest items worth keeping apart.
const SPLIT_MIN_ITEMS: usize = 6;
/// Frames timed of each way before choosing, and frames the choice then holds.
const PROBE_FRAMES: usize = 3;
const HOLD_FRAMES: u32 = 100;

/// Which way of drawing a page that can be kept apart is faster on this chip, found by timing
/// both: drawing the unchanged lower part as an image costs something of its own, so it pays only
/// when the lower part is heavy enough, and that is not worked out from the page but measured.
#[derive(Default)]
struct Pace {
    /// Times (ms) of frames that kept a base, and of whole frames when a base could have been kept.
    split: Vec<f64>,
    whole: Vec<f64>,
    /// The way chosen, and how many frames it still holds before both are timed again.
    use_split: bool,
    hold: u32,
}

impl Pace {
    /// Whether the next frame that could keep a base should.
    fn split_next(&mut self) -> bool {
        if self.hold > 0 {
            self.hold -= 1;
            return self.use_split;
        }
        // Timing both: the kept way first, then the whole.
        if self.split.len() < PROBE_FRAMES {
            return true;
        }
        if self.whole.len() < PROBE_FRAMES {
            return false;
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        self.use_split = mean(&self.split) < mean(&self.whole) * 0.9;
        self.hold = HOLD_FRAMES;
        self.split.clear();
        self.whole.clear();
        self.hold -= 1;
        self.use_split
    }
}

/// Vello on the window's graphics chip.
///
/// Its time grows with what is drawn, so a frame that changes only what is on top (a shape being
/// drawn, an object being dragged) draws the unchanged lower part of the page once, into a texture
/// of its own that later frames reuse, and only the rest again, over it as an image. The picture is
/// the one drawing everything in order makes.
pub struct Chip {
    device: wgpu::Device,
    queue: wgpu::Queue,
    egui: std::sync::Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>,
    /// Started on the first frame, so the window opens without waiting for vello's shaders.
    vello: Option<vello::Renderer>,
    /// The texture shown.
    target: Option<Target>,
    /// The kept lower part, when there is one.
    base: Option<Base>,
    last: Option<Last>,
    pace: Pace,
    /// Whole-page-sized draws the last frame made (see `ChipRenderer::last_frame_draws`).
    draws: u32,
    /// The small texture a changed patch is drawn into before it is copied over the page.
    patch_tex: Option<((u32, u32), wgpu::Texture)>,
    /// Frames drawn as a patch (the tests count them).
    #[cfg(test)]
    patches: u32,
    /// When the patch way and the stored lower part are used (the tests lower these).
    tune: Tune,
}

/// Sizes that decide when a frame is drawn as a patch (rung 11b: typing on the chip).
#[derive(Clone, Copy)]
struct Tune {
    /// Fewest path elements on the whole page worth drawing a patch for: under this a whole draw
    /// is quick anyway.
    patch_min: usize,
    /// Most path elements below the patch drawn straight into it; more is kept as an image.
    direct_max: usize,
}

impl Default for Tune {
    fn default() -> Self {
        Tune {
            patch_min: PATCH_MIN_WEIGHT,
            direct_max: DIRECT_MAX_WEIGHT,
        }
    }
}

/// See [`Tune`].
const PATCH_MIN_WEIGHT: usize = 3000;
const DIRECT_MAX_WEIGHT: usize = 1500;
/// A patch over this share of the page (percent) is not worth it: the page is drawn the old ways.
const PATCH_MAX_PERCENT: u64 = 60;

impl Chip {
    /// The renderer for egui's device. Vello itself starts on the first frame (building its
    /// shaders takes seconds on the Pi), which the fallback rule does not judge; if it cannot
    /// start, that frame fails and the canvas falls back.
    pub fn new(rs: &egui_wgpu::RenderState) -> Self {
        Self::with_parts(rs.device.clone(), rs.queue.clone(), rs.renderer.clone())
    }

    /// The renderer for a device, its queue and the egui renderer that shows its texture.
    fn with_parts(
        device: wgpu::Device,
        queue: wgpu::Queue,
        egui: std::sync::Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>,
    ) -> Self {
        Chip {
            device,
            queue,
            egui,
            vello: None,
            target: None,
            base: None,
            last: None,
            pace: Pace::default(),
            draws: 1,
            patch_tex: None,
            #[cfg(test)]
            patches: 0,
            tune: Tune::default(),
        }
    }

    /// A texture of `size` to draw into in `slot`, and its id in egui: the last one when the size
    /// is the same, else a new one under the same id.
    fn target_in(
        device: &wgpu::Device,
        egui: &egui::mutex::RwLock<egui_wgpu::Renderer>,
        slot: &mut Option<Target>,
        size: (u32, u32),
    ) -> Result<(wgpu::TextureView, egui::TextureId), String> {
        if let Some(t) = slot
            && t.size == size
        {
            return Ok((t.texture.create_view(&Default::default()), t.id));
        }
        let max = device.limits().max_texture_dimension_2d;
        if size.0 > max || size.1 > max {
            return Err(format!("{}×{} is over the chip's {max} px", size.0, size.1));
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let mut egui = egui.write();
        let id = match slot.take() {
            Some(old) => {
                egui.update_egui_texture_from_wgpu_texture(
                    device,
                    &view,
                    wgpu::FilterMode::Nearest,
                    old.id,
                );
                old.id
            }
            None => egui.register_native_texture(device, &view, wgpu::FilterMode::Nearest),
        };
        *slot = Some(Target { size, texture, id });
        Ok((view, id))
    }

    fn draw(
        &mut self,
        sc: &vello::Scene,
        view: &wgpu::TextureView,
        size: (u32, u32),
        base_color: Color,
    ) -> Result<(), String> {
        let params = vello::RenderParams {
            base_color,
            width: size.0,
            height: size.1,
            antialiasing_method: vello::AaConfig::Area,
        };
        self.vello
            .as_mut()
            .expect("started by render")
            .render_to_texture(&self.device, &self.queue, sc, view, &params)
            .map_err(|e| e.to_string())
    }
}

impl Chip {
    /// Lets go of the kept lower part.
    fn drop_base(&mut self) {
        if let Some(b) = self.base.take()
            && let Some(v) = self.vello.as_mut()
        {
            v.unregister_texture(b.image);
        }
    }

    /// Draws `items` into the kept lower part (a new texture when the size changed).
    fn make_base(
        &mut self,
        items: &[Item],
        size: (u32, u32),
        background: Color,
    ) -> Result<(), String> {
        if self.base.as_ref().is_some_and(|b| b.size != size) {
            self.drop_base();
        }
        if self.base.is_none() {
            let max = self.device.limits().max_texture_dimension_2d;
            if size.0 > max || size.1 > max {
                return Err(format!("{}×{} is over the chip's {max} px", size.0, size.1));
            }
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("quadrule-chip-base"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let image = self
                .vello
                .as_mut()
                .expect("started by render")
                .register_texture(texture.clone());
            self.base = Some(Base {
                len: 0,
                size,
                texture,
                image,
            });
        }
        let view = self
            .base
            .as_ref()
            .expect("made above")
            .texture
            .create_view(&Default::default());
        self.draw(&scene(items), &view, size, background)?;
        if let Some(b) = self.base.as_mut() {
            b.len = items.len();
        }
        Ok(())
    }
}

/// How many items from the bottom are the same in both lists.
fn common_prefix(a: &[Item], b: &[Item]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

/// The largest `n` up to `limit` at which no opacity group is open.
fn closed_cut(items: &[Item], limit: usize) -> usize {
    let (mut depth, mut cut) = (0usize, 0usize);
    for (i, it) in items.iter().take(limit).enumerate() {
        match it {
            Item::BeginAlpha { .. } => depth += 1,
            Item::EndAlpha => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 {
            cut = i + 1;
        }
    }
    cut
}

/// A box in device (here screen) pixels: left, top, right, bottom.
type Bounds = (f64, f64, f64, f64);

/// The pixels an item can touch, with room for anti-aliasing; none for an item that draws nothing.
fn item_bounds(it: &Item) -> Option<Bounds> {
    let of_rect =
        |r: vello::kurbo::Rect, pad: f64| Some((r.x0 - pad, r.y0 - pad, r.x1 + pad, r.y1 + pad));
    match it {
        Item::Rect { x0, y0, x1, y1, .. } => of_rect(Rect::new(*x0, *y0, *x1, *y1).abs(), 1.0),
        Item::Stroke {
            x0,
            y0,
            x1,
            y1,
            width,
            ..
        } => of_rect(Rect::new(*x0, *y0, *x1, *y1).abs(), width + 1.0),
        Item::Fill { path, .. } if !path.elements().is_empty() => of_rect(path.control_box(), 1.0),
        // A mitre can reach twice the width from the line, a square cap half of it.
        Item::StrokePath { path, width, .. } if !path.elements().is_empty() => {
            of_rect(path.control_box(), width * 2.0 + 1.0)
        }
        Item::BeginAlpha { bounds, .. } => Some((
            bounds.0.floor() - 1.0,
            bounds.1.floor() - 1.0,
            bounds.2.ceil() + 1.0,
            bounds.3.ceil() + 1.0,
        )),
        _ => None,
    }
}

fn union(a: Option<Bounds>, b: Option<Bounds>) -> Option<Bounds> {
    match (a, b) {
        (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))),
        (a, None) => a,
        (None, b) => b,
    }
}

fn overlaps(a: Bounds, b: Bounds) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

/// One object of the stack: a single item, or an opacity group with all it holds.
struct Unit {
    start: usize,
    end: usize,
    bounds: Bounds,
}

/// The objects of `items[from..to]` in order, which must start and end with no group open. An
/// object that draws nothing is left out.
fn units(items: &[Item], from: usize, to: usize) -> impl Iterator<Item = Unit> + '_ {
    let mut i = from;
    std::iter::from_fn(move || {
        while i < to {
            let start = i;
            let mut depth = 0usize;
            loop {
                match &items[i] {
                    Item::BeginAlpha { .. } => depth += 1,
                    Item::EndAlpha => depth = depth.saturating_sub(1),
                    _ => {}
                }
                i += 1;
                if depth == 0 || i >= to {
                    break;
                }
            }
            if let Some(bounds) = item_bounds(&items[start]) {
                return Some(Unit {
                    start,
                    end: i,
                    bounds,
                });
            }
        }
        None
    })
}

/// The part of the page that changed between two frames, found from the lists.
struct Patch {
    /// Left, top, width and height in whole pixels, clipped to the page. Empty when what changed
    /// lies off it.
    rect: (u32, u32, u32, u32),
}

/// How deep in opacity groups the list is before each item (and after the last).
fn depths(items: &[Item]) -> Vec<usize> {
    let mut d = 0usize;
    let mut out = Vec::with_capacity(items.len() + 1);
    out.push(0);
    for it in items {
        match it {
            Item::BeginAlpha { .. } => d += 1,
            Item::EndAlpha => d = d.saturating_sub(1),
            _ => {}
        }
        out.push(d);
    }
    out
}

/// The patch between the last frame's list `old` and `new`, which are the same below `cut` (a
/// place with no group open): everything from `cut` up to the last items the two share is the
/// changed middle, and the pixels it touches in either list are the patch. Where the middles have
/// as many items, only the items that differ count; the rest of the middle is drawn again where
/// it reaches the patch. None when the patch is over most of the page, or cannot be found.
fn plan_patch(
    old: &[Item],
    new: &[Item],
    cut: usize,
    size: (u32, u32),
    (below, total): (usize, usize),
    tune: Tune,
) -> Option<Patch> {
    let limit = old.len().min(new.len()).checked_sub(cut)?;
    let mut q = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take(limit)
        .take_while(|(a, b)| a == b)
        .count();
    // The shared top must begin with no group open in either list.
    let (dold, dnew) = (depths(old), depths(new));
    while q > 0 && (dold[old.len() - q] != 0 || dnew[new.len() - q] != 0) {
        q -= 1;
    }
    let (mold, mnew) = (&old[cut..old.len() - q], &new[cut..new.len() - q]);
    let mut changed: Option<Bounds> = None;
    if mold.len() == mnew.len() {
        for (a, b) in mold.iter().zip(mnew) {
            if a != b {
                changed = union(changed, union(item_bounds(a), item_bounds(b)));
            }
        }
    } else {
        for it in mold.iter().chain(mnew) {
            changed = union(changed, item_bounds(it));
        }
    }
    let b = changed?;
    let (w, h) = (f64::from(size.0), f64::from(size.1));
    let (x0, y0) = (b.0.floor().clamp(0.0, w), b.1.floor().clamp(0.0, h));
    let (x1, y1) = (b.2.ceil().clamp(0.0, w), b.3.ceil().clamp(0.0, h));
    let (rw, rh) = ((x1 - x0).max(0.0) as u32, (y1 - y0).max(0.0) as u32);
    if u64::from(rw) * u64::from(rh) * 100
        > u64::from(size.0) * u64::from(size.1) * PATCH_MAX_PERCENT
    {
        return None;
    }
    // What the patch has to draw: the objects that reach it (the lower part too when it is not
    // stored as an image). Over three quarters of what the whole page holds, and the patch saves too little.
    let rect = (x0, y0, x1, y1);
    let from = if below > tune.direct_max { cut } else { 0 };
    let drawn: usize = units(new, from, cut)
        .chain(units(new, cut, new.len()))
        .filter(|u| overlaps(u.bounds, rect))
        .map(|u| new[u.start..u.end].iter().map(weight).sum::<usize>())
        .sum();
    if drawn * 4 > total * 3 {
        return None;
    }
    Some(Patch {
        rect: (x0 as u32, y0 as u32, rw, rh),
    })
}

/// A rough cost of drawing an item: its path's size.
fn weight(it: &Item) -> usize {
    match it {
        Item::Fill { path, .. } | Item::StrokePath { path, .. } => path.elements().len(),
        _ => 4,
    }
}

impl quadrule_ui::ChipRenderer for Chip {
    fn render(&mut self, dl: &DisplayList) -> Result<(egui::TextureId, f64), String> {
        let start = Instant::now();
        self.draws = 1;
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
        let size = (dl.width, dl.height);
        let prev = self.last.take();
        // What is the same as last frame, from the bottom of the stack.
        let same = match &prev {
            Some(l) if l.size == size && l.background == dl.background => {
                common_prefix(&l.items, &dl.items)
            }
            _ => 0,
        };
        // The very same page again: the texture shown still holds it.
        if same == dl.items.len()
            && prev.as_ref().is_some_and(|l| l.items.len() == same)
            && let Some(t) = self.target.as_ref().filter(|t| t.size == size)
        {
            let id = t.id;
            self.last = prev;
            return Ok((id, start.elapsed().as_secs_f64() * 1000.0));
        }
        let cut = closed_cut(&dl.items, same);
        let (below, total): (usize, usize) = (
            dl.items[..cut].iter().map(weight).sum(),
            dl.items.iter().map(weight).sum(),
        );
        // A small part of the page changed (the box being typed in, an object being moved) and the
        // texture shown holds the page as it was: draw just that part, over the page's unchanged
        // lower part, and copy it into the texture shown.
        let patch = match &prev {
            Some(l)
                if l.size == size
                    && l.background == dl.background
                    && total >= self.tune.patch_min
                    && self.target.as_ref().is_some_and(|t| t.size == size) =>
            {
                plan_patch(&l.items, &dl.items, cut, size, (below, total), self.tune)
            }
            _ => None,
        };
        self.last = Some(Last {
            size,
            background: dl.background,
            items: dl.items.clone(),
        });
        // A base drawn from items that have changed since is no good.
        if self
            .base
            .as_ref()
            .is_some_and(|b| b.len > same || b.size != size)
        {
            self.drop_base();
        }
        let (view, id) = Self::target_in(&self.device, &self.egui, &mut self.target, size)?;
        // A patch is not timed against a whole draw: on a page of text that draw can itself pass
        // the fall-back line (110 ms on the fonts page), and the patch is chosen only when what
        // it draws is under half of the page.
        if let Some(p) = &patch {
            #[cfg(test)]
            {
                self.patches += 1;
            }
            self.draw_patch(dl, p, cut, below, total)?;
            return Ok((id, start.elapsed().as_secs_f64() * 1000.0));
        }
        // Worth keeping apart when it holds at least half of what there is to draw.
        let could = cut >= SPLIT_MIN_ITEMS && below * 2 >= total;
        let split = could && self.pace.split_next();
        if !split {
            self.drop_base();
            self.draw(&scene(&dl.items), &view, size, gray(dl.background))?;
            self.wait()?;
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            if could && self.pace.whole.len() < PROBE_FRAMES && self.pace.hold == 0 {
                self.pace.whole.push(ms);
            }
            return Ok((id, ms));
        }
        let kept = self.draw_over_base(dl, &view, cut, total)?;
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        // A frame that had to draw the base is not a fair sample.
        if kept && self.pace.split.len() < PROBE_FRAMES && self.pace.hold == 0 {
            self.pace.split.push(ms);
        }
        Ok((id, ms))
    }

    fn last_frame_draws(&self) -> u32 {
        self.draws
    }
}

impl Chip {
    /// Whether the stored lower part of `dl` (the items below `cut`) stands, else draws it anew
    /// (two page-sized draws in this frame). True when it stood.
    fn keep_base(&mut self, dl: &DisplayList, cut: usize, total: usize) -> Result<bool, String> {
        let size = (dl.width, dl.height);
        // Keep the base it has unless a good deal more has stood unchanged since, else draw a new one.
        let keep = self.base.as_ref().is_some_and(|b| {
            let between: usize = dl.items[b.len.min(cut)..cut].iter().map(weight).sum();
            between * 5 <= total
        });
        if !keep {
            self.make_base(&dl.items[..cut], size, gray(dl.background))?;
            // The lower part, then the rest over it: two draws in one frame.
            self.draws = 2;
        }
        let base = self.base.as_ref().expect("made above");
        // Vello keeps images in an atlas it may reuse between frames, so the picture is handed over
        // again each time (a copy of the texture, which costs next to nothing).
        if let Some(v) = self.vello.as_mut() {
            v.mark_override_image_dirty(&base.image);
        }
        Ok(keep)
    }

    /// The whole page as the stored lower part with what is above it drawn over (rung 10).
    fn draw_over_base(
        &mut self,
        dl: &DisplayList,
        view: &wgpu::TextureView,
        cut: usize,
        total: usize,
    ) -> Result<bool, String> {
        let size = (dl.width, dl.height);
        let keep = self.keep_base(dl, cut, total)?;
        let base = self.base.as_ref().expect("made above");
        let mut sc = vello::Scene::new();
        sc.draw_image(&base.image, Affine::IDENTITY);
        sc.append(&scene(&dl.items[base.len..]), None);
        self.draw(&sc, view, size, gray(dl.background))?;
        self.wait()?;
        Ok(keep)
    }

    /// Only the patch of the page that changed (rung 11b): drawn into a texture of its own, over
    /// the stored lower part, with whatever lies above it that reaches it, and copied into the
    /// texture shown, which keeps the rest of the page as the last frame left it.
    fn draw_patch(
        &mut self,
        dl: &DisplayList,
        patch: &Patch,
        cut: usize,
        below: usize,
        total: usize,
    ) -> Result<(), String> {
        let (rx, ry, rw, rh) = patch.rect;
        if rw == 0 || rh == 0 {
            // What changed lies off the page: the picture is the one shown.
            return Ok(());
        }
        let rect = (
            f64::from(rx),
            f64::from(ry),
            f64::from(rx + rw),
            f64::from(ry + rh),
        );
        let shift = Affine::translate((-rect.0, -rect.1));
        let mut sc = vello::Scene::new();
        let from = if below > self.tune.direct_max {
            self.keep_base(dl, cut, total)?;
            let base = self.base.as_ref().expect("made above");
            sc.draw_image(&base.image, shift);
            base.len
        } else {
            0
        };
        // The unstored part of the lower part, then the changed part and what is above it: each
        // object that reaches the patch, in order.
        let mid = from.max(cut);
        for u in units(&dl.items, from, cut).chain(units(&dl.items, mid, dl.items.len())) {
            if overlaps(u.bounds, rect) {
                push_items(&mut sc, &dl.items[u.start..u.end], shift);
            }
        }
        let tex_size = (rw.div_ceil(16) * 16, rh.div_ceil(16) * 16);
        if self
            .patch_tex
            .as_ref()
            .is_none_or(|(sz, _)| *sz != tex_size)
        {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("quadrule-chip-patch"),
                size: wgpu::Extent3d {
                    width: tex_size.0,
                    height: tex_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            self.patch_tex = Some((tex_size, texture));
        }
        let (_, texture) = self.patch_tex.clone().expect("made above");
        let view = texture.create_view(&Default::default());
        self.draw(&sc, &view, tex_size, gray(dl.background))?;
        let target = &self.target.as_ref().expect("checked by the plan").texture;
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: target,
                mip_level: 0,
                origin: wgpu::Origin3d { x: rx, y: ry, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: rw,
                height: rh,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([enc.finish()]);
        self.wait()
    }
}

impl Chip {
    /// Waits for the chip, so the time measured is the whole frame (the fallback rule).
    fn wait(&self) -> Result<(), String> {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

fn gray(v: u8) -> Color {
    Color::from_rgba8(v, v, v, 255)
}

/// `items` as a vello scene.
fn scene(items: &[Item]) -> vello::Scene {
    let mut s = vello::Scene::new();
    push_items(&mut s, items, Affine::IDENTITY);
    s
}

/// Adds `items` to `s`, moved by `id`.
fn push_items(s: &mut vello::Scene, items: &[Item], id: Affine) {
    for item in items {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use quadrule_ui::ChipRenderer;
    use vello::kurbo::BezPath;

    const W: u32 = 512;
    const H: u32 = 320;

    /// A device on whatever the machine has (none on a bare CI runner: the test then says so).
    fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok()?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .ok()
    }

    fn chip_on(device: &wgpu::Device, queue: &wgpu::Queue, tune: Tune) -> Chip {
        let egui = egui_wgpu::Renderer::new(
            device,
            wgpu::TextureFormat::Rgba8Unorm,
            egui_wgpu::RendererOptions::default(),
        );
        let mut chip = Chip::with_parts(
            device.clone(),
            queue.clone(),
            std::sync::Arc::new(egui::mutex::RwLock::new(egui)),
        );
        chip.tune = tune;
        chip
    }

    /// The target's pixels, red channel (the page is gray).
    fn pixels(chip: &Chip) -> Vec<u8> {
        let t = chip.target.as_ref().expect("a frame was drawn");
        let buf = chip.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(W * H * 4),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = chip.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &t.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(W * 4),
                    rows_per_image: Some(H),
                },
            },
            wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
        );
        chip.queue.submit([enc.finish()]);
        buf.slice(..)
            .map_async(wgpu::MapMode::Read, |r| r.expect("mapped"));
        chip.wait().expect("waited");
        let data = buf.slice(..).get_mapped_range().expect("mapped");
        data.iter().step_by(4).copied().collect()
    }

    /// Small slanted shapes in a row: what text looks like to the chip (many short curves).
    fn blobs(x: f64, y: f64, n: usize) -> BezPath {
        let mut p = BezPath::new();
        for i in 0..n {
            let x0 = x + (i % 20) as f64 * 9.3 + 0.37 * (i / 20) as f64;
            let y0 = y + (i / 20) as f64 * 11.7;
            p.move_to((x0, y0));
            p.curve_to(
                (x0 + 3.1, y0 - 1.3),
                (x0 + 6.4, y0 + 0.7),
                (x0 + 5.2, y0 + 8.9),
            );
            p.quad_to((x0 + 2.6, y0 + 9.4), (x0 + 0.4, y0 + 7.7));
            p.close_path();
        }
        p
    }

    fn fill(x: f64, y: f64, n: usize, gray: u8) -> Item {
        Item::Fill {
            path: blobs(x, y, n),
            gray,
        }
    }

    fn list(items: Vec<Item>) -> DisplayList {
        DisplayList {
            width: W,
            height: H,
            background: 255,
            items,
        }
    }

    fn group(alpha: u8, inner: Vec<Item>) -> Vec<Item> {
        let mut v = vec![Item::BeginAlpha {
            alpha,
            bounds: (20.0, 20.0, 400.0, 200.0),
        }];
        v.extend(inner);
        v.push(Item::EndAlpha);
        v
    }

    /// Frames of a page whose middle object is edited, the way a page is while typing: the same
    /// frames through the patch way and through whole draws must make the same picture.
    #[test]
    fn patch_matches_whole() {
        let Some((device, queue)) = device() else {
            println!("CHECK crate=app name=chip_patch result=skip reason=no_gpu");
            return;
        };
        let mut worst = 0u8;
        let mut patches = 0;
        let mut frames = 0;
        // Both ways of holding the lower part: as an image, and drawn straight into the patch.
        for direct_max in [0, usize::MAX] {
            let tune = Tune {
                patch_min: 0,
                direct_max,
            };
            let mut patched = chip_on(&device, &queue, tune);
            let mut whole = chip_on(
                &device,
                &queue,
                Tune {
                    patch_min: usize::MAX,
                    direct_max,
                },
            );
            let low = || {
                vec![
                    Item::Rect {
                        x0: 0.0,
                        y0: 100.0,
                        x1: 512.0,
                        y1: 101.0,
                        gray: 200,
                    },
                    fill(10.0, 10.0, 120, 150),
                    fill(7.5, 150.0, 60, 90),
                ]
            };
            let top = |shift: f64| {
                let mut v = group(140, vec![fill(30.0 + shift, 30.0, 14, 40)]);
                v.push(fill(300.0, 250.0 - shift, 20, 10));
                v
            };
            let frame = |edit: Vec<Item>, above: Vec<Item>| {
                let mut v = low();
                v.extend(edit);
                v.extend(above);
                list(v)
            };
            let steps: Vec<DisplayList> = vec![
                frame(vec![fill(40.0, 40.0, 30, 0)], top(0.0)),
                frame(vec![fill(40.0, 40.0, 31, 0)], top(0.0)),
                frame(vec![fill(40.0, 40.0, 32, 0)], top(0.0)),
                frame(vec![fill(40.0, 40.0, 20, 0)], top(0.0)),
                // two objects change together: the box and an object above it
                frame(vec![fill(40.0, 40.0, 21, 0)], top(3.0)),
                // more items, so the middles differ in length
                frame(
                    vec![fill(40.0, 40.0, 21, 0), fill(200.0, 60.0, 5, 70)],
                    top(3.0),
                ),
                // a change in the lower part
                {
                    let mut v = low();
                    v[1] = fill(10.0, 10.0, 121, 150);
                    v.push(fill(40.0, 40.0, 21, 0));
                    v.extend(top(3.0));
                    list(v)
                },
                // a change that reaches over the edge of the page
                frame(vec![fill(480.0, 300.0, 8, 0)], top(3.0)),
                frame(vec![fill(480.0, 300.0, 9, 0)], top(3.0)),
                // the very same page
                frame(vec![fill(480.0, 300.0, 9, 0)], top(3.0)),
            ];
            for dl in &steps {
                patched.render(dl).expect("patch way");
                whole.render(dl).expect("whole way");
                let (a, b) = (pixels(&patched), pixels(&whole));
                let d = a
                    .iter()
                    .zip(&b)
                    .map(|(x, y)| x.abs_diff(*y))
                    .max()
                    .unwrap_or(0);
                worst = worst.max(d);
                frames += 1;
            }
            patches += patched.patches;
        }
        println!(
            "CHECK crate=app name=chip_patch result={} frames={frames} patches={patches} worst_gray_diff={worst}",
            if worst <= 1 && patches >= 10 {
                "pass"
            } else {
                "fail"
            }
        );
        assert!(
            worst <= 1,
            "the patch way drew a picture {worst} gray levels from a whole draw"
        );
        assert!(patches >= 10, "only {patches} frames were drawn as a patch");
    }
}
