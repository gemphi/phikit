//! Custom — reroutes to the active host's center content.

pub mod item;
pub mod list;
pub mod set;

// Per-host center content.
pub mod action;
pub mod buts;
pub mod clip;
pub mod file;
pub mod image;
pub mod node;
pub mod outliner;
pub mod prefs;
pub mod seq;
pub mod spreadsheet;
pub mod text;
pub mod time;
pub mod view3d;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

pub use item::BCustomItem;
pub use list::BCustomList;

#[derive(Clone, Copy, Debug)]
pub struct BCustom {
    pub host: BHostType,
}

impl BCustom {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BCustom {
    fn build(&self, s: &BState) -> BView {
        BCustomItem::new(self.host).build(s)
    }
}
