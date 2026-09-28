//! The action log at the foot of the window (`state::log`).
//!
//! Folded, one row in the status bar: the newest line and the chevron that
//! unfolds it. Unfolded, a drawer opens upward over the workspace from that
//! row — `BPlace::Drawer`, `BDim::StatusLogHeight` tall, unfolding over the
//! theme's motion duration — holding every line, newest first, in a
//! scroller. Each line is a step dimmer down the list, so the eye lands on
//! the latest. The status bar itself never changes height: like Blender's
//! Info editor the log is a place to look, not a bar that grows.

use super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BInk, BJustify, BLen, BOverflow, BPlace, BRadius, BSpace, BType};
use super::ctx::BRenderCtx;
use super::button;
use crate::v2::state::log;
use crate::v2::theme::{BElevation, BStyleRole, BSurfaceTone};
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString};

enum BLogPart {
    /// The whole part: the head row, and the anchor the drawer opens from.
    Frame,
    /// The head row: the fold control and the latest line.
    Head,
    /// One line of text, taking the width it is given, cut with an ellipsis.
    Line { step: u8 },
    /// The drawer: floating over the workspace above the head, the status
    /// log's height, clipping the list it holds.
    Drawer,
    /// The unfolded list inside the drawer, one row per line.
    List,
}

impl BFlexed for BLogPart {
    fn flex(&self, _c: &BRenderCtx) -> BFlex {
        match self {
            Self::Frame => BFlex::column()
                .fills()
                .min_width(BLen::Zero)
                .height(BLen::Full)
                .place(BPlace::Anchor)
                .type_size(BType::Caption),
            Self::Head => BFlex::row()
                .align(BAlign::Center)
                .justify(BJustify::Start)
                .gap(BSpace::GapSm)
                .width(BLen::Full)
                .height(BLen::Full)
                .min_width(BLen::Zero),
            Self::Line { step } => BFlex::cell()
                .fills()
                .min_width(BLen::Zero)
                .overflow(BOverflow::Clip)
                .ink_tone(if *step == 0 { BInk::Dim } else { BInk::Faded(*step) }),
            Self::Drawer => BFlex::column()
                .place(BPlace::Drawer)
                .height(BLen::Dim(BDim::StatusLogHeight))
                .fill(BFill::Tone(BSurfaceTone::Modal))
                .border(BEdge::All)
                .radius(BRadius::Role(BStyleRole::Popover))
                .elevated(BElevation::Overlay)
                .overflow(BOverflow::Clip)
                .occludes(),
            Self::List => BFlex::column()
                .fills()
                .unclamped()
                .width(BLen::Full)
                .pad(BSpace::PadSm)
                .gap(BSpace::GapXs),
        }
    }
}

fn line(text: &str, step: u8, ctx: &BRenderCtx) -> AnyElement {
    BLogPart::Line { step }
        .flex(ctx)
        .div(ctx)
        .whitespace_nowrap()
        .text_ellipsis()
        .child(SharedString::from(text.to_string()))
        .into_any_element()
}

pub fn view(open: bool, toggle: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let entries = log::entries();
    let latest = entries.first().map_or("", |e| e.text.as_str());
    let head = BLogPart::Head
        .flex(ctx)
        .div(ctx)
        .id("status-log-head")
        .debug_selector(|| "status-log-head".to_string())
        .child(button::view(toggle, ctx))
        .child(line(latest, 0, ctx));
    let mut frame = BLogPart::Frame
        .flex(ctx)
        .div(ctx)
        .id("status-log")
        .debug_selector(|| "status-log".to_string())
        .child(head);
    if open {
        let mut list = BLogPart::List
            .flex(ctx)
            .scroller("status-log-list", ctx)
            .debug_selector(|| "status-log-list".to_string());
        for (i, entry) in entries.iter().enumerate() {
            // Each older line a step dimmer, down to a floor that stays legible.
            list = list.child(line(&entry.text, u8::try_from(i).unwrap_or(u8::MAX), ctx));
        }
        if entries.is_empty() {
            list = list.child(line("Nothing has happened yet.", 4, ctx));
        }
        let drawer = BLogPart::Drawer
            .flex(ctx)
            .div_unfolding("status-log-drawer", ctx)
            .map_element(|element| element.debug_selector(|| "status-log-drawer".to_string()).child(list));
        frame = frame.child(drawer);
    }
    frame.into_any_element()
}
