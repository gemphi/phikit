//! The hierarchy lines behind a tree.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use crate::v2::theme::{BColor, BResolvedUiStyle};
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

/// How strongly a hierarchy line reads. `text_dim` on its own was too faint to
/// see; at this alpha the accent and the bright text that colour the levels
/// both stand out against the rows they cross.
const GUIDE_ALPHA: f32 = 0.55;

/// The colour of the hierarchy line at indent `level`. Adjacent levels differ
/// — the accent on even levels, the bright text tone on odd ones — so a deep
/// row's run of lines reads as distinct columns rather than one grey smear,
/// and each is visible against both zebra bands.
pub(super) fn guide_color(style: &BResolvedUiStyle, level: u16) -> BColor {
    let (h, s, l, _) = if level % 2 == 0 {
        style.colors.accent
    } else {
        style.colors.text
    };
    (h, s, l, GUIDE_ALPHA)
}

/// The hierarchy lines behind a tree, one per indent unit of the deepest row,
/// as tall as the rows they belong to. Each level takes its own colour
/// (`guide_color`) so the nesting is legible and visible.
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
    for level in 0..depth {
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
                        .bg(to_hsla(guide_color(ctx.style, level))),
                ),
        );
    }
    backdrop.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::BState;
    use crate::v2::theme::{BPaletteName, BThemeMode, resolve_ui_style, ui_style_defaults_in};

    #[test]
    fn adjacent_levels_take_different_visible_colours() {
        let style = resolve_ui_style(
            ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark),
            &BState::new(),
        );
        let even = guide_color(&style, 0);
        let odd = guide_color(&style, 1);
        assert_ne!((even.0, even.1, even.2), (odd.0, odd.1, odd.2), "levels differ in colour");
        assert_eq!(guide_color(&style, 2), even, "the colours alternate by level");
        assert!(even.3 >= 0.5 && odd.3 >= 0.5, "both are strong enough to see");
    }
}
