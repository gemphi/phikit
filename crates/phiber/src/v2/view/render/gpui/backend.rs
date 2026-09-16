//! GPUI backend entry point and one-style-per-render-pass setup.

use super::{BActionHandler, BRenderCtx, BResizeHandler, walk};
use crate::v2::state::{BState, key::K_THEME};
use crate::v2::theme::{BPaletteName, resolve_ui_style, ui_style_defaults};
use crate::v2::view::{BView, render::BRenderer};
use gpui::AnyElement;

pub struct BGpuiRenderer {
    pub handler: BActionHandler,
    pub resize: BResizeHandler,
}

impl BGpuiRenderer {
    pub fn new(handler: BActionHandler, resize: BResizeHandler) -> Self {
        Self { handler, resize }
    }

    fn palette_name(state: &BState) -> BPaletteName {
        state
            .get(&K_THEME)
            .and_then(BPaletteName::from_slug)
            .unwrap_or(BPaletteName::DEFAULT)
    }
}

impl BRenderer for BGpuiRenderer {
    type Output = AnyElement;

    fn render(&self, view: &BView, state: &BState) -> AnyElement {
        let palette_name = Self::palette_name(state);
        let style = resolve_ui_style(ui_style_defaults(palette_name), state);
        let context = BRenderCtx::new(&style, state, self.handler.clone(), self.resize.clone());
        walk(view, &context)
    }
}
