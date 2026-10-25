//! The eight themes.

use super::{BTheme, BThemeSpec};
use crate::colour::Rgb;

impl BTheme {
    pub fn spec(self) -> &'static BThemeSpec {
        match self {
            BTheme::Bloom => &BLOOM,
            BTheme::DesertDusk => &DESERT_DUSK,
            BTheme::Lagoon => &LAGOON,
            BTheme::NeonNight => &NEON_NIGHT,
            BTheme::Savanna => &SAVANNA,
            BTheme::Glacier => &GLACIER,
            BTheme::Carnival => &CARNIVAL,
            BTheme::Studio => &STUDIO,
        }
    }
}

/// The LA28 palette the demo court was painted in: poppy, bluebell, sage.
static BLOOM: BThemeSpec = BThemeSpec {
    label: "Bloom",
    caption: "Poppy, bluebell and sage, the colours of a Californian spring",
    primary: Rgb::hex(0xD9452B),
    secondary: Rgb::hex(0x5B7FD1),
    accent: Rgb::hex(0xF2B33D),
    apron: Rgb::hex(0x8FA88A),
    line: Rgb::hex(0xF7F5F0),
    glow: Rgb::hex(0xFF8A5C),
    seats: [Rgb::hex(0xD9452B), Rgb::hex(0xE8DCC2), Rgb::hex(0x5B7FD1), Rgb::hex(0x8FA88A)],
    sky: Rgb::hex(0xCFE0F2),
    key: Rgb::hex(0xFFF1DC),
    tempo: 96.0,
};

/// Maroon and gold on sand, with the lantern light of a Gulf evening.
static DESERT_DUSK: BThemeSpec = BThemeSpec {
    label: "Desert Dusk",
    caption: "Maroon, gold and sand under a lantern sky",
    primary: Rgb::hex(0x8A1538),
    secondary: Rgb::hex(0xC9A227),
    accent: Rgb::hex(0xF4D58D),
    apron: Rgb::hex(0x3B1E2A),
    line: Rgb::hex(0xF5E6C8),
    glow: Rgb::hex(0xF0B54A),
    seats: [Rgb::hex(0x8A1538), Rgb::hex(0xE8D3A9), Rgb::hex(0xB8862B), Rgb::hex(0x5C0F26)],
    sky: Rgb::hex(0xF3B27A),
    key: Rgb::hex(0xFFD9A8),
    tempo: 72.0,
};

static LAGOON: BThemeSpec = BThemeSpec {
    label: "Lagoon",
    caption: "Teal water, coral and white sand",
    primary: Rgb::hex(0x0E7C86),
    secondary: Rgb::hex(0xF06B5B),
    accent: Rgb::hex(0xFFE08A),
    apron: Rgb::hex(0x0B4F5C),
    line: Rgb::hex(0xF2FBFA),
    glow: Rgb::hex(0x48D1CC),
    seats: [Rgb::hex(0x0E7C86), Rgb::hex(0xF2FBFA), Rgb::hex(0xF06B5B), Rgb::hex(0x7FD3CF)],
    sky: Rgb::hex(0xBFE9F0),
    key: Rgb::hex(0xFFF6E4),
    tempo: 88.0,
};

static NEON_NIGHT: BThemeSpec = BThemeSpec {
    label: "Neon Night",
    caption: "Magenta and cyan on black, for a game under the lights",
    primary: Rgb::hex(0xE0218A),
    secondary: Rgb::hex(0x22D3EE),
    accent: Rgb::hex(0xA3E635),
    apron: Rgb::hex(0x14121F),
    line: Rgb::hex(0xF5F3FF),
    glow: Rgb::hex(0x8B5CF6),
    seats: [Rgb::hex(0x2A2340), Rgb::hex(0xE0218A), Rgb::hex(0x22D3EE), Rgb::hex(0x1C1830)],
    sky: Rgb::hex(0x241B3D),
    key: Rgb::hex(0xE9E4FF),
    tempo: 128.0,
};

static SAVANNA: BThemeSpec = BThemeSpec {
    label: "Savanna",
    caption: "Ochre, acacia green and terracotta at golden hour",
    primary: Rgb::hex(0xC0702A),
    secondary: Rgb::hex(0x6B7F3A),
    accent: Rgb::hex(0xF2C14E),
    apron: Rgb::hex(0x5A3A22),
    line: Rgb::hex(0xF8EEDC),
    glow: Rgb::hex(0xF29E4C),
    seats: [Rgb::hex(0xC0702A), Rgb::hex(0x6B7F3A), Rgb::hex(0xE9D8B4), Rgb::hex(0x8C4A2F)],
    sky: Rgb::hex(0xF6D7A7),
    key: Rgb::hex(0xFFE2B0),
    tempo: 84.0,
};

static GLACIER: BThemeSpec = BThemeSpec {
    label: "Glacier",
    caption: "Ice blue, silver and white, cold and clean",
    primary: Rgb::hex(0x4A90C2),
    secondary: Rgb::hex(0xB8C4CE),
    accent: Rgb::hex(0x0B3C5D),
    apron: Rgb::hex(0x1E3A52),
    line: Rgb::hex(0xFFFFFF),
    glow: Rgb::hex(0xA7D8F5),
    seats: [Rgb::hex(0xF4F8FB), Rgb::hex(0x4A90C2), Rgb::hex(0xB8C4CE), Rgb::hex(0x0B3C5D)],
    sky: Rgb::hex(0xDDEBF5),
    key: Rgb::hex(0xF4FAFF),
    tempo: 76.0,
};

static CARNIVAL: BThemeSpec = BThemeSpec {
    label: "Carnival",
    caption: "Samba yellow, green and blue, loud and happy",
    primary: Rgb::hex(0x009C3B),
    secondary: Rgb::hex(0xFFDF00),
    accent: Rgb::hex(0x002776),
    apron: Rgb::hex(0x0B5A2E),
    line: Rgb::hex(0xFFFDF2),
    glow: Rgb::hex(0xFFD23F),
    seats: [Rgb::hex(0xFFDF00), Rgb::hex(0x009C3B), Rgb::hex(0x002776), Rgb::hex(0xFFFDF2)],
    sky: Rgb::hex(0xC9F0FF),
    key: Rgb::hex(0xFFF3C4),
    tempo: 132.0,
};

/// Black, white and one red: a broadcast studio's restraint.
static STUDIO: BThemeSpec = BThemeSpec {
    label: "Studio",
    caption: "Black and white with one red, nothing else",
    primary: Rgb::hex(0x1A1A1E),
    secondary: Rgb::hex(0xE9E9EC),
    accent: Rgb::hex(0xE10600),
    apron: Rgb::hex(0x2B2B31),
    line: Rgb::hex(0xF4F4F6),
    glow: Rgb::hex(0xFFFFFF),
    seats: [Rgb::hex(0x1A1A1E), Rgb::hex(0x3A3A40), Rgb::hex(0xE9E9EC), Rgb::hex(0x1A1A1E)],
    sky: Rgb::hex(0xE6E6EA),
    key: Rgb::hex(0xFFFFFF),
    tempo: 100.0,
};
