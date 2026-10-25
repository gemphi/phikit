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

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BDim, BFill, BFlex, BInk, BLen, BRadius, BSpace, BType};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

/// The separator's ink: dim text faded seven sixteenths further (0.56 of a
/// label's alpha) — punctuation, quieter than the captions either side.
const SEPARATOR: BInk = BInk::Faded(7);

/// Words in one ink.
fn inked(ink: BInk, text: String, ctx: &BRenderCtx) -> gpui::Div {
    BFlex::row().ink_tone(ink).div(ctx).child(SharedString::from(text))
}

/// One `label: value`. In the status bar each one stands on a chip of its
/// own — a light, rounded ground, a control's height — so the bar reads as
/// a row of read-outs that can come and go, not a sentence.
///
/// Elsewhere — a details card — a pair takes the line, and a value longer
/// than it (a folder's path) ends in an ellipsis inside the card rather
/// than running out of it.
pub fn pair(label: &str, value: &str, ctx: &BRenderCtx) -> AnyElement {
    let chip = ctx.host == crate::v2::host::BHostType::Statusbar;
    let flex = if chip { CHIP } else { PAIR };
    let value = match chip {
        true => inked(BInk::Text, value.to_string(), ctx),
        false => BFlex::row().fills().min_width(BLen::Zero).ink_tone(BInk::Text).div(ctx).truncate().child(SharedString::from(value.to_string())),
    };
    flex.div(ctx).child(inked(BInk::Dim, format!("{label}:"), ctx)).child(value).into_any_element()
}

/// Several pairs on one line, `|` between them.
pub fn strip(pairs: &[(String, String)], ctx: &BRenderCtx) -> AnyElement {
    let mut line = STRIP.div(ctx);
    for (i, (label, value)) in pairs.iter().enumerate() {
        if i > 0 {
            line = line.child(inked(SEPARATOR, "|".to_string(), ctx));
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
    PAIR.div(ctx)
        .child(inked(BInk::Dim, lead.to_string(), ctx))
        .child(inked(BInk::Text, tail.to_string(), ctx))
        .into_any_element()
}

/// A pair: caption-sized, centred, its label and value a small gap apart.
const LINE: BFlex = BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).type_size(BType::Caption);
/// A pair in a card: the line's width, its label on the value's first line.
const PAIR: BFlex = LINE.fills().min_width(BLen::Zero).align(BAlign::Start);
/// A pair on its chip: the pair, padded, on the floating chrome's light
/// shade, rounded all round.
const CHIP: BFlex = LINE.pad_x(BSpace::PadSm).height(BLen::Dim(BDim::ButtonHeight)).radius(BRadius::Pill).fill(BFill::Floating);
/// A line of pairs: the same, the pairs a wider gap apart.
const STRIP: BFlex = BFlex::row().align(BAlign::Center).gap(BSpace::GapSm).type_size(BType::Caption);
