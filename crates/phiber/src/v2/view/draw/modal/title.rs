//! Modal title bar with existing minimize, maximize/restore, and close actions.

use crate::v2::state::{BKey as _, ModalKey};
use super::super::flex::BFlexed;
use super::super::ctx::BRenderCtx;
use super::super::surface::compatible_inner_radius;
use super::BModalPart;
use crate::v2::modal::BModalType;
use crate::v2::state::{
    val::V_EMPTY,
};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString};
use std::rc::Rc;

pub struct BTitle {
    pub kind: BModalType,
    pub maximized: bool,
    pub rounded: bool,
}

impl BTitle {
    /// A modal's window control — the same button the OS caption draws, with
    /// the modal's own action attached.
    fn button(&self, slug: &str, icon: &str, action: BAction, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        let handler = ctx.handler.clone();
        let dispatched = action;
        super::super::caption::control(
            SharedString::from(format!("modal-{slug}")),
            icon,
            slug == "close",
            ctx,
        )
        .on_click(move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
            (Rc::clone(&handler))(&dispatched, w, cx);
        })
    }

    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let (icon, label) = self.kind.heading(ctx.state);
        let title_row = BModalPart::TitleGroup
            .flex(ctx)
            .div(ctx)
            // `atom::icon` resolves the brand slugs, so a modal titled with
            // the app's mark shows the mark, not a flattened copy of it.
            .child(super::super::atom::icon(icon, ctx))
            .child(SharedString::from(label.to_string()));
        // A card modal's bar rounds its top corners to sit inside the card's
        // border; a fullscreen modal's bar is square to the window.
        let top_radius = self.rounded.then(|| {
            compatible_inner_radius(ctx.style.radius_for(BStyleRole::Modal), ctx.style.strokes.border)
        });
        BModalPart::TitleBar { top_radius }
            .flex(ctx)
            .div(ctx)
            .child(title_row)
            .child({
                let mut controls = BModalPart::TitleControls.flex(ctx).div(ctx);
                for ctl in super::super::caption::WINDOW_CONTROLS {
                    let action = match ctl.slug {
                        "min" => BAction::Toggle(ModalKey::Minimized.path()),
                        // Set, not Toggle: the button shows the state this
                        // modal is drawn in, so it always flips what is seen.
                        "max" => BAction::Set(
                            ModalKey::Maximized.path(),
                            if self.maximized { crate::v2::state::val::V_FALSE } else { crate::v2::state::val::V_TRUE }
                                .to_string(),
                        ),
                        _ => BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string()),
                    };
                    controls = controls.child(self.button(
                        ctl.slug,
                        ctl.icon(self.maximized),
                        action,
                        ctx,
                    ));
                }
                controls
            })
            .into_any_element()
    }
}
