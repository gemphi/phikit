//! ThemeToken — zero-ceremony access to themed layout values.
//!
//! Usage at call site:
//! ```rust,ignore
//! use rust_appkit::theme::ThemeToken as T;
//!
//! let toolbar_h = T::TITLE_BAR_HEIGHT;
//! let activity_w = T::ACTIVITY_W;
//! let panel_radius = T::RADIUS_PANEL;
//! ```
//!
//! Each associated constant-style function reads from the live registry.
//! No fallbacks, no string keys, no ceremony.

use super::{keys, registry};

/// Typed theme token accessor. Each method reads the current value from the registry.
pub struct ThemeToken;

#[allow(non_snake_case)]
impl ThemeToken {
    // ── Sizing ───────────────────────────────────────────────────────────────

    pub fn TITLE_BAR_HEIGHT() -> f32 { Self::get(keys::SIZE_TITLE_BAR_HEIGHT) }
    pub fn MENU_BAR_HEIGHT() -> f32 { Self::get(keys::SIZE_MENU_BAR_HEIGHT) }
    pub fn SIDEBAR_WIDTH() -> f32 { Self::get(keys::SIZE_SIDEBAR_WIDTH) }
    pub fn CONTROL_HEIGHT() -> f32 { Self::get(keys::SIZE_CONTROL_HEIGHT) }
    pub fn TAB_HEIGHT() -> f32 { Self::get(keys::SIZE_TAB_HEIGHT) }
    pub fn TAB_PAD_X() -> f32 { Self::get(keys::SIZE_TAB_PAD_X) }
    pub fn FONT_UI() -> f32 { Self::get(keys::SIZE_FONT_UI) }
    pub fn FONT_SMALL() -> f32 { Self::get(keys::SIZE_FONT_SMALL) }
    pub fn FONT_TITLE() -> f32 { Self::get(keys::SIZE_FONT_TITLE) }
    pub fn FONT_HEADER() -> f32 { Self::get(keys::SIZE_FONT_HEADER) }
    pub fn FONT_CAPTION() -> f32 { Self::get(keys::SIZE_FONT_CAPTION) }
    pub fn CHAR_WIDTH() -> f32 { Self::get(keys::SIZE_CHAR_WIDTH) }

    // ── Panel header / toolbar ───────────────────────────────────────────────

    pub fn PANEL_HEADER_H() -> f32 { Self::get(keys::SIZE_PANEL_HEADER_H) }
    pub fn PANEL_TOOLBAR_H() -> f32 { Self::get(keys::SIZE_PANEL_TOOLBAR_H) }
    pub fn HEADER_TAB_H() -> f32 { Self::get(keys::SIZE_HEADER_TAB_H) }
    pub fn HEADER_TAB_PAD() -> f32 { Self::get(keys::SIZE_HEADER_TAB_PAD) }
    pub fn HEADER_TAB_GAP() -> f32 { Self::get(keys::SIZE_HEADER_TAB_GAP) }

    // ── Content rows ─────────────────────────────────────────────────────────

    pub fn ROW_H() -> f32 { Self::get(keys::SIZE_ROW_H) }
    pub fn ROW_GAP() -> f32 { Self::get(keys::SIZE_ROW_GAP) }
    pub fn ROW_INDENT() -> f32 { Self::get(keys::SIZE_ROW_INDENT) }
    pub fn CONTENT_PAD() -> f32 { Self::get(keys::SIZE_CONTENT_PAD) }

    // ── Search field ─────────────────────────────────────────────────────────

    pub fn SEARCH_H() -> f32 { Self::get(keys::SIZE_SEARCH_H) }
    pub fn SEARCH_W() -> f32 { Self::get(keys::SIZE_SEARCH_W) }
    pub fn SEARCH_MARGIN() -> f32 { Self::get(keys::SIZE_SEARCH_MARGIN) }

    // ── Gizmo ────────────────────────────────────────────────────────────────

    pub fn GIZMO() -> f32 { Self::get(keys::SIZE_GIZMO) }
    pub fn GIZMO_GAP() -> f32 { Self::get(keys::SIZE_GIZMO_GAP) }
    pub fn GIZMO_MARGIN() -> f32 { Self::get(keys::SIZE_GIZMO_MARGIN) }
    pub fn ORIENT_GIZMO() -> f32 { Self::get(keys::SIZE_ORIENT_GIZMO) }
    pub fn ORIENT_MARGIN() -> f32 { Self::get(keys::SIZE_ORIENT_MARGIN) }

    // ── Grid ─────────────────────────────────────────────────────────────────

