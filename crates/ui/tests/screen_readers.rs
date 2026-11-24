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

//! Screen readers (rung 17 bbq, Q5; debt D8): walks the AccessKit tree of every window and sheet and fails on any
//! button, text field, tick box, choice or slider with no name, and on two controls in one window sharing a name. The
//! canvas itself is out of v1's scope (spec section 14), so its pixels are not a control here; its toolbar, panels,
//! menus and sheets are. Headless. The test prints one structured `CHECK` line.

use egui::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::sample::{device_ids, ids, repeat_ids};
use quadrule_ui::{Tab, Viewer, ViewerUi};
use std::collections::BTreeMap;

struct Window {
    ui: ViewerUi,
}

/// A window of this size on a page (or on the welcome screen), set up by `setup`.
fn window(open: fn(&mut Viewer), setup: impl FnOnce(&mut ViewerUi)) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    open(&mut v);
    let mut ui = ViewerUi::new(v);
    setup(&mut ui);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1100.0))
        .with_max_steps(24)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window { ui },
        );
    h.run();
    h.run();
    h
}

fn sample(v: &mut Viewer) {
    v.open_sample();
}
fn none(_: &mut Viewer) {}

/// The controls a person using a screen reader steers by.
fn is_control(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::Slider
            | Role::SpinButton
            | Role::ComboBox
    )
}

/// What the tree of `h` holds that a screen reader cannot use: the controls with no name, and the names two
/// controls share.
fn audit(h: &Harness<'static, Window>) -> (usize, Vec<String>, Vec<String>) {
    let mut controls = 0;
    let mut unnamed = Vec::new();
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    for node in h.root().children_recursive() {
        let a = node.accesskit_node();
        if a.is_hidden() || !is_control(a.role()) {
            continue;
        }
        controls += 1;
        let name = a.label().unwrap_or_default();
        if name.trim().is_empty() {
            unnamed.push(format!("{:?} at {:?}", a.role(), node.rect()));
        } else {
            *names.entry(format!("{:?} {name}", a.role())).or_default() += 1;
        }
    }
    let duplicates = names
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(name, n)| format!("{name} x{n}"))
        .collect();
    (controls, unnamed, duplicates)
}

