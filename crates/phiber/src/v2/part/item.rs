//! `BPartItem` — one part; dispatcher by `BPartType`.
//! Every per-part composer implements `BViewItem`.

use super::BPartType;
use super::{
    activity::BActivity, assetshelf::BAssetShelf, channels::BChannels, custom::BCustom,
    execute::BExecute, hud::BHud, menubar::BMenubar, modebar::BModebar, navbar::BNavBar,
    panel::BPanel, sidebar::BSidebar, spacebar::BSpacebar, statusbar::BStatusbar,
    switcher::BSwitcher, toolbar::BToolbar, toolprops::BToolProps, zonebar::BZonebar,
};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BPartItem {
    pub host: BHostType,
    pub part: BPartType,
}

impl BPartItem {
    pub fn new(host: BHostType, part: BPartType) -> Self {
        Self { host, part }
    }

    fn body(&self, s: &BState) -> BView {
        let h = self.host;
        match self.part {
            BPartType::Menubar => BMenubar::new(h).build(s),
            BPartType::Modebar => BModebar::new(h).build(s),
            BPartType::Switcher => BSwitcher::new(h).build(s),
            BPartType::Spacebar => BSpacebar::new(h).build(s),
            BPartType::Toolbar => BToolbar::new(h).build(s),
            BPartType::Sidebar => BSidebar::new(h).build(s),
            BPartType::Panel => BPanel::new(h).build(s),
            BPartType::Channels => BChannels::new(h).build(s),
            BPartType::Statusbar => BStatusbar::new(h).build(s),
            BPartType::Zonebar => BZonebar::new(h).build(s),
            BPartType::NavBar => BNavBar::new(h).build(s),
            BPartType::HUD => BHud::new(h).build(s),
            BPartType::Execute => BExecute::new(h).build(s),
            BPartType::AssetShelf => BAssetShelf::new(h).build(s),
            BPartType::ToolProps | BPartType::AdjustLastOp => BToolProps::new(h).build(s),
            BPartType::Activity => BActivity::new(h).build(s),
            BPartType::Custom => BCustom::new(h).build(s),
            BPartType::Scrubbing => BView::empty(),
        }
    }
}

impl BViewItem for BPartItem {
    fn build(&self, state: &BState) -> BView {
        BView::Part {
            kind: self.part,
            body: Box::new(self.body(state)),
        }
    }
}
