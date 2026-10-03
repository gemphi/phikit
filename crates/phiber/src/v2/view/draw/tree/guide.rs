//! The hierarchy lines behind a tree.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

/// The hierarchy lines behind a tree, one per indent unit of the deepest row,
/// as tall as the rows they belong to. They are drawn in the theme's opposite
/// shade (`text_dim`: light on a dark theme, dark on a light one) so they read
/// against the rows they cross.
///
/// `rows` is that height. They used to span the whole column — `inset_0` in a
/// scroller is the scroller's height, not its content's — so on any tree
/// shorter than its pane the lines carried on down past the last row into
/// empty space, which is a hierarchy drawn for rows that are not there. A
/// guide belongs to the rows it guides.
pub fn guides_backdrop(depth: u16, rows: f32, ctx: &BRenderCtx) -> AnyElement {
    let unit = ctx.style.dimensions.indent_unit;
    let stroke = ctx.style.strokes.hairline;
    let mut backdrop = div().absolute().top_0().left_0().h(px(rows)).flex().flex_row();
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

