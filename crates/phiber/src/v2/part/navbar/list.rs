//! `BNavBarList` — Properties context tabs. Reads `tabs_for(host)` and
//! per-host state at `ctx.<host>.navbar.tab`, then emits ONE `BView::Rail` —
//! the same strip element the Toolbar, the Sidebar and the Activity bar
//! use. The expand state lives at `<host>.navbar.expanded`, shared with the
//! other strips via `part::strip`.

use super::set::tabs_for;
use crate::v2::host::{host_slug, BHostType};
use crate::v2::part::strip;
use crate::v2::state::{BPath, BState};
use crate::v2::view::{BView, BViewList};
use crate::v2::view::prim::{
    BAction, BControl, BRailEntry, BRailGroup, BRailKind, BRailSide, BRailView,
};

#[derive(Clone, Copy, Debug)]
pub struct BNavBarList {
    pub host: BHostType,
}

impl BNavBarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The whole strip as one view.
    pub fn rail(&self, s: &BState) -> BView {
        let tabs_spec = tabs_for(self.host);
        let default_tab = tabs_spec.first().map(|t| t.label).unwrap_or("");
        let active = s
            .host(self.host)
            .get("navbar.tab")
            .unwrap_or(default_tab);
        let expanded = strip::is_expanded(s, self.host, "navbar");

        let select_key = BPath::owned(vec![host_slug(self.host), "navbar", "tab"]);
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
            "navbar",
            select_key,
            &labels,
            active,
            expanded,
        ));
        BView::rail(BRailView {
            kind: BRailKind::NavBar,
            side: BRailSide::Left,
            expanded,
            entries,
        })
    }
}

impl BViewList for BNavBarList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}
