//! Keys — every path the state is addressed by, spelt once, as an enum.
//!
//! A key is `ctx.{domain}.{owner}.{field}`. The domain is a file here; each
//! variant of its enum is one field, and `BKey::path` is the only place the
//! segments are written. A caller writes `s.get(FileKey::Path)`, never a
//! string with a dot in it, so a reader looking for what the file browser
//! stores opens `keys/file.rs` and sees all of it; a search for a key's uses
//! is a search for a variant.
//!
//! The older `K_…` constants in `state/key.rs` and the `k_…()` functions
//! scattered through the parts are the domains not yet moved here; they
//! implement `BKey` through `BPath`, so both spellings meet at the store.

pub mod activity;
pub mod app;
pub mod area;
pub mod buts;
pub mod dope;
pub mod ctx;
pub mod file;
pub mod image;
pub mod uv;
pub mod menu;
pub mod modal;
pub mod op;
pub mod outliner;
pub mod screen;
pub mod scene;
pub mod time_view;
pub mod view3d;
pub mod zone;

pub use activity::ActivityKey;
pub use app::AppKey;
pub use area::AreaKey;
pub use buts::ButsKey;
pub use dope::DopeKey;
pub use ctx::CtxKey;
pub use file::FileKey;
pub use image::ImageKey;
pub use uv::UvKey;
pub use menu::MenuKey;
pub use modal::ModalKey;
pub use op::OpKey;
pub use outliner::OutlinerKey;
pub use screen::ScreenKey;
pub use scene::SceneKey;
pub use time_view::TimeViewKey;
pub use view3d::View3dKey;
pub use zone::ZoneKey;

use super::BPath;

/// Anything that names a state path.
pub trait BKey {
    fn path(&self) -> BPath;
}

impl BKey for BPath {
    fn path(&self) -> BPath {
        self.clone()
    }
}

impl<K: BKey> BKey for &K {
    fn path(&self) -> BPath {
        (**self).path()
    }
}

// ── The helpers that are not a key enum (was `state/key.rs`) ──


/// Integer / float state reads with a fallback.
pub fn int_or(s: &super::BState, key: impl super::BKey, dflt: i32) -> i32 {
    s.get(key).and_then(|v| v.parse().ok()).unwrap_or(dflt)
}
pub fn float_or(s: &super::BState, key: impl super::BKey, dflt: f32) -> f32 {
    s.get(key).and_then(|v| v.parse().ok()).unwrap_or(dflt)
}


/// Fixed keys consumed by both UI-style resolution and Preferences.
///
/// Every path encodes as `ctx.ui_style.<domain>.<token>`. Keeping the complete
/// catalog here prevents resolver and UI declarations from inventing aliases.
pub mod ui_style {
    use crate::v2::state::BPath;

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
        COLOR_BG_VIEWPORT => "bg_viewport", COLOR_BG_CONTROL => "bg_control", COLOR_BG_HOVER => "bg_hover",
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
    keys!("typography"; TYPOGRAPHY_FAMILY => "family", TYPOGRAPHY_BODY => "body",
        TYPOGRAPHY_HEADER => "header", TYPOGRAPHY_CAPTION => "caption");
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
        DIMENSION_STATUS_LOG_HEIGHT => "status_log_height",
        DIMENSION_BANNER_HEIGHT => "banner_height",
        DIMENSION_ACTIVITY_WIDTH => "activity_width", DIMENSION_TOOLBAR_WIDTH => "toolbar_width",
        DIMENSION_SIDEBAR_WIDTH => "sidebar_width",
        DIMENSION_SIDEBAR_MIN_WIDTH => "sidebar_min_width",
        DIMENSION_NUMBER_MIN_WIDTH => "number_min_width",
        DIMENSION_PANEL_WIDTH => "panel_width", DIMENSION_NAVBAR_WIDTH => "navbar_width",
        DIMENSION_CHANNELS_WIDTH => "channels_width",
        DIMENSION_TOOL_PROPS_WIDTH => "tool_props_width",
        DIMENSION_AREA_LEFT_WIDTH => "area_left_width",
        DIMENSION_AREA_RIGHT_WIDTH => "area_right_width",
        DIMENSION_FILE_DATE_COLUMN_WIDTH => "file_date_column_width",
        DIMENSION_FILE_SIZE_COLUMN_WIDTH => "file_size_column_width",
        DIMENSION_INDENT_UNIT => "indent_unit",
        DIMENSION_RESTRICTION_COLUMN_WIDTH => "restriction_column_width",
        DIMENSION_ICON_SIZE => "icon_size",
        DIMENSION_ICON_SIZE_SM => "icon_size_sm",
        DIMENSION_BRAND_SIZE => "brand_size",
        DIMENSION_GIZMO_BUTTON_SIZE => "gizmo_button_size",
        DIMENSION_GIZMO_ICON_SIZE => "gizmo_icon_size",
        DIMENSION_OBJECT_ICON_SIZE => "object_icon_size",
        DIMENSION_OBJECT_GLYPH_SIZE => "object_glyph_size",
    );
    keys!("strokes"; STROKE_HAIRLINE => "hairline", STROKE_BORDER => "border",
        STROKE_GRID => "grid");
    keys!("resize";
        RESIZE_SEAM_REST_UNDER => "seam_rest_under", RESIZE_SEAM_HOVER_OVER => "seam_hover_over",
        RESIZE_HIT_WIDTH => "hit_width",
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
        EFFECT_MOTION_MS => "motion_ms",
        EFFECT_AXIS_ALPHA => "axis_alpha", EFFECT_GRIP_ALPHA => "grip_alpha",
        EFFECT_NODE_GRID_LIFT => "node_grid_lift",
        EFFECT_NODE_GRID_ALPHA => "node_grid_alpha",
        EFFECT_IMAGE_LIGHT_LIFT => "image_light_lift",
        EFFECT_IMAGE_DARK_LIFT => "image_dark_lift",
        EFFECT_SURFACE_SHADOW => "surface_shadow",
    );
}