    pub fn GRID_SPACING() -> f32 { Self::get(keys::SIZE_GRID_SPACING) }
    pub fn GRID_THICKNESS() -> f32 { Self::get(keys::SIZE_GRID_THICKNESS) }
    pub fn CROSSHAIR_LEN() -> f32 { Self::get(keys::SIZE_CROSSHAIR_LEN) }
    pub fn CROSSHAIR_W() -> f32 { Self::get(keys::SIZE_CROSSHAIR_W) }

    // ── Inspector / sections ─────────────────────────────────────────────────

    pub fn SECTION_PAD() -> f32 { Self::get(keys::SIZE_SECTION_PAD) }
    pub fn LABEL_W() -> f32 { Self::get(keys::SIZE_LABEL_W) }

    // ── Preferences ──────────────────────────────────────────────────────────

    pub fn PREF_SIDEBAR_W() -> f32 { Self::get(keys::SIZE_PREF_SIDEBAR_W) }
    pub fn PREF_TAB_H() -> f32 { Self::get(keys::SIZE_PREF_TAB_H) }
    pub fn PREF_ROW_H() -> f32 { Self::get(keys::SIZE_PREF_ROW_H) }
    pub fn PREF_CONTROL_W() -> f32 { Self::get(keys::SIZE_PREF_CONTROL_W) }
    pub fn PREF_CONTROL_H() -> f32 { Self::get(keys::SIZE_PREF_CONTROL_H) }

    // ── Activity bar ─────────────────────────────────────────────────────────

    pub fn ACTIVITY_W() -> f32 { Self::get(keys::SIZE_ACTIVITY_W) }
    pub fn ACTIVITY_ICON() -> f32 { Self::get(keys::SIZE_ACTIVITY_ICON) }
    pub fn ACTIVITY_GAP() -> f32 { Self::get(keys::SIZE_ACTIVITY_GAP) }

    // ── Button ───────────────────────────────────────────────────────────────

    pub fn BUTTON_H() -> f32 { Self::get(keys::SIZE_BUTTON_H) }
    pub fn BUTTON_GAP() -> f32 { Self::get(keys::SIZE_BUTTON_GAP) }

    // ── Spacing ──────────────────────────────────────────────────────────────

    pub fn SPACING_XS() -> f32 { Self::get(keys::SPACING_XS) }
    pub fn SPACING_SM() -> f32 { Self::get(keys::SPACING_SM) }
    pub fn SPACING_MD() -> f32 { Self::get(keys::SPACING_MD) }
    pub fn SPACING_LG() -> f32 { Self::get(keys::SPACING_LG) }

    // ── Radii ────────────────────────────────────────────────────────────────

    pub fn RADIUS_BUTTON() -> f32 { Self::get(keys::RADIUS_BUTTON) }
    pub fn RADIUS_PANEL() -> f32 { Self::get(keys::RADIUS_PANEL) }
    pub fn RADIUS_CONTROL() -> f32 { Self::get(keys::RADIUS_CONTROL) }
    pub fn RADIUS_NODE() -> f32 { Self::get(keys::RADIUS_NODE) }
    pub fn RADIUS_TAB() -> f32 { Self::get(keys::RADIUS_TAB) }
    pub fn RADIUS_DROPDOWN() -> f32 { Self::get(keys::RADIUS_DROPDOWN) }
    pub fn RADIUS_TOOLTIP() -> f32 { Self::get(keys::RADIUS_TOOLTIP) }
    pub fn RADIUS_HEADER_TAB() -> f32 { Self::get(keys::RADIUS_HEADER_TAB) }
    pub fn RADIUS_ROW() -> f32 { Self::get(keys::RADIUS_ROW) }
    pub fn RADIUS_GIZMO() -> f32 { Self::get(keys::RADIUS_GIZMO) }
    pub fn RADIUS_SECTION() -> f32 { Self::get(keys::RADIUS_SECTION) }
    pub fn RADIUS_ACTIVITY_ICON() -> f32 { Self::get(keys::RADIUS_ACTIVITY_ICON) }

    // ── Glass ────────────────────────────────────────────────────────────────

    pub fn GLASS_OPACITY() -> f32 { Self::get(keys::GLASS_OPACITY) }

    // ── Icons ────────────────────────────────────────────────────────────────

    pub fn ICON_SM() -> f32 { Self::get(keys::SIZE_ICON_SM) }
    pub fn ICON_MD() -> f32 { Self::get(keys::SIZE_ICON_MD) }
    pub fn ICON_LG() -> f32 { Self::get(keys::SIZE_ICON_LG) }
    pub fn ICON_XL() -> f32 { Self::get(keys::SIZE_ICON_XL) }

    // ── Internal ─────────────────────────────────────────────────────────────

    #[inline]
    fn get(key: &str) -> f32 {
        registry::get_geometry(key)
    }
}
