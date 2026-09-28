//! Fallback canvas for hosts without a dedicated focused canvas.

use super::super::ctx::BRenderCtx;
use crate::v2::host::BHostType;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BFallback(pub BHostType);

impl BFallback {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        debug_assert!(
            !matches!(self.0, BHostType::File | BHostType::Asset),
            "File/Asset hosts must be routed through BCustomItem → BFileItem, not canvas"
        );
        super::base(ctx)
            .bg(super::super::color::resolved_gradient(ctx.style.viewport_gradient()))
            .into_any_element()
    }
}
