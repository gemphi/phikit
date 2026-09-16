//! `BToolPropsList` — reads `props_for(host, active_tool)` + host state.

use super::{BToolPropsItem, set::props_for};
use crate::v2::host::BHostType;
use crate::v2::part::toolbar::set::tools_for;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BToolPropsList {
    pub host: BHostType,
}

impl BToolPropsList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BToolPropsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        // Active tool comes from state; fallback = first tool the toolbar
        // declares for this host (no hardcoded name).
        let fallback = tools_for(self.host).first().map(|t| t.label).unwrap_or("");
        let tool = s.host(self.host).get("tool").unwrap_or(fallback);
        props_for(self.host, tool)
            .iter()
            .map(|p| BToolPropsItem::new(*p).build(s))
            .collect()
    }
}
