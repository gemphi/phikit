//! `BSidebarItem` — one sidebar tab; carries a `BSidebarTabSpec` + active flag.
//!
//! Emits a `BView::tab_with_tooltip` (icon-only with a hover tooltip) so the
//! renderer paints the active tab with the paper-tab continuation and inactive
//! tabs as transparent. Same visual as the Buts navbar tabs.

use super::set::BSidebarTabSpec;
use crate::v2::host::{BHostType, host_slug};
use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BSidebarItem {
    pub host: BHostType,
    pub spec: BSidebarTabSpec,
    pub active: bool,
}

impl BSidebarItem {
    pub fn new(host: BHostType, spec: BSidebarTabSpec, active: bool) -> Self {
        Self { host, spec, active }
    }
}

impl BViewItem for BSidebarItem {
    fn build(&self, _s: &BState) -> BView {
        let key = BPath::owned(vec![host_slug(self.host), "sidebar", "tab"]);
        BView::tab_with_tooltip(
            self.spec.icon,
            self.spec.label,
            self.active,
            BAction::Set(key, self.spec.label.to_string()),
        )
    }
}
