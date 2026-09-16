//! `BSwitcherList` — two-column category/editor popup content.
//! Categories filter the host candidates declared by the current `BAreaSpec`;
//! host labels, icons, and actions continue to come from canonical metadata.

use super::{
    BSwitcherItem,
    set::{BHostCategory, category, category_key, hosts},
};
use crate::v2::host::BHostType;
use crate::v2::state::{BPath, BState, val::V_TRUE};
use crate::v2::view::prim::BAction;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Debug)]
pub struct BSwitcherList {
    pub slot: BPath,
    pub switcher: BPath,
    pub candidates: &'static [BHostType],
    pub active: BHostType,
}

impl BSwitcherList {
    pub fn new(
        slot: BPath,
        switcher: BPath,
        candidates: &'static [BHostType],
        active: BHostType,
    ) -> Self {
        let candidates = if candidates.is_empty() {
            hosts()
        } else {
            candidates
        };
        Self {
            slot,
            switcher,
            candidates,
            active,
        }
    }

    fn has_category(&self, candidate: BHostCategory) -> bool {
        self.candidates
            .iter()
            .copied()
            .any(|host| category(host) == candidate)
    }

    fn selected_category(&self, state: &BState) -> BHostCategory {
        BHostCategory::ALL
            .iter()
            .copied()
            .find(|&candidate| {
                self.has_category(candidate)
                    && state.get(&category_key(&self.switcher, candidate)) == Some(V_TRUE)
            })
            .unwrap_or_else(|| category(self.active))
    }
}

impl BViewList for BSwitcherList {
    fn items(&self, state: &BState) -> Vec<BView> {
        let selected = self.selected_category(state);
        let categories = BHostCategory::ALL
            .iter()
            .copied()
            .filter(|&candidate| self.has_category(candidate))
            .map(|candidate| {
                BView::menu_item(
                    if candidate == selected {
                        "check"
                    } else {
                        candidate.icon()
                    },
                    candidate.label(),
                    "",
                    BAction::Open(category_key(&self.switcher, candidate)),
                )
            })
            .collect();
        let hosts = self
            .candidates
            .iter()
            .copied()
            .filter(|&host| category(host) == selected)
            .map(|host| {
                BSwitcherItem::new(host, self.slot.clone(), host == self.active).build(state)
            })
            .collect();

        vec![BView::row(vec![
            BView::column(categories),
            BView::column(hosts),
        ])]
    }
}
