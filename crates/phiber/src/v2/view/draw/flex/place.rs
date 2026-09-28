//! Where a container sits, what it clips, and the text defaults it hands
//! down — the tail of `BFlex::styled`, split out for the file cap.

use super::element::BFlex;
use super::token::{BOverflow, BPlace, BSide, BType};
use super::super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::px;

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
            BPlace::Float { side, inset } => {
                let d = px(inset.px(ctx));
                let element = element.absolute().top(d).bottom(d);
                match side {
                    BSide::Left => element.left(d),
                    BSide::Right => element.right(d),
                }
            }
        };
        element = match self.overflow {
            BOverflow::Visible => element,
            BOverflow::Clip => element.overflow_hidden(),
        };
        if self.dimmed {
            element = element.opacity(ctx.style.effects.grip_alpha);
        }
        element
    }
}
