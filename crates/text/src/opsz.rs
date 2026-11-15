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

//! Optical size (rung 11a, bbq record Defaulted "Font files": the `opsz` axis follows the point size).
//!
//! cosmic-text 0.19 sets only the `wght` axis when it loads a face: every other axis stays at the
//! font's default, for shaping and for the outlines read through the same face. To get a font's
//! other-than-default optical size without changing cosmic-text, [`pin_opsz`] makes a copy of the
//! font file whose `opsz` axis is *retagged as a second `wght` axis* with a constant `avar` map: the
//! code that sets "the weight" then also sets the optical size, because skrifa gives every axis with
//! a requested tag its own mapped coordinate. The copy is only ever used in memory; no copy is
//! saved or shipped, and a saved `.quad` holds the drawn shapes, not the font (ADR-001). A test
//! (`tests/fonts.rs`) requires the copy to give exactly the coordinates the original gives when
//! asked for the optical size by name.

use cosmic_text::skrifa::{FontRef, MetadataProvider, Tag};

const OPSZ: Tag = Tag::new(b"opsz");
const WGHT: Tag = Tag::new(b"wght");

/// The `opsz` axis of a font: (min, default, max) in points, or `None` for a font without one.
pub fn opsz_range(data: &[u8]) -> Option<(f32, f32, f32)> {
    let f = FontRef::new(data).ok()?;
    let a = f.axes().get_by_tag(OPSZ)?;
    Some((a.min_value(), a.default_value(), a.max_value()))
}

/// The `wght` axis of a font: (min, default, max), or `None` for a font without one.
pub fn wght_range(data: &[u8]) -> Option<(f32, f32, f32)> {
    let f = FontRef::new(data).ok()?;
    let a = f.axes().get_by_tag(WGHT)?;
    Some((a.min_value(), a.default_value(), a.max_value()))
}

fn be16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The sfnt checksum of a table: the sum of its big-endian 32-bit words, zero padded.
fn checksum(b: &[u8]) -> u32 {
    b.chunks(4).fold(0u32, |sum, c| {
        let mut w = [0u8; 4];
        w[..c.len()].copy_from_slice(c);
        sum.wrapping_add(u32::from_be_bytes(w))
    })
}

/// A copy of the font file whose requested "weight" also sets the optical size to `size_pt`
/// (clamped to the font's own range). `None` when the file has no `opsz` axis or cannot be
/// patched (not a single-font TrueType/OpenType file, or an `avar` table newer than version 1).
pub fn pin_opsz(data: &[u8], size_pt: f32) -> Option<Vec<u8>> {
    let font = FontRef::new(data).ok()?;
    let axes = font.axes();
    let opsz_i = axes.iter().position(|a| a.tag() == OPSZ)?;
    let wght = axes.get_by_tag(WGHT)?;
    let axis_count = axes.len();
    // The coordinate the optical size really maps to, `avar` included.
    let target = axes.location([(OPSZ, size_pt)]).coords()[opsz_i].to_bits();

    // The table directory.
    let num = usize::from(be16(data, 4)?);
    let mut tables: Vec<([u8; 4], Vec<u8>)> = Vec::with_capacity(num + 1);
    for i in 0..num {
        let rec = 12 + 16 * i;
        let tag: [u8; 4] = data.get(rec..rec + 4)?.try_into().ok()?;
        let off = be32(data, rec + 8)? as usize;
        let len = be32(data, rec + 12)? as usize;
        tables.push((tag, data.get(off..off.checked_add(len)?)?.to_vec()));
    }

    // fvar: the opsz axis becomes a second wght axis with the real weight axis's range, so
    // anything that asks for "the weight axis" sees a plausible one.
    let fvar = &mut tables.iter_mut().find(|(t, _)| t == b"fvar")?.1;
    let axes_at = usize::from(be16(fvar, 4)?);
    let axis_size = usize::from(be16(fvar, 10)?);
    let rec = axes_at + opsz_i * axis_size;
    let fixed = |v: f32| ((f64::from(v) * 65536.0).round() as i32).to_be_bytes();
    fvar.get_mut(rec..rec + 16)?.copy_from_slice(
        &[
            b"wght".as_slice(),
            &fixed(wght.min_value()),
            &fixed(wght.default_value()),
            &fixed(wght.max_value()),
        ]
        .concat(),
    );

    // avar version 1: the maps of every axis, the opsz one replaced by a constant.
    let old_avar = tables
        .iter()
        .find(|(t, _)| t == b"avar")
        .map(|(_, b)| b.clone());
    let mut maps: Vec<Vec<(i16, i16)>> = Vec::with_capacity(axis_count);
    match &old_avar {
        Some(a) => {
            if be16(a, 0)? != 1 || usize::from(be16(a, 6)?) != axis_count {
                return None;
            }
            let mut at = 8;
            for _ in 0..axis_count {
                let n = usize::from(be16(a, at)?);
                at += 2;
                let mut m = Vec::with_capacity(n);
                for _ in 0..n {
                    m.push((be16(a, at)? as i16, be16(a, at + 2)? as i16));
                    at += 4;
                }
                maps.push(m);
            }
        }
        None => maps = vec![vec![(-0x4000, -0x4000), (0, 0), (0x4000, 0x4000)]; axis_count],
    }
    maps[opsz_i] = vec![(-0x4000, target), (0, target), (0x4000, target)];
    let mut avar = vec![0, 1, 0, 0, 0, 0];
    avar.extend_from_slice(&(axis_count as u16).to_be_bytes());
    for m in &maps {
        avar.extend_from_slice(&(m.len() as u16).to_be_bytes());
        for (from, to) in m {
            avar.extend_from_slice(&from.to_be_bytes());
            avar.extend_from_slice(&to.to_be_bytes());
        }
    }
    match tables.iter_mut().find(|(t, _)| t == b"avar") {
        Some(t) => t.1 = avar,
        None => {
            tables.push((*b"avar", avar));
            tables.sort_by_key(|(t, _)| *t);
        }
    }

    // Write the file again: header, directory, tables on four-byte boundaries.
    let n = tables.len();
    let entry_selector = usize::BITS - 1 - n.leading_zeros();
    let search_range = 16usize << entry_selector;
    let mut out = Vec::with_capacity(data.len() + 1024);
    out.extend_from_slice(data.get(0..4)?);
    out.extend_from_slice(&(n as u16).to_be_bytes());
    out.extend_from_slice(&(search_range as u16).to_be_bytes());
    out.extend_from_slice(&(entry_selector as u16).to_be_bytes());
    out.extend_from_slice(&((n * 16 - search_range) as u16).to_be_bytes());
    let mut off = 12 + 16 * n;
    for (tag, bytes) in &tables {
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(bytes).to_be_bytes());
        out.extend_from_slice(&(off as u32).to_be_bytes());
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        off += bytes.len().div_ceil(4) * 4;
    }
    for (_, bytes) in &tables {
        out.extend_from_slice(bytes);
        out.resize(out.len().div_ceil(4) * 4, 0);
    }
    Some(out)
}
