//! Part view — dispatcher: `BPartType` -> concrete part view. Corresponds to store/parts/.
//!
//! Concrete part views live in `bui/part/<part>/` and keep their existing
//! `.new(host, state).build()` shape; this dispatcher is the single entry point.

use super::super::bview::BView;
use super::super::part::{
    ActivityView, AssetShelfView, ChannelsView, ControlsView, CustomView, ExecuteView, HudView,
    MenubarView, ModebarView, NavBarView, PanelView, ScrubbingView, SidebarView, SpacebarView,
    StatusbarView, SwitcherView, ToolPropsView, ToolbarView,
};
use super::super::state::BState;
use super::BViewBuilder;
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;

/// A single part (Menubar, Modebar, Switcher, Toolbar, …) within a pane.
#[derive(Clone, Copy, Debug)]
pub struct PartView {
    pub part: BPartType,
    pub host: BHostType,
}

impl PartView {
    pub fn new(part: BPartType, host: BHostType) -> Self {
        Self { part, host }
    }
}

impl BViewBuilder for PartView {
    fn build(&self, state: &BState) -> BView {
        match self.part {
            BPartType::Menubar => MenubarView::new(self.host, state).build(),
            BPartType::Modebar => ModebarView::new(self.host, state).build(),
            BPartType::Switcher => SwitcherView::new(self.host, state).build(),
            BPartType::Spacebar => SpacebarView::new(self.host, state).build(),
            BPartType::Toolbar => ToolbarView::new(self.host, state).build(),
            BPartType::Sidebar => SidebarView::new(self.host, state).build(),
            BPartType::Panel => PanelView::new(self.host, state).build(),
            BPartType::Channels => ChannelsView::new(self.host, state).build(),
            BPartType::Statusbar => StatusbarView::new(self.host, state).build(),
            BPartType::ToolProps | BPartType::AdjustLastOp => {
                ToolPropsView::new(self.host, state).build()
            }
            BPartType::AssetShelf => AssetShelfView::new(self.host, state).build(),
            BPartType::NavBar => NavBarView::new(self.host, state).build(),
            BPartType::Execute => ExecuteView::new(self.host, state).build(),
            BPartType::HUD => HudView::new(self.host, state).build(),
            BPartType::Scrubbing => ScrubbingView::new(self.host, state).build(),
            BPartType::Controls => ControlsView::new(self.host, state).build(),
            BPartType::Custom => super::host::center_body(self.host, state),
            _ => BView::empty(),
        }
    }
}

// ── BViewBuilder impls for every concrete part view ─────────────────────────
// Each part view keeps its `.new(host, state).build()` shape; these impls put
// the unified trait behind it, delegating with the passed state. Inherent
// `build` still takes precedence at existing call sites, so behavior is
// unchanged and nothing downstream needs to move yet.

macro_rules! impl_part_builder {
    ($($view:ident),+ $(,)?) => {
        $(
            impl<'a> BViewBuilder for $view<'a> {
                fn build(&self, state: &BState) -> BView {
                    $view::new(self.host, state).build()
                }
            }
        )+
    };
}

impl_part_builder!(
    AssetShelfView,
    ChannelsView,
    ControlsView,
    CustomView,
    ExecuteView,
    HudView,
    MenubarView,
    ModebarView,
    NavBarView,
    PanelView,
    ScrubbingView,
    SidebarView,
    SpacebarView,
    StatusbarView,
    SwitcherView,
    ToolbarView,
    ToolPropsView,
);

// ActivityView carries no host (global activity bar).
impl<'a> BViewBuilder for ActivityView<'a> {
    fn build(&self, state: &BState) -> BView {
        ActivityView::new(state).build()
    }
}
