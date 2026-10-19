//! Switcher — the editor-type dropdown in a host header (Blender's
//! `Editor Type` selector at the far left of every header).
//!
//! The trigger shows the area's current editor icon; clicking it deploys a
//! popover of the editor types that area may switch to. Selection writes
//! `ctx.area.<space>.<slot>.<default_host>`, which `BAreaItem::active_host`
//! reads on the next frame.

pub mod set;

use crate::v2::area::current;
use crate::v2::host::BHostType;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::state::val::V_EMPTY;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::prim::BPopoverSide;
use self::set::category;
use self::set::category_key;
use self::set::hosts;
pub use set::BHostCategory;

// `BSwitcherList` — two-column category/editor popup content.
// Categories filter the host candidates declared by the current `BAreaSpec`;
// host labels, icons, and actions continue to come from canonical metadata.
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
            .map(|category| {
                BView::MenuItem(
                    BControl::new()
                        .with_icon(if category == selected {
                            "check"
                        } else {
                            category.icon()
                        })
                        .with_label(category.label())
                        .with_action(BAction::Open(category_key(&self.switcher, category))),
                )
            })
            .collect();
        let hosts = self
            .candidates
            .iter()
            .copied()
            .filter(|&host| category(host) == selected)
            .map(|host| {
                // The active editor type shows a check in place of its own
                // icon, matching how the mode dropdown marks its current
                // entry.
                BView::MenuItem(
                    BControl::new()
                        .with_icon(if host == self.active {
                            "check"
                        } else {
                            host.icon()
                        })
                        .with_label(host.label())
                        .with_action(BAction::Set(
                            self.slot.clone(),
                            host.slug().to_string(),
                        )),
                )
            })
            .collect();

        vec![BView::row(vec![
            BView::column(categories),
            BView::column(hosts),
        ])]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BSwitcher {
    pub host: BHostType,
}

impl BSwitcher {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BSwitcher {
    fn build(&self, s: &BState) -> BView {
        // Outside an area (the Topbar, the Status bar) there is nothing to
        // switch, and naming the host there just repeats the window's own
        // chrome — Blender shows its logo at the head of the top bar, which
        // is the app menu's mark, not an editor-type trigger.
        let (Some(scope), Some(area)) = (current::current_scope(), current::current_area()) else {
            return BView::Spacer;
        };

        let open_key = current::switcher_key(scope, &area);
        let slot = current::area_key(scope, &area);
        let open = s.get(&open_key) == Some(V_TRUE);

        // The trigger names the editor it switches: icon + name, at the head of
        // the header, so every host reads the same way whichever area it is in.
        let anchor = BView::MenuAnchor(
            BControl::new()
                .with_icon(self.host.icon())
                .with_label(self.host.label())
                .with_active(open)
                .with_action(BAction::Toggle(open_key.clone())),
        );
        let body = BSwitcherList::new(slot, open_key.clone(), area.hosts, self.host).items(s);
        BView::popover(
            anchor,
            body,
            open,
            BAction::Set(open_key, V_EMPTY.to_string()),
            BPopoverSide::Switcher,
        )
    }
}
