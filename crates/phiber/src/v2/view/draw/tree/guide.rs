//! The hierarchy lines behind a tree.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

/// The hierarchy lines behind a tree, one per indent unit of the deepest
/// row, spanning the whole column rather than stopping at the last row. They
/// are drawn in the theme's opposite shade (`text_dim`: light on a dark
/// theme, dark on a light one) so they read against the rows they cross.
pub fn guides_backdrop(depth: u16, ctx: &BRenderCtx) -> AnyElement {
    let unit = ctx.style.dimensions.indent_unit;
    let stroke = ctx.style.strokes.hairline;
    let mut backdrop = div().absolute().inset_0().flex().flex_row();
    for _ in 0..depth {
        backdrop = backdrop.child(
            div()
                .flex_shrink_0()
                .relative()
                .w(px(unit))
                .h_full()
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(px((unit - stroke) / 2.0))
                        .w(px(stroke))
                        .bg(to_hsla(ctx.style.colors.text_dim)),
                ),
        );
    }
    backdrop.into_any_element()
}

