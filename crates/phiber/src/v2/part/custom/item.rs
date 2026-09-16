//! `BCustomItem` — per-host center content dispatcher.
//! Hosts that share a Blender editor space share a submodule here.

use super::{
    action, buts, clip, file, image, node, outliner, prefs, seq, spreadsheet, text, time, view3d,
};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

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
            BHostType::View3D => view3d::BView3dItem::new().build(s),
            BHostType::Outliner => outliner::BOutlinerItem::new().build(s),
            BHostType::Buts => buts::BButsItem::new().build(s),
            BHostType::Time => time::BTimeItem::new().build(s),
            BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => {
                node::BNodeItem::new(self.host).build(s)
            }
            BHostType::Image | BHostType::Uv => image::BImageItem::new(self.host).build(s),
            BHostType::Seq => seq::BSeqItem::new().build(s),
            BHostType::Text | BHostType::Info | BHostType::Console => {
                text::BTextItem::new(self.host).build(s)
            }
            BHostType::File | BHostType::Asset => file::BFileItem::new(self.host).build(s),
            BHostType::Action | BHostType::Nla | BHostType::Ipo | BHostType::Drivers => {
                action::BActionItem::new(self.host).build(s)
            }
            BHostType::Spreadsheet => spreadsheet::BSpreadsheetItem::new().build(s),
            BHostType::Clip => clip::BClipItem::new().build(s),
            BHostType::Prefs => prefs::BPrefsItem::new().build(s),
            // Activity host's center content is the global-nav icon strip,
            // built from the existing `part::activity` entries as one Rail.
            BHostType::Activity => {
                crate::v2::part::activity::BActivityList::new().rail(s)
            }
            BHostType::Topbar | BHostType::Statusbar | BHostType::Logic | BHostType::Empty => {
                BView::empty()
            }
        }
    }
}
