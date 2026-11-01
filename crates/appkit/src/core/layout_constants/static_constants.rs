//! Static layout constants.
//!
//! Named `pub const` values for window, toolbar, panel, gizmo, grid,
//! menu, overlay, and preferences geometry.

// ── Window & Toolbar ─────────────────────────────────────────────────────────

pub const WINDOW_W: f32 = 1600.0;
pub const WINDOW_H: f32 = 850.0; // Reduced from 900 to account for Windows taskbar
pub const TOOLBAR_H: f32 = 44.0;
pub const SIDEBAR_W: f32 = 320.0;
pub const PANEL_INSET_H: f32 = 12.0;
pub const PANEL_INSET_V: f32 = 8.0;
pub const SECTION_SPACING: f32 = 8.0;
pub const ITEM_SPACING: f32 = 4.0;

// ── Gizmo & Grid ────────────────────────────────────────────────────────────

pub const GIZMO_AXIS_LENGTH: f32 = 50.0;
pub const GIZMO_AXIS_THICKNESS: f32 = 3.0;
pub const GIZMO_Z_OFFSET: f32 = 20.0;
pub const GRID_LINE_WIDTH: f32 = 1.0;

// ── Menu ─────────────────────────────────────────────────────────────────────

pub const MENU_BTN_W: f32 = 60.0;
pub const MENU_BTN_H: f32 = 30.0;
pub const MENU_DROPDOWN_W: f32 = 180.0;
pub const MENU_ITEM_H: f32 = 28.0;
pub const MENU_ITEM_PAD: f32 = 6.0;

// ── Overlay ──────────────────────────────────────────────────────────────────

pub const OVERLAY_X: f32 = 100.0;
pub const OVERLAY_Y: f32 = 80.0;
pub const OVERLAY_W: f32 = 280.0;
pub const OVERLAY_CONTENT_W: f32 = 256.0;
pub const OVERLAY_SEP_H: f32 = 1.0;

// ── Preferences ──────────────────────────────────────────────────────────────

pub const PREFS_WINDOW_W: f32 = 560.0;
pub const PREFS_WINDOW_H: f32 = 400.0;
pub const PREFS_SWITCH_W: f32 = 50.0;
pub const PREFS_SWITCH_H: f32 = 24.0;
pub const PREFS_SWITCH_OFFSET_X: f32 = 70.0;
pub const PREFS_LABEL_FONT_SIZE: f32 = 12.0;
pub const PREFS_ROW_H: f32 = 20.0;
pub const PREFS_LABEL_OFFSET_Y: f32 = 4.0;
pub const PREFS_TAB_HEIGHT: f32 = 28.0;
pub const PREFS_ROW_CULLING_Y: f32 = 50.0;
pub const PREFS_ROW_AO_Y: f32 = 90.0;
pub const PREFS_ROW_SANDBOX_Y: f32 = 130.0;
pub const PREFS_ROW_STATS_Y: f32 = 170.0;

// ── NSLayoutConstants ────────────────────────────────────────────────────────

/// Framework Layout Constants
///
/// Provides standard layout metrics following AppKit conventions.
/// Applications can use these constants or define their own.
///
/// This struct consolidates layout constants used throughout the framework.
pub struct NSLayoutConstants;

impl NSLayoutConstants {
    pub const SIDEBAR_WIDTH: f32 = 320.0;
    pub const MENU_BAR_HEIGHT: f32 = 28.0;
    pub const TITLE_BAR_HEIGHT: f32 = 44.0;
    pub const HEADER_HEIGHT: f32 = Self::MENU_BAR_HEIGHT + Self::TITLE_BAR_HEIGHT;
    pub const FOOTER_HEIGHT: f32 = 258.0;
    pub const INSPECTOR_GUTTER: f32 = 100.0;
    pub const UI_SPACING: f32 = 8.0;
    pub const UI_MARGIN: f32 = 12.0;
    pub const WINDOW_WIDTH: f32 = 1600.0;
    pub const WINDOW_HEIGHT: f32 = 850.0; // Reduced from 900 to account for Windows taskbar (~50px)

    // Icon button standardisation — shared across all consumers
    /// Standard icon-only button frame (toolbar icons, filter, gear, etc.)
    pub const ICON_BUTTON_SIZE: f32 = 20.0;
    /// Compact icon button for tree row items (entity icons, folder icons)
    pub const ICON_BUTTON_INLINE: f32 = 16.0;
    /// Disclosure triangle size
    pub const DISCLOSURE_SIZE: f32 = 12.0;
    /// Uniform inset inside every icon button
    pub const ICON_BUTTON_INSET: f32 = 3.0;
    /// Icon render target for toolbar icons
    pub const TOOLBAR_ICON_SIZE: f32 = 18.0;

    // Scene Hierarchy layout constants
    pub const TREE_ITEM_HEIGHT: f32 = 26.0;
    pub const TREE_DISCLOSURE_SIZE: f32 = 18.0;
    pub const TREE_ICON_SIZE: f32 = 16.0;
    pub const TREE_LABEL_HEIGHT: f32 = 18.0;
    pub const TREE_ROW_PADDING_Y: f32 = 4.0;
    pub const TREE_ICON_PADDING_Y: f32 = 5.0;

