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

//! `quadrule`, the app (napkin §15.1). Rung 2: the viewer window
//! (docs/bbq/2026-11-09-rung-2-viewer.md). Usage: `quadrule [file.quad]`, or
//! `cargo run -p quadrule-app -- [file.quad]`.
//!
//! This crate holds only what needs a real window: eframe setup, the graphics-chip connection,
//! the system file dialogs and remembered preferences. The window itself is `quadrule-ui`.

#![forbid(unsafe_code)]

use eframe::egui;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu;
use quadrule_ui::{Request, Viewer, ViewerUi};
use std::path::PathBuf;
use std::sync::Arc;

/// Preference keys in eframe's storage.
const KEY_THEME: &str = "theme";
const KEY_RULER: &str = "screen_px_per_mm";

struct App {
    ui: ViewerUi,
    title_for: u64,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        quadrule_ui::install_fonts(&cc.egui_ctx, desktop_ui_font());
        let mut ui = ViewerUi::new(Viewer::default());
        if let Some(storage) = cc.storage {
            if let Some(theme) = eframe::get_value::<String>(storage, KEY_THEME) {
                ui.theme = match theme.as_str() {
                    "light" => egui::ThemePreference::Light,
                    "dark" => egui::ThemePreference::Dark,
                    _ => egui::ThemePreference::System,
                };
            }
            ui.viewer.screen_px_per_mm = eframe::get_value(storage, KEY_RULER);
        }
        cc.egui_ctx.set_theme(ui.theme);
        if let Some(p) = path {
            let _ = ui.viewer.open_path(&p);
        }
        App {
            ui,
            title_for: u64::MAX,
        }
    }

    fn answer(&mut self, req: Request) {
        match req {
            Request::OpenDialog => {
                let picked = rfd::FileDialog::new()
                    .add_filter("quadrule", &["quad"])
                    .pick_file();
                if let Some(path) = picked {
                    let _ = self.ui.viewer.open_path(&path);
                }
            }
            Request::ExportDialog(which) => {
                let mut dialog = rfd::FileDialog::new();
                if let Some(dir) = &self.ui.viewer.export_dir {
                    dialog = dialog.set_directory(dir);
                }
                if let Some(dir) = dialog.pick_folder() {
                    // A failure is shown in the status bar by the viewer itself.
                    let _ = self.ui.viewer.export(&dir, which);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for req in self.ui.show(ui) {
            self.answer(req);
        }
        // The window title names what is open.
        if self.title_for != self.ui.viewer.generation {
            self.title_for = self.ui.viewer.generation;
            let app = self.ui.strings.get("app-title");
            let title = match self.ui.viewer.title() {
                Some(t) => format!("{t} — {app}"),
                None if self.ui.viewer.open.is_some() => {
                    format!("{} — {app}", self.ui.strings.get("app-open-sample"))
                }
                None => app,
            };
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(title));
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

fn main() -> eframe::Result {
    let path = std::env::args_os().nth(1).map(PathBuf::from);
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
        Box::new(|cc| Ok(Box::new(App::new(cc, path)))),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_gtk_font;

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
