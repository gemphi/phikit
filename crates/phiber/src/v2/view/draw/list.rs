//! Focused renderer for `BView::ListRow` — the welcome window's recent files
//! and the Info editor's report log.
//!
//! The third of the three row builders. All three take their fill, their
//! hover and their band from the same tokens (`BFill::Row` / `BFill::RowHover`
//! → `tone::row_fill`), so a selected row is one thing in the app rather than
//! three that happen to agree.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFill, BFlex, BFlexed, BLen, BRadius, BSpace, BType};
use super::walk;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::BRowShade;
use gpui::prelude::*;
use gpui::AnyElement;

pub struct BList<'a> {
    pub list: &'a [BView],
    pub shade: BRowShade,
}

impl BList<'_> {
    pub fn render(&self, ctx: &BRenderCtx<'_>) -> AnyElement {
        let mut element = BListPart::Row(self.shade).flex(ctx).div(ctx);
        for child in self.list {
            element = element.child(walk(child, ctx));
        }
        element.into_any_element()
    }
}

/// The row band.
///
/// One geometry, selected or not: a row used to shrink by a border and gain a
/// margin when it became selected, so selecting or hovering one nudged it and
/// everything below it — the "dancing" on hover. State may change what a row
/// paints; it may never change what it measures, which is why the row always
/// carries the `Stroke` margin and the height that goes with it.
enum BListPart {
    /// The row band.
    Row(BRowShade),
}

impl BFlexed for BListPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        let Self::Row(shade) = *self;
        BFlex::row()
            .width(BLen::Full)
            .height(BLen::Measured(
                ctx.style.dimensions.row_height - 2.0 * ctx.style.strokes.border,
            ))
            .align(BAlign::Center)
            .pad_x(BSpace::PadMd)
            .gap(BSpace::GapXs)
            .fill(BFill::Row(shade))
            .hovered(BFill::RowHover(shade))
            .radius(BRadius::Role(BStyleRole::MenuItem))
            .margin_x(BSpace::Stroke)
            .margin_y(BSpace::Stroke)
            .ink()
            .type_size(BType::Body)
    }
}
