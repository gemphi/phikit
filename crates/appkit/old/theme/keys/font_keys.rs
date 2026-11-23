// Appearance layer constants — use these instead of bare strings everywhere.

/// Mode layer — 3 options
pub mod mode {
    pub const DARK: &str = "dark";
    pub const LIGHT: &str = "light";
    pub const OMBRE: &str = "ombre";
    pub const ALL: [&str; 3] = [DARK, LIGHT, OMBRE];
}

/// Variant layer — 9 options
pub mod variant {
    pub const APPLE: &str = "apple";
    pub const WINDOWS: &str = "windows";
    pub const TESLA: &str = "tesla";
    pub const SWIFT: &str = "swift";
    pub const BARBIE: &str = "barbie";
    pub const FENTY: &str = "fenty";
    pub const CYBERPUNK: &str = "cyberpunk";
    pub const CHANEL: &str = "chanel";
    pub const CUSTOM: &str = "custom";
    pub const ALL: [&str; 9] = [
        APPLE, WINDOWS, TESLA, SWIFT, BARBIE, FENTY, CYBERPUNK, CHANEL, CUSTOM,
    ];
}

/// Material layer — 8 options
pub mod material {
    pub const STANDARD: &str = "standard";
    pub const GLASS: &str = "glass";
    pub const METAL: &str = "metal";
    pub const ONYX: &str = "onyx";
    pub const LACE: &str = "lace";
    pub const WETLAND: &str = "wetland";
    pub const PLATINUM: &str = "platinum";
    pub const HOLOGRAPHIC: &str = "holographic";
    pub const ALL: [&str; 8] = [
        STANDARD, GLASS, METAL, ONYX, LACE, WETLAND, PLATINUM, HOLOGRAPHIC,
    ];
}

/// Typography layer — 8 options
pub mod typography {
    pub const SYSTEM: &str = "system";
    pub const MONO: &str = "mono";
    pub const SERIF: &str = "serif";
    pub const ROUNDED: &str = "rounded";
    pub const DISPLAY: &str = "display";
    pub const CONDENSED: &str = "condensed";
    pub const HANDWRITTEN: &str = "handwritten";
    pub const CUSTOM: &str = "custom";
    pub const ALL: [&str; 8] = [
        SYSTEM, MONO, SERIF, ROUNDED, DISPLAY, CONDENSED, HANDWRITTEN, CUSTOM,
    ];
}

/// Radius layer — 8 options
pub mod radius {
    pub const RAZOR: &str = "razor"; // 0px
    pub const TIGHT: &str = "tight"; // 2px
    pub const STANDARD: &str = "standard"; // 6px
    pub const SOFT: &str = "soft"; // 12px
    pub const ROUND: &str = "round"; // 20px
    pub const PILL: &str = "pill"; // full (50%)
    pub const KINKY: &str = "kinky";
    pub const SEE_THROUGH: &str = "see_through";
    pub const ALL: [&str; 8] = [
        RAZOR, TIGHT, STANDARD, SOFT, ROUND, PILL, KINKY, SEE_THROUGH,
    ];
}

/// Accent color layer — from NSColor.controlAccentColor + systemColors
pub mod accent {
    pub const BLUE: &str = "blue";
    pub const PURPLE: &str = "purple";
    pub const PINK: &str = "pink";
    pub const RED: &str = "red";
    pub const ORANGE: &str = "orange";
    pub const YELLOW: &str = "yellow";
    pub const GREEN: &str = "green";
    pub const GRAPHITE: &str = "graphite";
    pub const ALL: [&str; 8] = [BLUE, PURPLE, PINK, RED, ORANGE, YELLOW, GREEN, GRAPHITE];
}

/// Tint / Vibrancy layer — from NSVisualEffectMaterial
pub mod tint {
    pub const NONE: &str = "none";
    pub const VIBRANT_LIGHT: &str = "vibrant_light";
    pub const VIBRANT_DARK: &str = "vibrant_dark";
    pub const SIDEBAR: &str = "sidebar";
    pub const HUD: &str = "hud";
    pub const SHEET: &str = "sheet";
    pub const POPOVER: &str = "popover";
    pub const UNDER_WINDOW: &str = "under_window";
    pub const ALL: [&str; 8] = [
        NONE, VIBRANT_LIGHT, VIBRANT_DARK, SIDEBAR, HUD, SHEET, POPOVER, UNDER_WINDOW,
    ];
}

/// Shadow intensity layer — from NSColor.shadowColor / highlightColor
pub mod shadows {
    pub const NONE: &str = "none";
    pub const SUBTLE: &str = "subtle";
    pub const STANDARD: &str = "standard";
    pub const ELEVATED: &str = "elevated";
    pub const DEEP: &str = "deep";
    pub const INNER_GLOW: &str = "inner_glow";
    pub const DROP: &str = "drop";
    pub const CINEMATIC: &str = "cinematic";
    pub const ALL: [&str; 8] = [
        NONE, SUBTLE, STANDARD, ELEVATED, DEEP, INNER_GLOW, DROP, CINEMATIC,
    ];
}

/// Color gradient layer — 8 options
pub mod gradient {
    pub const FLAT: &str = "flat";
    pub const SUBTLE: &str = "subtle";
    pub const WARM: &str = "warm";
    pub const COOL: &str = "cool";
    pub const VIBRANT: &str = "vibrant";
    pub const PASTEL: &str = "pastel";
    pub const NEON: &str = "neon";
    pub const DUOTONE: &str = "duotone";
    pub const ALL: [&str; 8] = [FLAT, SUBTLE, WARM, COOL, VIBRANT, PASTEL, NEON, DUOTONE];
}

/// Visual effects layer — 8 options
pub mod effects {
    pub const NONE: &str = "none";
    pub const SUBTLE: &str = "subtle";
    pub const STANDARD: &str = "standard";
    pub const ELEVATED: &str = "elevated";
    pub const ANIMATED: &str = "animated";
    pub const BLUR: &str = "blur";
    pub const GLOW: &str = "glow";
    pub const CINEMATIC: &str = "cinematic";
    pub const ALL: [&str; 8] = [
        NONE, SUBTLE, STANDARD, ELEVATED, ANIMATED, BLUR, GLOW, CINEMATIC,
    ];
}

/// Layout density layer — 8 options
pub mod density {
    pub const COMPACT: &str = "compact";
    pub const STANDARD: &str = "standard";
    pub const RELAXED: &str = "relaxed";
    pub const SPACIOUS: &str = "spacious";
    pub const EDITORIAL: &str = "editorial";
    pub const GRID: &str = "grid";
    pub const FLUID: &str = "fluid";
    pub const CUSTOM: &str = "custom";
    pub const ALL: [&str; 8] = [
        COMPACT, STANDARD, RELAXED, SPACIOUS, EDITORIAL, GRID, FLUID, CUSTOM,
    ];
}

/// Layout style presets — 8 options
pub mod layout_style {
    pub const COMPACT: &str = "compact";
    pub const MACOS: &str = "macos";
    pub const MODERN: &str = "modern";
    pub const KINKY: &str = "kinky";
    pub const CLASSIC: &str = "classic";
    pub const MINIMAL: &str = "minimal";
    pub const EDITORIAL: &str = "editorial";
    pub const CUSTOM: &str = "custom";
    pub const ALL: [&str; 8] = [
        COMPACT, MACOS, MODERN, KINKY, CLASSIC, MINIMAL, EDITORIAL, CUSTOM,
    ];
}
