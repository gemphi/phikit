//! `BNavBarItem` — one Properties context tab. Emits a `BView::Tab` so
//! the walker can paint the active tab with the host's pane bg (paper-tab
//! continuation) and inactive ones as transparent on the darker strip.

use super::set::BNavTabSpec;
use crate::v2::host::{BHostType, host_slug};
use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BNavBarItem {
    pub host: BHostType,
    pub spec: BNavTabSpec,
    pub active: bool,
}

impl BNavBarItem {
    pub fn new(host: BHostType, spec: BNavTabSpec, active: bool) -> Self {
        Self { host, spec, active }
    }
}

impl BViewItem for BNavBarItem {
    fn build(&self, _s: &BState) -> BView {
        let key = BPath::owned(vec![host_slug(self.host), "navbar", "tab"]);
        // Icon-only tab with the context label as hover tooltip, shown to the
        // right of the strip when the cursor rests on the icon.
        BView::tab_with_tooltip(
            self.spec.icon,
            self.spec.label,
            self.active,
            BAction::Set(key, self.spec.label.to_string()),
        )
    }
}
