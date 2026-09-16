//! Static `BPath` constants — every fixed state key in one table so raw
//! string literals never leak into call sites. Dynamic keys (host-scoped,
//! per-menu-label) still use `BPath::of(&[..])` at the callsite that owns
//! the varying segment.

use super::BPath;

// ── Scene / world context ──
pub const K_SPACE: BPath = BPath::from_static(&["space"]);
pub const K_SCENE: BPath = BPath::from_static(&["scene"]);
pub const K_VIEW_LAYER: BPath = BPath::from_static(&["view_layer"]);
pub const K_OBJECT: BPath = BPath::from_static(&["object"]);
pub const K_MODE: BPath = BPath::from_static(&["mode"]);
pub const K_TOOL: BPath = BPath::from_static(&["tool"]);
pub const K_FRAME: BPath = BPath::from_static(&["frame"]);
pub const K_PLAYING: BPath = BPath::from_static(&["playing"]);

// ── Theme / gradients ──
pub const K_THEME: BPath = BPath::from_static(&["theme"]);
pub const K_USE_GRADIENTS: BPath = BPath::from_static(&["use_gradients"]);

// ── Modal ──
pub const K_MODAL: BPath = BPath::from_static(&["modal"]);
pub const K_MODAL_MAXIMIZED: BPath = BPath::from_static(&["modal", "maximized"]);
pub const K_MODAL_MINIMIZED: BPath = BPath::from_static(&["modal", "minimized"]);

// ── Activity bar ──
pub const K_ACTIVITY_TAB: BPath = BPath::from_static(&["activity", "tab"]);

// ── Properties navbar ──
/// When set to "true", the Properties navbar expands to show labels alongside
/// icons. When absent or any other value, it collapses to icon-only mode.
pub const K_NAVBAR_EXPANDED: BPath = BPath::from_static(&["buts", "navbar", "expanded"]);

// ── Zone visibility (per-zone slug appended at callsite) ──
pub const K_ZONE_LEFT: BPath = BPath::from_static(&["zone", "left"]);
pub const K_ZONE_RIGHT: BPath = BPath::from_static(&["zone", "right"]);
pub const K_ZONE_BOTTOM: BPath = BPath::from_static(&["zone", "bottom"]);

// ── Menu prefix (concatenate with the menu label at callsite) ──
pub const P_MENU: &str = "menu";
pub const P_AREA: &str = "area";

/// Fixed keys consumed by both UI-style resolution and Preferences.
///
/// Every path encodes as `ctx.ui_style.<domain>.<token>`. Keeping the complete
/// catalog here prevents resolver and UI declarations from inventing aliases.
pub mod ui_style {
    use super::BPath;

    macro_rules! keys {
        ($domain:literal; $($name:ident => $token:literal),+ $(,)?) => {
            $(pub const $name: BPath =
                BPath::from_static(&["ui_style", $domain, $token]);)+
        };
    }

