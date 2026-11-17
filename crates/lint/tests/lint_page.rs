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

//! The lint page (`quadrule --lint-page`; rung 14a): every object named "L03 ...", "L05 ..." and so on makes
//! that rule fire on it, on both Supernotes, every object named "Passes: ..." gets no finding at all, and the
//! page has no other finding but the ones meant. The page is a Template with the shipped profiles' values.
//! Prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::lay_out_text;
use quadrule_lint::*;
use quadrule_model::sample::*;
use quadrule_profiles::builtin;
use quadrule_text::FontRegistry;

#[test]
fn each_named_object_makes_its_rule_fire_and_the_passing_ones_stay_quiet() {
    let doc = lint_document();
    let primary = builtin(MANTA_ID).unwrap();
    let mut wrong = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut total = 0;
    for id in [MANTA_ID, NOMAD_ID] {
        let target = builtin(id).unwrap();
        let laid =
            lay_out_text(&doc, &primary, &target, &mut FontRegistry::bundled_only()).unwrap();
        let findings = lint(&doc, &primary, &target, &laid.outlines);
        total += findings.len();
        for o in doc.leaves() {
            let mine: Vec<&Finding> = findings.iter().filter(|f| f.object == Some(o.id)).collect();
            let first = o.name.split_whitespace().next().unwrap_or("");
            if let Some(rule) = first.strip_prefix('L').map(|n| format!("L{n}")) {
                if mine.iter().any(|f| f.rule == rule) {
                    seen.insert(rule);
                } else {
                    wrong.push(format!("{id}: \"{}\" lacks {rule}", o.name));
                }
            } else if o.name.starts_with("Passes") && !mine.is_empty() {
                wrong.push(format!(
                    "{id}: \"{}\" has {:?}",
                    o.name,
                    mine.iter().map(|f| f.rule).collect::<Vec<_>>()
                ));
            }
        }
        // Nothing else is said but the six rules, L05/L03 together on the pale ones, and the
        // page-wide notes the sample already carries.
        for f in &findings {
            // (L18 is the slice's own note about how the rhythm fits the Nomad's margins.)
            if !["L03", "L05", "L07", "L09", "L13", "L14", "L18"].contains(&f.rule) {
                wrong.push(format!("{id}: unexpected {} on {:?}", f.rule, f.object));
            }
        }
    }
    let six = ["L03", "L05", "L07", "L09", "L13", "L14"]
        .iter()
        .all(|r| seen.contains(*r));
    let ok = wrong.is_empty() && six;
    println!(
        "CHECK crate=lint name=lint_page result={} rules_shown={:?} findings_on_two_devices={total} wrong={wrong:?}",
        if ok { "pass" } else { "fail" },
        seen
    );
    assert!(ok);
}
