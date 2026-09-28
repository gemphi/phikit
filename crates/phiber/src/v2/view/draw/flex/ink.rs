//! The ink: the colour text is set in, and — through `BEdge::Ink` and
//! `BFill::Ink` — the colour a glyph is drawn in. One of the theme's text
//! tones or its accent, resolved here and nowhere else.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;

/// The text colour inside — and, through `BEdge::Ink` and `BFill::Ink`, the
/// colour a glyph is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BInk {
    /// The theme's text.
    Text,
    /// The theme's dim text.
    Dim,
    /// Dim text faded further: `step` steps of a 16th toward nothing, floored
    /// so the last step is still legible — the action log's older lines.
    Faded(u8),
    /// The theme's accent — a glyph that says "on".
    Accent,
}

impl BInk {
    pub(super) fn color(self, ctx: &BRenderCtx) -> gpui::Hsla {
        match self {
            Self::Text => to_hsla(ctx.style.colors.text),
            Self::Dim => to_hsla(ctx.style.colors.text_dim),
            Self::Faded(step) => {
                let dim = to_hsla(ctx.style.colors.text_dim);
                let fade = (1.0 - f32::from(step) / 16.0).max(0.45);
                gpui::Hsla { a: dim.a * fade, ..dim }
            }
            Self::Accent => to_hsla(ctx.style.colors.accent),
        }
    }
}
