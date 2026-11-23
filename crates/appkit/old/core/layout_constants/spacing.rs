//! Spacing tokens, radii, and typography constants.
//!
//! These are the foundational token-driven geometry values used throughout the app.

use crate::theme::{self, keys as tk};

// ── Spacing ──────────────────────────────────────────────────────────────────

pub fn spacing_xs() -> f32 {
    theme::f32_or(tk::SPACING_XS, 2.0)
}
pub fn spacing_sm() -> f32 {
    theme::f32_or(tk::SPACING_SM, 4.0)
}
pub fn spacing_md() -> f32 {
    theme::f32_or(tk::SPACING_MD, 8.0)
}
pub fn spacing_lg() -> f32 {
    theme::f32_or(tk::SPACING_LG, 16.0)
}

// ── Panel Insets & Section Spacing ───────────────────────────────────────────

pub fn panel_inset_h() -> f32 {
    spacing_lg() - spacing_sm()
}
pub fn panel_inset_v() -> f32 {
    spacing_md()
}
pub fn section_spacing() -> f32 {
    spacing_md()
}
pub fn item_spacing() -> f32 {
    spacing_sm()
}
pub fn menu_item_pad_x() -> f32 {
    spacing_md() + spacing_sm()
}

// ── Radii ────────────────────────────────────────────────────────────────────

pub fn radius_button() -> f32 {
    theme::f32_or(tk::RADIUS_BUTTON, 6.0)
}
pub fn radius_panel() -> f32 {
    theme::f32_or(tk::RADIUS_PANEL, 12.0)
}
pub fn radius_control() -> f32 {
    theme::f32_or(tk::RADIUS_CONTROL, 4.0)
}
pub fn btn_radius_large() -> f32 {
    spacing_lg() + spacing_sm()
}
pub fn btn_radius_medium() -> f32 {
    spacing_md() + spacing_sm()
}
pub fn card_radius_large() -> f32 {
    spacing_md() + spacing_sm() + spacing_xs()
}

// ── Typography ───────────────────────────────────────────────────────────────

pub fn font_size_ui() -> f32 {
    theme::f32_or(tk::SIZE_FONT_UI, 13.0)
}
pub fn font_size_small() -> f32 {
    theme::f32_or(tk::SIZE_FONT_SMALL, 11.0)
}
pub fn font_size_header() -> f32 {
    font_size_ui() + 2.0
}
pub fn font_size_title() -> f32 {
    font_size_ui() + 7.0
}