    keys!("colors";
        COLOR_BG_APP => "bg_app", COLOR_BG_ACTIVITY => "bg_activity",
        COLOR_BG_TOP => "bg_top", COLOR_BG_BOTTOM => "bg_bottom",
        COLOR_BG_LEFT => "bg_left", COLOR_BG_RIGHT => "bg_right",
        COLOR_BG_HEADER => "bg_header", COLOR_BG_PANEL => "bg_panel",
        COLOR_BG_VIEWPORT => "bg_viewport", COLOR_BG_HOVER => "bg_hover",
        COLOR_BG_ACTIVE => "bg_active", COLOR_BG_MODAL_DIM => "bg_modal_dim",
        COLOR_TEXT => "text", COLOR_TEXT_DIM => "text_dim",
        COLOR_ACCENT => "accent", COLOR_ACCENT_SOFT => "accent_soft",
        COLOR_ONLINE => "online", COLOR_BORDER => "border", COLOR_SEP => "sep",
        COLOR_ROW_ALT => "row_alt", COLOR_ROW_SELECTED => "row_selected",
        COLOR_ROW_ACTIVE => "row_active", COLOR_ROW_MATCH => "row_match",
        COLOR_VIEWPORT_GRID => "viewport_grid", COLOR_AXIS_X => "axis_x",
        COLOR_AXIS_Y => "axis_y", COLOR_AXIS_Z => "axis_z",
        COLOR_WIRE_ACTIVE => "wire_active",
    );
    keys!("radii"; RADIUS_XS => "xs", RADIUS_SM => "sm", RADIUS_MD => "md",
        RADIUS_LG => "lg", RADIUS_XL => "xl");
    keys!("typography"; TYPOGRAPHY_BODY => "body", TYPOGRAPHY_HEADER => "header",
        TYPOGRAPHY_CAPTION => "caption");
    keys!("spacing";
        SPACING_GAP_XS => "gap_xs", SPACING_GAP_SM => "gap_sm",
        SPACING_GAP_MD => "gap_md", SPACING_GAP_LG => "gap_lg",
        SPACING_PAD_XS => "pad_xs", SPACING_PAD_SM => "pad_sm",
        SPACING_PAD_MD => "pad_md", SPACING_PAD_LG => "pad_lg",
    );
    keys!("dimensions";
        DIMENSION_HEADER_HEIGHT => "header_height", DIMENSION_FOOTER_HEIGHT => "footer_height",
        DIMENSION_ROW_HEIGHT => "row_height", DIMENSION_BUTTON_HEIGHT => "button_height",
        DIMENSION_SCRUBBING_HEIGHT => "scrubbing_height",
        DIMENSION_ASSET_SHELF_HEIGHT => "asset_shelf_height",
        DIMENSION_BOTTOM_STRIP_HEIGHT => "bottom_strip_height",
        DIMENSION_ACTIVITY_WIDTH => "activity_width", DIMENSION_TOOLBAR_WIDTH => "toolbar_width",
        DIMENSION_SIDEBAR_WIDTH => "sidebar_width",
        DIMENSION_SIDEBAR_MIN_WIDTH => "sidebar_min_width",
        DIMENSION_PANEL_WIDTH => "panel_width", DIMENSION_NAVBAR_WIDTH => "navbar_width",
        DIMENSION_CHANNELS_WIDTH => "channels_width",
        DIMENSION_TOOL_PROPS_WIDTH => "tool_props_width",
        DIMENSION_AREA_LEFT_WIDTH => "area_left_width",
        DIMENSION_AREA_RIGHT_WIDTH => "area_right_width",
        DIMENSION_FILE_DATE_COLUMN_WIDTH => "file_date_column_width",
        DIMENSION_FILE_SIZE_COLUMN_WIDTH => "file_size_column_width",
        DIMENSION_INDENT_UNIT => "indent_unit",
        DIMENSION_RESTRICTION_COLUMN_WIDTH => "restriction_column_width",
        DIMENSION_GIZMO_BUTTON_SIZE => "gizmo_button_size",
        DIMENSION_GIZMO_ICON_SIZE => "gizmo_icon_size",
        DIMENSION_OBJECT_ICON_SIZE => "object_icon_size",
        DIMENSION_OBJECT_GLYPH_SIZE => "object_glyph_size",
    );
    keys!("strokes"; STROKE_HAIRLINE => "hairline", STROKE_BORDER => "border",
        STROKE_GRID => "grid");
    keys!("resize";
        RESIZE_VISIBLE_SEAM => "visible_seam", RESIZE_HIT_WIDTH => "hit_width",
        RESIZE_GAP => "gap", RESIZE_INSET => "inset",
        RESIZE_SNAP_THRESHOLD => "snap_threshold", RESIZE_MIN_EXTENT => "min_extent",
    );
    keys!("effects";
        EFFECT_GRADIENTS => "gradients", EFFECT_GRADIENT_LIFT => "gradient_lift",
        EFFECT_SHEEN_LIFT => "sheen_lift",
        EFFECT_SELECTED_HOVER_LIFT => "selected_hover_lift",
        EFFECT_FOOTER_LIFT => "footer_lift", EFFECT_SIDEBAR_LIFT => "sidebar_lift",
        EFFECT_CARD_LIFT => "card_lift", EFFECT_SECTION_LIFT => "section_lift",
        EFFECT_STRIP_RECESS => "strip_recess",
        EFFECT_STRIP_GRADIENT_LIFT => "strip_gradient_lift",
        EFFECT_TAB_ACCENT_MIX => "tab_accent_mix",
        EFFECT_TAB_SATURATION_LIFT => "tab_saturation_lift",
        EFFECT_TAB_TOP_LIFT => "tab_top_lift",
        EFFECT_VIEWPORT_LIGHTNESS_DELTA => "viewport_lightness_delta",
        EFFECT_VIEWPORT_GRID_ALPHA => "viewport_grid_alpha",
        EFFECT_AXIS_ALPHA => "axis_alpha", EFFECT_NODE_GRID_LIFT => "node_grid_lift",
        EFFECT_NODE_GRID_ALPHA => "node_grid_alpha",
        EFFECT_IMAGE_LIGHT_LIFT => "image_light_lift",
        EFFECT_IMAGE_DARK_LIFT => "image_dark_lift",
        EFFECT_SURFACE_SHADOW => "surface_shadow",
    );
}
