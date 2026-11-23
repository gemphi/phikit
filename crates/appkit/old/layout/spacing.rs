//! # Unified Spacing System
//!
//! Single source of truth for all spacing, padding, margin, gap, and inset values.
//! Based on an 8pt grid system with mathematical consistency.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rust_appkit::layout::spacing;
//!
//! // Use spacing scale
//! let small_gap = spacing::SM;      // 12pt
//! let medium_gap = spacing::MD;     // 16pt
//!
//! // Use component-specific padding
//! let btn_pad = spacing::padding::BUTTON_H;  // 16pt
//!
//! // Use component-specific margin
//! let panel_margin = spacing::margin::PANEL;  // 16pt
//!
//! // Use component-specific gap
//! let toolbar_gap = spacing::gap::TOOLBAR;  // 8pt
//!
//! // Use component-specific inset
//! let activity_inset = spacing::inset::ACTIVITY_BAR;  // 8pt
//! ```

// ═══════════════════════════════════════════════════════════════════════
// Base Unit (8pt grid)
// ═══════════════════════════════════════════════════════════════════════

/// Base unit for all spacing (8pt grid)
pub const BASE: f32 = 8.0;

// ═══════════════════════════════════════════════════════════════════════
// Spacing Scale (8pt grid, mathematically consistent)
// ═══════════════════════════════════════════════════════════════════════

/// 2pt - Minimal (0.25x base)
pub const XXXS: f32 = BASE * 0.25;

/// 4pt - Tiny (0.5x base)
pub const XXS: f32 = BASE * 0.5;

/// 8pt - Extra small (1x base)
pub const XS: f32 = BASE;

/// 12pt - Small (1.5x base)
pub const SM: f32 = BASE * 1.5;

/// 16pt - Medium (2x base)
pub const MD: f32 = BASE * 2.0;

/// 24pt - Large (3x base)
pub const LG: f32 = BASE * 3.0;

/// 32pt - Extra large (4x base)
pub const XL: f32 = BASE * 4.0;

/// 48pt - Huge (6x base)
pub const XXL: f32 = BASE * 6.0;

// ═══════════════════════════════════════════════════════════════════════
// Padding Module (component-specific padding values)
// ═══════════════════════════════════════════════════════════════════════

pub mod padding {
    use super::*;

    // ─── Controls ────────────────────────────────────────────────────

    /// Button padding horizontal
    pub const BUTTON_H: f32 = MD;  // 16pt

    /// Button padding vertical
    pub const BUTTON_V: f32 = XS;  // 8pt

    /// TextField padding horizontal
    pub const TEXTFIELD_H: f32 = SM;  // 12pt

    /// TextField padding vertical
    pub const TEXTFIELD_V: f32 = XS;  // 8pt

    /// Slider padding
    pub const SLIDER: f32 = XS;  // 8pt

    /// Switch padding
    pub const SWITCH: f32 = XXS;  // 4pt

    /// Stepper padding
    pub const STEPPER: f32 = XXS;  // 4pt

    /// DatePicker padding
    pub const DATEPICKER: f32 = SM;  // 12pt

    /// SearchField padding
    pub const SEARCHFIELD: f32 = SM;  // 12pt

    /// SegmentedControl padding
    pub const SEGMENTED: f32 = XXS;  // 4pt

    /// PopupButton padding
    pub const POPUP: f32 = SM;  // 12pt

    // ─── Panels ──────────────────────────────────────────────────────

    /// Panel padding (all sides)
    pub const PANEL: f32 = MD;  // 16pt

    /// Inspector panel padding
    pub const INSPECTOR: f32 = MD;  // 16pt

    /// Hierarchy panel padding
    pub const HIERARCHY: f32 = SM;  // 12pt

    /// Viewport panel padding
    pub const VIEWPORT: f32 = XS;  // 8pt

    /// Statistics panel padding
    pub const STATISTICS: f32 = MD;  // 16pt

