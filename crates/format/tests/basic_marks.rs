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
//! The Show in Basic marks (format 0.21.0, ADR-026; rung 16b, Q5): an object's `show_in_basic` and the page's `basic`
//! (margins, rhythm, title). A marked file saves and opens with the marks; an unmarked file writes no mark at all, so
//! every file saved before 0.21 reads, and saves, as it did; a file with marks opens the same whatever the app is
//! started with (the reader always understands them). Each test prints one structured `CHECK` line (run with
//! `-- --nocapture`).

use ciborium::Value;
use quadrule_format::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::builtin;
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

/// Whether any map in `v` has a key called `name`.
fn has_key(v: &Value, name: &str) -> bool {
    match v {
        Value::Map(m) => m
            .iter()
            .any(|(k, x)| k.as_text() == Some(name) || has_key(x, name)),
        Value::Array(a) => a.iter().any(|x| has_key(x, name)),
        _ => false,
    }
}

/// How many maps in `v` have a key called `name`.
fn count_key(v: &Value, name: &str) -> usize {
    match v {
        Value::Map(m) => m
            .iter()
            .map(|(k, x)| usize::from(k.as_text() == Some(name)) + count_key(x, name))
            .sum(),
        Value::Array(a) => a.iter().map(|x| count_key(x, name)).sum(),
        _ => 0,
    }
}

fn scene(doc: &Document) -> Value {
    Value::serialized(doc).unwrap()
}

/// The slice page with its first two objects and the page's margins and title marked.
fn marked_doc() -> (Document, Vec<Uuid>) {
    let mut doc = slice_document();
    let ids: Vec<Uuid> = doc.objects().take(2).map(|o| o.id).collect();
    for id in &ids {
        doc.object_mut(*id).unwrap().show_in_basic = true;
    }
    doc.page.basic = BasicPage {
        margins: true,
        rhythm: false,
        title: true,
    };
    (doc, ids)
}

/// The page's real text outlines, one per profile.
fn outlines(doc: &Document) -> Vec<ProfileOutlines> {
    let primary = builtin(&doc.primary_profile_id).unwrap();
    let mut reg = FontRegistry::bundled_only();
    doc.profile_ids()
        .iter()
        .map(|id| {
            quadrule_layout::lay_out_text(doc, &primary, &builtin(id).unwrap(), &mut reg)
                .unwrap()
                .outlines
        })
        .collect()
}

fn file_of(doc: Document) -> QuadFile {
    let o = outlines(&doc);
    QuadFile::new(doc, o, vec![])
}

#[test]
fn marks_round_trip_and_unmarked_files_write_none() {
    let (doc, ids) = marked_doc();
    let bytes = file_of(doc.clone()).to_bytes("basic-test").unwrap();
    let back = QuadFile::from_bytes(&bytes).unwrap();
    let kept = back.doc == doc
        && back.doc.page.basic
            == BasicPage {
                margins: true,
                rhythm: false,
                title: true,
            }
        && ids
            .iter()
            .all(|id| back.doc.object(*id).is_some_and(|o| o.show_in_basic))
        && back.doc.objects().filter(|o| o.show_in_basic).count() == 2;
    let again = back.to_bytes("basic-test").unwrap() == bytes;
    // Written where marked and only there: two objects, and the page's two marks.
    let s = scene(&back.doc);
    let objects_written = count_key(&s, "show_in_basic");
    let page_written = count_key(&s, "basic");
    let page_keys_ok = {
        let Value::Map(m) = &s else { panic!() };
        let page = m
            .iter()
            .find(|(k, _)| k.as_text() == Some("page"))
            .map(|(_, v)| v)
            .unwrap();
        let Value::Map(p) = page else { panic!() };
        let basic = p
            .iter()
            .find(|(k, _)| k.as_text() == Some("basic"))
            .map(|(_, v)| v);
        matches!(basic, Some(Value::Map(b)) if b.len() == 2
            && b.iter().any(|(k, _)| k.as_text() == Some("margins"))
            && b.iter().any(|(k, _)| k.as_text() == Some("title"))
            && !b.iter().any(|(k, _)| k.as_text() == Some("rhythm")))
    };
    // An unmarked page writes no mark of either kind.
    let plain = file_of(slice_document()).to_bytes("basic-test").unwrap();
    let plain_back = QuadFile::from_bytes(&plain).unwrap();
    let ps = scene(&plain_back.doc);
    let none_written = !has_key(&ps, "show_in_basic") && !has_key(&ps, "basic");
    let ok =
        kept && again && objects_written == 2 && page_written == 1 && page_keys_ok && none_written;
    println!(
        "CHECK crate=format name=basic_marks_round_trip result={} version={FORMAT_VERSION} kept={kept} resave_identical={again} object_marks_written={objects_written} page_mark_maps_written={page_written} page_marks_only_where_on={page_keys_ok} unmarked_writes_none={none_written}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn taking_a_mark_off_writes_nothing_for_it() {
    // A mark taken off is left out of the file again, so a page with none left reads as an old one.
    let (doc, ids) = marked_doc();
    let mut doc2 = doc.clone();
    doc2.object_mut(ids[0]).unwrap().show_in_basic = false;
    doc2.page.basic = BasicPage::default();
    let back =
        QuadFile::from_bytes(&file_of(doc2.clone()).to_bytes("basic-test").unwrap()).unwrap();
    let unmarked = back.doc.objects().filter(|o| o.show_in_basic).count() == 1
        && !back.doc.page.basic.any()
        && back.doc == doc2;
    let s = scene(&back.doc);
    let one_object_mark = count_key(&s, "show_in_basic") == 1 && !has_key(&s, "basic");
    let ok = unmarked && one_object_mark;
    println!(
        "CHECK crate=format name=basic_marks_off_again result={} marked_after_unmarking_one={} basic_key_gone={}",
        pass(ok),
        back.doc.objects().filter(|o| o.show_in_basic).count(),
        !has_key(&s, "basic")
    );
    assert!(ok);
}
