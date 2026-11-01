//! Panel, overlay, accordion, menu, and preferences layout defaults.

use super::spacing::{spacing_sm, spacing_md, spacing_lg, font_size_ui, font_size_small};
use super::sizing::{sidebar_width, control_height, button_height, tab_bar_height};

// ── Panel Defaults ───────────────────────────────────────────────────────────

pub fn panel_min_size_default() -> f32 {
    100.0
}
pub fn panel_preferred_size_default() -> f32 {
    sidebar_width()
}
pub fn hierarchy_default_w() -> f32 {
    250.0
}
pub fn hierarchy_default_h() -> f32 {
    400.0
}
pub fn hierarchy_min_w() -> f32 {
    200.0
}
pub fn hierarchy_max_w() -> f32 {
    500.0
}
pub fn hierarchy_preferred_w() -> f32 {
    sidebar_width()
}
pub fn viewport_default_w() -> f32 {
    800.0
}
pub fn viewport_default_h() -> f32 {
    600.0
}
pub fn viewport_min_w() -> f32 {
    300.0
}
pub fn viewport_preferred_w() -> f32 {
    800.0
}
pub fn timeline_default_w() -> f32 {
    800.0
}
pub fn timeline_default_h() -> f32 {
    150.0
}
pub fn timeline_min_h() -> f32 {
    150.0
}
pub fn timeline_preferred_h() -> f32 {
    250.0
}
pub fn asset_library_default_w() -> f32 {
    300.0
}
pub fn asset_library_default_h() -> f32 {
    400.0
}
pub fn asset_library_min_w() -> f32 {
    150.0
}
pub fn asset_library_preferred_w() -> f32 {
    250.0
}
pub fn statistics_default_w() -> f32 {
    200.0
}
pub fn statistics_default_h() -> f32 {
    150.0
}
pub fn audio_mixer_default_w() -> f32 {
    800.0
}
pub fn audio_mixer_default_h() -> f32 {
    300.0
}
pub fn bottom_tabs_preferred_h() -> f32 {
    250.0
}
pub fn inspector_default_h() -> f32 {
    500.0
}

// ── Overlay ──────────────────────────────────────────────────────────────────

pub fn overlay_x() -> f32 {
    100.0
}
pub fn overlay_y() -> f32 {
    80.0
}
pub fn overlay_w() -> f32 {
    280.0
}
pub fn overlay_h() -> f32 {
    440.0
}
pub fn overlay_content_w() -> f32 {
    overlay_w() - spacing_lg() - spacing_md()
}
pub fn overlay_title_y() -> f32 {
    spacing_md() + 2.0
}
pub fn overlay_sep_y() -> f32 {
    35.0
}
pub fn overlay_menu_y() -> f32 {
    45.0
}
pub fn overlay_menu_h() -> f32 {
    380.0
}

// ── Accordion ────────────────────────────────────────────────────────────────

pub fn accordion_header_h() -> f32 {
    button_height()
}
pub fn accordion_option_h() -> f32 {
    control_height() + 2.0
}
pub fn accordion_icon_size() -> usize {
    12
}
pub fn accordion_icon_inset() -> f32 {
    6.0
}
pub fn accordion_text_inset() -> f32 {
    24.0
}
pub fn accordion_option_text_inset() -> f32 {
    36.0
}
pub fn accordion_checkmark_inset() -> f32 {
    14.0
}
pub fn accordion_checkmark_size() -> f32 {
    10.0
}
pub fn accordion_header_font() -> f32 {
    font_size_ui()
}
pub fn accordion_option_font() -> f32 {
    font_size_small() + 1.0
}

// ── Menu ─────────────────────────────────────────────────────────────────────

pub fn menu_btn_w() -> f32 {
    60.0
}
pub fn menu_btn_h() -> f32 {
    button_height() + 2.0
}
pub fn menu_dropdown_w() -> f32 {
    180.0
}
pub fn menu_item_h() -> f32 {
    button_height()
}
pub fn menu_item_pad() -> f32 {
    spacing_sm() + 2.0
}

// ── Preferences ──────────────────────────────────────────────────────────────

pub fn prefs_window_w() -> f32 {
    560.0
}
pub fn prefs_window_h() -> f32 {
    400.0
}
pub fn prefs_row_h() -> f32 {
    control_height()
}
pub fn prefs_tab_height() -> f32 {
    tab_bar_height()
}

// ── New Project Dialog ───────────────────────────────────────────────────────

pub fn new_project_dialog_w() -> f32 {
    640.0
}
pub fn new_project_dialog_h() -> f32 {
    420.0
}
pub fn new_project_label_w() -> f32 {
    100.0
}
pub fn new_project_button_w() -> f32 {
    80.0
}
