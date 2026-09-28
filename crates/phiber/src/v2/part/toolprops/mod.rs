//! Tool Properties — settings shown alongside the mode bar for the active tool.
//! Blender ref: `RGN_TYPE_TOOL_HEADER` content.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::part::toolbar::set::tools_for;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use self::set::props_for;
pub use set::BToolPropSpec;

// `BToolPropsItem` — one tool-property field (label; edited via a control).
#[derive(Clone, Copy, Debug)]
pub struct BToolPropsItem {
    pub spec: BToolPropSpec,
}

impl BToolPropsItem {
    pub fn new(spec: BToolPropSpec) -> Self {
        Self { spec }
    }
}

impl BViewItem for BToolPropsItem {
    fn build(&self, _s: &BState) -> BView {
        BView::label(self.spec.label)
    }
}

// `BToolPropsList` — reads `props_for(host, active_tool)` + host state.

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

#[derive(Clone, Copy, Debug)]
pub struct BToolProps {
    pub host: BHostType,
}

impl BToolProps {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BToolProps {
    fn build(&self, s: &BState) -> BView {
        let items = BToolPropsList::new(self.host).items(s);
        // A tool with no settings leaves the tool header nothing to show, and
        // the host drops the empty region rather than a bare strip.
        if items.is_empty() {
            return BView::empty();
        }
        BView::row(items)
    }
}
