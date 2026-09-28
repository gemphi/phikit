//! Spacebar — workspace tabs at the top center.

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
        spaces()
            .iter()
            .map(|&sp| {
                // A labeled tab, not a flat button: the active workspace gets
                // the top-rounded gradient cap and the accent underline, and
                // hovering an inactive one previews the same cap.
                BView::Tab(
                    BControl::new()
                        .with_icon(sp.icon())
                        .with_label(sp.label())
                        .with_active(sp == active)
                        .with_action(BAction::Set(CtxKey::Space.path(), sp.slug().to_string())),
                )
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
        BView::row(BSpacebarList::new().items(s))
    }
}
