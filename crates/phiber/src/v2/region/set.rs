//! What a `BRegionType` is called.
//!
//! The level's *content* — its names — lives here; `mod.rs` holds the
//! type and its structure. Every level keeps the same split, so there is
//! one place to look whichever level you are in.

use super::BRegionType;

impl BRegionType {
    pub fn label(self) -> &'static str {
        match self {
            BRegionType::Tabs => "Tabs",
            BRegionType::Header => "Header",
            BRegionType::Center => "Center",
            BRegionType::Footer => "Footer",
            BRegionType::ToolHeader => "Tool Header",
            BRegionType::Tools => "Tools",
            BRegionType::UI => "UI",
            BRegionType::ToolProps => "Tool Properties",
            BRegionType::AssetShelf => "Asset Shelf",
            BRegionType::AssetShelfHeader => "Asset Shelf Header",
            BRegionType::Channels => "Channels",
            BRegionType::HUD => "HUD",
            BRegionType::NavBar => "Nav Bar",
            BRegionType::Execute => "Execute",
            BRegionType::Scrubbing => "Scrubbing",
            BRegionType::Preview => "Preview",
            BRegionType::Temporary => "Temporary",
            BRegionType::XR => "XR",
        }
    }
}
