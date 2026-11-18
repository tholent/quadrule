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

//! `quadrule`, the app (napkin §15.1). Rung 2 made the viewer window
//! (docs/bbq/2026-11-09-rung-2-viewer.md); rung 3 the first editor
//! (docs/bbq/2026-11-10-rung-3-first-editor.md); rung 4 dragging and snapping
//! (docs/bbq/2026-11-10-rung-4-drag-and-snap.md). Usage: `quadrule [--canvas chip|export]
//! [--bench-drag [frames]] [file.quad]`, or `cargo run -p quadrule-app -- [file.quad]`.
//! `--canvas chip` starts on the graphics-chip canvas (View → Canvas), for measuring it.
//! `--bench-drag` opens the sample page, replays a drag of its dot grid through the real window
//! (150 steps unless told otherwise), prints one `CHECK crate=ui name=drag_bench` line with the
//! frame times and closes. `--bench-draw` does the same for dragging a pattern region out with
//! the Pattern tool (rung 5; `name=draw_bench`). Rung 6a adds `--shapes-page` (open the page that holds
//! every new kind; the benchmarks then run over it) and `--bench-tool rect|ellipse|polygon|...` (draw
//! with that tool instead). Rung 6b adds `--transforms-page` (rotated shapes, lines every way, opacity;
//! the benchmarks then run over it) and `--bench-rotate [frames]` (turn its big rectangle with the
//! rotate handle; `name=rotate_bench`). Rung 7a adds `--groups-page` (nested, turned and faded groups and
//! a heavy group; the benchmarks then run over it and the drag moves the heavy group). Rung 8a adds
//! `--devices-page` (the overrides page), `--bench-secondary` (run on the Nomad, where a drag writes
//! overrides) and `--no-strip` (fold the device previews strip shut; it is open in a bench
//! otherwise). Rung 8b adds `--landscape-page` (the landscape page; the benchmarks then run over it). Rung 9 adds
//! `--patterns-page` (the pattern page: every new pattern; the benchmarks then run over it). Rung 10 adds
//! `--repeats-page` (the repeat page; the benchmarks then run over it and the drag moves the habit tracker). Rung 11a adds
//! `--fonts-page` (the fonts page: a page of text; the benchmarks then run over it and the drag moves its dot grid). Rung 11b adds
//! `--textbox-page` (the text-box page, same use), `--missing-fonts-page` (the same page set in a font this computer lacks) and `--bench-type [frames]` (type into a long text box; `name=type_bench`). Rung 12b adds
//! `--pattern-angle-page` (the ruling turned inside its frame; the benchmarks then run over it) and
//! `--turned-patterns-page` (pattern regions turned whole, one in a group and one in a repeat; the benchmarks then run over it, and `--bench-rotate` turns its 45 degree grid). Rung 14b adds `--look lint|style|preflight|export|…` (start with that part showing, for a screenshot). Rung 14a adds `--lint-page` (objects that make the gray, text-size, handwriting, dark-fill and thin-line rules fire, beside ones that pass; the benchmarks then run over it). Rung 13b adds
//! `--paper-page` (ISO A4 at 300 PPI and ISO A5 at 600; the benchmarks then run over it) and `--paper-landscape-page` (US Letter held sideways, exported wide).
//!
//! This crate holds only what needs a real window: eframe setup, the graphics-chip connection,
//! the system file dialogs and remembered preferences. The window itself is `quadrule-ui`.

#![forbid(unsafe_code)]

mod chip;

use eframe::egui;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu;
use quadrule_ui::{Recovery, Request, Viewer, ViewerUi};
use std::path::PathBuf;
use std::sync::Arc;

/// Preference keys in eframe's storage.
/// What `--look` takes: a part of the window to start with, for a screenshot (rungs 14b, 15 and 16).
const LOOKS: [&str; 10] = [
    "lint",
    "style",
    "preflight",
    "export",
    "export-preset",
    "export-mistake",
    "exported",
    "exported-open",
    "welcome",
    "file-menu",
];
const KEY_THEME: &str = "theme";
const KEY_RULER: &str = "screen_px_per_mm";
/// The snap helpers and the rhythm overlay (rung 4 bbq, Defaulted "Toggles"): app settings.
const KEY_SNAP_PIXEL: &str = "snap_pixel_grid";
const KEY_SNAP_RHYTHM: &str = "snap_rhythm";
const KEY_SNAP_MARGINS: &str = "snap_margins_safe";
const KEY_SNAP_POINTS: &str = "snap_points";
const KEY_SNAP_SMART: &str = "snap_smart_guides";
const KEY_SNAP_EQUAL: &str = "snap_equal_spacing";
const KEY_RHYTHM_OVERLAY: &str = "show_rhythm_overlay";
/// The device the New dialog starts on: the last one picked (rung 5).
const KEY_NEW_DEVICE: &str = "new_device";
/// What Align to was last set to, for one object picked and for several (rung 7b, Q5).
const KEY_ALIGN_SINGLE: &str = "align_to_single";
const KEY_ALIGN_MULTI: &str = "align_to_multi";
/// Whether the device previews strip is open (rung 8a, Q5).
const KEY_STRIP: &str = "show_strip";

