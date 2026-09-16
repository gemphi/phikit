//! `BMenubarItem` — one top-level menu. Emits a `BView::Popover` whose
//! anchor is the menubar button and whose body is the item tree. Once ANY
//! top-level menu is open, all sibling buttons attach a hover-Open action
//! so pointer movement switches between menus (Blender behavior).

pub mod tree;

use super::set::BMenuSpec;
use crate::v2::host::{BHostType, host_slug};
use crate::v2::state::{BState, is_menubar_active_in, k_menu, key::P_MENU, val::*};
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};
use crate::v2::view::{BView, BViewItem};

/// One top-level menu, scoped to the host whose header renders it. The host
/// is part of the state key (`ctx.menu.<host>.<Label>`), so two hosts that
/// declare a menu with the same label — "View" appears in View3D, Outliner,
/// Properties, Image, Node and File — open and close independently.
#[derive(Clone, Copy, Debug)]
pub struct BMenubarItem {
    pub host: BHostType,
    pub spec: BMenuSpec,
}

impl BMenubarItem {
    pub fn new(host: BHostType, spec: BMenuSpec) -> Self {
        Self { host, spec }
    }
}

impl BViewItem for BMenubarItem {
    fn build(&self, s: &BState) -> BView {
        let host = host_slug(self.host);
        let key = k_menu(host, self.spec.label);
        let open = s.get(&key) == Some(V_TRUE);
        let toggle = BAction::Toggle(key.clone());
        let opener = BAction::Open(key.clone());
        let close = BAction::Set(key, V_EMPTY.to_string());
        // Hover-switching arms only once a menu in THIS host is open, so
        // sweeping the pointer over another editor's header does nothing.
        // The anchor spans the full header height: when it is open its accent
        // underline lands on the header's bottom border and the popover drops
        // from that same line.
        let hover = is_menubar_active_in(s, host).then_some(opener);
        let anchor = BView::MenuAnchor(
            BControl::new()
                .with_label(self.spec.label)
                .with_active(open)
                .with_action(toggle)
                .with_hover(hover),
        );
        let body = self
            .spec
            .items
            .iter()
            .map(|it| tree::item_view(&[P_MENU, host, self.spec.label], it, s))
            .collect();
        BView::popover(anchor, body, open, close, BPopoverSide::Below)
    }
}
