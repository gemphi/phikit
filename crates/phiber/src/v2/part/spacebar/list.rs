//! `BSpacebarList` — reads `set::spaces()` + `ctx.space`.

use super::{BSpacebarItem, set::spaces};
use crate::v2::space::BSpaceType;
use crate::v2::state::{BState, key::K_SPACE};
use crate::v2::view::{BView, BViewItem, BViewList};

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
            .get(&K_SPACE)
            .and_then(BSpaceType::from_slug)
            .unwrap_or(BSpaceType::Layout);
        spaces()
            .iter()
            .map(|&sp| BSpacebarItem::new(sp, sp == active).build(s))
            .collect()
    }
}