struct App {
    ui: ViewerUi,
    /// The title last sent to the window.
    title: String,
    /// Set once the user has answered the unsaved-changes prompt with Save or Don't save.
    closing: bool,
    /// Close the window at the end of this frame.
    ctx_close: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        let path = args.path;
        quadrule_ui::install_fonts(&cc.egui_ctx, desktop_ui_font());
        // The profiles the user keeps (rung 13a): the shipped ones, the user's edits of them and the
        // user's own, from `<config folder>/profiles/` (on Linux `~/.config/quadrule/profiles/`),
        // read once now and never again per frame. A timing run reads none, so a number never depends
        // on someone's edits.
        let timing = args.bench_drag.is_some()
            || args.bench_draw.is_some()
            || args.bench_type.is_some()
            || args.bench_rotate.is_some();
        quadrule_profiles::active::set(
            match directories::ProjectDirs::from("org", "quadrule", "quadrule") {
                Some(d) if !timing => {
                    quadrule_profiles::Registry::load(&d.config_dir().join("profiles"))
                }
                _ => quadrule_profiles::Registry::builtin_only(),
            },
        );
        let mut ui = ViewerUi::new(Viewer::default());
        // What Export remembers (rung 15): the path template, the folders, the note tick, in
        // `settings.ron` beside `profiles/`. A timing run reads none and writes none.
        if !timing && let Some(d) = directories::ProjectDirs::from("org", "quadrule", "quadrule") {
            ui.viewer
                .use_settings_file(d.config_dir().join(quadrule_ui::settings::FILE_NAME));
        }
        // Text can use the fonts installed on this computer as well as the bundled ones (rung 11a).
        ui.viewer.use_system_fonts();
        if let Some(storage) = cc.storage {
            if let Some(theme) = eframe::get_value::<String>(storage, KEY_THEME) {
                ui.theme = match theme.as_str() {
                    "light" => egui::ThemePreference::Light,
                    "dark" => egui::ThemePreference::Dark,
                    _ => egui::ThemePreference::System,
                };
            }
            ui.viewer.screen_px_per_mm = eframe::get_value(storage, KEY_RULER);
            let snap = &mut ui.viewer.snap;
            snap.pixel_grid = eframe::get_value(storage, KEY_SNAP_PIXEL).unwrap_or(snap.pixel_grid);
            snap.rhythm = eframe::get_value(storage, KEY_SNAP_RHYTHM).unwrap_or(snap.rhythm);
            snap.margins_safe =
                eframe::get_value(storage, KEY_SNAP_MARGINS).unwrap_or(snap.margins_safe);
            snap.points = eframe::get_value(storage, KEY_SNAP_POINTS).unwrap_or(snap.points);
            snap.smart_guides =
                eframe::get_value(storage, KEY_SNAP_SMART).unwrap_or(snap.smart_guides);
            snap.equal_spacing =
                eframe::get_value(storage, KEY_SNAP_EQUAL).unwrap_or(snap.equal_spacing);
            ui.viewer.show_rhythm =
                eframe::get_value(storage, KEY_RHYTHM_OVERLAY).unwrap_or(ui.viewer.show_rhythm);
            let kept = |key| {
                eframe::get_value::<String>(storage, key)
                    .and_then(|k| quadrule_layout::align::AlignTo::from_key(&k))
            };
            ui.viewer.show_strip =
                eframe::get_value(storage, KEY_STRIP).unwrap_or(ui.viewer.show_strip);
            ui.viewer.align_single = kept(KEY_ALIGN_SINGLE).unwrap_or(ui.viewer.align_single);
            ui.viewer.align_multi = kept(KEY_ALIGN_MULTI).unwrap_or(ui.viewer.align_multi);
            if let Some(id) = eframe::get_value::<String>(storage, KEY_NEW_DEVICE) {
                ui.set_new_device(&id);
            }
        }
        cc.egui_ctx.set_theme(ui.theme);
        // Recovery files live in the user's data folder (rung 3 bbq, Defaulted "Autosave"),
        // e.g. ~/.local/share/quadrule/recovery on Linux.
        let dirs = directories::ProjectDirs::from("org", "quadrule", "quadrule");
        ui.recovery = dirs
            .as_ref()
            .map(|d| Recovery::new(d.data_dir().join("recovery")));
        // My templates (rung 16 bbq, Q4) are kept beside it: `templates/` in the same data folder. A timing run
        // has none.
        if !timing {
            ui.viewer.templates_dir = dirs
                .as_ref()
                .map(|d| d.data_dir().join(quadrule_ui::templates::DIR_NAME));
        }
        ui.offer_recovery();
        // The graphics-chip canvas (rung 3 bbq Q4), on egui's own device; when it cannot start,
        // the canvas falls back to vello_cpu and says why.
        ui.chip = match &cc.wgpu_render_state {
            Some(rs) => Ok(Box::new(chip::Chip::new(rs)) as Box<dyn quadrule_ui::ChipRenderer>),
            None => Err("no wgpu device".into()),
        };
        if args.chip_canvas {
            ui.canvas_mode = quadrule_ui::CanvasMode::Chip;
        }
        if let Some(p) = path {
            ui.open_file(&p);
        } else if args.missing_fonts_page {
            ui.viewer.open_missing_fonts();
        } else if args.textbox_page {
            ui.viewer.open_textbox();
        } else if args.fonts_page {
            ui.viewer.open_fonts();
        } else if args.lint_page {
            ui.viewer.open_lint();
        } else if args.paper_page {
            ui.viewer.open_paper();
        } else if args.paper_landscape_page {
            ui.viewer.open_paper_landscape();
        } else if args.pattern_angle_page {
            ui.viewer.open_pattern_angle();
        } else if args.turned_patterns_page {
            ui.viewer.open_turned_patterns();
        } else if args.repeats_page {
            ui.viewer.open_repeats();
        } else if args.patterns_page {
            ui.viewer.open_patterns();
        } else if args.landscape_page {
            ui.viewer.open_landscape();
        } else if args.devices_page {
            ui.viewer.open_devices();
        } else if args.groups_page {
            ui.viewer.open_groups();
        } else if args.transforms_page || args.bench_rotate.is_some() {
            ui.viewer.open_transforms();
        } else if args.shapes_page {
            ui.viewer.open_shapes();
        }
        let bench = if let Some(frames) = args.bench_rotate {
            Some(quadrule_ui::DragBench::rotate(frames))
        } else if let Some(frames) = args.bench_type {
            Some(quadrule_ui::DragBench::typing(frames))
        } else if let Some(frames) = args.bench_drag {
            // A timing run is not a session: it leaves no settings behind and opens no recovery.
            Some(quadrule_ui::DragBench::new(frames))
        } else {
            args.bench_draw.map(|frames| {
                quadrule_ui::DragBench::draw(frames)
                    .with_tool(args.bench_tool.unwrap_or(quadrule_ui::Tool::Pattern))
            })
        };
        if args.profiles {
            ui.open_profiles_window();
        }
        if let Some(look) = &args.look {
            // A look with no page named opens the sample page, so there is something to export; the welcome
            // screen's look is the screen with nothing open.
            if ui.viewer.open.is_none() && look != "welcome" {
                ui.viewer.open_sample();
            }
            ui.show_look(look);
        }
        ui.note_damaged_profiles();
        ui.bench = bench.map(|b| {
            let b = if args.bench_secondary {
                b.on_secondary()
            } else {
                b
            };
            let b = if args.no_strip { b.without_strip() } else { b };
            if args.missing_fonts_page {
                b.with_missing_fonts()
            } else if args.textbox_page {
                b.with_textbox()
            } else if args.fonts_page {
                b.with_fonts()
            } else if args.lint_page {
                b.with_lint()
            } else if args.paper_page {
                b.with_paper()
            } else if args.pattern_angle_page {
                b.with_pattern_angle()
            } else if args.turned_patterns_page {
                b.with_turned_patterns()
            } else if args.repeats_page {
                b.with_repeats()
            } else if args.patterns_page {
                b.with_patterns()
            } else if args.landscape_page {
                b.with_landscape()
            } else if args.devices_page {
                b.with_devices()
            } else if args.groups_page {
                b.with_groups()
            } else if args.transforms_page {
                b.with_transforms()
            } else if args.shapes_page {
                b.with_shapes()
            } else {
                b
            }
        });
        App {
            ui,
            title: String::new(),
            closing: false,
            ctx_close: false,
        }
    }

    fn answer(&mut self, req: Request) {
        match req {
            Request::OpenDialog => {
                let picked = rfd::FileDialog::new()
                    .add_filter("quadrule", &["quad"])
                    .pick_file();
                if let Some(path) = picked {
                    self.ui.open_file(&path);
                }
            }
            Request::SaveAsDialog(then) => {
                let mut dialog = rfd::FileDialog::new()
                    .add_filter("quadrule", &["quad"])
                    .set_file_name(format!(
                        "{}.quad",
                        quadrule_ui::viewer::slug(self.ui.viewer.title().unwrap_or("untitled"))
                    ));
                if let Some(dir) = self.ui.viewer.file_path().and_then(|p| p.parent()) {
                    dialog = dialog.set_directory(dir);
                }
                if let Some(mut path) = dialog.save_file() {
                    if path.extension().is_none_or(|e| e != "quad") {
                        path.set_extension("quad");
                    }
                    for next in self.ui.save_as(&path, then) {
                        self.answer(next);
                    }
                }
            }
            Request::Close => {
                self.closing = true;
                self.ctx_close = true;
            }
            Request::ProfileImportDialog => {
                let picked = rfd::FileDialog::new()
                    .add_filter("profile", &["ron"])
                    .pick_file();
                if let Some(path) = picked {
                    self.ui.profiles_import_path(&path);
                }
            }
            Request::ProfileExportDialog(name) => {
                let picked = rfd::FileDialog::new()
                    .add_filter("profile", &["ron"])
                    .set_file_name(name)
                    .save_file();
                if let Some(path) = picked {
                    self.ui.profiles_export_to(&path);
                }
            }
            Request::ExportFolderDialog => {
                let mut dialog = rfd::FileDialog::new();
                // Where the sheet's folder is now: where this document last exported, else the last folder
                // used, else its own folder, else home (rung 15 bbq, Q4).
                if let Some(dir) = self.ui.export_dialog_start() {
                    dialog = dialog.set_directory(dir);
                }
                if let Some(dir) = dialog.pick_folder() {
                    self.ui.export_set_folder(dir);
                }
            }
            Request::RevealFolder(folder) => {
                let (program, args) = quadrule_ui::export_ui::reveal_command(&folder);
                // The file manager is not waited for (explorer.exe, for one, exits with 1 when it works).
                if let Err(e) = std::process::Command::new(program).args(args).spawn() {
                    self.ui.reveal_failed(format!("{program}: {e}"));
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Closing with unsaved changes asks first (rung 3 bbq, Defaulted "Save").
        if ctx.input(|i| i.viewport().close_requested()) && !self.closing && !self.ui.may_close() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        for req in self.ui.show(ui) {
            self.answer(req);
        }
        if self.ui.bench_finished() {
            self.closing = true;
            self.ctx_close = true;
        }
        if std::mem::take(&mut self.ctx_close) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // The window title names the file, with a dot for unsaved changes.
        let title = self.ui.window_title();
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let theme = match self.ui.theme {
            egui::ThemePreference::Light => "light",
            egui::ThemePreference::Dark => "dark",
            egui::ThemePreference::System => "system",
        };
        eframe::set_value(storage, KEY_THEME, &theme.to_string());
        eframe::set_value(storage, KEY_RULER, &self.ui.viewer.screen_px_per_mm);
        let v = &self.ui.viewer;
        eframe::set_value(storage, KEY_SNAP_PIXEL, &v.snap.pixel_grid);
        eframe::set_value(storage, KEY_SNAP_RHYTHM, &v.snap.rhythm);
        eframe::set_value(storage, KEY_SNAP_MARGINS, &v.snap.margins_safe);
        eframe::set_value(storage, KEY_SNAP_POINTS, &v.snap.points);
        eframe::set_value(storage, KEY_SNAP_SMART, &v.snap.smart_guides);
        eframe::set_value(storage, KEY_SNAP_EQUAL, &v.snap.equal_spacing);
        eframe::set_value(storage, KEY_RHYTHM_OVERLAY, &v.show_rhythm);
        eframe::set_value(storage, KEY_STRIP, &v.show_strip);
        eframe::set_value(storage, KEY_ALIGN_SINGLE, &v.align_single.key().to_string());
        eframe::set_value(storage, KEY_ALIGN_MULTI, &v.align_multi.key().to_string());
        eframe::set_value(storage, KEY_NEW_DEVICE, &self.ui.new_device().to_string());
    }
}

/// The desktop's interface font, for the controls (chris, 2026-11-09: "I would like the system
/// default font for the controls"). `None` (egui's own font) when it cannot be found.
fn desktop_ui_font() -> Option<quadrule_ui::UiFont> {
    let (family, weight) = desktop_font_name()?;
    let face = quadrule_text::system_face(&family, weight)?;
    Some(quadrule_ui::UiFont {
        data: face.data,
        index: face.index,
        weight,
    })
}

/// The desktop interface font's family and weight. Linux and the BSDs: GTK's setting, which the
/// Raspberry Pi desktop and GNOME set (`gsettings get org.gnome.desktop.interface font-name`, e.g.
/// `'Nunito Sans Light 12'`). Windows: Segoe UI. macOS: the system font.
fn desktop_font_name() -> Option<(String, u16)> {
    if cfg!(target_os = "windows") {
        return Some(("Segoe UI".into(), 400));
    }
    if cfg!(target_os = "macos") {
        return Some((".AppleSystemUIFont".into(), 400));
    }
    let out = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "font-name"])
        .output()
        .ok()?;
    parse_gtk_font(&String::from_utf8_lossy(&out.stdout))
}

