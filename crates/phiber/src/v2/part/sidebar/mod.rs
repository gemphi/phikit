//! Sidebar — the vertical tab strip and, while it is open, the region of
//! panels beside it (`show.rs`, `panel.rs`).

pub mod panel;
pub mod set;
pub mod show;

use crate::v2::host::BHostType;
use crate::v2::part::strip;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BRailEntry;
use crate::v2::view::prim::BRailGroup;
use crate::v2::view::prim::BRailKind;
use crate::v2::view::prim::BRailSide;
use crate::v2::view::prim::BRailView;
use self::panel::BSidebarPanels;
use self::set::tabs_for;
use self::show::BSidebarShown;
use crate::v2::state::val::V_FALSE;
pub use set::{BSidebarBody, BSidebarTabSpec};

// `BSidebarList` — reads `tabs_for(host)` + `host.<h>.sidebar.tab` and emits
// ONE `BView::Rail` (the shared vertical strip element) with a fixed bottom
// expand/paging cluster.
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

        let shown = BSidebarShown(self.host);
        let open = shown.on(s);
        let select_key = shown.tab_key();
        // Picking a tab opens the region on it (`show::BSidebarOpenRule`);
        // picking the one already showing puts the region away.
        let pick = |label: &'static str| match label == active && open {
            true => BAction::Set(shown.key(), V_FALSE.to_string()),
            false => BAction::Set(select_key.clone(), label.to_string()),
        };
        let mut entries: Vec<BRailEntry> = tabs_spec
            .iter()
            .map(|t| BRailEntry {
                control: BControl::new()
                    .with_icon(t.icon)
                    .with_label(t.label)
                    .with_active(t.label == active)
                    .with_action(pick(t.label)),
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
            panels: open.then(|| BSidebarPanels::new(self.host).views(s)),
        })
    }
}

impl BViewList for BSidebarList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BSidebar {
    pub host: BHostType,
}

impl BSidebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BSidebar {
    fn build(&self, s: &BState) -> BView {
        BSidebarList::new(self.host).rail(s)
    }
}
