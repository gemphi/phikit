//! Where a container sits, what it clips, and the text defaults it hands
//! down — the tail of `BFlex::styled`, split out for the file cap.

use super::element::BFlex;
use super::token::{BPlace, BSide, BType};
use super::super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::px;

/// What happens to content that does not fit. Scrolling is not an overflow
/// mode here: a scroller keeps state, so it is built by [`BFlex::scroller`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BOverflow {
    /// Clip only if this container paints a rounded fill — the fill and the
    /// radius together promise the surface stops at the corner, and without
    /// clipping the children paint square over it. Nothing to clip to
    /// otherwise, so canvases and rails are left alone.
    Auto,
    /// Never clip: a child is meant to escape.
    Visible,
    /// Always clip.
    Clip,
}

impl BFlex {
    /// Text size, placement, overflow and dimming.
    pub(super) fn placed<E: Styled>(&self, element: E, ctx: &BRenderCtx) -> E {
        let mut element = element;
        match self.type_size {
            Some(BType::Body) => element = element.text_size(px(ctx.style.typography.body)),
            Some(BType::Header) => element = element.text_size(px(ctx.style.typography.header)),
            Some(BType::Caption) => element = element.text_size(px(ctx.style.typography.caption)),
            None => {}
        }
        element = match self.place {
            BPlace::Flow => element,
            BPlace::Anchor => element.relative(),
            BPlace::Overlay => element.absolute().inset_0(),
            BPlace::Drawer => element.absolute().bottom(gpui::relative(1.0)).left(gpui::Pixels::ZERO).right(gpui::Pixels::ZERO),
            BPlace::Float { side, inset, foot } => {
                let d = px(inset.px(ctx));
                let element = element.absolute().top(d).bottom(px(foot.px(ctx)));
                match side {
                    BSide::Left => element.left(d),
                    BSide::Right => element.right(d),
                }
            }
        };
        element = match self.overflow {
            // A rounded fill only reads as rounded if what is inside stops
            // at the same corner.
            BOverflow::Auto if self.fill.is_some() && self.radius.is_some() => element.overflow_hidden(),
            BOverflow::Auto | BOverflow::Visible => element,
            BOverflow::Clip => element.overflow_hidden(),
        };
        if self.dimmed {
            element = element.opacity(ctx.style.effects.grip_alpha);
        }
        element
    }
}
