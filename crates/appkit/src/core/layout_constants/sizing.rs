//! Sizing constants for icons, controls, windows, toolbars, and bars.
//!
//! These define the dimensions of UI elements across the application.

use crate::theme::{self, keys as tk};
use crate::workspace as workspace_layout;
use super::spacing::{spacing_xs, spacing_sm, spacing_md, spacing_lg};
use super::spacing::font_size_small;

// ── Icon Sizes ───────────────────────────────────────────────────────────────

pub fn icon_size_sm() -> f32 {
    theme::f32_or(tk::SIZE_ICON_SM, 16.0)
}
pub fn icon_size_md() -> f32 {
    theme::f32_or(tk::SIZE_ICON_MD, 24.0)
}
pub fn icon_size_lg() -> f32 {
    theme::f32_or(tk::SIZE_ICON_LG, 32.0)
}
pub fn icon_size_xl() -> f32 {
    theme::f32_or(tk::SIZE_ICON_XL, 32.0)
}

// ── Control Heights ──────────────────────────────────────────────────────────

pub fn control_height() -> f32 {
    theme::f32_or(tk::SIZE_CONTROL_HEIGHT, 22.0)
}
pub fn button_height() -> f32 {
    control_height() + 6.0
}
pub fn search_field_height() -> f32 {
    button_height()
}

// ── Window & Layout ──────────────────────────────────────────────────────────

pub fn sidebar_width() -> f32 {
    workspace_layout::sidebar_width()
}
pub fn title_bar_height() -> f32 {
    workspace_layout::title_bar_height()
}
pub fn tab_bar_height() -> f32 {
    workspace_layout::tab_bar_height()
}
pub fn panel_gap() -> f32 {
    workspace_layout::panel_gap()
}
pub fn window_width() -> f32 {
    1600.0
}
pub fn window_height() -> f32 {
    900.0
}

// ── Toolbar ──────────────────────────────────────────────────────────────────

pub fn toolbar_btn_size() -> f32 {
    button_height() + 4.0
}
pub fn toolbar_icon_size() -> f32 {
    icon_size_lg()
}
pub fn mode_switcher_w() -> f32 {
    260.0
}
pub fn mode_switcher_h() -> f32 {
    control_height() + 2.0
}
pub fn title_bar_total_h() -> f32 {
    40.0
}

// ── Search Bar ───────────────────────────────────────────────────────────────

pub fn search_bar_h() -> f32 {
    control_height() + spacing_sm()
}
pub fn search_bar_w() -> f32 {
    400.0
}
pub fn search_bar_radius() -> f32 {
    search_bar_h() / 2.0
}
pub fn search_radius_pill() -> f32 {
    search_bar_h() / 2.0
}
pub fn window_control_btn_size() -> f32 {
    icon_size_md()
}

// ── Tab Bar ──────────────────────────────────────────────────────────────────

pub fn tab_bar_full_h() -> f32 {
    control_height() + spacing_md() + spacing_xs()
}
pub fn tab_close_size() -> f32 {
    icon_size_sm() - spacing_xs()
}
pub fn tab_text_inset() -> f32 {
    spacing_md() + spacing_sm()
}

// ── Status Bar ───────────────────────────────────────────────────────────────

pub fn status_bar_h() -> f32 {
    control_height() + spacing_xs()
}
pub fn status_bar_item_pad_x() -> f32 {
    spacing_md()
}
pub fn status_bar_font_sz() -> f32 {
    font_size_small() - 1.0
}

// ── Activity Bar ─────────────────────────────────────────────────────────────

pub fn activity_bar_w() -> f32 {
    icon_size_xl() + spacing_lg()
}
pub fn activity_bar_icon_gap() -> f32 {
    icon_size_md() + spacing_lg()
}
pub fn activity_bar_icon_inset_y() -> f32 {
    spacing_lg()
}

// ── Menu Bar ─────────────────────────────────────────────────────────────────

pub fn menu_bar_separator_y_offset() -> f32 {
    title_bar_total_h()
}
