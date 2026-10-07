//! GPUI backend entry point and one-style-per-render-pass setup.

use super::flex::BFlex;
use super::ctx::BInputWiring;
use super::{BActionHandler, BRenderCtx, BResizeHandler, walk};
use crate::v2::state::BState;
use crate::v2::theme::resolve_ui_style;
use crate::v2::view::{BView, layout::BRenderer};
use gpui::prelude::*;
use gpui::{AnyElement, FontFallbacks};

pub struct BGpuiRenderer {
    pub handler: BActionHandler,
    pub resize: BResizeHandler,
    /// The window entity + focus, when a real window drives the pass, so the
    /// editing field can wire text input. `None` in headless renders.
    pub input: Option<BInputWiring>,
}

impl BGpuiRenderer {
    pub fn new(handler: BActionHandler, resize: BResizeHandler, input: Option<BInputWiring>) -> Self {
        Self { handler, resize, input }
    }

}

impl BRenderer for BGpuiRenderer {
    type Output = AnyElement;

    fn render(&self, view: &BView, state: &BState) -> AnyElement {
        // The theme in force, in its mode (`ctx.theme`, `ctx.theme_mode`).
        let style = resolve_ui_style(crate::v2::theme::ui_style_defaults_for(state), state);
        let context = BRenderCtx::new(&style, state, self.handler.clone(), self.resize.clone())
            .with_input(self.input.clone());
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
