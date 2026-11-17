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

//! The Lint tab's and the preflight sheet's actions, with no screen in them (rung 14b,
//! docs/bbq/2026-11-17-rung-14-lint-and-preflight.md, Q5 to Q7): findings on every device, the quick fixes
//! of a finding and doing one as one undo step (through the device's overrides on a secondary device, the way
//! rung 8 edits work), Ignore this rule for this document and turning it back on, and Free placement.

use crate::viewer::{Notice, Viewer};
use quadrule_i18n::Message;
use quadrule_layout::rotate::rotate_point;
use quadrule_lint::{Edit, Finding, Fix, NoFix, offered, plan, plan_snap_all};
use quadrule_model::edit::{Command, EditError, set_font_size_override, set_position_override};
use quadrule_model::{Frame, ObjectKind};
use quadrule_profiles::active::lookup_in;

impl Viewer {
    /// The findings of each of the devices `ids`, in that order (each page is made on first use).
    pub fn findings_on(&mut self, ids: &[String]) -> Vec<(String, Vec<Finding>)> {
        ids.iter()
            .map(|id| {
                let f = self
                    .page_for(id)
                    .map(|p| p.findings.clone())
                    .unwrap_or_default();
                (id.clone(), f)
            })
            .collect()
    }

    /// The quick fixes a finding has, in the order their buttons show.
    pub fn fixes_for(&self, f: &Finding) -> Vec<Fix> {
        let Some(doc) = self.doc() else {
            return Vec::new();
        };
        let (Ok(primary), Ok(target)) = (
            lookup_in(doc, &doc.primary_profile_id),
            lookup_in(doc, &f.profile),
        ) else {
            return Vec::new();
        };
        offered(doc, &primary, &target, f)
    }

    /// Does quick fix `fix` for finding `f` as one undo step. `all` is the findings the Snap all button works
    /// from (the page's L16 findings on the primary device).
    pub fn apply_fix(&mut self, f: &Finding, fix: Fix, all: &[Finding]) -> Result<(), EditError> {
        let Some(doc) = self.doc() else {
            return Ok(());
        };
        let (Ok(primary), Ok(target)) = (
            lookup_in(doc, &doc.primary_profile_id),
            lookup_in(doc, &f.profile),
        ) else {
            return Ok(());
        };
        let planned = match fix {
            Fix::SnapAllToRhythm => plan_snap_all(doc, &primary, all),
            _ => plan(doc, &primary, &target, f, fix),
        };
        let edits = match planned {
            Ok(e) => e,
            Err(NoFix) => {
                self.notice = Some(Notice {
                    message: Message::new("lint-fix-none"),
                    error: true,
                });
                return Ok(());
            }
        };
        let cmds = self.commands_for(&f.profile, &primary.id, target.mm_to_px(1.0), edits);
        if cmds.is_empty() {
            return Ok(());
        }
        self.edit(Command::Batch(cmds), None)
    }

