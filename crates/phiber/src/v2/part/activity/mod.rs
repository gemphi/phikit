//! Activity Bar — Phiber-native leftmost vertical navigation.

pub mod choose;
pub mod option;
pub mod profile;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::ActivityKey;
use crate::v2::state::BKey as _;
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
use self::set::ENTRIES;
pub use set::BActivityEntry;
pub use set::BActivityGroup;
pub use set::BActivityOptions;

// `BActivityList` — entries grouped into three regions (Top / Center /
// Bottom) per each entry's `BActivityGroup`, emitted as ONE `BView::Rail` —
// the same strip element the Toolbar, the NavBar and the Sidebar use. Active
// is read from `ctx.activity.tab`.
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
        let active = s.get(ActivityKey::Tab);
        let entries: Vec<BRailEntry> = ENTRIES
            .iter()
            .map(|e| {
                // Selecting an entry records it; the reducer opens the
                // entry's declared modal (`set.rs`).
                let action = BAction::Set(ActivityKey::Tab.path(), e.label.to_string());
                // A grouping reads as active while its strip is open, so the
                // rail shows which column beside it belongs to which icon.
                let lit = Some(e.label) == active
                    || option::open_group(s).is_some_and(|open| open.label == e.label);
                BRailEntry {
                    control: BControl::new()
                        .with_icon(e.icon)
                        .with_label(e.label)
                        .with_active(lit)
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
            panels: None,
        })
    }

    /// The panel beside the rail: the open primary's options, or nothing.
    ///
    /// As Slack's sidebar stands beside its rail: a bordered card on the
    /// card shade, headed by the primary's name, its rows menu rows — the
    /// icon slot reserved on every one, so the labels line up down the
    /// panel. The Profile primary opens the profile card instead
    /// (`profile.rs`).
    pub fn group_strip(&self, s: &BState) -> BView {
        let Some(entry) = option::open_group(s) else {
            return BView::empty();
        };
        // Profile's rows are built differently (identity, then folding sections)
        // but its CONTAINER is the same card every other column gets — it used to
        // return a bare column, which is why it was the one secondary column with
        // no border.
        let rows: Vec<BView> = if entry.options == BActivityOptions::Profile {
            self::profile::body(s)
        } else {
            option::options_for(entry, s).into_iter().map(BView::MenuItem).collect()
        };
        if rows.is_empty() {
            return BView::empty();
        }
        BView::Card(Box::new(crate::v2::view::prim::BCard::panel(entry.label).icon(entry.icon).body(rows)))
    }
}

impl BViewList for BActivityList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        vec![]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BActivity {
    pub host: BHostType,
}

impl BActivity {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BActivity {
    fn build(&self, s: &BState) -> BView {
        BActivityList::new().rail(s)
    }
}
