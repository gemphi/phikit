//! A path as breadcrumbs — Explorer's address bar: This PC › Local Disk
//! (C:) › Users › …, each folder a link to itself, in the field's own box.
//! A click on the box between the crumbs puts the caret in the path, which
//! is then typed as the directory field is (`file::path_bar`).

use super::ctx::BRenderCtx;
use super::field;
use super::flex::{BAlign, BFill, BFlex, BFlexed, BLen, BOverflow, BRadius, BSpace, BType};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

/// The bar's own containers, named once.
enum BCrumbPart {
    /// The crumbs in a line, clipped from the start's side when the path is
    /// longer than the box.
    Trail,
    /// One folder: its name on a hover ground, the field's corner.
    Crumb,
}

impl BFlexed for BCrumbPart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Trail => BFlex::row().align(BAlign::Center).gap(BSpace::GapXs).grows().min_width(BLen::Zero).overflow(BOverflow::Clip),
            Self::Crumb => BFlex::row()
                .align(BAlign::Center)
                .pad_x(BSpace::PadXs)
                .radius(BRadius::Role(BStyleRole::Field))
                .keeps_size()
                .hovered(BFill::ItemHover)
                .type_size(BType::Body)
                .ink(),
        }
    }
}

pub fn view(trail: &[(String, BAction)], edit: &[BAction], ctx: &BRenderCtx) -> AnyElement {
    let mut line = BCrumbPart::Trail.flex(ctx).div(ctx);
    for (n, (name, action)) in trail.iter().enumerate() {
        if n > 0 {
            line = line.child(super::atom::icon_in("chevron_right", ctx.style.colors.text_dim, ctx));
        }
        let handler = ctx.handler.clone();
        let action = action.clone();
        let id = format!("crumb-{name}");
        line = line.child(
            BCrumbPart::Crumb
                .flex(ctx)
                .div(ctx)
                .id(SharedString::from(id.clone()))
                .debug_selector(move || id)
                .cursor_pointer()
                .child(SharedString::from(name.clone()))
                .on_click(move |_: &ClickEvent, window, app| {
                    // The crumb's folder, not the box's typing under it.
                    app.stop_propagation();
                    (Rc::clone(&handler))(&action, window, app);
                }),
        );
    }
    let handler = ctx.handler.clone();
    let edit = edit.to_vec();
    field::shell(false, ctx)
        .id(SharedString::from("crumbs"))
        .debug_selector(|| "crumbs".to_string())
        .cursor_text()
        .child(line)
        .on_click(move |_: &ClickEvent, window, app| {
            for action in &edit {
                (Rc::clone(&handler))(action, window, app);
            }
        })
        .into_any_element()
}