    /// The commands that make `edits` happen on device `dev` (`primary` is the document's primary id; `ppmm`
    /// the device's pixels per mm).
    fn commands_for(
        &mut self,
        dev: &str,
        primary: &str,
        ppmm: f64,
        edits: Vec<Edit>,
    ) -> Vec<Command> {
        let on_secondary = dev != primary;
        // Where each moved object stands on the device, for a move on the page and for overrides.
        let nodes: Vec<quadrule_layout::tree::Node> = match self.page_for(dev) {
            Ok(page) => page.placed.iter().map(|p| p.node).collect(),
            Err(_) => Vec::new(),
        };
        let Some(doc) = self.doc() else {
            return Vec::new();
        };
        let mut cmds = Vec::new();
        for e in edits {
            match e {
                Edit::Object(o) => cmds.push(Command::SetObject(o)),
                Edit::Rhythm(r) => cmds.push(Command::SetRhythm(r)),
                Edit::Translate { id, dx_mm, dy_mm } => {
                    if let Some(o) = doc.object(id) {
                        let mut o = o.clone();
                        o.translate(dx_mm, dy_mm);
                        cmds.push(Command::SetObject(o));
                    }
                }
                Edit::MovePage { id, dx_px, dy_px } => {
                    let Some(n) = nodes.iter().find(|n| n.id == id) else {
                        continue;
                    };
                    let Some(o) = doc.object(id) else { continue };
                    let step = (dx_px / ppmm, dy_px / ppmm);
                    // Inside a turned group the page's down and right are not the group's.
                    let (mx, my) = rotate_point(step, (0.0, 0.0), -n.ctx.xf.deg);
                    let mut o = o.clone();
                    if on_secondary {
                        let moved = Frame {
                            x_mm: n.local.x_mm + mx,
                            y_mm: n.local.y_mm + my,
                            ..n.local
                        };
                        let unfit = n.ctx.unfit(moved, o.rotation(), o.keeps_height());
                        set_position_override(&mut o, dev, n.base, unfit.x_mm, unfit.y_mm);
                    } else {
                        o.translate(mx, my);
                    }
                    cmds.push(Command::SetObject(o));
                }
                Edit::TextSize { id, pt } => {
                    let Some(o) = doc.object(id) else { continue };
                    let mut o = o.clone();
                    if on_secondary {
                        set_font_size_override(&mut o, dev, pt);
                    } else if let ObjectKind::Text(t) = &mut o.kind {
                        t.size_pt = pt;
                    }
                    cmds.push(Command::SetObject(o));
                }
            }
        }
        cmds
    }

    /// Opens `doc` as an untitled page, for tests that need a page of their own.
    #[doc(hidden)]
    pub fn open_document_for_test(&mut self, doc: quadrule_model::Document) {
        self.set_open(
            crate::viewer::Source::Untitled,
            doc,
            std::collections::BTreeMap::new(),
            None,
        );
    }

    /// The rules this document turns off, in the order they were turned off.
    pub fn ignored_rules(&self) -> Vec<String> {
        self.doc()
            .map(|d| d.ignored_rules.clone())
            .unwrap_or_default()
    }

    /// Turns rule `rule` off for this document, on every device (one undo step). Nothing happens if it is off.
    pub fn ignore_rule(&mut self, rule: &str) -> Result<(), EditError> {
        let mut rules = self.ignored_rules();
        if rules.iter().any(|r| r == rule) || self.doc().is_none() {
            return Ok(());
        }
        rules.push(rule.to_string());
        self.edit(Command::SetIgnoredRules(rules), None)
    }

    /// Turns rule `rule` back on for this document (one undo step).
    pub fn restore_rule(&mut self, rule: &str) -> Result<(), EditError> {
        let mut rules = self.ignored_rules();
        let before = rules.len();
        rules.retain(|r| r != rule);
        if rules.len() == before {
            return Ok(());
        }
        self.edit(Command::SetIgnoredRules(rules), None)
    }

    /// Marks the editable selected objects Free placement, or takes the mark off (one undo step): L16 leaves a
    /// marked object, and a group's mark covers what is in it.
    pub fn set_free_placement(&mut self, on: bool) -> Result<(), EditError> {
        self.edit_selected(None, |x| x.free_placement = on)
    }

    /// Shows device `dev` and outlines its finding number `index` on the page.
    pub fn show_finding(&mut self, dev: &str, index: usize) {
        self.set_profile(dev);
        self.selected = Some(index);
    }

    /// Jumps to the object a finding is about: shows its device, selects the object and outlines the finding
    /// (so the canvas scrolls to it). Returns whether the object is still there.
    pub fn jump_to(&mut self, f: &Finding) -> bool {
        let Some(id) = f.object else { return false };
        if self.doc().and_then(|d| d.object(id)).is_none() {
            return false;
        }
        self.set_profile(&f.profile);
        self.select_row(id, false);
        self.selected = self
            .page()
            .and_then(|p| p.findings.iter().position(|x| x == f));
        true
    }
}
