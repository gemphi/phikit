//! `BActivityList` — entries grouped into three regions (Top / Center /
//! Bottom) per each entry's `BActivityGroup`, emitted as ONE `BView::Rail` —
//! the same strip element the Toolbar, the NavBar and the Sidebar use. Active
//! is read from `ctx.activity.tab`.

use super::set::{BActivityGroup, ENTRIES};
use crate::v2::state::{BState, key::{K_ACTIVITY_TAB, K_MODAL}};
use crate::v2::view::{BView, BViewList};
use crate::v2::view::prim::{
    BAction, BControl, BRailEntry, BRailGroup, BRailKind, BRailSide, BRailView,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct BActivityList;

impl BActivityList {
    pub fn new() -> Self {
        Self
    }

    fn group(g: BActivityGroup) -> BRailGroup {
        match g {
            BActivityGroup::Top => BRailGroup::Top,
            BActivityGroup::Center => BRailGroup::Center,
            BActivityGroup::Bottom => BRailGroup::Bottom,
        }
    }

    /// The whole strip as one view.
    pub fn rail(&self, s: &BState) -> BView {
        let active = s.get(&K_ACTIVITY_TAB);
        let entries: Vec<BRailEntry> = ENTRIES
            .iter()
            .map(|e| {
                // Entries with a modal open it directly; entries without a
                // modal fall back to tracking `K_ACTIVITY_TAB` so the visual
                // active state still updates.
                let action = if let Some(modal) = e.modal {
                    BAction::Set(K_MODAL, modal.to_string())
                } else {
                    BAction::Set(K_ACTIVITY_TAB, e.label.to_string())
                };
                BRailEntry {
                    control: BControl::new()
                        .with_icon(e.icon)
                        .with_label(e.label)
                        .with_active(Some(e.label) == active)
                        .with_action(action),
                    group: Self::group(e.group),
                }
            })
            .collect();
        BView::rail(BRailView {
            kind: BRailKind::Activity,
            side: BRailSide::Left,
            expanded: false,
            entries,
        })
    }
}

impl BViewList for BActivityList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}
