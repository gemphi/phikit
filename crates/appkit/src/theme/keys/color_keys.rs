// ============================================================
// Theme Color Key Constants
//
// Named keys for all color-related theme tokens.
// ============================================================

// OS / variant identifiers
pub const OS_WINDOWS: &str = "windows";
pub const OS_MACOS: &str = "macos";
pub const WIN_11: &str = "11";
pub const MAC_SONOMA: &str = "sonoma";
pub const MAT_ACRYLIC: &str = "acrylic";
pub const MAT_VIBRANCY: &str = "vibrancy";
pub const VAR_LIGHT: &str = "light";
pub const VAR_DARK: &str = "dark";

// --- Colors: backgrounds ---
pub const CLR_BG_WINDOW: &str = "bg_window";
pub const CLR_BG_WINDOW_TOP: &str = "bg_window_top";
pub const CLR_BG_WINDOW_BOTTOM: &str = "bg_window_bottom";
pub const CLR_BG_EDITOR_TOP: &str = "bg_editor_top";
pub const CLR_BG_EDITOR_BOTTOM: &str = "bg_editor_bottom";
pub const CLR_BG_TITLEBAR: &str = "bg_titlebar";
pub const CLR_BG_CONTENT: &str = "bg_content";
pub const CLR_BG_CONTROL: &str = "bg_control";
pub const CLR_BG_CONTROL_FOCUSED: &str = "bg_control_focused";
pub const CLR_BG_BUTTON: &str = "bg_button";
pub const CLR_BG_BUTTON_HOVER: &str = "bg_button_hover";
pub const CLR_BG_BUTTON_PRESSED: &str = "bg_button_pressed";
pub const CLR_BG_SELECTED: &str = "bg_selected";
pub const CLR_BG_HOVER: &str = "bg_hover";
pub const CLR_BG_SIDEBAR: &str = "bg_sidebar";
pub const CLR_BG_PANEL: &str = "bg_panel";
pub const CLR_BG_HEADER: &str = "bg_header";
pub const CLR_BG_TIMELINE: &str = "bg_timeline";
pub const CLR_BG_RULER: &str = "bg_ruler";
pub const CLR_BG_DROPDOWN: &str = "bg_dropdown";
pub const CLR_BG_CANVAS: &str = "bg_canvas";
pub const CLR_BG_GRID: &str = "bg_grid";
pub const CLR_BG_TOOLBAR: &str = "bg_toolbar";
pub const CLR_BG_TAB_BAR: &str = "bg_tab_bar";
pub const CLR_BG_TAB_ACTIVE: &str = "bg_tab_active";
pub const CLR_BG_TAB_INACTIVE: &str = "bg_tab_inactive";
pub const CLR_BG_CARD: &str = "bg_card";
pub const CLR_BORDER_CARD: &str = "border_card";
pub const CLR_NODE_BODY: &str = "node_body";
pub const CLR_NODE_HEADER: &str = "node_header";
pub const CLR_PORT_OUT: &str = "port_out";
pub const CLR_PORT_IN: &str = "port_in";
pub const CLR_CONN_LINE: &str = "conn_line";

// --- Colors: text ---
pub const CLR_TEXT_LABEL: &str = "text_label";
pub const CLR_TEXT_SECONDARY: &str = "text_secondary";
pub const CLR_TEXT_TERTIARY: &str = "text_tertiary";
pub const CLR_TEXT_DISABLED: &str = "text_disabled";
pub const CLR_TEXT_PLACEHOLDER: &str = "text_placeholder";
pub const CLR_TEXT_SELECTED: &str = "text_selected";
pub const CLR_TEXT_ON_ACCENT: &str = "text_on_accent";
pub const CLR_TAB_TEXT_ACTIVE: &str = "tab_text_active";
pub const CLR_TAB_TEXT_INACTIVE: &str = "tab_text_inactive";

// --- Colors: accents / chrome ---
pub const CLR_ACCENT: &str = "accent";
pub const CLR_ACCENT_LIGHT: &str = "accent_light";
pub const CLR_ACCENT_DARK: &str = "accent_dark";
pub const CLR_BORDER: &str = "border";
pub const CLR_SEPARATOR: &str = "separator";
pub const CLR_SHADOW: &str = "shadow";
pub const CLR_TITLE_BAR: &str = "title_bar";
pub const CLR_TITLE_BAR_TEXT: &str = "title_bar_text";
pub const CLR_TRAFFIC_CLOSE: &str = "traffic_close";
pub const CLR_TRAFFIC_MIN: &str = "traffic_min";
pub const CLR_TRAFFIC_ZOOM: &str = "traffic_zoom";
pub const CLR_RED: &str = "red";
pub const CLR_GREEN: &str = "green";
pub const CLR_YELLOW: &str = "yellow";
pub const CLR_BLUE: &str = "blue";

// --- Colors: entity icon tints ---
pub const CLR_ICON_DEFAULT: &str = "icon_default";
pub const CLR_ICON_CAMERA: &str = "icon_camera";
pub const CLR_ICON_LIGHT: &str = "icon_light";
pub const CLR_ICON_MATERIAL: &str = "icon_material";
pub const CLR_ICON_MESH: &str = "icon_mesh";
pub const CLR_ICON_ARMATURE: &str = "icon_armature";

// --- Glass / material ---
// 1.0 = enabled, 0.0 = disabled
pub const GLASS_ENABLED: &str = "glass_enabled";
/// How much of a view's colour covers its Regular glass, 0..=1.
pub const GLASS_OPACITY: &str = "glass_opacity";
pub const CLR_GLASS_EDGE: &str = "glass_edge";

// --- Shadows (Apple NSShadow: offset, blurRadius, color) ---
pub const SHADOW_OFFSET_X: &str = "shadow_offset_x"; // px
pub const SHADOW_OFFSET_Y: &str = "shadow_offset_y"; // px
pub const SHADOW_BLUR: &str = "shadow_blur"; // blur radius in px
pub const CLR_SHADOW_COLOR: &str = "shadow_color"; // NSColor key

// --- Smoothness / anti-aliasing ---
pub const SMOOTHNESS: &str = "smoothness"; // 0.0=crisp .. 1.0=max AA
pub const ANIMATION_SPEED: &str = "animation_speed"; // multiplier: 0.0=instant, 1.0=normal