/// Parses a GTK font name, `'Family Style… Size'`, into family and weight. The size is left to
/// egui's own text sizes.
fn parse_gtk_font(s: &str) -> Option<(String, u16)> {
    let s = s.trim().trim_matches('\'').trim();
    let mut words: Vec<&str> = s.split_whitespace().collect();
    if words.last().is_some_and(|w| w.parse::<f32>().is_ok()) {
        words.pop();
    }
    let mut weight = 400;
    while let Some(w) = words.last() {
        let found = match w.to_ascii_lowercase().as_str() {
            "thin" | "hairline" => Some(100),
            "extralight" | "ultralight" => Some(200),
            "light" => Some(300),
            "regular" | "normal" | "book" | "italic" | "oblique" => Some(0),
            "medium" => Some(500),
            "semibold" | "demibold" => Some(600),
            "bold" => Some(700),
            "extrabold" | "ultrabold" => Some(800),
            "black" | "heavy" => Some(900),
            _ => None,
        };
        match found {
            Some(0) => {}
            Some(v) => weight = v,
            None => break,
        }
        words.pop();
    }
    (!words.is_empty()).then(|| (words.join(" "), weight))
}

/// The graphics-chip connection. eframe asks for wgpu's default limits, which the Raspberry Pi 5's
/// V3D refuses (`max_inter_stage_shader_variables` 16 asked, 15 allowed; on OpenGL,
/// `max_texture_dimension_2d` 8192 asked, 4096 allowed — spike R15, 2026-11-09). The viewer draws
/// only egui and one page texture (rung 2 bbq, Q3), so it asks for exactly what the adapter has.
fn wgpu_options() -> WgpuConfiguration {
    let mut setup = WgpuSetupCreateNew::without_display_handle();
    setup.device_descriptor = Arc::new(|adapter: &wgpu::Adapter| wgpu::DeviceDescriptor {
        label: Some("quadrule"),
        required_limits: adapter.limits(),
        ..Default::default()
    });
    WgpuConfiguration {
        wgpu_setup: WgpuSetup::CreateNew(setup),
        ..Default::default()
    }
}

