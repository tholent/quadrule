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

//! The paper PPI in format 0.14 (rung 13b, docs/bbq/2026-11-17-rung-13-profile-system.md, Q12 and Defaulted
//! "Carrying in the file"): each profile of a page may carry the export PPI it uses, an optional field with the
//! default 300 that is left out of a file when it is 300, so a 0.14.0 file saved by a 13a build opens, and saves,
//! unchanged. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use ciborium::Value;
use quadrule_format::*;
use quadrule_model::sample::*;
use quadrule_model::{Document, PAPER_PPIS};
use quadrule_profiles::builtin_at;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn scene(doc: &Document) -> Value {
    let mut bytes = Vec::new();
    ciborium::into_writer(doc, &mut bytes).unwrap();
    ciborium::from_reader(bytes.as_slice()).unwrap()
}

/// A file for `doc`, with the text laid out for each of its profiles.
fn file(doc: &Document) -> QuadFile {
    let o = doc.page.orientation;
    let at = |id: &str| builtin_at(id, o, doc.ppi_of(id)).unwrap();
    let primary = at(&doc.primary_profile_id);
    let mut reg = quadrule_text::FontRegistry::bundled_only();
    let outlines = doc
        .profile_ids()
        .into_iter()
        .map(|id| {
            quadrule_layout::lay_out_text(doc, &primary, &at(id), &mut reg)
                .unwrap()
                .outlines
        })
        .collect();
    QuadFile::new(doc.clone(), outlines, vec![])
}

/// The keys of the `i`th profile entry of a scene.
fn profile_keys(v: &Value, i: usize) -> Vec<String> {
    let Value::Map(m) = v else { return vec![] };
    let Some((_, Value::Array(ps))) = m
        .iter()
        .find(|(k, _)| matches!(k, Value::Text(t) if t == "profiles"))
    else {
        return vec![];
    };
    let Value::Map(p) = &ps[i] else { return vec![] };
    p.iter()
        .filter_map(|(k, _)| match k {
            Value::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_page_at_300_writes_no_ppi_and_one_saved_before_the_field_opens_unchanged() {
    // What a 13a build wrote: profile entries holding only the id.
    let doc = slice_document();
    let v = scene(&doc);
    let only_id = profile_keys(&v, 0) == ["profile_id"] && profile_keys(&v, 1) == ["profile_id"];
    // The paper page at 300 for A4 writes none either; at 600 for A5 it writes `ppi`.
    let paper = paper_document();
    let pv = scene(&paper);
    let a4_plain = profile_keys(&pv, 0) == ["profile_id"];
    let a5_marked = profile_keys(&pv, 1) == ["profile_id", "ppi"];
    // Read back a scene that has no `ppi` anywhere: every profile is 300, and writing it again gives
    // the very same scene.
    let back: Document = ciborium::from_reader(
        {
            let mut b = Vec::new();
            ciborium::into_writer(&v, &mut b).unwrap();
            b
        }
        .as_slice(),
    )
    .unwrap();
    let all_300 = back.profile_ids().iter().all(|id| back.ppi_of(id) == 300)
        && back.profiles.iter().all(|c| c.ppi == 300);
    let same = back == doc && scene(&back) == v;
    // A whole saved file at 300 holds the same scene bytes as one that never had the field.
    let file = file(&doc);
    let saved = QuadFile::from_bytes(&file.to_bytes("paper-ppi").unwrap());
    let opened = saved.as_ref().is_ok_and(|f| {
        f.doc == doc && f.migrated_from.is_none() && f.manifest.format_version == FORMAT_VERSION
    });
    let ok = only_id && a4_plain && a5_marked && all_300 && same && opened;
    println!(
        "CHECK crate=format name=paper_ppi_default_unwritten result={} only_id={only_id} a4_plain={a4_plain} a5_marked={a5_marked} all_300={all_300} same_scene={same} opens={opened}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn each_ppi_survives_a_save_and_a_number_that_is_not_offered_reads_as_300_and_is_kept() {
    let mut kept = true;
    for ppi in PAPER_PPIS {
        let mut doc = paper_document();
        doc.profiles[1].ppi = ppi;
        let file = file(&doc);
        let bytes = file.to_bytes("paper-ppi").unwrap();
        let again = QuadFile::from_bytes(&bytes).unwrap();
        kept &= again.doc.ppi_of("paper.a5") == ppi && again.doc == doc;
    }
    let mut odd = paper_document();
    odd.profiles[1].ppi = 123;
    let again = QuadFile::from_bytes(&file(&odd).to_bytes("paper-ppi").unwrap()).unwrap();
    let reads_300 = again.doc.ppi_of("paper.a5") == 300 && again.doc.profiles[1].ppi == 123;
    let ok = kept && reads_300;
    println!(
        "CHECK crate=format name=paper_ppi_round_trip result={} each_offered_kept={kept} odd_reads_300_and_is_kept={reads_300}",
        pass(ok)
    );
    assert!(ok);
}
