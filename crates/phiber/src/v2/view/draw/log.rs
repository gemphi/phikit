//! The action log at the foot of the window (`state::log`).
//!
//! Folded, one row in the status bar: the newest line and the double
//! chevron that unfolds it. Unfolded, a drawer slides up over the
//! workspace from that row — `BPlace::Drawer`, `BDim::StatusLogHeight`
//! tall, unfolding over the theme's motion duration — holding a *table* of
//! every line: `#`, `When`, `Event`, under headings you can click to order
//! by (`part::statuslog::set`). The status bar itself never changes
//! height: like Blender's Info editor the log is a place to look, not a
//! bar that grows.
//!
//! It was a plain list before, and five hundred unlabelled lines is a wall
//! — you cannot tell what part of a row is what, and you cannot find the
//! one event you came for. The headings are the File Browser's
//! (`editors/file_browser`, "Sorting"), which is where the rest of the app
//! already puts orderable columns.

use super::flex::{BAlign, BDim, BEdge, BFill, BFlex, BFlexed, BInk, BJustify, BLen, BOverflow, BPlace, BRadius, BSpace, BType};
use super::ctx::BRenderCtx;
use super::button;
use crate::v2::part::statuslog::{self, BLogColumn, BLogOrder, COLUMNS};
use crate::v2::state::log;
use crate::v2::theme::{BElevation, BStyleRole, BSurfaceTone};
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

enum BLogPart {
    /// The whole part: the head row, and the anchor the drawer opens from.
    Frame,
    /// The head row: the fold control and the latest line.
    Head,
    /// One line of text, taking the width it is given, cut with an ellipsis.
    Line { step: u8 },
    /// The drawer: floating over the workspace above the head, the status
    /// log's height, clipping the table it holds.
    Drawer,
    /// The heading row — the only row that does not scroll away.
    Headings,
    /// The unfolded table inside the drawer, one row per line.
    List,
    /// One row of the table.
    Row,
    /// One cell, at its column's share of the width.
    Cell { share: f32, step: u8 },
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
            Self::Headings => BFlex::row()
                .align(BAlign::Center)
                .width(BLen::Full)
                .pad(BSpace::PadSm)
                .gap(BSpace::GapXs)
                .fill(BFill::Tone(BSurfaceTone::Header))
                .border(BEdge::Bottom),
            Self::List => BFlex::column()
                .fills()
                .unclamped()
                .width(BLen::Full)
                .pad(BSpace::PadSm)
                .gap(BSpace::GapXs),
            Self::Row => BFlex::row().align(BAlign::Center).width(BLen::Full).gap(BSpace::GapXs).min_width(BLen::Zero),
            Self::Cell { share, step } => BFlex::cell()
                .width(BLen::Share(*share))
                .min_width(BLen::Zero)
                .overflow(BOverflow::Clip)
                .ink_tone(if *step == 0 { BInk::Dim } else { BInk::Faded(*step) }),
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

fn cell(text: &str, column: BLogColumn, step: u8, ctx: &BRenderCtx) -> AnyElement {
    BLogPart::Cell { share: column.share, step }
        .flex(ctx)
        .div(ctx)
        .whitespace_nowrap()
        .text_ellipsis()
        .child(SharedString::from(text.to_string()))
        .into_any_element()
}

/// The heading row: each column's name, and on the one being ordered by,
/// which way round it is.
fn headings(ctx: &BRenderCtx) -> AnyElement {
    let by = statuslog::order(ctx.state);
    let up = statuslog::ascending(ctx.state);
    let mut row = BLogPart::Headings.flex(ctx).div(ctx).id("status-log-headings");
    for column in COLUMNS {
        let active = column.orders && by == column.order;
        let mut control = BControl::new().with_label(column.label);
        if active {
            control = control.with_icon(if up { "chevron_up" } else { "chevron_down" }).with_active(true);
        }
        if let Some(action) = statuslog::heading_action(ctx.state, *column) {
            control = control.with_action(action);
        }
        row = row.child(
            BLogPart::Cell { share: column.share, step: 0 }
                .flex(ctx)
                .div(ctx)
                .child(button::view(&control, ctx)),
        );
    }
    row.into_any_element()
}

pub fn view(open: bool, toggle: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let entries = log::entries();
    let latest = entries.first().map_or("", |e| e.text.as_str());
    // The latest line carries the same toggle the chevron does. The
    // chevron is a small target in a thin bar, and clicking the line
    // itself did nothing — which reads as the bar refusing to open. The
    // line and not the whole row, so a click on the chevron is not also a
    // click on its parent and does not toggle twice.
    let head = BLogPart::Head
        .flex(ctx)
        .div(ctx)
        .id("status-log-head")
        .debug_selector(|| "status-log-head".to_string())
        .child(button::view(toggle, ctx))
        .child(match toggle.action.clone() {
            Some(action) => {
                let handler = ctx.handler.clone();
                BLogPart::Line { step: 0 }
                    .flex(ctx)
                    .div(ctx)
                    .id("status-log-latest")
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .cursor_pointer()
                    .child(SharedString::from(latest.to_string()))
                    .on_click(move |_: &ClickEvent, window, app| (Rc::clone(&handler))(&action, window, app))
                    .into_any_element()
            }
            None => line(latest, 0, ctx),
        });
    let mut frame = BLogPart::Frame
        .flex(ctx)
        .div(ctx)
        .id("status-log")
        .debug_selector(|| "status-log".to_string())
        .child(head);
    if open {
        let by = statuslog::order(ctx.state);
        let up = statuslog::ascending(ctx.state);
        let rows = statuslog::sorted(&entries, by, up);
        let mut list = BLogPart::List
            .flex(ctx)
            .scroller("status-log-list", ctx)
            .debug_selector(|| "status-log-list".to_string());
        let newest = entries.first().map_or(0, |e| e.seq);
        for entry in &rows {
            // Each older line a step dimmer, down to a floor that stays
            // legible — by age, not by position, so ordering the table a
            // different way does not re-shade it.
            let step = u8::try_from(newest.saturating_sub(entry.seq)).unwrap_or(u8::MAX);
            let when = entry.at.elapsed();
            let mut row = BLogPart::Row.flex(ctx).div(ctx);
            for column in COLUMNS {
                let text = match column.order {
                    BLogOrder::Seq => entry.seq.to_string(),
                    BLogOrder::When => format!("{:.1}s", when.as_secs_f32()),
                    BLogOrder::Event => entry.text.clone(),
                };
                row = row.child(cell(&text, *column, step, ctx));
            }
            list = list.child(row);
        }
        if rows.is_empty() {
            list = list.child(line("Nothing has happened yet.", 4, ctx));
        }
        let drawer = BLogPart::Drawer
            .flex(ctx)
            .div_unfolding("status-log-drawer", ctx)
            .map_element(|element| {
                element
                    .debug_selector(|| "status-log-drawer".to_string())
                    .child(headings(ctx))
                    .child(list)
            });
        frame = frame.child(drawer);
    }
    frame.into_any_element()
}