/// What the command line asked for.
#[derive(Debug, Default, PartialEq)]
struct Args {
    path: Option<PathBuf>,
    chip_canvas: bool,
    /// Replay a drag this many steps and close (`--bench-drag [frames]`).
    bench_drag: Option<u32>,
    /// Replay drawing a pattern region this many steps and close (`--bench-draw [frames]`).
    bench_draw: Option<u32>,
    /// Type this many letters into a long text box and close (`--bench-type [frames]`).
    bench_type: Option<u32>,
    /// Open the shapes page (every new kind) instead of the empty start (`--shapes-page`); the
    /// benchmarks then run over it.
    shapes_page: bool,
    /// Open the transforms page (rotated shapes, lines every way, opacity) instead of the empty
    /// start (`--transforms-page`); the benchmarks then run over it.
    transforms_page: bool,
    /// Open the groups page (nested, turned and faded groups, a heavy group) instead of the empty
    /// start (`--groups-page`); the benchmarks then run over it, and the drag moves the heavy group.
    groups_page: bool,
    /// Open the rung 8a overrides page instead of the empty start (`--devices-page`); the
    /// benchmarks then run over it.
    devices_page: bool,
    /// Open the rung 8b landscape page instead of the empty start (`--landscape-page`); the
    /// benchmarks then run over it.
    landscape_page: bool,
    /// Open the rung 9 pattern page instead of the empty start (`--patterns-page`); the benchmarks
    /// then run over it.
    patterns_page: bool,
    /// Open the rung 14a lint page instead of the empty start (`--lint-page`: objects that make L03, L05,
    /// L07, L09, L13 and L14 fire, beside ones that pass); the benchmarks then run over it.
    lint_page: bool,
    /// Open the rung 13b paper page instead of the empty start (`--paper-page`: ISO A4 at 300 PPI and ISO
    /// A5 at 600); the benchmarks then run over it.
    paper_page: bool,
    /// Open the rung 13b landscape paper page instead of the empty start (`--paper-landscape-page`: US
    /// Letter held sideways, exported wide).
    paper_landscape_page: bool,
    /// Open the rung 12b pattern-angle page instead of the empty start (`--pattern-angle-page`); the
    /// benchmarks then run over it.
    pattern_angle_page: bool,
    /// Open the rung 12b turned-patterns page instead of the empty start (`--turned-patterns-page`); the
    /// benchmarks then run over it, and `--bench-rotate` turns its 45 degree grid.
    turned_patterns_page: bool,
    /// Open the rung 10 repeat page instead of the empty start (`--repeats-page`); the benchmarks
    /// then run over it, and the drag moves the habit tracker.
    repeats_page: bool,
    /// Open the rung 11a fonts page instead of the empty start (`--fonts-page`); the benchmarks
    /// then run over it, and the drag moves its dot grid.
    fonts_page: bool,
    /// Open the Profiles window at start (`--profiles`), for a look at it (rung 13a).
    profiles: bool,
    /// Start with the window showing one part, for a look at it (`--look lint|style|preflight|export|…`, rung 14b): the
    /// Lint tab, the Style tab with a pale custom line selected, or the sheet before Export.
    look: Option<String>,
    /// Open the rung 11b text-box page instead of the empty start (`--textbox-page`); the benchmarks
    /// then run over it, and the drag moves its dot grid.
    textbox_page: bool,
    /// Open the rung 12a missing-fonts page (`--missing-fonts-page`): the text-box page with every
    /// text box in a font this computer lacks, drawn from saved outlines, with the bar above it; the
    /// benchmarks then run over it.
    missing_fonts_page: bool,
    /// Run the benchmark on the first secondary device, so a drag writes overrides
    /// (`--bench-secondary`).
    bench_secondary: bool,
    /// Fold the previews strip shut for the benchmark (`--no-strip`); it is open in a bench
    /// otherwise.
    no_strip: bool,
    /// Turn the transforms page's big rectangle with its rotate handle this many steps and close
    /// (`--bench-rotate [frames]`).
    bench_rotate: Option<u32>,
    /// The tool `--bench-draw` drags out (`--bench-tool pattern|rect|ellipse|polygon|line|text`).
    bench_tool: Option<quadrule_ui::Tool>,
}

