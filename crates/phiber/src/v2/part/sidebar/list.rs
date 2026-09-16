//! `BSidebarList` — reads `tabs_for(host)` + `host.<h>.sidebar.tab` and emits
//! ONE `BView::Rail` (the shared vertical strip element) with a fixed bottom
//! expand/paging cluster.

use super::{set::tabs_for};
use crate::v2::host::{host_slug, BHostType};
use crate::v2::part::strip;
use crate::v2::state::{BPath, BState};
use crate::v2::view::{BView, BViewList};
use crate::v2::view::prim::{
    BAction, BControl, BRailEntry, BRailGroup, BRailKind, BRailSide, BRailView,
};

#[derive(Clone, Copy, Debug)]
pub struct BSidebarList {
    pub host: BHostType,
}

impl BSidebarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The whole strip as one view.
    pub fn rail(&self, s: &BState) -> BView {
        let tabs_spec = tabs_for(self.host);
        let default_tab = tabs_spec.first().map(|t| t.label).unwrap_or("");
        let active = s
            .host(self.host)
            .get("sidebar.tab")
            .unwrap_or(default_tab);
        let expanded = strip::is_expanded(s, self.host, "sidebar");

        let select_key = BPath::owned(vec![host_slug(self.host), "sidebar", "tab"]);
        let mut entries: Vec<BRailEntry> = tabs_spec
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
        let labels: Vec<&'static str> = tabs_spec.iter().map(|t| t.label).collect();
        entries.extend(strip::cluster_entries(
            self.host,
            "sidebar",
            select_key,
            &labels,
            active,
            expanded,
        ));
        BView::rail(BRailView {
            kind: BRailKind::Sidebar,
            side: BRailSide::Right,
            expanded,
            entries,
        })
    }
}

impl BViewList for BSidebarList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}
