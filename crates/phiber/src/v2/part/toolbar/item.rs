//! `BToolbarItem` — one tool icon; carries a `BToolSpec` + active flag.
//!
//! Emits a `BView::tab_with_tooltip` (icon-only with a hover tooltip) so the
//! renderer paints the active tool with the pane bg (paper-tab continuation)
//! and inactive tools as transparent — the same visual as the Buts navbar /
//! Sidebar tabs. Clicking sets `host.<h>.tool` to the tool's label.

use super::set::BToolSpec;
use crate::v2::host::{host_slug, BHostType};
use crate::v2::state::{BPath, BState};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BToolbarItem {
    pub host: BHostType,
    pub spec: BToolSpec,
    pub active: bool,
}

impl BToolbarItem {
    pub fn new(host: BHostType, spec: BToolSpec, active: bool) -> Self {
        Self { host, spec, active }
    }
}

impl BViewItem for BToolbarItem {
    fn build(&self, _s: &BState) -> BView {
        let key = BPath::owned(vec![host_slug(self.host), "tool"]);
        BView::tab_with_tooltip(
            self.spec.icon,
            self.spec.label,
            self.active,
            BAction::Set(key, self.spec.label.to_string()),
        )
    }
}
