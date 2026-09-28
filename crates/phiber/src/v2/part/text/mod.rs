//! Text editor / Info log / Console canvas — line-based text hosts.
//! Blender ref: `space_text`, `space_info`, `space_console`.

pub mod info;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BTextItem` — line-buffer canvas. Shared by Text/Info/Console.
#[derive(Clone, Copy, Debug)]
pub struct BTextItem {
    pub host: BHostType,
}

impl BTextItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BTextItem {
    fn build(&self, s: &BState) -> BView {
        match self.host {
            // Info editor: the report log (file reads, errors).
            BHostType::Info => self::info::view(s),
            _ => BView::canvas(self.host),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BTextList {
    pub host: BHostType,
}

impl BTextList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BTextList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BTextItem::new(self.host).build(s)]
    }
}
