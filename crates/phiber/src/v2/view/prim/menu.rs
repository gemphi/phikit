//! Menu primitives — separators and popovers. Menu rows are `BView::MenuItem(BControl)`
//! built at call sites from the shared control record.

use crate::v2::view::BView;
use crate::v2::view::prim::BAction;

/// Popover placement relative to its anchor. `Below` for menubar drops,
/// `BelowEnd` for right-aligned host-switch drops, and `Right` for nested
/// submenus (Blender's `TOPBAR_MT_file_recover` etc).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPopoverSide {
    Below,
    BelowEnd,
    Right,
}

impl BView {
    pub fn menu_sep() -> Self {
        Self::MenuSep
    }
    /// The ONE popover constructor — placement is an explicit `side` value,
    /// not a constructor-per-placement family.
    pub fn popover(
        anchor: BView,
        body: Vec<BView>,
        open: bool,
        close: BAction,
        side: BPopoverSide,
    ) -> Self {
        Self::Popover {
            anchor: Box::new(anchor),
            body,
            open,
            close,
            side,
        }
    }
}

/// A header dropdown — `layout.prop(…, text="")` on an enum, or a
/// `template_ID`'s browse button: the anchor shows the choice (or only its
/// icon), and opens under it the list with the one in use ticked. Whether
/// it is open is `open_key` (a `MenuKey`), so opening another closes it.
#[derive(Clone, Debug)]
pub struct BPick {
    open_key: crate::v2::state::BPath,
    anchor: crate::v2::view::prim::BControl,
    items: Vec<BView>,
}

impl BPick {
    pub fn new(open_key: crate::v2::state::BPath, anchor: crate::v2::view::prim::BControl) -> Self {
        Self { open_key, anchor, items: Vec::new() }
    }

    /// One row: `label`, ticked when it is the one in use, doing `action`.
    pub fn item(mut self, label: &str, ticked: bool, action: BAction) -> Self {
        let row = crate::v2::view::prim::BControl::new().with_icon(if ticked { "check" } else { "" }).with_label(label).with_action(action);
        self.items.push(BView::MenuItem(row));
        self
    }

    pub fn view(self, s: &crate::v2::state::BState) -> BView {
        let open = s.get(&self.open_key) == Some(crate::v2::state::val::V_TRUE);
        let anchor = BView::MenuAnchor(self.anchor.with_active(open).with_action(BAction::Toggle(self.open_key.clone())));
        BView::popover(anchor, self.items, open, BAction::Set(self.open_key, crate::v2::state::val::V_EMPTY.to_string()), BPopoverSide::Below)
    }
}
