//! Region set - static spec arrays and labels.

use super::BRegionSpec;
use super::def::*;

pub static ALL: &[BRegionType] = &[
    BRegionType::Header,
    BRegionType::Center,
    BRegionType::Footer,
    BRegionType::Channels,
    BRegionType::Temporary,
    BRegionType::UI,
    BRegionType::Tools,
    BRegionType::ToolProps,
    BRegionType::Preview,
    BRegionType::HUD,
    BRegionType::NavBar,
    BRegionType::Execute,
    BRegionType::ToolHeader,
    BRegionType::XR,
    BRegionType::AssetShelf,
    BRegionType::AssetShelfHeader,
    BRegionType::Scrubbing,
];

pub static ALL_SPECS: &[BRegionSpec] = &[
    BRegionSpec::new(BRegionType::Header),
    BRegionSpec::new(BRegionType::Center),
    BRegionSpec::new(BRegionType::Footer),
    BRegionSpec::new(BRegionType::Channels),
    BRegionSpec::new(BRegionType::Temporary),
    BRegionSpec::new(BRegionType::UI),
    BRegionSpec::new(BRegionType::Tools),
    BRegionSpec::new(BRegionType::ToolProps),
    BRegionSpec::new(BRegionType::Preview),
    BRegionSpec::new(BRegionType::HUD),
    BRegionSpec::new(BRegionType::NavBar),
    BRegionSpec::new(BRegionType::Execute),
    BRegionSpec::new(BRegionType::ToolHeader),
    BRegionSpec::new(BRegionType::XR),
    BRegionSpec::new(BRegionType::AssetShelf),
    BRegionSpec::new(BRegionType::AssetShelfHeader),
    BRegionSpec::new(BRegionType::Scrubbing),
];

pub static LABELS: &[&str] = &[
    HEADER_LABEL,
    CENTER_LABEL,
    FOOTER_LABEL,
    CHANNELS_LABEL,
    TEMPORARY_LABEL,
    UI_LABEL,
    TOOLS_LABEL,
    TOOL_PROPS_LABEL,
    PREVIEW_LABEL,
    HUD_LABEL,
    NAV_BAR_LABEL,
    EXECUTE_LABEL,
    TOOL_HEADER_LABEL,
    XR_LABEL,
    ASSET_SHELF_LABEL,
    ASSET_SHELF_HEADER_LABEL,
    SCRUBBING_LABEL,
];
