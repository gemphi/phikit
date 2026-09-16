//! Color palettes per theme name. Every rendered color resolves through
//! `palette_for(active_theme)`; style geometry resolves through centralized
//! UI-style defaults.

pub mod set;

pub use set::{BColor, BPalette, palette_for};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaletteName {
    AppleDark,
    BlenderDark,
    TokyoNight,
}

impl BPaletteName {
    /// Declaration order — also the order the Preferences theme list shows.
    pub const ALL: &'static [BPaletteName] = &[
        BPaletteName::AppleDark,
        BPaletteName::BlenderDark,
        BPaletteName::TokyoNight,
    ];

    /// The palette applied when no theme has been chosen.
    pub const DEFAULT: BPaletteName = BPaletteName::AppleDark;

    pub fn label(self) -> &'static str {
        match self {
            BPaletteName::AppleDark => "Apple Dark",
            BPaletteName::BlenderDark => "Blender Dark",
            BPaletteName::TokyoNight => "Tokyo Night",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            BPaletteName::AppleDark => "AppleDark",
            BPaletteName::BlenderDark => "BlenderDark",
            BPaletteName::TokyoNight => "TokyoNight",
        }
    }
    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.slug() == s)
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
