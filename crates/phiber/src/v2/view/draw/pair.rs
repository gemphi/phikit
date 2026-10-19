//! A read-out's `label: value`, in two shades.
//!
//! Every status strip in the app says the same kind of thing — a name and
//! a number — and every one of them used to say it in one colour, as a run
//! of text: `Objects: 995 | Verts: 1,266,143 | Faces: 1,242,745`. Read at a
//! glance that is a hedge, not a read-out: the words and the figures weigh
//! the same, so the eye has to parse the sentence to find the number it
//! came for, and the numbers shift sideways as the words around them
//! change width.
//!
//! A pair is drawn in two shades instead — the label at `text_dim`, the
//! value at `text` — so the figures carry and the words recede to being
//! what they are, captions on them. The split lives here rather than in
//! each strip, because the viewport's overlay, the status bar's segments
//! and the render window's info bar are three places that must look like
//! one app.
//!
//! The separator between pairs is dimmer still: it is punctuation, and
//! punctuation at full strength competes with the figures either side of
//! it.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

/// How much dimmer the separator is than a label, as a share of its alpha.
const SEPARATOR_FADE: f32 = 0.55;

/// One `label: value`.
pub fn pair(label: &str, value: &str, ctx: &BRenderCtx) -> AnyElement {
    let dim = ctx.style.colors.text_dim;
    let bright = ctx.style.colors.text;
    row_of(ctx)
        .child(
            div()
                .text_color(to_hsla(dim))
                .child(SharedString::from(format!("{label}:"))),
        )
        .child(div().text_color(to_hsla(bright)).child(SharedString::from(value.to_string())))
        .into_any_element()
}

/// Several pairs on one line, `|` between them.
pub fn strip(pairs: &[(String, String)], ctx: &BRenderCtx) -> AnyElement {
    let mut sep = ctx.style.colors.text_dim;
    sep.3 *= SEPARATOR_FADE;
    let mut line = row_of(ctx).gap(px(ctx.style.spacing.gap_sm));
    for (i, (label, value)) in pairs.iter().enumerate() {
        if i > 0 {
            line = line.child(div().text_color(to_hsla(sep)).child(SharedString::from("|")));
        }
        line = line.child(pair(label, value, ctx));
    }
    line.into_any_element()
}

/// A line whose leading words are the caption and whose tail is the value —
/// for the read-outs Blender words as a phrase rather than as `name: n`
/// ("(Scene) Collection | Cube"), which keep their wording and still get
/// the two shades.
pub fn phrase(lead: &str, tail: &str, ctx: &BRenderCtx) -> AnyElement {
    row_of(ctx)
        .child(div().text_color(to_hsla(ctx.style.colors.text_dim)).child(SharedString::from(lead.to_string())))
        .child(div().text_color(to_hsla(ctx.style.colors.text)).child(SharedString::from(tail.to_string())))
        .into_any_element()
}

fn row_of(ctx: &BRenderCtx) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(ctx.style.spacing.gap_xs))
        .text_size(px(ctx.style.typography.caption))
}
