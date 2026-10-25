//! Custom — reroutes to the active host's center content.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use self::set::hosts_with_center;

// `BCustomItem` — per-host center content dispatcher.
// Hosts that share a Blender editor space share a submodule here.
#[derive(Clone, Copy, Debug)]
pub struct BCustomItem {
    pub host: BHostType,
}

impl BCustomItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BCustomItem {
    fn build(&self, s: &BState) -> BView {
        match self.host {
            BHostType::View3D => super::view3d::BView3dItem::new().build(s),
            BHostType::Outliner => super::outliner::BOutlinerItem::new().build(s),
            BHostType::Buts => super::buts::BButsItem::new().build(s),
            BHostType::Time => super::time::BTimeItem::new().build(s),
            BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => {
                super::node::BNodeItem::new(self.host).build(s)
            }
            BHostType::Image | BHostType::Uv => super::image::BImageItem::new(self.host).build(s),
            BHostType::Seq => super::seq::BSeqItem::new().build(s),
            BHostType::Text | BHostType::Info | BHostType::Console => {
                super::text::BTextItem::new(self.host).build(s)
            }
            BHostType::File | BHostType::Asset => super::file::BFileItem::new(self.host).build(s),
            BHostType::Action | BHostType::Nla | BHostType::Ipo | BHostType::Drivers => {
                super::action::BActionItem::new(self.host).build(s)
            }
            BHostType::Spreadsheet => super::spreadsheet::BSpreadsheetItem::new().build(s),
            BHostType::Clip => super::clip::BClipItem::new().build(s),
            BHostType::Prefs => super::pref::BPrefsItem::new().build(s),
            BHostType::Play => super::play::BPlayItem.build(s),
            BHostType::Project => super::project::BProjectItem.build(s),
            // The rail itself is the `Activity` part in the Left pane; this
            // is the secondary strip beside it, carrying the options of
            // whichever primary icon is open.
            BHostType::Activity => {
                crate::v2::part::activity::BActivityList::new().group_strip(s)
            }
            BHostType::Topbar | BHostType::Statusbar | BHostType::Logic | BHostType::Empty => {
                BView::empty()
            }
        }
    }
}

// `BCustomList` — one `BCustomItem` per host that has center content.

#[derive(Clone, Copy, Debug, Default)]
pub struct BCustomList;

impl BCustomList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BCustomList {
    fn items(&self, s: &BState) -> Vec<BView> {
        hosts_with_center()
            .iter()
            .map(|&h| BCustomItem::new(h).build(s))
            .collect()
    }
}

// Per-host center content.

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