    // Panel origins (default starting position)
    pub const PANEL_ORIGIN_X: f32 = 0.0;
    pub const PANEL_ORIGIN_Y: f32 = 0.0;

    // Margins
    pub const MARGIN: f32 = 10.0;
    pub const MARGIN_SMALL: f32 = 8.0;
    pub const MARGIN_XSMALL: f32 = 4.0;
    pub const MARGIN_LARGE: f32 = 12.0;
    pub const MARGIN_XLARGE: f32 = 16.0;
    pub const PANEL_PADDING: f32 = 8.0;
    pub const LABEL_HEIGHT: f32 = 20.0;

    // Buttons & Controls
    pub const BUTTON_HEIGHT: f32 = 28.0;
    pub const SEARCH_FIELD_HEIGHT: f32 = 28.0;
    pub const FIELD_ROW_HEIGHT: f32 = 32.0;

    // Toolbar
    pub const TOOLBAR_HEIGHT: f32 = 38.0;
    pub const MODE_SWITCHER_WIDTH: f32 = 280.0;
    pub const MODE_SWITCHER_HEIGHT: f32 = 24.0;
    pub const SIDEBAR_TOGGLE_OFFSET: f32 = 80.0;
    pub const LAYOUT_TOGGLE_SPACING: f32 = 30.0;

    // Traffic lights (window controls)
    pub const TRAFFIC_LIGHT_SIZE: f32 = 12.0;
    pub const TRAFFIC_LIGHT_SPACING: f32 = 8.0;
    pub const TRAFFIC_LIGHT_MARGIN_LEFT: f32 = 12.0;

    // Viewport constraints
    pub const VIEWPORT_MIN_HEIGHT: f32 = 200.0;
    pub const VIEWPORT_MIN_WIDTH: f32 = 300.0;
    pub const FOOTER_MIN_HEIGHT: f32 = 120.0;
    pub const FOOTER_MAX_HEIGHT: f32 = 400.0;

    // Navigator / secondary sidebar
    pub const NAVIGATOR_WIDTH: f32 = 260.0;

    // Additional layout standardizations
    pub const SIDEBAR_MAX_WIDTH: f32 = 450.0;
    pub const SEARCH_WIDTH_WIDE: f32 = 220.0;
    pub const TAB_BAR_HEIGHT: f32 = 24.0;
    pub const PROJECT_INSPECTOR_WIDTH: f32 = 320.0;
    pub const GRID_ICON_SPACING: f32 = 8.0;
    pub const GRID_LINE_WIDTH: f32 = 1.0;
    pub const SEPARATOR_THICKNESS: f32 = 1.0;

    // Font sizes
    pub const FONT_SIZE_LARGE: f32 = 15.0;
    pub const FONT_SIZE_HEADER: f32 = 12.0;
    pub const FONT_SIZE_LABEL: f32 = 13.0;
    pub const FONT_SIZE_SMALL: f32 = 11.0;
    pub const FONT_SIZE_TINY: f32 = 8.0;

    // Hierarchy Specific
    pub const HIERARCHY_TAB_WIDTH: f32 = 54.0;
    pub const HIERARCHY_TAB_HEIGHT: f32 = 22.0;
    pub const HIERARCHY_SEARCH_X: f32 = 64.0;

    // Property Inspector
    pub const INSPECTOR_LABEL_WIDTH: f32 = 120.0;
    pub const INSPECTOR_SWATCH_WIDTH: f32 = 40.0;
    pub const INSPECTOR_SWATCH_HEIGHT: f32 = 18.0;

    // Asset Library
    pub const ASSET_GRID_ITEM_SIZE: f32 = 64.0;
    pub const ASSET_GRID_LABEL_HEIGHT: f32 = 20.0;

    // Statistics
    pub const STAT_LINE_HEIGHT: f32 = 20.0;

    // Menu System
    pub const MENU_BAR_FONT_SIZE: f32 = 14.0;
    pub const MENU_BAR_CHAR_WIDTH: f32 = 7.5;
    pub const MENU_BAR_BUTTON_HEIGHT: f32 = 26.0;
    pub const MENU_BAR_BUTTON_RADIUS: f32 = 6.0;
    pub const MENU_BAR_PADDING_X: f32 = 12.0;
    pub const MENU_BAR_START_X: f32 = 80.0;
    pub const MENU_DROPDOWN_FONT_SIZE: f32 = 14.0;
    pub const MENU_DROPDOWN_ITEM_HEIGHT: f32 = 28.0;
    pub const MENU_DROPDOWN_PADDING_X: f32 = 12.0;
    pub const MENU_DROPDOWN_PADDING_Y: f32 = 4.0;
    pub const MENU_DROPDOWN_WIDTH: f32 = 220.0;
    pub const MENU_DROPDOWN_RADIUS: f32 = 8.0;
    pub const MENU_DROPDOWN_ITEM_RADIUS: f32 = 4.0;
    pub const MENU_DROPDOWN_ITEM_INSET: f32 = 4.0;
}
