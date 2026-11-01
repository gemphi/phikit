// ============================================================
// Theme Spacing & Sizing Key Constants
//
// Named keys for geometry, corner radii, spacing, and sizing tokens.
// ============================================================

// --- Geometry: corner radii (f32 stored as NSColor.r) ---
// Retrieve with: theme::f32_or(key, fallback)
pub const RADIUS_BUTTON: &str = "radius_button";
pub const RADIUS_PANEL: &str = "radius_panel";
pub const RADIUS_CONTROL: &str = "radius_control";
pub const RADIUS_NODE: &str = "radius_node";
pub const RADIUS_TAB: &str = "radius_tab";
pub const RADIUS_DROPDOWN: &str = "radius_dropdown";
pub const RADIUS_TOOLTIP: &str = "radius_tooltip";
pub const RADIUS_ICON: &str = "radius_icon"; // SF Symbols style

// --- Geometry: spacing / sizing (f32 stored as NSColor.r) ---
pub const SPACING_XS: &str = "spacing_xs"; // 2
pub const SPACING_SM: &str = "spacing_sm"; // 4
pub const SPACING_MD: &str = "spacing_md"; // 8
pub const SPACING_LG: &str = "spacing_lg"; // 16
pub const SIZE_CONTROL_HEIGHT: &str = "size_control_h"; // e.g. 22, 24, 28
pub const SIZE_TAB_HEIGHT: &str = "size_tab_h";
pub const SIZE_TAB_PAD_X: &str = "size_tab_pad_x";
pub const SIZE_MENU_BAR_HEIGHT: &str = "size_menu_bar_h";
pub const SIZE_TITLE_BAR_HEIGHT: &str = "size_title_bar_h";
pub const SIZE_SIDEBAR_WIDTH: &str = "size_sidebar_w";
pub const SIZE_FONT_UI: &str = "size_font_ui";
pub const SIZE_FONT_SMALL: &str = "size_font_sm";
pub const SIZE_CHAR_WIDTH: &str = "size_char_w"; // approx px per char at ui font size

// --- Icon sizing (uniform across all panels) ---
pub const SIZE_ICON_SM: &str = "size_icon_sm"; // 16px — inline text icons
pub const SIZE_ICON_MD: &str = "size_icon_md"; // 20px — toolbar / header
pub const SIZE_ICON_LG: &str = "size_icon_lg"; // 24px — default panel icons
pub const SIZE_ICON_XL: &str = "size_icon_xl"; // 32px — tools sidebar
pub const ICON_STROKE_WEIGHT: &str = "icon_stroke_w"; // SVG stroke-width multiplier (1.0 = default, 1.8 = bold)

// --- Panel header / toolbar (sub-panel toolbars, not the window title bar) ---
pub const SIZE_PANEL_HEADER_H: &str = "size_panel_header_h"; // 32 — panel header bar height
pub const SIZE_PANEL_TOOLBAR_H: &str = "size_panel_toolbar_h"; // 28 — sub-toolbar (search/filter bar)
pub const SIZE_HEADER_TAB_H: &str = "size_header_tab_h"; // 24 — tab pill inside header
pub const SIZE_HEADER_TAB_PAD: &str = "size_header_tab_pad"; // 10 — horizontal padding inside tab
pub const SIZE_HEADER_TAB_GAP: &str = "size_header_tab_gap"; // 4 — gap between tabs
pub const RADIUS_HEADER_TAB: &str = "radius_header_tab"; // 8 — tab pill corner radius

// --- Content rows (hierarchy, inspector, project browser) ---
pub const SIZE_ROW_H: &str = "size_row_h"; // 24 — standard row height
pub const SIZE_ROW_GAP: &str = "size_row_gap"; // 2 — vertical gap between rows
pub const SIZE_ROW_INDENT: &str = "size_row_indent"; // 16 — tree indent per level
pub const RADIUS_ROW: &str = "radius_row"; // 4 — row background radius

// --- Search field ---
pub const SIZE_SEARCH_H: &str = "size_search_h"; // 20 — search input height
pub const SIZE_SEARCH_W: &str = "size_search_w"; // 180 — default search field width
pub const SIZE_SEARCH_MARGIN: &str = "size_search_margin"; // 8 — margin around search

// --- Gizmo toolbar (viewport tool buttons) ---
pub const SIZE_GIZMO: &str = "size_gizmo"; // 28 — gizmo button size
pub const SIZE_GIZMO_GAP: &str = "size_gizmo_gap"; // 4 — gap between gizmo buttons
pub const SIZE_GIZMO_MARGIN: &str = "size_gizmo_margin"; // 12 — margin from panel edge
pub const RADIUS_GIZMO: &str = "radius_gizmo"; // 6 — gizmo button corner radius

// --- Orientation gizmo (3D cube in viewport corner) ---
pub const SIZE_ORIENT_GIZMO: &str = "size_orient_gizmo"; // 48 — orientation cube size
pub const SIZE_ORIENT_MARGIN: &str = "size_orient_margin"; // 12 — margin from corner

// --- Viewport grid ---
pub const SIZE_GRID_SPACING: &str = "size_grid_spacing"; // 80 — distance between grid lines
pub const SIZE_GRID_THICKNESS: &str = "size_grid_thickness"; // 1 — grid line width
pub const SIZE_CROSSHAIR_LEN: &str = "size_crosshair_len"; // 20 — origin crosshair arm length
pub const SIZE_CROSSHAIR_W: &str = "size_crosshair_w"; // 2 — origin crosshair thickness

// --- Inspector sections ---
pub const SIZE_SECTION_PAD: &str = "size_section_pad"; // 8 — padding inside section card
pub const RADIUS_SECTION: &str = "radius_section"; // 8 — section card corner radius
pub const SIZE_LABEL_W: &str = "size_label_w"; // 80 — property label width

// --- Content padding (universal panel inset) ---
pub const SIZE_CONTENT_PAD: &str = "size_content_pad"; // 12 — content area padding

// --- Preferences / settings ---
pub const SIZE_PREF_SIDEBAR_W: &str = "size_pref_sidebar_w"; // 200 — preferences sidebar width
pub const SIZE_PREF_TAB_H: &str = "size_pref_tab_h"; // 32 — preferences tab height
pub const SIZE_PREF_ROW_H: &str = "size_pref_row_h"; // 32 — settings row height
pub const SIZE_PREF_CONTROL_W: &str = "size_pref_control_w"; // 180 — settings control width
pub const SIZE_PREF_CONTROL_H: &str = "size_pref_control_h"; // 28 — settings control height

// --- Font sizes (beyond the base ui/small) ---
pub const SIZE_FONT_TITLE: &str = "size_font_title"; // 20 — section/page title
pub const SIZE_FONT_HEADER: &str = "size_font_header"; // 14 — panel header text
pub const SIZE_FONT_CAPTION: &str = "size_font_caption"; // 9 — category headers, captions

// --- Activity bar ---
pub const SIZE_ACTIVITY_W: &str = "size_activity_w"; // 58 — activity bar width
pub const SIZE_ACTIVITY_ICON: &str = "size_activity_icon"; // 24 — activity bar icon size
pub const SIZE_ACTIVITY_GAP: &str = "size_activity_gap"; // 14 — gap between activity icons
pub const RADIUS_ACTIVITY_ICON: &str = "radius_activity_icon"; // 6 — activity icon radius

// --- Button sizing ---
pub const SIZE_BUTTON_H: &str = "size_button_h"; // 26 — standard button height
pub const SIZE_BUTTON_GAP: &str = "size_button_gap"; // 30 — vertical spacing between buttons
