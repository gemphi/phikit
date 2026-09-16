//! Modal dispatcher — routes to `fullscreen` (Splash/About) or `card`
//! (all other modals). GPUI bodies live in focused modules.

pub mod card;
pub mod fullscreen;
pub mod title;
pub mod welcome;

use super::ctx::BRenderCtx;
use crate::v2::modal::BModalType;
use crate::v2::state::{
    key::{K_MODAL, K_MODAL_MAXIMIZED},
    val::V_TRUE,
};
use crate::v2::view::BView;
use crate::v2::view::prim::BAction;
use gpui::AnyElement;

pub(super) fn is_maximized(c: &BRenderCtx) -> bool {
    c.s.get(&K_MODAL_MAXIMIZED) == Some(V_TRUE)
}

pub(super) fn close_action() -> BAction {
    use crate::v2::state::val::V_EMPTY;
    BAction::Set(K_MODAL, V_EMPTY.to_string())
}

pub fn view(kind: BModalType, body: &BView, c: &BRenderCtx) -> AnyElement {
    match kind {
        BModalType::Splash | BModalType::About => fullscreen::BFullscreen { kind, body }.render(c),
        _ => card::BCard { kind, body }.render(c),
    }
}
