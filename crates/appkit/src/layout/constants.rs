//! # Layout Constants
//!
//! Consolidated heights, widths, and size specifications for all UI components.
//! All values follow the 8pt grid system.

use super::spacing;

// ═══════════════════════════════════════════════════════════════════════
// Window Sizes
// ═══════════════════════════════════════════════════════════════════════

/// Default editor window width
pub const WINDOW_WIDTH: f32 = 1280.0;

/// Default editor window height
pub const WINDOW_HEIGHT: f32 = 800.0;

/// Minimum window width
pub const WINDOW_MIN_WIDTH: f32 = 800.0;

/// Minimum window height
pub const WINDOW_MIN_HEIGHT: f32 = 600.0;

/// Welcome window width
pub const WELCOME_WIDTH: f32 = 460.0;

/// Welcome window height
pub const WELCOME_HEIGHT: f32 = 480.0;

// ═══════════════════════════════════════════════════════════════════════
// Panel Heights
// ═══════════════════════════════════════════════════════════════════════

/// Toolbar height (title bar area)
pub const TOOLBAR_HEIGHT: f32 = 44.0;

/// Tab bar height
pub const TAB_BAR_HEIGHT: f32 = 44.0;

/// Status bar height
pub const STATUS_BAR_HEIGHT: f32 = 24.0;

/// Activity bar width (vertical sidebar)
pub const ACTIVITY_BAR_WIDTH: f32 = 48.0;

/// Tools sidebar width
pub const TOOLS_SIDEBAR_WIDTH: f32 = 44.0;

// ═══════════════════════════════════════════════════════════════════════
// Panel Widths (Preferred)
// ═══════════════════════════════════════════════════════════════════════

/// Inspector panel preferred width
pub const INSPECTOR_WIDTH: f32 = 320.0;

/// Inspector panel minimum width
pub const INSPECTOR_MIN_WIDTH: f32 = 250.0;

/// Hierarchy panel preferred width
pub const HIERARCHY_WIDTH: f32 = 320.0;

/// Hierarchy panel minimum width
pub const HIERARCHY_MIN_WIDTH: f32 = 200.0;

/// Sidebar preferred width
pub const SIDEBAR_WIDTH: f32 = 260.0;

/// Sidebar minimum width
pub const SIDEBAR_MIN_WIDTH: f32 = 200.0;

/// Viewport minimum width
pub const VIEWPORT_MIN_WIDTH: f32 = 300.0;

/// Bottom panel preferred height
pub const BOTTOM_PANEL_HEIGHT: f32 = 250.0;

/// Bottom panel minimum height
pub const BOTTOM_PANEL_MIN_HEIGHT: f32 = 120.0;

// ═══════════════════════════════════════════════════════════════════════
// Control Sizes
// ═══════════════════════════════════════════════════════════════════════

/// Standard button height
pub const BUTTON_HEIGHT: f32 = 32.0;

/// Small button height
pub const BUTTON_HEIGHT_SMALL: f32 = 24.0;

/// Large button height
pub const BUTTON_HEIGHT_LARGE: f32 = 40.0;

/// Icon button size (square)
pub const ICON_BUTTON_SIZE: f32 = 32.0;

/// Activity bar icon button size (square)
pub const ACTIVITY_ICON_SIZE: f32 = 48.0;

/// Text field height
pub const TEXTFIELD_HEIGHT: f32 = 28.0;

/// Search field height
pub const SEARCHFIELD_HEIGHT: f32 = 28.0;

/// Slider height
pub const SLIDER_HEIGHT: f32 = 24.0;

/// Switch width
pub const SWITCH_WIDTH: f32 = 44.0;

/// Switch height
pub const SWITCH_HEIGHT: f32 = 24.0;

/// Stepper width
pub const STEPPER_WIDTH: f32 = 56.0;

/// Stepper height
pub const STEPPER_HEIGHT: f32 = 24.0;

/// Segmented control height
pub const SEGMENTED_HEIGHT: f32 = 28.0;

// ═══════════════════════════════════════════════════════════════════════
// Row Heights
// ═══════════════════════════════════════════════════════════════════════

/// Inspector row height
pub const INSPECTOR_ROW_HEIGHT: f32 = 28.0;

/// Table row height
pub const TABLE_ROW_HEIGHT: f32 = 24.0;

/// Outline view row height
pub const OUTLINE_ROW_HEIGHT: f32 = 24.0;

/// Menu item height
pub const MENU_ITEM_HEIGHT: f32 = 24.0;

/// List item height
pub const LIST_ITEM_HEIGHT: f32 = 24.0;

// ═══════════════════════════════════════════════════════════════════════
// Label Widths
// ═══════════════════════════════════════════════════════════════════════

/// Inspector label width (left column)
pub const INSPECTOR_LABEL_WIDTH: f32 = 80.0;

/// Form label width
pub const FORM_LABEL_WIDTH: f32 = 100.0;

// ═══════════════════════════════════════════════════════════════════════
// Corner Radii (from ThemeToken)
// ═══════════════════════════════════════════════════════════════════════

/// Panel corner radius
pub const RADIUS_PANEL: f32 = 12.0;

/// Button corner radius
pub const RADIUS_BUTTON: f32 = 6.0;

/// Control corner radius (textfield, etc)
pub const RADIUS_CONTROL: f32 = 4.0;

/// Tab corner radius
pub const RADIUS_TAB: f32 = 5.0;

/// Menu corner radius
pub const RADIUS_MENU: f32 = 8.0;

// ═══════════════════════════════════════════════════════════════════════
// Separator Sizes
// ═══════════════════════════════════════════════════════════════════════

/// Separator line width
pub const SEPARATOR_WIDTH: f32 = 1.0;

/// Divider width (thicker separator)
pub const DIVIDER_WIDTH: f32 = 2.0;

// ═══════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════

/// Get button size for a given height
pub const fn button_size(height: f32) -> (f32, f32) {
    let width = height * 2.5; // Aspect ratio for standard buttons
    (width, height)
}

/// Get icon button size (square)
pub const fn icon_button_size(size: f32) -> (f32, f32) {
    (size, size)
}
