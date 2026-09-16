//! `BPartType` — leaf variants.
//!
//! `Activity` is Phiber-native (not in Blender): a full-height leftmost bar
//! with global navigation (home / chat / spaces / stream / settings / profile).
//! Everything else mirrors a Blender part.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPartType {
    Menubar,
    Modebar,
    Switcher,
    Spacebar,
    Toolbar,
    Sidebar,
    Panel,
    Channels,
    Statusbar,
    Zonebar,
    NavBar,
    HUD,
    Execute,
    AssetShelf,
    ToolProps,
    AdjustLastOp,
    Scrubbing,
    Activity,
    Custom,
}

impl BPartType {
    pub fn label(self) -> &'static str {
        match self {
            BPartType::Menubar => "Menu Bar",
            BPartType::Modebar => "Mode Bar",
            BPartType::Switcher => "Switcher",
            BPartType::Spacebar => "Space Bar",
            BPartType::Toolbar => "Toolbar",
            BPartType::Sidebar => "Sidebar",
            BPartType::Panel => "Panel",
            BPartType::Channels => "Channels",
            BPartType::Statusbar => "Status Bar",
            BPartType::Zonebar => "Zone Visibility",
            BPartType::NavBar => "Nav Bar",
            BPartType::HUD => "HUD",
            BPartType::Execute => "Execute",
            BPartType::AssetShelf => "Asset Shelf",
            BPartType::ToolProps => "Tool Properties",
            BPartType::AdjustLastOp => "Adjust Last Op",
            BPartType::Scrubbing => "Scrubbing",
            BPartType::Activity => "Activity",
            BPartType::Custom => "Custom",
        }
    }
}
