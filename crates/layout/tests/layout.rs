//! Layout checks. Each test prints one structured `CHECK` line (run with `-- --nocapture`).

use quadrule_layout::*;
use quadrule_model::outlines::ProfileOutlines;
use quadrule_model::sample::*;
use quadrule_model::*;
use quadrule_profiles::{Profile, builtin};
use quadrule_text::FontRegistry;

fn pass(ok: bool) -> &'static str {
    if ok { "pass" } else { "fail" }
}

fn profiles() -> (Profile, Profile) {
    (builtin(MANTA_ID).unwrap(), builtin(NOMAD_ID).unwrap())
}

fn page(doc: &Document, target: &Profile) -> DisplayList {
    let (manta, _) = profiles();
    let mut reg = FontRegistry::bundled_only();
    let laid = lay_out_text(doc, &manta, target, &mut reg).unwrap();
    display_list(doc, &manta, target, &laid.outlines).unwrap()
}

fn rects(dl: &DisplayList) -> Vec<(f64, f64, f64, f64, u8)> {
    dl.items
        .iter()
        .filter_map(|i| match *i {
            Item::Rect {
                x0,
                y0,
                x1,
                y1,
                gray,
            } => Some((x0, y0, x1, y1, gray)),
            _ => None,
        })
        .collect()
}

#[test]
fn constraints_resolve() {
    // A 10 mm box at x = 20 on a 100 mm page, moved to a 50 mm page.
    let f = Frame::new(20.0, 20.0, 10.0, 10.0);
    let (from, to) = ((100.0, 100.0), (50.0, 50.0));
    let h = |h| {
        resolve_frame(
            f,
            Constraints {
                h,
                v: VConstraint::Top,
            },
            from,
            to,
        )
    };
    let v = |v| {
        resolve_frame(
            f,
            Constraints {
                h: HConstraint::Left,
                v,
            },
            from,
            to,
        )
    };
    let cases = [
        (
            "left",
            (h(HConstraint::Left).x_mm, h(HConstraint::Left).w_mm),
            (20.0, 10.0),
        ),
        (
            "right",
            (h(HConstraint::Right).x_mm, h(HConstraint::Right).w_mm),
            (-30.0, 10.0),
        ),
        (
            "left_right",
            (
                h(HConstraint::LeftRight).x_mm,
                h(HConstraint::LeftRight).w_mm,
            ),
            (20.0, -40.0),
        ),
        (
            "center",
            (h(HConstraint::Center).x_mm, h(HConstraint::Center).w_mm),
            (-5.0, 10.0),
        ),
        (
            "scale",
            (h(HConstraint::Scale).x_mm, h(HConstraint::Scale).w_mm),
            (10.0, 5.0),
        ),
        (
            "top",
            (v(VConstraint::Top).y_mm, v(VConstraint::Top).h_mm),
            (20.0, 10.0),
        ),
        (
            "bottom",
            (v(VConstraint::Bottom).y_mm, v(VConstraint::Bottom).h_mm),
            (-30.0, 10.0),
        ),
        (
            "top_bottom",
            (
                v(VConstraint::TopBottom).y_mm,
                v(VConstraint::TopBottom).h_mm,
            ),
            (20.0, -40.0),
        ),
        (
            "v_center",
            (v(VConstraint::Center).y_mm, v(VConstraint::Center).h_mm),
            (-5.0, 10.0),
        ),
        (
            "v_scale",
            (v(VConstraint::Scale).y_mm, v(VConstraint::Scale).h_mm),
            (10.0, 5.0),
        ),
    ];
    let fails: Vec<&str> = cases
        .iter()
        .filter(|(_, got, want)| got != want)
        .map(|(n, _, _)| *n)
        .collect();
    println!(
        "CHECK crate=layout name=constraints_resolve result={} cases={} fails={fails:?}",
        pass(fails.is_empty()),
        cases.len()
    );
    assert!(fails.is_empty());
}

