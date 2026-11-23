//! Composable style layers — our extensions on top of Apple's appearance system.
//!
//! Each layer has 8 options. They compose independently:
//!   Variant        — accent palette, identity colors (apple, tesla, barbie…)
//!   Typography     — font family stack
//!   RadiusStyle    — corner radius preset
//!   Material       — surface treatment (glass, metal, onyx…)
//!   ColorGradient  — gradient/tone treatment
//!   VisualEffects  — shadows, animation, smoothness
//!   LayoutDensity  — spacing, padding

// ── Variant (9) — identity: accent palette, semantic colors ──────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    Apple,     // 0 — macOS Sonoma blue
    Windows,   // 1 — Fluent blue
    Tesla,     // 2 — Tesla red, deep black
    Swift,     // 3 — Swift orange
    Barbie,    // 4 — Hot pink
    Fenty,     // 5 — Warm bronze/terracotta
    Cyberpunk, // 6 — Neon cyan/magenta/yellow
    Chanel,    // 7 — Haute Couture SS26 mushroom forest
    Custom,    // 8 — User-defined
}
pub type ThemeVariant = Variant;

// ── Typography (8) — font family, sizes, weights ─────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Typography {
    System,      // 0 — SF Pro / Segoe UI / system default
    Mono,        // 1 — JetBrains Mono / Cascadia
    Serif,       // 2 — New York / Georgia
    Rounded,     // 3 — SF Rounded / Nunito
    Display,     // 4 — SF Display / Playfair
    Condensed,   // 5 — SF Condensed / Barlow
    Handwritten, // 6 — Caveat / Comic Neue
    Custom,      // 7 — User-defined
}

// ── RadiusStyle (8) — corner radius preset ───────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadiusStyle {
    Razor,      // 0 — 0px, sharp
    Tight,      // 1 — 2px
    Standard,   // 2 — 6px (Apple default)
    Soft,       // 3 — 12px
    Round,      // 4 — 20px
    Pill,       // 5 — Full capsule
    Kinky,      // 6 — Asymmetric corners
    SeeThrough, // 7 — No visible container
}

// ── Material (8) — surface treatment ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Material {
    Standard,    // 0 — Opaque solid
    Glass,       // 1 — Frosted glassmorphism
    Metal,       // 2 — Brushed metallic
    Onyx,        // 3 — Pure black polish
    Lace,        // 4 — Delicate, visible borders
    Wetland,     // 5 — Organic, earthy overlay
    Platinum,    // 6 — Premium metallic sheen
    Holographic, // 7 — Iridescent / gradient shift
}
pub type ThemeMaterial = Material;

// ── ColorGradient (8) — tone/gradient treatment ──────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorGradient {
    Flat,    // 0 — Solid colors, no gradients
    Subtle,  // 1 — Very slight depth gradients
    Warm,    // 2 — Warm-shifted gradients
    Cool,    // 3 — Cool-shifted gradients
    Vibrant, // 4 — Saturated, bold gradients
    Pastel,  // 5 — Muted, desaturated
    Neon,    // 6 — Bright, electric
    Duotone, // 7 — Two-tone palette
}

// ── VisualEffects (8) — shadows, animation, smoothness ───────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualEffects {
    None,      // 0 — Flat, no effects
    Subtle,    // 1 — Minimal shadows
    Standard,  // 2 — Apple-level shadows + hover
    Elevated,  // 3 — Strong depth, layered shadows
    Animated,  // 4 — Micro-animations on hover/click
    Blur,      // 5 — Background blur behind panels
    Glow,      // 6 — Accent glow on active elements
    Cinematic, // 7 — Full animation suite + parallax
}

// ── LayoutDensity (8) — spacing, padding ─────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayoutDensity {
    Compact,   // 0 — Dense, minimal padding (IDE style)
    Standard,  // 1 — Apple HIG defaults
    Relaxed,   // 2 — Extra breathing room
    Spacious,  // 3 — Large gaps, touch-friendly
    Editorial, // 4 — Magazine-like whitespace
    Grid,      // 5 — Strict grid alignment
    Fluid,     // 6 — Responsive, percentage-based
    Custom,    // 7 — User-defined
}


