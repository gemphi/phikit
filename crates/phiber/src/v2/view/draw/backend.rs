//! GPUI backend entry point and one-style-per-render-pass setup.

use super::flex::BFlex;
use super::{BActionHandler, BRenderCtx, BResizeHandler, walk};
use crate::v2::state::BState;
use crate::v2::theme::resolve_ui_style;
use crate::v2::view::{BView, layout::BRenderer};
use gpui::prelude::*;
use gpui::{AnyElement, FontFallbacks};

pub struct BGpuiRenderer {
    pub handler: BActionHandler,
    pub resize: BResizeHandler,
}

impl BGpuiRenderer {
    pub fn new(handler: BActionHandler, resize: BResizeHandler) -> Self {
        Self { handler, resize }
    }

}

impl BRenderer for BGpuiRenderer {
    type Output = AnyElement;

    fn render(&self, view: &BView, state: &BState) -> AnyElement {
        // The theme in force, in its mode (`ctx.theme`, `ctx.theme_mode`).
        let style = resolve_ui_style(crate::v2::theme::ui_style_defaults_for(state), state);
        let context = BRenderCtx::new(&style, state, self.handler.clone(), self.resize.clone());
        // The typeface, once, at the root: every letter under it inherits
        // the family Preferences chose (`typography.family`), falling back
        // down that family's own stack when the machine lacks it.
        let family = style.typography.family;
        let mut root = BFlex::column().full().div(&context).font_family(family.family());
        root.text_style().font_fallbacks =
            Some(FontFallbacks::from_fonts(family.fallbacks().iter().map(|name| name.to_string()).collect()));
        root.child(walk(view, &context)).into_any_element()
    }
}