#[test]
fn snap_odd_on_centres_even_on_edges() {
    let ok = snap_center(10.3, 1) == 10.5
        && snap_center(10.3, 3) == 10.5
        && snap_center(10.3, 2) == 10.0
        && snap_center(10.6, 4) == 11.0;
    println!(
        "CHECK crate=layout name=snap_center result={} cases=4",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn rhythm_resolves_per_profile() {
    // 6 mm at 300 PPI = 70.87 px, pixel-locked to 71; the 8 mm margin origin = 94.49 → 94.
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let (m, n) = (RhythmPx::new(&doc, &manta), RhythmPx::new(&doc, &nomad));
    let ok =
        m.unit_y == 71.0 && m.unit_x == 71.0 && m.origin_x == 94.0 && m.origin_y == 94.0 && n == m;
    println!(
        "CHECK crate=layout name=rhythm_px result={} unit_px={} origin_px=({},{})",
        pass(ok),
        m.unit_y,
        m.origin_x,
        m.origin_y
    );
    assert!(ok);
}

#[test]
fn physical_locked_rhythm_varies_by_one_pixel() {
    let mut doc = slice_document();
    doc.rhythm.resolution = RhythmResolution::PhysicalLocked;
    let (manta, _) = profiles();
    let r = RhythmPx::new(&doc, &manta);
    let ys = RhythmPx::positions(r.origin_y, r.unit_y, 0.0, 2560.0);
    let mut gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    gaps.sort_by(f64::total_cmp);
    gaps.dedup();
    let ok = gaps == [70.0, 71.0];
    println!(
        "CHECK crate=layout name=physical_locked_gaps result={} gaps_px={gaps:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn slice_page_snaps_to_whole_pixels() {
    // Every hard-edged rectangle sits on whole pixels and is exactly its role's width.
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let mut parts = Vec::new();
    let mut ok = true;
    for p in [&manta, &nomad] {
        let dl = page(&doc, p);
        let rs = rects(&dl);
        let whole = rs
            .iter()
            .all(|r| [r.0, r.1, r.2, r.3].iter().all(|v| v.fract() == 0.0));
        let dots = rs
            .iter()
            .filter(|r| r.4 == p.role(LineRole::Whisper).gray)
            .count();
        let lines: Vec<_> = rs
            .iter()
            .filter(|r| r.4 == p.role(LineRole::Guide).gray)
            .collect();
        let line_w_ok = lines
            .iter()
            .all(|r| r.3 - r.1 == p.role(LineRole::Guide).width_px as f64);
        let fills = dl
            .items
            .iter()
            .filter(|i| matches!(i, Item::Fill { .. }))
            .count();
        ok &= whole && line_w_ok && dots > 0 && !lines.is_empty() && fills == 1;
        parts.push(format!(
            "{}:rects={}:dots={dots}:lines={}:text_fills={fills}:whole_px={whole}",
            p.id,
            rs.len(),
            lines.len()
        ));
    }
    println!(
        "CHECK crate=layout name=slice_whole_pixels result={} {}",
        pass(ok),
        parts.join(" ")
    );
    assert!(ok);
}

#[test]
fn pinned_header_keeps_physical_distance() {
    // The title rule is pinned to the top (VConstraint::Top), so it sits at the same physical
    // height on both profiles; both are 300 PPI, so the same pixel row.
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let structure = manta.role(LineRole::Structure).gray;
    let rule_y = |p: &Profile| {
        rects(&page(&doc, p))
            .into_iter()
            .find(|r| r.4 == structure)
            .map(|r| r.1)
    };
    let (m, n) = (rule_y(&manta), rule_y(&nomad));
    let ok = m.is_some() && m == n;
    println!(
        "CHECK crate=layout name=pinned_header result={} rule_y0_manta={m:?} rule_y0_nomad={n:?}",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn bad_outlines_are_errors_not_panics() {
    let doc = slice_document();
    let (manta, nomad) = profiles();
    let mut reg = FontRegistry::bundled_only();
    let good = lay_out_text(&doc, &manta, &manta, &mut reg)
        .unwrap()
        .outlines;

    let wrong_profile = display_list(&doc, &manta, &nomad, &good);
    let empty = ProfileOutlines {
        texts: vec![],
        ..good.clone()
    };
    let missing_text = display_list(&doc, &manta, &manta, &empty);
    let mut no_tables = good.clone();
    no_tables.tables.clear();
    let missing_table = display_list(&doc, &manta, &manta, &no_tables);
    let mut broken = good.clone();
    for s in broken.tables[0].glyphs.values_mut() {
        s.coords.pop();
    }
    let malformed = display_list(&doc, &manta, &manta, &broken);

    let ok = matches!(wrong_profile, Err(LayoutError::WrongProfile { .. }))
        && matches!(missing_text, Err(LayoutError::MissingText(_)))
        && matches!(missing_table, Err(LayoutError::BadOutlines(_)))
        && matches!(malformed, Err(LayoutError::BadOutlines(_)));
    println!(
        "CHECK crate=layout name=bad_outlines_errors result={} cases=4",
        pass(ok)
    );
    assert!(ok);
}

#[test]
fn hidden_objects_and_layers_are_not_drawn() {
    let mut doc = slice_document();
    let (manta, _) = profiles();
    let all = page(&doc, &manta).items.len();
    doc.layers[0].visible = false; // the grid
    let no_grid = page(&doc, &manta).items.len();
    doc.layers[1].objects[0].visible = false; // the title
    let no_title = page(&doc, &manta).items.len();
    let ok = no_grid == 2 && no_title == 1 && all > no_grid;
    println!(
        "CHECK crate=layout name=hidden_not_drawn result={} items_all={all} no_grid={no_grid} no_title={no_title}",
        pass(ok)
    );
    assert!(ok);
}
