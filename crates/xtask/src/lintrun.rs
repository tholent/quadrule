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

//! Lints a document on each of its profiles, the way `cargo xtask lint` does: shared by that command and by the
//! test that lints every starter (ADR-021, item 1).

use quadrule_layout::lay_out_text_saved;
use quadrule_model::Document;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_text::FontRegistry;
use std::collections::BTreeMap;

/// The toolbar positions the linter checks unless told otherwise: the docked ones (chris, 2026-11-10: "the safe area
/// linting should look at all safe areas, including the right one", and after rung 5 that checking the floating menu
/// bar "isn't desired"; on 2026-11-09 it was "Docked left, Docked top";
/// docs/device-measurements/supernote.a6x2/calibration.md). `all` adds the floating one.
pub const DEFAULT_TOOLBARS: &str = "left,right,top";

/// The options for a comma-separated list of toolbar positions (`left`, `right`, `top`, `floating`) or `all`.
pub fn options(toolbars: &str) -> Result<quadrule_lint::Options, String> {
    Ok(quadrule_lint::Options {
        toolbars: if toolbars == "all" {
            quadrule_lint::Toolbar::ALL.to_vec()
        } else {
            toolbars
                .split(',')
                .map(|s| {
                    quadrule_lint::Toolbar::parse(s.trim())
                        .ok_or_else(|| format!("unknown toolbar position {s:?}"))
                })
                .collect::<Result<_, _>>()?
        },
        ..quadrule_lint::Options::default()
    })
}

/// Lints `doc` on each of its profiles, with `saved` the text outlines the file carries (they draw the text whose font
/// this computer lacks, L11). One line per finding: profile, rule, severity, message.
pub fn lint_lines(
    doc: &Document,
    saved: &BTreeMap<String, ProfileOutlines>,
    opts: &quadrule_lint::Options,
) -> Result<Vec<String>, String> {
    let primary = quadrule_profiles::active::lookup_in(doc, &doc.primary_profile_id)
        .map_err(|e| e.to_string())?;
    let mut reg = FontRegistry::bundled_only();
    let strings = quadrule_i18n::Strings::english();
    let mut lines = Vec::new();
    for id in doc.profile_ids() {
        let target = quadrule_profiles::active::lookup_in(doc, id).map_err(|e| e.to_string())?;
        let (laid, how) = lay_out_text_saved(doc, &primary, &target, &mut reg, saved)
            .map_err(|e| e.to_string())?;
        let mut opts = opts.clone();
        for id in &laid.unavailable {
            if let Some(quadrule_model::ObjectKind::Text(t)) = doc.object(*id).map(|o| &o.kind) {
                opts.fonts.missing.push(quadrule_lint::MissingBox {
                    id: *id,
                    family: t.family.clone(),
                    needs_font: how.needs_font.contains(id),
                });
            }
        }
        for f in quadrule_lint::lint_with(doc, &primary, &target, &laid.outlines, &opts) {
            let message = strings.format(&f.message);
            lines.push(format!(
                "{}  {}  {:?}  {message}",
                f.profile, f.rule, f.severity
            ));
        }
    }
    Ok(lines)
}
