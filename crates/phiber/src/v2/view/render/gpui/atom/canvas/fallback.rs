//! Fallback canvas for hosts without a dedicated focused canvas.

use super::super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::host::BHostType;
use gpui::prelude::*;
use gpui::{AnyElement, div};

pub struct BFallback(pub BHostType);

impl BFallback {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        debug_assert!(
            !matches!(self.0, BHostType::File | BHostType::Asset),
            "File/Asset hosts must be routed through BCustomItem → BFileItem, not canvas"
        );
        div()
            .flex_1()
            .min_h_0()
            .size_full()
            .bg(to_hsla(c.style.viewport_surface()))
            .into_any_element()
    }
}

use crate::v2::view::render::gpui::prim::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BFallback {
    fn render(&self, c: &crate::v2::view::render::gpui::ctx::BRenderCtx) -> gpui::AnyElement {
        BFallback::render(self, c)
    }
}
