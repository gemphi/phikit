//! Part view builders — matching store/parts/ hierarchy.
//!
//! Each module corresponds directly to a part in store/parts/.
//! All views are singular struct names implementing .new(host, state).build() -> BView.

pub mod activity;
pub mod assetshelf;
pub mod channels;
pub mod controls;
pub mod custom;
pub mod execute;
pub mod hud;
pub mod menubar;
pub mod modebar;
pub mod navbar;
pub mod panel;
pub mod scrubbing;
pub mod sidebar;
pub mod spacebar;
pub mod statusbar;
pub mod switcher;
pub mod toolbar;
pub mod toolprops;

pub use activity::ActivityView;
pub use assetshelf::AssetShelfView;
pub use channels::ChannelsView;
pub use controls::ControlsView;
pub use custom::CustomView;
pub use execute::ExecuteView;
pub use hud::HudView;
pub use menubar::MenubarView;
pub use modebar::ModebarView;
pub use navbar::NavBarView;
pub use panel::PanelView;
pub use scrubbing::ScrubbingView;
pub use sidebar::SidebarView;
pub use spacebar::SpacebarView;
pub use statusbar::StatusbarView;
pub use switcher::SwitcherView;
pub use toolbar::ToolbarView;
pub use toolprops::ToolPropsView;

use super::{BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;

/// Part view dispatcher — match part type → build BView node.
pub struct PartView<'a> {
    pub part: BPartType,
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> PartView<'a> {
    pub fn new(part: BPartType, host: BHostType, state: &'a BState) -> Self {
        Self { part, host, state }
    }

    pub fn build(self) -> BView {
        match self.part {
            BPartType::Menubar => MenubarView::new(self.host, self.state).build(),
            BPartType::Modebar => ModebarView::new(self.host, self.state).build(),
            BPartType::Switcher => SwitcherView::new(self.host, self.state).build(),
            BPartType::Spacebar => SpacebarView::new(self.host, self.state).build(),
            BPartType::Toolbar => ToolbarView::new(self.host, self.state).build(),
            BPartType::Sidebar => SidebarView::new(self.host, self.state).build(),
            BPartType::Panel => PanelView::new(self.host, self.state).build(),
            BPartType::Channels => ChannelsView::new(self.host, self.state).build(),
            BPartType::Statusbar => StatusbarView::new(self.host, self.state).build(),
            BPartType::ToolProps | BPartType::AdjustLastOp => {
                ToolPropsView::new(self.host, self.state).build()
            }
            BPartType::AssetShelf => AssetShelfView::new(self.host, self.state).build(),
            BPartType::NavBar => NavBarView::new(self.host, self.state).build(),
            BPartType::Execute => ExecuteView::new(self.host, self.state).build(),
            BPartType::HUD => HudView::new(self.host, self.state).build(),
            BPartType::Scrubbing => ScrubbingView::new(self.host, self.state).build(),
            BPartType::Controls => ControlsView::new(self.host, self.state).build(),
            BPartType::Custom => CustomView::new(self.host, self.state).build(),
            _ => BView::empty(),
        }
    }
}
