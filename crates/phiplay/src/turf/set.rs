//! The eight surfaces.

use super::{BGrain, BTurf, BTurfSpec};
use crate::colour::Rgb;

impl BTurf {
    pub fn spec(self) -> &'static BTurfSpec {
        match self {
            BTurf::Maple => &MAPLE,
            BTurf::Polymer => &POLYMER,
            BTurf::Glass => &GLASS,
            BTurf::Rubber => &RUBBER,
            BTurf::Grass => &GRASS,
            BTurf::Stripes => &STRIPES,
            BTurf::Checks => &CHECKS,
            BTurf::Sand => &SAND,
        }
    }
}

static MAPLE: BTurfSpec = BTurfSpec { label: "Maple", base: Rgb::hex(0xC99A62), alt: Rgb::hex(0xB8854E), grain: BGrain::Planks { width: 0.18 }, roughness: 0.3, emissive: false };
static POLYMER: BTurfSpec = BTurfSpec { label: "Polymer court", base: Rgb::hex(0x2F6FA8), alt: Rgb::hex(0x2A6398), grain: BGrain::Speckle, roughness: 0.55, emissive: false };
static GLASS: BTurfSpec = BTurfSpec { label: "LED glass", base: Rgb::hex(0x15161C), alt: Rgb::hex(0x22242E), grain: BGrain::Tiles { size: 1.0 }, roughness: 0.15, emissive: true };
static RUBBER: BTurfSpec = BTurfSpec { label: "Rubber", base: Rgb::hex(0xB5523B), alt: Rgb::hex(0xA64A35), grain: BGrain::Speckle, roughness: 0.8, emissive: false };
static GRASS: BTurfSpec = BTurfSpec { label: "Grass", base: Rgb::hex(0x3E8E3A), alt: Rgb::hex(0x377F33), grain: BGrain::Speckle, roughness: 0.9, emissive: false };
static STRIPES: BTurfSpec = BTurfSpec { label: "Mown stripes", base: Rgb::hex(0x4A9A40), alt: Rgb::hex(0x3B8433), grain: BGrain::Mown { width: 5.25 }, roughness: 0.9, emissive: false };
static CHECKS: BTurfSpec = BTurfSpec { label: "Mown checks", base: Rgb::hex(0x4A9A40), alt: Rgb::hex(0x3B8433), grain: BGrain::Chequer { size: 5.25 }, roughness: 0.9, emissive: false };
static SAND: BTurfSpec = BTurfSpec { label: "Sand", base: Rgb::hex(0xE3C99A), alt: Rgb::hex(0xD6B884), grain: BGrain::Ripples, roughness: 0.95, emissive: false };
