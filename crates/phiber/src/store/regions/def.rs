//! Region constants - labels, alignments, and the complete 17 Blender 5.2 region types.
//!
//! Directly matches `eRegion_Type` and `eRegion_Alignment` in `source/blender/makesdna/DNA_screen_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum BRegionType {
    // ── Primary 3 Base Regions (full backward compatibility) ──
    Header = 1,
    Center = 0, // In Blender DNA this is RGN_TYPE_WINDOW (0)
    Footer = 11,

    // ── Full Blender 5.2 DNA Region Types (eRegion_Type) ──
    Window = 100, // Alias for Center/Work region
    Channels = 2,
    Temporary = 3,
    UI = 4,                // Sidebar / N-Panel
    Tools = 5,             // Toolbar / T-Panel
    ToolProps = 6,         // Tool Properties / Adjust Last Operation
    Preview = 7,           // Preview swatch/renderer
    HUD = 8,               // Floating Heads-up Display
    NavBar = 9,            // Navigation bar
    Execute = 10,          // Execution buttons (OK/Cancel)
    ToolHeader = 12,       // Mode-specific tool header
    XR = 13,               // Virtual Reality / XR Surface
    AssetShelf = 14,       // Asset Shelf
    AssetShelfHeader = 15, // Asset Shelf Header Tabs
    Scrubbing = 16,        // Timeline scrubbing region
}

/// Region alignment enum matching `eRegion_Alignment` in `DNA_screen_types.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum BRegionAlignment {
    None = 0,
    Top = 1,
    Bottom = 2,
    Left = 3,
    Right = 4,
    HSplit = 5,
    VSplit = 6,
    Float = 7,
    QSplit = 8,
}

pub const HEADER_LABEL: &str = "Header";
pub const CENTER_LABEL: &str = "Center";
pub const FOOTER_LABEL: &str = "Footer";
pub const CHANNELS_LABEL: &str = "Channels";
pub const TEMPORARY_LABEL: &str = "Temporary";
pub const UI_LABEL: &str = "Sidebar";
pub const TOOLS_LABEL: &str = "Toolbar";
pub const TOOL_PROPS_LABEL: &str = "Tool Properties";
pub const PREVIEW_LABEL: &str = "Preview";
pub const HUD_LABEL: &str = "HUD";
pub const NAV_BAR_LABEL: &str = "Navigation Bar";
pub const EXECUTE_LABEL: &str = "Execute";
pub const TOOL_HEADER_LABEL: &str = "Tool Header";
pub const XR_LABEL: &str = "XR Context";
pub const ASSET_SHELF_LABEL: &str = "Asset Shelf";
pub const ASSET_SHELF_HEADER_LABEL: &str = "Asset Shelf Header";
pub const SCRUBBING_LABEL: &str = "Scrubbing";

impl BRegionType {
    /// DNA region type code matching Blender 5.2 `eRegion_Type`.
    pub fn dna_type_id(self) -> i16 {
        match self {
            BRegionType::Header => 1,
            BRegionType::Center | BRegionType::Window => 0,
            BRegionType::Footer => 11,
            BRegionType::Channels => 2,
            BRegionType::Temporary => 3,
            BRegionType::UI => 4,
            BRegionType::Tools => 5,
            BRegionType::ToolProps => 6,
            BRegionType::Preview => 7,
            BRegionType::HUD => 8,
            BRegionType::NavBar => 9,
            BRegionType::Execute => 10,
            BRegionType::ToolHeader => 12,
            BRegionType::XR => 13,
            BRegionType::AssetShelf => 14,
            BRegionType::AssetShelfHeader => 15,
            BRegionType::Scrubbing => 16,
        }
    }

    /// Default alignment for this region type.
    pub fn default_alignment(self) -> BRegionAlignment {
        match self {
            BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
                BRegionAlignment::Top
            }
            BRegionType::Footer => BRegionAlignment::Bottom,
            BRegionType::Tools => BRegionAlignment::Left,
            BRegionType::UI | BRegionType::ToolProps => BRegionAlignment::Right,
            BRegionType::HUD => BRegionAlignment::Float,
            _ => BRegionAlignment::None,
        }
    }

    /// CSS-like role key for `styled_panel` in the bui layer.
    pub fn role(self) -> &'static str {
        match self {
            BRegionType::Header => "header",
            BRegionType::Center | BRegionType::Window => "work",
            BRegionType::Footer => "statusbar",
            BRegionType::UI => "sidebar",
            BRegionType::Tools => "toolbar",
            BRegionType::ToolProps => "toolprops",
            BRegionType::AssetShelf => "assetshelf",
            BRegionType::Channels => "channels",
            _ => "work",
        }
    }

    /// Typed panel role for the BUI layer.
    pub fn panel_role(self) -> crate::store::BPanelRole {
        crate::store::BPanelRole::from_region(self)
    }
}
