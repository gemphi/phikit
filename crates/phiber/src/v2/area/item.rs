//! `BAreaItem` — one area. Resolves the active host and delegates to `BHostItem`.

use super::{BAreaSpec, current};
use crate::v2::host::{BHostItem, BHostType};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BAreaItem {
    pub area: BAreaSpec,
}

impl BAreaItem {
    pub fn new(area: BAreaSpec) -> Self {
        Self { area }
    }

    /// The editor type this area is showing: whatever the switcher wrote into
    /// this area's slot, else the area's declared default.
    fn active_host(&self, s: &BState) -> BHostType {
        let Some(space) = current::current_space() else {
            return self.area.default_host;
        };
        s.get(&current::area_key(space, &self.area))
            .and_then(BHostType::from_slug)
            .unwrap_or(self.area.default_host)
    }
}

impl BViewItem for BAreaItem {
    fn build(&self, state: &BState) -> BView {
        let host = self.active_host(state);
        // Record this area for the whole subtree so the switcher in the
        // host header can address the slot it lives in.
        current::with_area(self.area, || BView::Area {
            kind: self.area.kind,
            child: Box::new(BHostItem::new(host).build(state)),
        })
    }
}
