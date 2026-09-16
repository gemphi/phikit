//! `BSwitcherItem` — one row in the editor-type popover. Selecting it writes
//! the target host slug into the enclosing area's slot, which is what makes
//! `BAreaItem::active_host` resolve to a different editor on the next frame.

use crate::v2::host::BHostType;
use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Debug)]
pub struct BSwitcherItem {
    /// The host this row selects.
    pub host: BHostType,
    /// The area slot the selection is written to.
    pub slot: BPath,
    /// Whether this row is the area's current editor type.
    pub active: bool,
}

impl BSwitcherItem {
    pub fn new(host: BHostType, slot: BPath, active: bool) -> Self {
        Self { host, slot, active }
    }
}

impl BViewItem for BSwitcherItem {
    fn build(&self, _s: &BState) -> BView {
        // The active editor type shows a check in place of its own icon,
        // matching how the mode dropdown marks its current entry.
        let icon = if self.active {
            "check"
        } else {
            self.host.icon()
        };
        BView::menu_item(
            icon,
            self.host.label(),
            "",
            BAction::Set(
                self.slot.clone(),
                crate::v2::host::host_slug(self.host).to_string(),
            ),
        )
    }
}
