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

//! What a profile file may hold (rung 13a bbq, Defaulted "Limits"). The form, the import and the
//! carried-copy reader accept only these values; a profile outside them counts as damaged, so a
//! hostile file cannot ask for a huge page. Nothing here panics, whatever the input.

use crate::{PaperSize, Profile};
use quadrule_model::{MM_PER_IN, PAPER_PPIS};

/// The most bytes a profile file may hold; a larger one is damaged and is not read.
pub const MAX_FILE_BYTES: usize = 256 * 1024;
/// Smallest width or height in pixels.
pub const MIN_PX: u32 = 16;
/// Largest width or height in pixels.
pub const MAX_PX: u32 = 16_384;
/// Smallest pixels per inch.
pub const MIN_PPI: f64 = 50.0;
/// Largest pixels per inch.
pub const MAX_PPI: f64 = 1_200.0;
/// Narrowest and widest a line role or dot may be, in pixels.
pub const ROLE_PX: std::ops::RangeInclusive<u32> = 1..=32;
/// Largest width or height of a paper page at the top export PPI (600), in pixels (rung 17, Q7; 13b's debt: 10,400
/// px is 440 mm). Measured on `gibson` in a release build, a page's export pixels drawn and written as a PNG: A3
/// (7,016 × 9,921, 70 M pixels) 0.49 s and 0.23 s, 350 MB at the peak; 420 × 420 mm (98 M) 0.72 s and 0.26 s,
/// 480 MB; 500 × 500 mm (139 M) 0.98 s and 0.31 s, 680 MB; 693 × 693 mm (the side the form allowed before, 16,370
/// px, 268 M) 1.5 s and 0.5 s, 1.3 GB. Zoomed in past 1:1 each edit redraws all of it, so the cap sits just over
/// Tabloid's long side (431.8 mm, 10,200 px) and A3's (420 mm), about 108 M pixels at most.
pub const MAX_PAPER_PX: u32 = 10_400;
/// The shortest and longest side a custom paper size may have, in mm, as the form offers: the span in
/// which every export PPI a page can pick (150 to 600) gives [`MIN_PX`] to [`MAX_PAPER_PX`] whole pixels.
/// [`check`] holds a paper size to the pixel limits themselves; these are those limits rounded
/// inwards.
pub const PAPER_MM: std::ops::RangeInclusive<f64> = 2.8..=440.0;
/// The most safe zones a profile may hold.
pub const MAX_ZONES: usize = 64;
/// The longest display name, in characters.
pub const MAX_NAME_CHARS: usize = 80;
/// The longest short name, in characters.
pub const MAX_SHORT_CHARS: usize = 40;
/// The longest id, in characters.
pub const MAX_ID_CHARS: usize = 80;
/// The longest free text (panel, notes, firmware, date, a hint), in characters.
pub const MAX_TEXT_CHARS: usize = 2_000;
/// The most export hints, and gray or ink values, a profile may list.
pub const MAX_LIST: usize = 64;

/// Whether `s` is a short name: lowercase ASCII letters, digits and hyphens, 1 to
/// [`MAX_SHORT_CHARS`] of them, not starting or ending with a hyphen.
pub fn check_short_name(s: &str) -> Result<(), String> {
    let ok = !s.is_empty()
        && s.chars().count() <= MAX_SHORT_CHARS
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-');
    if ok {
        Ok(())
    } else {
        Err(format!(
            "short name `{}` must be 1 to {MAX_SHORT_CHARS} lowercase letters, digits or hyphens",
            s.chars().take(MAX_SHORT_CHARS + 1).collect::<String>()
        ))
    }
}

/// Whether `s` can be an id, and so a file name: ASCII letters, digits, dots, hyphens and
/// underscores, not starting with a dot.
pub fn check_id(s: &str) -> Result<(), String> {
    let ok = !s.is_empty()
        && s.chars().count() <= MAX_ID_CHARS
        && !s.starts_with('.')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if ok {
        Ok(())
    } else {
        Err("the id must be 1 to 80 letters, digits, dots, hyphens or underscores".into())
    }
}

fn text(what: &str, s: &str) -> Result<(), String> {
    if s.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{what} is longer than {MAX_TEXT_CHARS} characters"))
    } else {
        Ok(())
    }
}

/// A paper profile's size: each side must come to 16 to 16,384 whole pixels at every export PPI a
/// page can pick (150 to 600), so no page it makes breaks the screen limits above; and its pixel
/// size and ppi are those of its size at 300 PPI ([`Profile::sync_paper`]).
fn check_paper(p: &Profile, size: PaperSize) -> Result<(), String> {
    for (what, mm) in [("width", size.width_mm), ("height", size.height_mm)] {
        if !mm.is_finite() || mm <= 0.0 {
            return Err(format!("the paper {what} must be a number above 0 mm"));
        }
    }
    let (lo_w, lo_h) = size.px_at(PAPER_PPIS[0]);
    let (hi_w, hi_h) = size.px_at(PAPER_PPIS[PAPER_PPIS.len() - 1]);
    if lo_w.min(lo_h) < MIN_PX || hi_w.max(hi_h) > MAX_PAPER_PX {
        let (lo_mm, hi_mm) = (
            f64::from(MIN_PX) * MM_PER_IN / f64::from(PAPER_PPIS[0]),
            f64::from(MAX_PAPER_PX) * MM_PER_IN / f64::from(PAPER_PPIS[PAPER_PPIS.len() - 1]),
        );
        return Err(format!(
            "each side of a paper size must be {lo_mm:.1} to {hi_mm:.1} mm, so that every export PPI gives {MIN_PX} to {MAX_PAPER_PX} px"
        ));
    }
    let mut want = p.clone();
    want.sync_paper();
    if (want.width_px, want.height_px) != (p.width_px, p.height_px) || want.ppi != p.ppi {
        return Err("a paper profile's pixel size and ppi are those of its size at 300 PPI".into());
    }
    Ok(())
}

