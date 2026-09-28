//! A strip: groups of controls placed at the start, the centre and the end of
//! one axis.
//!
//! A header row and a vertical rail are the same element. The header divides
//! its width three ways; the activity bar, the tool strip, the properties
//! navbar and the editor side rail divide their height the same three ways.
//! Only the axis differs, and whether the groups hug their content or share
//! out the slack — so both go through here rather than each growing its own
//! flex rules.

use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFlex, BJustify, BLen, BSpace};
use crate::v2::view::layout::BAxis;
use gpui::prelude::*;
use gpui::{AnyElement, Div};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BStripAxis {
    /// A header: groups run left to right.
    Row,
    /// A rail: groups run top to bottom.
    Column,
}

impl BStripAxis {
    fn axis(self) -> BAxis {
        match self {
            BStripAxis::Row => BAxis::Row,
            BStripAxis::Column => BAxis::Column,
        }
    }
}

/// How a group shares the strip's slack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BStripFit {
    /// Groups hug their content; the strip's own container divides the space.
    Hug,
    /// Groups take the slack: first at the start, last at the end, the rest
    /// centred between them. Their children sit on the strip's centre line,
    /// which is what a bar of controls wants.
    Divide,
    /// As `Divide`, but each group also stretches its children across the
    /// strip. A content row is columns, not controls: a file browser's
    /// bookmark column and its file table both run the full height, rather
    /// than floating at whatever height their own content happens to be.
    Fill,
}

/// One group's cell, at position `n` of `total`.
pub fn slot(
    axis: BStripAxis,
    fit: BStripFit,
    n: usize,
    total: usize,
    ctx: &BRenderCtx,
) -> Div {
    let mut element = BFlex::along(axis.axis()).min_width(BLen::Zero).gap(BSpace::GapXs);
    element = if fit == BStripFit::Fill {
        element.align(BAlign::Stretch).min_height(BLen::Zero)
    } else {
        element.align(BAlign::Center)
    };
    if fit == BStripFit::Hug {
        return element.keeps_size().div(ctx);
    }
    element = match (n, total) {
        (_, 1) => element.fills(),
        (0, 2) => element.fills().justify(BJustify::Start),
        (0, _) => element.keeps_size().justify(BJustify::Start),
        (n, total) if n + 1 == total => element.keeps_size().justify(BJustify::End),
        _ => element.fills().justify(BJustify::Center),
    };
    element.div(ctx)
}

/// The strip's own container: groups pushed apart along `axis`.
pub fn container(axis: BStripAxis, ctx: &BRenderCtx) -> Div {
    BFlex::along(axis.axis())
        .align(BAlign::Center)
        .min_height(BLen::Zero)
        .min_width(BLen::Zero)
        .justify(BJustify::Between)
        .gap(BSpace::GapXs)
        .div(ctx)
}

/// Place `groups` along `axis`, dropping the empty ones so a strip with only
/// a start and an end group still puts them at the two ends.
pub fn place(
    axis: BStripAxis,
    fit: BStripFit,
    groups: Vec<Vec<AnyElement>>,
    ctx: &BRenderCtx,
) -> Div {
    let filled: Vec<Vec<AnyElement>> = groups.into_iter().filter(|g| !g.is_empty()).collect();
    let total = filled.len();
    let mut element = container(axis, ctx);
    for (n, group) in filled.into_iter().enumerate() {
        let mut cell = slot(axis, fit, n, total, ctx);
        for child in group {
            cell = cell.child(child);
        }
        element = element.child(cell);
    }
    element
}
