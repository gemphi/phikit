//! `BFlex` — the one layout element.
//!
//! Every container the renderer draws is described by a `BFlex`: its axis, how
//! it takes space in its parent, how it aligns and spaces its children, its
//! padding, its surface and its position. Each field is a *decision*, and
//! each number in it is a theme token — a `BSpace`, a `BDim`, a `BStyleRole` —
//! resolved against the render pass's style only here, in [`BFlex::div`].
//!
//! Nothing else constructs styled containers. A level does not write
//! `div().flex().flex_col().flex_1().min_h_0()`; it says what it is, through
//! [`BFlexed`], and the answer is applied in one place. That is what makes a
//! rule like "a Center fills" or "a header pads its sides" hold everywhere it
//! applies instead of wherever someone remembered to write it —
//! `tests/style_budget.rs` holds the rest of the renderer to it.

mod apply;
mod edge;
mod element;
mod fill;
mod ink;
mod paint;
mod place;
mod space;
mod share;
mod stateful;
mod surface;
mod token;

pub(super) use paint::shadowed;
pub use element::BFlex;
pub use fill::BFill;
pub use place::BOverflow;
pub use token::{
    BSelf,
    BAlign, BBasis, BDim, BEdge, BInk, BJustify, BLen, BPlace, BRadius, BSide,
    BSpace, BType, PROSE_HEADING_SCALE, PROSE_SCALE, PROSE_TITLE_SCALE,
};

use super::ctx::BRenderCtx;

/// Something the renderer draws, described by the layout it takes. Every
/// level implements it — the space, its zones, areas, hosts, regions, panes,
/// and the modals — so the question "how is this laid out?" has one answer,
/// asked the same way at every level.
pub trait BFlexed {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex;
}
