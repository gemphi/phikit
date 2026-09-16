//! Switcher — the editor-type dropdown in a host header (Blender's
//! `Editor Type` selector at the far left of every header).
//!
//! The trigger shows the area's current editor icon; clicking it deploys a
//! popover of the editor types that area may switch to. Selection writes
//! `ctx.area.<space>.<slot>.<default_host>`, which `BAreaItem::active_host`
//! reads on the next frame.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::area::current;
use crate::v2::host::BHostType;
use crate::v2::state::{
    BState,
    val::{V_EMPTY, V_TRUE},
};
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BSwitcherItem;
pub use list::BSwitcherList;
pub use set::BHostCategory;

#[derive(Clone, Copy, Debug)]
pub struct BSwitcher {
    pub host: BHostType,
}

impl BSwitcher {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BSwitcher {
    fn build(&self, s: &BState) -> BView {
        // Outside an area (e.g. the Topbar host) there is nothing to switch.
        let (Some(space), Some(area)) = (current::current_space(), current::current_area()) else {
            return BView::button(self.host.icon());
        };

        let open_key = current::switcher_key(space, &area);
        let slot = current::area_key(space, &area);
        let open = s.get(&open_key) == Some(V_TRUE);

        let anchor = BView::MenuAnchor(
            BControl::new()
                .with_icon(self.host.icon())
                .with_active(open)
                .with_action(BAction::Toggle(open_key.clone())),
        );
        let body = BSwitcherList::new(slot, open_key.clone(), area.hosts, self.host).items(s);
        BView::popover(
            anchor,
            body,
            open,
            BAction::Set(open_key, V_EMPTY.to_string()),
            BPopoverSide::BelowEnd,
        )
    }
}
