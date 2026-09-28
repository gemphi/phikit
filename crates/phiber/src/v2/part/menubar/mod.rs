//! Menubar — horizontal row of menu buttons.
//! Layout is driven by the host's BPaneSpec; the Switcher part separately
//! provides the editor-type icon on the left pane.

pub mod editor;
pub mod item;
pub mod op;
pub mod set;
pub mod topbar;
pub mod tree;

use crate::v2::host::BHostType;
use crate::v2::state::BKey as _;
use crate::v2::state::BState;
use crate::v2::state::MenuKey;
use crate::v2::state::is_menubar_active_in;
use crate::v2::state::val::*;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BPopoverSide;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
pub use item::BMenuSpec;

// `BMenubarItem` — one top-level menu. Emits a `BView::Popover` whose
// anchor is the menubar button and whose body is the item tree. Once ANY
// top-level menu is open, all sibling buttons attach a hover-Open action
// so pointer movement switches between menus (Blender behavior).
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
        let host = self.host.slug();
        if let Some(body) = self.spec.panel {
            // A panel popover: its own key, so ticking a box inside keeps it
            // open; the anchor is the same button a menu has.
            let key = MenuKey::Panel(host, self.spec.label).path();
            let open = s.get(&key) == Some(V_TRUE);
            let toggle = BAction::Set(key.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
            let anchor = BView::MenuAnchor(BControl::new().with_label(self.spec.label).with_active(open).with_action(toggle));
            return BView::popover(anchor, body(s), open, BAction::Set(key, V_FALSE.to_string()), BPopoverSide::Below);
        }
        let key = MenuKey::Top(host, self.spec.label).path();
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
        // A marked menu (the app menu) shows its mark beside its name, the
        // way Blender's logo sits at the head of the top bar.
        let mut control = BControl::new()
            .with_label(self.spec.label)
            .with_active(open)
            .with_action(toggle)
            .with_hover(hover);
        if let Some(icon) = self.spec.mark {
            control = control.with_icon(icon).as_brand();
        }
        let anchor = BView::MenuAnchor(control);
        let body = self
            .spec
            .items
            .iter()
            .map(|it| tree::item_view(host, &[self.spec.label], it, s))
            .collect();
        BView::popover(anchor, body, open, close, BPopoverSide::Below)
    }
}

// `BMenubarList` — reads `BMenuSpec::list_for(host)` and delegates to `BMenubarItem`.

#[derive(Clone, Copy, Debug)]
pub struct BMenubarList {
    pub host: BHostType,
}

impl BMenubarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BMenubarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        BMenuSpec::list_for(self.host)
            .iter()
            .map(|m| BMenubarItem::new(self.host, *m).build(s))
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BMenubar {
    pub host: BHostType,
}

impl BMenubar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BMenubar {
    fn build(&self, s: &BState) -> BView {
        BView::row(BMenubarList::new(self.host).items(s))
    }
}