    /// Timeline panel padding
    pub const TIMELINE: f32 = SM;  // 12pt

    // ─── Views ───────────────────────────────────────────────────────

    /// Sidebar padding (all sides)
    pub const SIDEBAR: f32 = SM;  // 12pt

    /// Toolbar padding horizontal
    pub const TOOLBAR_H: f32 = MD;  // 16pt

    /// Toolbar padding vertical
    pub const TOOLBAR_V: f32 = XS;  // 8pt

    /// Tab padding horizontal
    pub const TAB_H: f32 = MD;  // 16pt

    /// Tab padding vertical
    pub const TAB_V: f32 = XS;  // 8pt

    /// ActivityBar padding
    pub const ACTIVITY_BAR: f32 = XS;  // 8pt

    /// OutlineView padding
    pub const OUTLINE: f32 = SM;  // 12pt

    /// TableView padding
    pub const TABLE: f32 = SM;  // 12pt

    // ─── Menus ───────────────────────────────────────────────────────

    /// Menu padding (all sides)
    pub const MENU: f32 = XXS;  // 4pt

    /// MenuItem padding horizontal
    pub const MENUITEM_H: f32 = SM;  // 12pt

    /// MenuItem padding vertical
    pub const MENUITEM_V: f32 = XS;  // 8pt

    /// Popover padding
    pub const POPOVER: f32 = MD;  // 16pt

    /// Dropdown padding
    pub const DROPDOWN: f32 = XXS;  // 4pt

    // ─── Inspector Specific ──────────────────────────────────────────

    /// Inspector row padding
    pub const INSPECTOR_ROW: f32 = SM;  // 12pt

    /// Inspector section padding
    pub const INSPECTOR_SECTION: f32 = MD;  // 16pt

    /// Inspector header padding
    pub const INSPECTOR_HEADER: f32 = SM;  // 12pt
}

// ═══════════════════════════════════════════════════════════════════════
// Margin Module (component-specific margin values)
// ═══════════════════════════════════════════════════════════════════════

pub mod margin {
    use super::*;

    // ─── Windows & Containers ────────────────────────────────────────

    /// Window content margin
    pub const WINDOW: f32 = LG;  // 24pt

    /// Panel content margin
    pub const PANEL: f32 = MD;  // 16pt

    /// Sidebar content margin
    pub const SIDEBAR: f32 = SM;  // 12pt

    /// Toolbar content margin
    pub const TOOLBAR: f32 = SM;  // 12pt

    /// Dialog content margin
    pub const DIALOG: f32 = LG;  // 24pt

    /// Sheet content margin
    pub const SHEET: f32 = LG;  // 24pt

    /// Popover content margin
    pub const POPOVER: f32 = MD;  // 16pt

    // ─── Controls ────────────────────────────────────────────────────

    /// Control margin (general)
    pub const CONTROL: f32 = XS;  // 8pt

    /// Button margin
    pub const BUTTON: f32 = XS;  // 8pt

    /// TextField margin
    pub const TEXTFIELD: f32 = XS;  // 8pt

    /// Label margin
    pub const LABEL: f32 = XXS;  // 4pt

    // ─── Layout ──────────────────────────────────────────────────────

    /// Section margin
    pub const SECTION: f32 = MD;  // 16pt

    /// Group margin
    pub const GROUP: f32 = SM;  // 12pt

    /// Row margin
    pub const ROW: f32 = XXS;  // 4pt
}

// ═══════════════════════════════════════════════════════════════════════
// Gap Module (component-specific gap values)
// ═══════════════════════════════════════════════════════════════════════

pub mod gap {
    use super::*;

    // ─── Views ───────────────────────────────────────────────────────

    /// Toolbar item gap
    pub const TOOLBAR: f32 = XS;  // 8pt

    /// Tab item gap
    pub const TAB: f32 = XXS;  // 4pt

