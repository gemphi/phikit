//! `BModebarItem` — one mode row inside the modebar dropdown popover.
//! Renders as a menu row that fires `Set(ctx.mode, <label>)` on click; the
//! active mode shows a check icon.

use super::set::{BModeSpec, mode_key};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BModebarItem {
    pub host: BHostType,
    pub spec: BModeSpec,
    pub active: bool,
}

impl BModebarItem {
    pub fn new(host: BHostType, spec: BModeSpec, active: bool) -> Self {
        Self { host, spec, active }
    }
}

impl BViewItem for BModebarItem {
    fn build(&self, _s: &BState) -> BView {
        let icon = if self.active { "check" } else { self.spec.icon };
        BView::menu_item(
            icon,
            self.spec.label,
            "",
            BAction::Set(mode_key(self.host), self.spec.label.to_string()),
        )
    }
}
