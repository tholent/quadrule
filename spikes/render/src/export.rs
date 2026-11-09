//! 8-bit grayscale PNG, no alpha, optional tEXt metadata (napkin §11.1).

use crate::Gray;

pub struct Meta<'a> {
    pub app_version: &'a str,
    pub title: &'a str,
}

pub fn encode_png(img: &Gray, meta: Option<&Meta>) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.width, img.height);
        enc.set_color(png::ColorType::Grayscale);
        enc.set_depth(png::BitDepth::Eight);
        if let Some(m) = meta {
            enc.add_text_chunk("Software".into(), format!("Quadrule {}", m.app_version)).expect("text");
            enc.add_text_chunk("Title".into(), m.title.into()).expect("text");
        }
        let mut w = enc.write_header().expect("header");
        w.write_image_data(&img.data).expect("data");
    }
    out
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}
