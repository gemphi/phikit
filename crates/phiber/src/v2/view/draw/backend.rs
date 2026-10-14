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

/// The complete style for this pass: the theme in force (`ctx.theme`,
/// `ctx.theme_mode`) resolved, then worn in the window's dimensional coat.
///
/// The coat (`ctx.flavour`) is set once, for the whole window — flat, or the
/// spatial virtual-world look — and overrides the palette's own depth cues so
/// any theme can be worn either way. `Auto`, the default, leaves them alone
/// (`theme/style/flavour`).
pub(crate) fn resolved_style(state: &BState) -> crate::v2::theme::BResolvedUiStyle {
    let mut style = resolve_ui_style(crate::v2::theme::ui_style_defaults_for(state), state);
    let scope = state
        .get(crate::v2::state::AppKey::Flavour)
        .and_then(crate::v2::theme::BFlavourScope::from_slug)
        .unwrap_or_default();
    if let Some(coat) = scope.coat(crate::v2::space::BSpaceList::active(state)) {
        style.effects = coat.wear(style.effects);
    }
    style
}

impl BRenderer for BGpuiRenderer {
    type Output = AnyElement;

    fn render(&self, view: &BView, state: &BState) -> AnyElement {
        // The theme in force, in its mode and dimensional coat.
        let style = resolved_style(state);
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

#[cfg(test)]
mod tests {
    use super::resolved_style;
    use crate::v2::state::{AppKey, BState, CtxKey};
    use crate::v2::theme::BShadowStyle;

    #[test]
    fn auto_leaves_the_palette_s_depth_cues_alone() {
        // No coat chosen: the window looks exactly as the theme resolves it.
        let state = BState::new();
        let bare = crate::v2::theme::resolve_ui_style(crate::v2::theme::ui_style_defaults_for(&state), &state);
        assert_eq!(resolved_style(&state).effects, bare.effects);
    }

    #[test]
    fn the_spatial_coat_reaches_the_whole_window() {
        // Forced spatial: gradients on and something cast, whatever space is
        // active — the coat is the window's, not one host's.
        let mut state = BState::new();
        state.set(AppKey::Flavour, "spatial");
        state.set(CtxKey::Space, "Shading");
        let effects = resolved_style(&state).effects;
        assert!(effects.gradients, "the whole window sweeps");
        assert_ne!(effects.surface_shadow, BShadowStyle::None, "and casts");
    }

    #[test]
    fn the_flat_coat_strips_every_depth_cue() {
        // Flat forces both off from any base, so a spatial theme comes out
        // flat everywhere.
        let mut state = BState::new();
        state.set(AppKey::Theme, "Sphere");
        state.set(AppKey::Flavour, "flat");
        let effects = resolved_style(&state).effects;
        assert!(!effects.gradients);
        assert_eq!(effects.surface_shadow, BShadowStyle::None);
    }
}
