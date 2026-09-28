//! NavBar — vertical column of context tabs (Properties).

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::part::strip;
use crate::v2::state::BPath;
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
use self::set::tabs_for;
pub use set::BNavTabSpec;

// `BNavBarList` — Properties context tabs. Reads `tabs_for(host)` and
// per-host state at `ctx.<host>.navbar.tab`, then emits ONE `BView::Rail` —
// the same strip element the Toolbar, the Sidebar and the Activity bar
// use. The expand state lives at `<host>.navbar.expanded`, shared with the
// other strips via `part::strip`.
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

        let select_key = BPath::owned(vec![self.host.slug(), "navbar", "tab"]);
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
            panels: None,
        })
    }
}

impl BViewList for BNavBarList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BNavBar {
    pub host: BHostType,
}

impl BNavBar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BNavBar {
    fn build(&self, s: &BState) -> BView {
        BNavBarList::new(self.host).rail(s)
    }
}
