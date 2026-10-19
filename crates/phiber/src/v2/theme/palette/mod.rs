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

/// The sixteen themes Preferences offers — as many as it ever will
/// (`BPaletteName::MAX`). Each is one seed's ladder with its own accent,
/// radius scale, gradients, shadow and seam gap, in a dark and a light mode
/// (`BThemeMode`). Sphere, Graphite and Tokyo are phiber's own; Moss, Ember
/// and Plum its three warm ones; Onyx and Grey are black and grey (Grey's
/// light mode the silver); Silver is the show's blue-grey with its dark blue;
/// Apple is macOS's window greys with its blue, and Apple Dark its dark grey
/// on both sides; the last five are the `phi` appkit's variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaletteName {
    Sphere,
    Graphite,
    Tokyo,
    Moss,
    Ember,
    Plum,
    Onyx,
    Grey,
    Silver,
    Apple,
    AppleDark,
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

    /// The mode in force until one is chosen.
    ///
    /// Light. Sphere's dark ladder puts shade 10 — the viewport canvas — at
    /// the seed itself, `#3B4657`, so a fresh window opened on a near-black
    /// blue-grey and the only way out was Preferences → Theme → Light, which
    /// is not something a first run should have to go and find. The palette is
    /// unchanged: the same blue-grey, read from white instead of from black.
    pub const DEFAULT: BThemeMode = BThemeMode::Light;

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
        BPaletteName::Moss,
        BPaletteName::Ember,
        BPaletteName::Plum,
        BPaletteName::Onyx,
        BPaletteName::Grey,
        BPaletteName::Silver,
        BPaletteName::Apple,
        BPaletteName::AppleDark,
        BPaletteName::Swift,
        BPaletteName::Barbie,
        BPaletteName::Fenty,
        BPaletteName::Cyberpunk,
        BPaletteName::Chanel,
    ];

    /// The most themes Preferences may offer: a list longer than this is
    /// a list nobody reads to the end.
    pub const MAX: usize = 16;

    /// The palette applied when no theme has been chosen.
    ///
    /// Silver, because it is the grey the manual's figures are drawn in
    /// (`manual::wave::BWave::SILVER`): a figure and the screen it was
    /// taken from are then the same colour, and the cascade that makes
    /// phiber's wave series reads the same on the page as in the editor.
    pub const DEFAULT: BPaletteName = BPaletteName::Silver;

    pub fn label(self) -> &'static str {
        match self {
            BPaletteName::Sphere => "Sphere",
            BPaletteName::Graphite => "Graphite",
            BPaletteName::Tokyo => "Tokyo",
            BPaletteName::Moss => "Moss",
            BPaletteName::Ember => "Ember",
            BPaletteName::Plum => "Plum",
            BPaletteName::Onyx => "Onyx",
            BPaletteName::Grey => "Grey",
            BPaletteName::Silver => "Silver",
            BPaletteName::Apple => "Apple",
            BPaletteName::AppleDark => "Apple Dark",
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
            BPaletteName::Moss => "Moss",
            BPaletteName::Ember => "Ember",
            BPaletteName::Plum => "Plum",
            BPaletteName::Onyx => "Onyx",
            BPaletteName::Grey => "Grey",
            BPaletteName::Silver => "Silver",
            BPaletteName::Apple => "Apple",
            BPaletteName::AppleDark => "AppleDark",
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
