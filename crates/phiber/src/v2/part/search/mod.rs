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
        // `SpaceProperties.search_filter` — Blender has searched the
        // Properties editor from its header since 2.90.
        BHostType::Buts => Some(crate::v2::state::ButsKey::Search.path()),
        BHostType::File | BHostType::Asset => {
            Some(FileKey::Search.path())
        }
        // Reality Composer Pro's browser filters its cards by name.
        BHostType::Project => Some(crate::v2::state::ProjectKey::Search.path()),
        // `DopeSheet.filter_text` — the animation editors narrow their
        // channels by name, which is how you find one in a rig. One key for
        // the four of them, as Blender has one `DopeSheet` per space showing
        // the same panel (`part::filter::dope`).
        host if crate::v2::part::filter::filters_channels(host) => {
            Some(crate::v2::state::DopeKey::Search.path())
        }
        // The top bar's search is the palette's: it finds every place the
        // window can take you (`modal::command::find`).
        BHostType::Topbar => Some(crate::v2::state::AppKey::CommandQuery.path()),
        // The rest of the window's own chrome — the status bar, the activity
        // rail — is not an editor and has no search of its own; nor has an
        // empty area, which holds nothing to find.
        BHostType::Statusbar | BHostType::Activity | BHostType::Empty => None,
        // Every other editor searches from the centre of its header, the same
        // field everywhere: `<host>.search`.
        host => Some(BPath::owned(vec![host.slug(), "search"])),
    }
}

impl BViewItem for BSearch {
    fn build(&self, s: &BState) -> BView {
        let Some(key) = search_key(self.host) else {
            return BView::empty();
        };
        let value = s.get(&key).unwrap_or("").to_string();
        if self.host == BHostType::Topbar && crate::v2::part::filter::topbar_wraps(s) {
            // A window too narrow for the bar on one line has no room for a
            // field either: the search is its icon, which opens the palette.
            let open = BAction::Set(crate::v2::state::ModalKey::Open.path(), crate::v2::state::val::V_MODAL_COMMAND.to_string());
            return BView::Button(crate::v2::view::prim::BControl::new().with_icon("search").with_tooltip("Search   F3").with_action(open));
        }
        if self.host == BHostType::Topbar {
            // A click opens the palette (`find::BFindFromBarRule`), which
            // takes the caret; the bar's field never types itself.
            return BView::filter_field("Search spaces, editors, panels and menus   F3", String::new(), false, BAction::Set(key, String::new()));
        }
        let editing = edit::is_editing(s, &key);
        BView::filter_field("Search", value, editing, BAction::Set(key, String::new()))
    }
}
