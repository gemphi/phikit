//! Runtime layout metrics — token-driven, no hardcoding.
//!
//! Each function reads from the active theme token at call time.

use super::layout as defaults;

/// Sidebar width — reads from theme token.
pub fn sidebar_width() -> f32 {
    defaults::sidebar_preferred()
}

/// Title bar height — reads from theme token.
pub fn title_bar_height() -> f32 {
    crate::theme::ThemeToken::TITLE_BAR_HEIGHT()
}

/// Menu bar height — reads from theme token.
pub fn menu_bar_height() -> f32 {
    crate::theme::ThemeToken::MENU_BAR_HEIGHT()
}

/// Tab bar height — reads from theme token.
pub fn tab_bar_height() -> f32 {
    crate::theme::ThemeToken::TAB_HEIGHT()
}

/// Panel gap — spacing between docked panels.
pub fn panel_gap() -> f32 {
    crate::theme::ThemeToken::SPACING_MD()
}

/// Panel corner radius — from theme token.
pub fn panel_radius() -> f32 {
    crate::theme::ThemeToken::RADIUS_PANEL()
}

/// Control height — standard button/field height.
pub fn control_height() -> f32 {
    crate::theme::ThemeToken::CONTROL_HEIGHT()
}

/// Toolbar height — title bar area.
pub fn toolbar_height() -> f32 {
    title_bar_height()
}