/// The lint block's numbers (rung 14a): sizes and steps finite and from 0 to 1000, the page share 0 to 1,
/// the thin-line width within the role widths. A gray is a whole number 0 to 255 already.
fn check_lint(l: &crate::LintThresholds) -> Result<(), String> {
    for (what, v, hi) in [
        ("the smallest text size", l.smallest_text_pt, 1_000.0),
        (
            "the smallest handwriting step",
            l.smallest_handwriting_step_mm,
            1_000.0,
        ),
        ("the dark fill page share", l.dark_fill_page_share, 1.0),
    ] {
        if let Some(v) = v
            && (!v.is_finite() || !(0.0..=hi).contains(&v))
        {
            return Err(format!("{what} must be a number from 0 to {hi}"));
        }
    }
    if let Some(w) = l.thin_line_px
        && !(0..=*ROLE_PX.end()).contains(&w)
    {
        return Err(format!(
            "the thin line width must be 0 to {} px",
            ROLE_PX.end()
        ));
    }
    Ok(())
}

/// Checks `p` against the limits. The error says which value is out and why.
pub fn check(p: &Profile) -> Result<(), String> {
    check_id(&p.id)?;
    if p.display_name.trim().is_empty() || p.display_name.chars().count() > MAX_NAME_CHARS {
        return Err(format!("the name must be 1 to {MAX_NAME_CHARS} characters"));
    }
    check_short_name(&p.short_name)?;
    for (what, v) in [("width", p.width_px), ("height", p.height_px)] {
        if !(MIN_PX..=MAX_PX).contains(&v) {
            return Err(format!("{what} {v} px is outside {MIN_PX} to {MAX_PX}"));
        }
    }
    if !p.ppi.is_finite() || !(MIN_PPI..=MAX_PPI).contains(&p.ppi) {
        return Err(format!("ppi {} is outside {MIN_PPI} to {MAX_PPI}", p.ppi));
    }
    if let Some(size) = p.paper {
        check_paper(p, size)?;
    }
    if !p.diagonal_in.is_finite() || !(0.0..=1_000.0).contains(&p.diagonal_in) {
        return Err("the diagonal must be 0 to 1000 inches".into());
    }
    text("the panel", &p.panel)?;
    if p.gray_levels.len() > MAX_LIST || p.ink_values.len() > MAX_LIST {
        return Err(format!("at most {MAX_LIST} gray levels or pen grays"));
    }
    if p.orientations.is_empty() || p.orientations.len() > 2 {
        return Err("a profile lists one or two orientations".into());
    }
    let r = &p.line_roles;
    for (name, w) in [
        ("whisper", r.whisper.width_px),
        ("guide", r.guide.width_px),
        ("structure", r.structure.width_px),
        ("emphasis", r.emphasis.width_px),
        ("dot_grid", r.dot_grid.width_px),
        ("margin", r.margin.width_px),
    ] {
        if !ROLE_PX.contains(&w) {
            return Err(format!(
                "role {name} is {w} px wide, outside {} to {}",
                ROLE_PX.start(),
                ROLE_PX.end()
            ));
        }
    }
    check_lint(&p.lint)?;
    if p.safe_zones.len() > MAX_ZONES {
        return Err(format!("more than {MAX_ZONES} safe zones"));
    }
    for z in &p.safe_zones {
        let (x, y, w, h) = z.rect;
        let (x, y, w, h) = (x as u64, y as u64, w as u64, h as u64);
        if w == 0 || h == 0 || x + w > p.width_px as u64 || y + h > p.height_px as u64 {
            return Err(format!("safe zone `{}` is not inside the screen", z.name));
        }
        if z.name.chars().count() > MAX_NAME_CHARS {
            return Err("a safe zone name is too long".into());
        }
    }
    let m = &p.default_margins_mm;
    let (wmm, hmm) = p.page_mm();
    let finite = [m.top, m.right, m.bottom, m.left]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0);
    if !finite || m.left + m.right >= wmm || m.top + m.bottom >= hmm {
        return Err("the margins must be 0 or more and leave room on the page".into());
    }
    if let Some(d) = &p.calibration_meta.date {
        text("the date", d)?;
    }
    if let Some(f) = &p.calibration_meta.firmware {
        text("the firmware", f)?;
    }
    text("the notes", &p.calibration_meta.method)?;
    if p.export_hints.len() > MAX_LIST {
        return Err(format!("more than {MAX_LIST} export hints"));
    }
    for h in &p.export_hints {
        text("an export hint", h)?;
    }
    Ok(())
}