fn parse_args(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Args, String> {
    let mut out = Args::default();
    let mut args = args.peekable();
    while let Some(a) = args.next() {
        if a == "--canvas" {
            match args.next().as_ref().and_then(|v| v.to_str()) {
                Some("chip") => out.chip_canvas = true,
                Some("export") => out.chip_canvas = false,
                other => return Err(format!("--canvas takes chip or export, not {other:?}")),
            }
        } else if a == "--bench-drag" {
            // An optional step count follows; anything else is the next argument.
            let steps = args
                .next_if(|n| n.to_str().is_some_and(|s| s.parse::<u32>().is_ok()))
                .and_then(|n| n.to_str().and_then(|s| s.parse().ok()));
            out.bench_drag = Some(steps.unwrap_or(150));
        } else if a == "--bench-draw" {
            let steps = args
                .next_if(|n| n.to_str().is_some_and(|s| s.parse::<u32>().is_ok()))
                .and_then(|n| n.to_str().and_then(|s| s.parse().ok()));
            out.bench_draw = Some(steps.unwrap_or(150));
        } else if a == "--bench-type" {
            let steps = args
                .next_if(|n| n.to_str().is_some_and(|s| s.parse::<u32>().is_ok()))
                .and_then(|n| n.to_str().and_then(|s| s.parse().ok()));
            out.bench_type = Some(steps.unwrap_or(150));
        } else if a == "--bench-rotate" {
            let steps = args
                .next_if(|n| n.to_str().is_some_and(|s| s.parse::<u32>().is_ok()))
                .and_then(|n| n.to_str().and_then(|s| s.parse().ok()));
            out.bench_rotate = Some(steps.unwrap_or(150));
        } else if a == "--shapes-page" {
            out.shapes_page = true;
        } else if a == "--transforms-page" {
            out.transforms_page = true;
        } else if a == "--groups-page" {
            out.groups_page = true;
        } else if a == "--devices-page" {
            out.devices_page = true;
        } else if a == "--landscape-page" {
            out.landscape_page = true;
        } else if a == "--patterns-page" {
            out.patterns_page = true;
        } else if a == "--lint-page" {
            out.lint_page = true;
        } else if a == "--paper-page" {
            out.paper_page = true;
        } else if a == "--paper-landscape-page" {
            out.paper_landscape_page = true;
        } else if a == "--pattern-angle-page" {
            out.pattern_angle_page = true;
        } else if a == "--turned-patterns-page" {
            out.turned_patterns_page = true;
        } else if a == "--repeats-page" {
            out.repeats_page = true;
        } else if a == "--profiles" {
            out.profiles = true;
        } else if a == "--look" {
            match args.next().and_then(|v| v.into_string().ok()) {
                Some(v) if LOOKS.contains(&v.as_str()) => {
                    out.look = Some(v)
                }
                _ => return Err("--look takes lint, style, preflight, export, export-preset, export-mistake, exported, exported-open, welcome or file-menu".into()),
            }
        } else if a == "--fonts-page" {
            out.fonts_page = true;
        } else if a == "--textbox-page" {
            out.textbox_page = true;
        } else if a == "--missing-fonts-page" {
            out.missing_fonts_page = true;
        } else if a == "--bench-secondary" {
            out.bench_secondary = true;
        } else if a == "--no-strip" {
            out.no_strip = true;
        } else if a == "--bench-tool" {
            let name = args.next().and_then(|n| n.into_string().ok());
            match name.as_deref().and_then(quadrule_ui::Tool::from_name) {
                Some(t) if t != quadrule_ui::Tool::Select => out.bench_tool = Some(t),
                _ => {
                    return Err(
                        "--bench-tool takes line, text, pattern, rect, ellipse or polygon".into(),
                    );
                }
            }
        } else {
            out.path = Some(PathBuf::from(a));
        }
    }
    Ok(out)
}

fn main() -> eframe::Result {
    let args = match parse_args(std::env::args_os().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("quadrule: {e}");
            std::process::exit(2);
        }
    };
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_options(),
        viewport: egui::ViewportBuilder::default()
            .with_app_id("quadrule")
            .with_inner_size([1280.0, 860.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "quadrule",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, args)))),
    )
}

