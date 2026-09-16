//! Part type constants - all part type labels and component names.
//!
//! Matches Blender 5.2 DNA panel types and region components.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPartType {
    // Content parts (map to BArea* components)
    Menubar,
    Modebar,
    Switcher,
    Toolbar,
    Sidebar,
    Panel,
    Channels,
    Statusbar,
    Spacebar,
    ToolProps,
    Preview,
    AssetShelf,
    AdjustLastOp,
    NavBar,
    Execute,
    HUD,
    Scrubbing,
    Custom,
    Controls,
}

pub const MENUBAR_LABEL: &str = "Menu Bar";
pub const MODEBAR_LABEL: &str = "Mode Bar";
pub const SWITCHER_LABEL: &str = "Switcher";
pub const TOOLBAR_LABEL: &str = "Toolbar";
pub const SIDEBAR_LABEL: &str = "Sidebar";
pub const PANEL_LABEL: &str = "Panel";
pub const CHANNELS_LABEL: &str = "Channels";
pub const STATUSBAR_LABEL: &str = "Status Bar";
pub const SPACEBAR_LABEL: &str = "Space Bar";
pub const TOOL_PROPS_LABEL: &str = "Tool Properties";
pub const PREVIEW_LABEL: &str = "Preview";
pub const ASSET_SHELF_LABEL: &str = "Asset Shelf";
pub const ADJUST_LAST_OP_LABEL: &str = "Adjust Last Operation";
pub const NAV_BAR_LABEL: &str = "Navigation Bar";
pub const EXECUTE_LABEL: &str = "Execute Bar";
pub const HUD_LABEL: &str = "HUD Overlay";
pub const SCRUBBING_LABEL: &str = "Timeline Scrubbing";
pub const CUSTOM_LABEL: &str = "Custom";
pub const CONTROLS_LABEL: &str = "Header Controls";

pub const MENUBAR_COMPONENT: &str = "BAreaMenubar";
pub const MODEBAR_COMPONENT: &str = "BAreaModebar";
pub const SWITCHER_COMPONENT: &str = "BSwitcher";
pub const TOOLBAR_COMPONENT: &str = "BAreaToolbar";
pub const SIDEBAR_COMPONENT: &str = "BAreaSidebar";
pub const PANEL_COMPONENT: &str = "BAreaPanel";
pub const CHANNELS_COMPONENT: &str = "BAreaChannels";
pub const STATUSBAR_COMPONENT: &str = "BAreaStatusbar";
pub const SPACEBAR_COMPONENT: &str = "BAreaSpacebar";
pub const TOOL_PROPS_COMPONENT: &str = "BAreaToolProps";
pub const PREVIEW_COMPONENT: &str = "BAreaPreview";
pub const ASSET_SHELF_COMPONENT: &str = "BAreaAssetShelf";
pub const ADJUST_LAST_OP_COMPONENT: &str = "BAreaAdjustLastOp";
pub const NAV_BAR_COMPONENT: &str = "BAreaNavBar";
pub const EXECUTE_COMPONENT: &str = "BAreaExecute";
pub const HUD_COMPONENT: &str = "BAreaHUD";
pub const SCRUBBING_COMPONENT: &str = "BAreaScrubbing";
pub const CUSTOM_COMPONENT: &str = "Custom BView";
pub const CONTROLS_COMPONENT: &str = "BAreaControls";
