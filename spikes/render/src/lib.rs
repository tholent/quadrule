//! Throwaway rendering spike for quadrule (napkin §10, §11.1, §16 spike 1).
//!
//! Scene in millimetres -> per-profile layout with pixel snapping -> display list in device px
//! -> backends (tiny-skia always; vello_cpu and vello GPU behind features) -> 8-bit gray PNG.

pub mod checks;
pub mod export;
pub mod layout;
pub mod scene;
pub mod tiny;

#[cfg(feature = "vello-cpu")]
pub mod vcpu;

#[cfg(feature = "gpu")]
pub mod vgpu;

/// A target screen. Only the fields this spike needs.
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub id: &'static str,
    pub width_px: u32,
    pub height_px: u32,
    pub ppi: f64,
}

pub const MANTA: Profile = Profile { id: "manta", width_px: 1920, height_px: 2560, ppi: 300.0 };
pub const A5X: Profile = Profile { id: "a5x", width_px: 1404, height_px: 1872, ppi: 226.0 };

impl Profile {
    pub fn mm_to_px(&self, mm: f64) -> f64 {
        mm * self.ppi / 25.4
    }
}

/// An 8-bit grayscale raster: 0 = black, 255 = white. Row-major, no alpha.
#[derive(Clone, PartialEq, Eq)]
pub struct Gray {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Gray {
    pub fn at(&self, x: u32, y: u32) -> u8 {
        self.data[(y * self.width + x) as usize]
    }
}
