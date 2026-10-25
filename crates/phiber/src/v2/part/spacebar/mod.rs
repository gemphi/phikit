//! Spacebar — the workspace tabs, in the top bar's right-hand group.
//!
//! They show what fits (`part::display`): every tab labelled on a wide
//! window, the open space labelled and the rest as icons otherwise — or as
//! the view's Labels setting says — and the palette finds any of them.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::space::BSpaceType;
use crate::v2::state::BKey as _;
use crate::v2::state::BState;
use crate::v2::state::CtxKey;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use self::set::spaces;
use crate::v2::part::display::BLabels;

/// The window width from which every space tab carries its name: a third of
/// the window (the top bar's right-hand cell) holds the nine labels. Below
/// it the open space keeps its name and the rest are icons.
const EVERY_LABEL: f32 = 2900.0;

// `BSpacebarList` — reads `set::spaces()` + `ctx.space`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BSpacebarList;

impl BSpacebarList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BSpacebarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let active = s
            .get(CtxKey::Space)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout);
        let width = crate::v2::part::display::window_width(s).unwrap_or(f32::MAX);
        let every = BLabels::shown(s, BHostType::Topbar, width >= EVERY_LABEL);
        let open = BLabels::shown(s, BHostType::Topbar, true);
        spaces()
            .iter()
            .map(|&sp| {
                let labelled = every || (open && sp == active);
                let tab = BControl::new()
                    .with_icon(sp.icon())
                    .with_tooltip(sp.label())
                    .with_active(sp == active)
                    .with_action(BAction::Set(CtxKey::Space.path(), sp.slug().to_string()));
                BView::Tab(if labelled { tab.with_label(sp.label()) } else { tab })
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BSpacebar {
    pub host: BHostType,
}

impl BSpacebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BSpacebar {
    fn build(&self, s: &BState) -> BView {
        if crate::v2::part::filter::topbar_wraps(s) {
            return folded(s);
        }
        BView::row(BSpacebarList::new().items(s))
    }
}

/// A window too narrow for the top bar on one line has no room for nine
/// tabs either: the open space stays, and the rest fold into a menu beside
/// it — the header collapsing into a menu, as a page's bar does on a phone.
fn folded(s: &BState) -> BView {
    use crate::v2::state::val::{V_EMPTY, V_TRUE};
    use crate::v2::state::MenuKey;
    let active = s.get(CtxKey::Space).and_then(BSpaceType::from_slug).unwrap_or(BSpaceType::Layout);
    let key = MenuKey::Top(BHostType::Topbar.slug(), "Spaces").path();
    let open = s.get(&key) == Some(V_TRUE);
    let tab = BControl::new()
        .with_icon(active.icon())
        .with_tooltip(active.label())
        .with_active(true)
        .with_action(BAction::Set(CtxKey::Space.path(), active.slug().to_string()));
    let tab = BView::Tab(if BLabels::shown(s, BHostType::Topbar, true) { tab.with_label(active.label()) } else { tab });
    let anchor = BView::Button(BControl::new().with_icon("chevron_down").with_tooltip("Spaces").with_active(open).with_action(BAction::Toggle(key.clone())));
    let menu = spaces()
        .iter()
        .map(|&sp| {
            BView::MenuItem(
                BControl::new()
                    .with_icon(sp.icon())
                    .with_label(sp.label())
                    .with_active(sp == active)
                    .with_action(BAction::Set(CtxKey::Space.path(), sp.slug().to_string())),
            )
        })
        .collect();
    let popover = BView::popover(anchor, menu, open, BAction::Set(key, V_EMPTY.to_string()), crate::v2::view::prim::BPopoverSide::BelowEnd);
    BView::row(vec![tab, popover])
}
