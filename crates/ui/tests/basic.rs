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
//! The Basic section (rung 16b, Q5; ADR-026): the **Show in Basic** switches and the **Basic** tab, first in the
//! inspector. Both always show: the Arrange tab and the Page tab have the switches, the Basic tab lists what
//! is marked (whether or not anything is selected) with the other tabs' own widgets, says so in a sentence when
//! nothing is marked, and an edit made there is the same edit, in one undo step, with the selection left as it was.
//! Headless. Each test prints one structured `CHECK` line starting with a newline (run with `-- --nocapture`).

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_ui::{Tab, Viewer, ViewerUi};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

struct Window {
    ui: ViewerUi,
}

const SHOW: &str = "Show in Basic";

/// The sample page, with the Cornell pattern on its ruled lines; `marked` objects (and, when `page`, the page's three
/// settings) are marked; `tab` is showing.
fn page(marked: &[Uuid], page: bool, tab: Tab) -> Harness<'static, Window> {
    let mut v = Viewer::default();
    v.open_sample();
    v.selection = vec![ids::LINES];
    v.edit_selected(None, |x| x.set_pattern_kind(PatternKind::Cornell))
        .unwrap();
    for id in marked {
        v.selection = vec![*id];
        v.set_show_in_basic(true).unwrap();
    }
    if page {
        for which in [
            quadrule_ui::basic::PageMark::Title,
            quadrule_ui::basic::PageMark::Margins,
            quadrule_ui::basic::PageMark::Rhythm,
        ] {
            v.set_page_basic(which, true).unwrap();
        }
    }
    v.selection.clear();
    let ui = ViewerUi::new(v);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1280.0, 1400.0))
        .with_max_steps(16)
        .build_ui_state(
            |ui, w: &mut Window| {
                let _ = w.ui.show(ui);
            },
            Window { ui },
        );
    h.state_mut().ui.tab = tab;
    h.run();
    h.run();
    h
}

fn select(h: &mut Harness<'static, Window>, id: Uuid) {
    h.state_mut().ui.viewer.selection = vec![id];
    h.run();
    h.run();
}

fn object(h: &Harness<'static, Window>, id: Uuid) -> Object {
    h.state()
        .ui
        .viewer
        .doc()
        .unwrap()
        .object(id)
        .unwrap()
        .clone()
}

fn steps(h: &Harness<'static, Window>) -> usize {
    h.state()
        .ui
        .viewer
        .open
        .as_ref()
        .unwrap()
        .history
        .undo_len()
}

fn count(h: &Harness<'static, Window>, role: Role, label: &str) -> usize {
    h.query_all_by_role_and_label(role, label).count()
}

fn has(h: &Harness<'static, Window>, label: &str) -> bool {
    h.query_all_by_label(label).count() > 0
}

