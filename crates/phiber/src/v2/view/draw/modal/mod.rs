//! Modal dispatcher — routes to `fullscreen` (Splash/About) or `card`
//! (all other modals). GPUI bodies live in focused modules.

pub mod card;
pub mod fullscreen;
mod part;
pub(super) use part::BModalPart;
pub mod title;
pub mod welcome;

use crate::v2::state::{BKey as _, ModalKey};
use super::ctx::BRenderCtx;
use crate::v2::modal::BModalType;
use crate::v2::state::{
    val::V_TRUE,
};
use crate::v2::view::BView;
use crate::v2::view::prim::BAction;
use gpui::AnyElement;

pub(super) fn is_maximized(ctx: &BRenderCtx) -> bool {
    ctx.state.get(ModalKey::Maximized) == Some(V_TRUE)
}

/// A fullscreen modal (Welcome, About) arrives filling the window; only an
/// explicit Restore makes it a window of its own.
pub(super) fn is_fullscreen_maximized(ctx: &BRenderCtx) -> bool {
    ctx.state.get(ModalKey::Maximized) != Some(crate::v2::state::val::V_FALSE)
}

pub(super) fn close_action() -> BAction {
    use crate::v2::state::val::V_EMPTY;
    BAction::Set(ModalKey::Open.path(), V_EMPTY.to_string())
}

pub fn view(kind: BModalType, body: &BView, ctx: &BRenderCtx) -> AnyElement {
    match kind {
        BModalType::Splash | BModalType::About => fullscreen::BFullscreen { kind, body }.render(ctx),
        _ => card::BCard { kind, body }.render(ctx),
    }
}
