//! The themes: one palette per `BPaletteName` and mode. Structure here —
//! the lookup; the palettes themselves, seed by seed, are `set.rs`.

pub mod set;

use super::{BPalette, BPaletteName, BThemeMode};

/// A theme's palette in its dark mode.
pub fn palette_for(name: BPaletteName) -> &'static BPalette {
    palette_in(name, BThemeMode::Dark)
}

/// A theme's palette in `mode`.
pub fn palette_in(name: BPaletteName, mode: BThemeMode) -> &'static BPalette {
    match (name, mode) {
        (BPaletteName::Sphere, BThemeMode::Dark) => &set::SPHERE,
        (BPaletteName::Sphere, BThemeMode::Light) => &set::SPHERE_LIGHT,
        (BPaletteName::Graphite, BThemeMode::Dark) => &set::GRAPHITE,
        (BPaletteName::Graphite, BThemeMode::Light) => &set::GRAPHITE_LIGHT,
        (BPaletteName::Tokyo, BThemeMode::Dark) => &set::TOKYO,
        (BPaletteName::Tokyo, BThemeMode::Light) => &set::TOKYO_LIGHT,
        (BPaletteName::Onyx, BThemeMode::Dark) => &set::ONYX,
        (BPaletteName::Onyx, BThemeMode::Light) => &set::ONYX_LIGHT,
        (BPaletteName::Grey, BThemeMode::Dark) => &set::GREY,
        (BPaletteName::Grey, BThemeMode::Light) => &set::GREY_LIGHT,
        (BPaletteName::Apple, BThemeMode::Dark) => &set::APPLE,
        (BPaletteName::Apple, BThemeMode::Light) => &set::APPLE_LIGHT,
        (BPaletteName::Swift, BThemeMode::Dark) => &set::SWIFT,
        (BPaletteName::Swift, BThemeMode::Light) => &set::SWIFT_LIGHT,
        (BPaletteName::Barbie, BThemeMode::Dark) => &set::BARBIE,
        (BPaletteName::Barbie, BThemeMode::Light) => &set::BARBIE_LIGHT,
        (BPaletteName::Fenty, BThemeMode::Dark) => &set::FENTY,
        (BPaletteName::Fenty, BThemeMode::Light) => &set::FENTY_LIGHT,
        (BPaletteName::Cyberpunk, BThemeMode::Dark) => &set::CYBERPUNK,
        (BPaletteName::Cyberpunk, BThemeMode::Light) => &set::CYBERPUNK_LIGHT,
        (BPaletteName::Chanel, BThemeMode::Dark) => &set::CHANEL,
        (BPaletteName::Chanel, BThemeMode::Light) => &set::CHANEL_LIGHT,
    }
}
