//! Color palettes per theme name — each a shade ladder (`shade/`) read at
//! fixed steps. Every rendered color resolves through
//! `palette_for(active_theme)`; style geometry resolves through centralized
//! UI-style defaults. The themes themselves are `theme/`.

pub mod set;
pub mod shade;
pub mod theme;

pub use set::{BAccents, BColor, BPalette};
pub use shade::BShadeLadder;
pub use theme::{palette_for, palette_in};

/// The eleven themes Preferences offers. Each is one seed's ladder with
/// its own accent, radius scale, gradients, shadow and seam gap, in a dark
/// and a light mode (`BThemeMode`). Sphere, Graphite and Tokyo are phiber's
/// own; Onyx and Grey are black and grey (Grey's light mode the silver);
/// Apple is macOS's window greys with its blue; the last five are the
/// `phi` appkit's variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaletteName {
    Sphere,
    Graphite,
    Tokyo,
    Onyx,
    Grey,
    Apple,
    Swift,
    Barbie,
    Fenty,
    Cyberpunk,
    Chanel,
}

/// A theme's mode: the ladder read from black (dark) or from white (light)
/// — Blender's theme presets have a light and a dark; here every theme has
/// both, and the switch is one setting over all of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BThemeMode {
    Dark,
    Light,
}

impl BThemeMode {
    pub const ALL: [BThemeMode; 2] = [BThemeMode::Dark, BThemeMode::Light];
    pub const DEFAULT: BThemeMode = BThemeMode::Dark;

    pub fn label(self) -> &'static str {
        match self {
            BThemeMode::Dark => "Dark",
            BThemeMode::Light => "Light",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            BThemeMode::Dark => "dark",
            BThemeMode::Light => "light",
        }
    }
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.slug() == slug)
    }
}

impl BPaletteName {
    /// Declaration order — also the order the Preferences theme list shows.
    pub const ALL: &'static [BPaletteName] = &[
        BPaletteName::Sphere,
        BPaletteName::Graphite,
        BPaletteName::Tokyo,
        BPaletteName::Onyx,
        BPaletteName::Grey,
        BPaletteName::Apple,
        BPaletteName::Swift,
        BPaletteName::Barbie,
        BPaletteName::Fenty,
        BPaletteName::Cyberpunk,
        BPaletteName::Chanel,
    ];

    /// The palette applied when no theme has been chosen.
    pub const DEFAULT: BPaletteName = BPaletteName::Sphere;

    pub fn label(self) -> &'static str {
        match self {
            BPaletteName::Sphere => "Sphere",
            BPaletteName::Graphite => "Graphite",
            BPaletteName::Tokyo => "Tokyo",
            BPaletteName::Onyx => "Onyx",
            BPaletteName::Grey => "Grey",
            BPaletteName::Apple => "Apple",
            BPaletteName::Swift => "Swift",
            BPaletteName::Barbie => "Barbie",
            BPaletteName::Fenty => "Fenty",
            BPaletteName::Cyberpunk => "Cyberpunk",
            BPaletteName::Chanel => "Chanel",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            BPaletteName::Sphere => "Sphere",
            BPaletteName::Graphite => "Graphite",
            BPaletteName::Tokyo => "Tokyo",
            BPaletteName::Onyx => "Onyx",
            BPaletteName::Grey => "Grey",
            BPaletteName::Apple => "Apple",
            BPaletteName::Swift => "Swift",
            BPaletteName::Barbie => "Barbie",
            BPaletteName::Fenty => "Fenty",
            BPaletteName::Cyberpunk => "Cyberpunk",
            BPaletteName::Chanel => "Chanel",
        }
    }
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|name| name.slug() == slug)
    }
}

impl BPalette {
    /// Non-owning compatibility access to centralized radius defaults.
    pub fn radii(&self) -> &'static super::style::BRadiusScale {
        &super::style::ui_style_defaults(self.name).radii
    }

    /// Non-owning compatibility access to centralized effect defaults.
    pub fn effects(&self) -> &'static super::style::BEffects {
        &super::style::ui_style_defaults(self.name).effects
    }
}