    /// Sidebar item gap
    pub const SIDEBAR: f32 = XXS;  // 4pt

    /// ActivityBar item gap
    pub const ACTIVITY_BAR: f32 = XS;  // 8pt

    /// OutlineView item gap
    pub const OUTLINE: f32 = XXS;  // 4pt

    /// TableView row gap
    pub const TABLE: f32 = 0.0;  // No gap

    // ─── Menus ───────────────────────────────────────────────────────

    /// MenuItem gap
    pub const MENUITEM: f32 = 0.0;  // No gap

    /// Menu section gap
    pub const MENU_SECTION: f32 = XS;  // 8pt

    // ─── Inspector ───────────────────────────────────────────────────

    /// Inspector row gap
    pub const INSPECTOR_ROW: f32 = XXS;  // 4pt

    /// Inspector section gap
    pub const INSPECTOR_SECTION: f32 = MD;  // 16pt

    /// Inspector field gap
    pub const INSPECTOR_FIELD: f32 = XS;  // 8pt

    // ─── Forms & Controls ────────────────────────────────────────────

    /// Form field gap
    pub const FORM_FIELD: f32 = SM;  // 12pt

    /// Button group gap
    pub const BUTTON_GROUP: f32 = XS;  // 8pt

    /// SegmentedControl gap
    pub const SEGMENTED: f32 = 0.0;  // No gap (segments touch)

    /// Stepper button gap
    pub const STEPPER: f32 = 0.0;  // No gap

    // ─── Layout ──────────────────────────────────────────────────────

    /// Stack horizontal gap (default)
    pub const STACK_H: f32 = XS;  // 8pt

    /// Stack vertical gap (default)
    pub const STACK_V: f32 = XS;  // 8pt

    /// Grid gap (default)
    pub const GRID: f32 = XS;  // 8pt
}

// ═══════════════════════════════════════════════════════════════════════
// Inset Module (component-specific inset values)
// ═══════════════════════════════════════════════════════════════════════

pub mod inset {
    use super::*;

    // ─── Views ───────────────────────────────────────────────────────

    /// ActivityBar icon inset
    pub const ACTIVITY_BAR: f32 = XS;  // 8pt

    /// TabBar inset
    pub const TAB_BAR: f32 = XXS;  // 4pt

    /// Toolbar inset
    pub const TOOLBAR: f32 = XXS;  // 4pt

    /// Sidebar inset
    pub const SIDEBAR: f32 = XXS;  // 4pt

    // ─── Controls ────────────────────────────────────────────────────

    /// SegmentedControl inset
    pub const SEGMENTED: f32 = XXS;  // 4pt

    /// Switch inset
    pub const SWITCH: f32 = XXXS;  // 2pt

    /// Button inset (for icon buttons)
    pub const BUTTON: f32 = XXS;  // 4pt

    /// TextField inset
    pub const TEXTFIELD: f32 = XXS;  // 4pt

    // ─── Panels ──────────────────────────────────────────────────────

    /// Panel content inset
    pub const PANEL: f32 = MD;  // 16pt

    /// Inspector content inset
    pub const INSPECTOR: f32 = MD;  // 16pt

    /// Viewport content inset
    pub const VIEWPORT: f32 = 0.0;  // No inset (full bleed)
}

// ═══════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════

/// Get spacing for a specific multiplier (maintain 8pt grid)
pub const fn space(multiplier: f32) -> f32 {
    BASE * multiplier
}

/// Get spacing at a specific scale level (0-7)
pub const fn at_scale(level: u8) -> f32 {
    match level {
        0 => XXXS,  // 2pt
        1 => XXS,   // 4pt
        2 => XS,    // 8pt
        3 => SM,    // 12pt
        4 => MD,    // 16pt
        5 => LG,    // 24pt
        6 => XL,    // 32pt
        7 => XXL,   // 48pt
        _ => MD,    // Default to medium
    }
}
