//! Execute bar — footer action buttons (Cancel / Open / Save …).
//! Blender ref: `RGN_TYPE_EXECUTE`.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BControl;
use self::set::actions_for;
pub use set::BExecuteAction;

// `BExecuteList` — action buttons for the active host.
#[derive(Clone, Copy, Debug)]
pub struct BExecuteList {
    pub host: BHostType,
}

impl BExecuteList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BExecuteList {
    fn items(&self, _s: &BState) -> Vec<BView> {
        actions_for(self.host)
            .iter()
            // The label was being passed as the icon name, so these buttons
            // painted a blank glyph and showed no text at all.
            .map(|a| {
                BView::Button(
                    BControl::new()
                        .with_icon(a.icon)
                        .with_label(a.label)
                        .with_active(a.primary),
                )
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BExecute {
    pub host: BHostType,
}

impl BExecute {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BExecute {
    fn build(&self, s: &BState) -> BView {
        // The file browser's bar is the real one: the file name field and the
        // Cancel / <purpose> pair, up only while there is a purpose.
        if matches!(self.host, BHostType::File | BHostType::Asset) {
            return crate::v2::part::file::execute::view(s);
        }
        BView::row(BExecuteList::new(self.host).items(s))
    }
}
