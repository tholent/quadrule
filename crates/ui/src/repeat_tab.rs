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

//! The inspector's Repeat tab (rung 10, docs/bbq/2026-11-13-rung-10-repeat-tool.md, Defaulted "The
//! Repeat tab"): shown when a repeat is selected, with every setting and Expand. Every edit is one
//! undo step through [`crate::viewer::Viewer::set_repeat`]; every word is a Fluent message.

use crate::view::ViewerUi;
use egui::RichText;
use quadrule_i18n::Message;
use quadrule_layout::units::{Axis, format_spacing, parse_spacing};
use quadrule_model::{
    AlternationRule, FillRole, LineRole, Object, REPEAT_COUNT_MAX, RepeatMode, RepeatShape,
    RuleAction, RuleCounts, RuleWhen, Spacing, SpacingBasis, Uuid,
};

/// What a rule does, without its role.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Does {
    Skip,
    Line,
    Fill,
}

fn does_of(a: RuleAction) -> Does {
    match a {
        RuleAction::Skip => Does::Skip,
        RuleAction::LineRole(_) => Does::Line,
        RuleAction::FillRole(_) => Does::Fill,
    }
}

/// Reads "3, 7" (commas or spaces) as the numbers 1 and up it names.
fn parse_numbers(text: &str) -> Vec<u32> {
    let mut out: Vec<u32> = text
        .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter_map(|t| t.trim().parse::<u32>().ok())
        .filter(|n| *n >= 1)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn numbers_text(list: &[u32]) -> String {
    list.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

impl ViewerUi {
    /// The first selected repeat.
    pub(crate) fn repeat_subject(&self) -> Option<Object> {
        let doc = self.viewer.doc()?;
        self.viewer
            .selection
            .iter()
            .filter_map(|id| doc.object(*id))
            .find(|o| o.is_repeat())
            .cloned()
    }

    /// The Repeat tab.
    pub(crate) fn repeat_tab(&mut self, ui: &mut egui::Ui) {
        let Some(o) = self.repeat_subject() else {
            return;
        };
        let Some(r) = o.repeat().cloned() else {
            return;
        };
        let id = o.id;
        let editable = self.viewer.doc().is_some_and(|d| d.editable(id));
        if !editable {
            ui.colored_label(ui.visuals().warn_fg_color, self.t("app-repeat-locked"));
        }
        let (Some(cx), Some(cy)) = (
            self.viewer.unit_context(Axis::X),
            self.viewer.unit_context(Axis::Y),
        ) else {
            return;
        };
        ui.add_enabled_ui(editable, |ui| {
            egui::Grid::new("repeat-grid")
                .num_columns(2)
                .show(ui, |ui| {
                    // Layout.
                    let label = ui.label(self.t("app-repeat-layout"));
                    let mut mode = r.mode;
                    self.combo(
                        ui,
                        "repeat-mode",
                        label.id,
                        &mut mode,
                        &[
                            (RepeatMode::Across, "app-repeat-across"),
                            (RepeatMode::Down, "app-repeat-down"),
                            (RepeatMode::Grid, "app-repeat-grid"),
                        ],
                    );
                    if mode != r.mode {
                        let _ = self.viewer.set_repeat(id, "repeat-mode", |s| {
                            s.mode = mode;
                            if mode != RepeatMode::Grid {
                                // A line of copies counts copies: rows and columns are one.
                                for rule in &mut s.rules {
                                    rule.counts = RuleCounts::Cells;
                                }
                            }
                        });
                    }
                    ui.end_row();

                    // Count or Fill.
                    let label = ui.label(self.t("app-repeat-how-many"));
                    let mut fill = r.fill;
                    self.combo(
                        ui,
                        "repeat-fill",
                        label.id,
                        &mut fill,
                        &[
                            (false, "app-repeat-count-fixed"),
                            (true, "app-repeat-count-fill"),
                        ],
                    );
                    if fill != r.fill {
                        let _ = self.viewer.set_repeat(id, "repeat-fill", |s| s.fill = fill);
                    }
                    ui.end_row();

                    if !r.fill {
                        match r.mode {
                            RepeatMode::Across | RepeatMode::Down => {
                                if let Some(v) = self.count_row(ui, "app-repeat-copies", r.count) {
                                    let _ =
                                        self.viewer.set_repeat(id, "repeat-count", |s| s.count = v);
                                }
                                ui.end_row();
                            }
                            RepeatMode::Grid => {
                                if let Some(v) = self.count_row(ui, "app-repeat-columns", r.columns)
                                {
                                    let _ = self
                                        .viewer
                                        .set_repeat(id, "repeat-columns", |s| s.columns = v);
                                }
                                ui.end_row();
                                if let Some(v) = self.count_row(ui, "app-repeat-rows", r.rows) {
                                    let _ =
                                        self.viewer.set_repeat(id, "repeat-rows", |s| s.rows = v);
                                }
                                ui.end_row();
                            }
                        }
                    }

                    // Pitch or gap.
                    let label = ui.label(self.t("app-repeat-spacing-by"));
                    let mut basis = r.basis;
                    self.combo(
                        ui,
                        "repeat-basis",
                        label.id,
                        &mut basis,
                        &[
                            (SpacingBasis::Pitch, "app-repeat-pitch"),
                            (SpacingBasis::Gap, "app-repeat-gap"),
                        ],
                    );
                    if basis != r.basis {
                        let _ = self
                            .viewer
                            .set_repeat(id, "repeat-basis", |s| s.basis = basis);
                    }
                    ui.end_row();

                    let (ax, ay) = r.axes();
                    if ax
                        && let Some(sp) = self.spacing_row(
                            ui,
                            "repeat-step-x",
                            "app-repeat-step-across",
                            r.h_spacing,
                            &cx,
                        )
                    {
                        let _ = self
                            .viewer
                            .set_repeat(id, "repeat-step-x", |s| s.h_spacing = sp);
                    }
                    if ax {
                        ui.end_row();
                    }
                    if ay
                        && let Some(sp) = self.spacing_row(
                            ui,
                            "repeat-step-y",
                            "app-repeat-step-down",
                            r.v_spacing,
                            &cy,
                        )
                    {
                        let _ = self
                            .viewer
                            .set_repeat(id, "repeat-step-y", |s| s.v_spacing = sp);
                    }
                    if ay {
                        ui.end_row();
                    }

                    // Start or centre.
                    let label = ui.label(self.t("app-repeat-leftover"));
                    let mut left = r.leftover;
                    self.combo(
                        ui,
                        "repeat-leftover",
                        label.id,
                        &mut left,
                        &[
                            (quadrule_model::Leftover::Start, "app-repeat-start"),
                            (quadrule_model::Leftover::Centre, "app-repeat-centre"),
                        ],
                    );
                    if left != r.leftover {
                        let _ = self
                            .viewer
                            .set_repeat(id, "repeat-leftover", |s| s.leftover = left);
                    }
                    ui.end_row();
                });
            if r.fill {
                ui.label(RichText::new(self.t("app-repeat-fill-note")).small());
            }
            self.repeat_info(ui, id, &r);
            ui.add_space(6.0);
            self.rules_section(ui, id, &r);
            ui.add_space(8.0);
            if ui
                .button(self.t("app-repeat-expand"))
                .on_hover_text(self.t("app-repeat-expand-hint"))
                .clicked()
            {
                let _ = self.viewer.expand_selection();
            }
        });
    }

    /// How many copies the device on screen draws.
    fn repeat_info(&mut self, ui: &mut egui::Ui, id: Uuid, r: &RepeatShape) {
        let Some(plan) = self.viewer.page().and_then(|p| {
            p.placed
                .iter()
                .find(|q| q.id == id && !q.copy)
                .and_then(|q| q.node.plan)
        }) else {
            return;
        };
        let msg = Message::new("app-repeat-drawn")
            .num("drawn", plan.drawn(r) as f64, 0)
            .num("total", plan.total() as f64, 0);
        ui.label(RichText::new(self.strings.format(&msg)).small());
        if plan.capped() {
            let msg = Message::new("app-repeat-capped-note").num(
                "max",
                quadrule_model::REPEAT_COPIES_MAX as f64,
                0,
            );
            ui.label(RichText::new(self.strings.format(&msg)).small());
        }
    }

    fn count_row(&mut self, ui: &mut egui::Ui, label_id: &str, value: u32) -> Option<u32> {
        let label = ui.label(self.t(label_id));
        let mut v = value;
        let changed = ui
            .add(egui::DragValue::new(&mut v).range(1..=REPEAT_COUNT_MAX))
            .labelled_by(label.id)
            .changed();
        changed.then_some(v)
    }

    /// A step field: `auto`, `2u` (follows the rhythm) or a length (kept in mm).
    fn spacing_row(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        label_id: &str,
        value: Spacing,
        ctx: &quadrule_layout::units::UnitContext,
    ) -> Option<Spacing> {
        let label = ui.label(self.t(label_id));
        let auto = self.t("app-repeat-auto");
        let id = ui.make_persistent_id(("spacing", key));
        let focused = ui.memory(|m| m.has_focus(id));
        let mut text = match (focused, self.field_text.get(key)) {
            (true, Some(t)) => t.clone(),
            _ => format_spacing(value, ctx, &auto),
        };
        let parsed = parse_spacing(&text, ctx, &[&auto]);
        let bad = focused && parsed.is_err();
        let mut edit = egui::TextEdit::singleline(&mut text)
            .id(id)
            .desired_width(110.0)
            .hint_text(self.t("app-repeat-step-hint"));
        if bad {
            edit = edit.background_color(ui.visuals().error_fg_color.gamma_multiply(0.25));
        }
        let resp = ui.add(edit).labelled_by(label.id);
        let resp = if bad {
            resp.on_hover_text(self.t("app-field-invalid"))
        } else {
            resp
        };
        if resp.lost_focus() || !resp.has_focus() {
            self.field_text.remove(key);
        }
        if resp.changed() {
            self.field_text.insert(key.to_string(), text.clone());
            if let Ok(sp) = parse_spacing(&text, ctx, &[&auto]) {
                return Some(sp);
            }
        }
        None
    }

    /// The alternation rules: add, remove, reorder and edit.
    fn rules_section(&mut self, ui: &mut egui::Ui, id: Uuid, r: &RepeatShape) {
        ui.label(RichText::new(self.t("app-repeat-rules")).strong());
        if r.rules.is_empty() {
            ui.label(RichText::new(self.t("app-repeat-rules-none")).small());
        } else {
            ui.label(RichText::new(self.t("app-repeat-rules-later")).small());
        }
        for (i, rule) in r.rules.iter().enumerate() {
            ui.separator();
            let key = format!("repeat-rule-{i}");
            // What it counts.
            let grid = r.is_grid();
            let mut counts = rule.counts;
            ui.horizontal(|ui| {
                let label = ui.label(self.t("app-repeat-rule-counts"));
                if grid {
                    self.combo(
                        ui,
                        &format!("{key}-counts"),
                        label.id,
                        &mut counts,
                        &[
                            (RuleCounts::Columns, "app-repeat-rule-columns"),
                            (RuleCounts::Rows, "app-repeat-rule-rows"),
                            (RuleCounts::Cells, "app-repeat-rule-cells"),
                        ],
                    );
                } else {
                    ui.label(self.t("app-repeat-rule-copies"));
                }
            });
            if counts != rule.counts {
                let _ = self
                    .viewer
                    .set_repeat(id, &format!("{key}-counts"), |s| s.rules[i].counts = counts);
            }
            // Which numbers.
            let every = matches!(rule.when, RuleWhen::Every { .. });
            let mut want_every = every;
            ui.horizontal(|ui| {
                let label = ui.label(self.t("app-repeat-rule-when"));
                self.combo(
                    ui,
                    &format!("{key}-when"),
                    label.id,
                    &mut want_every,
                    &[
                        (true, "app-repeat-rule-every"),
                        (false, "app-repeat-rule-list"),
                    ],
                );
            });
            if want_every != every {
                let _ = self.viewer.set_repeat(id, &format!("{key}-when"), |s| {
                    s.rules[i].when = if want_every {
                        RuleWhen::Every { n: 2, start: None }
                    } else {
                        RuleWhen::List(vec![1])
                    };
                });
            }
            match &rule.when {
                RuleWhen::Every { n, start } => {
                    let (mut nn, mut ss) = (*n, start.unwrap_or(*n));
                    ui.horizontal(|ui| {
                        let l1 = ui.label(self.t("app-repeat-rule-n"));
                        let c1 = ui
                            .add(egui::DragValue::new(&mut nn).range(1..=REPEAT_COUNT_MAX))
                            .labelled_by(l1.id)
                            .changed();
                        let l2 = ui.label(self.t("app-repeat-rule-from"));
                        let c2 = ui
                            .add(egui::DragValue::new(&mut ss).range(1..=REPEAT_COUNT_MAX))
                            .labelled_by(l2.id)
                            .changed();
                        if c1 || c2 {
                            // A start equal to N is the default and is not stored.
                            let _ = self.viewer.set_repeat(id, &format!("{key}-every"), |s| {
                                s.rules[i].when = RuleWhen::Every {
                                    n: nn,
                                    start: (!(ss == nn && !c2)).then_some(ss),
                                };
                            });
                        }
                    });
                }
                RuleWhen::List(list) => {
                    let fkey = format!("{key}-numbers");
                    let tid = ui.make_persistent_id(("rule-numbers", &fkey));
                    let focused = ui.memory(|m| m.has_focus(tid));
                    let mut text = match (focused, self.field_text.get(&fkey)) {
                        (true, Some(t)) => t.clone(),
                        _ => numbers_text(list),
                    };
                    let changed = ui
                        .horizontal(|ui| {
                            let label = ui.label(self.t("app-repeat-rule-numbers"));
                            let resp = ui
                                .add(
                                    egui::TextEdit::singleline(&mut text)
                                        .id(tid)
                                        .desired_width(110.0)
                                        .hint_text(self.t("app-repeat-rule-numbers-hint")),
                                )
                                .labelled_by(label.id);
                            if resp.lost_focus() || !resp.has_focus() {
                                self.field_text.remove(&fkey);
                            }
                            resp.changed()
                        })
                        .inner;
                    if changed {
                        self.field_text.insert(fkey, text.clone());
                        let nums = parse_numbers(&text);
                        let _ = self.viewer.set_repeat(id, &format!("{key}-list"), |s| {
                            s.rules[i].when = RuleWhen::List(nums);
                        });
                    }
                }
            }
            // What it does.
            let does = does_of(rule.action);
            let mut want = does;
            ui.horizontal(|ui| {
                let label = ui.label(self.t("app-repeat-rule-does"));
                self.combo(
                    ui,
                    &format!("{key}-does"),
                    label.id,
                    &mut want,
                    &[
                        (Does::Skip, "app-repeat-rule-skip"),
                        (Does::Line, "app-repeat-rule-line"),
                        (Does::Fill, "app-repeat-rule-fill"),
                    ],
                );
            });
            if want != does {
                let _ = self.viewer.set_repeat(id, &format!("{key}-does"), |s| {
                    s.rules[i].action = match want {
                        Does::Skip => RuleAction::Skip,
                        Does::Line => RuleAction::LineRole(LineRole::Structure),
                        Does::Fill => RuleAction::FillRole(FillRole::Shade),
                    };
                });
            }
            match rule.action {
                RuleAction::LineRole(role) => {
                    let mut pick = role;
                    ui.horizontal(|ui| {
                        let label = ui.label(self.t("app-repeat-rule-role"));
                        self.combo(
                            ui,
                            &format!("{key}-line"),
                            label.id,
                            &mut pick,
                            &crate::panels::LINE_ROLE_OPTIONS,
                        );
                    });
                    if pick != role {
                        let _ = self.viewer.set_repeat(id, &format!("{key}-line"), |s| {
                            s.rules[i].action = RuleAction::LineRole(pick);
                        });
                    }
                }
                RuleAction::FillRole(role) => {
                    let mut pick = role;
                    ui.horizontal(|ui| {
                        let label = ui.label(self.t("app-repeat-rule-role"));
                        self.combo(
                            ui,
                            &format!("{key}-fill"),
                            label.id,
                            &mut pick,
                            &crate::panels::FILL_ROLE_OPTIONS,
                        );
                    });
                    if pick != role {
                        let _ = self.viewer.set_repeat(id, &format!("{key}-fill"), |s| {
                            s.rules[i].action = RuleAction::FillRole(pick);
                        });
                    }
                }
                RuleAction::Skip => {}
            }
            // Order and removal.
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(i > 0, egui::Button::new(self.t("app-move-up")))
                    .on_hover_text(self.t("app-repeat-rule-up"))
                    .clicked()
                {
                    let _ = self.viewer.set_repeat(id, "repeat-rule-order", |s| {
                        s.rules.swap(i, i - 1);
                    });
                }
                if ui
                    .add_enabled(
                        i + 1 < r.rules.len(),
                        egui::Button::new(self.t("app-move-down")),
                    )
                    .on_hover_text(self.t("app-repeat-rule-down"))
                    .clicked()
                {
                    let _ = self.viewer.set_repeat(id, "repeat-rule-order", |s| {
                        s.rules.swap(i, i + 1);
                    });
                }
                if ui.button(self.t("app-repeat-rule-remove")).clicked() {
                    let _ = self.viewer.set_repeat(id, "repeat-rule-remove", |s| {
                        s.rules.remove(i);
                    });
                    self.viewer.seal();
                }
            });
        }
        ui.separator();
        if ui.button(self.t("app-repeat-rule-add")).clicked() {
            let grid = r.is_grid();
            let _ = self.viewer.set_repeat(id, "repeat-rule-add", |s| {
                let mut rule = AlternationRule::new();
                if grid {
                    rule.counts = RuleCounts::Columns;
                }
                s.rules.push(rule);
            });
            self.viewer.seal();
        }
    }
}
