//! Header search — the field in the middle of a host header.
//!
//! Every host presents itself the same way: what it is on the left, what you
//! are looking for in the middle, how it is filtered on the right. A host that
//! has nothing to search leaves the middle empty.
//!
//! Blender ref: the outliner's `filter_text` and the file browser's
//! `filter_search`, both drawn in their header.

use crate::v2::state::BKey as _;
use crate::v2::state::OutlinerKey;
use crate::v2::state::FileKey;
use crate::v2::host::BHostType;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::state::edit;
use crate::v2::view::prim::BAction;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;

#[derive(Clone, Copy, Debug)]
pub struct BSearch {
    pub host: BHostType,
}

impl BSearch {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

/// The key a host filters by, when it has one.
pub fn search_key(host: BHostType) -> Option<BPath> {
    match host {
        BHostType::Outliner => Some(OutlinerKey::Search.path()),
        BHostType::File | BHostType::Asset => {
            Some(FileKey::Search.path())
        }
        _ => None,
    }
}

impl BViewItem for BSearch {
    fn build(&self, s: &BState) -> BView {
        let Some(key) = search_key(self.host) else {
            return BView::empty();
        };
        let value = s.get(&key).unwrap_or("").to_string();
        let editing = edit::is_editing(s, &key);
        BView::text_field("Search", value, editing, BAction::Set(key, String::new()))
    }
}
