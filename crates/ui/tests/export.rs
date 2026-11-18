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

//! Export's engine (rung 15 bbq, leg 1): the path template, the settings file, the text note and Export again, all
//! headless. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_model::edit::Command;
use quadrule_model::sample::*;
use quadrule_ui::export_path::{self, *};
use quadrule_ui::settings::{MAX_FOLDERS, Settings};
use quadrule_ui::{DevicesChoice, ExportError, ExportWhich, Request, Source, Viewer, ViewerUi};
use std::path::{Path, PathBuf};

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn tmp(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn doc() -> DocTokens {
    DocTokens {
        title: "Isometric 5mm".into(),
        orientation: "portrait".into(),
        kind: "template".into(),
        date: "2026-11-17".into(),
        version: "0-15-0".into(),
        set: "Planner 2027".into(),
    }
}

fn manta() -> DeviceTokens {
    DeviceTokens {
        id: MANTA_ID.into(),
        device: "manta".into(),
        model: "Supernote Manta (A5 X2)".into(),
        width: 1920,
        height: 2560,
    }
}

fn nomad() -> DeviceTokens {
    DeviceTokens {
        id: NOMAD_ID.into(),
        device: "nomad".into(),
        model: "Supernote Nomad".into(),
        width: 1404,
        height: 1872,
    }
}

fn both(t: &str, d: &DocTokens) -> Result<Vec<String>, PathError> {
    plan(t, d, &[manta(), nomad()])
}

#[test]
fn the_default_template_and_the_three_presets() {
    let d = doc();
    let flat = both(DEFAULT_TEMPLATE, &d).unwrap();
    let per = both(PRESET_FOLDER_PER_DEVICE, &d).unwrap();
    let then_set = both(PRESET_DEVICE_THEN_SET, &d).unwrap();
    let mut no_set = d.clone();
    no_set.set = String::new();
    let dropped = both(PRESET_DEVICE_THEN_SET, &no_set).unwrap();
    let ok = DEFAULT_TEMPLATE == "{title}-{device}.png"
        && PRESET_FLAT == DEFAULT_TEMPLATE
        && flat == ["isometric-5mm-manta.png", "isometric-5mm-nomad.png"]
        && per
            == [
                "manta/isometric-5mm-manta.png",
                "nomad/isometric-5mm-nomad.png",
            ]
        && then_set
            == [
                "manta/planner-2027/isometric-5mm-manta.png",
                "nomad/planner-2027/isometric-5mm-nomad.png",
            ]
        && dropped
            == [
                "manta/isometric-5mm-manta.png",
                "nomad/isometric-5mm-nomad.png",
            ]
        && PRESETS.map(|(_, t)| t)
            == [
                PRESET_FLAT,
                PRESET_FOLDER_PER_DEVICE,
                PRESET_DEVICE_THEN_SET,
            ];
    println!(
        "CHECK crate=ui name=export_path_default_and_presets result={} default={DEFAULT_TEMPLATE} flat={} folder_per_device={} device_then_set={} empty_set_drops_folder={}",
        pass(ok),
        flat[0],
        per[0],
        then_set[0],
        dropped[0]
    );
    assert!(ok);
}

#[test]
fn every_token_expands_and_is_made_safe() {
    let d = doc();
    let all = "{title}|{device}|{profile}|{model}|{width}|{height}|{orientation}|{type}|{date}|{version}|{set}";
    let got = expand(all, &d, &manta()).unwrap();
    // `|` is not allowed in the template's own text, so it became a hyphen.
    let want = "isometric-5mm-manta-manta-supernote-manta-a5-x2-1920-2560-portrait-template-2026-11-17-0-15-0-planner-2027.png";
    // A title with accents, quotes, a slash and emoji is slugged; the slash cannot make a folder.
    let mut odd = d.clone();
    odd.title = "Caf\u{e9} \"A/B\" \u{1f4d3}: 5\\mm".into();
    let slugged = expand("{title}.png", &odd, &manta()).unwrap();
    let profile_is_device = expand("{profile}", &d, &nomad()) == expand("{device}", &d, &nomad());
    let ok =
        got == want && slugged == "caf-a-b-5-mm.png" && profile_is_device && TOKENS.len() == 11;
    println!(
        "CHECK crate=ui name=export_path_tokens result={} all_eleven={} odd_title={slugged} profile_equals_device={profile_is_device}",
        pass(ok),
        got == want
    );
    assert!(ok, "{got}");
}

#[test]
fn a_path_never_leaves_the_chosen_folder() {
    let d = doc();
    let cases = [
        ("../../etc/{title}", "etc/isometric-5mm.png"),
        ("/abs/{title}", "abs/isometric-5mm.png"),
        ("C:\\Users\\me\\{title}", "C-Users-me-isometric-5mm.png"),
        ("..\\..\\{title}", "isometric-5mm.png"),
        ("./{title}", "isometric-5mm.png"),
        ("a//b///{title}", "a/b/isometric-5mm.png"),
        ("{set}/../{title}", "planner-2027/isometric-5mm.png"),
        ("{title}.png", "isometric-5mm.png"),
        ("{title}.PNG", "isometric-5mm.png"),
        ("{title}.v2", "isometric-5mm-v2.png"),
        ("con/{title}", "con_/isometric-5mm.png"),
    ];
    let mut bad = Vec::new();
    for (t, want) in cases {
        let got = expand(t, &d, &manta()).unwrap();
        let escapes = got.starts_with('/')
            || got
                .split('/')
                .any(|p| p == ".." || p == "." || p.is_empty())
            || got.contains(':')
            || got.contains('\\');
        if got != want || escapes {
            bad.push(format!("{t} -> {got} (wanted {want})"));
        }
    }
    // Reserved Windows names and a very long title.
    let mut long = d.clone();
    long.title = "x".repeat(400);
    let capped = expand("{title}", &long, &manta()).unwrap();
    let ok = bad.is_empty() && capped.len() <= 124 && capped.ends_with(".png");
    println!(
        "CHECK crate=ui name=export_path_stays_inside result={} cases={} bad={bad:?} long_title_len={}",
        pass(ok),
        cases.len(),
        capped.len()
    );
    assert!(ok, "{bad:?}");
}

#[test]
fn mistakes_are_data() {
    let d = doc();
    let unknown = both("{title}-{colour}", &d);
    let unclosed = both("{title}-{device", &d);
    let no_file = both("{device}/", &d);
    let mut empty_set = d.clone();
    empty_set.set = String::new();
    let only_set = both("{set}", &empty_set);
    let same = both("{title}.png", &d);
    let same_case = plan(
        "{model}",
        &d,
        &[
            DeviceTokens {
                model: "Same".into(),
                ..manta()
            },
            DeviceTokens {
                model: "SAME".into(),
                ..nomad()
            },
        ],
    );
    let one_device = plan("{title}.png", &d, &[manta()]);
    let ok = unknown == Err(PathError::UnknownToken("colour".into()))
        && unclosed == Err(PathError::Unclosed)
        && no_file == Err(PathError::NoFileName)
        && only_set == Err(PathError::NoFileName)
        && matches!(&same, Err(PathError::SamePath { first, second, path })
            if first == MANTA_ID && second == NOMAD_ID && path == "isometric-5mm.png")
        && matches!(same_case, Err(PathError::SamePath { .. }))
        && one_device == Ok(vec!["isometric-5mm.png".to_string()])
        && check("{title}-{device}.png").is_ok()
        && check("{nope}") == Err(PathError::UnknownToken("nope".into()));
    println!(
        "CHECK crate=ui name=export_path_errors result={} unknown_token={} unclosed={} no_file_name={} same_path={}",
        pass(ok),
        unknown.is_err(),
        unclosed.is_err(),
        no_file.is_err() && only_set.is_err(),
        same.is_err() && same_case.is_err()
    );
    assert!(ok);
}

#[test]
fn settings_round_trip_default_and_damage() {
    let dir = tmp("export-settings");
    let path = dir.join("config").join("settings.ron");
    // No file: the defaults, no fuss.
    let none = Settings::load(&path);
    let defaults = none.damaged.is_none()
        && none.settings == Settings::default()
        && Settings::default().template == "{title}-{device}.png"
        && Settings::default().note
        && Settings::default().devices == DevicesChoice::All
        && !Settings::default().completion_note_open;
    // Saved (the folder is made), read back.
    let mut s = Settings {
        template: PRESET_DEVICE_THEN_SET.into(),
        set: "planner-2027".into(),
        note: false,
        devices: DevicesChoice::This,
        completion_note_open: true,
        ..Settings::default()
    };
    s.remember_folder(Some(Path::new("/docs/a.quad")), Path::new("/out/a"));
    s.save(&path).unwrap();
    let back = Settings::load(&path);
    let round = back.damaged.is_none() && back.settings == s;
    // A damaged file: the defaults and a reason; the file stays as it was.
    std::fs::write(&path, "(template: \"x\", note: maybe").unwrap();
    let bad = Settings::load(&path);
    let damaged = bad.settings == Settings::default()
        && bad
            .damaged
            .as_deref()
            .is_some_and(|m| m.contains("settings.ron"))
        && std::fs::read_to_string(&path).unwrap().contains("maybe");
    // A file missing fields, or with fields this build does not know, still loads.
    std::fs::write(&path, "(note: false, from_the_future: 3)").unwrap();
    let partial = Settings::load(&path);
    let tolerant = partial.damaged.is_none()
        && !partial.settings.note
        && partial.settings.template == DEFAULT_TEMPLATE;
    // The viewer turns damage into a notice and runs on the defaults.
    std::fs::write(&path, "((((").unwrap();
    let mut v = Viewer::default();
    v.use_settings_file(path.clone());
    let notice = v
        .notice
        .as_ref()
        .is_some_and(|n| n.error && n.message.id == "notice-settings-damaged")
        && v.settings == Settings::default();
    let ok = defaults && round && damaged && tolerant && notice;
    println!(
        "CHECK crate=ui name=export_settings_file result={} defaults={defaults} round_trip={round} damaged_gives_defaults={damaged} tolerant={tolerant} damaged_notice={notice}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn folders_are_remembered_by_document_path_and_capped() {
    let dir = tmp("export-folders");
    let own = dir.join("own");
    let last = dir.join("last");
    let mine = dir.join("mine");
    let home = dir.join("home");
    for d in [&own, &last, &mine, &home] {
        std::fs::create_dir_all(d).unwrap();
    }
    let doc = own.join("page.quad");
    let other = dir.join("elsewhere").join("other.quad");
    let mut s = Settings::default();
    // Nothing remembered: the document's own folder if saved, else home.
    let first_saved = s.start_folder(Some(&doc), Some(&home)) == Some(own.clone());
    let first_unsaved = s.start_folder(None, Some(&home)) == Some(home.clone());
    // Another document exported: its folder is the global last folder.
    s.remember_folder(Some(&other), &last);
    let global = s.start_folder(Some(&doc), Some(&home)) == Some(last.clone())
        && s.start_folder(None, Some(&home)) == Some(last.clone());
    // This document's own entry wins over the global one.
    s.remember_folder(Some(&doc), &mine);
    s.remember_folder(Some(&other), &last);
    let by_path = s.start_folder(Some(&doc), Some(&home)) == Some(mine.clone())
        && s.folder_of(&doc) == Some(mine.clone());
    // A remembered folder that is gone is passed over.
    let mut gone = Settings::default();
    gone.remember_folder(Some(&doc), &dir.join("deleted"));
    let passed_over = gone.start_folder(Some(&doc), Some(&home)) == Some(own.clone());
    // The 200 most recent documents are kept; the oldest go.
    let mut many = Settings::default();
    for i in 0..(MAX_FOLDERS + 30) {
        many.remember_folder(
            Some(Path::new(&format!("/docs/{i}.quad"))),
            Path::new("/out"),
        );
    }
    many.remember_folder(Some(Path::new("/docs/100.quad")), Path::new("/out2"));
    let capped = many.folders.len() == MAX_FOLDERS
        && many.folder_of(Path::new("/docs/0.quad")).is_none()
        && many.folder_of(Path::new("/docs/229.quad")).is_some()
        && many.folders.last().map(|e| e.document.as_str()) == Some("/docs/100.quad")
        && many.folder_of(Path::new("/docs/100.quad")) == Some(PathBuf::from("/out2"));
    // A file that has no path is not keyed, but is the global last folder.
    let mut unsaved = Settings::default();
    unsaved.remember_folder(None, &last);
    let keyless = unsaved.folders.is_empty() && unsaved.last_folder.is_some();
    let ok = first_saved && first_unsaved && global && by_path && passed_over && capped && keyless;
    println!(
        "CHECK crate=ui name=export_folder_memory result={} own_folder_first={first_saved} home_if_unsaved={first_unsaved} global_last={global} by_document_path={by_path} missing_passed_over={passed_over} capped_at_200={capped} unsaved_not_keyed={keyless}",
        pass(ok)
    );
    assert!(ok);
}

/// A PNG's text chunks: (tEXt as key=value, iTXt as key=value).
fn notes(path: &Path) -> (Vec<String>, Vec<String>) {
    let dec = png::Decoder::new(std::io::Cursor::new(std::fs::read(path).unwrap()));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    r.next_frame(&mut buf).unwrap();
    // The text chunks come after the image data too, so read to the end.
    r.finish().ok();
    let info = r.info();
    (
        info.uncompressed_latin1_text
            .iter()
            .map(|t| format!("{}={}", t.keyword, t.text))
            .collect(),
        info.utf8_text
            .iter()
            .map(|t| format!("{}={}", t.keyword, t.get_text().unwrap()))
            .collect(),
    )
}

#[test]
fn an_export_writes_folders_notes_and_thumbnails_and_counts_replacements() {
    let dir = tmp("export-engine");
    let mut v = Viewer::default();
    v.open_sample();
    v.export_date = Some("2026-11-17".into());
    v.settings.template = PRESET_DEVICE_THEN_SET.into();
    v.settings.set = "Planner 2027".into();
    let req = v.export_request(Some(ExportWhich::AllDevices), dir.clone());
    let plan = v.plan_export(&req).unwrap();
    let first = v.run_export(&req).unwrap();
    let want = [
        dir.join("manta/planner-2027/rung-1-slice-manta.png"),
        dir.join("nomad/planner-2027/rung-1-slice-nomad.png"),
    ];
    let paths_ok = first.paths() == want
        && want.iter().all(|p| p.is_file())
        && plan[0].1 == "manta/planner-2027/rung-1-slice-manta.png"
        && first.replaced == 0
        && first.shared_folder == dir;
    // The note: Software is the app and its version, Title the document's title.
    let (text, utf8) = notes(&want[0]);
    let note_ok =
        text == [
            format!("Software=quadrule {}", env!("CARGO_PKG_VERSION")),
            "Title=Rung 1 slice".to_string(),
        ] && utf8.is_empty();
    // Thumbnails are shrunk from the pixels just written, not read back, and never larger than 240.
    let thumbs = first.files.iter().all(|f| {
        f.thumbnail.width.max(f.thumbnail.height) == quadrule_ui::export::THUMB_SIDE
            && f.size.0 > f.thumbnail.width
    });
    // Again into the same place: all replaced. With the note off, the file is the bare export.
    v.settings.note = false;
    let req2 = v.export_request(Some(ExportWhich::AllDevices), dir.clone());
    let second = v.run_export(&req2).unwrap();
    let bare = std::fs::read(&want[1]).unwrap();
    let (t2, u2) = notes(&want[1]);
    let page = v.page_for(NOMAD_ID).unwrap();
    let bytes_ok = bare
        == quadrule_render::encode_png(&quadrule_ui::viewer::export_pixels(page), None)
        && t2.is_empty()
        && u2.is_empty();
    let replaced = second.replaced == 2 && second.files.iter().all(|f| f.replaced);
    // The notice, the session folder and the settings.
    let notice = v
        .notice
        .as_ref()
        .is_some_and(|n| !n.error && n.message.id == "notice-exported");
    let remembered = v.export_dir.as_deref() == Some(dir.as_path())
        && v.settings.last_folder.as_deref() == dir.to_str();
    // A title that is not Latin-1 goes in an iTXt chunk; a Latin-1 one in tEXt.
    let mut t = Viewer::default();
    t.open_sample();
    let mut m = t.doc().unwrap().metadata.clone();
    m.title = "Caf\u{e9} \u{65e5}\u{672c}".into();
    t.edit(Command::SetMetadata(m), None).unwrap();
    t.settings.template = "{title}.png".into();
    let treq = t.export_request(Some(ExportWhich::ThisDevice), dir.join("t"));
    let tr = t.run_export(&treq).unwrap();
    let (tt, tu) = notes(&tr.files[0].path);
    let itxt = tt.len() == 1 && tu == ["Title=Caf\u{e9} \u{65e5}\u{672c}"];
    let ok = paths_ok && note_ok && thumbs && bytes_ok && replaced && notice && remembered && itxt;
    println!(
        "CHECK crate=ui name=export_engine result={} paths={paths_ok} text_note={note_ok} thumbnails_240={thumbs} no_note_is_bare_export={bytes_ok} second_run_replaced={} notice_and_memory={} non_latin1_title_itxt={itxt}",
        pass(ok),
        second.replaced,
        notice && remembered
    );
    assert!(ok);
}

#[test]
fn a_mistake_in_the_template_writes_nothing() {
    let dir = tmp("export-mistake");
    let mut v = Viewer::default();
    v.open_sample();
    v.settings.template = "{title}.png".into();
    let req = v.export_request(Some(ExportWhich::AllDevices), dir.clone());
    let same = v.run_export(&req);
    let same_ok = matches!(same, Err(ExportError::Path(PathError::SamePath { .. })))
        && std::fs::read_dir(&dir).unwrap().count() == 0;
    let notice = v
        .notice
        .as_ref()
        .is_some_and(|n| n.error && n.message.id == "export-error-same-path");
    v.settings.template = "{title}-{nope}".into();
    let req = v.export_request(Some(ExportWhich::AllDevices), dir.clone());
    let unknown = matches!(
        v.run_export(&req),
        Err(ExportError::Path(PathError::UnknownToken(_)))
    ) && std::fs::read_dir(&dir).unwrap().count() == 0;
    // The toolbar's route says it in words.
    let words = v
        .export(&dir, ExportWhich::ThisDevice)
        .err()
        .is_some_and(|e| e.contains("nope"));
    let ok = same_ok && notice && unknown && words;
    println!(
        "CHECK crate=ui name=export_mistake_writes_nothing result={} same_path={same_ok} unknown_token={unknown} notice_id={notice} words={words}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_folder_an_export_went_to_is_remembered_by_document_and_starts_the_sheet() {
    // Rung 16 bbq, Q9: there is no Export again; the sheet opens with the last choices filled in, and the folder this
    // document last went to is what it starts in (rung 15, Q4).
    let dir = tmp("export-again");
    let out = dir.join("out");
    let cfg = dir.join("cfg").join("settings.ron");
    let file = dir.join("doc").join("page.quad");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut v = Viewer::default();
    v.open_sample();
    v.use_settings_file(cfg.clone());
    v.save_to(&file).unwrap();
    v.settings.devices = DevicesChoice::This;
    // Nothing exported from this document yet: no folder is remembered for it.
    let none = v.last_export_folder().is_none();
    // The first export goes by a request, as the sheet makes it.
    v.set_profile(NOMAD_ID);
    let req = v.export_request(None, out.clone());
    v.run_export(&req).unwrap();
    let one = out.join("rung-1-slice-nomad.png");
    let wrote_one = one.is_file() && !out.join("rung-1-slice-manta.png").exists();
    let can = v.last_export_folder() == Some(out.clone());
    // The memory is in settings.ron by the document's path, so a new run of the app has it too.
    let mut w = Viewer::default();
    w.use_settings_file(cfg.clone());
    let bytes = std::fs::read(&file).unwrap();
    w.open_bytes(Source::File(file.clone()), &bytes).unwrap();
    let remembered = w.last_export_folder() == Some(out.clone());
    // The template and note changed in settings are what the next request carries.
    w.set_profile(NOMAD_ID);
    w.change_settings(|s| s.note = false);
    w.change_settings(|s| s.devices = DevicesChoice::All);
    let again = w.export_request(None, out.clone());
    let all = w.run_export(&again).is_ok_and(|r| r.files.len() == 2);
    let text_gone = notes(&one).0.is_empty();
    // A different saved document has no memory of this one's folder, but starts there.
    let other = dir.join("doc").join("other.quad");
    w.save_to(&other).unwrap();
    // (Saved under a new name, the same window keeps what it exported this session.)
    let kept_session = w.last_export_folder().is_some();
    let mut x = Viewer::default();
    x.use_settings_file(cfg.clone());
    x.open_bytes(Source::File(other.clone()), &std::fs::read(&other).unwrap())
        .unwrap();
    let separate = x.last_export_folder().is_none() && kept_session;
    let starts = x.export_start_folder() == Some(out.clone());
    let ok = none && wrote_one && can && remembered && all && text_gone && separate && starts;
    println!(
        "CHECK crate=ui name=export_folder_memory result={} none_at_first={none} first_export={wrote_one} remembered={can} remembered_across_runs={remembered} changes_used={all} bare_after_note_off={text_gone} other_document_separate={separate} starts_in_last_folder={starts}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn the_export_sheet_goes_on_behind_the_one_preflight_sheet() {
    // Rung 16 bbq, Q9: there is one Export… (the sheet, with the last choices filled in); Export again is gone. The
    // sheet's Export goes on behind the one preflight sheet when the rules call for it, never both at once.
    let dir = tmp("export-sheet-pre");
    let mut v = Viewer::default();
    v.open_sample();
    v.settings.last_folder = Some(dir.to_string_lossy().into_owned());
    let mut ui = ViewerUi::new(v);
    ui.open_export_sheet();
    let sheet = ui.export_sheet_open() && !ui.preflight_open() && !ui.completion_open();
    // Opening it again does not stack a second sheet.
    ui.open_export_sheet();
    let one = ui.export_sheet_open() && !ui.preflight_open();
    // Export on the sample page, which has warnings: the sheet closes and the preflight sheet holds the export.
    ui.show_look("preflight");
    let held = ui.preflight_open() && !ui.export_sheet_open() && !ui.completion_open();
    ui.answer_preflight(true);
    let wrote = !ui.preflight_open()
        && ui.completion_open()
        && dir.join("rung-1-slice-manta.png").is_file();
    // Cancel drops it and writes nothing more.
    std::fs::remove_file(dir.join("rung-1-slice-manta.png")).unwrap();
    ui.show_look("preflight");
    ui.answer_preflight(false);
    let cancelled = !ui.preflight_open() && !dir.join("rung-1-slice-manta.png").exists();
    let ok = sheet && one && held && wrote && cancelled;
    println!(
        "CHECK crate=ui name=export_sheet_preflight result={} sheet_opens={sheet} not_stacked={one} held_behind_preflight={held} export_anyway_writes_and_completes={wrote} cancel_drops={cancelled}",
        pass(ok)
    );
    assert!(ok);
    let _ = (export_path::DEFAULT_TEMPLATE, Request::ExportFolderDialog);
}
