//! `BModebarList` — reads `modes_for(host)` + active mode from `ctx.mode`.

use super::{
    BModebarItem,
    set::{default_mode, mode_key, modes_for},
};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BModebarList {
    pub host: BHostType,
}

impl BModebarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BModebarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let key = mode_key(self.host);
        let active = s.get(&key).unwrap_or_else(|| default_mode(self.host));
        modes_for(self.host)
            .iter()
            .map(|m| BModebarItem::new(self.host, *m, m.label == active).build(s))
            .collect()
    }
}