#[cfg(test)]
mod tests {
    use super::{Args, parse_args, parse_gtk_font};

    #[test]
    fn command_line() {
        let p = |v: &[&str]| parse_args(v.iter().map(std::ffi::OsString::from));
        let cases = [
            (p(&[]), Ok(Args::default())),
            (
                p(&["a.quad"]),
                Ok(Args {
                    path: Some("a.quad".into()),
                    chip_canvas: false,
                    bench_drag: None,
                    ..Args::default()
                }),
            ),
            (
                p(&["--canvas", "chip", "a.quad"]),
                Ok(Args {
                    path: Some("a.quad".into()),
                    chip_canvas: true,
                    bench_drag: None,
                    ..Args::default()
                }),
            ),
            (
                p(&["--bench-drag"]),
                Ok(Args {
                    path: None,
                    chip_canvas: false,
                    bench_drag: Some(150),
                    ..Args::default()
                }),
            ),
            (
                p(&["--bench-drag", "60", "--canvas", "chip"]),
                Ok(Args {
                    path: None,
                    chip_canvas: true,
                    bench_drag: Some(60),
                    ..Args::default()
                }),
            ),
            (
                p(&["--bench-draw", "40"]),
                Ok(Args {
                    bench_draw: Some(40),
                    ..Args::default()
                }),
            ),
            (
                p(&[
                    "--shapes-page",
                    "--bench-draw",
                    "40",
                    "--bench-tool",
                    "Rect",
                ]),
                Ok(Args {
                    bench_draw: Some(40),
                    shapes_page: true,
                    bench_tool: Some(quadrule_ui::Tool::Rect),
                    ..Args::default()
                }),
            ),
            (
                p(&["--groups-page", "--bench-drag", "30"]),
                Ok(Args {
                    bench_drag: Some(30),
                    groups_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&[
                    "--transforms-page",
                    "--bench-draw",
                    "30",
                    "--bench-tool",
                    "line",
                ]),
                Ok(Args {
                    bench_draw: Some(30),
                    transforms_page: true,
                    bench_tool: Some(quadrule_ui::Tool::Line),
                    ..Args::default()
                }),
            ),
            (
                p(&["--bench-rotate", "60"]),
                Ok(Args {
                    bench_rotate: Some(60),
                    ..Args::default()
                }),
            ),
            (
                p(&["--shapes-page", "--bench-drag"]),
                Ok(Args {
                    bench_drag: Some(150),
                    shapes_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--patterns-page", "--bench-draw", "20"]),
                Ok(Args {
                    bench_draw: Some(20),
                    patterns_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--lint-page", "--bench-drag", "30"]),
                Ok(Args {
                    bench_drag: Some(30),
                    lint_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--lint-page", "--look", "preflight"]),
                Ok(Args {
                    lint_page: true,
                    look: Some("preflight".into()),
                    ..Args::default()
                }),
            ),
            (
                p(&["--look", "sparkles"]),
                Err("--look takes lint, style, preflight, export, export-preset, export-mistake, exported, exported-open, welcome or file-menu".into()),
            ),
            (
                p(&["--paper-page", "--bench-drag", "30"]),
                Ok(Args {
                    bench_drag: Some(30),
                    paper_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--pattern-angle-page", "--bench-drag", "30"]),
                Ok(Args {
                    bench_drag: Some(30),
                    pattern_angle_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--turned-patterns-page", "--bench-rotate", "30"]),
                Ok(Args {
                    bench_rotate: Some(30),
                    turned_patterns_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--repeats-page", "--bench-drag", "40"]),
                Ok(Args {
                    bench_drag: Some(40),
                    repeats_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--profiles"]),
                Ok(Args {
                    profiles: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--textbox-page", "--bench-type", "40"]),
                Ok(Args {
                    bench_type: Some(40),
                    textbox_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--fonts-page", "--bench-drag", "40"]),
                Ok(Args {
                    bench_drag: Some(40),
                    fonts_page: true,
                    ..Args::default()
                }),
            ),
            (
                p(&["--repeats-page", "--bench-draw", "20"]),
                Ok(Args {
                    bench_draw: Some(20),
                    repeats_page: true,
                    ..Args::default()
                }),
            ),
        ];
        let good = cases.iter().all(|(got, want)| got == want);
        let bad = p(&["--canvas", "gpu"]).is_err()
            && p(&["--canvas"]).is_err()
            && p(&["--bench-tool", "blob"]).is_err()
            && p(&["--bench-tool"]).is_err();
        let ok = good && bad;
        println!(
            "CHECK crate=app name=command_line result={} cases={} bad_refused={bad}",
            if ok { "pass" } else { "fail" },
            cases.len()
        );
        assert!(ok);
    }

    #[test]
    fn gtk_font_names() {
        // The Raspberry Pi desktop's setting (2026-11-09) and the shapes GNOME uses.
        let cases = [
            ("'Nunito Sans Light 12'\n", Some(("Nunito Sans", 300))),
            ("'Cantarell 11'", Some(("Cantarell", 400))),
            ("'Ubuntu Bold Italic 10.5'", Some(("Ubuntu", 700))),
            ("'Noto Sans'", Some(("Noto Sans", 400))),
            ("''", None),
            ("", None),
        ];
        let wrong: Vec<_> = cases
            .iter()
            .filter(|(i, o)| parse_gtk_font(i) != o.map(|(f, w): (&str, u16)| (f.to_string(), w)))
            .map(|(i, _)| format!("{i:?}->{:?}", parse_gtk_font(i)))
            .collect();
        let ok = wrong.is_empty();
        println!(
            "CHECK crate=app name=gtk_font_names result={} cases={} wrong={wrong:?}",
            if ok { "pass" } else { "fail" },
            cases.len()
        );
        assert!(ok);
    }
}