#[test]
fn every_window_and_sheet_names_its_controls_once() {
    type Setup = Box<dyn Fn() -> Harness<'static, Window>>;
    let looks =
        |name: &'static str| -> Setup { Box::new(move || window(sample, |ui| ui.show_look(name))) };
    let tab = |tab: Tab, select: Option<quadrule_model::Uuid>, nomad: bool| -> Setup {
        Box::new(move || {
            window(sample, move |ui| {
                if let Some(id) = select {
                    ui.viewer.selection = vec![id];
                }
                if nomad {
                    ui.viewer.set_profile("supernote.nomad");
                }
                ui.tab = tab;
            })
        })
    };
    let scenarios: Vec<(&str, Setup)> = vec![
        ("welcome", Box::new(|| window(none, |_| {}))),
        ("tab basic, nothing picked", tab(Tab::Basic, None, false)),
        ("tab page", tab(Tab::Page, None, false)),
        ("tab style", tab(Tab::Style, Some(ids::RULE), false)),
        ("tab arrange", tab(Tab::Arrange, Some(ids::RULE), false)),
        ("tab arrange, two picked", {
            Box::new(|| {
                window(sample, |ui| {
                    ui.viewer.selection = vec![ids::RULE, ids::DOTS, ids::LINES];
                    ui.tab = Tab::Arrange;
                })
            })
        }),
        ("tab text", tab(Tab::Text, Some(ids::TITLE), false)),
        (
            "tab constraints",
            tab(Tab::Constraints, Some(ids::RULE), false),
        ),
        ("tab overrides", {
            Box::new(|| {
                window(
                    |v| v.open_devices(),
                    |ui| {
                        ui.viewer.selection = vec![device_ids::MOVED];
                        ui.viewer.set_profile("supernote.nomad");
                        ui.tab = Tab::Overrides;
                    },
                )
            })
        }),
        ("tab repeat", {
            Box::new(|| {
                window(
                    // The page's objects are given names of their own: two boxes both called "Box" would share a name
                    // because the person named them alike, which no label can mend.
                    |v| {
                        let mut doc = quadrule_model::sample::repeats_document();
                        let ids: Vec<_> = doc.objects().map(|o| o.id).collect();
                        for (n, id) in ids.into_iter().enumerate() {
                            if let Some(o) = doc.object_mut(id) {
                                o.name = format!("{} {n}", o.name);
                            }
                        }
                        v.open_document_for_test(doc);
                    },
                    |ui| {
                        ui.viewer.selection = vec![repeat_ids::ACROSS];
                        ui.tab = Tab::Repeat;
                    },
                )
            })
        }),
        ("tab lint, the lint page", {
            Box::new(|| {
                window(
                    |v| v.open_lint(),
                    |ui| {
                        ui.tab = Tab::Lint;
                    },
                )
            })
        }),
        ("menu file", looks("menu-file")),
        ("menu edit", looks("menu-edit")),
        ("menu view and snapping", looks("menu-view")),
        ("menu help", looks("menu-help")),
        ("menu help, developer", looks("menu-developer")),
        ("about", looks("about")),
        ("preferences", looks("preferences")),
        ("keyboard shortcuts", looks("shortcuts")),
        ("export sheet", looks("export")),
        ("export sheet, a mistake", looks("export-mistake")),
        ("export done", looks("exported")),
        ("preflight sheet", {
            Box::new(|| window(|v| v.open_lint(), |ui| ui.show_look("preflight")))
        }),
        ("profiles window", {
            Box::new(|| window(sample, |ui| ui.open_profiles_window()))
        }),
        ("layers, delete sheet", looks("layers-ask")),
        ("layers, new layer", looks("layers-new")),
        ("layers, row menu", looks("layers-menu")),
        ("welcome, file menu", looks("file-menu")),
        ("welcome over a page", {
            Box::new(|| window(sample, |ui| ui.show_look("welcome")))
        }),
    ];
    let mut total_controls = 0;
    let mut windows = 0;
    let mut problems = Vec::new();
    for (name, build) in &scenarios {
        let h = build();
        let (controls, unnamed, duplicates) = audit(&h);
        total_controls += controls;
        windows += 1;
        if !unnamed.is_empty() || !duplicates.is_empty() {
            problems.push(format!(
                "[{name}] unnamed={unnamed:?} shared={duplicates:?}"
            ));
        }
    }
    println!(
        "\nCHECK crate=ui name=screen_readers result={} windows={windows} controls={total_controls} problems={} {}",
        if problems.is_empty() { "pass" } else { "fail" },
        problems.len(),
        problems.join(" | ")
    );
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn the_font_and_emoji_pickers_the_missing_fonts_dialog_and_the_save_sheet_name_their_controls_once()
{
    let mut problems = Vec::new();
    let mut windows = 0;
    let mut controls = 0;
    let mut check = |name: &str, h: &Harness<'static, Window>| {
        let (c, unnamed, duplicates) = audit(h);
        controls += c;
        windows += 1;
        if !unnamed.is_empty() || !duplicates.is_empty() {
            problems.push(format!(
                "[{name}] unnamed={unnamed:?} shared={duplicates:?}"
            ));
        }
    };
    // The font picker, over a text box.
    let mut h = window(sample, |ui| {
        ui.viewer.selection = vec![ids::TITLE];
        ui.tab = Tab::Text;
    });
    h.get_by_label("Font").click();
    h.run();
    h.run();
    check("font picker", &h);
    // The emoji picker.
    let mut h = window(sample, |ui| {
        ui.viewer.selection = vec![ids::TITLE];
        ui.tab = Tab::Text;
    });
    h.get_all_by_label("Emoji").next().unwrap().click();
    h.run();
    h.run();
    check("emoji picker", &h);
    // The Missing fonts dialog, and the bar above the page.
    let mut h = window(|v| v.open_missing_fonts(), |ui| ui.open_missing_dialog());
    h.run();
    check("missing fonts dialog and bar", &h);
    // The Save sheet that warns about installed fonts: a text box set in a font installed on this computer, if there
    // is one to pick (the sheet only lists installed fonts).
    let mut h = window(sample, |ui| {
        ui.viewer.use_system_fonts();
        ui.viewer.selection = vec![ids::TITLE];
        ui.tab = Tab::Text;
    });
    h.get_by_label("Font").click();
    h.run();
    let installed = h.state().ui.font_picker_names().pop();
    let mut sheet = false;
    if let Some(family) = installed {
        let bundled = quadrule_text::BUNDLED.iter().any(|b| b.family == family);
        if !bundled {
            h.state_mut()
                .ui
                .viewer
                .edit_selected(None, |x| {
                    if let quadrule_model::ObjectKind::Text(t) = &mut x.kind {
                        t.family.clone_from(&family);
                    }
                })
                .unwrap();
            let _ = h.state_mut().ui.save();
            h.run();
            h.run();
            sheet = h.state().ui.font_sheet_open();
            if sheet {
                check("save sheet for fonts", &h);
            }
        }
    }
    println!(
        "\nCHECK crate=ui name=screen_readers_pickers result={} windows={windows} controls={controls} save_sheet_showed={sheet} problems={} {}",
        if problems.is_empty() { "pass" } else { "fail" },
        problems.len(),
        problems.join(" | ")
    );
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn the_new_page_dialog_the_real_size_check_and_the_unsaved_prompt_name_their_controls_once() {
    use quadrule_ui::Cmd;
    let mut problems = Vec::new();
    let mut windows = 0;
    let mut controls = 0;
    for (name, cmd, edit_first) in [
        ("new page dialog", Cmd::New, false),
        ("real size check", Cmd::RealSizeCheck, false),
        ("unsaved changes prompt", Cmd::Open, true),
        ("save as template", Cmd::SaveTemplate, false),
    ] {
        let mut h = window(sample, |_| {});
        if edit_first {
            h.state_mut().ui.viewer.selection = vec![ids::RULE];
            h.state_mut().ui.viewer.nudge(1.0, 0.0, false).unwrap();
            h.run();
        }
        let ctx = h.ctx.clone();
        let _ = h.state_mut().ui.run_cmd(&ctx, cmd);
        h.run();
        h.run();
        let (c, unnamed, duplicates) = audit(&h);
        controls += c;
        windows += 1;
        if !unnamed.is_empty() || !duplicates.is_empty() {
            problems.push(format!(
                "[{name}] unnamed={unnamed:?} shared={duplicates:?}"
            ));
        }
    }
    println!(
        "\nCHECK crate=ui name=screen_readers_sheets result={} windows={windows} controls={controls} problems={} {}",
        if problems.is_empty() { "pass" } else { "fail" },
        problems.len(),
        problems.join(" | ")
    );
    assert!(problems.is_empty(), "{problems:#?}");
}
