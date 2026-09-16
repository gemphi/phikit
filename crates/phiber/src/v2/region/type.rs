//! `BRegionType` — DNA region kind. Mirrors `eRegion_Type` in Blender 5.2.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRegionType {
    Header,
    Center, // WINDOW: the primary work canvas
    Footer,
    ToolHeader,
    Tools, // T panel
    UI,    // N panel
    ToolProps,
    AssetShelf,
    AssetShelfHeader,
    Channels,
    HUD,
    NavBar,
    Execute,
    Scrubbing,
    Preview,
    Temporary,
    XR,
}

impl BRegionType {
    pub fn label(self) -> &'static str {
        match self {
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
