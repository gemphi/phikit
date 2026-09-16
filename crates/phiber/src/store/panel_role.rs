//! BPanelRole — unified typed enum for all panel roles in the BUI layer.
//! Replaces string-based role matching with a single typed enum that
//! covers every level of the hierarchy: space, area, host, region, and part.

use crate::store::areas::BAreaType;
use crate::store::parts::BPartType;
use crate::store::regions::BRegionType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPanelRole {
    // Space-level (root container)
    App,

    // Area-level (screen zones)
    TopArea,
    BottomArea,
    LeftArea,
    RightArea,
    CenterArea,
    RightColumn,

    // Host-level (editor container)
    Host,

    // Region-level
    Header,
    Work,
    Footer,
    Sidebar,
    Toolbar,
    ToolProps,
    AssetShelf,
    Channels,
    HUD,
    NavBar,
    Execute,
    Scrubbing,

    // Pane-level (alignments: left -> start, center -> center, right -> end)
    PaneLeft,
    PaneCenter,
    PaneRight,

    // Part-level
    Menubar,
    Modebar,
    Switcher,
    Spacebar,
    Panel,
    Statusbar,
    Section,
    Viewport,
    /// Modal overlay — centered, rounded, with backdrop.
    Modal,
}

impl BPanelRole {
    /// Convert from a BAreaType to its panel role.
    pub fn from_area(area: BAreaType) -> Self {
        match area {
            BAreaType::Top => BPanelRole::TopArea,
            BAreaType::Bottom => BPanelRole::BottomArea,
            BAreaType::Left => BPanelRole::LeftArea,
            BAreaType::Right => BPanelRole::RightArea,
            BAreaType::Center => BPanelRole::CenterArea,
        }
    }

    /// Convert from a BRegionType to its panel role.
    pub fn from_region(region: BRegionType) -> Self {
        match region {
            BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
                BPanelRole::Header
            }
            BRegionType::Center
            | BRegionType::Window
            | BRegionType::Temporary
            | BRegionType::Preview
            | BRegionType::XR => BPanelRole::Work,
            BRegionType::Footer => BPanelRole::Footer,
            BRegionType::UI => BPanelRole::Sidebar,
            BRegionType::Tools => BPanelRole::Toolbar,
            BRegionType::ToolProps => BPanelRole::ToolProps,
            BRegionType::AssetShelf => BPanelRole::AssetShelf,
            BRegionType::Channels => BPanelRole::Channels,
            BRegionType::HUD => BPanelRole::HUD,
            BRegionType::NavBar => BPanelRole::NavBar,
            BRegionType::Execute => BPanelRole::Execute,
            BRegionType::Scrubbing => BPanelRole::Scrubbing,
        }
    }

    /// Convert from a BPartType to its panel role.
    pub fn from_part(part: BPartType) -> Self {
        match part {
            BPartType::Menubar => BPanelRole::Menubar,
            BPartType::Modebar => BPanelRole::Modebar,
            BPartType::Switcher => BPanelRole::Switcher,
            BPartType::Spacebar => BPanelRole::Spacebar,
            BPartType::Toolbar => BPanelRole::Toolbar,
            BPartType::Sidebar => BPanelRole::Sidebar,
            BPartType::Panel => BPanelRole::Panel,
            BPartType::Channels => BPanelRole::Channels,
            BPartType::Statusbar => BPanelRole::Section,
            BPartType::ToolProps | BPartType::AdjustLastOp => BPanelRole::ToolProps,
            BPartType::AssetShelf => BPanelRole::AssetShelf,
            BPartType::NavBar => BPanelRole::NavBar,
            BPartType::Execute => BPanelRole::Execute,
            BPartType::HUD => BPanelRole::HUD,
            BPartType::Scrubbing => BPanelRole::Scrubbing,
            BPartType::Controls => BPanelRole::Modebar,
            BPartType::Custom => BPanelRole::Viewport,
            _ => BPanelRole::Section,
        }
    }
}
