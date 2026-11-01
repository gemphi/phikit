//! Layout constants — single source of truth for all panel geometry.
//!
//! RULE: No bare numeric literals in preset definitions.
//! Every value resolves through a named constant or theme token.

// ═══════════════════════════════════════════════════════════════════════
// TOOLBAR (top zone)
// ═══════════════════════════════════════════════════════════════════════

pub const TOOLBAR_MIN_HEIGHT: f32 = 38.0;
pub const TOOLBAR_PREFERRED_HEIGHT: f32 = 44.0;
pub const TOOLBAR_MAX_HEIGHT: f32 = 52.0;

// ═══════════════════════════════════════════════════════════════════════
// SIDEBAR / HIERARCHY
// ═══════════════════════════════════════════════════════════════════════

pub fn sidebar_min() -> f32 {
    crate::theme::ThemeToken::SIDEBAR_WIDTH() * 0.625
} // 200

pub fn sidebar_preferred() -> f32 {
    crate::theme::ThemeToken::SIDEBAR_WIDTH()
} // 320

pub fn sidebar_max() -> f32 {
    crate::theme::ThemeToken::SIDEBAR_WIDTH() * 1.4
} // 450

// ═══════════════════════════════════════════════════════════════════════
// INSPECTOR
// ═══════════════════════════════════════════════════════════════════════

pub fn inspector_min() -> f32 {
    sidebar_min() + 50.0
} // 250

pub fn inspector_preferred() -> f32 {
    sidebar_preferred()
} // 320

pub fn inspector_max() -> f32 {
    sidebar_max()
} // 450

// ═══════════════════════════════════════════════════════════════════════
// VIEWPORT
// ═══════════════════════════════════════════════════════════════════════

pub fn viewport_min() -> f32 {
    300.0
}

pub fn viewport_preferred() -> f32 {
    800.0
}

// ═══════════════════════════════════════════════════════════════════════
// BOTTOM PANEL (Timeline, etc.)
// ═══════════════════════════════════════════════════════════════════════

pub fn bottom_min() -> f32 {
    120.0
}

pub fn bottom_preferred() -> f32 {
    200.0
}

pub fn bottom_max() -> f32 {
    400.0
}

// ═══════════════════════════════════════════════════════════════════════
// TOOLS SIDEBAR (icon strip)
// ═══════════════════════════════════════════════════════════════════════

pub fn tools_min() -> f32 {
    40.0
}

pub fn tools_preferred() -> f32 {
    44.0
}

pub fn tools_max() -> f32 {
    60.0
}

// ═══════════════════════════════════════════════════════════════════════
// FLOATING PANEL
// ═══════════════════════════════════════════════════════════════════════

pub fn floating_min() -> f32 {
    sidebar_min()
}

pub fn floating_preferred() -> f32 {
    280.0
}

pub fn floating_max() -> f32 {
    400.0
}

// ═══════════════════════════════════════════════════════════════════════
// EDITORIAL SIDEBAR (narrower)
// ═══════════════════════════════════════════════════════════════════════

pub fn editorial_sidebar() -> f32 {
    220.0
}

// ═══════════════════════════════════════════════════════════════════════
// COMPACT SCALE FACTOR
// ═══════════════════════════════════════════════════════════════════════

pub const COMPACT_SCALE: f32 = 0.8;
