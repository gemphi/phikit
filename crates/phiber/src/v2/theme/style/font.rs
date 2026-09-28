//! The interface's typeface — one of eight families, chosen in Preferences
//! (*Typography ▸ Family*) and applied to every letter the window draws.
//!
//! Four are text faces and four are code faces: the system's own face
//! (Segoe UI on Windows, Helvetica Neue on a Mac), the two macOS faces
//! Blender users know from its menus, Inter, and the four monospace faces
//! VS Code ships with or recommends. A face the machine does not have falls
//! back down its own stack, then to the system's (`Font.fallbacks`), so a
//! choice never leaves the window blank.

/// One of the eight families Preferences offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BFontFamily {
    /// The platform's interface face: Segoe UI, Helvetica Neue, or whatever
    /// the system names as `sans-serif`.
    System,
    /// macOS's interface face before San Francisco; VS Code's Mac fallback.
    HelveticaNeue,
    /// Windows' interface face.
    SegoeUi,
    /// The open interface face, sized for screens.
    Inter,
    /// macOS's terminal face; VS Code's default on a Mac.
    Menlo,
    /// Windows' code face; VS Code's default there.
    Consolas,
    /// Windows Terminal's face; VS Code's recommended ligature face.
    CascadiaCode,
    /// JetBrains' code face, VS Code's most-installed third-party one.
    JetBrainsMono,
}

impl BFontFamily {
    /// Declaration order — the order Preferences lists them.
    pub const ALL: [BFontFamily; 8] = [
        BFontFamily::System,
        BFontFamily::HelveticaNeue,
        BFontFamily::SegoeUi,
        BFontFamily::Inter,
        BFontFamily::Menlo,
        BFontFamily::Consolas,
        BFontFamily::CascadiaCode,
        BFontFamily::JetBrainsMono,
    ];
    pub const DEFAULT: BFontFamily = BFontFamily::System;

    pub const fn label(self) -> &'static str {
        match self {
            BFontFamily::System => "System",
            BFontFamily::HelveticaNeue => "Helvetica Neue",
            BFontFamily::SegoeUi => "Segoe UI",
            BFontFamily::Inter => "Inter",
            BFontFamily::Menlo => "Menlo",
            BFontFamily::Consolas => "Consolas",
            BFontFamily::CascadiaCode => "Cascadia Code",
            BFontFamily::JetBrainsMono => "JetBrains Mono",
        }
    }

    /// The value `ctx.typography.family` holds.
    pub const fn slug(self) -> &'static str {
        match self {
            BFontFamily::System => "system",
            BFontFamily::HelveticaNeue => "helvetica_neue",
            BFontFamily::SegoeUi => "segoe_ui",
            BFontFamily::Inter => "inter",
            BFontFamily::Menlo => "menlo",
            BFontFamily::Consolas => "consolas",
            BFontFamily::CascadiaCode => "cascadia_code",
            BFontFamily::JetBrainsMono => "jetbrains_mono",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|family| family.slug() == slug)
    }

    /// The family name the text system is asked for.
    pub fn family(self) -> &'static str {
        match self {
            BFontFamily::System => "sans-serif",
            BFontFamily::HelveticaNeue => "Helvetica Neue",
            BFontFamily::SegoeUi => "Segoe UI",
            BFontFamily::Inter => "Inter",
            BFontFamily::Menlo => "Menlo",
            BFontFamily::Consolas => "Consolas",
            BFontFamily::CascadiaCode => "Cascadia Code",
            BFontFamily::JetBrainsMono => "JetBrains Mono",
        }
    }

    /// The faces tried when `family` is not installed, nearest first, so a
    /// Mac face on Windows lands on its Windows counterpart and a code face
    /// stays a code face.
    pub fn fallbacks(self) -> &'static [&'static str] {
        match self {
            BFontFamily::System => &["Segoe UI", "Helvetica Neue", "Inter"],
            BFontFamily::HelveticaNeue => &["Helvetica", "Segoe UI", "Inter", "sans-serif"],
            BFontFamily::SegoeUi => &["Helvetica Neue", "Inter", "sans-serif"],
            BFontFamily::Inter => &["Segoe UI", "Helvetica Neue", "sans-serif"],
            BFontFamily::Menlo => &["Consolas", "Cascadia Code", "DejaVu Sans Mono", "monospace"],
            BFontFamily::Consolas => &["Menlo", "Cascadia Code", "DejaVu Sans Mono", "monospace"],
            BFontFamily::CascadiaCode => &["Consolas", "Menlo", "JetBrains Mono", "monospace"],
            BFontFamily::JetBrainsMono => &["Cascadia Code", "Consolas", "Menlo", "monospace"],
        }
    }

    /// Whether the face is a code face — every glyph the same width.
    pub fn monospace(self) -> bool {
        match self {
            BFontFamily::System | BFontFamily::HelveticaNeue | BFontFamily::SegoeUi | BFontFamily::Inter => false,
            BFontFamily::Menlo | BFontFamily::Consolas | BFontFamily::CascadiaCode | BFontFamily::JetBrainsMono => true,
        }
    }
}
