//! Welcome window layout constants.
//!
//! All geometry for the welcome/onboarding window, including both
//! token-driven functions and static constants.

use super::spacing::{spacing_xs, spacing_sm, spacing_md, spacing_lg, radius_panel, font_size_title};
use super::sizing::{icon_size_xl, button_height, control_height};

// ── Welcome Window Functions ─────────────────────────────────────────────────

pub fn welcome_block_max_w() -> f32 {
    480.0
}
pub fn welcome_block_ratio() -> f32 {
    0.6
}
pub fn welcome_icon_row_h() -> f32 {
    icon_size_xl() + spacing_lg()
}
pub fn welcome_ghost_sz() -> f32 {
    icon_size_xl() + spacing_md()
}
pub fn welcome_title_font_sz() -> f32 {
    font_size_title()
}
pub fn welcome_subtitle_row_h() -> f32 {
    control_height() + 2.0
}
pub fn welcome_section_row_h() -> f32 {
    button_height()
}
pub fn welcome_main_btn_h() -> f32 {
    button_height() + 16.0
}
pub fn welcome_card_row_h() -> f32 {
    140.0
}
pub fn welcome_bottom_btn_h() -> f32 {
    button_height() + 12.0
}
pub fn welcome_card_row_item_h() -> f32 {
    button_height() + 4.0
}
pub fn welcome_min_top_margin() -> f32 {
    30.0
}
pub fn welcome_window_w() -> f32 {
    720.0
}
pub fn welcome_window_h() -> f32 {
    480.0
}
pub fn welcome_icon_size() -> f32 {
    icon_size_xl() * 2.0
}
pub fn welcome_icon_radius() -> f32 {
    radius_panel() + spacing_sm()
}
pub fn welcome_title_font() -> f32 {
    font_size_title()
}
pub fn welcome_button_w() -> f32 {
    200.0
}
pub fn welcome_button_h() -> f32 {
    button_height() + 8.0
}
pub fn welcome_gap_icon_title() -> f32 {
    spacing_lg() + spacing_sm()
}
pub fn welcome_gap_title_subtitle() -> f32 {
    spacing_md()
}
pub fn welcome_gap_subtitle_btn() -> f32 {
    spacing_lg() + spacing_md()
}
pub fn welcome_gap_btn_btn() -> f32 {
    spacing_lg() - spacing_sm()
}
pub fn welcome_row_height() -> f32 {
    button_height()
}
pub fn welcome_window_radius() -> f32 {
    radius_panel() + 2.0
}
pub fn welcome_card_radius() -> f32 {
    radius_panel()
}

// ── Welcome Static Constants ─────────────────────────────────────────────────

pub const WELCOME_ICON_SIZE: f32 = 64.0;
pub const WELCOME_ICON_RADIUS: f32 = 14.0;
pub const WELCOME_TITLE_FONT: f32 = 20.0;
pub const WELCOME_TITLE_HEIGHT: f32 = 30.0;
pub const WELCOME_SUBTITLE_FONT: f32 = 13.0;
pub const WELCOME_SUBTITLE_HEIGHT: f32 = 20.0;
pub const WELCOME_BUTTON_W: f32 = 200.0;
pub const WELCOME_BUTTON_H: f32 = 36.0;
pub const WELCOME_GAP_ICON_TITLE: f32 = 20.0;
pub const WELCOME_GAP_TITLE_SUBTITLE: f32 = 8.0;
pub const WELCOME_GAP_SUBTITLE_BTN: f32 = 24.0;
pub const WELCOME_GAP_BTN_BTN: f32 = 12.0;
pub const WELCOME_GAP_BTN_LIST: f32 = 24.0;
pub const WELCOME_ROW_HEIGHT: f32 = 28.0;
pub const WELCOME_LIST_PAD_H: f32 = 12.0;
pub const WELCOME_LIST_PAD_V: f32 = 8.0;
pub const WELCOME_LIST_INSET: f32 = 60.0;
pub const WELCOME_WINDOW_RADIUS: f32 = 12.0;
pub const WELCOME_CARD_RADIUS: f32 = 10.0;
pub const WELCOME_CARD_BORDER: f32 = 1.0;
pub const WELCOME_MIN_MARGIN: f32 = 20.0;
pub const WELCOME_MIN_LIST_H: f32 = 80.0;
pub const WELCOME_WINDOW_W: f32 = 720.0;
pub const WELCOME_WINDOW_H: f32 = 480.0;
