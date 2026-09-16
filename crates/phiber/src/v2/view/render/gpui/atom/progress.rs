//! Progress-bar atom — a determinate bar with a leading status label.
//!
//! Blender's file-read job paints a thin progress bar in the status bar with
//! a short label; this atom mirrors that. It resolves all geometry and colour
//! from the render-pass style (no design literals): the track uses the recessed
//! panel tone, the fill uses the accent, and the label uses the dim text tone.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::theme::BStyleRole;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

/// Render a `[label] [====      ]` progress row. `fraction` is clamped to
/// `0.0..=1.0`. The bar is a fixed-width track (the statusbar segment sizes
/// it) whose accent fill grows with `fraction`; `relative(fraction)` lets the
/// fill track the resolved track width without a hardcoded pixel span.
pub fn view(fraction: f32, label: &str, c: &BRenderCtx) -> AnyElement {
    let frac = fraction.clamp(0.0, 1.0);
    let bar_width = c.style.dimensions.channels_width; // reuse a resolved medium width
    let bar_height = c.style.typography.body;
    let radius = px(c.style.radius_for(BStyleRole::Button));

    let track = div()
        .w(px(bar_width))
        .h(px(bar_height))
        .rounded(radius)
        .overflow_hidden()
        .bg(to_hsla(c.style.colors.bg_panel))
        .child(
            div()
                .h_full()
                .w(gpui::relative(frac))
                .bg(to_hsla(c.style.colors.accent)),
        );

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(c.style.spacing.gap_sm))
        .text_color(to_hsla(c.style.colors.text_dim))
        .text_size(px(c.style.typography.body))
        .child(SharedString::from(label.to_string()))
        .child(track)
        .into_any_element()
}
