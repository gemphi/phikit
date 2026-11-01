//! Apple's NSAppearance system — color scheme, vibrancy, visual effects.
//!
//! Maps directly to Apple's layered appearance architecture:
//!   NSAppearance            → AppearanceName (color scheme)
//!   NSVisualEffectMaterial  → NSVisualEffectMaterial (surface context)
//!   NSVisualEffectBlendingMode → BlendingMode
//!   NSVisualEffectState     → AppearanceState
//!   NSBackgroundStyle       → BackgroundStyle (cell/content rendering hint)
//!   NSControlSize           → ControlSize (mini/small/regular/large)

use super::registry;

// ── NSAppearanceName ─────────────────────────────────────────────────────────
// Apple: NSAppearanceNameAqua, DarkAqua, VibrantDark, VibrantLight,
//        HighContrastAqua, HighContrastDarkAqua, etc.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceName {
    Aqua,                     // Light (macOS default)
    DarkAqua,                 // Dark
    VibrantLight,             // Translucent light (sidebar vibrancy)
    VibrantDark,              // Translucent dark
    HighContrastAqua,         // Accessibility: high contrast light
    HighContrastDarkAqua,     // Accessibility: high contrast dark
    HighContrastVibrantLight, // Accessibility + vibrancy
    HighContrastVibrantDark,  // Accessibility + vibrancy dark
}

impl AppearanceName {
    /// Returns true if this appearance is a dark variant.
    pub fn is_dark(self) -> bool {
        matches!(
            self,
            Self::DarkAqua
                | Self::VibrantDark
                | Self::HighContrastDarkAqua
                | Self::HighContrastVibrantDark
        )
    }

    /// Returns true if this appearance uses vibrancy (translucent materials).
    pub fn is_vibrant(self) -> bool {
        matches!(
            self,
            Self::VibrantLight
                | Self::VibrantDark
                | Self::HighContrastVibrantLight
                | Self::HighContrastVibrantDark
        )
    }

    /// Returns true if this is a high-contrast accessibility variant.
    pub fn is_high_contrast(self) -> bool {
        matches!(
            self,
            Self::HighContrastAqua
                | Self::HighContrastDarkAqua
                | Self::HighContrastVibrantLight
                | Self::HighContrastVibrantDark
        )
    }

    /// The best match for the current mode from the registry.
    pub fn from_mode(mode: super::AppearanceMode) -> Self {
        match mode {
            super::AppearanceMode::Light => Self::Aqua,
            super::AppearanceMode::Dark => Self::DarkAqua,
            super::AppearanceMode::Ombre => Self::DarkAqua, // Ombre is our extension, maps to dark
        }
    }
}

// ── NSAppearance ─────────────────────────────────────────────────────────────
// Apple's NSAppearance is a lightweight object that carries a name.
// Views inherit their parent's appearance unless explicitly overridden.
// Dynamic colors (NSColor.labelColor, etc.) resolve against the
// effective appearance of the view they're drawn in.

/// Mirrors Apple's `NSAppearance` — a named visual style.
///
/// Usage:
/// ```rust,ignore
/// // Set a view's appearance (overrides inheritance from parent)
/// view.set_appearance(Some(NSAppearance::named(AppearanceName::DarkAqua)));
///
/// // Query the resolved appearance (walks up the view tree)
/// let effective = view.effective_appearance();
/// if effective.name().is_dark() { /* dark mode rendering */ }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NSAppearance {
    name: AppearanceName,
}

impl NSAppearance {
    /// Create an appearance with the given name.
    pub fn named(name: AppearanceName) -> Self {
        Self { name }
    }

    /// The current system appearance (reads from the theme registry).
    pub fn current_drawing() -> Self {
        let mode = registry::get_registry().lock().unwrap().current_mode_enum();
        Self {
            name: AppearanceName::from_mode(mode),
        }
    }

    /// The appearance name.
    pub fn name(&self) -> AppearanceName {
        self.name
    }

    /// Convenience: is this a dark appearance?
    pub fn is_dark(&self) -> bool {
        self.name.is_dark()
    }
}

/// Simplified mode for typical use. Maps to AppearanceName internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceMode {
    Light,
    Dark,
    Ombre, // Gradient blend (our extension — not Apple)
}
pub type ThemeMode = AppearanceMode;

// ── NSVisualEffectMaterial ────────────────────────────────────────────────────
// Apple: context-dependent surface treatment.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NSVisualEffectMaterial {
    Titlebar,              // 3
    Selection,             // 4
    Menu,                  // 5
    Popover,               // 6
    Sidebar,               // 7
    HeaderView,            // 10
    Sheet,                 // 11
    WindowBackground,      // 12
    HUDWindow,             // 13
    FullScreenUI,          // 15
    ToolTip,               // 17
    ContentBackground,     // 18
    UnderWindowBackground, // 21
    UnderPageBackground,   // 22
}

// ── NSVisualEffectBlendingMode ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendingMode {
    BehindWindow, // Blend with desktop / other windows
    WithinWindow, // Blend with content behind this view in same window
    Opaque,       // Our extension — no transparency
}

// ── NSVisualEffectState ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceState {
    FollowsWindowActiveState,
    Active,
    Inactive,
}

// ── NSBackgroundStyle (from NSCell.h) ────────────────────────────────────────
// Tells a cell/view what kind of surface it's drawn onto.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackgroundStyle {
    Normal,     // Default — light or standard background
    Emphasized, // Selected row, accent background — use contrasting text
    Raised,     // Embossed, textured button — engraved look
    Lowered,    // Recessed surface
}

// ── NSControlSize (from NSCell.h) ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlSize {
    Mini,       // 2 — smallest (macOS 10.0+)
    Small,      // 1
    Regular,    // 0 — default
    Large,      // 3 — (macOS 11.0+)
    ExtraLarge, // 4 — (macOS 26.0+)
}

// ── allowsVibrancy ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vibrancy {
    Off,
    On,
}
