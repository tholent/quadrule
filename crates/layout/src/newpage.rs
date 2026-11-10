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

//! A new page's margins and the document built from the New dialog (rung 5,
//! docs/bbq/2026-11-10-rung-5-new-and-insert.md, Defaulted "New page margins"). The margins
//! clear the docked left, right and top toolbars and the footers of every device the page
//! targets, rounded up to whole rhythm units, so a region that fills the margins passes lint
//! rule L06. The floating toolbar moves, so it is not cleared (the linter only notes it).

use quadrule_model::blank::{RhythmPreset, blank_document};
use quadrule_model::{Document, Margins};
use quadrule_profiles::{Profile, ProfileError, builtin, builtin_ids};

/// The thickness each side of the page must clear on `p`, in device pixels: the docked toolbars
/// on the left, right and top, and the footers at the bottom (`Margins` order: top, right,
/// bottom, left).
pub fn zone_thickness_px(p: &Profile) -> Margins {
    let (w, h) = (f64::from(p.width_px), f64::from(p.height_px));
    let mut m = Margins::all(0.0);
    for z in &p.safe_zones {
        let (x, y, zw, zh) = z.rect;
        let (x, y, zw, zh) = (f64::from(x), f64::from(y), f64::from(zw), f64::from(zh));
        match z.name.as_str() {
            "toolbar_left" => m.left = m.left.max(x + zw),
            "toolbar_right" => m.right = m.right.max(w - x),
            "toolbar_top" => m.top = m.top.max(y + zh),
            n if n.starts_with("footer_") => m.bottom = m.bottom.max(h - y),
            _ => {}
        }
    }
    m
}

/// Margins, in mm, that clear the docked toolbars and the footers of every profile in
/// `profiles`: each the least whole number of rhythm units (`h_unit_mm` across, `v_unit_mm`
/// down) that is thick enough on every device.
pub fn clearing_margins(profiles: &[Profile], h_unit_mm: f64, v_unit_mm: f64) -> Margins {
    // The least whole number of units whose length is at least `need_px` on every profile.
    let side = |unit_mm: f64, need: &dyn Fn(&Profile) -> f64| -> f64 {
        let mut n = 1.0;
        while profiles
            .iter()
            .any(|p| p.mm_to_px(n * unit_mm) < need(p) - 1e-9)
        {
            n += 1.0;
        }
        n * unit_mm
    };
    Margins {
        top: side(v_unit_mm, &|p| zone_thickness_px(p).top),
        right: side(h_unit_mm, &|p| zone_thickness_px(p).right),
        bottom: side(v_unit_mm, &|p| zone_thickness_px(p).bottom),
        left: side(h_unit_mm, &|p| zone_thickness_px(p).left),
    }
}

/// What the New dialog asks for.
#[derive(Clone, Debug, PartialEq)]
pub struct NewPage {
    /// The title.
    pub title: String,
    /// The device the page is designed on (a built-in profile id).
    pub device_id: String,
    /// The rhythm to start from.
    pub rhythm: RhythmPreset,
}

/// The blank template for `spec`: designed on the chosen device and also targeting every other
/// built-in one, margins clearing the toolbars and footers of all of them, `now` as the created
/// and modified time (RFC 3339).
pub fn new_document(spec: &NewPage, now: &str) -> Result<Document, ProfileError> {
    let primary = builtin(&spec.device_id)?;
    let others: Vec<String> = builtin_ids()
        .filter(|id| *id != primary.id)
        .map(String::from)
        .collect();
    let mut profiles = vec![primary.clone()];
    for id in &others {
        profiles.push(builtin(id)?);
    }
    let unit = spec.rhythm.unit_mm();
    let margins = clearing_margins(&profiles, unit, unit);
    let others: Vec<&str> = others.iter().map(String::as_str).collect();
    Ok(blank_document(
        &spec.title,
        &primary.id,
        &others,
        unit,
        margins,
        now,
    ))
}