#[test]
fn the_basic_tab_is_first_and_always_there() {
    // No flag: with nothing selected the tabs are Basic, Page; with one object selected Basic is still first, ahead
    // of Arrange; the Page tab has three switches and the Arrange tab one; and the marks in the file are read.
    let mut h = page(&[ids::RULE, ids::LINES], true, Tab::Page);
    let first_of = |h: &Harness<'static, Window>, next: &str| {
        let basic = h.get_by_role_and_label(Role::Button, "Basic").rect();
        let other = h.get_by_role_and_label(Role::Button, next).rect();
        basic.top() == other.top() && basic.left() < other.left()
    };
    let page_first = first_of(&h, "Page");
    let page_switches = count(&h, Role::CheckBox, SHOW);
    select(&mut h, ids::RULE);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    let arrange_first = first_of(&h, "Arrange");
    let arrange_switches = count(&h, Role::CheckBox, SHOW);
    let doc = h.state().ui.viewer.doc().unwrap();
    let read = doc.page.basic.any()
        && object(&h, ids::RULE).show_in_basic
        && object(&h, ids::LINES).show_in_basic;
    let ok = page_first && page_switches == 3 && arrange_first && arrange_switches == 1 && read;
    println!(
        "\nCHECK crate=ui name=basic_always result={} basic_before_page={page_first} page_tab_switches={page_switches} basic_before_arrange={arrange_first} arrange_switches={arrange_switches} marks_read={read}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn empty_basic_tab_says_how_to_mark_something() {
    let mut h = page(&[], false, Tab::Basic);
    let words = h.state().ui.strings.get("app-basic-empty");
    let nothing_selected = has(&h, "Basic") && has(&h, &words);
    // The same with an object selected: Basic is still the first tab and still about the document.
    select(&mut h, ids::RULE);
    let with_selection = has(&h, &words) && has(&h, "Arrange") && has(&h, "Style");
    let ok = nothing_selected && with_selection && !words.is_empty();
    println!(
        "\nCHECK crate=ui name=basic_empty result={} says_it_with_nothing_selected={nothing_selected} says_it_with_one_selected={with_selection}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_switches_mark_and_unmark_in_one_undo_step_each() {
    let mut h = page(&[], false, Tab::Arrange);
    select(&mut h, ids::RULE);
    h.state_mut().ui.tab = Tab::Arrange;
    h.run();
    h.run();
    let n0 = steps(&h);
    h.get_by_role_and_label(Role::CheckBox, SHOW).click();
    h.run();
    h.run();
    let marked = object(&h, ids::RULE).show_in_basic && steps(&h) == n0 + 1;
    h.state_mut().ui.viewer.undo();
    h.run();
    let undone = !object(&h, ids::RULE).show_in_basic;
    // The Page tab: three switches, in the order Title, Margins, Rhythm; each is one step.
    h.state_mut().ui.viewer.selection.clear();
    h.state_mut().ui.tab = Tab::Page;
    h.run();
    h.run();
    let three = count(&h, Role::CheckBox, SHOW) == 3;
    let n1 = steps(&h);
    let mut seen = Vec::new();
    for i in 0..3 {
        h.query_all_by_role_and_label(Role::CheckBox, SHOW)
            .nth(i)
            .unwrap()
            .click();
        h.run();
        h.run();
        let b = h.state().ui.viewer.doc().unwrap().page.basic;
        seen.push((b.title, b.margins, b.rhythm));
    }
    let page_ok = three
        && seen
            == [
                (true, false, false),
                (true, true, false),
                (true, true, true),
            ]
        && steps(&h) == n1 + 3;
    let ok = marked && undone && page_ok;
    println!(
        "\nCHECK crate=ui name=basic_switches result={} arrange_marks_in_one_step={marked} undo_unmarks={undone} page_switches={three} page_in_order_one_step_each={page_ok}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_basic_tab_lists_what_is_marked_with_the_other_tabs_widgets() {
    let h = page(&[ids::TITLE, ids::RULE, ids::LINES], true, Tab::Basic);
    // The page: its heading and its three, each with its widgets (the Page tab's).
    let page_part = has(&h, "Page")
        && has(&h, "Margins")
        && has(&h, "Rhythm")
        && has(&h, "Same on all sides")
        && has(&h, "Left")
        && has(&h, "Across")
        && has(&h, "Down")
        && h.query_all_by_label("Title").count() >= 2;
    // The objects, by name, each with its kind's widgets: the text box's text; the line's kind and role; the pattern's
    // cue and summary sizes, its spacing and its major lines.
    // (The layers panel lists every object once; a marked one is listed again, by the Basic tab.)
    let listed = |n: &str| h.query_all_by_label(n).count();
    let names = listed("Title rule") == 2 && listed("Ruled lines") == 2;
    let text = count(&h, Role::MultilineTextInput, "Text") == 1;
    let line = has(&h, "Look") && has(&h, "Role");
    let cornell = has(&h, "Cue column")
        && has(&h, "Summary band")
        && has(&h, "Bound to rhythm")
        && has(&h, "Major lines");
    // What is not marked is not listed, and nothing is selected.
    let unmarked_absent = listed("Dot grid") == 1;
    let none_selected = h.state().ui.viewer.selection.is_empty();
    // One switch per marked object and three for the page.
    let ticks = count(&h, Role::CheckBox, SHOW);
    let ok = page_part
        && names
        && text
        && line
        && cornell
        && unmarked_absent
        && none_selected
        && ticks == 6;
    println!(
        "\nCHECK crate=ui name=basic_lists result={} page={page_part} names={names} text_box={text} line={line} cornell={cornell} unmarked_absent={unmarked_absent} nothing_selected={none_selected} switches={ticks}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn an_edit_in_the_basic_tab_is_the_same_edit_and_leaves_the_selection() {
    let mut h = page(&[ids::LINES], false, Tab::Basic);
    let was_bound = |h: &Harness<'static, Window>| match &object(h, ids::LINES).kind {
        ObjectKind::Pattern(p) => p.bound,
        _ => panic!("not a pattern"),
    };
    let before = was_bound(&h);
    let n0 = steps(&h);
    h.get_by_role_and_label(Role::CheckBox, "Bound to rhythm")
        .click();
    h.run();
    h.run();
    let flipped = was_bound(&h) != before && steps(&h) == n0 + 1;
    let kept = h.state().ui.viewer.selection.is_empty();
    // The same region in the Style tab shows the same value: the row is the same widget on the same data.
    select(&mut h, ids::LINES);
    h.state_mut().ui.tab = Tab::Style;
    h.run();
    h.run();
    let style_same = h
        .get_by_role_and_label(Role::CheckBox, "Bound to rhythm")
        .accesskit_node()
        .toggled()
        == Some(if was_bound(&h) {
            egui::accesskit::Toggled::True
        } else {
            egui::accesskit::Toggled::False
        });
    // Unmarking in the Basic tab takes the item off the list in one step.
    h.state_mut().ui.viewer.selection.clear();
    h.state_mut().ui.tab = Tab::Basic;
    h.run();
    h.run();
    let n1 = steps(&h);
    h.get_by_role_and_label(Role::CheckBox, SHOW).click();
    h.run();
    h.run();
    let unmarked = !object(&h, ids::LINES).show_in_basic && steps(&h) == n1 + 1;
    let empty = has(&h, &h.state().ui.strings.get("app-basic-empty"));
    let ok = flipped && kept && style_same && unmarked && empty;
    println!(
        "\nCHECK crate=ui name=basic_edit result={} edit_is_one_step={flipped} selection_kept={kept} style_tab_agrees={style_same} unmarking_is_one_step={unmarked} list_empty_after={empty}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn select_picks_the_object_on_the_page() {
    let mut h = page(&[ids::RULE], false, Tab::Basic);
    let none = h.state().ui.viewer.selection.is_empty();
    h.get_by_role_and_label(Role::Button, "Select").click();
    h.run();
    h.run();
    let picked = h.state().ui.viewer.selection == vec![ids::RULE];
    // Basic is still the tab showing, and still first.
    let still = h.state().ui.tab == Tab::Basic && has(&h, "Arrange");
    let ok = none && picked && still;
    println!(
        "\nCHECK crate=ui name=basic_select result={} was_empty={none} picked={picked} tab_kept={still}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_basic_look_marks_a_page_and_shows_the_tab() {
    let mut v = Viewer::default();
    v.open_sample();
    let mut ui = ViewerUi::new(v);
    ui.show_look("basic");
    let page_marks = ui.viewer.doc().unwrap().page.basic
        == BasicPage {
            margins: true,
            rhythm: true,
            title: true,
        };
    let marked = ui.viewer.basic_objects().len();
    let tab = ui.tab == Tab::Basic;
    let ok = page_marks && marked == 3 && tab;
    println!(
        "\nCHECK crate=ui name=basic_look result={} page_marks={page_marks} objects_marked={marked} tab_basic={tab}",
        pass(ok)
    );
    assert!(ok);
}
