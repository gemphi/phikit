//! `BToolbarList` — reads `tools_for(host)` + `host.<h>.tool` and emits ONE
//! `BView::Rail` (the shared vertical strip element) with a fixed bottom
//! expand/paging cluster.

use super::set::tools_for;
use crate::v2::host::{host_slug, BHostType};
use crate::v2::part::strip;
use crate::v2::state::{BPath, BState};
use crate::v2::view::{BView, BViewList};
use crate::v2::view::prim::{
    BAction, BControl, BRailEntry, BRailGroup, BRailKind, BRailSide, BRailView,
};

#[derive(Clone, Copy, Debug)]
pub struct BToolbarList {
    pub host: BHostType,
}

impl BToolbarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The whole strip as one view.
    pub fn rail(&self, s: &BState) -> BView {
        let tools = tools_for(self.host);
        let default_tool = tools.first().map(|t| t.label).unwrap_or("");
        let active = s.host(self.host).get("tool").unwrap_or(default_tool);
        let expanded = strip::is_expanded(s, self.host, "tool");

        let select_key = BPath::owned(vec![host_slug(self.host), "tool"]);
        let mut entries: Vec<BRailEntry> = tools
            .iter()
            .map(|t| BRailEntry {
                control: BControl::new()
                    .with_icon(t.icon)
                    .with_label(t.label)
                    .with_active(t.label == active)
                    .with_action(BAction::Set(select_key.clone(), t.label.to_string())),
                group: BRailGroup::Top,
            })
            .collect();
        let labels: Vec<&'static str> = tools.iter().map(|t| t.label).collect();
        entries.extend(strip::cluster_entries(
            self.host,
            "tool",
            select_key,
            &labels,
            active,
            expanded,
        ));
        BView::rail(BRailView {
            kind: BRailKind::Tools,
            side: BRailSide::Left,
            expanded,
            entries,
        })
    }
}

impl BViewList for BToolbarList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}
