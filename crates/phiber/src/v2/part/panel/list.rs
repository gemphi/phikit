//! `BPanelList` — reads `sections_for(host, tab)` + `host.<h>.panel.<title>.expanded`.

use super::{BPanelItem, set::sections_for};
use crate::v2::host::BHostType;
use crate::v2::part::navbar::set::tabs_for;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BPanelList {
    pub host: BHostType,
}

impl BPanelList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

use crate::v2::state::val::V_FALSE;

impl BViewList for BPanelList {
    fn items(&self, s: &BState) -> Vec<BView> {
        // Active context tab from state; fallback = navbar's first declared
        // tab for this host (no hardcoded name).
        let fallback = tabs_for(self.host).first().map(|t| t.label).unwrap_or("");
        let selected = s.host(self.host).get("navbar.tab").unwrap_or(fallback);
        // Resolve back to the static table entry. Besides avoiding unbounded
        // path-segment allocation, this guarantees the key uses the same tab
        // name that selected the section table.
        let tab = tabs_for(self.host)
            .iter()
            .find(|candidate| candidate.label == selected)
            .map(|candidate| candidate.label)
            .unwrap_or(fallback);
        sections_for(self.host, tab)
            .iter()
            .map(|section| {
                let key = BPanelItem::expand_key(self.host, tab, section.title);
                let expanded = !s.has(&key, V_FALSE);
                BPanelItem::new(self.host, tab, *section, expanded).build(s)
            })
            .collect()
    }
}
